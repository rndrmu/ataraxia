use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(rename = "revolt")]
    pub base_version: String,
    pub features: Features,
    #[serde(rename = "ws")]
    pub websocket_url: String,
    #[serde(rename = "app")]
    pub app_url: String,
    pub vapid: String,
    #[serde(default)]
    pub build: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    pub captcha: CaptchaConfig,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: AutumnConfig,
    pub january: JanuaryConfig,
    #[serde(default)]
    pub livekit: Option<LiveKitConfig>,
    #[serde(default)]
    pub limits: Option<serde_json::Value>,
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
pub struct LiveKitConfig {
    pub enabled: bool,
    pub nodes: Vec<LiveKitNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiveKitNode {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub public_url: String,
}
