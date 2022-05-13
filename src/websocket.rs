
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use serde_json::json;
use tokio::{net::TcpStream, spawn, sync::Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

#[derive(Clone)]

pub struct Client {
    pub token: String,
    socket: Option<Socket>,
}

#[derive(Clone)]
struct Socket {
    socket_writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    socket_reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    handler: Arc<Box<dyn EventHandler>>
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync + 'static {
    /*async fn error(&self);*/
    async fn authenticated(&self);
    async fn ready(&self);
    async fn on_message(&self, message: Message);
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
    pub async fn new(token: String) -> Self {
        Self {
            token,
            socket: None,
        }
    }


    pub async fn run<S>(&mut self, event_handler: S) where S: EventHandler + Send + Sync + 'static {
        let websocket = Socket::new(Box::new(event_handler)).await;
        self.socket = Some(websocket);
        self.socket.as_mut().unwrap().connect(self.token.clone()).await;
    }
}

impl Socket {
    pub async fn new(handler: Box<dyn EventHandler>) -> Socket {
        let (ws_stream, _) = connect_async("wss://ws.revolt.chat").await.unwrap();
        let (writer, reader) = ws_stream.split();

        Socket {
            socket_writer: Arc::from(Mutex::new(writer)),
            socket_reader: Arc::from(Mutex::new(reader)),
            handler: Arc::from(handler)
        }
    }

    pub async fn connect(&self, token: String) -> &Socket {
        println!("Connecting...");
        self.socket_writer.lock().await.send(Message::Text(json!({
            "type": "Authenticate",
            "token": token
        }).to_string())).await.unwrap();

        let handler_reader = Arc::clone(&self.socket_reader);
        let handler_writer = Arc::clone(&self.socket_writer);
        let arc_token = Arc::clone(&Arc::new(token.to_owned()));
        let arc_handler = Arc::clone(&self.handler);

        spawn(async move {
            println!("Spawning Event Loop...");
            crate::websocket::Socket::handler(handler_reader, handler_writer, arc_token, arc_handler).await;
        }).await.unwrap();

        self
    }

    pub async fn send_heartbeat(&self) {
        println!("[GATEWAY] Sending Heartbeat...");
        self.socket_writer.lock().await.send(Message::Ping(serde_json::json!({
            "type": "Ping",
            "time": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
        }).to_string()
        .as_bytes()
        .to_vec())).await.unwrap();
    }

    pub async fn handler(reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
        token: Arc<String>,
        event: Arc<Box<dyn EventHandler>>)
    {
            while let Some(message) = reader.lock().await.next().await {
                match message {
                    Ok(message) => {

                        if message.is_text() {
                            let json: serde_json::Value = serde_json::from_str(&message.to_string()).unwrap();

                            if let Some(_type) = json["type"].as_str() {
                                if _type == "Ready" {
                                    event.ready().await
                                }
                            }
                            
                            match json["type"].as_str() {
                                Some("Authenticated") => event.authenticated().await,
                                Some("Message") => {
                                    event.on_message(message).await;
                                },
                                Some(&_) => {},
                                None => {},
                            }
                        }

                    }
                    Err(e) => {
                        return eprintln!("{:?}", e);
                    }
                }
            }
    }
}