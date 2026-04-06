use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::AbilityActivationBinding,
    data::{build_string_index, build_uuid_index, once_lock_with},
    enums::RiskLevel,
    stats::TriggeredEffects,
};

fn default_allow_duplicate_equip() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon,
    Armor,
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
    /// 장기적으로 사용하는 proc/activation 기반 ability 연결
    #[serde(default)]
    pub ability_activations: Vec<AbilityActivationBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentDatabase {
    pub items: Vec<EquipmentMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl EquipmentDatabase {
    pub fn new(items: Vec<EquipmentMetadata>) -> Self {
        let by_id = once_lock_with(build_string_index(&items, "equipment id", |item| &item.id));
        let by_uuid = once_lock_with(build_uuid_index(&items, "equipment uuid", |item| item.uuid));

        Self {
            items,
            by_id,
            by_uuid,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.items, "equipment id", |item| &item.id))
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid
            .get_or_init(|| build_uuid_index(&self.items, "equipment uuid", |item| item.uuid))
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_uuid();
    }

    pub fn get_by_id(&self, id: &str) -> Option<&EquipmentMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.items.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&EquipmentMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.items.get(index))
    }
}
