use crate::{protocol::ServerMessage, provider::FindAvailableServer};
use actix::{AsyncContext, Handler, ResponseFuture};
use futures_util::stream::StreamExt;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Script};
use serde::{Deserialize, Serialize};
use simulator_metrics::{
    HTTP_TIMEOUT_ERRORS_TOTAL, MATCHES_CREATED_TOTAL, MATCHMAKING_ERRORS_TOTAL, PLAYERS_IN_QUEUE,
    SYSTEM_TIME_ERRORS_TOTAL,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use uuid::Uuid;

// test_client 작성해서 시나리오 테스트 해야함.

use super::{
    actor::Matchmaker,
    lock::DistributedLock, // DistributedLock 임포트
    messages::*,
    scripts::{
        get_atomic_cancel_session_script, get_atomic_loading_complete_script,
        get_atomic_match_script, get_cleanup_stale_session_script,
    },
};

const LOCK_DURATION_MS: usize = 30_000; // 30초

// --- Helper Functions ---
async fn publish_message(redis: &mut ConnectionManager, player_id: Uuid, message: ServerMessage) {
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

async fn requeue_players(redis: &mut ConnectionManager, queue_key: &str, player_ids: &[String]) {
    warn!("Re-queuing players due to an error: {:?}", player_ids);
    if player_ids.is_empty() {
        return;
    }
    PLAYERS_IN_QUEUE.add(player_ids.len() as i64);
    let result: Result<i32, _> = redis.sadd(queue_key, player_ids).await;
    if let Err(e) = result {
        error!(
            "CRITICAL: Failed to re-queue players {:?} into {}: {}",
            player_ids, queue_key, e
        );
    }
}

// --- Message Handlers ---

/// EnqueuePlayer: 플레이어를 큐에 추가하는 메시지
impl Handler<EnqueuePlayer> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: EnqueuePlayer, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let settings = self.settings.clone();

        Box::pin(async move {
            // 게임 모드 유효성 검사
            let is_valid_game_mode = settings.game_modes.iter().any(|m| m.id == msg.game_mode);
            if !is_valid_game_mode {
                warn!(
                    "Player {} tried to enqueue for invalid game mode: {}",
                    msg.player_id, msg.game_mode
                );
                publish_message(
                    &mut redis,
                    msg.player_id,
                    ServerMessage::Error {
                        message: format!("Invalid game mode: {}", msg.game_mode),
                    },
                )
                .await;
                return;
            }

            let player_id_str = msg.player_id.to_string();
            let queue_key = format!("{}:{}", settings.queue_key_prefix, msg.game_mode);

            // Redis SADD는 원자적이므로 락 불필요
            let result: Result<i32, _> = redis.sadd(&queue_key, &player_id_str).await;
            match result {
                Ok(count) if count > 0 => {
                    info!("Player {} added to queue {}", player_id_str, queue_key);
                    PLAYERS_IN_QUEUE.inc();
                    publish_message(&mut redis, msg.player_id, ServerMessage::EnQueued).await;
                }
                Ok(_) => {
                    warn!("Player {} already in queue {}", player_id_str, queue_key);
                    publish_message(
                        &mut redis,
                        msg.player_id,
                        ServerMessage::Error {
                            message: "Already in queue".to_string(),
                        },
                    )
                    .await;
                }
                Err(e) => {
                    error!("Failed to add player to queue: {}", e);
                    publish_message(
                        &mut redis,
                        msg.player_id,
                        ServerMessage::Error {
                            message: "Internal server error".to_string(),
                        },
                    )
                    .await;
                }
            }
        })
    }
}

impl Handler<DequeuePlayer> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: DequeuePlayer, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let queue_key_prefix = self.settings.queue_key_prefix.clone();
        Box::pin(async move {
            let queue_key = format!("{}:{}", queue_key_prefix, msg.game_mode);
            let player_id_str = msg.player_id.to_string();

            // Redis SREM은 원자적이므로 락 불필요
            let result: Result<i32, _> = redis.srem(&queue_key, &player_id_str).await;
            match result {
                Ok(count) if count > 0 => {
                    info!(
                        "Player {} (disconnected) removed from queue {}",
                        player_id_str, queue_key
                    );
                    PLAYERS_IN_QUEUE.dec();
                }
                Ok(_) => {
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
}

impl Handler<TryMatch> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: TryMatch, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let game_mode_settings = msg.game_mode.clone();
        let settings = self.settings.clone();

        Box::pin(async move {
            let queue_key = format!("{}:{}", settings.queue_key_prefix, game_mode_settings.id);
            let required_players = game_mode_settings.required_players;
            let lock_key = format!("lock:match:{}", game_mode_settings.id);

            if game_mode_settings.use_mmr_matching {
                warn!(
                    "MMR-based matching for '{}' is not yet implemented. Falling back to simple matching.",
                    game_mode_settings.id
                );
            }

            let lock = match DistributedLock::acquire(&mut redis, &lock_key, LOCK_DURATION_MS).await
            {
                Ok(Some(lock)) => lock,
                Ok(None) => return,
                Err(e) => {
                    error!(
                        "Failed to acquire lock for matching in {}: {}",
                        game_mode_settings.id, e
                    );
                    return;
                }
            };

            let current_timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(d) => d.as_secs().to_string(),
                Err(e) => {
                    SYSTEM_TIME_ERRORS_TOTAL.inc();
                    error!(
                        "System time is before UNIX EPOCH, cannot get timestamp: {}",
                        e
                    );
                    if let Err(e) = lock.release(&mut redis).await {
                        error!("Failed to release lock: {}", e);
                    }
                    return;
                }
            };

            let loading_session_id = Uuid::new_v4();
            let script = Script::new(get_atomic_match_script());
            let script_result: Vec<String> = match script
                .key(&queue_key)
                .arg(required_players)
                .arg(loading_session_id.to_string())
                .arg(current_timestamp)
                .arg(settings.loading_session_timeout_seconds)
                .invoke_async(&mut redis)
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    error!("Matchmaking script failed for queue {}: {}", queue_key, e);
                    if let Err(e) = lock.release(&mut redis).await {
                        error!("Failed to release lock: {}", e);
                    }
                    return;
                }
            };

            if script_result.len() as u32 >= (required_players + 2) {
                let game_mode = script_result.get(0).cloned().unwrap_or_default();
                let returned_loading_session_id = script_result
                    .get(1)
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_else(|| {
                        error!("Could not parse loading session ID from script result.");
                        loading_session_id // Fallback to the one we generated
                    });
                let player_ids: Vec<String> = script_result[2..].to_vec();

                PLAYERS_IN_QUEUE.sub(required_players as i64);
                info!(
                    "[{}] Found a potential match with players: {:?} for session {}",
                    game_mode, player_ids, returned_loading_session_id
                );

                let message = ServerMessage::StartLoading {
                    loading_session_id: returned_loading_session_id,
                };
                for player_id_str in &player_ids {
                    if let Ok(player_id) = Uuid::parse_str(player_id_str) {
                        publish_message(&mut redis, player_id, message.clone()).await;
                    } else {
                        warn!(
                            "Could not parse player UUID from script result: {}",
                            player_id_str
                        );
                    }
                }
            }

            if let Err(e) = lock.release(&mut redis).await {
                error!(
                    "Failed to release lock for matching in {}: {}",
                    game_mode_settings.id, e
                );
            }
        })
    }
}

impl Handler<HandleLoadingComplete> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: HandleLoadingComplete, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let http_client = self.http_client.clone();
        let provider_addr = self.provider_addr.clone();
        let queue_key_prefix = self.settings.queue_key_prefix.clone();

        Box::pin(async move {
            let loading_key = format!("loading:{}", msg.loading_session_id);
            let player_id_str = msg.player_id.to_string();
            let lock_key = format!("lock:{}", loading_key);

            let lock = match DistributedLock::acquire(&mut redis, &lock_key, LOCK_DURATION_MS).await
            {
                Ok(Some(lock)) => lock,
                Ok(None) => {
                    info!(
                        "Could not acquire lock for session {}, another process is handling it.",
                        msg.loading_session_id
                    );
                    return;
                }
                Err(e) => {
                    error!(
                        "Failed to acquire lock for session {}: {}",
                        msg.loading_session_id, e
                    );
                    return;
                }
            };

            let script = Script::new(get_atomic_loading_complete_script());
            let result: Result<Vec<String>, _> = script
                .key(&loading_key)
                .arg(&player_id_str)
                .invoke_async(&mut redis)
                .await;

            let mut script_result: Vec<String> = match result {
                Ok(ids) if !ids.is_empty() => ids,
                Ok(_) => {
                    info!(
                        "Player {} is ready, but waiting for others in session {}.",
                        player_id_str, msg.loading_session_id
                    );
                    if let Err(e) = lock.release(&mut redis).await {
                        error!(
                            "Failed to release lock for session {}: {}",
                            msg.loading_session_id, e
                        );
                    }
                    return;
                }
                Err(e) => {
                    error!(
                        "Atomic loading script failed for session {}: {}",
                        msg.loading_session_id, e
                    );
                    if let Err(e) = lock.release(&mut redis).await {
                        error!(
                            "Failed to release lock for session {}: {}",
                            msg.loading_session_id, e
                        );
                    }
                    return;
                }
            };

            let game_mode = if !script_result.is_empty() {
                script_result.remove(0)
            } else {
                error!("Script result for loading complete is empty, cannot proceed.");
                if let Err(e) = lock.release(&mut redis).await {
                    error!("Failed to release lock: {}", e);
                }
                return;
            };
            let player_ids = script_result;

            info!(
                "All players {:?} are ready for session {}. Finding a dedicated server...",
                player_ids, msg.loading_session_id
            );

            let find_server_result = provider_addr.send(FindAvailableServer).await;

            match find_server_result {
                Ok(Ok(server_info)) => {
                    let create_session_url =
                        format!("http://{}/session/create", server_info.address);
                    #[derive(Serialize)]
                    struct CreateSessionReq {
                        players: Vec<Uuid>,
                    }
                    let req_body = CreateSessionReq {
                        players: player_ids
                            .iter()
                            .filter_map(|id| Uuid::parse_str(id).ok())
                            .collect(),
                    };

                    match http_client
                        .post(&create_session_url)
                        .json(&req_body)
                        .timeout(Duration::from_secs(5))
                        .send()
                        .await
                    {
                        Ok(resp) if resp.status().is_success() => {
                            #[derive(Deserialize, Debug)]
                            struct CreateSessionResp {
                                server_address: String,
                                session_id: Uuid,
                            }

                            match resp.json::<CreateSessionResp>().await {
                                Ok(session_info) => {
                                    info!(
                                        "[{}] Successfully created session: {:?}",
                                        game_mode, session_info
                                    );
                                    MATCHES_CREATED_TOTAL.inc();
                                    let message = ServerMessage::MatchFound {
                                        session_id: session_info.session_id,
                                        server_address: session_info.server_address.clone(),
                                    };
                                    for player_id_str in &player_ids {
                                        if let Ok(player_id) = Uuid::parse_str(player_id_str) {
                                            publish_message(&mut redis, player_id, message.clone())
                                                .await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                                    MATCHMAKING_ERRORS_TOTAL
                                        .with_label_values(&["session_response_parse_failed"])
                                        .inc();
                                    error!("[{}] Failed to parse session creation response: {}. Re-queuing players.", game_mode, e);
                                    requeue_players(&mut redis, &queue_key, &player_ids).await;
                                }
                            }
                        }
                        Ok(resp) => {
                            let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                            MATCHMAKING_ERRORS_TOTAL
                                .with_label_values(&["dedicated_server_error_response"])
                                .inc();
                            error!(
                                "[{}] Dedicated server returned error: {}. Re-queuing players.",
                                game_mode,
                                resp.status()
                            );
                            requeue_players(&mut redis, &queue_key, &player_ids).await;
                        }
                        Err(e) => {
                            let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                            if e.is_timeout() {
                                HTTP_TIMEOUT_ERRORS_TOTAL
                                    .with_label_values(&["dedicated_server"])
                                    .inc();
                            }
                            MATCHMAKING_ERRORS_TOTAL
                                .with_label_values(&["dedicated_server_request_failed"])
                                .inc();
                            error!(
                                "[{}] Failed to contact dedicated server (timeout or network error): {}. Re-queuing players.",
                                game_mode, e
                            );
                            requeue_players(&mut redis, &queue_key, &player_ids).await;
                        }
                    }
                }
                Ok(Err(e)) => {
                    let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                    MATCHMAKING_ERRORS_TOTAL
                        .with_label_values(&["server_provider_failed"])
                        .inc();
                    error!(
                        "[{}] Failed to find available server: {}. Re-queuing players.",
                        game_mode, e
                    );
                    requeue_players(&mut redis, &queue_key, &player_ids).await;
                }
                Err(e) => {
                    let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                    MATCHMAKING_ERRORS_TOTAL
                        .with_label_values(&["server_provider_mailbox_error"])
                        .inc();
                    error!(
                        "[{}] Mailbox error when contacting provider: {}. Re-queuing players.",
                        game_mode, e
                    );
                    requeue_players(&mut redis, &queue_key, &player_ids).await;
                }
            }

            if let Err(e) = lock.release(&mut redis).await {
                error!(
                    "Failed to release lock for session {}: {}",
                    msg.loading_session_id, e
                );
            }
        })
    }
}

impl Handler<CancelLoadingSession> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: CancelLoadingSession, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let queue_key_prefix = self.settings.queue_key_prefix.clone();

        Box::pin(async move {
            let loading_key = format!("loading:{}", msg.loading_session_id);
            let disconnected_player_id_str = msg.player_id.to_string();

            info!(
                "Attempting to cancel loading session {} due to player {} disconnection.",
                msg.loading_session_id, msg.player_id
            );

            let script = Script::new(get_atomic_cancel_session_script());
            let result: Result<Vec<String>, _> = script
                .key(&loading_key)
                .arg(&disconnected_player_id_str)
                .invoke_async(&mut redis)
                .await;

            let mut script_result = match result {
                Ok(val) if !val.is_empty() => val,
                Ok(_) => {
                    warn!(
                        "Loading session {} already handled or cleaned up before cancellation.",
                        msg.loading_session_id
                    );
                    return;
                }
                Err(e) => {
                    error!(
                        "Failed to run cancellation script for session {}: {}",
                        msg.loading_session_id, e
                    );
                    return;
                }
            };

            let game_mode = if !script_result.is_empty() {
                script_result.remove(0)
            } else {
                error!("Script result for cancellation is empty, cannot proceed.");
                return;
            };
            let players_to_requeue = script_result;

            if !players_to_requeue.is_empty() {
                info!(
                    "Notifying remaining players {:?} and re-queuing them for game mode '{}'.",
                    players_to_requeue, game_mode
                );
                let queue_key = format!("{}:{}", queue_key_prefix, game_mode);

                let message = ServerMessage::Error {
                    message:
                        "A player disconnected during loading. You have been returned to the queue."
                            .to_string(),
                };

                for player_id_str in &players_to_requeue {
                    if let Ok(player_id) = Uuid::parse_str(player_id_str) {
                        publish_message(&mut redis, player_id, message.clone()).await;
                    }
                }
                requeue_players(&mut redis, &queue_key, &players_to_requeue).await;
            }
        })
    }
}
impl Handler<CheckStaleLoadingSessions> for Matchmaker {
    type Result = ResponseFuture<()>;

    fn handle(
        &mut self,
        _msg: CheckStaleLoadingSessions,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let mut redis = self.redis.clone();
        let matchmaker_addr = _ctx.address();
        let settings = self.settings.clone();

        Box::pin(async move {
            info!("Checking for stale loading sessions...");

            let mut keys: Vec<String> = Vec::new();
            match redis.scan_match::<_, String>("loading:*").await {
                Ok(mut iter) => {
                    while let Some(key) = iter.next().await {
                        keys.push(key);
                    }
                }
                Err(e) => {
                    error!("Failed to scan loading sessions: {}", e);
                    return;
                }
            };

            for key in keys {
                let lock_key = format!("lock:{}", key);

                let lock =
                    match DistributedLock::acquire(&mut redis, &lock_key, LOCK_DURATION_MS).await {
                        Ok(Some(lock)) => lock,
                        _ => continue,
                    };

                let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(d) => d.as_secs(),
                    Err(e) => {
                        SYSTEM_TIME_ERRORS_TOTAL.inc();
                        error!(
                            "System time is before UNIX EPOCH, cannot check stale sessions: {}",
                            e
                        );
                        if let Err(e) = lock.release(&mut redis).await {
                            error!("Failed to release lock: {}", e);
                        }
                        continue;
                    }
                };

                let script = Script::new(get_cleanup_stale_session_script());
                let result: Result<Vec<String>, _> = script
                    .key(&key)
                    .arg(now as i64)
                    .arg(settings.loading_session_timeout_seconds as i64)
                    .invoke_async(&mut redis)
                    .await;

                let mut script_result = match result {
                    Ok(val) if !val.is_empty() => val,
                    Ok(_) => {
                        if let Err(e) = lock.release(&mut redis).await {
                            error!(
                                "Failed to release lock for stale check on key {}: {}",
                                key, e
                            );
                        }
                        continue;
                    }
                    Err(e) => {
                        error!(
                            "Failed to run stale session cleanup script for key {}: {}",
                            key, e
                        );
                        if let Err(e) = lock.release(&mut redis).await {
                            error!(
                                "Failed to release lock for stale check on key {}: {}",
                                key, e
                            );
                        }
                        continue;
                    }
                };

                let game_mode = if !script_result.is_empty() {
                    script_result.remove(0)
                } else {
                    error!("Script result for stale check is empty, cannot proceed.");
                    if let Err(e) = lock.release(&mut redis).await {
                        error!("Failed to release lock: {}", e);
                    }
                    continue;
                };
                let players_to_requeue = script_result;

                if !players_to_requeue.is_empty() {
                    warn!(
                        "Found stale loading session {}. Scheduling re-queuing for players {:?} for game mode '{}'.",
                        key, players_to_requeue, game_mode
                    );

                    let message = ServerMessage::Error {
                        message:
                            "Matchmaking timed out. You will be returned to the queue shortly."
                                .to_string(),
                    };
                    for player_id_str in &players_to_requeue {
                        if let Ok(player_id) = Uuid::parse_str(player_id_str) {
                            publish_message(&mut redis, player_id, message.clone()).await;
                        }
                    }

                    matchmaker_addr.do_send(DelayedRequeuePlayers {
                        player_ids: players_to_requeue,
                        game_mode: game_mode,
                        delay: Duration::from_secs(5),
                    });
                }

                if let Err(e) = lock.release(&mut redis).await {
                    error!(
                        "Failed to release lock for stale check on key {}: {}",
                        key, e
                    );
                }
            }
        })
    }
}

impl Handler<DelayedRequeuePlayers> for Matchmaker {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: DelayedRequeuePlayers, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let queue_key_prefix = self.settings.queue_key_prefix.clone();

        Box::pin(async move {
            info!(
                "Re-queuing players {:?} for game mode {} after delay.",
                msg.player_ids, msg.game_mode
            );
            let queue_key = format!("{}:{}", queue_key_prefix, msg.game_mode);
            requeue_players(&mut redis, &queue_key, &msg.player_ids).await;
        })
    }
}
