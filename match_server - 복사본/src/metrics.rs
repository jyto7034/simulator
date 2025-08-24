pub struct MetricsCtx;

impl MetricsCtx {
    pub fn new() -> Self {
        Self
    }

    // domain helpers (no run_id labels)
    pub fn inc_enqueued_new(&self) {
        metrics::PLAYERS_ENQUEUED_NEW_TOTAL.inc();
    }
    pub fn inc_requeued_by(&self, by: u64) {
        metrics::PLAYERS_REQUEUED_TOTAL.inc_by(by);
    }
    pub fn inc_allocated_success_by(&self, by: u64) {
        metrics::PLAYERS_ALLOCATED_TOTAL.inc_by(by);
    }
    pub fn observe_wait_secs(&self, secs: f64) {
        metrics::MATCH_WAIT_DURATION_SECONDS.observe(secs);
    }

    // labeled helpers
    pub fn set_queue_size_for(&self, game_mode: &str, size: i64) {
        metrics::PLAYERS_IN_QUEUE_BY_MODE
            .with_label_values(&[game_mode])
            .set(size);
    }
    pub fn inc_enqueued_by_mode(&self, game_mode: &str) {
        metrics::ENQUEUED_TOTAL_BY_MODE
            .with_label_values(&[game_mode])
            .inc();
    }
    pub fn inc_matched_players_by_mode(&self, game_mode: &str, by: u64) {
        metrics::MATCHED_PLAYERS_TOTAL_BY_MODE
            .with_label_values(&[game_mode])
            .inc_by(by);
    }
    pub fn inc_loading_completed_by_mode(&self, game_mode: &str, by: u64) {
        metrics::LOADING_COMPLETED_TOTAL_BY_MODE
            .with_label_values(&[game_mode])
            .inc_by(by);
    }
    pub fn inc_dedicated_success_by_mode(&self, game_mode: &str, by: u64) {
        metrics::DEDICATED_ALLOCATION_SUCCESS_TOTAL_BY_MODE
            .with_label_values(&[game_mode])
            .inc_by(by);
    }
    pub fn observe_match_time_secs_by_mode(&self, game_mode: &str, secs: f64) {
        metrics::MATCH_TIME_SECONDS
            .with_label_values(&[game_mode])
            .observe(secs);
    }
    pub fn observe_loading_duration_secs_by_mode(&self, game_mode: &str, secs: f64) {
        metrics::LOADING_DURATION_SECONDS
            .with_label_values(&[game_mode])
            .observe(secs);
    }

    pub fn inc_state_violation(&self) {
        metrics::STATE_VIOLATIONS_TOTAL.inc();
    }
    pub fn inc_http_timeout_error(&self) {
        metrics::HTTP_TIMEOUT_ERRORS_TOTAL.inc();
    }
    pub fn inc_redis_connection_failure(&self) {
        metrics::REDIS_CONNECTION_FAILURES_TOTAL.inc();
    }
    pub fn inc_matchmaking_error(&self) {
        metrics::MATCHMAKING_ERRORS_TOTAL.inc();
    }

    pub fn inc_timeout_players_by(&self, by: u64) {
        metrics::LOADING_SESSION_TIMEOUT_PLAYERS_TOTAL.inc_by(by);
    }

    pub fn abnormal_unknown_type(&self) {
        metrics::ABNORMAL_UNKNOWN_TYPE_TOTAL.inc();
    }
    pub fn abnormal_missing_field(&self) {
        metrics::ABNORMAL_MISSING_FIELD_TOTAL.inc();
    }
    pub fn abnormal_duplicate_enqueue(&self) {
        metrics::ABNORMAL_DUPLICATE_ENQUEUE_TOTAL.inc();
    }
    pub fn abnormal_wrong_session_id(&self) {
        metrics::ABNORMAL_WRONG_SESSION_ID_TOTAL.inc();
    }
}
