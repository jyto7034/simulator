pub struct MetricsCtx;

impl MetricsCtx {
    pub fn new() -> Self {
        Self
    }
    pub fn inc_redis_connection_failure(&self) {
        metrics::REDIS_CONNECTION_FAILURES_TOTAL.inc();
    }
}
