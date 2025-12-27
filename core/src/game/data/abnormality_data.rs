use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{ability::DeliveryDef, enums::RiskLevel};

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

fn default_attack_windup_ms() -> u32 {
    0
}

fn default_basic_attack_interval_ms() -> u64 {
    1500
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
    /// 전투용 기본 방어력
    pub defense: u32,

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
}

impl AbnormalityDatabase {
    pub fn new(items: Vec<AbnormalityMetadata>) -> Self {
        Self { items }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&AbnormalityMetadata> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&AbnormalityMetadata> {
        self.items.iter().find(|item| item.uuid == *uuid)
    }
}
