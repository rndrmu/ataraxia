use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::sync::mpsc;

use crate::config::Config;
use crate::player::Player;
use crate::protocol::OutgoingEvent;

pub type SessionId = String;
pub type GuildId = String;

pub struct Session {
    pub id: SessionId,
    pub event_tx: mpsc::UnboundedSender<OutgoingEvent>,
    pub players: Arc<Mutex<HashMap<GuildId, Player>>>,
    pub resume_key: Option<String>,
}

impl Session {
    pub fn new(id: SessionId, event_tx: mpsc::UnboundedSender<OutgoingEvent>) -> Self {
        Self {
            id,
            event_tx,
            players: Arc::new(Mutex::new(HashMap::new())),
            resume_key: None,
        }
    }

    pub async fn get_or_create_player(
        &self,
        guild_id: &str,
        channel_id: &str,
        config: Arc<Config>,
    ) -> tokio::sync::MutexGuard<'_, HashMap<GuildId, Player>> {
        let mut players = self.players.lock().await;
        if !players.contains_key(guild_id) {
            let player = Player::new(
                guild_id.to_string(),
                channel_id.to_string(),
                self.event_tx.clone(),
                config,
            );
            players.insert(guild_id.to_string(), player);
        }
        players
    }
}

pub struct SessionManager {
    sessions: RwLock<HashMap<SessionId, Arc<Session>>>,
}

impl SessionManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            sessions: RwLock::new(HashMap::new()),
        })
    }

    pub async fn create(&self, id: SessionId, event_tx: mpsc::UnboundedSender<OutgoingEvent>) -> Arc<Session> {
        let session = Arc::new(Session::new(id.clone(), event_tx));
        self.sessions.write().await.insert(id, session.clone());
        session
    }

    pub async fn get(&self, id: &str) -> Option<Arc<Session>> {
        self.sessions.read().await.get(id).cloned()
    }

    pub async fn remove(&self, id: &str) {
        self.sessions.write().await.remove(id);
    }

    pub async fn count(&self) -> usize {
        self.sessions.read().await.len()
    }

    pub async fn playing_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        let mut count = 0;
        for session in sessions.values() {
            let players = session.players.lock().await;
            count += players.values()
                .filter(|p| p.status == crate::player::PlayerStatus::Playing)
                .count();
        }
        count
    }
}
