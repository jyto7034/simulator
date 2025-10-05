use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
use tracing::warn;

/// Redis Stream에 테스트 이벤트를 발행합니다.
///
/// # Arguments
/// * `redis` - Redis 연결
/// * `session_id` - 테스트 세션 ID (test_session_id from metadata)
/// * `event_type` - 이벤트 타입 (enqueued, dequeued, match_found, error 등)
/// * `pod_id` - 현재 Match Server의 pod ID
/// * `fields` - 추가 필드 (key-value 쌍)
///
/// # Stream Key Format
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
    let stream_key = format!("events:test:{}", session_id);

    // XADD events:test:{session_id} * type {event_type} pod_id {pod_id} ...
    let mut xadd_args: Vec<(&str, String)> = Vec::new();
    xadd_args.push(("type", event_type.to_string()));
    xadd_args.push(("pod_id", pod_id.to_string()));
    xadd_args.extend(fields);

    // Redis cmd로 직접 구성
    let mut cmd = redis::cmd("XADD");
    cmd.arg(&stream_key).arg("*"); // * = 자동 ID 생성

    for (key, value) in xadd_args {
        cmd.arg(key).arg(value);
    }

    // XADD 실행
    let _: String = cmd.query_async(redis).await?;

    // 스트림 자동 만료 설정 (1시간)
    // EXPIRE가 실패해도 무시 (이미 설정되어 있을 수 있음)
    let _: Result<i32, _> = redis
        .expire(&stream_key, 3600)
        .await;

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
