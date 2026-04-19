use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::models::id::*;
use super::GatewayEvent;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerUpdate {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub server_id: ServerId,
    pub data: Value,
    pub clear: Option<Vec<ServerClear>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerClear { Icon, Banner, Description }

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerDelete {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub server_id: ServerId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerMemberUpdate {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub ids: ServerMemberIds,
    pub data: Value,
    pub clear: Option<Vec<ServerMemberCleared>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerMemberIds {
    pub server: ServerId,
    pub user: UserId,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMemberCleared { Nickname, Avatar }

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerMemberJoin {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub server_id: ServerId,
    #[serde(rename = "user")]
    pub user_id: UserId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerMemberLeave {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub server_id: ServerId,
    #[serde(rename = "user")]
    pub user_id: UserId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerRoleUpdate {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub server_id: ServerId,
    pub role_id: RoleId,
    pub data: Value,
    pub clear: Option<Vec<ServerRoleCleared>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerRoleCleared { Color }

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerRoleDelete {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub server_id: ServerId,
    pub role_id: RoleId,
}
