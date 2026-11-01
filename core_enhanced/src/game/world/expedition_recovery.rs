//! Recovery witnesses reuse the conditions, loadout planner and authored offers used by commands.
use super::GameCore;
use crate::game::{
    behavior::GameError,
    data::{
        consumable_data::ConsumableEffect, equipment_data::EquipmentType,
        run_policy_data::TraumaRecovery,
    },
    employee::{Employee, EmployeeActivity, EmployeeCondition, EmployeeRoster},
    map::{
        GameMode, MapNodeCategory, MapNodeId, MapNodePayload, MapProgression, RunMap,
        RunProgression,
    },
    recruitment::RecruitmentState,
    resources::{EquipmentLoadoutTransition, Inventory},
    skill_fragment::SkillFragmentInventory,
};

/// Every authority is borrowed from the same current or staged completion state.
#[derive(Clone, Copy)]
pub(super) struct RecoveryContext<'a> {
    pub map: &'a RunMap,
    pub navigation: &'a MapProgression,
    pub progression: &'a RunProgression,
    pub roster: &'a EmployeeRoster,
    pub inventory: &'a Inventory,
    pub fragments: &'a SkillFragmentInventory,
    pub recruitment: &'a RecruitmentState,
    pub enkephalin: u32,
}

impl GameCore {
    pub(super) fn current_recovery_context(&self) -> Result<RecoveryContext<'_>, GameError> {
        let run = self.run_state()?;
        Ok(RecoveryContext {
            map: &run.map,
            navigation: &run.map_progression,
            progression: &run.run_progression,
            roster: self.roster()?,
            inventory: self.inventory()?,
            fragments: &self.state.skill_fragments,
            recruitment: &run.recruitment,
            enkephalin: self.state.enkephalin.amount,
        })
    }

    fn physical_treatment_amount(inventory: &Inventory) -> u32 {
        inventory
            .consumables
            .iter()
            .filter_map(|stack| match stack.meta.effect {
                ConsumableEffect::ReducePhysicalDamage { amount } => {
                    Some(u32::from(amount).saturating_mul(stack.quantity))
                }
                _ => None,
            })
            .fold(0u32, u32::saturating_add)
    }

    fn condition_after_recovery(
        employee: &Employee,
        treatment: u32,
        recovery: Option<TraumaRecovery>,
    ) -> EmployeeCondition {
        let physical = if employee
            .eligibility(EmployeeActivity::PhysicalTreatment)
            .eligible
        {
            u32::from(employee.condition.physical_damage).saturating_sub(treatment) as u8
        } else {
            employee.condition.physical_damage
        };
        let trauma = recovery.map_or(employee.condition.trauma, |policy| {
            policy.apply(employee.condition.trauma)
        });
        EmployeeCondition::new(physical, trauma)
    }

    fn weapon_plan_compatibility(
        &self,
        context: RecoveryContext<'_>,
        employee: &Employee,
        item_uuid: uuid::Uuid,
    ) -> Option<bool> {
        let plan = EquipmentLoadoutTransition::plan_equip(
            context.inventory,
            context.roster,
            item_uuid,
            employee.uuid,
        )
        .ok()?;
        let Some(active_id) = employee.skill_fragments.active_copy_uuid() else {
            return Some(true);
        };
        let profile = crate::game::combat_setup::player_spawns::skill_fragment_compatibility_profile_for_employee_with_item_slot(
            employee, context.inventory, context.fragments, &self.game_data, plan.projected_target(),
        ).ok()?;
        let (active, definition, _) = context
            .fragments
            .resolve(&active_id, &self.game_data.skill_fragment_data)
            .ok()?;
        definition
            .form_compatibility_report(&active.selected_form, &profile)
            .map(|report| report.is_compatible)
    }

    fn employee_has_weapon_recovery_path(
        &self,
        context: RecoveryContext<'_>,
        employee: &Employee,
    ) -> bool {
        if employee.has_equipped_weapon() {
            return true;
        }
        context
            .inventory
            .equipments
            .iter()
            .filter(|copy| copy.meta.equipment_type == EquipmentType::Weapon)
            .any(|copy| {
                match self.weapon_plan_compatibility(context, employee, copy.instance_uuid) {
                    Some(true) => true,
                    // Public active-fragment removal opens this exact otherwise legal weapon plan.
                    Some(false) => {
                        employee.eligibility(EmployeeActivity::Loadout).eligible
                            && employee.skill_fragments.active_copy_uuid().is_some()
                            && context
                                .fragments
                                .validate_bindings(
                                    context.roster,
                                    &self.game_data.skill_fragment_data,
                                )
                                .is_ok()
                    }
                    None => false,
                }
            })
    }

    fn recovery_entry_cost(&self, context: RecoveryContext<'_>, node_id: MapNodeId) -> u32 {
        if context.progression.game_mode != GameMode::Expedition
            || context.progression.entry_receipts.contains(&node_id) { return 0; }
        context.map.node(node_id).and_then(|node| {
            self.run_state().ok().and_then(|run|
                self.regulated_node_entry_cost(node, &run.facility_run_state).ok())
        }).unwrap_or(0)
    }

    fn breakdown_dies_at_cost(&self, employee: &Employee, cost: u32) -> bool {
        employee.is_alive() && employee.condition.is_breakdown() && cost > 0
            && cost >= employee.breakdown.remaining_cost()
                .unwrap_or(self.run_policy().corrosion.breakdown_collapse_cost_limit)
    }

    fn roster_can_recover_deployability(
        &self,
        context: RecoveryContext<'_>,
        recovery: Option<TraumaRecovery>,
        entry_cost: u32,
    ) -> bool {
        let treatment = Self::physical_treatment_amount(context.inventory);
        context.roster.iter().any(|employee| {
            !self.breakdown_dies_at_cost(employee, entry_cost)
                && Self::condition_after_recovery(employee, treatment, recovery)
                .eligibility(EmployeeActivity::Combat)
                .eligible
                && self.employee_has_weapon_recovery_path(context, employee)
        })
    }

    fn employee_condition_has_recovery_path(
        &self,
        context: RecoveryContext<'_>,
        employee: &Employee,
    ) -> bool {
        let treatment = Self::physical_treatment_amount(context.inventory);
        if Self::condition_after_recovery(employee, treatment, None)
            .eligibility(EmployeeActivity::Combat)
            .eligible
        {
            return true;
        }
        let Ok(template) = self.facility_template_for_map(context.map) else {
            return false;
        };
        context
            .navigation
            .selectable_node_ids(context.map, &template)
            .into_iter()
            .any(|id| {
                let policy = match context
                    .map
                    .node(id)
                    .map(|node| (&node.category, &node.payload))
                {
                    Some((MapNodeCategory::SavePoint, MapNodePayload::SavePoint)) => {
                        self.run_policy().save_point_trauma_recovery
                    }
                    Some((MapNodeCategory::Gate, MapNodePayload::None)) => {
                        self.run_policy().gate_trauma_recovery
                    }
                    _ => return false,
                };
                !self.breakdown_dies_at_cost(employee, self.recovery_entry_cost(context, id))
                    && Self::condition_after_recovery(employee, treatment, Some(policy))
                    .eligibility(EmployeeActivity::Combat)
                    .eligible
            })
    }

    pub(super) fn recovery_treatment_payload_is_useful(
        &self,
        employee: &Employee,
        amount: u8,
    ) -> bool {
        amount > 0
            && employee.condition.physical_damage > 0
            && employee
                .eligibility(EmployeeActivity::PhysicalTreatment)
                .eligible
            && self.current_recovery_context().is_ok_and(|context| {
                self.employee_condition_has_recovery_path(context, employee)
                    && self.employee_has_weapon_recovery_path(context, employee)
            })
    }

    pub(super) fn recovery_weapon_payload_is_useful(&self, employee: &Employee) -> bool {
        !employee.has_equipped_weapon()
            && self
                .current_recovery_context()
                .is_ok_and(|context| self.employee_condition_has_recovery_path(context, employee))
    }

    pub(super) fn recovery_fragment_removal_payload_is_useful(&self, employee: &Employee) -> bool {
        !employee.has_equipped_weapon()
            && self.current_recovery_context().is_ok_and(|context| {
                self.employee_condition_has_recovery_path(context, employee)
                    && context
                        .inventory
                        .equipments
                        .iter()
                        .filter(|copy| copy.meta.equipment_type == EquipmentType::Weapon)
                        .any(|copy| {
                            self.weapon_plan_compatibility(context, employee, copy.instance_uuid)
                                == Some(false)
                        })
            })
    }

    /// A Continue payload must perform recovery rather than relying on the available terminal return.
    pub(super) fn recovery_node_is_useful(
        &self,
        context: RecoveryContext<'_>,
        node_id: MapNodeId,
        allow_return: bool,
    ) -> bool {
        let Some(node) = context.map.node(node_id) else {
            return false;
        };
        let depth = context.progression.floor_index().saturating_add(1);
        let expedition = context.progression.game_mode == GameMode::Expedition;
        let exhaustion_return = allow_return
            && expedition
            && depth >= 3
            && !matches!(
                node.category,
                MapNodeCategory::Start
                    | MapNodeCategory::Gate
                    | MapNodeCategory::Combat
                    | MapNodeCategory::Boss
            )
            && context.progression.deterrence > 0
            && context.progression.deterrence <= self.recovery_entry_cost(context, node_id)
            && !context.progression.entry_receipts.contains(&node_id);
        exhaustion_return
            || match (&node.category, &node.payload) {
                (MapNodeCategory::SavePoint, MapNodePayload::SavePoint) => self
                    .roster_can_recover_deployability(
                        context,
                        Some(self.run_policy().save_point_trauma_recovery),
                        self.recovery_entry_cost(context, node_id),
                    ),
                (MapNodeCategory::Gate, MapNodePayload::None) => {
                    (allow_return && expedition && depth >= 3)
                        || self.roster_can_recover_deployability(
                            context,
                            Some(self.run_policy().gate_trauma_recovery),
                            0,
                        )
                }
                (
                    MapNodeCategory::HeadquartersContact,
                    MapNodePayload::HeadquartersContact {
                        candidate_count, ..
                    },
                ) => {
                    let policy = self.run_policy().headquarters;
                    depth <= policy.recruit_max_depth
                        && policy.recruit_limit_per_session > 0
                        && context.roster.len() < crate::game::employee::MAX_EMPLOYEE_ROSTER_SIZE
                        && context.enkephalin >= policy.recruit_price
                        && match context
                            .recruitment
                            .active_session()
                            .filter(|session| session.node_id == node_id)
                        {
                            Some(session) => {
                                session.recruits_used < policy.recruit_limit_per_session
                                    && session
                                        .employee_definition_ids
                                        .iter()
                                        .any(|id| self.recruitment_definition_is_deployable(id))
                            }
                            None => context
                                .recruitment
                                .queue_definition_ids()
                                .take(*candidate_count)
                                .any(|id| self.recruitment_definition_is_deployable(id)),
                        }
                }
                _ => false,
            }
    }

    pub(super) fn recovery_path_in_context(
        &self,
        context: RecoveryContext<'_>,
    ) -> (Vec<MapNodeId>, bool) {
        let Ok(template) = self.facility_template_for_map(context.map) else {
            return (Vec::new(), false);
        };
        let allowed = context
            .navigation
            .selectable_node_ids(context.map, &template)
            .into_iter()
            .filter(|id| self.recovery_node_is_useful(context, *id, true))
            .collect::<Vec<_>>();
        let path_exists =
            !allowed.is_empty() || self.roster_can_recover_deployability(context, None, 0);
        (allowed, path_exists)
    }

    pub(super) fn recruitment_definition_is_deployable(&self, id: &str) -> bool {
        self.game_data
            .employee_definition_data
            .get_by_id(id)
            .is_some_and(|definition| {
                definition
                    .initial_loadout
                    .equipment_ids
                    .iter()
                    .any(|equipment_id| {
                        self.game_data
                            .equipment_data
                            .get_by_id(equipment_id)
                            .is_some_and(|equipment| {
                                equipment.equipment_type == EquipmentType::Weapon
                                    && equipment.weapon_profile.is_some()
                            })
                    })
            })
    }
}
