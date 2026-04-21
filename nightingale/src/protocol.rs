use serde::{Deserialize, Serialize};

// ── Bot → Node ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum IncomingMessage {
    #[serde(rename_all = "camelCase")]
    VoiceUpdate {
        guild_id: String,
        channel_id: String,
    },
    // Bot resolves join_call itself and hands Nightingale the LiveKit credentials directly.
    #[serde(rename_all = "camelCase")]
    VoiceConnect {
        guild_id: String,
        channel_id: String,
        url: String,
        token: String,
    },
    #[serde(rename_all = "camelCase")]
    Play {
        guild_id: String,
        track: EncodedTrack,
        #[serde(default)]
        start_time: u64,
        #[serde(default)]
        end_time: Option<u64>,
        #[serde(default = "default_volume")]
        volume: u8,
        #[serde(default)]
        pause: bool,
        #[serde(default)]
        no_replace: bool,
    },
    #[serde(rename_all = "camelCase")]
    Stop {
        guild_id: String,
    },
    #[serde(rename_all = "camelCase")]
    Pause {
        guild_id: String,
        pause: bool,
    },
    #[serde(rename_all = "camelCase")]
    Seek {
        guild_id: String,
        position: u64,
    },
    #[serde(rename_all = "camelCase")]
    Volume {
        guild_id: String,
        volume: u8,
    },
    #[serde(rename_all = "camelCase")]
    Destroy {
        guild_id: String,
    },
    #[serde(rename_all = "camelCase")]
    ConfigureResuming {
        key: String,
        timeout: u64,
    },
}

fn default_volume() -> u8 { 100 }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EncodedTrack {
    pub encoded: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<serde_json::Value>,
}

// ── Node → Bot ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum OutgoingEvent {
    Ready {
        resumed: bool,
        session_id: String,
    },
    PlayerUpdate {
        guild_id: String,
        state: PlayerState,
    },
    TrackStart {
        guild_id: String,
        track: TrackInfo,
    },
    TrackEnd {
        guild_id: String,
        track: TrackInfo,
        reason: TrackEndReason,
    },
    TrackException {
        guild_id: String,
        track: TrackInfo,
        exception: TrackException,
    },
    Stats(NodeStats),
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerState {
    pub time: u64,
    pub position: u64,
    pub connected: bool,
    pub paused: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackInfo {
    pub identifier: String,
    pub title: String,
    pub author: String,
    pub length: u64,
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artwork_url: Option<String>,
    pub is_stream: bool,
    pub source_name: String,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TrackEndReason {
    Finished,
    Stopped,
    Replaced,
    LoadFailed,
    Cleanup,
}

#[derive(Debug, Serialize, Clone)]
pub struct TrackException {
    pub message: String,
    pub severity: ExceptionSeverity,
    pub cause: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ExceptionSeverity {
    Common,
    Suspicious,
    Fault,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct NodeStats {
    pub players: usize,
    pub playing_players: usize,
    pub uptime: u64,
}
