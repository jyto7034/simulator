use std::collections::BTreeMap;

use crate::game::{
    battle::{
        enums::BattleEvent,
        event_log::{BattleEventCause, BattleEventRootCause},
        ids::UnitInstanceId,
        scenario::{
            ScenarioGroupId, ScenarioSpawnGroup, ScenarioWaveActivation, ScenarioWavePrimaryTrigger,
        },
    },
    behavior::GameError,
};

use super::{
    BattleCore, RuntimeUnitLifecycle, SpawnCancellationReason, SpawnOccupancyClass,
    SpawnOccupancyPolicy, SpawnPlacement, SpawnRequest, SpawnRequestId, SpawnRequestSettlement,
    SpawnRequestSource, SpawnTriggerId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::game::battle::core) enum WaveActivationCause {
    Primary,
    Deadline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WaveTriggerStatus {
    Pending,
    Spawned { cause: WaveActivationCause },
    Canceled { dependency_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BoundActor {
    unit_id: UnitInstanceId,
    death_epoch_at_bind: u64,
}

#[derive(Debug, Clone)]
struct WaveTriggerRecord {
    group_id: ScenarioGroupId,
    activation: ScenarioWaveActivation,
    request: SpawnRequest,
    bound_actor: Option<BoundActor>,
    status: WaveTriggerStatus,
}

#[derive(Debug, Clone, Default)]
pub(in crate::game::battle::core) struct WaveRuntimeState {
    triggers: BTreeMap<String, WaveTriggerRecord>,
    death_epochs: BTreeMap<UnitInstanceId, u64>,
    next_death_epoch: u64,
    stress: Option<StressRounds>,
}

#[derive(Debug, Clone)]
struct StressRounds {
    rounds: Vec<Vec<String>>,
    current: usize,
    completed: usize,
    waiting: bool,
}
crate::authoring::clone_shape::struct_shape!(StressRounds { rounds, current, completed, waiting });

crate::authoring::clone_shape::copy_shape!(BoundActor, WaveActivationCause);
crate::authoring::clone_shape::struct_shape!(WaveRuntimeState {
    triggers,
    death_epochs,
    next_death_epoch,
    stress
});
crate::authoring::clone_shape::struct_shape!(WaveTriggerRecord {
    group_id,
    activation,
    request,
    bound_actor,
    status
});
crate::authoring::clone_shape::enum_shape!(WaveTriggerStatus {
    units: [Pending],
    tuples: [],
    records: [Spawned { cause }, Canceled { dependency_id }]
});

impl WaveRuntimeState {
    fn death_epoch(&self, unit_id: UnitInstanceId) -> u64 {
        self.death_epochs.get(&unit_id).copied().unwrap_or(0)
    }
}

impl BattleCore {
    pub(super) fn stress_protected_target(&self, id: UnitInstanceId) -> bool {
        self.scenario_runtime.wave_runtime.stress.is_some()
            && self.units.get(&id).is_some_and(|u| u.role == crate::game::battle::types::BattleUnitRole::DefenseObject)
    }
    pub(super) fn stress_allied_effect_allowed(&self, source: Option<UnitInstanceId>, target: UnitInstanceId) -> bool {
        !self.stress_protected_target(target) || source.and_then(|id| self.units.get(&id))
            .zip(self.units.get(&target)).is_some_and(|(a,b)| a.owner != b.owner)
    }
    pub(crate) fn configure_stress_rounds(&mut self, rounds: Vec<Vec<String>>) {
        self.scenario_runtime.wave_runtime.stress = Some(StressRounds { rounds, current: 0, completed: 0, waiting: false });
    }
    pub(crate) fn stress_round_status(&self) -> Option<(usize, usize, bool)> {
        self.scenario_runtime.wave_runtime.stress.as_ref().map(|s| (s.current, s.completed, s.waiting))
    }
    pub(crate) fn continue_stress_round(&mut self) -> Result<(), GameError> {
        let s = self.scenario_runtime.wave_runtime.stress.as_mut().ok_or(GameError::InvalidAction)?;
        if !s.waiting || s.completed >= s.rounds.len() { return Err(GameError::InvalidAction); }
        s.current += 1;
        s.waiting = false;
        Ok(())
    }
    pub(super) fn settle_stress_round(&mut self) {
        let Some(s) = self.scenario_runtime.wave_runtime.stress.as_ref() else { return; };
        if !s.waiting && s.rounds[s.current].iter().all(|id| self.scenario_group_is_clear(&ScenarioGroupId::new(id))) {
            let s = self.scenario_runtime.wave_runtime.stress.as_mut().unwrap();
            s.completed = s.current + 1;
            s.waiting = s.completed < s.rounds.len();
        }
    }
    pub(in crate::game::battle::core) fn register_wave_trigger(
        &mut self,
        group_id: ScenarioGroupId,
        activation: ScenarioWaveActivation,
    ) -> Result<(), GameError> {
        if self
            .scenario_runtime
            .wave_runtime
            .triggers
            .contains_key(&activation.trigger_id)
        {
            return Err(GameError::InvalidStaticData(format!(
                "duplicate Wave trigger '{}' reached runtime registration",
                activation.trigger_id
            )));
        }

        let group = self.scenario.group(&group_id).cloned().ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "Wave trigger '{}' references missing scenario group '{}'",
                activation.trigger_id, group_id.0
            ))
        })?;
        let request = self.wave_spawn_request(&activation.trigger_id, &group);
        let trigger_id = wave_spawn_trigger_id(&activation.trigger_id);
        self.register_spawn_trigger(trigger_id.clone())
            .map_err(|error| wave_kernel_error(&activation.trigger_id, error))?;
        self.bind_spawn_trigger(&trigger_id, request.clone())
            .map_err(|error| wave_kernel_error(&activation.trigger_id, error))?;

        self.scenario_runtime.wave_runtime.triggers.insert(
            activation.trigger_id.clone(),
            WaveTriggerRecord {
                group_id,
                activation,
                request,
                bound_actor: None,
                status: WaveTriggerStatus::Pending,
            },
        );
        Ok(())
    }

    fn wave_spawn_request(&self, trigger_id: &str, group: &ScenarioSpawnGroup) -> SpawnRequest {
        SpawnRequest {
            id: SpawnRequestId(format!("wave_spawn:{trigger_id}")),
            source: SpawnRequestSource::Wave {
                wave_id: group.id.0.clone(),
            },
            owner: group.side,
            occupancy: SpawnOccupancyPolicy::AllowOccupied(vec![SpawnOccupancyClass::Any]),
            placements: group
                .spawns
                .iter()
                .enumerate()
                .map(|(index, spawn)| SpawnPlacement {
                    order: index as u32,
                    position: spawn.position,
                    terrain_walkable: self.battlefield.is_walkable_tile(spawn.position),
                    occupants: Vec::new(),
                })
                .collect(),
        }
    }

    pub(in crate::game::battle::core) fn note_wave_actor_death(&mut self, unit_id: UnitInstanceId) {
        self.scenario_runtime.wave_runtime.next_death_epoch = self
            .scenario_runtime
            .wave_runtime
            .next_death_epoch
            .saturating_add(1);
        let epoch = self.scenario_runtime.wave_runtime.next_death_epoch;
        self.scenario_runtime
            .wave_runtime
            .death_epochs
            .insert(unit_id, epoch);
    }

    /// Settles pending Waves inside the current timestamp candidate, before its
    /// enclosing transaction commits. Ready triggers use a deterministic
    /// priority/ID sequence; newly bound already-true triggers are included
    /// before terminal outcome evaluation.
    pub(in crate::game::battle::core) fn settle_wave_triggers(
        &mut self,
        now_ms: u64,
    ) -> Result<bool, GameError> {
        let trigger_count = self.scenario_runtime.wave_runtime.triggers.len();
        let mut spawned_any = false;

        for _ in 0..=trigger_count {
            self.bind_wave_actors();

            let mut ready = self
                .scenario_runtime
                .wave_runtime
                .triggers
                .iter()
                .filter_map(|(trigger_id, record)| {
                    if record.status != WaveTriggerStatus::Pending {
                        return None;
                    }
                    self.wave_activation_cause(record, now_ms)
                        .map(|cause| (record.activation.priority, trigger_id.clone(), cause))
                })
                .collect::<Vec<_>>();
            ready.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));

            if let Some((_, trigger_id, cause)) = ready.into_iter().next() {
                self.spawn_wave_trigger(&trigger_id, cause, now_ms)?;
                spawned_any = true;
                continue;
            }

            let mut canceled = self
                .scenario_runtime
                .wave_runtime
                .triggers
                .iter()
                .filter_map(|(trigger_id, record)| {
                    if record.status != WaveTriggerStatus::Pending
                        || record.activation.deadline_ms.is_some()
                        || self
                            .scenario
                            .group(&record.group_id)
                            .is_some_and(|group| group.required_for_victory)
                    {
                        return None;
                    }
                    self.wave_unavailable_dependency(record)
                        .map(|dependency_id| (trigger_id.clone(), dependency_id))
                })
                .collect::<Vec<_>>();
            canceled.sort();
            if canceled.is_empty() {
                return Ok(spawned_any);
            }
            if let Some((trigger_id, dependency_id)) = canceled.into_iter().next() {
                self.cancel_wave_trigger(&trigger_id, dependency_id)?;
            }
        }

        Err(GameError::InvalidStaticData(
            "Wave trigger settlement did not reach a stable state".to_string(),
        ))
    }

    fn bind_wave_actors(&mut self) {
        let unbound = self
            .scenario_runtime
            .wave_runtime
            .triggers
            .iter()
            .filter_map(|(trigger_id, record)| {
                if record.status != WaveTriggerStatus::Pending || record.bound_actor.is_some() {
                    return None;
                }
                actor_slot(&record.activation.primary)
                    .cloned()
                    .map(|slot| (trigger_id.clone(), slot))
            })
            .collect::<Vec<_>>();

        for (trigger_id, actor_slot_id) in unbound {
            let Some(unit_id) = self
                .boss_actor_adapter_view(&actor_slot_id)
                .and_then(|view| view.unit_id)
            else {
                continue;
            };
            let death_epoch_at_bind = self.scenario_runtime.wave_runtime.death_epoch(unit_id);
            if let Some(record) = self
                .scenario_runtime
                .wave_runtime
                .triggers
                .get_mut(&trigger_id)
            {
                record.bound_actor = Some(BoundActor {
                    unit_id,
                    death_epoch_at_bind,
                });
            }
        }
    }

    fn wave_activation_cause(
        &self,
        record: &WaveTriggerRecord,
        now_ms: u64,
    ) -> Option<WaveActivationCause> {
        if self.scenario_runtime.wave_runtime.stress.as_ref().is_some_and(|s|
            !s.rounds[..=s.current].iter().any(|r| r.contains(&record.group_id.0))) {
            return None;
        }
        if self.wave_primary_is_true(record, now_ms) {
            return Some(WaveActivationCause::Primary);
        }
        record
            .activation
            .deadline_ms
            .is_some_and(|deadline_ms| now_ms >= deadline_ms)
            .then_some(WaveActivationCause::Deadline)
    }

    fn wave_primary_is_true(&self, record: &WaveTriggerRecord, now_ms: u64) -> bool {
        match &record.activation.primary {
            ScenarioWavePrimaryTrigger::AtTimeMs { time_ms } => now_ms >= *time_ms,
            ScenarioWavePrimaryTrigger::GroupCleared { group_id } => {
                self.scenario_group_is_clear(group_id)
            }
            ScenarioWavePrimaryTrigger::ActorHealthAtOrBelowPercent { percent, .. } => record
                .bound_actor
                .and_then(|bound| self.units.get(&bound.unit_id))
                .is_some_and(|unit| {
                    matches!(
                        unit.lifecycle,
                        RuntimeUnitLifecycle::Active | RuntimeUnitLifecycle::Dead
                    ) && u64::from(unit.stats.current_health) * 100
                        <= u64::from(unit.stats.max_health) * u64::from(*percent)
                }),
            ScenarioWavePrimaryTrigger::ActorDefeated { .. } => {
                record.bound_actor.is_some_and(|bound| {
                    self.scenario_runtime
                        .wave_runtime
                        .death_epoch(bound.unit_id)
                        > bound.death_epoch_at_bind
                })
            }
            ScenarioWavePrimaryTrigger::ActorReachedPoint {
                point_id,
                radius_cells,
                ..
            } => record
                .bound_actor
                .and_then(|bound| self.units.get(&bound.unit_id))
                .filter(|unit| unit.is_active())
                .and_then(|unit| {
                    self.scenario
                        .tactical_plan
                        .point(point_id)
                        .map(|point| (unit.body.projected_tile(), point.position))
                })
                .is_some_and(|(actor, point)| {
                    actor.chebyshev(&point) <= i32::try_from(*radius_cells).unwrap_or(i32::MAX)
                }),
        }
    }

    fn scenario_group_is_clear(&self, group_id: &ScenarioGroupId) -> bool {
        let Some(group) = self.scenario.group(group_id) else {
            return false;
        };
        !group.spawns.is_empty()
            && self
                .scenario_runtime
                .spawned
                .spawned_groups
                .contains(group_id)
            && group.spawns.iter().all(|spawn| {
                self.scenario_runtime
                    .spawned
                    .unit_refs
                    .get(&spawn.unit_ref)
                    .and_then(|unit_id| self.units.get(unit_id))
                    .is_none_or(|unit| !unit.is_active())
            })
    }

    fn wave_unavailable_dependency(&self, record: &WaveTriggerRecord) -> Option<String> {
        match &record.activation.primary {
            ScenarioWavePrimaryTrigger::GroupCleared { group_id } => self
                .wave_record_for_group(group_id)
                .and_then(|source| match &source.status {
                    WaveTriggerStatus::Canceled { .. } => Some(group_id.0.clone()),
                    _ => None,
                }),
            ScenarioWavePrimaryTrigger::ActorHealthAtOrBelowPercent { actor_slot_id, .. }
            | ScenarioWavePrimaryTrigger::ActorDefeated { actor_slot_id }
            | ScenarioWavePrimaryTrigger::ActorReachedPoint { actor_slot_id, .. } => {
                if let Some(bound) = record.bound_actor {
                    return self
                        .units
                        .get(&bound.unit_id)
                        .is_none_or(|unit| !unit.is_active() && !unit.is_dead())
                        .then(|| actor_slot_id.as_str().to_string());
                }
                let view = self.boss_actor_adapter_view(actor_slot_id)?;
                let source_group = self.scenario.groups.iter().find(|group| {
                    group
                        .spawns
                        .iter()
                        .any(|spawn| spawn.unit_ref == view.unit_ref)
                })?;
                self.wave_record_for_group(&source_group.id)
                    .and_then(|source| match &source.status {
                        WaveTriggerStatus::Canceled { .. } => {
                            Some(actor_slot_id.as_str().to_string())
                        }
                        _ => None,
                    })
            }
            ScenarioWavePrimaryTrigger::AtTimeMs { .. } => None,
        }
    }

    fn wave_record_for_group(&self, group_id: &ScenarioGroupId) -> Option<&WaveTriggerRecord> {
        self.scenario_runtime
            .wave_runtime
            .triggers
            .values()
            .find(|record| &record.group_id == group_id)
    }

    fn spawn_wave_trigger(
        &mut self,
        trigger_id: &str,
        cause: WaveActivationCause,
        now_ms: u64,
    ) -> Result<(), GameError> {
        let record = self
            .scenario_runtime
            .wave_runtime
            .triggers
            .get(trigger_id)
            .cloned()
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "missing registered Wave trigger '{trigger_id}'"
                ))
            })?;
        if record.status != WaveTriggerStatus::Pending {
            return Ok(());
        }
        let group = self
            .scenario
            .group(&record.group_id)
            .cloned()
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "Wave trigger '{trigger_id}' lost scenario group '{}'",
                    record.group_id.0
                ))
            })?;

        let settlement = self
            .execute_spawn_request(record.request.clone(), |core, placement| {
                core.spawn_scenario_group_member(&group, placement.order as usize)
            })
            .map_err(|error| wave_kernel_error(trigger_id, error))?;
        let spawned_unit_ids = match settlement {
            SpawnRequestSettlement::Spawned { unit_ids } => unit_ids,
            other => {
                return Err(GameError::InvalidStaticData(format!(
                    "Wave trigger '{trigger_id}' failed atomic spawn: {other:?}"
                )));
            }
        };
        let trigger = wave_spawn_trigger_id(trigger_id);
        let settled = self
            .settle_spawn_trigger(&trigger)
            .map_err(|error| wave_kernel_error(trigger_id, error))?;
        if settled
            != (SpawnRequestSettlement::Spawned {
                unit_ids: spawned_unit_ids.clone(),
            })
        {
            return Err(GameError::InvalidStaticData(format!(
                "Wave trigger '{trigger_id}' and Spawn Kernel settlement diverged"
            )));
        }

        self.scenario_runtime
            .spawned_for_write()?
            .spawned_groups
            .insert(record.group_id.clone());
        if let Some(record) = self
            .scenario_runtime
            .wave_runtime
            .triggers
            .get_mut(trigger_id)
        {
            record.status = WaveTriggerStatus::Spawned { cause };
        }
        self.record_spawned_units(
            now_ms,
            BattleEventCause::Root {
                kind: BattleEventRootCause::System,
            },
            &spawned_unit_ids,
        )?;
        for unit_id in spawned_unit_ids {
            self.schedule_initial_attack_for_unit(unit_id, now_ms);
        }
        Ok(())
    }

    fn cancel_wave_trigger(
        &mut self,
        trigger_id: &str,
        dependency_id: String,
    ) -> Result<(), GameError> {
        let request = self
            .scenario_runtime
            .wave_runtime
            .triggers
            .get(trigger_id)
            .map(|record| record.request.clone())
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "missing registered Wave trigger '{trigger_id}'"
                ))
            })?;
        let settlement = self
            .cancel_spawn_request(
                request,
                SpawnCancellationReason::WaveDependencyUnavailable {
                    dependency_id: dependency_id.clone(),
                },
            )
            .map_err(|error| wave_kernel_error(trigger_id, error))?;
        if !matches!(settlement, SpawnRequestSettlement::Canceled { .. }) {
            return Err(GameError::InvalidStaticData(format!(
                "Wave trigger '{trigger_id}' cancellation raced a terminal settlement"
            )));
        }
        self.settle_spawn_trigger(&wave_spawn_trigger_id(trigger_id))
            .map_err(|error| wave_kernel_error(trigger_id, error))?;
        if let Some(record) = self
            .scenario_runtime
            .wave_runtime
            .triggers
            .get_mut(trigger_id)
        {
            record.status = WaveTriggerStatus::Canceled { dependency_id };
        }
        Ok(())
    }

    pub(in crate::game::battle::core) fn enqueue_wave_wakes(
        &mut self,
        activation: &ScenarioWaveActivation,
    ) {
        if let ScenarioWavePrimaryTrigger::AtTimeMs { time_ms } = activation.primary {
            self.event_queue.push(BattleEvent::WaveTriggerWake {
                time_ms,
                trigger_id: activation.trigger_id.clone(),
            });
        }
        if let Some(time_ms) = activation.deadline_ms {
            self.event_queue.push(BattleEvent::WaveTriggerWake {
                time_ms,
                trigger_id: activation.trigger_id.clone(),
            });
        }
    }
}

fn actor_slot(
    primary: &ScenarioWavePrimaryTrigger,
) -> Option<&crate::game::data::boss_runtime_data::BossActorSlotId> {
    match primary {
        ScenarioWavePrimaryTrigger::ActorHealthAtOrBelowPercent { actor_slot_id, .. }
        | ScenarioWavePrimaryTrigger::ActorDefeated { actor_slot_id }
        | ScenarioWavePrimaryTrigger::ActorReachedPoint { actor_slot_id, .. } => {
            Some(actor_slot_id)
        }
        ScenarioWavePrimaryTrigger::AtTimeMs { .. }
        | ScenarioWavePrimaryTrigger::GroupCleared { .. } => None,
    }
}

fn wave_kernel_error(trigger_id: &str, error: super::SpawnKernelError) -> GameError {
    GameError::InvalidStaticData(format!(
        "Wave trigger '{trigger_id}' violated Spawn Kernel identity: {error:?}"
    ))
}

fn wave_spawn_trigger_id(trigger_id: &str) -> SpawnTriggerId {
    SpawnTriggerId(format!("wave:{trigger_id}"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use super::*;
    use crate::game::{
        ability::SkillActivationMode,
        battle::{
            scenario::{
                BattleFieldSpec, BattleScenario, EnemyMovementPlan, ScenarioAction, ScenarioEvent,
                ScenarioEventId, ScenarioTrigger, ScenarioUnitRef, ScenarioUnitSpawn, TacticalPlan,
                TacticalPoint, TacticalPointId,
            },
            types::{BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass, UnitCombatProfile},
        },
        data::{
            boss_runtime_data::{
                BossActorSlotId, BossEncounterRuntimeId, ResolvedBossActorSetup,
                ResolvedBossEncounterSetup,
            },
            GameDataBase, GameDataBuilder,
        },
        enums::{Side, Tier},
        growth::GrowthStack,
        resources::Position,
        stats::UnitStats,
    };

    fn fixture_data() -> Arc<GameDataBase> {
        GameDataBuilder::empty().build_arc()
    }

    fn fixture_draft(id: u128, hp: u32) -> BattleUnitDraft {
        let mut profile = UnitCombatProfile::employee_default();
        profile.weapon_profile = Some(Default::default());
        profile.stats = UnitStats::with_values(hp, hp, 1, 0, 1_000);
        profile.stats.move_speed_units_per_ms = 0;
        profile.basic_attack.interval_ms = 100_000;
        profile.basic_attack.windup_ms = 1_000;
        profile.skill_activation_mode = SkillActivationMode::Auto;
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(id),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(id + 10_000),
                profile,
            },
            threat_class: BattleUnitThreatClass::Boss,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        }
    }

    fn boss_setup() -> ResolvedBossEncounterSetup {
        let database = crate::game::data::boss_runtime_data::tests::valid_catalog()
            .compile()
            .expect("valid Boss fixture");
        ResolvedBossEncounterSetup {
            definition: database
                .get_encounter(&BossEncounterRuntimeId::new("tracking_encounter"))
                .expect("tracking encounter")
                .clone(),
            actors: vec![ResolvedBossActorSetup {
                actor_slot_index: 0,
                unit_ref: ScenarioUnitRef::new("boss_0"),
            }],
            enemy_stat_scale: Default::default(),
            summon_spawn_zones: Vec::new(),
            summon_routes: Vec::new(),
        }
    }

    fn wave_group(
        id: &str,
        unit_ref: &str,
        position: Position,
        salt: u32,
        required: bool,
    ) -> ScenarioSpawnGroup {
        ScenarioSpawnGroup {
            id: ScenarioGroupId::new(id),
            side: Side::Opponent,
            required_for_victory: required,
            enemy_movement_plan: Some(EnemyMovementPlan::PathAlongCells {
                cells: vec![position, Position::new(1, position.y)],
            }),
            spawns: vec![ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(unit_ref),
                side: Side::Opponent,
                draft: fixture_draft(100 + u128::from(salt), 1_000),
                position,
                instance_salt: salt,
            }],
        }
    }

    fn wave_event(
        group_id: &str,
        trigger_id: &str,
        priority: i32,
        primary: ScenarioWavePrimaryTrigger,
        deadline_ms: Option<u64>,
    ) -> ScenarioEvent {
        ScenarioEvent {
            id: ScenarioEventId::new(format!("event_{trigger_id}")),
            trigger: ScenarioTrigger::Wave(ScenarioWaveActivation {
                trigger_id: trigger_id.to_string(),
                priority,
                primary,
                deadline_ms,
            }),
            action: ScenarioAction::SpawnGroup {
                group_id: ScenarioGroupId::new(group_id),
            },
        }
    }

    fn scenario(
        wave_groups: Vec<ScenarioSpawnGroup>,
        wave_events: Vec<ScenarioEvent>,
        with_boss: bool,
    ) -> BattleScenario {
        let player_group = ScenarioSpawnGroup {
            id: ScenarioGroupId::new("player"),
            side: Side::Player,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new("player_0"),
                side: Side::Player,
                draft: fixture_draft(1, 10_000),
                position: Position::new(0, 0),
                instance_salt: 0,
            }],
        };
        let mut groups = vec![player_group];
        groups.extend(wave_groups);
        let mut events = vec![ScenarioEvent {
            id: ScenarioEventId::new("event_player"),
            trigger: ScenarioTrigger::AtBattleStart,
            action: ScenarioAction::SpawnGroup {
                group_id: ScenarioGroupId::new("player"),
            },
        }];
        events.extend(wave_events);
        BattleScenario {
            battlefield: BattleFieldSpec {
            walkable_paths: Vec::new(),
                object_placement: None,
                width: 10,
                height: 6,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            groups,
            events,
            terminal_outcomes: BattleScenario::fixture_required_group_outcomes(),
            tactical_plan: TacticalPlan {
                points: vec![TacticalPoint {
                    id: TacticalPointId::new("threshold_point"),
                    position: Position::new(6, 1),
                }],
                ..TacticalPlan::default()
            },
            boss_runtime: with_boss.then(boss_setup),
        }
    }

    fn step_zero(core: &mut BattleCore) -> super::super::sim::BattleExecutionState {
        let mut state = core.start_battle_execution().expect("battle starts");
        assert_eq!(
            core.step_battle_execution(&mut state)
                .expect("time-zero bucket"),
            super::super::sim::BattleStepOutcome::Running
        );
        state
    }

    fn status(core: &BattleCore, trigger_id: &str) -> WaveTriggerStatus {
        core.scenario_runtime.wave_runtime.triggers[trigger_id]
            .status
            .clone()
    }

    #[test]
    fn same_bucket_time_triggers_use_priority_then_stable_trigger_id() {
        let groups = vec![
            wave_group("wave_low", "low_0", Position::new(7, 1), 1, true),
            wave_group("wave_b", "b_0", Position::new(8, 2), 2, true),
            wave_group("wave_a", "a_0", Position::new(8, 3), 3, true),
        ];
        let events = vec![
            wave_event(
                "wave_low",
                "low",
                0,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "wave_b",
                "b",
                10,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "wave_a",
                "a",
                10,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
        ];
        let mut core =
            BattleCore::new_from_scenario(scenario(groups, events, false), fixture_data(), 7);
        let _state = step_zero(&mut core);

        let order = |unit_ref: &str| {
            let unit_id = core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new(unit_ref)];
            core.units[&unit_id].spawn_order
        };
        assert!(order("a_0") < order("b_0"));
        assert!(order("b_0") < order("low_0"));
    }

    #[test]
    fn primary_wins_an_equal_deadline_and_required_unspawned_wave_gates_victory() {
        let group = wave_group("wave", "wave_0", Position::new(7, 1), 1, true);
        let event = wave_event(
            "wave",
            "time_gate",
            0,
            ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 10 },
            None,
        );
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![group], vec![event], false),
            fixture_data(),
            8,
        );
        let mut state = step_zero(&mut core);
        assert!(matches!(
            status(&core, "time_gate"),
            WaveTriggerStatus::Pending
        ));
        assert_eq!(
            core.step_battle_execution(&mut state).expect("time gate"),
            super::super::sim::BattleStepOutcome::Running
        );
        assert_eq!(
            status(&core, "time_gate"),
            WaveTriggerStatus::Spawned {
                cause: WaveActivationCause::Primary
            }
        );

        // Equal primary/deadline is invalid in publication for AtTimeMs, so the
        // runtime cause tie is exercised with an already-clear source.
        let source = wave_group("source", "source_0", Position::new(6, 2), 11, true);
        let target = wave_group("target", "target_0", Position::new(8, 2), 12, true);
        let events = vec![
            wave_event(
                "source",
                "source",
                0,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "target",
                "target",
                0,
                ScenarioWavePrimaryTrigger::GroupCleared {
                    group_id: ScenarioGroupId::new("source"),
                },
                Some(1),
            ),
        ];
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![source, target], events, false),
            fixture_data(),
            9,
        );
        let _state = step_zero(&mut core);
        let source_id = core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("source_0")];
        core.units.get_mut(&source_id).unwrap().lifecycle = RuntimeUnitLifecycle::Despawned;
        core.settle_wave_triggers(1).unwrap();
        assert_eq!(
            status(&core, "target"),
            WaveTriggerStatus::Spawned {
                cause: WaveActivationCause::Primary
            }
        );
    }

    #[test]
    fn group_clear_counts_non_kill_inactive_members_and_spawns_once() {
        let source = wave_group("source", "source_0", Position::new(6, 1), 21, true);
        let target = wave_group("target", "target_0", Position::new(8, 1), 22, true);
        let events = vec![
            wave_event(
                "source",
                "source",
                0,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "target",
                "target",
                0,
                ScenarioWavePrimaryTrigger::GroupCleared {
                    group_id: ScenarioGroupId::new("source"),
                },
                None,
            ),
        ];
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![source, target], events, false),
            fixture_data(),
            10,
        );
        let _state = step_zero(&mut core);
        let source_id = core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("source_0")];
        core.units.get_mut(&source_id).unwrap().lifecycle = RuntimeUnitLifecycle::Despawned;
        assert!(core.settle_wave_triggers(1).unwrap());
        assert!(!core.settle_wave_triggers(2).unwrap());
        assert_eq!(
            status(&core, "target"),
            WaveTriggerStatus::Spawned {
                cause: WaveActivationCause::Primary
            }
        );
    }

    #[test]
    fn actor_hp_and_point_are_evaluated_immediately_on_first_binding() {
        let boss = wave_group("boss", "boss_0", Position::new(6, 1), 31, true);
        let hp_wave = wave_group("hp", "hp_0", Position::new(8, 1), 32, true);
        let point_wave = wave_group("point", "point_0", Position::new(8, 2), 33, true);
        let low_wave = wave_group("low", "low_0", Position::new(9, 2), 34, true);
        let slot = BossActorSlotId::new("primary");
        let events = vec![
            wave_event(
                "boss",
                "boss",
                100,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "hp",
                "hp",
                10,
                ScenarioWavePrimaryTrigger::ActorHealthAtOrBelowPercent {
                    actor_slot_id: slot.clone(),
                    percent: 100,
                },
                None,
            ),
            wave_event(
                "point",
                "point",
                0,
                ScenarioWavePrimaryTrigger::ActorReachedPoint {
                    actor_slot_id: slot,
                    point_id: TacticalPointId::new("threshold_point"),
                    radius_cells: 1,
                },
                Some(100),
            ),
            wave_event(
                "low",
                "low",
                -10,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
        ];
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![boss, hp_wave, point_wave, low_wave], events, true),
            fixture_data(),
            11,
        );
        let _state = step_zero(&mut core);
        assert!(matches!(
            status(&core, "hp"),
            WaveTriggerStatus::Spawned { .. }
        ));
        assert!(matches!(
            status(&core, "point"),
            WaveTriggerStatus::Spawned { .. }
        ));
        let spawn_order = |unit_ref: &str| {
            let unit_id = core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new(unit_ref)];
            core.units[&unit_id].spawn_order
        };
        assert!(spawn_order("hp_0") < spawn_order("point_0"));
        assert!(spawn_order("point_0") < spawn_order("low_0"));
    }

    #[test]
    fn one_health_change_settles_every_crossed_threshold_before_returning() {
        let boss = wave_group("boss", "boss_0", Position::new(6, 1), 35, true);
        let high = wave_group("high", "high_0", Position::new(8, 1), 36, true);
        let low = wave_group("low", "low_0", Position::new(8, 2), 37, true);
        let slot = BossActorSlotId::new("primary");
        let events = vec![
            wave_event(
                "boss",
                "boss",
                10,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "high",
                "high",
                20,
                ScenarioWavePrimaryTrigger::ActorHealthAtOrBelowPercent {
                    actor_slot_id: slot.clone(),
                    percent: 75,
                },
                None,
            ),
            wave_event(
                "low",
                "low",
                10,
                ScenarioWavePrimaryTrigger::ActorHealthAtOrBelowPercent {
                    actor_slot_id: slot,
                    percent: 50,
                },
                None,
            ),
        ];
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![boss, high, low], events, true),
            fixture_data(),
            17,
        );
        let _state = step_zero(&mut core);
        let boss_id = core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("boss_0")];
        core.units.get_mut(&boss_id).unwrap().stats.current_health = 400;
        assert!(core.settle_wave_triggers(1).unwrap());
        assert!(matches!(
            status(&core, "high"),
            WaveTriggerStatus::Spawned { .. }
        ));
        assert!(matches!(
            status(&core, "low"),
            WaveTriggerStatus::Spawned { .. }
        ));
        let high_id = core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("high_0")];
        let low_id = core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("low_0")];
        assert!(core.units[&high_id].spawn_order < core.units[&low_id].spawn_order);
    }

    #[test]
    fn actor_defeated_observes_only_a_post_bind_death_epoch_and_restore_rolls_it_back() {
        let boss = wave_group("boss", "boss_0", Position::new(6, 1), 41, true);
        let death_wave = wave_group("death", "death_0", Position::new(8, 1), 42, true);
        let events = vec![
            wave_event(
                "boss",
                "boss",
                10,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "death",
                "death",
                0,
                ScenarioWavePrimaryTrigger::ActorDefeated {
                    actor_slot_id: BossActorSlotId::new("primary"),
                },
                None,
            ),
        ];
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![boss, death_wave], events, true),
            fixture_data(),
            12,
        );
        let _state = step_zero(&mut core);
        assert!(matches!(status(&core, "death"), WaveTriggerStatus::Pending));
        let before_death = core.clone();
        let boss_id = core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("boss_0")];
        core.finalize_unit_death(None, boss_id, Side::Opponent, 1);
        assert!(core.settle_wave_triggers(1).unwrap());
        assert!(matches!(
            status(&core, "death"),
            WaveTriggerStatus::Spawned { .. }
        ));

        core = before_death;
        assert!(!core.settle_wave_triggers(2).unwrap());
        assert!(matches!(status(&core, "death"), WaveTriggerStatus::Pending));
        core.units.get_mut(&boss_id).unwrap().lifecycle = RuntimeUnitLifecycle::Despawned;
        assert!(!core.settle_wave_triggers(3).unwrap());
        assert!(matches!(status(&core, "death"), WaveTriggerStatus::Pending));
    }

    #[test]
    fn pre_bind_death_epoch_is_baselined_and_does_not_fire_after_binding() {
        let boss = wave_group("boss", "boss_0", Position::new(6, 1), 45, true);
        let death_wave = wave_group("death", "death_0", Position::new(8, 1), 46, true);
        let events = vec![
            wave_event(
                "boss",
                "boss",
                10,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "death",
                "death",
                0,
                ScenarioWavePrimaryTrigger::ActorDefeated {
                    actor_slot_id: BossActorSlotId::new("primary"),
                },
                None,
            ),
        ];
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![boss, death_wave], events, true),
            fixture_data(),
            14,
        );
        let mut state = core.start_battle_execution().unwrap();
        let predicted_id =
            BattleCore::make_instance_id(Uuid::from_u128(100 + 45 + 10_000), Side::Opponent, 45);
        core.note_wave_actor_death(predicted_id);
        assert_eq!(
            core.step_battle_execution(&mut state).unwrap(),
            super::super::sim::BattleStepOutcome::Running
        );
        assert_eq!(
            core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("boss_0")],
            predicted_id
        );
        assert!(matches!(status(&core, "death"), WaveTriggerStatus::Pending));
    }

    #[test]
    fn deadline_spawns_when_primary_is_still_false() {
        let source = wave_group("source", "source_0", Position::new(6, 1), 61, true);
        let target = wave_group("target", "target_0", Position::new(8, 1), 62, true);
        let events = vec![
            wave_event(
                "source",
                "source",
                10,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "target",
                "target",
                0,
                ScenarioWavePrimaryTrigger::GroupCleared {
                    group_id: ScenarioGroupId::new("source"),
                },
                Some(5),
            ),
        ];
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![source, target], events, false),
            fixture_data(),
            15,
        );
        let mut state = step_zero(&mut core);
        assert_eq!(
            core.step_battle_execution(&mut state).unwrap(),
            super::super::sim::BattleStepOutcome::Running
        );
        assert_eq!(
            status(&core, "target"),
            WaveTriggerStatus::Spawned {
                cause: WaveActivationCause::Deadline
            }
        );
    }

    #[test]
    fn nonrequired_deadline_free_chain_cancels_stably_after_actor_despawn() {
        let boss = wave_group("boss", "boss_0", Position::new(6, 1), 71, true);
        let actor_wait = wave_group("actor_wait", "actor_wait_0", Position::new(8, 1), 72, false);
        let chain = wave_group("chain", "chain_0", Position::new(8, 2), 73, false);
        let events = vec![
            wave_event(
                "boss",
                "boss",
                10,
                ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
                None,
            ),
            wave_event(
                "actor_wait",
                "actor_wait",
                0,
                ScenarioWavePrimaryTrigger::ActorDefeated {
                    actor_slot_id: BossActorSlotId::new("primary"),
                },
                None,
            ),
            wave_event(
                "chain",
                "chain",
                0,
                ScenarioWavePrimaryTrigger::GroupCleared {
                    group_id: ScenarioGroupId::new("actor_wait"),
                },
                None,
            ),
        ];
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![boss, actor_wait, chain], events, true),
            fixture_data(),
            16,
        );
        let _state = step_zero(&mut core);
        let boss_id = core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("boss_0")];
        core.units.get_mut(&boss_id).unwrap().lifecycle = RuntimeUnitLifecycle::Despawned;
        assert!(!core.settle_wave_triggers(1).unwrap());
        assert!(matches!(
            status(&core, "actor_wait"),
            WaveTriggerStatus::Canceled { .. }
        ));
        assert!(matches!(
            status(&core, "chain"),
            WaveTriggerStatus::Canceled { .. }
        ));
    }

    #[test]
    fn kernel_rejection_rolls_back_the_entire_wave_batch() {
        let mut duplicate = wave_group("duplicate", "duplicate_0", Position::new(6, 1), 51, true);
        let mut second = duplicate.spawns[0].clone();
        second.unit_ref = ScenarioUnitRef::new("duplicate_1");
        second.position = Position::new(7, 1);
        duplicate.spawns.push(second);
        let event = wave_event(
            "duplicate",
            "duplicate",
            0,
            ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
            None,
        );
        let mut core = BattleCore::new_from_scenario(
            scenario(vec![duplicate], vec![event], false),
            fixture_data(),
            13,
        );
        let mut state = core.start_battle_execution().unwrap();
        let error = core
            .step_battle_execution(&mut state)
            .expect_err("duplicate deterministic unit ID must reject atomically");
        assert!(matches!(error, GameError::InvalidStaticData(_)));
        assert!(!core
            .scenario_runtime
            .spawned
            .unit_refs
            .contains_key(&ScenarioUnitRef::new("duplicate_0")));
        assert!(!core
            .scenario_runtime
            .spawned
            .unit_refs
            .contains_key(&ScenarioUnitRef::new("duplicate_1")));
    }
}
