use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GrowthId {
    KillStack,
    PveWinStack,
    QuestRewardStack,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrowthStack {
    pub stacks: HashMap<GrowthId, i32>,
}

impl GrowthStack {
    pub fn new() -> Self {
        Self {
            stacks: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: GrowthId, delta: i32) {
        if delta == 0 {
            return;
        }
        *self.stacks.entry(id).or_insert(0) += delta;
    }
}
