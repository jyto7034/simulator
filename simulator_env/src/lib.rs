use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env as std_env;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// 전체 Simulator 프로젝트를 위한 통합 환경 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorConfig {
    pub servers: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub testing: TestingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub auth_server: ServerEndpoint,
    pub match_server: ServerEndpoint,
    pub dedicated_server: ServerEndpoint,
    pub client: ClientConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEndpoint {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
}

impl ServerEndpoint {
    pub fn url(&self) -> String {
        let protocol = if self.use_tls { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.host, self.port)
    }

    pub fn ws_url(&self) -> String {
        let protocol = if self.use_tls { "wss" } else { "ws" };
        format!("{}://{}:{}", protocol, self.host, self.port)
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: u8,
}

impl RedisConfig {
    pub fn url(&self) -> String {
        match &self.password {
            Some(pass) => format!("redis://:{}@{}:{}/{}", pass, self.host, self.port, self.db),
            None => format!("redis://{}:{}/{}", self.host, self.port, self.db),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub directory: String,
    pub filename: String,
    pub max_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingConfig {
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub parallel_tests: u32,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            servers: ServerConfig {
                auth_server: ServerEndpoint {
                    host: "127.0.0.1".to_string(),
                    port: 8081,
                    use_tls: false,
                },
                match_server: ServerEndpoint {
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                    use_tls: false,
                },
                dedicated_server: ServerEndpoint {
                    host: "127.0.0.1".to_string(),
                    port: 8082,
                    use_tls: false,
                },
                client: ClientConfig {
                    host: "127.0.0.1".to_string(),
                    port: 3000,
                },
            },
            database: DatabaseConfig {
                redis: RedisConfig {
                    host: "127.0.0.1".to_string(),
                    port: 6379,
                    password: None,
                    db: 0,
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                directory: "logs".to_string(),
                filename: "app.log".to_string(),
                max_files: 10,
            },
            testing: TestingConfig {
                timeout_seconds: 30,
                retry_count: 3,
                parallel_tests: 4,
            },
        }
    }
}

static CONFIG: Lazy<SimulatorConfig> = Lazy::new(|| {
    SimulatorConfig::load().unwrap_or_else(|e| {
        warn!("Failed to load config: {}. Using defaults.", e);
        SimulatorConfig::default()
    })
});

impl SimulatorConfig {
    /// 전역 설정 인스턴스 가져오기
    pub fn global() -> &'static SimulatorConfig {
        &CONFIG
    }

    /// 설정 파일 로드
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = Self::get_config_dir();
        let config_file = config_dir.join("simulator.toml");

        info!("Loading configuration from: {:?}", config_file);

        let settings = Config::builder()
            // 기본값 설정
            .add_source(Config::try_from(&Self::default())?)
            // 설정 파일 로드 (선택사항)
            .add_source(File::from(config_file).required(false))
            // 환경 변수 오버라이드 (SIMULATOR_ 접두사)
            .add_source(Environment::with_prefix("SIMULATOR").separator("_"))
            .build()?;

        let config: SimulatorConfig = settings.try_deserialize()?;
        debug!("Loaded configuration: {:?}", config);
        Ok(config)
    }

    /// 설정 파일 저장
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_config_dir();
        std::fs::create_dir_all(&config_dir)?;

        let config_file = config_dir.join("simulator.toml");
        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(config_file, toml_string)?;

        Ok(())
    }

    /// 설정 디렉토리 가져오기
    fn get_config_dir() -> PathBuf {
        if let Ok(config_home) = std_env::var("XDG_CONFIG_HOME") {
            PathBuf::from(config_home).join("simulator")
        } else if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".config").join("simulator")
        } else {
            // 최후의 수단으로 현재 디렉토리
            PathBuf::from("./config")
        }
    }

    /// 개발 환경용 설정 생성
    pub fn development() -> Self {
        let mut config = Self::default();
        config.logging.level = "debug".to_string();
        config.testing.timeout_seconds = 60;
        config
    }

    /// 프로덕션 환경용 설정 생성
    pub fn production() -> Self {
        let mut config = Self::default();
        config.logging.level = "warn".to_string();
        config.servers.auth_server.use_tls = true;
        config.servers.match_server.use_tls = true;
        config.servers.dedicated_server.use_tls = true;
        config.database.redis.password = Some("production_password".to_string());
        config
    }

    /// 테스트 환경용 설정 생성
    pub fn testing() -> Self {
        let mut config = Self::default();
        config.logging.level = "trace".to_string();
        config.testing.timeout_seconds = 10;
        config.testing.parallel_tests = 1;
        config.database.redis.db = 1; // 테스트용 DB 분리
        config
    }
}

/// 환경 변수 헬퍼 함수들
pub mod env {
    use super::SimulatorConfig;

    /// 매치 서버 URL 가져오기
    pub fn match_server_url() -> String {
        SimulatorConfig::global().servers.match_server.url()
    }

    /// 매치 서버 WebSocket URL 가져오기
    pub fn match_server_ws_url() -> String {
        SimulatorConfig::global().servers.match_server.ws_url()
    }

    /// 인증 서버 URL 가져오기
    pub fn auth_server_url() -> String {
        SimulatorConfig::global().servers.auth_server.url()
    }

    /// 전용 서버 URL 가져오기
    pub fn dedicated_server_url() -> String {
        SimulatorConfig::global().servers.dedicated_server.url()
    }

    /// Redis URL 가져오기
    pub fn redis_url() -> String {
        SimulatorConfig::global().database.redis.url()
    }

    /// 로그 레벨 가져오기
    pub fn log_level() -> String {
        SimulatorConfig::global().logging.level.clone()
    }

    /// 테스트 타임아웃 가져오기
    pub fn test_timeout_seconds() -> u64 {
        SimulatorConfig::global().testing.timeout_seconds
    }
}

/// 설정 초기화 함수
pub fn init() -> Result<()> {
    dotenv::dotenv().ok();

    // 전역 설정 초기화 (Lazy 실행)
    let config = SimulatorConfig::global();
    info!("Simulator configuration initialized");
    debug!("Configuration: {:?}", config);

    Ok(())
}

/// 설정 파일 생성 헬퍼
pub fn create_default_config() -> Result<()> {
    let config = SimulatorConfig::default();
    config.save()?;
    info!("Default configuration file created");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_endpoint_url() {
        let endpoint = ServerEndpoint {
            host: "localhost".to_string(),
            port: 8080,
            use_tls: false,
        };
        assert_eq!(endpoint.url(), "http://localhost:8080");
        assert_eq!(endpoint.ws_url(), "ws://localhost:8080");
    }

    #[test]
    fn test_redis_url() {
        let redis = RedisConfig {
            host: "localhost".to_string(),
            port: 6379,
            password: None,
            db: 0,
        };
        assert_eq!(redis.url(), "redis://localhost:6379/0");

        let redis_with_password = RedisConfig {
            host: "localhost".to_string(),
            port: 6379,
            password: Some("secret".to_string()),
            db: 1,
        };
        assert_eq!(
            redis_with_password.url(),
            "redis://:secret@localhost:6379/1"
        );
    }

    #[test]
    fn test_config_environments() {
        let dev = SimulatorConfig::development();
        assert_eq!(dev.logging.level, "debug");

        let prod = SimulatorConfig::production();
        assert_eq!(prod.logging.level, "warn");
        assert!(prod.servers.auth_server.use_tls);
    }
}
