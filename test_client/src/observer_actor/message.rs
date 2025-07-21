use actix::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::observer_actor::ObservationResult;

// WebSocket을 통해 서버로부터 받는 이벤트 메시지
#[derive(Serialize, Deserialize, Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct EventStreamMessage {
    pub event_type: String,
    pub player_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

// PlayerActor가 ObserverActor에게 보내는 검증 요청
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct ExpectEvent {
    pub event_type: String,
    pub player_id: Option<Uuid>,
    // pub data_matcher: Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>,
    pub timeout: Duration,
}

// SingleScenarioActor가 ObserverActor에게 관찰 시작을 알리는 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct StartObservation {
    pub player_id_filter: Option<Uuid>, // 특정 플레이어의 이벤트만 필터링할 경우
}

// 관찰 결과를 담아 SingleScenarioActor에게 보내는 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub struct ObservationCompleted(pub ObservationResult);

// 내부적으로 WebSocket 스트림에서 받은 메시지를 처리하기 위한 메시지
#[derive(Message)]
#[rtype(result = "()")]
pub(super) struct InternalEvent(pub EventStreamMessage);
