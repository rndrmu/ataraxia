use serde::{Deserialize, Serialize};

use super::{id::{MessageId, ChannelId, UserId, EmojiId}, message::{Embed}};

#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug)]
pub enum GatewayEvent {
    Authenticate,
    BeginTyping,
    EndTyping,
    Ping,
    Pong,
    Error,
    Authenticated,
    Bulk,
    Ready,
    Message,
    MessageUpdate,
    MessageAppend,
    MessageDelete,
    MessageReact,
    MessageUnreact,
    ChannelCreate,
    ChannelUpdate,
    ChannelDelete,
    ChannelGroupJoin,
    ChannelGroupLeave,
    ChannelStartTyping,
    ChannelStopTyping,
    ChannelAck,
    ServerCreate,
    ServerUpdate,
    ServerDelete,
    ServerMemberUpdate,
    ServerMemberJoin,
    ServerMemberLeave,
    ServerRoleUpdate,
    ServerRoleDelete,
    UserUpdate,
    UserRelationship,
    EmojiCreate,
    EmojiDelete

}


#[derive(Serialize, Deserialize, Debug)]
pub struct GenericPayload {
    /// The event name.
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    /// If event is "Bulk", this Vector contains the events.
    #[serde(rename = "v")]
    pub bulk_events: Option<Vec<GatewayEvent>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageUpdate {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    pub id: MessageId,
    pub channel: ChannelId,
    pub data: MessageUpdateData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageUpdateData {
    pub content: Option<String>,
    pub mentions: Option<Vec<String>>,
    pub embeds: Option<Vec<Embed>>,
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
pub struct ChannelStartTyping {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    #[serde(rename = "user")]
    pub user_id: UserId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelStopTyping {
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    #[serde(rename = "id")]
    pub channel_id: ChannelId,
    #[serde(rename = "user")]
    pub user_id: UserId,
}