use metrics::{BATTLE_RESULTS_LOCAL_TOTAL, BATTLE_RESULTS_REMOTE_TOTAL};
use prometheus::IntCounter;

pub struct MetricsCtx {
    pub battle_results_local_total: IntCounter,
    pub battle_results_remote_total: IntCounter,
}

impl MetricsCtx {
    pub fn new() -> Self {
        Self {
            battle_results_local_total: BATTLE_RESULTS_LOCAL_TOTAL.clone(),
            battle_results_remote_total: BATTLE_RESULTS_REMOTE_TOTAL.clone(),
        }
    }

    pub fn inc_redis_connection_failure(&self) {
        metrics::REDIS_CONNECTION_FAILURES_TOTAL.inc();
    }
}
