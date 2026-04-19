use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::http::{Http, API_BASE_URL};

use super::{
    channel::{Channel, DMChannel, Invite},
    message::{to_value, CreateMessage, Message},
    user::User,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserId(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageId(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelId(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerId(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmojiId(pub String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoleId(pub String);

// ── UserId ────────────────────────────────────────────────────────────────────

impl UserId {
    pub async fn get_user(&self, http: &Http) -> Result<User, reqwest::Error> {
        http.client
            .get(format!("{}/users/{}", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?
            .json::<User>()
            .await
    }

    /// Backwards-compatible alias for `get_user`.
    pub async fn get_author_user(&self, http: &Http) -> Result<User, reqwest::Error> {
        self.get_user(http).await
    }

    /// Open a DM channel with this user (or Saved Messages if it's yourself).
    pub async fn get_direct_message_channel(
        &self,
        http: &Http,
    ) -> Result<DMChannel, reqwest::Error> {
        http.client
            .get(format!("{}/users/{}/dm", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?
            .json::<DMChannel>()
            .await
    }
}

// ── ChannelId ─────────────────────────────────────────────────────────────────

impl ChannelId {
    pub async fn send_message<F>(&self, http: &Http, f: F) -> Result<Message, reqwest::Error>
    where
        F: FnOnce(&mut CreateMessage) -> &mut CreateMessage,
    {
        let mut msg = CreateMessage::default();
        f(&mut msg);
        let json = to_value(msg).unwrap();

        http.client
            .post(format!("{}/channels/{}/messages", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .json(&json)
            .send()
            .await?
            .json::<Message>()
            .await
    }

    pub async fn get_channel(&self, http: &Http) -> Result<Channel, reqwest::Error> {
        http.client
            .get(format!("{}/channels/{}", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?
            .json::<Channel>()
            .await
    }

    pub async fn delete_channel(&self, http: &Http) -> Result<(), reqwest::Error> {
        http.client
            .delete(format!("{}/channels/{}", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?;
        Ok(())
    }

    /// Create an invite to this channel.
    /// **Note:** requires a session token, not a bot token.
    pub async fn create_invite(&self, http: &Http) -> Result<Invite, reqwest::Error> {
        http.client
            .post(format!("{}/channels/{}/invites", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?
            .json::<Invite>()
            .await
    }

    /// Bulk-delete messages (requires ManageMessages, sent within the last week).
    pub async fn bulk_delete_messages(
        &self,
        http: &Http,
        message_ids: Vec<MessageId>,
    ) -> Result<bool, reqwest::Error> {
        let ids: Vec<&str> = message_ids.iter().map(|x| x.0.as_str()).collect();
        let res = http
            .client
            .delete(format!("{}/channels/{}/messages/bulk", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .json(&json!({ "ids": ids }))
            .send()
            .await?;
        Ok(res.status().is_success())
    }

    pub async fn get_messages(
        &self,
        http: &Http,
        limit: u8,
    ) -> Result<Vec<Message>, reqwest::Error> {
        http.client
            .get(format!("{}/channels/{}/messages", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .query(&[("limit", limit.to_string())])
            .send()
            .await?
            .json::<Vec<Message>>()
            .await
    }
}

// ── MessageId ─────────────────────────────────────────────────────────────────

impl MessageId {
    pub async fn get_message(&self, http: &Http) -> Result<Message, reqwest::Error> {
        http.client
            .get(format!("{}/messages/{}", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?
            .json::<Message>()
            .await
    }

    pub async fn delete_message(&self, http: &Http) -> Result<bool, reqwest::Error> {
        let res = http
            .client
            .delete(format!("{}/messages/{}", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?;
        Ok(res.status().is_success())
    }

    pub async fn remove_all_reactions(&self, http: &Http) -> Result<bool, reqwest::Error> {
        let res = http
            .client
            .delete(format!("{}/messages/{}/reactions", API_BASE_URL, self.0))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?;
        Ok(res.status().is_success())
    }

    pub async fn edit_message<F>(
        &self,
        http: &Http,
        channel_id: &ChannelId,
        f: F,
    ) -> Result<Message, reqwest::Error>
    where
        F: FnOnce(&mut CreateMessage) -> &mut CreateMessage,
    {
        let mut msg = CreateMessage::default();
        f(&mut msg);
        let json = to_value(msg).unwrap();

        http.client
            .patch(format!(
                "{}/channels/{}/messages/{}",
                API_BASE_URL, channel_id.0, self.0
            ))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .json(&json)
            .send()
            .await?
            .json::<Message>()
            .await
    }
}

// ── EmojiId ───────────────────────────────────────────────────────────────────

impl EmojiId {
    pub async fn add_reaction(
        &self,
        http: &Http,
        channel_id: &ChannelId,
        message_id: &MessageId,
    ) -> Result<bool, reqwest::Error> {
        let res = http
            .client
            .put(format!(
                "{}/channels/{}/messages/{}/reactions/{}",
                API_BASE_URL, channel_id.0, message_id.0, self.0
            ))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?;
        Ok(res.status().is_success())
    }

    pub async fn remove_reaction(
        &self,
        http: &Http,
        channel_id: &ChannelId,
        message_id: &MessageId,
    ) -> Result<bool, reqwest::Error> {
        let res = http
            .client
            .delete(format!(
                "{}/channels/{}/messages/{}/reactions/{}",
                API_BASE_URL, channel_id.0, message_id.0, self.0
            ))
            .header("x-bot-token", http.token.as_ref().unwrap())
            .send()
            .await?;
        Ok(res.status().is_success())
    }
}
