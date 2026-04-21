use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::player::PlayerStatus;
use crate::protocol::TrackInfo;
use crate::resolver;

// ── /v1/loadtracks ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoadTracksQuery {
    pub identifier: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadTracksResponse {
    pub load_type: LoadType,
    pub data: Option<LoadTracksData>,
    pub error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LoadType {
    Track,
    Empty,
    Error,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadTracksData {
    pub encoded: String,
    pub info: TrackInfo,
}

pub async fn load_tracks(
    State(state): State<AppState>,
    Query(q): Query<LoadTracksQuery>,
) -> impl IntoResponse {
    match resolver::resolve(&q.identifier, &state.config.audio.ytdlp_path).await {
        Ok(resolved) => {
            let encoded = base64_encode(&resolved.info);
            Json(LoadTracksResponse {
                load_type: LoadType::Track,
                data: Some(LoadTracksData { encoded, info: resolved.info }),
                error: None,
            })
        }
        Err(e) => Json(LoadTracksResponse {
            load_type: LoadType::Error,
            data: None,
            error: Some(e),
        }),
    }
}

// ── /v1/sessions/:id/players/:guild ──────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchPlayer {
    pub track: Option<PatchTrack>,
    pub position: Option<u64>,
    pub volume: Option<u8>,
    pub paused: Option<bool>,
    pub voice: Option<PatchVoice>,
    #[serde(default)]
    pub no_replace: bool,
}

#[derive(Deserialize)]
pub struct PatchTrack {
    pub encoded: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchVoice {
    pub channel_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponse {
    pub guild_id: String,
    pub channel_id: String,
    pub status: String,
    pub position: u64,
    pub volume: u8,
    pub paused: bool,
    pub track: Option<TrackInfo>,
}

pub async fn get_player(
    State(state): State<AppState>,
    Path((session_id, guild_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let session = match state.sessions.get(&session_id).await {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "session not found").into_response(),
    };
    let players = session.players.lock().await;
    match players.get(&guild_id) {
        Some(p) => Json(player_response(p)).into_response(),
        None => (StatusCode::NOT_FOUND, "player not found").into_response(),
    }
}

pub async fn patch_player(
    State(state): State<AppState>,
    Path((session_id, guild_id)): Path<(String, String)>,
    Json(body): Json<PatchPlayer>,
) -> impl IntoResponse {
    let session = match state.sessions.get(&session_id).await {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "session not found").into_response(),
    };

    // Connect to voice channel if requested
    if let Some(voice) = &body.voice {
        let mut players = session
            .get_or_create_player(&guild_id, &voice.channel_id, state.config.clone())
            .await;
        let player = players.get_mut(&guild_id).unwrap();
        // Voice connection is established via VoiceConnect WS op by the bot client.
        // REST PATCH with voice field is a no-op for now.
    }

    let mut players = session.players.lock().await;
    let player = match players.get_mut(&guild_id) {
        Some(p) => p,
        None => return (StatusCode::NOT_FOUND, "player not found — send voice first").into_response(),
    };

    if let Some(v) = body.volume {
        player.volume = v;
    }
    if let Some(paused) = body.paused {
        player.set_pause(paused);
    }
    if let Some(pos) = body.position {
        let track = player.current_track.clone();
        if let Some(t) = track {
            player.play(&t.uri.clone(), pos).await;
        }
    }
    if let Some(track) = body.track {
        if !(body.no_replace && player.status == PlayerStatus::Playing) {
            player.play(&track.encoded, 0).await;
        }
    }

    Json(player_response(player)).into_response()
}

pub async fn delete_player(
    State(state): State<AppState>,
    Path((session_id, guild_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let session = match state.sessions.get(&session_id).await {
        Some(s) => s,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let mut players = session.players.lock().await;
    if let Some(mut player) = players.remove(&guild_id) {
        player.disconnect().await;
    }
    StatusCode::NO_CONTENT.into_response()
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn player_response(p: &crate::player::Player) -> PlayerResponse {
    PlayerResponse {
        guild_id: p.guild_id.clone(),
        channel_id: p.channel_id.clone(),
        status: format!("{:?}", p.status),
        position: p.position(),
        volume: p.volume,
        paused: p.status == PlayerStatus::Paused,
        track: p.current_track.clone(),
    }
}

fn base64_encode(info: &TrackInfo) -> String {
    use std::io::Write;
    let json = serde_json::to_vec(info).unwrap_or_default();
    // simple base64 without external dep — use std's built-in via a byte-by-byte encode
    base64_bytes(&json)
}

fn base64_bytes(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(CHARS[(b0 >> 2)] as char);
        out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        out.push(if chunk.len() > 1 { CHARS[((b1 & 0xf) << 2) | (b2 >> 6)] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[b2 & 0x3f] as char } else { '=' });
    }
    out
}
