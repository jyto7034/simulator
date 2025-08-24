use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::Path;

use super::concrete::ConcreteConfig;

#[derive(Debug, Serialize)]
pub struct RunManifest {
    pub seed: u64,
    pub config: ConcreteConfig,
}

pub fn save_manifest(path: &Path, manifest: &RunManifest) -> Result<()> {
    let payload = serde_json::to_vec_pretty(manifest)?;
    std::fs::write(path, payload)?;
    Ok(())
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct BehaviorOutcomeCounts {
    pub successful_matches: u64,
    pub loading_timeouts: u64,
    pub quit_before_match: u64,
    pub quit_during_loading: u64,
    pub connection_failures: u64,
    pub invalid_requests: u64,
    pub other_failures: u64,
}

#[derive(Debug, Serialize)]
pub struct SwarmRunSummary<CfgT: Serialize> {
    pub timestamp: DateTime<Utc>,
    pub seed: u64,

    pub config: CfgT,
    pub metrics_url: String,
    pub slo: crate::swarm::slo::SloReport,
    pub outcome_counts: BehaviorOutcomeCounts,
    /// Snapshot of players remaining in queues at run end
    pub still_queued_at_end: RemainingQueueSummary,
}

pub fn save_swarm_summary<CfgT: Serialize>(
    path: &Path,
    summary: &SwarmRunSummary<CfgT>,
) -> Result<()> {
    let payload = serde_json::to_vec_pretty(summary)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    std::fs::write(path, payload)?;
    Ok(())
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct RemainingQueueSummary {
    pub total: u64,
    pub by_mode: std::collections::HashMap<String, u64>,
}
