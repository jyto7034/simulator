use test_client::{
    behaviors::BehaviorType, scenario_actor::Scenario, setup_logger,
    test_utils::flush_redis_default,
};
use uuid::Uuid;

#[actix::test]
async fn test_invalid_enqueue_unknown_type() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs InvalidEnqueueUnknownType".to_string(),
        description: "Invalid player sends unknown message type, server should reject with error"
            .to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::InvalidEnqueueUnknownType,
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
async fn test_invalid_enqueue_missing_field() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs InvalidEnqueueMissingField".to_string(),
        description: "Invalid player sends enqueue message with missing required field".to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::InvalidEnqueueMissingField,
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
async fn test_invalid_enqueue_duplicate() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs InvalidEnqueueDuplicate".to_string(),
        description:
            "Invalid player sends duplicate enqueue request, should receive AlreadyInQueue error"
                .to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::InvalidEnqueueDuplicate,
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
