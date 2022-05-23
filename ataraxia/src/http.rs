use reqwest::Client;

#[derive(Clone)]
pub struct Http;

impl Http {

    /// Sends a simple Content Only Message to the revolt API.
    /// ```rs
    /// let a = Http::new();
    /// let b = a.say("topsecrettoken", "channel_id", "Hello, world!").await;
    /// ```
    pub async fn post(destination: &str, token: &str, body: serde_json::Value) -> Result<(), reqwest::Error> {
        let client = Client::new();
        let res = client.post(destination)
        .header("x-bot-token", format!("{token}", token = token))
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .await;

        res.map(|_| ())
    }
}