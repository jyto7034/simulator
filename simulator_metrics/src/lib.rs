use lazy_static::lazy_static;
use prometheus::{
    opts, IntCounter,
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
    Ok(())
}
