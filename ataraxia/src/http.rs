use reqwest::Client;

use crate::models::{user::User, id::UserId};
use super::models::delta::ServerConfig;

use std::sync::Arc;


pub static API_BASE_URL: &str = "https://api.revolt.chat";


/// Reusable Reqwest Client, used for all requests.
#[derive(Clone)]
pub struct Http {
    /// The actual reqwest Client.
    pub client: Arc<Client>,
    /// The bot's token or your session Token.
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

    /// Gets the server config.
    pub async fn get_server_config(&self) -> Result<ServerConfig, reqwest::Error> {
        let res = self.client.get(format!("{}", API_BASE_URL))
        .send()
        .await?
        .json::<ServerConfig>().await?;

        Ok(res)
    }

    /// Raw method to get a User from the API.
    /// 
    ///  Can be used to get the Authorized User ig.
    pub async fn get_user(&self, user: UserId) -> Result<User, reqwest::Error> {
        let res = self.client.get(format!("{}/users/{}", API_BASE_URL, user.0))
        .header("x-bot-token", &self.token.clone().unwrap())
        .send()
        .await?
        .json::<User>().await?;

        Ok(res)
    }

}