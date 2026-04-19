use serde_json::{json, Error};
use tracing::error;

use crate::{
    http::{Http, API_BASE_URL},
    models::{
        channel::Channel,
        message::{to_value, CreateMessage, Message},
    },
};

#[derive(Clone)]
pub struct Context {
    pub token: String,
    pub http: Http,
    pub json: serde_json::Value,
}

#[derive(serde::Deserialize)]
pub struct VoiceChannel {
    pub token: String,
    pub url: String,
}

impl Context {
    pub fn new(token: &str, json: &str) -> Context {
        Context {
            token: token.to_owned(),
            http: Http::new_with_token(token),
            json: serde_json::from_str(json).unwrap(),
        }
    }

    /// Reply to the message currently in context.
    pub async fn reply(&self, message: &str) {
        let json: Result<Message, Error> = serde_json::from_value(self.json.clone());
        if let Ok(json) = json {
            reqwest::Client::new()
                .post(
                    format!(
                        "{API_BASE_URL}/channels/{}/messages",
                        json.channel_id.0
                    )
                    .as_str(),
                )
                .header("x-bot-token", self.token.clone())
                .header("content-type", "application/json")
                .body(
                    json!({
                        "content": message,
                        "replies": [{ "id": json.id.0, "mention": false }],
                    })
                    .to_string(),
                )
                .send()
                .await
                .unwrap();
        }
    }

    /// Send a message to any channel using the builder API.
    pub async fn reply_builder<F>(&self, channel_id: &str, f: F)
    where
        F: FnOnce(&mut CreateMessage) -> &mut CreateMessage,
    {
        let mut builder = CreateMessage::default();
        f(&mut builder);

        reqwest::Client::new()
            .post(
                format!("{API_BASE_URL}/channels/{}/messages", channel_id).as_str(),
            )
            .header("x-bot-token", self.token.clone())
            .header("content-type", "application/json")
            .body(to_value(builder).unwrap().to_string())
            .send()
            .await
            .unwrap();
    }

    /// Join a voice channel and return the `VoiceChannel` token.
    ///
    /// Pass the returned token to `ataraxia_voice::VoiceConnection::connect`.
    pub async fn join_voice_channel(
        &self,
        channel: &str,
    ) -> Result<VoiceChannel, serde_json::Error> {
        // Fetch the node name from the server config so the join_call body is valid.
        let node = self.get_livekit_node().await;

        let res = reqwest::Client::new()
            .post(
                format!("{API_BASE_URL}/channels/{}/join_call", channel).as_str(),
            )
            .header("x-bot-token", self.token.clone())
            .header("content-type", "application/json")
            .body(json!({ "force_disconnect": false, "node": node }).to_string())
            .send()
            .await
            .unwrap();

        let text = res.text().await.unwrap();
        match serde_json::from_str::<VoiceChannel>(&text) {
            Ok(vc) => Ok(vc),
            Err(e) => {
                error!("Failed to parse join_call response (raw: {}): {:?}", text, e);
                Err(e)
            }
        }
    }

    async fn get_livekit_node(&self) -> String {
        use crate::models::server::ServerConfig;
        let res = reqwest::Client::new()
            .get(API_BASE_URL)
            .send()
            .await;
        if let Ok(r) = res {
            if let Ok(cfg) = r.json::<ServerConfig>().await {
                if let Some(lk) = cfg.features.livekit {
                    if let Some(node) = lk.nodes.into_iter().next() {
                        return node.name;
                    }
                }
            }
        }
        "worldwide".to_string()
    }

    /// Kick a member from a server.
    pub async fn kick_member(&self, server_id: &str, member_id: &str) {
        reqwest::Client::new()
            .delete(
                format!(
                    "{API_BASE_URL}/servers/{}/members/{}",
                    server_id, member_id
                )
                .as_str(),
            )
            .header("x-bot-token", &self.token)
            .send()
            .await
            .unwrap();
    }

    /// Ban a member from a server.
    pub async fn ban_member(&self, server_id: &str, member_id: &str) {
        self.ban_with_reason(server_id, member_id, "").await;
    }

    /// Ban a member from a server with a reason.
    pub async fn ban_with_reason(&self, server_id: &str, member_id: &str, reason: &str) {
        reqwest::Client::new()
            .put(
                format!(
                    "{API_BASE_URL}/servers/{}/bans/{}",
                    server_id, member_id
                )
                .as_str(),
            )
            .header("x-bot-token", &self.token)
            .header("content-type", "application/json")
            .body(json!({ "reason": reason }).to_string())
            .send()
            .await
            .unwrap();
    }

    /// Fetch a channel by ID.
    pub async fn get_channel(&self, channel_id: &str) -> Result<Channel, serde_json::Error> {
        let res = reqwest::Client::new()
            .get(format!("{API_BASE_URL}/channels/{}", channel_id).as_str())
            .header("x-bot-token", &self.token)
            .send()
            .await
            .unwrap();

        let text = res.text().await.unwrap();
        match serde_json::from_str::<Channel>(&text) {
            Ok(ch) => Ok(ch),
            Err(e) => {
                error!("Failed to parse channel response: {:?}", e);
                Err(e)
            }
        }
    }
}
