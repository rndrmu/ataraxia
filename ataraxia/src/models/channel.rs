use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::id::ChannelId;


#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum ChannelType {
    TextChannel,
    VoiceChannel,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Channel {
    pub channel_type: ChannelType,
    #[serde(rename = "_id")]
    pub channel_id: ChannelId,
    pub server: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<ChannelIcon>,
    pub default_permissions: Option<ChannelDefaultPermissions>,
    pub last_message_id: Option<String>,
    pub nsfw: Option<bool>,
    #[serde(flatten)]
    pub role_permissions: Option<HashMap<String, ChannelDefaultPermissions>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelIcon {
    #[serde(rename = "_id")]
    pub icon_id: String,
    pub tag: String,
    pub filename: String,
    pub metadata: ChannelIconMetadata,
    pub content_type: String,
    pub size: i32,
    pub deleted: Option<bool>,
    pub reported: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelIconMetadata {
    #[serde(rename = "type")]
    pub file_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelDefaultPermissions {
    #[serde(rename = "a")]
    pub allow: i32,
    #[serde(rename = "d")]
    pub deny: i32,
}

