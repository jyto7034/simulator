use ::redis::aio::ConnectionManager;
use actix::{Addr, Message};
use actix_web::HttpRequest;
use backoff::ExponentialBackoff;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::env::RetrySettings;
use crate::subscript::SubScriptionManager;
use crate::{env::Settings, matchmaker::MatchmakerAddr, metrics::MetricsCtx};

lazy_static! {
    static ref RETRY_CONFIG: RwLock<Option<ExponentialBackoff>> = RwLock::new(None);
}

pub mod env;
pub mod event_stream;
pub mod matchmaker;
pub mod metrics;
pub mod protocol;
pub mod redis_events;
pub mod session;
pub mod subscript;

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
            eprintln!("Failed to create log directory '{}': {}", settings.logging.directory, e);
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

pub fn init_retry_config(settings: &RetrySettings) {
    let backoff = ExponentialBackoff {
        max_elapsed_time: Some(Duration::from_millis(settings.message_max_elapsed_time_ms)),
        initial_interval: Duration::from_millis(settings.message_initial_interval_ms),
        max_interval: Duration::from_millis(settings.message_max_interval_ms),
        ..Default::default()
    };

    *RETRY_CONFIG.write().unwrap() = Some(backoff);
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub matchmakers: HashMap<GameMode, MatchmakerAddr>,
    pub sub_manager_addr: Addr<SubScriptionManager>,
    pub redis: ConnectionManager,
    pub logger_manager: Arc<LoggerManager>,
    pub current_run_id: Arc<RwLock<Option<String>>>,
    pub metrics: Arc<MetricsCtx>,
    pub metrics_registry: prometheus::Registry,
    pub rate_limiter: Arc<RateLimiter>,
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameMode {
    None,
    #[serde(rename = "Normal")]
    Normal,
    #[serde(rename = "Ranked")]
    Ranked,
}

/// Simple rate limiter using token bucket algorithm
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<IpAddr, TokenBucket>>>,
    max_requests_per_second: u32,
    #[allow(dead_code)]
    cleanup_interval: Duration,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

impl RateLimiter {
    pub fn new(max_requests_per_second: u32) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            max_requests_per_second,
            cleanup_interval: Duration::from_secs(300), // cleanup every 5 minutes
        }
    }

    pub fn check(&self, ip: &IpAddr) -> bool {
        let mut buckets = self.buckets.write().unwrap();
        let bucket = buckets.entry(*ip).or_insert_with(|| TokenBucket {
            tokens: self.max_requests_per_second as f64,
            last_refill: Instant::now(),
            max_tokens: self.max_requests_per_second as f64,
            refill_rate: self.max_requests_per_second as f64,
        });

        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * bucket.refill_rate).min(bucket.max_tokens);
        bucket.last_refill = now;

        // Check if we have tokens
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Cleanup old entries (call periodically)
    pub fn cleanup(&self) {
        let mut buckets = self.buckets.write().unwrap();
        let now = Instant::now();
        buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_refill) < Duration::from_secs(600) // 10 minutes
        });
    }
}
