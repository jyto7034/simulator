use actix::ResponseFuture;
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
