use super::behavior_mix::BehaviorMixConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwarmConfig {
    pub duration_secs: u32,
    pub shards: usize,
    pub players_per_shard: usize,
    pub game_mode: Option<String>,
    /// Base URL like ws://127.0.0.1:8080 (Observer builds /events/stream)
    pub match_server_base: Option<String>,
    /// Deterministic seed for the run (overrides SWARM_SEED env)
    pub seed: Option<u64>,
    /// Inline behavior mix configuration (concrete values)
    pub behavior_mix: Option<BehaviorMixConfig>,
    /// Optional result summary output path (JSON). If not set, defaults to logs/swarm_summary_<ts>.json
    pub result_path: Option<String>,
    /// Ratio of players per shard to start simultaneously (0.0..=1.0). None or 0.0 disables burst.
    pub burst_ratio: Option<f64>,
}

impl SwarmConfig {
    pub fn from_toml_str(s: &str) -> anyhow::Result<Self> {
        let cfg: SwarmConfig = toml::from_str(s)?;
        Ok(cfg)
    }

    pub fn events_base_url(&self) -> Option<String> {
        self.match_server_base.clone()
    }
}
