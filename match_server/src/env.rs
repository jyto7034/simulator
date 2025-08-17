use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub bind_address: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub directory: String,
    pub filename: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub url: String,
    pub max_reconnect_attempts: u32,
    pub initial_reconnect_delay_ms: u64,
    pub max_reconnect_delay_ms: u64,
    pub dedicated_server_key_pattern: String,
    pub notification_channel_pattern: String,
    pub state_event_channel_pattern: String,
    pub enable_state_events: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtSettings {
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerStatusSettings {
    pub idle: String,
}

/// TOML 설정 파일의 [[matchmaking.game_modes]] 테이블에 대응하는 구조체입니다.
#[derive(Debug, Deserialize, Clone)]
pub struct GameModeSettings {
    pub id: String,
    pub required_players: u32,
    pub use_mmr_matching: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatchmakingSettings {
    pub tick_interval_seconds: u64,
    pub queue_key_prefix: String,
    pub game_modes: Vec<GameModeSettings>,
    pub heartbeat_interval_seconds: u64,
    pub client_timeout_seconds: u64,
    pub loading_session_timeout_seconds: u64,
    pub max_dedicated_server_retries: Option<u32>,
    // New: external request timeout for dedicated server allocation (seconds)
    pub dedicated_request_timeout_seconds: u64,
    // New: allocation token TTL to guard single allocation winner (seconds)
    pub allocation_token_ttl_seconds: u64,
    // New: classify a player as "slow_loader" if loading takes longer than this threshold (seconds)
    pub slow_loading_threshold_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub logging: LoggingSettings,
    pub redis: RedisSettings,
    pub jwt: JwtSettings,
    pub matchmaking: MatchmakingSettings,
    pub server_status: ServerStatusSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Read RUN_MODE, be tolerant of accidental whitespace or extension suffix
        let raw = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let cleaned = raw.trim().trim_end_matches(".toml").to_string();
        let file_stem = if cleaned.is_empty() { "development".to_string() } else { cleaned };

        let s = Config::builder()
            .add_source(
                File::with_name(&format!("config/{}", file_stem))
                    .format(FileFormat::Toml)
                    .required(true),
            )
            .build()?;

        s.try_deserialize()
    }
}
