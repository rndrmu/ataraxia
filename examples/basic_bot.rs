use ataraxia::{
    async_trait,
    context::Context,
    models::{message::Message as RevoltMessage, ready::Ready},
    websocket::{Client, EventHandler},
};

struct Handler;

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
        println!("{}", message);

        if message.content == "!ping" {
            // Send a reply with an embed and a masquerade
            let msg = message
                .channel_id
                .send_message(&ctx.http, |r| {
                    r.content("pong!")
                        .set_masquerade(|m| m.name("Pong Bot"))
                        .create_embed(|e| {
                            e.title("Pong!")
                                .description("I'm alive!")
                                .colour("#00ff00")
                        })
                })
                .await;

            println!("Sent: {:?}", msg);

        } else if message.content.starts_with("!join") {
            // Join a voice channel and connect with ataraxia-voice
            let parts: Vec<&str> = message.content.split_whitespace().collect();
            if parts.len() < 2 {
                ctx.reply("Usage: !join <channel_id>").await;
                return;
            }
            let channel_id = parts[1];

            let vc = match ctx.join_voice_channel(channel_id).await {
                Ok(vc) => vc,
                Err(e) => {
                    ctx.reply(&format!("Failed to join voice: {}", e)).await;
                    return;
                }
            };

            ctx.reply("Joining voice channel...").await;

            let mut conn = match ataraxia_voice::VoiceConnection::connect(&vc.token, channel_id).await {
                Ok(c) => c,
                Err(e) => {
                    ctx.reply(&format!("Voice connection failed: {}", e)).await;
                    return;
                }
            };

            ctx.reply("Connected! Playing audio...").await;

            // Play a file — change this path to your audio file
            if let Err(e) = conn.play_file("/tmp/audio.mp3").await {
                ctx.reply(&format!("Playback error: {}", e)).await;
            }

        } else if message.content == "!me" {
            let user = message.author.get_author_user(&ctx.http).await.unwrap();
            ctx.reply(&format!("{:?}", user)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = std::env::var("REVOLT_TOKEN").expect("REVOLT_TOKEN not set");

    let mut client = Client::new(token)
        .event_handler(Handler)
        .set_api_url("https://api.revolt.chat")
        .await;

    client.start().await;
}
