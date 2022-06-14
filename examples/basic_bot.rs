


use tokio;

use ataraxia::{
    websocket::{Client, EventHandler},
    models::{message::Message as RevoltMessage, ready::Ready},
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
    /// Function called when the client is ready to receive events
    async fn ready(&self, _ctx: Context, ready: serde_json::Value) {
        println!("Ready!");
        println!("Received Payload {:?}", ready);
    }

    /// Function called when a message is received, you can reply to the message with the `ctx.reply` function
    /// 
    /// ### How to use Arguments
    /// To use arguments you need to somehow split the message into a command and the arguments
    /// See the `!join` command for an example
    async fn on_message(&self, ctx: Context, message: RevoltMessage) {
        println!("{}", message);

        if message.content == "!ping" {


            ctx.reply_builder(&message.channel_id, |r| {
                r.content("hello!")
                .set_masquerade(|masquerade| {
                    masquerade.name("Rainer Winkler").avatar("https://cdn.discordapp.com/avatars/242385294123335690/d0b2755d7113d0630a51ead93b4dfd67.png")
                })
                .create_embed(|embed| {
                    embed.title("Test Embed")
                    .description("Ich bin nicht derjeniche!")
                    .url("https://www.youtube.com/watch?v=FcSeR4fdqbs")
                    .colour("#ff0000")
                    .icon_url("https://imgflip.com/meme/382391167/Rainer-Winkler-Br")
                })
                .create_embed(|embed2| {
                    embed2.title("Test Embed 2")
                    .description("Ich bin nicht derjeniche!")
                    .url("https://www.youtube.com/watch?v=FcSeR4fdqbs")
                    .colour("#00ffff")
                    .icon_url("https://imgflip.com/meme/382391167/Rainer-Winkler-Br")
                })
            }).await


        } else if message.content.starts_with("!join") {


            let voice_channel_id = message.content.split(" ").collect::<Vec<&str>>()[1];
            println!("Joining voice channel {}", voice_channel_id);
            let _vc = ctx.join_voice_channel(voice_channel_id).await.unwrap();
            ctx.reply("Okay, i joined the channel!").await;


        } else if message.content == "!channelinfo" {
            let chn = ctx.get_channel(&message.channel_id).await.unwrap();
            println!("{:?}", chn);
        }
    }
}




#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    std::env::set_var("RUST_LOG", "debug"); // noisy logging 
    
    tracing_subscriber::fmt::init();




    let token = std::env::var("REVOLT_TOKEN").expect("token");


    // Build the client and start it
    // Handler is the Handler Struct
    // which implements the EventHandler trait
    // and acts as "Event Loop" for the client
    // to use the default public instance of revolt, pass None as second parameter
    // otherwise do Some("https://delta.revolt.chat") where delta.revolt.chat is your delta instance
    let mut client = Client::new(token)
    .event_handler(Handler)
    .set_api_url("https://api.revolt.chat")
    .await;


    client.start().await;






}


