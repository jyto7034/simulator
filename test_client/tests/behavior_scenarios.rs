use anyhow::Result;
use env as _;
use test_client::{
    behaviors::{
        invalid::InvalidMode,
        BehaviorType,
    },
    scenario_actor::{Scenario, ScenarioRunnerActor, ScenarioSummary},
    setup_logger,
};

use test_client::test_utils::flush_redis_default;
async fn run_single_scenario_test(
    name: &str,
    perpetrator: BehaviorType,
    victim: BehaviorType,
) -> Result<ScenarioSummary> {
    env::init()?;
    let _ = setup_logger();
    // Flush Redis to ensure clean state
    flush_redis_default().await?;

    let scenarios = vec![Scenario::new(
        name.to_string(),
        name.to_string(),
        perpetrator,
        victim,
    )];

    let (tx, rx) = tokio::sync::oneshot::channel::<ScenarioSummary>();
    let _addr = ScenarioRunnerActor::start_with_notifier(scenarios, tx);

    let summary = tokio::time::timeout(tokio::time::Duration::from_secs(60), rx)
        .await
        .map_err(|_| anyhow::anyhow!("scenario completion timed out"))??;

    Ok(summary)
}

#[actix_web::test]
async fn test_normal_vs_timeout_loader() -> Result<()> {
    let summary = run_single_scenario_test(
        "Normal vs TimeoutLoader",
        BehaviorType::TimeoutLoader,
        BehaviorType::Normal,
    )
    .await?;

    // 성공적인 전이로 Finished에 도달하면 Success로 집계됨
    assert_eq!(summary.total, 1);
    assert_eq!(summary.success_count, 1);
    Ok(())
}

#[actix_web::test]
async fn test_normal_vs_normal() -> Result<()> {
    let summary = run_single_scenario_test(
        "Normal vs Normal",
        BehaviorType::Normal,
        BehaviorType::Normal,
    )
    .await?;

    // 성공적인 전이로 Finished에 도달하면 Success로 집계됨
    assert_eq!(summary.total, 1);
    assert_eq!(summary.success_count, 1);
    Ok(())
}

#[actix_web::test]
async fn test_normal_vs_quit_before_match() -> Result<()> {
    let summary = run_single_scenario_test(
        "Normal vs QuitBeforeMatch",
        BehaviorType::QuitBeforeMatch,
        BehaviorType::Normal,
    )
    .await?;

    assert_eq!(summary.total, 1);
    assert_eq!(summary.success_count, 1);
    Ok(())
}

#[actix_web::test]
async fn test_normal_vs_quit_during_loading() -> Result<()> {
    let summary = run_single_scenario_test(
        "Normal vs QuitDuringLoading",
        BehaviorType::QuitDuringLoading,
        BehaviorType::Normal,
    )
    .await?;

    assert_eq!(summary.total, 1);
    assert_eq!(summary.success_count, 1);
    Ok(())
}

#[actix_web::test]
async fn test_normal_vs_spiky_loader() -> Result<()> {
    let summary = run_single_scenario_test(
        "Normal vs SpikyLoader",
        BehaviorType::SpikyLoader { delay_ms: 500 },
        BehaviorType::Normal,
    )
    .await?;

    assert_eq!(summary.total, 1);
    assert_eq!(summary.success_count, 1);
    Ok(())
}

#[actix_web::test]
async fn test_normal_vs_invalid_early_loading_complete() -> Result<()> {
    let summary = run_single_scenario_test(
        "Normal vs Invalid(EarlyLoadingComplete)",
        BehaviorType::Invalid { mode: InvalidMode::EarlyLoadingComplete },
        BehaviorType::Normal,
    )
    .await?;

    assert_eq!(summary.total, 1);
    assert_eq!(summary.success_count, 1);
    Ok(())
}
