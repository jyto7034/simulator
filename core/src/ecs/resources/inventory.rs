use bevy_ecs::resource::Resource;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::ecs::resources::item_slot::{EquippedRef, ItemSlot};
use crate::game::{
    behavior::GameError,
    data::{
        abnormality_data::AbnormalityMetadata,
        artifact_data::ArtifactItem,
        equipment_data::{EquipmentItem, EquipmentType},
        Item,
    },
    enums::RiskLevel,
    growth::GrowthStack,
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
    pub definition_id: String,
    pub name: String,
    pub rarity: RiskLevel,
    pub equipment_type: EquipmentType,
    pub price: u32,
}

impl EquipmentItemDto {
    pub fn from_owned(instance_uuid: Uuid, meta: &EquipmentItem) -> Self {
        Self {
            uuid: instance_uuid,
            definition_id: meta.id.clone(),
            name: meta.name.clone(),
            rarity: meta.rarity,
            equipment_type: meta.equipment_type,
            price: meta.price,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbnormalityItemDto {
    pub uuid: Uuid,
    pub base_uuid: Uuid,
    pub id: String,
    pub name: String,
    pub risk_level: RiskLevel,
    pub price: u32,
    pub max_health: u32,
    pub resonance_start: u32,
    pub resonance_max: u32,
}

impl AbnormalityItemDto {
    pub fn from_owned(instance_uuid: Uuid, meta: &AbnormalityMetadata) -> Self {
        Self {
            uuid: instance_uuid,
            base_uuid: meta.uuid,
            id: meta.id.clone(),
            name: meta.name.clone(),
            risk_level: meta.risk_level,
            price: meta.price,
            max_health: meta.max_health,
            resonance_start: meta.resonance.start,
            resonance_max: meta.resonance.max,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactItemDto {
    pub uuid: Uuid,
    pub definition_id: String,
    pub name: String,
    pub description: String,
    pub rarity: RiskLevel,
    pub price: u32,
}

impl ArtifactItemDto {
    pub fn from_metadata(meta: &ArtifactItem) -> Self {
        Self {
            uuid: meta.uuid,
            definition_id: meta.id.clone(),
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
    pub fn from_item_with_uuid(item: &Item, uuid: Uuid) -> Self {
        match item {
            Item::Equipment(meta) => {
                InventoryItemDto::Equipment(EquipmentItemDto::from_owned(uuid, meta.as_ref()))
            }
            Item::Abnormality(meta) => {
                InventoryItemDto::Abnormality(AbnormalityItemDto::from_owned(uuid, meta.as_ref()))
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EquippedItemDto {
    pub instance_uuid: Uuid,
    pub base_uuid: Uuid,
    pub equipment_type: EquipmentType,
}

impl EquippedItemDto {
    pub fn from_equipped_ref(equipped: &EquippedRef) -> Self {
        Self {
            instance_uuid: equipped.instance_uuid,
            base_uuid: equipped.base_uuid,
            equipment_type: equipped.equipment_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EquipItemOutcomeDto {
    Equipped {
        item_uuid: Uuid,
    },
    Combined {
        ingredient_item_uuids: Vec<Uuid>,
        result_item_uuid: Uuid,
        result_base_uuid: Uuid,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipItemResultDto {
    pub requested_item_uuid: Uuid,
    pub target_unit: Uuid,
    pub outcome: EquipItemOutcomeDto,
    pub equipped_items: Vec<EquippedItemDto>,
    pub inventory_diff: InventoryDiffDto,
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
            Item::Artifact(meta) => {
                self.artifacts.can_add_item() && !self.artifacts.contains_uuid(meta.uuid)
            }
        }
    }

    /// UUID로 아이템 찾기 (조회만, 제거하지 않음)
    pub fn find_item(&self, uuid: Uuid) -> Option<Item> {
        // Equipment는 "소유 인스턴스 UUID"로 조회
        if let Some(item) = self.equipments.get_item(&uuid) {
            return Some(Item::Equipment(Arc::clone(&item.meta)));
        }

        // Abnormality는 "소유 인스턴스 UUID"로 조회
        if let Some(item) = self.abnormalities.get_item(&uuid) {
            return Some(Item::Abnormality(Arc::clone(item)));
        }

        None
    }

    /// UUID로 아이템 제거
    pub fn remove_item(&mut self, uuid: Uuid) -> Option<Item> {
        // Equipment는 "소유 인스턴스 UUID"로 제거
        if let Some(item) = self.equipments.remove_item(uuid) {
            return Some(Item::Equipment(item.meta));
        }

        // Abnormality에서 제거 시도
        if let Some(item) = self.abnormalities.remove_item(uuid) {
            return Some(Item::Abnormality(item));
        }

        // Artifact는 UUID로 직접 제거할 수 없음 (index 기반)
        // TODO: ArtifactSlots에 remove_by_uuid 추가 필요
        None
    }

    /// 아이템을 소유 인스턴스로 추가합니다.
    ///
    /// - Equipment: `owned_uuid`는 별도의 인스턴스 UUID여야 합니다(중복 소유 지원).
    /// - Abnormality: `owned_uuid`는 별도의 인스턴스 UUID여야 합니다(중복 소유 지원).
    /// - Artifact: 현재는 `meta.uuid`를 그대로 owned_uuid로 사용합니다.
    pub fn add_item_owned(&mut self, owned_uuid: Uuid, item: Item) -> Result<(), GameError> {
        match item {
            Item::Abnormality(data) => {
                if let Err(err) = self.abnormalities.add_item(owned_uuid, data) {
                    tracing::warn!("Failed to add abnormality to inventory: {}", err);
                    Err(GameError::InventoryFull)
                } else {
                    tracing::debug!("Added abnormality item to inventory");
                    Ok(())
                }
            }
            Item::Equipment(data) => {
                if let Err(err) = self
                    .equipments
                    .add_item(OwnedEquipment::new(owned_uuid, data))
                {
                    tracing::warn!("Failed to add equipment to inventory: {}", err);
                    Err(GameError::InventoryFull)
                } else {
                    tracing::debug!("Added equipment item to inventory");
                    Ok(())
                }
            }
            Item::Artifact(data) => {
                if let Err(err) = self.artifacts.add_item(data) {
                    tracing::warn!("Failed to add artifact to slots: {:?}", err);
                    match err {
                        ArtifactSlotError::Full => Err(GameError::InventoryFull),
                        ArtifactSlotError::DuplicateUuid(_) => Err(GameError::AlreadyOwnedArtifact),
                    }
                } else {
                    tracing::debug!("Added artifact item to slots");
                    Ok(())
                }
            }
        }
    }

    /// 아티팩트 UUID를 이미 소유 중인지 확인
    ///
    /// 아티팩트는 귀속 개념으로 제거/판매가 불가능하므로, 중복 소유 여부 확인용으로 사용합니다.
    pub fn has_artifact(&self, uuid: Uuid) -> bool {
        self.artifacts.contains_uuid(uuid)
    }
}

// ============================================================
// 환상체 인벤토리
// ============================================================

#[derive(Debug, Clone)]
pub struct AbnormalityInventory {
    items: HashMap<Uuid, OwnedAbnormality>,
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
    pub fn add_item(
        &mut self,
        instance_uuid: Uuid,
        item: Arc<AbnormalityMetadata>,
    ) -> Result<(), String> {
        if !self.can_add_item() {
            return Err(format!(
                "환상체 인벤토리가 가득 찼습니다 ({}/{})",
                self.items.len(),
                self.max_slots
            ));
        }

        if self.items.contains_key(&instance_uuid) {
            return Err(format!(
                "이미 존재하는 소유 환상체 UUID 입니다 (uuid={})",
                instance_uuid
            ));
        }

        self.items
            .insert(instance_uuid, OwnedAbnormality::new(instance_uuid, item));
        Ok(())
    }

    /// 환상체 추가 (슬롯 제한 무시)
    ///
    /// 보너스 등 특수 규칙으로 "인벤토리(벤치) 슬롯이 가득 찼을 때"
    /// 필드에 즉시 배치하며 소유 목록에는 포함시켜야 하는 경우에 사용합니다.
    pub fn add_item_ignore_capacity(
        &mut self,
        instance_uuid: Uuid,
        item: Arc<AbnormalityMetadata>,
    ) -> Result<(), String> {
        if self.items.contains_key(&instance_uuid) {
            return Err(format!(
                "이미 존재하는 소유 환상체 UUID 입니다 (uuid={})",
                instance_uuid
            ));
        }

        self.items
            .insert(instance_uuid, OwnedAbnormality::new(instance_uuid, item));
        Ok(())
    }

    pub fn remove_item(&mut self, uuid: Uuid) -> Option<Arc<AbnormalityMetadata>> {
        self.items.remove(&uuid).map(|owned| owned.meta)
    }

    pub fn get_item(&self, uuid: &Uuid) -> Option<&Arc<AbnormalityMetadata>> {
        self.items.get(uuid).map(|owned| &owned.meta)
    }

    pub fn get_growth_stacks(&self, uuid: &Uuid) -> Option<&GrowthStack> {
        self.items.get(uuid).map(|owned| &owned.growth_stacks)
    }

    pub fn get_growth_stacks_mut(&mut self, uuid: &Uuid) -> Option<&mut GrowthStack> {
        self.items
            .get_mut(uuid)
            .map(|owned| &mut owned.growth_stacks)
    }

    pub fn get_owned(&self, uuid: &Uuid) -> Option<&OwnedAbnormality> {
        self.items.get(uuid)
    }

    pub fn get_owned_mut(&mut self, uuid: &Uuid) -> Option<&mut OwnedAbnormality> {
        self.items.get_mut(uuid)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<AbnormalityMetadata>> {
        self.items.values().map(|owned| &owned.meta)
    }

    pub fn iter_owned(&self) -> impl Iterator<Item = &OwnedAbnormality> {
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

#[derive(Debug, Clone)]
pub struct OwnedAbnormality {
    pub instance_uuid: Uuid,
    pub meta: Arc<AbnormalityMetadata>,
    pub growth_stacks: GrowthStack,
    pub item_slot: ItemSlot,
}

impl OwnedAbnormality {
    pub fn new(instance_uuid: Uuid, meta: Arc<AbnormalityMetadata>) -> Self {
        Self {
            instance_uuid,
            meta,
            growth_stacks: GrowthStack::new(),
            item_slot: ItemSlot::default(),
        }
    }
}

// ============================================================
// 장비 인벤토리
// ============================================================

#[derive(Debug, Clone)]
pub struct EquipmentInventory {
    items: HashMap<Uuid, OwnedEquipment>,
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
    pub fn add_item(&mut self, item: OwnedEquipment) -> Result<(), String> {
        if !self.can_add_item() {
            return Err(format!(
                "장비 인벤토리가 가득 찼습니다 ({}/{})",
                self.items.len(),
                self.max_slots
            ));
        }

        if self.items.contains_key(&item.instance_uuid) {
            return Err(format!(
                "이미 존재하는 소유 장비 UUID 입니다 (uuid={})",
                item.instance_uuid
            ));
        }

        self.items.insert(item.instance_uuid, item);
        Ok(())
    }

    pub fn remove_item(&mut self, uuid: Uuid) -> Option<OwnedEquipment> {
        self.items.remove(&uuid)
    }

    pub fn get_item(&self, uuid: &Uuid) -> Option<&OwnedEquipment> {
        self.items.get(uuid)
    }

    pub fn get_item_mut(&mut self, uuid: &Uuid) -> Option<&mut OwnedEquipment> {
        self.items.get_mut(uuid)
    }

    pub fn iter(&self) -> impl Iterator<Item = &OwnedEquipment> {
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

#[derive(Debug, Clone)]
pub struct OwnedEquipment {
    pub instance_uuid: Uuid,
    pub meta: Arc<EquipmentItem>,
    pub equipped_to: Option<Uuid>,
}

impl OwnedEquipment {
    pub fn new(instance_uuid: Uuid, meta: Arc<EquipmentItem>) -> Self {
        Self {
            instance_uuid,
            meta,
            equipped_to: None,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactSlotError {
    Full,
    DuplicateUuid(Uuid),
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
    pub fn add_item(&mut self, item: Arc<ArtifactItem>) -> Result<(), ArtifactSlotError> {
        if self.contains_uuid(item.uuid) {
            return Err(ArtifactSlotError::DuplicateUuid(item.uuid));
        }

        if !self.can_add_item() {
            return Err(ArtifactSlotError::Full);
        }

        self.slots.push(item);
        Ok(())
    }

    pub fn contains_uuid(&self, uuid: Uuid) -> bool {
        self.slots.iter().any(|item| item.uuid == uuid)
    }

    pub fn get_item(&self, index: usize) -> Option<&Arc<ArtifactItem>> {
        self.slots.get(index)
    }

    pub fn get_all_items(&self) -> Vec<Arc<ArtifactItem>> {
        self.slots.to_vec()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::{
        abnormality_data::{AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef},
        artifact_data::ArtifactMetadata,
        equipment_data::{EquipmentMetadata, EquipmentType},
        Item,
    };
    use std::collections::HashMap;

    fn equipment_meta(uuid: u128, equipment_type: EquipmentType) -> Arc<EquipmentMetadata> {
        Arc::new(EquipmentMetadata {
            id: format!("equip_{uuid}"),
            uuid: Uuid::from_u128(uuid),
            name: "Equip".to_string(),
            equipment_type,
            rarity: RiskLevel::ZAYIN,
            price: 1,
            allow_duplicate_equip: true,
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
        })
    }

    fn abnormality_meta(uuid: u128) -> Arc<AbnormalityMetadata> {
        Arc::new(AbnormalityMetadata {
            id: format!("abno_{uuid}"),
            uuid: Uuid::from_u128(uuid),
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 1,
            max_health: 10,
            attack: 2,
            defense: 1,
            magic_resist: 0,
            movement: MovementDef::default(),
            basic_attack: BasicAttackDef::default(),
            resonance: ResonanceDef::default(),
            skill_id: None,
        })
    }

    fn artifact_meta(uuid: u128) -> Arc<ArtifactMetadata> {
        Arc::new(ArtifactMetadata {
            id: format!("art_{uuid}"),
            uuid: Uuid::from_u128(uuid),
            name: "Art".to_string(),
            description: "desc".to_string(),
            rarity: RiskLevel::ZAYIN,
            price: 1,
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
        })
    }

    #[test]
    fn inventory_add_find_and_remove_equipment_and_abnormality() {
        let mut inv = Inventory::new();

        let equip_owned_uuid = Uuid::from_u128(100);
        let equip = Item::Equipment(equipment_meta(1, EquipmentType::Weapon));
        inv.add_item_owned(equip_owned_uuid, equip).unwrap();

        let found = inv.find_item(equip_owned_uuid).unwrap();
        assert!(matches!(found, Item::Equipment(_)));

        let removed = inv.remove_item(equip_owned_uuid).unwrap();
        assert!(matches!(removed, Item::Equipment(_)));
        assert!(inv.find_item(equip_owned_uuid).is_none());

        let abno_owned_uuid = Uuid::from_u128(200);
        let abno = Item::Abnormality(abnormality_meta(2));
        inv.add_item_owned(abno_owned_uuid, abno).unwrap();

        let found = inv.find_item(abno_owned_uuid).unwrap();
        assert!(matches!(found, Item::Abnormality(_)));

        let removed = inv.remove_item(abno_owned_uuid).unwrap();
        assert!(matches!(removed, Item::Abnormality(_)));
        assert!(inv.find_item(abno_owned_uuid).is_none());
    }

    #[test]
    fn inventory_artifact_is_added_to_slots_and_is_not_removable_by_uuid() {
        let mut inv = Inventory::new();
        let artifact = Item::Artifact(artifact_meta(10));

        // owned_uuid is ignored for artifacts (meta.uuid is used).
        inv.add_item_owned(Uuid::from_u128(999), artifact).unwrap();

        assert!(inv.has_artifact(Uuid::from_u128(10)));
        assert!(inv.find_item(Uuid::from_u128(10)).is_none());
        assert!(inv.remove_item(Uuid::from_u128(10)).is_none());
    }

    #[test]
    fn abnormality_inventory_rejects_duplicates_and_full_inventory() {
        let mut inv = AbnormalityInventory::with_max_slots(2);
        let owned_uuid = Uuid::from_u128(1);

        inv.add_item(owned_uuid, abnormality_meta(1)).unwrap();

        let err = inv.add_item(owned_uuid, abnormality_meta(2)).unwrap_err();
        assert!(err.contains("이미 존재"));

        inv.add_item(Uuid::from_u128(2), abnormality_meta(3))
            .unwrap();

        let err = inv
            .add_item(Uuid::from_u128(3), abnormality_meta(4))
            .unwrap_err();
        assert!(err.contains("가득"));
    }

    #[test]
    fn equipment_inventory_rejects_duplicates_and_full_inventory() {
        let mut inv = EquipmentInventory::with_max_slots(2);
        let owned_uuid = Uuid::from_u128(1);
        inv.add_item(OwnedEquipment::new(
            owned_uuid,
            equipment_meta(1, EquipmentType::Weapon),
        ))
        .unwrap();

        let err = inv
            .add_item(OwnedEquipment::new(
                owned_uuid,
                equipment_meta(2, EquipmentType::Armor),
            ))
            .unwrap_err();
        assert!(err.contains("이미 존재"));

        inv.add_item(OwnedEquipment::new(
            Uuid::from_u128(2),
            equipment_meta(3, EquipmentType::Accessory),
        ))
        .unwrap();

        let err = inv
            .add_item(OwnedEquipment::new(
                Uuid::from_u128(3),
                equipment_meta(4, EquipmentType::Armor),
            ))
            .unwrap_err();
        assert!(err.contains("가득"));
    }

    #[test]
    fn artifact_slots_limit_capacity_and_reject_duplicates() {
        let mut slots = ArtifactSlots::with_max_slots(1);
        let a = artifact_meta(1);
        let duplicate = artifact_meta(1);
        let b = artifact_meta(2);

        slots.add_item(Arc::clone(&a)).unwrap();
        assert!(slots.contains_uuid(a.uuid));
        assert!(!slots.contains_uuid(b.uuid));

        let duplicate_err = slots.add_item(duplicate).unwrap_err();
        assert!(matches!(
            duplicate_err,
            ArtifactSlotError::DuplicateUuid(uuid) if uuid == a.uuid
        ));

        let full_err = slots.add_item(b).unwrap_err();
        assert!(matches!(full_err, ArtifactSlotError::Full));
    }

    #[test]
    fn inventory_rejects_duplicate_artifact_uuid() {
        let mut inv = Inventory::new();
        let artifact = Item::Artifact(artifact_meta(10));
        inv.add_item_owned(Uuid::from_u128(1), artifact.clone())
            .unwrap();

        let err = inv
            .add_item_owned(Uuid::from_u128(2), artifact)
            .unwrap_err();
        assert!(matches!(err, GameError::AlreadyOwnedArtifact));
    }

    #[test]
    fn inventory_add_item_owned_maps_full_to_game_error() {
        let inv = Inventory {
            abnormalities: AbnormalityInventory::with_max_slots(0),
            equipments: EquipmentInventory::with_max_slots(0),
            artifacts: ArtifactSlots::with_max_slots(0),
        };
        let mut inv = inv;

        assert!(matches!(
            inv.add_item_owned(Uuid::from_u128(1), Item::Abnormality(abnormality_meta(1)))
                .unwrap_err(),
            GameError::InventoryFull
        ));
        assert!(matches!(
            inv.add_item_owned(
                Uuid::from_u128(2),
                Item::Equipment(equipment_meta(1, EquipmentType::Weapon))
            )
            .unwrap_err(),
            GameError::InventoryFull
        ));
        assert!(matches!(
            inv.add_item_owned(Uuid::from_u128(3), Item::Artifact(artifact_meta(1)))
                .unwrap_err(),
            GameError::InventoryFull
        ));
    }
}
