use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::http::{Http, API_BASE_URL};

use super::{message::{to_value, CreateMessage, Message}, user::User, channel::{DMChannel, Channel, Invite}};

/// An Identifier for a User.
#[derive(Serialize, Deserialize, Debug)]
pub struct UserId (
    pub String
);

/// An Identifier for a Message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageId (
    pub String
);

/// An Identifier for a Channel.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelId (
    pub String
);

/// An Identifier for a Server,
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerId (
    pub String
);

/// An Identifier for an Emoji.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmojiId (
    pub String
);

impl UserId {

    /// Fetch a User from the API.
    /// 
    /// 
    pub async fn get_user(&self, http: &Http) -> Result<User, reqwest::Error> {
        let url = format!("{}/users/{}", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .send()
        .await?
        .json::<super::user::User>().await?;
        Ok(res)
    }

    /// Open a DM with a User, or, if the target is oneself, a "Saved Messages" channel.
    pub async fn get_direct_message_channel(&self, http: &Http) -> Result<DMChannel, reqwest::Error> {
        let url = format!("{}/users/{}/dm", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?
        .json::<DMChannel>().await?;

        Ok(res)
    }
}

impl ChannelId {

    /// Send a message to a Channel.
    pub async fn send_message<F>(&self, http: &Http, f: F) -> Result<Message, reqwest::Error>
    where
        F: FnOnce(&mut CreateMessage) -> &mut CreateMessage,
    {
        let mut message = CreateMessage::default();
        f(&mut message);

        let json = to_value(message).unwrap(); // this should never fail :^) 

        let url = format!("https://api.revolt.chat/channels/{}/messages", self.0);

        let res = http.client.post(&url)
            .header("x-bot-token", http.token.as_ref().unwrap())
            .json(&json)
            .send()
            .await?
            .json::<Message>()
            .await?;

        Ok(res)

    }

    /// Get a Channel from a specified Identifier.
    pub async fn get_channel(&self, http: &Http) -> Result<Channel, reqwest::Error> {
        let url = format!("{}/channels/{}", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .send()
        .await?
        .json::<Channel>().await?;

        Ok(res)
    }

    /// Delete a specified Channel.
    pub async fn delete_channel(&self, http: &Http) -> Result<(), reqwest::Error> {
        let url = format!("{}/channels/{}", API_BASE_URL, self.0);

        let res = http.client.delete(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?
        .json::<()>().await?;

        Ok(res)
    }

    /// Creates an invite to this channel.
    /// 
    /// ### Only usable with a session token. Bot Tokens receive an error here.
    pub async fn create_invite(&self, http: &Http) -> Result<Invite, reqwest::Error> {

        let url = format!("{}/channels/{}/invites", API_BASE_URL, self.0);

        let res = http.client.post(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?
        .json::<Invite>().await?;

        Ok(res)
    }


    /// Bulk delete messages from a channel.
    /// 
    /// Delete multiple messages you've sent or one you have permission to delete.
    /// 
    /// 
    /// Note:
    /// 
    /// This will always require 'ManageMessages' permission regardless of whether you own the message or not.
    /// Messages must have been sent within the past 1 week.
    /// 
    pub async fn bulk_delete_messages(&self, http: &Http, message_ids: Vec<MessageId>) -> Result<bool, reqwest::Error> {
        let url = format!("{}/channels/{}/messages/bulk", API_BASE_URL, self.0);

        let res = http.client.delete(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .json(&json!({
            "ids": message_ids.iter().map(|x| x.0.clone()).collect::<Vec<String>>()
        }));

        println!("{:?}", res);
        let b = res.send().await?;

        Ok(b.status().is_success())
    }

    /// Get multiple Messages from a channel
    pub async fn get_messages(&self, http: &Http, limit: u8) -> Result<Vec<Message>, reqwest::Error> {
        let url = format!("{}/channels/{}/messages", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .query(&[("limit", limit.to_string())])
        .send()
        .await?
        .json::<Vec<Message>>().await?;

        Ok(res)
    }
}

impl MessageId {

    /// Get a Message from a specified Identifier.
    pub async fn get_message(&self, http: &Http) -> Result<Message, reqwest::Error> {
        let url = format!("{}/messages/{}", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?
        .json::<Message>().await?;

        Ok(res)
    }

    /// Delete a specified Message.
    /// 
    /// If the message is not yours, you will need the "Manage Messages" permission.
    pub async fn delete_message(&self, http: &Http) -> Result<bool, reqwest::Error> {
        let url = format!("{}/messages/{}", API_BASE_URL, self.0);

        let res = http.client.delete(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?;

        Ok(res.status().is_success())
 
    }

    /// Remove all reactions from a specified Message.
    pub async fn remove_all_reactions(&self, http: &Http) -> Result<bool, reqwest::Error> {
        let url = format!("{}/messages/{}/reactions", API_BASE_URL, self.0);

        let res = http.client.delete(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?;

        Ok(res.status().is_success())
    }
    
}

impl EmojiId {

    /// Add a reaction to a Message.
    pub async fn add_reaction(&self, http: &Http, channel_id: &ChannelId, message_id: &MessageId) -> Result<bool, reqwest::Error> {
        let url = format!("{}/channels/{}/messages/{}/reactions/{}", API_BASE_URL, channel_id.0, message_id.0, self.0);

        let res = http.client.put(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?;

        Ok(res.status().is_success())
    }

    /// Remove a reaction from a Message.
    pub async fn remove_reaction(&self, http: &Http, channel_id: &ChannelId, message_id: &MessageId) -> Result<bool, reqwest::Error> {
        let url = format!("{}/channels/{}/messages/{}/reactions/{}", API_BASE_URL, channel_id.0, message_id.0, self.0);

        let res = http.client.delete(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?;

        Ok(res.status().is_success())
    }
}