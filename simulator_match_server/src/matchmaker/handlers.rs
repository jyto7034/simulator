use crate::{protocol::ServerMessage, provider::FindAvailableServer};
use actix::{Handler, ResponseFuture};
use futures_util::stream::StreamExt;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult, Script};
use serde::{Deserialize, Serialize};
use simulator_metrics::{MATCHES_CREATED_TOTAL, PLAYERS_IN_QUEUE};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use uuid::Uuid;

/*
  1. 아키텍처 및 액터(Actor)의 역할
   * Matchmaker 액터의 가장 핵심적인 책임은 무엇이며, DedicatedServerProvider 액터와는 어떤
     관계를 맺고 있나요? 두 액터가 서로 통신하는 주된 이유는 무엇입니까?
answer:
    Matchmaker 액터는 started() 에서 run_interval 를 통해 주기적으로 TryMatch 메시지와 CheckStaleLoadingSessions 메시지를 처리합니다.
    이는 매칭 요청을 처리하고 오래된 로딩 세션을 정리하는 책임을 가집니다.
    매칭 성공 시, dedicated server를 찾아서 게임을 생성하는데, 이 때 dedicated server를 찾기 위해 DedicatedServerProvider 액터와 통신합니다.

  2. 매치메이킹 핵심 로직
   * 플레이어가 매칭을 요청(EnqueuePlayer)했을 때부터, 매치가 성사되어 로딩을 시작하라는
     메시지(StartLoading)를 받기까지의 과정을 단계별로 설명해주세요. 이 과정에서 어떤 액터와
     Redis 키(key)가 관련되나요?
answer:
    1. 먼저 게임 모드 유효성 검사를 실시합니다. 만약 유효하지 않다면 publish_message 함수를 통해 구독자에게 ServerMessage::Error 메시지를 발행합니다.
    2. redis.sadd(&queue_key, &player_id_str).await; 함수를 통해 redis 에 원자적으로 k:queue, v:player_id_str 데이터 쌍을 추가합니다.
    2-1. 만약 성공 한다면 ( Ok(count) )
        - 플레이어가 큐에 추가되었다는 로그를 남기고, PLAYERS_IN_QUEUE 메트릭을 증가시킵니다.
        - publish_message 함수를 통해 구독자에게 ServerMessage::Queued 메시지를 발행합니다.
    2-2. 만약 실패한다면 ( Ok(_) )
        - 플레이어가 이미 큐에 존재한다는 경고 로그를 남기고, publish_message 함수를 통해 구독자에게 ServerMessage::Error 메시지를 발행합니다.
    2-3. 만약 redis.sadd 함수가 실패한다면 ( Err(e) )
        - 에러 로그를 남기고, publish_message 함수를 통해 구독자에게 ServerMessage::Error 메시지를 발행합니다.
    3. MatchMaker Actor 에서 주기적으로 수행되는 TryMatch 메시지의 Handler 가 수행됩니다.
    4. lock 을 획득 후 ATOMIC_MATCH_SCRIPT 를 실행하여 redis 에서 매칭 대기중인 player_id 를 가져옵니다.
    5.

  3. 상태 관리와 Redis
   * 이 시스템에서 Redis는 여러 가지 중요한 상태를 관리합니다. '플레이어 대기열', '로딩 중인
     세션', '전용 서버 목록' 이 세 가지를 관리하기 위해 각각 어떤 Redis 자료구조(Data
     Structure)와 키(Key) 패턴이 사용되고 있나요?

  4. 분산 환경 및 동시성 제어
   * match_server가 여러 인스턴스로 실행될 수 있는 분산 환경을 가정해 보겠습니다. 두 개의 다른
     서버 인스턴스가 정확히 같은 시간에 동일한 게임 모드의 매칭을 시도할 때, 플레이어들이
     중복으로 매칭되는 문제를 방지하기 위해 어떤 장치가 마련되어 있나요?

  5. 오류 처리 및 복원력
   * 플레이어 그룹이 성공적으로 매칭되어 로딩 상태에 들어갔지만, 한 명의 플레이어가 로딩을
     완료하지 못하고 타임아웃이 발생했다고 가정해봅시다.
       * 시스템은 이 "오래된(stale)" 로딩 세션을 어떻게 감지하고 정리하나요?
       * 남아있는 다른 플레이어들은 어떻게 처리되나요?
*/

use super::{
    actor::{Matchmaker, LOADING_SESSION_TIMEOUT_SECONDS},
    lock::DistributedLock, // DistributedLock 임포트
    messages::*,
    scripts::{ATOMIC_LOADING_COMPLETE_SCRIPT, ATOMIC_MATCH_SCRIPT},
};

const LOCK_DURATION_MS: usize = 10_000; // 10초

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
        let queue_key_prefix = self.settings.queue_key_prefix.clone();

        // 게임 모드 유효성 검사
        let is_valid_game_mode = self
            .settings
            .game_modes
            .iter()
            .any(|m| m.id == msg.game_mode);
        if !is_valid_game_mode {
            let player_id = msg.player_id;
            return Box::pin(async move {
                warn!(
                    "Player {} tried to enqueue for invalid game mode: {}",
                    player_id, msg.game_mode
                );
                publish_message(
                    &mut redis,
                    player_id,
                    ServerMessage::Error {
                        message: format!("Invalid game mode: {}", msg.game_mode),
                    },
                )
                .await;
            });
        }

        Box::pin(async move {
            let player_id_str = msg.player_id.to_string();
            let queue_key = format!("{}:{}", queue_key_prefix, msg.game_mode);

            // Redis SADD는 원자적이므로 락 불필요
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
        let game_mode_settings = msg.game_mode;
        let queue_key_prefix = self.settings.queue_key_prefix.clone();

        Box::pin(async move {
            let queue_key = format!("{}:{}", queue_key_prefix, game_mode_settings.id);
            // 게임에 필요한 플레이어 수 입니다.
            let required_players = game_mode_settings.required_players;
            let lock_key = format!("lock:match:{}", game_mode_settings.id);

            if game_mode_settings.use_mmr_matching {
                warn!(
                    "MMR-based matching for '{}' is not yet implemented. Falling back to simple matching.",
                    game_mode_settings.id
                );
            }

            // --- 분산락 획득 ---
            let lock = match DistributedLock::acquire(&mut redis, &lock_key, LOCK_DURATION_MS).await
            {
                Ok(Some(lock)) => lock,
                Ok(None) => {
                    // 다른 서버가 이미 매칭 처리 중이므로 건너뛰기
                    return;
                }
                Err(e) => {
                    error!(
                        "Failed to acquire lock for matching in {}: {}",
                        game_mode_settings.id, e
                    );
                    return;
                }
            };

            // 필요한 플레이어 수 만큼 redis 에서 player id 를 가져옵니다.
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
                    // 락 해제
                    if let Err(lock_err) = lock.release(&mut redis).await {
                        error!(
                            "Failed to release lock for matching in {}: {}",
                            game_mode_settings.id, lock_err
                        );
                    }
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

                let mut players_map = HashMap::new();
                for player_id in &player_ids {
                    players_map.insert(player_id.clone(), "loading".to_string());
                }
                players_map.insert("game_mode".to_string(), game_mode_settings.id.clone());
                let current_timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string();
                players_map.insert("created_at".to_string(), current_timestamp);
                players_map.insert("status".to_string(), "loading".to_string());

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
                    // 락 해제
                    if let Err(lock_err) = lock.release(&mut redis).await {
                        error!(
                            "Failed to release lock for matching in {}: {}",
                            game_mode_settings.id, lock_err
                        );
                    }
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

            // --- 분산락 해제 ---
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

            // --- 분산락 획득 ---
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

            let script = Script::new(ATOMIC_LOADING_COMPLETE_SCRIPT);
            let result: Result<Vec<String>, _> = script
                .key(&loading_key)
                .arg(&player_id_str)
                .invoke_async(&mut redis)
                .await;

            // --- 분산락 해제 ---
            if let Err(e) = lock.release(&mut redis).await {
                error!(
                    "Failed to release lock for session {}: {}",
                    msg.loading_session_id, e
                );
            }

            let mut script_result: Vec<String> = match result {
                Ok(ids) if !ids.is_empty() => ids,
                Ok(_) => {
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

            let game_mode = script_result.remove(0);
            let player_ids = script_result;

            info!(
                "All players {:?} are ready for session {}. Finding a dedicated server...",
                player_ids, msg.loading_session_id
            );

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

                    match http_client
                        .post(&create_session_url)
                        .json(&req_body)
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
                                        let player_id = Uuid::parse_str(player_id_str).unwrap();
                                        publish_message(&mut redis, player_id, message.clone())
                                            .await;
                                    }
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
                            error!(
                                "[{}] Failed to contact dedicated server: {}. Re-queuing players.",
                                game_mode, e
                            );
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
            let lock_key = format!("lock:{}", loading_key);

            info!(
                "Attempting to cancel loading session {} due to player {} disconnection.",
                msg.loading_session_id, msg.player_id
            );

            // --- 1. 분산락 획득 ---
            let lock = match DistributedLock::acquire(&mut redis, &lock_key, LOCK_DURATION_MS).await
            {
                Ok(Some(lock)) => lock,
                Ok(None) => {
                    info!("Could not acquire lock for cancellation on session {}, another process is handling it.", msg.loading_session_id);
                    return;
                }
                Err(e) => {
                    error!(
                        "Failed to acquire lock for cancellation on session {}: {}",
                        msg.loading_session_id, e
                    );
                    return;
                }
            };

            // --- 2. 락 내부에서 세션 데이터 확인 ---
            // 이 시점에 HandleLoadingComplete가 먼저 성공했다면, 키는 이미 존재하지 않을 것입니다.
            let all_players_status: HashMap<String, String> = match redis
                .hgetall::<_, HashMap<String, String>>(&loading_key)
                .await
            {
                Ok(statuses) if !statuses.is_empty() => statuses,
                _ => {
                    // 세션이 존재하지 않으면(이미 처리되었으면) 아무것도 할 필요가 없습니다.
                    warn!(
                        "Loading session {} already handled or cleaned up before cancellation.",
                        msg.loading_session_id
                    );
                    // 락을 해제하고 종료합니다.
                    if let Err(e) = lock.release(&mut redis).await {
                        error!(
                            "Failed to release lock for session {}: {}",
                            msg.loading_session_id, e
                        );
                    }
                    return;
                }
            };

            // --- 3. 세션 키 삭제 (취소 로직의 핵심) ---
            // 여기서 키를 삭제함으로써, 뒤늦게 도착할 수 있는 HandleLoadingComplete가 아무 작업도 못하게 만듭니다.
            info!(
                "Cancelling and deleting loading session {}.",
                msg.loading_session_id
            );
            if let Err(e) = redis.del::<_, ()>(&loading_key).await {
                // 키 삭제 실패는 크리티컬한 상황일 수 있으나, 로직은 계속 진행하여 플레이어 재입장을 시도합니다.
                error!(
                    "Failed to delete loading session key {}: {}",
                    loading_key, e
                );
            }

            // --- 4. 락 해제 ---
            // 재입장 로직은 다른 플레이어에게 알림을 보내는 등 시간이 걸릴 수 있으므로,
            // 임계 영역에 해당하는 키 삭제 후에는 가능한 한 빨리 락을 해제합니다.
            if let Err(e) = lock.release(&mut redis).await {
                error!(
                    "Failed to release lock for session {}: {}",
                    msg.loading_session_id, e
                );
            }

            // --- 5. 나머지 플레이어 처리 ---
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
                        && k_str != "status"
                        && k_str != disconnected_player_id_str
                })
                .cloned()
                .collect();

            if !players_to_requeue.is_empty() {
                info!("Notifying remaining players and re-queuing them.");
                let message = ServerMessage::Error {
                    message:
                        "A player disconnected during loading. You have been returned to the queue."
                            .to_string(),
                };
                for player_id_str in &players_to_requeue {
                    let player_id = Uuid::parse_str(player_id_str).unwrap();
                    publish_message(&mut redis, player_id, message.clone()).await;
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

            // SCAN 사용으로 Redis 블로킹 방지
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

                // --- 각 세션에 대한 분산락 획득 시도 ---
                let lock =
                    match DistributedLock::acquire(&mut redis, &lock_key, LOCK_DURATION_MS).await {
                        Ok(Some(lock)) => lock,
                        _ => continue, // 락 획득 실패 시 (다른 프로세스가 처리 중이거나 에러), 다음 키로 넘어감
                    };

                let Ok(all_players_status): RedisResult<HashMap<String, String>> =
                    redis.hgetall(&key).await
                else {
                    // 락 해제
                    let _ = lock.release(&mut redis).await;
                    continue;
                };

                let Some(created_at_str) = all_players_status.get("created_at") else {
                    let _ = lock.release(&mut redis).await;
                    continue;
                };
                let Ok(created_at) = created_at_str.parse::<u64>() else {
                    let _ = lock.release(&mut redis).await;
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
                        .filter(|k| *k != "game_mode" && *k != "created_at" && *k != "status")
                        .cloned()
                        .collect();

                    if !players_to_requeue.is_empty() {
                        let message = ServerMessage::Error {
                            message: "Matchmaking timed out. You have been returned to the queue."
                                .to_string(),
                        };
                        for player_id_str in &players_to_requeue {
                            let player_id = Uuid::parse_str(player_id_str).unwrap();
                            publish_message(&mut redis, player_id, message.clone()).await;
                        }
                        requeue_players(&mut redis, &queue_key, &players_to_requeue).await;
                    }
                }
                // --- 작업 완료 후 락 해제 ---
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
