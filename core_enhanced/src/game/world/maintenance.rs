use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::resources::{
    EquipItemOutcomeDto, EquipItemResultDto, EquipmentItemDto, EquipmentLoadoutTransition,
    InventoryDiffDto, InventoryItemDto, UnequipItemResultDto,
};
use crate::game::reward::RewardEffect;

impl GameCore {
    pub(super) fn handle_equip_item(
        &mut self,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let plan = self.plan_equip_item_payload(item_uuid, target_unit)?;

        let mut staged_roster = self.state.roster.clone();
        let mut staged_inventory = self.state.inventory.clone();
        let transition = EquipmentLoadoutTransition::new(&mut staged_inventory, &mut staged_roster)
            .equip(plan)?;
        let updated = transition
            .updated_item_uuids
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
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }

        let mut staged_roster = self.state.roster.clone();
        let mut staged_inventory = self.state.inventory.clone();
        EquipmentLoadoutTransition::new(&mut staged_inventory, &mut staged_roster)
            .unequip(item_uuid, target_unit)?;
        let owned_equipment = staged_inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        let updated_item =
            InventoryItemDto::Equipment(EquipmentItemDto::from_owned_equipment(owned_equipment));
        let inventory_diff = InventoryDiffDto {
            added: vec![],
            updated: vec![updated_item],
            removed: vec![],
            material_stacks: vec![],
        };
        let equipped_items = Self::equipped_item_dtos_from_target(
            &staged_inventory,
            Some(&staged_roster),
            target_unit,
        )?;

        self.state.roster = staged_roster;
        self.state.inventory = staged_inventory;

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
        copy_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_equip_skill_fragment_payload(employee_uuid, copy_uuid)?;
        let mut staged = super::grant_state::EconomyGrantState::capture(&self.state)?;
        staged.skill_fragments.equip_copy(
            &mut staged.roster,
            &self.game_data.skill_fragment_data,
            copy_uuid,
            employee_uuid,
            false,
        )?;
        staged.commit_into(&mut self.state);
        Ok(BehaviorResult::SkillFragmentLoadoutUpdated {
            employee_uuid,
            active_copy_uuid: Some(copy_uuid),
        })
    }

    pub(super) fn handle_unequip_skill_fragment(
        &mut self,
        employee_uuid: Uuid,
        copy_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_unequip_skill_fragment_payload(employee_uuid, copy_uuid)?;
        let mut staged = super::grant_state::EconomyGrantState::capture(&self.state)?;
        staged.skill_fragments.unequip_copy(
            &mut staged.roster,
            &self.game_data.skill_fragment_data,
            copy_uuid,
            employee_uuid,
        )?;
        staged.commit_into(&mut self.state);
        Ok(BehaviorResult::SkillFragmentLoadoutUpdated {
            employee_uuid,
            active_copy_uuid: None,
        })
    }

    pub(super) fn validate_transfer_skill_fragment_payload(
        &self,
        copy_uuid: Uuid,
        target: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        let copy = self
            .state
            .skill_fragments
            .get(&copy_uuid)
            .ok_or(GameError::InvalidAction)?;
        copy.equipped_to.ok_or(GameError::InvalidAction)?;
        let report = self.skill_fragment_compatibility_report_for_employee(target, copy_uuid)?;
        if !report.is_compatible {
            return Err(GameError::SkillFragmentIncompatible {
                fragment_id: copy.definition_id.clone(),
                failure_codes: report.failure_codes,
            });
        }
        let mut staged = super::grant_state::EconomyGrantState::capture(&self.state)?;
        staged.skill_fragments.equip_copy(
            &mut staged.roster,
            &self.game_data.skill_fragment_data,
            copy_uuid,
            target,
            true,
        )?;
        Ok(())
    }

    pub(super) fn handle_transfer_skill_fragment(
        &mut self,
        copy_uuid: Uuid,
        target: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_transfer_skill_fragment_payload(copy_uuid, target)?;
        let mut staged = super::grant_state::EconomyGrantState::capture(&self.state)?;
        let previous_owner = staged
            .skill_fragments
            .get(&copy_uuid)
            .and_then(|c| c.equipped_to)
            .ok_or(GameError::InvalidAction)?;
        let changed = staged.skill_fragments.equip_copy(
            &mut staged.roster,
            &self.game_data.skill_fragment_data,
            copy_uuid,
            target,
            true,
        )?;
        let copy = staged
            .skill_fragments
            .get(&copy_uuid)
            .ok_or(GameError::InvalidAction)?
            .clone();
        staged.commit_into(&mut self.state);
        Ok(BehaviorResult::SkillFragmentTransferred {
            copy,
            previous_owner,
            changed,
        })
    }

    pub(super) fn handle_awaken_skill_fragment(
        &mut self,
        copy_uuid: Uuid,
        form_id: &str,
    ) -> Result<BehaviorResult, GameError> {
        if self
            .fragment_form_unavailable_reason(copy_uuid, form_id, false)
            .is_some()
        {
            return Err(GameError::InvalidAction);
        }
        let mut staged = super::grant_state::EconomyGrantState::capture(&self.state)?;
        let copy = staged
            .skill_fragments
            .get(&copy_uuid)
            .ok_or(GameError::InvalidAction)?;
        let mut enkephalin_spent = 0;
        let mut inventory_diff = InventoryDiffDto {
            added: vec![],
            updated: vec![],
            removed: vec![],
            material_stacks: vec![],
        };
        if !copy.unlocked_forms.contains(form_id) {
            let metadata = self
                .game_data
                .skill_fragment_data
                .get_by_id(&copy.definition_id)
                .ok_or(GameError::InvalidAction)?;
            let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
                forms,
                ..
            } = &metadata.effect
            else {
                return Err(GameError::InvalidAction);
            };
            let recipe = forms
                .get(form_id)
                .and_then(|f| f.bloom_recipe.as_ref())
                .ok_or(GameError::InvalidAction)?;
            enkephalin_spent = recipe.enkephalin;
            staged.enkephalin.amount = staged
                .enkephalin
                .amount
                .checked_sub(enkephalin_spent)
                .ok_or(GameError::InvalidAction)?;
            for (id, amount) in recipe
                .aggregated_materials()
                .ok_or(GameError::InvalidAction)?
            {
                let metadata = self
                    .game_data
                    .equipment_data
                    .get_material_by_id(&id)
                    .ok_or(GameError::InvalidAction)?;
                let remaining = staged.inventory.equipment_materials.consume(&id, amount)?;
                inventory_diff.material_stacks.push(
                    crate::game::resources::EquipmentMaterialStackDto::from_metadata(
                        metadata, remaining,
                    ),
                );
            }
        }
        let changed = staged.skill_fragments.unlock(
            copy_uuid,
            form_id,
            &self.game_data.skill_fragment_data,
        )?;
        let copy = staged
            .skill_fragments
            .get(&copy_uuid)
            .ok_or(GameError::InvalidAction)?
            .clone();
        staged.commit_into(&mut self.state);
        Ok(BehaviorResult::SkillFragmentAwakened {
            copy,
            changed,
            enkephalin_spent,
            inventory_diff,
        })
    }

    pub(super) fn handle_select_skill_fragment_form(
        &mut self,
        copy_uuid: Uuid,
        form_id: &str,
    ) -> Result<BehaviorResult, GameError> {
        if self
            .fragment_form_unavailable_reason(copy_uuid, form_id, true)
            .is_some()
        {
            return Err(GameError::InvalidAction);
        }
        let mut staged = super::grant_state::EconomyGrantState::capture(&self.state)?;
        let changed = staged.skill_fragments.select(
            copy_uuid,
            form_id,
            &self.game_data.skill_fragment_data,
        )?;
        let copy = staged
            .skill_fragments
            .get(&copy_uuid)
            .ok_or(GameError::InvalidAction)?
            .clone();
        staged.commit_into(&mut self.state);
        Ok(BehaviorResult::SkillFragmentFormSelected { copy, changed })
    }

    pub(super) fn fragment_dismantle_effects(
        &self,
        copy_uuid: Uuid,
        confirmed: bool,
    ) -> Result<(u32, Vec<RewardEffect>), GameError> {
        if !self.is_in_maintenance_node() {
            return Err(GameError::InvalidAction);
        }
        self.state
            .skill_fragments
            .validate_bindings(&self.state.roster, &self.game_data.skill_fragment_data)?;
        let (copy, metadata, _) = self
            .state
            .skill_fragments
            .resolve(&copy_uuid, &self.game_data.skill_fragment_data)?;
        if let Some(owner) = copy.equipped_to {
            if !confirmed {
                return Err(GameError::InvalidAction);
            }
            self.validate_unequip_skill_fragment_payload(owner, copy_uuid)?;
        }
        let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            dismantle_recipe: Some(recipe),
            ..
        } = &metadata.effect
        else {
            return Err(GameError::InvalidAction);
        };
        let mut effects = recipe
            .aggregated_materials()
            .ok_or(GameError::InvalidAction)?
            .into_iter()
            .map(
                |(material_id, amount)| RewardEffect::GrantEquipmentMaterial {
                    material_id,
                    amount,
                },
            )
            .collect::<Vec<_>>();
        if recipe.enkephalin > 0 {
            effects.push(RewardEffect::GrantEnkephalin {
                amount: recipe.enkephalin,
            });
        }
        Ok((recipe.enkephalin, effects))
    }

    pub(super) fn handle_dismantle_skill_fragment(
        &mut self,
        copy_uuid: Uuid,
        confirmed: bool,
    ) -> Result<BehaviorResult, GameError> {
        let (enkephalin_gained, effects) = self.fragment_dismantle_effects(copy_uuid, confirmed)?;
        let mut staged = super::grant_state::EconomyGrantState::capture(&self.state)?;
        if let Some(owner) = staged
            .skill_fragments
            .get(&copy_uuid)
            .and_then(|c| c.equipped_to)
        {
            staged.skill_fragments.unequip_copy(
                &mut staged.roster,
                &self.game_data.skill_fragment_data,
                copy_uuid,
                owner,
            )?;
        }
        let copy = staged.skill_fragments.remove(copy_uuid)?;
        let granted = staged.grant_effects(
            &self.game_data,
            &self.state.skill_fragment_policy,
            &crate::game::reward::GrantExecutionContext::default(),
            &effects,
            self.run_seed ^ 0x4752_414e_5446_5853,
        )?;
        staged.commit_into(&mut self.state);
        Ok(BehaviorResult::SkillFragmentDismantled {
            copy_uuid,
            definition_id: copy.definition_id,
            enkephalin_gained,
            inventory_diff: granted.inventory_diff,
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
        let grant_effects = recipe
            .yields
            .iter()
            .map(|material| RewardEffect::GrantEquipmentMaterial {
                material_id: material.material_id.clone(),
                amount: material.amount,
            })
            .collect::<Vec<_>>();
        self.preview_grant_effects(&grant_effects)?;

        let state = &mut self.state;
        EquipmentLoadoutTransition::new(&mut state.inventory, &mut state.roster)
            .remove_for_dismantle(item_uuid)?;

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
        let aggregated_costs = recipe.aggregated_costs().ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "equipment enhancement recipe for '{}' has aggregate cost amount overflow",
                recipe.equipment_id
            ))
        })?;
        let cost_materials = aggregated_costs
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

        let mut staged_inventory = self.inventory()?.clone();
        let mut material_stacks = Vec::with_capacity(cost_materials.len());
        for (material_id, amount, metadata) in &cost_materials {
            let remaining = staged_inventory
                .equipment_materials
                .consume(material_id, *amount)?;
            material_stacks.push(
                crate::game::resources::EquipmentMaterialStackDto::from_metadata(
                    metadata, remaining,
                ),
            );
        }

        let equipment = staged_inventory
            .equipments
            .get_item_mut(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        equipment.enhancement_level = equipment.enhancement_level.saturating_add(1);
        let enhancement_level = equipment.enhancement_level;
        let updated_item =
            InventoryItemDto::Equipment(EquipmentItemDto::from_owned_equipment(equipment));
        *self.inventory_mut()? = staged_inventory;

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
