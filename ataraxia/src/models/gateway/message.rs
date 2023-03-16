use serde::{Deserialize, Serialize};
use crate::models::{id::*, message::{Embed, PartialMessage}};

use super::GatewayEvent;

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageUpdate {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    pub id: MessageId,
    pub channel: ChannelId,
    pub data: PartialMessage,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct MessageDelete {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    pub id: MessageId,
    pub channel: ChannelId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageReact {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    #[serde(rename = "emoji_id")]
    pub emoji: EmojiId,
    #[serde(rename = "id")]
    pub message_id: MessageId, 
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageUnreact {
    #[serde(rename = "type")]
    pub(crate) event_type: GatewayEvent,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    #[serde(rename = "emoji_id")]
    pub emoji: EmojiId,
    #[serde(rename = "id")]
    pub message_id: MessageId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageRemoveReactions {
    #[serde(rename = "type")]
    pub(crate) event_type: GatewayEvent,
    pub channel_id: ChannelId,
    #[serde(rename = "emoji_id")]
    pub emoji: EmojiId,
}
