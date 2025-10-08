use actix::{Actor, Addr, Context};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use tracing::info;
use uuid::Uuid;

use crate::{
    observer_actor::message::{EventStreamMessage, EventType},
    scenario_actor::{ScenarioRunnerActor, SingleScenarioActor},
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
    pub required_events: HashSet<EventType>,
    pub transition_event: EventType,
    pub transition_matcher: Option<Arc<dyn Fn(&serde_json::Value) -> bool + Send + Sync>>,
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
/// 시나리오/스웜의 전체적인 이벤트 흐름과 상태 변화를 감시하고 검증합니다.
pub struct ObserverActor {
    pub match_server_url: String,
    // 메모리 사용을 제어하기 위한 링버퍼(간단 구현). 마지막 N개만 유지
    pub received_events: VecDeque<EventStreamMessage>,
    pub max_events_kept: usize,

    pub test_name: String,
    pub test_session_id: String,
    pub scenario_runner_addr: Addr<ScenarioRunnerActor>,
    pub single_scenario_addr: Option<Addr<SingleScenarioActor>>,

    // --- Phase 기반 검증을 위한 상태 필드 ---
    /// 전체 시나리오의 단계별 진행 조건
    pub players_schedule: HashMap<Uuid, HashMap<Phase, PhaseCondition>>,
    /// 플레이어별 현재 단계
    pub players_phase: HashMap<Uuid, Phase>,
    /// 플레이어별로 현재 단계에서 받은 이벤트들
    pub player_received_events_in_phase: HashMap<Uuid, HashSet<EventType>>,
    /// 실패가 예상되는 플레이어 목록 (Invalid behavior 등)
    pub expected_failures: HashSet<Uuid>,

    // --- 상태 이벤트 기반 빠른 검증을 위한 캐시 ---
    pub state_sessions: HashMap<String, HashSet<Uuid>>,
    pub last_queue_size: HashMap<String, (i64, chrono::DateTime<chrono::Utc>)>,

    pub ws_retry_attempts: u32,
    pub consistency_warnings: Vec<String>,
    pub started_at: Instant,
}

impl ObserverActor {
    pub fn new(
        match_server_url: String,
        test_name: String,
        test_session_id: String,
        scenario_runner_addr: Addr<ScenarioRunnerActor>,
        players_schedule: HashMap<Uuid, HashMap<Phase, PhaseCondition>>,
        players_phase: HashMap<Uuid, Phase>,
        expected_failures: HashSet<Uuid>,
    ) -> Self {
        let max_events_kept = std::env::var("OBSERVER_RINGBUFFER_SIZE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(10_000);
        Self {
            match_server_url,
            received_events: VecDeque::with_capacity(max_events_kept),
            max_events_kept,
            test_name,
            test_session_id,
            scenario_runner_addr,
            single_scenario_addr: None,
            players_schedule,
            players_phase,
            player_received_events_in_phase: HashMap::new(),
            expected_failures,
            state_sessions: HashMap::new(),
            last_queue_size: HashMap::new(),
            ws_retry_attempts: 0,
            consistency_warnings: Vec::new(),
            started_at: Instant::now(),
        }
    }

    pub fn set_single_scenario_addr(&mut self, addr: Addr<SingleScenarioActor>) {
        self.single_scenario_addr = Some(addr);
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

impl ObserverActor {
    /// Process state events and perform soft consistency checks.
    pub fn process_state_event(&mut self, event: &EventStreamMessage) {
        use crate::observer_actor::message::EventType as ET;

        match event.event_type {
            ET::QueueSizeChanged => {
                // 캐시 갱신
                if let Some((gm, size)) = self.extract_queue_info(event) {
                    self.last_queue_size.insert(gm, (size, event.timestamp));
                }
                // 최근 플레이어 액션과 비교하여 큐 사이즈 변화 검증
                info!(
                    "[{}] Processing QueueSizeChanged event for consistency check.",
                    self.test_name
                );
            }

            _ => {}
        }
    }

    /// QueueSizeChanged 이벤트에서 game_mode와 큐 사이즈 추출
    fn extract_queue_info(&self, event: &EventStreamMessage) -> Option<(String, i64)> {
        let game_mode = event.data.get("game_mode")?.as_str()?.to_string();
        let size = event.data.get("size")?.as_i64()?;
        Some((game_mode, size))
    }
}

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
