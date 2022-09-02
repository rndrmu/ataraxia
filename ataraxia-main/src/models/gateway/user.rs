use serde::{Deserialize, Serialize};
use crate::models::{id::*, user::{User, PartialUser}};
use super::GatewayEvent;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserUpdate {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub user_id: UserId,
    pub data: PartialUser,
    pub clear: Vec<Cleared>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Cleared {
    ProfileContent,
    ProfileBackground,
    StatusText,
    Avatar
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRelationship {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub user: User,
    pub status: Value,
}