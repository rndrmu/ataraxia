use std::sync::Arc;

use crate::{
    context::Context,
    gateway::shard::Shard,
    http::Http,
    models::{
        gateway::{
            channel::{
                ChannelAck, ChannelCreate, ChannelDelete, ChannelGroupJoin, ChannelGroupLeave,
                ChannelStartTyping, ChannelStopTyping, ChannelUpdate,
            },
            message::{
                MessageDelete, MessageReact, MessageRemoveReactions, MessageUnreact, MessageUpdate,
            },
            server::{
                ServerDelete, ServerMemberJoin, ServerMemberLeave, ServerMemberUpdate,
                ServerRoleDelete, ServerRoleUpdate, ServerUpdate,
            },
            user::{UserRelationship, UserUpdate},
        },
        message::Message as RevoltMessage,
        ready::Ready,
    },
};

pub struct Client {
    pub(crate) token: String,
    #[allow(dead_code)]
    api_url: Option<String>,
    socket_url: Option<String>,
    event_handler: Option<Arc<dyn EventHandler>>,
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync + 'static {
    /// Fired after successful gateway authentication.
    async fn authenticated(&self);

    /// Fired when the gateway sends the `Ready` payload.
    async fn ready(&self, context: Context, ready: Ready);

    async fn on_message(&self, _ctx: Context, _msg: RevoltMessage) {}
    async fn message_update(&self, _ctx: Context, _msg: MessageUpdate) {}
    async fn message_delete(&self, _ctx: Context, _msg: MessageDelete) {}
    async fn message_react(&self, _ctx: Context, _r: MessageReact) {}
    async fn message_unreact(&self, _ctx: Context, _r: MessageUnreact) {}
    async fn message_remove_reactions(&self, _ctx: Context, _r: MessageRemoveReactions) {}

    async fn channel_create(&self, _ctx: Context, _c: ChannelCreate) {}
    async fn channel_update(&self, _ctx: Context, _c: ChannelUpdate) {}
    async fn channel_delete(&self, _ctx: Context, _c: ChannelDelete) {}
    async fn channel_group_join(&self, _ctx: Context, _c: ChannelGroupJoin) {}
    async fn channel_group_leave(&self, _ctx: Context, _c: ChannelGroupLeave) {}
    async fn channel_start_typing(&self, _ctx: Context, _c: ChannelStartTyping) {}
    async fn channel_stop_typing(&self, _ctx: Context, _c: ChannelStopTyping) {}
    async fn channel_ack(&self, _ctx: Context, _c: ChannelAck) {}

    async fn server_update(&self, _ctx: Context, _s: ServerUpdate) {}
    async fn server_delete(&self, _ctx: Context, _s: ServerDelete) {}
    async fn server_member_update(&self, _ctx: Context, _s: ServerMemberUpdate) {}
    async fn server_member_join(&self, _ctx: Context, _s: ServerMemberJoin) {}
    async fn server_member_leave(&self, _ctx: Context, _s: ServerMemberLeave) {}
    async fn server_role_update(&self, _ctx: Context, _s: ServerRoleUpdate) {}
    async fn server_role_delete(&self, _ctx: Context, _s: ServerRoleDelete) {}

    async fn user_update(&self, _ctx: Context, _u: UserUpdate) {}
    async fn user_relationship(&self, _ctx: Context, _u: UserRelationship) {}
}

impl Client {
    pub fn new(token: String) -> Self {
        Self { token, api_url: None, socket_url: None, event_handler: None }
    }

    pub fn event_handler<H: EventHandler + 'static>(mut self, handler: H) -> Self {
        self.event_handler = Some(Arc::new(handler));
        self
    }

    pub async fn set_api_url<D: ToString>(mut self, api_url: D) -> Self {
        self.api_url = Some(api_url.to_string());
        let config = Http::new().get_server_config().await.unwrap();
        self.socket_url = Some(config.websocket_url);
        self
    }

    pub async fn start(&mut self) {
        let handler = self
            .event_handler
            .as_ref()
            .expect("Event handler must be set before calling start()");

        let socket_url = self
            .socket_url
            .clone()
            .unwrap_or_else(|| "wss://ws.revolt.chat".to_owned());

        let shard = Shard::new(socket_url, handler.clone()).await;
        shard.connect(self.token.clone()).await;
    }
}
