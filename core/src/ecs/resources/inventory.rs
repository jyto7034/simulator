use bevy_ecs::resource::Resource;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    data::{
        abnormality_data::AbnormalityItem,
        artifact_data::ArtifactItem,
        equipment_data::{EquipmentItem, EquipmentMetadata},
        ItemReference,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryMetadata {
    Abnormality(AbnormalityInventory),
    Equipment(EquipmentInventory),
    Artifact(ArtifactSlots),
}

/// 인벤토리 시스템 (3가지 독립된 보관소)
///
/// 1. 기물 인벤토리 - 자유롭게 입출 가능
/// 2. 장비 인벤토리 - 자유롭게 입출 가능
/// 3. 아티팩트 슬롯 - 자유롭게 입출 불가능, 한 번 장착되면 귀속됨.

#[derive(Resource, Default)]
pub struct Inventory {
    pub abnormalities: AbnormalityInventory,

    pub equipments: EquipmentInventory,

    pub artifacts: ArtifactSlots,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            abnormalities: AbnormalityInventory::new(),
            equipments: EquipmentInventory::new(),
            artifacts: ArtifactSlots::new(),
        }
    }

    pub fn add_item(&mut self, item: ItemReference) -> Result<(), GameError> {
        match item {
            ItemReference::Abnormality(data) => {
                self.abnormalities.add_item(data);
                Ok(())
            }
            ItemReference::Equipment(data) => {
                self.equipments.add_item(data);
                Ok(())
            }
            ItemReference::Artifact(data) => self.artifacts.add_item(data).map_err(|_| {
                // TODO: 정교한 오류 타입으로 개선
                GameError::InvalidAction
            }),
        }
    }
}

// ============================================================
// 환상체 인벤토리
// ============================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbnormalityInventory {
    items: HashMap<Uuid, Arc<AbnormalityItem>>,
}

impl AbnormalityInventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
    pub fn add_item(&mut self, item: Arc<AbnormalityItem>) {
        self.items.insert(item.uuid, item);
    }

    pub fn remove_item(&mut self, uuid: Uuid) -> Option<Arc<AbnormalityItem>> {
        self.items.remove(&uuid)
    }

    pub fn get_item(&self, uuid: &Uuid) -> Option<&Arc<AbnormalityItem>> {
        self.items.get(uuid)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<AbnormalityItem>> {
        self.items.values()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// ============================================================
// 장비 인벤토리
// ============================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquipmentInventory {
    items: HashMap<Uuid, Arc<EquipmentItem>>,
}

impl EquipmentInventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
    pub fn add_item(&mut self, item: Arc<EquipmentMetadata>) {
        self.items.insert(item.uuid, item);
    }

    pub fn remove_item(&mut self, uuid: Uuid) -> Option<Arc<EquipmentMetadata>> {
        self.items.remove(&uuid)
    }

    pub fn get_item(&self, uuid: &Uuid) -> Option<&Arc<EquipmentMetadata>> {
        self.items.get(uuid)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<EquipmentMetadata>> {
        self.items.values()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

// ============================================================
// 아티팩트 인벤토리
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSlots {
    pub(super) slots: Vec<Arc<ArtifactItem>>,
    max_slots: usize,
}

impl Default for ArtifactSlots {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactSlots {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            max_slots: 10, // 기본 10개 슬롯
        }
    }

    pub fn with_max_slots(max_slots: usize) -> Self {
        Self {
            slots: Vec::new(),
            max_slots,
        }
    }

    /// 아티팩트 추가 (슬롯 제한 있음)
    pub fn add_item(&mut self, item: Arc<ArtifactItem>) -> Result<(), String> {
        if self.slots.len() >= self.max_slots {
            // TODO: Error 제대로.
            return Err(format!(
                "아티팩트 슬롯이 가득 찼습니다 ({}/{})",
                self.slots.len(),
                self.max_slots
            ));
        }

        self.slots.push(item);
        Ok(())
    }

    pub fn get_item(&self, index: usize) -> Option<&Arc<ArtifactItem>> {
        self.slots.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<ArtifactItem>> {
        self.slots.iter()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn max_slots(&self) -> usize {
        self.max_slots
    }
}
