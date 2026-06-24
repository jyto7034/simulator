use std::sync::Arc;

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
        let combine_plan = {
            let inventory = self.inventory()?;

            let incoming = inventory
                .equipments
                .get_item(&item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;

            if incoming.equipped_to.is_some() {
                return Err(GameError::InvalidAction);
            }

            let mut plan = None;
            let roster = self.roster()?;
            if !roster.is_empty() {
                let target = roster.get(&target_unit).ok_or(GameError::UnitNotFound)?;
                for equipped in target.loadout.item_slot.iter() {
                    let Some(result_meta) = self
                        .game_data
                        .equipment_data
                        .combination_result(incoming.meta.uuid, equipped.base_uuid)
                    else {
                        continue;
                    };
                    let equipped_item = inventory
                        .equipments
                        .get_item(&equipped.instance_uuid)
                        .ok_or(GameError::InventoryItemNotFound)?;
                    if equipped_item.meta.bound {
                        return Err(GameError::InvalidAction);
                    }

                    let result_ref = EquippedRef {
                        instance_uuid: item_uuid,
                        base_uuid: result_meta.uuid,
                        equipment_type: result_meta.equipment_type,
                    };
                    let mut projected_item_slot = target.loadout.item_slot.clone();
                    projected_item_slot
                        .remove_by_instance(equipped.instance_uuid)
                        .ok_or(GameError::InvalidAction)?;
                    projected_item_slot
                        .equip(result_ref, result_meta.allow_duplicate_equip)
                        .map_err(|_| GameError::InvalidAction)?;
                    self.validate_active_skill_fragment_for_projected_item_slot(
                        target_unit,
                        &projected_item_slot,
                    )?;

                    plan = Some((equipped.instance_uuid, Arc::new(result_meta.clone())));
                    break;
                }
            }

            plan
        };

        if let Some((equipped_item_uuid, result_meta)) = combine_plan {
            let incoming_equipped_to = {
                let inventory = self.inventory()?;
                inventory
                    .equipments
                    .get_item(&item_uuid)
                    .ok_or(GameError::InventoryItemNotFound)?
                    .equipped_to
            };
            if incoming_equipped_to.is_some() {
                return Err(GameError::InvalidAction);
            }

            let roster = self.roster_mut()?;
            let employee = roster
                .get_mut(&target_unit)
                .ok_or(GameError::UnitNotFound)?;
            employee
                .loadout
                .item_slot
                .remove_by_instance(equipped_item_uuid)
                .ok_or(GameError::InvalidAction)?;

            let inventory = self.inventory_mut()?;
            inventory
                .equipments
                .remove_item(item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            inventory
                .equipments
                .remove_item(equipped_item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;

            let granted = self.apply_grant_effects(&[RewardEffect::GrantEquipment {
                equipment_id: result_meta.id.clone(),
            }])?;
            let result_instance_uuid = granted
                .inventory_diff
                .added
                .iter()
                .find_map(|item| match item {
                    InventoryItemDto::Equipment(item) if item.definition_id == result_meta.id => {
                        Some(item.uuid)
                    }
                    _ => None,
                })
                .ok_or(GameError::InvalidAction)?;

            let roster = self.roster_mut()?;
            let employee = roster
                .get_mut(&target_unit)
                .ok_or(GameError::UnitNotFound)?;
            employee
                .loadout
                .item_slot
                .equip(
                    EquippedRef {
                        instance_uuid: result_instance_uuid,
                        base_uuid: result_meta.uuid,
                        equipment_type: result_meta.equipment_type,
                    },
                    result_meta.allow_duplicate_equip,
                )
                .map_err(|_| GameError::InvalidAction)?;

            let inventory = self.inventory_mut()?;
            inventory
                .equipments
                .get_item_mut(&result_instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?
                .equipped_to = Some(target_unit);

            let inventory_diff = InventoryDiffDto {
                added: granted.inventory_diff.added,
                updated: vec![],
                removed: vec![item_uuid, equipped_item_uuid],
                material_stacks: vec![],
            };
            let _ = inventory;
            let roster = self.roster()?;
            let inventory = self.inventory()?;
            let equipped_items =
                Self::equipped_item_dtos_from_target(inventory, Some(roster), target_unit)?;

            return Ok(BehaviorResult::EquipItem {
                result: EquipItemResultDto {
                    requested_item_uuid: item_uuid,
                    target_unit,
                    outcome: EquipItemOutcomeDto::Combined {
                        ingredient_item_uuids: vec![equipped_item_uuid, item_uuid],
                        result_item_uuid: result_instance_uuid,
                        result_base_uuid: result_meta.uuid,
                    },
                    equipped_items,
                    inventory_diff,
                },
            });
        }

        let inventory = self.inventory_mut()?;

        let (base_uuid, equipment_type, allow_duplicate, equipped_to) = {
            let owned_equipment = inventory
                .equipments
                .get_item(&item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            (
                owned_equipment.meta.uuid,
                owned_equipment.meta.equipment_type,
                owned_equipment.meta.allow_duplicate_equip,
                owned_equipment.equipped_to,
            )
        };

        if equipped_to.is_some() {
            return Err(GameError::InvalidAction);
        }

        let _ = inventory;
        let projected_item_slot = {
            let roster = self.roster()?;
            let employee = roster.get(&target_unit).ok_or(GameError::UnitNotFound)?;
            let mut projected_item_slot = employee.loadout.item_slot.clone();
            projected_item_slot
                .equip(
                    EquippedRef {
                        instance_uuid: item_uuid,
                        base_uuid,
                        equipment_type,
                    },
                    allow_duplicate,
                )
                .map_err(|_| GameError::InvalidAction)?;
            projected_item_slot
        };
        self.validate_active_skill_fragment_for_projected_item_slot(
            target_unit,
            &projected_item_slot,
        )?;

        let roster = self.roster_mut()?;
        let employee = roster
            .get_mut(&target_unit)
            .ok_or(GameError::UnitNotFound)?;
        employee
            .loadout
            .item_slot
            .equip(
                EquippedRef {
                    instance_uuid: item_uuid,
                    base_uuid,
                    equipment_type,
                },
                allow_duplicate,
            )
            .map_err(|_| GameError::InvalidAction)?;

        let inventory = self.inventory_mut()?;
        let owned_equipment = inventory
            .equipments
            .get_item_mut(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        owned_equipment.equipped_to = Some(target_unit);
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

        Ok(BehaviorResult::EquipItem {
            result: EquipItemResultDto {
                requested_item_uuid: item_uuid,
                target_unit,
                outcome: EquipItemOutcomeDto::Equipped { item_uuid },
                equipped_items,
                inventory_diff,
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

        let owned_consumable = self
            .inventory_mut()?
            .consumables
            .remove_item(item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        let replaced_modifier = {
            let employee = self
                .roster_mut()?
                .get_mut(&target_employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            employee.apply_consumable_modifier(item_uuid, owned_consumable.meta.as_ref())
        };
        let employee = self
            .roster()?
            .get(&target_employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        let applied_modifier = employee
            .active_consumable_modifier
            .clone()
            .ok_or(GameError::InvalidAction)?;
        let inventory_diff = InventoryDiffDto {
            added: vec![],
            updated: vec![],
            removed: vec![item_uuid],
            material_stacks: vec![],
        };

        Ok(BehaviorResult::ConsumableItemUsed {
            item_uuid,
            target_employee_uuid,
            replaced_modifier,
            applied_modifier,
            inventory_diff,
        })
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
