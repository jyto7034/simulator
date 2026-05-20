use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{
    battle::{
        ids::UnitInstanceId,
        scenario::{TacticalGroupMembers, TacticalGroupPlanId},
    },
    behavior::GameError,
};

use super::{
    movement::{types::UnitBody, ActionState},
    BattleCore, RuntimeArtifact, RuntimeItem, RuntimeUnit,
};

impl BattleCore {
    pub(super) fn build_runtime_artifacts_from_scenario(&mut self) -> Result<(), GameError> {
        let mut seen_artifacts: HashSet<(crate::game::enums::Side, Uuid)> = HashSet::new();
        for artifact in &self.scenario.artifacts {
            if !seen_artifacts.insert((artifact.side, artifact.base_uuid)) {
                return Err(GameError::InvalidAction);
            }
            let instance_id = Self::make_artifact_instance_id(
                artifact.base_uuid,
                artifact.side,
                artifact.instance_salt,
            );
            if self
                .artifacts
                .insert(
                    instance_id,
                    RuntimeArtifact {
                        instance_id,
                        owner: artifact.side,
                        base_uuid: artifact.base_uuid,
                    },
                )
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }
        }

        Ok(())
    }

    pub(super) fn spawn_scenario_group(
        &mut self,
        group_id: &crate::game::battle::scenario::ScenarioGroupId,
    ) -> Result<Vec<UnitInstanceId>, GameError> {
        if self.scenario_runtime.spawned_groups.contains(group_id) {
            return Ok(Vec::new());
        }
        let group = self
            .scenario
            .group(group_id)
            .cloned()
            .ok_or(GameError::InvalidAction)?;

        let artifact_base_uuids: Vec<Uuid> = self
            .scenario
            .artifacts
            .iter()
            .filter(|artifact| artifact.side == group.side)
            .map(|artifact| artifact.base_uuid)
            .collect();

        let mut spawned_unit_ids = Vec::new();
        for spawn in &group.spawns {
            let unit = &spawn.draft;
            let combat_profile = unit.combat_profile(&self.game_data)?;
            let mut stats = unit.effective_stats(&self.game_data, &artifact_base_uuids)?;

            let base_uuid = unit.base_uuid();
            let instance_id = Self::make_instance_id(base_uuid, group.side, spawn.instance_salt);
            let resonance_start = combat_profile.resonance.start;
            let resonance_max = combat_profile.resonance.max.max(1);
            let resonance_lock_ms = combat_profile.resonance.gain_lock_ms;
            let resonance_current = resonance_start.min(resonance_max);
            let move_speed_units_per_ms = combat_profile.movement.speed_units_per_ms;
            stats.move_speed_units_per_ms = move_speed_units_per_ms;
            let spawn_position =
                super::movement::types::WorldVec2::from_tile_center(spawn.position);
            let radius = combat_profile.movement.radius_units as f32
                / super::movement::types::DATA_UNITS_PER_WORLD;
            let body = UnitBody::new_at(
                spawn_position,
                radius,
                move_speed_units_per_ms as f32 * 1_000.0
                    / super::movement::types::DATA_UNITS_PER_WORLD,
            );
            let tactical_anchor = Some(spawn_position);
            let tactical_group_id =
                self.tactical_group_for_spawn(&group.id, &spawn.unit_ref, group.side);
            if self
                .units
                .insert(
                    instance_id,
                    RuntimeUnit {
                        instance_id,
                        source_owned_uuid: unit.owned_uuid,
                        owner: group.side,
                        role: unit.source.role(),
                        base_uuid,
                        stats,
                        basic_attack: combat_profile.basic_attack,
                        skill_id: combat_profile.skill_id,
                        body,
                        tactical_anchor,
                        tactical_group_id: tactical_group_id.clone(),
                        move_epoch: 0,
                        action_state: ActionState::Idle,
                        action_locks: Default::default(),
                        current_target: None,
                        next_basic_attack_ms: 0,
                        pending_basic_attack: false,
                        resonance_current,
                        resonance_max,
                        resonance_lock_ms,
                        next_action_time: 0,
                        pending_cast: false,
                        pending_cast_cause: None,
                        pending_skill_cast: None,
                    },
                )
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }

            self.battlefield.place(instance_id, spawn.position)?;
            self.scenario_runtime
                .unit_refs
                .insert(spawn.unit_ref.clone(), instance_id);
            if let Some(tactical_group_id) = tactical_group_id {
                self.scenario_runtime
                    .tactical_groups
                    .entry(tactical_group_id)
                    .or_default()
                    .push(instance_id);
            }
            spawned_unit_ids.push(instance_id);

            for (idx, equipment_uuid) in unit.equipped_items.iter().enumerate() {
                let item_instance_id = Self::make_item_instance_id(
                    *equipment_uuid,
                    group.side,
                    instance_id,
                    idx as u32,
                );
                self.items.insert(
                    item_instance_id,
                    RuntimeItem {
                        instance_id: item_instance_id,
                        owner: group.side,
                        owner_unit_instance: instance_id,
                        base_uuid: *equipment_uuid,
                    },
                );
            }
        }

        self.scenario_runtime
            .spawned_groups
            .insert(group_id.clone());
        Ok(spawned_unit_ids)
    }

    pub(super) fn build_runtime_field(&mut self) -> Result<(), GameError> {
        for unit_id in self.units.keys().copied().collect::<Vec<_>>() {
            if self.battlefield.position_of(unit_id).is_none() {
                return Err(GameError::UnitNotFound);
            }
        }
        Ok(())
    }

    fn tactical_group_for_spawn(
        &self,
        spawn_group_id: &crate::game::battle::scenario::ScenarioGroupId,
        unit_ref: &crate::game::battle::scenario::ScenarioUnitRef,
        side: crate::game::enums::Side,
    ) -> Option<TacticalGroupPlanId> {
        self.scenario
            .tactical_plan
            .group_plans
            .iter()
            .find(|plan| {
                if plan.side != side {
                    return false;
                }
                match &plan.members {
                    TacticalGroupMembers::SpawnGroup(group_id) => group_id == spawn_group_id,
                    TacticalGroupMembers::ExplicitUnits(units) => {
                        units.iter().any(|id| id == unit_ref)
                    }
                    TacticalGroupMembers::SideAll(_) => false,
                }
            })
            .or_else(|| {
                self.scenario.tactical_plan.group_plans.iter().find(|plan| {
                    plan.side == side
                        && matches!(
                            &plan.members,
                            TacticalGroupMembers::SideAll(plan_side) if *plan_side == side
                        )
                })
            })
            .map(|plan| plan.id.clone())
    }
}
