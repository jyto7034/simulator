use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::game::resources::item_slot::EquippedRef;
use crate::game::{
    behavior::GameError,
    data::{
        artifact_data::ArtifactItem,
        consumable_data::{
            ConsumableDurationPolicy, ConsumableEffect, ConsumableItem, ConsumableTargetPolicy,
            ConsumableTier,
        },
        equipment_data::{
            EquipmentItem, EquipmentMaterialMetadata, EquipmentMaterialType, EquipmentType,
            WeaponCombatProfile,
        },
        Item,
    },
    enums::RiskLevel,
};

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
    pub enhancement_level: u8,
    pub bound: bool,
    pub can_unequip: bool,
    pub cannot_unequip_reason: Option<String>,
    pub weapon_profile: Option<WeaponCombatProfile>,
}

impl EquipmentItemDto {
    pub fn from_owned(instance_uuid: Uuid, meta: &EquipmentItem) -> Self {
        Self::from_owned_with_level(instance_uuid, meta, 0)
    }

    pub fn from_owned_equipment(owned: &OwnedEquipment) -> Self {
        Self::from_owned_with_level(
            owned.instance_uuid,
            owned.meta.as_ref(),
            owned.enhancement_level,
        )
    }

    pub fn from_owned_with_level(
        instance_uuid: Uuid,
        meta: &EquipmentItem,
        enhancement_level: u8,
    ) -> Self {
        Self {
            uuid: instance_uuid,
            definition_id: meta.id.clone(),
            name: meta.name.clone(),
            rarity: meta.rarity,
            equipment_type: meta.equipment_type,
            price: meta.price,
            enhancement_level,
            bound: meta.bound,
            can_unequip: !meta.bound,
            cannot_unequip_reason: meta.bound.then(|| meta.cannot_unequip_reason.clone()),
            weapon_profile: meta.weapon_profile.clone(),
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
pub struct ConsumableItemDto {
    pub uuid: Uuid,
    pub definition_id: String,
    pub name: String,
    pub description: String,
    pub tier: ConsumableTier,
    pub rarity: RiskLevel,
    pub price: u32,
    pub target_policy: ConsumableTargetPolicy,
    pub duration_policy: ConsumableDurationPolicy,
    pub effect: ConsumableEffect,
}

impl ConsumableItemDto {
    pub fn from_owned(instance_uuid: Uuid, meta: &ConsumableItem) -> Self {
        Self::from_metadata(instance_uuid, meta)
    }

    pub fn from_metadata(uuid: Uuid, meta: &ConsumableItem) -> Self {
        Self {
            uuid,
            definition_id: meta.id.clone(),
            name: meta.name.clone(),
            description: meta.description.clone(),
            tier: meta.tier,
            rarity: meta.rarity,
            price: meta.price,
            target_policy: meta.target_policy,
            duration_policy: meta.duration_policy,
            effect: meta.effect.clone(),
        }
    }

    pub fn from_owned_consumable(owned: &OwnedConsumable) -> Self {
        Self::from_owned(owned.instance_uuid, owned.meta.as_ref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentMaterialStackDto {
    pub material_id: String,
    pub name: String,
    pub description: String,
    pub material_type: EquipmentMaterialType,
    pub rarity: RiskLevel,
    pub equipment_type: Option<EquipmentType>,
    pub amount: u32,
}

impl EquipmentMaterialStackDto {
    pub fn from_metadata(meta: &EquipmentMaterialMetadata, amount: u32) -> Self {
        Self {
            material_id: meta.id.clone(),
            name: meta.name.clone(),
            description: meta.description.clone(),
            material_type: meta.material_type,
            rarity: meta.rarity,
            equipment_type: meta.equipment_type,
            amount,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryItemDto {
    Equipment(EquipmentItemDto),
    Artifact(ArtifactItemDto),
    Consumable(ConsumableItemDto),
}

impl InventoryItemDto {
    pub fn from_item_with_uuid(item: &Item, uuid: Uuid) -> Result<Self, GameError> {
        match item {
            Item::Equipment(meta) => Ok(InventoryItemDto::Equipment(EquipmentItemDto::from_owned(
                uuid,
                meta.as_ref(),
            ))),
            Item::Abnormality(_) => Err(GameError::InvalidAction),
            Item::Artifact(meta) => Ok(InventoryItemDto::Artifact(ArtifactItemDto::from_metadata(
                meta.as_ref(),
            ))),
            Item::Consumable(meta) => Ok(InventoryItemDto::Consumable(
                ConsumableItemDto::from_owned(uuid, meta.as_ref()),
            )),
        }
    }

    pub fn uuid(&self) -> Uuid {
        match self {
            Self::Equipment(equipment_item_dto) => equipment_item_dto.uuid,
            Self::Artifact(artifact_item_dto) => artifact_item_dto.uuid,
            Self::Consumable(consumable_item_dto) => consumable_item_dto.uuid,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InventoryDiffDto {
    pub added: Vec<InventoryItemDto>,
    pub updated: Vec<InventoryItemDto>,
    pub removed: Vec<Uuid>,
    #[serde(default)]
    pub material_stacks: Vec<EquipmentMaterialStackDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EquippedItemDto {
    pub instance_uuid: Uuid,
    pub base_uuid: Uuid,
    pub equipment_type: EquipmentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnequipItemResultDto {
    pub item_uuid: Uuid,
    pub target_unit: Uuid,
    pub equipped_items: Vec<EquippedItemDto>,
    pub inventory_diff: InventoryDiffDto,
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

/// 플레이어 인벤토리 시스템.
///
/// 환상체는 더 이상 플레이어 소유물이 아니며, 직원/전투 상대 데이터로만 사용됩니다.
/// 인벤토리는 장비, 소모품, 아티팩트를 보관합니다.

#[derive(Default)]
pub struct Inventory {
    pub equipments: EquipmentInventory,
    pub equipment_materials: EquipmentMaterialInventory,
    pub consumables: ConsumableInventory,
    pub artifacts: ArtifactSlots,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            equipments: EquipmentInventory::new(),
            equipment_materials: EquipmentMaterialInventory::new(),
            consumables: ConsumableInventory::new(),
            artifacts: ArtifactSlots::new(),
        }
    }

    /// 아이템을 추가할 수 있는지 검증 (실제로 추가하지 않음)
    pub fn can_add_item(&self, item: &Item) -> bool {
        match item {
            Item::Abnormality(_) => false,
            Item::Equipment(_) => self.equipments.can_add_item(),
            Item::Consumable(_) => self.consumables.can_add_item(),
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

        if let Some(item) = self.consumables.get_item(&uuid) {
            return Some(Item::Consumable(Arc::clone(&item.meta)));
        }

        None
    }

    /// UUID로 아이템 제거
    pub fn remove_item(&mut self, uuid: Uuid) -> Option<Item> {
        // Equipment는 "소유 인스턴스 UUID"로 제거
        if let Some(item) = self.equipments.remove_item(uuid) {
            return Some(Item::Equipment(item.meta));
        }

        if let Some(item) = self.consumables.remove_item(uuid) {
            return Some(Item::Consumable(item.meta));
        }

        // Artifact는 UUID로 직접 제거할 수 없음 (index 기반)
        // TODO: ArtifactSlots에 remove_by_uuid 추가 필요
        None
    }

    /// 아이템을 소유 인스턴스로 추가합니다.
    ///
    /// - Equipment: `owned_uuid`는 별도의 인스턴스 UUID여야 합니다(중복 소유 지원).
    /// - Artifact: 현재는 `meta.uuid`를 그대로 owned_uuid로 사용합니다.
    pub fn add_item_owned(&mut self, owned_uuid: Uuid, item: Item) -> Result<(), GameError> {
        match item {
            Item::Abnormality(_) => {
                tracing::warn!("Rejected abnormality item for player inventory");
                Err(GameError::InvalidAction)
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
            Item::Consumable(data) => {
                if let Err(err) = self
                    .consumables
                    .add_item(OwnedConsumable::new(owned_uuid, data))
                {
                    tracing::warn!("Failed to add consumable to inventory: {}", err);
                    Err(GameError::InventoryFull)
                } else {
                    tracing::debug!("Added consumable item to inventory");
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
// 장비 인벤토리
// ============================================================

#[derive(Debug, Clone, Default)]
pub struct EquipmentMaterialInventory {
    stacks: HashMap<String, u32>,
}

impl EquipmentMaterialInventory {
    pub fn new() -> Self {
        Self {
            stacks: HashMap::new(),
        }
    }

    pub fn add(&mut self, material_id: &str, amount: u32) -> Result<u32, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }

        let stack = self.stacks.entry(material_id.to_string()).or_default();
        *stack = stack.checked_add(amount).ok_or(GameError::InvalidAction)?;
        Ok(*stack)
    }

    pub fn consume(&mut self, material_id: &str, amount: u32) -> Result<u32, GameError> {
        if amount == 0 {
            return Err(GameError::InvalidAction);
        }

        let Some(stack) = self.stacks.get_mut(material_id) else {
            return Err(GameError::InvalidAction);
        };
        if *stack < amount {
            return Err(GameError::InvalidAction);
        }

        *stack -= amount;
        let remaining = *stack;
        if remaining == 0 {
            self.stacks.remove(material_id);
        }
        Ok(remaining)
    }

    pub fn amount(&self, material_id: &str) -> u32 {
        self.stacks.get(material_id).copied().unwrap_or_default()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &u32)> {
        self.stacks.iter()
    }
}

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
    pub enhancement_level: u8,
}

impl OwnedEquipment {
    pub fn new(instance_uuid: Uuid, meta: Arc<EquipmentItem>) -> Self {
        Self {
            instance_uuid,
            meta,
            equipped_to: None,
            enhancement_level: 0,
        }
    }
}

// ============================================================
// 섭취 아이템 인벤토리
// ============================================================

#[derive(Debug, Clone)]
pub struct ConsumableInventory {
    items: HashMap<Uuid, OwnedConsumable>,
    max_slots: usize,
}

impl Default for ConsumableInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsumableInventory {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            max_slots: 30,
        }
    }

    pub fn with_max_slots(max_slots: usize) -> Self {
        Self {
            items: HashMap::new(),
            max_slots,
        }
    }

    pub fn can_add_item(&self) -> bool {
        self.items.len() < self.max_slots
    }

    pub fn add_item(&mut self, item: OwnedConsumable) -> Result<(), String> {
        if !self.can_add_item() {
            return Err(format!(
                "소모품 인벤토리가 가득 찼습니다 ({}/{})",
                self.items.len(),
                self.max_slots
            ));
        }

        if self.items.contains_key(&item.instance_uuid) {
            return Err(format!(
                "이미 존재하는 소유 소모품 UUID 입니다 (uuid={})",
                item.instance_uuid
            ));
        }

        self.items.insert(item.instance_uuid, item);
        Ok(())
    }

    pub fn remove_item(&mut self, uuid: Uuid) -> Option<OwnedConsumable> {
        self.items.remove(&uuid)
    }

    pub fn get_item(&self, uuid: &Uuid) -> Option<&OwnedConsumable> {
        self.items.get(uuid)
    }

    pub fn iter(&self) -> impl Iterator<Item = &OwnedConsumable> {
        self.items.values()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct OwnedConsumable {
    pub instance_uuid: Uuid,
    pub meta: Arc<ConsumableItem>,
}

impl OwnedConsumable {
    pub fn new(instance_uuid: Uuid, meta: Arc<ConsumableItem>) -> Self {
        Self {
            instance_uuid,
            meta,
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
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
            weapon_profile: (equipment_type == EquipmentType::Weapon).then(Default::default),
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
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
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
    fn inventory_add_find_and_remove_equipment() {
        let mut inv = Inventory::new();

        let equip_owned_uuid = Uuid::from_u128(100);
        let equip = Item::Equipment(equipment_meta(1, EquipmentType::Weapon));
        inv.add_item_owned(equip_owned_uuid, equip).unwrap();

        let found = inv.find_item(equip_owned_uuid).unwrap();
        assert!(matches!(found, Item::Equipment(_)));

        let removed = inv.remove_item(equip_owned_uuid).unwrap();
        assert!(matches!(removed, Item::Equipment(_)));
        assert!(inv.find_item(equip_owned_uuid).is_none());
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
            equipments: EquipmentInventory::with_max_slots(0),
            equipment_materials: EquipmentMaterialInventory::new(),
            consumables: ConsumableInventory::with_max_slots(0),
            artifacts: ArtifactSlots::with_max_slots(0),
        };
        let mut inv = inv;

        assert!(matches!(
            inv.add_item_owned(Uuid::from_u128(1), Item::Abnormality(abnormality_meta(1)))
                .unwrap_err(),
            GameError::InvalidAction
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
