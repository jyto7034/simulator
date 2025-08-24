use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::warn;
use uuid::Uuid;

use crate::{protocol::ServerMessage, pubsub::SubscriptionManager};
use actix::Addr;

/// Publish a server message to a single player's notification channel.
pub async fn notify_player(redis: &mut ConnectionManager, player_id: Uuid, message: ServerMessage) {
    let channel = format!("notifications:{}", player_id);
    let payload = match serde_json::to_string(&message) {
        Ok(p) => p,
        Err(e) => {
            warn!(
                "Failed to serialize ServerMessage for player {}: {}",
                player_id, e
            );
            return;
        }
    };
    if let Err(e) = redis.publish::<_, _, ()>(&channel, &payload).await {
        warn!("Failed to publish message to channel {}: {}", channel, e);
    }
}

/// Unified message sending with direct delivery and Redis fallback
pub async fn send_player_message(
    sub_manager: &Addr<SubscriptionManager>,
    redis: &mut ConnectionManager,
    player_id: Uuid,
    message: ServerMessage,
) -> bool {
    // Try direct delivery first
    let direct_result = sub_manager
        .send(crate::pubsub::ForwardMessage {
            player_id,
            message: message.clone(),
        })
        .await;

    // Only use Redis pub/sub if direct delivery failed
    match direct_result {
        Ok(()) => {
            // Direct delivery succeeded, no need for Redis fallback
            true
        }
        Err(_) => {
            // Direct delivery failed, use Redis pub/sub as fallback
            let channel = format!("notifications:{}", player_id);
            let payload = match serde_json::to_string(&message) {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "Failed to serialize ServerMessage for player {}: {}",
                        player_id, e
                    );
                    return false;
                }
            };

            match redis.publish::<_, _, ()>(&channel, &payload).await {
                Ok(_) => true,
                Err(e) => {
                    warn!("Failed to publish message to channel {}: {}", channel, e);
                    false
                }
            }
        }
    }
}

/// Publish a server message to multiple players.
pub async fn notify_players(
    redis: &mut ConnectionManager,
    player_ids: &[String],
    message: ServerMessage,
) {
    for player_id_str in player_ids {
        if let Ok(player_id) = Uuid::parse_str(player_id_str) {
            notify_player(redis, player_id, message.clone()).await;
        }
    }
}
