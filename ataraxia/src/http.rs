use reqwest::Client;

use crate::models::{user::User, message::UserId};
use super::models::server::ServerConfig;

use std::sync::Arc;


pub static API_BASE_URL: &str = "https://api.revolt.chat";


#[derive(Clone)]
pub struct Http {
    pub client: Arc<Client>,
    pub token: Option<String>,
}

impl Http
{

    /// Instantiates a new Http Client with a given token.
    /// 
    pub fn new_with_token<T: ToString>(token: T) -> Self {
        let client = reqwest::Client::new();
        let token = token.to_string();

        Http {
            client: Arc::new(client),
            token: Some(token),
        }
    }

    /// Instantiates a new Http Client without a token.
    /// 
    /// Useful if you want to get the server config.
    pub fn new() -> Self {
        let client = reqwest::Client::new();

        Http {
            client: Arc::new(client),
            token: None,
        }
    }

    pub async fn get_server_config(&self) -> Result<ServerConfig, reqwest::Error> {
        let res = self.client.get(format!("{}", API_BASE_URL))
        .send()
        .await?
        .json::<ServerConfig>().await?;

        Ok(res)
    }

    pub async fn get_user(&self, user: UserId) -> Result<User, reqwest::Error> {
        let client = Client::new();
        let res = client.get(format!("{}/users/{}", API_BASE_URL, user.0))
        .header("x-bot-token", &self.token.clone().unwrap())
        .send()
        .await?
        .json::<User>().await?;

        Ok(res)
    }

}