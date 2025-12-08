pub mod config;
pub mod ecs;
pub mod game;

use std::{io, sync::Once};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct LoggerManager {
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

impl LoggerManager {
    pub fn setup() -> Self {
        // 1. 로그 디렉토리 생성 (존재하지 않으면)
        if let Err(e) = std::fs::create_dir_all("./logs") {
            eprintln!("Failed to create log directory '{}': {}", "./", e);
        }

        // 2. 파일 로거 설정
        let file_appender = RollingFileAppender::new(Rotation::DAILY, "./logs", "log");
        let (non_blocking_file_writer, guard) = tracing_appender::non_blocking(file_appender);

        // 3. 로그 레벨 필터 설정 (환경 변수 또는 설정 파일 값)
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        // 4. 콘솔 출력 레이어 설정
        let console_layer = fmt::layer()
            .with_writer(io::stdout)
            .with_ansi(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .pretty();

        // 5. 파일 출력 레이어 설정
        let file_layer = fmt::layer()
            .with_writer(non_blocking_file_writer)
            .with_ansi(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .pretty();

        // 6. 레지스트리(Registry)에 필터와 레이어 결합
        tracing_subscriber::registry()
            .with(filter)
            .with(console_layer)
            .with(file_layer)
            .init();

        tracing::info!(
            "Logger initialization complete: console and file ({}/{}) output enabled",
            "./logs",
            "log"
        );

        Self { _guard: guard }
    }
}

// ============================================================
// Test / global logging init helper
// ============================================================

static LOGGER_INIT: Once = Once::new();

/// Initialize logging once for the current process.
///
/// This is safe to call multiple times; subsequent calls are no-ops.
pub fn init_logging() {
    LOGGER_INIT.call_once(|| {
        // Leak LoggerManager so the WorkerGuard lives for the whole process.
        let logger = LoggerManager::setup();
        let _ = Box::leak(Box::new(logger));
    });
}

#[ctor::ctor]
fn init_tracing_for_tests() {
    // init_logging();
}
