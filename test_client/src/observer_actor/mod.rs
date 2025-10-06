use actix::{Actor, Addr, Context};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use tracing::{info, warn};
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

            ET::StartLoading => {
                if let (Some(pid), Some(session)) = (
                    event.player_id,
                    event
                        .data
                        .get("loading_session_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                ) {
                    let ok =
                        check_start_loading_against_created(&self.state_sessions, &session, pid);
                    if !ok {
                        let msg = format!(
                            "Soft check failed: player {} StartLoading for unknown/mismatched session {}",
                            pid, session
                        );
                        warn!("{}", msg);
                        self.consistency_warnings.push(msg);
                    }
                }
            }
            ET::QueueSizeChanged => {
                // 캐시 갱신
                if let Some((gm, size)) = self.extract_queue_info(event) {
                    self.last_queue_size.insert(gm, (size, event.timestamp));
                }
                // 최근 플레이어 액션과 비교하여 큐 사이즈 변화 검증
                info!("[{}] Processing QueueSizeChanged event for consistency check.", self.test_name);
            }

            _ => {}
        }
    }

    /// QueueSizeChanged 이벤트를 최근 플레이어 액션과 비교하여 검증
    fn _validate_queue_size_change(&self, current_event: &EventStreamMessage) {
        // 현재 큐 정보 추출
        let (game_mode, current_size) = match self.extract_queue_info(current_event) {
            Some(info) => info,
            None => return, // 잘못된 이벤트 형식
        };

        // 이 게임 모드의 이전 큐 사이즈 찾기
        let previous_size = self.find_previous_queue_size(&game_mode);
        let actual_change = current_size - previous_size;

        // 최근 플레이어 액션들을 기반으로 예상 변화량 계산
        let expected_change =
            self.calculate_expected_queue_change(&game_mode, current_event.timestamp);

        // 검증
        if actual_change == expected_change {
            info!(
                "[{}] ✓ 큐 사이즈 검증 통과 {}: {} -> {} (변화량: {})",
                self.test_name, game_mode, previous_size, current_size, actual_change
            );
        } else {
            let warning = format!(
                "큐 사이즈 검증 실패 {}: 예상 변화량 {}, 실제 변화량 {} ({}->{})",
                game_mode, expected_change, actual_change, previous_size, current_size
            );
            warn!("[{}] {}", self.test_name, warning);
            // 현재는 정보 제공용이므로 consistency_warnings에 추가하지 않음
        }
    }

    /// QueueSizeChanged 이벤트에서 game_mode와 큐 사이즈 추출
    fn extract_queue_info(&self, event: &EventStreamMessage) -> Option<(String, i64)> {
        let game_mode = event.data.get("game_mode")?.as_str()?.to_string();
        let size = event.data.get("size")?.as_i64()?;
        Some((game_mode, size))
    }

    /// 특정 게임 모드의 가장 최근 큐 사이즈 찾기(O(1) 캐시 이용)
    fn find_previous_queue_size(&self, target_game_mode: &str) -> i64 {
        self.last_queue_size
            .get(target_game_mode)
            .map(|(size, _ts)| *size)
            .unwrap_or(0)
    }

    /// 최근 플레이어 액션들을 기반으로 예상 큐 변화량 계산(링버퍼에서 필요한 범위만)
    fn calculate_expected_queue_change(
        &self,
        target_game_mode: &str,
        current_timestamp: chrono::DateTime<chrono::Utc>,
    ) -> i64 {
        use crate::observer_actor::message::EventType as ET;

        // 이전 QueueSizeChanged 이벤트의 타임스탬프 찾기(캐시)
        let previous_timestamp = self
            .last_queue_size
            .get(target_game_mode)
            .map(|(_size, ts)| *ts)
            .unwrap_or(chrono::DateTime::<chrono::Utc>::MIN_UTC);

        let mut expected_change = 0i64;

        // 링버퍼 순회(최근 이벤트만)
        for event in self.received_events.iter() {
            // 시간 윈도우 내의 이벤트만 고려
            if event.timestamp <= previous_timestamp || event.timestamp >= current_timestamp {
                continue;
            }

            // 이 이벤트가 대상 게임 모드에 영향을 주는지 확인
            let affects_game_mode = match event.event_type {
                ET::Enqueued | ET::Dequeued | ET::StartLoading => {
                    // TODO: player_id -> game_mode 매핑이 가능해지면 대체
                    target_game_mode == "Normal_1v1"
                }
                ET::PlayersRequeued => event
                    .data
                    .get("game_mode")
                    .and_then(|v| v.as_str())
                    .map(|mode| mode == target_game_mode)
                    .unwrap_or(false),
                _ => false,
            };

            if affects_game_mode {
                expected_change += match event.event_type {
                    ET::Enqueued => 1,
                    ET::Dequeued => -1,
                    ET::StartLoading => -1,
                    ET::PlayersRequeued => event
                        .data
                        .get("players")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len() as i64)
                        .unwrap_or(0),
                    _ => 0,
                };
            }
        }

        expected_change
    }
}

/// 단위 테스트용 순수 헬퍼 함수: LoadingSessionCreated 상태와 비교하여 StartLoading 검증
pub fn check_start_loading_against_created(
    sessions: &HashMap<String, HashSet<Uuid>>,
    session_id: &str,
    player_id: Uuid,
) -> bool {
    if let Some(players) = sessions.get(session_id) {
        players.contains(&player_id)
    } else {
        false
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
