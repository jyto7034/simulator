use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    battle::{
        enums::BattleEvent,
        event_log::{BattleLogEvent, HpChangeReason},
        ids::UnitInstanceId,
        scenario::{
            BattleScenario, FieldWorkCellDef, FieldWorkCellId, FieldWorkCompletionEffect,
            FieldWorkObjectDef, FieldWorkObjectId,
        },
        types::{BattleUnitRole, EngagementStance, TacticalTileKind, UnitExclusiveActivity},
    },
    behavior::GameError,
    enums::Side,
    resources::Position,
};

use super::BattleCore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldWorkObjectStatus {
    Active,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldWorkCellSnapshot {
    pub id: FieldWorkCellId,
    pub position: Position,
    pub tile_kind: TacticalTileKind,
    pub worker_unit_id: Option<UnitInstanceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldWorkObjectSnapshot {
    pub id: FieldWorkObjectId,
    pub status: FieldWorkObjectStatus,
    pub accumulated_work_ms: u64,
    pub required_work_ms: u64,
    pub work_cells: Vec<FieldWorkCellSnapshot>,
}

#[derive(Debug, Clone)]
pub(super) struct FieldWorkObjectRuntime {
    definition: FieldWorkObjectDef,
    status: FieldWorkObjectStatus,
    accumulated_work_ms: u64,
    last_progress_time_ms: u64,
    contributing_worker_count: u64,
    schedule_generation: u64,
    assignments: BTreeMap<FieldWorkCellId, Option<UnitInstanceId>>,
}

crate::authoring::clone_shape::struct_shape!(FieldWorkObjectRuntime {
    definition,
    status,
    accumulated_work_ms,
    last_progress_time_ms,
    contributing_worker_count,
    schedule_generation,
    assignments,
});
crate::authoring::clone_shape::copy_shape!(FieldWorkObjectStatus);

pub(super) fn runtime_from_scenario(
    scenario: &BattleScenario,
) -> BTreeMap<FieldWorkObjectId, FieldWorkObjectRuntime> {
    scenario
        .tactical_plan
        .field_work_objects
        .iter()
        .cloned()
        .map(|definition| {
            let assignments = definition
                .work_cells
                .iter()
                .map(|cell| (cell.id.clone(), None))
                .collect();
            (
                definition.id.clone(),
                FieldWorkObjectRuntime {
                    definition,
                    status: FieldWorkObjectStatus::Active,
                    accumulated_work_ms: 0,
                    last_progress_time_ms: 0,
                    contributing_worker_count: 0,
                    schedule_generation: 0,
                    assignments,
                },
            )
        })
        .collect()
}

impl BattleCore {
    pub fn field_work_object_snapshots(&self) -> Vec<FieldWorkObjectSnapshot> {
        self.field_work_objects
            .values()
            .map(|runtime| FieldWorkObjectSnapshot {
                id: runtime.definition.id.clone(),
                status: runtime.status,
                accumulated_work_ms: runtime.accumulated_work_ms,
                required_work_ms: runtime.definition.required_work_ms,
                work_cells: sorted_cells(&runtime.definition)
                    .into_iter()
                    .map(|cell| FieldWorkCellSnapshot {
                        id: cell.id.clone(),
                        position: cell.position,
                        tile_kind: cell.tile_kind,
                        worker_unit_id: runtime.assignments.get(&cell.id).copied().flatten(),
                    })
                    .collect(),
            })
            .collect()
    }

    pub fn unit_is_field_working(&self, unit_instance_id: UnitInstanceId) -> bool {
        self.tactical_states
            .get(&unit_instance_id)
            .is_some_and(|state| state.activity == UnitExclusiveActivity::FieldWorking)
    }

    pub(super) fn validate_field_work_cell_destination(
        &self,
        position: Position,
        tile_kind: TacticalTileKind,
    ) -> Result<(), GameError> {
        if self
            .active_field_work_cell_at(position)
            .is_some_and(|cell| cell.tile_kind != tile_kind)
        {
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }

    pub(crate) fn destination_starts_field_work(&self, position: Position) -> bool {
        self.active_field_work_cell_at(position).is_some()
    }

    fn active_field_work_cell_at(&self, position: Position) -> Option<&FieldWorkCellDef> {
        self.field_work_objects
            .values()
            .filter(|runtime| runtime.status == FieldWorkObjectStatus::Active)
            .flat_map(|runtime| runtime.definition.work_cells.iter())
            .find(|cell| cell.position == position)
    }

    pub(super) fn settle_field_work_progress(&mut self, time_ms: u64) {
        for runtime in self.field_work_objects.values_mut() {
            if runtime.status != FieldWorkObjectStatus::Active {
                runtime.last_progress_time_ms = time_ms.max(runtime.last_progress_time_ms);
                continue;
            }
            let settled_at = time_ms.max(runtime.last_progress_time_ms);
            let elapsed_ms = settled_at.saturating_sub(runtime.last_progress_time_ms);
            let contributed = elapsed_ms.saturating_mul(runtime.contributing_worker_count);
            runtime.accumulated_work_ms = runtime
                .definition
                .required_work_ms
                .min(runtime.accumulated_work_ms.saturating_add(contributed));
            runtime.last_progress_time_ms = settled_at;
        }
    }

    pub(super) fn reconcile_field_work(&mut self, time_ms: u64) {
        self.reconcile_field_work_with_deferred_tactical_effects(time_ms, &[]);
    }

    pub(super) fn reconcile_field_work_with_deferred_tactical_effects(
        &mut self,
        time_ms: u64,
        deferred_tactical_effect_units: &[UnitInstanceId],
    ) {
        self.settle_field_work_progress(time_ms);

        let previous_workers = self.current_field_workers();
        let mut assignment_changes = Vec::new();
        let object_ids = self.field_work_objects.keys().cloned().collect::<Vec<_>>();

        for object_id in object_ids {
            let Some(runtime) = self.field_work_objects.get(&object_id) else {
                continue;
            };
            let desired = if runtime.status == FieldWorkObjectStatus::Completed {
                BTreeMap::new()
            } else {
                sorted_cells(&runtime.definition)
                    .into_iter()
                    .map(|cell| {
                        let worker = self.eligible_worker_at(cell.position, cell.tile_kind);
                        (cell.id.clone(), worker)
                    })
                    .collect::<BTreeMap<_, _>>()
            };

            let cell_ids = runtime.assignments.keys().cloned().collect::<Vec<_>>();
            for cell_id in cell_ids {
                let before = runtime.assignments.get(&cell_id).copied().flatten();
                let after = desired.get(&cell_id).copied().flatten();
                if before != after {
                    assignment_changes.push((object_id.clone(), cell_id, before, after));
                }
            }
        }

        let changed_objects = assignment_changes
            .iter()
            .map(|(object_id, _, _, _)| object_id.clone())
            .collect::<BTreeSet<_>>();
        for (object_id, cell_id, before, after) in &assignment_changes {
            if let Some(runtime) = self.field_work_objects.get_mut(object_id) {
                runtime.assignments.insert(cell_id.clone(), *after);
            }
            if let Some(worker_unit_id) = before {
                self.record_event_log(
                    time_ms,
                    BattleLogEvent::FieldWorkAssignmentStopped {
                        object_id: object_id.0.clone(),
                        work_cell_id: cell_id.0.clone(),
                        worker_unit_instance_id: *worker_unit_id,
                    },
                );
            }
            if let Some(worker_unit_id) = after {
                self.record_event_log(
                    time_ms,
                    BattleLogEvent::FieldWorkAssignmentStarted {
                        object_id: object_id.0.clone(),
                        work_cell_id: cell_id.0.clone(),
                        worker_unit_instance_id: *worker_unit_id,
                    },
                );
            }
        }

        let next_workers = self.current_field_workers();
        let exiting = previous_workers
            .difference(&next_workers)
            .copied()
            .collect::<Vec<_>>();
        let entering = next_workers
            .difference(&previous_workers)
            .copied()
            .collect::<Vec<_>>();
        if !entering.is_empty() {
            self.release_tactical_engagements(&entering);
        }
        for unit_id in exiting {
            self.transition_field_work_activity(
                unit_id,
                false,
                time_ms,
                !deferred_tactical_effect_units.contains(&unit_id),
            );
        }
        for unit_id in entering {
            self.transition_field_work_activity(
                unit_id,
                true,
                time_ms,
                !deferred_tactical_effect_units.contains(&unit_id),
            );
        }

        let contributor_counts = self
            .field_work_objects
            .iter()
            .map(|(object_id, runtime)| {
                let count = runtime
                    .assignments
                    .values()
                    .filter_map(|worker| *worker)
                    .filter(|worker| self.worker_contributes(*worker, time_ms))
                    .count() as u64;
                (object_id.clone(), count)
            })
            .collect::<Vec<_>>();

        let mut completion_schedules = Vec::new();
        for (object_id, count) in contributor_counts {
            let Some(runtime) = self.field_work_objects.get_mut(&object_id) else {
                continue;
            };
            if changed_objects.contains(&object_id) || runtime.contributing_worker_count != count {
                runtime.schedule_generation = runtime.schedule_generation.wrapping_add(1);
            }
            runtime.contributing_worker_count = count;
            if runtime.status != FieldWorkObjectStatus::Active || count == 0 {
                continue;
            }
            let remaining = runtime
                .definition
                .required_work_ms
                .saturating_sub(runtime.accumulated_work_ms);
            let delay_ms = remaining.saturating_add(count - 1) / count;
            completion_schedules.push((
                time_ms.saturating_add(delay_ms),
                object_id,
                runtime.schedule_generation,
            ));
        }
        for (completion_time_ms, object_id, generation) in completion_schedules {
            if self.event_queue.iter().any(|event| {
                matches!(
                    event.as_ref(),
                    BattleEvent::FieldWorkCompletion {
                        object_id: queued_object_id,
                        generation: queued_generation,
                        ..
                    } if queued_object_id == &object_id && *queued_generation == generation
                )
            }) {
                continue;
            }
            self.event_queue.push(BattleEvent::FieldWorkCompletion {
                time_ms: completion_time_ms,
                object_id,
                generation,
            });
        }
        self.refresh_block_state();
    }

    fn eligible_worker_at(
        &self,
        position: Position,
        tile_kind: TacticalTileKind,
    ) -> Option<UnitInstanceId> {
        let mut workers = self
            .units
            .values()
            .filter(|unit| {
                !self.channel_active(unit.instance_id)
                    && unit.owner == Side::Player
                    && unit.role == BattleUnitRole::Combatant
                    && unit.is_active()
                    && unit.body.projected_tile() == position
                    && self
                        .tactical_states
                        .get(&unit.instance_id)
                        .is_some_and(|state| state.tile_kind == tile_kind)
            })
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        workers.sort();
        workers.first().copied()
    }

    fn current_field_workers(&self) -> BTreeSet<UnitInstanceId> {
        self.field_work_objects
            .values()
            .flat_map(|runtime| runtime.assignments.values().filter_map(|worker| *worker))
            .collect()
    }

    fn worker_contributes(&self, worker: UnitInstanceId, time_ms: u64) -> bool {
        self.units.get(&worker).is_some_and(|unit| unit.is_active())
            && !self
                .unit_control_state(worker, time_ms)
                .is_some_and(|control| control.hard_cc_active())
    }

    fn transition_field_work_activity(
        &mut self,
        unit_id: UnitInstanceId,
        entering: bool,
        time_ms: u64,
        reconcile_tactical_effects: bool,
    ) {
        let Some(state) = self.tactical_states.get(&unit_id) else {
            return;
        };
        let expected = if entering {
            UnitExclusiveActivity::Ready
        } else {
            UnitExclusiveActivity::FieldWorking
        };
        if state.activity != expected {
            return;
        }
        let before_effects = self
            .derived_tactical_effects(unit_id)
            .expect("field-work activity transition requires tactical state");
        {
            let Some(state) = self.tactical_states.get_mut(&unit_id) else {
                return;
            };
            state.activity = if entering {
                UnitExclusiveActivity::FieldWorking
            } else {
                UnitExclusiveActivity::Ready
            };
            state.stance = if entering {
                EngagementStance::Defense
            } else {
                EngagementStance::Attack
            };
            state.forced_defense_until_ms = None;
            state.forced_defense_epoch = state.forced_defense_epoch.wrapping_add(1);
        }
        if reconcile_tactical_effects {
            self.reconcile_tactical_effects(unit_id, before_effects, time_ms);
        }
    }

    pub(super) fn handle_field_work_completion(
        &mut self,
        time_ms: u64,
        object_id: FieldWorkObjectId,
        generation: u64,
    ) {
        let Some(runtime) = self.field_work_objects.get(&object_id) else {
            return;
        };
        if runtime.status != FieldWorkObjectStatus::Active
            || runtime.schedule_generation != generation
            || runtime.accumulated_work_ms < runtime.definition.required_work_ms
        {
            return;
        }
        let effect = runtime.definition.completion_effect.clone();
        if let Some(runtime) = self.field_work_objects.get_mut(&object_id) {
            runtime.status = FieldWorkObjectStatus::Completed;
            runtime.accumulated_work_ms = runtime.definition.required_work_ms;
            runtime.schedule_generation = runtime.schedule_generation.wrapping_add(1);
            runtime.contributing_worker_count = 0;
        }

        let (target_unit_id, requested_flat_hp, actual_repaired_hp) = match effect {
            FieldWorkCompletionEffect::RepairProtectedUnit { unit_ref, flat_hp } => {
                let target = self
                    .scenario_runtime
                    .spawned
                    .unit_refs
                    .get(&unit_ref)
                    .copied();
                let before = target
                    .and_then(|unit_id| self.units.get(&unit_id))
                    .map(|unit| unit.stats.current_health)
                    .unwrap_or(0);
                if let Some(target_unit_id) = target {
                    self.apply_hp_delta_and_record(
                        None,
                        target_unit_id,
                        i32::try_from(flat_hp).unwrap_or(i32::MAX),
                        time_ms,
                        HpChangeReason::FieldWork,
                    );
                }
                let after = target
                    .and_then(|unit_id| self.units.get(&unit_id))
                    .map(|unit| unit.stats.current_health)
                    .unwrap_or(before);
                (target, flat_hp, after.saturating_sub(before))
            }
        };

        self.record_event_log(
            time_ms,
            BattleLogEvent::FieldWorkCompleted {
                object_id: object_id.0,
                repaired_unit_instance_id: target_unit_id,
                requested_flat_hp,
                actual_repaired_hp,
            },
        );
        self.reconcile_field_work(time_ms);
    }
}

fn sorted_cells(definition: &FieldWorkObjectDef) -> Vec<&FieldWorkCellDef> {
    let mut cells = definition.work_cells.iter().collect::<Vec<_>>();
    cells.sort_by(|left, right| left.id.cmp(&right.id));
    cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        battle::{
            buffs::BuffId,
            core::sim::{BattleLiveCommand, BattleLiveCommandOutcome},
            event_log::BattleEventCause,
            scenario::{
                BattleObjective, FieldWorkPresentationKind, ScenarioAction, ScenarioEvent,
                ScenarioEventId, ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger,
                ScenarioUnitRef, ScenarioUnitSpawn, TacticalPlan,
            },
            tile_range::FacingDirection,
            types::{
                BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass, DeploymentAffinity,
                UnitCombatProfile,
            },
        },
        data::{
            equipment_data::{GroundDefenseFocusProfile, PlatformRestProfile, WeaponCombatProfile},
            GameDataBuilder,
        },
        enums::{Side, Tier},
    };
    use uuid::Uuid;

    fn worker_draft(owned_uuid: u128) -> BattleUnitDraft {
        let mut weapon_profile = WeaponCombatProfile::default();
        weapon_profile.deployment_affinity = DeploymentAffinity::GroundOnly;
        weapon_profile.block_capacity = 2;
        weapon_profile.ground_defense_focus = Some(GroundDefenseFocusProfile {
            block_capacity_bonus: 3,
            incoming_damage_reduction_percent: 10,
        });
        weapon_profile.platform_rest = None;

        let mut profile = UnitCombatProfile::employee_default();
        profile.weapon_profile = Some(weapon_profile.clone());
        profile.block_capacity = weapon_profile.block_capacity;
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(owned_uuid),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(owned_uuid + 1),
                profile,
            },
            threat_class: BattleUnitThreatClass::Normal,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: Default::default(),
            equipment_layout: None,
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        }
    }

    fn protected_draft() -> BattleUnitDraft {
        let mut profile = UnitCombatProfile::employee_default();
        profile.stats.max_health = 200;
        profile.stats.current_health = 200;
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(0xF001),
            source: BattleUnitSource::DefenseObject {
                base_uuid: Uuid::from_u128(0xF002),
                profile,
            },
            threat_class: BattleUnitThreatClass::Normal,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: Default::default(),
            equipment_layout: None,
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        }
    }

    fn platform_worker_draft(owned_uuid: u128) -> BattleUnitDraft {
        let weapon_profile = WeaponCombatProfile {
            deployment_affinity: DeploymentAffinity::PlatformOnly,
            block_capacity: 0,
            ground_defense_focus: None,
            platform_rest: Some(PlatformRestProfile {
                hp_recovery_per_tick: 3,
            }),
            ..WeaponCombatProfile::default()
        };
        let mut profile = UnitCombatProfile::employee_default();
        profile.weapon_profile = Some(weapon_profile);
        profile.block_capacity = 0;
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(owned_uuid),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(owned_uuid + 1),
                profile,
            },
            threat_class: BattleUnitThreatClass::Normal,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: Default::default(),
            equipment_layout: None,
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        }
    }

    fn any_worker_draft(owned_uuid: u128) -> BattleUnitDraft {
        let weapon_profile = WeaponCombatProfile {
            deployment_affinity: DeploymentAffinity::Any,
            block_capacity: 2,
            ground_defense_focus: Some(GroundDefenseFocusProfile {
                block_capacity_bonus: 3,
                incoming_damage_reduction_percent: 10,
            }),
            platform_rest: Some(PlatformRestProfile {
                hp_recovery_per_tick: 3,
            }),
            ..WeaponCombatProfile::default()
        };
        let mut profile = UnitCombatProfile::employee_default();
        profile.weapon_profile = Some(weapon_profile.clone());
        profile.block_capacity = weapon_profile.block_capacity;
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(owned_uuid),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(owned_uuid + 1),
                profile,
            },
            threat_class: BattleUnitThreatClass::Normal,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: Default::default(),
            equipment_layout: None,
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        }
    }

    fn field_work_core(
        work_cells: Vec<FieldWorkCellDef>,
        required_work_ms: u64,
    ) -> (BattleCore, super::super::sim::BattleExecutionState) {
        field_work_core_with_data(
            work_cells,
            required_work_ms,
            GameDataBuilder::live_defaults().build_arc(),
        )
    }

    fn field_work_core_with_data(
        work_cells: Vec<FieldWorkCellDef>,
        required_work_ms: u64,
        game_data: std::sync::Arc<crate::game::data::GameDataBase>,
    ) -> (BattleCore, super::super::sim::BattleExecutionState) {
        let protected_ref = ScenarioUnitRef::new("protected");
        let protected_group_id = ScenarioGroupId::new("protected_group");
        let mut scenario = BattleScenario::empty((6, 6));
        scenario.groups.push(ScenarioSpawnGroup {
            id: protected_group_id.clone(),
            side: Side::Player,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![ScenarioUnitSpawn {
                unit_ref: protected_ref.clone(),
                side: Side::Player,
                draft: protected_draft(),
                position: Position::new(5, 5),
                instance_salt: 0,
            }],
        });
        scenario.events.push(ScenarioEvent {
            id: ScenarioEventId::new("spawn_protected"),
            trigger: ScenarioTrigger::AtBattleStart,
            action: ScenarioAction::SpawnGroup {
                group_id: protected_group_id,
            },
        });
        scenario.terminal_outcomes =
            BattleScenario::fixture_protected_outcomes(protected_ref.clone(), Some(20_000));
        scenario.tactical_plan = TacticalPlan {
            objective: BattleObjective::ProtectUnit {
                unit_ref: protected_ref.clone(),
            },
            field_work_objects: vec![FieldWorkObjectDef {
                id: FieldWorkObjectId::new("repair"),
                presentation_kind: FieldWorkPresentationKind::EmergencyRepairTerminal,
                position: Position::new(1, 1),
                work_cells,
                required_work_ms,
                completion_effect: FieldWorkCompletionEffect::RepairProtectedUnit {
                    unit_ref: protected_ref,
                    flat_hp: 100,
                },
            }],
            ..TacticalPlan::default()
        };

        let mut core = BattleCore::new_from_scenario(scenario, game_data, 0xF100);
        let mut execution = core.start_battle_execution().unwrap();
        assert!(matches!(
            core.step_battle_execution_by(&mut execution, 0).unwrap(),
            super::super::sim::BattleStepOutcome::Running
        ));
        (core, execution)
    }

    fn deploy_worker(
        core: &mut BattleCore,
        owned_uuid: u128,
        salt: u32,
        position: Position,
        time_ms: u64,
    ) -> UnitInstanceId {
        let outcome = core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft: worker_draft(owned_uuid),
                position,
                tile_kind: TacticalTileKind::Ground,
                facing: FacingDirection::Right,
                instance_salt: salt,
                time_ms,
                current_hp_policy: None,
            })
            .unwrap();
        let BattleLiveCommandOutcome::UnitDeployed { unit_id } = outcome else {
            panic!("expected deployed worker");
        };
        unit_id
    }

    #[test]
    fn u05_actual_work_relocation_withdrawal_and_redeploy_share_global_employee_pulses() {
        let data = crate::game::data::ValidatedGameData::load_live_embedded()
            .unwrap()
            .into_fixture_database_arc();
        let (mut core, mut execution) = field_work_core_with_data(
            vec![FieldWorkCellDef {
                id: FieldWorkCellId::new("u05_work"),
                position: Position::new(1, 1),
                tile_kind: TacticalTileKind::Ground,
            }],
            10_000,
            data,
        );
        let mut draft = worker_draft(0x505);
        let BattleUnitSource::TestFixture { profile, .. } = draft.source else {
            unreachable!()
        };
        draft.source = BattleUnitSource::Employee(profile);
        let weapon = core
            .game_data
            .equipment_data
            .items
            .iter()
            .find(|item| {
                item.weapon_profile
                    .as_ref()
                    .is_some_and(|profile| profile.deployment_affinity.allows_ground())
            })
            .unwrap();
        draft
            .equipped_items
            .push(crate::game::battle::types::BattleEquipmentBinding {
                base_uuid: weapon.uuid,
                persistent_copy_uuid: None,
                slot: None,
            });
        let deploy = |core: &mut BattleCore, salt, now| {
            let BattleLiveCommandOutcome::UnitDeployed { unit_id } = core
                .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                    draft: draft.clone(),
                    position: Position::new(1, 1),
                    tile_kind: TacticalTileKind::Ground,
                    facing: FacingDirection::Right,
                    instance_salt: salt,
                    time_ms: now,
                    current_hp_policy: None,
                })
                .unwrap()
            else {
                unreachable!()
            };
            unit_id
        };
        let actor = deploy(&mut core, 1, 0);
        assert_eq!(
            core.unit_tactical_snapshot(actor, 0).unwrap().activity,
            UnitExclusiveActivity::FieldWorking
        );
        assert_eq!(core.units[&actor].resonance_current, 20);
        core.step_battle_execution_until(&mut execution, 1000)
            .unwrap();
        assert_eq!(core.units[&actor].resonance_current, 21);
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 1000,
                caster_instance_id: actor,
                target_instance_id: actor,
                buff_id: crate::game::battle::buffs::BuffId::from_name("silence"),
                duration_ms: 1500,
                cause: Default::default(),
            },
            1000,
        )
        .unwrap();
        core.step_battle_execution_until(&mut execution, 2000)
            .unwrap();
        assert_eq!(core.units[&actor].resonance_current, 22);
        core.apply_live_command(BattleLiveCommand::TacticalRelocateUnit {
            unit_id: actor,
            destination: Position::new(2, 1),
            tile_kind: TacticalTileKind::Ground,
            facing: FacingDirection::Right,
            time_ms: 2000,
        })
        .unwrap();
        assert_eq!(core.units[&actor].resonance_current, 22);
        assert_eq!(
            core.unit_tactical_snapshot(actor, 2000).unwrap().activity,
            UnitExclusiveActivity::Ready
        );
        core.step_battle_execution_until(&mut execution, 3000)
            .unwrap();
        assert_eq!(core.units[&actor].resonance_current, 23);
        assert!(matches!(
            core.set_engagement_stance(actor, EngagementStance::Attack, 3000),
            Err(GameError::ActionUnavailable {
                reason: crate::game::behavior::ActionDisabledReasonCode::ForcedDefense,
                ..
            })
        ));
        core.step_battle_execution_until(&mut execution, 3500)
            .unwrap();
        assert_eq!(
            core.unit_tactical_snapshot(actor, 3500).unwrap().stance,
            EngagementStance::Attack
        );
        core.step_battle_execution_until(&mut execution, 4000)
            .unwrap();
        assert_eq!(core.units[&actor].resonance_current, 24);
        core.withdraw_unit(actor, 4000).unwrap();
        let replacement = deploy(&mut core, 2, 4100);
        assert_ne!(actor, replacement);
        assert_eq!(core.units[&replacement].resonance_current, 20);
        core.step_battle_execution_until(&mut execution, 5000)
            .unwrap();
        assert_eq!(core.units[&replacement].resonance_current, 21);
        assert_eq!(core.units[&actor].resonance_current, 24);
    }

    fn deploy_any_worker(
        core: &mut BattleCore,
        owned_uuid: u128,
        salt: u32,
        position: Position,
        tile_kind: TacticalTileKind,
        time_ms: u64,
    ) -> UnitInstanceId {
        let outcome = core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft: any_worker_draft(owned_uuid),
                position,
                tile_kind,
                facing: FacingDirection::Right,
                instance_salt: salt,
                time_ms,
                current_hp_policy: None,
            })
            .unwrap();
        let BattleLiveCommandOutcome::UnitDeployed { unit_id } = outcome else {
            panic!("expected deployed Any worker");
        };
        unit_id
    }

    #[test]
    fn field_work_preserves_progress_suspends_for_hard_cc_and_completes_once() {
        let (mut core, mut execution) = field_work_core(
            vec![FieldWorkCellDef {
                id: FieldWorkCellId::new("cell"),
                position: Position::new(1, 1),
                tile_kind: TacticalTileKind::Ground,
            }],
            1_000,
        );
        let protected_id = core.protected_unit_id().unwrap();
        core.apply_hp_delta_and_record(None, protected_id, -100, 0, HpChangeReason::Command);

        let first_worker = deploy_worker(&mut core, 0xF201, 1, Position::new(1, 1), 0);
        let initial = core.unit_tactical_snapshot(first_worker, 0).unwrap();
        assert_eq!(initial.activity, UnitExclusiveActivity::FieldWorking);
        assert_eq!(initial.stance, EngagementStance::Defense);
        assert_eq!(initial.tile_effect, super::super::TacticalTileEffect::None);
        assert_eq!(initial.effective_block_capacity, 0);
        assert_eq!(
            initial.attack_disabled_reason,
            Some(crate::game::behavior::ActionDisabledReasonCode::FieldWorking)
        );
        assert!(core
            .tactical_relocation_disabled_reason(first_worker, 0)
            .is_none());

        {
            let overlapping_enemy = core.units.get_mut(&protected_id).unwrap();
            overlapping_enemy.owner = Side::Opponent;
            overlapping_enemy.set_world_position(
                crate::game::battle::core::movement::types::WorldVec2::from_tile_center(
                    Position::new(1, 1),
                ),
            );
        }
        core.reconcile_field_work(0);
        assert_eq!(
            core.field_work_object_snapshots()[0].work_cells[0].worker_unit_id,
            Some(first_worker),
            "enemy overlap must not displace a field worker"
        );

        core.step_battle_execution_until(&mut execution, 200)
            .unwrap();
        core.withdraw_unit(first_worker, 200).unwrap();
        assert_eq!(
            core.field_work_object_snapshots()[0].accumulated_work_ms,
            200
        );

        core.step_battle_execution_until(&mut execution, 500)
            .unwrap();
        assert_eq!(
            core.field_work_object_snapshots()[0].accumulated_work_ms,
            200,
            "worker-zero interval must preserve without catch-up"
        );

        let second_worker = deploy_worker(&mut core, 0xF201, 2, Position::new(1, 1), 500);
        assert_ne!(
            second_worker, first_worker,
            "redeploying the same employee uses a new runtime instance"
        );
        core.step_battle_execution_until(&mut execution, 600)
            .unwrap();
        core.process_event_for_test(
            BattleEvent::ApplyBuff {
                time_ms: 600,
                caster_instance_id: protected_id,
                target_instance_id: second_worker,
                buff_id: BuffId::from_name("stun"),
                duration_ms: 200,
                cause: BattleEventCause::default(),
            },
            600,
        )
        .unwrap();
        core.step_battle_execution_until(&mut execution, 700)
            .unwrap();
        let hp_before_damage = core.units[&second_worker].stats.current_health;
        core.apply_hp_delta_and_record(
            Some(protected_id),
            second_worker,
            -10,
            700,
            HpChangeReason::Command,
        );
        assert_eq!(
            core.units[&second_worker].stats.current_health,
            hp_before_damage - 10,
            "worker must remain damageable"
        );
        assert!(core.unit_is_field_working(second_worker));

        core.step_battle_execution_until(&mut execution, 800)
            .unwrap();
        assert_eq!(
            core.field_work_object_snapshots()[0].accumulated_work_ms,
            300,
            "Hard CC interval must not contribute or catch up"
        );

        core.step_battle_execution_until(&mut execution, 1_500)
            .unwrap();
        let object = &core.field_work_object_snapshots()[0];
        assert_eq!(object.status, FieldWorkObjectStatus::Completed);
        assert_eq!(object.accumulated_work_ms, 1_000);
        assert_eq!(object.work_cells[0].worker_unit_id, None);
        assert_eq!(core.units[&protected_id].stats.current_health, 200);

        let completed_worker = core.unit_tactical_snapshot(second_worker, 1_500).unwrap();
        assert_eq!(completed_worker.activity, UnitExclusiveActivity::Ready);
        assert_eq!(completed_worker.stance, EngagementStance::Attack);
        assert_eq!(
            completed_worker.tile_effect,
            super::super::TacticalTileEffect::None
        );
        assert_eq!(completed_worker.effective_block_capacity, 2);

        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(
                    entry.event,
                    BattleLogEvent::FieldWorkAssignmentStarted { .. }
                ))
                .count(),
            2
        );
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(
                    entry.event,
                    BattleLogEvent::FieldWorkAssignmentStopped { .. }
                ))
                .count(),
            2
        );
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(entry.event, BattleLogEvent::FieldWorkCompleted { .. }))
                .count(),
            1
        );

        core.step_battle_execution_until(&mut execution, 2_000)
            .unwrap();
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(entry.event, BattleLogEvent::FieldWorkCompleted { .. }))
                .count(),
            1,
            "stale scheduled completions must no-op"
        );
    }

    #[test]
    fn multiple_workers_sum_and_full_hp_completion_is_consumed_once() {
        let (mut core, mut execution) = field_work_core(
            vec![
                FieldWorkCellDef {
                    id: FieldWorkCellId::new("cell_a"),
                    position: Position::new(1, 1),
                    tile_kind: TacticalTileKind::Ground,
                },
                FieldWorkCellDef {
                    id: FieldWorkCellId::new("cell_b"),
                    position: Position::new(2, 1),
                    tile_kind: TacticalTileKind::Ground,
                },
            ],
            1_000,
        );
        let protected_id = core.protected_unit_id().unwrap();
        let first = deploy_worker(&mut core, 0xF401, 3, Position::new(1, 1), 0);
        let second = deploy_worker(&mut core, 0xF501, 4, Position::new(2, 1), 0);
        assert!(core.unit_is_field_working(first));
        assert!(core.unit_is_field_working(second));

        core.step_battle_execution_until(&mut execution, 500)
            .unwrap();
        let object = &core.field_work_object_snapshots()[0];
        assert_eq!(object.status, FieldWorkObjectStatus::Completed);
        assert_eq!(object.accumulated_work_ms, 1_000);
        assert_eq!(core.units[&protected_id].stats.current_health, 200);

        let completion = core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match &entry.event {
                BattleLogEvent::FieldWorkCompleted {
                    requested_flat_hp,
                    actual_repaired_hp,
                    ..
                } => Some((*requested_flat_hp, *actual_repaired_hp)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(completion, vec![(100, 0)]);
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(
                    entry.event,
                    BattleLogEvent::FieldWorkAssignmentStarted { .. }
                ))
                .count(),
            2
        );
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(
                    entry.event,
                    BattleLogEvent::FieldWorkAssignmentStopped { .. }
                ))
                .count(),
            2
        );
    }

    #[test]
    fn platform_worker_suppresses_rest_and_stance_until_completion() {
        let (mut core, mut execution) = field_work_core(
            vec![FieldWorkCellDef {
                id: FieldWorkCellId::new("platform_cell"),
                position: Position::new(1, 1),
                tile_kind: TacticalTileKind::Platform,
            }],
            1_000,
        );
        let outcome = core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft: platform_worker_draft(0xF551),
                position: Position::new(1, 1),
                tile_kind: TacticalTileKind::Platform,
                facing: FacingDirection::Right,
                instance_salt: 5,
                time_ms: 0,
                current_hp_policy: None,
            })
            .unwrap();
        let BattleLiveCommandOutcome::UnitDeployed { unit_id } = outcome else {
            panic!("expected platform worker deployment");
        };
        let working = core.unit_tactical_snapshot(unit_id, 0).unwrap();
        assert_eq!(working.activity, UnitExclusiveActivity::FieldWorking);
        assert_eq!(working.stance, EngagementStance::Defense);
        assert_eq!(working.tile_effect, super::super::TacticalTileEffect::None);
        assert_eq!(working.next_rest_tick_ms, None);
        assert_eq!(
            core.stance_change_disabled_reason(unit_id, EngagementStance::Attack, 2_000),
            Some(crate::game::behavior::ActionDisabledReasonCode::FieldWorking)
        );

        core.step_battle_execution_until(&mut execution, 1_000)
            .unwrap();
        let completed = core.unit_tactical_snapshot(unit_id, 1_000).unwrap();
        assert_eq!(completed.activity, UnitExclusiveActivity::Ready);
        assert_eq!(completed.stance, EngagementStance::Attack);
        assert_eq!(
            completed.tile_effect,
            super::super::TacticalTileEffect::None
        );
        assert_eq!(completed.next_rest_tick_ms, None);
    }

    #[test]
    fn swap_and_relocation_reconcile_workers_from_final_occupancy() {
        let (mut core, _execution) = field_work_core(
            vec![
                FieldWorkCellDef {
                    id: FieldWorkCellId::new("cell_a"),
                    position: Position::new(1, 1),
                    tile_kind: TacticalTileKind::Ground,
                },
                FieldWorkCellDef {
                    id: FieldWorkCellId::new("cell_b"),
                    position: Position::new(2, 1),
                    tile_kind: TacticalTileKind::Ground,
                },
            ],
            100_000,
        );
        let first = deploy_worker(&mut core, 0xF601, 5, Position::new(1, 1), 0);
        let second = deploy_worker(&mut core, 0xF701, 6, Position::new(3, 1), 0);
        core.set_engagement_stance(second, EngagementStance::Defense, 1_500)
            .unwrap();

        core.tactically_swap_units(
            first,
            second,
            TacticalTileKind::Ground,
            TacticalTileKind::Ground,
            FacingDirection::Left,
            FacingDirection::Right,
            1_500,
        )
        .unwrap();
        assert!(!core.unit_is_field_working(first));
        assert!(core.unit_is_field_working(second));
        let after_swap = core.field_work_object_snapshots().remove(0);
        assert_eq!(after_swap.accumulated_work_ms, 1_500);
        assert_eq!(after_swap.work_cells[0].worker_unit_id, Some(second));
        assert_eq!(after_swap.work_cells[1].worker_unit_id, None);

        core.tactically_relocate_unit(
            second,
            Position::new(2, 1),
            TacticalTileKind::Ground,
            FacingDirection::Right,
            11_500,
        )
        .unwrap();
        let after_handoff = core.field_work_object_snapshots().remove(0);
        assert_eq!(after_handoff.work_cells[0].worker_unit_id, None);
        assert_eq!(after_handoff.work_cells[1].worker_unit_id, Some(second));
        assert!(core.unit_is_field_working(second));

        core.tactically_relocate_unit(
            second,
            Position::new(4, 1),
            TacticalTileKind::Ground,
            FacingDirection::Right,
            21_500,
        )
        .unwrap();
        let after_exit = core.field_work_object_snapshots().remove(0);
        assert_eq!(after_exit.accumulated_work_ms, 21_500);
        assert!(after_exit
            .work_cells
            .iter()
            .all(|cell| cell.worker_unit_id.is_none()));
        let second_state = core.unit_tactical_snapshot(second, 21_500).unwrap();
        assert_eq!(second_state.activity, UnitExclusiveActivity::Ready);
        assert_eq!(
            second_state.tile_effect,
            super::super::TacticalTileEffect::DefenseFocus
        );
    }

    #[test]
    fn any_worker_cross_kind_relocation_preserves_working_final_effects() {
        let (mut core, _execution) = field_work_core(
            vec![
                FieldWorkCellDef {
                    id: FieldWorkCellId::new("ground_cell"),
                    position: Position::new(1, 1),
                    tile_kind: TacticalTileKind::Ground,
                },
                FieldWorkCellDef {
                    id: FieldWorkCellId::new("platform_cell"),
                    position: Position::new(2, 1),
                    tile_kind: TacticalTileKind::Platform,
                },
            ],
            100_000,
        );
        let worker = deploy_any_worker(
            &mut core,
            0xF801,
            7,
            Position::new(1, 1),
            TacticalTileKind::Ground,
            0,
        );
        core.units
            .get_mut(&worker)
            .unwrap()
            .incoming_damage_modifiers
            .damage_reduction_percent = 7;
        core.units
            .get_mut(&worker)
            .unwrap()
            .incoming_damage_modifiers
            .physical_damage_reduction_percent = 23;
        let rest_epoch_before = core.tactical_states[&worker].rest_epoch;

        core.tactically_relocate_unit(
            worker,
            Position::new(2, 1),
            TacticalTileKind::Platform,
            FacingDirection::Left,
            1_500,
        )
        .unwrap();

        let tactical = core.unit_tactical_snapshot(worker, 1_500).unwrap();
        assert_eq!(tactical.activity, UnitExclusiveActivity::FieldWorking);
        assert_eq!(tactical.stance, EngagementStance::Defense);
        assert_eq!(tactical.tile_kind, TacticalTileKind::Platform);
        assert_eq!(tactical.tile_effect, super::super::TacticalTileEffect::None);
        assert_eq!(tactical.effective_block_capacity, 0);
        assert_eq!(tactical.defense_focus_damage_reduction_percent, 0);
        assert_eq!(tactical.next_rest_tick_ms, None);
        assert_eq!(core.tactical_states[&worker].rest_epoch, rest_epoch_before);
        assert_eq!(
            core.units[&worker]
                .incoming_damage_modifiers
                .damage_reduction_percent,
            7
        );
        assert_eq!(
            core.units[&worker]
                .incoming_damage_modifiers
                .physical_damage_reduction_percent,
            23
        );
    }

    #[test]
    fn any_worker_cross_kind_relocation_applies_focus_once_after_work_exit() {
        let (mut core, _execution) = field_work_core(
            vec![FieldWorkCellDef {
                id: FieldWorkCellId::new("platform_cell"),
                position: Position::new(1, 1),
                tile_kind: TacticalTileKind::Platform,
            }],
            100_000,
        );
        let worker = deploy_any_worker(
            &mut core,
            0xF811,
            8,
            Position::new(1, 1),
            TacticalTileKind::Platform,
            0,
        );
        core.units
            .get_mut(&worker)
            .unwrap()
            .incoming_damage_modifiers
            .damage_reduction_percent = 7;
        core.units
            .get_mut(&worker)
            .unwrap()
            .incoming_damage_modifiers
            .physical_damage_reduction_percent = 23;

        core.tactically_relocate_unit(
            worker,
            Position::new(3, 1),
            TacticalTileKind::Ground,
            FacingDirection::Right,
            1_500,
        )
        .unwrap();

        let tactical = core.unit_tactical_snapshot(worker, 1_500).unwrap();
        assert_eq!(tactical.activity, UnitExclusiveActivity::Ready);
        assert_eq!(tactical.stance, EngagementStance::Defense);
        assert_eq!(tactical.tile_kind, TacticalTileKind::Ground);
        assert_eq!(
            tactical.tile_effect,
            super::super::TacticalTileEffect::DefenseFocus
        );
        assert_eq!(tactical.effective_block_capacity, 5);
        assert_eq!(tactical.defense_focus_damage_reduction_percent, 10);
        assert_eq!(tactical.next_rest_tick_ms, None);
        assert_eq!(
            core.units[&worker]
                .incoming_damage_modifiers
                .damage_reduction_percent,
            17
        );
        assert_eq!(
            core.units[&worker]
                .incoming_damage_modifiers
                .physical_damage_reduction_percent,
            23
        );
    }

    #[test]
    fn any_workers_cross_kind_swap_preserves_both_working_final_effects() {
        let (mut core, _execution) = field_work_core(
            vec![
                FieldWorkCellDef {
                    id: FieldWorkCellId::new("ground_cell"),
                    position: Position::new(1, 1),
                    tile_kind: TacticalTileKind::Ground,
                },
                FieldWorkCellDef {
                    id: FieldWorkCellId::new("platform_cell"),
                    position: Position::new(2, 1),
                    tile_kind: TacticalTileKind::Platform,
                },
            ],
            100_000,
        );
        let first = deploy_any_worker(
            &mut core,
            0xF821,
            9,
            Position::new(1, 1),
            TacticalTileKind::Ground,
            0,
        );
        let second = deploy_any_worker(
            &mut core,
            0xF831,
            10,
            Position::new(2, 1),
            TacticalTileKind::Platform,
            0,
        );
        core.units
            .get_mut(&first)
            .unwrap()
            .incoming_damage_modifiers
            .damage_reduction_percent = 7;
        core.units
            .get_mut(&second)
            .unwrap()
            .incoming_damage_modifiers
            .damage_reduction_percent = 11;
        core.units
            .get_mut(&first)
            .unwrap()
            .incoming_damage_modifiers
            .physical_damage_reduction_percent = 23;
        core.units
            .get_mut(&second)
            .unwrap()
            .incoming_damage_modifiers
            .magic_damage_reduction_percent = 31;
        let first_epoch = core.tactical_states[&first].rest_epoch;
        let second_epoch = core.tactical_states[&second].rest_epoch;

        core.tactically_swap_units(
            first,
            second,
            TacticalTileKind::Platform,
            TacticalTileKind::Ground,
            FacingDirection::Left,
            FacingDirection::Right,
            1_500,
        )
        .unwrap();

        for (unit_id, tile_kind, general_reduction) in [
            (first, TacticalTileKind::Platform, 7),
            (second, TacticalTileKind::Ground, 11),
        ] {
            let tactical = core.unit_tactical_snapshot(unit_id, 1_500).unwrap();
            assert_eq!(tactical.activity, UnitExclusiveActivity::FieldWorking);
            assert_eq!(tactical.stance, EngagementStance::Defense);
            assert_eq!(tactical.tile_kind, tile_kind);
            assert_eq!(tactical.tile_effect, super::super::TacticalTileEffect::None);
            assert_eq!(tactical.effective_block_capacity, 0);
            assert_eq!(tactical.defense_focus_damage_reduction_percent, 0);
            assert_eq!(tactical.next_rest_tick_ms, None);
            assert_eq!(
                core.units[&unit_id]
                    .incoming_damage_modifiers
                    .damage_reduction_percent,
                general_reduction
            );
        }
        assert_eq!(core.tactical_states[&first].rest_epoch, first_epoch);
        assert_eq!(core.tactical_states[&second].rest_epoch, second_epoch);
        assert_eq!(
            core.units[&first]
                .incoming_damage_modifiers
                .physical_damage_reduction_percent,
            23
        );
        assert_eq!(
            core.units[&second]
                .incoming_damage_modifiers
                .magic_damage_reduction_percent,
            31
        );
    }

    #[test]
    fn cross_kind_relocation_into_work_invalidates_one_stale_rest_epoch() {
        let (mut core, _execution) = field_work_core(
            vec![FieldWorkCellDef {
                id: FieldWorkCellId::new("ground_cell"),
                position: Position::new(1, 1),
                tile_kind: TacticalTileKind::Ground,
            }],
            100_000,
        );
        let worker = deploy_any_worker(
            &mut core,
            0xF841,
            11,
            Position::new(2, 1),
            TacticalTileKind::Platform,
            0,
        );
        core.set_engagement_stance(worker, EngagementStance::Defense, 1_500)
            .unwrap();
        let stale_epoch = core.tactical_states[&worker].rest_epoch;
        let stale_tick = core.tactical_states[&worker]
            .next_rest_tick_ms
            .expect("Platform Rest deadline");
        core.units
            .get_mut(&worker)
            .unwrap()
            .incoming_damage_modifiers
            .damage_reduction_percent = 7;

        core.tactically_relocate_unit(
            worker,
            Position::new(1, 1),
            TacticalTileKind::Ground,
            FacingDirection::Right,
            1_500,
        )
        .unwrap();

        let tactical = core.unit_tactical_snapshot(worker, 1_500).unwrap();
        assert_eq!(tactical.activity, UnitExclusiveActivity::FieldWorking);
        assert_eq!(tactical.tile_effect, super::super::TacticalTileEffect::None);
        assert_eq!(tactical.effective_block_capacity, 0);
        assert_eq!(tactical.next_rest_tick_ms, None);
        assert_eq!(
            core.tactical_states[&worker].rest_epoch,
            stale_epoch.wrapping_add(1)
        );
        assert_eq!(
            core.units[&worker]
                .incoming_damage_modifiers
                .damage_reduction_percent,
            7
        );
        let hp_before = core.units[&worker].stats.current_health;
        let resonance_before = core.units[&worker].resonance_current;

        core.handle_rest_tick(stale_tick, worker, stale_epoch);

        assert_eq!(core.units[&worker].stats.current_health, hp_before);
        assert_eq!(core.units[&worker].resonance_current, resonance_before);
        assert_eq!(core.tactical_states[&worker].next_rest_tick_ms, None);
    }

    #[test]
    fn worker_death_stops_assignment_and_preserves_object_progress() {
        let (mut core, mut execution) = field_work_core(
            vec![FieldWorkCellDef {
                id: FieldWorkCellId::new("cell"),
                position: Position::new(1, 1),
                tile_kind: TacticalTileKind::Ground,
            }],
            1_000,
        );
        let worker = deploy_worker(&mut core, 0xF801, 7, Position::new(1, 1), 0);
        core.step_battle_execution_until(&mut execution, 250)
            .unwrap();
        core.apply_hp_delta_and_record(None, worker, -10_000, 250, HpChangeReason::Command);

        let object = &core.field_work_object_snapshots()[0];
        assert_eq!(object.status, FieldWorkObjectStatus::Active);
        assert_eq!(object.accumulated_work_ms, 250);
        assert_eq!(object.work_cells[0].worker_unit_id, None);
        assert!(core.units[&worker].is_dead());
        assert_eq!(
            core.event_log
                .entries
                .iter()
                .filter(|entry| matches!(
                    entry.event,
                    BattleLogEvent::FieldWorkAssignmentStopped { .. }
                ))
                .count(),
            1
        );
    }
}
