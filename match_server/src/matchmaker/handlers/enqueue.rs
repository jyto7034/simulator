use actix::ResponseFuture;
use tracing::{error, info, warn};

use crate::matchmaker::{
    actor::Matchmaker,
    messages::EnqueuePlayer,
    runtime::{notify, queue as queue_rt},
};
use crate::{
    invariants,
    protocol::{ErrorCode, ServerMessage},
};
// metrics removed

pub(super) fn handle_enqueue(mm: &mut Matchmaker, msg: EnqueuePlayer) -> ResponseFuture<()> {
    let mut redis = mm.redis.clone();
    let settings = mm.settings.clone();
    let _run_id_store = mm.current_run_id.clone();

    Box::pin(async move {
        let is_valid_game_mode = settings.game_modes.iter().any(|m| m.id == msg.game_mode);
        if !is_valid_game_mode {
            warn!(
                "Player {} tried to enqueue for invalid game mode: {}",
                msg.player_id, msg.game_mode
            );
            invariants::emit_violation_kv(
                &mut redis,
                "invalid_game_mode_request",
                &[
                    ("player_id", msg.player_id.to_string()),
                    ("game_mode", msg.game_mode.clone()),
                ],
            )
            .await;
            notify::notify_player(
                &mut redis,
                msg.player_id,
                ServerMessage::Error {
                    code: Some(ErrorCode::InvalidGameMode),
                    message: format!("Invalid game mode: {}", msg.game_mode),
                },
            )
            .await;
            return;
        }

        let player_id_str = msg.player_id.to_string();
        let queue_key = format!("{}:{}", settings.queue_key_prefix, msg.game_mode);
        match queue_rt::add_player(&mut redis, &queue_key, &player_id_str).await {
            Ok(true) => {
                info!("Player {} added to queue {}", player_id_str, queue_key);
                notify::notify_player(&mut redis, msg.player_id, ServerMessage::EnQueued).await;
            }
            Ok(false) => {
                warn!("Player {} already in queue {}", player_id_str, queue_key);
                let _gm = msg.game_mode.clone();
                notify::notify_player(
                    &mut redis,
                    msg.player_id,
                    ServerMessage::Error {
                        code: Some(ErrorCode::AlreadyInQueue),
                        message: "Already in queue".to_string(),
                    },
                )
                .await;
            }
            Err(e) => {
                error!("Failed to add player to queue: {}", e);
                notify::notify_player(
                    &mut redis,
                    msg.player_id,
                    ServerMessage::Error {
                        code: Some(ErrorCode::InternalError),
                        message: "Internal server error".to_string(),
                    },
                )
                .await;
            }
        }
    })
}
