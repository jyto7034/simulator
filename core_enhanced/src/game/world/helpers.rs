use std::collections::HashSet;

use tracing::{debug, info};
use uuid::Uuid;

use super::state::ActiveBattleSession;
use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::combat_setup::player_spawns::{
    effective_combat_profile_for_employee, effective_combat_profile_for_employee_with_item_slot,
};
use crate::game::data::skill_fragment_data::{
    SkillFragmentCompatibilityReport, SkillFragmentEquipLimit, SkillFragmentId,
};
use crate::game::employee::{Employee, EmployeeRoster};
use crate::game::enums::RewardMode;
use crate::game::map::MapNodeId;
use crate::game::resources::item_slot::{EquippedRef, ItemSlot};
use crate::game::resources::{
    EquippedItemDto, GameState, Inventory, InventoryItemDto, RewardSessionState,
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
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        let inventory = self.inventory()?;

        let owned_equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;

        if owned_equipment.meta.bound {
            return Err(GameError::InvalidAction);
        }

        let roster = self.roster()?;
        let target = roster.get(&target_unit).ok_or(GameError::UnitNotFound)?;
        if !target
            .eligibility(crate::game::employee::EmployeeActivity::Loadout)
            .eligible
        {
            return Err(GameError::InvalidAction);
        }

        let source_unit = owned_equipment.equipped_to;
        if source_unit == Some(target_unit) {
            return Err(GameError::InvalidAction);
        }
        if let Some(source_unit) = source_unit {
            let source = roster.get(&source_unit).ok_or(GameError::UnitNotFound)?;
            if !source
                .eligibility(crate::game::employee::EmployeeActivity::Loadout)
                .eligible
                || !source.loadout.item_slot.contains_instance(item_uuid)
            {
                return Err(GameError::InvalidAction);
            }
        }

        let incoming_ref = EquippedRef {
            instance_uuid: item_uuid,
            base_uuid: owned_equipment.meta.uuid,
            equipment_type: owned_equipment.meta.equipment_type,
        };
        let replaced = target
            .loadout
            .item_slot
            .iter()
            .find(|equipped| equipped.equipment_type == owned_equipment.meta.equipment_type)
            .copied();
        let mut projected_target = target.loadout.item_slot.clone();
        if let Some(replaced) = replaced {
            let replaced_item = inventory
                .equipments
                .get_item(&replaced.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            if replaced_item.equipped_to != Some(target_unit) || replaced_item.meta.bound {
                return Err(GameError::InvalidAction);
            }
            projected_target
                .remove_by_instance(replaced.instance_uuid)
                .ok_or(GameError::InvalidAction)?;
        }
        projected_target
            .equip(incoming_ref, owned_equipment.meta.allow_duplicate_equip)
            .map_err(|_| GameError::InvalidAction)?;
        self.validate_active_skill_fragment_for_projected_item_slot(target_unit, &projected_target)
    }

    pub(super) fn validate_unequip_item_payload(
        &self,
        item_uuid: Uuid,
        target_unit: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        let inventory = self.inventory()?;
        let owned_equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        if owned_equipment.equipped_to != Some(target_unit) {
            return Err(GameError::InvalidAction);
        }
        if owned_equipment.meta.bound {
            return Err(GameError::InvalidAction);
        }

        let roster = self.roster()?;
        let employee = roster.get(&target_unit).ok_or(GameError::UnitNotFound)?;
        if !employee
            .eligibility(crate::game::employee::EmployeeActivity::Loadout)
            .eligible
        {
            return Err(GameError::InvalidAction);
        }
        if employee
            .loadout
            .item_slot
            .iter()
            .any(|equipped| equipped.instance_uuid == item_uuid)
        {
            Ok(())
        } else {
            Err(GameError::InvalidAction)
        }
    }

    pub(super) fn validate_skill_fragment_equip_limit(
        &self,
        employee_uuid: Uuid,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        let metadata = self
            .game_data
            .skill_fragment_data
            .get_by_id(fragment_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "employee loadout references missing skill fragment '{fragment_id}'"
                ))
            })?;
        let roster = self.roster()?;
        let current_other_equipped = roster
            .iter()
            .filter(|employee| employee.uuid != employee_uuid)
            .filter(|employee| employee.skill_fragments.active_fragment_id() == Some(fragment_id))
            .count();
        match metadata.equip_limit {
            SkillFragmentEquipLimit::GlobalExclusive => {
                if current_other_equipped == 0 {
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            SkillFragmentEquipLimit::OwnedCopies => {
                let required_count = u32::try_from(current_other_equipped + 1)
                    .map_err(|_| GameError::InvalidAction)?;
                if self.state.skill_fragments.count(fragment_id) >= required_count {
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
        }
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
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        let employee = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if !employee
            .eligibility(crate::game::employee::EmployeeActivity::Loadout)
            .eligible
        {
            return Err(GameError::InvalidAction);
        }
        if !self.state.skill_fragments.contains(fragment_id) {
            return Err(GameError::InvalidAction);
        }
        if self
            .game_data
            .skill_fragment_data
            .get_by_id(fragment_id)
            .is_none()
        {
            return Err(GameError::InvalidStaticData(format!(
                "skill fragment '{}' is owned by run but missing from static data",
                fragment_id
            )));
        }
        let compatibility =
            self.skill_fragment_compatibility_report_for_employee(employee_uuid, fragment_id)?;
        if !compatibility.is_compatible {
            return Err(GameError::SkillFragmentIncompatible {
                fragment_id: fragment_id.clone(),
                failure_codes: compatibility.failure_codes,
            });
        }
        Ok(())
    }

    pub(super) fn skill_fragment_compatibility_report_for_employee(
        &self,
        employee_uuid: Uuid,
        fragment_id: &SkillFragmentId,
    ) -> Result<SkillFragmentCompatibilityReport, GameError> {
        let profile = effective_combat_profile_for_employee(
            self.roster()?,
            self.inventory()?,
            &self.state.skill_fragments,
            &self.game_data,
            employee_uuid,
        )?;
        let metadata = self
            .game_data
            .skill_fragment_data
            .get_by_id(fragment_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "skill fragment '{}' is missing from static data",
                    fragment_id
                ))
            })?;
        Ok(metadata.compatibility_report(&profile))
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
        let Some(active_fragment_id) = employee.skill_fragments.active_fragment_id() else {
            return Ok(());
        };

        let profile = effective_combat_profile_for_employee_with_item_slot(
            employee,
            self.inventory()?,
            &self.state.skill_fragments,
            &self.game_data,
            projected_item_slot,
        )?;

        let metadata = self
            .game_data
            .skill_fragment_data
            .get_by_id(active_fragment_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "active skill fragment '{}' is missing from static data",
                    active_fragment_id
                ))
            })?;
        let compatibility = metadata.compatibility_report(&profile);
        if compatibility.is_compatible {
            Ok(())
        } else {
            Err(GameError::SkillFragmentIncompatible {
                fragment_id: active_fragment_id.clone(),
                failure_codes: compatibility.failure_codes,
            })
        }
    }

    pub(super) fn validate_unequip_skill_fragment_payload(
        &self,
        employee_uuid: Uuid,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        let employee = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if !employee
            .eligibility(crate::game::employee::EmployeeActivity::Loadout)
            .eligible
        {
            return Err(GameError::InvalidAction);
        }
        if !employee
            .skill_fragments
            .equipped_ids()
            .iter()
            .any(|id| id == fragment_id)
        {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }

    pub(super) fn is_loadout_action_context(&self) -> bool {
        (matches!(
            self.state.game_state,
            GameState::ViewingMap | GameState::NodeConfirm { .. }
        ) && !self.has_combat_recovery_restriction())
            || self.is_in_maintenance_node()
    }

    pub(super) fn is_consumable_use_context(&self) -> bool {
        matches!(
            self.state.game_state,
            GameState::ViewingMap | GameState::NodeConfirm { .. }
        )
    }

    pub(super) fn validate_upgrade_skill_fragment_payload(
        &self,
        target_fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_node() {
            return Err(GameError::InvalidAction);
        }
        self.state
            .skill_fragment_policy
            .composition
            .validate_dust_upgrade(
                &self.state.skill_fragments,
                target_fragment_id,
                &self.game_data.skill_fragment_data,
            )
            .map(|_| ())
    }

    pub(super) fn validate_awaken_skill_fragment_payload(
        &self,
        target_fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_node() {
            return Err(GameError::InvalidAction);
        }
        self.state
            .skill_fragment_policy
            .composition
            .validate_dust_awakening(
                &self.state.skill_fragments,
                target_fragment_id,
                &self.game_data.skill_fragment_data,
            )
            .map(|_| ())
    }

    pub(super) fn validate_dismantle_skill_fragment_payload(
        &self,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_node() {
            return Err(GameError::InvalidAction);
        }
        self.state
            .skill_fragment_policy
            .dismantle
            .validate_dismantle(
                &self.state.skill_fragments,
                fragment_id,
                &self.game_data.skill_fragment_data,
            )
            .map(|_| ())
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

        let recipe = self
            .game_data
            .equipment_data
            .get_dismantle_recipe_by_equipment_id(&equipment.meta.id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "missing equipment dismantle recipe for '{}'",
                    equipment.meta.id
                ))
            })?;

        let grant_effects = recipe
            .yields
            .iter()
            .map(|material| RewardEffect::GrantEquipmentMaterial {
                material_id: material.material_id.clone(),
                amount: material.amount,
            })
            .collect::<Vec<_>>();
        self.preview_grant_effects(&grant_effects).map(|_| ())
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
        if equipment.meta.bound {
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

        for cost in &recipe.costs_per_level {
            if inventory.equipment_materials.amount(&cost.material_id) < cost.amount {
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
        let old_state = self.state.game_state.clone();
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
        let starter_employee_count = self.run_policy().setup.starter_employee_count;
        let mut candidate_plans = Vec::with_capacity(starter_employee_count);
        let mut grant_effects = Vec::new();
        let mut equipment_assignments = Vec::new();
        let mut employee_uuids = Vec::with_capacity(starter_employee_count);
        for employee_definition_id in employee_definition_ids {
            let candidate = self
                .state
                .starter_candidates
                .iter()
                .find(|definition| definition.employee_definition_id == *employee_definition_id)
                .ok_or(GameError::InvalidAction)?
                .clone();
            let employee_uuid = self.state.uuid_manager.next_employee();
            employee_uuids.push(employee_uuid);
            let mut projected_employee = Employee::from_definition(employee_uuid, &candidate);
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
            candidate_plans.push((employee_uuid, candidate));
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

        let mut employees = Vec::with_capacity(starter_employee_count);
        for (employee_uuid, candidate) in candidate_plans {
            employees.push(Employee::from_definition(employee_uuid, &candidate));
        }

        let roster = self.roster_mut()?;
        roster.clear();
        for employee in employees {
            roster.add(employee);
        }

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
            let base_uuid = equipment.uuid;
            let equipment_type = equipment.equipment_type;
            let allow_duplicate_equip = equipment.allow_duplicate_equip;
            let roster = self.roster_mut()?;
            let employee = roster
                .get_mut(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            employee
                .loadout
                .item_slot
                .equip(
                    EquippedRef {
                        instance_uuid,
                        base_uuid,
                        equipment_type,
                    },
                    allow_duplicate_equip,
                )
                .map_err(|_| GameError::InvalidAction)?;
            let inventory = self.inventory_mut()?;
            inventory
                .equipments
                .get_item_mut(&instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?
                .equipped_to = Some(employee_uuid);
        }

        info!(
            "Initialized selected starter employee roster with {} employees",
            starter_employee_count
        );
        Ok(employee_uuids)
    }
}
