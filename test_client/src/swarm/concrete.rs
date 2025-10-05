use serde::Serialize;

use super::behavior_mix::BehaviorMixConfig;

#[derive(Debug, Clone, Serialize)]
pub struct ConcreteConfig {
    pub duration_secs: u64,
    pub player_count: u64,
    pub cps: f64,
    pub behavior_mix: BehaviorMixConfig,
}
