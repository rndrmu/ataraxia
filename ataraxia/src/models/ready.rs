use super::channel::{ChannelDefaultPermissions, ChannelIcon, ChannelIconMetadata, ChannelType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ready {
    pub channels: Vec<serde_json::Value>,
    pub members: Vec<serde_json::Value>,
    pub servers: Vec<serde_json::Value>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub users: Vec<ReadyUser>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadyChannel {
    #[serde(rename = "_id")]
    pub channel_id: String,
    pub channel_type: ChannelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<String>,
    pub name: String,
    pub server: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_permissions: Option<ChannelDefaultPermissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<ChannelIcon>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadyUser {
    #[serde(rename = "_id")]
    pub user_id: String,
    pub avatar: Option<UserAvatar>,
    pub badges: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<Bot>,
    pub online: bool,
    pub relationship: Option<String>,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAvatar {
    #[serde(rename = "_id")]
    pub avatar_id: String,
    pub content_type: String,
    pub filename: String,
    pub metadata: ChannelIconMetadata,
    pub size: i32,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bot {
    pub owner: String,
}
