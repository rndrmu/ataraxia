use ataraxia::{
    command::CommandFramework,
    context::Context,
    models::message::Message,
    websocket::Client,
};

// Each #[command] generates a `pub static <NAME>_COMMAND: Command`.
// The function receives the full Context and the triggering Message.

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

#[ataraxia::command(name = "info", description = "Show bot info")]
async fn info(ctx: Context, _msg: Message) {
    ctx.reply("I'm a bot built with **ataraxia** 🦀").await;
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = std::env::var("REVOLT_TOKEN").expect("REVOLT_TOKEN not set");

    let framework = CommandFramework::new("!")
        .command(&PING_COMMAND)
        .command(&HELLO_COMMAND)
        .command(&INFO_COMMAND);

    let mut client = Client::new(token)
        .event_handler(framework)
        .set_api_url("https://stoat.chat/api")
        .await;

    client.start().await;
}
