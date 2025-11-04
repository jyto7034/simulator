use lazy_static::lazy_static;
use prometheus::{
    opts, Counter, Histogram, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge,
    IntGaugeVec, Opts, Registry,
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

    /// Active websocket connections.
    pub static ref ACTIVE_WS_CONNECTIONS: IntGauge =
        IntGauge::with_opts(opts!("active_ws_connections", "Number of active websocket connections")).unwrap();

    /// Total number of players newly enqueued (excluding requeues)
    pub static ref PLAYERS_ENQUEUED_NEW_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("players_enqueued_new_total", "Total players newly enqueued (excluding requeues)")).unwrap();

    /// Total number of players re-enqueued (requeues only)
    pub static ref PLAYERS_REQUEUED_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("players_requeued_total", "Total players re-enqueued (requeues only)")).unwrap();

    /// Total number of players who reached dedicated allocation (counted per player)
    pub static ref PLAYERS_ALLOCATED_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("players_allocated_total", "Total players who reached dedicated allocation")).unwrap();

    /// Histogram of queue wait time from initial enqueue to TryMatch collection (seconds)
    pub static ref MATCH_WAIT_DURATION_SECONDS: Histogram =
        Histogram::with_opts(HistogramOpts::new(
            "match_wait_duration_seconds",
            "Time players spent waiting in queue until matched (seconds)"
        ).buckets(vec![0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 45.0, 60.0, 90.0, 120.0])).unwrap();

    /// Abnormal behavior counters
    pub static ref ABNORMAL_UNKNOWN_TYPE_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("abnormal_unknown_type_total", "Unknown message type received")).unwrap();
    pub static ref ABNORMAL_MISSING_FIELD_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("abnormal_missing_field_total", "Client message missing required field")).unwrap();

    pub static ref ABNORMAL_DUPLICATE_ENQUEUE_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("abnormal_duplicate_enqueue_total", "Duplicate Enqueue attempts")).unwrap();
    pub static ref ABNORMAL_WRONG_SESSION_ID_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("abnormal_wrong_session_id_total", "LoadingComplete with wrong session id")).unwrap();

    // By-mode counters/gauges for richer SLO and dashboards
    pub static ref ENQUEUED_TOTAL_BY_MODE: IntCounterVec =
        IntCounterVec::new(Opts::new("enqueued_total_by_mode", "Total enqueued players by game mode"), &[
            "game_mode",
        ])
        .unwrap();
    pub static ref MATCHED_PLAYERS_TOTAL_BY_MODE: IntCounterVec =
        IntCounterVec::new(
            Opts::new(
                "matched_players_total_by_mode",
                "Total matched players collected by mode (at match time)",
            ),
            &["game_mode"],
        )
        .unwrap();
    pub static ref LOADING_COMPLETED_TOTAL_BY_MODE: IntCounterVec =
        IntCounterVec::new(
            Opts::new(
                "loading_completed_total_by_mode",
                "Total players whose loading completed (all ready) by mode",
            ),
            &["game_mode"],
        )
        .unwrap();
    pub static ref DEDICATED_ALLOCATION_SUCCESS_TOTAL_BY_MODE: IntCounterVec =
        IntCounterVec::new(
            Opts::new(
                "dedicated_allocation_success_total_by_mode",
                "Total players successfully allocated to dedicated session by mode",
            ),
            &["game_mode"],
        )
        .unwrap();

    pub static ref PLAYERS_IN_QUEUE_BY_MODE: IntGaugeVec = IntGaugeVec::new(
        Opts::new(
            "players_in_queue_by_mode",
            "Current number of players in queue by mode",
        ),
        &["game_mode"],
    )
    .unwrap();

    // Histograms for SLO
    pub static ref MATCH_TIME_SECONDS: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "match_time_seconds",
            "Queue wait time from enqueue to match (seconds)",
        )
        .buckets(vec![
            0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 45.0, 60.0, 90.0, 120.0,
        ]),
        &["game_mode"],
    )
    .unwrap();

    pub static ref LOADING_DURATION_SECONDS: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "loading_duration_seconds",
            "Time from loading_session_created to loading all-ready (seconds)",
        )
        .buckets(vec![
            0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 20.0, 30.0, 45.0, 60.0, 90.0,
        ]),
        &["game_mode"],
    )
    .unwrap();

    // System/violation/error counters
    pub static ref STATE_VIOLATIONS_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("state_violations_total", "Total state violations observed")).unwrap();
    pub static ref HTTP_TIMEOUT_ERRORS_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("http_timeout_errors_total", "HTTP timeout errors contacting dependencies")).unwrap();
    pub static ref REDIS_CONNECTION_FAILURES_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("redis_connection_failures_total", "Redis connection failures")).unwrap();
    pub static ref MATCHMAKING_ERRORS_TOTAL: IntCounter =
        IntCounter::with_opts(opts!("matchmaking_errors_total", "Total matchmaking error messages emitted to clients")).unwrap();

    /// Total number of players who timed out during loading sessions
    pub static ref LOADING_SESSION_TIMEOUT_PLAYERS_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "loading_session_timeout_players_total",
            "Total players who timed out during loading sessions"
        ))
        .unwrap();

    /// Total number of poisoned candidates removed from queue
    pub static ref POISONED_CANDIDATES_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "poisoned_candidates_total",
            "Total number of poisoned candidates removed from queue (invalid metadata)"
        ))
        .unwrap();

    /// Total times no game server was available (subscriber_count == 0)
    pub static ref GAME_SERVER_UNAVAILABLE_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "game_server_unavailable_total",
            "Total times no game server was subscribed to battle:request channel"
        ))
        .unwrap();

    /// Current game server availability status
    pub static ref GAME_SERVER_AVAILABLE: IntGauge =
        IntGauge::with_opts(opts!(
            "game_server_available",
            "Game server availability (1=available, 0=unavailable)"
        ))
        .unwrap();

    /// Total times TryMatch was skipped due to already running
    pub static ref TRY_MATCH_SKIPPED_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "try_match_skipped_total",
            "Total times TryMatch was skipped due to in-flight limit"
        ))
        .unwrap();

    /// Total times circuit breaker opened
    pub static ref CIRCUIT_BREAKER_OPEN_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "circuit_breaker_open_total",
            "Total times Redis circuit breaker opened due to failures"
        ))
        .unwrap();

    /// Total number of matches where both players are in the same pod
    pub static ref MATCHES_SAME_POD_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "matches_same_pod_total",
            "Total number of matches where both players are in the same pod"
        ))
        .unwrap();

    /// Total number of matches across different pods
    pub static ref MATCHES_CROSS_POD_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "matches_cross_pod_total",
            "Total number of matches across different pods"
        ))
        .unwrap();

    /// Total number of battle results sent via local actor message
    pub static ref BATTLE_RESULTS_LOCAL_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "battle_results_local_total",
            "Total number of battle results sent via local actor message (same pod)"
        ))
        .unwrap();

    /// Total number of battle results sent via Redis Pub/Sub
    pub static ref BATTLE_RESULTS_REMOTE_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "battle_results_remote_total",
            "Total number of battle results sent via Redis Pub/Sub (cross pod)"
        ))
        .unwrap();

    /// Total number of messages routed to same-pod players via LoadBalanceActor
    pub static ref MESSAGES_ROUTED_SAME_POD_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "messages_routed_same_pod_total",
            "Total messages routed to same-pod players via LoadBalanceActor"
        ))
        .unwrap();

    /// Total number of messages routed to cross-pod players via Redis Pub/Sub
    pub static ref MESSAGES_ROUTED_CROSS_POD_TOTAL: IntCounter =
        IntCounter::with_opts(opts!(
            "messages_routed_cross_pod_total",
            "Total messages routed to cross-pod players via Redis Pub/Sub"
        ))
        .unwrap();
}

// All test and per-mode metrics removed

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
    registry.register(Box::new(ACTIVE_WS_CONNECTIONS.clone()))?;
    registry.register(Box::new(PLAYERS_ENQUEUED_NEW_TOTAL.clone()))?;
    registry.register(Box::new(PLAYERS_REQUEUED_TOTAL.clone()))?;
    registry.register(Box::new(PLAYERS_ALLOCATED_TOTAL.clone()))?;
    registry.register(Box::new(MATCH_WAIT_DURATION_SECONDS.clone()))?;
    registry.register(Box::new(ABNORMAL_UNKNOWN_TYPE_TOTAL.clone()))?;
    registry.register(Box::new(ABNORMAL_MISSING_FIELD_TOTAL.clone()))?;

    registry.register(Box::new(ABNORMAL_DUPLICATE_ENQUEUE_TOTAL.clone()))?;
    registry.register(Box::new(ABNORMAL_WRONG_SESSION_ID_TOTAL.clone()))?;

    // New registrations
    registry.register(Box::new(ENQUEUED_TOTAL_BY_MODE.clone()))?;
    registry.register(Box::new(MATCHED_PLAYERS_TOTAL_BY_MODE.clone()))?;
    registry.register(Box::new(LOADING_COMPLETED_TOTAL_BY_MODE.clone()))?;
    registry.register(Box::new(DEDICATED_ALLOCATION_SUCCESS_TOTAL_BY_MODE.clone()))?;
    registry.register(Box::new(PLAYERS_IN_QUEUE_BY_MODE.clone()))?;
    registry.register(Box::new(MATCH_TIME_SECONDS.clone()))?;
    registry.register(Box::new(LOADING_DURATION_SECONDS.clone()))?;
    registry.register(Box::new(STATE_VIOLATIONS_TOTAL.clone()))?;
    registry.register(Box::new(HTTP_TIMEOUT_ERRORS_TOTAL.clone()))?;
    registry.register(Box::new(REDIS_CONNECTION_FAILURES_TOTAL.clone()))?;
    registry.register(Box::new(MATCHMAKING_ERRORS_TOTAL.clone()))?;
    registry.register(Box::new(LOADING_SESSION_TIMEOUT_PLAYERS_TOTAL.clone()))?;

    // Safety improvements metrics
    registry.register(Box::new(POISONED_CANDIDATES_TOTAL.clone()))?;
    registry.register(Box::new(GAME_SERVER_UNAVAILABLE_TOTAL.clone()))?;
    registry.register(Box::new(GAME_SERVER_AVAILABLE.clone()))?;
    registry.register(Box::new(TRY_MATCH_SKIPPED_TOTAL.clone()))?;
    registry.register(Box::new(CIRCUIT_BREAKER_OPEN_TOTAL.clone()))?;

    // Pod optimization metrics
    registry.register(Box::new(MATCHES_SAME_POD_TOTAL.clone()))?;
    registry.register(Box::new(MATCHES_CROSS_POD_TOTAL.clone()))?;
    registry.register(Box::new(BATTLE_RESULTS_LOCAL_TOTAL.clone()))?;
    registry.register(Box::new(BATTLE_RESULTS_REMOTE_TOTAL.clone()))?;
    registry.register(Box::new(MESSAGES_ROUTED_SAME_POD_TOTAL.clone()))?;
    registry.register(Box::new(MESSAGES_ROUTED_CROSS_POD_TOTAL.clone()))?;

    Ok(())
}
