use lazy_static::lazy_static;
use prometheus::{
    opts, IntCounter, IntCounterVec,
    IntGauge, Registry,
};

lazy_static! {
    // lazy_static을 사용하여 메트릭을 전역적으로 생성합니다.
    // register_... 매크로는 기본 레지스트리에 자동으로 등록하므로,
    // 여기서는 Opts만 생성하고 나중에 수동으로 등록합니다.

    /// The total number of matches created successfully.
    pub static ref MATCHES_CREATED_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("matches_created_total", "Total number of matches created")).unwrap();

    /// The current number of players waiting in the matchmaking queue.
    pub static ref PLAYERS_IN_QUEUE: IntGauge =
        IntGauge::with_opts(opts!("players_in_queue", "Current number of players in matchmaking queue")).unwrap();

    /// The current number of active game sessions across all dedicated servers.
    pub static ref ACTIVE_SESSIONS: IntGauge =
        IntGauge::with_opts(opts!("active_sessions", "Current number of active game sessions")).unwrap();

    /// The total number of Redis connection failures by component.
    pub static ref REDIS_CONNECTION_FAILURES_TOTAL: IntCounterVec =
        IntCounterVec::new(opts!("redis_connection_failures_total", "Total number of Redis connection failures"), &["component"]).unwrap();

    /// The total number of HTTP timeout errors by target.
    pub static ref HTTP_TIMEOUT_ERRORS_TOTAL: IntCounterVec =
        IntCounterVec::new(opts!("http_timeout_errors_total", "Total number of HTTP timeout errors"), &["target"]).unwrap();

    /// The total number of matchmaking errors by type.
    pub static ref MATCHMAKING_ERRORS_TOTAL: IntCounterVec =
        IntCounterVec::new(opts!("matchmaking_errors_total", "Total number of matchmaking errors"), &["error_type"]).unwrap();

    /// The total number of application crashes/restarts.
    pub static ref APPLICATION_RESTARTS_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("application_restarts_total", "Total number of application restarts")).unwrap();

    /// The total number of critical system time errors.
    pub static ref SYSTEM_TIME_ERRORS_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("system_time_errors_total", "Total number of critical system time errors")).unwrap();
}

/// Registers all custom metrics defined in this crate to the given registry.
///
/// This function should be called by each service during its startup phase
/// to ensure metrics are available for scraping.
///
/// # Arguments
///
/// * `registry` - The Prometheus registry provided by the `actix-web-prom` middleware.
///
/// # Returns
///
/// * `Result<(), prometheus::Error>` - Returns an error if any metric fails to register.
pub fn register_custom_metrics(registry: &Registry) -> Result<(), prometheus::Error> {
    registry.register(Box::new(MATCHES_CREATED_TOTAL.clone()))?;
    registry.register(Box::new(PLAYERS_IN_QUEUE.clone()))?;
    registry.register(Box::new(ACTIVE_SESSIONS.clone()))?;
    registry.register(Box::new(REDIS_CONNECTION_FAILURES_TOTAL.clone()))?;
    registry.register(Box::new(HTTP_TIMEOUT_ERRORS_TOTAL.clone()))?;
    registry.register(Box::new(MATCHMAKING_ERRORS_TOTAL.clone()))?;
    registry.register(Box::new(APPLICATION_RESTARTS_TOTAL.clone()))?;
    registry.register(Box::new(SYSTEM_TIME_ERRORS_TOTAL.clone()))?;
    Ok(())
}
