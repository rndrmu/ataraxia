use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::models::{id::UserId, user::{PartialUser, User}};
use super::GatewayEvent;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserUpdate {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub user_id: UserId,
    pub data: PartialUser,
    pub clear: Vec<UserCleared>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserCleared {
    ProfileContent,
    ProfileBackground,
    StatusText,
    Avatar,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRelationship {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    pub id: UserId,
    pub user: User,
    pub status: Value,
}
