use actix::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{observer_actor::ObservationResult, BehaviorResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Matchmaking
    Enqueued,
    Dequeued,
    MatchFound,

    ServerMessage,
    Error,

    // State events (events:*)
    QueueSizeChanged,
    PlayerReady,
    PlayersRequeued,
    StateViolation,

    #[serde(other)]
    Unknown,
}

// WebSocket을 통해 서버로부터 받는 이벤트 메시지
#[derive(Serialize, Deserialize, Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct EventStreamMessage {
    pub event_type: EventType,
    pub player_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

// SingleScenarioActor가 ObserverActor에게 관찰 시작을 알리는 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct StartObservation {
    pub player_ids: Vec<Uuid>, // 관찰할 플레이어들의 ID 목록
}

// 관찰 결과를 담아 SingleScenarioActor에게 보내는 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct ObservationCompleted(pub ObservationResult);

// 내부적으로 WebSocket 스트림에서 받은 메시지를 처리하기 위한 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub(super) struct InternalEvent(pub EventStreamMessage);

// SingleScenarioActor 주소를 설정하는 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct SetSingleScenarioAddr {
    pub addr: actix::Addr<crate::scenario_actor::SingleScenarioActor>,
}

// 관찰을 중단하고 WebSocket 스트림을 종료하기 위한 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct StopObservation;

// PlayerActor -> ObserverActor: notify per-player behavior completion so Observer can forward to SingleScenarioActor
#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerFinishedFromActor {
    pub player_id: uuid::Uuid,
    pub result: BehaviorResult,
}
