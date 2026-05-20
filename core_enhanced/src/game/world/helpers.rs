use std::collections::HashSet;

use tracing::{debug, info};
use uuid::Uuid;

use super::{GameCore, RUN_SYSTEM_POLICY};
use crate::game::behavior::{ActionKind, BehaviorResult, GameError, PlayerBehavior};
use crate::game::data::skill_fragment_data::SkillFragmentId;
use crate::game::employee::{Employee, EmployeeRoster};
use crate::game::enums::RewardMode;
use crate::game::managers::action_scheduler::ActionScheduler;
use crate::game::map::MapNodeId;
use crate::game::resources::{
    Bench, EquippedItemDto, Field, GameState, Inventory, RewardSessionState,
};
use crate::game::reward::RewardOption;

impl GameCore {
    pub(super) fn inventory(&self) -> Result<&Inventory, GameError> {
        Ok(&self.state.inventory)
    }

    pub(super) fn inventory_mut(&mut self) -> Result<&mut Inventory, GameError> {
        Ok(&mut self.state.inventory)
    }

    pub(super) fn field(&self) -> Result<&Field, GameError> {
        Ok(&self.state.field)
    }

    pub(super) fn bench(&self) -> Result<&Bench, GameError> {
        Ok(&self.state.bench)
    }

    pub(super) fn bench_mut(&mut self) -> Result<&mut Bench, GameError> {
        Ok(&mut self.state.bench)
    }

    pub(super) fn roster(&self) -> Result<&EmployeeRoster, GameError> {
        Ok(&self.state.roster)
    }

    pub(super) fn roster_mut(&mut self) -> Result<&mut EmployeeRoster, GameError> {
        Ok(&mut self.state.roster)
    }

    pub(super) fn available_player_unit_ids(&self) -> Result<Vec<Uuid>, GameError> {
        let roster = self.roster()?;
        Ok(roster.available_employee_ids())
    }

    pub(super) fn sync_bench_with_owned_units(&mut self) -> Result<(), GameError> {
        let owned_units = self.available_player_unit_ids()?;

        let field_units = {
            let field = self.field()?;
            field
                .unit_positions
                .keys()
                .copied()
                .collect::<std::collections::HashSet<_>>()
        };

        let bench = self.bench_mut()?;
        bench.sync_owned_units(&owned_units, &field_units);
        Ok(())
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

    pub(super) fn reward_state_result_with_deliveries(
        &self,
        reward: &RewardSessionState,
        research_deliveries: Vec<crate::game::skill_fragment::SkillFragmentResearchDelivery>,
    ) -> BehaviorResult {
        BehaviorResult::RewardState {
            mode: reward.mode,
            rewards: reward.rewards.clone(),
            selected_reward_uuid: reward.selected_reward_uuid,
            research_deliveries,
        }
    }

    pub(super) fn current_reward_can_skip(&self) -> bool {
        self.state
            .selected_event
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

    pub(super) fn validate_behavior_payload(
        &self,
        behavior: &PlayerBehavior,
    ) -> Result<(), GameError> {
        match behavior {
            PlayerBehavior::StartNewGame
            | PlayerBehavior::RequestMapData
            | PlayerBehavior::UseReconScan
            | PlayerBehavior::ConfirmEnterNode
            | PlayerBehavior::CancelSelectedNode
            | PlayerBehavior::CompleteNode
            | PlayerBehavior::ChooseSupport { .. }
            | PlayerBehavior::SelectSupportTarget { .. }
            | PlayerBehavior::SelectMedicalTreatment { .. }
            | PlayerBehavior::RerollShop
            | PlayerBehavior::ExitShop
            | PlayerBehavior::ClaimReward
            | PlayerBehavior::ExitReward
            | PlayerBehavior::FinishCombatReplay
            | PlayerBehavior::UnEquipItem { .. } => Ok(()),
            PlayerBehavior::SelectStarterEmployees { candidate_ids } => {
                self.validate_starter_employee_selection(candidate_ids)
            }
            PlayerBehavior::SelectReward { reward_id } => {
                self.validate_select_reward_payload(*reward_id)
            }
            PlayerBehavior::SelectMapNode { node_id } => self.validate_select_map_node(*node_id),
            PlayerBehavior::EquipItem {
                item_uuid,
                target_unit,
            } => self.validate_equip_item_payload(*item_uuid, *target_unit),
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                fragment_id,
            } => self.validate_equip_skill_fragment_payload(*employee_uuid, fragment_id),
            PlayerBehavior::UnequipSkillFragment {
                employee_uuid,
                fragment_id,
            } => self.validate_unequip_skill_fragment_payload(*employee_uuid, fragment_id),
            PlayerBehavior::UpgradeSkillFragment {
                target_fragment_id,
                material_fragment_id,
            } => self
                .validate_upgrade_skill_fragment_payload(target_fragment_id, material_fragment_id),
            PlayerBehavior::AwakenSkillFragment {
                target_fragment_id,
                material_fragment_ids,
            } => self
                .validate_awaken_skill_fragment_payload(target_fragment_id, material_fragment_ids),
            PlayerBehavior::DismantleSkillFragment { fragment_id } => {
                self.validate_dismantle_skill_fragment_payload(fragment_id)
            }
            PlayerBehavior::RestoreEquipment { recipe_id } => {
                self.validate_restore_equipment_payload(recipe_id)
            }
            PlayerBehavior::DismantleEquipment { item_uuid } => {
                self.validate_dismantle_equipment_payload(*item_uuid)
            }
            PlayerBehavior::EnhanceEquipment { item_uuid } => {
                self.validate_enhance_equipment_payload(*item_uuid)
            }
            PlayerBehavior::MoveUnit {
                target_unit_uuid, ..
            } => self.validate_owned_unit_exists(*target_unit_uuid),
            PlayerBehavior::MoveBenchUnit {
                target_unit_uuid, ..
            } => self.validate_owned_unit_exists(*target_unit_uuid),
            PlayerBehavior::PurchaseItem { .. } | PlayerBehavior::SellItem { .. } => Ok(()),
        }
    }

    pub(super) fn validate_select_reward_payload(&self, reward_id: Uuid) -> Result<(), GameError> {
        let selected = self
            .state
            .selected_event
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
        let run = self.run_state()?;
        if run.map_progression.available_node_ids.contains(&node_id) {
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
        let inventory = self.inventory()?;

        let owned_equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;

        if owned_equipment.equipped_to.is_some() {
            return Err(GameError::InvalidAction);
        }

        let roster = self.roster()?;
        let employee = roster.get(&target_unit).ok_or(GameError::UnitNotFound)?;
        if employee.is_available_for_combat() {
            return Ok(());
        }
        Err(GameError::InvalidAction)
    }

    pub(super) fn validate_equip_skill_fragment_payload(
        &self,
        employee_uuid: Uuid,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        let employee = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if !employee.is_available_for_combat() {
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
        Ok(())
    }

    pub(super) fn validate_unequip_skill_fragment_payload(
        &self,
        employee_uuid: Uuid,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        let employee = self
            .roster()?
            .get(&employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if !employee.is_available_for_combat() {
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

    pub(super) fn validate_upgrade_skill_fragment_payload(
        &self,
        target_fragment_id: &SkillFragmentId,
        material_fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_support_node() {
            return Err(GameError::InvalidAction);
        }
        let protected_material_ids = self.protected_skill_fragment_material_ids()?;
        self.state
            .skill_fragment_policy
            .composition
            .validate_material_upgrade(
                &self.state.skill_fragments,
                target_fragment_id,
                material_fragment_id,
                &self.game_data.skill_fragment_data,
                &protected_material_ids,
            )
            .map(|_| ())
    }

    pub(super) fn validate_awaken_skill_fragment_payload(
        &self,
        target_fragment_id: &SkillFragmentId,
        material_fragment_ids: &[SkillFragmentId],
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_support_node() {
            return Err(GameError::InvalidAction);
        }
        let protected_material_ids = self.protected_skill_fragment_material_ids()?;
        self.state
            .skill_fragment_policy
            .composition
            .validate_awakening(
                &self.state.skill_fragments,
                target_fragment_id,
                material_fragment_ids,
                &self.game_data.skill_fragment_data,
                &protected_material_ids,
            )
            .map(|_| ())
    }

    pub(super) fn validate_dismantle_skill_fragment_payload(
        &self,
        fragment_id: &SkillFragmentId,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_support_node() {
            return Err(GameError::InvalidAction);
        }
        let protected_material_ids = self.protected_skill_fragment_material_ids()?;
        self.state
            .skill_fragment_policy
            .dismantle
            .validate_dismantle(
                &self.state.skill_fragments,
                fragment_id,
                &self.game_data.skill_fragment_data,
                &protected_material_ids,
            )
            .map(|_| ())
    }

    pub(super) fn validate_restore_equipment_payload(
        &self,
        recipe_id: &str,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_support_node() {
            return Err(GameError::InvalidAction);
        }

        let recipe = self
            .game_data
            .equipment_data
            .get_restoration_recipe_by_id(recipe_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "unknown equipment restoration recipe '{recipe_id}'"
                ))
            })?;
        let inventory = self.inventory()?;
        if !inventory.equipments.can_add_item() {
            return Err(GameError::InventoryFull);
        }

        for cost in &recipe.costs {
            if inventory.equipment_materials.amount(&cost.material_id) < cost.amount {
                return Err(GameError::InvalidAction);
            }
        }

        Ok(())
    }

    pub(super) fn validate_dismantle_equipment_payload(
        &self,
        item_uuid: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_support_node() {
            return Err(GameError::InvalidAction);
        }

        let inventory = self.inventory()?;
        let equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        if equipment.equipped_to.is_some() {
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

        for material in &recipe.yields {
            let current = inventory.equipment_materials.amount(&material.material_id);
            if current > u32::MAX.saturating_sub(material.amount) {
                return Err(GameError::InvalidAction);
            }
        }

        Ok(())
    }

    pub(super) fn validate_enhance_equipment_payload(
        &self,
        item_uuid: Uuid,
    ) -> Result<(), GameError> {
        if !self.is_in_maintenance_support_node() {
            return Err(GameError::InvalidAction);
        }

        let inventory = self.inventory()?;
        let equipment = inventory
            .equipments
            .get_item(&item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
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

    pub(super) fn protected_skill_fragment_material_ids(
        &self,
    ) -> Result<Vec<SkillFragmentId>, GameError> {
        Ok(self
            .roster()?
            .iter()
            .flat_map(|employee| employee.skill_fragments.equipped_ids())
            .collect())
    }

    pub(super) fn validate_owned_unit_exists(
        &self,
        target_unit_uuid: Uuid,
    ) -> Result<(), GameError> {
        let roster = self.roster()?;
        let employee = roster
            .get(&target_unit_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if employee.is_available_for_combat() {
            return Ok(());
        }
        Err(GameError::InvalidAction)
    }

    /// 게임 상태 전환
    ///
    /// # Arguments
    /// * `new_state` - 전환할 새로운 상태
    ///
    /// # Effects
    /// 1. GameCoreState의 GameState 업데이트
    /// 2. ActionScheduler를 통해 allowed_actions 자동 업데이트
    pub(super) fn transition_to(&mut self, new_state: GameState) -> Result<(), GameError> {
        let old_state = self.state.game_state.clone();
        info!("Game state transition: {:?} -> {:?}", old_state, new_state);

        let mut allowed = ActionScheduler::get_allowed_actions(&new_state);
        if matches!(new_state, GameState::InReward { .. }) && !self.current_reward_can_skip() {
            allowed.retain(|action| *action != ActionKind::ExitReward);
        }

        self.state.transition_to(new_state, allowed);

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
        candidate_ids: &[String],
    ) -> Result<(), GameError> {
        if candidate_ids.len() != RUN_SYSTEM_POLICY.starter_employee_count {
            return Err(GameError::InvalidAction);
        }
        let mut seen = HashSet::new();
        if !candidate_ids.iter().all(|id| seen.insert(id)) {
            return Err(GameError::InvalidAction);
        }
        let candidates = self
            .state
            .starter_candidates
            .iter()
            .map(|candidate| candidate.id.as_str())
            .collect::<HashSet<_>>();
        if candidate_ids
            .iter()
            .any(|candidate_id| !candidates.contains(candidate_id.as_str()))
        {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }

    pub(super) fn initialize_selected_starter_employees(
        &mut self,
        candidate_ids: &[String],
    ) -> Result<Vec<Uuid>, GameError> {
        self.validate_starter_employee_selection(candidate_ids)?;
        let mut employees = Vec::with_capacity(RUN_SYSTEM_POLICY.starter_employee_count);
        let mut employee_uuids = Vec::with_capacity(RUN_SYSTEM_POLICY.starter_employee_count);
        for candidate_id in candidate_ids {
            let candidate = self
                .state
                .starter_candidates
                .iter()
                .find(|candidate| candidate.id == *candidate_id)
                .ok_or(GameError::InvalidAction)?
                .clone();
            let employee_uuid = self.state.uuid_manager.next_employee();
            employee_uuids.push(employee_uuid);
            employees.push(Employee::from_starter_candidate(employee_uuid, &candidate));
        }

        let roster = self.roster_mut()?;
        roster.clear();
        for employee in employees {
            roster.add(employee);
        }

        info!(
            "Initialized selected starter employee roster with {} employees",
            RUN_SYSTEM_POLICY.starter_employee_count
        );
        Ok(employee_uuids)
    }
}
