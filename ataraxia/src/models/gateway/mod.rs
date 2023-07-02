pub mod channel;
pub mod emoji;
pub mod message;
pub mod reaction;
pub mod server;
pub mod user;

use serde::{Deserialize, Serialize};

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
    MessageRemoveReactions,
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
pub struct BulkGatewayEvent {
    /// The event name.
    #[serde(rename = "type")]
    pub event_type: GatewayEvent,
    /// If event is "Bulk", this Vector contains the events.
    #[serde(rename = "v")]
    pub bulk_events: Option<Vec<GatewayEvent>>,
}