use anyhow::Result;
use test_client::{
    behaviors::BehaviorType,
    scenario_actor::{Scenario, ScenarioRunnerActor, ScenarioSummary},
    setup_logger,
    test_utils::flush_redis_default,
};
use tracing::info;

pub async fn run_single_scenario_test(
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
async fn test_quit_after_enqueue_vs_quit_after_enqueue() -> Result<()> {
    info!("🧪 Testing QuitAfterEnqueue vs QuitAfterEnqueue scenario");
    info!("📝 Both players enqueue, then immediately dequeue");

    let summary = run_single_scenario_test(
        "QuitAfterEnqueue vs QuitAfterEnqueue",
        BehaviorType::QuitAfterEnqueue,
        BehaviorType::QuitAfterEnqueue,
    )
    .await?;

    info!("✅ Test completed: {:?}", summary);

    // 핵심 검증:
    // 1. 두 플레이어 모두 Enqueued 이벤트를 Redis Stream에서 받음
    // 2. 두 플레이어 모두 Dequeue 메시지 전송
    // 3. 두 플레이어 모두 Dequeued 이벤트를 Redis Stream에서 받음
    // 4. ObserverActor가 두 플레이어의 Phase를 Matching → Finished로 전환
    // 5. 시나리오 성공적으로 완료
    assert_eq!(summary.total, 1, "Should have 1 scenario");
    assert_eq!(
        summary.success_count, 1,
        "Both QuitAfterEnqueue players should receive Dequeued events and complete"
    );

    Ok(())
}
