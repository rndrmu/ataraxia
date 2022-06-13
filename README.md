# ataraxia
A minimal barebones API Wrapper for the [revolt.chat API](https://revolt.chat)

## A basic Ping-Pong Bot looks like

```rs 
use ataraxia::{
    websocket::{Client, EventHandler},
    models::message::Message as RevoltMessage,
    context::Context,
    async_trait
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn authenticated(&self) {
        println!("Authenticated!");
    }
    async fn ready(&self, _ctx: Context) {
        println!("Ready!");
    }
    async fn on_message(&self, ctx: Context, message: RevoltMessage) {
        println!("{}", message);

        if message.content == "!ping" {


            ctx.reply_builder(&message.channel_id, |r| {
                r.content("Pong!")
            }).await

    }
}


#[tokio::main]
async fn main() {

    let token = std::env::var("TOKEN").expect("token");
     let mut client = Client::new(token)
        .event_handler(Handler)
        .set_api_url("https://api.revolt.chat");

    client.start().await;

}

```