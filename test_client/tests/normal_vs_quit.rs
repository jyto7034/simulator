use test_client::{
    behaviors::BehaviorType, scenario_actor::Scenario, setup_logger,
    test_utils::flush_redis_default,
};
use uuid::Uuid;

#[actix::test]
async fn test_normal_vs_quit_before_match() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs QuitBeforeMatch".to_string(),
        description: "Normal player continues waiting, Quit player disconnects after enqueue"
            .to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::QuitBeforeMatch,
    };

    // Run scenario
    let _observer_addr = scenario.run(Some(tx));

    // Wait for test completion with timeout
    let result = tokio::time::timeout(tokio::time::Duration::from_secs(30), rx)
        .await
        .expect("Test timed out after 30 seconds")
        .expect("Failed to receive completion signal");

    assert!(result, "Test should succeed");
}

#[actix::test]
async fn test_normal_vs_quit_after_enqueue() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs QuitAfterEnqueue".to_string(),
        description: "Normal player continues waiting, Quit player dequeues successfully"
            .to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::QuitAfterEnqueue,
    };

    // Run scenario
    let _observer_addr = scenario.run(Some(tx));

    // Wait for test completion with timeout
    let result = tokio::time::timeout(tokio::time::Duration::from_secs(30), rx)
        .await
        .expect("Test timed out after 30 seconds")
        .expect("Failed to receive completion signal");

    assert!(result, "Test should succeed");
}
