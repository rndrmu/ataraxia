use reqwest::Client;
use std::sync::Arc;

use crate::models::{id::UserId, server::ServerConfig, user::User};

pub static API_BASE_URL: &str = "https://api.revolt.chat";

#[derive(Clone)]
pub struct Http {
    pub client: Arc<Client>,
    pub token: Option<String>,
}

impl Http {
    pub fn new_with_token<T: ToString>(token: T) -> Self {
        Http {
            client: Arc::new(Client::new()),
            token: Some(token.to_string()),
        }
    }

    pub fn new() -> Self {
        Http {
            client: Arc::new(Client::new()),
            token: None,
        }
    }

    pub async fn get_server_config(&self) -> Result<ServerConfig, reqwest::Error> {
        self.client
            .get(API_BASE_URL)
            .send()
            .await?
            .json::<ServerConfig>()
            .await
    }

    pub async fn get_user(&self, user: UserId) -> Result<User, reqwest::Error> {
        self.client
            .get(format!("{}/users/{}", API_BASE_URL, user.0))
            .header("x-bot-token", self.token.as_ref().unwrap())
            .send()
            .await?
            .json::<User>()
            .await
    }
}

impl Default for Http {
    fn default() -> Self {
        Self::new()
    }
}
