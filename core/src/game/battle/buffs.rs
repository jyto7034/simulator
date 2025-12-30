use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    PeriodicDamage { damage_per_tick: u32 },
    Stun,
    Freeze,
    Silence,
}

#[derive(Debug, Clone, Copy)]
pub struct BuffDef {
    pub id: BuffId,
    pub name: &'static str,
    pub kind: BuffKind,
    pub tick_interval_ms: u64,
    pub max_stacks: u8,
}

static REGISTRY: Lazy<HashMap<BuffId, BuffDef>> = Lazy::new(|| {
    let poison = BuffDef {
        id: BuffId::from_name("poison"),
        name: "poison",
        kind: BuffKind::PeriodicDamage { damage_per_tick: 2 },
        tick_interval_ms: 1000,
        max_stacks: 10,
    };

    let stun = BuffDef {
        id: BuffId::from_name("stun"),
        name: "stun",
        kind: BuffKind::Stun,
        tick_interval_ms: 0,
        max_stacks: 1,
    };

    let freeze = BuffDef {
        id: BuffId::from_name("freeze"),
        name: "freeze",
        kind: BuffKind::Freeze,
        tick_interval_ms: 0,
        max_stacks: 1,
    };

    [poison, stun, freeze]
        .into_iter()
        .map(|def| (def.id, def))
        .collect()
});

pub fn get(buff_id: BuffId) -> Option<&'static BuffDef> {
    REGISTRY.get(&buff_id)
}
