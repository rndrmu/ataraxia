use serde::{Deserialize, Serialize};
use serde_json::*;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum ChannelType {
    TextChannel,
    VoiceChannel,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Channel {
    pub channel_type: ChannelType,
    #[serde(rename = "_id")]
    pub channel_id : String,
    pub server: String,
    pub name: String,
    pub description: String,
    pub icon: ChannelIcon
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
    pub deleted: bool,
    pub reported: bool,
    pub message_id: String,
    pub user_id: String,
    pub server_id: String,
    pub object_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelIconMetadata {
    #[serde(rename = "type")]
    pub file_type: String,
}