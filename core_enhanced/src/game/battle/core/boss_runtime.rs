use crate::game::{
    ability::SkillId,
    battle::{
        event_log::SkillCastTarget,
        runtime_facts::{ActionHandle, ActionTerminalKind},
        tile_range::FacingDirection,
    },
    data::boss_runtime_data::{
        BossActionMilestoneData, BossActionTargetData, BossActionTimeoutData,
    },
};
use crate::owned_collections::OwnedHashMap;
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    sync::Arc,
};

use uuid::Uuid;

use crate::game::{
    battle::{
        core::{
            movement::{
                types::{MovementGoal, WorldVec2},
                ActionState,
            },
            BattleCore,
        },
        enums::BattleEvent,
        event_log::{BattleLogEvent, MovementStopReason},
        ids::UnitInstanceId,
        scenario::{EnemyMovementPlan, ScenarioUnitRef},
    },
    behavior::GameError,
    data::boss_runtime_data::{
        BossActorSlotId, BossConditionData, BossCooldownGroupId, BossEncounterConditionData,
        BossEventConditionData, BossEventMemoryData, BossPhaseTransitionTimingData, BossSignalId,
        BossStatePredicateData, BossTargetFilterData, BossTargetKindData, BossTargetLockPolicyData,
        BossTargetOrderKeyData, BossTargetQueryData, BossTaskId, BossTaskResultData,
        CompiledBossActorBehaviorDefinition, CompiledBossControlNode, CompiledBossDurationValue,
        CompiledBossEncounterDefinition, CompiledBossPatternCandidate,
        CompiledBossPatternDefinition, CompiledBossPatternTransition,
        CompiledBossPatternTransitionTarget, CompiledBossSummonInsufficientSpace,
        CompiledBossTargetQueryValue, CompiledBossTask, ResolvedBossArgumentFrame,
        ResolvedBossEncounterSetup, ResolvedBossParameterValue, ResolvedBossSummonRoute,
        ResolvedBossSummonSpawnZone,
    },
    resources::Position,
};

const DEFAULT_BOSS_MICROSTEP_LIMIT: usize = 128;
const MOVE_TO_TARGET_STOP_CHEBYSHEV: i32 = 1;
const ROUTE_CELL_STOP_RADIUS: f32 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum BossMovementDirective {
    OrdinaryFallback,
    Suppress,
    Goal(MovementGoal),
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BossActorRuntimeDebugSnapshot {
    pub unit_id: Option<UnitInstanceId>,
    pub control_node_id: Option<String>,
    pub pattern_id: Option<String>,
    pub pattern_activation_id: Option<u64>,
    pub state_id: Option<String>,
    pub state_activation_id: Option<u64>,
    pub target_id: Option<UnitInstanceId>,
    pub route_cursor_index: usize,
}

/// Authority-free actor-slot observation used by encounter Wave triggers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::game::battle::core) struct BossActorAdapterView {
    pub actor_slot_id: BossActorSlotId,
    pub unit_ref: ScenarioUnitRef,
    pub unit_id: Option<UnitInstanceId>,
    pub ever_bound: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct BossEncounterRuntime {
    definition: Arc<CompiledBossEncounterDefinition>,
    initialized: bool,
    current_phase_index: usize,
    phase_activation_id: Option<u64>,
    pending_phase_transition: Option<PendingBossPhaseTransition>,
    actors: Vec<BossActorBehaviorRuntime>,
    encounter_actor_count: usize,
    next_actor_runtime_key: usize,
    enemy_stat_scale: crate::game::battle::types::BattleUnitStatScale,
    summon_spawn_zones: Vec<ResolvedBossSummonSpawnZone>,
    summon_routes: Vec<ResolvedBossSummonRoute>,
    signal_epochs: BTreeMap<BossSignalId, u64>,
    latched_signals: HashSet<BossSignalId>,
    next_activation_id: u64,
    microstep_limit: usize,
}

#[derive(Debug, Clone)]
struct PendingBossPhaseTransition {
    target_phase_index: usize,
    timing: BossPhaseTransitionTimingData,
    actor_boundary_reached: Vec<bool>,
}

#[derive(Debug, Clone)]
struct BossActorBehaviorRuntime {
    /// Encounter actors use their slot index; summoned actors receive a
    /// monotonically allocated runtime key used by wake events only.
    actor_slot_index: usize,
    encounter_slot_index: Option<usize>,
    unit_ref: Option<ScenarioUnitRef>,
    unit_id: Option<UnitInstanceId>,
    ever_bound: bool,
    behavior: Option<Arc<CompiledBossActorBehaviorDefinition>>,
    behavior_activation_id: Option<u64>,
    control: Option<ActiveBossControlNode>,
    fixed_sequence_cursors: OwnedHashMap<usize, usize>,
    cooldown_ready_at_ms: OwnedHashMap<BossCooldownGroupId, u64>,
    target_lock: Option<BossTargetLock>,
    route_cursor_index: usize,
    waiting_for_phase_transition: bool,
}

#[derive(Debug, Clone)]
struct ActiveBossControlNode {
    node_index: usize,
    activation_id: u64,
    resident_state: Option<ActiveBossState>,
    pattern: Option<ActiveBossPattern>,
}

#[derive(Debug, Clone)]
struct ActiveBossPattern {
    definition: Arc<CompiledBossPatternDefinition>,
    argument_frame: Arc<ResolvedBossArgumentFrame>,
    activation_id: u64,
    state_index: usize,
    state: ActiveBossState,
    loop_counters: OwnedHashMap<String, u32>,
    batch_handles: OwnedHashMap<crate::game::data::boss_runtime_data::SummonBatchHandleId, Uuid>,
    cleared_batch_handles: HashSet<crate::game::data::boss_runtime_data::SummonBatchHandleId>,
}

#[derive(Debug, Clone)]
struct ActiveBossState {
    activation_id: u64,
    tasks: Vec<ActiveBossTask>,
    target_invalidated_pulse: bool,
    target_invalidated_latch: bool,
}

#[derive(Debug, Clone)]
struct ActiveBossTask {
    id: BossTaskId,
    activation_id: u64,
    kind: ActiveBossTaskKind,
    result: Option<BossTaskResultData>,
    result_consumed: bool,
}

#[derive(Debug, Clone)]
enum ActiveBossTaskKind {
    StartAction {
        action: Option<ActionHandle>,
        deadline_ms: u64,
        milestone: BossActionMilestoneData,
        on_timeout: BossActionTimeoutData,
    },
    RouteCombat,
    AcquireTarget,
    MoveToTarget {
        deadline_ms: u64,
    },
    RejoinForwardRoute {
        deadline_ms: u64,
        target_route_index: Option<usize>,
    },
    Wait {
        completes_at_ms: u64,
    },
    Summon(ActiveBossSummonTask),
    EmitSignal,
}

#[derive(Debug, Clone)]
pub(super) struct ActiveBossSummonTask {
    pub request_id: String,
    pub definition_id: crate::game::data::summoned_unit_data::SummonedUnitDefinitionId,
    pub count: u32,
    pub batch_handle_id: crate::game::data::boss_runtime_data::SummonBatchHandleId,
    pub placement: crate::game::data::boss_runtime_data::BossSummonPlacementData,
    pub occupancy_policy: crate::game::data::boss_runtime_data::BossSummonOccupancyPolicyData,
    pub insufficient_space: ActiveBossSummonInsufficientSpace,
    pub route_id: Option<String>,
    pub lifetime: Option<ActiveBossSummonLifetime>,
    pub removal_signal: Option<crate::game::data::boss_runtime_data::BossSummonRemovalSignalData>,
    pub admitted_at_ms: u64,
}

#[derive(Debug, Clone)]
pub(super) enum ActiveBossSummonInsufficientSpace {
    Fail,
    WaitUntilAvailable { deadline_ms: u64 },
}

#[derive(Debug, Clone)]
pub(super) struct ActiveBossSummonLifetime {
    pub duration_ms: u64,
    pub mode: crate::game::data::boss_runtime_data::BossSummonRemovalModeData,
}

#[derive(Debug, Clone)]
pub(super) struct BossSummonTaskView {
    pub actor_runtime_key: usize,
    pub summoner_unit_id: UnitInstanceId,
    pub behavior_activation_id: u64,
    pub pattern_activation_id: u64,
    pub state_activation_id: u64,
    pub task_activation_id: u64,
    pub task: ActiveBossSummonTask,
}

// One typed field list drives the borrowed owned-output projection and its
// actual native copies. The complete target literal rejects added/mistyped fields.
// This is not the native CloneShape of a reference-only projection object.
macro_rules! summon_view_projection {
    ($($field:ident: $ty:ty),* $(,)?) => {
        struct BossSummonTaskViewProjection<'a> {
            $($field: &'a $ty),*
        }
        impl BossSummonTaskViewProjection<'_> {
            fn visit_owned_shape(&self, meter: &mut crate::authoring::control::CloneMeter)
                -> Result<(), GameError>
            {
                meter.scan_storage(Some(1))?;
                $(<$ty as crate::authoring::clone_shape::CloneShape>::visit_clone_shape(self.$field, meter)?;)*
                Ok(())
            }
            fn into_owned(self) -> BossSummonTaskView {
                let Self { $($field),* } = self;
                BossSummonTaskView { $($field: <$ty as Clone>::clone($field)),* }
            }
        }
    };
}
summon_view_projection! {
    actor_runtime_key: usize,
    summoner_unit_id: UnitInstanceId,
    behavior_activation_id: u64,
    pattern_activation_id: u64,
    state_activation_id: u64,
    task_activation_id: u64,
    task: ActiveBossSummonTask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BossTargetLockOwner {
    Pattern(u64),
    ResidentState(u64),
}

#[derive(Debug, Clone)]
struct BossTargetLock {
    target_id: UnitInstanceId,
    owner: BossTargetLockOwner,
    policy: BossTargetLockPolicyData,
}

// Both sets have closed constructor/insert/whole-clone provenance. They never
// remove entries or accept arbitrary external native tables in an application.
crate::authoring::clone_shape::struct_shape_with_sets!(BossEncounterRuntime {
    definition, initialized, current_phase_index, phase_activation_id, pending_phase_transition,
    actors, encounter_actor_count, next_actor_runtime_key, enemy_stat_scale, summon_spawn_zones,
    summon_routes, signal_epochs, next_activation_id, microstep_limit,
} sets { latched_signals });
crate::authoring::clone_shape::struct_shape_with_sets!(ActiveBossPattern {
    definition, argument_frame, activation_id, state_index, state, loop_counters, batch_handles,
} sets { cleared_batch_handles });
crate::authoring::clone_shape::struct_shape!(PendingBossPhaseTransition {
    target_phase_index,
    timing,
    actor_boundary_reached,
});
crate::authoring::clone_shape::struct_shape!(BossActorBehaviorRuntime {
    actor_slot_index,
    encounter_slot_index,
    unit_ref,
    unit_id,
    ever_bound,
    behavior,
    behavior_activation_id,
    control,
    fixed_sequence_cursors,
    cooldown_ready_at_ms,
    target_lock,
    route_cursor_index,
    waiting_for_phase_transition,
});
crate::authoring::clone_shape::struct_shape!(ActiveBossControlNode {
    node_index,
    activation_id,
    resident_state,
    pattern
});
crate::authoring::clone_shape::struct_shape!(ActiveBossState {
    activation_id,
    tasks,
    target_invalidated_pulse,
    target_invalidated_latch
});
crate::authoring::clone_shape::struct_shape!(ActiveBossTask {
    id,
    activation_id,
    kind,
    result,
    result_consumed
});
crate::authoring::clone_shape::struct_shape!(ActiveBossSummonTask {
    request_id,
    definition_id,
    count,
    batch_handle_id,
    placement,
    occupancy_policy,
    insufficient_space,
    route_id,
    lifetime,
    removal_signal,
    admitted_at_ms,
});
crate::authoring::clone_shape::struct_shape!(ActiveBossSummonLifetime { duration_ms, mode });
crate::authoring::clone_shape::struct_shape!(BossSummonTaskView {
    actor_runtime_key,
    summoner_unit_id,
    behavior_activation_id,
    pattern_activation_id,
    state_activation_id,
    task_activation_id,
    task,
});
crate::authoring::clone_shape::struct_shape!(BossTargetLock {
    target_id,
    owner,
    policy
});
crate::authoring::clone_shape::copy_shape!(BossTargetLockOwner);
crate::authoring::clone_shape::enum_shape!(ActiveBossTaskKind {
    units: [RouteCombat, AcquireTarget, EmitSignal],
    tuples: [Summon(task)],
    records: [
        StartAction {
            action,
            deadline_ms,
            milestone,
            on_timeout
        },
        MoveToTarget { deadline_ms },
        RejoinForwardRoute {
            deadline_ms,
            target_route_index
        },
        Wait { completes_at_ms }
    ]
});
crate::authoring::clone_shape::enum_shape!(ActiveBossSummonInsufficientSpace {
    units: [Fail],
    tuples: [],
    records: [WaitUntilAvailable { deadline_ms }]
});

#[derive(Debug, Clone)]
struct BossRuntimeTransitionPlan {
    next_runtime: BossEncounterRuntime,
    effects: Vec<BossRuntimeEffect>,
}

#[derive(Debug, Clone)]
enum BossRuntimeEffect {
    StartAction {
        unit_id: UnitInstanceId,
        actor_index: usize,
        task_activation_id: u64,
        skill_id: SkillId,
        target: Option<SkillCastTarget>,
        facing: FacingDirection,
    },
    DetachAction {
        unit_id: UnitInstanceId,
        action: ActionHandle,
        cancel_unreleased: bool,
    },
    EnqueueWake {
        time_ms: u64,
        actor_slot_index: usize,
        task_activation_id: u64,
    },
    CancelOwnedWork {
        unit_id: UnitInstanceId,
        cancel_combat: bool,
        clear_leaf_target: bool,
    },
    ResumeRouteCombat {
        unit_id: UnitInstanceId,
        time_ms: u64,
    },
    EmitSignal {
        signal_id: BossSignalId,
    },
}

impl BossEncounterRuntime {
    pub(crate) fn from_setup(setup: &ResolvedBossEncounterSetup) -> Self {
        let mut actors = setup
            .actors
            .iter()
            .map(|actor| BossActorBehaviorRuntime {
                actor_slot_index: actor.actor_slot_index,
                encounter_slot_index: Some(actor.actor_slot_index),
                unit_ref: Some(actor.unit_ref.clone()),
                unit_id: None,
                ever_bound: false,
                behavior: None,
                behavior_activation_id: None,
                control: None,
                fixed_sequence_cursors: OwnedHashMap::new(),
                cooldown_ready_at_ms: OwnedHashMap::new(),
                target_lock: None,
                route_cursor_index: 0,
                waiting_for_phase_transition: false,
            })
            .collect::<Vec<_>>();
        actors.sort_by_key(|actor| actor.actor_slot_index);
        let encounter_actor_count = actors.len();
        Self {
            definition: setup.definition.clone(),
            initialized: false,
            current_phase_index: setup.definition.entry_phase_index,
            phase_activation_id: None,
            pending_phase_transition: None,
            actors,
            encounter_actor_count,
            next_actor_runtime_key: encounter_actor_count,
            enemy_stat_scale: setup.enemy_stat_scale,
            summon_spawn_zones: setup.summon_spawn_zones.clone(),
            summon_routes: setup.summon_routes.clone(),
            signal_epochs: BTreeMap::new(),
            latched_signals: HashSet::new(),
            next_activation_id: 1,
            microstep_limit: DEFAULT_BOSS_MICROSTEP_LIMIT,
        }
    }

    fn plan_next(
        &self,
        core: &BattleCore,
        now_ms: u64,
    ) -> Result<Option<BossRuntimeTransitionPlan>, GameError> {
        self.validate_activation_identities()?;
        let mut next = crate::authoring::clone_shape::controlled_clone(self)?;
        let mut effects = Vec::new();

        if !next.initialized {
            next.initialized = true;
            next.enter_phase(core, next.current_phase_index, now_ms, &mut effects, false)?;
            return Ok(Some(BossRuntimeTransitionPlan {
                next_runtime: next,
                effects,
            }));
        }

        if next.reconcile_one_actor(core, now_ms, &mut effects)? {
            return Ok(Some(BossRuntimeTransitionPlan {
                next_runtime: next,
                effects,
            }));
        }

        if next
            .pending_phase_transition
            .as_ref()
            .is_some_and(|pending| {
                pending
                    .actor_boundary_reached
                    .iter()
                    .all(|reached| *reached)
            })
        {
            let Some(target_phase_index) = next
                .pending_phase_transition
                .as_ref()
                .map(|pending| pending.target_phase_index)
            else {
                return Err(GameError::InvalidAction);
            };
            next.enter_phase(core, target_phase_index, now_ms, &mut effects, true)?;
            return Ok(Some(BossRuntimeTransitionPlan {
                next_runtime: next,
                effects,
            }));
        }

        if next.pending_phase_transition.is_none() {
            if let Some(transition_index) = next.select_phase_transition(core) {
                let transition =
                    &next.definition.phases[next.current_phase_index].transitions[transition_index];
                let (timing, target_phase_index) =
                    (transition.timing, transition.target_phase_index);
                match timing {
                    BossPhaseTransitionTimingData::Immediate => {
                        next.enter_phase(core, target_phase_index, now_ms, &mut effects, true)?;
                    }
                    timing => {
                        next.pending_phase_transition = Some(PendingBossPhaseTransition {
                            target_phase_index,
                            timing,
                            actor_boundary_reached: next
                                .actors
                                .iter()
                                .take(next.encounter_actor_count)
                                .map(|actor| match timing {
                                    BossPhaseTransitionTimingData::AfterCurrentState => {
                                        actor.active_state().is_none()
                                    }
                                    BossPhaseTransitionTimingData::AfterCurrentPattern => actor
                                        .control
                                        .as_ref()
                                        .and_then(|control| control.pattern.as_ref())
                                        .is_none(),
                                    BossPhaseTransitionTimingData::Immediate => true,
                                })
                                .collect(),
                        });
                    }
                }
                return Ok(Some(BossRuntimeTransitionPlan {
                    next_runtime: next,
                    effects,
                }));
            }
        }

        for actor_index in 0..next.actors.len() {
            if next.poll_actor_once(core, actor_index, now_ms)? {
                return Ok(Some(BossRuntimeTransitionPlan {
                    next_runtime: next,
                    effects,
                }));
            }
            if next.advance_actor_once(core, actor_index, now_ms, &mut effects)? {
                return Ok(Some(BossRuntimeTransitionPlan {
                    next_runtime: next,
                    effects,
                }));
            }
        }

        Ok(None)
    }

    fn enter_phase(
        &mut self,
        core: &BattleCore,
        phase_index: usize,
        now_ms: u64,
        effects: &mut Vec<BossRuntimeEffect>,
        cancel_previous: bool,
    ) -> Result<(), GameError> {
        if phase_index >= self.definition.phases.len() {
            return Err(GameError::InvalidAction);
        }
        if cancel_previous {
            for actor in self.actors.iter_mut().take(self.encounter_actor_count) {
                actor.cancel_all_owned_work(effects);
                actor.behavior = None;
                actor.behavior_activation_id = None;
                actor.control = None;
                actor.fixed_sequence_cursors.clear();
                actor.waiting_for_phase_transition = false;
            }
        }
        self.current_phase_index = phase_index;
        self.phase_activation_id = Some(allocate_activation_id(&mut self.next_activation_id)?);
        self.pending_phase_transition = None;

        // Keep the immutable owner alive while actor state changes.
        let definition = Arc::clone(&self.definition);
        let phase = &definition.phases[phase_index];
        for actor_index in 0..self.encounter_actor_count {
            let slot_index = self.actors[actor_index]
                .encounter_slot_index
                .ok_or(GameError::InvalidAction)?;
            let behavior = phase
                .actor_behaviors
                .iter()
                .find(|binding| binding.actor_slot_index == slot_index)
                .map(|binding| binding.behavior.clone())
                .ok_or(GameError::InvalidAction)?;
            let unit_id = core
                .scenario_runtime
                .spawned
                .unit_refs
                .get(
                    self.actors[actor_index]
                        .unit_ref
                        .as_ref()
                        .ok_or(GameError::InvalidAction)?,
                )
                .copied();
            self.actors[actor_index].unit_id = unit_id;
            self.actors[actor_index].ever_bound |= unit_id.is_some();
            self.actors[actor_index].behavior = Some(behavior.clone());
            self.actors[actor_index].behavior_activation_id = None;
            self.actors[actor_index].control = None;
            if unit_id.is_some_and(|id| core.units.get(&id).is_some_and(|unit| unit.is_active())) {
                self.activate_actor_behavior(core, actor_index, behavior, now_ms, effects)?;
            }
        }
        Ok(())
    }

    fn activate_actor_behavior(
        &mut self,
        core: &BattleCore,
        actor_index: usize,
        behavior: Arc<CompiledBossActorBehaviorDefinition>,
        now_ms: u64,
        effects: &mut Vec<BossRuntimeEffect>,
    ) -> Result<(), GameError> {
        let behavior_activation_id = allocate_activation_id(&mut self.next_activation_id)?;
        self.actors[actor_index].behavior_activation_id = Some(behavior_activation_id);
        self.actors[actor_index].waiting_for_phase_transition = false;
        let entry_node_index = behavior.entry_node_index;
        self.enter_control_node(core, actor_index, entry_node_index, now_ms, effects)
    }

    fn enter_control_node(
        &mut self,
        core: &BattleCore,
        actor_index: usize,
        node_index: usize,
        now_ms: u64,
        effects: &mut Vec<BossRuntimeEffect>,
    ) -> Result<(), GameError> {
        let behavior = self.actors[actor_index]
            .behavior
            .as_ref()
            .cloned()
            .ok_or(GameError::InvalidAction)?;
        let node = behavior
            .nodes
            .get(node_index)
            .ok_or(GameError::InvalidAction)?;
        let control_activation_id = allocate_activation_id(&mut self.next_activation_id)?;
        let resident_state = match node {
            CompiledBossControlNode::State { tasks, .. } => {
                let state_activation_id = allocate_activation_id(&mut self.next_activation_id)?;
                Some(self.admit_state_tasks(
                    core,
                    actor_index,
                    tasks,
                    None,
                    BossTargetLockOwner::ResidentState(state_activation_id),
                    state_activation_id,
                    now_ms,
                    effects,
                )?)
            }
            CompiledBossControlNode::PatternCall { .. } => None,
        };
        self.actors[actor_index].control = Some(ActiveBossControlNode {
            node_index,
            activation_id: control_activation_id,
            resident_state,
            pattern: None,
        });
        Ok(())
    }

    fn admit_state_tasks(
        &mut self,
        core: &BattleCore,
        actor_index: usize,
        tasks: &[CompiledBossTask],
        frame: Option<&ResolvedBossArgumentFrame>,
        lock_owner: BossTargetLockOwner,
        state_activation_id: u64,
        now_ms: u64,
        effects: &mut Vec<BossRuntimeEffect>,
    ) -> Result<ActiveBossState, GameError> {
        let mut active_tasks = Vec::with_capacity(tasks.len());
        for task in tasks {
            let activation_id = allocate_activation_id(&mut self.next_activation_id)?;
            let (id, kind, result, wake_at) = match task {
                CompiledBossTask::StartAction {
                    id,
                    skill_id,
                    target,
                    facing,
                    milestone,
                    timeout_ms,
                    on_timeout,
                } => {
                    let unit_id = self.actors[actor_index]
                        .unit_id
                        .ok_or(GameError::InvalidAction)?;
                    let target = match target {
                        BossActionTargetData::SelfUnit => Some(SkillCastTarget::Unit {
                            unit_instance_id: unit_id,
                        }),
                        BossActionTargetData::LockedUnit => self.actors[actor_index]
                            .target_lock
                            .as_ref()
                            .map(|lock| SkillCastTarget::Unit {
                                unit_instance_id: lock.target_id,
                            }),
                        BossActionTargetData::Tile { position } => Some(SkillCastTarget::Tile {
                            position: *position,
                        }),
                    };
                    let deadline_ms = now_ms
                        .checked_add(*timeout_ms)
                        .ok_or(GameError::InvalidAction)?;
                    effects.push(BossRuntimeEffect::StartAction {
                        unit_id,
                        actor_index,
                        task_activation_id: activation_id,
                        skill_id: skill_id.clone(),
                        target,
                        facing: *facing,
                    });
                    (
                        id.clone(),
                        ActiveBossTaskKind::StartAction {
                            action: None,
                            deadline_ms,
                            milestone: *milestone,
                            on_timeout: *on_timeout,
                        },
                        None,
                        Some(deadline_ms),
                    )
                }
                CompiledBossTask::RouteCombat { id } => {
                    if let Some(unit_id) = self.actors[actor_index].unit_id {
                        effects.push(BossRuntimeEffect::ResumeRouteCombat {
                            unit_id,
                            time_ms: now_ms,
                        });
                    }
                    (id.clone(), ActiveBossTaskKind::RouteCombat, None, None)
                }
                CompiledBossTask::AcquireTarget { id, query } => {
                    let query = resolve_target_query(query, frame)?;
                    let current_lock = self.actors[actor_index].target_lock.as_ref();
                    let preserve = current_lock.is_some_and(|lock| {
                        lock.policy == BossTargetLockPolicyData::LockUntilInvalid
                            && target_identity_is_valid(
                                core,
                                actor_index,
                                &self.actors,
                                lock.target_id,
                            )
                    });
                    let selected = if preserve {
                        current_lock.map(|lock| lock.target_id)
                    } else {
                        select_target(core, self.actors[actor_index].unit_id, query)
                    };
                    if let Some(target_id) = selected {
                        self.actors[actor_index].target_lock = Some(BossTargetLock {
                            target_id,
                            owner: lock_owner,
                            policy: query.lock_policy,
                        });
                    } else {
                        self.actors[actor_index].target_lock = None;
                    }
                    (
                        id.clone(),
                        ActiveBossTaskKind::AcquireTarget,
                        Some(if selected.is_some() {
                            BossTaskResultData::Acquired
                        } else {
                            BossTaskResultData::NoEligibleTarget
                        }),
                        None,
                    )
                }
                CompiledBossTask::MoveToTarget { id, timeout_ms } => {
                    let duration_ms = resolve_duration(timeout_ms, frame)?;
                    let deadline_ms = now_ms
                        .checked_add(duration_ms)
                        .ok_or(GameError::InvalidAction)?;
                    (
                        id.clone(),
                        ActiveBossTaskKind::MoveToTarget { deadline_ms },
                        None,
                        Some(deadline_ms),
                    )
                }
                CompiledBossTask::RejoinForwardRoute { id, timeout_ms } => {
                    let duration_ms = resolve_duration(timeout_ms, frame)?;
                    let deadline_ms = now_ms
                        .checked_add(duration_ms)
                        .ok_or(GameError::InvalidAction)?;
                    let target_route_index = self.select_rejoin_route_index(core, actor_index);
                    (
                        id.clone(),
                        ActiveBossTaskKind::RejoinForwardRoute {
                            deadline_ms,
                            target_route_index,
                        },
                        None,
                        Some(deadline_ms),
                    )
                }
                CompiledBossTask::Wait { id, duration_ms } => {
                    let duration_ms = resolve_duration(duration_ms, frame)?;
                    let completes_at_ms = now_ms
                        .checked_add(duration_ms)
                        .ok_or(GameError::InvalidAction)?;
                    (
                        id.clone(),
                        ActiveBossTaskKind::Wait { completes_at_ms },
                        None,
                        Some(completes_at_ms),
                    )
                }
                CompiledBossTask::Summon {
                    id,
                    definition_id,
                    count,
                    batch_handle_id,
                    placement,
                    occupancy_policy,
                    on_insufficient_space,
                    route_id,
                    lifetime,
                    removal_signal,
                } => {
                    let summoner = self.actors[actor_index]
                        .unit_id
                        .ok_or(GameError::InvalidAction)?;
                    let (insufficient_space, wake_at) = match on_insufficient_space {
                        CompiledBossSummonInsufficientSpace::Fail => {
                            (ActiveBossSummonInsufficientSpace::Fail, None)
                        }
                        CompiledBossSummonInsufficientSpace::WaitUntilAvailable { timeout_ms } => {
                            let deadline_ms = now_ms
                                .checked_add(resolve_duration(timeout_ms, frame)?)
                                .ok_or(GameError::InvalidAction)?;
                            (
                                ActiveBossSummonInsufficientSpace::WaitUntilAvailable {
                                    deadline_ms,
                                },
                                Some(deadline_ms),
                            )
                        }
                    };
                    let lifetime = lifetime
                        .as_ref()
                        .map(|lifetime| {
                            Ok(ActiveBossSummonLifetime {
                                duration_ms: resolve_duration(&lifetime.lifetime_ms, frame)?,
                                mode: lifetime.mode,
                            })
                        })
                        .transpose()?;
                    let request_id = format!(
                        "summon:{summoner}:{state_activation_id}:{}:{activation_id}",
                        id.as_str()
                    );
                    (
                        id.clone(),
                        ActiveBossTaskKind::Summon(ActiveBossSummonTask {
                            request_id,
                            definition_id: definition_id.clone(),
                            count: *count,
                            batch_handle_id: batch_handle_id.clone(),
                            placement: placement.clone(),
                            occupancy_policy: occupancy_policy.clone(),
                            insufficient_space,
                            route_id: route_id.clone(),
                            lifetime,
                            removal_signal: removal_signal.clone(),
                            admitted_at_ms: now_ms,
                        }),
                        None,
                        wake_at,
                    )
                }
                CompiledBossTask::EmitSignal { id, signal_id } => {
                    effects.push(BossRuntimeEffect::EmitSignal {
                        signal_id: signal_id.clone(),
                    });
                    (
                        id.clone(),
                        ActiveBossTaskKind::EmitSignal,
                        Some(BossTaskResultData::Completed),
                        None,
                    )
                }
            };
            if let Some(time_ms) = wake_at {
                effects.push(BossRuntimeEffect::EnqueueWake {
                    time_ms,
                    actor_slot_index: self.actors[actor_index].actor_slot_index,
                    task_activation_id: activation_id,
                });
            }
            active_tasks.push(ActiveBossTask {
                id,
                activation_id,
                kind,
                result,
                result_consumed: false,
            });
        }
        Ok(ActiveBossState {
            activation_id: state_activation_id,
            tasks: active_tasks,
            target_invalidated_pulse: false,
            target_invalidated_latch: false,
        })
    }

    fn reconcile_one_actor(
        &mut self,
        core: &BattleCore,
        now_ms: u64,
        effects: &mut Vec<BossRuntimeEffect>,
    ) -> Result<bool, GameError> {
        for actor_index in 0..self.actors.len() {
            let resolved = self.actors[actor_index]
                .unit_ref
                .as_ref()
                .and_then(|unit_ref| core.scenario_runtime.spawned.unit_refs.get(unit_ref))
                .copied();
            if self.actors[actor_index].unit_id.is_none() && resolved.is_some() {
                self.actors[actor_index].unit_id = resolved;
                self.actors[actor_index].ever_bound = true;
                if let Some(behavior) = self.actors[actor_index].behavior.clone() {
                    self.activate_actor_behavior(core, actor_index, behavior, now_ms, effects)?;
                }
                return Ok(true);
            }
            let Some(unit_id) = self.actors[actor_index].unit_id else {
                continue;
            };
            if self.actors[actor_index].control.is_none()
                && self.actors[actor_index].behavior_activation_id.is_none()
                && !self.actors[actor_index].waiting_for_phase_transition
                && core
                    .units
                    .get(&unit_id)
                    .is_some_and(|unit| unit.is_active())
            {
                let behavior = self.actors[actor_index]
                    .behavior
                    .clone()
                    .ok_or(GameError::InvalidAction)?;
                self.activate_actor_behavior(core, actor_index, behavior, now_ms, effects)?;
                return Ok(true);
            }
            if self.actors[actor_index].control.is_some()
                && core
                    .units
                    .get(&unit_id)
                    .is_none_or(|unit| !unit.is_active())
            {
                self.actors[actor_index].cancel_all_owned_work(effects);
                self.actors[actor_index].control = None;
                self.actors[actor_index].target_lock = None;
                self.actors[actor_index].waiting_for_phase_transition =
                    self.actors[actor_index].encounter_slot_index.is_some();
                if actor_index < self.encounter_actor_count {
                    if let Some(pending) = &mut self.pending_phase_transition {
                        pending.actor_boundary_reached[actor_index] = true;
                    }
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn select_phase_transition(&self, core: &BattleCore) -> Option<usize> {
        self.definition.phases[self.current_phase_index]
            .transitions
            .iter()
            .enumerate()
            .filter(|(_, transition)| self.encounter_condition_is_true(core, &transition.condition))
            .max_by_key(|(_, transition)| transition.priority)
            .map(|(index, _)| index)
    }

    fn encounter_condition_is_true(
        &self,
        core: &BattleCore,
        condition: &BossEncounterConditionData,
    ) -> bool {
        match condition {
            BossEncounterConditionData::Always => true,
            BossEncounterConditionData::ActorHealthAtOrBelowPercent {
                actor_slot_id,
                percent,
            } => self
                .definition
                .actor_slots
                .iter()
                .position(|slot| slot == actor_slot_id)
                .and_then(|slot_index| {
                    self.actors
                        .iter()
                        .find(|actor| actor.encounter_slot_index == Some(slot_index))
                })
                .and_then(|actor| actor.unit_id)
                .and_then(|unit_id| core.units.get(&unit_id))
                .is_some_and(|unit| {
                    u64::from(unit.stats.current_health) * 100
                        <= u64::from(unit.stats.max_health) * u64::from(*percent)
                }),
            BossEncounterConditionData::ActorDefeated { actor_slot_id } => self
                .definition
                .actor_slots
                .iter()
                .position(|slot| slot == actor_slot_id)
                .and_then(|slot_index| {
                    self.actors
                        .iter()
                        .find(|actor| actor.encounter_slot_index == Some(slot_index))
                })
                .is_some_and(|actor| {
                    actor.ever_bound
                        && actor.unit_id.is_some_and(|unit_id| {
                            core.units
                                .get(&unit_id)
                                .is_none_or(|unit| !unit.is_active())
                        })
                }),
        }
    }

    fn poll_actor_once(
        &mut self,
        core: &BattleCore,
        actor_index: usize,
        now_ms: u64,
    ) -> Result<bool, GameError> {
        if self.actors[actor_index].control.is_none()
            || self.actors[actor_index].waiting_for_phase_transition
        {
            return Ok(false);
        }

        if let Some(lock) = self.actors[actor_index].target_lock.clone() {
            if !target_identity_is_valid(core, actor_index, &self.actors, lock.target_id) {
                self.actors[actor_index].target_lock = None;
                if let Some(state) = self.actors[actor_index].active_state_mut() {
                    state.target_invalidated_pulse = true;
                    state.target_invalidated_latch = true;
                    for task in &mut state.tasks {
                        if matches!(task.kind, ActiveBossTaskKind::MoveToTarget { .. })
                            && task.result.is_none()
                        {
                            task.result = Some(BossTaskResultData::TargetInvalidated);
                        }
                    }
                }
                return Ok(true);
            }
        }

        let unit_id = self.actors[actor_index].unit_id;
        let target_id = self.actors[actor_index]
            .target_lock
            .as_ref()
            .map(|lock| lock.target_id);
        let route_cells = self.route_cells_for_actor(core, actor_index);
        let actor_route_cursor = self.actors[actor_index].route_cursor_index;
        let mut route_cursor_update = None;
        let mut changed = false;
        {
            let Some(state) = self.actors[actor_index].active_state_mut() else {
                return Ok(false);
            };
            for task in &mut state.tasks {
                if task.result.is_some() {
                    continue;
                }
                let result = match &task.kind {
                    ActiveBossTaskKind::StartAction {
                        action: Some(action),
                        deadline_ms,
                        milestone,
                        ..
                    } => {
                        let snapshot = core
                            .query_action(action)
                            .map_err(|_| GameError::InvalidAction)?;
                        let target_lost = snapshot.released_at_ms.is_none()
                            && unit_id
                                .and_then(|id| core.units.get(&id))
                                .and_then(|unit| unit.pending_skill_cast.as_ref())
                                .and_then(|pending| pending.cast_target)
                                .is_some_and(|target| match target {
                                    SkillCastTarget::Unit { unit_instance_id } => core
                                        .units
                                        .get(&unit_instance_id)
                                        .is_none_or(|unit| !unit.is_active()),
                                    SkillCastTarget::Tile { .. } => false,
                                });
                        if target_lost {
                            Some(BossTaskResultData::TargetInvalidated)
                        } else if matches!(
                            snapshot.terminal.map(|t| t.kind),
                            Some(ActionTerminalKind::Cancelled(_))
                        ) {
                            Some(BossTaskResultData::Canceled)
                        } else if matches!(
                            snapshot.terminal.map(|t| t.kind),
                            Some(ActionTerminalKind::Failed(_))
                        ) {
                            Some(BossTaskResultData::RequestRejected)
                        } else if match milestone {
                            BossActionMilestoneData::Released => snapshot.released_at_ms.is_some(),
                            BossActionMilestoneData::EffectsSettled => {
                                snapshot.effects_settled_at_ms.is_some()
                            }
                        } {
                            Some(BossTaskResultData::Completed)
                        } else if now_ms >= *deadline_ms {
                            Some(BossTaskResultData::TimedOut)
                        } else {
                            None
                        }
                    }
                    ActiveBossTaskKind::StartAction { action: None, .. } => None,
                    ActiveBossTaskKind::RouteCombat => {
                        route_cursor_update = unit_id
                            .and_then(|id| core.units.get(&id))
                            .and_then(|unit| nearest_route_index(&route_cells, unit.body.position))
                            .map(|index| index.max(actor_route_cursor))
                            .filter(|index| *index > actor_route_cursor);
                        if route_cursor_update.is_some() {
                            changed = true;
                            break;
                        }
                        None
                    }
                    ActiveBossTaskKind::AcquireTarget => None,
                    ActiveBossTaskKind::Wait { completes_at_ms } => {
                        (now_ms >= *completes_at_ms).then_some(BossTaskResultData::Completed)
                    }
                    ActiveBossTaskKind::MoveToTarget { deadline_ms } => {
                        match (unit_id, target_id) {
                            (Some(unit_id), Some(target_id)) => {
                                if move_to_target_reached(core, unit_id, target_id) {
                                    Some(BossTaskResultData::Reached)
                                } else if path_to_target_approach(core, unit_id, target_id)
                                    .is_none()
                                {
                                    Some(BossTaskResultData::PathUnavailable)
                                } else if now_ms >= *deadline_ms {
                                    Some(BossTaskResultData::TimedOut)
                                } else {
                                    None
                                }
                            }
                            _ => Some(BossTaskResultData::TargetInvalidated),
                        }
                    }
                    ActiveBossTaskKind::RejoinForwardRoute {
                        deadline_ms,
                        target_route_index,
                    } => match (unit_id, target_route_index) {
                        (Some(unit_id), Some(route_index)) => {
                            let target = route_cells.get(*route_index).copied();
                            if target.is_some_and(|position| {
                                route_point_reached(core, unit_id, position)
                            }) {
                                route_cursor_update = Some(actor_route_cursor.max(*route_index));
                                Some(BossTaskResultData::Reached)
                            } else if target.is_none_or(|position| {
                                grid_path(
                                    core,
                                    core.unit_projected_tile(unit_id),
                                    &[position],
                                    core.units
                                        .get(&unit_id)
                                        .is_some_and(|unit| unit.is_airborne()),
                                )
                                .is_none()
                            }) {
                                Some(BossTaskResultData::PathUnavailable)
                            } else if now_ms >= *deadline_ms {
                                Some(BossTaskResultData::TimedOut)
                            } else {
                                None
                            }
                        }
                        _ => Some(BossTaskResultData::PathUnavailable),
                    },
                    ActiveBossTaskKind::Summon(_) | ActiveBossTaskKind::EmitSignal => None,
                };
                if let Some(result) = result {
                    task.result = Some(result);
                    changed = true;
                    break;
                }
            }
        }
        if let Some(cursor) = route_cursor_update {
            self.actors[actor_index].route_cursor_index = cursor;
        }
        Ok(changed)
    }

    fn advance_actor_once(
        &mut self,
        core: &BattleCore,
        actor_index: usize,
        now_ms: u64,
        effects: &mut Vec<BossRuntimeEffect>,
    ) -> Result<bool, GameError> {
        if self.actors[actor_index].waiting_for_phase_transition {
            return Ok(false);
        }
        let Some(control) = self.actors[actor_index].control.as_ref() else {
            return Ok(false);
        };
        let behavior = self.actors[actor_index]
            .behavior
            .as_ref()
            .cloned()
            .ok_or(GameError::InvalidAction)?;
        let node = behavior
            .nodes
            .get(control.node_index)
            .ok_or(GameError::InvalidAction)?;

        if control.pattern.is_some() {
            return self.advance_pattern_once(core, actor_index, now_ms, effects);
        }

        match node {
            CompiledBossControlNode::State { transitions, .. } => {
                let Some(state) = self.actors[actor_index].active_state() else {
                    return Err(GameError::InvalidAction);
                };
                let winner = transitions
                    .iter()
                    .enumerate()
                    .filter(|(_, transition)| {
                        condition_is_true(
                            core,
                            &self.actors[actor_index],
                            state,
                            &transition.condition,
                        )
                    })
                    .max_by_key(|(_, transition)| transition.priority)
                    .map(|(index, _)| index);
                let Some(winner) = winner else {
                    return Ok(false);
                };
                let target_node_index = transitions[winner].target_node_index;
                if self.intercept_pending_boundary(actor_index, false, effects) {
                    return Ok(true);
                }
                self.actors[actor_index].cancel_current_state(effects, false);
                self.enter_control_node(core, actor_index, target_node_index, now_ms, effects)?;
                Ok(true)
            }
            CompiledBossControlNode::PatternCall {
                candidates,
                repeat,
                no_eligible_node_index,
                ..
            } => {
                let node_index = control.node_index;
                let cursor = self.actors[actor_index]
                    .fixed_sequence_cursors
                    .get(&node_index)
                    .copied()
                    .unwrap_or(0);
                let candidate_index = if candidates.is_empty() {
                    None
                } else if cursor < candidates.len() {
                    Some(cursor)
                } else if *repeat {
                    Some(cursor % candidates.len())
                } else {
                    None
                };
                let eligible = candidate_index.filter(|index| {
                    candidate_cooldown_ready(&self.actors[actor_index], &candidates[*index], now_ms)
                });
                let Some(candidate_index) = eligible else {
                    self.actors[actor_index].cancel_current_state(effects, false);
                    self.enter_control_node(
                        core,
                        actor_index,
                        *no_eligible_node_index,
                        now_ms,
                        effects,
                    )?;
                    return Ok(true);
                };
                self.admit_pattern(
                    core,
                    actor_index,
                    node_index,
                    candidate_index,
                    &candidates[candidate_index],
                    now_ms,
                    effects,
                )?;
                Ok(true)
            }
        }
    }

    fn admit_pattern(
        &mut self,
        core: &BattleCore,
        actor_index: usize,
        node_index: usize,
        candidate_index: usize,
        candidate: &CompiledBossPatternCandidate,
        now_ms: u64,
        effects: &mut Vec<BossRuntimeEffect>,
    ) -> Result<(), GameError> {
        if self.actors[actor_index]
            .control
            .as_ref()
            .and_then(|control| control.pattern.as_ref())
            .is_some()
        {
            return Err(GameError::InvalidAction);
        }
        let pattern_activation_id = allocate_activation_id(&mut self.next_activation_id)?;
        let state_activation_id = allocate_activation_id(&mut self.next_activation_id)?;
        let entry_state_index = candidate.pattern.entry_state_index;
        let entry_state = candidate
            .pattern
            .states
            .get(entry_state_index)
            .ok_or(GameError::InvalidAction)?;
        let state = self.admit_state_tasks(
            core,
            actor_index,
            &entry_state.tasks,
            Some(&candidate.argument_frame),
            BossTargetLockOwner::Pattern(pattern_activation_id),
            state_activation_id,
            now_ms,
            effects,
        )?;
        if let Some(cooldown) = &candidate.cooldown {
            let ready_at_ms = now_ms
                .checked_add(cooldown.duration_ms)
                .ok_or(GameError::InvalidAction)?;
            self.actors[actor_index]
                .cooldown_ready_at_ms
                .insert(cooldown.group_id.clone(), ready_at_ms);
        }
        self.actors[actor_index]
            .fixed_sequence_cursors
            .insert(node_index, candidate_index.saturating_add(1));
        let control = self.actors[actor_index]
            .control
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        control.pattern = Some(ActiveBossPattern {
            definition: candidate.pattern.clone(),
            argument_frame: candidate.argument_frame.clone(),
            activation_id: pattern_activation_id,
            state_index: entry_state_index,
            state,
            loop_counters: OwnedHashMap::new(),
            batch_handles: OwnedHashMap::new(),
            cleared_batch_handles: HashSet::new(),
        });
        Ok(())
    }

    fn advance_pattern_once(
        &mut self,
        core: &BattleCore,
        actor_index: usize,
        now_ms: u64,
        effects: &mut Vec<BossRuntimeEffect>,
    ) -> Result<bool, GameError> {
        let pattern = self.actors[actor_index]
            .control
            .as_ref()
            .and_then(|control| control.pattern.as_ref())
            .ok_or(GameError::InvalidAction)?;
        let definition = Arc::clone(&pattern.definition);
        let state_def = definition
            .states
            .get(pattern.state_index)
            .ok_or(GameError::InvalidAction)?;
        let winner = state_def
            .transitions
            .iter()
            .enumerate()
            .filter(|(_, transition)| {
                condition_is_true(
                    core,
                    &self.actors[actor_index],
                    &pattern.state,
                    &transition.condition,
                )
            })
            .max_by_key(|(_, transition)| transition.priority)
            .map(|(index, _)| index);
        let Some(winner) = winner else {
            return Ok(false);
        };
        let transition = &state_def.transitions[winner];
        let target = self.resolve_loop_target(actor_index, transition)?;

        if matches!(target, CompiledBossPatternTransitionTarget::ContinueState) {
            consume_condition_source(
                self.actors[actor_index]
                    .active_state_mut()
                    .ok_or(GameError::InvalidAction)?,
                &transition.condition,
            );
            return Ok(true);
        }

        let is_pattern_outcome = matches!(target, CompiledBossPatternTransitionTarget::Outcome(_));
        if self.intercept_pending_boundary(actor_index, is_pattern_outcome, effects) {
            return Ok(true);
        }

        match target {
            CompiledBossPatternTransitionTarget::State(next_state_index) => {
                self.actors[actor_index].cancel_current_state(effects, false);
                let pattern = self.actors[actor_index]
                    .control
                    .as_ref()
                    .and_then(|control| control.pattern.as_ref())
                    .ok_or(GameError::InvalidAction)?;
                let argument_frame = Arc::clone(&pattern.argument_frame);
                let pattern_activation_id = pattern.activation_id;
                let next_state = definition
                    .states
                    .get(next_state_index)
                    .ok_or(GameError::InvalidAction)?;
                let state_activation_id = allocate_activation_id(&mut self.next_activation_id)?;
                let state = self.admit_state_tasks(
                    core,
                    actor_index,
                    &next_state.tasks,
                    Some(&argument_frame),
                    BossTargetLockOwner::Pattern(pattern_activation_id),
                    state_activation_id,
                    now_ms,
                    effects,
                )?;
                let active_pattern = self.actors[actor_index]
                    .control
                    .as_mut()
                    .and_then(|control| control.pattern.as_mut())
                    .ok_or(GameError::InvalidAction)?;
                active_pattern.state_index = next_state_index;
                active_pattern.state = state;
            }
            CompiledBossPatternTransitionTarget::Outcome(outcome_index) => {
                let pattern_id = &definition.id;
                self.actors[actor_index].cancel_current_state(effects, true);
                let behavior = self.actors[actor_index]
                    .behavior
                    .as_ref()
                    .cloned()
                    .ok_or(GameError::InvalidAction)?;
                let control_node_index = self.actors[actor_index]
                    .control
                    .as_ref()
                    .ok_or(GameError::InvalidAction)?
                    .node_index;
                let target_node_index = match behavior.nodes.get(control_node_index) {
                    Some(CompiledBossControlNode::PatternCall { outcome_routes, .. }) => {
                        outcome_routes
                            .iter()
                            .find(|route| {
                                route.pattern_id == *pattern_id
                                    && route.outcome_index == outcome_index
                            })
                            .map(|route| route.target_node_index)
                            .ok_or(GameError::InvalidAction)?
                    }
                    _ => return Err(GameError::InvalidAction),
                };
                self.enter_control_node(core, actor_index, target_node_index, now_ms, effects)?;
            }
            CompiledBossPatternTransitionTarget::ContinueState => {
                return Err(GameError::InvalidAction);
            }
        }
        Ok(true)
    }

    fn resolve_loop_target(
        &mut self,
        actor_index: usize,
        transition: &CompiledBossPatternTransition,
    ) -> Result<CompiledBossPatternTransitionTarget, GameError> {
        let Some(guard) = &transition.loop_guard else {
            return Ok(transition.target.clone());
        };
        let pattern = self.actors[actor_index]
            .control
            .as_mut()
            .and_then(|control| control.pattern.as_mut())
            .ok_or(GameError::InvalidAction)?;
        let count = pattern
            .loop_counters
            .entry(guard.id.as_str().to_string())
            .or_insert(0);
        if *count >= guard.max_iterations {
            Ok(guard.exhausted_target.clone())
        } else {
            *count = count.checked_add(1).ok_or(GameError::InvalidAction)?;
            Ok(transition.target.clone())
        }
    }

    fn intercept_pending_boundary(
        &mut self,
        actor_index: usize,
        pattern_will_end: bool,
        effects: &mut Vec<BossRuntimeEffect>,
    ) -> bool {
        if actor_index >= self.encounter_actor_count {
            return false;
        }
        let Some(pending) = self.pending_phase_transition.as_mut() else {
            return false;
        };
        let reaches_boundary = match pending.timing {
            BossPhaseTransitionTimingData::AfterCurrentState => true,
            BossPhaseTransitionTimingData::AfterCurrentPattern => pattern_will_end,
            BossPhaseTransitionTimingData::Immediate => true,
        };
        if !reaches_boundary {
            return false;
        }
        pending.actor_boundary_reached[actor_index] = true;
        self.actors[actor_index].cancel_all_owned_work(effects);
        self.actors[actor_index].control = None;
        self.actors[actor_index].target_lock = None;
        self.actors[actor_index].waiting_for_phase_transition = true;
        true
    }

    fn select_rejoin_route_index(&self, core: &BattleCore, actor_index: usize) -> Option<usize> {
        let unit_id = self.actors[actor_index].unit_id?;
        let unit = core.units.get(&unit_id)?;
        let cells = self.route_cells_for_actor(core, actor_index);
        if cells.is_empty() {
            return None;
        }
        let nearest = nearest_route_index(cells, unit.body.position).unwrap_or(0);
        let minimum = nearest.max(self.actors[actor_index].route_cursor_index);
        cells
            .iter()
            .enumerate()
            .skip(minimum)
            .filter_map(|(index, position)| {
                grid_path(
                    core,
                    Some(unit.body.projected_tile()),
                    &[*position],
                    unit.is_airborne(),
                )
                .map(|path| (path.len().saturating_sub(1), position.y, position.x, index))
            })
            .min()
            .map(|(_, _, _, index)| index)
    }

    fn route_cells_for_actor<'core>(
        &self,
        core: &'core BattleCore,
        actor_index: usize,
    ) -> &'core [Position] {
        let plan = self.actors[actor_index]
            .unit_id
            .and_then(|unit_id| core.units.get(&unit_id))
            .and_then(|unit| unit.enemy_movement_plan.as_ref())
            .unwrap_or(&core.scenario.tactical_plan.enemy_plan);
        match plan {
            EnemyMovementPlan::PathAlongCells { cells } => cells,
        }
    }

    fn clear_boundary_pulses(&mut self) {
        for actor in &mut self.actors {
            if let Some(state) = actor.active_state_mut() {
                state.target_invalidated_pulse = false;
            }
        }
    }

    fn validate_activation_identities(&self) -> Result<(), GameError> {
        for actor in &self.actors {
            if let Some(control) = &actor.control {
                if control.activation_id == 0 {
                    return Err(GameError::InvalidAction);
                }
                if let Some(state) = actor.active_state() {
                    if state.activation_id == 0
                        || state.tasks.iter().any(|task| task.activation_id == 0)
                    {
                        return Err(GameError::InvalidAction);
                    }
                }
            }
        }
        Ok(())
    }

    fn wake_is_current(&self, actor_slot_index: usize, task_activation_id: u64) -> bool {
        self.actors
            .iter()
            .find(|actor| actor.actor_slot_index == actor_slot_index)
            .and_then(BossActorBehaviorRuntime::active_state)
            .is_some_and(|state| {
                state.tasks.iter().any(|task| {
                    task.activation_id == task_activation_id
                        && task.result.is_none()
                        && matches!(
                            task.kind,
                            ActiveBossTaskKind::StartAction { .. }
                                | ActiveBossTaskKind::MoveToTarget { .. }
                                | ActiveBossTaskKind::RejoinForwardRoute { .. }
                                | ActiveBossTaskKind::Wait { .. }
                                | ActiveBossTaskKind::Summon(
                                    ActiveBossSummonTask {
                                        insufficient_space:
                                            ActiveBossSummonInsufficientSpace::WaitUntilAvailable { .. },
                                        ..
                                    }
                                )
                        )
                })
            })
    }

    fn visit_summon_view_sources<'a>(
        &'a self,
        scan_units: u64,
        mut visit: impl FnMut(BossSummonTaskViewProjection<'a>) -> Result<(), GameError>,
    ) -> Result<(), GameError> {
        for actor in &self.actors {
            // Preflight charges both passes; zero on materialization still polls.
            crate::authoring::control::charge(scan_units, "boss_summon_view_actor_scan")?;
            let (Some(summoner_unit_id), Some(behavior_activation_id), Some(control)) = (
                actor.unit_id.as_ref(),
                actor.behavior_activation_id.as_ref(),
                actor.control.as_ref(),
            ) else {
                continue;
            };
            let Some(pattern) = control.pattern.as_ref() else {
                continue;
            };
            for task in &pattern.state.tasks {
                crate::authoring::control::charge(scan_units, "boss_summon_view_task_scan")?;
                let ActiveBossTaskKind::Summon(summon) = &task.kind else {
                    continue;
                };
                if task.result.is_some() {
                    continue;
                }
                visit(BossSummonTaskViewProjection {
                    actor_runtime_key: &actor.actor_slot_index,
                    summoner_unit_id,
                    behavior_activation_id,
                    pattern_activation_id: &pattern.activation_id,
                    state_activation_id: &pattern.state.activation_id,
                    task_activation_id: &task.activation_id,
                    task: summon,
                })?;
            }
        }
        Ok(())
    }

    pub(super) fn summon_task_views(&self) -> Result<Vec<BossSummonTaskView>, GameError> {
        let controlled = crate::authoring::control::is_active();
        let mut count = 0_usize;
        if controlled {
            crate::authoring::control::poll("boss_summon_view_entry")?;
            let mut meter = crate::authoring::control::CloneMeter::default();
            meter.scan_storage(Some(1))?; // Vec record, including empty output.
            self.visit_summon_view_sources(2, |source| {
                meter.scan_storage(Some(1))?; // Selected Vec element slot.
                source.visit_owned_shape(&mut meter)?;
                // Finite slot admission above bounds this increment.
                count += 1;
                Ok(())
            })?;
            meter.finish("boss_summon_view_storage", "boss_summon_view_copy")?;
        }
        let mut views = if controlled {
            Vec::with_capacity(count)
        } else {
            Vec::new()
        };
        self.visit_summon_view_sources(0, |source| {
            views.push(source.into_owned());
            Ok(())
        })?;
        if controlled {
            // Stable, fallible ordering: charge before each byte comparison and
            // return immediately on interruption, dropping the local partial list.
            for index in 1..views.len() {
                let mut cursor = index;
                while cursor > 0 {
                    let left = &views[cursor - 1].task.request_id;
                    let right = &views[cursor].task.request_id;
                    let bytes = left.len().min(right.len());
                    let work = u64::try_from(bytes)
                        .ok()
                        .and_then(|n| n.checked_add(1))
                        .unwrap_or(u64::MAX);
                    crate::authoring::control::charge(work, "boss_summon_view_order")?;
                    if left <= right {
                        break;
                    }
                    views.swap(cursor - 1, cursor);
                    cursor -= 1;
                }
            }
        } else {
            views.sort_by(|left, right| left.task.request_id.cmp(&right.task.request_id));
        }
        crate::authoring::control::poll("boss_summon_view_complete")?;
        Ok(views)
    }

    pub(super) fn summon_request_is_active(&self, view: &BossSummonTaskView) -> bool {
        // This predicate needs no retained snapshot or ordering. Preserve the
        // exact eligibility and identity predicates of summon_task_views().
        self.actors.iter().any(|actor| {
            if actor.unit_id.is_none()
                || actor.actor_slot_index != view.actor_runtime_key
                || actor.behavior_activation_id != Some(view.behavior_activation_id)
            {
                return false;
            }
            let Some(pattern) = actor
                .control
                .as_ref()
                .and_then(|control| control.pattern.as_ref())
            else {
                return false;
            };
            pattern.activation_id == view.pattern_activation_id
                && pattern.state.activation_id == view.state_activation_id
                && pattern.state.tasks.iter().any(|task| {
                    task.result.is_none()
                        && task.activation_id == view.task_activation_id
                        && matches!(&task.kind, ActiveBossTaskKind::Summon(summon)
                            if summon.request_id == view.task.request_id)
                })
        })
    }

    pub(super) fn settle_summon_task(
        &mut self,
        view: &BossSummonTaskView,
        result: BossTaskResultData,
        batch_uuid: Option<Uuid>,
    ) -> Result<bool, GameError> {
        let Some(actor) = self
            .actors
            .iter_mut()
            .find(|actor| actor.actor_slot_index == view.actor_runtime_key)
        else {
            return Ok(false);
        };
        if actor.behavior_activation_id != Some(view.behavior_activation_id) {
            return Ok(false);
        }
        let Some(pattern) = actor
            .control
            .as_mut()
            .and_then(|control| control.pattern.as_mut())
        else {
            return Ok(false);
        };
        if pattern.activation_id != view.pattern_activation_id
            || pattern.state.activation_id != view.state_activation_id
        {
            return Ok(false);
        }
        let Some(task) = pattern
            .state
            .tasks
            .iter_mut()
            .find(|task| task.activation_id == view.task_activation_id)
        else {
            return Ok(false);
        };
        if task.result.is_some() {
            return Ok(false);
        }
        if result == BossTaskResultData::Spawned {
            let batch_uuid = batch_uuid.ok_or(GameError::InvalidAction)?;
            if pattern
                .batch_handles
                .insert(view.task.batch_handle_id.clone(), batch_uuid)
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }
        } else if batch_uuid.is_some() {
            return Err(GameError::InvalidAction);
        }
        task.result = Some(result);
        Ok(true)
    }

    pub(super) fn note_summon_batch_cleared(
        &mut self,
        pattern_activation_id: u64,
        batch_handle_id: &crate::game::data::boss_runtime_data::SummonBatchHandleId,
        batch_uuid: Uuid,
    ) {
        for actor in &mut self.actors {
            let Some(pattern) = actor
                .control
                .as_mut()
                .and_then(|control| control.pattern.as_mut())
            else {
                continue;
            };
            if pattern.activation_id == pattern_activation_id
                && pattern.batch_handles.get(batch_handle_id) == Some(&batch_uuid)
            {
                pattern
                    .cleared_batch_handles
                    .insert(batch_handle_id.clone());
            }
        }
    }

    pub(super) fn signal_epoch(&self, signal_id: &BossSignalId) -> u64 {
        self.signal_epochs.get(signal_id).copied().unwrap_or(0)
    }

    pub(super) fn is_signal_latched(&self, signal_id: &BossSignalId) -> bool {
        self.latched_signals.contains(signal_id)
    }

    fn note_signal_emitted(&mut self, signal_id: BossSignalId) {
        let epoch = self.signal_epochs.entry(signal_id.clone()).or_insert(0);
        *epoch = epoch.saturating_add(1);
        self.latched_signals.insert(signal_id);
    }

    #[cfg(test)]
    pub(super) fn emit_signal_for_test(&mut self, signal_id: BossSignalId) {
        self.note_signal_emitted(signal_id);
    }

    #[cfg(test)]
    pub(super) fn authority_debug_counts(&self) -> (usize, usize, usize) {
        (
            self.encounter_actor_count,
            self.actors.len(),
            self.current_phase_index,
        )
    }

    #[cfg(test)]
    pub(super) fn actor_unit_ids_for_test(&self) -> Vec<Option<UnitInstanceId>> {
        self.actors.iter().map(|actor| actor.unit_id).collect()
    }

    pub(super) fn summon_spawn_zone(&self, zone_id: &str) -> Option<&ResolvedBossSummonSpawnZone> {
        self.summon_spawn_zones
            .iter()
            .find(|zone| zone.id == zone_id)
    }

    pub(super) fn summon_route(&self, route_id: &str) -> Option<&ResolvedBossSummonRoute> {
        self.summon_routes.iter().find(|route| route.id == route_id)
    }

    pub(super) fn summon_enemy_stat_scale(
        &self,
    ) -> crate::game::battle::types::BattleUnitStatScale {
        self.enemy_stat_scale
    }

    pub(super) fn attach_summoned_actor(
        &mut self,
        unit_id: UnitInstanceId,
        behavior: Arc<CompiledBossActorBehaviorDefinition>,
    ) -> Result<(), GameError> {
        let actor_runtime_key = self.next_actor_runtime_key;
        self.next_actor_runtime_key = self
            .next_actor_runtime_key
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        self.actors.push(BossActorBehaviorRuntime {
            actor_slot_index: actor_runtime_key,
            encounter_slot_index: None,
            unit_ref: None,
            unit_id: Some(unit_id),
            ever_bound: true,
            behavior: Some(behavior),
            behavior_activation_id: None,
            control: None,
            fixed_sequence_cursors: OwnedHashMap::new(),
            cooldown_ready_at_ms: OwnedHashMap::new(),
            target_lock: None,
            route_cursor_index: 0,
            waiting_for_phase_transition: false,
        });
        Ok(())
    }
}

impl BossActorBehaviorRuntime {
    fn active_state(&self) -> Option<&ActiveBossState> {
        let control = self.control.as_ref()?;
        control
            .pattern
            .as_ref()
            .map(|pattern| &pattern.state)
            .or(control.resident_state.as_ref())
    }

    fn active_state_mut(&mut self) -> Option<&mut ActiveBossState> {
        let control = self.control.as_mut()?;
        if let Some(pattern) = control.pattern.as_mut() {
            Some(&mut pattern.state)
        } else {
            control.resident_state.as_mut()
        }
    }

    fn cancel_current_state(&mut self, effects: &mut Vec<BossRuntimeEffect>, end_pattern: bool) {
        if let (Some(unit_id), Some(state)) = (self.unit_id, self.active_state()) {
            for task in &state.tasks {
                if let ActiveBossTaskKind::StartAction {
                    action: Some(action),
                    ..
                } = &task.kind
                {
                    effects.push(BossRuntimeEffect::DetachAction {
                        unit_id,
                        action: action.clone(),
                        cancel_unreleased: true,
                    });
                }
            }
        }
        let cancel_combat = self.active_state().is_some_and(|state| {
            state
                .tasks
                .iter()
                .any(|task| matches!(task.kind, ActiveBossTaskKind::RouteCombat))
        });
        let state_activation_id = self.active_state().map(|state| state.activation_id);
        if let Some(unit_id) = self.unit_id {
            effects.push(BossRuntimeEffect::CancelOwnedWork {
                unit_id,
                cancel_combat,
                clear_leaf_target: self.target_lock.as_ref().is_some_and(|lock| {
                    matches!(lock.owner, BossTargetLockOwner::ResidentState(id) if Some(id) == state_activation_id)
                        || end_pattern
                }),
            });
        }
        if self.target_lock.as_ref().is_some_and(|lock| {
            matches!(lock.owner, BossTargetLockOwner::ResidentState(id) if Some(id) == state_activation_id)
                || end_pattern
        }) {
            self.target_lock = None;
        }
        if let Some(control) = self.control.as_mut() {
            if end_pattern {
                control.pattern = None;
            } else if let Some(pattern) = control.pattern.as_mut() {
                pattern.state.tasks.clear();
            } else {
                control.resident_state = None;
            }
        }
    }

    fn cancel_all_owned_work(&mut self, effects: &mut Vec<BossRuntimeEffect>) {
        self.cancel_current_state(effects, true);
        self.control = None;
        self.target_lock = None;
    }
}

impl BossRuntimeTransitionPlan {
    fn commit(self, core: &mut BattleCore, now_ms: u64) -> Result<(), GameError> {
        core.boss_runtime = Some(self.next_runtime);
        for effect in self.effects {
            match effect {
                BossRuntimeEffect::StartAction {
                    unit_id,
                    actor_index,
                    task_activation_id,
                    skill_id,
                    target,
                    facing,
                } => {
                    let result = match target {
                        Some(target) => {
                            core.start_boss_skill_action(now_ms, unit_id, &skill_id, target, facing)
                        }
                        None => Err(GameError::UnitNotFound),
                    };
                    let (action, failure) = match result {
                        Ok(action) => (Some(action), None),
                        Err(GameError::UnitNotFound) => {
                            (None, Some(BossTaskResultData::TargetInvalidated))
                        }
                        Err(GameError::InvalidAction | GameError::ActionUnavailable { .. }) => {
                            (None, Some(BossTaskResultData::RequestRejected))
                        }
                        Err(error) => return Err(error),
                    };
                    let task = core
                        .boss_runtime
                        .as_mut()
                        .and_then(|r| r.actors.get_mut(actor_index))
                        .and_then(|a| a.active_state_mut())
                        .and_then(|state| {
                            state
                                .tasks
                                .iter_mut()
                                .find(|t| t.activation_id == task_activation_id)
                        })
                        .ok_or(GameError::InvalidAction)?;
                    if let ActiveBossTaskKind::StartAction { action: owned, .. } = &mut task.kind {
                        *owned = action;
                    }
                    task.result = failure;
                }
                BossRuntimeEffect::DetachAction {
                    unit_id,
                    action,
                    cancel_unreleased,
                } => core.detach_boss_action(now_ms, unit_id, &action, cancel_unreleased)?,
                BossRuntimeEffect::EnqueueWake {
                    time_ms,
                    actor_slot_index,
                    task_activation_id,
                } => core.event_queue.push(BattleEvent::BossRuntimeWake {
                    time_ms,
                    actor_slot_index,
                    task_activation_id,
                }),
                BossRuntimeEffect::CancelOwnedWork {
                    unit_id,
                    cancel_combat,
                    clear_leaf_target,
                } => core.cancel_boss_owned_work(now_ms, unit_id, cancel_combat, clear_leaf_target),
                BossRuntimeEffect::ResumeRouteCombat { unit_id, time_ms } => {
                    core.resume_boss_route_combat(unit_id, time_ms)
                }
                BossRuntimeEffect::EmitSignal { signal_id } => {
                    if let Some(runtime) = core.boss_runtime.as_mut() {
                        runtime.note_signal_emitted(signal_id);
                    }
                }
            }
        }
        Ok(())
    }
}

impl BattleCore {
    fn detach_boss_action(
        &mut self,
        now_ms: u64,
        unit_id: UnitInstanceId,
        action: &ActionHandle,
        cancel_unreleased: bool,
    ) -> Result<(), GameError> {
        if cancel_unreleased {
            self.cancel_boss_action(now_ms, unit_id, action)?;
        }
        self.runtime_facts.detach_result(action)
    }

    /// Cancel only the work owned by this task's opaque action. Released carriers
    /// own independent sources and leases and must finish normally.
    fn cancel_boss_action(
        &mut self,
        now_ms: u64,
        unit_id: UnitInstanceId,
        action: &ActionHandle,
    ) -> Result<(), GameError> {
        use crate::authoring::control;
        use crate::game::battle::runtime_facts::ActionCancelReason;

        let pending = self.units.get_mut(&unit_id).and_then(|unit| {
            if !unit
                .pending_skill_cast
                .as_ref()
                .is_some_and(|cast| cast.action.as_ref() == Some(action))
            {
                return None;
            }
            let pending = unit.pending_skill_cast.take()?;
            let lock_until = unit.next_action_time;
            unit.pending_cast = false;
            unit.pending_cast_cause = None;
            unit.next_action_time = 0;
            Some((pending, lock_until))
        });
        if let Some((pending, lock_until)) = pending {
            let permissions = self
                .game_data
                .skill_data
                .get_by_id(pending.skill_id.as_str())
                .map(|skill| skill.focus_permissions)
                .unwrap_or_default();
            if let Some(unit) = self.units.get_mut(&unit_id) {
                if unit.action_locks.resonance_gain_until_ms == lock_until {
                    unit.action_locks.resonance_gain_until_ms = now_ms;
                }
                if !permissions.allows_basic_attack
                    && unit.action_locks.basic_attack_until_ms == lock_until
                {
                    unit.action_locks.basic_attack_until_ms = now_ms;
                }
                if !permissions.allows_move && unit.action_locks.movement_until_ms == lock_until {
                    unit.action_locks.movement_until_ms = now_ms;
                }
            }
            self.record_event_log(
                now_ms,
                BattleLogEvent::SkillCastInterrupted {
                    interrupter_instance_id: unit_id,
                    caster_instance_id: unit_id,
                    interrupted_skill_id: pending.skill_id,
                    interrupted_cast_seq: pending.start_seq,
                },
            );
        }
        control::poll("boss_action_pending_cancelled")?;
        loop {
            control::charge(
                self.active_skill_casts.len() as u64,
                "boss_action_cancel_scan",
            )?;
            let next = self
                .active_skill_casts
                .iter()
                .filter(|(_, cast)| {
                    cast.caster_instance_id == unit_id
                        && cast.fact_source.action.as_ref() == Some(action)
                })
                .map(|(seq, _)| *seq)
                .min();
            let Some(seq) = next else {
                break;
            };
            self.active_skill_casts.remove(&seq);
            control::poll("boss_action_continuation_removed")?;
            self.event_queue.discard_skill_continuations(seq)?;
        }
        self.runtime_facts.terminal(
            action,
            ActionTerminalKind::Cancelled(ActionCancelReason::Interrupted),
            now_ms,
        )?;
        control::poll("boss_action_cancel_committed")
    }

    fn release_finished_boss_waiters(&mut self, now_ms: u64) -> Result<(), GameError> {
        // Only active task-owned handles, never the global action store.
        let Some(mut runtime) = self.boss_runtime.take() else {
            return Ok(());
        };
        for actor in &mut runtime.actors {
            let Some(unit_id) = actor.unit_id else {
                continue;
            };
            if let Some(state) = actor.active_state_mut() {
                for task in &mut state.tasks {
                    if let (
                        Some(result),
                        ActiveBossTaskKind::StartAction {
                            action, on_timeout, ..
                        },
                    ) = (task.result, &mut task.kind)
                    {
                        if let Some(action) = action.take() {
                            self.detach_boss_action(
                                now_ms,
                                unit_id,
                                &action,
                                result == BossTaskResultData::TargetInvalidated
                                    || (result == BossTaskResultData::TimedOut
                                        && *on_timeout == BossActionTimeoutData::CancelUnreleased),
                            )?;
                        }
                    }
                }
            }
        }
        self.boss_runtime = Some(runtime);
        Ok(())
    }

    pub(in crate::game::battle::core) fn boss_actor_adapter_view(
        &self,
        actor_slot_id: &BossActorSlotId,
    ) -> Option<BossActorAdapterView> {
        let runtime = self.boss_runtime.as_ref()?;
        let actor_slot_index = runtime
            .definition
            .actor_slots
            .iter()
            .position(|slot| slot == actor_slot_id)?;
        let actor = runtime
            .actors
            .iter()
            .find(|actor| actor.encounter_slot_index == Some(actor_slot_index))?;
        let unit_ref = actor.unit_ref.as_ref()?;
        let resolved_unit_id = actor.unit_id.or_else(|| {
            self.scenario_runtime
                .spawned
                .unit_refs
                .get(unit_ref)
                .copied()
        });
        Some(BossActorAdapterView {
            actor_slot_id: actor_slot_id.clone(),
            unit_ref: unit_ref.clone(),
            unit_id: resolved_unit_id,
            ever_bound: actor.ever_bound || resolved_unit_id.is_some(),
        })
    }

    #[cfg(test)]
    pub(crate) fn boss_actor_runtime_debug_snapshots(&self) -> Vec<BossActorRuntimeDebugSnapshot> {
        let Some(runtime) = &self.boss_runtime else {
            return Vec::new();
        };
        runtime
            .actors
            .iter()
            .map(|actor| {
                let control = actor.control.as_ref();
                let control_node = control.and_then(|control| {
                    actor
                        .behavior
                        .as_ref()
                        .and_then(|behavior| behavior.nodes.get(control.node_index))
                });
                let control_node_id = control_node.map(|node| match node {
                    CompiledBossControlNode::State { id, .. }
                    | CompiledBossControlNode::PatternCall { id, .. } => id.as_str().to_string(),
                });
                let pattern = control.and_then(|control| control.pattern.as_ref());
                let state_id = pattern.map(|pattern| {
                    pattern.definition.states[pattern.state_index]
                        .id
                        .as_str()
                        .to_string()
                });
                let state = actor.active_state();
                BossActorRuntimeDebugSnapshot {
                    unit_id: actor.unit_id,
                    control_node_id,
                    pattern_id: pattern.map(|pattern| pattern.definition.id.as_str().to_string()),
                    pattern_activation_id: pattern.map(|pattern| pattern.activation_id),
                    state_id,
                    state_activation_id: state.map(|state| state.activation_id),
                    target_id: actor.target_lock.as_ref().map(|lock| lock.target_id),
                    route_cursor_index: actor.route_cursor_index,
                }
            })
            .collect()
    }

    pub(in crate::game::battle::core) fn run_boss_runtime_boundary(
        &mut self,
        now_ms: u64,
    ) -> Result<(), GameError> {
        self.run_boss_runtime_boundary_inner(now_ms)
            .map_err(|error| {
                crate::game::battle::runtime_control::trusted_failure(
                    error,
                    "boss_runtime_boundary",
                )
            })
    }

    fn run_boss_runtime_boundary_inner(&mut self, now_ms: u64) -> Result<(), GameError> {
        let Some(runtime) = self.boss_runtime.as_ref() else {
            return Ok(());
        };
        let limit = runtime.microstep_limit;
        self.preflight_authoring_clone()?;
        let mut staged = self.clone();
        crate::authoring::control::poll("boss_boundary_clone_complete")?;
        staged.event_log.prepare_authoring_write()?;
        let mut steps = 0usize;
        loop {
            staged.release_finished_boss_waiters(now_ms)?;
            let runtime = staged
                .boss_runtime
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            let Some(plan) = runtime.plan_next(&staged, now_ms)? else {
                if staged.settle_boss_summons(now_ms)? {
                    if steps >= limit {
                        return Err(GameError::InvalidAction);
                    }
                    steps += 1;
                    continue;
                }
                if let Some(runtime) = staged.boss_runtime.as_mut() {
                    runtime.clear_boundary_pulses();
                }
                *self = staged;
                return Ok(());
            };
            if steps >= limit {
                return Err(GameError::InvalidAction);
            }
            plan.commit(&mut staged, now_ms)?;
            staged.capture_boss_summon_requests()?;
            steps += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn u03_exhaust_boss_activation_fixture(&mut self) {
        self.boss_runtime.as_mut().unwrap().next_activation_id = u64::MAX;
    }

    #[cfg(test)]
    pub(crate) fn boss_signal_latched_for_test(
        &self,
        signal_id: &crate::game::data::boss_runtime_data::BossSignalId,
    ) -> bool {
        self.boss_runtime
            .as_ref()
            .is_some_and(|runtime| runtime.is_signal_latched(signal_id))
    }

    pub(in crate::game::battle::core) fn boss_runtime_allows_combat_action(
        &self,
        unit_id: UnitInstanceId,
    ) -> bool {
        let Some(runtime) = &self.boss_runtime else {
            return true;
        };
        let Some(actor) = runtime
            .actors
            .iter()
            .find(|actor| actor.unit_id == Some(unit_id))
        else {
            return true;
        };
        actor.active_state().is_some_and(|state| {
            state
                .tasks
                .iter()
                .any(|task| matches!(task.kind, ActiveBossTaskKind::RouteCombat))
        })
    }

    pub(super) fn wake_boss_action_waiters(&mut self, time_ms: u64) -> Result<(), GameError> {
        if let Some(runtime) = &self.boss_runtime {
            for actor in &runtime.actors {
                if let Some(state) = actor.active_state() {
                    for task in &state.tasks {
                        crate::authoring::control::charge(1, "boss_action_waiter_wake")?;
                        if task.result.is_none()
                            && matches!(task.kind, ActiveBossTaskKind::StartAction { .. })
                        {
                            self.event_queue.push(BattleEvent::BossRuntimeWake {
                                time_ms,
                                actor_slot_index: actor.actor_slot_index,
                                task_activation_id: task.activation_id,
                            });
                        }
                    }
                }
            }
        }
        crate::authoring::control::poll("boss_action_waiter_wake_complete")
    }

    pub(in crate::game::battle::core) fn battle_event_requests_boss_boundary(
        &self,
        event: &BattleEvent,
    ) -> bool {
        match event {
            BattleEvent::BossRuntimeWake {
                actor_slot_index,
                task_activation_id,
                ..
            } => self.boss_runtime.as_ref().is_some_and(|runtime| {
                runtime.wake_is_current(*actor_slot_index, *task_activation_id)
            }),
            _ => true,
        }
    }

    pub(in crate::game::battle::core) fn boss_movement_directive(
        &self,
        unit_id: UnitInstanceId,
        _now_ms: u64,
    ) -> BossMovementDirective {
        let Some(runtime) = &self.boss_runtime else {
            return BossMovementDirective::OrdinaryFallback;
        };
        let Some((actor_index, actor)) = runtime
            .actors
            .iter()
            .enumerate()
            .find(|(_, actor)| actor.unit_id == Some(unit_id))
        else {
            return BossMovementDirective::OrdinaryFallback;
        };
        let Some(state) = actor.active_state() else {
            return BossMovementDirective::Suppress;
        };
        if state
            .tasks
            .iter()
            .any(|task| matches!(task.kind, ActiveBossTaskKind::RouteCombat))
        {
            return BossMovementDirective::OrdinaryFallback;
        }
        for task in &state.tasks {
            if task.result.is_some() {
                continue;
            }
            match task.kind {
                ActiveBossTaskKind::MoveToTarget { .. } => {
                    let Some(target_id) = actor.target_lock.as_ref().map(|lock| lock.target_id)
                    else {
                        return BossMovementDirective::Suppress;
                    };
                    let Some(path) = path_to_target_approach(self, unit_id, target_id) else {
                        return BossMovementDirective::Suppress;
                    };
                    if let Some(next) = path.get(1).copied().or_else(|| path.first().copied()) {
                        return BossMovementDirective::Goal(MovementGoal::MoveToPoint {
                            point: WorldVec2::from_tile_center(next),
                            stop_radius: ROUTE_CELL_STOP_RADIUS,
                        });
                    }
                }
                ActiveBossTaskKind::RejoinForwardRoute {
                    target_route_index, ..
                } => {
                    let Some(target_index) = target_route_index else {
                        return BossMovementDirective::Suppress;
                    };
                    let route = runtime.route_cells_for_actor(self, actor_index);
                    let Some(target) = route.get(target_index).copied() else {
                        return BossMovementDirective::Suppress;
                    };
                    let Some(path) = grid_path(
                        self,
                        self.unit_projected_tile(unit_id),
                        &[target],
                        self.units
                            .get(&unit_id)
                            .is_some_and(|unit| unit.is_airborne()),
                    ) else {
                        return BossMovementDirective::Suppress;
                    };
                    if let Some(next) = path.get(1).copied().or_else(|| path.first().copied()) {
                        return BossMovementDirective::Goal(MovementGoal::MoveToPoint {
                            point: WorldVec2::from_tile_center(next),
                            stop_radius: ROUTE_CELL_STOP_RADIUS,
                        });
                    }
                }
                ActiveBossTaskKind::StartAction { .. }
                | ActiveBossTaskKind::RouteCombat
                | ActiveBossTaskKind::AcquireTarget
                | ActiveBossTaskKind::Wait { .. }
                | ActiveBossTaskKind::Summon(_)
                | ActiveBossTaskKind::EmitSignal => {}
            }
        }
        BossMovementDirective::Suppress
    }

    pub(in crate::game::battle::core) fn boss_route_cursor(
        &self,
        unit_id: UnitInstanceId,
    ) -> Option<usize> {
        self.boss_runtime.as_ref().and_then(|runtime| {
            runtime
                .actors
                .iter()
                .find(|actor| actor.unit_id == Some(unit_id))
                .map(|actor| actor.route_cursor_index)
        })
    }

    pub(super) fn cancel_boss_owned_work(
        &mut self,
        time_ms: u64,
        unit_id: UnitInstanceId,
        cancel_combat: bool,
        clear_leaf_target: bool,
    ) {
        if self.active_movement_segments.remove(&unit_id).is_some() {
            self.record_movement_stopped(time_ms, unit_id, MovementStopReason::WaitRepath, None);
        }
        let (pending_cast, cast_lock_until) = self
            .units
            .get_mut(&unit_id)
            .map(|unit| {
                unit.move_epoch = unit.move_epoch.wrapping_add(1);
                unit.action_state = ActionState::Idle;
                if clear_leaf_target {
                    unit.current_target = None;
                }
                if !cancel_combat {
                    return (None, None);
                }
                unit.basic_attack_epoch = unit.basic_attack_epoch.wrapping_add(1);
                unit.pending_basic_attack = false;
                if let Some(windup_until_ms) = unit.basic_attack_windup_until_ms.take() {
                    if unit.action_locks.basic_attack_until_ms == windup_until_ms {
                        unit.action_locks.basic_attack_until_ms = time_ms;
                    }
                    if unit.action_locks.movement_until_ms == windup_until_ms {
                        unit.action_locks.movement_until_ms = time_ms;
                    }
                }
                unit.pending_cast = false;
                unit.pending_cast_cause = None;
                let pending = unit.pending_skill_cast.take();
                let cast_lock_until = pending.as_ref().map(|_| unit.next_action_time);
                if pending.is_some() {
                    unit.next_action_time = 0;
                }
                (pending, cast_lock_until)
            })
            .unwrap_or((None, None));
        if cancel_combat {
            self.cancel_resonance_use_control(unit_id);
            self.cancel_structured_flows(
                unit_id,
                time_ms,
                crate::game::battle::runtime_facts::ActionCancelReason::Interrupted,
            );
            let _ = self.runtime_facts.cancel_unreleased_basic(
                unit_id,
                crate::game::battle::runtime_facts::ActionCancelReason::Interrupted,
                time_ms,
            );
        }
        if let Some(pending) = pending_cast {
            self.cancel_pending_action(
                pending.action.as_ref(),
                time_ms,
                crate::game::battle::runtime_facts::ActionCancelReason::Interrupted,
            );
            if let Some(cast_lock_until) = cast_lock_until {
                let focus_permissions = self
                    .game_data
                    .skill_data
                    .get_by_id(pending.skill_id.as_str())
                    .map(|skill| skill.focus_permissions)
                    .unwrap_or_default();
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    if unit.action_locks.resonance_gain_until_ms == cast_lock_until {
                        unit.action_locks.resonance_gain_until_ms = time_ms;
                    }
                    if !focus_permissions.allows_basic_attack
                        && unit.action_locks.basic_attack_until_ms == cast_lock_until
                    {
                        unit.action_locks.basic_attack_until_ms = time_ms;
                    }
                    if !focus_permissions.allows_move
                        && unit.action_locks.movement_until_ms == cast_lock_until
                    {
                        unit.action_locks.movement_until_ms = time_ms;
                    }
                }
            }
            self.record_event_log(
                time_ms,
                BattleLogEvent::SkillCastInterrupted {
                    interrupter_instance_id: unit_id,
                    caster_instance_id: unit_id,
                    interrupted_skill_id: pending.skill_id,
                    interrupted_cast_seq: pending.start_seq,
                },
            );
        }
    }

    fn resume_boss_route_combat(&mut self, unit_id: UnitInstanceId, time_ms: u64) {
        let attack_already_queued = self.event_queue.iter().any(|event| {
            matches!(
                event.as_ref(),
                BattleEvent::AttackStart {
                    time_ms: queued_time,
                    attacker_instance_id,
                    schedule_next: true,
                    ..
                } if *attacker_instance_id == unit_id && *queued_time >= time_ms
            )
        });
        if !attack_already_queued {
            self.schedule_initial_attack_for_unit(unit_id, time_ms);
        }
        self.schedule_pending_autocast_for(unit_id, time_ms);
    }
}

fn allocate_activation_id(next: &mut u64) -> Result<u64, GameError> {
    let id = *next;
    *next = next.checked_add(1).ok_or(GameError::InvalidAction)?;
    Ok(id)
}

fn resolve_duration(
    value: &CompiledBossDurationValue,
    frame: Option<&ResolvedBossArgumentFrame>,
) -> Result<u64, GameError> {
    let value = match value {
        CompiledBossDurationValue::Literal(value) => *value,
        CompiledBossDurationValue::Parameter(index) => {
            match frame.and_then(|frame| frame.values.get(*index)) {
                Some(ResolvedBossParameterValue::DurationMs(value)) => *value,
                _ => return Err(GameError::InvalidAction),
            }
        }
    };
    (value > 0).then_some(value).ok_or(GameError::InvalidAction)
}

fn resolve_target_query<'a>(
    value: &'a CompiledBossTargetQueryValue,
    frame: Option<&'a ResolvedBossArgumentFrame>,
) -> Result<&'a BossTargetQueryData, GameError> {
    match value {
        CompiledBossTargetQueryValue::Literal(query) => Ok(query),
        CompiledBossTargetQueryValue::Parameter(index) => {
            match frame.and_then(|frame| frame.values.get(*index)) {
                Some(ResolvedBossParameterValue::TargetQuery(query)) => Ok(query),
                _ => Err(GameError::InvalidAction),
            }
        }
    }
}

fn candidate_cooldown_ready(
    actor: &BossActorBehaviorRuntime,
    candidate: &CompiledBossPatternCandidate,
    now_ms: u64,
) -> bool {
    candidate.cooldown.as_ref().is_none_or(|cooldown| {
        actor
            .cooldown_ready_at_ms
            .get(&cooldown.group_id)
            .is_none_or(|ready_at_ms| now_ms >= *ready_at_ms)
    })
}

fn condition_is_true(
    core: &BattleCore,
    actor: &BossActorBehaviorRuntime,
    state: &ActiveBossState,
    condition: &BossConditionData,
) -> bool {
    match condition {
        BossConditionData::Always => true,
        BossConditionData::TaskResult { task_id, result } => state.tasks.iter().any(|task| {
            &task.id == task_id && task.result == Some(*result) && !task.result_consumed
        }),
        BossConditionData::StatePredicate(BossStatePredicateData::SelfHealthAtOrBelowPercent {
            percent,
        }) => actor
            .unit_id
            .and_then(|unit_id| core.units.get(&unit_id))
            .is_some_and(|unit| {
                u64::from(unit.stats.current_health) * 100
                    <= u64::from(unit.stats.max_health) * u64::from(*percent)
            }),
        BossConditionData::EventOccurred {
            event: BossEventConditionData::TargetInvalidated,
            memory,
        } => match memory {
            BossEventMemoryData::Pulse => state.target_invalidated_pulse,
            BossEventMemoryData::UntilStateExit => state.target_invalidated_latch,
        },
        BossConditionData::SummonBatchCleared { batch_handle_id } => actor
            .control
            .as_ref()
            .and_then(|control| control.pattern.as_ref())
            .is_some_and(|pattern| pattern.cleared_batch_handles.contains(batch_handle_id)),
    }
}

fn consume_condition_source(state: &mut ActiveBossState, condition: &BossConditionData) {
    match condition {
        BossConditionData::TaskResult { task_id, result } => {
            if let Some(task) = state.tasks.iter_mut().find(|task| {
                &task.id == task_id && task.result == Some(*result) && !task.result_consumed
            }) {
                task.result_consumed = true;
            }
        }
        BossConditionData::EventOccurred {
            event: BossEventConditionData::TargetInvalidated,
            memory: BossEventMemoryData::Pulse,
        } => state.target_invalidated_pulse = false,
        BossConditionData::Always
        | BossConditionData::StatePredicate(_)
        | BossConditionData::SummonBatchCleared { .. }
        | BossConditionData::EventOccurred {
            memory: BossEventMemoryData::UntilStateExit,
            ..
        } => {}
    }
}

fn target_identity_is_valid(
    core: &BattleCore,
    actor_index: usize,
    actors: &[BossActorBehaviorRuntime],
    target_id: UnitInstanceId,
) -> bool {
    let Some(owner) = actors
        .get(actor_index)
        .and_then(|actor| actor.unit_id)
        .and_then(|unit_id| core.units.get(&unit_id))
        .map(|unit| unit.owner)
    else {
        return false;
    };
    core.units
        .get(&target_id)
        .is_some_and(|target| target.is_active() && target.owner != owner)
}

fn select_target(
    core: &BattleCore,
    actor_id: Option<UnitInstanceId>,
    query: &BossTargetQueryData,
) -> Option<UnitInstanceId> {
    let actor_id = actor_id?;
    let actor = core.units.get(&actor_id)?;
    if query.target_kind != BossTargetKindData::EnemyUnit {
        return None;
    }
    let mut candidates = core
        .units
        .values()
        .filter(|candidate| {
            candidate.instance_id != actor_id
                && candidate.owner != actor.owner
                && query.filters.iter().all(|filter| match filter {
                    BossTargetFilterData::ActiveHostile => {
                        candidate.is_active() && candidate.owner != actor.owner
                    }
                    BossTargetFilterData::Grounded => !candidate.is_airborne(),
                    BossTargetFilterData::Reachable => {
                        path_to_target_approach(core, actor_id, candidate.instance_id).is_some()
                    }
                })
        })
        .map(|candidate| candidate.instance_id)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        for key in &query.order_by {
            let order = match key {
                BossTargetOrderKeyData::MinPathCost => {
                    let left_cost = path_to_target_approach(core, actor_id, *left)
                        .map(|path| path.len().saturating_sub(1))
                        .unwrap_or(usize::MAX);
                    let right_cost = path_to_target_approach(core, actor_id, *right)
                        .map(|path| path.len().saturating_sub(1))
                        .unwrap_or(usize::MAX);
                    left_cost.cmp(&right_cost)
                }
                BossTargetOrderKeyData::MinHpRatio => {
                    let left_unit = &core.units[left];
                    let right_unit = &core.units[right];
                    (u64::from(left_unit.stats.current_health)
                        * u64::from(right_unit.stats.max_health))
                    .cmp(
                        &(u64::from(right_unit.stats.current_health)
                            * u64::from(left_unit.stats.max_health)),
                    )
                }
            };
            if !order.is_eq() {
                return order;
            }
        }
        let left_unit = &core.units[left];
        let right_unit = &core.units[right];
        left_unit
            .spawn_order
            .cmp(&right_unit.spawn_order)
            .then_with(|| left.as_bytes().cmp(right.as_bytes()))
    });
    candidates.into_iter().next()
}

fn move_to_target_reached(
    core: &BattleCore,
    actor_id: UnitInstanceId,
    target_id: UnitInstanceId,
) -> bool {
    match (
        core.unit_projected_tile(actor_id),
        core.unit_projected_tile(target_id),
    ) {
        (Some(actor), Some(target)) => actor.chebyshev(&target) <= MOVE_TO_TARGET_STOP_CHEBYSHEV,
        _ => false,
    }
}

fn path_to_target_approach(
    core: &BattleCore,
    actor_id: UnitInstanceId,
    target_id: UnitInstanceId,
) -> Option<Vec<Position>> {
    let actor = core.units.get(&actor_id)?;
    let target = core.units.get(&target_id)?;
    if !actor.is_active() || !target.is_active() || actor.owner == target.owner {
        return None;
    }
    let target_tile = target.body.projected_tile();
    let mut goals = vec![target_tile];
    goals.extend(grid_neighbors(target_tile));
    goals.retain(|position| {
        if actor.is_airborne() {
            core.battlefield.is_valid_tile(*position)
        } else {
            core.battlefield.is_walkable_tile(*position)
        }
    });
    goals.sort_by_key(|position| (position.y, position.x));
    goals.dedup();
    grid_path(
        core,
        Some(actor.body.projected_tile()),
        &goals,
        actor.is_airborne(),
    )
}

fn grid_path(
    core: &BattleCore,
    start: Option<Position>,
    goals: &[Position],
    airborne: bool,
) -> Option<Vec<Position>> {
    let start = start?;
    let goals = goals.iter().copied().collect::<HashSet<_>>();
    if goals.is_empty() {
        return None;
    }
    let walkable = |position: Position| {
        if airborne {
            core.battlefield.is_valid_tile(position)
        } else {
            core.battlefield.is_walkable_tile(position)
        }
    };
    if !walkable(start) {
        return None;
    }
    let mut queue = VecDeque::from([start]);
    let mut distance = HashMap::from([(start, 0usize)]);
    let mut predecessor = HashMap::<Position, Position>::new();
    while let Some(position) = queue.pop_front() {
        let next_distance = distance[&position].saturating_add(1);
        for neighbor in grid_neighbors(position) {
            if !walkable(neighbor) || distance.contains_key(&neighbor) {
                continue;
            }
            distance.insert(neighbor, next_distance);
            predecessor.insert(neighbor, position);
            queue.push_back(neighbor);
        }
    }
    let goal = goals
        .into_iter()
        .filter_map(|goal| {
            distance
                .get(&goal)
                .map(|cost| (*cost, goal.y, goal.x, goal))
        })
        .min_by_key(|(cost, y, x, _)| (*cost, *y, *x))
        .map(|(_, _, _, goal)| goal)?;
    let mut reversed = vec![goal];
    let mut current = goal;
    while current != start {
        current = *predecessor.get(&current)?;
        reversed.push(current);
    }
    reversed.reverse();
    Some(reversed)
}

fn grid_neighbors(position: Position) -> [Position; 4] {
    [
        Position::new(position.x, position.y - 1),
        Position::new(position.x - 1, position.y),
        Position::new(position.x + 1, position.y),
        Position::new(position.x, position.y + 1),
    ]
}

fn nearest_route_index(cells: &[Position], position: WorldVec2) -> Option<usize> {
    cells
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            let distance = position.distance_squared(WorldVec2::from_tile_center(*cell));
            (index, distance)
        })
        .min_by(
            |(left_index, left_distance), (right_index, right_distance)| {
                left_distance
                    .total_cmp(right_distance)
                    .then_with(|| right_index.cmp(left_index))
            },
        )
        .map(|(index, _)| index)
}

fn route_point_reached(core: &BattleCore, unit_id: UnitInstanceId, target: Position) -> bool {
    core.units.get(&unit_id).is_some_and(|unit| {
        unit.body
            .position
            .distance(WorldVec2::from_tile_center(target))
            <= ROUTE_CELL_STOP_RADIUS
    })
}

#[cfg(test)]
mod tests {
    include!("boss_action_tests.rs");
    include!("boss_action_cancellation_tests.rs");
    include!("tests/u06_starter_effects.inc.rs");
    use super::*;
    use uuid::Uuid;

    use crate::game::{
        ability::{
            DeliveryDef, FocusPermissions, ProjectileHitPolicy, SkillActivationMode,
            SkillCastTargetingDef, SkillDef, SkillId, SkillStepDef, SkillTarget, SkillTargetReach,
        },
        battle::{
            core::types::PendingSkillCast,
            event_log::{BattleEventCause, BattleEventRootCause, BattleLogEvent},
            scenario::{
                BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioEvent, ScenarioEventId,
                ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger, ScenarioUnitSpawn,
                TacticalPlan,
            },
            types::{BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass, UnitCombatProfile},
        },
        data::{
            boss_runtime_data::{
                BossActorSlotId, BossEncounterConditionData, BossEncounterPhaseId,
                BossEncounterPhaseTransitionData, BossPhaseActorBehaviorData, BossTransitionId,
                ResolvedBossActorSetup, ResolvedBossEncounterSetup,
            },
            skill_data::SkillDatabase,
            GameDataBuilder,
        },
        enums::{Side, Tier},
        growth::GrowthStack,
        stats::UnitStats,
    };

    fn fixture_draft(id: u128, hp: u32, move_speed_units_per_ms: u32) -> BattleUnitDraft {
        let mut profile = UnitCombatProfile::employee_default();
        profile.weapon_profile = Some(Default::default());
        profile.stats = UnitStats::with_values(hp, hp, 1, 0, 1_000);
        profile.stats.move_speed_units_per_ms = move_speed_units_per_ms;
        profile.basic_attack.interval_ms = 1_000;
        profile.basic_attack.windup_ms = 1;
        profile.skill_activation_mode = SkillActivationMode::Auto;
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(id),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(id + 100),
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

    fn fixture_setup() -> ResolvedBossEncounterSetup {
        let database = crate::game::data::boss_runtime_data::tests::valid_catalog()
            .compile()
            .expect("valid Boss fixture");
        ResolvedBossEncounterSetup {
            definition: database
                .get_encounter(
                    &crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new(
                        "tracking_encounter",
                    ),
                )
                .expect("fixture encounter")
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

    fn fixture_setup_with_phase_timing(
        timing: BossPhaseTransitionTimingData,
    ) -> ResolvedBossEncounterSetup {
        let mut catalog = crate::game::data::boss_runtime_data::tests::valid_catalog();
        let phase_two_id = BossEncounterPhaseId::new("phase_two");
        let mut phase_two = catalog.encounters[0].phases[0].clone();
        phase_two.id = phase_two_id.clone();
        phase_two.transitions.clear();
        catalog.encounters[0].phases[0].transitions = vec![BossEncounterPhaseTransitionData {
            id: BossTransitionId::new("health_phase_change"),
            priority: 10,
            condition: BossEncounterConditionData::ActorHealthAtOrBelowPercent {
                actor_slot_id: BossActorSlotId::new("primary"),
                percent: 50,
            },
            timing,
            target_phase_id: phase_two_id,
        }];
        catalog.encounters[0].phases.push(phase_two);
        let database = catalog.compile().expect("two-phase Boss fixture");
        ResolvedBossEncounterSetup {
            definition: database
                .get_encounter(
                    &crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new(
                        "tracking_encounter",
                    ),
                )
                .expect("fixture encounter")
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

    fn fixture_setup_with_two_actors(
        timing: BossPhaseTransitionTimingData,
    ) -> ResolvedBossEncounterSetup {
        let mut catalog = crate::game::data::boss_runtime_data::tests::valid_catalog();
        let secondary = BossActorSlotId::new("secondary");
        catalog.encounters[0].actor_slots.push(secondary.clone());
        catalog.encounters[0].phases[0]
            .actor_behaviors
            .push(BossPhaseActorBehaviorData {
                actor_slot_id: secondary,
                behavior_id: catalog.actor_behaviors[0].id.clone(),
            });
        let phase_two_id = BossEncounterPhaseId::new("phase_two");
        let mut phase_two = catalog.encounters[0].phases[0].clone();
        phase_two.id = phase_two_id.clone();
        phase_two.transitions.clear();
        catalog.encounters[0].phases[0].transitions = vec![BossEncounterPhaseTransitionData {
            id: BossTransitionId::new("health_phase_change"),
            priority: 10,
            condition: BossEncounterConditionData::ActorHealthAtOrBelowPercent {
                actor_slot_id: BossActorSlotId::new("primary"),
                percent: 50,
            },
            timing,
            target_phase_id: phase_two_id,
        }];
        catalog.encounters[0].phases.push(phase_two);
        let database = catalog.compile().expect("two-actor Boss fixture");
        ResolvedBossEncounterSetup {
            definition: database
                .get_encounter(
                    &crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new(
                        "tracking_encounter",
                    ),
                )
                .expect("fixture encounter")
                .clone(),
            actors: vec![
                ResolvedBossActorSetup {
                    actor_slot_index: 0,
                    unit_ref: ScenarioUnitRef::new("boss_0"),
                },
                ResolvedBossActorSetup {
                    actor_slot_index: 1,
                    unit_ref: ScenarioUnitRef::new("boss_1"),
                },
            ],
            enemy_stat_scale: Default::default(),
            summon_spawn_zones: Vec::new(),
            summon_routes: Vec::new(),
        }
    }

    fn fixture_scenario_with_setup(
        boss_runtime: Option<ResolvedBossEncounterSetup>,
    ) -> BattleScenario {
        let boss_group_id = ScenarioGroupId::new("boss_group");
        let player_group_id = ScenarioGroupId::new("player_group");
        BattleScenario {
            battlefield: BattleFieldSpec {
                object_placement: None,
                width: 8,
                height: 4,
                valid_tiles: Vec::new(),
                obstacles: Vec::new(),
            },
            groups: vec![
                ScenarioSpawnGroup {
                    id: boss_group_id.clone(),
                    side: Side::Opponent,
                    required_for_victory: true,
                    enemy_movement_plan: Some(EnemyMovementPlan::PathAlongCells {
                        cells: vec![
                            Position::new(6, 1),
                            Position::new(5, 1),
                            Position::new(4, 1),
                            Position::new(3, 1),
                            Position::new(2, 1),
                        ],
                    }),
                    spawns: vec![ScenarioUnitSpawn {
                        unit_ref: ScenarioUnitRef::new("boss_0"),
                        side: Side::Opponent,
                        draft: fixture_draft(1, 1_000, 1_000),
                        position: Position::new(6, 1),
                        instance_salt: 0,
                    }],
                },
                ScenarioSpawnGroup {
                    id: player_group_id.clone(),
                    side: Side::Player,
                    required_for_victory: false,
                    enemy_movement_plan: None,
                    spawns: vec![
                        ScenarioUnitSpawn {
                            unit_ref: ScenarioUnitRef::new("player_0"),
                            side: Side::Player,
                            draft: fixture_draft(2, 1_000, 0),
                            position: Position::new(1, 1),
                            instance_salt: 0,
                        },
                        ScenarioUnitSpawn {
                            unit_ref: ScenarioUnitRef::new("player_1"),
                            side: Side::Player,
                            draft: fixture_draft(3, 1_000, 0),
                            position: Position::new(1, 2),
                            instance_salt: 1,
                        },
                    ],
                },
            ],
            events: vec![
                ScenarioEvent {
                    id: ScenarioEventId::new("spawn_boss"),
                    trigger: ScenarioTrigger::AtBattleStart,
                    action: ScenarioAction::SpawnGroup {
                        group_id: boss_group_id,
                    },
                },
                ScenarioEvent {
                    id: ScenarioEventId::new("spawn_players"),
                    trigger: ScenarioTrigger::AtBattleStart,
                    action: ScenarioAction::SpawnGroup {
                        group_id: player_group_id,
                    },
                },
            ],
            terminal_outcomes: BattleScenario::fixture_required_group_outcomes(),
            tactical_plan: TacticalPlan::default(),
            boss_runtime,
        }
    }

    fn fixture_scenario(with_boss: bool) -> BattleScenario {
        fixture_scenario_with_setup(with_boss.then(fixture_setup))
    }

    fn fixture_core(with_boss: bool) -> BattleCore {
        BattleCore::new_from_scenario(
            fixture_scenario(with_boss),
            GameDataBuilder::empty().build_arc(),
            17,
        )
    }

    fn actor_runtime(core: &BattleCore) -> &BossActorBehaviorRuntime {
        &core.boss_runtime.as_ref().expect("Boss runtime").actors[0]
    }

    #[test]
    fn opt_in_initializes_two_layer_runtime_before_same_time_actor_actions() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().expect("start");

        assert!(matches!(
            core.step_battle_execution(&mut execution),
            Ok(crate::game::battle::core::sim::BattleStepOutcome::Running)
        ));

        let runtime = core.boss_runtime.as_ref().expect("encounter runtime");
        let actor = &runtime.actors[0];
        assert!(runtime.initialized);
        assert!(runtime.phase_activation_id.is_some());
        assert!(actor.unit_id.is_some());
        assert!(actor.behavior_activation_id.is_some());
        let pattern = actor
            .control
            .as_ref()
            .and_then(|control| control.pattern.as_ref())
            .expect("Pattern should be admitted at t0");
        assert_eq!(pattern.definition.id.as_str(), "track_and_rejoin");
        assert_eq!(
            pattern.definition.states[pattern.state_index].id.as_str(),
            "approach"
        );
        assert!(actor.target_lock.is_some());
        assert_eq!(
            actor
                .cooldown_ready_at_ms
                .get(&BossCooldownGroupId::new("tracking")),
            Some(&2_000)
        );
        assert!(!core.boss_runtime_allows_combat_action(actor.unit_id.unwrap()));
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::AttackStart {
                attacker_instance_id,
                ..
            } if attacker_instance_id == actor.unit_id.unwrap()
        )));
    }

    #[test]
    fn no_binding_has_no_runtime_and_keeps_ordinary_authority() {
        let mut core = fixture_core(false);
        let mut execution = core.start_battle_execution().expect("start");
        core.step_battle_execution(&mut execution)
            .expect("first bucket");
        assert!(core.boss_runtime.is_none());
        for unit_id in core.units.keys().copied() {
            assert!(core.boss_runtime_allows_combat_action(unit_id));
            assert_eq!(
                core.boss_movement_directive(unit_id, 0),
                BossMovementDirective::OrdinaryFallback
            );
        }
    }

    #[test]
    fn lexicographic_query_and_final_spawn_identity_are_insertion_independent() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_id = actor_runtime(&core).unit_id.unwrap();
        let query = BossTargetQueryData {
            target_kind: BossTargetKindData::EnemyUnit,
            filters: vec![
                BossTargetFilterData::ActiveHostile,
                BossTargetFilterData::Grounded,
                BossTargetFilterData::Reachable,
            ],
            order_by: vec![
                BossTargetOrderKeyData::MinPathCost,
                BossTargetOrderKeyData::MinHpRatio,
            ],
            lock_policy: BossTargetLockPolicyData::LockUntilInvalid,
        };
        let mut candidate_ids = core
            .units
            .values()
            .filter(|unit| unit.owner == Side::Player)
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        candidate_ids.sort_by_key(|id| core.units[id].spawn_order);
        let equal_position = WorldVec2::from_tile_center(Position::new(1, 1));
        for candidate_id in &candidate_ids {
            let unit = core.units.get_mut(candidate_id).unwrap();
            unit.set_world_position(equal_position);
            unit.stats.current_health = unit.stats.max_health;
        }
        let expected = select_target(&core, Some(actor_id), &query).unwrap();
        assert_eq!(expected, candidate_ids[0]);
        let removed = candidate_ids
            .iter()
            .map(|id| (*id, core.units.remove(id).unwrap()))
            .collect::<Vec<_>>();
        for (id, unit) in removed.into_iter().rev() {
            core.units.insert(id, unit);
        }
        assert_eq!(select_target(&core, Some(actor_id), &query), Some(expected));
    }

    #[test]
    fn locked_target_identity_survives_relocation_without_reselection() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let locked = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        core.units
            .get_mut(&locked)
            .unwrap()
            .set_world_position(WorldVec2::from_tile_center(Position::new(3, 3)));

        core.run_boss_runtime_boundary(1).unwrap();

        assert_eq!(
            actor_runtime(&core).target_lock.as_ref().unwrap().target_id,
            locked
        );
        assert!(
            !actor_runtime(&core)
                .active_state()
                .unwrap()
                .target_invalidated_latch
        );
    }

    #[test]
    fn late_spawn_binds_and_activates_before_its_same_time_actor_action() {
        let mut scenario = fixture_scenario(true);
        scenario.events[0].trigger = ScenarioTrigger::AtTimeMs(10);
        let mut core =
            BattleCore::new_from_scenario(scenario, GameDataBuilder::empty().build_arc(), 17);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor = actor_runtime(&core);
        assert!(actor.unit_id.is_none());
        assert!(actor.behavior.is_some());
        assert!(actor.control.is_none());

        core.step_battle_execution_until(&mut execution, 10)
            .unwrap();

        let actor = actor_runtime(&core);
        let actor_id = actor.unit_id.expect("late bound identity");
        assert!(actor.behavior_activation_id.is_some());
        assert!(actor
            .control
            .as_ref()
            .and_then(|control| control.pattern.as_ref())
            .is_some());
        assert!(!core.boss_runtime_allows_combat_action(actor_id));
        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::AttackStart {
                attacker_instance_id,
                ..
            } if attacker_instance_id == actor_id
        )));
    }

    #[test]
    fn enemy_target_kind_excludes_self_and_allies_without_relying_on_optional_filters() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_id = actor_runtime(&core).unit_id.unwrap();
        let query = BossTargetQueryData {
            target_kind: BossTargetKindData::EnemyUnit,
            filters: Vec::new(),
            order_by: vec![BossTargetOrderKeyData::MinHpRatio],
            lock_policy: BossTargetLockPolicyData::LockUntilInvalid,
        };

        let selected = select_target(&core, Some(actor_id), &query).expect("hostile candidate");
        assert_ne!(selected, actor_id);
        assert_ne!(core.units[&selected].owner, core.units[&actor_id].owner);
    }

    #[test]
    fn exact_deadline_prefers_domain_completion_and_stale_wakes_do_not_reenter() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_id = actor_runtime(&core).unit_id.unwrap();
        let target_id = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        let target_tile = core.units[&target_id].body.projected_tile();
        core.units
            .get_mut(&actor_id)
            .unwrap()
            .set_world_position(WorldVec2::from_tile_center(Position::new(
                target_tile.x + 1,
                target_tile.y,
            )));

        let mut runtime = core.boss_runtime.take().unwrap();
        let move_task = runtime.actors[0]
            .active_state_mut()
            .unwrap()
            .tasks
            .iter_mut()
            .find(|task| matches!(task.kind, ActiveBossTaskKind::MoveToTarget { .. }))
            .unwrap();
        move_task.kind = ActiveBossTaskKind::MoveToTarget { deadline_ms: 10 };
        move_task.result = None;
        assert!(runtime.poll_actor_once(&core, 0, 10).unwrap());
        assert_eq!(
            runtime.actors[0].active_state().unwrap().tasks[0].result,
            Some(BossTaskResultData::Reached)
        );

        let state = runtime.actors[0].active_state_mut().unwrap();
        state.tasks = vec![ActiveBossTask {
            id: BossTaskId::new("wait_exact"),
            activation_id: 4_242,
            kind: ActiveBossTaskKind::Wait {
                completes_at_ms: 20,
            },
            result: None,
            result_consumed: false,
        }];
        assert!(runtime.wake_is_current(0, 4_242));
        assert!(!runtime.poll_actor_once(&core, 0, 19).unwrap());
        assert!(runtime.poll_actor_once(&core, 0, 20).unwrap());
        assert_eq!(
            runtime.actors[0].active_state().unwrap().tasks[0].result,
            Some(BossTaskResultData::Completed)
        );
        assert!(!runtime.wake_is_current(0, 4_242));
        core.boss_runtime = Some(runtime);
        assert!(
            !core.battle_event_requests_boss_boundary(&BattleEvent::BossRuntimeWake {
                time_ms: 20,
                actor_slot_index: 0,
                task_activation_id: 4_242,
            })
        );
    }

    #[test]
    fn phase_reentry_uses_fresh_ids_and_resets_phase_local_selector_only() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let mut runtime = core.boss_runtime.take().unwrap();
        let old_phase_id = runtime.phase_activation_id.unwrap();
        let old_behavior_id = runtime.actors[0].behavior_activation_id.unwrap();
        let old_control_id = runtime.actors[0].control.as_ref().unwrap().activation_id;
        let old_pattern_id = runtime.actors[0]
            .control
            .as_ref()
            .unwrap()
            .pattern
            .as_ref()
            .unwrap()
            .activation_id;
        let cooldowns = runtime.actors[0].cooldown_ready_at_ms.clone();
        assert_eq!(runtime.actors[0].fixed_sequence_cursors.get(&0), Some(&1));

        let mut effects = Vec::new();
        runtime
            .enter_phase(&core, 0, 100, &mut effects, true)
            .unwrap();

        let actor = &runtime.actors[0];
        assert!(runtime.phase_activation_id.unwrap() > old_phase_id);
        assert!(actor.behavior_activation_id.unwrap() > old_behavior_id);
        assert!(actor.control.as_ref().unwrap().activation_id > old_control_id);
        assert!(actor.control.as_ref().unwrap().pattern.is_none());
        assert!(actor.target_lock.is_none());
        assert!(actor.fixed_sequence_cursors.is_empty());
        assert_eq!(actor.cooldown_ready_at_ms, cooldowns);
        assert!(actor.control.as_ref().unwrap().activation_id > old_pattern_id);

        let next = runtime
            .plan_next(&core, 100)
            .unwrap()
            .expect("cooldown no-eligible route");
        let next_actor = &next.next_runtime.actors[0];
        let next_control = next_actor.control.as_ref().unwrap();
        assert!(next_control.pattern.is_none());
        assert!(matches!(
            &next_actor.behavior.as_ref().unwrap().nodes[next_control.node_index],
            CompiledBossControlNode::State { id, .. } if id.as_str() == "route_combat"
        ));
    }

    #[test]
    fn phase_transition_timing_waits_for_the_authored_boundary() {
        for (timing, expected_wakes) in [
            (BossPhaseTransitionTimingData::Immediate, 1),
            (BossPhaseTransitionTimingData::AfterCurrentState, 1),
            (BossPhaseTransitionTimingData::AfterCurrentPattern, 2),
        ] {
            let setup = fixture_setup_with_phase_timing(timing);
            let mut core = BattleCore::new_from_scenario(
                fixture_scenario_with_setup(Some(setup)),
                GameDataBuilder::empty().build_arc(),
                17,
            );
            let mut execution = core.start_battle_execution().unwrap();
            core.step_battle_execution(&mut execution).unwrap();
            let actor_id = actor_runtime(&core).unit_id.unwrap();
            let target_id = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
            let target_tile = core.units[&target_id].body.projected_tile();
            core.units.get_mut(&actor_id).unwrap().stats.current_health = 500;
            core.units
                .get_mut(&actor_id)
                .unwrap()
                .set_world_position(WorldVec2::from_tile_center(Position::new(
                    target_tile.x + 1,
                    target_tile.y,
                )));

            core.run_boss_runtime_boundary(1).unwrap();

            let runtime = core.boss_runtime.as_ref().unwrap();
            assert_eq!(runtime.current_phase_index, 1, "timing: {timing:?}");
            assert!(runtime.pending_phase_transition.is_none());
            assert_eq!(
                core.event_queue
                    .iter()
                    .filter(|event| matches!(event.as_ref(), BattleEvent::BossRuntimeWake { .. }))
                    .count(),
                expected_wakes,
                "timing: {timing:?}"
            );
        }
    }

    #[test]
    fn authored_priority_wins_independently_of_task_result_storage_order() {
        for results in [
            [
                BossTaskResultData::NoEligibleTarget,
                BossTaskResultData::Acquired,
            ],
            [
                BossTaskResultData::Acquired,
                BossTaskResultData::NoEligibleTarget,
            ],
        ] {
            let mut core = fixture_core(true);
            let mut execution = core.start_battle_execution().unwrap();
            core.step_battle_execution(&mut execution).unwrap();
            let mut runtime = core.boss_runtime.take().unwrap();
            let pattern = runtime.actors[0]
                .control
                .as_mut()
                .unwrap()
                .pattern
                .as_mut()
                .unwrap();
            pattern.state_index = 0;
            pattern.state = ActiveBossState {
                activation_id: 9_000,
                tasks: results
                    .into_iter()
                    .enumerate()
                    .map(|(index, result)| ActiveBossTask {
                        id: BossTaskId::new("acquire_target"),
                        activation_id: 9_001 + index as u64,
                        kind: ActiveBossTaskKind::AcquireTarget,
                        result: Some(result),
                        result_consumed: false,
                    })
                    .collect(),
                target_invalidated_pulse: false,
                target_invalidated_latch: false,
            };
            let mut effects = Vec::new();
            assert!(runtime
                .advance_pattern_once(&core, 0, 1, &mut effects)
                .unwrap());
            let pattern = runtime.actors[0]
                .control
                .as_ref()
                .unwrap()
                .pattern
                .as_ref()
                .expect("higher-priority Acquired transition keeps the Pattern active");
            assert_eq!(
                pattern.definition.states[pattern.state_index].id.as_str(),
                "approach"
            );
        }
    }

    #[test]
    fn continue_state_consumes_pulse_but_preserves_until_exit_latch() {
        let core = fixture_core(true);
        let actor = &core.boss_runtime.as_ref().unwrap().actors[0];
        let mut state = ActiveBossState {
            activation_id: 1,
            tasks: Vec::new(),
            target_invalidated_pulse: true,
            target_invalidated_latch: true,
        };
        let pulse = BossConditionData::EventOccurred {
            event: BossEventConditionData::TargetInvalidated,
            memory: BossEventMemoryData::Pulse,
        };
        let latch = BossConditionData::EventOccurred {
            event: BossEventConditionData::TargetInvalidated,
            memory: BossEventMemoryData::UntilStateExit,
        };
        assert!(condition_is_true(&core, actor, &state, &pulse));
        assert!(condition_is_true(&core, actor, &state, &latch));

        consume_condition_source(&mut state, &pulse);

        assert!(!condition_is_true(&core, actor, &state, &pulse));
        assert!(condition_is_true(&core, actor, &state, &latch));
    }

    #[test]
    fn guarded_continue_state_exhausts_at_the_authored_iteration_limit() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let mut runtime = core.boss_runtime.take().unwrap();
        let pattern = runtime.actors[0]
            .control
            .as_ref()
            .unwrap()
            .pattern
            .as_ref()
            .unwrap();
        let mut transition = pattern.definition.states[pattern.state_index].transitions[0].clone();
        transition.target = CompiledBossPatternTransitionTarget::ContinueState;
        transition.loop_guard = Some(
            crate::game::data::boss_runtime_data::CompiledBossLoopGuard {
                id: crate::game::data::boss_runtime_data::BossLoopId::new("bounded_continue"),
                max_iterations: 2,
                exhausted_target: CompiledBossPatternTransitionTarget::Outcome(0),
            },
        );

        assert!(matches!(
            runtime.resolve_loop_target(0, &transition).unwrap(),
            CompiledBossPatternTransitionTarget::ContinueState
        ));
        assert!(matches!(
            runtime.resolve_loop_target(0, &transition).unwrap(),
            CompiledBossPatternTransitionTarget::ContinueState
        ));
        assert!(matches!(
            runtime.resolve_loop_target(0, &transition).unwrap(),
            CompiledBossPatternTransitionTarget::Outcome(0)
        ));
    }

    #[test]
    fn after_pattern_phase_transition_waits_for_every_bound_actor() {
        let setup =
            fixture_setup_with_two_actors(BossPhaseTransitionTimingData::AfterCurrentPattern);
        let mut scenario = fixture_scenario_with_setup(Some(setup));
        scenario.groups[0].spawns.push(ScenarioUnitSpawn {
            unit_ref: ScenarioUnitRef::new("boss_1"),
            side: Side::Opponent,
            draft: fixture_draft(4, 1_000, 1_000),
            position: Position::new(6, 2),
            instance_salt: 1,
        });
        let mut core =
            BattleCore::new_from_scenario(scenario, GameDataBuilder::empty().build_arc(), 17);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_ids = core
            .boss_runtime
            .as_ref()
            .unwrap()
            .actors
            .iter()
            .map(|actor| actor.unit_id.unwrap())
            .collect::<Vec<_>>();
        core.units
            .get_mut(&actor_ids[0])
            .unwrap()
            .stats
            .current_health = 500;
        let first_target = core.boss_runtime.as_ref().unwrap().actors[0]
            .target_lock
            .as_ref()
            .unwrap()
            .target_id;
        let first_target_tile = core.units[&first_target].body.projected_tile();
        core.units
            .get_mut(&actor_ids[0])
            .unwrap()
            .set_world_position(WorldVec2::from_tile_center(Position::new(
                first_target_tile.x + 1,
                first_target_tile.y,
            )));

        core.run_boss_runtime_boundary(1).unwrap();
        let runtime = core.boss_runtime.as_ref().unwrap();
        assert_eq!(runtime.current_phase_index, 0);
        assert!(runtime.pending_phase_transition.is_some());
        assert!(runtime.actors[0].waiting_for_phase_transition);
        assert!(!runtime.actors[1].waiting_for_phase_transition);

        let second_target = runtime.actors[1].target_lock.as_ref().unwrap().target_id;
        assert_eq!(core.units[&second_target].body.projected_tile().x, 1);
        core.units
            .get_mut(&actor_ids[1])
            .unwrap()
            .set_world_position(WorldVec2::from_tile_center(Position::new(2, 1)));
        core.run_boss_runtime_boundary(2).unwrap();
        let runtime = core.boss_runtime.as_ref().unwrap();
        assert_eq!(runtime.current_phase_index, 1);
        assert!(runtime.pending_phase_transition.is_none());
        assert!(runtime
            .actors
            .iter()
            .all(|actor| !actor.waiting_for_phase_transition));
    }

    #[test]
    fn route_cursor_advances_monotonically_and_never_follows_backward_relocation() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_id = actor_runtime(&core).unit_id.unwrap();
        let locked = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        core.units.get_mut(&locked).unwrap().lifecycle =
            crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
        core.run_boss_runtime_boundary(1).unwrap();

        core.units
            .get_mut(&actor_id)
            .unwrap()
            .set_world_position(WorldVec2::from_tile_center(Position::new(3, 1)));
        core.run_boss_runtime_boundary(2).unwrap();
        assert_eq!(actor_runtime(&core).route_cursor_index, 3);

        core.units
            .get_mut(&actor_id)
            .unwrap()
            .set_world_position(WorldVec2::from_tile_center(Position::new(5, 1)));
        core.run_boss_runtime_boundary(3).unwrap();
        assert_eq!(actor_runtime(&core).route_cursor_index, 3);
    }

    #[test]
    fn move_and_rejoin_run_through_the_existing_continuous_movement_leaf() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_id = actor_runtime(&core).unit_id.unwrap();
        let start = core.units[&actor_id].body.position;

        core.step_battle_execution_until(&mut execution, 4_500)
            .unwrap();

        let actor = actor_runtime(&core);
        let control = actor.control.as_ref().expect("route control");
        assert!(control.pattern.is_none());
        assert!(matches!(
            &actor.behavior.as_ref().unwrap().nodes[control.node_index],
            CompiledBossControlNode::State { id, .. } if id.as_str() == "route_combat"
        ));
        assert!(core.units[&actor_id].body.position.x < start.x);
        assert!(actor.route_cursor_index > 0);
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::MovementSegmentStarted {
                unit_instance_id,
                ..
            } if unit_instance_id == actor_id
        )));
    }

    #[test]
    fn movement_lock_stalls_the_leaf_until_the_authored_timeout() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_id = actor_runtime(&core).unit_id.unwrap();
        let stalled_at = core.units[&actor_id].body.position;
        core.units
            .get_mut(&actor_id)
            .unwrap()
            .action_locks
            .lock_movement_until(6_000);

        core.step_battle_execution_until(&mut execution, 4_999)
            .unwrap();
        assert_eq!(core.units[&actor_id].body.position, stalled_at);
        assert!(actor_runtime(&core)
            .active_state()
            .unwrap()
            .tasks
            .iter()
            .any(|task| {
                matches!(task.kind, ActiveBossTaskKind::MoveToTarget { .. })
                    && task.result.is_none()
            }));

        core.step_battle_execution_until(&mut execution, 5_000)
            .unwrap();
        let actor = actor_runtime(&core);
        let control = actor.control.as_ref().expect("timeout recovery route");
        assert!(control.pattern.is_none());
        assert!(matches!(
            &actor.behavior.as_ref().unwrap().nodes[control.node_index],
            CompiledBossControlNode::State { id, .. } if id.as_str() == "route_combat"
        ));
        assert_eq!(core.units[&actor_id].body.position, stalled_at);
    }

    #[test]
    fn cancellation_drops_unreleased_attack_but_preserves_released_projectile() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_id = actor_runtime(&core).unit_id.unwrap();
        let locked = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        core.units.get_mut(&locked).unwrap().lifecycle =
            crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
        core.run_boss_runtime_boundary(1).unwrap();
        let target_id = core
            .units
            .values()
            .find(|unit| unit.owner == Side::Player && unit.is_active())
            .unwrap()
            .instance_id;
        let target_position = core.units[&target_id].body.position;
        let actor = core.units.get_mut(&actor_id).unwrap();
        actor.set_world_position(WorldVec2::new(target_position.x + 1.0, target_position.y));
        actor.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
        actor.basic_attack.delivery = DeliveryDef::Projectile {
            speed_units_per_ms: 1_000_000,
            hit_policy: ProjectileHitPolicy::TargetLocked,
            allow_targetless_cast: false,
            max_range_tiles: None,
            max_lifetime_ms: None,
            max_kills: None,
            max_pierces: None,
            collision: Default::default(),
        };
        actor.next_basic_attack_ms = 0;
        actor.current_target = Some(target_id);
        let target_health = core.units[&target_id].stats.current_health;
        assert!(core.boss_runtime_allows_combat_action(actor_id));
        assert!(core.units[&actor_id].can_basic_attack());
        assert_eq!(
            core.select_basic_attack_target(actor_id, Some(target_id), None),
            Some(target_id)
        );
        core.handle_basic_attack_start_event(
            10,
            10,
            actor_id,
            Some(target_id),
            true,
            BattleEventCause::Root {
                kind: BattleEventRootCause::Period,
            },
        );
        let unresolved = core
            .event_queue
            .iter()
            .find(|event| {
                matches!(
                    event.as_ref(),
                    BattleEvent::AttackResolve {
                        attacker_instance_id,
                        ..
                    } if *attacker_instance_id == actor_id
                )
            })
            .map(std::borrow::Cow::into_owned)
            .expect("unreleased attack callback");
        core.cancel_boss_owned_work(10, actor_id, true, true);
        assert_eq!(core.units[&actor_id].action_locks.basic_attack_until_ms, 10);
        assert_eq!(core.units[&actor_id].action_locks.movement_until_ms, 10);
        let BattleEvent::AttackResolve {
            time_ms,
            attacker_instance_id,
            target_instance_id,
            attack_epoch,
            source_snapshot,
            kind,
            delivery,
            cause,
        } = unresolved
        else {
            unreachable!()
        };
        core.handle_basic_attack_resolve_event(
            time_ms,
            attacker_instance_id,
            target_instance_id,
            attack_epoch,
            source_snapshot,
            kind,
            delivery,
            cause,
        );
        assert!(core.projectiles.is_empty());
        assert_eq!(core.units[&target_id].stats.current_health, target_health);

        assert!(core.resolve_basic_attack(actor_id, target_id, 20));
        let projectile_id = *core.projectiles.keys().next().expect("released projectile");
        let impact_time = {
            let projectile = &core.projectiles[&projectile_id];
            projectile
                .fired_at_ms
                .saturating_add(projectile.max_travel_ms)
        };
        core.cancel_boss_owned_work(20, actor_id, true, true);
        assert!(core.projectiles.contains_key(&projectile_id));
        core.advance_basic_attack_projectile(impact_time, projectile_id);
        assert!(core.units[&target_id].stats.current_health < target_health);
    }

    #[test]
    fn cancellation_preserves_focus_permitted_locks() {
        let skill_id = SkillId::from("focus_permitted_boss_skill");
        let skill = SkillDef {
            use_lifetime: crate::game::ability::SkillUseLifetime::Action,
            flow: None,
            id: skill_id.clone(),
            name: "Focus-permitted Boss skill".to_string(),
            kind: Default::default(),
            cast_targeting: SkillCastTargetingDef::explicit(
                SkillTarget::SelfUnit,
                SkillTargetReach::Pattern,
                None,
                false,
            ),
            focus_time_ms: 100,
            focus_permissions: FocusPermissions {
                allows_move: true,
                allows_basic_attack: true,
            },
            steps: vec![SkillStepDef {
                id: "noop".to_string(),
                delay_ms: 0,
                target: SkillTarget::SelfUnit,
                targeting: Default::default(),
                when: Default::default(),
                repeat: Default::default(),
                delivery: Default::default(),
                effects: Vec::new(),
                presentation: Default::default(),
            }],
        };
        let mut core = BattleCore::new_from_scenario(
            fixture_scenario(true),
            GameDataBuilder::empty()
                .with_skills(SkillDatabase::new(vec![skill]))
                .build_arc(),
            17,
        );
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_id = actor_runtime(&core).unit_id.unwrap();
        let cast_lock_until = 110;
        let actor = core.units.get_mut(&actor_id).unwrap();
        actor.pending_cast = true;
        actor.pending_skill_cast = Some(PendingSkillCast {
            start_gate_pending: false,
            reset_resonance: true,
            action: None,
            skill_id: skill_id.clone(),
            cast_target: None,
            accepted_target_tile: None,
            start_seq: 7,
        });
        actor.next_action_time = cast_lock_until;
        actor.action_locks.resonance_gain_until_ms = cast_lock_until;
        actor.action_locks.basic_attack_until_ms = cast_lock_until;
        actor.action_locks.movement_until_ms = cast_lock_until;

        core.cancel_boss_owned_work(10, actor_id, true, false);

        let actor = &core.units[&actor_id];
        assert!(!actor.pending_cast);
        assert!(actor.pending_skill_cast.is_none());
        assert_eq!(actor.next_action_time, 0);
        assert_eq!(actor.action_locks.resonance_gain_until_ms, 10);
        assert_eq!(actor.action_locks.basic_attack_until_ms, cast_lock_until);
        assert_eq!(actor.action_locks.movement_until_ms, cast_lock_until);
        assert!(core.event_log.entries.iter().any(|entry| matches!(
            &entry.event,
            BattleLogEvent::SkillCastInterrupted {
                caster_instance_id,
                interrupted_skill_id,
                interrupted_cast_seq: 7,
                ..
            } if *caster_instance_id == actor_id && interrupted_skill_id == &skill_id
        )));
    }

    #[test]
    fn target_invalidation_is_immediate_and_never_silently_retargets() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let locked = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        core.units.get_mut(&locked).unwrap().lifecycle =
            crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;

        core.run_boss_runtime_boundary(1).unwrap();

        let actor = actor_runtime(&core);
        assert!(actor.target_lock.is_none());
        let control = actor.control.as_ref().expect("authored outcome route");
        assert!(control.pattern.is_none());
        let behavior = actor.behavior.as_ref().unwrap();
        assert!(matches!(
            &behavior.nodes[control.node_index],
            CompiledBossControlNode::State { id, .. } if id.as_str() == "route_combat"
        ));
        assert_eq!(core.units[&actor.unit_id.unwrap()].current_target, None);
    }

    #[test]
    fn no_target_uses_the_authored_named_outcome_without_engine_fallback() {
        let mut core = fixture_core(true);
        core.spawn_scenario_group(&ScenarioGroupId::new("boss_group"))
            .unwrap();
        core.spawn_scenario_group(&ScenarioGroupId::new("player_group"))
            .unwrap();
        for unit in core.units.values_mut() {
            if unit.owner == Side::Player {
                unit.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
            }
        }

        core.run_boss_runtime_boundary(0).unwrap();

        let actor = actor_runtime(&core);
        assert!(actor.target_lock.is_none());
        let control = actor
            .control
            .as_ref()
            .expect("target-missing outcome route");
        assert!(control.pattern.is_none());
        assert!(matches!(
            &actor.behavior.as_ref().unwrap().nodes[control.node_index],
            CompiledBossControlNode::State { id, .. } if id.as_str() == "route_combat"
        ));
    }

    #[test]
    fn static_grid_query_rejects_unreachable_target_without_infinite_cost_fallback() {
        let mut core = fixture_core(true);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution(&mut execution).unwrap();
        let actor_id = actor_runtime(&core).unit_id.unwrap();
        let target_id = core
            .units
            .values()
            .find(|unit| unit.owner == Side::Player)
            .unwrap()
            .instance_id;
        for y in 0..4 {
            core.battlefield
                .add_static_obstacle(Position::new(4, y))
                .unwrap();
        }
        assert!(path_to_target_approach(&core, actor_id, target_id).is_none());
        let query = BossTargetQueryData {
            target_kind: BossTargetKindData::EnemyUnit,
            filters: vec![
                BossTargetFilterData::ActiveHostile,
                BossTargetFilterData::Reachable,
            ],
            order_by: vec![BossTargetOrderKeyData::MinPathCost],
            lock_policy: BossTargetLockPolicyData::LockUntilInvalid,
        };
        assert_eq!(select_target(&core, Some(actor_id), &query), None);
    }

    #[test]
    fn microstep_and_activation_overflow_leave_the_complete_core_unchanged() {
        let mut core = fixture_core(true);
        core.spawn_scenario_group(&ScenarioGroupId::new("boss_group"))
            .unwrap();
        core.spawn_scenario_group(&ScenarioGroupId::new("player_group"))
            .unwrap();
        core.boss_runtime.as_mut().unwrap().microstep_limit = 0;
        let before = core.transaction_debug_snapshot();
        assert!(matches!(
            core.run_boss_runtime_boundary(0),
            Err(GameError::BattleRuntimeFault(fault))
                if fault.code == crate::game::battle::runtime_control::RuntimeFaultCode::Invariant
                    && fault.site == "boss_runtime_boundary"
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);

        core.boss_runtime.as_mut().unwrap().microstep_limit = DEFAULT_BOSS_MICROSTEP_LIMIT;
        core.boss_runtime.as_mut().unwrap().next_activation_id = u64::MAX;
        let before = core.transaction_debug_snapshot();
        assert!(matches!(
            core.run_boss_runtime_boundary(0),
            Err(GameError::BattleRuntimeFault(fault))
                if fault.code == crate::game::battle::runtime_control::RuntimeFaultCode::Invariant
                    && fault.site == "boss_runtime_boundary"
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);

        core.boss_runtime.as_mut().unwrap().next_activation_id = 1;
        core.run_boss_runtime_boundary(0).unwrap();
        {
            let actor = &mut core.boss_runtime.as_mut().unwrap().actors[0];
            let mut behavior = actor.behavior.as_ref().unwrap().as_ref().clone();
            let CompiledBossControlNode::PatternCall { candidates, .. } = &mut behavior.nodes[0]
            else {
                unreachable!()
            };
            candidates[0].cooldown = None;
            actor.behavior = Some(Arc::new(behavior));
            actor.control = Some(ActiveBossControlNode {
                node_index: 0,
                activation_id: 50_000,
                resident_state: None,
                pattern: None,
            });
            actor.fixed_sequence_cursors.clear();
            actor.cooldown_ready_at_ms.clear();
            actor.target_lock = None;
        }
        let before = core.transaction_debug_snapshot();
        assert!(matches!(
            core.run_boss_runtime_boundary(u64::MAX),
            Err(GameError::BattleRuntimeFault(fault))
                if fault.code == crate::game::battle::runtime_control::RuntimeFaultCode::Invariant
                    && fault.site == "boss_runtime_boundary"
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);
    }
}
