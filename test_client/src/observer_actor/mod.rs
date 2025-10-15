use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

use actix::{Actor, Context};
use uuid::Uuid;

use crate::observer_actor::message::EventType;

pub mod handler;
pub mod message;

// 1. 시나리오의 단계를 정의하는 Enum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Phase {
    Enqueuing,  // Enqueue 요청 ~ PlayerEnqueued 대기
    InQueue,    // PlayerEnqueued 받음, 매칭 대기중
    Matched,    // PlayerMatchFound 받음 (성공 종료 상태)
    Dequeued,   // PlayerDequeued 받음 (성공 종료 상태)
    Finished,   // 테스트 성공 완료
    Failed,     // 실패한 단계
}

// 이벤트와 해당 이벤트의 검증 로직을 묶은 구조체
#[derive(Clone)]
pub struct EventRequirement {
    pub event_type: EventType,
    pub matcher: Option<Arc<dyn Fn(&serde_json::Value) -> bool + Send + Sync>>,
}

impl EventRequirement {
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            matcher: None,
        }
    }

    pub fn with_matcher(
        event_type: EventType,
        matcher: Arc<dyn Fn(&serde_json::Value) -> bool + Send + Sync>,
    ) -> Self {
        Self {
            event_type,
            matcher: Some(matcher),
        }
    }

    /// 이벤트가 요구사항을 만족하는지 확인
    pub fn matches(&self, event_type: &EventType, data: &serde_json::Value) -> bool {
        if &self.event_type != event_type {
            return false;
        }

        self.matcher.as_ref().map_or(true, |matcher| matcher(data))
    }
}

// 2. 각 단계(Phase)의 완료 조건을 정의하는 구조체
pub struct PhaseCondition {
    /// 전환 전에 받아야 하는 필수 이벤트들
    pub required_events: Vec<EventRequirement>,
    /// 실제 전환을 발생시키는 이벤트
    pub transition_event: EventRequirement,
    /// 다음 단계
    pub next_phase: Phase,
}

pub struct ObserverActor {
    pub match_server_url: String,
    pub test_name: String,
    pub test_session_id: String,

    // --- Phase 기반 검증을 위한 상태 필드 ---
    /// 전체 시나리오의 단계별 진행 조건
    pub players_schedule: HashMap<Uuid, HashMap<Phase, PhaseCondition>>,
    /// 플레이어별 현재 단계
    pub players_phase: HashMap<Uuid, Phase>,
    /// 플레이어별로 현재 단계에서 만족한 EventRequirement의 인덱스들
    pub player_satisfied_requirements: HashMap<Uuid, HashSet<usize>>,

    pub started_at: Instant,

    /// 테스트 완료 신호를 보내기 위한 채널
    pub completion_tx: Option<tokio::sync::oneshot::Sender<bool>>,
}

impl Actor for ObserverActor {
    type Context = Context<Self>;
}

impl ObserverActor {
    pub fn new(
        match_server_url: String,
        test_name: String,
        test_session_id: String,
        players_schedule: HashMap<Uuid, HashMap<Phase, PhaseCondition>>,
        players_phase: HashMap<Uuid, Phase>,
    ) -> Self {
        Self {
            match_server_url,
            test_name,
            test_session_id,
            players_schedule,
            players_phase,
            player_satisfied_requirements: HashMap::new(),
            started_at: Instant::now(),
            completion_tx: None,
        }
    }

    pub fn with_completion_tx(mut self, tx: tokio::sync::oneshot::Sender<bool>) -> Self {
        self.completion_tx = Some(tx);
        self
    }
}

#[derive(Debug, Clone)]
pub enum TestResult {
    Success { duration: std::time::Duration },
    Failure(String),
}
