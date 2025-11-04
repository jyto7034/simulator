use actix::{dev::ContextFutureSpawner, ActorContext, Handler, WrapFuture};
use tracing::{error, info, warn};

use crate::{
    matchmaking::matchmaker::{
        messages::{Dequeue, Enqueue, TryMatch},
        normal::NormalMatchmaker,
        operations::{
            dequeue::dequeue,
            enqueue::{enqueue, re_enqueue_candidates},
            notify::{self, MessageRoutingDeps},
            try_match_collect::collect_candidates_with_retry,
            try_match_process::process_match_pair,
        },
        MatchmakerDeps,
    },
    shared::protocol::{ErrorCode, ServerMessage},
    GameMode, Stop,
};

impl Handler<Enqueue> for NormalMatchmaker {
    type Result = ();

    fn handle(&mut self, msg: Enqueue, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "NormalMatchmaker: Enqueue handler called for player {}",
            msg.player_id
        );
        let deps: MatchmakerDeps = (&self.inner).into();
        let game_mode = msg.game_mode;
        let queue_prefix = self.queue_suffix(game_mode);
        let player_id = msg.player_id;
        let routing_deps = MessageRoutingDeps::from(&deps);

        async move {
            info!(
                "NormalMatchmaker: Enqueue async block started for player {}",
                player_id
            );
            if game_mode != GameMode::Normal {
                warn!(
                    "Player {} tried to enqueue using mismatched matchmaker for mode {:?}",
                    player_id, game_mode
                );
                notify::send_message_to_player_by_id(
                    player_id,
                    ServerMessage::Error {
                        code: ErrorCode::InvalidGameMode,
                        message: "Invalid game mode".to_string(),
                    },
                    &routing_deps,
                )
                .await;
                return;
            }
            enqueue(queue_prefix, msg.game_mode, player_id, msg.metadata, &deps).await;
        }
        .into_actor(self)
        .spawn(ctx);
    }
}

impl Handler<Dequeue> for NormalMatchmaker {
    type Result = ();

    fn handle(&mut self, msg: Dequeue, ctx: &mut Self::Context) -> Self::Result {
        let deps: MatchmakerDeps = (&self.inner).into();
        let game_mode = msg.game_mode;
        let queue_prefix = self.queue_suffix(game_mode);
        let player_id = msg.player_id;
        let routing_deps = MessageRoutingDeps::from(&deps);

        async move {
            if game_mode != GameMode::Normal {
                warn!(
                    "Player {} tried to dequeue using mismatched matchmaker for mode {:?}",
                    player_id, game_mode
                );
                notify::send_message_to_player_by_id(
                    player_id,
                    ServerMessage::Error {
                        code: ErrorCode::InvalidGameMode,
                        message: "Invalid game mode".to_string(),
                    },
                    &routing_deps,
                )
                .await;
                return;
            }
            dequeue(queue_prefix, msg.game_mode, player_id, &deps).await;
        }
        .into_actor(self)
        .spawn(ctx);
    }
}

impl Handler<TryMatch> for NormalMatchmaker {
    type Result = ();

    fn handle(&mut self, msg: TryMatch, ctx: &mut Self::Context) -> Self::Result {
        // Skip if already matching
        if self.is_matching.load(std::sync::atomic::Ordering::Relaxed) {
            info!("TryMatch already in progress, skipping this tick");
            metrics::TRY_MATCH_SKIPPED_TOTAL.inc();
            return;
        }

        let deps: MatchmakerDeps = (&self.inner).into();
        let settings = msg.match_mode_settings;
        let queue_suffix = self.queue_suffix(settings.game_mode);
        let required_players = settings.required_players;
        let mut redis = deps.redis.clone();
        let shutdown_token = self.shutdown_token.clone();
        let subscription_addr = self.sub_manager_addr.clone();
        let load_balance_addr = self.inner.load_balance_addr.clone();
        let is_matching = self.is_matching.clone();

        // Set matching flag
        is_matching.store(true, std::sync::atomic::Ordering::Relaxed);

        // Spawn async future
        async move {
            // ===== 1. Early exit checks =====
            if shutdown_token.is_cancelled() {
                is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }

            if let Err(e) = deps.redis_circuit.check() {
                warn!("Redis circuit open, skipping TryMatch: {}", e);
                is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }

            // ===== 2. Collect candidates =====
            let candidates = match collect_candidates_with_retry(
                queue_suffix,
                required_players as usize * 2,
                &deps,
                &shutdown_token,
            )
            .await
            {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to collect candidates: {}", e);
                    is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                    return;
                }
            };

            if candidates.is_empty() {
                warn!("No candidates available");
                is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }

            info!("Found {} candidates", candidates.len());

            // ===== 3. Process matches =====
            for chunk in candidates.chunks(2) {
                // Shutdown check
                if shutdown_token.is_cancelled() {
                    warn!("Shutdown requested, re-enqueueing remaining candidates");
                    re_enqueue_candidates(queue_suffix, settings.game_mode, chunk, &deps).await;
                    continue;
                }

                match chunk {
                    [player1, player2] => {
                        // Process match pair
                        process_match_pair(
                            player1,
                            player2,
                            settings.game_mode,
                            queue_suffix,
                            &deps,
                            &mut redis,
                            subscription_addr.clone(),
                            &load_balance_addr,
                        )
                        .await;
                    }
                    [single] => {
                        // Re-enqueue single player
                        info!("Single player left, re-enqueueing: {}", single.player_id);
                        re_enqueue_candidates(queue_suffix, settings.game_mode, chunk, &deps).await;
                    }
                    _ => unreachable!("chunks(2) only returns 1 or 2 elements"),
                }
            }

            // Release matching flag
            is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        .into_actor(self)
        .spawn(ctx);
    }
}

impl Handler<Stop> for NormalMatchmaker {
    type Result = ();

    fn handle(&mut self, msg: Stop, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "NormalMatchmaker for mode {:?} stopping: {:?}",
            self.mode_settings.game_mode, msg.reason
        );
        ctx.stop();
    }
}
