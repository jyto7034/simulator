use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistConfig {
    /// Number of timeout violations required to trigger a block
    pub timeout_threshold: u32,
    /// Time window in hours to count violations
    pub timeout_window_hours: u64,
    /// Block duration in minutes
    pub block_duration_minutes: u64,
    /// Whether to check for IP changes and clear violations
    pub check_ip_change: bool,
    /// IP change violation reduction strategy
    pub ip_change_strategy: IpChangeStrategy,
    /// Percentage of violations to reduce on IP change (0-100)
    pub ip_change_reduction_percent: u32,
    /// Minimum violations to keep on IP change
    pub ip_change_min_violations_to_keep: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpChangeStrategy {
    /// Clear all violations (original behavior)
    ClearAll,
    /// Reduce violations by percentage
    ReduceByPercent,
    /// Keep recent violations only
    KeepRecent,
    /// No reduction (disable IP change effects)
    NoReduction,
}

impl Default for BlacklistConfig {
    fn default() -> Self {
        Self {
            timeout_threshold: 2,
            timeout_window_hours: 1,
            block_duration_minutes: 10,
            check_ip_change: true,
            ip_change_strategy: IpChangeStrategy::ReduceByPercent,
            ip_change_reduction_percent: 50, // Reduce by 50%
            ip_change_min_violations_to_keep: 1, // Keep at least 1 violation
        }
    }
}

impl BlacklistConfig {
    pub fn from_env() -> Self {
        let ip_change_strategy = std::env::var("BLACKLIST_IP_CHANGE_STRATEGY")
            .ok()
            .and_then(|s| match s.to_lowercase().as_str() {
                "clear_all" => Some(IpChangeStrategy::ClearAll),
                "reduce_by_percent" => Some(IpChangeStrategy::ReduceByPercent),
                "keep_recent" => Some(IpChangeStrategy::KeepRecent),
                "no_reduction" => Some(IpChangeStrategy::NoReduction),
                _ => None,
            })
            .unwrap_or(IpChangeStrategy::ReduceByPercent);

        Self {
            timeout_threshold: std::env::var("BLACKLIST_TIMEOUT_THRESHOLD")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            timeout_window_hours: std::env::var("BLACKLIST_TIMEOUT_WINDOW_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
            block_duration_minutes: std::env::var("BLACKLIST_BLOCK_DURATION_MINUTES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            check_ip_change: std::env::var("BLACKLIST_CHECK_IP_CHANGE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            ip_change_strategy,
            ip_change_reduction_percent: std::env::var("BLACKLIST_IP_CHANGE_REDUCTION_PERCENT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
            ip_change_min_violations_to_keep: std::env::var("BLACKLIST_IP_CHANGE_MIN_VIOLATIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
        }
    }
}