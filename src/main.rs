use tokio;

use revoltchat_rs::websocket::Client;
use revoltchat_rs::websocket::EventHandler;
use revoltchat_rs::{models::message::Message as RevoltMessage, http::Http};
use revoltchat_rs::context::Context;

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
            ctx.reply("pong").await;
        }
    }
}




#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let token = std::env::var("REVOLT_TOKEN").expect("token");



    Client::new(token).await.run(Handler).await;




}


