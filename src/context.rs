use std::sync::Arc;

use crate::models::message::Message;
use crate::http::Http;
use serde_json::{json, Error};


#[derive(Clone)]
pub struct Context {
    pub token: String,
    pub http: Http,
    pub json: serde_json::Value,
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
        println!("JSON is okay?: {}", json.is_ok());
        if let Ok(json) = json {
            println!("Channel Id is -> {}", json.channel);
           /*  let _ = Http::post(
                format!("https://api.revolt.chat/channels/{}/messages", json.channel).as_str(),
                &self.token,
                json!({
                        "content": message,
                        "nonce": std::time::SystemTime::UNIX_EPOCH,
                        /* "replies": [{
                            "id": json._id,
                            "mention": true,
                        }] */
                    }),
                ).await.unwrap(); */

            reqwest::Client::new().post(
                format!("https://api.revolt.chat/channels/{}/messages", json.channel).as_str(),
            )
            .header("x-bot-token", self.token.clone())
            .header("content-type", "application/json")
            .body(json!({
                "content": message,
                "replies": [{
                    "id": json._id,
                    "mention": true,
                }]
            }).to_string())
            .send()
            .await
            .unwrap();

            // Fuck you revolt 
        }
    }


}