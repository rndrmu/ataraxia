use serde::{Deserialize, Serialize};

use crate::http::{Http, API_BASE_URL};

use super::{message::{to_value, CreateMessage, Message}, user::User};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserId (
    pub String
);

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageId (
    pub String
);

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelId (
    pub String
);

impl UserId {
    pub async fn get_author_user(&self, http: Http) -> Result<User, reqwest::Error> {
        let url = format!("{}/users/{}", API_BASE_URL, self.0);

        let res = http.client.get(url)
        .header("x-bot-token", &http.token.unwrap())
        .send()
        .await?
        .json::<super::user::User>().await?;


        Ok(res)
    }
}

impl ChannelId {
    pub async fn send_message<F>(&self, http: Http, f: F) -> Result<Message, reqwest::Error>
    where
        F: FnOnce(&mut CreateMessage) -> &mut CreateMessage,
    {
        let mut message = CreateMessage::default();
        f(&mut message);

        let json = to_value(message).unwrap(); // this should never fail :^) 

        let url = format!("https://api.revolt.chat/channels/{}/messages", self.0);

        let res = http.client.post(&url)
            .header("x-bot-token", http.token.unwrap())
            .json(&json)
            .send()
            .await?
            .json::<Message>()
            .await?;

        Ok(res)

    }
}
