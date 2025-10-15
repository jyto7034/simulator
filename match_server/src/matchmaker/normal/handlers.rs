use actix::{dev::ContextFutureSpawner, ActorContext, Handler, WrapFuture};
use backoff::backoff::Backoff;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    matchmaker::{
        messages::{Dequeue, Enqueue, TryMatch},
        normal::NormalMatchmaker,
        operations::{
            dequeue::dequeue,
            enqueue::{enqueue, re_enqueue_candidates},
            notify,
            try_match::{pop_candidates, publish_battle_request},
        },
        MatchmakerDeps,
    },
    protocol::{BattleRequest, ErrorCode, ServerMessage},
    redis_events, GameMode, Stop, RETRY_CONFIG,
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
        let mut redis = deps.redis.clone();

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
                notify::send_message_to_player(
                    deps.subscription_addr,
                    &mut redis,
                    player_id,
                    ServerMessage::Error {
                        code: ErrorCode::InvalidGameMode,
                        message: "Invalid game mode".to_string(),
                    },
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
        let mut redis = deps.redis.clone();

        async move {
            if game_mode != GameMode::Normal {
                warn!(
                    "Player {} tried to dequeue using mismatched matchmaker for mode {:?}",
                    player_id, game_mode
                );
                notify::send_message_to_player(
                    deps.subscription_addr,
                    &mut redis,
                    player_id,
                    ServerMessage::Error {
                        code: ErrorCode::InvalidGameMode,
                        message: "Invalid game mode".to_string(),
                    },
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
        // redis 입출이 10초씩 걸렸던 이유는 풀링을 100ms 마다 했기 때문임.
        // 지금은 pub/sub 으로 변경해서 문제 해결함.

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
        let is_matching = self.is_matching.clone();

        // Set matching flag
        is_matching.store(true, std::sync::atomic::Ordering::Relaxed);

        info!("asd");

        // Spawn as actor future instead of tokio task
        async move {
            // 종료 신호 체크
            if shutdown_token.is_cancelled() {
                is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }

            // Circuit breaker 체크
            if let Err(e) = deps.redis_circuit.check() {
                warn!("Redis circuit open, skipping TryMatch: {}", e);
                is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            }

            let mut backoff = RETRY_CONFIG
                .read()
                .await
                .as_ref()
                .expect("Retry config not initialized")
                .clone();

            let candidates = loop {
                // 종료 신호 체크
                if shutdown_token.is_cancelled() {
                    is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                    return;
                }

                match pop_candidates(queue_suffix, required_players as usize * 2, &deps).await {
                    Ok((candidates, poisoned_player_ids)) => {
                        // Circuit breaker 성공 기록
                        deps.redis_circuit.record_success();

                        // poisoned 플레이어들에게 Dequeued + Error 메시지 전송
                        for player_id_str in poisoned_player_ids {
                            error!("Notifying poisoned candidate {} that they were dequeued", player_id_str);

                            if let Ok(player_uuid) = Uuid::parse_str(&player_id_str) {
                                // Dequeued 메시지 전송
                                notify::send_message_to_player(
                                    subscription_addr.clone(),
                                    &mut redis,
                                    player_uuid,
                                    ServerMessage::DeQueued,
                                )
                                .await;

                                // Error 메시지도 전송 (왜 dequeue되었는지)
                                notify::send_message_to_player(
                                    subscription_addr.clone(),
                                    &mut redis,
                                    player_uuid,
                                    ServerMessage::Error {
                                        code: ErrorCode::InvalidMetadata,
                                        message: "Invalid player metadata - removed from queue".to_string(),
                                    },
                                )
                                .await;
                            } else {
                                error!("Failed to parse poisoned player_id as UUID: {}", player_id_str);
                            }
                        }

                        break candidates;
                    }
                    Err(err) => {
                        // Circuit breaker 실패 기록
                        deps.redis_circuit.record_failure();
                        if let Some(delay) = backoff.next_backoff() {
                            warn!(
                                "Failed to pop candidates from queue {}: {} (retrying in {:?})",
                                queue_suffix, err, delay
                            );
                            // 종료 신호와 함께 대기
                            tokio::select! {
                                _ = sleep(delay) => continue,
                                _ = shutdown_token.cancelled() => {
                                    is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                                    return;
                                }
                            }
                        } else {
                            error!(
                                "Failed to pop candidates after all retries from queue {}: {}",
                                queue_suffix, err
                            );
                            is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                            return; // async move 블록 종료
                        }
                    }
                }
            };

            if candidates.is_empty() {
                warn!("No candidates available");
                is_matching.store(false, std::sync::atomic::Ordering::Relaxed);
                return;
            } else {
                info!("Found some candidates!, {}", candidates.len());
            }

            // try_match 에 의해 4명이 수집되는데, 모종의 이유로 0~3명이 수집될 수 있음.
            // candidates 에서 2명씩 꺼내어 매칭을 시도하고, 남은 인원은 Re enqueue
            // 2명씩 묶어서 처리
            for chunk in candidates.chunks(2) {
                // 종료 신호 체크 - 이미 pop한 candidates는 re-enqueue
                if shutdown_token.is_cancelled() {
                    warn!("Shutdown requested, re-enqueueing remaining candidates");
                    re_enqueue_candidates(queue_suffix, settings.game_mode, chunk, &deps).await;
                    continue;
                }

                match chunk {
                    [player1, player2] => {
                        info!("Trying to match");
                        // 2명 매칭
                        let request = BattleRequest {
                            player1: player1.clone(),
                            player2: player2.clone(),
                        };

                        let timeout_secs = deps.settings.redis_operation_timeout_seconds;
                        match publish_battle_request(
                            &mut redis,
                            &deps.settings.battle_request_channel,
                            &request,
                            timeout_secs,
                        )
                        .await
                        {
                            Ok(subscriber_count) => {
                                if subscriber_count == 0 && !deps.settings.skip_game_server_check {
                                    // TODO: Game Server 가 구독중이지 않음. -> Game Server 가 죽어있을 가능성이 존재함.
                                    // 자세하게 오류 파악하고 관리 시스템에게 보고 해야함 ( Orchestrator )
                                    warn!("No Game Server is subscribed to battle:request channel");
                                    metrics::GAME_SERVER_UNAVAILABLE_TOTAL.inc();
                                    metrics::GAME_SERVER_AVAILABLE.set(0);
                                    // 매칭 실패: player1, player2 re enqueue
                                    let failed_match = [player1.clone(), player2.clone()];
                                    re_enqueue_candidates(
                                        queue_suffix,
                                        settings.game_mode,
                                        &failed_match,
                                        &deps,
                                    )
                                    .await;
                                } else {
                                    if subscriber_count == 0 {
                                        info!("Development mode: Skipping game server check, treating match as successful");
                                    } else {
                                        // Game server is available
                                        metrics::GAME_SERVER_AVAILABLE.set(1);
                                    }

                                    // Metrics: 매칭 성공
                                    metrics::MATCHES_CREATED_TOTAL.inc();
                                    metrics::MATCHED_PLAYERS_TOTAL_BY_MODE
                                        .with_label_values(&[&format!("{:?}", settings.game_mode)])
                                        .inc_by(2);

                                    info!(
                                        "Battle request sent to {} Game Server(s) for players {} and {}",
                                        subscriber_count, player1.player_id, player2.player_id
                                    );

                                    // Publish test events for both players
                                    if let Ok(metadata1_str) =
                                        serde_json::to_string(&player1.metadata)
                                    {
                                        redis_events::try_publish_test_event(
                                            &mut redis,
                                            &metadata1_str,
                                            "player.match_found",
                                            &player1.pod_id,
                                            vec![
                                                ("player_id", player1.player_id.clone()),
                                                ("opponent_id", player2.player_id.clone()),
                                                ("game_mode", format!("{:?}", settings.game_mode)),
                                            ],
                                        )
                                        .await;
                                    }

                                    if let Ok(metadata2_str) =
                                        serde_json::to_string(&player2.metadata)
                                    {
                                        redis_events::try_publish_test_event(
                                            &mut redis,
                                            &metadata2_str,
                                            "player.match_found",
                                            &player2.pod_id,
                                            vec![
                                                ("player_id", player2.player_id.clone()),
                                                ("opponent_id", player1.player_id.clone()),
                                                ("game_mode", format!("{:?}", settings.game_mode)),
                                            ],
                                        )
                                        .await;
                                    }

                                    // Send MatchFound to both players via WebSocket
                                    if let (Ok(p1_uuid), Ok(p2_uuid)) = (
                                        Uuid::parse_str(&player1.player_id),
                                        Uuid::parse_str(&player2.player_id),
                                    ) {
                                        notify::send_message_to_player(
                                            subscription_addr.clone(),
                                            &mut redis,
                                            p1_uuid,
                                            ServerMessage::MatchFound,
                                        )
                                        .await;

                                        notify::send_message_to_player(
                                            subscription_addr.clone(),
                                            &mut redis,
                                            p2_uuid,
                                            ServerMessage::MatchFound,
                                        )
                                        .await;
                                    } else {
                                        error!(
                                            "Failed to parse player IDs as UUID: {} and {}",
                                            player1.player_id, player2.player_id
                                        );
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Failed to publish battle request: {}", err);
                                // 매칭 실패: player1, player2 re enqueue
                                let failed_match = [player1.clone(), player2.clone()];
                                re_enqueue_candidates(
                                    queue_suffix,
                                    settings.game_mode,
                                    &failed_match,
                                    &deps,
                                )
                                .await;
                            }
                        }
                    }
                    [single] => {
                        // 1명 남음, re enqueue
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
