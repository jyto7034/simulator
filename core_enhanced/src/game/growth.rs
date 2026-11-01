use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GrowthId {
    KillStack,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrowthStack {
    pub stacks: HashMap<GrowthId, i32>,
}

impl crate::authoring::clone_shape::CloneShape for GrowthStack {
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        let Self { stacks } = self;
        // Opaque authoring applications originate at the production constructor:
        // new/fresh serde, add(entry), and complete clones/restores are the only
        // ingress/mutations. Raw external GrowthStack/core imports are excluded.
        // Keys and values are Copy; there are no deeply owned bucket children.
        // If a production remove/raw import is added, replace this proof with
        // retained allocation history before accepting that application path.
        let slots = crate::owned_collections::MapCloneHistory::from_tombstone_free_capacity(
            stacks.capacity(),
        )
        .bucket_bound()
        .and_then(|n| n.checked_add(1));
        meter.scan_storage(slots)
    }
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
