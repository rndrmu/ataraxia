use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub revolt: RevoltConfig,
    pub audio: AudioConfig,
    pub session: SessionConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RevoltConfig {
    pub api_url: String,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioConfig {
    pub ffmpeg_path: String,
    pub ytdlp_path: String,
    pub default_volume: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SessionConfig {
    pub resume_timeout_secs: u64,
    pub player_update_interval_secs: u64,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string("nightingale.toml")
            .unwrap_or_else(|_| DEFAULT_CONFIG.to_string());
        Ok(toml::from_str(&text)?)
    }
}

const DEFAULT_CONFIG: &str = r#"
[server]
host = "0.0.0.0"
port = 2333
password = "youshallnotpass"

[revolt]
api_url = "https://stoat.chat/api"
token = ""

[audio]
ffmpeg_path = "ffmpeg"
ytdlp_path = "yt-dlp"
default_volume = 100

[session]
resume_timeout_secs = 60
player_update_interval_secs = 5
"#;
