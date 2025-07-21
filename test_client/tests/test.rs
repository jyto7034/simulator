use actix::Actor;
use anyhow::Result;
use simulator_env;
use test_client::{
    behaviors::{normal::NormalPlayer, quit, BehaviorType},
    scenario_actor::{Scenario, ScenarioRunnerActor},
};
use tracing::info;

#[actix_web::test]
pub async fn run_example_test() -> Result<()> {
    // 환경 설정 초기화
    simulator_env::init()?;

    // 디버깅을 위해 사용되는 URL 출력
    let match_url = simulator_env::env::match_server_url();
    let ws_url = simulator_env::env::match_server_ws_url();

    let scenarios = vec![
        Scenario::new(
            "Normal Scenario".to_string(),
            "_".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::Normal(NormalPlayer),
        ),
        Scenario::new(
            "Normal : QuitDuringMatch".to_string(),
            "_".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::QuitDuringMatch(quit::QuitDuringMatch),
        ),
    ];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));

    info!("Match server URL: {}", match_url);
    info!("Match server WebSocket URL: {}", ws_url);

    anyhow::Ok(())
}
