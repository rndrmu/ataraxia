mod config;
mod livekit_util;
mod player;
mod protocol;
mod resolver;
mod routes;
mod session;
mod ws;

use std::sync::Arc;
use axum::{
    extract::{State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, patch},
    Router,
};
use tracing::info;

#[derive(Clone)]
struct AppState {
    sessions: Arc<session::SessionManager>,
    config: Arc<config::Config>,
    started_at: std::time::Instant,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("nightingale=info".parse().unwrap()))
        .init();

    let cfg = config::Config::load().expect("failed to load config");
    info!("starting nightingale on {}:{}", cfg.server.host, cfg.server.port);

    let state = AppState {
        sessions: session::SessionManager::new(),
        config: Arc::new(cfg.clone()),
        started_at: std::time::Instant::now(),
    };

    let password = cfg.server.password.clone();
    let auth = axum::middleware::from_fn(move |req: axum::extract::Request, next: axum::middleware::Next| {
        let password = password.clone();
        async move {
            let auth = req.headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            if auth == password {
                next.run(req).await
            } else {
                StatusCode::UNAUTHORIZED.into_response()
            }
        }
    });

    let app = Router::new()
        .route("/v1/websocket", get(ws_handler))
        .route("/v1/info", get(info_handler))
        .route("/v1/loadtracks", get(routes::load_tracks))
        .route("/v1/sessions/:session_id/players/:guild_id", get(routes::get_player))
        .route("/v1/sessions/:session_id/players/:guild_id", patch(routes::patch_player))
        .route("/v1/sessions/:session_id/players/:guild_id", delete(routes::delete_player))
        .layer(auth)
        .with_state(state);

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Password check
    let auth = headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if auth != state.config.server.password {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    let resume_key = headers.get("Resume-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    ws.on_upgrade(move |socket| {
        ws::handle_socket(socket, state.sessions, state.config, resume_key)
    })
}

async fn info_handler(State(state): State<AppState>) -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": state.started_at.elapsed().as_millis(),
    }))
}
