
use std::{sync::{Arc}, ops::DerefMut};

use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use serde_json::json;
use tokio::{net::TcpStream, spawn, sync::Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};
use crate::{models::message::Message as RevoltMessage, context::Context};

#[derive(Clone)]

pub struct Client {
    pub token: String,
    socket: Option<Socket>,
    api_url: String
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
    async fn ready(&self, context: Context);
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
    pub async fn new(token: String,  api_url: Option<String>) -> Self {

        let api_url = match api_url {
            Some(a) => a,
            None => "https://api.revolt.chat/".to_owned()
        };


        Self {
            token,
            socket: None,
            api_url
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
                            let json_clone = json.clone();
                            
                            match json["type"].as_str() {
                                Some("Ready") => {
                                    println!("{}", json);
                                    event.ready(Context::new(&token, &message.to_string())).await;

                                    
                                },
                                
                                Some("Authenticated") => {
                                    event.authenticated().await;

                                    // spawn heartbeat thread 
                                    
                                    let writer_clone = Arc::clone(&writer);
                                    tokio::spawn(async move {
                                        loop {
                                            println!("[GATEWAY] Sending Heartbeat...");
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
                        return eprintln!("{:?}", e);
                    }
                }
            }
    }
}