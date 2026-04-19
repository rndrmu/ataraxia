use serde::{Deserialize, Serialize};
use crate::models::id::EmojiId;
use super::GatewayEvent;

#[derive(Serialize, Deserialize, Debug)]
pub struct EmojiCreate {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    pub emoji: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmojiDelete {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub emoji_id: EmojiId,
}
