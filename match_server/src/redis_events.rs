use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
use serde_json::json;
use tracing::warn;

/// Redis Pub/Sub로 테스트 이벤트를 발행합니다.
///
/// # Arguments
/// * `redis` - Redis 연결
/// * `session_id` - 테스트 세션 ID (test_session_id from metadata)
/// * `event_type` - 이벤트 타입 (enqueued, dequeued, match_found, error 등)
/// * `pod_id` - 현재 Match Server의 pod ID
/// * `fields` - 추가 필드 (key-value 쌍)
///
/// # Channel Format
/// `events:test:{session_id}`
///
/// # Example
/// ```rust
/// publish_test_event(
///     &mut redis,
///     "abc-123",
///     "enqueued",
///     "pod-a",
///     vec![
///         ("player_id", player_id.to_string()),
///         ("queue_size", "5".to_string()),
///     ],
/// ).await;
/// ```
pub async fn publish_test_event(
    redis: &mut ConnectionManager,
    session_id: &str,
    event_type: &str,
    pod_id: &str,
    fields: Vec<(&str, String)>,
) -> Result<(), RedisError> {
    let channel = format!("events:test:{}", session_id);

    // JSON 메시지 구성
    let mut data = serde_json::Map::new();
    data.insert("type".to_string(), json!(event_type));
    data.insert("pod_id".to_string(), json!(pod_id));
    data.insert("timestamp".to_string(), json!(chrono::Utc::now().to_rfc3339()));

    for (key, value) in fields {
        data.insert(key.to_string(), json!(value));
    }

    let message = serde_json::to_string(&data)
        .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "JSON serialization failed", e.to_string())))?;

    // PUBLISH events:test:{session_id} {json_message}
    let _: i32 = redis.publish(&channel, message).await?;

    Ok(())
}

/// metadata에서 test_session_id를 추출합니다.
/// 테스트 환경이 아니거나 session_id가 없으면 None을 반환합니다.
pub fn extract_test_session_id(metadata: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(metadata)
        .ok()?
        .get("test_session_id")?
        .as_str()
        .map(String::from)
}

/// 테스트 이벤트를 발행합니다. session_id가 없으면 무시합니다.
pub async fn try_publish_test_event(
    redis: &mut ConnectionManager,
    metadata: &str,
    event_type: &str,
    pod_id: &str,
    fields: Vec<(&str, String)>,
) {
    if let Some(session_id) = extract_test_session_id(metadata) {
        if let Err(e) = publish_test_event(redis, &session_id, event_type, pod_id, fields).await {
            warn!(
                "Failed to publish test event to session {}: {}",
                session_id, e
            );
        }
    }
}
