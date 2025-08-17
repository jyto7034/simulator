// metrics removed
use redis::{aio::ConnectionManager, AsyncCommands};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, warn};

use crate::{invariants, state_events::StateEventEmitter};

/// Add a single player to queue and emit queue_size_changed on success.
pub async fn add_player(
    redis: &mut ConnectionManager,
    queue_key: &str,
    player_id_str: &str,
) -> Result<bool, String> {
    match redis.sadd::<_, _, i32>(queue_key, player_id_str).await {
        Ok(added) => {
            if added > 0 {
                // Extract game_mode and publish size change
                let game_mode = queue_key.split(':').nth(1).unwrap_or("unknown").to_string();

                // Record enqueue timestamp for match_time_seconds
                if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                    let key = format!("queue_time:{}", player_id_str);
                    let _ = redis
                        .set_ex::<_, _, ()>(key, now.as_secs().to_string(), 24 * 3600)
                        .await; // set TTL 24h as a safety cap
                }

                if let Ok(size) = redis.scard::<_, usize>(queue_key).await {
                    let mut emitter = StateEventEmitter::new(redis);
                    let _ = emitter.queue_size_changed(game_mode, size).await;
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => Err(format!("sadd failed: {}", e)),
    }
}

/// Remove a single player from queue and emit queue_size_changed on success.
pub async fn remove_player(
    redis: &mut ConnectionManager,
    queue_key: &str,
    player_id_str: &str,
) -> Result<bool, String> {
    match redis.srem::<_, _, i32>(queue_key, player_id_str).await {
        Ok(removed) => {
            if removed > 0 {
                let game_mode = queue_key.split(':').nth(1).unwrap_or("unknown").to_string();
                if let Ok(size) = redis.scard::<_, usize>(queue_key).await {
                    let mut emitter = StateEventEmitter::new(redis);
                    let _ = emitter.queue_size_changed(game_mode, size).await;
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => Err(format!("srem failed: {}", e)),
    }
}

/// Re-queue players back into the given queue key and emit observability events.
/// - SADD players
/// - Adjust PLAYERS_IN_QUEUE by number actually added
/// - Emit players_requeued and queue_size_changed
pub async fn requeue_players(
    redis: &mut ConnectionManager,
    queue_key: &str,
    player_ids: &[String],
) {
    warn!("Re-queuing players due to an error: {:?}", player_ids);
    if player_ids.is_empty() {
        return;
    }

    // Extract game_mode from queue_key (e.g., "queue:Normal_1v1" -> "Normal_1v1")
    let game_mode = queue_key.split(':').nth(1).unwrap_or("unknown").to_string();

    // Capture size before
    let _before_size: Option<usize> = match redis.scard::<_, usize>(queue_key).await {
        Ok(sz) => Some(sz),
        Err(_) => None,
    };

    // Perform atomic requeue using a Redis Lua script to avoid race conditions
    let script = redis::Script::new(
        r#"
        local before = redis.call('SCARD', KEYS[1])
        local added = redis.call('SADD', KEYS[1], unpack(ARGV))
        local after = redis.call('SCARD', KEYS[1])
        return {before, added, after}
    "#,
    );

    let mut inv = script.prepare_invoke();
    inv.key(queue_key);
    for pid in player_ids.iter() {
        inv.arg(pid);
    }

    match inv.invoke_async::<_, (i64, i64, i64)>(redis).await {
        Ok((before_i, added_i, after_i)) => {
            let before = before_i.max(0) as usize;
            let added = added_i.max(0) as usize;
            let after = after_i.max(0) as usize;

            // no global queue gauge adjustments

            // Publish state event for players requeued
            {
                let mut emitter = StateEventEmitter::new(redis);
                if let Err(e) = emitter
                    .players_requeued(game_mode.clone(), player_ids.to_vec())
                    .await
                {
                    warn!("Failed to publish players_requeued event: {}", e);
                }
            }

            // Publish queue size changed for better observability using 'after'
            {
                let mut emitter = StateEventEmitter::new(redis);
                if let Err(e) = emitter.queue_size_changed(game_mode, after).await {
                    warn!("Failed to publish queue_size_changed after requeue: {}", e);
                }
            }

            // Reset queue_time for requeued players to now (fresh wait time)
            if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                let now_secs = now.as_secs().to_string();
                for pid in player_ids.iter() {
                    let key = format!("queue_time:{}", pid);
                    let _ = redis
                        .set_ex::<_, _, ()>(key, now_secs.clone(), 24 * 3600)
                        .await;
                }
            }

            // Invariant: after size should equal before + added (within the same atomic script)
            let expected = before.saturating_add(added);
            if after != expected {
                let intended = player_ids.len();
                tracing::error!(
                    queue_key=%queue_key,
                    before, added, after, expected, intended,
                    "Invariant violation: queue size mismatch after requeue (atomic)"
                );
                invariants::emit_violation_kv(
                    redis,
                    "queue_size_mismatch_after_requeue",
                    &[
                        ("before", before.to_string()),
                        ("added", added.to_string()),
                        ("after", after.to_string()),
                        ("expected", expected.to_string()),
                        ("intended", intended.to_string()),
                        ("queue_key", queue_key.to_string()),
                    ],
                )
                .await;
            }
        }
        Err(e) => {
            error!(
                "CRITICAL: Redis script requeue failed for players {:?} into {}: {} (falling back to non-atomic path)",
                player_ids, queue_key, e
            );
            // Fallback: original non-atomic path
            match redis.sadd::<_, _, i32>(queue_key, player_ids).await {
                Ok(_added) => {
                    // no global queue gauge adjustments
                    if let Ok(size) = redis.scard::<_, usize>(queue_key).await {
                        let mut emitter = StateEventEmitter::new(redis);
                        let _ = emitter.queue_size_changed(game_mode, size).await;
                    }
                }
                Err(e2) => {
                    error!(
                        "CRITICAL: Failed to re-queue players {:?} into {}: {}",
                        player_ids, queue_key, e2
                    );
                }
            }
        }
    }
}
