use crate::models::message::{Message, CreateMessage, hashmap_to_json_map, to_value};
use crate::http::Http;

use serde_json::{json, Error};

#[cfg(feature = "voice")]
use ataraxia_voice::vortex_socket::*;

use tracing::{error};
use super::models::channel::Channel as RevoltChannel;

/// Helpful Struct relating to the current execution context
/// 
/// Also contains a lot of very useful functions 
/// 
/// ## Examples 
/// 
/// ### To Join a Voice Channel
/// ```no_run
/// use ataraxia::context::Context;
/// 
/// // You won't need to Instantiate the Context, it will be created for you
/// // When your client receives an event
/// let ctx = Context::new("token", "{\"some\": \"json\"}");
/// let vc = ctx.join_voice_channel("channel_id");
/// ```
/// 
/// ### To Send a Message
/// ```no_run
/// use ataraxia::context::Context;
/// let ctx = Context::new("token", "{\"some\": \"json\"}");
/// ctx.reply("Hello World!");
/// ```
/// 
/// 
/// 
#[derive(Clone)]
pub struct Context {
    pub token: String,
    pub http: Http,
    pub json: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct VoiceChannel {
    pub token: String,
}



impl Context {
    pub fn new(
        token: &str,
        json: &str,
    ) -> Context
    {
        Context  {
            token: token.to_owned(),
            http: Http,
            json: serde_json::from_str(json).unwrap(),
        }
    }

    /// Replies to the current message that is in context
    pub async fn reply(&self, message: &str) {
        let json: Result<Message, Error> = serde_json::from_value(self.json.clone());

        // If should masquerade, do 

        if let Ok(json) = json {

            

            reqwest::Client::new().post(
                format!("https://api.revolt.chat/channels/{}/messages", json.channel).as_str(),
            )
            .header("x-bot-token", self.token.clone())
            .header("content-type", "application/json")
            .body(json!({
                "content": message,
                "replies": [{
                    "id": json._id,
                    // setting mention to true here somehow leads to the server never sending a response back? 
                    "mention": false,
                }],
            }).to_string())
            .send()
            .await
            .unwrap();
        }
    }

    pub async fn reply_builder<F>(&self, channel_id: &str, f: F)
    where F: FnOnce(&mut CreateMessage) -> &mut CreateMessage {

        // get result from builder
        let mut builder = CreateMessage::default();
        f(&mut builder);



        //let json: Result<CreateMessage, Error> = serde_json::from_value(to_value(builder.0).unwrap());

        println!("hello world");


            reqwest::Client::new().post(
                format!("https://api.revolt.chat/channels/{}/messages", channel_id).as_str(),
            )
            .header("x-bot-token", self.token.clone())
            .header("content-type", "application/json")
            .body(to_value(builder).unwrap().to_string())
            .send()
            .await
            .unwrap();
        
    }

    /// Joins the specified voice channel
    pub async fn join_voice_channel(&self, channel: &str) -> Result<(), serde_json::Error> {
        let res = reqwest::Client::new().post(
            format!("https://api.revolt.chat/channels/{}/join_call", channel).as_str(),
        )
        .header("x-bot-token", self.token.clone())
        .header("content-type", "application/json")
        .send()
        .await
        .unwrap();

        // get result 
        let json: Result<VoiceChannel, Error> = serde_json::from_str(res.text().await.unwrap().as_str());

        let vc = match json {
            Ok(json) => json,
            Err(e) => {error!("FAILED TO JOIN VOICE CALL -> {:?}", e); return Ok(())},
        };

        let mut voice_client = ataraxia_voice::vortex_socket::VoiceClient::new(vc.token, None).await;
        voice_client.init(channel).await;

        Ok(())

    }

    /// Kick a user from the Server 
    pub async fn kick_member(&self, server_id: &str, member_id: &str) {
        let _res = reqwest::Client::new().delete(
            format!("https://api.revolt.chat/servers/{}/member/{}", server_id, member_id).as_str(),
        )
        .header("x-bot-token", &self.token)
        .send()
        .await
        .unwrap();
    }

    /// Ban a member from the server
    pub async fn ban_member(&self, server_id: &str, member_id: &str) {
        let _res = reqwest::Client::new().post(
            format!("https://api.revolt.chat/servers/{}/bans/{}", server_id, member_id).as_str(),
        )
        .header("x-bot-token", &self.token)
        .header("content-type", "application/json")
        .body(json!({
            "reason": "null",
        }).to_string())
        .send()
        .await
        .unwrap();
    }

    /// Ban a User from the Server with a Reason
    pub async fn ban_with_reason(&self, server_id: &str, member_id: &str, reason: &str) {
        let _res = reqwest::Client::new().post(
            format!("https://api.revolt.chat/servers/{}/bans/{}", server_id, member_id).as_str(),
        )
        .header("x-bot-token", &self.token)
        .header("content-type", "application/json")
        .body(json!({
            "reason": reason,
        }).to_string())
        .send()
        .await
        .unwrap();
    }

    pub async fn get_channel(&self, channel_id: &str) -> Result<RevoltChannel, serde_json::Error> {
        let res = reqwest::Client::new().get(
            format!("https://api.revolt.chat/channels/{}", channel_id).as_str(),
        )
        .header("x-bot-token", &self.token)
        .send()
        .await
        .unwrap();

        let json: Result<RevoltChannel, Error> = serde_json::from_str(res.text().await.unwrap().as_str());

        match json {
            Ok(json) => Ok(json),
            Err(e) => {error!("FAILED TO GET CHANNEL -> {:?}", e); return Err(e)},
        }
    }





}





