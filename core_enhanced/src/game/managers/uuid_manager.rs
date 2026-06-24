use std::collections::BTreeMap;

use uuid::Uuid;

use crate::game::determinism;

/// Deterministic UUID generator for the whole game run.
///
/// IMPORTANT: Use separate namespaces for unrelated streams so adding a new call site
/// doesn't shift every subsequent UUID in other systems.
#[derive(Debug, Clone)]
pub struct UuidManager {
    run_seed: u64,
    counters: BTreeMap<u64, u64>,
}

impl UuidManager {
    pub const NS_OWNED_EQUIPMENT: u64 = 0x4f57_4e44_4551_5549; // "OWNDEQUI"
    pub const NS_OWNED_CONSUMABLE: u64 = 0x4f57_4e44_434f_4e53; // "OWNDCONS"
    pub const NS_EMPLOYEE: u64 = 0x454d_504c_4f59_4545; // "EMPLOYEE"

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

    pub fn next_owned_equipment(&mut self) -> Uuid {
        self.next(Self::NS_OWNED_EQUIPMENT)
    }

    pub fn next_owned_consumable(&mut self) -> Uuid {
        self.next(Self::NS_OWNED_CONSUMABLE)
    }

    pub fn next_employee(&mut self) -> Uuid {
        self.next(Self::NS_EMPLOYEE)
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
}
