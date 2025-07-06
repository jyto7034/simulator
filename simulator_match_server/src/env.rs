use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    pub bind_address: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtSettings {
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MatchmakingSettings {
    pub tick_interval_seconds: u64,
    pub queue_key_prefix: String,
    pub game_modes: Vec<String>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub redis: RedisSettings,
    pub jwt: JwtSettings,
    pub matchmaking: MatchmakingSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(true))
            .build()?;

        s.try_deserialize()
    }
}