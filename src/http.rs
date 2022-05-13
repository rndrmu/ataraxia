use reqwest::Client;

#[derive(Clone)]
pub struct Http {
    client: crate::websocket::Client,
}

impl Http {

    pub async fn new(revolt_client: crate::websocket::Client) -> Self {
        Self {
            client: revolt_client,
        }
    }


    /// Sends a simple Content Only Message to the revolt API.
    /// ```rs
    /// let a = Http::new();
    /// let b = a.say("topsecrettoken", "channel_id", "Hello, world!").await;
    /// ```
    pub async fn say(&self, channel_id: &str,content: impl Into<String>) -> Result<(), reqwest::Error> {
        let client = Client::new();
        let res = client.post(format!(
            "https://api.revolt.chat/channels/{channel_id}/messages"
        ))
        .header("x-bot-token", format!("{token}", token = self.client.token))
        .header("content-type", "application/json")
        .body(serde_json::json!({
            "content": content.into()
        }).to_string())
        .send()
        .await;

        res.map(|_| ())
    }
}