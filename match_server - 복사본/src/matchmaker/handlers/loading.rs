use actix::{prelude::Addr, ResponseFuture};
use redis::AsyncCommands;
use redis::Script;
use tracing::{error, info, warn};

use crate::matchmaker::runtime::{
    alloc_token::{self, TokenOutcome},
    allocator, cleanup,
    notify::notify_players,
    retry::{self, RetryDecision},
};
use crate::matchmaker::{
    actor::Matchmaker, lock::{DistributedLock, LockResult}, messages::HandleLoadingComplete,
    scripts::get_atomic_loading_complete_script,
};
use crate::protocol::ServerMessage;
use crate::state_events::StateEventEmitter;
// metrics removed

const LOCK_DURATION_MS: usize = 30_000;

pub(super) fn handle_loading_complete(
    mm: Matchmaker,
    msg: HandleLoadingComplete,
    addr: Addr<Matchmaker>,
) -> ResponseFuture<()> {
    let mut redis = mm.redis.clone();
    let http_client = mm.http_client.clone();
    let provider_addr = mm.provider_addr.clone();
    let queue_key_prefix = mm.settings.queue_key_prefix.clone();
    let loading_session_ttl = mm.settings.loading_session_timeout_seconds;
    let alloc_token_ttl = mm.settings.allocation_token_ttl_seconds;
    let request_timeout = mm.settings.dedicated_request_timeout_seconds;
    let max_retries = mm.settings.max_dedicated_server_retries.unwrap_or(3);
    let metrics = mm.metrics.clone();

    Box::pin(async move {
        let loading_key = format!("loading:{}", msg.loading_session_id);
        let player_id_str = msg.player_id.to_string();
        let lock_key = format!("lock:{}", loading_key);

        let lock = match DistributedLock::acquire(&mut redis, &lock_key, LOCK_DURATION_MS).await {
            Ok((Some(lock), LockResult::Acquired)) => lock,
            Ok((None, LockResult::Busy { remaining_ttl_ms })) => {
                info!(
                    "Could not acquire lock for session {}, another process is handling it (TTL: {}ms). Scheduling short retry...",
                    msg.loading_session_id, remaining_ttl_ms
                );
                // 짧은 지연 후 재시도 (경합 해소용). request_timeout과 무관하게 아주 짧게.
                let addr_clone = addr.clone();
                let player = msg.player_id;
                let sid = msg.loading_session_id;
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    addr_clone.do_send(HandleLoadingComplete {
                        player_id: player,
                        loading_session_id: sid,
                    });
                });
                return;
            }
            Ok((None, LockResult::Error(msg_err))) => {
                error!(
                    "Lock acquisition error for session {}: {}",
                    msg.loading_session_id, msg_err
                );
                return;
            }
            Ok((Some(_), _)) => {
                error!(
                    "Unexpected lock state for session {}",
                    msg.loading_session_id
                );
                return;
            }
            Ok((None, LockResult::Acquired)) => {
                error!(
                    "Impossible lock state for session {}",
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

        // Handle loading completion result with readiness-aware fallback
        let mut just_completed = false;
        let mut script_result: Vec<String> = match result {
            Ok(ids) if !ids.is_empty() => {
                // This call transitioned the session to 'ready' and returned the roster
                just_completed = true;
                ids
            }
            Ok(_) => {
                // No immediate roster returned. Check if the session is already in 'ready' state
                let status: Option<String> = match redis
                    .hget::<_, _, Option<String>>(&loading_key, "status")
                    .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Failed to read status for {}: {}", loading_key, e);
                        None
                    }
                };

                if matches!(status.as_deref(), Some("ready")) {
                    // Reconstruct game_mode and players from the hash and proceed to allocation
                    let data: std::collections::HashMap<String, String> =
                        redis.hgetall(&loading_key).await.unwrap_or_default();
                    let game_mode = data
                        .get("game_mode")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    let mut player_ids: Vec<String> = data
                        .into_iter()
                        .filter_map(|(k, _v)| {
                            if k != "game_mode" && k != "created_at" && k != "status" {
                                Some(k)
                            } else {
                                None
                            }
                        })
                        .collect();
                    player_ids.sort();

                    let mut v = Vec::with_capacity(1 + player_ids.len());
                    v.push(game_mode);
                    v.extend(player_ids);
                    v
                } else {
                    info!(
                        "Player {} is ready, but waiting for others in session {}.",
                        player_id_str, msg.loading_session_id
                    );
                    {
                        let mut emitter = StateEventEmitter::new(&mut redis);
                        if let Err(e) = emitter
                            .player_ready(msg.loading_session_id.to_string(), player_id_str.clone())
                            .await
                        {
                            warn!("Failed to publish player_ready event: {}", e);
                        }
                    }
                    if let Err(e) = lock.release(&mut redis).await {
                        error!("Failed to release lock: {}", e);
                    }
                    return;
                }
            }
            Err(e) => {
                error!("Loading complete script failed for {}: {}", loading_key, e);
                if let Err(e) = lock.release(&mut redis).await {
                    error!("Failed to release lock: {}", e);
                }
                return;
            }
        };

        // Script returns [game_mode, player_id1, player_id2, ...]
        let game_mode = if !script_result.is_empty() {
            script_result.remove(0)
        } else {
            "unknown".to_string()
        };
        let returned_loading_session_id = msg.loading_session_id;
        let player_ids: Vec<String> = script_result;

        if just_completed {
            info!(
                "All players {:?} are ready for session {}. Proceeding to allocation...",
                player_ids, returned_loading_session_id
            );
        } else {
            info!(
                "Session {} already marked ready with players {:?}. Proceeding to allocation (retry path).",
                returned_loading_session_id, player_ids
            );
        }

        // Loading duration metrics: from created_at to now
        if let Ok(Some(created_at_str)) =
            redis.hget::<_, _, Option<String>>(&loading_key, "created_at").await
        {
            if let Ok(created) = created_at_str.parse::<u64>() {
                if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                {
                    let secs = now.as_secs().saturating_sub(created) as f64;
                    metrics.observe_loading_duration_secs_by_mode(&game_mode, secs);
                    if !player_ids.is_empty() {
                        metrics.inc_loading_completed_by_mode(&game_mode, player_ids.len() as u64);
                    }
                }
            }
        }

        // Guard: single allocator wins per session
        match alloc_token::try_acquire(&mut redis, returned_loading_session_id, alloc_token_ttl)
            .await
        {
            Ok(TokenOutcome::Busy) => {
                // Another client handles allocation; set watchdog to retry confirm path
                alloc_token::schedule_watchdog(
                    addr.clone(),
                    msg.player_id,
                    returned_loading_session_id,
                    2,
                );
                if let Err(e) = lock.release(&mut redis).await {
                    error!("Failed to release lock: {}", e);
                }
                return;
            }
            Ok(TokenOutcome::Won) => {
                // continue
            }
            Err(e) => {
                error!("Failed to acquire allocation token: {}", e);
                if let Err(e) = lock.release(&mut redis).await {
                    error!("Failed to release lock: {}", e);
                }
                return;
            }
        }

        // Mark loading completed (state event) before allocation
        {
            let mut emitter = StateEventEmitter::new(&mut redis);
            if let Err(e) = emitter
                .loading_session_completed(
                    returned_loading_session_id.to_string(),
                    player_ids.clone(),
                )
                .await
            {
                warn!("Failed to publish loading_session_completed event: {}", e);
            }
        }

        // no counters for completed loading

        // Find server and create dedicated session
        match allocator::find_and_create(
            &http_client,
            &provider_addr,
            request_timeout,
            &game_mode,
            &player_ids,
        )
        .await
        {
            Ok(resp) => {
                // Notify clients
                let msg = ServerMessage::MatchFound {
                    session_id: resp.session_id,
                    server_address: resp.server_address.clone(),
                };
                notify_players(&mut redis, &player_ids, msg).await;
                // metrics: count players who successfully reached MatchFound (allocation success)
                if !player_ids.is_empty() {
                    metrics.inc_allocated_success_by(player_ids.len() as u64);
                    metrics.inc_dedicated_success_by_mode(&game_mode, player_ids.len() as u64);
                }

                // no metrics counters

                // Emit state event
                {
                    let mut emitter = StateEventEmitter::new(&mut redis);
                    if let Err(e) = emitter
                        .dedicated_session_created(resp.session_id.to_string(), resp.server_address)
                        .await
                    {
                        warn!("Failed to publish dedicated_session_created: {}", e);
                    }
                }

                // Allocation 성공 후에는 더 이상 loading:<sid> 키가 필요 없습니다. 남겨두면 stale 정리에 걸려 재큐 루프를 유발할 수 있으므로 삭제합니다.
                let loading_key = format!("loading:{}", returned_loading_session_id);
                match redis.del::<_, i32>(&loading_key).await {
                    Ok(n) if n > 0 => info!(
                        "[{}] Deleted loading session key {} after successful allocation",
                        game_mode, loading_key
                    ),
                    Ok(_) => info!(
                        "[{}] Loading session key {} already absent after allocation",
                        game_mode, loading_key
                    ),
                    Err(e) => warn!(
                        "[{}] Failed to delete loading key {} after allocation: {}",
                        game_mode, loading_key, e
                    ),
                }
            }
            Err(err) => {
                // Classify/log and increment metrics
                match &err {
                    allocator::CreateError::HttpTimeout => {
                        warn!(
                            "Dedicated allocation timeout for session {}",
                            returned_loading_session_id
                        );
                        metrics.inc_http_timeout_error();
                    }
                    allocator::CreateError::HttpError(code) => {
                        warn!(
                            "Dedicated allocation HTTP error {} for session {}",
                            code, returned_loading_session_id
                        );
                    }
                    allocator::CreateError::HttpOther(msg) => {
                        warn!(
                            "Dedicated allocation HTTP error: {} for session {}",
                            msg, returned_loading_session_id
                        );
                    }
                    allocator::CreateError::Provider => {
                        warn!("No idle dedicated server available.")
                    }
                    allocator::CreateError::Mailbox => {
                        warn!("Provider mailbox error while requesting server.")
                    }
                    allocator::CreateError::ResponseParse => {
                        warn!("Dedicated allocation response parse error.")
                    }
                }

                // Decide retry vs final failure
                let decision = retry::incr_and_decide(
                    &mut redis,
                    &game_mode,
                    &player_ids,
                    max_retries,
                    loading_session_ttl as usize,
                )
                .await;

                match decision {
                    RetryDecision::RetryRemaining { attempt, max, .. } => {
                        warn!(
                            "Allocation attempt {}/{} failed for session {}. Scheduling retry...",
                            attempt, max, returned_loading_session_id
                        );
                        // Re-try after a short backoff tied to request timeout
                        let addr_clone = addr.clone();
                        let player = msg.player_id;
                        let sid = returned_loading_session_id;
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_secs(request_timeout))
                                .await;
                            addr_clone.do_send(HandleLoadingComplete {
                                player_id: player,
                                loading_session_id: sid,
                            });
                        });
                    }
                    RetryDecision::FinalExhausted { max, retry_key } => {
                        warn!(
                            "Max retries ({}) exceeded. Performing final failure cleanup for {}",
                            max, returned_loading_session_id
                        );
                        cleanup::final_failure_cleanup(
                            &mut redis,
                            &queue_key_prefix,
                            &game_mode,
                            returned_loading_session_id,
                            &player_ids,
                            max,
                            Some(&retry_key),
                        )
                        .await;
                        // Count emitted matchmaking errors per affected player
                        for _ in &player_ids {
                            metrics.inc_matchmaking_error();
                        }
                        {
                            let mut emitter = StateEventEmitter::new(&mut redis);
                            if let Err(e) = emitter
                                .dedicated_session_failed(
                                    returned_loading_session_id.to_string(),
                                    "max_retries_exceeded".to_string(),
                                )
                                .await
                            {
                                warn!("Failed to publish dedicated_session_failed: {}", e);
                            }
                        }
                    }
                }
            }
        }

        // Update visible queue size for this mode after completion path
        let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
        if let Ok(queue_size) = redis.scard::<_, usize>(&queue_key).await {
            let mut emitter = StateEventEmitter::new(&mut redis);
            if let Err(e) = emitter
                .queue_size_changed(game_mode.clone(), queue_size)
                .await
            {
                warn!(
                    "Failed to publish queue_size_changed after loading ready: {}",
                    e
                );
            }
            metrics.set_queue_size_for(&game_mode, queue_size as i64);
        }

        if let Err(e) = lock.release(&mut redis).await {
            error!("Failed to release lock: {}", e);
        }
    })
}
