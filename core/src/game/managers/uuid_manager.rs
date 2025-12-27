use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use uuid::Uuid;

use crate::game::determinism;

/// Deterministic UUID generator for the whole game run.
///
/// IMPORTANT: Use separate namespaces for unrelated streams so adding a new call site
/// doesn't shift every subsequent UUID in other systems.
#[derive(Resource, Debug)]
pub struct UuidManager {
    run_seed: u64,
    counters: BTreeMap<u64, u64>,
}

impl UuidManager {
    pub const NS_OWNED_EQUIPMENT: u64 = 0x4f57_4e44_4551_5549; // "OWNDEQUI"

    pub fn new(run_seed: u64) -> Self {
        Self {
            run_seed,
            counters: BTreeMap::new(),
        }
    }

    pub fn next(&mut self, namespace: u64) -> Uuid {
        let index = self.counters.entry(namespace).or_insert(0);
        let uuid = determinism::uuid_v4_from_seed(self.run_seed, namespace, *index);
        *index = index.wrapping_add(1);
        uuid
    }

    pub fn next_owned_equipment(&mut self) -> Uuid {
        self.next(Self::NS_OWNED_EQUIPMENT)
    }
}
