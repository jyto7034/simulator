use test_client::{
    behaviors::BehaviorType, scenario_actor::Scenario, setup_logger,
    test_utils::flush_redis_default,
};
use uuid::Uuid;

#[actix::test]
async fn test_normal_vs_normal() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs Normal".to_string(),
        description: "Two normal players should match successfully".to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::Normal,
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
