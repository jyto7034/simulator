use actix::{Actor, Addr, Context};
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

use crate::{
    observer_actor::message::{EventStreamMessage, ExpectEvent},
    scenario_actor::ScenarioRunnerActor,
};

pub mod handler;
pub mod message;

/// ObserverActor
///
/// ## 역할
/// 시나리오의 전체적인 이벤트 흐름과 상태 변화를 감시하고 검증합니다.
/// 1. WebSocket을 통해 매치 서버의 이벤트 스트림을 구독합니다.
/// 2. Redis 상태를 직접 확인하여 `PlayerActor`의 행동이 올바르게 반영되었는지 검증합니다.
/// 3. `PlayerActor`로부터 검증 요청(`ExpectEvent`)을 받아 플레이어별로 처리합니다.
/// 4. 시나리오의 성공/실패 여부를 최종적으로 판별하여 `SingleScenarioActor`에게 보고합니다.
pub struct ObserverActor {
    pub match_server_url: String,
    pub player_expectations: HashMap<Uuid, Vec<ExpectEvent>>, // 플레이어별 기대 이벤트
    pub player_steps: HashMap<Uuid, usize>,                   // 플레이어별 현재 step
    pub received_events: Vec<EventStreamMessage>,
    pub test_name: String,
    pub scenario_runner_addr: Addr<ScenarioRunnerActor>, // 결과 보고용
}

impl ObserverActor {
    pub fn new(
        match_server_url: String,
        test_name: String,
        scenario_runner_addr: Addr<ScenarioRunnerActor>,
    ) -> Self {
        Self {
            match_server_url,
            player_expectations: HashMap::new(),
            player_steps: HashMap::new(),
            received_events: Vec::new(),
            test_name,
            scenario_runner_addr,
        }
    }
}

impl Actor for ObserverActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("[{}] ObserverActor started.", self.test_name);
        // `StartObservation` 메시지를 받으면 WebSocket 연결 및 구독 시작
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("[{}] ObserverActor stopped.", self.test_name);
        // 최종 결과 보고
        // let result = self.summarize_result();
        // self.scenario_runner_addr.do_send(ScenarioCompleted { ... });
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
