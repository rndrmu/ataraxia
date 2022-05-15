use tokio;


use ataraxia::websocket::Client;
use ataraxia::websocket::EventHandler;
use ataraxia::{models::message::Message as RevoltMessage, http::Http};
use ataraxia::context::Context;


struct Handler;

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn authenticated(&self) {
        println!("Authenticated!");
    }
    async fn ready(&self, ctx: Context) {
        println!("Ready!");
        println!(":trol:");
    }
    async fn on_message(&self, ctx: Context, message: RevoltMessage) {
        println!("{}", message);
        if message.content == "!ping" {
            println!("Pong!");
            ctx.reply("pong").await;
        } else if message.content.starts_with("!join") {
            let voice_channel_id = message.content.split(" ").collect::<Vec<&str>>()[1];
            println!("Joining voice channel {}", voice_channel_id);
            let vc = ctx.join_voice_channel(voice_channel_id).await.unwrap();
        }
    }
}




#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let token = std::env::var("REVOLT_TOKEN").expect("token");



    Client::new(token).await.run(Handler).await;




}


