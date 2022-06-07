


use tokio;

use ataraxia::{
    websocket::{Client, EventHandler},
    models::message::Message as RevoltMessage,
    http::Http,
    context::Context,
    async_trait
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /// Function called when the client is authenticated
    async fn authenticated(&self) {
        println!("Authenticated!");
    }
    /// Function called when the client is ready
    async fn ready(&self, _ctx: Context) {
        println!("Ready!");
        println!(":trol:");
    }
    /// Function called when a message is received, you can reply to the message with the `ctx.reply` function
    /// 
    /// ### Arguments
    /// To use arguments you need to somehow split the message into a command and the arguments
    /// See the `!join` command for an example
    async fn on_message(&self, ctx: Context, message: RevoltMessage) {
        println!("{}", message);
        if message.content == "!ping" {
            println!("{:?}", ctx.json);
            println!("Pong!");
            ctx.reply("pong").await;
        } else if message.content.starts_with("!join") {
            let voice_channel_id = message.content.split(" ").collect::<Vec<&str>>()[1];
            println!("Joining voice channel {}", voice_channel_id);
            let _vc = ctx.join_voice_channel(voice_channel_id).await.unwrap();
            ctx.reply("Okay, i joined the channel!").await;
        } else if message.content == "!chinf" {
            let chn = ctx.get_channel(&message.channel).await.unwrap();
            println!("{:?}", chn);
        }
    }
}




#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    // enable tracing at the lowest level    
    std::env::set_var("RUST_LOG", "debug");
    
    tracing_subscriber::fmt::init();




    let token = std::env::var("REVOLT_TOKEN").expect("token");


    // Build the client and start it
    // Handler is the Handler Struct
    // which implements the EventHandler trait
    // and acts as "Event Loop" for the client
    // to use the default public instance of revolt, pass None as second parameter
    // other wise do Some("https://delta.revolt.chat") where delta.revolt.chat is your delta instance
    let mut client = Client::new(token, None).await;
    client.run(Handler).await;




}


