use actix::{Actor, Addr, Context};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

use crate::{
    observer_actor::message::{EventStreamMessage, EventType},
    scenario_actor::ScenarioRunnerActor,
};

pub mod handler;
pub mod message;

// 1. 시나리오의 단계를 정의하는 Enum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Phase {
    Matching,
    Loading,
    Finished, // 모든 과정이 끝난 최종 단계
    Failed,   // 실패한 단계
}

use std::sync::Arc;

// 2. 각 단계(Phase)의 완료 조건을 정의하는 구조체
pub struct PhaseCondition {
    // 현 단계에서 순서에 상관없이 꼭 받아야 하는 이벤트들
    pub required_events: HashSet<EventType>,
    // 이 이벤트가 발생하고, required_events가 모두 충족되었을 때 다음 단계로 전환
    pub transition_event: EventType,
    // transition_event의 data 필드를 검증하는 클로저
    pub transition_matcher: Option<Arc<dyn Fn(&serde_json::Value) -> bool + Send + Sync>>,
    // 전환될 다음 단계
    pub next_phase: Phase,
}

impl Clone for PhaseCondition {
    fn clone(&self) -> Self {
        Self {
            required_events: self.required_events.clone(),
            transition_event: self.transition_event.clone(),
            transition_matcher: self.transition_matcher.clone(),
            next_phase: self.next_phase.clone(),
        }
    }
}

/// ObserverActor
///
/// ## 역할
/// 시나리오의 전체적인 이벤트 흐름과 상태 변화를 감시하고 검증합니다.
pub struct ObserverActor {
    pub match_server_url: String,
    pub received_events: Vec<EventStreamMessage>,
    pub test_name: String,
    pub scenario_runner_addr: Addr<ScenarioRunnerActor>,

    // --- Phase 기반 검증을 위한 상태 필드 ---
    /// 전체 시나리오의 단계별 진행 조건
    pub players_schedule: HashMap<Uuid, HashMap<Phase, PhaseCondition>>,
    /// 플레이어별 현재 단계
    pub players_phase: HashMap<Uuid, Phase>,
    /// 플레이어별로 현재 단계에서 받은 이벤트들
    pub player_received_events_in_phase: HashMap<Uuid, HashSet<EventType>>,
}

impl ObserverActor {
    pub fn new(
        match_server_url: String,
        test_name: String,
        scenario_runner_addr: Addr<ScenarioRunnerActor>,
        players_schedule: HashMap<Uuid, HashMap<Phase, PhaseCondition>>,
        players_phase: HashMap<Uuid, Phase>,
    ) -> Self {
        Self {
            match_server_url,
            received_events: Vec::new(),
            test_name,
            scenario_runner_addr,
            players_schedule,
            players_phase,
            player_received_events_in_phase: HashMap::new(),
        }
    }
}

impl Actor for ObserverActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("[{}] ObserverActor started.", self.test_name);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("[{}] ObserverActor stopped.", self.test_name);
    }
}

#[derive(Debug)]
pub enum ObservationResult {
    Success {
        events: Vec<EventStreamMessage>,
        duration: Duration,
    },
    Timeout {
        failed_step: usize,
        reason: String,
        events: Vec<EventStreamMessage>,
    },
    Error {
        failed_step: usize,
        reason: String,
        events: Vec<EventStreamMessage>,
    },
}
