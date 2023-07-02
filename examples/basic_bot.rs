

use ataraxia::{
    client::{Client, EventHandler},
    models::{
        message::Message,
        ready::Ready,
        id::EmojiId,
        gateway::{message::{
            MessageDelete,
            MessageUpdate,
            MessageUnreact,
            MessageReact, MessageRemoveReactions
        }},
    },
    context::Context,
    async_trait,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /// Function called when the client is authenticated
    async fn authenticated(&self) {
        println!("Authenticated!");
    }
    /// Function called when the client is ready to receive events
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Ready!");
        println!("{:?} is connected!", ready.users.iter().map(|u| &u.username).collect::<Vec<_>>());
    }

    /// Function called when a message is received, you can reply to the message with the `ctx.reply` function
    /// 
    /// ## How to use Arguments
    /// To use arguments you need to somehow split the message into a command and the arguments
    /// See the `!join` command for an example
    async fn on_message(&self, ctx: Context, message: Message) {
        println!("{}", message);

        if message.content == "!ping" {

            let msg = message.channel_id.send_message(&ctx.http, |r| {
                r.content("hello!")
                .set_masquerade(|masquerade| {
                    masquerade.name("Rainer Winkler").avatar("https://i.imgflip.com/6bnywv.jpg")
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
                    .icon_url("https://i.imgflip.com/6bnywv.jpg")
                })
                .create_embed(|e| {
                    e.title("Test")
                })
            }).await.map_err(|e| println!("{}", e));

            println!("Sent Message with Content '{:?}' Successfully!", msg);

            // sleep 5s 
            std::thread::sleep(std::time::Duration::from_secs(5));

            match msg {
                Ok(msg) => {
                   let edited = msg.edit(&ctx.http, |f| {
                        f.content("hello again!")
                        .create_embed(|f| {
                            f.title(":trol:")
                            .url("https://i.redd.it/ztfffav639991.jpg")
                        })
                   }).await;
                   println!("Edited Message with Content '{:?}' Successfully!", edited);
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }


        } else if message.content.starts_with("!join") {


            let voice_channel_id = message.content.split(' ').collect::<Vec<&str>>()[1];
            println!("Joining voice channel {}", voice_channel_id);
            ctx.reply("Okay, i joined the channel!").await;


        } else if message.content == "!channelinfo" {
            let chn = ctx.get_channel(&message.channel_id.0).await.unwrap();
            println!("{:?}", chn);
        } else if message.content == "!me" {
            let user = message.author.get_user(&ctx.http).await.unwrap();

            ctx.reply(&format!("{:?}", user)).await;

            println!("{:?}", user);
        } else if message.content == "!react" {
            let reaction_result = EmojiId("🇪🇺".to_string()).add_reaction(&ctx.http, &message.channel_id, &message.id).await;
            println!("{:?}", reaction_result);
        } else if message.content == "!dm" {
            let dm_channel = message.author.get_direct_message_channel(&ctx.http).await.unwrap();
            let _dmresult = dm_channel.channel_id.send_message(&ctx.http, |r| {
                r.content(":trol:")
            }).await;
        } else if message.content == "!massdelete" {
            let amt_to_delete = 10;
            let mess = message.channel_id.get_messages(&ctx.http, amt_to_delete).await.unwrap();
            let msg_ids = mess.iter().map(|m| m.id.clone()).collect::<Vec<_>>();
            println!("{:?}", mess);
            let del_res = message.channel_id.bulk_delete_messages(&ctx.http, msg_ids).await;
            message.channel_id.send_message(&ctx.http, |r| {
                r.content(format!("Deleted {} messages -> Result: {:?}", amt_to_delete, del_res))
            }).await.unwrap();
        }
    }

    async fn message_delete(&self, _ctx: Context, _message: MessageDelete) {
        println!("Message Deleted in {:?}/{:?}", _message.channel, _message.id);
    }

    async fn message_update(&self, _ctx: Context, _message: MessageUpdate) {
        println!("Message Updated!");
    }

    async fn message_unreact(&self, _ctx: Context, _message: MessageUnreact) {
        println!("Message Unreacted!");
    }

    async fn message_react(&self, _ctx: Context, _message: MessageReact) {
        println!("{:?} reacted in channel {:?} with {:?}", _message.user_id, _message.message_id ,_message.emoji);
    }

    async fn message_remove_reactions(&self, _context: Context, _reaction: MessageRemoveReactions) {}
    

}




#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    std::env::set_var("RUST_LOG", "debug"); // noisy logging 
    
    tracing_subscriber::fmt::init();

    /// Get the token from the environment
    let token = std::env::var("REVOLT_TOKEN").expect("token");


    

    // Build the client and start it
    // Handler is the Handler Struct
    // which implements the EventHandler trait
    // and acts as "Event Loop" for the client
    // to use the default public instance of revolt, simply don't call `set_api_url` 
    // or call it with `https://api.revolt.chat` as the argument

    let mut client = Client::new(token)
    .event_handler(Handler)
    .set_api_url("https://api.revolt.chat")
    .await;


    client.start().await;






}

// Helper type alias for commands
type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>; 

// A command function looks like
// async fn command_name(ctx: Context, message: Message) -> CommandResult { ... }

#[ataraxia::command]
async fn ping(ctx: Context, _message: Message) -> CommandResult {
    ctx.reply("Pong!").await;
    Ok(())
}