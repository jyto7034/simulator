use actix::ResponseFuture;
use redis::AsyncCommands;
use tracing::{error, info};

use crate::matchmaker::{actor::Matchmaker, messages::DequeuePlayer, runtime::queue as queue_rt};

pub(super) fn handle_dequeue(mm: &mut Matchmaker, msg: DequeuePlayer) -> ResponseFuture<()> {
    let mut redis = mm.redis.clone();
    let queue_key_prefix = mm.settings.queue_key_prefix.clone();

    Box::pin(async move {
        let queue_key = format!("{}:{}", queue_key_prefix, msg.game_mode);
        let player_id_str = msg.player_id.to_string();

        match queue_rt::remove_player(&mut redis, &queue_key, &player_id_str).await {
            Ok(true) => {
                info!(
                    "Player {} (disconnected) removed from queue {}",
                    player_id_str, queue_key
                );
                
                // Also cleanup queue_time key to prevent orphaned data
                let queue_time_key = format!("queue_time:{}", player_id_str);
                if let Err(e) = redis.del::<_, i32>(&queue_time_key).await {
                    tracing::debug!("Failed to cleanup queue_time key {}: {}", queue_time_key, e);
                }
            }
            Ok(false) => {
                tracing::debug!(
                    "Player {} was not in queue {}, likely already matched.",
                    player_id_str,
                    queue_key
                );
            }
            Err(e) => {
                error!(
                    "Failed to remove player {} from queue {}: {}",
                    player_id_str, queue_key, e
                );
            }
        }
    })
}
