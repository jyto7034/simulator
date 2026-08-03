use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::data::skill_fragment_data::SkillFragmentId;
use crate::game::resources::item_slot::EquippedRef;
use crate::game::resources::{
    EquipItemOutcomeDto, EquipItemResultDto, EquipmentItemDto, InventoryDiffDto, InventoryItemDto,
    UnequipItemResultDto,
};
use crate::game::reward::RewardEffect;

impl GameCore {
    pub(super) fn handle_equip_item(
        &mut self,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_equip_item_payload(item_uuid, target_unit)?;

        let incoming = self
            .state
            .inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        let source_unit = incoming.equipped_to;
        let incoming_ref = EquippedRef {
            instance_uuid: item_uuid,
            base_uuid: incoming.meta.uuid,
            equipment_type: incoming.meta.equipment_type,
        };
        let allow_duplicate = incoming.meta.allow_duplicate_equip;
        let replaced_item_uuid = self
            .state
            .roster
            .get(&target_unit)
            .ok_or(GameError::UnitNotFound)?
            .loadout
            .item_slot
            .iter()
            .find(|equipped| equipped.equipment_type == incoming_ref.equipment_type)
            .map(|equipped| equipped.instance_uuid);

        let mut staged_roster = self.state.roster.clone();
        let mut staged_inventory = self.state.inventory.clone();

        if let Some(source_unit) = source_unit {
            staged_roster
                .get_mut(&source_unit)
                .ok_or(GameError::UnitNotFound)?
                .loadout
                .item_slot
                .remove_by_instance(item_uuid)
                .ok_or(GameError::InvalidAction)?;
        }

        if let Some(replaced_item_uuid) = replaced_item_uuid {
            staged_roster
                .get_mut(&target_unit)
                .ok_or(GameError::UnitNotFound)?
                .loadout
                .item_slot
                .remove_by_instance(replaced_item_uuid)
                .ok_or(GameError::InvalidAction)?;
            staged_inventory
                .equipments
                .get_item_mut(&replaced_item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?
                .equipped_to = None;
        }

        staged_roster
            .get_mut(&target_unit)
            .ok_or(GameError::UnitNotFound)?
            .loadout
            .item_slot
            .equip(incoming_ref, allow_duplicate)
            .map_err(|_| GameError::InvalidAction)?;
        staged_inventory
            .equipments
            .get_item_mut(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?
            .equipped_to = Some(target_unit);

        let mut updated_item_uuids = vec![item_uuid];
        if let Some(replaced_item_uuid) = replaced_item_uuid {
            updated_item_uuids.push(replaced_item_uuid);
        }
        let updated = updated_item_uuids
            .iter()
            .map(|uuid| {
                staged_inventory
                    .equipments
                    .get_item(uuid)
                    .map(EquipmentItemDto::from_owned_equipment)
                    .map(InventoryItemDto::Equipment)
                    .ok_or(GameError::InventoryItemNotFound)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let equipped_items = Self::equipped_item_dtos_from_target(
            &staged_inventory,
            Some(&staged_roster),
            target_unit,
        )?;

        self.state.roster = staged_roster;
        self.state.inventory = staged_inventory;

        Ok(BehaviorResult::EquipItem {
            result: EquipItemResultDto {
                requested_item_uuid: item_uuid,
                target_unit,
                outcome: EquipItemOutcomeDto::Equipped { item_uuid },
                equipped_items,
                inventory_diff: InventoryDiffDto {
                    added: vec![],
                    updated,
                    removed: vec![],
                    material_stacks: vec![],
                },
            },
        })
    }

    pub(super) fn handle_unequip_item(
        &mut self,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_unequip_item_payload(item_uuid, target_unit)?;

        let roster = self.roster_mut()?;
        let employee = roster
            .get_mut(&target_unit)
            .ok_or(GameError::UnitNotFound)?;
        employee
            .loadout
            .item_slot
            .remove_by_instance(item_uuid)
            .ok_or(GameError::InvalidAction)?;

        let inventory = self.inventory_mut()?;
        let owned_equipment = inventory
            .equipments
            .get_item_mut(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        owned_equipment.equipped_to = None;
        let updated_item =
            InventoryItemDto::Equipment(EquipmentItemDto::from_owned_equipment(owned_equipment));
        let inventory_diff = InventoryDiffDto {
            added: vec![],
            updated: vec![updated_item],
            removed: vec![],
            material_stacks: vec![],
        };
        let _ = inventory;

        let roster = self.roster()?;
        let inventory = self.inventory()?;
        let equipped_items =
            Self::equipped_item_dtos_from_target(inventory, Some(roster), target_unit)?;

        Ok(BehaviorResult::UnEquipItem {
            result: UnequipItemResultDto {
                item_uuid,
                target_unit,
                equipped_items,
                inventory_diff,
            },
        })
    }

    pub(super) fn handle_use_consumable_item(
        &mut self,
        item_uuid: Uuid,
        target_employee_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_use_consumable_item_payload(item_uuid, target_employee_uuid)?;

        let mut next_inventory = self.state.inventory.clone();
        let mut next_roster = self.state.roster.clone();
        let owned_consumable = next_inventory
            .consumables
            .remove_one_by_uuid(item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        let (replaced_modifier, active_modifier, condition) = {
            let employee = next_roster
                .get_mut(&target_employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            let replaced_modifier = match owned_consumable.meta.effect {
                crate::game::data::consumable_data::ConsumableEffect::ReducePhysicalDamage {
                    amount,
                } => {
                    employee.condition.reduce_physical_damage(amount);
                    None
                }
                _ => employee.apply_consumable_modifier(item_uuid, owned_consumable.meta.as_ref()),
            };
            (
                replaced_modifier,
                employee.active_consumable_modifier.clone(),
                employee.condition,
            )
        };
        let updated = if owned_consumable.remaining_quantity > 0 {
            let stack = next_inventory
                .consumables
                .get_item(&item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            vec![InventoryItemDto::Consumable(
                crate::game::resources::ConsumableItemDto::from_consumable_stack(stack),
            )]
        } else {
            vec![]
        };
        let inventory_diff = InventoryDiffDto {
            added: vec![],
            updated,
            removed: (owned_consumable.remaining_quantity == 0)
                .then_some(item_uuid)
                .into_iter()
                .collect(),
            material_stacks: vec![],
        };
        self.state.inventory = next_inventory;
        self.state.roster = next_roster;

        let result = BehaviorResult::ConsumableItemUsed {
            item_uuid,
            target_employee_uuid,
            replaced_modifier,
            active_modifier,
            condition,
            inventory_diff,
        };
        self.finalize_combat_recovery_node_resolution(result)
    }

    pub(super) fn handle_equip_skill_fragment(
        &mut self,
        employee_uuid: Uuid,
        fragment_id: &SkillFragmentId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_equip_skill_fragment_payload(employee_uuid, fragment_id)?;
        self.validate_skill_fragment_equip_limit(employee_uuid, fragment_id)?;
        let inventory = self.state.skill_fragments.clone();
        let database = self.game_data.skill_fragment_data.clone();
        let employee = self.state.roster.equip_skill_fragment(
            employee_uuid,
            &inventory,
            &database,
            fragment_id,
        )?;

        Ok(BehaviorResult::SkillFragmentLoadoutUpdated {
            employee_uuid,
            equipped_fragment_ids: employee.skill_fragments.equipped_ids(),
        })
    }

    pub(super) fn handle_unequip_skill_fragment(
        &mut self,
        employee_uuid: Uuid,
        fragment_id: &SkillFragmentId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_unequip_skill_fragment_payload(employee_uuid, fragment_id)?;
        let database = self.game_data.skill_fragment_data.clone();
        let employee =
            self.state
                .roster
                .unequip_skill_fragment(employee_uuid, &database, fragment_id)?;

        Ok(BehaviorResult::SkillFragmentLoadoutUpdated {
            employee_uuid,
            equipped_fragment_ids: employee.skill_fragments.equipped_ids(),
        })
    }

    pub(super) fn handle_upgrade_skill_fragment(
        &mut self,
        target_fragment_id: &SkillFragmentId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_upgrade_skill_fragment_payload(target_fragment_id)?;
        let result = self.state.skill_fragments.upgrade_with_dust_policy(
            target_fragment_id,
            &self.game_data.skill_fragment_data,
            &self.state.skill_fragment_policy,
        )?;

        Ok(BehaviorResult::SkillFragmentUpgraded {
            target_fragment_id: target_fragment_id.clone(),
            dust_spent: result.dust_spent,
            remaining_dust: result.remaining_dust,
            progress: result.progress,
        })
    }

    pub(super) fn handle_awaken_skill_fragment(
        &mut self,
        target_fragment_id: &SkillFragmentId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_awaken_skill_fragment_payload(target_fragment_id)?;
        let result = self.state.skill_fragments.awaken_with_dust_policy(
            target_fragment_id,
            &self.game_data.skill_fragment_data,
            &self.state.skill_fragment_policy,
        )?;

        Ok(BehaviorResult::SkillFragmentAwakened {
            target_fragment_id: target_fragment_id.clone(),
            dust_spent: result.dust_spent,
            remaining_dust: result.remaining_dust,
            progress: result.progress,
        })
    }

    pub(super) fn handle_dismantle_skill_fragment(
        &mut self,
        fragment_id: &SkillFragmentId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_dismantle_skill_fragment_payload(fragment_id)?;
        let dust_gained = self
            .state
            .skill_fragment_policy
            .dismantle
            .validate_dismantle(
                &self.state.skill_fragments,
                fragment_id,
                &self.game_data.skill_fragment_data,
            )?;
        self.preview_grant_effects(&[RewardEffect::GrantFragmentDust {
            amount: dust_gained,
        }])?;
        let result = self
            .state
            .skill_fragments
            .remove_copy_for_dismantle_with_policy(
                fragment_id,
                &self.game_data.skill_fragment_data,
                &self.state.skill_fragment_policy,
            )?;
        let metadata = self
            .game_data
            .skill_fragment_data
            .get_by_id(fragment_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "skill fragment '{}' is owned by run but missing from static data",
                    fragment_id
                ))
            })?;
        let allowed_equipped_count = match metadata.equip_limit {
            crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies => {
                result.remaining_count as usize
            }
            crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::GlobalExclusive => {
                usize::from(result.remaining_count > 0)
            }
        };
        let mut equipped_employee_ids = self
            .state
            .roster
            .iter()
            .filter(|employee| {
                employee
                    .skill_fragments
                    .active_fragment_id()
                    .is_some_and(|active_id| active_id == fragment_id)
            })
            .map(|employee| employee.uuid)
            .collect::<Vec<_>>();
        equipped_employee_ids.sort_by_key(|uuid| uuid.as_u128());
        for employee_uuid in equipped_employee_ids
            .into_iter()
            .skip(allowed_equipped_count)
        {
            let database = self.game_data.skill_fragment_data.clone();
            self.state
                .roster
                .unequip_skill_fragment(employee_uuid, &database, fragment_id)?;
        }
        self.apply_grant_effects(&[RewardEffect::GrantFragmentDust {
            amount: result.dust_gained,
        }])?;

        Ok(BehaviorResult::SkillFragmentDismantled {
            fragment_id: fragment_id.clone(),
            remaining_count: result.remaining_count,
            dust_gained: result.dust_gained,
            total_dust: self.state.skill_fragments.fragment_dust(),
        })
    }

    pub(super) fn handle_dismantle_equipment(
        &mut self,
        item_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_dismantle_equipment_payload(item_uuid)?;
        let equipment_id = self
            .inventory()?
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?
            .meta
            .id
            .clone();
        let equipped_to = self
            .inventory()?
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?
            .equipped_to;
        let recipe = self
            .game_data
            .equipment_data
            .get_dismantle_recipe_by_equipment_id(&equipment_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "missing equipment dismantle recipe for '{equipment_id}'"
                ))
            })?
            .clone();
        let grant_effects = recipe
            .yields
            .iter()
            .map(|material| RewardEffect::GrantEquipmentMaterial {
                material_id: material.material_id.clone(),
                amount: material.amount,
            })
            .collect::<Vec<_>>();
        self.preview_grant_effects(&grant_effects)?;

        if let Some(employee_uuid) = equipped_to {
            let roster = self.roster_mut()?;
            let employee = roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            employee
                .loadout
                .item_slot
                .remove_by_instance(item_uuid)
                .ok_or(GameError::InvalidAction)?;
        }

        let inventory = self.inventory_mut()?;
        inventory
            .equipments
            .remove_item(item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;

        let granted = self.apply_grant_effects(&grant_effects)?;

        Ok(BehaviorResult::EquipmentDismantled {
            item_uuid,
            equipment_id,
            inventory_diff: InventoryDiffDto {
                added: vec![],
                updated: vec![],
                removed: vec![item_uuid],
                material_stacks: granted.inventory_diff.material_stacks,
            },
        })
    }

    pub(super) fn handle_enhance_equipment(
        &mut self,
        item_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_enhance_equipment_payload(item_uuid)?;
        let equipment_id = self
            .inventory()?
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?
            .meta
            .id
            .clone();
        let recipe = self
            .game_data
            .equipment_data
            .get_enhancement_recipe_by_equipment_id(&equipment_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "missing equipment enhancement recipe for '{equipment_id}'"
                ))
            })?
            .clone();
        let cost_materials = recipe
            .costs_per_level
            .iter()
            .map(|cost| {
                self.game_data
                    .equipment_data
                    .get_material_by_id(&cost.material_id)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "equipment enhancement recipe for '{}' references missing material '{}'",
                            recipe.equipment_id, cost.material_id
                        ))
                    })
                    .cloned()
                    .map(|metadata| (cost.material_id.clone(), cost.amount, metadata))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let inventory = self.inventory_mut()?;
        let mut material_stacks = Vec::with_capacity(cost_materials.len());
        for (material_id, amount, metadata) in &cost_materials {
            let remaining = inventory
                .equipment_materials
                .consume(material_id, *amount)?;
            material_stacks.push(
                crate::game::resources::EquipmentMaterialStackDto::from_metadata(
                    metadata, remaining,
                ),
            );
        }

        let equipment = inventory
            .equipments
            .get_item_mut(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        equipment.enhancement_level = equipment.enhancement_level.saturating_add(1);
        let enhancement_level = equipment.enhancement_level;
        let updated_item =
            InventoryItemDto::Equipment(EquipmentItemDto::from_owned_equipment(equipment));

        Ok(BehaviorResult::EquipmentEnhanced {
            item_uuid,
            equipment_id,
            enhancement_level,
            inventory_diff: InventoryDiffDto {
                added: vec![],
                updated: vec![updated_item],
                removed: vec![],
                material_stacks,
            },
        })
    }
}
