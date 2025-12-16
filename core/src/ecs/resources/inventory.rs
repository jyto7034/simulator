use bevy_ecs::resource::Resource;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    data::{
        abnormality_data::AbnormalityMetadata, artifact_data::ArtifactItem,
        equipment_data::EquipmentItem, Item,
    },
    enums::RiskLevel,
};

#[derive(Debug, Clone)]
pub enum InventoryMetadata {
    Abnormality(AbnormalityInventory),
    Equipment(EquipmentInventory),
    Artifact(ArtifactSlots),
}

// ============================================================
// Inventory DTOs
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentItemDto {
    pub uuid: Uuid,
    pub id: String,
    pub name: String,
    pub rarity: RiskLevel,
    pub price: u32,
}

impl EquipmentItemDto {
    pub fn from_metadata(meta: &EquipmentItem) -> Self {
        Self {
            uuid: meta.uuid,
            id: meta.id.clone(),
            name: meta.name.clone(),
            rarity: meta.rarity,
            price: meta.price,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbnormalityItemDto {
    pub uuid: Uuid,
    pub id: String,
    pub name: String,
    pub risk_level: RiskLevel,
    pub price: u32,
}

impl AbnormalityItemDto {
    pub fn from_metadata(meta: &AbnormalityMetadata) -> Self {
        Self {
            uuid: meta.uuid,
            id: meta.id.clone(),
            name: meta.name.clone(),
            risk_level: meta.risk_level,
            price: meta.price,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactItemDto {
    pub uuid: Uuid,
    pub id: String,
    pub name: String,
    pub description: String,
    pub rarity: RiskLevel,
    pub price: u32,
}

impl ArtifactItemDto {
    pub fn from_metadata(meta: &ArtifactItem) -> Self {
        Self {
            uuid: meta.uuid,
            id: meta.id.clone(),
            name: meta.name.clone(),
            description: meta.description.clone(),
            rarity: meta.rarity,
            price: meta.price,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryItemDto {
    Equipment(EquipmentItemDto),
    Abnormality(AbnormalityItemDto),
    Artifact(ArtifactItemDto),
}

impl InventoryItemDto {
    pub fn from_item(item: &Item) -> Self {
        match item {
            Item::Equipment(meta) => {
                InventoryItemDto::Equipment(EquipmentItemDto::from_metadata(meta.as_ref()))
            }
            Item::Abnormality(meta) => {
                InventoryItemDto::Abnormality(AbnormalityItemDto::from_metadata(meta.as_ref()))
            }
            Item::Artifact(meta) => {
                InventoryItemDto::Artifact(ArtifactItemDto::from_metadata(meta.as_ref()))
            }
        }
    }

    pub fn uuid(&self) -> Uuid {
        match self {
            Self::Equipment(equipment_item_dto) => equipment_item_dto.uuid,
            Self::Abnormality(abnormality_item_dto) => abnormality_item_dto.uuid,
            Self::Artifact(artifact_item_dto) => artifact_item_dto.uuid,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InventoryDiffDto {
    pub added: Vec<InventoryItemDto>,
    pub updated: Vec<InventoryItemDto>,
    pub removed: Vec<Uuid>,
}

/// 인벤토리 시스템 (3가지 독립된 보관소)
///
/// 1. 기물 인벤토리 - 최대 20개 슬롯 (기본값)
/// 2. 장비 인벤토리 - 최대 20개 슬롯 (기본값)
/// 3. 아티팩트 슬롯 - 최대 10개 슬롯 (기본값), 한 번 장착되면 귀속됨

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

    /// 아이템을 추가할 수 있는지 검증 (실제로 추가하지 않음)
    pub fn can_add_item(&self, item: &Item) -> bool {
        match item {
            Item::Abnormality(_) => self.abnormalities.can_add_item(),
            Item::Equipment(_) => self.equipments.can_add_item(),
            Item::Artifact(_) => self.artifacts.can_add_item(),
        }
    }

    /// UUID로 아이템 찾기 (조회만, 제거하지 않음)
    pub fn find_item(&self, uuid: Uuid) -> Option<Item> {
        // Equipment에서 찾기
        if let Some(item) = self.equipments.get_item(&uuid) {
            return Some(Item::Equipment(Arc::clone(item)));
        }

        // Abnormality에서 찾기
        if let Some(item) = self.abnormalities.get_item(&uuid) {
            return Some(Item::Abnormality(Arc::clone(item)));
        }

        // Artifact는 UUID로 직접 찾을 수 없음 (index 기반)
        // TODO: ArtifactSlots에 find_by_uuid 추가 필요
        None
    }

    /// UUID로 아이템 제거
    pub fn remove_item(&mut self, uuid: Uuid) -> Option<Item> {
        // Equipment에서 제거 시도
        if let Some(item) = self.equipments.remove_item(uuid) {
            return Some(Item::Equipment(item));
        }

        // Abnormality에서 제거 시도
        if let Some(item) = self.abnormalities.remove_item(uuid) {
            return Some(Item::Abnormality(item));
        }

        // Artifact는 UUID로 직접 제거할 수 없음 (index 기반)
        // TODO: ArtifactSlots에 remove_by_uuid 추가 필요
        None
    }

    pub fn add_item(&mut self, item: Item) -> Result<(), GameError> {
        match item {
            Item::Abnormality(data) => {
                if let Err(err) = self.abnormalities.add_item(data) {
                    tracing::warn!("Failed to add abnormality to inventory: {}", err);
                    Err(GameError::InventoryFull)
                } else {
                    tracing::debug!("Added abnormality item to inventory");
                    Ok(())
                }
            }
            Item::Equipment(data) => {
                if let Err(err) = self.equipments.add_item(data) {
                    tracing::warn!("Failed to add equipment to inventory: {}", err);
                    Err(GameError::InventoryFull)
                } else {
                    tracing::debug!("Added equipment item to inventory");
                    Ok(())
                }
            }
            Item::Artifact(data) => {
                if let Err(err) = self.artifacts.add_item(data) {
                    tracing::warn!("Failed to add artifact to slots: {}", err);
                    Err(GameError::InventoryFull)
                } else {
                    tracing::debug!("Added artifact item to slots");
                    Ok(())
                }
            }
        }
    }
}

// ============================================================
// 환상체 인벤토리
// ============================================================

#[derive(Debug, Clone)]
pub struct AbnormalityInventory {
    items: HashMap<Uuid, Arc<AbnormalityMetadata>>,
    max_slots: usize,
}

impl Default for AbnormalityInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl AbnormalityInventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            max_slots: 20, // 기본 20개 슬롯
        }
    }

    pub fn with_max_slots(max_slots: usize) -> Self {
        Self {
            items: HashMap::new(),
            max_slots,
        }
    }

    /// 환상체를 추가할 수 있는지 확인
    pub fn can_add_item(&self) -> bool {
        self.items.len() < self.max_slots
    }

    /// 환상체 추가 (슬롯 제한 있음)
    pub fn add_item(&mut self, item: Arc<AbnormalityMetadata>) -> Result<(), String> {
        if !self.can_add_item() {
            return Err(format!(
                "환상체 인벤토리가 가득 찼습니다 ({}/{})",
                self.items.len(),
                self.max_slots
            ));
        }

        self.items.insert(item.uuid, item);
        Ok(())
    }

    pub fn remove_item(&mut self, uuid: Uuid) -> Option<Arc<AbnormalityMetadata>> {
        self.items.remove(&uuid)
    }

    pub fn get_item(&self, uuid: &Uuid) -> Option<&Arc<AbnormalityMetadata>> {
        self.items.get(uuid)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<AbnormalityMetadata>> {
        self.items.values()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn max_slots(&self) -> usize {
        self.max_slots
    }
}

// ============================================================
// 장비 인벤토리
// ============================================================

#[derive(Debug, Clone)]
pub struct EquipmentInventory {
    items: HashMap<Uuid, Arc<EquipmentItem>>,
    max_slots: usize,
}

impl Default for EquipmentInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl EquipmentInventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            max_slots: 20, // 기본 20개 슬롯
        }
    }

    pub fn with_max_slots(max_slots: usize) -> Self {
        Self {
            items: HashMap::new(),
            max_slots,
        }
    }

    /// 장비를 추가할 수 있는지 확인
    pub fn can_add_item(&self) -> bool {
        self.items.len() < self.max_slots
    }

    /// 장비 추가 (슬롯 제한 있음)
    pub fn add_item(&mut self, item: Arc<EquipmentItem>) -> Result<(), String> {
        if !self.can_add_item() {
            return Err(format!(
                "장비 인벤토리가 가득 찼습니다 ({}/{})",
                self.items.len(),
                self.max_slots
            ));
        }

        self.items.insert(item.uuid, item);
        Ok(())
    }

    pub fn remove_item(&mut self, uuid: Uuid) -> Option<Arc<EquipmentItem>> {
        self.items.remove(&uuid)
    }

    pub fn get_item(&self, uuid: &Uuid) -> Option<&Arc<EquipmentItem>> {
        self.items.get(uuid)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<EquipmentItem>> {
        self.items.values()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn max_slots(&self) -> usize {
        self.max_slots
    }
}

// ============================================================
// 아티팩트 인벤토리
// ============================================================

#[derive(Debug, Clone)]
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

    /// 아티팩트를 추가할 수 있는지 확인
    pub fn can_add_item(&self) -> bool {
        self.slots.len() < self.max_slots
    }

    /// 아티팩트 추가 (슬롯 제한 있음)
    pub fn add_item(&mut self, item: Arc<ArtifactItem>) -> Result<(), String> {
        if !self.can_add_item() {
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

    pub fn get_all_items(&self) -> Vec<Arc<ArtifactItem>> {
        self.slots.iter().cloned().collect()
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
