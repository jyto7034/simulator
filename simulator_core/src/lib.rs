// #![allow(unused_variables, unused_labels, dead_code)]

use std::{collections::HashMap, future::Future, time::Duration};

use card::types::{PlayerIdentity, PlayerKind};
use exception::{GameError, SystemError};
use tracing::{debug, error, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

pub mod card;
pub mod card_gen;
pub mod effect;
pub mod enums;
pub mod exception;
pub mod game;
pub mod player;
pub mod resource;
pub mod selector;
pub mod sync;
pub mod utils;
pub mod zone;

use std::sync::Once;
static INIT: Once = Once::new();
static mut GUARD: Option<tracing_appender::non_blocking::WorkerGuard> = None;
pub fn setup_logger() {
    INIT.call_once(|| {
        // 1. 파일 로거 설정
        let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "app.log");
        let (non_blocking_file_writer, _guard) = tracing_appender::non_blocking(file_appender);

        // 2. 로그 레벨 필터 설정 (환경 변수 또는 기본값 INFO)
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")); // 기본 INFO 레벨

        // 3. 콘솔 출력 레이어 설정
        // let console_layer = fmt::layer()
        //     .with_writer(io::stdout) // 표준 출력으로 설정
        //     .with_ansi(true) // ANSI 색상 코드 사용 (터미널 지원 시)
        //     .with_thread_ids(true) // 스레드 ID 포함
        //     .with_thread_names(true) // 스레드 이름 포함
        //     .with_file(true) // 파일 경로 포함
        //     .with_line_number(true) // 라인 번호 포함
        //     .with_target(false) // target 정보 제외 (선택 사항)
        //     .pretty(); // 사람이 읽기 좋은 포맷

        // 4. 파일 출력 레이어 설정
        let file_layer = fmt::layer()
            .with_writer(non_blocking_file_writer) // Non-blocking 파일 로거 사용
            .with_ansi(false) // 파일에는 ANSI 코드 제외
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .pretty();

        // 5. 레지스트리(Registry)에 필터와 레이어 결합
        tracing_subscriber::registry()
            .with(filter) // 필터를 먼저 적용
            // .with(console_layer) // 콘솔 레이어 추가
            .with(file_layer) // 파일 레이어 추가
            .init(); // 전역 Subscriber로 설정

        unsafe {
            GUARD = Some(_guard);
        }

        tracing::info!("로거 초기화 완료: 콘솔 및 파일(logs/app.log) 출력 활성화.");
    });
}

pub trait StringUuidExt {
    fn to_uuid(&self) -> Result<Uuid, GameError>;
}

impl StringUuidExt for String {
    fn to_uuid(&self) -> Result<Uuid, GameError> {
        Uuid::parse_str(self)
            .map_err(|_| GameError::System(SystemError::Internal("UUID parse failed".to_string())))
    }
}

pub trait VecUuidExt {
    fn to_vec_string(&self) -> Vec<String>;
}

impl VecUuidExt for Vec<Uuid> {
    fn to_vec_string(&self) -> Vec<String> {
        self.iter()
            .map(|uuid| uuid.to_string())
            .collect::<Vec<String>>()
    }
}

pub trait VecStringExt {
    fn to_vec_uuid(&self) -> Result<Vec<Uuid>, GameError>;
}

impl VecStringExt for Vec<String> {
    fn to_vec_uuid(&self) -> Result<Vec<Uuid>, GameError> {
        self.iter()
            .map(|uuid| {
                Uuid::parse_str(uuid).map_err(|_| {
                    GameError::System(SystemError::Internal("UUID parse failed".to_string()))
                })
            })
            .collect::<Result<Vec<Uuid>, GameError>>()
    }
}

pub trait LogExt<T, E> {
    fn log_ok(self, f: impl FnOnce()) -> Self;
    fn log_err(self, f: impl FnOnce(&E)) -> Self;
}

impl<T, E> LogExt<T, E> for Result<T, E> {
    fn log_ok(self, f: impl FnOnce()) -> Self {
        if self.is_ok() {
            f()
        }
        self
    }

    fn log_err(self, f: impl FnOnce(&E)) -> Self {
        if let Err(ref e) = self {
            f(e);
        }
        self
    }
}
pub trait PlayerHashMapExt<V> {
    fn get_by_uuid(&self, uuid_key: &Uuid) -> Option<&V>;
    fn get_by_kind(&self, kind_key: PlayerKind) -> Option<&V>;
}

impl<V> PlayerHashMapExt<V> for HashMap<PlayerIdentity, V> {
    fn get_by_uuid(&self, uuid_key: &Uuid) -> Option<&V> {
        self.iter()
            .find(|(player_identity_key, _value)| player_identity_key.id == *uuid_key)
            .map(|(_player_identity_key, value)| value)
    }

    fn get_by_kind(&self, kind_key: PlayerKind) -> Option<&V> {
        self.iter()
            .find(|(player_identity_key, _value)| player_identity_key.kind == kind_key)
            .map(|(_player_identity_key, value)| value)
    }
}

pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            backoff_multiplier: 2.0,
            max_delay_ms: 5000,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Condition {
    Continue,
    Stop,
}

pub async fn retry_with_condition<F, Fut, T, E>(
    operation: F,
    config: RetryConfig,
    should_retry: impl Fn(&E) -> Condition,
    operation_name: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        "{} succeeded on attempt {}/{}",
                        operation_name, attempt, config.max_attempts
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                if should_retry(&e) == Condition::Stop {
                    warn!(
                        "{} failed with non-retryable error: {:?}",
                        operation_name, e
                    );
                    return Err(e);
                }
                if attempt == config.max_attempts {
                    error!(
                        "{} failed after {} attempts. Final error: {:?}",
                        operation_name, config.max_attempts, e
                    );
                    return Err(e);
                }

                warn!(
                    "{} failed on attempt {}/{}. Error: {:?}. Retrying...",
                    operation_name, attempt, config.max_attempts, e
                );

                let delay_ms = (config.base_delay_ms as f64
                    * config.backoff_multiplier.powi(attempt as i32 - 1))
                .min(config.max_delay_ms as f64) as u64;

                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    // 이 부분은 실제로 도달하지 않아야 함
    unreachable!()
}

pub async fn retry<F, Fut, T, E>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        "{} succeeded on attempt {}/{}",
                        operation_name, attempt, config.max_attempts
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt == config.max_attempts {
                    error!(
                        "{} failed after {} attempts. Final error: {:?}",
                        operation_name, config.max_attempts, e
                    );
                    return Err(e);
                }

                warn!(
                    "{} failed on attempt {}/{}. Error: {:?}. Retrying...",
                    operation_name, attempt, config.max_attempts, e
                );

                let delay_ms = (config.base_delay_ms as f64
                    * config.backoff_multiplier.powi(attempt as i32 - 1))
                .min(config.max_delay_ms as f64) as u64;

                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    // 이 부분은 실제로 도달하지 않아야 함
    unreachable!()
}
