//! # A Voice Server Implementation for Ataraxia revolt library




use std::{sync::{Arc}};

use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use serde_json::json;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};


use crate::FRAME_LENGTH;
#[derive(Clone)]

pub struct VoiceClient {
    /// Your bot's Token
    pub token: String,
    /// The actual Socket Connection
    socket: Option<Socket>,
    #[allow(dead_code)]
    api_url: String,
    ata_socket: Arc<Option<Socket>>

}


#[derive(Clone)]
struct Socket {
    socket_writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    socket_reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    udp_socket: Arc<Mutex<tokio::net::UdpSocket>>,
    srtp_parameters: Option<String>,
    vortex_id: Option<String>,
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
            api_url,
            ata_socket: Arc::new(None)
        }
    }


    pub async fn init(&mut self, channel_id: &str) {
        let websocket = Socket::new().await;
        self.socket = Some(websocket);
        let conn = self.socket.clone().unwrap().connect(&self.token, channel_id).await;
        self.ata_socket = Arc::new(Some(conn));
        println!("Connected!");
    }


    pub async fn play_source(&mut self, _source: &str) {

       
    }

}

impl Socket {
    pub async fn new() -> Socket {
        let (ws_stream, _) = connect_async("wss://vortex.revolt.chat").await.unwrap();
        let (writer, reader) = ws_stream.split();
        let udp_socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await.unwrap();

        Socket {
            socket_writer: Arc::from(Mutex::new(writer)),
            socket_reader: Arc::from(Mutex::new(reader)),
            udp_socket: Arc::from(Mutex::new(udp_socket)),
            srtp_parameters: None,
            vortex_id: None
        }
    }

    async fn initialize_transports(&self) {
        self.socket_writer.lock().await.send(Message::Text(json!(
            {
                "id":25,
                "type":"InitializeTransports",
                "data":{"mode":"CombinedRTP",
                "rtpCapabilities":{
                    "codecs":[{"mimeType":"audio/opus","kind":"audio","preferredPayloadType":100,"clockRate":48000,"channels":2,"parameters":{"minptime":10,"useinbandfec":1},"rtcpFeedback":[{"type":"transport-cc","parameter":""}]}],
                    "headerExtensions":[
                        {"kind":"audio","uri":"urn:ietf:params:rtp-hdrext:sdes:mid","preferredId":1,"preferredEncrypt":false,"direction":"sendrecv"},
                        {"kind":"video","uri":"urn:ietf:params:rtp-hdrext:sdes:mid","preferredId":1,"preferredEncrypt":false,"direction":"sendrecv"},
                        {"kind":"audio","uri":"http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time","preferredId":4,"preferredEncrypt":false,"direction":"sendrecv"},
                        {"kind":"video","uri":"http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time","preferredId":4,"preferredEncrypt":false,"direction":"sendrecv"},
                        {"kind":"video","uri":"http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01","preferredId":5,"preferredEncrypt":false,"direction":"sendrecv"},
                        {"kind":"audio","uri":"urn:ietf:params:rtp-hdrext:ssrc-audio-level","preferredId":10,"preferredEncrypt":false,"direction":"sendrecv"},
                        {"kind":"video","uri":"urn:3gpp:video-orientation","preferredId":11,"preferredEncrypt":false,"direction":"sendrecv"},
                        {"kind":"video","uri":"urn:ietf:params:rtp-hdrext:toffset","preferredId":12,"preferredEncrypt":false,"direction":"sendrecv"}]}}}
    ).to_string())).await.unwrap();
    }

    async fn connect_transport(&self) {
        self.socket_writer.lock().await.send(Message::Text(json!(
        {
            "id":30,
            "type":"ConnectTransport",
            "data":{
                "id": self.vortex_id.as_ref().unwrap(),
                "srtpParameters": {
                    "cryptoSuite": "AES_CM_128_HMAC_SHA1_80",
                    "keyBase64": "bWVkZGxfbG9pZGU=", // this is simply "meddl_loide" encoded into base64 :^)
                },
            }
        }
).to_string())).await.unwrap();
    }

    async fn start_produce(&self) {

    self.socket_writer.lock().await.send(Message::Text(json!(
        {
            "id":30,
            "type":"StartProduce",
            "data":{
                "type":"audio","rtpParameters":{
                    "codecs":[
                        {"mimeType":"audio/opus","payloadType":111,"clockRate":48000,"channels":2,"parameters":{"minptime":10,"useinbandfec":1},
                        "rtcpFeedback":[{"type":"transport-cc","parameter":""}]}],
                        "headerExtensions":[
                            {"uri":"urn:ietf:params:rtp-hdrext:sdes:mid","id":4,"encrypt":false,"parameters":{}},
                            {"uri":"http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time","id":2,"encrypt":false,"parameters":{}},
                            {"uri":"http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01","id":3,"encrypt":false,"parameters":{}},
                            {"uri":"urn:ietf:params:rtp-hdrext:ssrc-audio-level","id":1,"encrypt":false,"parameters":{}}],
                            "encodings":[{"ssrc":3082236920i64,"dtx":false}],
                            "rtcp":{"cname":"PxvC7Ug841mk/2iE","reducedSize":false},
                            "mid":"0"}}}
).to_string())).await.unwrap();
    }

    /// Authenticate to Voice Servers
    /// Where `token` is your bots token
    /// and `channel_id` is the channel id of the voice channel you are connecting to
    pub async fn connect(self, token: &String, channel_id: &str) -> Socket {
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

        self.initialize_transports().await;

        
    


println!("3 pew pew");


        let handler_reader = Arc::clone(&self.socket_reader);
        let handler_writer = Arc::clone(&self.socket_writer);
        let arc_token = Arc::clone(&Arc::new(token.to_owned()));

        let mut self_clone = self.clone();
        println!("2 pew pew");


        tokio::spawn(async move {
            crate::vortex_socket::Socket::handler(&mut self_clone, handler_reader, handler_writer, arc_token).await;
        });

        println!("pew pew");

        self
    }




    pub async fn handler(&mut self, reader: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
        _writer: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
        _token: Arc<String>
    )
    {
            while let Some(message) = reader.lock().await.next().await {
                match message {
                    Ok(message) => {

                        if message.is_text() {
                            let json: serde_json::Value = serde_json::from_str(&message.to_string()).unwrap();
                            let _json_clone = json.clone();
                            
                            match json["type"].as_str() {                                
                                Some("Authenticate") => {

                                    println!("Received Authenticated");

                                },
                                Some("InitializeTransports") => {
                                    println!("[DEBUG] JSON PAYLOAD {:?}", json);

                                    let ip = json["data"]["ip"].as_str().unwrap();
                                    let port = json["data"]["port"].as_i64().unwrap();
                                    let srtp_crypto_suite = json["data"]["srtpCryptoSuite"].as_str().unwrap();
                                    self.srtp_parameters = Some(srtp_crypto_suite.to_string());
                                    let id = json["data"]["id"].as_str().unwrap();
                                    self.vortex_id = Some(id.to_string());

                                    self.udp_socket.lock().await.connect(
                                        format!("{}:{}", ip, port)
                                    ).await.unwrap();
                                    println!("[VORTEX] UDP Socket Connected");

                                    self.connect_transport().await;
                                    println!("[VORTEX] Connected to Transport");
                                    self.start_produce().await;
                                    println!("[VORTEX] Started Produce");

                                },

                                Some("StartProduce") => {
                                    // Send Audio here
                                    println!("[VORTEX] Start Produce");

                                    // sleep for a bit to let the client connect
                                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                                    println!("[VORTEX] Sending Audio");

                                    let ffmpeg = std::process::Command::new("ffmpeg")
                                        .arg("-i")
                                        .arg("/home/me/audio/meddl.mp3")
                                        .arg("-f")
                                        .arg("s16le")
                                        .arg("-ac")
                                        .arg("2")
                                        .arg("-ar")
                                        .arg("48000")
                                        .arg("-acodec")
                                        .arg("pcm_s16le")
                                        .arg("-")
                                        .output()

                                        .expect("[CRITICAL] Failed to execute ffmpeg");

                                    // split 

                                    let packet = ffmpeg.stdout;



                                   let opus = super::encode_to_opus(packet);

                                let opus_paket =  match opus {
                                    Ok(opus) => opus,
                                    Err(e) => {
                                        println!("[CRITICAL] Failed to encode to opus: {}", e);
                                        return;
                                    }
                                };

                                println!("{:?}", opus_paket)
                                    

                                   
                                }

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