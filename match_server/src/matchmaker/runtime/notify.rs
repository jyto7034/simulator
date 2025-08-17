use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::warn;
use uuid::Uuid;

use crate::protocol::ServerMessage;

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
