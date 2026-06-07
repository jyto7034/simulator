use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::{DeliveryDef, SkillId},
    battle::{
        damage::DamageType,
        tile_range::TileRangePattern,
        types::{MobilityKind, UnitTargetTrait},
    },
    data::equipment_data::{TargetingProfile, WeaponRangeRole},
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

fn default_attack_range_units() -> f32 {
    1.0
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

fn default_body_radius_units() -> u32 {
    350_000
}

fn default_attack_delivery() -> DeliveryDef {
    DeliveryDef::Instant
}

fn default_basic_attack_damage_type() -> DamageType {
    DamageType::Physical
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementDef {
    #[serde(default = "default_move_speed_units_per_ms")]
    pub speed_units_per_ms: u32,
    #[serde(default = "default_body_radius_units")]
    pub radius_units: u32,
}

impl Default for MovementDef {
    fn default() -> Self {
        Self {
            speed_units_per_ms: default_move_speed_units_per_ms(),
            radius_units: default_body_radius_units(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAttackDef {
    #[serde(default = "default_attack_range_units")]
    pub range_units: f32,
    #[serde(default)]
    pub defense_tile_range: Option<TileRangePattern>,
    #[serde(default = "default_basic_attack_damage_type")]
    pub damage_type: DamageType,
    #[serde(default)]
    pub targeting_profile: TargetingProfile,
    #[serde(default)]
    pub air_capable: bool,
    #[serde(default)]
    pub range_role: WeaponRangeRole,
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
            range_units: default_attack_range_units(),
            defense_tile_range: None,
            damage_type: default_basic_attack_damage_type(),
            targeting_profile: TargetingProfile::DefaultForward,
            air_capable: false,
            range_role: WeaponRangeRole::Melee,
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

    pub(crate) fn validate_runtime_contract(&self, owner_label: impl std::fmt::Display) {
        let owner_label = owner_label.to_string();
        assert!(
            self.interval_ms > 0,
            "{} basic_attack interval_ms must be greater than zero",
            owner_label
        );
        if let Some(pattern) = &self.defense_tile_range {
            pattern.validate().unwrap_or_else(|error| {
                panic!(
                    "{} basic_attack has invalid defense_tile_range: {}",
                    owner_label, error
                )
            });
        }
        assert!(
            !matches!(&self.delivery, DeliveryDef::TileArea { .. }),
            "{} basic_attack delivery must be Instant or Projectile; TileArea is skill-only",
            owner_label
        );
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
    /// 타겟팅에서 사용하는 전투 trait.
    #[serde(default)]
    pub target_traits: Vec<UnitTargetTrait>,
    /// 이동/저지/대공 판정의 source of truth.
    #[serde(default)]
    pub mobility_kind: MobilityKind,

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
    pub skill_id: Option<SkillId>,
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

        let database = Self {
            items,
            by_id,
            by_uuid,
        };
        database.validate_indexes();
        database
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
        for item in &self.items {
            assert!(
                !item.target_traits.contains(&UnitTargetTrait::Airborne),
                "abnormality '{}' must use mobility_kind: Airborne instead of target_traits: [Airborne]",
                item.id
            );
            item.basic_attack
                .validate_runtime_contract(format!("abnormality '{}'", item.id));
        }
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
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
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

    #[test]
    #[should_panic(expected = "must use mobility_kind: Airborne instead of target_traits")]
    fn new_rejects_airborne_target_trait_as_source_of_truth() {
        let mut metadata = abnormality("air_trait", Uuid::from_u128(3));
        metadata.target_traits = vec![UnitTargetTrait::Airborne];

        let _ = AbnormalityDatabase::new(vec![metadata]);
    }

    #[test]
    #[should_panic(expected = "basic_attack delivery must be Instant or Projectile")]
    fn basic_attack_contract_rejects_tile_area_delivery() {
        let basic_attack = BasicAttackDef {
            delivery: DeliveryDef::TileArea {
                area: Default::default(),
            },
            ..Default::default()
        };

        basic_attack.validate_runtime_contract("test unit");
    }
}
