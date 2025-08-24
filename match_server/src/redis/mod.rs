pub mod handlers;
pub mod messages;
use std::time::Duration;

use actix::{dev::ContextFutureSpawner, Actor, Addr, AsyncContext, Context, WrapFuture};
use redis::Client as RedisClient;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::{
    env::Settings,
    redis::messages::{Connect, RecordFailure, ResetReconnectAttempts},
    subscript::{messages::GracefulShutdown, SubScriptionManager},
};

pub struct RedisSubscriber {
    redis_client: RedisClient,
    subcription_addr: Addr<SubScriptionManager>,
    reconnect_attempts: u32,
    consecutive_failures: u32,
    last_failure_time: Option<std::time::Instant>,
    settings: Settings,
    shutdown_tx: mpsc::Sender<()>, // Shutdown channel sender
}

impl Actor for RedisSubscriber {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("RedisSubscriber actor started.");
        self.connect_and_subscribe(ctx);
    }
}

impl RedisSubscriber {
    pub fn connect_and_subscribe(&mut self, ctx: &mut Context<Self>) {
        let client = self.redis_client.clone();
        let manager = self.subcription_addr.clone();
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
                manager.send(GracefulShutdown).await.unwrap_or_else(|e| {
                    error!("Failed to send graceful shutdown message: {}", e);
                });

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
                        Duration::from_secs(30),
                    );

                    info!(
                        "Scheduling reconnect with backoff delay: {:?}",
                        backoff_delay
                    );
                    tokio::time::sleep(backoff_delay).await;

                    self_addr.do_send(Connect); // Trigger reconnect
                    return;
                }
            };

            let mut pubsub = conn.into_pubsub();
        }
        .into_actor(self)
        .wait(ctx);
    }
}
