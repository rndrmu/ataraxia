use crate::models::message::Message;
use crate::http::Http;
use futures_util::Future;
use serde_json::{json, Error};
use ataraxia_voice::vortex_socket::*;

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

    pub async fn reply(&self, message: &str) {
        let json: Result<Message, Error> = serde_json::from_value(self.json.clone());
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
                    // true here somehow leads to the server never sending a response back? 
                    "mention": false,
                }]
            }).to_string())
            .send()
            .await
            .unwrap();
        }
    }

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
            Err(e) => {println!("FAILED TO JOIN VOICE CALL -> {:?}", e); return Ok(())},
        };

        let mut voice_client = ataraxia_voice::vortex_socket::VoiceClient::new(vc.token, None).await;
        voice_client.init(channel).await;

        Ok(())

    }

    pub async fn kick_member(&self, server_id: &str, member_id: &str) {
        let res = reqwest::Client::new().delete(
            format!("https://api.revolt.chat/servers/{}/member/{}", server_id, member_id).as_str(),
        )
        .header("x-bot-token", &self.token)
        .send()
        .await
        .unwrap();
    }

    pub async fn ban_member(&self, server_id: &str, member_id: &str) {
        let res = reqwest::Client::new().post(
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

    pub async fn ban_with_reason(&self, server_id: &str, member_id: &str, reason: &str) {
        let res = reqwest::Client::new().post(
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



}





