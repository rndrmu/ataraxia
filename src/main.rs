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


