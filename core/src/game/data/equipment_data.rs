use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{enums::RiskLevel, stats::TriggeredEffects};

fn default_allow_duplicate_equip() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon,
    Suit,
    Accessory,
}

pub type EquipmentItem = EquipmentMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub equipment_type: EquipmentType,
    pub rarity: RiskLevel,
    pub price: u32,
    #[serde(default = "default_allow_duplicate_equip")]
    pub allow_duplicate_equip: bool,
    /// 트리거 기반 효과 (Permanent = 상시 적용)
    #[serde(default)]
    pub triggered_effects: TriggeredEffects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentDatabase {
    pub items: Vec<EquipmentMetadata>,
}

impl EquipmentDatabase {
    pub fn new(items: Vec<EquipmentMetadata>) -> Self {
        Self { items }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&EquipmentMetadata> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&EquipmentMetadata> {
        self.items.iter().find(|item| item.uuid == *uuid)
    }
}
