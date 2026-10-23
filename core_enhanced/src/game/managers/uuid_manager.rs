use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::determinism;

/// Deterministic UUID generator for the whole game run.
///
/// IMPORTANT: Use separate namespaces for unrelated streams so adding a new call site
/// doesn't shift every subsequent UUID in other systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UuidManager {
    run_seed: u64,
    instance_namespace: Uuid,
    counters: BTreeMap<u64, u64>,
}

crate::authoring::clone_shape::struct_shape!(UuidManager { run_seed, instance_namespace, counters });

impl UuidManager {
    pub const NS_OWNED_EQUIPMENT: u64 = 0x4f57_4e44_4551_5549; // "OWNDEQUI"
    pub const NS_OWNED_FRAGMENT: u64 = 0x4652_4147_434f_5059; // "FRAGCOPY"
    pub const NS_OWNED_RELIC: u64 = 0x5245_4c49_4343_4f50; // "RELICCOP"
    pub const NS_EMPLOYEE: u64 = 0x454d_504c_4f59_4545; // "EMPLOYEE"
    pub const NS_VISUAL_NOVEL_SESSION: u64 = 0x5653_4e4f_5645_4c53; // "VSNOVELS"
    pub const NS_BATTLE_SESSION: u64 = 0x4241_5454_4c45_5345; // "BATTLESE"
    pub const NS_MASTERED_ELITE_SWEEP_RESULT: u64 = 0x454c_4954_5357_4550; // "ELITSWEP"

    pub fn new(run_seed: u64) -> Self {
        Self {
            run_seed,
            instance_namespace: Uuid::nil(),
            counters: BTreeMap::new(),
        }
    }

    pub fn for_instance(run_seed: u64, instance_namespace: Uuid) -> Self {
        Self { run_seed, instance_namespace, counters: BTreeMap::new() }
    }

    pub fn instance_namespace(&self) -> Uuid {
        self.instance_namespace
    }

    fn generated_uuid(&self, namespace: u64, index: u64) -> Uuid {
        if self.instance_namespace.is_nil() {
            return determinism::uuid_v4_from_seed(self.run_seed, namespace, index);
        }
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        hash.update(b"core.instance.uuid.v1\0");
        hash.update(self.instance_namespace.as_bytes());
        hash.update(self.run_seed.to_le_bytes());
        hash.update(namespace.to_le_bytes());
        hash.update(index.to_le_bytes());
        let mut bytes = [0; 16];
        bytes.copy_from_slice(&hash.finalize()[..16]);
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Uuid::from_bytes(bytes)
    }

    pub(crate) fn run_seed(&self) -> u64 {
        self.run_seed
    }

    pub fn next(&mut self, namespace: u64) -> Uuid {
        let index = self.counters.get(&namespace).copied().unwrap_or(0);
        let uuid = self.generated_uuid(namespace, index);
        self.counters.insert(namespace, index.wrapping_add(1));
        uuid
    }

    pub fn peek(&self, namespace: u64) -> Uuid {
        let index = self.counters.get(&namespace).copied().unwrap_or(0);
        self.generated_uuid(namespace, index)
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
        debug_assert_eq!(self.instance_namespace, checkpoint.instance_namespace,
            "a run checkpoint must belong to the same instance");
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
            // A missing stream already denotes index zero. Only an actual
            // advance may add it, preserving canonical bytes on fresh restore.
            if current_counter > self.counters.get(&namespace).copied().unwrap_or(0) {
                self.counters.insert(namespace, current_counter);
            }
        }
    }

    pub fn next_owned_equipment(&mut self) -> Uuid {
        self.next(Self::NS_OWNED_EQUIPMENT)
    }

    pub(crate) fn fragment_counter(&self) -> u64 {
        self.counters
            .get(&Self::NS_OWNED_FRAGMENT)
            .copied()
            .unwrap_or(0)
    }

    pub fn next_owned_fragment(&mut self) -> Result<Uuid, crate::game::behavior::GameError> {
        let index = self
            .counters
            .get(&Self::NS_OWNED_FRAGMENT)
            .copied()
            .unwrap_or(0);
        let next = index
            .checked_add(1)
            .ok_or(crate::game::behavior::GameError::InvalidAction)?;
        let uuid = self.generated_uuid(Self::NS_OWNED_FRAGMENT, index);
        self.counters.insert(Self::NS_OWNED_FRAGMENT, next);
        Ok(uuid)
    }

    pub fn next_employee(&mut self) -> Uuid {
        self.next(Self::NS_EMPLOYEE)
    }

    pub(crate) fn has_relic_stream(&self) -> bool {
        self.counters.contains_key(&Self::NS_OWNED_RELIC)
    }
    fn relic_counter_table_extent(len: usize) -> Option<usize> {
        if len == 0 {
            return Some(0);
        }
        // Once RELICCOP is present, its UUID owner is the actual shared counter
        // table, not an imaginary independent node. Non-root BTree nodes have
        // at least five keys; include a root and full eleven-slot nodes.
        let nodes = 1_usize.checked_add((len - 1) / 5)?;
        nodes.checked_mul(std::mem::size_of::<(u64, u64)>() * 11 + 224)
    }
    pub(crate) fn relic_stream_extent(&self) -> Result<usize, crate::game::behavior::GameError> {
        if !self.has_relic_stream() {
            return Ok(0);
        }
        Self::relic_counter_table_extent(self.counters.len()).ok_or_else(|| {
            crate::game::battle::runtime_control::reject(
                crate::game::battle::runtime_control::BattleRuntimeFault {
                    code: crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
                    site: "relic_uuid_extent",
                    limit: None,
                    requested: None,
                },
            )
        })
    }
    pub(crate) fn preflight_relic_stream_clone(
        &self,
    ) -> Result<(), crate::game::behavior::GameError> {
        let bytes = self.relic_stream_extent()?;
        if bytes != 0 {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(bytes),
                "relic_uuid_stream_clone",
            )?;
            if crate::game::battle::runtime_control::is_active()
                && !crate::authoring::control::is_active()
            {
                crate::game::resource::store::preflight_resource_clone(&self.counters)
                    .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
            }
        }
        Ok(())
    }
    pub(crate) fn next_owned_relic(&mut self) -> Result<Uuid, crate::game::behavior::GameError> {
        let index = self
            .counters
            .get(&Self::NS_OWNED_RELIC)
            .copied()
            .unwrap_or(0);
        let next = index
            .checked_add(1)
            .ok_or(crate::game::behavior::GameError::InvalidAction)?;
        if !self.counters.contains_key(&Self::NS_OWNED_RELIC) {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                self.counters
                    .len()
                    .checked_add(1)
                    .and_then(Self::relic_counter_table_extent),
                "relic_uuid_stream",
            )?;
        }
        let uuid = self.generated_uuid(Self::NS_OWNED_RELIC, index);
        self.counters.insert(Self::NS_OWNED_RELIC, next);
        Ok(uuid)
    }

    pub fn next_visual_novel_session(&mut self) -> Uuid {
        self.next(Self::NS_VISUAL_NOVEL_SESSION)
    }

    pub fn next_battle_session(&mut self) -> Uuid {
        self.next(Self::NS_BATTLE_SESSION)
    }

    pub fn next_mastered_elite_sweep_result(&mut self) -> Uuid {
        self.next(Self::NS_MASTERED_ELITE_SWEEP_RESULT)
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
