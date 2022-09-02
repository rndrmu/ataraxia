use serde::{Deserialize, Serialize};

use crate::models::{id::*, channel::PartialChannel};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelStartTyping {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    #[serde(rename = "user")]
    pub user_id: UserId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelStopTyping {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    #[serde(rename = "user")]
    pub user_id: UserId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelCreate {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "id")]
    pub channel_id: ChannelId,

}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelUpdate {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    pub data: PartialChannel,
    pub clear: ChannelUpdateCleared,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ChannelUpdateCleared {
    Icon,
    Description
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelDelete {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelGroupJoin {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    #[serde(rename = "user")]
    pub user_id: UserId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelGroupLeave {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    #[serde(rename = "user")]
    pub user_id: UserId,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelAck {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "user")]
    pub user_id: ChannelId,
    #[serde(rename = "message_id")]
    pub message_id: MessageId,
}