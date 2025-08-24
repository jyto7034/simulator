use std::time::Duration;

use actix::{AsyncContext, Context, Handler};
use tracing::{info, warn};
use tracing_subscriber::field::delimited;

use crate::redis::{
    messages::{Connect, RecordFailure, ResetReconnectAttempts},
    RedisSubscriber,
};

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
