use std::sync::Arc;

use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::data::skill_fragment_data::SkillFragmentId;
use crate::game::resources::item_slot::EquippedRef;
use crate::game::resources::{
    EquipItemOutcomeDto, EquipItemResultDto, EquipmentItemDto, InventoryDiffDto, InventoryItemDto,
    OwnedEquipment,
};

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

                    let result_ref = EquippedRef {
                        instance_uuid: item_uuid,
                        base_uuid: result_meta.uuid,
                        equipment_type: result_meta.equipment_type,
                    };
                    target
                        .loadout
                        .item_slot
                        .can_equip_after_removing(
                            equipped.instance_uuid,
                            result_ref,
                            result_meta.allow_duplicate_equip,
                        )
                        .map_err(|_| GameError::InvalidAction)?;

                    plan = Some((equipped.instance_uuid, Arc::new(result_meta.clone())));
                    break;
                }
            }

            plan
        };

        if let Some((equipped_item_uuid, result_meta)) = combine_plan {
            let result_instance_uuid = self.state.uuid_manager.next_owned_equipment();

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
                .remove_item(item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            inventory
                .equipments
                .remove_item(equipped_item_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;

            inventory
                .equipments
                .add_item(OwnedEquipment::new(
                    result_instance_uuid,
                    Arc::clone(&result_meta),
                ))
                .map_err(|_| GameError::InventoryFull)?;
            inventory
                .equipments
                .get_item_mut(&result_instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?
                .equipped_to = Some(target_unit);

            let inventory_diff = InventoryDiffDto {
                added: vec![InventoryItemDto::Equipment(EquipmentItemDto::from_owned(
                    result_instance_uuid,
                    result_meta.as_ref(),
                ))],
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
        _item_uuid: Uuid,
        _target_unit: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        // 기본 룰: 아이템은 귀속이며 일반 해제는 불가 (해제 아이템으로만 가능)
        Err(GameError::InvalidAction)
    }

    pub(super) fn handle_equip_skill_fragment(
        &mut self,
        employee_uuid: Uuid,
        fragment_id: &SkillFragmentId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_equip_skill_fragment_payload(employee_uuid, fragment_id)?;
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
        material_fragment_id: &SkillFragmentId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_upgrade_skill_fragment_payload(target_fragment_id, material_fragment_id)?;
        let protected_material_ids = self.protected_skill_fragment_material_ids()?;
        let result = self.state.skill_fragments.upgrade_with_material_policy(
            target_fragment_id,
            material_fragment_id,
            &self.game_data.skill_fragment_data,
            &protected_material_ids,
            &self.state.skill_fragment_policy,
        )?;

        Ok(BehaviorResult::SkillFragmentUpgraded {
            target_fragment_id: target_fragment_id.clone(),
            material_fragment_id: material_fragment_id.clone(),
            material_remaining_count: result.material_remaining_count,
            progress: result.progress,
        })
    }

    pub(super) fn handle_awaken_skill_fragment(
        &mut self,
        target_fragment_id: &SkillFragmentId,
        material_fragment_ids: &[SkillFragmentId],
    ) -> Result<BehaviorResult, GameError> {
        self.validate_awaken_skill_fragment_payload(target_fragment_id, material_fragment_ids)?;
        let protected_material_ids = self.protected_skill_fragment_material_ids()?;
        let result = self.state.skill_fragments.awaken_with_materials_policy(
            target_fragment_id,
            material_fragment_ids,
            &self.game_data.skill_fragment_data,
            &protected_material_ids,
            &self.state.skill_fragment_policy,
        )?;

        Ok(BehaviorResult::SkillFragmentAwakened {
            target_fragment_id: target_fragment_id.clone(),
            progress: result.progress,
        })
    }

    pub(super) fn handle_dismantle_skill_fragment(
        &mut self,
        fragment_id: &SkillFragmentId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_dismantle_skill_fragment_payload(fragment_id)?;
        let protected_material_ids = self.protected_skill_fragment_material_ids()?;
        let result = self.state.skill_fragments.dismantle_with_policy(
            fragment_id,
            &self.game_data.skill_fragment_data,
            &protected_material_ids,
            &self.state.skill_fragment_policy,
        )?;

        Ok(BehaviorResult::SkillFragmentDismantled {
            fragment_id: fragment_id.clone(),
            remaining_count: result.remaining_count,
            dust_gained: result.dust_gained,
            total_dust: result.total_dust,
        })
    }

    pub(super) fn handle_restore_equipment(
        &mut self,
        recipe_id: &str,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_restore_equipment_payload(recipe_id)?;
        let recipe = self
            .game_data
            .equipment_data
            .get_restoration_recipe_by_id(recipe_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "unknown equipment restoration recipe '{recipe_id}'"
                ))
            })?
            .clone();
        let result_meta = Arc::new(
            self.game_data
                .equipment_data
                .get_by_id(&recipe.result_equipment_id)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "equipment restoration recipe '{}' references missing result '{}'",
                        recipe.id, recipe.result_equipment_id
                    ))
                })?
                .clone(),
        );
        let cost_materials = recipe
            .costs
            .iter()
            .map(|cost| {
                self.game_data
                    .equipment_data
                    .get_material_by_id(&cost.material_id)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "equipment restoration recipe '{}' references missing material '{}'",
                            recipe.id, cost.material_id
                        ))
                    })
                    .cloned()
                    .map(|metadata| (cost.material_id.clone(), cost.amount, metadata))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result_instance_uuid = self.state.uuid_manager.next_owned_equipment();

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
        inventory
            .equipments
            .add_item(OwnedEquipment::new(
                result_instance_uuid,
                Arc::clone(&result_meta),
            ))
            .map_err(|_| GameError::InventoryFull)?;

        Ok(BehaviorResult::EquipmentRestored {
            recipe_id: recipe.id,
            result_equipment_id: recipe.result_equipment_id,
            inventory_diff: InventoryDiffDto {
                added: vec![InventoryItemDto::Equipment(EquipmentItemDto::from_owned(
                    result_instance_uuid,
                    result_meta.as_ref(),
                ))],
                updated: vec![],
                removed: vec![],
                material_stacks,
            },
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
        let yield_materials = recipe
            .yields
            .iter()
            .map(|material| {
                self.game_data
                    .equipment_data
                    .get_material_by_id(&material.material_id)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "equipment dismantle recipe for '{}' references missing material '{}'",
                            recipe.equipment_id, material.material_id
                        ))
                    })
                    .cloned()
                    .map(|metadata| (material.material_id.clone(), material.amount, metadata))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let inventory = self.inventory_mut()?;
        inventory
            .equipments
            .remove_item(item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;

        let mut material_stacks = Vec::with_capacity(yield_materials.len());
        for (material_id, amount, metadata) in &yield_materials {
            let new_amount = inventory.equipment_materials.add(material_id, *amount)?;
            material_stacks.push(
                crate::game::resources::EquipmentMaterialStackDto::from_metadata(
                    metadata, new_amount,
                ),
            );
        }

        Ok(BehaviorResult::EquipmentDismantled {
            item_uuid,
            equipment_id,
            inventory_diff: InventoryDiffDto {
                added: vec![],
                updated: vec![],
                removed: vec![item_uuid],
                material_stacks,
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
