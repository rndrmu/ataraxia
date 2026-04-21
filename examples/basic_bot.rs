use std::sync::Arc;
use tokio::sync::Mutex;

use ataraxia::{
    async_trait,
    context::Context,
    models::{message::Message as RevoltMessage, ready::Ready},
    websocket::{Client, EventHandler},
};
use ataraxia_voice::VoiceConnection;

struct Handler {
    voice: Arc<Mutex<Option<VoiceConnection>>>,
}

impl Handler {
    fn new() -> Self {
        Self {
            voice: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn authenticated(&self) {
        println!("Authenticated!");
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Ready!");
        let names: Vec<_> = ready.users.iter().map(|u| &u.username).collect();
        println!("Logged in, saw users: {:?}", names);
    }

    async fn on_message(&self, ctx: Context, message: RevoltMessage) {
        let content = message.content.as_str();

        if content == "!ping" {
            let _ = message
                .channel_id
                .send_message(&ctx.http, |r| {
                    r.content("pong!").create_embed(|e| {
                        e.title("Pong!").description("I'm alive!").colour("#00ff00")
                    })
                })
                .await;
        } else if let Some(channel_id) = content.strip_prefix("!join ") {
            let channel_id = channel_id.trim();

            let vc = match ctx.join_voice_channel(channel_id).await {
                Ok(vc) => vc,
                Err(e) => {
                    ctx.reply(&format!("Failed to join voice: {}", e)).await;
                    return;
                }
            };

            ctx.reply("Joining voice channel...").await;

            match VoiceConnection::connect(&vc.url, &vc.token).await {
                Ok(conn) => {
                    *self.voice.lock().await = Some(conn);
                    ctx.reply("Connected! Use `!play <url>` to play audio.")
                        .await;
                }
                Err(e) => {
                    ctx.reply(&format!("Voice connection failed: {}", e)).await;
                }
            }
        } else if let Some(url) = content.strip_prefix("!play ") {
            let url = url.trim().to_string();
            let voice = self.voice.clone();

            let guard = voice.lock().await;
            match guard.as_ref() {
                None => {
                    ctx.reply("Not in a voice channel — use `!join <channel_id>` first.")
                        .await;
                }
                Some(conn) => {
                    ctx.reply(&format!("Playing `{}`...", url)).await;
                    if let Err(e) = conn.play_youtube(&url).await {
                        ctx.reply(&format!("Playback error: {}", e)).await;
                    } else {
                        ctx.reply("Done!").await;
                    }
                }
            }
        } else if content == "!leave" {
            let mut guard = self.voice.lock().await;
            match guard.take() {
                None => {
                    ctx.reply("Not in a voice channel.").await;
                }
                Some(conn) => {
                    let _ = conn.disconnect().await;
                    ctx.reply("Left the voice channel.").await;
                }
            }
        } else if content == "!me" {
            match message.author.get_author_user(&ctx.http).await {
                Ok(user) => ctx.reply(&format!("{:?}", user)).await,
                Err(e) => ctx.reply(&format!("Error: {}", e)).await,
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info,ataraxia=info");
    tracing_subscriber::fmt::init();

    let token = std::env::var("REVOLT_TOKEN").expect("REVOLT_TOKEN not set");

    let mut client = Client::new(token)
        .event_handler(Handler::new())
        .set_api_url("https://stoat.chat/api")
        .await;

    client.start().await;
}
