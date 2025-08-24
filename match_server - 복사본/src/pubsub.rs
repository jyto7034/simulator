use crate::{env::Settings, protocol::ServerMessage, ws_session::MatchmakingSession};
use actix::{
    Actor, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, Message, WrapFuture,
};
use futures_util::stream::StreamExt;
// metrics removed
use redis::Client as RedisClient;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

// --- Messages for this module ---
#[derive(Message)]
#[rtype(result = "()")]
struct Connect;

#[derive(Message)]
#[rtype(result = "()")]
struct GracefulShutdown;

#[derive(Message)]
#[rtype(result = "()")]
struct RecordFailure;

// --- SubscriptionManager Actor ---

/// Manages the mapping between player_id and their WebSocket session actor address.
pub struct SubscriptionManager {
    sessions: HashMap<Uuid, Addr<MatchmakingSession>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }
}

impl Actor for SubscriptionManager {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Register {
    pub player_id: Uuid,
    pub addr: Addr<MatchmakingSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Deregister {
    pub player_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ForwardMessage {
    pub player_id: Uuid,
    pub message: ServerMessage,
}

#[derive(Message)]
#[rtype(result = "Result<Vec<SessionInfo>, anyhow::Error>")]
pub struct GetActiveSessionsDebug;

#[derive(Message)]
#[rtype(result = "Vec<Uuid>")]
pub struct ValidateActivePlayers {
    pub player_ids: Vec<Uuid>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SessionInfo {
    pub player_id: String,
    pub connected_at: String,
}

impl Handler<Register> for SubscriptionManager {
    type Result = ();
    fn handle(&mut self, msg: Register, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Player {} registered for notifications.", msg.player_id);
        self.sessions.insert(msg.player_id, msg.addr);
        // metrics removed
    }
}

impl Handler<Deregister> for SubscriptionManager {
    type Result = ();
    fn handle(&mut self, msg: Deregister, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Player {} deregistered.", msg.player_id);
        let _ = self.sessions.remove(&msg.player_id);
    }
}

impl Handler<ForwardMessage> for SubscriptionManager {
    type Result = ();
    fn handle(&mut self, msg: ForwardMessage, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(recipient_addr) = self.sessions.get(&msg.player_id) {
            recipient_addr.do_send(msg.message);
        } else {
            warn!(
                "Could not find session for player {} to forward message.",
                msg.player_id
            );
        }
    }
}

impl Handler<GracefulShutdown> for SubscriptionManager {
    type Result = ();
    fn handle(&mut self, _msg: GracefulShutdown, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Performing graceful shutdown - notifying {} connected players", self.sessions.len());
        
        // Notify all connected players about the service shutdown
        let shutdown_message = ServerMessage::Error {
            code: Some(crate::protocol::ErrorCode::InternalError),
            message: "Service is temporarily unavailable. Please try again later.".to_string(),
        };
        
        for (player_id, session_addr) in &self.sessions {
            info!("Notifying player {} about service shutdown", player_id);
            session_addr.do_send(shutdown_message.clone());
        }
        
        info!("All players notified. Clearing session registry.");
        self.sessions.clear();
    }
}

impl Handler<GetActiveSessionsDebug> for SubscriptionManager {
    type Result = Result<Vec<SessionInfo>, anyhow::Error>;

    fn handle(&mut self, _msg: GetActiveSessionsDebug, _ctx: &mut Context<Self>) -> Self::Result {
        let sessions: Vec<SessionInfo> = self
            .sessions
            .keys()
            .map(|player_id| SessionInfo {
                player_id: player_id.to_string(),
                connected_at: chrono::Utc::now().to_rfc3339(), // 실제로는 연결 시간을 저장해야 함
            })
            .collect();

        Ok(sessions)
    }
}

impl Handler<ValidateActivePlayers> for SubscriptionManager {
    type Result = Vec<Uuid>;

    fn handle(&mut self, msg: ValidateActivePlayers, _ctx: &mut Context<Self>) -> Self::Result {
        msg.player_ids
            .into_iter()
            .filter(|player_id| self.sessions.contains_key(player_id))
            .collect()
    }
}

// --- RedisSubscriber Actor ---

pub struct RedisSubscriber {
    redis_client: RedisClient,
    manager_addr: Addr<SubscriptionManager>,
    reconnect_attempts: u32,
    consecutive_failures: u32,
    last_failure_time: Option<std::time::Instant>,
    settings: Settings,
    shutdown_tx: mpsc::Sender<()>, // Shutdown channel sender
}

impl RedisSubscriber {
    pub fn new(
        redis_client: RedisClient,
        manager_addr: Addr<SubscriptionManager>,
        settings: Settings,
        shutdown_tx: mpsc::Sender<()>,
    ) -> Self {
        Self {
            redis_client,
            manager_addr,
            reconnect_attempts: 0,
            consecutive_failures: 0,
            last_failure_time: None,
            settings,
            shutdown_tx,
        }
    }

    fn connect_and_subscribe(&mut self, ctx: &mut Context<Self>) {
        info!("Attempting to connect and subscribe to Redis...");
        let client = self.redis_client.clone();
        let manager = self.manager_addr.clone();
        let self_addr = ctx.address();
        let current_reconnect_attempts = self.reconnect_attempts;
        let current_consecutive_failures = self.consecutive_failures;
        let settings = self.settings.clone();
        let shutdown_tx = self.shutdown_tx.clone();

        async move {
            if current_reconnect_attempts >= settings.redis.max_reconnect_attempts {
                error!(
                    "Max Redis reconnect attempts ({}) reached. Performing graceful shutdown.",
                    settings.redis.max_reconnect_attempts
                );
                
                // Perform graceful shutdown of connected players first
                manager.do_send(GracefulShutdown);
                
                // Wait a bit for player notifications to be sent
                tokio::time::sleep(Duration::from_secs(2)).await;
                
                // Then send shutdown signal
                if shutdown_tx.send(()).await.is_err() {
                    error!("Failed to send shutdown signal. Forcing exit.");
                    std::process::exit(1); // Fallback
                }
                return;
            }

            let conn = match client.get_async_connection().await {
                Ok(c) => {
                    info!("Successfully connected to Redis.");
                    self_addr.do_send(ResetReconnectAttempts); // Reset on success
                    c
                }
                Err(e) => {
                    error!("RedisSubscriber failed to get connection: {}", e);
                    crate::metrics::MetricsCtx::new().inc_redis_connection_failure();
                    
                    // Update failure tracking
                    self_addr.do_send(RecordFailure);
                    
                    // Circuit breaker: introduce exponential backoff for consecutive failures
                    let backoff_delay = std::cmp::min(
                        Duration::from_secs(1 << current_consecutive_failures.min(5)),
                        Duration::from_secs(30)
                    );
                    
                    info!("Scheduling reconnect with backoff delay: {:?}", backoff_delay);
                    tokio::time::sleep(backoff_delay).await;
                    
                    self_addr.do_send(Connect); // Trigger reconnect
                    return;
                }
            };
            let mut pubsub = conn.into_pubsub();
            let channel_pattern = &settings.redis.notification_channel_pattern;
            if let Err(e) = pubsub.psubscribe(channel_pattern).await {
                error!("RedisSubscriber failed to psubscribe: {}", e);
                crate::metrics::MetricsCtx::new().inc_redis_connection_failure();
                self_addr.do_send(Connect); // Trigger reconnect
                return;
            }
            info!("Successfully subscribed to '{}'", channel_pattern);

            let mut stream = pubsub.on_message();
            let prefix_to_strip = channel_pattern.trim_end_matches('*');

            while let Some(msg) = stream.next().await {
                let channel: String = msg.get_channel_name().to_string();
                let payload: String = match msg.get_payload() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                if let Some(player_id_str) = channel.strip_prefix(prefix_to_strip) {
                    if let Ok(player_id) = Uuid::parse_str(player_id_str) {
                        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&payload) {
                            manager.do_send(ForwardMessage {
                                player_id,
                                message: server_msg,
                            });
                        }
                    }
                }
            }
            warn!("Redis Pub/Sub stream ended. Attempting to reconnect...");
            self_addr.do_send(Connect); // Trigger reconnect
        }
        .into_actor(self)
        .wait(ctx);
    }
}

impl Actor for RedisSubscriber {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        info!("RedisSubscriber actor started.");
        self.connect_and_subscribe(ctx);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct ResetReconnectAttempts;

impl Handler<ResetReconnectAttempts> for RedisSubscriber {
    type Result = ();
    fn handle(&mut self, _msg: ResetReconnectAttempts, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Redis connection successful. Resetting reconnect attempts and failure tracking.");
        self.reconnect_attempts = 0;
        self.consecutive_failures = 0;
        self.last_failure_time = None;
    }
}

impl Handler<RecordFailure> for RedisSubscriber {
    type Result = ();
    fn handle(&mut self, _msg: RecordFailure, _ctx: &mut Context<Self>) -> Self::Result {
        self.consecutive_failures += 1;
        self.last_failure_time = Some(std::time::Instant::now());
        warn!(
            "Redis connection failure recorded. Consecutive failures: {}",
            self.consecutive_failures
        );
    }
}

impl Handler<Connect> for RedisSubscriber {
    type Result = ();

    fn handle(&mut self, _msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        self.reconnect_attempts += 1;
        let delay = Duration::from_millis(std::cmp::min(
            self.settings.redis.max_reconnect_delay_ms,
            self.settings.redis.initial_reconnect_delay_ms
                * (2u64.pow(self.reconnect_attempts - 1)),
        ));
        info!("Reconnect message received. Attempt: {}. Waiting for a delay of {:?} before next attempt.", self.reconnect_attempts, delay);
        ctx.run_later(delay, |act, ctx| {
            act.connect_and_subscribe(ctx);
        });
    }
}
