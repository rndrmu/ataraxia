use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::task::JoinHandle;

use ataraxia_voice::{AudioFrame, VoiceConnection, SAMPLE_RATE, NUM_CHANNELS, SAMPLES_PER_CHANNEL};

use crate::config::Config;
use crate::protocol::{OutgoingEvent, TrackEndReason, TrackInfo};
use crate::resolver::{self, ResolvedTrack};

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerStatus {
    Idle,
    Connecting,
    Playing,
    Paused,
    Stopped,
}

pub struct Player {
    pub guild_id: String,
    pub channel_id: String,
    pub status: PlayerStatus,
    pub position_ms: Arc<AtomicU64>,
    pub volume: u8,
    pub current_track: Option<TrackInfo>,
    voice: Option<Arc<VoiceConnection>>,
    pipeline: Option<JoinHandle<()>>,
    event_tx: tokio::sync::mpsc::UnboundedSender<OutgoingEvent>,
    config: Arc<Config>,
}

impl Player {
    pub fn new(
        guild_id: String,
        channel_id: String,
        event_tx: tokio::sync::mpsc::UnboundedSender<OutgoingEvent>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            guild_id,
            channel_id,
            status: PlayerStatus::Idle,
            position_ms: Arc::new(AtomicU64::new(0)),
            volume: config.audio.default_volume,
            current_track: None,
            voice: None,
            pipeline: None,
            event_tx,
            config,
        }
    }

    pub async fn connect_livekit(&mut self, url: &str, token: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.status = PlayerStatus::Connecting;
        let conn = VoiceConnection::connect(url, token).await?;
        self.voice = Some(Arc::new(conn));
        self.status = PlayerStatus::Idle;
        Ok(())
    }

    pub async fn play(&mut self, identifier: &str, start_ms: u64) {
        if let Some(h) = self.pipeline.take() {
            h.abort();
        }

        let resolved = match resolver::resolve(identifier, &self.config.audio.ytdlp_path).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("resolver error for {}: {}", identifier, e);
                let _ = self.event_tx.send(OutgoingEvent::TrackEnd {
                    guild_id: self.guild_id.clone(),
                    track: placeholder_track(identifier),
                    reason: TrackEndReason::LoadFailed,
                });
                return;
            }
        };

        self.current_track = Some(resolved.info.clone());
        self.position_ms.store(start_ms, Ordering::Relaxed);
        self.status = PlayerStatus::Playing;

        let _ = self.event_tx.send(OutgoingEvent::TrackStart {
            guild_id: self.guild_id.clone(),
            track: resolved.info.clone(),
        });

        let voice = match self.voice.clone() {
            Some(v) => v,
            None => {
                tracing::error!("play called but not connected — use !join first");
                return;
            }
        };

        let event_tx = self.event_tx.clone();
        let guild_id = self.guild_id.clone();
        let track_info = resolved.info.clone();
        let position_ms = self.position_ms.clone();
        let ffmpeg = self.config.audio.ffmpeg_path.clone();

        self.pipeline = Some(tokio::spawn(async move {
            let reason = match stream_track(&voice, &resolved, start_ms, &ffmpeg, position_ms).await {
                Ok(()) => TrackEndReason::Finished,
                Err(e) => {
                    tracing::error!("stream error: {}", e);
                    TrackEndReason::LoadFailed
                }
            };
            let _ = event_tx.send(OutgoingEvent::TrackEnd { guild_id, track: track_info, reason });
        }));
    }

    pub fn stop(&mut self) {
        if let Some(h) = self.pipeline.take() {
            h.abort();
        }
        self.status = PlayerStatus::Stopped;
        if let Some(track) = self.current_track.take() {
            let _ = self.event_tx.send(OutgoingEvent::TrackEnd {
                guild_id: self.guild_id.clone(),
                track,
                reason: TrackEndReason::Stopped,
            });
        }
    }

    pub fn set_pause(&mut self, pause: bool) {
        self.status = if pause { PlayerStatus::Paused } else { PlayerStatus::Playing };
    }

    pub async fn disconnect(&mut self) {
        self.stop();
        if let Some(conn) = self.voice.take() {
            let _ = conn.disconnect().await;
        }
    }

    pub fn position(&self) -> u64 {
        self.position_ms.load(Ordering::Relaxed)
    }

}

async fn stream_track(
    voice: &VoiceConnection,
    resolved: &ResolvedTrack,
    start_ms: u64,
    ffmpeg: &str,
    position_ms: Arc<AtomicU64>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::AsyncReadExt;
    use tokio::sync::mpsc::error::TryRecvError;

    const FRAME_BYTES: usize = SAMPLES_PER_CHANNEL as usize * NUM_CHANNELS as usize * 2;

    let mut cmd = tokio::process::Command::new(ffmpeg);
    if !resolved.http_headers.is_empty() {
        cmd.args(["-headers", &resolved.http_headers]);
    }
    cmd.args(["-reconnect", "1", "-reconnect_streamed", "1", "-reconnect_delay_max", "5"]);
    if start_ms > 0 {
        cmd.args(["-ss", &format!("{:.3}", start_ms as f64 / 1000.0)]);
    }
    cmd.args(["-i", &resolved.stream_url, "-vn", "-f", "s16le", "-ac", "2", "-ar", "48000", "-acodec", "pcm_s16le", "-"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());

    let mut child = cmd.spawn()?;
    let mut stdout = child.stdout.take().ok_or("ffmpeg: no stdout")?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<i16>>(50);
    tokio::spawn(async move {
        let mut buf = vec![0u8; FRAME_BYTES];
        loop {
            match stdout.read_exact(&mut buf).await {
                Ok(_) => {}
                Err(_) => break,
            }
            let samples: Vec<i16> = buf.chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();
            if tx.send(samples).await.is_err() { break; }
        }
    });

    let silence = vec![0i16; SAMPLES_PER_CHANNEL as usize * NUM_CHANNELS as usize];
    let mut ticker = tokio::time::interval(Duration::from_millis(20));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut pos = start_ms;

    loop {
        ticker.tick().await;
        let samples = match rx.try_recv() {
            Ok(s) => s,
            Err(TryRecvError::Empty) => silence.clone(),
            Err(TryRecvError::Disconnected) => break,
        };
        pos += 20;
        position_ms.store(pos, Ordering::Relaxed);
        voice.capture_frame(AudioFrame {
            data: samples.into(),
            sample_rate: SAMPLE_RATE,
            num_channels: NUM_CHANNELS,
            samples_per_channel: SAMPLES_PER_CHANNEL,
        }).await?;
    }

    child.wait().await?;
    Ok(())
}

fn placeholder_track(identifier: &str) -> TrackInfo {
    TrackInfo {
        identifier: identifier.to_string(),
        title: identifier.to_string(),
        author: String::new(),
        length: 0,
        uri: identifier.to_string(),
        artwork_url: None,
        is_stream: false,
        source_name: String::new(),
    }
}
