use tokio;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};

use tokio::{net::TcpStream, spawn, sync::Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use futures_util::{stream::{SplitSink, SplitStream}};

use revoltchat_rs::websocket::Client;
use revoltchat_rs::websocket::EventHandler;

struct Handler;

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn authenticated(&self) {
        println!("Authenticated!");
    }
    async fn ready(&self) {
        println!("Ready!");
        println!(":trol:");
    }
    async fn on_message(&self, message: Message) {
        println!("{:?}", message);
    }
}




#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let token = std::env::var("REVOLT_TOKEN").expect("token");

    let connect_addr = "wss://ws.revolt.chat";


    Client::new(token).await.run(Handler).await;




}


async fn _old_main() {

    dotenv::dotenv().ok();

    let token = std::env::var("REVOLT_TOKEN").expect("token");

    let connect_addr = "wss://ws.revolt.chat"; //&format!("wss://ws.revolt.chat?version=1&format=json&token=-BvXOcyZkwe6HufC9mYM3dc9eR7iae-HIKleL6SgCrLvnZLWHQjsRT77GBNtfRQM", TOKEN);




    let (ws_stream, _) = connect_async(connect_addr).await.expect(":trol:");
    println!("WebSocket handshake has been successfully completed");

    

    let (mut write, mut read) = ws_stream.split();

    write.send(Message::Text(serde_json::json!(
        {
            "type": "Authenticate",
            "token": token
        }
    ).to_string())).await.unwrap();

    println!("[GATEWAY] Successfully authenticated");

    

        while let Some(msg) = read.next().await {
            match msg {
                Ok(message) => {

                    if message.is_text() {
                        let json: serde_json::Value = serde_json::from_str(&message.to_string()).unwrap();

                        if let Some(_type) = json["type"].as_str() {
                            if _type == "Ready" {
                                println!("[GATEWAY] Ready");
                            }
                        }

                        println!("[GATEWAY] {}", json["type"]);

                        match json["type"].as_str() {
                            Some("Authenticated") => {
                                println!("[GATEWAY] Authenticated");
                                
                            },
                            Some("Message") => {
                                println!("[GATEWAY] Message: {}", json["content"].as_str().unwrap());
                            },
                            Some("Pong") => {
                                println!("[GATEWAY] Pong");
                            },
                            Some(&_) => {
                              
                            },
                            None => {},
                        }

                       
                        
                    }

                    
                },
                Err(e) => {
                    println!("Error: {}", e);
                }
            }

           
        }
    // send Authentication message

    
    

    
}
