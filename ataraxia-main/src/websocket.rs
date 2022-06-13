
use std::{sync::{Arc}};
use tracing::{debug, info, error};

use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use serde_json::json;
use tokio::{net::TcpStream, spawn, sync::Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use crate::{models::{message::Message as RevoltMessage, ready::Ready}, context::Context, http::Http};

#[derive(Clone)]
pub struct Client {
    /// Your bot's Token
    /// 
    /// pub, because - well its set by the user 
    pub(crate) token: String,
    /// The actual Socket Connection
    socket: Option<Socket>,
    #[allow(dead_code)]
    api_url: Option<String>,
    socket_url: Option<String>,
    event_handler: Option<Arc<dyn EventHandler>>
}


#[derive(Clone)]
struct Socket {
    socket_writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    socket_reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    event_handler: Arc<dyn EventHandler>,
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
    async fn on_message(&self, context: Context, message: RevoltMessage);
    /*async fn message_update(&self);
    async fn message_delete(&self);
    async fn channel_create(&self);
    async fn channel_update(&self);
    async fn channel_delete(&self);
    async fn channel_group_join(&self);
    async fn channel_group_leave(&self);
    async fn channel_start_typing(&self);
    async fn channel_stop_typing(&self);
    async fn channel_ack(&self);
    async fn server_update(&self);
    async fn server_delete(&self);
    async fn server_member_update(&self);
    async fn server_member_join(&self);
    async fn server_member_leave(&self);
    async fn server_role_update(&self);
    async fn server_role_delete(&self);
    async fn user_update(&self);
    async fn user_relationship(&self);*/
}



impl Client {
    pub fn new(token: String) -> Self {

        Self {
            token,
            socket: None,
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

        let websocket = Socket::new(socket_url ,handler.to_owned()).await;
        self.socket = Some(websocket);
        self.socket.as_mut().unwrap().connect(self.token.clone()).await;
    }
}

impl Socket {
    pub async fn new(socket_url: String ,handler: Arc<dyn EventHandler>) -> Socket {
        let (ws_stream, _) = connect_async(socket_url).await.unwrap();
        let (writer, reader) = ws_stream.split();

        Socket {
            socket_writer: Arc::from(Mutex::new(writer)),
            socket_reader: Arc::from(Mutex::new(reader)),
            event_handler: handler
        }
    }

    pub async fn connect(&self, token: String) -> &Socket {
        debug!("Connecting...");
        self.socket_writer.lock().await.send(Message::Text(json!({
            "type": "Authenticate",
            "token": token
        }).to_string())).await.unwrap();

        let handler_reader = Arc::clone(&self.socket_reader);
        let handler_writer = Arc::clone(&self.socket_writer);
        let arc_token = Arc::clone(&Arc::new(token.to_owned()));
        let arc_handler = Arc::clone(&self.event_handler);

        spawn(async move {
            crate::websocket::Socket::handler(handler_reader, handler_writer, arc_token, arc_handler).await;
        }).await.unwrap();

        self
    }



    pub async fn handler(reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
        token: Arc<String>,
        event: Arc<dyn EventHandler>)
    {
            while let Some(message) = reader.lock().await.next().await {
                match message {
                    Ok(message) => {

                        if message.is_text() {
                            let json: serde_json::Value = serde_json::from_str(&message.to_string()).unwrap();
                            let json_clone = json.clone();
                            
                            match json["type"].as_str() {
                                Some("Ready") => {
                                    let ready: Ready = serde_json::from_value(json).unwrap();
                                    event.ready(Context::new(&token, &message.to_string()), ready).await;

                                    
                                },
                                
                                Some("Authenticated") => {
                                    event.authenticated().await;

                                    // spawn heartbeat thread 
                                    
                                    let writer_clone = Arc::clone(&writer);
                                    tokio::spawn(async move {
                                        loop {
                                            info!("[GATEWAY] Sending Heartbeat...");
                                            writer_clone.lock().await.send(Message::Text(serde_json::json!({
                                                "type": "Ping",
                                                "data": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                                            }).to_string())).await.unwrap();
                                            // release lock and wait for next heartbeat
                                            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                                        }
                                    });
                                },
                                Some("Message") => {
                                    let message: Result<crate::models::message::Message, serde_json::Error> = serde_json::from_value(json);
                                    if let Ok(message) = message {
                                        event.on_message(Context::new(&token, &json_clone.to_string()), message).await;
                                    }
                                },
                                Some(&_) => {},
                                None => {},
                            }
                        }

                    }
                    Err(e) => {
                        return error!("{:?}", e);
                    }
                }
            }
    }
}
