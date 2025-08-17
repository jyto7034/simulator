use crate::{protocol::ServerMessage, state_events::StateEventEmitter};
use actix::{AsyncContext, Handler, ResponseFuture};
use futures_util::stream::StreamExt;
// metrics removed
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Script};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{
    actor::Matchmaker,
    lock::DistributedLock,
    messages::*,
    scripts::{
        get_atomic_cancel_session_script, get_atomic_match_script, get_cleanup_stale_session_script,
    },
};

mod dequeue;
mod enqueue;
mod loading;

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
                .arg(&current_timestamp)
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
                        publish_message(&mut redis, player_id, message.clone()).await;
                    }
                }
                crate::matchmaker::runtime::queue::requeue_players(
                    &mut redis,
                    &queue_key,
                    &players_to_requeue,
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
                        code: None,
                        message:
                            "Matchmaking timed out. You will be returned to the queue shortly."
                                .to_string(),
                    };
                    // Extract session_id from key (e.g., "loading:session_id" -> "session_id")
                    let session_id = key.strip_prefix("loading:").unwrap_or(&key).to_string();

                    // Publish state event for loading session timeout
                    {
                        let mut emitter = StateEventEmitter::new(&mut redis);
                        if let Err(e) = emitter
                            .loading_session_timeout(session_id.clone(), players_to_requeue.clone())
                            .await
                        {
                            warn!("Failed to publish loading_session_timeout event: {}", e);
                        }
                    }

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
            crate::matchmaker::runtime::queue::requeue_players(
                &mut redis,
                &queue_key,
                &msg.player_ids,
            )
            .await;
        })
    }
}
