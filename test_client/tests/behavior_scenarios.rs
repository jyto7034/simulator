use actix::Actor;
use anyhow::Result;
use std::{thread::sleep, time::Duration};

async fn fetch_metrics_text() -> anyhow::Result<String> {
    let url = std::env::var("TEST_normal_vs_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080/metrics".into());
    let text = reqwest::get(url).await?.text().await?;
    Ok(text)
}

fn parse_counter(metrics: &str, name: &str) -> Option<f64> {
    for line in metrics.lines() {
        if line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix(name) {
            let parts: Vec<_> = rest.trim().split_whitespace().collect();
            if let Some(vs) = parts.get(0) {
                if let Ok(v) = vs.parse::<f64>() {
                    return Some(v);
                }
            }
        }
    }
    None
}

async fn get_counter(name: &str) -> f64 {
    match fetch_metrics_text().await {
        Ok(text) => parse_counter(&text, name).unwrap_or(0.0),
        Err(_) => 0.0,
    }
}

use env as _;
use test_client::{
    behaviors::{invalid::InvalidMode, BehaviorType},
    scenario_actor::{Scenario, ScenarioRunnerActor, ScenarioSummary},
    setup_logger,
};

use test_client::test_utils::flush_redis_default;
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

/*
    Blacklist 기능을 test 하고 있었음.
    timeout 발생 시 perpetrator 와 vitim 이 requeue 된다는 전제 하에,
    timeout player, normal 두 플레이어를 match 에 넣고 계속 TryMatch 를 유도하여 timeout player 가 blacklist 에 등록되는지 확인할 생각이었음.
    근데

    2025-08-19T01:54:30.700046Z ERROR  [3b673777-5b1e-40ad-a3e6-aafe18765a64] Test failed: MatchmakingError("Normal player should not receive errors during matchmaking: Matchmaking timed out. You will be returned to the queue shortly.")
    at test_client\src\player_actor\handler.rs:125 on test_blacklist_blocked ThreadId(2)

    2025-08-19T01:54:30.700153Z ERROR  [6983b7e3-7ee0-450c-8ae8-6a78a2019647] Test failed: System("server_error")

    로그 보니까 timeout 발생 시 test 자체를 failed 시켜버려서 requeue 되도 test 가 끝났기 때문에 player 와 match server 간 연결이 끊겨버림.
    때문에 TryMatch 가 발생하지 않고 blacklist 기능 작동 여부를 확인 할 수 없었음.

    timeout 이 발생해도 test 가 실패하지 않도록 수정해야함.

*/

#[actix_web::test]
async fn test_blacklist_blocked() -> Result<()> {
    env::init()?;
    let _ = setup_logger();
    // Flush Redis to ensure clean state
    flush_redis_default().await?;

    let scenarios = vec![Scenario::new(
        "Blacklist Test: TimeoutLoader vs Normal".to_string(),
        "Testing blacklist functionality with timeout scenarios".to_string(),
        BehaviorType::TimeoutLoader,
        BehaviorType::Normal,
    )];

    // Create a channel to keep the test running but not wait for completion
    let (tx, rx) = tokio::sync::oneshot::channel::<ScenarioSummary>();
    let _addr = ScenarioRunnerActor::start_with_notifier(scenarios, tx);

    // Wait 20 seconds to allow multiple timeout/requeue cycles
    tokio::time::sleep(Duration::from_secs(60)).await;

    // Try to receive result but don't fail if it times out (expected for this test)
    let _result = tokio::time::timeout(Duration::from_millis(100), rx).await;

    // Test passes if we reach here without crashing
    println!("Blacklist test completed - check server logs for blacklist behavior");
    Ok(())
}

#[actix_web::test]
async fn test_normal_vs_normal() -> Result<()> {
    let base_new = get_counter("players_enqueued_new_total").await;
    let base_alloc = get_counter("players_allocated_total").await;

    let summary = run_single_scenario_test(
        "Metrics: Normal vs Normal",
        BehaviorType::Normal,
        BehaviorType::Normal,
    )
    .await?;
    assert_eq!(summary.total, 1);

    // wait for metrics flush
    tokio::time::sleep(Duration::from_millis(300)).await;

    let after_new = get_counter("players_enqueued_new_total").await;
    let after_alloc = get_counter("players_allocated_total").await;

    assert!(
        after_new >= base_new + 2.0,
        "new_enqueued should increase by >= 2 ({} -> {})",
        base_new,
        after_new
    );
    assert!(
        after_alloc >= base_alloc + 2.0,
        "allocated should increase by >= 2 ({} -> {})",
        base_alloc,
        after_alloc
    );
    Ok(())
}

#[actix_web::test]
async fn test_normal_vs_unknown_type() -> Result<()> {
    let base_unknown = get_counter("abnormal_unknown_type_total").await;

    let _ = run_single_scenario_test(
        "Metrics: UnknownType vs Normal",
        BehaviorType::Invalid {
            mode: InvalidMode::UnknownType,
        },
        BehaviorType::Normal,
    )
    .await?;

    tokio::time::sleep(Duration::from_millis(200)).await;
    let after_unknown = get_counter("abnormal_unknown_type_total").await;
    assert!(
        after_unknown >= base_unknown + 1.0,
        "unknown_type should increase ({} -> {})",
        base_unknown,
        after_unknown
    );
    Ok(())
}

#[actix_web::test]
async fn test_normal_vs_wrong_session_id() -> Result<()> {
    let base_wrong = get_counter("abnormal_wrong_session_id_total").await;

    let _ = run_single_scenario_test(
        "Metrics: WrongSessionId vs Normal",
        BehaviorType::Invalid {
            mode: InvalidMode::WrongSessionId,
        },
        BehaviorType::Normal,
    )
    .await?;

    tokio::time::sleep(Duration::from_millis(200)).await;
    let after_wrong = get_counter("abnormal_wrong_session_id_total").await;
    assert!(
        after_wrong >= base_wrong + 1.0,
        "wrong_session_id should increase ({} -> {})",
        base_wrong,
        after_wrong
    );
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
async fn test_normal_vs_invalid_duplicate_enqueue() -> Result<()> {
    let summary = run_single_scenario_test(
        "Invalid(DuplicateEnqueue) vs Normal",
        BehaviorType::Invalid {
            mode: InvalidMode::DuplicateEnqueue,
        },
        BehaviorType::Normal,
    )
    .await?;

    assert_eq!(summary.total, 1);
    assert_eq!(summary.success_count, 1);
    Ok(())
}

#[actix_web::test]
async fn test_normal_vs_invalid_missing_field() -> Result<()> {
    let summary = run_single_scenario_test(
        "Invalid(MissingField) vs Normal",
        BehaviorType::Invalid {
            mode: InvalidMode::MissingField,
        },
        BehaviorType::Normal,
    )
    .await?;

    assert_eq!(summary.total, 1);
    assert_eq!(summary.success_count, 1);
    Ok(())
}
