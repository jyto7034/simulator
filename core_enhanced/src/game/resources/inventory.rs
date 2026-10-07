use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use uuid::Uuid;

use crate::game::resources::item_slot::EquippedRef;
use crate::game::{
    behavior::GameError,
    data::{
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
    employee::EmployeeRoster,
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
pub struct ConsumableItemDto {
    /// Static item-definition UUID. Consumables do not have owned-copy UUIDs.
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
    pub quantity: u32,
}

impl ConsumableItemDto {
    pub fn from_metadata(meta: &ConsumableItem, quantity: u32) -> Self {
        Self {
            uuid: meta.uuid,
            definition_id: meta.id.clone(),
            name: meta.name.clone(),
            description: meta.description.clone(),
            tier: meta.tier,
            rarity: meta.rarity,
            price: meta.price,
            target_policy: meta.target_policy,
            duration_policy: meta.duration_policy,
            effect: meta.effect.clone(),
            quantity,
        }
    }

    pub fn from_consumable_stack(stack: &ConsumableStack) -> Self {
        Self::from_metadata(stack.meta.as_ref(), stack.quantity)
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
    Consumable(ConsumableItemDto),
}

impl InventoryItemDto {
    pub fn from_item_with_uuid(item: &Item, uuid: Uuid) -> Result<Self, GameError> {
        match item {
            Item::Equipment(meta) => Ok(InventoryItemDto::Equipment(EquipmentItemDto::from_owned(
                uuid,
                meta.as_ref(),
            ))),
            Item::Consumable(_) => Err(GameError::InvalidAction),
        }
    }

    pub fn uuid(&self) -> Uuid {
        match self {
            Self::Equipment(equipment_item_dto) => equipment_item_dto.uuid,
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
    Equipped { item_uuid: Uuid },
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
/// 환상체는 플레이어 소유물이 아니며, 직원/전투 상대 데이터로만 사용됩니다.
/// 일반 인벤토리는 장비 인스턴스와 definition별 소모품 stack을 무제한으로 보관합니다.

#[derive(Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub equipments: EquipmentInventory,
    pub equipment_materials: EquipmentMaterialInventory,
    pub consumables: ConsumableInventory,
    reserved_slot_count: usize,
}

crate::authoring::clone_shape::struct_shape!(Inventory {
    equipments,
    equipment_materials,
    consumables,
    reserved_slot_count,
});
crate::authoring::clone_shape::struct_shape!(EquipmentInventory { items });
crate::authoring::clone_shape::struct_shape!(EquipmentMaterialInventory { stacks });
crate::authoring::clone_shape::struct_shape!(ConsumableInventory { items });
crate::authoring::clone_shape::struct_shape!(OwnedEquipment {
    instance_uuid,
    meta,
    equipped_to,
    enhancement_level,
});
crate::authoring::clone_shape::struct_shape!(ConsumableStack { meta, quantity });

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InventoryCheckpointValidationError {
    Ownership,
    Stack,
}

pub const INITIAL_RESERVED_SLOT_COUNT: usize = 50;
pub const RESERVED_SLOT_GROWTH_INCREMENT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryAddOutcome {
    Added,
    Updated,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryRemoveError {
    NotFound,
}

impl InventoryRemoveError {
    pub fn into_game_error(self) -> GameError {
        match self {
            InventoryRemoveError::NotFound => GameError::InventoryItemNotFound,
        }
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            equipments: EquipmentInventory::new(),
            equipment_materials: EquipmentMaterialInventory::new(),
            consumables: ConsumableInventory::new(),
            reserved_slot_count: INITIAL_RESERVED_SLOT_COUNT,
        }
    }

    pub fn occupied_slot_count(&self) -> usize {
        self.equipments.len() + self.consumables.len()
    }

    pub fn reserved_slot_count(&self) -> usize {
        self.reserved_slot_count
    }

    pub(crate) fn validate_checkpoint_state(
        &self,
        roster: &EmployeeRoster,
    ) -> Result<(), InventoryCheckpointValidationError> {
        if self.reserved_slot_count < INITIAL_RESERVED_SLOT_COUNT
            || (self.reserved_slot_count - INITIAL_RESERVED_SLOT_COUNT)
                % RESERVED_SLOT_GROWTH_INCREMENT
                != 0
            || self.occupied_slot_count() > self.reserved_slot_count
            || self
                .equipment_materials
                .stacks
                .iter()
                .any(|(material_id, amount)| material_id.is_empty() || *amount == 0)
        {
            return Err(InventoryCheckpointValidationError::Stack);
        }

        let mut consumable_uuids = HashSet::new();
        if self.consumables.items.iter().any(|(definition_id, stack)| {
            definition_id != &stack.meta.id
                || stack.quantity == 0
                || !consumable_uuids.insert(stack.meta.uuid)
        }) {
            return Err(InventoryCheckpointValidationError::Stack);
        }

        let mut slot_owners = HashMap::new();
        for employee in roster.iter() {
            if !employee.loadout.item_slot.validate_checkpoint_state() {
                return Err(InventoryCheckpointValidationError::Ownership);
            }
            for equipped in employee.loadout.item_slot.iter() {
                if slot_owners
                    .insert(equipped.instance_uuid, employee.uuid)
                    .is_some()
                {
                    return Err(InventoryCheckpointValidationError::Ownership);
                }
                let Some(owned) = self.equipments.get_item(&equipped.instance_uuid) else {
                    return Err(InventoryCheckpointValidationError::Ownership);
                };
                if owned.instance_uuid != equipped.instance_uuid
                    || owned.meta.uuid != equipped.base_uuid
                    || owned.meta.equipment_type != equipped.equipment_type
                    || owned.equipped_to != Some(employee.uuid)
                {
                    return Err(InventoryCheckpointValidationError::Ownership);
                }
            }
        }

        if !self.equipments.items.iter().all(|(instance_uuid, owned)| {
            *instance_uuid == owned.instance_uuid
                && owned.equipped_to == slot_owners.get(instance_uuid).copied()
        }) {
            return Err(InventoryCheckpointValidationError::Ownership);
        }
        Ok(())
    }

    fn reserve_after_new_slot(&self) -> Result<usize, GameError> {
        let occupied_after = self
            .occupied_slot_count()
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        if occupied_after <= self.reserved_slot_count {
            return Ok(self.reserved_slot_count);
        }

        let missing = occupied_after - self.reserved_slot_count;
        let increments = missing
            .checked_add(RESERVED_SLOT_GROWTH_INCREMENT - 1)
            .ok_or(GameError::InvalidAction)?
            / RESERVED_SLOT_GROWTH_INCREMENT;
        self.reserved_slot_count
            .checked_add(
                increments
                    .checked_mul(RESERVED_SLOT_GROWTH_INCREMENT)
                    .ok_or(GameError::InvalidAction)?,
            )
            .ok_or(GameError::InvalidAction)
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

    /// UUID selector로 제거 가능한 item 한 단위를 제거합니다.
    ///
    /// 장비는 owned instance UUID, 소모품은 static definition UUID를 사용합니다.
    pub fn remove_item(&mut self, uuid: Uuid) -> Result<Item, InventoryRemoveError> {
        if let Some(item) = self.equipments.remove_item(uuid) {
            return Ok(Item::Equipment(item.meta));
        }

        if let Some(item) = self.consumables.remove_one_by_uuid(uuid) {
            return Ok(Item::Consumable(item.meta));
        }

        Err(InventoryRemoveError::NotFound)
    }

    /// 아이템을 소유 인스턴스로 추가합니다.
    ///
    /// - Equipment: `owned_uuid`는 별도의 인스턴스 UUID여야 합니다(중복 소유 지원).
    /// - Consumable: 별도 `add_consumable` quantity path가 canonical입니다. 이 generic single-item
    ///   path를 사용할 때 selector는 반드시 definition metadata UUID여야 합니다.
    pub fn add_item_owned(
        &mut self,
        owned_uuid: Uuid,
        item: Item,
    ) -> Result<InventoryAddOutcome, GameError> {
        match item {
            Item::Equipment(data) => {
                let next_reserve = self.reserve_after_new_slot()?;
                self.equipments
                    .add_item(OwnedEquipment::new(owned_uuid, data))
                    .map_err(|err| {
                        tracing::warn!("Failed to add equipment to inventory: {}", err);
                        GameError::InvalidAction
                    })?;
                self.reserved_slot_count = next_reserve;
                tracing::debug!("Added equipment item to inventory");
                Ok(InventoryAddOutcome::Added)
            }
            Item::Consumable(data) => {
                if owned_uuid != data.uuid {
                    return Err(GameError::InvalidAction);
                }
                self.add_consumable(data, 1)
            }
        }
    }

    pub fn add_consumable(
        &mut self,
        data: Arc<ConsumableItem>,
        quantity: u32,
    ) -> Result<InventoryAddOutcome, GameError> {
        if quantity == 0 {
            return Err(GameError::InvalidAction);
        }

        let definition_id = data.id.clone();
        if self.consumables.contains_definition_id(&definition_id) {
            self.consumables.add_quantity(data, quantity)?;
            tracing::debug!(
                definition_id,
                quantity,
                "Increased consumable stack quantity"
            );
            return Ok(InventoryAddOutcome::Updated);
        }

        let next_reserve = self.reserve_after_new_slot()?;
        self.consumables.add_quantity(data, quantity)?;
        self.reserved_slot_count = next_reserve;
        tracing::debug!(definition_id, quantity, "Added consumable stack");
        Ok(InventoryAddOutcome::Added)
    }
}

// ============================================================
// 장비 인벤토리
// ============================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EquipmentMaterialInventory {
    stacks: crate::owned_collections::OwnedHashMap<String, u32>,
}

impl EquipmentMaterialInventory {
    pub fn new() -> Self {
        Self {
            stacks: crate::owned_collections::OwnedHashMap::new(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentInventory {
    items: crate::owned_collections::OwnedHashMap<Uuid, OwnedEquipment>,
}

impl Default for EquipmentInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl EquipmentInventory {
    pub fn new() -> Self {
        Self {
            items: crate::owned_collections::OwnedHashMap::new(),
        }
    }

    /// 장비 인스턴스 추가. 일반 인벤토리는 capacity admission limit이 없습니다.
    pub fn add_item(&mut self, item: OwnedEquipment) -> Result<(), String> {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableInventory {
    items: crate::owned_collections::OwnedHashMap<String, ConsumableStack>,
}

impl Default for ConsumableInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsumableInventory {
    pub fn new() -> Self {
        Self {
            items: crate::owned_collections::OwnedHashMap::new(),
        }
    }

    pub fn contains_definition_id(&self, definition_id: &str) -> bool {
        self.items.contains_key(definition_id)
    }

    pub fn add_quantity(
        &mut self,
        meta: Arc<ConsumableItem>,
        quantity: u32,
    ) -> Result<u32, GameError> {
        if quantity == 0 {
            return Err(GameError::InvalidAction);
        }

        if let Some(stack) = self.items.get_mut(&meta.id) {
            if stack.meta.uuid != meta.uuid {
                return Err(GameError::InvalidStaticData(format!(
                    "consumable definition '{}' changed UUID inside an owned stack",
                    meta.id
                )));
            }
            stack.quantity = stack
                .quantity
                .checked_add(quantity)
                .ok_or(GameError::InvalidAction)?;
            return Ok(stack.quantity);
        }

        let definition_id = meta.id.clone();
        self.items
            .insert(definition_id, ConsumableStack { meta, quantity });
        Ok(quantity)
    }

    pub fn remove_one_by_uuid(&mut self, uuid: Uuid) -> Option<ConsumableRemoval> {
        let definition_id = self.items.iter().find_map(|(definition_id, stack)| {
            (stack.meta.uuid == uuid).then(|| definition_id.clone())
        })?;
        self.remove_quantity(&definition_id, 1).ok()
    }

    pub fn get_item(&self, uuid: &Uuid) -> Option<&ConsumableStack> {
        self.items.values().find(|stack| stack.meta.uuid == *uuid)
    }

    pub fn get_by_definition_id(&self, definition_id: &str) -> Option<&ConsumableStack> {
        self.items.get(definition_id)
    }

    pub fn remove_quantity(
        &mut self,
        definition_id: &str,
        quantity: u32,
    ) -> Result<ConsumableRemoval, GameError> {
        if quantity == 0 {
            return Err(GameError::InvalidAction);
        }
        let stack = self
            .items
            .get_mut(definition_id)
            .ok_or(GameError::InventoryItemNotFound)?;
        if stack.quantity < quantity {
            return Err(GameError::InvalidAction);
        }

        stack.quantity -= quantity;
        let remaining_quantity = stack.quantity;
        let meta = Arc::clone(&stack.meta);
        if remaining_quantity == 0 {
            self.items.remove(definition_id);
        }
        Ok(ConsumableRemoval {
            meta,
            removed_quantity: quantity,
            remaining_quantity,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &ConsumableStack> {
        self.items.values()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn total_quantity(&self) -> u64 {
        self.items
            .values()
            .map(|stack| u64::from(stack.quantity))
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumableStack {
    pub meta: Arc<ConsumableItem>,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct ConsumableRemoval {
    pub meta: Arc<ConsumableItem>,
    pub removed_quantity: u32,
    pub remaining_quantity: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::{
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
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        })
    }

    fn consumable_meta(uuid: u128, id: &str) -> Arc<ConsumableItem> {
        Arc::new(ConsumableItem {
            id: id.to_string(),
            uuid: Uuid::from_u128(uuid),
            name: id.to_string(),
            description: "desc".to_string(),
            tier: ConsumableTier::Common,
            rarity: RiskLevel::ZAYIN,
            price: 1,
            target_policy: ConsumableTargetPolicy::SingleEmployee,
            duration_policy: ConsumableDurationPolicy::NextCombatNode,
            effect: ConsumableEffect::TraumaMitigation { percent: 10 },
            live_pool: true,
        })
    }

    #[test]
    fn owned_clone_shape_inventory_preserves_metadata_sharing_and_retained_tables() {
        use crate::authoring::{
            clone_shape::CloneShape,
            control::{self, CancellationToken, CloneMeter, ExecutionLimits, ExecutionPhase},
        };
        let measure = |inventory: &Inventory| {
            control::run(
                &ExecutionLimits::default(),
                &CancellationToken::default(),
                ExecutionPhase::TrialPreparation,
                || {
                    let mut meter = CloneMeter::default();
                    inventory.visit_clone_shape(&mut meter).unwrap();
                    meter.finish("inventory_storage", "inventory_copy").unwrap();
                },
            )
            .unwrap()
            .1
        };
        let id = Uuid::from_u128(100);
        let mut inventory = Inventory::new();
        inventory
            .add_item_owned(
                id,
                Item::Equipment(equipment_meta(1, EquipmentType::Weapon)),
            )
            .unwrap();
        inventory
            .add_consumable(consumable_meta(2, "tonic"), 1)
            .unwrap();
        let before = measure(&inventory);
        let item = inventory.equipments.items.get_mut(&id).unwrap();
        Arc::make_mut(&mut item.meta).name = "shared definition".repeat(100_000);
        assert_eq!(measure(&inventory), before);
        let cloned = inventory.clone();
        assert!(Arc::ptr_eq(
            &inventory.equipments.items.get(&id).unwrap().meta,
            &cloned.equipments.items.get(&id).unwrap().meta
        ));
        assert!(Arc::ptr_eq(
            &inventory.consumables.items.get("tonic").unwrap().meta,
            &cloned.consumables.items.get("tonic").unwrap().meta
        ));
        assert_eq!(measure(&cloned), before);
        for index in 0..128 {
            inventory
                .equipment_materials
                .add(&format!("material_{index}"), 1)
                .unwrap();
        }
        inventory.equipment_materials.stacks.clear();
        let retained = measure(&inventory);
        assert!(retained.clone_slots_peak > before.clone_slots_peak);
        let mut copied = false;
        let failure = control::run(
            &ExecutionLimits {
                max_clone_slots: before.clone_slots_peak,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialPreparation,
            || {
                let mut meter = CloneMeter::default();
                inventory.visit_clone_shape(&mut meter)?;
                meter.finish("inventory_storage", "inventory_copy")?;
                let _candidate = inventory.clone();
                copied = true;
                Ok::<_, GameError>(())
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
        assert!(!copied);
        assert_eq!(inventory.equipment_materials.stacks.len(), 0);
        assert!(inventory.find_item(id).is_some());
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
    fn equipment_inventory_rejects_duplicate_instance_uuid_without_capacity_limit() {
        let mut inv = EquipmentInventory::new();
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

        for index in 2..=75_u128 {
            inv.add_item(OwnedEquipment::new(
                Uuid::from_u128(index),
                equipment_meta(index + 100, EquipmentType::Accessory),
            ))
            .unwrap();
        }
        assert_eq!(inv.len(), 75);
    }

    #[test]
    fn ordinary_reserve_grows_by_tens_and_never_shrinks() {
        let mut inv = Inventory::new();
        assert_eq!(inv.occupied_slot_count(), 0);
        assert_eq!(inv.reserved_slot_count(), 50);

        for index in 1..=50_u128 {
            inv.add_item_owned(
                Uuid::from_u128(index),
                Item::Equipment(equipment_meta(index + 1000, EquipmentType::Accessory)),
            )
            .unwrap();
        }
        assert_eq!(inv.occupied_slot_count(), 50);
        assert_eq!(inv.reserved_slot_count(), 50);

        inv.add_item_owned(
            Uuid::from_u128(51),
            Item::Equipment(equipment_meta(1051, EquipmentType::Accessory)),
        )
        .unwrap();
        assert_eq!(inv.reserved_slot_count(), 60);

        for index in 52..=61_u128 {
            inv.add_item_owned(
                Uuid::from_u128(index),
                Item::Equipment(equipment_meta(index + 1000, EquipmentType::Accessory)),
            )
            .unwrap();
        }
        assert_eq!(inv.occupied_slot_count(), 61);
        assert_eq!(inv.reserved_slot_count(), 70);

        for index in 1..=20_u128 {
            inv.remove_item(Uuid::from_u128(index)).unwrap();
        }
        assert_eq!(inv.occupied_slot_count(), 41);
        assert_eq!(inv.reserved_slot_count(), 70);
    }

    #[test]
    fn consumables_stack_by_definition_without_growing_reserve_for_quantity() {
        let mut inv = Inventory::new();
        let consumable = consumable_meta(2000, "field_tonic");

        assert_eq!(
            inv.add_consumable(Arc::clone(&consumable), 3).unwrap(),
            InventoryAddOutcome::Added
        );
        assert_eq!(inv.occupied_slot_count(), 1);
        assert_eq!(inv.reserved_slot_count(), 50);
        assert_eq!(inv.consumables.total_quantity(), 3);

        assert_eq!(
            inv.add_consumable(Arc::clone(&consumable), 7).unwrap(),
            InventoryAddOutcome::Updated
        );
        assert_eq!(inv.occupied_slot_count(), 1);
        assert_eq!(inv.reserved_slot_count(), 50);
        assert_eq!(
            inv.consumables
                .get_by_definition_id("field_tonic")
                .unwrap()
                .quantity,
            10
        );

        let removal = inv.consumables.remove_quantity("field_tonic", 9).unwrap();
        assert_eq!(removal.remaining_quantity, 1);
        let removal = inv.consumables.remove_quantity("field_tonic", 1).unwrap();
        assert_eq!(removal.remaining_quantity, 0);
        assert_eq!(inv.occupied_slot_count(), 0);
        assert_eq!(inv.reserved_slot_count(), 50);
    }

    #[test]
    fn consumable_overflow_and_duplicate_equipment_leave_inventory_unchanged() {
        let mut inv = Inventory::new();
        let consumable = consumable_meta(2001, "overflow_tonic");
        inv.add_consumable(Arc::clone(&consumable), u32::MAX)
            .unwrap();
        let before_reserve = inv.reserved_slot_count();
        assert!(matches!(
            inv.add_consumable(consumable, 1),
            Err(GameError::InvalidAction)
        ));
        assert_eq!(inv.consumables.total_quantity(), u64::from(u32::MAX));
        assert_eq!(inv.reserved_slot_count(), before_reserve);

        let instance_uuid = Uuid::from_u128(3000);
        inv.add_item_owned(
            instance_uuid,
            Item::Equipment(equipment_meta(3001, EquipmentType::Weapon)),
        )
        .unwrap();
        let before_slots = inv.occupied_slot_count();
        assert!(matches!(
            inv.add_item_owned(
                instance_uuid,
                Item::Equipment(equipment_meta(3002, EquipmentType::Armor))
            ),
            Err(GameError::InvalidAction)
        ));
        assert_eq!(inv.occupied_slot_count(), before_slots);
        assert_eq!(inv.reserved_slot_count(), before_reserve);
    }
}
