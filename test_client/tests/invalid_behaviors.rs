/// Test invalid behaviors to verify server robustness
///
/// 이 테스트는 서버가 비정상적인 입력을 올바르게 처리하는지 검증합니다.
///
/// 실행 방법:
/// ```bash
/// cargo test --test invalid_behaviors -- --nocapture
/// ```
use anyhow::Result;
use test_client::{
    behaviors::{invalid::InvalidMode, BehaviorType},
    scenario_actor::{Scenario, ScenarioRunnerActor, ScenarioSummary},
    setup_logger,
    test_utils::flush_redis_default,
};
use tracing::info;

/// 헬퍼 함수: Normal player vs Invalid player 시나리오 실행
pub async fn run_invalid_scenario_test(
    name: &str,
    invalid_mode: InvalidMode,
) -> Result<ScenarioSummary> {
    env::init()?;
    let _ = setup_logger();
    flush_redis_default().await?;

    let scenarios = vec![Scenario::new(
        name.to_string(),
        format!("{} - Invalid player should receive error", name),
        BehaviorType::Normal,
        BehaviorType::Invalid { mode: invalid_mode },
    )];

    let (tx, rx) = tokio::sync::oneshot::channel::<ScenarioSummary>();
    let _addr = ScenarioRunnerActor::start_with_notifier(scenarios, tx);

    let summary = tokio::time::timeout(tokio::time::Duration::from_secs(60), rx)
        .await
        .map_err(|_| anyhow::anyhow!("scenario completion timed out"))??;

    Ok(summary)
}

#[actix_web::test]
async fn test_invalid_game_mode_vs_normal() -> Result<()> {
    info!("🧪 Testing Normal vs InvalidGameMode");
    info!("📝 Invalid player tries to enqueue with non-existent game mode");

    let summary =
        run_invalid_scenario_test("Normal vs InvalidGameMode", InvalidMode::InvalidGameMode)
            .await?;

    info!("✅ Test completed: {:?}", summary);

    // 검증:
    // 1. Normal player는 정상적으로 Enqueued
    // 2. Invalid player는 Error 응답 받음 (InvalidGameMode)
    // 3. 시나리오 완료
    assert_eq!(summary.total, 1, "Should have 1 scenario");
    assert_eq!(
        summary.success_count, 1,
        "Invalid player should receive error and complete"
    );

    Ok(())
}

#[actix_web::test]
async fn test_large_metadata_vs_normal() -> Result<()> {
    info!("🧪 Testing Normal vs LargeMetadata");
    info!("📝 Invalid player tries to enqueue with 1MB metadata");

    let summary =
        run_invalid_scenario_test("Normal vs LargeMetadata", InvalidMode::LargeMetadata).await?;

    info!("✅ Test completed: {:?}", summary);

    // 검증:
    // 1. Normal player는 정상적으로 Enqueued
    // 2. Invalid player는 거대한 metadata로 인해 Error 또는 연결 종료
    // 3. 시나리오 완료
    assert_eq!(summary.total, 1, "Should have 1 scenario");
    assert_eq!(
        summary.success_count, 1,
        "Large metadata should be rejected"
    );

    Ok(())
}

#[actix_web::test]
async fn test_malformed_json_vs_normal() -> Result<()> {
    info!("🧪 Testing Normal vs MalformedJson");
    info!("📝 Invalid player sends malformed JSON message");

    let summary =
        run_invalid_scenario_test("Normal vs MalformedJson", InvalidMode::MalformedJson).await?;

    info!("✅ Test completed: {:?}", summary);

    // 검증:
    // 1. Normal player는 정상적으로 Enqueued
    // 2. Invalid player는 JSON 파싱 실패로 Error 또는 연결 종료
    // 3. 시나리오 완료
    assert_eq!(summary.total, 1, "Should have 1 scenario");
    assert_eq!(
        summary.success_count, 1,
        "Malformed JSON should be rejected"
    );

    Ok(())
}

#[actix_web::test]
async fn test_idle_to_dequeue_vs_normal() -> Result<()> {
    info!("🧪 Testing Normal vs IdleToDequeue");
    info!("📝 Invalid player tries to dequeue without enqueuing first");

    let summary =
        run_invalid_scenario_test("Normal vs IdleToDequeue", InvalidMode::IdleToDequeue).await?;

    info!("✅ Test completed: {:?}", summary);

    // 검증:
    // 1. Normal player는 정상적으로 Enqueued
    // 2. Invalid player는 state machine 위반으로 Error 응답
    // 3. 시나리오 완료
    assert_eq!(summary.total, 1, "Should have 1 scenario");
    assert_eq!(
        summary.success_count, 1,
        "State machine violation should be caught"
    );

    Ok(())
}

#[actix_web::test]
async fn test_unknown_type_vs_normal() -> Result<()> {
    info!("🧪 Testing Normal vs UnknownType");
    info!("📝 Invalid player sends unknown message type after enqueue");

    let summary =
        run_invalid_scenario_test("Normal vs UnknownType", InvalidMode::UnknownType).await?;

    info!("✅ Test completed: {:?}", summary);

    // 검증:
    // 1. Normal player는 정상적으로 Enqueued
    // 2. Invalid player는 Enqueued 후 unknown type 전송하여 Error
    // 3. 시나리오 완료
    assert_eq!(summary.total, 1, "Should have 1 scenario");
    assert_eq!(
        summary.success_count, 1,
        "Unknown message type should be rejected"
    );

    Ok(())
}

#[actix_web::test]
async fn test_missing_field_vs_normal() -> Result<()> {
    info!("🧪 Testing Normal vs MissingField");
    info!("📝 Invalid player sends message with missing required field");

    let summary =
        run_invalid_scenario_test("Normal vs MissingField", InvalidMode::MissingField).await?;

    info!("✅ Test completed: {:?}", summary);

    // 검증:
    // 1. Normal player는 정상적으로 Enqueued
    // 2. Invalid player는 Enqueued 후 필수 필드 누락 메시지 전송하여 Error
    // 3. 시나리오 완료
    assert_eq!(summary.total, 1, "Should have 1 scenario");
    assert_eq!(summary.success_count, 1, "Missing field should be rejected");

    Ok(())
}

#[actix_web::test]
async fn test_duplicate_enqueue_vs_normal() -> Result<()> {
    info!("🧪 Testing Normal vs DuplicateEnqueue");
    info!("📝 Invalid player tries to enqueue twice");

    let summary =
        run_invalid_scenario_test("Normal vs DuplicateEnqueue", InvalidMode::DuplicateEnqueue)
            .await?;

    info!("✅ Test completed: {:?}", summary);

    // 검증:
    // 1. Normal player는 정상적으로 Enqueued
    // 2. Invalid player는 Enqueued 후 다시 Enqueue 시도하여 AlreadyInQueue Error
    // 3. 시나리오 완료
    assert_eq!(summary.total, 1, "Should have 1 scenario");
    assert_eq!(
        summary.success_count, 1,
        "Duplicate enqueue should be rejected"
    );

    Ok(())
}

#[actix_web::test]
async fn test_wrong_player_id_vs_normal() -> Result<()> {
    info!("🧪 Testing Normal vs WrongPlayerId");
    info!("📝 Invalid player tries to dequeue with different player_id");

    let summary =
        run_invalid_scenario_test("Normal vs WrongPlayerId", InvalidMode::WrongPlayerId).await?;

    info!("✅ Test completed: {:?}", summary);

    // 검증:
    // 1. Normal player는 정상적으로 Enqueued
    // 2. Invalid player는 Enqueued 후 다른 player_id로 Dequeue 시도하여 Error
    // 3. 시나리오 완료
    assert_eq!(summary.total, 1, "Should have 1 scenario");
    assert_eq!(
        summary.success_count, 1,
        "Wrong player_id should be rejected"
    );

    Ok(())
}
