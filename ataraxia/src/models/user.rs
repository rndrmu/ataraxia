use serde::{Deserialize, Serialize};

use super::{id::UserId, ready::UserAvatar};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: UserId,
    pub username: String,
    pub avatar: UserAvatar,
    pub badges: u16,
    pub status: Option<UserStatus>,
    pub relationship: String,
    pub online: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserStatus {
    pub text: String,
    pub presence: UserPresence
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserPresence {
    Online,
    Idle,
    Busy,   
}


#[derive(Serialize, Deserialize, Debug)]
pub struct PartialUser {
    #[serde(rename = "_id")]
    pub id: Option<UserId>,
    pub username: Option<String>,
    pub avatar: Option<UserAvatar>,
    pub badges: Option<u16>,
    pub status: Option<UserStatus>,
    pub relationship: Option<String>,
    pub online: Option<bool>,
}