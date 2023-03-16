use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EmojiCreate {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "id")]
    pub emoji: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmojiDelete {
    #[serde(rename = "type")]
    pub event_type: super::GatewayEvent,
    #[serde(rename = "id")]
    pub emoji: crate::models::id::EmojiId,
}