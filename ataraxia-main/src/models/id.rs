use serde::{Deserialize, Serialize};

use crate::http::{Http, API_BASE_URL};

use super::{message::{to_value, CreateMessage, Message}, user::User, channel::{DMChannel, Channel, Invite}};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserId (
    pub String
);

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageId (
    pub String
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelId (
    pub String
);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerId (
    pub String
);

pub struct EmojiId (
    pub String
);

impl UserId {
    pub async fn get_author_user(&self, http: &Http) -> Result<User, reqwest::Error> {
        let url = format!("{}/users/{}", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?
        .json::<super::user::User>().await?;


        Ok(res)
    }

    pub async fn get_user(&self, http: &Http) -> Result<User, reqwest::Error> {
        let url = format!("{}/users/{}", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .send()
        .await?
        .json::<super::user::User>().await?;
        Ok(res)
    }

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

    pub async fn get_channel(&self, http: &Http) -> Result<Channel, reqwest::Error> {
        let url = format!("{}/channels/{}", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .send()
        .await?
        .json::<Channel>().await?;

        Ok(res)
    }

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
    /// # Only usable with a session token. Bot Tokens receive an error here.
    pub async fn create_invite(&self, http: &Http) -> Result<Invite, reqwest::Error> {

        let url = format!("{}/channels/{}/invites", API_BASE_URL, self.0);

        let res = http.client.post(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?
        .json::<Invite>().await?;

        Ok(res)
    }
}

impl MessageId {
    pub async fn get_message(&self, http: &Http) -> Result<Message, reqwest::Error> {
        let url = format!("{}/messages/{}", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?
        .json::<Message>().await?;

        Ok(res)
    }

    pub async fn delete_message(&self, http: &Http) -> Result<bool, reqwest::Error> {
        let url = format!("{}/messages/{}", API_BASE_URL, self.0);

        let res = http.client.delete(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?;

        Ok(res.status().is_success())
 
    }

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
    pub async fn add_reaction(&self, http: &Http, channel_id: &ChannelId, message_id: &MessageId) -> Result<bool, reqwest::Error> {
        let url = format!("{}/channels/{}/messages/{}/reactions/{}", API_BASE_URL, channel_id.0, message_id.0, self.0);

        let res = http.client.put(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?;

        Ok(res.status().is_success())
    }

    pub async fn remove_reaction(&self, http: &Http, channel_id: &ChannelId, message_id: &MessageId) -> Result<bool, reqwest::Error> {
        let url = format!("{}/channels/{}/messages/{}/reactions/{}", API_BASE_URL, channel_id.0, message_id.0, self.0);

        let res = http.client.delete(url)
        .header("x-bot-token", http.token.as_ref().unwrap())
        .send()
        .await?;

        Ok(res.status().is_success())
    }
}