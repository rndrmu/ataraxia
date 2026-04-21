use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use livekit::{
    options::TrackPublishOptions,
    track::{LocalAudioTrack, LocalTrack, TrackSource},
    webrtc::{
        audio_source::native::NativeAudioSource,
        prelude::{AudioFrame, AudioSourceOptions, RtcAudioSource},
    },
    Room, RoomOptions,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub const SAMPLE_RATE: u32 = 48000;
pub const NUM_CHANNELS: u32 = 2;
pub const SAMPLES_PER_CHANNEL: u32 = 960;

pub struct VoiceConnection {
    room: Arc<Room>,
    source: NativeAudioSource,
}

impl VoiceConnection {
    /// Connect to a LiveKit voice room.
    ///
    /// `url` and `token` come directly from the `join_call` API response.
    pub async fn connect(
        url: &str,
        token: &str,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // stoat.chat validates the token from ?access_token= in the URL (proxy-level auth),
        // not from the Authorization header that the Rust SDK sends by default.
        let url_with_token = format!("{}?access_token={}", url, token);

        // The LiveKit SDK uses tokio_tungstenite::connect_async which tries IPv6 first.
        // On networks where the LiveKit server's IPv6 address is unreachable (no RST,
        // so the OS just waits), the SDK times out before reaching IPv4.
        //
        // Fix: spin up an in-process HTTP CONNECT proxy that resolves to IPv4 only,
        // then point HTTPS_PROXY at it. The SDK's built-in proxy support picks it up.
        let proxy_port = spawn_ipv4_proxy().await?;
        // SAFETY: single-threaded startup, no other threads reading this env var yet.
        std::env::set_var("HTTPS_PROXY", format!("http://127.0.0.1:{}", proxy_port));

        let mut room_opts = RoomOptions::default();
        room_opts.connect_timeout = Duration::from_secs(10);
        let result = Room::connect(&url_with_token, token, room_opts).await;

        std::env::remove_var("HTTPS_PROXY");

        let (room, mut events) = result?;
        let room = Arc::new(room);

        tokio::spawn({
            async move {
                while let Some(_event) = events.recv().await {}
            }
        });

        let source = NativeAudioSource::new(
            AudioSourceOptions::default(),
            SAMPLE_RATE,
            NUM_CHANNELS,
            SAMPLES_PER_CHANNEL,
        );

        let track = LocalAudioTrack::create_audio_track(
            "microphone",
            RtcAudioSource::Native(source.clone()),
        );

        room.local_participant()
            .publish_track(
                LocalTrack::Audio(track),
                TrackPublishOptions {
                    source: TrackSource::Microphone,
                    ..Default::default()
                },
            )
            .await?;

        Ok(Self { room, source })
    }

    /// Play a local audio file over the voice channel.
    ///
    /// Requires `ffmpeg` in `PATH`. Streams in real-time; playback starts immediately.
    pub async fn play_file(&self, path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut child = tokio::process::Command::new("ffmpeg")
            .args(["-i", path, "-f", "s16le", "-ac", "2", "-ar", "48000", "-acodec", "pcm_s16le", "-"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        let stdout = child.stdout.take().ok_or("ffmpeg: no stdout")?;
        self.stream_frames(stdout).await?;
        child.wait().await?;
        Ok(())
    }

    /// Play audio from a YouTube (or any yt-dlp-supported) URL.
    ///
    /// Requires `yt-dlp` and `ffmpeg` in `PATH`. Resolves the direct CDN URL via
    /// yt-dlp --dump-json, then passes both the URL and the required HTTP headers to
    /// ffmpeg so the CDN doesn't 403 (YouTube ties URLs to request headers).
    pub async fn play_youtube(&self, url: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let output = tokio::process::Command::new("yt-dlp")
            .args([
                "-f", "bestaudio[ext=webm]/bestaudio[acodec=opus]/bestaudio/best",
                "--dump-json", "--no-playlist", "--quiet", url,
            ])
            .output()
            .await?;

        let info: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let stream_url = info["url"].as_str().ok_or("yt-dlp: no url in dump-json output")?.to_string();

        // Build the -headers string ffmpeg expects (CRLF-separated "Key: Value\r\n").
        let mut headers = String::new();
        if let Some(hdrs) = info["http_headers"].as_object() {
            for (k, v) in hdrs {
                if let Some(v) = v.as_str() {
                    use std::fmt::Write as _;
                    let _ = write!(headers, "{}: {}\r\n", k, v);
                }
            }
        }

        let mut cmd = tokio::process::Command::new("ffmpeg");
        if !headers.is_empty() {
            cmd.args(["-headers", &headers]);
        }
        // Reconnect on transient HTTP failures so a momentary CDN hiccup doesn't kill playback.
        cmd.args(["-reconnect", "1", "-reconnect_streamed", "1", "-reconnect_delay_max", "5"]);
        cmd.args(["-i", &stream_url, "-vn", "-f", "s16le", "-ac", "2", "-ar", "48000", "-acodec", "pcm_s16le", "-"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().ok_or("ffmpeg: no stdout")?;
        self.stream_frames(stdout).await?;
        child.wait().await?;
        Ok(())
    }

    /// Send a single pre-decoded PCM frame directly to the LiveKit source.
    ///
    /// Used by external audio nodes (e.g. Nightingale) that manage their own
    /// ffmpeg pipeline and just need a way to push frames without going through
    /// `play_file` / `play_youtube`.
    pub async fn capture_frame(&self, frame: AudioFrame<'_>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.source.capture_frame(&frame).await?;
        Ok(())
    }

    /// Disconnect from the voice channel.
    pub async fn disconnect(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.room.close().await?;
        Ok(())
    }

    async fn stream_frames(
        &self,
        mut stdout: tokio::process::ChildStdout,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        use tokio::io::AsyncReadExt;
        use tokio::sync::mpsc::error::TryRecvError;

        const FRAME_BYTES: usize = SAMPLES_PER_CHANNEL as usize * NUM_CHANNELS as usize * 2;
        // ~1 second of pre-buffer so transient read stalls don't cause audible gaps.
        const BUFFER_FRAMES: usize = 50;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<i16>>(BUFFER_FRAMES);

        // Producer: decode ffmpeg output as fast as possible into the channel.
        tokio::spawn(async move {
            let mut buf = vec![0u8; FRAME_BYTES];
            loop {
                match stdout.read_exact(&mut buf).await {
                    Ok(_) => {}
                    Err(_) => break,
                }
                let samples: Vec<i16> = buf
                    .chunks_exact(2)
                    .map(|b| i16::from_le_bytes([b[0], b[1]]))
                    .collect();
                if tx.send(samples).await.is_err() {
                    break;
                }
            }
        });

        // Consumer: pace at exactly 20ms regardless of how long reads take.
        // MissedTickBehavior::Delay shifts the schedule forward rather than
        // firing catch-up bursts, which keeps frame spacing even under load.
        let silence = vec![0i16; SAMPLES_PER_CHANNEL as usize * NUM_CHANNELS as usize];
        let mut ticker = tokio::time::interval(Duration::from_millis(20));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;
            let samples = match rx.try_recv() {
                Ok(s) => s,
                // Buffer underrun: send silence rather than stalling the ticker.
                Err(TryRecvError::Empty) => silence.clone(),
                // Producer exited and buffer is drained — we're done.
                Err(TryRecvError::Disconnected) => break,
            };
            self.source.capture_frame(&AudioFrame {
                data: samples.into(),
                sample_rate: SAMPLE_RATE,
                num_channels: NUM_CHANNELS,
                samples_per_channel: SAMPLES_PER_CHANNEL,
            }).await?;
        }

        Ok(())
    }
}

/// Spawns a minimal HTTP CONNECT proxy on a random localhost port.
///
/// When the LiveKit SDK routes through this proxy (via HTTPS_PROXY), the proxy
/// resolves the target hostname to its first IPv4 address, bypassing IPv6 entirely.
/// Returns the port the proxy is listening on.
async fn spawn_ipv4_proxy() -> Result<u16, Box<dyn Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move {
        loop {
            let Ok((client, _)) = listener.accept().await else { break };
            tokio::spawn(handle_proxy_conn(client));
        }
    });
    Ok(port)
}

async fn handle_proxy_conn(mut client: TcpStream) {
    // Read the CONNECT request line-by-line until the blank line.
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1];
    loop {
        if client.read_exact(&mut tmp).await.is_err() { return; }
        buf.push(tmp[0]);
        if buf.ends_with(b"\r\n\r\n") { break; }
        if buf.len() > 4096 { return; }
    }

    // Parse "CONNECT host:port HTTP/1.1"
    let req = String::from_utf8_lossy(&buf);
    let first_line = req.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    if parts.next() != Some("CONNECT") { return; }
    let target = match parts.next() { Some(t) => t, None => return };

    let (host, port_str) = match target.rsplit_once(':') {
        Some(p) => p,
        None => return,
    };
    let port: u16 = match port_str.parse() {
        Ok(p) => p,
        Err(_) => return,
    };

    // Resolve to IPv4 only.
    let addrs: Vec<_> = match tokio::net::lookup_host(format!("{}:{}", host, port)).await {
        Ok(a) => a.collect(),
        Err(_) => return,
    };
    let addr = match addrs.iter().find(|a| a.is_ipv4()).or_else(|| addrs.first()) {
        Some(a) => *a,
        None => return,
    };

    let mut server = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(_) => {
            let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
            return;
        }
    };

    let _ = client.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").await;

    // Bidirectional proxy.
    let _ = tokio::io::copy_bidirectional(&mut client, &mut server).await;
}


