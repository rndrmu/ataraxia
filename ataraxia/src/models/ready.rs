use std::collections::HashMap;


use serde::{Serialize, Deserialize};
use super::{channel::{Channel, ChannelIconMetadata, ChannelType, ChannelIcon, ChannelDefaultPermissions}, member::{Member}, server::{Server, SystemMessages}};

/// The Payload, received from the READY event
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ready {
    pub channels: Vec<ReadyChannels>,
    pub members: Vec<ReadyMembers>,
    pub servers: Vec<ReadyServers>,
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
    pub owner: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadyMembers {
    #[serde(rename = "_id")]
    pub id_strings: ReadyMembersIdentifiers,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadyMembersIdentifiers {
    #[serde(rename = "server")]
    pub server_id: String,
    #[serde(rename = "user")]
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadyServers {
    #[serde(rename = "_id")]
    pub server_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<ChannelCategory>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_permissions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<ServerIcon>,
    pub name: String,
    #[serde(rename = "owner")]
    pub owner_id: String,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<Vec<SystemMessages>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelCategory {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<String>>,
    #[serde(rename = "id")]
    pub category_id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerIcon {
    #[serde(rename = "_id")]
    pub icon_id: String,
    pub content_type: String,
    pub filename: String,
    pub metadata: ServerIconMetadata,
    pub size: u32,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerIconMetadata {
    pub height: u32,
    pub width: u32,
    #[serde(rename = "type")]
    pub icon_type: String,
}