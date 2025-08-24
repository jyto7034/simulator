use crate::blacklist::messages::RecordViolation;
use crate::blacklist::ViolationType;
use crate::pubsub::{GetActiveSessionsDebug, ValidateActivePlayers};
use crate::{protocol::ServerMessage, state_events::StateEventEmitter};
use actix::{AsyncContext, Handler, ResponseFuture};
use futures_util::stream::StreamExt;
// metrics removed
use redis::{AsyncCommands, Script};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::{
    actor::Matchmaker,
    lock::{DistributedLock, LockResult},
    messages::*,
    scripts::{
        get_atomic_cancel_session_script, get_atomic_match_script, get_cleanup_stale_session_script,
    },
};

mod dequeue;
mod enqueue;
mod loading;

const LOCK_DURATION_MS: usize = 30_000; // 30초

// Use unified messaging from runtime module
use crate::matchmaker::runtime::notify::{notify_player, send_player_message};

// requeue_players: 런타임 유틸 사용으로 일원화 (runtime::queue::requeue_players)

// --- Message Handlers ---

/// EnqueuePlayer 위임: 세부 로직은 handlers/enqueue.rs
impl Handler<EnqueuePlayer> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: EnqueuePlayer, _ctx: &mut Self::Context) -> Self::Result {
        enqueue::handle_enqueue(self, msg)
    }
}

/// DequeuePlayer 위임: 세부 로직은 handlers/dequeue.rs
impl Handler<DequeuePlayer> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: DequeuePlayer, _ctx: &mut Self::Context) -> Self::Result {
        dequeue::handle_dequeue(self, msg)
    }
}

impl Handler<TryMatch> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: TryMatch, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let game_mode_settings = msg.game_mode.clone();
        let settings = self.settings.clone();
        let sub_manager_addr = self.sub_manager_addr.clone();
        let metrics = self.metrics.clone();
        let loading_session_manager_addr = self.loading_session_manager_addr.clone();

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
                Ok((Some(lock), LockResult::Acquired)) => lock,
                Ok((None, LockResult::Busy { remaining_ttl_ms })) => {
                    info!(
                        "Lock for matching in {} is busy, remaining TTL: {}ms",
                        game_mode_settings.id, remaining_ttl_ms
                    );
                    return;
                },
                Ok((None, LockResult::Error(msg))) => {
                    error!(
                        "Lock acquisition error for matching in {}: {}",
                        game_mode_settings.id, msg
                    );
                    return;
                },
                Ok((None, LockResult::Acquired)) => {
                    error!(
                        "Impossible lock state for matching in {}",
                        game_mode_settings.id
                    );
                    return;
                },
                Ok((Some(_), LockResult::Busy { .. })) => {
                    error!(
                        "Unexpected lock state for matching in {}",
                        game_mode_settings.id
                    );
                    return;
                },
                Ok((Some(_), LockResult::Error(_))) => {
                    error!(
                        "Unexpected lock state for matching in {}",
                        game_mode_settings.id
                    );
                    return;
                },
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
                    error!(
                        "System time is before UNIX EPOCH, cannot get timestamp: {}",
                        e
                    );
                    match lock.release(&mut redis).await {
                        Ok(true) => info!("Lock released successfully after timestamp error"),
                        Ok(false) => warn!("Lock was already released or owned by another process"),
                        Err(e) => error!("Failed to release lock after timestamp error: {}", e),
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
                .arg(&current_timestamp)
                .arg(settings.loading_session_timeout_seconds)
                .invoke_async(&mut redis)
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    error!("Matchmaking script failed for queue {}: {}", queue_key, e);
                    match lock.release(&mut redis).await {
                        Ok(true) => info!("Lock released successfully after script failure"),
                        Ok(false) => warn!("Lock was already released or owned by another process"),
                        Err(e) => error!("Failed to release lock after script failure: {}", e),
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

                info!(
                    "[{}] Found a potential match with players: {:?} for session {}",
                    game_mode, player_ids, returned_loading_session_id
                );

                // Validate that all matched players are still actively connected
                let player_uuids: Vec<Uuid> = player_ids
                    .iter()
                    .filter_map(|s| Uuid::parse_str(s).ok())
                    .collect();

                let active_players = match sub_manager_addr
                    .send(ValidateActivePlayers {
                        player_ids: player_uuids.clone(),
                    })
                    .await
                {
                    Ok(active) => active,
                    Err(e) => {
                        error!("Failed to validate active players: {}", e);
                        // Continue with original logic if validation fails
                        player_uuids.clone()
                    }
                };

                if active_players.len() != player_uuids.len() {
                    let inactive_players: Vec<String> = player_uuids
                        .iter()
                        .filter(|pid| !active_players.contains(pid))
                        .map(|pid| pid.to_string())
                        .collect();

                    warn!(
                        "[{}] Cancelling match session {} - inactive players detected: {:?}",
                        game_mode, returned_loading_session_id, inactive_players
                    );

                    // Cancel the loading session and requeue active players
                    let cancel_script = Script::new(get_atomic_cancel_session_script());
                    let loading_key = format!("loading:{}", returned_loading_session_id);
                    let _: () = cancel_script
                        .key(&loading_key)
                        .invoke_async(&mut redis)
                        .await
                        .unwrap_or_default();

                    // Requeue only the active players
                    if !active_players.is_empty() {
                        let active_player_strings: Vec<String> =
                            active_players.iter().map(|pid| pid.to_string()).collect();
                        crate::matchmaker::runtime::queue::requeue_players(
                            &mut redis,
                            &queue_key,
                            &active_player_strings,
                            &metrics,
                        )
                        .await;
                    }

                    if let Err(e) = lock.release(&mut redis).await {
                        error!("Failed to release lock: {}", e);
                    }
                    return;
                }

                // metrics: observe queue wait duration for each matched player at match time (TryMatch)
                // also count matched players by mode
                metrics.inc_matched_players_by_mode(&game_mode, player_ids.len() as u64);
                for pid in &player_ids {
                    let key = format!("queue_time:{}", pid);
                    if let Ok(Some(start_str)) = redis.get::<_, Option<String>>(&key).await {
                        if let Ok(start_secs) = start_str.parse::<u64>() {
                            if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                                let wait = now.as_secs().saturating_sub(start_secs) as f64;
                                metrics.observe_wait_secs(wait);
                                metrics.observe_match_time_secs_by_mode(&game_mode, wait);
                                let _ = redis.del::<_, i32>(&key).await; // cleanup to avoid double counting
                            }
                        }
                    }
                }

                let message = ServerMessage::StartLoading {
                    loading_session_id: returned_loading_session_id,
                };
                for player_id_str in &player_ids {
                    if let Ok(player_id) = Uuid::parse_str(player_id_str) {
                        send_player_message(
                            &sub_manager_addr,
                            &mut redis,
                            player_id,
                            message.clone(),
                        )
                        .await;
                    } else {
                        warn!(
                            "Could not parse player UUID from script result: {}",
                            player_id_str
                        );
                    }
                }
                // metrics: record wait duration from enqueue to match (already handled above)

                // Publish state event for loading session creation (emitter)
                {
                    let mut emitter = StateEventEmitter::new(&mut redis);
                    if let Err(e) = emitter
                        .loading_session_created(
                            returned_loading_session_id.to_string(),
                            game_mode.clone(),
                            player_ids.clone(),
                            settings.loading_session_timeout_seconds,
                        )
                        .await
                    {
                        warn!("Failed to publish loading_session_created event: {}", e);
                    }
                }

                // Create LoadingSession using new event-driven architecture
                if let Some(loading_manager) = &loading_session_manager_addr {
                    let player_uuids: Vec<Uuid> = player_ids
                        .iter()
                        .filter_map(|s| Uuid::parse_str(s).ok())
                        .collect();
                    
                    loading_manager.do_send(crate::loading_session::CreateLoadingSession {
                        session_id: returned_loading_session_id,
                        players: player_uuids,
                        game_mode: game_mode.clone(),
                        timeout_seconds: settings.loading_session_timeout_seconds,
                    });
                }

                // no metrics side-effects

                // After successful match, publish queue size change since players were removed
                if let Ok(queue_size) = redis.scard::<_, usize>(&queue_key).await {
                    let mut emitter = StateEventEmitter::new(&mut redis);
                    if let Err(e) = emitter
                        .queue_size_changed(game_mode.clone(), queue_size)
                        .await
                    {
                        warn!("Failed to publish queue_size_changed after match: {}", e);
                    }
                    // update gauge for this mode
                    metrics.set_queue_size_for(&game_mode, queue_size as i64);
                }
            }

            match lock.release(&mut redis).await {
                Ok(true) => debug!(
                    "Lock released successfully for matching in {}",
                    game_mode_settings.id
                ),
                Ok(false) => warn!(
                    "Lock for matching in {} was already released or owned by another process",
                    game_mode_settings.id
                ),
                Err(e) => error!(
                    "Failed to release lock for matching in {}: {}",
                    game_mode_settings.id, e
                ),
            }
        })
    }
}

impl Handler<HandleLoadingComplete> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: HandleLoadingComplete, _ctx: &mut Self::Context) -> Self::Result {
        let mm = self.clone();
        let addr = _ctx.address();
        return loading::handle_loading_complete(mm, msg, addr);
    }
}

impl Handler<CancelLoadingSession> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: CancelLoadingSession, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let queue_key_prefix = self.settings.queue_key_prefix.clone();
        let metrics = self.metrics.clone();

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
                    code: None,
                    message:
                        "A player disconnected during loading. You have been returned to the queue."
                            .to_string(),
                };

                for player_id_str in &players_to_requeue {
                    if let Ok(player_id) = Uuid::parse_str(player_id_str) {
                        notify_player(&mut redis, player_id, message.clone()).await;
                    }
                }
                crate::matchmaker::runtime::queue::requeue_players(
                    &mut redis,
                    &queue_key,
                    &players_to_requeue,
                    &metrics,
                )
                .await;
            }

            // Cleanup loading_time key for metrics accuracy
            let loading_time_key = format!("loading_time:{}", msg.loading_session_id);
            let _ = redis.del::<_, i32>(&loading_time_key).await;
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
        let sub_manager_addr = self.sub_manager_addr.clone();
        let metrics = self.metrics.clone();
        let blacklist_manager_addr_inner = self.blacklist_manager_addr.clone();

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
                        Ok((Some(lock), LockResult::Acquired)) => lock,
                        Ok((None, LockResult::Busy { .. })) => continue,
                        Ok((None, LockResult::Error(_))) => continue,
                        Ok((None, LockResult::Acquired)) => continue,
                        Ok((Some(_), LockResult::Busy { .. })) => continue,
                        Ok((Some(_), LockResult::Error(_))) => continue,
                        Err(_) => continue,
                    };

                let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
                    Ok(d) => d.as_secs(),
                    Err(e) => {
                        error!(
                            "System time is before UNIX EPOCH, cannot check stale sessions: {}",
                            e
                        );
                        match lock.release(&mut redis).await {
                            Ok(true) => {},
                            Ok(false) => warn!("Lock was already released"),
                            Err(e) => error!("Failed to release lock: {}", e),
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
                        match lock.release(&mut redis).await {
                            Ok(true) => {},
                            Ok(false) => warn!("Lock for key {} was already released", key),
                            Err(e) => error!(
                                "Failed to release lock for stale check on key {}: {}",
                                key, e
                            ),
                        }
                        continue;
                    }
                    Err(e) => {
                        error!(
                            "Failed to run stale session cleanup script for key {}: {}",
                            key, e
                        );
                        match lock.release(&mut redis).await {
                            Ok(true) => {},
                            Ok(false) => warn!("Lock for key {} was already released", key),
                            Err(e) => error!(
                                "Failed to release lock for stale check on key {}: {}",
                                key, e
                            ),
                        }
                        continue;
                    }
                };

                let game_mode = if !script_result.is_empty() {
                    script_result.remove(0)
                } else {
                    error!("Script result for stale check is empty, cannot proceed.");
                    match lock.release(&mut redis).await {
                        Ok(true) => {},
                        Ok(false) => warn!("Lock was already released"),
                        Err(e) => error!("Failed to release lock: {}", e),
                    }
                    continue;
                };
                // New format: [game_mode, timed_out_count, player_id...]
                // Backward compatibility: if second element is not a number, treat the rest as player ids and count all
                let timed_out_count: usize;
                let players_to_requeue: Vec<String> = if let Some(first) = script_result.first() {
                    if let Ok(c) = first.parse::<usize>() {
                        timed_out_count = c;
                        let players = script_result[1..].to_vec();
                        
                        // Check for completed session (status was 'ready')
                        if c == 0 && players.is_empty() {
                            let session_id = key.strip_prefix("loading:").unwrap_or(&key);
                            info!(
                                "Cleaned up completed loading session {} for game mode '{}' - no requeue needed",
                                session_id, game_mode
                            );
                            
                            // Publish state event for completed session cleanup
                            let mut emitter = StateEventEmitter::new(&mut redis);
                            if let Err(e) = emitter
                                .loading_session_completed(session_id.to_string(), Vec::new())
                                .await
                            {
                                warn!("Failed to publish loading_session_completed event: {}", e);
                            }
                        }
                        
                        players
                    } else {
                        timed_out_count = script_result.len();
                        script_result
                    }
                } else {
                    timed_out_count = 0;
                    Vec::new()
                };

                

                match lock.release(&mut redis).await {
                    Ok(true) => {},
                    Ok(false) => warn!("Lock for key {} was already released", key),
                    Err(e) => error!(
                        "Failed to release lock for stale check on key {}: {}",
                        key, e
                    ),
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
        let metrics = self.metrics.clone();

        Box::pin(async move {
            info!(
                "Re-queuing players {:?} for game mode {} after delay.",
                msg.player_ids, msg.game_mode
            );
            let queue_key = format!("{}:{}", queue_key_prefix, msg.game_mode);
            crate::matchmaker::runtime::queue::requeue_players(
                &mut redis,
                &queue_key,
                &msg.player_ids,
                &metrics,
            )
            .await;
        })
    }
}
