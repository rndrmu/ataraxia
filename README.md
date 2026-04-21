# ataraxia

An experimental Rust wrapper for the [stoat.chat](https://stoat.chat) API (formerly revolt.chat).  
Heavily influenced by [Serenity](https://github.com/serenity-rs/serenity).

---

## Quick start — manual event handler

```rust
use ataraxia::{
    async_trait,
    context::Context,
    models::{message::Message, ready::Ready},
    websocket::{Client, EventHandler},
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn authenticated(&self) {
        println!("Authenticated!");
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Ready! Logged in as {:?}", ready.users[0].username);
    }

    async fn on_message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            ctx.reply("pong!").await;
        }
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("REVOLT_TOKEN").expect("REVOLT_TOKEN not set");

    let mut client = Client::new(token)
        .event_handler(Handler)
        .set_api_url("https://stoat.chat/api")
        .await;

    client.start().await;
}
```

---

## Command framework

Use `#[command]` to turn any `async fn(Context, Message)` into a registered command,
then hand a `CommandFramework` to the client as your event handler.

```rust
use ataraxia::{
    command::CommandFramework,
    context::Context,
    models::message::Message,
    websocket::Client,
};

#[ataraxia::command(name = "ping", description = "Replies with pong!")]
async fn ping(ctx: Context, _msg: Message) {
    ctx.reply("pong!").await;
}

#[ataraxia::command(
    name = "hello",
    description = "Says hello",
    aliases(hi, hey)
)]
async fn hello(ctx: Context, _msg: Message) {
    ctx.reply("Hello! 👋").await;
}

#[tokio::main]
async fn main() {
    let token = std::env::var("REVOLT_TOKEN").expect("REVOLT_TOKEN not set");

    let framework = CommandFramework::new("!")
        .command(&PING_COMMAND)
        .command(&HELLO_COMMAND);

    let mut client = Client::new(token)
        .event_handler(framework)
        .set_api_url("https://stoat.chat/api")
        .await;

    client.start().await;
}
```

`#[command]` generates a `pub static <NAME>_COMMAND: Command` in the current scope.  
Supported attributes:

| Attribute | Default | Description |
|---|---|---|
| `name = "..."` | function name | trigger word after the prefix |
| `description = "..."` | `""` | help text |
| `aliases(a, b)` | none | additional trigger words |

---

## Voice (LiveKit)

```rust
use ataraxia_voice::VoiceConnection;

// inside on_message:
let vc = ctx.join_voice_channel(channel_id).await?;
let conn = VoiceConnection::connect(&vc.url, &vc.token).await?;

// streams audio in real-time via yt-dlp + ffmpeg (no buffering)
conn.play_youtube("https://www.youtube.com/watch?v=8Nt9YPnYyjs").await?;

conn.disconnect().await?;
```

Requires `yt-dlp` and `ffmpeg` in `PATH`.  
Enable the `voice` feature on the `ataraxia` crate:

```toml
ataraxia = { path = "...", features = ["voice"] }
ataraxia-voice = { path = "../ataraxia-voice" }
```

---

## Nightingale — standalone audio node

Nightingale is a self-hosted audio node (similar to [Lavalink](https://github.com/lavalink-devs/Lavalink)) that handles voice connections and audio playback independently from your bot process.

Your bot resolves the LiveKit credentials via `ctx.join_voice_channel()` and hands them to Nightingale over WebSocket. Nightingale manages the ffmpeg pipeline and streams audio directly into the voice channel.

### Running

```toml
# nightingale/nightingale.toml
[server]
host = "0.0.0.0"
port = 2333
password = "youshallnotpass"

[audio]
ffmpeg_path = "ffmpeg"
ytdlp_path = "yt-dlp"
default_volume = 100
```

```sh
cargo run -p nightingale
```

Requires `ffmpeg` and `yt-dlp` in `PATH`.

### Bot-side usage

Send a `voiceConnect` op after joining the channel, then use the REST API or WebSocket ops to control playback:

```
POST   /v1/sessions/:id/players/:guild_id   { "track": "..." }
PATCH  /v1/sessions/:id/players/:guild_id   { "paused": true }
DELETE /v1/sessions/:id/players/:guild_id
GET    /v1/loadtracks?identifier=ytsearch:...
```

See `examples/music_bot.rs` for a full bot implementation with queue, shuffle, and repeat.

---

## Workspace layout

| Crate | Description |
|---|---|
| `ataraxia` | Core library — gateway, HTTP, models, command framework |
| `ataraxia/macros` | Proc-macro crate (`#[command]`) |
| `ataraxia-voice` | LiveKit voice support |
| `nightingale` | Standalone audio node (Lavalink-style) |
| `examples` | Example bots |
