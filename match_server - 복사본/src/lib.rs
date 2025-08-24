use crate::{env::Settings, matchmaker::Matchmaker, pubsub::SubscriptionManager, blacklist::BlacklistManager, loading_session::LoadingSessionManager};
use actix::Addr;
use redis::aio::ConnectionManager;
use std::{io, sync::{Arc, RwLock}};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod auth;
pub mod blacklist;
pub mod debug;
pub mod env;
pub mod events;
pub mod loading_session;
pub mod matchmaker;
pub mod protocol;
pub mod provider;
pub mod pubsub;
pub mod state_events;
// pub mod util; // removed: run_id moved to AppState
pub mod ws_session;
pub mod admin;

// metrics_helper removed

pub mod invariants;
pub mod metrics;
pub mod errors;

// RAII 패턴을 사용한 로거 매니저
pub struct LoggerManager {
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

impl LoggerManager {
    pub fn setup(settings: &Settings) -> Self {
        // 1. 파일 로거 설정
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            &settings.logging.directory,
            &settings.logging.filename,
        );
        let (non_blocking_file_writer, guard) = tracing_appender::non_blocking(file_appender);

        // 2. 로그 레벨 필터 설정 (환경 변수 또는 설정 파일 값)
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&settings.server.log_level));

        // 3. 콘솔 출력 레이어 설정
        let console_layer = fmt::layer()
            .with_writer(io::stdout) // 표준 출력으로 설정
            .with_ansi(true) // ANSI 색상 코드 사용 (터미널 지원 시)
            .with_thread_ids(true) // 스레드 ID 포함
            .with_thread_names(true) // 스레드 이름 포함
            .with_file(true) // 파일 경로 포함
            .with_line_number(true) // 라인 번호 포함
            .with_target(false) // target 정보 제외 (선택 사항)
            .pretty(); // 사람이 읽기 좋은 포맷

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
            .with(console_layer) // 콘솔 레이어 추가
            .with(file_layer) // 파일 레이어 추가
            .init(); // 전역 Subscriber로 설정

        tracing::info!(
            "로거 초기화 완료: 콘솔 및 파일({}/{}) 출력 활성화.",
            settings.logging.directory,
            settings.logging.filename
        );

        Self { _guard: guard }
    }
}

// 서버 전체에서 공유될 상태
#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub matchmaker_addr: Addr<Matchmaker>,
    pub sub_manager_addr: Addr<SubscriptionManager>,
    pub blacklist_manager_addr: Addr<BlacklistManager>,
    pub loading_session_manager_addr: Addr<LoadingSessionManager>,
    pub redis_conn_manager: ConnectionManager,
    pub _logger_manager: Arc<LoggerManager>, // RAII 패턴으로 메모리 관리
    pub current_run_id: Arc<RwLock<Option<String>>>,
    pub metrics: Arc<crate::metrics::MetricsCtx>,
    pub metrics_registry: prometheus::Registry,

}
