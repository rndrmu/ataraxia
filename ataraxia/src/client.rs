use std::sync::Arc;
use crate::{gateway::*, models::{message::Message as RevoltMessage, ready::Ready, gateway::{message::MessageUpdate, message::MessageDelete, message::MessageReact, message::MessageUnreact, message::MessageRemoveReactions, channel::ChannelCreate}, gateway::{channel::{ChannelUpdate, ChannelDelete, ChannelGroupJoin, ChannelGroupLeave, ChannelStartTyping, ChannelStopTyping, ChannelAck}, server::{ServerUpdate, ServerDelete, ServerMemberUpdate, ServerMemberJoin, ServerMemberLeave, ServerRoleUpdate, ServerRoleDelete}, user::{UserUpdate, UserRelationship}}}, context::Context, http::Http};

#[derive()]
pub struct Client {
    /// Your bot's Token
    /// 
    /// pub, because - well its set by the user 
    pub(crate) token: String,
    #[allow(dead_code)]
    api_url: Option<String>,
    socket_url: Option<String>,
    event_handler: Option<Arc<dyn EventHandler>>
}





#[async_trait::async_trait]
pub trait EventHandler: Send + Sync + 'static {
    /*async fn error(&self);*/
    /// Dispatched upon a successful connection to the Revolt Api.
    async fn authenticated(&self);

    /// Dispatched once the Client has been authenticated and the Socket has been connected.
    ///
    /// A `Ready` Payload is passed to this method, containing all users, channels and servers the
    /// bot is in.
    async fn ready(&self, context: Context, ready: Ready);

    /// Dispatched when a message is received.
    async fn on_message(&self,     _context: Context, _message: RevoltMessage) {}
    async fn message_update(&self, _context: Context, _updated_message: MessageUpdate) {}
    async fn message_delete(&self, _context: Context, _deleted_message: MessageDelete) {}

    
    async fn message_react(&self,  _context: Context, _reaction: MessageReact) {}
    async fn message_unreact(&self, _context: Context, _reaction: MessageUnreact) {}
    async fn message_remove_reactions(&self, _context: Context, _reaction: MessageRemoveReactions) {}


    async fn channel_create(&self, _context: Context, _channel: ChannelCreate) {}
    async fn channel_update(&self, _context: Context, _channel: ChannelUpdate) {}
    async fn channel_delete(&self, _context: Context, _channel: ChannelDelete) {}
    
    
    async fn channel_group_join(&self, _context: Context, _channel: ChannelGroupJoin) {}
    async fn channel_group_leave(&self, _context: Context, _channel: ChannelGroupLeave) {}
    
    
    async fn channel_start_typing(&self, _context: Context, _cst: ChannelStartTyping) {}
    async fn channel_stop_typing(&self, _context: Context, _cst: ChannelStopTyping) {}
    
    
    async fn channel_ack(&self, _context: Context, _ack: ChannelAck) {}
    
    
    async fn server_update(&self, _context: Context, _server_data: ServerUpdate) {}
    async fn server_delete(&self, _context: Context, _server_data: ServerDelete) {}
    
    
    async fn server_member_update(&self, _context: Context, _server_member_data: ServerMemberUpdate) {}
    async fn server_member_join(&self, _context: Context, _server_member_data: ServerMemberJoin) {}
    async fn server_member_leave(&self, _context: Context, _server_member_data: ServerMemberLeave) {}
    
    
    async fn server_role_update(&self, _context: Context, _server_role_data: ServerRoleUpdate) {}
    async fn server_role_delete(&self, _context: Context, _server_role_data: ServerRoleDelete) {}
    
    
    async fn user_update(&self, _context: Context, _user_data: UserUpdate) {}
    async fn user_relationship(&self, _context: Context, _user_data: UserRelationship) {}
}



impl Client {
    pub fn new(token: String) -> Self {

        Self {
            token,
            api_url: None,
            socket_url: None,
            event_handler: None
        }
    }

    pub fn event_handler<H: EventHandler + 'static>(mut self, event_handler: H) -> Self {
        self.event_handler = Some(Arc::new(event_handler));

        self
    }

    pub async fn set_api_url<D: ToString>(mut self, api_url: D) -> Self {
        self.api_url = Some(api_url.to_string());

        // get socket url from Http
        let server_config = Http::new().get_server_config().await.unwrap();

        // set socket url
        self.socket_url = Some(server_config.websocket_url);

        self
    }

    pub async fn start(&mut self) {

        let handler = match &self.event_handler {
            Some(h) => h,
            None => panic!("Expected Event Handler in initialisation!")
        };

        let socket_url = match self.socket_url.clone() {
            Some(url) => url,
            None => "wss://ws.revolt.chat".to_owned()
        };

        let websocket = shard::Shard::new(socket_url ,handler.to_owned()).await;
        
        websocket.connect(self.token.clone()).await;
    }
}

