use crate::{env::Settings, protocol::ServerMessage, ws_session::MatchmakingSession};
use actix::{
    Actor, Addr, AsyncContext, Context, ContextFutureSpawner, Handler, Message, WrapFuture,
};
use futures_util::stream::StreamExt;
use redis::Client as RedisClient;
use simulator_metrics::{APPLICATION_RESTARTS_TOTAL, REDIS_CONNECTION_FAILURES_TOTAL};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;
use serde::Serialize;

// --- Messages for this module ---
#[derive(Message)]
#[rtype(result = "()")]
struct Connect;

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
    }
}

impl Handler<Deregister> for SubscriptionManager {
    type Result = ();
    fn handle(&mut self, msg: Deregister, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Player {} deregistered.", msg.player_id);
        self.sessions.remove(&msg.player_id);
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

// --- RedisSubscriber Actor ---

pub struct RedisSubscriber {
    redis_client: RedisClient,
    manager_addr: Addr<SubscriptionManager>,
    reconnect_attempts: u32,
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
        let settings = self.settings.clone();
        let shutdown_tx = self.shutdown_tx.clone();

        async move {
            if current_reconnect_attempts >= settings.redis.max_reconnect_attempts {
                REDIS_CONNECTION_FAILURES_TOTAL
                    .with_label_values(&["pubsub"])
                    .inc();
                APPLICATION_RESTARTS_TOTAL.inc();
                error!(
                    "Max Redis reconnect attempts ({}) reached. Sending shutdown signal.",
                    settings.redis.max_reconnect_attempts
                );
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
                    REDIS_CONNECTION_FAILURES_TOTAL
                        .with_label_values(&["pubsub"])
                        .inc();
                    error!("RedisSubscriber failed to get connection: {}", e);
                    self_addr.do_send(Connect); // Trigger reconnect
                    return;
                }
            };
            let mut pubsub = conn.into_pubsub();
            let channel_pattern = &settings.redis.notification_channel_pattern;
            if let Err(e) = pubsub.psubscribe(channel_pattern).await {
                REDIS_CONNECTION_FAILURES_TOTAL
                    .with_label_values(&["pubsub"])
                    .inc();
                error!("RedisSubscriber failed to psubscribe: {}", e);
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
        info!("Redis connection successful. Resetting reconnect attempts.");
        self.reconnect_attempts = 0;
    }
}

impl Handler<Connect> for RedisSubscriber {
    type Result = ();

    fn handle(&mut self, _msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        self.reconnect_attempts += 1;
        let delay = Duration::from_millis(std::cmp::min(
            self.settings.redis.max_reconnect_delay_ms,
            self.settings.redis.initial_reconnect_delay_ms * (2u64.pow(self.reconnect_attempts - 1)),
        ));
        info!("Reconnect message received. Attempt: {}. Waiting for a delay of {:?} before next attempt.", self.reconnect_attempts, delay);
        ctx.run_later(delay, |act, ctx| {
            act.connect_and_subscribe(ctx);
        });
    }
}
