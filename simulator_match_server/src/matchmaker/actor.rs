use crate::{
    env::GameModeSettings,
    protocol::ServerMessage,
    provider::{DedicatedServerProvider, FindAvailableServer},
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, ResponseFuture};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, FromRedisValue, RedisResult, Script, ToRedisArgs, Value};
use serde::{Deserialize, Serialize};
use simulator_metrics::{MATCHES_CREATED_TOTAL, PLAYERS_IN_QUEUE};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use uuid::Uuid;

const LOADING_SESSION_TIMEOUT_SECONDS: u64 = 60;

// --- Actor Definition ---
pub struct Matchmaker {
    redis: ConnectionManager,
    http_client: reqwest::Client,
    settings: crate::env::MatchmakingSettings,
    provider_addr: Addr<DedicatedServerProvider>,
}

impl Matchmaker {
    pub fn new(
        redis: ConnectionManager,
        settings: crate::env::MatchmakingSettings,
        provider_addr: Addr<DedicatedServerProvider>,
    ) -> Self {
        Self {
            redis,
            http_client: reqwest::Client::new(),
            settings,
            provider_addr,
        }
    }
}

impl Actor for Matchmaker {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Matchmaker actor started.");
        // 매칭 시도 타이머
        ctx.run_interval(
            Duration::from_secs(self.settings.tick_interval_seconds),
            |act, ctx| {
                for mode_settings in act.settings.game_modes.clone() {
                    ctx.address().do_send(TryMatch {
                        game_mode: mode_settings,
                    });
                }
            },
        );
        // 오래된 로딩 세션 정리 타이머
        ctx.run_interval(
            Duration::from_secs(LOADING_SESSION_TIMEOUT_SECONDS),
            |_act, ctx| {
                ctx.address().do_send(CheckStaleLoadingSessions);
            },
        );
    }
}

// --- Message Definitions ---
#[derive(Message)]
#[rtype(result = "()")]
pub struct EnqueuePlayer {
    pub player_id: Uuid,
    pub game_mode: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DequeuePlayer {
    pub player_id: Uuid,
    pub game_mode: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct HandleLoadingComplete {
    pub player_id: Uuid,
    pub loading_session_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CancelLoadingSession {
    pub player_id: Uuid,
    pub loading_session_id: Uuid,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
struct TryMatch {
    game_mode: GameModeSettings,
}

/// 오래된 로딩 세션을 정리하기 위한 내부 메시지입니다.
#[derive(Message)]
#[rtype(result = "()")]
struct CheckStaleLoadingSessions;

// --- Lua Script ---
const ATOMIC_MATCH_SCRIPT: &str = r#"
    local queue_key = KEYS[1]
    local required_players = tonumber(ARGV[1])
    if redis.call('SCARD', queue_key) >= required_players then
        return redis.call('SPOP', queue_key, required_players)
    else
        return {}
    end
"#;

// --- Helper Functions ---
async fn publish_message(redis: &mut ConnectionManager, player_id: Uuid, message: ServerMessage) {
    let channel = format!("notifications:{}", player_id);
    let payload = serde_json::to_string(&message).unwrap();
    if let Err(e) = redis.publish::<_, _, ()>(&channel, &payload).await {
        warn!("Failed to publish message to channel {}: {}", channel, e);
    }
}

async fn requeue_players(redis: &mut ConnectionManager, queue_key: &str, player_ids: &[String]) {
    warn!("Re-queuing players due to an error: {:?}", player_ids);
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
impl Handler<EnqueuePlayer> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: EnqueuePlayer, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let queue_key_prefix = self.settings.queue_key_prefix.clone();
        Box::pin(async move {
            let player_id_str = msg.player_id.to_string();
            let queue_key = format!("{}:{}", queue_key_prefix, msg.game_mode);
            let result: Result<i32, _> = redis.sadd(&queue_key, &player_id_str).await;
            match result {
                Ok(count) if count > 0 => {
                    info!("Player {} added to queue {}", player_id_str, queue_key);
                    PLAYERS_IN_QUEUE.inc();
                    publish_message(&mut redis, msg.player_id, ServerMessage::Queued).await;
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
        let game_mode_settings = msg.game_mode;
        let queue_key_prefix = self.settings.queue_key_prefix.clone();

        Box::pin(async move {
            let queue_key = format!("{}:{}", queue_key_prefix, game_mode_settings.id);
            let required_players = game_mode_settings.required_players;

            if game_mode_settings.use_mmr_matching {
                // TODO: MMR 기반 매칭 로직 구현
                // 현재는 단순 매칭 로직을 사용하지만, 향후 이 분기 내에
                // MMR 값에 따라 플레이어를 정렬하고, 비슷한 MMR을 가진
                // 플레이어들을 매칭시키는 로직을 추가할 수 있습니다.
                // 예: ZRANGEBYSCORE, ZPOPMAX 등의 Redis 명령어를 활용
                warn!(
                    "MMR-based matching for '{}' is not yet implemented. Falling back to simple matching.",
                    game_mode_settings.id
                );
            }
            
            // 공통 매칭 로직 (현재는 MMR 사용 여부와 관계없이 동일)
            let script = Script::new(ATOMIC_MATCH_SCRIPT);
            let player_ids: Vec<String> = match script
                .key(&queue_key)
                .arg(required_players)
                .invoke_async(&mut redis)
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    error!("Matchmaking script failed for queue {}: {}", queue_key, e);
                    return;
                }
            };

            if player_ids.len() as u32 == required_players {
                PLAYERS_IN_QUEUE.sub(required_players as i64);
                info!(
                    "[{}] Found a potential match with players: {:?}",
                    game_mode_settings.id, player_ids
                );

                let loading_session_id = Uuid::new_v4();
                let loading_key = format!("loading:{}", loading_session_id);

                let mut players_map: HashMap<String, String> = HashMap::new();
                for player_id in &player_ids {
                    players_map.insert(player_id.clone(), "loading".to_string());
                }
                players_map.insert("game_mode".to_string(), game_mode_settings.id.clone());
                // 타임아웃 처리를 위해 생성 시간을 기록합니다.
                let current_timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string();
                players_map.insert("created_at".to_string(), current_timestamp);

                let players_map_slice: Vec<_> = players_map.iter().collect();
                if let Err(e) = redis
                    .hset_multiple::<_, _, _, ()>(&loading_key, &players_map_slice)
                    .await
                {
                    error!(
                        "Failed to create loading session in Redis: {}. Re-queuing players.",
                        e
                    );
                    requeue_players(&mut redis, &queue_key, &player_ids).await;
                    return;
                }

                info!(
                    "[{}] Notifying players to start loading for session {}",
                    game_mode_settings.id, loading_session_id
                );
                let message = ServerMessage::StartLoading { loading_session_id };
                for player_id_str in &player_ids {
                    let player_id = Uuid::parse_str(player_id_str).unwrap();
                    publish_message(&mut redis, player_id, message.clone()).await;
                }
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

            // Execute the atomic Lua script
            let script = Script::new(ATOMIC_LOADING_COMPLETE_SCRIPT);
            let result: Result<Vec<String>, _> = script
                .key(&loading_key)
                .arg(&player_id_str)
                .invoke_async(&mut redis)
                .await;

            let player_ids: Vec<String> = match result {
                Ok(ids) if !ids.is_empty() => ids,
                Ok(_) => {
                    // Script returned an empty list, meaning not all players are ready yet,
                    // or the session was already handled.
                    info!(
                        "Player {} is ready, but waiting for others in session {}.",
                        player_id_str, msg.loading_session_id
                    );
                    return;
                }
                Err(e) => {
                    error!(
                        "Atomic loading script failed for session {}: {}",
                        msg.loading_session_id, e
                    );
                    return;
                }
            };

            // If we get here, it means all players are ready and we are the one designated to create the session.
            info!(
                "All players {:?} are ready for session {}. Finding a dedicated server...",
                player_ids, msg.loading_session_id
            );

            // We need the game_mode to re-queue players on failure.
            // Since the loading key is now deleted, we can't fetch it from Redis anymore.
            // This is a simplification for now. A more robust solution might pass the game_mode
            // through the loading complete message or have the script return it.
            // For now, we'll assume the most common mode on failure.
            // A better approach would be to get it from one of the player's session actors.
            // But for this fix, we focus on the race condition.
            let game_mode = "Normal_1v1".to_string(); // Simplification

            match provider_addr.send(FindAvailableServer).await {
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
                            .map(|id| Uuid::parse_str(id).unwrap())
                            .collect(),
                    };

                    match http_client.post(&create_session_url).json(&req_body).send().await {
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
                                        let player_id = Uuid::parse_str(player_id_str).unwrap();
                                        publish_message(&mut redis, player_id, message.clone())
                                            .await;
                                    }
                                    // The loading key is already deleted by the Lua script.
                                }
                                Err(e) => {
                                    let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                                    error!("[{}] Failed to parse session creation response: {}. Re-queuing players.", game_mode, e);
                                    requeue_players(&mut redis, &queue_key, &player_ids).await;
                                }
                            }
                        }
                        Ok(resp) => {
                            let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                            error!(
                                "[{}] Dedicated server returned error: {}. Re-queuing players.",
                                game_mode,
                                resp.status()
                            );
                            requeue_players(&mut redis, &queue_key, &player_ids).await;
                        }
                        Err(e) => {
                            let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                            error!("[{}] Failed to contact dedicated server: {}. Re-queuing players.", game_mode, e);
                            requeue_players(&mut redis, &queue_key, &player_ids).await;
                        }
                    }
                }
                Ok(Err(e)) => {
                    let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                    error!(
                        "[{}] Failed to find available server: {}. Re-queuing players.",
                        game_mode, e
                    );
                    requeue_players(&mut redis, &queue_key, &player_ids).await;
                }
                Err(e) => {
                    let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
                    error!(
                        "[{}] Mailbox error when contacting provider: {}. Re-queuing players.",
                        game_mode, e
                    );
                    requeue_players(&mut redis, &queue_key, &player_ids).await;
                }
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
            info!(
                "Cancelling loading session {} due to player {} disconnection.",
                msg.loading_session_id, msg.player_id
            );

            let all_players_status: HashMap<String, String> = match redis
                .hgetall::<_, HashMap<String, String>>(&loading_key)
                .await
            {
                Ok(statuses) if !statuses.is_empty() => statuses,
                _ => {
                    warn!(
                        "Loading session {} not found or already cleaned up.",
                        msg.loading_session_id
                    );
                    return;
                }
            };

            let _: RedisResult<()> = redis.del(&loading_key).await;

            let game_mode = all_players_status
                .get("game_mode")
                .cloned()
                .unwrap_or_default();
            let queue_key = format!("{}:{}", queue_key_prefix, game_mode);

            let disconnected_player_id_str = msg.player_id.to_string();
            let players_to_requeue: Vec<String> = all_players_status
                .keys()
                .filter(|k| {
                    let k_str = k.as_str();
                    k_str != "game_mode"
                        && k_str != "created_at"
                        && k_str != disconnected_player_id_str
                })
                .cloned()
                .collect();

            if !players_to_requeue.is_empty() {
                info!("Notifying remaining players and re-queuing them.");
                for player_id_str in &players_to_requeue {
                    let player_id = Uuid::parse_str(player_id_str).unwrap();
                    let message = ServerMessage::Error { message: "A player disconnected during loading. You have been returned to the queue.".to_string() };
                    publish_message(&mut redis, player_id, message).await;
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
        let queue_key_prefix = self.settings.queue_key_prefix.clone();

        Box::pin(async move {
            info!("Checking for stale loading sessions...");
            let Ok(keys) = redis.keys::<_, Vec<String>>("loading:*").await else {
                return;
            };

            for key in keys {
                let Ok(all_players_status): RedisResult<HashMap<String, String>> =
                    redis.hgetall(&key).await
                else {
                    continue;
                };

                let Some(created_at_str) = all_players_status.get("created_at") else {
                    continue;
                };
                let Ok(created_at) = created_at_str.parse::<u64>() else {
                    continue;
                };

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if now > created_at + LOADING_SESSION_TIMEOUT_SECONDS {
                    warn!("Found stale loading session {}. Cleaning up.", key);

                    let _: RedisResult<()> = redis.del(&key).await;

                    let game_mode = all_players_status
                        .get("game_mode")
                        .cloned()
                        .unwrap_or_default();
                    let queue_key = format!("{}:{}", queue_key_prefix, game_mode);

                    let players_to_requeue: Vec<String> = all_players_status
                        .keys()
                        .filter(|k| *k != "game_mode" && *k != "created_at")
                        .cloned()
                        .collect();

                    if !players_to_requeue.is_empty() {
                        for player_id_str in &players_to_requeue {
                            let player_id = Uuid::parse_str(player_id_str).unwrap();
                            let message = ServerMessage::Error {
                                message:
                                    "Matchmaking timed out. You have been returned to the queue."
                                        .to_string(),
                            };
                            publish_message(&mut redis, player_id, message).await;
                        }
                        requeue_players(&mut redis, &queue_key, &players_to_requeue).await;
                    }
                }
            }
        })
    }
}
