use actix::prelude::*;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};
use uuid::Uuid;

pub mod config;
pub mod messages;

use config::{BlacklistConfig, IpChangeStrategy};
use messages::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    Timeout,
    UnknownType,
    MissingField,
    Duplicated,
    WrongSessionId,
}

#[derive(Debug, Clone)]
pub struct ViolationRecord {
    pub player_id: Uuid,
    pub violation_type: ViolationType,
    pub timestamp: u64,
    pub ip_addr: Option<IpAddr>,
}

pub struct BlacklistManager {
    redis_client: redis::Client,
    config: BlacklistConfig,
}

impl BlacklistManager {
    pub fn new(redis_client: redis::Client, config: BlacklistConfig) -> Self {
        Self {
            redis_client,
            config,
        }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn violation_key(&self, player_id: Uuid, violation_type: &ViolationType) -> String {
        match violation_type {
            ViolationType::Timeout => format!("violations:{}:timeout", player_id),
            _ => format!("violations:{}:other", player_id),
        }
    }

    fn block_key(&self, player_id: Uuid) -> String {
        format!("blocked:{}", player_id)
    }

    fn player_ip_key(&self, player_id: Uuid) -> String {
        format!("player_ips:{}", player_id)
    }

    async fn get_redis_connection(&self) -> Result<redis::aio::Connection, anyhow::Error> {
        Ok(self.redis_client.get_async_connection().await?)
    }

    async fn check_ip_change(
        &self,
        player_id: Uuid,
        current_ip: IpAddr,
    ) -> Result<bool, anyhow::Error> {
        let mut conn = self.get_redis_connection().await?;
        let ip_key = self.player_ip_key(player_id);

        let stored_ip: Option<String> = conn.get(&ip_key).await?;
        let current_ip_str = current_ip.to_string();

        if let Some(stored) = stored_ip {
            if stored != current_ip_str {
                info!(
                    "IP change detected for player {}: {} -> {}",
                    player_id, stored, current_ip_str
                );
                return Ok(true);
            }
        }

        // Update current IP
        let _: () = conn.set(&ip_key, &current_ip_str).await?;
        let _: () = conn.expire(&ip_key, 24 * 3600).await?; // 24 hours expiry

        Ok(false)
    }

    async fn reduce_player_violations(&self, player_id: Uuid) -> Result<(), anyhow::Error> {
        match self.config.ip_change_strategy {
            IpChangeStrategy::ClearAll => {
                // Original behavior - clear all violations
                self.clear_all_violations(player_id).await?;
                info!(
                    "Cleared all violations for player {} due to IP change",
                    player_id
                );
            }
            IpChangeStrategy::ReduceByPercent => {
                // Reduce violations by percentage
                self.reduce_violations_by_percent(player_id).await?;
                info!(
                    "Reduced violations by {}% for player {} due to IP change",
                    self.config.ip_change_reduction_percent, player_id
                );
            }
            IpChangeStrategy::KeepRecent => {
                // Keep only recent violations
                self.keep_recent_violations(player_id).await?;
                info!(
                    "Kept recent violations for player {} due to IP change",
                    player_id
                );
            }
            IpChangeStrategy::NoReduction => {
                // No reduction - just log the IP change
                info!(
                    "IP change detected for player {} but no violation reduction applied",
                    player_id
                );
            }
        }
        Ok(())
    }

    async fn clear_all_violations(&self, player_id: Uuid) -> Result<(), anyhow::Error> {
        let mut conn = self.get_redis_connection().await?;

        let timeout_key = self.violation_key(player_id, &ViolationType::Timeout);
        let other_key = self.violation_key(player_id, &ViolationType::UnknownType);
        let block_key = self.block_key(player_id);

        let _: () = conn.del(&timeout_key).await?;
        let _: () = conn.del(&other_key).await?;
        let _: () = conn.del(&block_key).await?;

        Ok(())
    }

    async fn reduce_violations_by_percent(&self, player_id: Uuid) -> Result<(), anyhow::Error> {
        let mut conn = self.get_redis_connection().await?;
        let current_time = Self::current_timestamp();
        let window_start = current_time - self.config.timeout_window_hours * 3600;

        // Process timeout violations
        let timeout_key = self.violation_key(player_id, &ViolationType::Timeout);
        let violations: Vec<String> = conn.lrange(&timeout_key, 0, -1).await?;

        let valid_violations: Vec<u64> = violations
            .iter()
            .filter_map(|v| v.parse::<u64>().ok())
            .filter(|&timestamp| timestamp > window_start)
            .collect();

        if !valid_violations.is_empty() {
            let keep_count = std::cmp::max(
                (valid_violations.len() * (100 - self.config.ip_change_reduction_percent as usize))
                    / 100,
                self.config.ip_change_min_violations_to_keep as usize,
            );

            // Clear and rebuild with reduced violations (keep most recent)
            let _: () = conn.del(&timeout_key).await?;

            if keep_count > 0 {
                let mut sorted_violations = valid_violations;
                sorted_violations.sort_by(|a, b| b.cmp(a)); // Most recent first

                for violation in sorted_violations.into_iter().take(keep_count) {
                    let _: () = conn.lpush(&timeout_key, violation.to_string()).await?;
                }

                let _: () = conn
                    .expire(
                        &timeout_key,
                        (self.config.timeout_window_hours * 3600) as usize,
                    )
                    .await?;
            }
        }

        // Always clear current block to give immediate relief
        let block_key = self.block_key(player_id);
        let _: () = conn.del(&block_key).await?;

        Ok(())
    }

    async fn keep_recent_violations(&self, player_id: Uuid) -> Result<(), anyhow::Error> {
        let mut conn = self.get_redis_connection().await?;
        let current_time = Self::current_timestamp();
        let keep_duration = 3600; // Keep violations from last hour only
        let cutoff_time = current_time - keep_duration;

        let timeout_key = self.violation_key(player_id, &ViolationType::Timeout);
        let violations: Vec<String> = conn.lrange(&timeout_key, 0, -1).await?;

        let recent_violations: Vec<String> = violations
            .iter()
            .filter_map(|v| v.parse::<u64>().ok())
            .filter(|&timestamp| timestamp > cutoff_time)
            .map(|t| t.to_string())
            .collect();

        // Clear and rebuild with recent violations only
        let _: () = conn.del(&timeout_key).await?;

        if !recent_violations.is_empty() {
            for violation in recent_violations {
                let _: () = conn.lpush(&timeout_key, violation).await?;
            }
            let _: () = conn
                .expire(
                    &timeout_key,
                    (self.config.timeout_window_hours * 3600) as usize,
                )
                .await?;
        }

        // Clear current block
        let block_key = self.block_key(player_id);
        let _: () = conn.del(&block_key).await?;

        Ok(())
    }

    async fn record_violation_internal(
        &self,
        player_id: Uuid,
        violation_type: ViolationType,
        _ip_addr: Option<IpAddr>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.get_redis_connection().await?;
        let violation_key = self.violation_key(player_id, &violation_type);
        let current_time = Self::current_timestamp();
        let window_start = current_time - self.config.timeout_window_hours * 3600;

        // Use Redis transaction for atomic operations to prevent race conditions
        let (violations, total_violations): (Vec<String>, usize) = redis::pipe()
            .atomic()
            .lrange(&violation_key, 0, -1)
            .query_async(&mut conn)
            .await
            .map(|violations: Vec<String>| {
                // Filter violations within time window
                let valid_violations: Vec<String> = violations
                    .iter()
                    .filter_map(|v| v.parse::<u64>().ok())
                    .filter(|&timestamp| timestamp > window_start)
                    .map(|t| t.to_string())
                    .collect();

                let total = valid_violations.len() + 1; // +1 for current violation
                (valid_violations, total)
            })?;

        // Atomic update: clear and rebuild list with valid violations + new one
        let mut pipe = redis::pipe();
        pipe.atomic().del(&violation_key);

        // Add valid violations back
        for violation in violations {
            pipe.lpush(&violation_key, violation);
        }

        // Add current violation
        pipe.lpush(&violation_key, current_time.to_string()).expire(
            &violation_key,
            (self.config.timeout_window_hours * 3600) as usize,
        );

        // Execute the pipeline atomically
        let _: () = pipe.query_async(&mut conn).await?;

        debug!(
            "Player {} has {} {} violations in last {} hours",
            player_id,
            total_violations,
            format!("{:?}", violation_type).to_lowercase(),
            self.config.timeout_window_hours
        );

        // Check if should be blocked (only for timeout violations)
        if matches!(violation_type, ViolationType::Timeout)
            && total_violations >= self.config.timeout_threshold as usize
        {
            let block_key = self.block_key(player_id);
            let block_until = current_time + (self.config.block_duration_minutes * 60);

            // Atomic block operation
            let _: () = redis::pipe()
                .atomic()
                .set(&block_key, block_until.to_string())
                .expire(
                    &block_key,
                    (self.config.block_duration_minutes * 60) as usize,
                )
                .query_async(&mut conn)
                .await?;

            warn!(
                "Player {} blocked for {} minutes due to {} timeout violations",
                player_id, self.config.block_duration_minutes, total_violations
            );
        }

        Ok(())
    }

    async fn is_player_blocked_internal(
        &self,
        player_id: Uuid,
        ip_addr: Option<IpAddr>,
    ) -> Result<BlockCheckResult, anyhow::Error> {
        // Check IP change if address provided
        if let Some(ip) = ip_addr {
            if self.config.check_ip_change && self.check_ip_change(player_id, ip).await? {
                // Apply IP change violation reduction strategy
                match self.config.ip_change_strategy {
                    IpChangeStrategy::NoReduction => {
                        // Still proceed with normal block check
                    }
                    _ => {
                        self.reduce_player_violations(player_id).await?;
                        return Ok(BlockCheckResult::Allowed);
                    }
                }
            }
        }

        let mut conn = self.get_redis_connection().await?;
        let block_key = self.block_key(player_id);

        let block_until: Option<String> = conn.get(&block_key).await?;

        if let Some(until_str) = block_until {
            if let Ok(until_timestamp) = until_str.parse::<u64>() {
                let current_time = Self::current_timestamp();
                if current_time < until_timestamp {
                    let remaining_seconds = until_timestamp - current_time;
                    return Ok(BlockCheckResult::Blocked {
                        remaining_seconds,
                        reason: "연속된 게임 중단으로 인해 일시적으로 접속이 제한되었습니다."
                            .to_string(),
                    });
                }
            }
        }

        Ok(BlockCheckResult::Allowed)
    }
}

impl Actor for BlacklistManager {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("BlacklistManager started with config: {:?}", self.config);
        info!("Blacklist timeout_threshold set to: {}", self.config.timeout_threshold);
    }
}

impl Handler<RecordViolation> for BlacklistManager {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: RecordViolation, _ctx: &mut Self::Context) -> Self::Result {
        let player_id = msg.player_id;
        let violation_type = msg.violation_type;
        let ip_addr = msg.ip_addr;
        let redis_client = self.redis_client.clone();
        let config = self.config.clone();

        Box::pin(
            async move {
                let manager = BlacklistManager {
                    redis_client,
                    config,
                };
                manager
                    .record_violation_internal(player_id, violation_type, ip_addr)
                    .await
            }
            .into_actor(self),
        )
    }
}

impl Handler<CheckPlayerBlock> for BlacklistManager {
    type Result = ResponseActFuture<Self, Result<BlockCheckResult, anyhow::Error>>;

    fn handle(&mut self, msg: CheckPlayerBlock, _ctx: &mut Self::Context) -> Self::Result {
        let player_id = msg.player_id;
        let ip_addr = msg.ip_addr;
        let redis_client = self.redis_client.clone();
        let config = self.config.clone();

        Box::pin(
            async move {
                let manager = BlacklistManager {
                    redis_client,
                    config,
                };
                manager.is_player_blocked_internal(player_id, ip_addr).await
            }
            .into_actor(self),
        )
    }
}

impl Handler<ClearPlayerViolations> for BlacklistManager {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: ClearPlayerViolations, _ctx: &mut Self::Context) -> Self::Result {
        let player_id = msg.player_id;
        let redis_client = self.redis_client.clone();
        let config = self.config.clone();

        Box::pin(
            async move {
                let manager = BlacklistManager {
                    redis_client,
                    config,
                };
                manager.clear_all_violations(player_id).await
            }
            .into_actor(self),
        )
    }
}
