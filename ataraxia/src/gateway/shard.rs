use std::{sync::Arc, time::Duration};
use tracing::{debug, error, info};

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::mpsc::UnboundedReceiver;

use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, Connector};

use crate::{
    client::EventHandler,
    context::Context,
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

pub type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type JsonResult<T> = std::result::Result<T, serde_json::Error>;

pub struct Shard {
    ws: WsStream,
    event_handler: Arc<dyn EventHandler>,
}

pub(crate) async fn create_client(url: String) -> Result<WsStream> {
    let parsed = url::Url::parse(&url)?;
    let host = parsed.host_str().ok_or("no host in gateway URL")?.to_string();
    let port = parsed.port_or_known_default().ok_or("no default port for scheme")?;

    // Prefer IPv4: async-tungstenite tries addresses sequentially, so unreachable IPv6
    // addresses cause a ~75s TCP timeout each before falling through to IPv4.
    let addrs: Vec<_> = tokio::net::lookup_host(format!("{}:{}", host, port))
        .await?
        .collect();
    let addr = addrs.iter()
        .find(|a| a.is_ipv4())
        .or_else(|| addrs.first())
        .copied()
        .ok_or("DNS resolution returned no addresses")?;

    let tcp = tokio::net::TcpStream::connect(addr).await?;

    let native_cx = tokio_native_tls::native_tls::TlsConnector::new()?;
    let connector = Connector::NativeTls(native_cx);

    let (stream, _) = tokio_tungstenite::client_async_tls_with_config(
        url.as_str(),
        tcp,
        None,
        Some(connector),
    ).await?;
    Ok(stream)
}

impl Shard {
    pub async fn new(socket_url: String, handler: Arc<dyn EventHandler>) -> Self {
        info!("[GATEWAY] Connecting to {}", socket_url);
        let ws = match create_client(socket_url).await {
            Ok(ws) => { info!("[GATEWAY] WebSocket connected"); ws }
            Err(e) => panic!("[GATEWAY] Failed to connect: {:?}", e),
        };
        Self { event_handler: handler, ws }
    }

    pub async fn connect(mut self, token: String) {
        self.authenticate(token.clone()).await;
        info!("[GATEWAY] Auth message sent, waiting for response...");

        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let (mut write, mut read) = self.ws.split();

        // Reader task: forwards WS messages into the mpsc channel
        tokio::spawn(async move {
            loop {
                match read.next().await {
                    Some(Ok(message)) => {
                        debug!("[GATEWAY] Raw frame: {:?}", message);
                        if let Err(e) = sender.send(message) {
                            error!("[GATEWAY] Channel send error: {:?}", e);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        error!("[GATEWAY] WS read error: {:?}", e);
                        break;
                    }
                    None => {
                        error!("[GATEWAY] WS stream closed by server");
                        break;
                    }
                }
            }
        });

        // Heartbeat task: runs independently without holding the write lock
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let ping = json!({
                    "type": "Ping",
                    "data": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                })
                .to_string();
                info!("[GATEWAY] Sending heartbeat");
                if let Err(e) = write.send(Message::Text(ping)).await {
                    error!("Heartbeat send error: {:?}", e);
                }
            }
        });

        // Event processing task
        let handler = self.event_handler.clone();
        tokio::spawn(async move {
            handle_events(receiver, Arc::new(token), handler).await;
        })
        .await
        .unwrap_or_else(|e| error!("Event handler task error: {:?}", e));
    }

    async fn authenticate(&mut self, token: String) {
        self.ws
            .send(Message::Text(
                json!({ "type": "Authenticate", "token": token }).to_string(),
            ))
            .await
            .unwrap_or_else(|e| error!("Auth send error: {:?}", e));
    }
}

pub async fn handle_events(
    mut receiver: UnboundedReceiver<Message>,
    token: Arc<String>,
    event: Arc<dyn EventHandler>,
) {
    while let Some(message) = receiver.recv().await {
        if !message.is_text() {
            continue;
        }

        let json: serde_json::Value = match serde_json::from_str(&message.to_string()) {
            Ok(v) => v,
            Err(e) => { error!("JSON parse error: {:?}", e); continue; }
        };
        let json_clone = json.clone();
        let raw = message.to_string();

        debug!("[GATEWAY] {:?}", json["type"]);

        let ctx = || Context::new(&token, &raw);

        match json["type"].as_str() {
            Some("Ready") => {
                match serde_json::from_value::<Ready>(json) {
                    Ok(r) => event.ready(ctx(), r).await,
                    Err(e) => error!("Ready deserialize: {:?}", e),
                }
            }
            Some("Authenticated") => {
                event.authenticated().await;
            }
            Some("Pong") => {}
            Some("Message") => {
                if let Ok(m) = serde_json::from_value::<RevoltMessage>(json) {
                    event.on_message(ctx(), m).await;
                }
            }
            Some("MessageUpdate") => {
                if let Ok(m) = serde_json::from_value::<MessageUpdate>(json) {
                    event.message_update(ctx(), m).await;
                }
            }
            Some("MessageDelete") => {
                if let Ok(m) = serde_json::from_value::<MessageDelete>(json) {
                    event.message_delete(ctx(), m).await;
                }
            }
            Some("MessageReact") => {
                if let Ok(m) = serde_json::from_value::<MessageReact>(json) {
                    event.message_react(ctx(), m).await;
                }
            }
            Some("MessageUnreact") => {
                if let Ok(m) = serde_json::from_value::<MessageUnreact>(json) {
                    event.message_unreact(ctx(), m).await;
                }
            }
            Some("MessageRemoveReactions") => {
                if let Ok(m) = serde_json::from_value::<MessageRemoveReactions>(json) {
                    event.message_remove_reactions(ctx(), m).await;
                }
            }
            Some("ChannelCreate") => {
                if let Ok(m) = serde_json::from_value::<ChannelCreate>(json) {
                    event.channel_create(ctx(), m).await;
                }
            }
            Some("ChannelUpdate") => {
                if let Ok(m) = serde_json::from_value::<ChannelUpdate>(json) {
                    event.channel_update(ctx(), m).await;
                }
            }
            Some("ChannelDelete") => {
                if let Ok(m) = serde_json::from_value::<ChannelDelete>(json) {
                    event.channel_delete(ctx(), m).await;
                }
            }
            Some("ChannelGroupJoin") => {
                if let Ok(m) = serde_json::from_value::<ChannelGroupJoin>(json) {
                    event.channel_group_join(ctx(), m).await;
                }
            }
            Some("ChannelGroupLeave") => {
                if let Ok(m) = serde_json::from_value::<ChannelGroupLeave>(json) {
                    event.channel_group_leave(ctx(), m).await;
                }
            }
            Some("ChannelStartTyping") => {
                if let Ok(m) = serde_json::from_value::<ChannelStartTyping>(json) {
                    event.channel_start_typing(ctx(), m).await;
                }
            }
            Some("ChannelStopTyping") => {
                if let Ok(m) = serde_json::from_value::<ChannelStopTyping>(json) {
                    event.channel_stop_typing(ctx(), m).await;
                }
            }
            Some("ChannelAck") => {
                if let Ok(m) = serde_json::from_value::<ChannelAck>(json) {
                    event.channel_ack(ctx(), m).await;
                }
            }
            Some("ServerUpdate") => {
                if let Ok(m) = serde_json::from_value::<ServerUpdate>(json) {
                    event.server_update(ctx(), m).await;
                }
            }
            Some("ServerDelete") => {
                if let Ok(m) = serde_json::from_value::<ServerDelete>(json) {
                    event.server_delete(ctx(), m).await;
                }
            }
            Some("ServerMemberJoin") => {
                if let Ok(m) = serde_json::from_value::<ServerMemberJoin>(json) {
                    event.server_member_join(ctx(), m).await;
                }
            }
            Some("ServerMemberLeave") => {
                if let Ok(m) = serde_json::from_value::<ServerMemberLeave>(json) {
                    event.server_member_leave(ctx(), m).await;
                }
            }
            Some("ServerMemberUpdate") => {
                if let Ok(m) = serde_json::from_value::<ServerMemberUpdate>(json) {
                    event.server_member_update(ctx(), m).await;
                }
            }
            Some("ServerRoleUpdate") => {
                if let Ok(m) = serde_json::from_value::<ServerRoleUpdate>(json) {
                    event.server_role_update(ctx(), m).await;
                }
            }
            Some("ServerRoleDelete") => {
                if let Ok(m) = serde_json::from_value::<ServerRoleDelete>(json) {
                    event.server_role_delete(ctx(), m).await;
                }
            }
            Some("UserUpdate") => {
                if let Ok(m) = serde_json::from_value::<UserUpdate>(json) {
                    event.user_update(ctx(), m).await;
                }
            }
            Some("UserRelationship") => {
                if let Ok(m) = serde_json::from_value::<UserRelationship>(json) {
                    event.user_relationship(ctx(), m).await;
                }
            }
            Some(unknown) => {
                info!("[GATEWAY] Unknown event type: {} -> {}", unknown, json_clone);
            }
            None => {}
        }
    }
}
