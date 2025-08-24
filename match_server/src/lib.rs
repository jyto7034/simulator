use actix::Addr;
use actix_web::HttpRequest;
use serde::{Deserialize, Serialize};
use std::io;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use ::redis::aio::ConnectionManager;

use crate::subscript::SubScriptionManager;
use crate::{
    blacklist::BlacklistManager, env::Settings, matchmaker::Matchmaker, metrics::MetricsCtx,
};

pub mod blacklist;
pub mod env;
pub mod matchmaker;
pub mod metrics;
pub mod protocol;
pub mod provider;
pub mod redis;
pub mod session;
pub mod subscript;

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

// 서버 전체에서 공

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub matchmaker_addr: Addr<Matchmaker>,
    pub sub_manager_addr: Addr<SubScriptionManager>,
    pub blacklist_manager_addr: Addr<BlacklistManager>,
    pub redis_conn_manager: ConnectionManager,
    pub logger_manager: Arc<LoggerManager>,
    pub current_run_id: Arc<RwLock<Option<String>>>,
    pub metrics: Arc<MetricsCtx>,
    pub metrics_registry: prometheus::Registry,
}

pub fn extract_client_ip(req: &HttpRequest) -> Option<IpAddr> {
    // 1. X-Forwarded-For 검증 강화
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            for ip_str in forwarded_str.split(',') {
                let ip_str = ip_str.trim();
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    // Private IP 및 localhost 필터링
                    if !is_private_or_loopback_ip(&ip) {
                        debug!("Extracted public client IP from X-Forwarded-For: {}", ip);
                        return Some(ip);
                    }
                }
            }
        }
    }

    // 2-3. 기존 X-Real-IP, CF-Connecting-IP 처리...

    // 4. Direct connection (개발 환경에서만 허용)
    if cfg!(debug_assertions) {
        // 디버그 빌드에서만
        if let Some(peer_addr) = req.connection_info().peer_addr() {
            if let Some(ip_str) = peer_addr.split(':').next() {
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    warn!("Using direct connection IP in development: {}", ip);
                    return Some(ip);
                }
            }
        }
    }

    error!(
        "Could not extract valid client IP from request headers: {:?}",
        req.headers()
    );
    None
}

fn is_private_or_loopback_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local(),
        IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unspecified(),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GameMode {
    #[serde(rename = "Normal")]
    Normal,
}
