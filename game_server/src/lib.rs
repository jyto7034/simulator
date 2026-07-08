use actix::{Addr, Message};
use backoff::ExponentialBackoff;
use game_core::game::data::GameDataBase;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock as TokioRwLock;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

use crate::env::RetrySettings;
use crate::env::Settings;
use crate::game::load_balance_actor::LoadBalanceActor;

lazy_static::lazy_static! {
    pub static ref RETRY_CONFIG: TokioRwLock<Option<ExponentialBackoff>> = TokioRwLock::new(None);
}

pub mod env;

// Module groups
pub mod game;
pub mod shared; // Shared infrastructure modules

pub struct LoggerManager {
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

#[derive(Debug)]
pub enum StopReason {
    ClientDisconnected,
    GracefulShutdown,
    Error(String),
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Stop {
    pub reason: StopReason,
}

impl LoggerManager {
    pub fn setup(settings: &Settings) -> Self {
        // 1. 로그 디렉토리 생성 (존재하지 않으면)
        if let Err(e) = std::fs::create_dir_all(&settings.logging.directory) {
            eprintln!(
                "Failed to create log directory '{}': {}",
                settings.logging.directory, e
            );
        }

        // 2. 파일 로거 설정
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            &settings.logging.directory,
            &settings.logging.filename,
        );
        let (non_blocking_file_writer, guard) = tracing_appender::non_blocking(file_appender);

        // 3. 로그 레벨 필터 설정 (환경 변수 또는 설정 파일 값)
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&settings.server.log_level));

        // 4. 콘솔 출력 레이어 설정
        let console_layer = fmt::layer()
            .with_writer(io::stdout) // 표준 출력으로 설정
            .with_ansi(true) // ANSI 색상 코드 사용 (터미널 지원 시)
            .with_thread_ids(true) // 스레드 ID 포함
            .with_thread_names(true) // 스레드 이름 포함
            .with_file(true) // 파일 경로 포함
            .with_line_number(true) // 라인 번호 포함
            .with_target(false) // target 정보 제외 (선택 사항)
            .pretty(); // 사람이 읽기 좋은 포맷

        // 5. 파일 출력 레이어 설정
        let file_layer = fmt::layer()
            .with_writer(non_blocking_file_writer) // Non-blocking 파일 로거 사용
            .with_ansi(false) // 파일에는 ANSI 코드 제외
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .pretty();

        // 6. 레지스트리(Registry)에 필터와 레이어 결합
        tracing_subscriber::registry()
            .with(filter) // 필터를 먼저 적용
            .with(console_layer) // 콘솔 레이어 추가
            .with(file_layer) // 파일 레이어 추가
            .init(); // 전역 Subscriber로 설정

        tracing::info!(
            "Logger initialization complete: console and file ({}/{}) output enabled",
            settings.logging.directory,
            settings.logging.filename
        );

        Self { _guard: guard }
    }
}

pub async fn init_retry_config(settings: &RetrySettings) {
    let backoff = ExponentialBackoff {
        max_elapsed_time: Some(Duration::from_millis(settings.message_max_elapsed_time_ms)),
        initial_interval: Duration::from_millis(settings.message_initial_interval_ms),
        max_interval: Duration::from_millis(settings.message_max_interval_ms),
        ..Default::default()
    };

    *RETRY_CONFIG.write().await = Some(backoff);
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub load_balance_addr: Addr<LoadBalanceActor>,
    pub logger_manager: Arc<LoggerManager>,
    pub current_run_id: Uuid,
    pub game_data: Arc<GameDataBase>,
}
