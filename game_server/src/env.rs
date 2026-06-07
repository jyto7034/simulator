use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub logging: LoggingSettings,
    pub server: ServerSettings,
    pub session: SessionSettings,
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
pub struct SessionSettings {
    pub heartbeat_interval_seconds: u64,
    pub heartbeat_timeout: u64,
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
pub struct RetrySettings {
    pub message_max_elapsed_time_ms: u64,
    pub message_initial_interval_ms: u64,
    pub message_max_interval_ms: u64,
}
