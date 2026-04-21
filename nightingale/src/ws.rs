use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use tokio::sync::mpsc;
use tracing::{error, info};
use uuid::Uuid;

use crate::config::Config;
use crate::protocol::{IncomingMessage, OutgoingEvent};
use crate::session::SessionManager;

pub async fn handle_socket(
    socket: WebSocket,
    sessions: Arc<SessionManager>,
    config: Arc<Config>,
    resume_key: Option<String>,
) {
    let session_id = Uuid::new_v4().to_string();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<OutgoingEvent>();

    let session = sessions.create(session_id.clone(), event_tx.clone()).await;

    // Send ready event
    let _ = event_tx.send(OutgoingEvent::Ready {
        resumed: false,
        session_id: session_id.clone(),
    });

    use futures_util::StreamExt as _;
    use futures_util::SinkExt as _;
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Outgoing: forward events to WS
    let send_task = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(e) => { error!("serialize error: {}", e); continue; }
            };
            use axum::extract::ws::Message;
            if ws_tx.send(Message::Text(json)).await.is_err() { break; }
        }
    });

    // Incoming: handle commands
    while let Some(Ok(msg)) = ws_rx.next().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let op: IncomingMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => { error!("parse error: {} — raw: {}", e, text); continue; }
        };

        handle_op(op, &session, config.clone()).await;
    }

    info!("session {} disconnected", session_id);
    send_task.abort();
    sessions.remove(&session_id).await;
}

async fn handle_op(
    op: IncomingMessage,
    session: &crate::session::Session,
    config: Arc<Config>,
) {
    use IncomingMessage::*;
    match op {
        VoiceUpdate { .. } => {
            // Deprecated: bot should use VoiceConnect and resolve join_call itself.
        }
        VoiceConnect { guild_id, channel_id, url, token } => {
            let mut players = session
                .get_or_create_player(&guild_id, &channel_id, config).await;
            let player = players.get_mut(&guild_id).unwrap();
            if let Err(e) = player.connect_livekit(&url, &token).await {
                error!("livekit connect error for {}: {}", guild_id, e);
            }
        }
        Play { guild_id, track, start_time, no_replace, .. } => {
            let mut players = session.players.lock().await;
            if let Some(player) = players.get_mut(&guild_id) {
                if no_replace && player.status == crate::player::PlayerStatus::Playing {
                    return;
                }
                player.play(&track.encoded, start_time).await;
            }
        }
        Stop { guild_id } => {
            let mut players = session.players.lock().await;
            if let Some(player) = players.get_mut(&guild_id) {
                player.stop();
            }
        }
        Pause { guild_id, pause } => {
            let mut players = session.players.lock().await;
            if let Some(player) = players.get_mut(&guild_id) {
                player.set_pause(pause);
            }
        }
        Seek { guild_id, position } => {
            let mut players = session.players.lock().await;
            if let Some(player) = players.get_mut(&guild_id) {
                let track = player.current_track.clone();
                if let Some(t) = track {
                    player.play(&t.uri, position).await;
                }
            }
        }
        Volume { guild_id, volume } => {
            let mut players = session.players.lock().await;
            if let Some(player) = players.get_mut(&guild_id) {
                player.volume = volume;
            }
        }
        Destroy { guild_id } => {
            let mut players = session.players.lock().await;
            if let Some(mut player) = players.remove(&guild_id) {
                player.disconnect().await;
            }
        }
        ConfigureResuming { key, .. } => {
            // resume key stored on session (simplified: not fully implemented)
            let _ = key;
        }
    }
}
