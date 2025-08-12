use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

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
}

impl StateEvent {
    /// 현재 시각으로 타임스탬프를 자동 설정하는 헬퍼 함수들
    pub fn queue_size_changed(game_mode: String, size: usize) -> Self {
        Self::QueueSizeChanged {
            ts: Utc::now(),
            game_mode,
            size,
        }
    }

    pub fn loading_session_created(
        session_id: String,
        game_mode: String,
        players: Vec<String>,
        ttl_seconds: u64,
    ) -> Self {
        Self::LoadingSessionCreated {
            ts: Utc::now(),
            session_id,
            game_mode,
            players,
            ttl_seconds,
        }
    }

    pub fn player_ready(session_id: String, player_id: String) -> Self {
        Self::PlayerReady {
            ts: Utc::now(),
            session_id,
            player_id,
        }
    }

    pub fn loading_session_completed(session_id: String, players: Vec<String>) -> Self {
        Self::LoadingSessionCompleted {
            ts: Utc::now(),
            session_id,
            players,
        }
    }

    pub fn loading_session_timeout(session_id: String, timed_out_players: Vec<String>) -> Self {
        Self::LoadingSessionTimeout {
            ts: Utc::now(),
            session_id,
            timed_out_players,
        }
    }

    pub fn players_requeued(game_mode: String, players: Vec<String>) -> Self {
        Self::PlayersRequeued {
            ts: Utc::now(),
            game_mode,
            players,
        }
    }

    pub fn dedicated_session_created(session_id: String, server_address: String) -> Self {
        Self::DedicatedSessionCreated {
            ts: Utc::now(),
            session_id,
            server_address,
        }
    }

    pub fn dedicated_session_failed(session_id: String, reason: String) -> Self {
        Self::DedicatedSessionFailed {
            ts: Utc::now(),
            session_id,
            reason,
        }
    }
}

pub fn get_channel_for_event(event: &StateEvent) -> String {
    match event {
        // 큐 관련 이벤트
        StateEvent::QueueSizeChanged { game_mode, .. } => {
            format!("events:queue:{}", game_mode)
        }
        StateEvent::PlayersRequeued { game_mode, .. } => {
            format!("events:queue:{}", game_mode)
        }

        // 세션 관련 이벤트
        StateEvent::LoadingSessionCreated { session_id, .. } => {
            format!("events:session:{}", session_id)
        }
        StateEvent::PlayerReady { session_id, .. } => {
            format!("events:session:{}", session_id)
        }
        StateEvent::LoadingSessionCompleted { session_id, .. } => {
            format!("events:session:{}", session_id)
        }
        StateEvent::LoadingSessionTimeout { session_id, .. } => {
            format!("events:session:{}", session_id)
        }
        StateEvent::DedicatedSessionCreated { session_id, .. } => {
            format!("events:session:{}", session_id)
        }
        StateEvent::DedicatedSessionFailed { session_id, .. } => {
            format!("events:session:{}", session_id)
        }
    }
}

pub async fn publish_state_event(
    redis_client: &mut redis::aio::ConnectionManager,
    event: StateEvent,
) -> Result<(), redis::RedisError> {
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

pub async fn publish_loading_session_timeout(
    redis_client: &mut redis::aio::ConnectionManager,
    session_id: String,
    timed_out_players: Vec<String>,
) -> Result<(), redis::RedisError> {
    let event = StateEvent::loading_session_timeout(session_id, timed_out_players);
    publish_state_event(redis_client, event).await
}

pub async fn publish_players_requeued(
    redis_client: &mut redis::aio::ConnectionManager,
    game_mode: String,
    players: Vec<String>,
) -> Result<(), redis::RedisError> {
    let event = StateEvent::players_requeued(game_mode, players);
    publish_state_event(redis_client, event).await
}

pub async fn publish_dedicated_session_created(
    redis_client: &mut redis::aio::ConnectionManager,
    session_id: String,
    server_address: String,
) -> Result<(), redis::RedisError> {
    let event = StateEvent::dedicated_session_created(session_id, server_address);
    publish_state_event(redis_client, event).await
}

pub async fn publish_dedicated_session_failed(
    redis_client: &mut redis::aio::ConnectionManager,
    session_id: String,
    reason: String,
) -> Result<(), redis::RedisError> {
    let event = StateEvent::dedicated_session_failed(session_id, reason);
    publish_state_event(redis_client, event).await
}
