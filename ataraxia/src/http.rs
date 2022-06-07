use reqwest::Client;
use reqwest::{Body, Method, Request, Url};
use std::time::Duration;
use tower::Service;
use tower::ServiceExt;

pub static API_BASE_URL: &str = "https://api.revolt.chat";


#[derive(Clone)]
pub struct Http;/* T> where T: Service<Request> {
} */


/* struct RateLimiter<T> where T: Service<Request> {
    service: tower::limit::RateLimit<tower::util::ServiceFn<T>>
} */

impl Http
{

    /// Instantiates a new Http Client
    /// 
/*     pub async fn new() {
        let client = reqwest::Client::new();
        let mut svc = tower::ServiceBuilder::new()
            .rate_limit(100, Duration::new(10, 0)) // 100 requests every 10 seconds
            .service(tower::service_fn(move |req| client.execute(req)));
/*         let mut http_ratelimiter = RateLimiter { service: svc };
 */
        let mut req = Request::new(Method::POST, Url::parse("http://httpbin.org/post")?);
        *req.body_mut() = Some(Body::from("the exact body that is sent"));

/*         let b = http_ratelimiter.service.ready().await?.call(req).await?;
 */
        let res = svc.ready().await?.call(req).await;

    
    } */

    /// Sends a POST request to the revolt api
    /// ```rs
    /// let a = Http::new();
    /// let b = a.post("channel/abc/messages", "my_bot_token", {"content": "Hello World!"}).await;
    /// ```
    pub async fn post(destination: &str, token: &str, body: serde_json::Value) -> Result<(), reqwest::Error> {
        let client = Client::new();
        let res = client.post(format!("{}/{}", API_BASE_URL, destination))
        .header("x-bot-token", format!("{token}", token = token))
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .await;

        res.map(|_| ())
    }
}