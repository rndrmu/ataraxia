


use tokio;


use ataraxia::websocket::Client;
use ataraxia::websocket::EventHandler;
use ataraxia::{models::message::Message as RevoltMessage, http::Http};
use ataraxia::context::Context;


struct Handler;

#[async_trait::async_trait]
impl EventHandler for Handler {
    /// Function called when the client is authenticated
    async fn authenticated(&self) {
        println!("Authenticated!");
    }
    /// Function called when the client is ready
    async fn ready(&self, ctx: Context) {
        println!("Ready!");
        println!(":trol:");
    }
    /// Function called when a message is received, you can reply to the message with the `ctx.reply` function
    /// 
    /// # Arguments
    /// To use arguments you need to somehow split the message into a command and the arguments
    /// See the `!join` command for an example
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


    // Build the client and start it
    // Handler is the Handler Struct
    // which implements the EventHandler trait
    // and acts as "Event Loop" for the client
    Client::new(token).await.run(Handler).await;




}


