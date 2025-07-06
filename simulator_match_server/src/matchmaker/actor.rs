use crate::{
    env::MatchmakingSettings,
    provider::{DedicatedServerProvider, FindAvailableServer},
    protocol::ServerMessage,
};
use simulator_metrics::{MATCHES_CREATED_TOTAL, PLAYERS_IN_QUEUE};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, ResponseFuture};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Script};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

// --- Actor Definition ---

/// Matchmaker 액터는 주기적으로 매칭을 시도하고, 성공 시 세션 생성을 요청하는 역할을 담당합니다.
pub struct Matchmaker {
    /// Redis 연결을 위한 커넥션 매니저입니다.
    redis: ConnectionManager,
    /// 외부 서비스(dedicated_server)와 통신하기 위한 HTTP 클라이언트입니다.
    http_client: reqwest::Client,
    /// 매치메이킹 관련 설정을 담고 있는 구조체입니다.
    settings: MatchmakingSettings,
    /// 사용 가능한 게임 서버를 찾아주는 Provider 액터의 주소입니다.
    provider_addr: Addr<DedicatedServerProvider>,
}

impl Matchmaker {
    /// Matchmaker 액터의 새 인스턴스를 생성합니다.
    pub fn new(
        redis: ConnectionManager, 
        settings: MatchmakingSettings,
        provider_addr: Addr<DedicatedServerProvider>
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
        ctx.run_interval(Duration::from_secs(self.settings.tick_interval_seconds), |act, ctx| {
            for mode in act.settings.game_modes.iter() {
                ctx.address().do_send(TryMatch { game_mode: mode.clone() });
            }
        });
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
struct TryMatch {
    game_mode: String,
}

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
        error!("CRITICAL: Failed to re-queue players {:?} into {}: {}", player_ids, queue_key, e);
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
                    publish_message(&mut redis, msg.player_id, ServerMessage::Error{message: "Already in queue".to_string()}).await;
                }
                Err(e) => {
                    error!("Failed to add player to queue: {}", e);
                    publish_message(&mut redis, msg.player_id, ServerMessage::Error{message: "Internal server error".to_string()}).await;
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
                    info!("Player {} (disconnected) removed from queue {}", player_id_str, queue_key);
                    PLAYERS_IN_QUEUE.dec();
                }
                Ok(_) => {
                    tracing::debug!("Player {} was not in queue {}, likely already matched.", player_id_str, queue_key);
                }
                Err(e) => {
                    error!("Failed to remove player {} from queue {}: {}", player_id_str, queue_key, e);
                }
            }
        })
    }
}

impl Handler<TryMatch> for Matchmaker {
    type Result = ResponseFuture<()>;
    fn handle(&mut self, msg: TryMatch, _ctx: &mut Self::Context) -> Self::Result {
        let mut redis = self.redis.clone();
        let http_client = self.http_client.clone();
        let game_mode = msg.game_mode;
        let required_players = 2;
        let queue_key_prefix = self.settings.queue_key_prefix.clone();
        let provider_addr = self.provider_addr.clone();

        Box::pin(async move {
            let queue_key = format!("{}:{}", queue_key_prefix, game_mode);
            let script = Script::new(ATOMIC_MATCH_SCRIPT);
            let player_ids: Vec<String> = match script.key(&queue_key).arg(required_players).invoke_async(&mut redis).await {
                Ok(p) => p,
                Err(e) => {
                    error!("Matchmaking script failed: {}", e);
                    return;
                }
            };

            if player_ids.len() == required_players {
                PLAYERS_IN_QUEUE.sub(required_players as i64);
                info!("Found a match with players: {:?}", player_ids);

                match provider_addr.send(FindAvailableServer).await {
                    Ok(Ok(server_info)) => {
                        let create_session_url = format!("http://{}/session/create", server_info.address);
                        #[derive(Serialize)]
                        struct CreateSessionReq { players: Vec<Uuid> }
                        let req_body = CreateSessionReq { 
                            players: player_ids.iter().map(|id| Uuid::parse_str(id).unwrap()).collect()
                        };

                        match http_client.post(&create_session_url).json(&req_body).send().await {
                            Ok(resp) if resp.status().is_success() => {
                                #[derive(Deserialize, Debug)]
                                struct CreateSessionResp { server_address: String, session_id: Uuid }
                                
                                match resp.json::<CreateSessionResp>().await {
                                    Ok(session_info) => {
                                        info!("Successfully created session: {:?}", session_info);
                                        MATCHES_CREATED_TOTAL.inc();
                                        let message = ServerMessage::MatchFound {
                                            session_id: session_info.session_id,
                                            server_address: session_info.server_address.clone(),
                                        };
                                        for player_id_str in &player_ids {
                                            let player_id = Uuid::parse_str(player_id_str).unwrap();
                                            publish_message(&mut redis, player_id, message.clone()).await;
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to parse session creation response: {}. Re-queuing players.", e);
                                        requeue_players(&mut redis, &queue_key, &player_ids).await;
                                    }
                                }
                            }
                            Ok(resp) => {
                                error!("Dedicated server returned error: {}. Re-queuing players.", resp.status());
                                requeue_players(&mut redis, &queue_key, &player_ids).await;
                            }
                            Err(e) => {
                                error!("Failed to contact dedicated server: {}. Re-queuing players.", e);
                                requeue_players(&mut redis, &queue_key, &player_ids).await;
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Failed to find available server: {}. Re-queuing players.", e);
                        requeue_players(&mut redis, &queue_key, &player_ids).await;
                    }
                    Err(e) => {
                        error!("Mailbox error when contacting provider: {}. Re-queuing players.", e);
                        requeue_players(&mut redis, &queue_key, &player_ids).await;
                    }
                }
            }
        })
    }
}