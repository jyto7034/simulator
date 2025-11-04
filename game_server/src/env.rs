use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

use crate::GameMode;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub logging: LoggingSettings,
    pub server: ServerSettings,
    pub matchmaking: MatchmakingSettings,
    pub redis: RedisSettings,
    pub retry: RetrySettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        println!("Loading configuration for RUN_MODE: {}", &run_mode);

        let s = Config::builder()
            // Load environment-specific file (e.g., development.toml, production.toml)
            .add_source(
                File::with_name(&format!("config/{}", run_mode))
                    .format(FileFormat::Toml)
                    .required(true),
            )
            // Add environment variables (e.g., APP_SERVER__PORT=8000)
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatchmakingSettings {
    pub try_match_tick_interval_seconds: u64,
    pub queue_key_prefix: String,
    pub queue_order_key_prefix: String,
    pub match_fetch_request_channel_prefix: String,
    pub match_fetch_ack_channel_prefix: String,
    pub battle_request_channel: String,
    pub battle_result_channel_prefix: String,
    pub game_modes: Vec<MatchModeSettings>,
    pub heartbeat_interval_seconds: u64,
    pub heartbeat_timeout: u64,
    pub max_dedicated_server_retries: Option<u32>,
    pub dedicated_request_timeout_seconds: u64,
    pub allocation_token_ttl_seconds: u64,
    pub slow_loading_threshold_seconds: u64,
    /// Redis operation timeout in seconds (prevents infinite waiting on Redis operations)
    pub redis_operation_timeout_seconds: u64,
    /// Skip game server availability check (for development environments without game servers)
    #[serde(default)]
    pub skip_game_server_check: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub max_reconnect_attempts: u32,
    pub max_reconnect_delay_ms: u64,
    pub initial_reconnect_delay_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub bind_address: String,
    pub port: u16,
    pub log_level: String,
    pub metrics_auth_token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub directory: String,
    pub filename: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatchModeSettings {
    pub game_mode: GameMode,
    pub required_players: u32,
    pub use_mmr_matching: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetrySettings {
    pub message_max_elapsed_time_ms: u64,
    pub message_initial_interval_ms: u64,
    pub message_max_interval_ms: u64,
}
