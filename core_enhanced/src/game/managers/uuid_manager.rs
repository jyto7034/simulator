use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::determinism;

/// Deterministic UUID generator for the whole game run.
///
/// IMPORTANT: Use separate namespaces for unrelated streams so adding a new call site
/// doesn't shift every subsequent UUID in other systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UuidManager {
    run_seed: u64,
    counters: BTreeMap<u64, u64>,
}

impl UuidManager {
    pub const NS_OWNED_EQUIPMENT: u64 = 0x4f57_4e44_4551_5549; // "OWNDEQUI"
    pub const NS_EMPLOYEE: u64 = 0x454d_504c_4f59_4545; // "EMPLOYEE"
    pub const NS_VISUAL_NOVEL_SESSION: u64 = 0x5653_4e4f_5645_4c53; // "VSNOVELS"
    pub const NS_BATTLE_SESSION: u64 = 0x4241_5454_4c45_5345; // "BATTLESE"

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

    pub fn peek(&self, namespace: u64) -> Uuid {
        let index = self.counters.get(&namespace).copied().unwrap_or(0);
        determinism::uuid_v4_from_seed(self.run_seed, namespace, index)
    }

    /// Restore checkpoint-owned UUID streams while keeping explicitly global
    /// streams monotonic across that rewind.
    pub fn restore_checkpoint_preserving_counters(
        &mut self,
        checkpoint: Self,
        monotonic_namespaces: &[u64],
    ) {
        debug_assert_eq!(
            self.run_seed, checkpoint.run_seed,
            "a run checkpoint must belong to the current run seed"
        );
        let preserved = monotonic_namespaces
            .iter()
            .map(|namespace| {
                (
                    *namespace,
                    self.counters.get(namespace).copied().unwrap_or(0),
                )
            })
            .collect::<Vec<_>>();

        *self = checkpoint;
        for (namespace, current_counter) in preserved {
            let checkpoint_counter = self.counters.entry(namespace).or_insert(0);
            *checkpoint_counter = (*checkpoint_counter).max(current_counter);
        }
    }

    pub fn next_owned_equipment(&mut self) -> Uuid {
        self.next(Self::NS_OWNED_EQUIPMENT)
    }

    pub fn next_employee(&mut self) -> Uuid {
        self.next(Self::NS_EMPLOYEE)
    }

    pub fn next_visual_novel_session(&mut self) -> Uuid {
        self.next(Self::NS_VISUAL_NOVEL_SESSION)
    }

    pub fn next_battle_session(&mut self) -> Uuid {
        self.next(Self::NS_BATTLE_SESSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peek_returns_next_uuid_without_advancing_stream() {
        let mut manager = UuidManager::new(123);
        let peeked = manager.peek(UuidManager::NS_OWNED_EQUIPMENT);

        assert_eq!(manager.peek(UuidManager::NS_OWNED_EQUIPMENT), peeked);
        assert_eq!(manager.next_owned_equipment(), peeked);
        assert_ne!(manager.peek(UuidManager::NS_OWNED_EQUIPMENT), peeked);
    }

    #[test]
    fn checkpoint_restore_rewinds_local_stream_and_preserves_global_stream() {
        let mut manager = UuidManager::new(123);
        manager.next_owned_equipment();
        manager.next_visual_novel_session();
        let checkpoint = manager.clone();

        let rewound_equipment = manager.next_owned_equipment();
        let preserved_visual_novel = manager.next_visual_novel_session();
        manager.restore_checkpoint_preserving_counters(
            checkpoint,
            &[UuidManager::NS_VISUAL_NOVEL_SESSION],
        );

        assert_eq!(manager.next_owned_equipment(), rewound_equipment);
        assert_ne!(manager.next_visual_novel_session(), preserved_visual_novel);
    }
}
