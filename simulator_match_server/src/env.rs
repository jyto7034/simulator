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
    // 이제 게임 모드는 단순 문자열이 아닌, 자체 설정을 가진 구조체의 벡터가 됩니다.
    pub game_modes: Vec<GameModeSettings>,
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
