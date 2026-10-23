use std::collections::HashSet;

use tracing::{debug, info};
use uuid::Uuid;

use super::state::ActiveBattleSession;
use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::combat_setup::player_spawns::skill_fragment_compatibility_profile_for_employee_with_item_slot;
use crate::game::data::skill_fragment_data::{
    SkillFragmentCompatibilityReport, SkillFragmentEquipLimit,
};
use crate::game::employee::{Employee, EmployeeRoster};
use crate::game::enums::RewardMode;
use crate::game::map::MapNodeId;
use crate::game::resources::item_slot::{EquippedRef, ItemSlot};
use crate::game::resources::{
    EquipmentLoadoutEquipPlan, EquipmentLoadoutTransition, EquippedItemDto, GameState, Inventory,
    InventoryItemDto, RewardSessionState, StarterEquipmentAssignment,
};
use crate::game::reward::{RewardEffect, RewardOption};

impl GameCore {
    pub(super) fn inventory(&self) -> Result<&Inventory, GameError> {
        Ok(&self.state.inventory)
    }

    pub(super) fn inventory_mut(&mut self) -> Result<&mut Inventory, GameError> {
        Ok(&mut self.state.inventory)
    }

    pub(super) fn roster(&self) -> Result<&EmployeeRoster, GameError> {
        Ok(self
            .state
            .active_battle
            .as_ref()
            .map(ActiveBattleSession::employee_roster)
            .unwrap_or(&self.state.roster))
    }

    pub(super) fn roster_mut(&mut self) -> Result<&mut EmployeeRoster, GameError> {
        Ok(&mut self.state.roster)
    }

    pub(super) fn build_reward_session(
        &self,
        stage_uuid: Uuid,
        mode: RewardMode,
        rewards: Vec<RewardOption>,
        can_skip: bool,
    ) -> RewardSessionState {
        RewardSessionState {
            stage_uuid,
            mode,
            rewards,
            selected_reward_uuid: None,
            can_skip,
            relic_activity: None,
            dealer: None,
        }
    }

    pub(super) fn reward_state_result(&self, reward: &RewardSessionState) -> BehaviorResult {
        BehaviorResult::RewardState {
            mode: reward.mode,
            rewards: reward.rewards.clone(),
            selected_reward_uuid: reward.selected_reward_uuid,
        }
    }

    pub(super) fn current_reward_can_skip(&self) -> bool {
        self.state
            .active_node_content
            .as_ref()
            .and_then(|selected| selected.as_reward().ok())
            .map(|reward| reward.can_skip)
            .unwrap_or(false)
    }

    pub(super) fn merge_inventory_diff(
        left: &mut crate::game::resources::InventoryDiffDto,
        right: crate::game::resources::InventoryDiffDto,
    ) {
        left.added.extend(right.added);
        left.updated.extend(right.updated);
        left.removed.extend(right.removed);
    }

    pub(super) fn equipped_item_dtos_from_target(
        _inventory: &Inventory,
        roster: Option<&EmployeeRoster>,
        target_unit: Uuid,
    ) -> Result<Vec<EquippedItemDto>, GameError> {
        let roster = roster.ok_or(GameError::UnitNotFound)?;
        let employee = roster.get(&target_unit).ok_or(GameError::UnitNotFound)?;
        Ok(employee
            .loadout
            .item_slot
            .iter()
            .map(EquippedItemDto::from_equipped_ref)
            .collect())
    }

    pub(super) fn validate_select_reward_payload(&self, reward_id: Uuid) -> Result<(), GameError> {
        let selected = self
            .state
            .active_node_content
            .as_ref()
            .ok_or(GameError::NotInRewardState)?;
        let reward = selected.as_reward()?;

        if reward.mode != RewardMode::ChooseOne {
            return Err(GameError::InvalidAction);
        }

        reward
            .rewards
            .iter()
            .any(|reward| reward.uuid == reward_id)
            .then_some(())
            .ok_or(GameError::EventNotFound)
    }

    pub(super) fn validate_select_map_node(&self, node_id: MapNodeId) -> Result<(), GameError> {
        if self.selectable_map_node_ids().contains(&node_id) {
            Ok(())
        } else {
            Err(GameError::InvalidAction)
        }
    }

    pub(super) fn validate_equip_item_payload(
        &self,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<(), GameError> {
        self.plan_equip_item_payload(item_uuid, target_unit)
            .map(|_| ())
    }

    pub(super) fn plan_equip_item_payload(
        &self,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<EquipmentLoadoutEquipPlan, GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        let inventory = self.inventory()?;
        let roster = self.roster()?;
        let plan =
            EquipmentLoadoutTransition::plan_equip(inventory, roster, item_uuid, target_unit)?;
        self.validate_active_skill_fragment_for_projected_item_slot(
            target_unit,
            plan.projected_target(),
        )?;
        if self.has_combat_recovery_restriction()
            && (inventory.equipments.get_item(&item_uuid)
                .is_none_or(|copy| copy.meta.equipment_type != crate::game::data::equipment_data::EquipmentType::Weapon)
                || roster.get(&target_unit).is_none_or(|employee| !self.recovery_weapon_payload_is_useful(employee)))
        {
            return Err(GameError::InvalidAction);
        }
        Ok(plan)
    }

    pub(super) fn validate_unequip_item_payload(
        &self,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        if self.has_combat_recovery_restriction() {
            return Err(GameError::InvalidAction);
        }
        EquipmentLoadoutTransition::validate_unequip(
            self.inventory()?,
            self.roster()?,
            item_uuid,
            target_unit,
        )
    }

    pub(super) fn validate_use_consumable_item_payload(
        &self,
        item_uuid: Uuid,
        target_employee_uuid: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_consumable_use_context() {
            return Err(GameError::InvalidAction);
        }
        let inventory = self.inventory()?;
        let consumable = inventory
            .consumables
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        let employee = self
            .roster()?
            .get(&target_employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if self.has_combat_recovery_restriction() {
            let crate::game::data::consumable_data::ConsumableEffect::ReducePhysicalDamage { amount } = consumable.meta.effect else {
                return Err(GameError::InvalidAction);
            };
            if consumable.quantity == 0 || !self.recovery_treatment_payload_is_useful(employee, amount) {
                return Err(GameError::InvalidAction);
            }
        }
        let activity = if matches!(
            consumable.meta.effect,
            crate::game::data::consumable_data::ConsumableEffect::ReducePhysicalDamage { .. }
        ) {
            crate::game::employee::EmployeeActivity::PhysicalTreatment
        } else {
            crate::game::employee::EmployeeActivity::Consumable
        };
        if employee.eligibility(activity).eligible {
            Ok(())
        } else {
            Err(GameError::InvalidAction)
        }
    }

    pub(super) fn validate_equip_skill_fragment_payload(
        &self,
        employee_uuid: Uuid,
        copy_uuid: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        if self.has_combat_recovery_restriction() {
            return Err(GameError::InvalidAction);
        }
        self.state
            .skill_fragments
            .validate_bindings(&self.state.roster, &self.game_data.skill_fragment_data)?;
        let employee = self
            .state
            .roster
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        let copy = self
            .state
            .skill_fragments
            .get(&copy_uuid)
            .ok_or(GameError::InvalidAction)?;
        if !employee
            .eligibility(crate::game::employee::EmployeeActivity::Loadout)
            .eligible
            || copy.equipped_to.is_some_and(|id| id != employee_uuid)
            || employee
                .skill_fragments
                .active_copy_uuid()
                .is_some_and(|id| id != copy_uuid)
        {
            return Err(GameError::InvalidAction);
        }
        let (_, metadata, _) = self
            .state
            .skill_fragments
            .resolve(&copy_uuid, &self.game_data.skill_fragment_data)?;
        if metadata.equip_limit == SkillFragmentEquipLimit::GlobalExclusive
            && self.state.skill_fragments.copies().any(|other| {
                other.copy_uuid != copy_uuid
                    && other.definition_id == copy.definition_id
                    && other.equipped_to.is_some()
            })
        {
            return Err(GameError::InvalidAction);
        }
        let report =
            self.skill_fragment_compatibility_report_for_employee(employee_uuid, copy_uuid)?;
        if !report.is_compatible {
            return Err(GameError::SkillFragmentIncompatible {
                fragment_id: copy.definition_id.clone(),
                failure_codes: report.failure_codes,
            });
        }
        Ok(())
    }

    pub(super) fn skill_fragment_compatibility_report_for_employee(
        &self,
        employee_uuid: Uuid,
        copy_uuid: Uuid,
    ) -> Result<SkillFragmentCompatibilityReport, GameError> {
        let employee = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        let profile = skill_fragment_compatibility_profile_for_employee_with_item_slot(
            employee,
            self.inventory()?,
            &self.state.skill_fragments,
            &self.game_data,
            &employee.loadout.item_slot,
        )?;
        let (copy, metadata, _) = self
            .state
            .skill_fragments
            .resolve(&copy_uuid, &self.game_data.skill_fragment_data)?;
        metadata
            .form_compatibility_report(&copy.selected_form, &profile)
            .ok_or(GameError::InvalidAction)
    }

    pub(super) fn validate_active_skill_fragment_for_projected_item_slot(
        &self,
        employee_uuid: Uuid,
        projected_item_slot: &ItemSlot,
    ) -> Result<(), GameError> {
        let employee = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        let Some(copy_uuid) = employee.skill_fragments.active_copy_uuid() else {
            return Ok(());
        };
        let profile = skill_fragment_compatibility_profile_for_employee_with_item_slot(
            employee,
            self.inventory()?,
            &self.state.skill_fragments,
            &self.game_data,
            projected_item_slot,
        )?;
        let (copy, metadata, _) = self
            .state
            .skill_fragments
            .resolve(&copy_uuid, &self.game_data.skill_fragment_data)?;
        let report = metadata
            .form_compatibility_report(&copy.selected_form, &profile)
            .ok_or(GameError::InvalidAction)?;
        if report.is_compatible {
            Ok(())
        } else {
            Err(GameError::SkillFragmentIncompatible {
                fragment_id: copy.definition_id.clone(),
                failure_codes: report.failure_codes,
            })
        }
    }

    pub(super) fn validate_unequip_skill_fragment_payload(
        &self,
        employee_uuid: Uuid,
        copy_uuid: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        self.state
            .skill_fragments
            .validate_bindings(&self.state.roster, &self.game_data.skill_fragment_data)?;
        let employee = self
            .state
            .roster
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if !employee
            .eligibility(crate::game::employee::EmployeeActivity::Loadout)
            .eligible
            || employee.skill_fragments.active_copy_uuid() != Some(copy_uuid)
        {
            return Err(GameError::InvalidAction);
        }
        if self.has_combat_recovery_restriction() && !self.recovery_fragment_removal_payload_is_useful(employee) {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }

    /// Read-only eligibility shared by command and form previews. An unlocked form costs zero.
    pub(super) fn fragment_form_unavailable_reason(
        &self,
        copy_uuid: Uuid,
        form_id: &str,
        selecting: bool,
    ) -> Option<String> {
        let reason = || -> Result<(), &'static str> {
            if !self.is_in_maintenance_node() {
                return Err("maintenance_required");
            }
            self.state
                .skill_fragments
                .validate_bindings(&self.state.roster, &self.game_data.skill_fragment_data)
                .map_err(|_| "invalid_fragment_binding")?;
            let copy = self
                .state
                .skill_fragments
                .get(&copy_uuid)
                .ok_or("fragment_copy_not_owned")?;
            let metadata = self
                .game_data
                .skill_fragment_data
                .get_by_id(&copy.definition_id)
                .ok_or("fragment_definition_missing")?;
            let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
                forms,
                ..
            } = &metadata.effect
            else {
                return Err("fragment_active_skill_unavailable");
            };
            let form = forms.get(form_id).ok_or("fragment_form_unknown")?;
            if selecting && !copy.unlocked_forms.contains(form_id) {
                return Err("fragment_form_locked");
            }
            if let Some(owner) = copy.equipped_to {
                let employee = self
                    .state
                    .roster
                    .get(&owner)
                    .ok_or("fragment_owner_missing")?;
                if !employee
                    .eligibility(crate::game::employee::EmployeeActivity::Loadout)
                    .eligible
                {
                    return Err("employee_ineligible");
                }
                let profile = skill_fragment_compatibility_profile_for_employee_with_item_slot(
                    employee,
                    &self.state.inventory,
                    &self.state.skill_fragments,
                    &self.game_data,
                    &employee.loadout.item_slot,
                )
                .map_err(|_| "fragment_profile_invalid")?;
                let selected = if selecting {
                    form_id
                } else {
                    &copy.selected_form
                };
                if !metadata
                    .form_compatibility_report(selected, &profile)
                    .is_some_and(|r| r.is_compatible)
                {
                    return Err("fragment_form_incompatible");
                }
            }
            if selecting || copy.unlocked_forms.contains(form_id) {
                return Ok(());
            }
            let recipe = form
                .bloom_recipe
                .as_ref()
                .ok_or("fragment_recipe_unavailable")?;
            let costs = recipe
                .aggregated_materials()
                .ok_or("fragment_recipe_invalid")?;
            if self.state.enkephalin.amount < recipe.enkephalin {
                return Err("insufficient_enkephalin");
            }
            copy.investment.validate_additional(&costs).map_err(|_| "fragment_investment_overflow")?;
            for (id, amount) in costs {
                let warehouse = if self.run_state().is_ok_and(|run| run.run_progression.game_mode == crate::game::map::GameMode::Expedition) { self.state.player_profile.lifecycle.vault.materials.get(&id).copied().unwrap_or(0) } else { 0 };
                if u64::from(crate::game::material_economy::carried_amount(&self.state.inventory, &self.state.crafting_resources, &id)) + u64::from(warehouse) < u64::from(amount) {
                    return Err("insufficient_fragment_material");
                }
            }
            Ok(())
        };
        reason().err().map(str::to_owned)
    }

    pub(super) fn is_loadout_action_context(&self) -> bool {
        (matches!(
            self.state.lifecycle.phase(),
            GameState::ViewingMap | GameState::NodeConfirm { .. }
        ))
            || self.is_in_maintenance_node()
            || self.facility_gate_reached()
    }

    pub(super) fn is_consumable_use_context(&self) -> bool {
        matches!(
            self.state.lifecycle.phase(),
            GameState::ViewingMap | GameState::NodeConfirm { .. }
        )
    }

    pub(super) fn validate_dismantle_equipment_payload(
        &self,
        item_uuid: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_node() {
            return Err(GameError::InvalidAction);
        }

        let inventory = self.inventory()?;
        let equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        if equipment.meta.bound {
            return Err(GameError::InvalidAction);
        }

        let effects = self.equipment_dismantle_effects(item_uuid)?;
        self.preview_grant_effects(&effects).map(|_| ())
    }

    pub(super) fn validate_enhance_equipment_payload(
        &self,
        item_uuid: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_node() {
            return Err(GameError::InvalidAction);
        }

        let inventory = self.inventory()?;
        let equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        if equipment.meta.bound || equipment.meta.equipment_type != crate::game::data::equipment_data::EquipmentType::Weapon {
            return Err(GameError::InvalidAction);
        }
        let recipe = self
            .game_data
            .equipment_data
            .get_enhancement_recipe_by_equipment_id(&equipment.meta.id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "missing equipment enhancement recipe for '{}'",
                    equipment.meta.id
                ))
            })?;
        if equipment.enhancement_level >= recipe.max_level {
            return Err(GameError::InvalidAction);
        }

        let aggregated_costs = recipe.aggregated_costs().ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "equipment enhancement recipe for '{}' has aggregate cost amount overflow",
                recipe.equipment_id
            ))
        })?;
        equipment.investment.validate_additional(&aggregated_costs.iter()
            .map(|cost| (cost.material_id.clone(), cost.amount)).collect())?;
        for cost in &aggregated_costs {
            if crate::game::material_economy::carried_amount(inventory, &self.state.crafting_resources, &cost.material_id) < cost.amount {
                return Err(GameError::InvalidAction);
            }
        }

        Ok(())
    }

    /// 게임 상태 전환
    ///
    /// # Arguments
    /// * `new_state` - 전환할 새로운 상태
    ///
    /// # Effects
    /// 1. GameCoreState의 GameState 업데이트
    /// 2. Derived projections are recomputed on demand from the new state.
    pub(super) fn transition_to(&mut self, new_state: GameState) -> Result<(), GameError> {
        let old_state = self.state.lifecycle.snapshot();
        info!("Game state transition: {:?} -> {:?}", old_state, new_state);

        self.state.transition_to(new_state);

        Ok(())
    }

    pub(super) fn initial_player(&mut self, player_id: Uuid) {
        if self.state.initialize_player(player_id) {
            info!("Initializing player {}", player_id);
        } else {
            debug!("Player {} already initialized", player_id);
        }
    }

    pub(super) fn validate_starter_employee_selection(
        &self,
        employee_definition_ids: &[String],
    ) -> Result<(), GameError> {
        let starter_employee_count = self.run_policy().setup.starter_employee_count;
        if employee_definition_ids.len() != starter_employee_count {
            return Err(GameError::InvalidAction);
        }
        let mut seen = HashSet::new();
        if !employee_definition_ids.iter().all(|id| seen.insert(id)) {
            return Err(GameError::InvalidAction);
        }
        let candidates = self
            .state
            .starter_candidates
            .iter()
            .map(|definition| definition.employee_definition_id.as_str())
            .collect::<HashSet<_>>();
        if employee_definition_ids
            .iter()
            .any(|definition_id| !candidates.contains(definition_id.as_str()))
        {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }

    pub(super) fn initialize_selected_starter_employees(
        &mut self,
        employee_definition_ids: &[String],
    ) -> Result<Vec<Uuid>, GameError> {
        self.validate_starter_employee_selection(employee_definition_ids)?;
        let mut candidate_plans = Vec::with_capacity(employee_definition_ids.len());
        for id in employee_definition_ids {
            let candidate = self
                .state
                .starter_candidates
                .iter()
                .find(|definition| &definition.employee_definition_id == id)
                .ok_or(GameError::InvalidAction)?
                .clone();
            candidate_plans.push((self.state.uuid_manager.next_employee(), candidate));
        }
        self.state.roster.clear();
        self.initialize_employees_with_loadouts(&candidate_plans)?;
        info!(
            "Initialized selected starter employee roster with {} employees",
            candidate_plans.len()
        );
        Ok(candidate_plans.into_iter().map(|(id, _)| id).collect())
    }

    /// Called within the start/recruit state transaction. Grants exact copies,
    /// assigns equipment, then binds each active copy to its intended employee.
    pub(super) fn initialize_employees_with_loadouts(
        &mut self,
        candidate_plans: &[(Uuid, crate::game::employee::EmployeeDefinition)],
    ) -> Result<(), GameError> {
        let mut grant_effects = Vec::new();
        let mut equipment_assignments = Vec::new();
        let mut fragment_assignments = Vec::new();
        for (employee_uuid, candidate) in candidate_plans {
            let employee_uuid = *employee_uuid;
            if self.state.roster.get(&employee_uuid).is_some() {
                return Err(GameError::InvalidAction);
            }
            let mut projected_employee = Employee::from_definition(employee_uuid, candidate);
            for equipment_id in &candidate.initial_loadout.equipment_ids {
                let equipment = self
                    .game_data
                    .equipment_data
                    .get_by_id(equipment_id)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "starter employee candidate '{}' references missing equipment '{}'",
                            candidate.employee_definition_id, equipment_id
                        ))
                    })?;
                let placeholder_uuid = Uuid::from_u128(
                    0xF000_0000_0000_0000_0000_0000_0000_0000u128
                        + u128::try_from(grant_effects.len())
                            .map_err(|_| GameError::InvalidAction)?,
                );
                projected_employee
                    .loadout
                    .item_slot
                    .equip(
                        EquippedRef {
                            instance_uuid: placeholder_uuid,
                            base_uuid: equipment.uuid,
                            equipment_type: equipment.equipment_type,
                        },
                        equipment.allow_duplicate_equip,
                    )
                    .map_err(|_| {
                        GameError::InvalidStaticData(format!(
                            "starter employee candidate '{}' has invalid equipment loadout",
                            candidate.employee_definition_id
                        ))
                    })?;
                grant_effects.push(RewardEffect::GrantEquipment {
                    equipment_id: equipment_id.clone(),
                });
                equipment_assignments.push((employee_uuid, equipment_id.clone()));
            }
            if let Some(id) = &candidate.initial_loadout.active_skill_fragment_id {
                grant_effects.push(RewardEffect::GrantSkillFragment {
                    fragment_id: id.clone(),
                });
                fragment_assignments.push((employee_uuid, id.clone()));
            }
        }
        self.preview_grant_effects(&grant_effects)?;
        let granted = self.apply_grant_effects(&grant_effects)?;
        let granted_equipment = granted
            .inventory_diff
            .added
            .iter()
            .filter_map(|item| match item {
                InventoryItemDto::Equipment(item) => Some((item.uuid, item.definition_id.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();
        if granted_equipment.len() != equipment_assignments.len() {
            return Err(GameError::InvalidAction);
        }

        let mut employees = Vec::with_capacity(candidate_plans.len());
        for (employee_uuid, candidate) in candidate_plans {
            employees.push(Employee::from_definition(*employee_uuid, candidate));
        }
        let mut starter_assignments = Vec::with_capacity(equipment_assignments.len());
        for ((employee_uuid, expected_equipment_id), (instance_uuid, granted_equipment_id)) in
            equipment_assignments.into_iter().zip(granted_equipment)
        {
            if expected_equipment_id != granted_equipment_id {
                return Err(GameError::InvalidAction);
            }
            let equipment = self
                .game_data
                .equipment_data
                .get_by_id(&granted_equipment_id)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "starter loadout grant returned missing equipment '{}'",
                        granted_equipment_id
                    ))
                })?;
            starter_assignments.push(StarterEquipmentAssignment::new(
                employee_uuid,
                instance_uuid,
                equipment.uuid,
                equipment.equipment_type,
                equipment.allow_duplicate_equip,
            ));
        }

        let roster = self.roster_mut()?;
        for employee in employees {
            roster.add(employee);
        }
        let state = &mut self.state;
        EquipmentLoadoutTransition::new(&mut state.inventory, &mut state.roster)
            .assign_starter_equipment(&starter_assignments)?;

        if granted.skill_fragment_diffs.len() != fragment_assignments.len() {
            return Err(GameError::InvalidAction);
        }
        for ((employee_uuid, expected_id), granted) in fragment_assignments
            .into_iter()
            .zip(granted.skill_fragment_diffs)
        {
            if expected_id != granted.fragment_id {
                return Err(GameError::InvalidAction);
            }
            let report = self.skill_fragment_compatibility_report_for_employee(
                employee_uuid,
                granted.copy_uuid,
            )?;
            if !report.is_compatible {
                return Err(GameError::SkillFragmentIncompatible {
                    fragment_id: expected_id,
                    failure_codes: report.failure_codes,
                });
            }
            self.state.skill_fragments.equip_copy(
                &mut self.state.roster,
                &self.game_data.skill_fragment_data,
                granted.copy_uuid,
                employee_uuid,
                false,
            )?;
        }
        for (_,definition) in candidate_plans {
            self.record_lab_definition("Employee",definition.employee_definition_id.as_str(),"employee_profile")?;
        }
        Ok(())
    }
}
