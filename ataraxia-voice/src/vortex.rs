//! Vortex voice connection for Revolt/stoat.chat.
//!
//! # Quick start
//! ```ignore
//! // 1. Get a voice token from the Revolt HTTP API:
//! let vc = ctx.join_voice_channel(channel_id).await?;
//!
//! // 2. Open a voice connection:
//! let mut conn = VoiceConnection::connect(&vc.token, channel_id).await?;
//!
//! // 3. Play a local audio file (requires ffmpeg in PATH):
//! conn.play_file("/path/to/audio.mp3").await?;
//! ```

use std::error::Error;

use base64::Engine as _;
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use futures_util::stream::{SplitSink, SplitStream};

use crate::{rtp::RtpPacket, srtp::SrtpContext};

type WsTx = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsRx = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// 20 ms Opus frame at 48 kHz stereo
const SAMPLES_PER_FRAME: usize = 960;
const OPUS_PAYLOAD_TYPE: u8 = 120;
// TODO: vortex.revolt.chat is superseded by stoat.chat Voice Chats v2.
// This URL needs updating once the Voice Chats v2 endpoint is documented.
const VORTEX_WS: &str = "wss://vortex.revolt.chat";

pub struct VoiceConnection {
    ws_tx: WsTx,
    // Kept alive to hold the WebSocket connection open; receives keepalives/server events.
    #[allow(dead_code)]
    ws_rx: WsRx,
    udp: tokio::net::UdpSocket,
    srtp: SrtpContext,
    ssrc: u32,
    sequence: u16,
    timestamp: u32,
}

impl VoiceConnection {
    /// Connect to Vortex, perform the full signalling handshake, and return a
    /// ready-to-use connection.
    ///
    /// - `voice_token`: the token returned by `POST /channels/{id}/join_call`
    /// - `channel_id`: the voice channel ID
    pub async fn connect(
        voice_token: &str,
        channel_id: &str,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Generate all random material before the first .await — ThreadRng is !Send
        // and cannot be held across await points.
        let (ssrc, master_key, master_salt, key_base64, cname) = {
            let mut rng = rand::thread_rng();
            let ssrc: u32 = rng.gen();
            let mut master_key = [0u8; 16];
            let mut master_salt = [0u8; 14];
            rng.fill(&mut master_key);
            rng.fill(&mut master_salt);
            let mut key_material = [0u8; 30];
            key_material[..16].copy_from_slice(&master_key);
            key_material[16..].copy_from_slice(&master_salt);
            let key_base64 = base64::engine::general_purpose::STANDARD.encode(key_material);
            let cname = uuid::Uuid::new_v4().to_string();
            (ssrc, master_key, master_salt, key_base64, cname)
        };

        let (ws_stream, _) = connect_async(VORTEX_WS).await?;
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        // ── Step 1: Authenticate ──────────────────────────────────────────────
        ws_tx
            .send(Message::Text(
                json!({
                    "id": 0,
                    "type": "Authenticate",
                    "data": { "roomId": channel_id, "token": voice_token }
                })
                .to_string(),
            ))
            .await?;

        // ── Step 2: InitializeTransports ─────────────────────────────────────
        ws_tx
            .send(Message::Text(
                json!({
                    "id": 1,
                    "type": "InitializeTransports",
                    "data": {
                        "mode": "CombinedRTP",
                        "rtpCapabilities": {
                            "codecs": [{
                                "mimeType": "audio/opus",
                                "kind": "audio",
                                "preferredPayloadType": 100,
                                "clockRate": 48000,
                                "channels": 2,
                                "parameters": { "minptime": 10, "useinbandfec": 1 },
                                "rtcpFeedback": [{ "type": "transport-cc", "parameter": "" }]
                            }],
                            "headerExtensions": [
                                {
                                    "kind": "audio",
                                    "uri": "urn:ietf:params:rtp-hdrext:sdes:mid",
                                    "preferredId": 1,
                                    "preferredEncrypt": false,
                                    "direction": "sendrecv"
                                },
                                {
                                    "kind": "audio",
                                    "uri": "urn:ietf:params:rtp-hdrext:ssrc-audio-level",
                                    "preferredId": 10,
                                    "preferredEncrypt": false,
                                    "direction": "sendrecv"
                                }
                            ]
                        }
                    }
                })
                .to_string(),
            ))
            .await?;

        // ── Step 3: Wait for InitializeTransports response ───────────────────
        let (transport_id, udp_ip, udp_port) = wait_for_init_transports(&mut ws_rx).await?;

        // ── Step 4: Connect UDP ───────────────────────────────────────────────
        let udp = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        udp.connect(format!("{}:{}", udp_ip, udp_port)).await?;

        // ── Step 5: ConnectTransport ──────────────────────────────────────────
        ws_tx
            .send(Message::Text(
                json!({
                    "id": 2,
                    "type": "ConnectTransport",
                    "data": {
                        "id": transport_id,
                        "srtpParameters": {
                            "cryptoSuite": "AES_CM_128_HMAC_SHA1_80",
                            "keyBase64": key_base64
                        }
                    }
                })
                .to_string(),
            ))
            .await?;

        // ── Step 6: StartProduce ──────────────────────────────────────────────
        ws_tx
            .send(Message::Text(
                json!({
                    "id": 3,
                    "type": "StartProduce",
                    "data": {
                        "type": "audio",
                        "rtpParameters": {
                            "mid": "0",
                            "codecs": [{
                                "channels": 2,
                                "clockRate": 48000,
                                "mimeType": "audio/opus",
                                "payloadType": OPUS_PAYLOAD_TYPE,
                                "parameters": {},
                                "rtcpFeedback": []
                            }],
                            "headerExtensions": [],
                            "encodings": [{ "maxBitrate": 512000, "ssrc": ssrc }],
                            "rtcp": { "cname": cname, "reducedSize": false }
                        }
                    }
                })
                .to_string(),
            ))
            .await?;

        // ── Step 7: Wait for StartProduce confirmation ────────────────────────
        wait_for_start_produce(&mut ws_rx).await?;

        let srtp = SrtpContext::new(master_key, master_salt, ssrc);

        Ok(VoiceConnection {
            ws_tx,
            ws_rx,
            udp,
            srtp,
            ssrc,
            sequence: 0,
            timestamp: 0,
        })
    }

    /// Play a local audio file over the voice channel.
    ///
    /// Requires `ffmpeg` to be available in `PATH`.
    /// Blocks (async) until the file finishes playing.
    pub async fn play_file(&mut self, path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let pcm = transcode_to_pcm(path).await?;

        let mut encoder =
            opus::Encoder::new(48000, opus::Channels::Stereo, opus::Application::Audio)?;

        // stereo i16 samples: 960 samples/ch * 2 ch * 2 bytes = 3840 bytes/frame
        const FRAME_BYTES: usize = SAMPLES_PER_FRAME * 2 * 2;
        let mut opus_buf = vec![0u8; 4000];

        for chunk in pcm.chunks(FRAME_BYTES) {
            if chunk.len() < FRAME_BYTES {
                break;
            }

            let samples: Vec<i16> = chunk
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect();

            let encoded_len = encoder.encode(&samples, &mut opus_buf)?;

            let rtp = RtpPacket {
                sequence: self.sequence,
                timestamp: self.timestamp,
                ssrc: self.ssrc,
                payload_type: OPUS_PAYLOAD_TYPE,
                payload: opus_buf[..encoded_len].to_vec(),
            };

            let rtp_bytes = rtp.to_bytes();
            let srtp_bytes = self.srtp.protect(&rtp_bytes);
            self.udp.send(&srtp_bytes).await?;

            self.sequence = self.sequence.wrapping_add(1);
            self.timestamp = self.timestamp.wrapping_add(SAMPLES_PER_FRAME as u32);

            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        Ok(())
    }

    /// Play audio from a YouTube (or any yt-dlp-supported) URL over the voice channel.
    ///
    /// Requires both `yt-dlp` and `ffmpeg` in `PATH`.
    /// Uses `yt-dlp -g` to resolve the best audio stream URL, then hands it to
    /// ffmpeg — no local file is written.
    pub async fn play_youtube(&mut self, url: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let stream_url = resolve_yt_dlp_url(url).await?;
        self.play_file(&stream_url).await
    }

    /// Send a "not speaking" signal to the Vortex server.
    pub async fn set_not_speaking(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.ws_tx
            .send(Message::Text(
                json!({ "id": 10, "type": "SetPaused", "data": { "paused": true } }).to_string(),
            ))
            .await?;
        Ok(())
    }
}

async fn wait_for_init_transports(
    rx: &mut WsRx,
) -> Result<(String, String, u16), Box<dyn Error + Send + Sync>> {
    while let Some(msg) = rx.next().await {
        let msg = msg?;
        if !msg.is_text() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(msg.to_text()?)?;
        if v["type"].as_str() == Some("InitializeTransports") {
            let id = v["data"]["id"]
                .as_str()
                .ok_or("missing transport id")?
                .to_string();
            let ip = v["data"]["ip"]
                .as_str()
                .ok_or("missing ip")?
                .to_string();
            let port = v["data"]["port"]
                .as_u64()
                .ok_or("missing port")? as u16;
            return Ok((id, ip, port));
        }
    }
    Err("Vortex connection closed before InitializeTransports".into())
}

async fn wait_for_start_produce(
    rx: &mut WsRx,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    while let Some(msg) = rx.next().await {
        let msg = msg?;
        if !msg.is_text() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(msg.to_text()?)?;
        if v["type"].as_str() == Some("StartProduce") {
            let producer_id = v["data"]["id"]
                .as_str()
                .unwrap_or("")
                .to_string();
            return Ok(producer_id);
        }
    }
    Err("Vortex connection closed before StartProduce".into())
}

/// Resolve the best audio stream URL for a YouTube (or other yt-dlp-supported) URL.
///
/// Runs `yt-dlp -f bestaudio/best -g <url>` and returns the first line of output.
async fn resolve_yt_dlp_url(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let output = tokio::process::Command::new("yt-dlp")
        .args(["-f", "bestaudio/best", "-g", url])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp failed: {}", stderr).into());
    }

    let stream_url = String::from_utf8(output.stdout)?
        .lines()
        .next()
        .ok_or("yt-dlp returned no URL")?
        .to_string();

    Ok(stream_url)
}

/// Transcode any audio file to raw signed 16-bit LE PCM at 48 kHz stereo using ffmpeg.
async fn transcode_to_pcm(path: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let output = tokio::process::Command::new("ffmpeg")
        .args([
            "-i", path,
            "-f", "s16le",
            "-ac", "2",
            "-ar", "48000",
            "-acodec", "pcm_s16le",
            "-",
        ])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffmpeg failed: {}", stderr).into());
    }

    Ok(output.stdout)
}
