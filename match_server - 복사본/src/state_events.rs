use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

// Channel prefixes
const EVENTS_QUEUE_PREFIX: &str = "events:queue:";
const EVENTS_SESSION_PREFIX: &str = "events:session:";
const EVENTS_VIOLATION_PREFIX: &str = "events:violation:";

// Runtime toggle for enabling/disabling state events
static STATE_EVENTS_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn set_state_events_enabled(enabled: bool) {
    STATE_EVENTS_ENABLED.store(enabled, Ordering::Relaxed);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StateEvent {
    #[serde(rename = "queue_size_changed")]
    QueueSizeChanged {
        ts: DateTime<Utc>,
        game_mode: String,
        size: usize,
    },

    #[serde(rename = "loading_session_created")]
    LoadingSessionCreated {
        ts: DateTime<Utc>,
        session_id: String,
        game_mode: String,
        players: Vec<String>,
        ttl_seconds: u64,
    },

    #[serde(rename = "player_ready")]
    PlayerReady {
        ts: DateTime<Utc>,
        session_id: String,
        player_id: String,
    },

    #[serde(rename = "loading_session_completed")]
    LoadingSessionCompleted {
        ts: DateTime<Utc>,
        session_id: String,
        players: Vec<String>,
    },

    #[serde(rename = "loading_session_timeout")]
    LoadingSessionTimeout {
        ts: DateTime<Utc>,
        session_id: String,
        timed_out_players: Vec<String>,
    },

    #[serde(rename = "players_requeued")]
    PlayersRequeued {
        ts: DateTime<Utc>,
        game_mode: String,
        players: Vec<String>,
    },

    #[serde(rename = "dedicated_session_created")]
    DedicatedSessionCreated {
        ts: DateTime<Utc>,
        session_id: String,
        server_address: String,
    },

    #[serde(rename = "dedicated_session_failed")]
    DedicatedSessionFailed {
        ts: DateTime<Utc>,
        session_id: String,
        reason: String,
    },

    #[serde(rename = "loading_session_canceled")]
    LoadingSessionCanceled {
        ts: DateTime<Utc>,
        session_id: String,
        reason: String,
        players: Vec<String>,
    },

    #[serde(rename = "state_violation")]
    StateViolation {
        ts: DateTime<Utc>,
        code: String,
        details: serde_json::Value,
    },
}

impl StateEvent {
    /// 현재 시각으로 타임스탬프를 자동 설정하는 헬퍼 함수들
    pub fn queue_size_changed<S: ToString>(game_mode: S, size: usize) -> Self {
        Self::QueueSizeChanged {
            ts: Utc::now(),
            game_mode: game_mode.to_string(),
            size,
        }
    }

    pub fn loading_session_created<S1: ToString, S2: ToString>(
        session_id: S1,
        game_mode: S2,
        players: Vec<String>,
        ttl_seconds: u64,
    ) -> Self {
        Self::LoadingSessionCreated {
            ts: Utc::now(),
            session_id: session_id.to_string(),
            game_mode: game_mode.to_string(),
            players,
            ttl_seconds,
        }
    }

    pub fn player_ready<S1: ToString, S2: ToString>(session_id: S1, player_id: S2) -> Self {
        Self::PlayerReady {
            ts: Utc::now(),
            session_id: session_id.to_string(),
            player_id: player_id.to_string(),
        }
    }

    pub fn loading_session_completed<S: ToString>(session_id: S, players: Vec<String>) -> Self {
        Self::LoadingSessionCompleted {
            ts: Utc::now(),
            session_id: session_id.to_string(),
            players,
        }
    }

    pub fn loading_session_timeout<S: ToString>(
        session_id: S,
        timed_out_players: Vec<String>,
    ) -> Self {
        Self::LoadingSessionTimeout {
            ts: Utc::now(),
            session_id: session_id.to_string(),
            timed_out_players,
        }
    }

    pub fn players_requeued<S: ToString>(game_mode: S, players: Vec<String>) -> Self {
        Self::PlayersRequeued {
            ts: Utc::now(),
            game_mode: game_mode.to_string(),
            players,
        }
    }

    pub fn dedicated_session_created<S1: ToString, S2: ToString>(
        session_id: S1,
        server_address: S2,
    ) -> Self {
        Self::DedicatedSessionCreated {
            ts: Utc::now(),
            session_id: session_id.to_string(),
            server_address: server_address.to_string(),
        }
    }

    pub fn dedicated_session_failed<S1: ToString, S2: ToString>(
        session_id: S1,
        reason: S2,
    ) -> Self {
        Self::DedicatedSessionFailed {
            ts: Utc::now(),
            session_id: session_id.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn loading_session_canceled<S1: ToString, S2: ToString>(
        session_id: S1,
        reason: S2,
        players: Vec<String>,
    ) -> Self {
        Self::LoadingSessionCanceled {
            ts: Utc::now(),
            session_id: session_id.to_string(),
            reason: reason.to_string(),
            players,
        }
    }

    pub fn state_violation<S: ToString>(code: S, details: serde_json::Value) -> Self {
        Self::StateViolation {
            ts: Utc::now(),
            code: code.to_string(),
            details,
        }
    }
}

pub fn get_channel_for_event(event: &StateEvent) -> String {
    match event {
        // 큐 관련 이벤트
        StateEvent::QueueSizeChanged { game_mode, .. } => {
            format!("{}{}", EVENTS_QUEUE_PREFIX, game_mode)
        }
        StateEvent::PlayersRequeued { game_mode, .. } => {
            format!("{}{}", EVENTS_QUEUE_PREFIX, game_mode)
        }

        // 세션 관련 이벤트
        StateEvent::LoadingSessionCreated { session_id, .. } => {
            format!("{}{}", EVENTS_SESSION_PREFIX, session_id)
        }
        StateEvent::PlayerReady { session_id, .. } => {
            format!("{}{}", EVENTS_SESSION_PREFIX, session_id)
        }
        StateEvent::LoadingSessionCompleted { session_id, .. } => {
            format!("{}{}", EVENTS_SESSION_PREFIX, session_id)
        }
        StateEvent::LoadingSessionTimeout { session_id, .. } => {
            format!("{}{}", EVENTS_SESSION_PREFIX, session_id)
        }
        StateEvent::DedicatedSessionCreated { session_id, .. } => {
            format!("{}{}", EVENTS_SESSION_PREFIX, session_id)
        }
        StateEvent::DedicatedSessionFailed { session_id, .. } => {
            format!("{}{}", EVENTS_SESSION_PREFIX, session_id)
        }
        StateEvent::LoadingSessionCanceled { session_id, .. } => {
            format!("{}{}", EVENTS_SESSION_PREFIX, session_id)
        }
        StateEvent::StateViolation { code, .. } => {
            format!("{}{}", EVENTS_VIOLATION_PREFIX, code)
        }
    }
}

pub async fn publish_state_event(
    redis_client: &mut redis::aio::ConnectionManager,
    event: StateEvent,
) -> Result<(), redis::RedisError> {
    if !STATE_EVENTS_ENABLED.load(Ordering::Relaxed) {
        // Feature disabled: treat as no-op
        return Ok(());
    }
    let channel = get_channel_for_event(&event);

    let payload = serde_json::to_string(&event).map_err(|e| {
        redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "JSON serialization failed",
            e.to_string(),
        ))
    })?;

    let _: i32 = redis_client.publish(&channel, &payload).await?;

    tracing::debug!("Published state event to {}: {}", channel, payload);

    Ok(())
}

pub async fn publish_queue_size_changed(
    redis_client: &mut redis::aio::ConnectionManager,
    game_mode: String,
    size: usize,
) -> Result<(), redis::RedisError> {
    let event = StateEvent::queue_size_changed(game_mode, size);
    publish_state_event(redis_client, event).await
}

pub async fn publish_loading_session_created(
    redis_client: &mut redis::aio::ConnectionManager,
    session_id: String,
    game_mode: String,
    players: Vec<String>,
    ttl_seconds: u64,
) -> Result<(), redis::RedisError> {
    let event = StateEvent::loading_session_created(session_id, game_mode, players, ttl_seconds);
    publish_state_event(redis_client, event).await
}

pub async fn publish_player_ready(
    redis_client: &mut redis::aio::ConnectionManager,
    session_id: String,
    player_id: String,
) -> Result<(), redis::RedisError> {
    let event = StateEvent::player_ready(session_id, player_id);
    publish_state_event(redis_client, event).await
}

pub async fn publish_loading_session_completed(
    redis_client: &mut redis::aio::ConnectionManager,
    session_id: String,
    players: Vec<String>,
) -> Result<(), redis::RedisError> {
    let event = StateEvent::loading_session_completed(session_id, players);
    publish_state_event(redis_client, event).await
}

pub struct StateEventEmitter<'a> {
    redis: &'a mut redis::aio::ConnectionManager,
}

impl<'a> StateEventEmitter<'a> {
    pub fn new(redis: &'a mut redis::aio::ConnectionManager) -> Self {
        Self { redis }
    }

    pub async fn publish(&mut self, event: StateEvent) -> Result<(), redis::RedisError> {
        publish_state_event(self.redis, event).await
    }

    pub async fn queue_size_changed(
        &mut self,
        game_mode: impl ToString,
        size: usize,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::queue_size_changed(game_mode, size))
            .await
    }

    pub async fn loading_session_created(
        &mut self,
        session_id: impl ToString,
        game_mode: impl ToString,
        players: Vec<String>,
        ttl_seconds: u64,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::loading_session_created(
            session_id,
            game_mode,
            players,
            ttl_seconds,
        ))
        .await
    }

    pub async fn player_ready(
        &mut self,
        session_id: impl ToString,
        player_id: impl ToString,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::player_ready(session_id, player_id))
            .await
    }

    pub async fn loading_session_completed(
        &mut self,
        session_id: impl ToString,
        players: Vec<String>,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::loading_session_completed(session_id, players))
            .await
    }

    pub async fn loading_session_timeout(
        &mut self,
        session_id: impl ToString,
        timed_out_players: Vec<String>,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::loading_session_timeout(
            session_id,
            timed_out_players,
        ))
        .await
    }

    pub async fn players_requeued(
        &mut self,
        game_mode: impl ToString,
        players: Vec<String>,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::players_requeued(game_mode, players))
            .await
    }

    pub async fn dedicated_session_created(
        &mut self,
        session_id: impl ToString,
        server_address: impl ToString,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::dedicated_session_created(
            session_id,
            server_address,
        ))
        .await
    }

    pub async fn dedicated_session_failed(
        &mut self,
        session_id: impl ToString,
        reason: impl ToString,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::dedicated_session_failed(session_id, reason))
            .await
    }

    pub async fn loading_session_canceled(
        &mut self,
        session_id: impl ToString,
        reason: impl ToString,
        players: Vec<String>,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::loading_session_canceled(
            session_id, reason, players,
        ))
        .await
    }

    pub async fn state_violation(
        &mut self,
        code: impl ToString,
        details: serde_json::Value,
    ) -> Result<(), redis::RedisError> {
        self.publish(StateEvent::state_violation(code, details))
            .await
    }
}
