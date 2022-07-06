use super::channel::{ChannelDefaultPermissions, ChannelIcon, ChannelIconMetadata, ChannelType};
use serde::{Deserialize, Serialize};

/// The Payload, received from the READY event
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ready {
    pub channels: Vec<serde_json::Value>, // we do
    pub members: Vec<serde_json::Value>,  // a little
    // TODO: Actually model this,
    // we need all those for caching
    // Plus, serde_json::Value is an expensive operation
    pub servers: Vec<serde_json::Value>, // trolling
    #[serde(rename = "type")]
    pub event_type: String,
    pub users: Vec<ReadyUsers>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadyChannels {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_permissions: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadyUsers {
    #[serde(rename = "_id")]
    pub user_id: String,
    pub avatar: UserAvatar,
    pub badges: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<Bot>,
    pub online: bool,
    pub relationship: String,
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
