// metrics removed
use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    protocol::{ErrorCode, ServerMessage},
    state_events::StateEventEmitter,
};

/// Perform final failure cleanup when max retries are exhausted.
/// Emits state event, deletes loading key, removes players from queue, adjusts metrics,
/// notifies players with error message, and cleans up retry key if provided.
pub async fn final_failure_cleanup(
    redis: &mut ConnectionManager,
    queue_key_prefix: &str,
    game_mode: &str,
    loading_session_id: Uuid,
    player_ids: &[String],
    max_retries: u32,
    retry_key: Option<&str>,
) {
    warn!(
        "Max retries ({}) exceeded for loading session {} with players {:?} in game mode {}. Notifying clients of failure.",
        max_retries, loading_session_id, player_ids, game_mode
    );

    // Metrics
    // metrics removed

    // State event
    {
        let mut emitter = StateEventEmitter::new(redis);
        if let Err(e) = emitter
            .dedicated_session_failed(loading_session_id.to_string(), "max_retries_exceeded")
            .await
        {
            warn!("Failed to publish dedicated_session_failed event: {}", e);
        } else {
            info!(
                "[{}] Published dedicated_session_failed (max_retries_exceeded) for session {}",
                game_mode, loading_session_id
            );
        }
    }

    // Delete loading:<sid>
    let loading_key = format!("loading:{}", loading_session_id);
    match redis.del::<_, i32>(&loading_key).await {
        Ok(n) if n > 0 => info!(
            "[{}] Deleted loading session key {} on final failure",
            game_mode, loading_key
        ),
        Ok(_) => info!(
            "[{}] Loading session key {} not present during cleanup",
            game_mode, loading_key
        ),
        Err(e) => warn!(
            "[{}] Failed to delete loading key {}: {}",
            game_mode, loading_key, e
        ),
    }

    // Remove players from queue and emit size changed
    let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
    let removed: i32 = match redis.srem::<_, _, i32>(&queue_key, player_ids).await {
        Ok(n) => n,
        Err(e) => {
            error!(
                "Failed to SREM players {:?} from {} on max retries: {}",
                player_ids, queue_key, e
            );
            0
        }
    };

    if removed > 0 {
        info!(
            "[{}] Removed {} players from queue {} after max retries",
            game_mode, removed, queue_key
        );

        if let Ok(size) = redis.scard::<_, usize>(&queue_key).await {
            let mut emitter = StateEventEmitter::new(redis);
            if let Err(e) = emitter
                .queue_size_changed(game_mode.to_string(), size)
                .await
            {
                warn!(
                    "Failed to publish queue_size_changed after max retries cleanup: {}",
                    e
                );
            } else {
                info!("[{}] Queue size after cleanup: {}", game_mode, size);
            }
        }
    } else {
        info!(
            "[{}] No players removed from queue {} (possibly already absent)",
            game_mode, queue_key
        );
    }

    // Notify players of final failure
    for pid in player_ids {
        if let Ok(player_id) = Uuid::parse_str(pid) {
            let message = ServerMessage::Error {
                code: Some(ErrorCode::MaxRetriesExceeded),
                message: format!(
                    "Matchmaking failed after {} attempts. Please try again later.",
                    max_retries
                ),
            };
            let channel = format!("notifications:{}", player_id);
            let payload = match serde_json::to_string(&message) {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "Failed to serialize ServerMessage for player {}: {}",
                        player_id, e
                    );
                    continue;
                }
            };
            if let Err(e) = redis.publish::<_, _, ()>(&channel, &payload).await {
                warn!("Failed to publish message to channel {}: {}", channel, e);
            }
        }
    }

    // Cleanup retry key if provided
    if let Some(key) = retry_key {
        let _ = redis.del::<_, i32>(key).await;
    }
}
