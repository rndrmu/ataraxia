// Music bot using the ataraxia command framework and Nightingale audio node.
//
// Reads from .env:
//   REVOLT_TOKEN        — bot token
//   NIGHTINGALE_URL     — ws://host:port/v1/websocket  (default: ws://127.0.0.1:2333/v1/websocket)
//   NIGHTINGALE_PASSWORD — (default: youshallnotpass)
//
// Commands:
//   !play <url>   — queue a track, start if idle
//   !skip         — skip current track
//   !stop         — stop and clear queue
//   !pause        — toggle pause/resume
//   !queue        — show queue
//   !shuffle      — shuffle the queue
//   !repeat       — cycle repeat (off → track → queue)
//   !np           — show current track

use std::collections::VecDeque;
use std::sync::{Arc, OnceLock};

use ataraxia::{
    async_trait,
    command::CommandFramework,
    context::Context,
    models::{message::Message, ready::Ready},
    websocket::{Client, EventHandler},
};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::error;

// ── Global state ──────────────────────────────────────────────────────────────

static STATE: OnceLock<Arc<MusicState>> = OnceLock::new();

fn state() -> &'static Arc<MusicState> {
    STATE.get().expect("MusicState not initialized")
}

struct MusicState {
    nightingale: NightingaleClient,
    guilds: DashMap<String, GuildState>,
}

impl MusicState {
    async fn play_next(&self, guild_id: &str) {
        let next = {
            let mut entry = self.guilds.entry(guild_id.to_string()).or_default();
            let s = entry.value_mut();
            if let Some(track) = s.queue.pop_front() {
                if s.repeat == RepeatMode::Queue {
                    s.queue.push_back(track.clone());
                }
                s.current = Some(track.clone());
                Some((track, s.channel_id.clone()))
            } else {
                s.current = None;
                None
            }
        };
        if let Some((track, Some(_channel_id))) = next {
            // LiveKit connection already established by !join — just play.
            self.nightingale.play(guild_id, &track, 0).await;
        }
    }
}

// ── Queue state ───────────────────────────────────────────────────────────────

#[derive(Default)]
struct GuildState {
    queue: VecDeque<String>,
    current: Option<String>,
    channel_id: Option<String>,
    repeat: RepeatMode,
    paused: bool,
}

#[derive(Default, PartialEq, Clone)]
enum RepeatMode { #[default] Off, Track, Queue }

impl RepeatMode {
    fn cycle(&self) -> Self {
        match self { RepeatMode::Off => RepeatMode::Track, RepeatMode::Track => RepeatMode::Queue, RepeatMode::Queue => RepeatMode::Off }
    }
    fn label(&self) -> &'static str {
        match self { RepeatMode::Off => "off", RepeatMode::Track => "🔂 track", RepeatMode::Queue => "🔁 queue" }
    }
}

// ── Nightingale WS client ─────────────────────────────────────────────────────

type WsTx = Arc<Mutex<futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    WsMessage,
>>>;

#[derive(Clone)]
struct NightingaleClient {
    tx: WsTx,
}

impl NightingaleClient {
    async fn connect(url: &str, password: &str, event_tx: mpsc::UnboundedSender<Value>) -> Self {
        use tokio_tungstenite::tungstenite::client::IntoClientRequest;
        let mut req = url.into_client_request().expect("invalid nightingale URL");
        let headers = req.headers_mut();
        headers.insert("Authorization", password.parse().unwrap());
        headers.insert("Client-Name", "ataraxia-music-bot/1.0".parse().unwrap());

        tracing::info!("connecting to nightingale at {}", url);
        let (ws, _) = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            connect_async(req),
        )
        .await
        .expect("timed out connecting to nightingale — is it running?")
        .expect("failed to connect to nightingale");
        tracing::info!("connected to nightingale");
        let (sink, mut stream) = ws.split();

        tokio::spawn(async move {
            while let Some(Ok(WsMessage::Text(text))) = stream.next().await {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    let _ = event_tx.send(v);
                }
            }
        });

        Self { tx: Arc::new(Mutex::new(sink)) }
    }

    async fn send(&self, msg: Value) {
        let _ = self.tx.lock().await.send(WsMessage::Text(msg.to_string())).await;
    }

    async fn voice_connect(&self, guild_id: &str, channel_id: &str, url: &str, token: &str) {
        self.send(json!({
            "op": "voiceConnect",
            "guildId": guild_id,
            "channelId": channel_id,
            "url": url,
            "token": token,
        })).await;
    }

    async fn play(&self, guild_id: &str, identifier: &str, start_ms: u64) {
        self.send(json!({ "op": "play", "guildId": guild_id, "track": { "encoded": identifier }, "startTime": start_ms })).await;
    }

    async fn stop(&self, guild_id: &str) {
        self.send(json!({ "op": "stop", "guildId": guild_id })).await;
    }

    async fn pause(&self, guild_id: &str, pause: bool) {
        self.send(json!({ "op": "pause", "guildId": guild_id, "pause": pause })).await;
    }

    async fn destroy(&self, guild_id: &str) {
        self.send(json!({ "op": "destroy", "guildId": guild_id })).await;
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Strip `!<cmd> ` prefix from message content to get the argument.
fn arg_of(msg: &Message, cmd: &str) -> String {
    msg.content
        .trim()
        .strip_prefix(&format!("!{}", cmd))
        .unwrap_or("")
        .trim()
        .to_string()
}

// ── Commands ──────────────────────────────────────────────────────────────────

#[ataraxia::command(name = "join", description = "Join a voice channel")]
async fn join(ctx: Context, msg: Message) {
    let channel_id = arg_of(&msg, "join");
    if channel_id.is_empty() {
        ctx.reply("usage: `!join <voice-channel-id>`").await;
        return;
    }

    let s = state();
    let guild_id = msg.channel_id.0.clone();

    let vc = match ctx.join_voice_channel(&channel_id).await {
        Ok(vc) => vc,
        Err(e) => {
            ctx.reply(&format!("failed to join voice channel: {}", e)).await;
            return;
        }
    };

    s.guilds.entry(guild_id.clone()).or_default().channel_id = Some(channel_id.clone());
    s.nightingale.voice_connect(&guild_id, &channel_id, &vc.url, &vc.token).await;
    ctx.reply("✅ joined voice channel").await;
}

#[ataraxia::command(name = "play", description = "Queue a track and start playback")]
async fn play(ctx: Context, msg: Message) {
    let url = arg_of(&msg, "play");
    if url.is_empty() {
        ctx.reply("usage: `!play <url>`").await;
        return;
    }

    let s = state();
    let guild_id = msg.channel_id.0.clone();

    let (is_idle, pos) = {
        let mut entry = s.guilds.entry(guild_id.clone()).or_default();
        let gs = entry.value_mut();
        if gs.channel_id.is_none() {
            ctx.reply("use `!join <channel-id>` first").await;
            return;
        }
        let idle = gs.current.is_none() && gs.queue.is_empty();
        gs.queue.push_back(url.clone());
        (idle, gs.queue.len())
    };

    if is_idle {
        s.play_next(&guild_id).await;
        ctx.reply(&format!("▶ playing: {}", url)).await;
    } else {
        ctx.reply(&format!("➕ added to queue (position {})", pos)).await;
    }
}

#[ataraxia::command(name = "skip", description = "Skip the current track")]
async fn skip(ctx: Context, msg: Message) {
    let s = state();
    let guild_id = msg.channel_id.0.clone();

    let has_current = s.guilds.get(&guild_id).map(|g| g.current.is_some()).unwrap_or(false);
    if !has_current {
        ctx.reply("nothing is playing").await;
        return;
    }

    // Clear repeat-track so play_next advances rather than repeating
    if let Some(mut gs) = s.guilds.get_mut(&guild_id) {
        if gs.repeat == RepeatMode::Track {
            gs.current = None;
        }
    }

    s.nightingale.stop(&guild_id).await;
    s.play_next(&guild_id).await;
    ctx.reply("⏭ skipped").await;
}

#[ataraxia::command(name = "stop", description = "Stop playback and clear the queue")]
async fn stop(ctx: Context, msg: Message) {
    let s = state();
    let guild_id = msg.channel_id.0.clone();

    if let Some(mut gs) = s.guilds.get_mut(&guild_id) {
        gs.queue.clear();
        gs.current = None;
    }
    s.nightingale.stop(&guild_id).await;
    s.nightingale.destroy(&guild_id).await;
    ctx.reply("⏹ stopped").await;
}

#[ataraxia::command(name = "pause", description = "Toggle pause/resume")]
async fn pause(ctx: Context, msg: Message) {
    let s = state();
    let guild_id = msg.channel_id.0.clone();

    let paused = {
        let mut entry = s.guilds.entry(guild_id.clone()).or_default();
        entry.paused = !entry.paused;
        entry.paused
    };
    s.nightingale.pause(&guild_id, paused).await;
    ctx.reply(if paused { "⏸ paused" } else { "▶ resumed" }).await;
}

#[ataraxia::command(name = "queue", description = "Show the queue", aliases(q))]
async fn queue(ctx: Context, msg: Message) {
    let s = state();
    let guild_id = msg.channel_id.0.clone();
    let entry = s.guilds.entry(guild_id).or_default();
    let gs = entry.value();

    if gs.current.is_none() && gs.queue.is_empty() {
        ctx.reply("queue is empty").await;
        return;
    }

    let mut lines = Vec::new();
    if let Some(cur) = &gs.current {
        lines.push(format!("▶ **now playing:** {}", cur));
    }
    for (i, track) in gs.queue.iter().enumerate().take(10) {
        lines.push(format!("{}. {}", i + 1, track));
    }
    if gs.queue.len() > 10 {
        lines.push(format!("*... and {} more*", gs.queue.len() - 10));
    }
    lines.push(format!("repeat: **{}**", gs.repeat.label()));
    ctx.reply(&lines.join("\n")).await;
}

#[ataraxia::command(name = "shuffle", description = "Shuffle the queue")]
async fn shuffle(ctx: Context, msg: Message) {
    let s = state();
    let guild_id = msg.channel_id.0.clone();

    let mut entry = s.guilds.entry(guild_id).or_default();
    let mut vec: Vec<_> = entry.queue.drain(..).collect();
    vec.shuffle(&mut rand::thread_rng());
    entry.queue = vec.into();
    ctx.reply("🔀 queue shuffled").await;
}

#[ataraxia::command(name = "repeat", description = "Cycle repeat mode (off → track → queue)")]
async fn repeat(ctx: Context, msg: Message) {
    let s = state();
    let guild_id = msg.channel_id.0.clone();

    let mut entry = s.guilds.entry(guild_id).or_default();
    entry.repeat = entry.repeat.cycle();
    ctx.reply(&format!("repeat: **{}**", entry.repeat.label())).await;
}

#[ataraxia::command(name = "np", description = "Show the current track", aliases(nowplaying))]
async fn np(ctx: Context, msg: Message) {
    let s = state();
    let guild_id = msg.channel_id.0.clone();

    let entry = s.guilds.entry(guild_id).or_default();
    match &entry.current {
        Some(cur) => ctx.reply(&format!("▶ **{}**", cur)).await,
        None => ctx.reply("nothing is playing").await,
    }
}

// ── Event handler wrapper ─────────────────────────────────────────────────────
//
// CommandFramework's ready/authenticated are no-ops. Wrap it so we get
// visible startup logs and can see if auth actually succeeded.

struct MusicHandler {
    framework: CommandFramework,
}

#[async_trait]
impl EventHandler for MusicHandler {
    async fn authenticated(&self) {
        tracing::info!("authenticated with stoat.chat");
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        let me = ready.users.iter()
            .find(|u| u.bot.is_some())
            .or_else(|| ready.users.first());
        if let Some(u) = me {
            tracing::info!("logged in as {} ({})", u.username, u.user_id);
        }
    }

    async fn on_message(&self, ctx: Context, msg: Message) {
        self.framework.dispatch(ctx, msg).await;
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let token = std::env::var("REVOLT_TOKEN").expect("REVOLT_TOKEN not set");
    let ng_url = std::env::var("NIGHTINGALE_URL")
        .unwrap_or_else(|_| "ws://127.0.0.1:2333/v1/websocket".to_string());
    let ng_pass = std::env::var("NIGHTINGALE_PASSWORD")
        .unwrap_or_else(|_| "youshallnotpass".to_string());

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Value>();
    let nightingale = NightingaleClient::connect(&ng_url, &ng_pass, event_tx).await;

    STATE.set(Arc::new(MusicState {
        nightingale,
        guilds: DashMap::new(),
    })).ok();

    // Handle Nightingale events: advance queue on trackEnd
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event["op"].as_str() {
                Some("ready") => {
                    tracing::info!("nightingale ready, session: {}", event["sessionId"].as_str().unwrap_or("?"));
                }
                Some("trackEnd") => {
                    let guild_id = match event["guildId"].as_str() {
                        Some(id) => id.to_string(),
                        None => continue,
                    };
                    let reason = event["reason"].as_str().unwrap_or("");
                    if reason == "replaced" || reason == "stopped" { continue; }

                    let s = state();
                    let next_track = {
                        let mut entry = s.guilds.entry(guild_id.clone()).or_default();
                        let gs = entry.value_mut();
                        if gs.repeat == RepeatMode::Track {
                            if let Some(cur) = gs.current.clone() {
                                gs.queue.push_front(cur);
                            }
                        }
                        gs.current = None;
                        gs.queue.front().cloned()
                    };
                    if next_track.is_some() {
                        s.play_next(&guild_id).await;
                    }
                }
                Some("trackException") => {
                    error!("track exception in {}: {}",
                        event["guildId"].as_str().unwrap_or("?"),
                        event["exception"]["message"].as_str().unwrap_or("unknown"),
                    );
                }
                _ => {}
            }
        }
    });

    let framework = MusicHandler {
        framework: CommandFramework::new("!")
            .command(&JOIN_COMMAND)
            .command(&PLAY_COMMAND)
            .command(&SKIP_COMMAND)
            .command(&STOP_COMMAND)
            .command(&PAUSE_COMMAND)
            .command(&QUEUE_COMMAND)
            .command(&SHUFFLE_COMMAND)
            .command(&REPEAT_COMMAND)
            .command(&NP_COMMAND),
    };

    let mut client = Client::new(token)
        .event_handler(framework)
        .set_api_url("https://stoat.chat/api")
        .await;

    client.start().await;
}
