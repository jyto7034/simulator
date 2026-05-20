use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::DeliveryDef,
    data::{build_string_index, build_uuid_index, once_lock_with},
    enums::RiskLevel,
};

fn default_resonance_start() -> u32 {
    0
}

fn default_resonance_max() -> u32 {
    100
}

fn default_resonance_lock_ms() -> u64 {
    1000
}

fn default_attack_range_tiles() -> u8 {
    1
}

pub const DEFAULT_INSTANT_BASIC_ATTACK_WINDUP_MS: u32 = 200;

fn default_attack_windup_ms() -> u32 {
    DEFAULT_INSTANT_BASIC_ATTACK_WINDUP_MS
}

fn default_basic_attack_interval_ms() -> u64 {
    1500
}

fn default_magic_resist() -> i32 {
    0
}

fn default_move_speed_units_per_ms() -> u32 {
    // 1 tile = 1_000_000 units, so 3 tiles/s ~= 3000 units/ms
    3000
}

fn default_attack_delivery() -> DeliveryDef {
    DeliveryDef::Instant
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementDef {
    #[serde(default = "default_move_speed_units_per_ms")]
    pub speed_units_per_ms: u32,
}

impl Default for MovementDef {
    fn default() -> Self {
        Self {
            speed_units_per_ms: default_move_speed_units_per_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAttackDef {
    #[serde(default = "default_attack_range_tiles")]
    pub range_tiles: u8,
    #[serde(default = "default_basic_attack_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_attack_windup_ms")]
    pub windup_ms: u32,
    #[serde(default = "default_attack_delivery")]
    pub delivery: DeliveryDef,
}

impl Default for BasicAttackDef {
    fn default() -> Self {
        Self {
            range_tiles: default_attack_range_tiles(),
            interval_ms: default_basic_attack_interval_ms(),
            windup_ms: default_attack_windup_ms(),
            delivery: default_attack_delivery(),
        }
    }
}

impl BasicAttackDef {
    pub fn effective_windup_ms(&self) -> u32 {
        match self.delivery {
            DeliveryDef::Instant if self.windup_ms == 0 => DEFAULT_INSTANT_BASIC_ATTACK_WINDUP_MS,
            _ => self.windup_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonanceDef {
    #[serde(default = "default_resonance_start")]
    pub start: u32,
    #[serde(default = "default_resonance_max")]
    pub max: u32,
    #[serde(default = "default_resonance_lock_ms")]
    pub gain_lock_ms: u64,
}

impl Default for ResonanceDef {
    fn default() -> Self {
        Self {
            start: default_resonance_start(),
            max: default_resonance_max(),
            gain_lock_ms: default_resonance_lock_ms(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbnormalityMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub risk_level: RiskLevel,
    pub price: u32,
    /// 전투용 기본 최대 체력
    pub max_health: u32,
    /// 전투용 기본 공격력
    pub attack: u32,
    /// 전투용 기본 방어력. 음수면 받는 물리 피해가 증가한다.
    pub defense: i32,
    /// 전투용 기본 마법 저항력. 음수면 받는 마법 피해가 증가한다.
    #[serde(default = "default_magic_resist")]
    pub magic_resist: i32,

    /// 이동 스펙
    #[serde(default)]
    pub movement: MovementDef,

    /// 기본 공격 스펙
    #[serde(default)]
    pub basic_attack: BasicAttackDef,

    /// 공명(=마나)
    #[serde(default)]
    pub resonance: ResonanceDef,

    /// 이 기물이 보유한 스킬(유닛당 1개)
    #[serde(default)]
    pub skill_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbnormalityDatabase {
    pub items: Vec<AbnormalityMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl AbnormalityDatabase {
    pub fn new(items: Vec<AbnormalityMetadata>) -> Self {
        let by_id = once_lock_with(build_string_index(&items, "abnormality id", |item| {
            &item.id
        }));
        let by_uuid = once_lock_with(build_uuid_index(&items, "abnormality uuid", |item| {
            item.uuid
        }));

        Self {
            items,
            by_id,
            by_uuid,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.items, "abnormality id", |item| &item.id))
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid
            .get_or_init(|| build_uuid_index(&self.items, "abnormality uuid", |item| item.uuid))
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_uuid();
    }

    pub fn get_by_id(&self, id: &str) -> Option<&AbnormalityMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.items.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&AbnormalityMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.items.get(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::enums::RiskLevel;

    fn abnormality(id: &str, uuid: Uuid) -> AbnormalityMetadata {
        AbnormalityMetadata {
            id: id.to_string(),
            uuid,
            name: id.to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 10,
            max_health: 10,
            attack: 1,
            defense: 1,
            magic_resist: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        }
    }

    #[test]
    #[should_panic(expected = "duplicate abnormality id 'dup'")]
    fn new_panics_on_duplicate_ids() {
        let _ = AbnormalityDatabase::new(vec![
            abnormality("dup", Uuid::from_u128(1)),
            abnormality("dup", Uuid::from_u128(2)),
        ]);
    }
}
