use redis::aio::ConnectionManager;
use serde_json::json;

use crate::state_events::StateEventEmitter;

/// Emit a typed state violation event and increment metrics.
pub async fn emit_violation(
    redis: &mut ConnectionManager,
    code: &str,
    details: serde_json::Value,
) {
    // Best-effort publish state event (errors are logged at emitter level)
    let mut emitter = StateEventEmitter::new(redis);
    let _ = emitter.state_violation(code.to_string(), details).await;
}

/// Convenience helper to emit violation with simple key-values.
pub async fn emit_violation_kv(
    redis: &mut ConnectionManager,
    code: &str,
    kv: &[(&str, String)],
) {
    let mut obj = serde_json::Map::new();
    for (k, v) in kv {
        obj.insert((*k).to_string(), json!(v));
    }
    emit_violation(redis, code, serde_json::Value::Object(obj)).await;
}
