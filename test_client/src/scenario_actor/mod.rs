use actix::{Actor, Addr, AsyncContext, Context};
use tracing::info;
use uuid::Uuid;

use crate::{behaviors::BehaviorType, observer_actor::ObserverActor, player_actor::PlayerActor};

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
}

impl ScenarioRunnerActor {
    pub fn new(scenarios: Vec<Scenario>) -> Self {
        let total_count = scenarios.len();
        Self {
            scenarios,
            completed_count: 0,
            total_count,
            results: Vec::new(),
        }
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
    player_results: Vec<crate::TestResult>,
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

impl Actor for SingleScenarioActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "SingleScenarioActor started for scenario: {}",
            self.scenario.name
        );

        // 1. ObserverActor 생성
        let observer = ObserverActor::new(
            "ws://127.0.0.1:8080".to_string(),
            self.scenario.name.clone(),
            self.runner_addr.clone(),
        );
        let observer_addr = observer.start();

        // 2. PlayerActor 생성 시 Observer 주소 주입
        let perpetrator_id = Uuid::new_v4();
        let victim_id = Uuid::new_v4();

        let perpetrator_behavior = Box::new(self.scenario.perpetrator_behavior.clone());
        let victim_behavior = Box::new(self.scenario.victim_behavior.clone());

        let perpetrator_actor =
            PlayerActor::new(observer_addr.clone(), perpetrator_behavior, perpetrator_id);
        let victim_actor = PlayerActor::new(observer_addr.clone(), victim_behavior, victim_id);

        perpetrator_actor.start();
        victim_actor.start();

        // 3. Observer에게 관찰 시작 알림
        // observer_addr.do_send(StartObservation { player_id_filter: None });

        info!(
            "Created players for scenario {}: perpetrator={}, victim={}",
            self.scenario.name, perpetrator_id, victim_id
        );
    }
}
