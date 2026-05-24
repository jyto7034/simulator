use super::{GameCore, RUN_SYSTEM_POLICY};
use crate::game::behavior::{ActionKind, BehaviorResult, GameError, MaintenanceOptionsDto};
use crate::game::employee::EmployeeLifeState;
use crate::game::employee_trust::{EmployeeTrustResolver, TrustEvent, TrustEventKind};
use crate::game::map::{MapProgression, MedicalTreatmentKind, RunMap, SupportNodeType};
use crate::game::resources::SupportSessionState;
use uuid::Uuid;

impl GameCore {
    pub(super) fn handle_choose_support(
        &mut self,
        support_type: SupportNodeType,
    ) -> Result<BehaviorResult, GameError> {
        {
            let support = self
                .state
                .selected_event
                .as_mut()
                .ok_or(GameError::InvalidAction)?
                .as_support_mut()?;
            support.select(support_type)?;
        }
        let candidates = self.support_target_candidates(support_type)?;
        let support = {
            let support = self
                .state
                .selected_event
                .as_mut()
                .ok_or(GameError::InvalidAction)?
                .as_support_mut()?;
            support.set_target_candidates(candidates);
            support.clone()
        };
        let result = self.support_state_result(&support);
        self.refresh_maintenance_action_gate(&support)?;

        Ok(result)
    }

    pub(super) fn handle_select_support_target(
        &mut self,
        employee_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let support = {
            let support = self
                .state
                .selected_event
                .as_mut()
                .ok_or(GameError::InvalidAction)?
                .as_support_mut()?;
            support.select_target(employee_uuid)?;
            support.clone()
        };

        Ok(self.support_state_result(&support))
    }

    pub(super) fn handle_select_medical_treatment(
        &mut self,
        treatment: MedicalTreatmentKind,
    ) -> Result<BehaviorResult, GameError> {
        let support = {
            let support = self
                .state
                .selected_event
                .as_mut()
                .ok_or(GameError::InvalidAction)?
                .as_support_mut()?;
            support.select_medical_treatment(treatment)?;
            support.clone()
        };

        Ok(self.support_state_result(&support))
    }

    pub(super) fn support_state_result(&self, support: &SupportSessionState) -> BehaviorResult {
        self.support_state_result_with_deliveries(support, vec![])
    }

    pub(super) fn support_state_result_with_deliveries(
        &self,
        support: &SupportSessionState,
        research_deliveries: Vec<crate::game::skill_fragment::SkillFragmentResearchDelivery>,
    ) -> BehaviorResult {
        BehaviorResult::SupportState {
            node_id: support.node_id,
            support_mode: support.support_mode,
            support_type: support.support_type,
            choices: support.choices.clone(),
            selected_support_type: support.selected_support_type,
            target_candidates: support.target_candidates.clone(),
            selected_employee_uuid: support.selected_employee_uuid,
            selected_medical_treatment: support.selected_medical_treatment,
            maintenance_options: self.maintenance_options_for_support(support),
            research_deliveries,
        }
    }

    pub(super) fn maintenance_options_for_support(
        &self,
        support: &SupportSessionState,
    ) -> Option<MaintenanceOptionsDto> {
        if support.resolved_support_type().ok()? != SupportNodeType::Maintenance {
            return None;
        }

        let inventory = self.inventory().ok()?;
        let protected_fragment_ids = self.protected_skill_fragment_material_ids().ok()?;

        let restorable_equipment_recipes = self
            .game_data
            .equipment_data
            .restoration_recipes
            .iter()
            .filter(|recipe| {
                inventory.equipments.can_add_item()
                    && recipe.costs.iter().all(|cost| {
                        inventory.equipment_materials.amount(&cost.material_id) >= cost.amount
                    })
            })
            .cloned()
            .collect::<Vec<_>>();

        let mut dismantle_equipment_item_uuids = Vec::new();
        let mut dismantle_recipes = Vec::new();
        let mut enhance_equipment_item_uuids = Vec::new();
        let mut enhancement_recipes = Vec::new();
        for equipment in inventory.equipments.iter() {
            if equipment.equipped_to.is_none() {
                if let Some(recipe) = self
                    .game_data
                    .equipment_data
                    .get_dismantle_recipe_by_equipment_id(&equipment.meta.id)
                {
                    dismantle_equipment_item_uuids.push(equipment.instance_uuid);
                    dismantle_recipes.push(recipe.clone());
                }
            }

            if let Some(recipe) = self
                .game_data
                .equipment_data
                .get_enhancement_recipe_by_equipment_id(&equipment.meta.id)
            {
                let has_materials = equipment.enhancement_level < recipe.max_level
                    && recipe.costs_per_level.iter().all(|cost| {
                        inventory.equipment_materials.amount(&cost.material_id) >= cost.amount
                    });
                if has_materials {
                    enhance_equipment_item_uuids.push(equipment.instance_uuid);
                    enhancement_recipes.push(recipe.clone());
                }
            }
        }
        dismantle_equipment_item_uuids.sort();
        enhance_equipment_item_uuids.sort();
        dismantle_recipes.sort_by(|left, right| left.equipment_id.cmp(&right.equipment_id));
        enhancement_recipes.sort_by(|left, right| left.equipment_id.cmp(&right.equipment_id));

        let mut dismantle_skill_fragment_ids = self
            .state
            .skill_fragments
            .owned_ids()
            .filter(|fragment_id| {
                self.state
                    .skill_fragment_policy
                    .dismantle
                    .validate_dismantle(
                        &self.state.skill_fragments,
                        fragment_id,
                        &self.game_data.skill_fragment_data,
                        &protected_fragment_ids,
                    )
                    .is_ok()
            })
            .cloned()
            .collect::<Vec<_>>();
        dismantle_skill_fragment_ids.sort();

        Some(MaintenanceOptionsDto {
            restorable_equipment_recipes,
            dismantle_equipment_item_uuids,
            enhance_equipment_item_uuids,
            enhancement_recipes,
            dismantle_recipes,
            dismantle_skill_fragment_ids,
        })
    }

    pub(super) fn refresh_support_target_candidates(
        &self,
        support: &mut SupportSessionState,
    ) -> Result<(), GameError> {
        let support_type = support.resolved_support_type().ok();
        let candidates = support_type
            .map(|support_type| self.support_target_candidates(support_type))
            .transpose()?
            .unwrap_or_default();
        support.set_target_candidates(candidates);
        Ok(())
    }

    fn support_target_candidates(
        &self,
        support_type: SupportNodeType,
    ) -> Result<Vec<Uuid>, GameError> {
        let roster = self.roster()?;
        let mut candidates = match support_type {
            SupportNodeType::Medical => roster
                .iter()
                .filter(|employee| employee.life_state == EmployeeLifeState::Alive)
                .filter(|employee| {
                    employee.health.current_hp < employee.health.max_hp
                        || employee.trauma > 0
                        || !employee.injuries.is_empty()
                })
                .map(|employee| employee.uuid)
                .collect::<Vec<_>>(),
            SupportNodeType::Rest | SupportNodeType::Maintenance => Vec::new(),
        };
        candidates.sort();
        Ok(candidates)
    }

    pub(super) fn default_support_choices() -> Vec<SupportNodeType> {
        vec![
            SupportNodeType::Medical,
            SupportNodeType::Rest,
            SupportNodeType::Maintenance,
        ]
    }

    pub(super) fn refresh_maintenance_action_gate(
        &mut self,
        support: &SupportSessionState,
    ) -> Result<(), GameError> {
        let is_maintenance = support
            .resolved_support_type()
            .is_ok_and(|support_type| support_type == SupportNodeType::Maintenance);
        if !is_maintenance {
            return Ok(());
        }

        let mut allowed = self.state.action_validator.allowed_actions();
        for action in [
            ActionKind::UpgradeSkillFragment,
            ActionKind::AwakenSkillFragment,
            ActionKind::DismantleSkillFragment,
            ActionKind::RestoreEquipment,
            ActionKind::DismantleEquipment,
            ActionKind::EnhanceEquipment,
        ] {
            if !allowed.contains(&action) {
                allowed.push(action);
            }
        }
        self.state.action_validator.set_allowed_actions(allowed);
        Ok(())
    }

    pub(super) fn is_in_maintenance_support_node(&self) -> bool {
        self.state
            .selected_event
            .as_ref()
            .and_then(|selected| selected.as_support().ok())
            .and_then(|support| support.resolved_support_type().ok())
            .is_some_and(|support_type| support_type == SupportNodeType::Maintenance)
    }

    pub(super) fn apply_current_support_node_effect(
        &mut self,
        map: &RunMap,
        progression: &MapProgression,
    ) -> Result<(), GameError> {
        let Some(node_id) = progression.current_node_id else {
            return Ok(());
        };
        let Some(node) = map.node(node_id) else {
            return Ok(());
        };
        let support_session = self
            .state
            .selected_event
            .as_ref()
            .and_then(|selected| selected.as_support().ok())
            .filter(|support| support.node_id == node_id);

        let Some(support_session) = support_session else {
            if matches!(
                node.payload,
                crate::game::map::MapNodePayload::Support { .. }
            ) {
                return Err(GameError::InvalidAction);
            }
            return Ok(());
        };
        let support_type = support_session.resolved_support_type()?;
        if support_session.needs_target_selection()? {
            return Err(GameError::InvalidAction);
        }
        if support_session.needs_medical_treatment_selection()? {
            return Err(GameError::InvalidAction);
        }
        let selected_employee_uuid = support_session.selected_employee_uuid;
        let selected_medical_treatment = support_session.selected_medical_treatment;

        match support_type {
            SupportNodeType::Medical => {
                self.support_medical_heal(selected_employee_uuid, selected_medical_treatment)
            }
            SupportNodeType::Rest => self.support_rest(),
            SupportNodeType::Maintenance => Ok(()),
        }
    }

    fn support_medical_heal(
        &mut self,
        target_employee_uuid: Option<Uuid>,
        treatment: Option<MedicalTreatmentKind>,
    ) -> Result<(), GameError> {
        let trust_policy = self.state.employee_trust_policy.clone();
        let roster = self.roster_mut()?;
        let Some(target_employee_uuid) = target_employee_uuid else {
            return Ok(());
        };
        let treatment = treatment.ok_or(GameError::InvalidAction)?;
        let employee = roster
            .get_mut(&target_employee_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if employee.life_state != EmployeeLifeState::Alive {
            return Err(GameError::InvalidAction);
        }

        match treatment {
            MedicalTreatmentKind::EmergencyCare => {
                employee
                    .health
                    .restore_hp_percent(RUN_SYSTEM_POLICY.support_medical_hp_heal_percent);
            }
            MedicalTreatmentKind::Counseling => {
                employee.trauma = employee
                    .trauma
                    .saturating_sub(RUN_SYSTEM_POLICY.support_medical_trauma_heal);
            }
            MedicalTreatmentKind::BalancedCare => {
                employee
                    .health
                    .restore_hp_percent(RUN_SYSTEM_POLICY.support_medical_balanced_hp_heal_percent);
                employee.trauma = employee
                    .trauma
                    .saturating_sub(RUN_SYSTEM_POLICY.support_medical_trauma_heal / 2);
            }
        }

        let reaction = EmployeeTrustResolver::apply_event(
            TrustEvent::new(employee.uuid, TrustEventKind::TreatedAfterIncapacitation),
            &trust_policy,
        );
        employee.trust.apply_reaction(&reaction);
        Ok(())
    }

    fn support_rest(&mut self) -> Result<(), GameError> {
        let trust_policy = self.state.employee_trust_policy.clone();
        let roster = self.roster_mut()?;
        for employee in roster
            .iter_mut()
            .filter(|employee| employee.life_state == EmployeeLifeState::Alive)
        {
            employee.trauma = employee
                .trauma
                .saturating_sub(RUN_SYSTEM_POLICY.support_rest_trauma_heal);
            let reaction = EmployeeTrustResolver::apply_event(
                TrustEvent::new(employee.uuid, TrustEventKind::RestedAtRecoveryNode),
                &trust_policy,
            );
            employee.trust.apply_reaction(&reaction);
        }
        Ok(())
    }
}
