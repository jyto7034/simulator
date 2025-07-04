use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub password: Option<String>, // 필요하다면
}

#[derive(Debug, Deserialize)]
pub struct MatchmakingLogicConfig {
    pub matchmaking_interval_seconds: u64,
    pub initial_mmr_range: i32,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub matchmaking: MatchmakingLogicConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // 기본 설정 파일 로드
            .add_source(File::with_name("config/default"))
            // 환경별 설정 파일 로드 (예: config/production.toml) - 선택 사항
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // 환경 변수 로드 (예: APP_SERVER_PORT=8000)
            // APP_ 접두사를 사용하고, 구분자는 __ (더블 언더스코어)
            // 예: APP_REDIS__URL="redis://..."
            .add_source(Environment::with_prefix("app").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
