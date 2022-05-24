//! # A Voice Server Implementation for Ataraxia revolt library




use std::{sync::{Arc}, ops::DerefMut};

use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use serde_json::json;
use tokio::{net::TcpStream, spawn, sync::Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};


#[derive(Clone)]

pub struct VoiceClient {
    /// Your bot's Token
    pub token: String,
    /// The actual Socket Connection
    socket: Option<Socket>,
    api_url: String
}


#[derive(Clone)]
struct Socket {
    socket_writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    socket_reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
}



impl VoiceClient {
    pub async fn new(token: String,  api_url: Option<String>) -> Self {

        let api_url = match api_url {
            Some(a) => a,
            None => "https://vortex.revolt.chat/".to_owned()
        };


        Self {
            token,
            socket: None,
            api_url
        }
    }


    pub async fn init(&mut self, channel_id: &str) {
        let websocket = Socket::new().await;
        self.socket = Some(websocket);
        self.socket.as_mut().unwrap().connect(&self.token, channel_id).await;
    }
}

impl Socket {
    pub async fn new() -> Socket {
        let (ws_stream, _) = connect_async("wss://vortex.revolt.chat").await.unwrap();
        let (writer, reader) = ws_stream.split();

        Socket {
            socket_writer: Arc::from(Mutex::new(writer)),
            socket_reader: Arc::from(Mutex::new(reader)),
        }
    }

    /// Authenticate to Voice Servers
    /// Where `token` is your bots token
    /// and `channel_id` is the channel id of the voice channel you are connecting to
    pub async fn connect(&self, token: &String, channel_id: &str) -> &Socket {
        self.socket_writer.lock().await.send(Message::Text(json!({
            "id": 0,
            "data": {
                "roomId": channel_id,
                "token": token,
            },
            "type": "Authenticate"
        }).to_string())).await.unwrap();

        self.socket_writer.lock().await.send(Message::Text(json!({
            "id": 1,
            "type": "RoomInfo"
        }).to_string())).await.unwrap();

        self.socket_writer.lock().await.send(Message::Text(json!({"id":25,"type":"InitializeTransports","data":{"mode":"SplitWebRTC","rtpCapabilities":{"codecs":[{"mimeType":"audio/opus","kind":"audio","preferredPayloadType":100,"clockRate":48000,"channels":2,"parameters":{"minptime":10,"useinbandfec":1},"rtcpFeedback":[{"type":"transport-cc","parameter":""}]}],"headerExtensions":[{"kind":"audio","uri":"urn:ietf:params:rtp-hdrext:sdes:mid","preferredId":1,"preferredEncrypt":false,"direction":"sendrecv"},{"kind":"video","uri":"urn:ietf:params:rtp-hdrext:sdes:mid","preferredId":1,"preferredEncrypt":false,"direction":"sendrecv"},{"kind":"audio","uri":"http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time","preferredId":4,"preferredEncrypt":false,"direction":"sendrecv"},{"kind":"video","uri":"http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time","preferredId":4,"preferredEncrypt":false,"direction":"sendrecv"},{"kind":"video","uri":"http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01","preferredId":5,"preferredEncrypt":false,"direction":"sendrecv"},{"kind":"audio","uri":"urn:ietf:params:rtp-hdrext:ssrc-audio-level","preferredId":10,"preferredEncrypt":false,"direction":"sendrecv"},{"kind":"video","uri":"urn:3gpp:video-orientation","preferredId":11,"preferredEncrypt":false,"direction":"sendrecv"},{"kind":"video","uri":"urn:ietf:params:rtp-hdrext:toffset","preferredId":12,"preferredEncrypt":false,"direction":"sendrecv"}]}}}
    ).to_string())).await.unwrap();
    

    self.socket_writer.lock().await.send(Message::Text(json!(
        {"id":30,"type":"StartProduce","data":{"type":"audio","rtpParameters":{"codecs":[{"mimeType":"audio/opus","payloadType":111,"clockRate":48000,"channels":2,"parameters":{"minptime":10,"useinbandfec":1},"rtcpFeedback":[{"type":"transport-cc","parameter":""}]}],"headerExtensions":[{"uri":"urn:ietf:params:rtp-hdrext:sdes:mid","id":4,"encrypt":false,"parameters":{}},{"uri":"http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time","id":2,"encrypt":false,"parameters":{}},{"uri":"http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01","id":3,"encrypt":false,"parameters":{}},{"uri":"urn:ietf:params:rtp-hdrext:ssrc-audio-level","id":1,"encrypt":false,"parameters":{}}],"encodings":[{"ssrc":3082236920i64,"dtx":false}],"rtcp":{"cname":"PxvC7Ug841mk/2iE","reducedSize":true},"mid":"0"}}}
).to_string())).await.unwrap();



        let handler_reader = Arc::clone(&self.socket_reader);
        let handler_writer = Arc::clone(&self.socket_writer);
        let arc_token = Arc::clone(&Arc::new(token.to_owned()));

        spawn(async move {
            crate::vortex_socket::Socket::handler(handler_reader, handler_writer, arc_token).await;
        }).await.unwrap();

        self
    }




    pub async fn handler(reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
        token: Arc<String>
    )
    {
            while let Some(message) = reader.lock().await.next().await {
                match message {
                    Ok(message) => {

                        if message.is_text() {
                            let json: serde_json::Value = serde_json::from_str(&message.to_string()).unwrap();
                            let json_clone = json.clone();
                            
                            match json["type"].as_str() {                                
                                Some("Authenticate") => {

                                    println!("Received Authenticated");

                                    // spawn heartbeat thread 
                                    
                                    //  let writer_clone = Arc::clone(&writer);
                                    //  tokio::spawn(async move {
                                    //      loop {
                                    //          println!("[VORTEX] Sending Heartbeat...");
                                    //          let ping_result = writer_clone.lock().await.send(Message::Text(serde_json::json!({
                                    //              "type": "Ping",
                                    //              "data": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                                    //          }).to_string())).await;

                                    //          match ping_result {
                                    //              Ok(_) => {
                                    //                  println!("[VORTEX] Heartbeat Sent");
                                    //              },
                                    //              Err(e) => {
                                    //                  println!("[VORTEX] Heartbeat Failed: {}", e);
                                    //                  // close socket
                                    //                  break;
                                    //                 writer_clone.lock().await.close();
                                    //              }
                                    //          }

                                    //          // release lock and wait for next heartbeat
                                    //          tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                                    //      }
                                    //  });
                                },
                                Some("InitializeTransports") => {
                                    let ip = json["data"]["sendTransport"]["iceCandidates"][0]["ip"].as_str().unwrap();
                                    let port = json["data"]["sendTransport"]["iceCandidates"][0]["port"].as_u64().unwrap();

                                    println!("[VORTEX] Initializing Transports");
                                    println!("[VORTEX] IP: {}", ip);
                                    println!("[VORTEX] Port: {}", port);

                                },
                                Some(&_) => {
                                    println!("[VORTEX] Received Message Type: {}", json["type"]);
                                },
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