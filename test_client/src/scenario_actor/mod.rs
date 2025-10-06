use std::collections::HashMap;

use actix::{Actor, Addr, AsyncContext, Context};
use tokio::sync::oneshot;
use tracing::info;
use uuid::Uuid;

use crate::{
    behaviors::BehaviorType,
    observer_actor::{
        message::{SetSingleScenarioAddr, StartObservation},
        ObserverActor,
    },
    player_actor::PlayerActor,
    schedules,
};

pub mod handler;
pub mod message;

/// Scenario 정의
#[derive(Debug, Clone)]
pub struct Scenario {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub perpetrator_behavior: BehaviorType,
    pub victim_behavior: BehaviorType,
}

impl Scenario {
    pub fn new(
        name: String,
        description: String,
        perpetrator_behavior: BehaviorType,
        victim_behavior: BehaviorType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            perpetrator_behavior,
            victim_behavior,
        }
    }
}

/// 시나리오 실행 결과
#[derive(Debug, Clone)]
pub enum ScenarioResult {
    Success,
    Failure(String),
}

/// 전체 테스트 스위트를 관리하는 Manager 액터
pub struct ScenarioRunnerActor {
    scenarios: Vec<Scenario>,
    completed_count: usize,
    total_count: usize,
    results: Vec<ScenarioResult>,
    completion_tx: Option<oneshot::Sender<ScenarioSummary>>, // notify tests when all done
}

impl ScenarioRunnerActor {
    pub fn new(scenarios: Vec<Scenario>) -> Self {
        let total_count = scenarios.len();
        Self {
            scenarios,
            completed_count: 0,
            total_count,
            results: Vec::new(),
            completion_tx: None,
        }
    }

    /// Start a runner and notify via oneshot when all scenarios finish.
    pub fn start_with_notifier(
        scenarios: Vec<Scenario>,
        completion_tx: oneshot::Sender<ScenarioSummary>,
    ) -> Addr<Self> {
        actix::Actor::create(|_ctx| Self {
            total_count: scenarios.len(),
            scenarios,
            completed_count: 0,
            results: Vec::new(),
            completion_tx: Some(completion_tx),
        })
    }
}

impl Actor for ScenarioRunnerActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "ScenarioRunnerActor started with {} scenarios",
            self.total_count
        );

        // 모든 시나리오에 대해 SingleScenarioActor 생성 및 시작
        for scenario in self.scenarios.clone() {
            let scenario_actor = SingleScenarioActor::new(scenario, ctx.address());
            scenario_actor.start();
        }
    }
}

/// 개별 시나리오를 책임지고 실행하는 Worker 액터
pub struct SingleScenarioActor {
    scenario: Scenario,
    runner_addr: Addr<ScenarioRunnerActor>,
    player_results: Vec<crate::BehaviorResult>,
}

impl SingleScenarioActor {
    pub fn new(scenario: Scenario, runner_addr: Addr<ScenarioRunnerActor>) -> Self {
        Self {
            scenario,
            runner_addr,
            player_results: Vec::new(),
        }
    }
}

/// Summary sent to tests when all scenarios complete.
#[derive(Debug, Clone)]
pub struct ScenarioSummary {
    pub total: usize,
    pub success_count: usize,
    pub results: Vec<ScenarioResult>,
}

impl Actor for SingleScenarioActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "SingleScenarioActor started for scenario: {}",
            self.scenario.name
        );

        // 테스트 세션 ID 생성
        let test_session_id = Uuid::new_v4().to_string();
        info!(
            "Generated test_session_id for scenario {}: {}",
            self.scenario.name, test_session_id
        );

        let perpetrator_id = Uuid::new_v4();
        let victim_id = Uuid::new_v4();

        let perpetrator_schedule =
            schedules::get_schedule_for_perpetrator(&self.scenario.perpetrator_behavior);
        let victim_schedule = schedules::get_schedule_for_victim(&self.scenario.victim_behavior);
        let mut players_schedule = HashMap::new();

        players_schedule.insert(perpetrator_id, perpetrator_schedule);
        players_schedule.insert(victim_id, victim_schedule);

        let observer = ObserverActor::new(
            "ws://127.0.0.1:8080".to_string(),
            self.scenario.name.clone(),
            test_session_id.clone(),
            self.runner_addr.clone(),
            players_schedule,
            HashMap::new(),
        );

        // SingleScenarioActor 주소를 Observer에 설정 (순환 참조 방지를 위해 나중에 설정)
        let observer_addr = observer.start();

        // ObserverActor에 SingleScenarioActor 주소 설정
        observer_addr.do_send(SetSingleScenarioAddr {
            addr: ctx.address(),
        });

        let perpetrator_behavior = Box::new(self.scenario.perpetrator_behavior.clone());
        let victim_behavior = Box::new(self.scenario.victim_behavior.clone());

        let perpetrator_actor = PlayerActor::new(
            observer_addr.clone(),
            perpetrator_behavior,
            perpetrator_id,
            test_session_id.clone(),
            true,
        );
        let victim_actor = PlayerActor::new(
            observer_addr.clone(),
            victim_behavior,
            victim_id,
            test_session_id.clone(),
            true,
        );

        perpetrator_actor.start();
        victim_actor.start();

        // 4. Observer에게 관찰 시작 알림
        observer_addr.do_send(StartObservation {
            player_ids: vec![perpetrator_id, victim_id],
        });

        info!(
            "Created players for scenario {}: perpetrator={}, victim={}, session={}",
            self.scenario.name, perpetrator_id, victim_id, test_session_id
        );
    }
}
