use actix::Actor;
use anyhow::Result;
use simulator_env;
use test_client::{
    behaviors::{
        disconnect::NetworkDisconnect,
        failure::{LoadingFailure, LoadingIgnorer},
        ignore::IgnoreMatchFound,
        normal::NormalPlayer,
        quit::{QuitDuringLoading, QuitDuringMatch},
        slow::SlowLoader,
        BehaviorType,
    },
    scenario_actor::{Scenario, ScenarioRunnerActor},
    setup_logger,
};
use tracing::info;

#[actix_web::test]
pub async fn test_normal_vs_normal_scenario() -> Result<()> {
    // 환경 설정 초기화
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting Normal vs Normal scenario test");

    let scenarios = vec![Scenario::new(
        "Normal vs Normal".to_string(),
        "Both players follow normal matchmaking flow".to_string(),
        BehaviorType::Normal(NormalPlayer),
        BehaviorType::Normal(NormalPlayer),
    )];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));

    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    info!("Normal vs Normal scenario test completed");

    anyhow::Ok(())
}

#[actix_web::test]
pub async fn test_normal_vs_quit_during_match() -> Result<()> {
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting Normal vs QuitDuringMatch scenario test");

    let scenarios = vec![Scenario::new(
        "Normal vs QuitDuringMatch".to_string(),
        "Normal player vs player who quits during match".to_string(),
        BehaviorType::Normal(NormalPlayer),
        BehaviorType::QuitDuringMatch(QuitDuringMatch),
    )];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    anyhow::Ok(())
}

#[actix_web::test]
pub async fn test_normal_vs_quit_during_loading() -> Result<()> {
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting Normal vs QuitDuringLoading scenario test");

    let scenarios = vec![Scenario::new(
        "Normal vs QuitDuringLoading".to_string(),
        "Normal player vs player who quits during loading".to_string(),
        BehaviorType::Normal(NormalPlayer),
        BehaviorType::QuitDuringLoading(QuitDuringLoading),
    )];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    anyhow::Ok(())
}

#[actix_web::test]
pub async fn test_normal_vs_slow_loader() -> Result<()> {
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting Normal vs SlowLoader scenario test");

    let scenarios = vec![Scenario::new(
        "Normal vs SlowLoader".to_string(),
        "Normal player vs slow loading player".to_string(),
        BehaviorType::Normal(NormalPlayer),
        BehaviorType::SlowLoader(SlowLoader { delay_seconds: 10 }),
    )];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));
    tokio::time::sleep(tokio::time::Duration::from_secs(45)).await; // SlowLoader 대응

    anyhow::Ok(())
}

#[actix_web::test]
pub async fn test_normal_vs_ignore_match_found() -> Result<()> {
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting Normal vs IgnoreMatchFound scenario test");

    let scenarios = vec![Scenario::new(
        "Normal vs IgnoreMatchFound".to_string(),
        "Normal player vs player who ignores match found".to_string(),
        BehaviorType::Normal(NormalPlayer),
        BehaviorType::IgnoreMatchFound(IgnoreMatchFound),
    )];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    anyhow::Ok(())
}

#[actix_web::test]
pub async fn test_normal_vs_sudden_disconnect() -> Result<()> {
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting Normal vs NetworkDisconnect scenario test");

    let scenarios = vec![Scenario::new(
        "Normal vs NetworkDisconnect".to_string(),
        "Normal player vs player who suddenly disconnects".to_string(),
        BehaviorType::Normal(NormalPlayer),
        BehaviorType::NetworkDisconnect(NetworkDisconnect),
    )];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    anyhow::Ok(())
}

#[actix_web::test]
pub async fn test_normal_vs_loading_failure() -> Result<()> {
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting Normal vs LoadingFailure scenario test");

    let scenarios = vec![Scenario::new(
        "Normal vs LoadingFailure".to_string(),
        "Normal player vs player who fails loading".to_string(),
        BehaviorType::Normal(NormalPlayer),
        BehaviorType::LoadingFailure(LoadingFailure),
    )];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    anyhow::Ok(())
}

#[actix_web::test]
pub async fn test_normal_vs_loading_ignorer() -> Result<()> {
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting Normal vs LoadingIgnorer scenario test");

    let scenarios = vec![Scenario::new(
        "Normal vs LoadingIgnorer".to_string(),
        "Normal player vs player who ignores loading start".to_string(),
        BehaviorType::Normal(NormalPlayer),
        BehaviorType::LoadingIgnorer(LoadingIgnorer),
    )];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

    anyhow::Ok(())
}

#[actix_web::test]
pub async fn test_chaos_scenarios() -> Result<()> {
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting Chaos scenarios test");

    let scenarios = vec![
        Scenario::new(
            "QuitDuringMatch vs QuitDuringLoading".to_string(),
            "Both players have disruptive behaviors".to_string(),
            BehaviorType::QuitDuringMatch(QuitDuringMatch),
            BehaviorType::QuitDuringLoading(QuitDuringLoading),
        ),
        Scenario::new(
            "SlowLoader vs LoadingFailure".to_string(),
            "Slow loader vs loading failure".to_string(),
            BehaviorType::SlowLoader(SlowLoader { delay_seconds: 10 }),
            BehaviorType::LoadingFailure(LoadingFailure),
        ),
        Scenario::new(
            "NetworkDisconnect vs LoadingIgnorer".to_string(),
            "Sudden disconnect vs loading ignorer".to_string(),
            BehaviorType::NetworkDisconnect(NetworkDisconnect),
            BehaviorType::LoadingIgnorer(LoadingIgnorer),
        ),
    ];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await; // 복잡한 시나리오들

    anyhow::Ok(())
}

#[actix_web::test]
pub async fn test_all_behaviors_comprehensive() -> Result<()> {
    simulator_env::init()?;
    let _ = setup_logger();

    info!("Starting comprehensive behavior test");

    let scenarios = vec![
        // Normal 기준 모든 조합
        Scenario::new(
            "Normal vs Normal".to_string(),
            "Baseline test".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::Normal(NormalPlayer),
        ),
        Scenario::new(
            "Normal vs QuitDuringMatch".to_string(),
            "".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::QuitDuringMatch(QuitDuringMatch),
        ),
        Scenario::new(
            "Normal vs QuitDuringLoading".to_string(),
            "".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::QuitDuringLoading(QuitDuringLoading),
        ),
        Scenario::new(
            "Normal vs SlowLoader".to_string(),
            "".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::SlowLoader(SlowLoader { delay_seconds: 10 }),
        ),
        Scenario::new(
            "Normal vs IgnoreMatchFound".to_string(),
            "".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::IgnoreMatchFound(IgnoreMatchFound),
        ),
        Scenario::new(
            "Normal vs NetworkDisconnect".to_string(),
            "".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::NetworkDisconnect(NetworkDisconnect),
        ),
        Scenario::new(
            "Normal vs LoadingFailure".to_string(),
            "".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::LoadingFailure(LoadingFailure),
        ),
        Scenario::new(
            "Normal vs LoadingIgnorer".to_string(),
            "".to_string(),
            BehaviorType::Normal(NormalPlayer),
            BehaviorType::LoadingIgnorer(LoadingIgnorer),
        ),
    ];

    let _ = ScenarioRunnerActor::create(|_ctx| ScenarioRunnerActor::new(scenarios));
    tokio::time::sleep(tokio::time::Duration::from_secs(120)).await; // 모든 시나리오 실행

    anyhow::Ok(())
}
