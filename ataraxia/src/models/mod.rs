use serde::{Deserialize, Serialize};

pub mod message;
pub mod channel;
pub mod ready;
pub mod member;
pub mod server;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(rename = "revolt")]
    pub base_version: String,
    pub features: Features,
    #[serde(rename = "ws")]
    pub websocket_url: String,
    #[serde(rename = "app")]
    pub app_url: String,
    pub vapid: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    pub captcha: CaptchaConfig,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: AutumnConfig,
    pub january: JanuaryConfig,
    pub voso: VortexConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaConfig {
    pub enabled: bool,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutumnConfig {
    pub enabled: bool,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JanuaryConfig {
    pub enabled: bool,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VortexConfig {
    pub enabled: bool,
    pub url: String,
    #[serde(rename = "ws")]
    pub ws_url: String,
}