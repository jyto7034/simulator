use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub logging: LoggingSettings,
    pub server: ServerSettings,
    pub matchmaking: MatchmakingSettings,
    pub redis: RedisSettings,
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
    pub dedicated_request_timeout_seconds: u64,
    pub allocation_token_ttl_seconds: u64,
    pub slow_loading_threshold_seconds: u64,
}

pub struct BlackListSettings {}

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
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    pub directory: String,
    pub filename: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GameModeSettings {
    pub id: String,
    pub required_players: u32,
    pub use_mmr_matching: bool,
}
