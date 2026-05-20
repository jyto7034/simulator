use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffKind {
    PeriodicDamage {
        damage_per_tick: u32,
        damage_type: DamageType,
    },
    Stun,
    Freeze,
    Silence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffReapplyPolicy {
    StackRefreshDurationKeepCadence,
    RefreshDuration,
}

#[derive(Debug, Clone, Copy)]
pub struct BuffDef {
    pub id: BuffId,
    pub name: &'static str,
    pub kind: BuffKind,
    pub tick_interval_ms: u64,
    pub max_stacks: u8,
    pub reapply_policy: BuffReapplyPolicy,
}

static REGISTRY: Lazy<HashMap<BuffId, BuffDef>> = Lazy::new(|| {
    let poison = BuffDef {
        id: BuffId::from_name("poison"),
        name: "poison",
        kind: BuffKind::PeriodicDamage {
            damage_per_tick: 2,
            damage_type: DamageType::Magic,
        },
        tick_interval_ms: 1000,
        max_stacks: 10,
        reapply_policy: BuffReapplyPolicy::StackRefreshDurationKeepCadence,
    };

    let stun = BuffDef {
        id: BuffId::from_name("stun"),
        name: "stun",
        kind: BuffKind::Stun,
        tick_interval_ms: 0,
        max_stacks: 1,
        reapply_policy: BuffReapplyPolicy::RefreshDuration,
    };

    let freeze = BuffDef {
        id: BuffId::from_name("freeze"),
        name: "freeze",
        kind: BuffKind::Freeze,
        tick_interval_ms: 0,
        max_stacks: 1,
        reapply_policy: BuffReapplyPolicy::RefreshDuration,
    };

    let silence = BuffDef {
        id: BuffId::from_name("silence"),
        name: "silence",
        kind: BuffKind::Silence,
        tick_interval_ms: 0,
        max_stacks: 1,
        reapply_policy: BuffReapplyPolicy::RefreshDuration,
    };

    [poison, stun, freeze, silence]
        .into_iter()
        .map(|def| (def.id, def))
        .collect()
});

pub fn get(buff_id: BuffId) -> Option<&'static BuffDef> {
    REGISTRY.get(&buff_id)
}

pub fn contains_name(name: &str) -> bool {
    get(BuffId::from_name(name)).is_some()
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
        let poison = get(BuffId::from_name("poison")).unwrap();
        assert_eq!(poison.name, "poison");
        assert!(matches!(poison.kind, BuffKind::PeriodicDamage { .. }));
        assert_eq!(poison.max_stacks, 10);
        assert_eq!(poison.tick_interval_ms, 1000);
        assert_eq!(
            poison.reapply_policy,
            BuffReapplyPolicy::StackRefreshDurationKeepCadence
        );

        let stun = get(BuffId::from_name("stun")).unwrap();
        assert_eq!(stun.name, "stun");
        assert!(matches!(stun.kind, BuffKind::Stun));
        assert_eq!(stun.max_stacks, 1);

        let freeze = get(BuffId::from_name("freeze")).unwrap();
        assert_eq!(freeze.name, "freeze");
        assert!(matches!(freeze.kind, BuffKind::Freeze));
        assert_eq!(freeze.max_stacks, 1);

        let silence = get(BuffId::from_name("silence")).unwrap();
        assert_eq!(silence.name, "silence");
        assert!(matches!(silence.kind, BuffKind::Silence));
        assert_eq!(silence.max_stacks, 1);

        assert!(get(BuffId::from_name("unknown_buff")).is_none());
    }
}
