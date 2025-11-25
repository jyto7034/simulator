use actix::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::BehaviorOutcome;

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerFinished {
    pub player_id: Uuid,
    pub result: BehaviorOutcome,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartObservation {
    pub player_ids: Vec<Uuid>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopObservation;

#[derive(Message)]
#[rtype(result = "()")]
pub struct InternalEvent(pub EventStreamMessage);

#[derive(Serialize, Deserialize, Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct EventStreamMessage {
    pub event_type: EventType,
    pub player_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Player events (player.*)
    #[serde(rename = "player.enqueued")]
    PlayerEnqueued,
    #[serde(rename = "player.re_enqueued")]
    PlayerReEnqueued,
    #[serde(rename = "player.dequeued")]
    PlayerDequeued,
    #[serde(rename = "player.match_found")]
    PlayerMatchFound,
    #[serde(rename = "player.error")]
    PlayerError,

    // Global events (global.*)
    #[serde(rename = "global.queue_size_changed")]
    GlobalQueueSizeChanged,

    // Legacy compatibility
    ServerMessage,

    #[serde(other)]
    Unknown,
}
