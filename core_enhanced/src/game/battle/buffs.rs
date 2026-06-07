use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use crate::game::battle::damage::DamageType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuffId(u64);

impl BuffId {
    pub fn from_name(name: &str) -> Self {
        const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET_BASIS;
        for byte in name.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        Self(hash)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

// Stun, Freeze 같은 하드 CC 는 한 가지 상태만 존재할 수 있음. (기존 CC 를 덮어씌움)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffKind {
    PeriodicDamage {
        damage_per_tick: u32,
        damage_type: DamageType,
    },
    Stun,
    Freeze,
    Silence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffReapplyPolicy {
    StackRefreshDurationKeepCadence,
    RefreshDuration,
}

#[derive(Debug, Clone)]
pub struct BuffDef {
    pub id: BuffId,
    pub name: String,
    pub kind: BuffKind,
    pub tick_interval_ms: u64,
    pub max_stacks: u8,
    pub reapply_policy: BuffReapplyPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffDatabase {
    pub buffs: Vec<BuffMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<BuffId, BuffDef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuffMetadata {
    pub name: String,
    pub kind: BuffKind,
    pub tick_interval_ms: u64,
    pub max_stacks: u8,
    pub reapply_policy: BuffReapplyPolicy,
}

impl BuffDatabase {
    pub fn new(buffs: Vec<BuffMetadata>) -> Self {
        let database = Self {
            buffs,
            by_id: OnceLock::new(),
        };
        database.validate_indexes();
        database
    }

    pub fn live_default() -> Self {
        ron::de::from_str(include_str!(
            "../../../../game_resources/data/buffs/base.ron"
        ))
        .expect("Failed to deserialize buffs/base.ron")
    }

    fn build_registry(&self) -> HashMap<BuffId, BuffDef> {
        let mut names = HashSet::new();
        self.buffs
            .iter()
            .map(|metadata| {
                assert!(
                    names.insert(metadata.name.clone()),
                    "duplicate buff name '{}'",
                    metadata.name
                );
                assert!(
                    metadata.max_stacks > 0,
                    "buff '{}' max_stacks must be greater than zero",
                    metadata.name
                );
                if matches!(metadata.kind, BuffKind::PeriodicDamage { .. }) {
                    assert!(
                        metadata.tick_interval_ms > 0,
                        "periodic damage buff '{}' requires tick_interval_ms",
                        metadata.name
                    );
                }
                let id = BuffId::from_name(&metadata.name);
                (
                    id,
                    BuffDef {
                        id,
                        name: metadata.name.clone(),
                        kind: metadata.kind,
                        tick_interval_ms: metadata.tick_interval_ms,
                        max_stacks: metadata.max_stacks,
                        reapply_policy: metadata.reapply_policy,
                    },
                )
            })
            .collect()
    }

    fn by_id(&self) -> &HashMap<BuffId, BuffDef> {
        self.by_id.get_or_init(|| self.build_registry())
    }

    pub fn get(&self, buff_id: BuffId) -> Option<&BuffDef> {
        self.by_id().get(&buff_id)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.get(BuffId::from_name(name)).is_some()
    }

    pub fn validate_indexes(&self) {
        let _ = self.by_id();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buff_id_is_deterministic_for_same_name() {
        assert_eq!(BuffId::from_name("poison"), BuffId::from_name("poison"));
        assert_ne!(BuffId::from_name("poison"), BuffId::from_name("stun"));
    }

    #[test]
    fn registry_contains_known_buffs_and_limits_hard_cc_to_single_stack() {
        let database = BuffDatabase::live_default();
        let poison = database.get(BuffId::from_name("poison")).unwrap();
        assert_eq!(poison.name, "poison");
        assert!(matches!(poison.kind, BuffKind::PeriodicDamage { .. }));
        assert_eq!(poison.max_stacks, 10);
        assert_eq!(poison.tick_interval_ms, 1000);
        assert_eq!(
            poison.reapply_policy,
            BuffReapplyPolicy::StackRefreshDurationKeepCadence
        );

        let stun = database.get(BuffId::from_name("stun")).unwrap();
        assert_eq!(stun.name, "stun");
        assert!(matches!(stun.kind, BuffKind::Stun));
        assert_eq!(stun.max_stacks, 1);

        let freeze = database.get(BuffId::from_name("freeze")).unwrap();
        assert_eq!(freeze.name, "freeze");
        assert!(matches!(freeze.kind, BuffKind::Freeze));
        assert_eq!(freeze.max_stacks, 1);

        let silence = database.get(BuffId::from_name("silence")).unwrap();
        assert_eq!(silence.name, "silence");
        assert!(matches!(silence.kind, BuffKind::Silence));
        assert_eq!(silence.max_stacks, 1);

        assert!(database.get(BuffId::from_name("unknown_buff")).is_none());
    }
}
