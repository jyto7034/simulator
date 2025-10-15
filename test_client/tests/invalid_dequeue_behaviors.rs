use test_client::{
    behaviors::BehaviorType, scenario_actor::Scenario, setup_logger,
    test_utils::flush_redis_default,
};
use uuid::Uuid;

#[actix::test]
async fn test_invalid_dequeue_unknown_type() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs InvalidDequeueUnknownType".to_string(),
        description:
            "Invalid player sends unknown message type during dequeue, server should reject"
                .to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::InvalidDequeueUnknownType,
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
async fn test_invalid_dequeue_missing_field() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs InvalidDequeueMissingField".to_string(),
        description: "Invalid player sends dequeue message with missing required field".to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::InvalidDequeueMissingField,
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
async fn test_invalid_dequeue_duplicate() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs InvalidDequeueDuplicate".to_string(),
        description:
            "Invalid player sends duplicate dequeue request, should receive NotInQueue error"
                .to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::InvalidDequeueDuplicate,
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
async fn test_invalid_dequeue_wrong_player_id() {
    // Setup logger
    setup_logger();
    flush_redis_default().await.unwrap();

    // Create completion channel
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    // Create scenario
    let scenario = Scenario {
        id: Uuid::new_v4(),
        name: "Normal vs InvalidDequeueWrongPlayerId".to_string(),
        description:
            "Invalid player sends dequeue with wrong player_id, should receive WrongSessionId error"
                .to_string(),
        normal_behavior: BehaviorType::Normal,
        abnormal_behavior: BehaviorType::InvalidDequeueWrongPlayerId,
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
