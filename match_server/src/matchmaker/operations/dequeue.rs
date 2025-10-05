use backoff::backoff::Backoff;
use redis::{aio::ConnectionManager, RedisResult, Script};
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    matchmaker::{
        operations::notify,
        scripts::{self},
        MatchmakerDeps,
    },
    protocol::{ErrorCode, ServerMessage},
    redis_events, GameMode, RETRY_CONFIG,
};

async fn invoke_dequeue_script(
    redis: &mut ConnectionManager,
    queue_key: String,
    player_id: Uuid,
) -> RedisResult<(i64, i64, String)> {
    let result: (i64, i64, String) = Script::new(scripts::dequeue_player_script())
        .key(queue_key)
        .arg(player_id.to_string())
        .invoke_async(redis)
        .await?;
    Ok(result)
}

pub async fn dequeue(
    queue_suffix: &str,
    game_mode: GameMode,
    player_id: Uuid,
    deps: &MatchmakerDeps,
) {
    let subscription_addr = deps.subscription_addr.clone();
    let mut redis = deps.redis.clone();
    let settings = deps.settings.clone();

    let is_known_mode = settings
        .game_modes
        .iter()
        .any(|mode| mode.game_mode == game_mode);

    if !is_known_mode {
        warn!(
            "Player {} tried to dequeue for unsupported mode {:?}",
            player_id, game_mode
        );
        notify::send_message_to_player(
            subscription_addr,
            &mut redis,
            player_id,
            ServerMessage::Error {
                code: ErrorCode::InvalidGameMode,
                message: "Unsupported game mode".to_string(),
            },
        )
        .await;
        return;
    }

    let suffix = queue_suffix;
    let hash_tag = format!("{{{}}}", suffix);
    let queue_key = format!("queue:{}", hash_tag);

    let backoff = RETRY_CONFIG
        .read()
        .unwrap()
        .as_ref()
        .expect("Retry config not initialized")
        .clone();

    let mut backoff_state = backoff;
    let dequeue_result = loop {
        let mut redis_clone = redis.clone();

        match invoke_dequeue_script(
            &mut redis_clone,
            queue_key.clone(),
            player_id,
        )
        .await
        {
            Ok(res) => break Ok(res),
            Err(err) => {
                if let Some(delay) = backoff_state.next_backoff() {
                    warn!(
                        "Temporary dequeue failure for player {}: {} (retrying in {:?})",
                        player_id, err, delay
                    );
                    sleep(delay).await;
                    continue;
                } else {
                    break Err(err);
                }
            }
        }
    };

    let (removed_flag, current_size, metadata) = match dequeue_result {
        Ok(res) => res,
        Err(err) => {
            error!(
                "Failed to dequeue player {} into {}: {}",
                player_id, queue_key, err
            );
            notify::send_message_to_player(
                subscription_addr,
                &mut redis,
                player_id,
                ServerMessage::Error {
                    code: ErrorCode::InternalError,
                    message: "Failed to dequeue".to_string(),
                },
            )
            .await;
            return;
        }
    };

    let pod_id = std::env::var("POD_ID").unwrap_or_else(|_| "default-pod".to_string());

    let response = if removed_flag == 1 {
        info!(
            "Player {} dequeued for {:?}. queue size = {}",
            player_id, game_mode, current_size
        );

        // Publish test event
        redis_events::try_publish_test_event(
            &mut redis,
            &metadata,
            "dequeued",
            &pod_id,
            vec![
                ("player_id", player_id.to_string()),
                ("queue_size", current_size.to_string()),
                ("game_mode", format!("{:?}", game_mode)),
            ],
        )
        .await;

        ServerMessage::DeQueued
    } else {
        warn!("Player {} not found in queue {:?}", player_id, game_mode);
        ServerMessage::Error {
            code: ErrorCode::NotInQueue,
            message: "Not found in queue".to_string(),
        }
    };

    notify::send_message_to_player(subscription_addr, &mut redis, player_id, response).await;
}
