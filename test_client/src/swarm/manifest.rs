use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use chrono::{DateTime, Utc};

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

#[derive(Debug, Serialize)]
pub struct SwarmRunSummary<CfgT: Serialize> {
    pub timestamp: DateTime<Utc>,
    pub seed: u64,
    pub run_id: String,
    pub config: CfgT,
    pub metrics_url: String,
    pub slo: crate::swarm::slo::SloReport,
}

pub fn save_swarm_summary<CfgT: Serialize>(path: &Path, summary: &SwarmRunSummary<CfgT>) -> Result<()> {
    let payload = serde_json::to_vec_pretty(summary)?;
    if let Some(dir) = path.parent() { std::fs::create_dir_all(dir).ok(); }
    std::fs::write(path, payload)?;
    Ok(())
}
