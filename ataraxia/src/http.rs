use std::time::Duration;

use reqwest::Client;

use crate::models::{ServerConfig, user::User, message::UserId};

use std::sync::Arc;


pub static API_BASE_URL: &str = "https://api.revolt.chat";


#[derive(Clone)]
pub struct Http {
    pub client: Arc<Client>,
    pub token: String,
}

impl Http
{

    /// Instantiates a new Http Client
    /// 
    pub async fn new_with_token<T: ToString>(token: T) -> Self {
        let client = reqwest::Client::new();
        let token = token.to_string();

        Http {
            client: Arc::new(client),
            token,
        }
    }


    pub async fn get_server_config(&self) -> Result<ServerConfig, reqwest::Error> {
        let client = Client::new();
        let res = self.client.get(format!("{}", API_BASE_URL))
        .send()
        .await?
        .json::<ServerConfig>().await?;

        Ok(res)
    }

    pub async fn get_user(&self, user: UserId) -> Result<User, reqwest::Error> {
        let client = Client::new();
        let res = client.get(format!("{}/users/{}", API_BASE_URL, user.0))
        .header("x-bot-token", self.token.clone())
        .send()
        .await?
        .json::<User>().await?;

        Ok(res)
    }

}