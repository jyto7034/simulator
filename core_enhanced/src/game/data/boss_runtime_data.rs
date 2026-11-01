use std::{
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use super::{
    summoned_unit_data::{SummonedUnitDatabase, SummonedUnitDefinitionId},
    DataValidationError,
};

#[cfg(test)]
use super::summoned_unit_data::SummonedUnitCatalogData;

#[cfg(test)]
thread_local! {
    static BOSS_RUNTIME_COMPILE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

string_id!(BossPatternId);
string_id!(BossActorBehaviorId);
string_id!(BossEncounterRuntimeId);
string_id!(BossActorSlotId);
string_id!(BossEncounterPhaseId);
string_id!(BossControlNodeId);
string_id!(BossPatternStateId);
string_id!(BossPatternOutcomeId);
string_id!(BossTaskId);
string_id!(BossTransitionId);
string_id!(BossParameterId);
string_id!(BossCooldownGroupId);
string_id!(BossLoopId);
string_id!(BossSignalId);
string_id!(SummonBatchHandleId);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossRuntimeCatalogData {
    #[serde(default)]
    pub patterns: Vec<BossPatternDefinitionData>,
    #[serde(default)]
    pub actor_behaviors: Vec<BossActorBehaviorDefinitionData>,
    #[serde(default)]
    pub encounters: Vec<BossEncounterDefinitionData>,
    #[serde(default)]
    pub signals: Vec<BossSignalDefinitionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossSignalDefinitionData {
    pub id: BossSignalId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPatternDefinitionData {
    pub id: BossPatternId,
    #[serde(default)]
    pub parameters: Vec<BossParameterDefinitionData>,
    pub outcomes: Vec<BossPatternOutcomeId>,
    pub entry_state_id: BossPatternStateId,
    pub states: Vec<BossPatternStateData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossParameterDefinitionData {
    pub id: BossParameterId,
    pub parameter_type: BossParameterTypeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossParameterTypeData {
    DurationMs { min_ms: u64, max_ms: u64 },
    TargetQuery,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossParameterValueData {
    DurationMs(u64),
    TargetQuery(BossTargetQueryData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPatternArgumentData {
    pub parameter_id: BossParameterId,
    pub value: BossParameterValueData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossDurationValueData {
    Literal(u64),
    Parameter(BossParameterId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossTargetQueryValueData {
    Literal(BossTargetQueryData),
    Parameter(BossParameterId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossTargetQueryData {
    pub target_kind: BossTargetKindData,
    pub filters: Vec<BossTargetFilterData>,
    pub order_by: Vec<BossTargetOrderKeyData>,
    #[serde(default)]
    pub lock_policy: BossTargetLockPolicyData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BossTargetKindData {
    EnemyUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BossTargetFilterData {
    ActiveHostile,
    Grounded,
    Reachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BossTargetOrderKeyData {
    MinPathCost,
    MinHpRatio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BossTargetLockPolicyData {
    #[default]
    LockUntilInvalid,
    ReevaluateAtDecisionPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPatternStateData {
    pub id: BossPatternStateId,
    #[serde(default)]
    pub tasks: Vec<BossTaskData>,
    pub transitions: Vec<BossPatternTransitionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossTaskData {
    StartAction {
        id: BossTaskId,
        skill_id: crate::game::ability::SkillId,
        target: BossActionTargetData,
        facing: crate::game::battle::tile_range::FacingDirection,
        milestone: BossActionMilestoneData,
        timeout_ms: u64,
        on_timeout: BossActionTimeoutData,
    },

    RouteCombat {
        id: BossTaskId,
    },
    AcquireTarget {
        id: BossTaskId,
        query: BossTargetQueryValueData,
    },
    MoveToTarget {
        id: BossTaskId,
        timeout_ms: BossDurationValueData,
    },
    RejoinForwardRoute {
        id: BossTaskId,
        timeout_ms: BossDurationValueData,
    },
    Wait {
        id: BossTaskId,
        duration_ms: BossDurationValueData,
    },
    Summon {
        id: BossTaskId,
        definition_id: SummonedUnitDefinitionId,
        count: u32,
        batch_handle_id: SummonBatchHandleId,
        placement: BossSummonPlacementData,
        occupancy_policy: BossSummonOccupancyPolicyData,
        on_insufficient_space: BossSummonInsufficientSpaceData,
        route_id: Option<String>,
        lifetime: Option<BossSummonLifetimeData>,
        removal_signal: Option<BossSummonRemovalSignalData>,
    },
    EmitSignal {
        id: BossTaskId,
        signal_id: BossSignalId,
    },
    RepathToProtectedObjective {
        id: BossTaskId,
        timeout_ms: BossDurationValueData,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossActionTargetData {
    SelfUnit,
    LockedUnit,
    Tile {
        position: crate::game::resources::Position,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossActionMilestoneData {
    Released,
    EffectsSettled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossActionTimeoutData {
    CancelUnreleased,
    Detach,
}
crate::authoring::clone_shape::copy_shape!(
    BossActionTargetData,
    BossActionMilestoneData,
    BossActionTimeoutData
);

impl BossTaskData {
    pub fn id(&self) -> &BossTaskId {
        match self {
            Self::StartAction { id, .. }
            | Self::RouteCombat { id }
            | Self::AcquireTarget { id, .. }
            | Self::MoveToTarget { id, .. }
            | Self::RejoinForwardRoute { id, .. }
            | Self::Wait { id, .. }
            | Self::Summon { id, .. }
            | Self::EmitSignal { id, .. }
            | Self::RepathToProtectedObjective { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossSummonPlacementData {
    AroundSummoner { radius_tiles: u32 },
    SpawnZone { zone_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BossSummonOccupancyClassData {
    Summoner,
    SameFaction,
    OpposingCombatant,
    ProtectedObjective,
    StaticDefenseObject,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossSummonOccupancyPolicyData {
    RequireUnoccupied,
    AllowOccupied {
        allowed: Vec<BossSummonOccupancyClassData>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossSummonInsufficientSpaceData {
    Fail,
    WaitUntilAvailable { timeout_ms: BossDurationValueData },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossSummonRemovalModeData {
    Despawn,
    Kill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossSummonLifetimeData {
    pub lifetime_ms: BossDurationValueData,
    pub mode: BossSummonRemovalModeData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossSummonRemovalSignalData {
    pub signal_id: BossSignalId,
    pub mode: BossSummonRemovalModeData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BossTaskResultData {
    RequestRejected,
    Acquired,
    NoEligibleTarget,
    Reached,
    PathUnavailable,
    TargetInvalidated,
    TimedOut,
    Completed,
    Spawned,
    LimitReached,
    NoSpawnSpace,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPatternTransitionData {
    pub id: BossTransitionId,
    pub priority: i32,
    pub condition: BossConditionData,
    pub target: BossPatternTransitionTargetData,
    #[serde(default)]
    pub loop_guard: Option<BossLoopGuardData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossPatternTransitionTargetData {
    State(BossPatternStateId),
    Outcome(BossPatternOutcomeId),
    ContinueState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossLoopGuardData {
    pub id: BossLoopId,
    pub max_iterations: u32,
    pub exhausted_target: BossPatternTransitionTargetData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossConditionData {
    Always,
    TaskResult {
        task_id: BossTaskId,
        result: BossTaskResultData,
    },
    StatePredicate(BossStatePredicateData),
    EventOccurred {
        event: BossEventConditionData,
        memory: BossEventMemoryData,
    },
    SummonBatchCleared {
        batch_handle_id: SummonBatchHandleId,
    },
}

impl BossConditionData {
    fn is_immediate(&self) -> bool {
        matches!(self, Self::Always | Self::StatePredicate(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossStatePredicateData {
    SelfHealthAtOrBelowPercent { percent: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossEventConditionData {
    TargetInvalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossEventMemoryData {
    Pulse,
    UntilStateExit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossActorBehaviorDefinitionData {
    pub id: BossActorBehaviorId,
    pub entry_node_id: BossControlNodeId,
    pub nodes: Vec<BossControlNodeData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossControlNodeData {
    State {
        id: BossControlNodeId,
        #[serde(default)]
        tasks: Vec<BossTaskData>,
        transitions: Vec<BossControlTransitionData>,
    },
    PatternCall {
        id: BossControlNodeId,
        selector: BossPatternSelectorData,
        outcome_routes: Vec<BossPatternOutcomeRouteData>,
        no_eligible_node_id: BossControlNodeId,
    },
}

impl BossControlNodeData {
    pub fn id(&self) -> &BossControlNodeId {
        match self {
            Self::State { id, .. } | Self::PatternCall { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossControlTransitionData {
    pub id: BossTransitionId,
    pub priority: i32,
    pub condition: BossConditionData,
    pub target_node_id: BossControlNodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossPatternSelectorData {
    FixedSequence {
        candidates: Vec<BossPatternCandidateData>,
        repeat: bool,
    },
    PriorityEligible {
        candidates: Vec<BossPatternCandidateData>,
    },
    WeightedEligible {
        candidates: Vec<BossWeightedPatternCandidateData>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPatternCandidateData {
    pub pattern_id: BossPatternId,
    #[serde(default)]
    pub arguments: Vec<BossPatternArgumentData>,
    #[serde(default)]
    pub cooldown: Option<BossPatternCooldownData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossWeightedPatternCandidateData {
    pub candidate: BossPatternCandidateData,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPatternCooldownData {
    pub group_id: BossCooldownGroupId,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPatternOutcomeRouteData {
    pub pattern_id: BossPatternId,
    pub outcome_id: BossPatternOutcomeId,
    pub target_node_id: BossControlNodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossEncounterDefinitionData {
    pub id: BossEncounterRuntimeId,
    pub actor_slots: Vec<BossActorSlotId>,
    pub entry_phase_id: BossEncounterPhaseId,
    pub phases: Vec<BossEncounterPhaseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossEncounterPhaseData {
    pub id: BossEncounterPhaseId,
    pub actor_behaviors: Vec<BossPhaseActorBehaviorData>,
    #[serde(default)]
    pub transitions: Vec<BossEncounterPhaseTransitionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossPhaseActorBehaviorData {
    pub actor_slot_id: BossActorSlotId,
    pub behavior_id: BossActorBehaviorId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossEncounterPhaseTransitionData {
    pub id: BossTransitionId,
    pub priority: i32,
    pub condition: BossEncounterConditionData,
    pub timing: BossPhaseTransitionTimingData,
    pub target_phase_id: BossEncounterPhaseId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BossEncounterConditionData {
    Always,
    ActorHealthAtOrBelowPercent {
        actor_slot_id: BossActorSlotId,
        percent: u32,
    },
    ActorDefeated {
        actor_slot_id: BossActorSlotId,
    },
}

impl BossEncounterConditionData {
    fn is_immediate(&self) -> bool {
        // Every v1 encounter condition is a level predicate. Health and defeated
        // facts can already be true at phase entry, so none of them proves that
        // authoritative battle time advances before the next phase edge.
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossPhaseTransitionTimingData {
    Immediate,
    AfterCurrentState,
    AfterCurrentPattern,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledBossPatternDefinition {
    pub id: BossPatternId,
    pub parameters: Vec<CompiledBossParameterDefinition>,
    pub outcomes: Vec<BossPatternOutcomeId>,
    pub entry_state_index: usize,
    pub states: Vec<CompiledBossPatternState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledBossParameterDefinition {
    pub id: BossParameterId,
    pub parameter_type: BossParameterTypeData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ResolvedBossParameterValue {
    DurationMs(u64),
    TargetQuery(BossTargetQueryData),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedBossArgumentFrame {
    pub values: Vec<ResolvedBossParameterValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledBossPatternState {
    pub id: BossPatternStateId,
    pub tasks: Vec<CompiledBossTask>,
    pub transitions: Vec<CompiledBossPatternTransition>,
}

#[derive(Debug, Clone, Serialize)]
pub enum CompiledBossTask {
    StartAction {
        id: BossTaskId,
        skill_id: crate::game::ability::SkillId,
        target: BossActionTargetData,
        facing: crate::game::battle::tile_range::FacingDirection,
        milestone: BossActionMilestoneData,
        timeout_ms: u64,
        on_timeout: BossActionTimeoutData,
    },

    RouteCombat {
        id: BossTaskId,
    },
    AcquireTarget {
        id: BossTaskId,
        query: CompiledBossTargetQueryValue,
    },
    MoveToTarget {
        id: BossTaskId,
        timeout_ms: CompiledBossDurationValue,
    },
    RejoinForwardRoute {
        id: BossTaskId,
        timeout_ms: CompiledBossDurationValue,
    },
    Wait {
        id: BossTaskId,
        duration_ms: CompiledBossDurationValue,
    },
    Summon {
        id: BossTaskId,
        definition_id: SummonedUnitDefinitionId,
        count: u32,
        batch_handle_id: SummonBatchHandleId,
        placement: BossSummonPlacementData,
        occupancy_policy: BossSummonOccupancyPolicyData,
        on_insufficient_space: CompiledBossSummonInsufficientSpace,
        route_id: Option<String>,
        lifetime: Option<CompiledBossSummonLifetime>,
        removal_signal: Option<BossSummonRemovalSignalData>,
    },
    EmitSignal {
        id: BossTaskId,
        signal_id: BossSignalId,
    },
}

#[derive(Debug, Clone, Serialize)]
pub enum CompiledBossSummonInsufficientSpace {
    Fail,
    WaitUntilAvailable {
        timeout_ms: CompiledBossDurationValue,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledBossSummonLifetime {
    pub lifetime_ms: CompiledBossDurationValue,
    pub mode: BossSummonRemovalModeData,
}

#[derive(Debug, Clone, Serialize)]
pub enum CompiledBossDurationValue {
    Literal(u64),
    Parameter(usize),
}

#[derive(Debug, Clone, Serialize)]
pub enum CompiledBossTargetQueryValue {
    Literal(BossTargetQueryData),
    Parameter(usize),
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledBossPatternTransition {
    pub id: BossTransitionId,
    pub priority: i32,
    pub condition: BossConditionData,
    pub target: CompiledBossPatternTransitionTarget,
    pub loop_guard: Option<CompiledBossLoopGuard>,
}

#[derive(Debug, Clone, Serialize)]
pub enum CompiledBossPatternTransitionTarget {
    State(usize),
    Outcome(usize),
    ContinueState,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledBossLoopGuard {
    pub id: BossLoopId,
    pub max_iterations: u32,
    pub exhausted_target: CompiledBossPatternTransitionTarget,
}

#[derive(Debug, Clone)]
pub struct CompiledBossActorBehaviorDefinition {
    pub id: BossActorBehaviorId,
    pub entry_node_index: usize,
    pub nodes: Vec<CompiledBossControlNode>,
}

impl CompiledBossActorBehaviorDefinition {
    pub fn requires_authored_route(&self) -> bool {
        self.nodes.iter().any(|node| match node {
            CompiledBossControlNode::State { tasks, .. } => {
                tasks.iter().any(compiled_task_requires_authored_route)
            }
            CompiledBossControlNode::PatternCall { candidates, .. } => {
                candidates.iter().any(|candidate| {
                    candidate.pattern.states.iter().any(|state| {
                        state
                            .tasks
                            .iter()
                            .any(compiled_task_requires_authored_route)
                    })
                })
            }
        })
    }
}

fn compiled_task_requires_authored_route(task: &CompiledBossTask) -> bool {
    matches!(
        task,
        CompiledBossTask::RouteCombat { .. } | CompiledBossTask::RejoinForwardRoute { .. }
    )
}

#[derive(Debug, Clone)]
pub enum CompiledBossControlNode {
    State {
        id: BossControlNodeId,
        tasks: Vec<CompiledBossTask>,
        transitions: Vec<CompiledBossControlTransition>,
    },
    PatternCall {
        id: BossControlNodeId,
        candidates: Vec<CompiledBossPatternCandidate>,
        repeat: bool,
        outcome_routes: Vec<CompiledBossPatternOutcomeRoute>,
        no_eligible_node_index: usize,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledBossControlTransition {
    pub id: BossTransitionId,
    pub priority: i32,
    pub condition: BossConditionData,
    pub target_node_index: usize,
}

#[derive(Debug, Clone)]
pub struct CompiledBossPatternCandidate {
    pub pattern: Arc<CompiledBossPatternDefinition>,
    pub argument_frame: Arc<ResolvedBossArgumentFrame>,
    pub cooldown: Option<BossPatternCooldownData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledBossPatternOutcomeRoute {
    pub pattern_id: BossPatternId,
    pub outcome_index: usize,
    pub target_node_index: usize,
}

#[derive(Debug, Clone)]
pub struct CompiledBossEncounterDefinition {
    pub id: BossEncounterRuntimeId,
    pub actor_slots: Vec<BossActorSlotId>,
    pub entry_phase_index: usize,
    pub phases: Vec<CompiledBossEncounterPhase>,
}

impl CompiledBossEncounterDefinition {
    pub fn actor_slot_requires_authored_route(&self, actor_slot_index: usize) -> bool {
        self.phases.iter().any(|phase| {
            phase.actor_behaviors.iter().any(|binding| {
                binding.actor_slot_index == actor_slot_index
                    && binding.behavior.requires_authored_route()
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct CompiledBossEncounterPhase {
    pub id: BossEncounterPhaseId,
    pub actor_behaviors: Vec<CompiledBossPhaseActorBehavior>,
    pub transitions: Vec<CompiledBossEncounterPhaseTransition>,
}

#[derive(Debug, Clone)]
pub struct CompiledBossPhaseActorBehavior {
    pub actor_slot_index: usize,
    pub behavior: Arc<CompiledBossActorBehaviorDefinition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledBossEncounterPhaseTransition {
    pub id: BossTransitionId,
    pub priority: i32,
    pub condition: BossEncounterConditionData,
    pub timing: BossPhaseTransitionTimingData,
    pub target_phase_index: usize,
}

#[derive(Debug, Clone)]
pub struct ResolvedBossEncounterSetup {
    pub definition: Arc<CompiledBossEncounterDefinition>,
    pub actors: Vec<ResolvedBossActorSetup>,
    pub enemy_stat_scale: crate::game::battle::types::BattleUnitStatScale,
    pub summon_spawn_zones: Vec<ResolvedBossSummonSpawnZone>,
    pub summon_routes: Vec<ResolvedBossSummonRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBossActorSetup {
    pub actor_slot_index: usize,
    pub unit_ref: crate::game::battle::scenario::ScenarioUnitRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBossSummonSpawnZone {
    pub id: String,
    pub cells: Vec<crate::game::resources::Position>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBossSummonRoute {
    pub id: String,
    pub cells: Vec<crate::game::resources::Position>,
}

crate::authoring::clone_shape::struct_shape!(ResolvedBossEncounterSetup {
    definition,
    actors,
    enemy_stat_scale,
    summon_spawn_zones,
    summon_routes,
});
crate::authoring::clone_shape::struct_shape!(ResolvedBossActorSetup {
    actor_slot_index,
    unit_ref
});
crate::authoring::clone_shape::struct_shape!(ResolvedBossSummonSpawnZone { id, cells });
crate::authoring::clone_shape::struct_shape!(ResolvedBossSummonRoute { id, cells });
crate::authoring::clone_shape::newtype_shape!(BossActorSlotId, BossSignalId);

crate::authoring::clone_shape::newtype_shape!(BossCooldownGroupId, SummonBatchHandleId, BossTaskId);
crate::authoring::clone_shape::copy_shape!(
    BossPhaseTransitionTimingData,
    BossTaskResultData,
    BossTargetLockPolicyData,
    BossSummonRemovalModeData,
    BossSummonOccupancyClassData
);
crate::authoring::clone_shape::struct_shape!(BossSummonRemovalSignalData { signal_id, mode });
crate::authoring::clone_shape::enum_shape!(BossSummonPlacementData {
    units: [],
    tuples: [],
    records: [AroundSummoner { radius_tiles }, SpawnZone { zone_id }]
});
crate::authoring::clone_shape::enum_shape!(BossSummonOccupancyPolicyData {
    units: [RequireUnoccupied],
    tuples: [],
    records: [AllowOccupied { allowed }]
});

#[derive(Debug, Clone, Default)]
pub struct BossRuntimeDatabase {
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    authored: BossRuntimeCatalogData,
    patterns: Vec<Arc<CompiledBossPatternDefinition>>,
    actor_behaviors: Vec<Arc<CompiledBossActorBehaviorDefinition>>,
    encounters: Vec<Arc<CompiledBossEncounterDefinition>>,
    pattern_by_id: HashMap<BossPatternId, usize>,
    actor_behavior_by_id: HashMap<BossActorBehaviorId, usize>,
    encounter_by_id: HashMap<BossEncounterRuntimeId, usize>,
    summoned_units: SummonedUnitDatabase,
    signal_ids: HashSet<BossSignalId>,
}

impl BossRuntimeDatabase {
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub(crate) fn authored_catalogs_for_fixture_identity(
        &self,
    ) -> (
        &BossRuntimeCatalogData,
        &super::summoned_unit_data::SummonedUnitCatalogData,
    ) {
        (
            &self.authored,
            self.summoned_units.authored_for_fixture_identity(),
        )
    }

    pub fn get_pattern(&self, id: &BossPatternId) -> Option<&Arc<CompiledBossPatternDefinition>> {
        self.pattern_by_id
            .get(id)
            .and_then(|index| self.patterns.get(*index))
    }

    pub fn get_actor_behavior(
        &self,
        id: &BossActorBehaviorId,
    ) -> Option<&Arc<CompiledBossActorBehaviorDefinition>> {
        self.actor_behavior_by_id
            .get(id)
            .and_then(|index| self.actor_behaviors.get(*index))
    }

    pub fn get_encounter(
        &self,
        id: &BossEncounterRuntimeId,
    ) -> Option<&Arc<CompiledBossEncounterDefinition>> {
        self.encounter_by_id
            .get(id)
            .and_then(|index| self.encounters.get(*index))
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn actor_behavior_count(&self) -> usize {
        self.actor_behaviors.len()
    }

    pub fn encounter_count(&self) -> usize {
        self.encounters.len()
    }

    pub fn get_summoned_unit(
        &self,
        id: &SummonedUnitDefinitionId,
    ) -> Option<&Arc<super::summoned_unit_data::CompiledSummonedUnitDefinition>> {
        self.summoned_units.get_by_id(id)
    }

    pub fn independent_objects(
        &self,
    ) -> &super::independent_object_data::IndependentObjectDatabase {
        &self.summoned_units.objects
    }

    pub fn summoned_unit_count(&self) -> usize {
        self.summoned_units.len()
    }

    pub fn declares_signal(&self, id: &BossSignalId) -> bool {
        self.signal_ids.contains(id)
    }

    pub(crate) fn get_summoned_unit_skill_validation(
        &self,
        skills: &super::skill_data::SkillDatabase,
    ) -> Result<(), DataValidationError> {
        self.summoned_units.validate_skill_references(skills)?;
        let validate = |tasks: &[CompiledBossTask],
                        owner: &str|
         -> Result<(), DataValidationError> {
            for task in tasks {
                if let CompiledBossTask::StartAction {
                    id,
                    skill_id,
                    target,
                    ..
                } = task
                {
                    let skill = skills.get_by_id(skill_id.as_str()).ok_or_else(|| {
                        boss_error(
                            owner,
                            format!("tasks.{}.skill_id", id.as_str()),
                            "unknown StartAction skill",
                        )
                    })?;
                    use crate::game::ability::SkillCastSelection;
                    let compatible = matches!(
                        (skill.cast_selection(), target),
                        (SkillCastSelection::SelfUnit, BossActionTargetData::SelfUnit)
                            | (
                                SkillCastSelection::AutomaticUnit | SkillCastSelection::PlayerUnit,
                                BossActionTargetData::LockedUnit
                            )
                            | (
                                SkillCastSelection::PlayerTile,
                                BossActionTargetData::Tile { .. }
                            )
                    );
                    if !compatible {
                        return Err(boss_error(
                            owner,
                            format!("tasks.{}.target", id.as_str()),
                            "StartAction target does not match skill selection",
                        ));
                    }
                }
            }
            Ok(())
        };
        for pattern in &self.patterns {
            for state in &pattern.states {
                validate(&state.tasks, pattern.id.as_str())?;
            }
        }
        for behavior in &self.actor_behaviors {
            for node in &behavior.nodes {
                if let CompiledBossControlNode::State { tasks, .. } = node {
                    validate(tasks, behavior.id.as_str())?;
                }
            }
        }
        Ok(())
    }
}

fn boss_error(
    object_id: impl Into<String>,
    field_path: impl Into<String>,
    cause: impl Into<String>,
) -> DataValidationError {
    DataValidationError::at(object_id, field_path, cause)
}

fn validate_stable_id(id: &str, object_id: &str, path: &str) -> Result<(), DataValidationError> {
    let valid = !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && id
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    if !valid {
        return Err(boss_error(
            object_id,
            path,
            format!("stable id '{id}' must be non-empty lower_snake_case"),
        ));
    }
    Ok(())
}

fn unique_index<I>(
    values: I,
    object_id: &str,
    path: &str,
) -> Result<HashMap<String, usize>, DataValidationError>
where
    I: IntoIterator<Item = String>,
{
    unique_index_with(values, object_id, path, validate_stable_id)
}

fn validate_catalog_id(id: &str, object_id: &str, path: &str) -> Result<(), DataValidationError> {
    let (_, local) = super::content_pack::canonical_key_parts(id).ok_or_else(|| {
        boss_error(
            object_id,
            path,
            "invalid global or namespace::local catalog ID",
        )
    })?;
    validate_stable_id(local, object_id, path)
}

fn unique_catalog_index<I>(
    values: I,
    object_id: &str,
    path: &str,
) -> Result<HashMap<String, usize>, DataValidationError>
where
    I: IntoIterator<Item = String>,
{
    unique_index_with(values, object_id, path, validate_catalog_id)
}

fn unique_index_with<I>(
    values: I,
    object_id: &str,
    path: &str,
    validate: fn(&str, &str, &str) -> Result<(), DataValidationError>,
) -> Result<HashMap<String, usize>, DataValidationError>
where
    I: IntoIterator<Item = String>,
{
    let mut result = HashMap::new();
    for (index, value) in values.into_iter().enumerate() {
        validate(&value, object_id, &format!("{path}[{index}].id"))?;
        if result.insert(value.clone(), index).is_some() {
            return Err(boss_error(
                object_id,
                format!("{path}[{index}].id"),
                format!("duplicate id '{value}'"),
            ));
        }
    }
    Ok(result)
}

fn has_cycle(adjacency: &[Vec<usize>]) -> bool {
    fn visit(node: usize, adjacency: &[Vec<usize>], marks: &mut [u8]) -> bool {
        if marks[node] == 1 {
            return true;
        }
        if marks[node] == 2 {
            return false;
        }
        marks[node] = 1;
        for &next in &adjacency[node] {
            if visit(next, adjacency, marks) {
                return true;
            }
        }
        marks[node] = 2;
        false
    }

    let mut marks = vec![0; adjacency.len()];
    (0..adjacency.len()).any(|node| marks[node] == 0 && visit(node, adjacency, &mut marks))
}

fn reachable_from(entry: usize, adjacency: &[Vec<usize>]) -> BTreeSet<usize> {
    let mut reached = BTreeSet::new();
    let mut pending = VecDeque::from([entry]);
    while let Some(node) = pending.pop_front() {
        if !reached.insert(node) {
            continue;
        }
        pending.extend(adjacency[node].iter().copied());
    }
    reached
}

fn task_result_set(task: &BossTaskData) -> Option<BTreeSet<BossTaskResultData>> {
    Some(match task {
        BossTaskData::StartAction { .. } => [
            BossTaskResultData::Completed,
            BossTaskResultData::RequestRejected,
            BossTaskResultData::TargetInvalidated,
            BossTaskResultData::TimedOut,
            BossTaskResultData::Canceled,
        ]
        .into_iter()
        .collect(),
        BossTaskData::RouteCombat { .. } => return None,
        BossTaskData::AcquireTarget { .. } => [
            BossTaskResultData::Acquired,
            BossTaskResultData::NoEligibleTarget,
        ]
        .into_iter()
        .collect(),
        BossTaskData::MoveToTarget { .. } => [
            BossTaskResultData::Reached,
            BossTaskResultData::PathUnavailable,
            BossTaskResultData::TargetInvalidated,
            BossTaskResultData::TimedOut,
        ]
        .into_iter()
        .collect(),
        BossTaskData::RejoinForwardRoute { .. }
        | BossTaskData::RepathToProtectedObjective { .. } => [
            BossTaskResultData::Reached,
            BossTaskResultData::PathUnavailable,
            BossTaskResultData::TimedOut,
        ]
        .into_iter()
        .collect(),
        BossTaskData::Wait { .. } => [BossTaskResultData::Completed].into_iter().collect(),
        BossTaskData::Summon { .. } => [
            BossTaskResultData::Spawned,
            BossTaskResultData::LimitReached,
            BossTaskResultData::NoSpawnSpace,
            BossTaskResultData::Canceled,
        ]
        .into_iter()
        .collect(),
        BossTaskData::EmitSignal { .. } => [BossTaskResultData::Completed].into_iter().collect(),
    })
}

fn task_capabilities(task: &BossTaskData) -> &'static [&'static str] {
    match task {
        BossTaskData::StartAction { .. } => &["movement", "combat_action"],
        BossTaskData::RouteCombat { .. } => &["movement", "combat_action"],
        BossTaskData::AcquireTarget { .. } => &["target_lock"],
        BossTaskData::MoveToTarget { .. }
        | BossTaskData::RejoinForwardRoute { .. }
        | BossTaskData::RepathToProtectedObjective { .. } => &["movement"],
        BossTaskData::Wait { .. }
        | BossTaskData::Summon { .. }
        | BossTaskData::EmitSignal { .. } => &[],
    }
}

fn validate_query(
    query: &BossTargetQueryData,
    object_id: &str,
    path: &str,
) -> Result<(), DataValidationError> {
    if query.order_by.is_empty() {
        return Err(boss_error(
            object_id,
            format!("{path}.order_by"),
            "target query must define at least one lexicographic order key",
        ));
    }
    let filter_set = query.filters.iter().copied().collect::<HashSet<_>>();
    if filter_set.len() != query.filters.len() {
        return Err(boss_error(
            object_id,
            format!("{path}.filters"),
            "target query filters must be unique",
        ));
    }
    let order_set = query.order_by.iter().copied().collect::<HashSet<_>>();
    if order_set.len() != query.order_by.len() {
        return Err(boss_error(
            object_id,
            format!("{path}.order_by"),
            "target query order keys must be unique",
        ));
    }
    if query
        .order_by
        .contains(&BossTargetOrderKeyData::MinPathCost)
        && !filter_set.contains(&BossTargetFilterData::Reachable)
    {
        return Err(boss_error(
            object_id,
            format!("{path}.filters"),
            "MinPathCost requires the explicit Reachable filter",
        ));
    }
    Ok(())
}

fn validate_condition(
    condition: &BossConditionData,
    task_results: &HashMap<BossTaskId, BTreeSet<BossTaskResultData>>,
    object_id: &str,
    path: &str,
) -> Result<(), DataValidationError> {
    match condition {
        BossConditionData::TaskResult { task_id, result } => {
            let Some(results) = task_results.get(task_id) else {
                return Err(boss_error(
                    object_id,
                    path,
                    format!(
                        "references missing or persistent task '{}'",
                        task_id.as_str()
                    ),
                ));
            };
            if !results.contains(result) {
                return Err(boss_error(
                    object_id,
                    path,
                    format!(
                        "result {result:?} is invalid for task '{}'",
                        task_id.as_str()
                    ),
                ));
            }
        }
        BossConditionData::StatePredicate(BossStatePredicateData::SelfHealthAtOrBelowPercent {
            percent,
        }) if *percent == 0 || *percent > 100 => {
            return Err(boss_error(
                object_id,
                path,
                "health percent must be in 1..=100",
            ));
        }
        _ => {}
    }
    Ok(())
}

fn validate_tasks_and_result_coverage<'a>(
    tasks: &'a [BossTaskData],
    conditions: impl IntoIterator<Item = &'a BossConditionData>,
    parameter_types: &HashMap<BossParameterId, BossParameterTypeData>,
    object_id: &str,
    path: &str,
) -> Result<(), DataValidationError> {
    let mut task_ids = HashSet::new();
    let mut capability_owner: HashMap<&str, &BossTaskId> = HashMap::new();
    let mut task_results = HashMap::new();
    for (index, task) in tasks.iter().enumerate() {
        validate_stable_id(
            task.id().as_str(),
            object_id,
            &format!("{path}.tasks[{index}].id"),
        )?;
        if !task_ids.insert(task.id().clone()) {
            return Err(boss_error(
                object_id,
                format!("{path}.tasks[{index}].id"),
                format!("duplicate task id '{}'", task.id().as_str()),
            ));
        }
        if matches!(task, BossTaskData::RepathToProtectedObjective { .. }) {
            return Err(boss_error(
                object_id,
                format!("{path}.tasks[{index}]"),
                "RepathToProtectedObjective is known future vocabulary but is not implemented in v1",
            ));
        }
        for capability in task_capabilities(task) {
            if let Some(previous) = capability_owner.insert(capability, task.id()) {
                return Err(boss_error(
                    object_id,
                    format!("{path}.tasks[{index}]"),
                    format!(
                        "task '{}' conflicts with task '{}' on intrinsic capability '{capability}'",
                        task.id().as_str(),
                        previous.as_str()
                    ),
                ));
            }
        }
        validate_task_values(
            task,
            parameter_types,
            object_id,
            &format!("{path}.tasks[{index}]"),
        )?;
        if let Some(results) = task_result_set(task) {
            task_results.insert(task.id().clone(), results);
        }
    }

    let mut handled: HashMap<BossTaskId, BTreeSet<BossTaskResultData>> = HashMap::new();
    for (index, condition) in conditions.into_iter().enumerate() {
        validate_condition(
            condition,
            &task_results,
            object_id,
            &format!("{path}.transitions[{index}].condition"),
        )?;
        if let BossConditionData::TaskResult { task_id, result } = condition {
            if !handled.entry(task_id.clone()).or_default().insert(*result) {
                return Err(boss_error(
                    object_id,
                    format!("{path}.transitions[{index}].condition"),
                    format!(
                        "duplicate result arm {result:?} for task '{}'",
                        task_id.as_str()
                    ),
                ));
            }
        }
    }
    for (task_id, expected) in task_results {
        let actual = handled.remove(&task_id).unwrap_or_default();
        if actual != expected {
            return Err(boss_error(
                object_id,
                format!("{path}.transitions"),
                format!(
                    "task '{}' result mapping is not exhaustive: expected {expected:?}, got {actual:?}",
                    task_id.as_str()
                ),
            ));
        }
    }
    Ok(())
}

fn validate_task_values(
    task: &BossTaskData,
    parameter_types: &HashMap<BossParameterId, BossParameterTypeData>,
    object_id: &str,
    path: &str,
) -> Result<(), DataValidationError> {
    match task {
        BossTaskData::StartAction {
            skill_id,
            timeout_ms,
            ..
        } => {
            if skill_id.as_str().trim().is_empty()
                || *timeout_ms == 0
                || *timeout_ms > u32::MAX as u64
            {
                return Err(boss_error(
                    object_id,
                    path,
                    "StartAction requires a skill and timeout in 1..=u32::MAX",
                ));
            }
            Ok(())
        }
        BossTaskData::AcquireTarget { query, .. } => match query {
            BossTargetQueryValueData::Literal(query) => validate_query(query, object_id, path),
            BossTargetQueryValueData::Parameter(parameter_id) => require_parameter_type(
                parameter_id,
                parameter_types,
                |kind| matches!(kind, BossParameterTypeData::TargetQuery),
                "TargetQuery",
                object_id,
                path,
            ),
        },
        BossTaskData::MoveToTarget { timeout_ms, .. }
        | BossTaskData::RejoinForwardRoute { timeout_ms, .. }
        | BossTaskData::RepathToProtectedObjective { timeout_ms, .. } => validate_duration_value(
            timeout_ms,
            parameter_types,
            object_id,
            &format!("{path}.timeout_ms"),
        ),
        BossTaskData::Wait { duration_ms, .. } => validate_duration_value(
            duration_ms,
            parameter_types,
            object_id,
            &format!("{path}.duration_ms"),
        ),
        BossTaskData::Summon {
            count,
            placement,
            occupancy_policy,
            on_insufficient_space,
            route_id,
            lifetime,
            ..
        } => {
            if *count == 0 {
                return Err(boss_error(
                    object_id,
                    format!("{path}.count"),
                    "count must be greater than zero",
                ));
            }
            match placement {
                BossSummonPlacementData::AroundSummoner { radius_tiles: 0 } => {
                    return Err(boss_error(
                        object_id,
                        format!("{path}.placement.radius_tiles"),
                        "radius_tiles must be greater than zero",
                    ));
                }
                BossSummonPlacementData::SpawnZone { zone_id } if zone_id.trim().is_empty() => {
                    return Err(boss_error(
                        object_id,
                        format!("{path}.placement.zone_id"),
                        "spawn zone ID must not be empty",
                    ));
                }
                _ => {}
            }
            validate_summon_occupancy(occupancy_policy, object_id, path)?;
            if let BossSummonInsufficientSpaceData::WaitUntilAvailable { timeout_ms } =
                on_insufficient_space
            {
                validate_duration_value(
                    timeout_ms,
                    parameter_types,
                    object_id,
                    &format!("{path}.on_insufficient_space.timeout_ms"),
                )?;
                if matches!(
                    occupancy_policy,
                    BossSummonOccupancyPolicyData::AllowOccupied { .. }
                ) {
                    return Err(boss_error(
                        object_id,
                        format!("{path}.on_insufficient_space"),
                        "AllowOccupied cannot use WaitUntilAvailable because candidate capacity cannot increase",
                    ));
                }
            }
            if route_id.as_ref().is_some_and(|id| id.trim().is_empty()) {
                return Err(boss_error(
                    object_id,
                    format!("{path}.route_id"),
                    "route ID must not be empty",
                ));
            }
            if let Some(lifetime) = lifetime {
                validate_duration_value(
                    &lifetime.lifetime_ms,
                    parameter_types,
                    object_id,
                    &format!("{path}.lifetime.lifetime_ms"),
                )?;
            }
            Ok(())
        }
        BossTaskData::RouteCombat { .. } | BossTaskData::EmitSignal { .. } => Ok(()),
    }
}

fn validate_summon_occupancy(
    policy: &BossSummonOccupancyPolicyData,
    object_id: &str,
    path: &str,
) -> Result<(), DataValidationError> {
    let BossSummonOccupancyPolicyData::AllowOccupied { allowed } = policy else {
        return Ok(());
    };
    if allowed.is_empty() {
        return Err(boss_error(
            object_id,
            format!("{path}.occupancy_policy.allowed"),
            "AllowOccupied must declare a non-empty allowed set",
        ));
    }
    let unique = allowed.iter().copied().collect::<HashSet<_>>();
    if unique.len() != allowed.len() {
        return Err(boss_error(
            object_id,
            format!("{path}.occupancy_policy.allowed"),
            "AllowOccupied values must be unique",
        ));
    }
    if unique.contains(&BossSummonOccupancyClassData::Any) && unique.len() != 1 {
        return Err(boss_error(
            object_id,
            format!("{path}.occupancy_policy.allowed"),
            "Any must be the only AllowOccupied value",
        ));
    }
    if unique.contains(&BossSummonOccupancyClassData::Summoner)
        && unique.contains(&BossSummonOccupancyClassData::SameFaction)
    {
        return Err(boss_error(
            object_id,
            format!("{path}.occupancy_policy.allowed"),
            "Summoner is redundant when SameFaction is allowed",
        ));
    }
    Ok(())
}

fn validate_duration_value(
    value: &BossDurationValueData,
    parameter_types: &HashMap<BossParameterId, BossParameterTypeData>,
    object_id: &str,
    path: &str,
) -> Result<(), DataValidationError> {
    match value {
        BossDurationValueData::Literal(0) => Err(boss_error(
            object_id,
            path,
            "duration must be greater than zero",
        )),
        BossDurationValueData::Literal(_) => Ok(()),
        BossDurationValueData::Parameter(parameter_id) => require_parameter_type(
            parameter_id,
            parameter_types,
            |kind| matches!(kind, BossParameterTypeData::DurationMs { .. }),
            "DurationMs",
            object_id,
            path,
        ),
    }
}

fn require_parameter_type(
    parameter_id: &BossParameterId,
    parameter_types: &HashMap<BossParameterId, BossParameterTypeData>,
    predicate: impl FnOnce(&BossParameterTypeData) -> bool,
    expected: &str,
    object_id: &str,
    path: &str,
) -> Result<(), DataValidationError> {
    let Some(actual) = parameter_types.get(parameter_id) else {
        return Err(boss_error(
            object_id,
            path,
            format!("references missing parameter '{}'", parameter_id.as_str()),
        ));
    };
    if !predicate(actual) {
        return Err(boss_error(
            object_id,
            path,
            format!(
                "parameter '{}' has type {actual:?}, expected {expected}",
                parameter_id.as_str()
            ),
        ));
    }
    Ok(())
}

fn validate_summon_contracts(
    catalog: &BossRuntimeCatalogData,
    summoned_units: &SummonedUnitDatabase,
    signal_ids: &HashSet<BossSignalId>,
) -> Result<(), DataValidationError> {
    for signal in &catalog.signals {
        validate_catalog_id(signal.id.as_str(), signal.id.as_str(), "id")?;
    }

    let patterns = catalog
        .patterns
        .iter()
        .map(|pattern| (pattern.id.clone(), pattern))
        .collect::<HashMap<_, _>>();
    let behaviors = catalog
        .actor_behaviors
        .iter()
        .map(|behavior| (behavior.id.clone(), behavior))
        .collect::<HashMap<_, _>>();

    let mut produced_signals = HashSet::new();
    let mut observe_task = |task: &BossTaskData, object_id: &str, path: &str| {
        match task {
            BossTaskData::EmitSignal { signal_id, .. } => {
                if !signal_ids.contains(signal_id) {
                    return Err(boss_error(
                        object_id,
                        path,
                        format!("references undeclared signal '{}'", signal_id.as_str()),
                    ));
                }
                produced_signals.insert(signal_id.clone());
            }
            BossTaskData::Summon { definition_id, .. } => {
                if summoned_units.get_by_id(definition_id).is_none() {
                    return Err(boss_error(
                        object_id,
                        path,
                        format!(
                            "references missing summoned-unit definition '{}'",
                            definition_id.as_str()
                        ),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    };

    for pattern in &catalog.patterns {
        for (state_index, state) in pattern.states.iter().enumerate() {
            for (task_index, task) in state.tasks.iter().enumerate() {
                observe_task(
                    task,
                    pattern.id.as_str(),
                    &format!("states[{state_index}].tasks[{task_index}]"),
                )?;
            }
        }
    }
    for behavior in &catalog.actor_behaviors {
        for (node_index, node) in behavior.nodes.iter().enumerate() {
            if let BossControlNodeData::State { tasks, .. } = node {
                for (task_index, task) in tasks.iter().enumerate() {
                    if matches!(task, BossTaskData::Summon { .. }) {
                        return Err(boss_error(
                            behavior.id.as_str(),
                            format!("nodes[{node_index}].tasks[{task_index}]"),
                            "Summon is valid only inside a Pattern state with a pattern-scoped batch handle",
                        ));
                    }
                    observe_task(
                        task,
                        behavior.id.as_str(),
                        &format!("nodes[{node_index}].tasks[{task_index}]"),
                    )?;
                }
            }
        }
    }
    drop(observe_task);

    for pattern in &catalog.patterns {
        let mut handles = HashSet::new();
        for (state_index, state) in pattern.states.iter().enumerate() {
            let has_summon = state
                .tasks
                .iter()
                .any(|task| matches!(task, BossTaskData::Summon { .. }));
            if has_summon
                && state.transitions.iter().any(|transition| {
                    !matches!(transition.condition, BossConditionData::TaskResult { .. })
                })
            {
                return Err(boss_error(
                    pattern.id.as_str(),
                    format!("states[{state_index}].transitions"),
                    "a state with an unresolved Summon task may transition only from explicit task results",
                ));
            }
            for (task_index, task) in state.tasks.iter().enumerate() {
                let BossTaskData::Summon {
                    definition_id,
                    batch_handle_id,
                    route_id,
                    removal_signal,
                    ..
                } = task
                else {
                    continue;
                };
                validate_stable_id(
                    batch_handle_id.as_str(),
                    pattern.id.as_str(),
                    &format!("states[{state_index}].tasks[{task_index}].batch_handle_id"),
                )?;
                if !handles.insert(batch_handle_id.clone()) {
                    return Err(boss_error(
                        pattern.id.as_str(),
                        format!("states[{state_index}].tasks[{task_index}].batch_handle_id"),
                        format!(
                            "batch handle '{}' is produced more than once in one Pattern scope",
                            batch_handle_id.as_str()
                        ),
                    ));
                }
                let definition = summoned_units
                    .get_by_id(definition_id)
                    .expect("summon definition references were checked above");
                let behavior = behaviors
                    .get(&definition.actor_behavior_id)
                    .ok_or_else(|| {
                        boss_error(
                            definition.id.as_str(),
                            "actor_behavior_id",
                            format!(
                                "references missing actor behavior '{}'",
                                definition.actor_behavior_id.as_str()
                            ),
                        )
                    })?;
                if raw_behavior_requires_route(behavior, &patterns) && route_id.is_none() {
                    return Err(boss_error(
                        pattern.id.as_str(),
                        format!("states[{state_index}].tasks[{task_index}].route_id"),
                        format!(
                            "summoned-unit behavior '{}' requires an explicit route context",
                            behavior.id.as_str()
                        ),
                    ));
                }
                if let Some(removal_signal) = removal_signal {
                    if !signal_ids.contains(&removal_signal.signal_id) {
                        return Err(boss_error(
                            pattern.id.as_str(),
                            format!("states[{state_index}].tasks[{task_index}].removal_signal"),
                            format!(
                                "references undeclared signal '{}'",
                                removal_signal.signal_id.as_str()
                            ),
                        ));
                    }
                    if !produced_signals.contains(&removal_signal.signal_id) {
                        return Err(boss_error(
                            pattern.id.as_str(),
                            format!("states[{state_index}].tasks[{task_index}].removal_signal"),
                            format!(
                                "signal '{}' has no Boss Runtime producer",
                                removal_signal.signal_id.as_str()
                            ),
                        ));
                    }
                }
            }
        }
        for (state_index, state) in pattern.states.iter().enumerate() {
            for (transition_index, transition) in state.transitions.iter().enumerate() {
                if let BossConditionData::SummonBatchCleared { batch_handle_id } =
                    &transition.condition
                {
                    if !handles.contains(batch_handle_id) {
                        return Err(boss_error(
                            pattern.id.as_str(),
                            format!(
                                "states[{state_index}].transitions[{transition_index}].condition"
                            ),
                            format!(
                                "references unproduced batch handle '{}'",
                                batch_handle_id.as_str()
                            ),
                        ));
                    }
                }
            }
        }
    }

    for behavior in &catalog.actor_behaviors {
        for (node_index, node) in behavior.nodes.iter().enumerate() {
            let transitions = match node {
                BossControlNodeData::State { transitions, .. } => transitions,
                BossControlNodeData::PatternCall { .. } => continue,
            };
            for (transition_index, transition) in transitions.iter().enumerate() {
                if matches!(
                    transition.condition,
                    BossConditionData::SummonBatchCleared { .. }
                ) {
                    return Err(boss_error(
                        behavior.id.as_str(),
                        format!("nodes[{node_index}].transitions[{transition_index}].condition"),
                        "SummonBatchCleared requires a Pattern-local scope",
                    ));
                }
            }
        }
    }

    for definition in summoned_units.definitions() {
        let behavior = behaviors
            .get(&definition.actor_behavior_id)
            .ok_or_else(|| {
                boss_error(
                    definition.id.as_str(),
                    "actor_behavior_id",
                    format!(
                        "references missing actor behavior '{}'",
                        definition.actor_behavior_id.as_str()
                    ),
                )
            })?;
        if raw_behavior_contains_summon(behavior, &patterns) {
            return Err(boss_error(
                definition.id.as_str(),
                "actor_behavior_id",
                "summoned-unit behavior graph must not contain a Summon task",
            ));
        }
    }

    validate_bound_signal_contracts(catalog, &patterns)?;

    Ok(())
}

fn validate_bound_signal_contracts(
    catalog: &BossRuntimeCatalogData,
    patterns: &HashMap<BossPatternId, &BossPatternDefinitionData>,
) -> Result<(), DataValidationError> {
    let bound_behavior_ids = catalog
        .encounters
        .iter()
        .flat_map(|encounter| encounter.phases.iter())
        .flat_map(|phase| phase.actor_behaviors.iter())
        .map(|binding| binding.behavior_id.clone())
        .collect::<HashSet<_>>();
    let bound_pattern_ids = catalog
        .actor_behaviors
        .iter()
        .filter(|behavior| bound_behavior_ids.contains(&behavior.id))
        .flat_map(raw_behavior_pattern_ids)
        .cloned()
        .collect::<HashSet<_>>();

    for pattern in &catalog.patterns {
        for (state_index, state) in pattern.states.iter().enumerate() {
            for (task_index, task) in state.tasks.iter().enumerate() {
                if task_signal_role(task).is_some() && !bound_pattern_ids.contains(&pattern.id) {
                    return Err(boss_error(
                        pattern.id.as_str(),
                        format!("states[{state_index}].tasks[{task_index}]"),
                        "signal task belongs to a Pattern that is not bound by any encounter behavior",
                    ));
                }
            }
        }
    }
    for behavior in &catalog.actor_behaviors {
        for (node_index, node) in behavior.nodes.iter().enumerate() {
            let BossControlNodeData::State { tasks, .. } = node else {
                continue;
            };
            for (task_index, task) in tasks.iter().enumerate() {
                if task_signal_role(task).is_some() && !bound_behavior_ids.contains(&behavior.id) {
                    return Err(boss_error(
                        behavior.id.as_str(),
                        format!("nodes[{node_index}].tasks[{task_index}]"),
                        "signal task belongs to a behavior that is not bound by any encounter",
                    ));
                }
            }
        }
    }

    let behaviors = catalog
        .actor_behaviors
        .iter()
        .map(|behavior| (behavior.id.clone(), behavior))
        .collect::<HashMap<_, _>>();
    for encounter in &catalog.encounters {
        let behavior_ids = encounter
            .phases
            .iter()
            .flat_map(|phase| phase.actor_behaviors.iter())
            .map(|binding| binding.behavior_id.clone())
            .collect::<HashSet<_>>();
        let mut produced = HashSet::new();
        let mut consumed = HashSet::new();
        for behavior_id in behavior_ids {
            let Some(behavior) = behaviors.get(&behavior_id) else {
                continue;
            };
            collect_behavior_signal_roles(behavior, patterns, &mut produced, &mut consumed);
        }
        for signal_id in consumed.difference(&produced) {
            return Err(boss_error(
                encounter.id.as_str(),
                "phases",
                format!(
                    "removal signal '{}' is consumed but has no producer bound to this encounter",
                    signal_id.as_str()
                ),
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalTaskRole<'a> {
    Producer(&'a BossSignalId),
    Consumer(&'a BossSignalId),
}

fn task_signal_role(task: &BossTaskData) -> Option<SignalTaskRole<'_>> {
    match task {
        BossTaskData::EmitSignal { signal_id, .. } => Some(SignalTaskRole::Producer(signal_id)),
        BossTaskData::Summon {
            removal_signal: Some(removal_signal),
            ..
        } => Some(SignalTaskRole::Consumer(&removal_signal.signal_id)),
        _ => None,
    }
}

fn collect_behavior_signal_roles(
    behavior: &BossActorBehaviorDefinitionData,
    patterns: &HashMap<BossPatternId, &BossPatternDefinitionData>,
    produced: &mut HashSet<BossSignalId>,
    consumed: &mut HashSet<BossSignalId>,
) {
    let mut observe = |task: &BossTaskData| match task_signal_role(task) {
        Some(SignalTaskRole::Producer(signal_id)) => {
            produced.insert(signal_id.clone());
        }
        Some(SignalTaskRole::Consumer(signal_id)) => {
            consumed.insert(signal_id.clone());
        }
        None => {}
    };
    for node in &behavior.nodes {
        if let BossControlNodeData::State { tasks, .. } = node {
            for task in tasks {
                observe(task);
            }
        }
    }
    for pattern_id in raw_behavior_pattern_ids(behavior) {
        let Some(pattern) = patterns.get(pattern_id) else {
            continue;
        };
        for state in &pattern.states {
            for task in &state.tasks {
                observe(task);
            }
        }
    }
}

fn raw_behavior_pattern_ids(behavior: &BossActorBehaviorDefinitionData) -> Vec<&BossPatternId> {
    let mut ids = Vec::new();
    for node in &behavior.nodes {
        let BossControlNodeData::PatternCall { selector, .. } = node else {
            continue;
        };
        match selector {
            BossPatternSelectorData::FixedSequence { candidates, .. }
            | BossPatternSelectorData::PriorityEligible { candidates } => {
                ids.extend(candidates.iter().map(|candidate| &candidate.pattern_id));
            }
            BossPatternSelectorData::WeightedEligible { candidates } => {
                ids.extend(
                    candidates
                        .iter()
                        .map(|candidate| &candidate.candidate.pattern_id),
                );
            }
        }
    }
    ids
}

fn raw_behavior_contains_summon(
    behavior: &BossActorBehaviorDefinitionData,
    patterns: &HashMap<BossPatternId, &BossPatternDefinitionData>,
) -> bool {
    behavior.nodes.iter().any(|node| match node {
        BossControlNodeData::State { tasks, .. } => tasks
            .iter()
            .any(|task| matches!(task, BossTaskData::Summon { .. })),
        BossControlNodeData::PatternCall { .. } => false,
    }) || raw_behavior_pattern_ids(behavior)
        .into_iter()
        .any(|pattern_id| {
            patterns.get(pattern_id).is_some_and(|pattern| {
                pattern.states.iter().any(|state| {
                    state
                        .tasks
                        .iter()
                        .any(|task| matches!(task, BossTaskData::Summon { .. }))
                })
            })
        })
}

fn raw_behavior_requires_route(
    behavior: &BossActorBehaviorDefinitionData,
    patterns: &HashMap<BossPatternId, &BossPatternDefinitionData>,
) -> bool {
    let task_requires_route = |task: &BossTaskData| {
        matches!(
            task,
            BossTaskData::RouteCombat { .. } | BossTaskData::RejoinForwardRoute { .. }
        )
    };
    behavior.nodes.iter().any(|node| match node {
        BossControlNodeData::State { tasks, .. } => tasks.iter().any(task_requires_route),
        BossControlNodeData::PatternCall { .. } => false,
    }) || raw_behavior_pattern_ids(behavior)
        .into_iter()
        .any(|pattern_id| {
            patterns.get(pattern_id).is_some_and(|pattern| {
                pattern
                    .states
                    .iter()
                    .any(|state| state.tasks.iter().any(task_requires_route))
            })
        })
}

impl BossRuntimeCatalogData {
    #[cfg(test)]
    pub(crate) fn compile_count_for_current_thread() -> usize {
        BOSS_RUNTIME_COMPILE_COUNT.get()
    }

    #[cfg(test)]
    pub(crate) fn compile(&self) -> Result<BossRuntimeDatabase, DataValidationError> {
        self.compile_with_summoned_units(&SummonedUnitCatalogData::default())
    }

    #[cfg(test)]
    pub(crate) fn compile_with_summoned_units(
        &self,
        summoned_unit_catalog: &SummonedUnitCatalogData,
    ) -> Result<BossRuntimeDatabase, DataValidationError> {
        let summoned_units = summoned_unit_catalog.compile()?;
        self.compile_with_summoned_unit_database(summoned_units)
    }

    pub(crate) fn compile_with_summoned_unit_database(
        &self,
        summoned_units: SummonedUnitDatabase,
    ) -> Result<BossRuntimeDatabase, DataValidationError> {
        #[cfg(test)]
        BOSS_RUNTIME_COMPILE_COUNT.set(BOSS_RUNTIME_COMPILE_COUNT.get() + 1);

        let signal_index = unique_catalog_index(
            self.signals.iter().map(|signal| signal.id.0.clone()),
            "boss_runtime_catalog",
            "signals",
        )?;
        let signal_ids = signal_index
            .keys()
            .cloned()
            .map(BossSignalId::new)
            .collect::<HashSet<_>>();
        validate_summon_contracts(self, &summoned_units, &signal_ids)?;

        let pattern_index = unique_catalog_index(
            self.patterns.iter().map(|pattern| pattern.id.0.clone()),
            "boss_runtime_catalog",
            "patterns",
        )?;
        let actor_behavior_index = unique_catalog_index(
            self.actor_behaviors
                .iter()
                .map(|behavior| behavior.id.0.clone()),
            "boss_runtime_catalog",
            "actor_behaviors",
        )?;
        let encounter_index = unique_catalog_index(
            self.encounters
                .iter()
                .map(|encounter| encounter.id.0.clone()),
            "boss_runtime_catalog",
            "encounters",
        )?;

        let patterns = self
            .patterns
            .iter()
            .map(compile_pattern)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();
        let patterns_by_id = pattern_index
            .iter()
            .map(|(id, index)| (BossPatternId::new(id.clone()), patterns[*index].clone()))
            .collect::<HashMap<_, _>>();

        let actor_behaviors = self
            .actor_behaviors
            .iter()
            .map(|behavior| compile_actor_behavior(behavior, &patterns_by_id))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();
        let actor_behaviors_by_id = actor_behavior_index
            .iter()
            .map(|(id, index)| {
                (
                    BossActorBehaviorId::new(id.clone()),
                    actor_behaviors[*index].clone(),
                )
            })
            .collect::<HashMap<_, _>>();

        let encounters = self
            .encounters
            .iter()
            .map(|encounter| compile_encounter(encounter, &actor_behaviors_by_id))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();

        Ok(BossRuntimeDatabase {
            #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
            authored: self.clone(),
            patterns,
            actor_behaviors,
            encounters,
            pattern_by_id: pattern_index
                .into_iter()
                .map(|(id, index)| (BossPatternId::new(id), index))
                .collect(),
            actor_behavior_by_id: actor_behavior_index
                .into_iter()
                .map(|(id, index)| (BossActorBehaviorId::new(id), index))
                .collect(),
            encounter_by_id: encounter_index
                .into_iter()
                .map(|(id, index)| (BossEncounterRuntimeId::new(id), index))
                .collect(),
            summoned_units,
            signal_ids,
        })
    }
}

fn compile_pattern(
    pattern: &BossPatternDefinitionData,
) -> Result<CompiledBossPatternDefinition, DataValidationError> {
    let object_id = pattern.id.as_str();
    let parameter_index = unique_index(
        pattern.parameters.iter().map(|item| item.id.0.clone()),
        object_id,
        "parameters",
    )?;
    let parameter_types = pattern
        .parameters
        .iter()
        .map(|parameter| (parameter.id.clone(), parameter.parameter_type.clone()))
        .collect::<HashMap<_, _>>();
    for (index, parameter) in pattern.parameters.iter().enumerate() {
        if let BossParameterTypeData::DurationMs { min_ms, max_ms } = parameter.parameter_type {
            if min_ms == 0 || min_ms > max_ms {
                return Err(boss_error(
                    object_id,
                    format!("parameters[{index}].parameter_type"),
                    "DurationMs bounds must satisfy 0 < min_ms <= max_ms",
                ));
            }
        }
    }

    if pattern.outcomes.is_empty() {
        return Err(boss_error(
            object_id,
            "outcomes",
            "finite Pattern must declare at least one outcome",
        ));
    }
    let outcome_index = unique_index(
        pattern.outcomes.iter().map(|outcome| outcome.0.clone()),
        object_id,
        "outcomes",
    )?;
    if pattern.states.is_empty() {
        return Err(boss_error(
            object_id,
            "states",
            "Pattern must define at least one state",
        ));
    }
    let state_index = unique_index(
        pattern.states.iter().map(|state| state.id.0.clone()),
        object_id,
        "states",
    )?;
    let Some(&entry_state_index) = state_index.get(pattern.entry_state_id.as_str()) else {
        return Err(boss_error(
            object_id,
            "entry_state_id",
            format!(
                "references missing state '{}'",
                pattern.entry_state_id.as_str()
            ),
        ));
    };

    let mut all_adjacency = vec![Vec::new(); pattern.states.len()];
    let mut unguarded_adjacency = vec![Vec::new(); pattern.states.len()];
    let mut immediate_adjacency = vec![Vec::new(); pattern.states.len()];
    let mut terminal_states = HashSet::new();
    let mut compiled_states = Vec::with_capacity(pattern.states.len());
    let mut loop_ids = HashSet::new();

    for (state_position, state) in pattern.states.iter().enumerate() {
        if state.transitions.is_empty() {
            return Err(boss_error(
                object_id,
                format!("states[{state_position}].transitions"),
                "Pattern state must explicitly transition or return an outcome",
            ));
        }
        validate_tasks_and_result_coverage(
            &state.tasks,
            state
                .transitions
                .iter()
                .map(|transition| &transition.condition),
            &parameter_types,
            object_id,
            &format!("states[{state_position}]"),
        )?;
        let task_index = state
            .tasks
            .iter()
            .enumerate()
            .map(|(index, task)| (task.id().clone(), index))
            .collect::<HashMap<_, _>>();
        let compiled_tasks = state
            .tasks
            .iter()
            .map(|task| compile_task(task, &parameter_index, &parameter_types, object_id))
            .collect::<Result<Vec<_>, _>>()?;

        let mut transition_ids = HashSet::new();
        let mut priorities = HashSet::new();
        let mut compiled_transitions = Vec::with_capacity(state.transitions.len());
        for (transition_position, transition) in state.transitions.iter().enumerate() {
            validate_stable_id(
                transition.id.as_str(),
                object_id,
                &format!("states[{state_position}].transitions[{transition_position}].id"),
            )?;
            if !transition_ids.insert(transition.id.clone()) {
                return Err(boss_error(
                    object_id,
                    format!("states[{state_position}].transitions[{transition_position}].id"),
                    format!("duplicate transition id '{}'", transition.id.as_str()),
                ));
            }
            if !priorities.insert(transition.priority) {
                return Err(boss_error(
                    object_id,
                    format!("states[{state_position}].transitions[{transition_position}].priority"),
                    format!(
                        "transition priority {} is not unique in this node",
                        transition.priority
                    ),
                ));
            }
            validate_condition(
                &transition.condition,
                &state
                    .tasks
                    .iter()
                    .filter_map(|task| task_result_set(task).map(|set| (task.id().clone(), set)))
                    .collect(),
                object_id,
                &format!("states[{state_position}].transitions[{transition_position}].condition"),
            )?;
            if let BossConditionData::TaskResult { task_id, .. } = &transition.condition {
                if !task_index.contains_key(task_id) {
                    return Err(boss_error(
                        object_id,
                        format!(
                            "states[{state_position}].transitions[{transition_position}].condition"
                        ),
                        format!("references missing task '{}'", task_id.as_str()),
                    ));
                }
            }

            let target = compile_pattern_target(
                &transition.target,
                &state_index,
                &outcome_index,
                object_id,
                &format!("states[{state_position}].transitions[{transition_position}].target"),
            )?;
            if matches!(target, CompiledBossPatternTransitionTarget::ContinueState)
                && transition.loop_guard.is_none()
                && matches!(
                    &transition.condition,
                    BossConditionData::Always
                        | BossConditionData::StatePredicate(_)
                        | BossConditionData::EventOccurred {
                            memory: BossEventMemoryData::UntilStateExit,
                            ..
                        }
                )
            {
                return Err(boss_error(
                    object_id,
                    format!("states[{state_position}].transitions[{transition_position}]"),
                    "persistent ContinueState condition requires a bounded loop guard",
                ));
            }
            match target {
                CompiledBossPatternTransitionTarget::State(next) => {
                    all_adjacency[state_position].push(next);
                    if transition.loop_guard.is_none() {
                        unguarded_adjacency[state_position].push(next);
                    }
                    if transition.condition.is_immediate() {
                        immediate_adjacency[state_position].push(next);
                    }
                }
                CompiledBossPatternTransitionTarget::Outcome(_) => {
                    terminal_states.insert(state_position);
                }
                CompiledBossPatternTransitionTarget::ContinueState => {}
            }

            let loop_guard = transition
                .loop_guard
                .as_ref()
                .map(|guard| {
                    validate_stable_id(
                        guard.id.as_str(),
                        object_id,
                        &format!(
                            "states[{state_position}].transitions[{transition_position}].loop_guard.id"
                        ),
                    )?;
                    if !loop_ids.insert(guard.id.clone()) {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "states[{state_position}].transitions[{transition_position}].loop_guard.id"
                            ),
                            format!("duplicate loop id '{}'", guard.id.as_str()),
                        ));
                    }
                    if guard.max_iterations == 0 {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "states[{state_position}].transitions[{transition_position}].loop_guard.max_iterations"
                            ),
                            "loop max_iterations must be greater than zero",
                        ));
                    }
                    let exhausted_target = compile_pattern_target(
                        &guard.exhausted_target,
                        &state_index,
                        &outcome_index,
                        object_id,
                        &format!(
                            "states[{state_position}].transitions[{transition_position}].loop_guard.exhausted_target"
                        ),
                    )?;
                    match exhausted_target {
                        CompiledBossPatternTransitionTarget::State(next) => {
                            all_adjacency[state_position].push(next);
                            // Exhaustion is the unguarded path after the bounded edge can no
                            // longer be taken. It must not lead back into a cycle of its own.
                            unguarded_adjacency[state_position].push(next);
                            if transition.condition.is_immediate() {
                                immediate_adjacency[state_position].push(next);
                            }
                        }
                        CompiledBossPatternTransitionTarget::Outcome(_) => {
                            terminal_states.insert(state_position);
                        }
                        CompiledBossPatternTransitionTarget::ContinueState => {
                            return Err(boss_error(
                                object_id,
                                format!(
                                    "states[{state_position}].transitions[{transition_position}].loop_guard.exhausted_target"
                                ),
                                "loop exhaustion must leave the exhausted edge",
                            ));
                        }
                    }
                    Ok(CompiledBossLoopGuard {
                        id: guard.id.clone(),
                        max_iterations: guard.max_iterations,
                        exhausted_target,
                    })
                })
                .transpose()?;
            compiled_transitions.push(CompiledBossPatternTransition {
                id: transition.id.clone(),
                priority: transition.priority,
                condition: transition.condition.clone(),
                target,
                loop_guard,
            });
        }
        compiled_states.push(CompiledBossPatternState {
            id: state.id.clone(),
            tasks: compiled_tasks,
            transitions: compiled_transitions,
        });
    }

    let reachable = reachable_from(entry_state_index, &all_adjacency);
    if reachable.len() != pattern.states.len() {
        return Err(boss_error(
            object_id,
            "states",
            "every Pattern state must be reachable from entry_state_id",
        ));
    }
    let mut reverse = vec![Vec::new(); pattern.states.len()];
    for (from, targets) in all_adjacency.iter().enumerate() {
        for &to in targets {
            reverse[to].push(from);
        }
    }
    let mut can_finish = terminal_states.clone();
    let mut pending = terminal_states.into_iter().collect::<VecDeque<_>>();
    while let Some(node) = pending.pop_front() {
        for &previous in &reverse[node] {
            if can_finish.insert(previous) {
                pending.push_back(previous);
            }
        }
    }
    if can_finish.len() != pattern.states.len() {
        return Err(boss_error(
            object_id,
            "states",
            "every Pattern state must have a path to a declared outcome",
        ));
    }
    if has_cycle(&unguarded_adjacency) {
        return Err(boss_error(
            object_id,
            "states",
            "Pattern contains a cycle without a bounded loop guard",
        ));
    }
    if has_cycle(&immediate_adjacency) {
        return Err(boss_error(
            object_id,
            "states",
            "Pattern contains a zero-time cycle",
        ));
    }

    Ok(CompiledBossPatternDefinition {
        id: pattern.id.clone(),
        parameters: pattern
            .parameters
            .iter()
            .map(|parameter| CompiledBossParameterDefinition {
                id: parameter.id.clone(),
                parameter_type: parameter.parameter_type.clone(),
            })
            .collect(),
        outcomes: pattern.outcomes.clone(),
        entry_state_index,
        states: compiled_states,
    })
}

fn compile_pattern_target(
    target: &BossPatternTransitionTargetData,
    state_index: &HashMap<String, usize>,
    outcome_index: &HashMap<String, usize>,
    object_id: &str,
    path: &str,
) -> Result<CompiledBossPatternTransitionTarget, DataValidationError> {
    match target {
        BossPatternTransitionTargetData::State(id) => state_index
            .get(id.as_str())
            .copied()
            .map(CompiledBossPatternTransitionTarget::State)
            .ok_or_else(|| {
                boss_error(
                    object_id,
                    path,
                    format!("references missing state '{}'", id.as_str()),
                )
            }),
        BossPatternTransitionTargetData::Outcome(id) => outcome_index
            .get(id.as_str())
            .copied()
            .map(CompiledBossPatternTransitionTarget::Outcome)
            .ok_or_else(|| {
                boss_error(
                    object_id,
                    path,
                    format!("references undeclared outcome '{}'", id.as_str()),
                )
            }),
        BossPatternTransitionTargetData::ContinueState => {
            Ok(CompiledBossPatternTransitionTarget::ContinueState)
        }
    }
}

fn compile_task(
    task: &BossTaskData,
    parameter_index: &HashMap<String, usize>,
    parameter_types: &HashMap<BossParameterId, BossParameterTypeData>,
    object_id: &str,
) -> Result<CompiledBossTask, DataValidationError> {
    validate_task_values(task, parameter_types, object_id, "task")?;
    let duration = |value: &BossDurationValueData| match value {
        BossDurationValueData::Literal(value) => Ok(CompiledBossDurationValue::Literal(*value)),
        BossDurationValueData::Parameter(id) => parameter_index
            .get(id.as_str())
            .copied()
            .map(CompiledBossDurationValue::Parameter)
            .ok_or_else(|| boss_error(object_id, "task", "missing duration parameter")),
    };
    Ok(match task {
        BossTaskData::StartAction {
            id,
            skill_id,
            target,
            facing,
            milestone,
            timeout_ms,
            on_timeout,
        } => CompiledBossTask::StartAction {
            id: id.clone(),
            skill_id: skill_id.clone(),
            target: *target,
            facing: *facing,
            milestone: *milestone,
            timeout_ms: *timeout_ms,
            on_timeout: *on_timeout,
        },
        BossTaskData::RouteCombat { id } => CompiledBossTask::RouteCombat { id: id.clone() },
        BossTaskData::AcquireTarget { id, query } => CompiledBossTask::AcquireTarget {
            id: id.clone(),
            query: match query {
                BossTargetQueryValueData::Literal(query) => {
                    CompiledBossTargetQueryValue::Literal(query.clone())
                }
                BossTargetQueryValueData::Parameter(parameter_id) => {
                    CompiledBossTargetQueryValue::Parameter(
                        *parameter_index.get(parameter_id.as_str()).ok_or_else(|| {
                            boss_error(object_id, "task", "missing target-query parameter")
                        })?,
                    )
                }
            },
        },
        BossTaskData::MoveToTarget { id, timeout_ms } => CompiledBossTask::MoveToTarget {
            id: id.clone(),
            timeout_ms: duration(timeout_ms)?,
        },
        BossTaskData::RejoinForwardRoute { id, timeout_ms } => {
            CompiledBossTask::RejoinForwardRoute {
                id: id.clone(),
                timeout_ms: duration(timeout_ms)?,
            }
        }
        BossTaskData::Wait { id, duration_ms } => CompiledBossTask::Wait {
            id: id.clone(),
            duration_ms: duration(duration_ms)?,
        },
        BossTaskData::Summon {
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
        } => CompiledBossTask::Summon {
            id: id.clone(),
            definition_id: definition_id.clone(),
            count: *count,
            batch_handle_id: batch_handle_id.clone(),
            placement: placement.clone(),
            occupancy_policy: occupancy_policy.clone(),
            on_insufficient_space: match on_insufficient_space {
                BossSummonInsufficientSpaceData::Fail => CompiledBossSummonInsufficientSpace::Fail,
                BossSummonInsufficientSpaceData::WaitUntilAvailable { timeout_ms } => {
                    CompiledBossSummonInsufficientSpace::WaitUntilAvailable {
                        timeout_ms: duration(timeout_ms)?,
                    }
                }
            },
            route_id: route_id.clone(),
            lifetime: lifetime
                .as_ref()
                .map(|lifetime| {
                    Ok(CompiledBossSummonLifetime {
                        lifetime_ms: duration(&lifetime.lifetime_ms)?,
                        mode: lifetime.mode,
                    })
                })
                .transpose()?,
            removal_signal: removal_signal.clone(),
        },
        BossTaskData::EmitSignal { id, signal_id } => CompiledBossTask::EmitSignal {
            id: id.clone(),
            signal_id: signal_id.clone(),
        },
        BossTaskData::RepathToProtectedObjective { .. } => {
            return Err(boss_error(
                object_id,
                "task",
                "RepathToProtectedObjective is not implemented in v1",
            ));
        }
    })
}

fn compile_actor_behavior(
    behavior: &BossActorBehaviorDefinitionData,
    patterns: &HashMap<BossPatternId, Arc<CompiledBossPatternDefinition>>,
) -> Result<CompiledBossActorBehaviorDefinition, DataValidationError> {
    let object_id = behavior.id.as_str();
    if behavior.nodes.is_empty() {
        return Err(boss_error(
            object_id,
            "nodes",
            "actor behavior must define at least one control node",
        ));
    }
    let node_index = unique_index(
        behavior.nodes.iter().map(|node| node.id().0.clone()),
        object_id,
        "nodes",
    )?;
    let Some(&entry_node_index) = node_index.get(behavior.entry_node_id.as_str()) else {
        return Err(boss_error(
            object_id,
            "entry_node_id",
            format!(
                "references missing control node '{}'",
                behavior.entry_node_id.as_str()
            ),
        ));
    };
    let empty_parameters = HashMap::new();
    let mut all_adjacency = vec![Vec::new(); behavior.nodes.len()];
    let mut immediate_adjacency = vec![Vec::new(); behavior.nodes.len()];
    let mut cooldown_durations: HashMap<BossCooldownGroupId, u64> = HashMap::new();
    let mut compiled_nodes = Vec::with_capacity(behavior.nodes.len());

    for (node_position, node) in behavior.nodes.iter().enumerate() {
        match node {
            BossControlNodeData::State {
                id,
                tasks,
                transitions,
            } => {
                validate_tasks_and_result_coverage(
                    tasks,
                    transitions.iter().map(|transition| &transition.condition),
                    &empty_parameters,
                    object_id,
                    &format!("nodes[{node_position}]"),
                )?;
                let compiled_tasks = tasks
                    .iter()
                    .map(|task| compile_task(task, &HashMap::new(), &empty_parameters, object_id))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut ids = HashSet::new();
                let mut priorities = HashSet::new();
                let task_results = tasks
                    .iter()
                    .filter_map(|task| task_result_set(task).map(|set| (task.id().clone(), set)))
                    .collect::<HashMap<_, _>>();
                let mut compiled_transitions = Vec::with_capacity(transitions.len());
                for (transition_position, transition) in transitions.iter().enumerate() {
                    validate_stable_id(
                        transition.id.as_str(),
                        object_id,
                        &format!("nodes[{node_position}].transitions[{transition_position}].id"),
                    )?;
                    if !ids.insert(transition.id.clone()) {
                        return Err(boss_error(
                            object_id,
                            format!("nodes[{node_position}].transitions[{transition_position}].id"),
                            format!("duplicate transition id '{}'", transition.id.as_str()),
                        ));
                    }
                    if !priorities.insert(transition.priority) {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "nodes[{node_position}].transitions[{transition_position}].priority"
                            ),
                            format!(
                                "transition priority {} is not unique in this node",
                                transition.priority
                            ),
                        ));
                    }
                    validate_condition(
                        &transition.condition,
                        &task_results,
                        object_id,
                        &format!(
                            "nodes[{node_position}].transitions[{transition_position}].condition"
                        ),
                    )?;
                    let Some(&target_node_index) =
                        node_index.get(transition.target_node_id.as_str())
                    else {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "nodes[{node_position}].transitions[{transition_position}].target_node_id"
                            ),
                            format!(
                                "references missing control node '{}'",
                                transition.target_node_id.as_str()
                            ),
                        ));
                    };
                    all_adjacency[node_position].push(target_node_index);
                    if transition.condition.is_immediate() {
                        immediate_adjacency[node_position].push(target_node_index);
                    }
                    compiled_transitions.push(CompiledBossControlTransition {
                        id: transition.id.clone(),
                        priority: transition.priority,
                        condition: transition.condition.clone(),
                        target_node_index,
                    });
                }
                compiled_nodes.push(CompiledBossControlNode::State {
                    id: id.clone(),
                    tasks: compiled_tasks,
                    transitions: compiled_transitions,
                });
            }
            BossControlNodeData::PatternCall {
                id,
                selector,
                outcome_routes,
                no_eligible_node_id,
            } => {
                let (candidate_data, repeat) = match selector {
                    BossPatternSelectorData::FixedSequence { candidates, repeat } => {
                        (candidates.as_slice(), *repeat)
                    }
                    BossPatternSelectorData::PriorityEligible { .. } => {
                        return Err(boss_error(
                            object_id,
                            format!("nodes[{node_position}].selector"),
                            "PriorityEligible is approved future vocabulary but is not implemented by the v1 reference slice",
                        ));
                    }
                    BossPatternSelectorData::WeightedEligible { candidates } => {
                        if candidates.iter().any(|candidate| candidate.weight == 0) {
                            return Err(boss_error(
                                object_id,
                                format!("nodes[{node_position}].selector"),
                                "WeightedEligible candidate weight must be greater than zero",
                            ));
                        }
                        return Err(boss_error(
                            object_id,
                            format!("nodes[{node_position}].selector"),
                            "WeightedEligible is approved future vocabulary but is not implemented by the v1 reference slice",
                        ));
                    }
                };
                if candidate_data.is_empty() {
                    return Err(boss_error(
                        object_id,
                        format!("nodes[{node_position}].selector.candidates"),
                        "FixedSequence must contain at least one Pattern candidate",
                    ));
                }
                let mut candidate_ids = HashSet::new();
                let mut candidates = Vec::with_capacity(candidate_data.len());
                for (candidate_position, candidate) in candidate_data.iter().enumerate() {
                    if !candidate_ids.insert(candidate.pattern_id.clone()) {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "nodes[{node_position}].selector.candidates[{candidate_position}].pattern_id"
                            ),
                            format!(
                                "duplicate Pattern candidate '{}'",
                                candidate.pattern_id.as_str()
                            ),
                        ));
                    }
                    let pattern = patterns.get(&candidate.pattern_id).cloned().ok_or_else(|| {
                        boss_error(
                            object_id,
                            format!(
                                "nodes[{node_position}].selector.candidates[{candidate_position}].pattern_id"
                            ),
                            format!(
                                "references missing Pattern '{}'",
                                candidate.pattern_id.as_str()
                            ),
                        )
                    })?;
                    let argument_frame = Arc::new(resolve_argument_frame(
                        &pattern,
                        &candidate.arguments,
                        object_id,
                        &format!(
                            "nodes[{node_position}].selector.candidates[{candidate_position}].arguments"
                        ),
                    )?);
                    if let Some(cooldown) = &candidate.cooldown {
                        validate_stable_id(
                            cooldown.group_id.as_str(),
                            object_id,
                            &format!(
                                "nodes[{node_position}].selector.candidates[{candidate_position}].cooldown.group_id"
                            ),
                        )?;
                        if cooldown.duration_ms == 0 {
                            return Err(boss_error(
                                object_id,
                                format!(
                                    "nodes[{node_position}].selector.candidates[{candidate_position}].cooldown.duration_ms"
                                ),
                                "cooldown duration must be greater than zero",
                            ));
                        }
                        if let Some(existing) = cooldown_durations
                            .insert(cooldown.group_id.clone(), cooldown.duration_ms)
                        {
                            if existing != cooldown.duration_ms {
                                return Err(boss_error(
                                    object_id,
                                    format!(
                                        "nodes[{node_position}].selector.candidates[{candidate_position}].cooldown"
                                    ),
                                    format!(
                                        "cooldown group '{}' has conflicting durations {existing} and {}",
                                        cooldown.group_id.as_str(), cooldown.duration_ms
                                    ),
                                ));
                            }
                        }
                    }
                    candidates.push(CompiledBossPatternCandidate {
                        pattern,
                        argument_frame,
                        cooldown: candidate.cooldown.clone(),
                    });
                }
                let Some(&no_eligible_node_index) = node_index.get(no_eligible_node_id.as_str())
                else {
                    return Err(boss_error(
                        object_id,
                        format!("nodes[{node_position}].no_eligible_node_id"),
                        format!(
                            "references missing control node '{}'",
                            no_eligible_node_id.as_str()
                        ),
                    ));
                };
                all_adjacency[node_position].push(no_eligible_node_index);
                immediate_adjacency[node_position].push(no_eligible_node_index);

                let expected_routes = candidates
                    .iter()
                    .flat_map(|candidate| {
                        candidate
                            .pattern
                            .outcomes
                            .iter()
                            .cloned()
                            .map(move |outcome| (candidate.pattern.id.clone(), outcome))
                    })
                    .collect::<BTreeSet<_>>();
                let mut actual_routes = BTreeSet::new();
                let mut compiled_routes = Vec::with_capacity(outcome_routes.len());
                for (route_position, route) in outcome_routes.iter().enumerate() {
                    let Some(pattern) = patterns.get(&route.pattern_id) else {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "nodes[{node_position}].outcome_routes[{route_position}].pattern_id"
                            ),
                            format!("references missing Pattern '{}'", route.pattern_id.as_str()),
                        ));
                    };
                    if !candidate_ids.contains(&route.pattern_id) {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "nodes[{node_position}].outcome_routes[{route_position}].pattern_id"
                            ),
                            "outcome route Pattern is not a selector candidate",
                        ));
                    }
                    let Some(outcome_index) = pattern
                        .outcomes
                        .iter()
                        .position(|outcome| outcome == &route.outcome_id)
                    else {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "nodes[{node_position}].outcome_routes[{route_position}].outcome_id"
                            ),
                            format!(
                                "Pattern '{}' does not declare outcome '{}'",
                                route.pattern_id.as_str(),
                                route.outcome_id.as_str()
                            ),
                        ));
                    };
                    if !actual_routes.insert((route.pattern_id.clone(), route.outcome_id.clone())) {
                        return Err(boss_error(
                            object_id,
                            format!("nodes[{node_position}].outcome_routes[{route_position}]"),
                            "duplicate Pattern outcome route",
                        ));
                    }
                    let Some(&target_node_index) = node_index.get(route.target_node_id.as_str())
                    else {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "nodes[{node_position}].outcome_routes[{route_position}].target_node_id"
                            ),
                            format!(
                                "references missing control node '{}'",
                                route.target_node_id.as_str()
                            ),
                        ));
                    };
                    all_adjacency[node_position].push(target_node_index);
                    immediate_adjacency[node_position].push(target_node_index);
                    compiled_routes.push(CompiledBossPatternOutcomeRoute {
                        pattern_id: route.pattern_id.clone(),
                        outcome_index,
                        target_node_index,
                    });
                }
                if actual_routes != expected_routes {
                    return Err(boss_error(
                        object_id,
                        format!("nodes[{node_position}].outcome_routes"),
                        "Pattern outcome mapping must be exact and exhaustive",
                    ));
                }
                compiled_nodes.push(CompiledBossControlNode::PatternCall {
                    id: id.clone(),
                    candidates,
                    repeat,
                    outcome_routes: compiled_routes,
                    no_eligible_node_index,
                });
            }
        }
    }

    if reachable_from(entry_node_index, &all_adjacency).len() != behavior.nodes.len() {
        return Err(boss_error(
            object_id,
            "nodes",
            "every actor behavior control node must be reachable from entry_node_id",
        ));
    }
    if has_cycle(&immediate_adjacency) {
        return Err(boss_error(
            object_id,
            "nodes",
            "actor behavior contains a zero-time control cycle",
        ));
    }

    Ok(CompiledBossActorBehaviorDefinition {
        id: behavior.id.clone(),
        entry_node_index,
        nodes: compiled_nodes,
    })
}

fn resolve_argument_frame(
    pattern: &CompiledBossPatternDefinition,
    arguments: &[BossPatternArgumentData],
    object_id: &str,
    path: &str,
) -> Result<ResolvedBossArgumentFrame, DataValidationError> {
    let mut by_id = HashMap::new();
    for (index, argument) in arguments.iter().enumerate() {
        if by_id
            .insert(argument.parameter_id.clone(), &argument.value)
            .is_some()
        {
            return Err(boss_error(
                object_id,
                format!("{path}[{index}].parameter_id"),
                format!("duplicate argument '{}'", argument.parameter_id.as_str()),
            ));
        }
    }
    if by_id.len() != pattern.parameters.len()
        || by_id.keys().any(|id| {
            !pattern
                .parameters
                .iter()
                .any(|parameter| &parameter.id == id)
        })
    {
        return Err(boss_error(
            object_id,
            path,
            format!(
                "arguments must bind every declared Pattern parameter exactly once for '{}'",
                pattern.id.as_str()
            ),
        ));
    }

    let values = pattern
        .parameters
        .iter()
        .map(|parameter| {
            let value = by_id
                .get(&parameter.id)
                .expect("exact parameter key set was checked");
            match (&parameter.parameter_type, *value) {
                (
                    BossParameterTypeData::DurationMs { min_ms, max_ms },
                    BossParameterValueData::DurationMs(value),
                ) if value >= min_ms && value <= max_ms => {
                    Ok(ResolvedBossParameterValue::DurationMs(*value))
                }
                (
                    BossParameterTypeData::DurationMs { min_ms, max_ms },
                    BossParameterValueData::DurationMs(value),
                ) => Err(boss_error(
                    object_id,
                    path,
                    format!(
                        "DurationMs argument '{}' value {value} is outside {min_ms}..={max_ms}",
                        parameter.id.as_str()
                    ),
                )),
                (
                    BossParameterTypeData::TargetQuery,
                    BossParameterValueData::TargetQuery(query),
                ) => {
                    validate_query(query, object_id, path)?;
                    Ok(ResolvedBossParameterValue::TargetQuery(query.clone()))
                }
                (expected, actual) => Err(boss_error(
                    object_id,
                    path,
                    format!(
                        "argument '{}' has value {actual:?}, expected {expected:?}",
                        parameter.id.as_str()
                    ),
                )),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ResolvedBossArgumentFrame { values })
}

fn compile_encounter(
    encounter: &BossEncounterDefinitionData,
    actor_behaviors: &HashMap<BossActorBehaviorId, Arc<CompiledBossActorBehaviorDefinition>>,
) -> Result<CompiledBossEncounterDefinition, DataValidationError> {
    let object_id = encounter.id.as_str();
    if encounter.actor_slots.is_empty() {
        return Err(boss_error(
            object_id,
            "actor_slots",
            "Boss encounter must declare at least one actor slot",
        ));
    }
    let actor_slot_index = unique_index(
        encounter.actor_slots.iter().map(|slot| slot.0.clone()),
        object_id,
        "actor_slots",
    )?;
    if encounter.phases.is_empty() {
        return Err(boss_error(
            object_id,
            "phases",
            "Boss encounter must define at least one phase",
        ));
    }
    let phase_index = unique_index(
        encounter.phases.iter().map(|phase| phase.id.0.clone()),
        object_id,
        "phases",
    )?;
    let Some(&entry_phase_index) = phase_index.get(encounter.entry_phase_id.as_str()) else {
        return Err(boss_error(
            object_id,
            "entry_phase_id",
            format!(
                "references missing phase '{}'",
                encounter.entry_phase_id.as_str()
            ),
        ));
    };

    let expected_slots = encounter
        .actor_slots
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut all_adjacency = vec![Vec::new(); encounter.phases.len()];
    let mut immediate_adjacency = vec![Vec::new(); encounter.phases.len()];
    let mut actor_cooldown_durations =
        vec![HashMap::<BossCooldownGroupId, u64>::new(); encounter.actor_slots.len()];
    let mut compiled_phases = Vec::with_capacity(encounter.phases.len());

    for (phase_position, phase) in encounter.phases.iter().enumerate() {
        let mut actual_slots = BTreeSet::new();
        let mut compiled_actor_behaviors = Vec::with_capacity(phase.actor_behaviors.len());
        for (binding_position, binding) in phase.actor_behaviors.iter().enumerate() {
            let Some(&slot_index) = actor_slot_index.get(binding.actor_slot_id.as_str()) else {
                return Err(boss_error(
                    object_id,
                    format!(
                        "phases[{phase_position}].actor_behaviors[{binding_position}].actor_slot_id"
                    ),
                    format!(
                        "references undeclared actor slot '{}'",
                        binding.actor_slot_id.as_str()
                    ),
                ));
            };
            if !actual_slots.insert(binding.actor_slot_id.clone()) {
                return Err(boss_error(
                    object_id,
                    format!(
                        "phases[{phase_position}].actor_behaviors[{binding_position}].actor_slot_id"
                    ),
                    format!(
                        "duplicate actor slot binding '{}'",
                        binding.actor_slot_id.as_str()
                    ),
                ));
            }
            let behavior = actor_behaviors
                .get(&binding.behavior_id)
                .cloned()
                .ok_or_else(|| {
                    boss_error(
                        object_id,
                        format!(
                            "phases[{phase_position}].actor_behaviors[{binding_position}].behavior_id"
                        ),
                        format!(
                            "references missing actor behavior '{}'",
                            binding.behavior_id.as_str()
                        ),
                    )
                })?;
            for node in &behavior.nodes {
                let CompiledBossControlNode::PatternCall { candidates, .. } = node else {
                    continue;
                };
                for candidate in candidates {
                    let Some(cooldown) = &candidate.cooldown else {
                        continue;
                    };
                    if let Some(existing) = actor_cooldown_durations[slot_index]
                        .insert(cooldown.group_id.clone(), cooldown.duration_ms)
                    {
                        if existing != cooldown.duration_ms {
                            return Err(boss_error(
                                object_id,
                                format!(
                                    "phases[{phase_position}].actor_behaviors[{binding_position}].behavior_id"
                                ),
                                format!(
                                    "actor slot '{}' uses cooldown group '{}' with conflicting durations {existing} and {} across phases",
                                    binding.actor_slot_id.as_str(),
                                    cooldown.group_id.as_str(),
                                    cooldown.duration_ms
                                ),
                            ));
                        }
                    }
                }
            }
            compiled_actor_behaviors.push(CompiledBossPhaseActorBehavior {
                actor_slot_index: slot_index,
                behavior,
            });
        }
        if actual_slots != expected_slots {
            return Err(boss_error(
                object_id,
                format!("phases[{phase_position}].actor_behaviors"),
                "every phase must bind every declared actor slot exactly once",
            ));
        }
        compiled_actor_behaviors.sort_by_key(|binding| binding.actor_slot_index);

        let mut transition_ids = HashSet::new();
        let mut priorities = HashSet::new();
        let mut compiled_transitions = Vec::with_capacity(phase.transitions.len());
        for (transition_position, transition) in phase.transitions.iter().enumerate() {
            validate_stable_id(
                transition.id.as_str(),
                object_id,
                &format!("phases[{phase_position}].transitions[{transition_position}].id"),
            )?;
            if !transition_ids.insert(transition.id.clone()) {
                return Err(boss_error(
                    object_id,
                    format!("phases[{phase_position}].transitions[{transition_position}].id"),
                    format!("duplicate transition id '{}'", transition.id.as_str()),
                ));
            }
            if !priorities.insert(transition.priority) {
                return Err(boss_error(
                    object_id,
                    format!("phases[{phase_position}].transitions[{transition_position}].priority"),
                    format!(
                        "transition priority {} is not unique in this phase",
                        transition.priority
                    ),
                ));
            }
            match &transition.condition {
                BossEncounterConditionData::ActorHealthAtOrBelowPercent {
                    actor_slot_id,
                    percent,
                } => {
                    if !actor_slot_index.contains_key(actor_slot_id.as_str()) {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "phases[{phase_position}].transitions[{transition_position}].condition.actor_slot_id"
                            ),
                            format!(
                                "references undeclared actor slot '{}'",
                                actor_slot_id.as_str()
                            ),
                        ));
                    }
                    if *percent == 0 || *percent > 100 {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "phases[{phase_position}].transitions[{transition_position}].condition.percent"
                            ),
                            "health percent must be in 1..=100",
                        ));
                    }
                }
                BossEncounterConditionData::ActorDefeated { actor_slot_id } => {
                    if !actor_slot_index.contains_key(actor_slot_id.as_str()) {
                        return Err(boss_error(
                            object_id,
                            format!(
                                "phases[{phase_position}].transitions[{transition_position}].condition.actor_slot_id"
                            ),
                            format!(
                                "references undeclared actor slot '{}'",
                                actor_slot_id.as_str()
                            ),
                        ));
                    }
                }
                BossEncounterConditionData::Always => {}
            }
            let Some(&target_phase_index) = phase_index.get(transition.target_phase_id.as_str())
            else {
                return Err(boss_error(
                    object_id,
                    format!(
                        "phases[{phase_position}].transitions[{transition_position}].target_phase_id"
                    ),
                    format!(
                        "references missing phase '{}'",
                        transition.target_phase_id.as_str()
                    ),
                ));
            };
            all_adjacency[phase_position].push(target_phase_index);
            if transition.condition.is_immediate() {
                immediate_adjacency[phase_position].push(target_phase_index);
            }
            compiled_transitions.push(CompiledBossEncounterPhaseTransition {
                id: transition.id.clone(),
                priority: transition.priority,
                condition: transition.condition.clone(),
                timing: transition.timing,
                target_phase_index,
            });
        }
        compiled_phases.push(CompiledBossEncounterPhase {
            id: phase.id.clone(),
            actor_behaviors: compiled_actor_behaviors,
            transitions: compiled_transitions,
        });
    }

    if reachable_from(entry_phase_index, &all_adjacency).len() != encounter.phases.len() {
        return Err(boss_error(
            object_id,
            "phases",
            "every Boss encounter phase must be reachable from entry_phase_id",
        ));
    }
    if has_cycle(&immediate_adjacency) {
        return Err(boss_error(
            object_id,
            "phases",
            "Boss encounter contains a zero-time phase cycle",
        ));
    }

    Ok(CompiledBossEncounterDefinition {
        id: encounter.id.clone(),
        actor_slots: encounter.actor_slots.clone(),
        entry_phase_index,
        phases: compiled_phases,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::game::data::{
        abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
        summoned_unit_data::{SummonedUnitDefinitionData, SummonerRemovalPolicyData},
    };

    fn target_query() -> BossTargetQueryData {
        BossTargetQueryData {
            target_kind: BossTargetKindData::EnemyUnit,
            filters: vec![
                BossTargetFilterData::ActiveHostile,
                BossTargetFilterData::Reachable,
            ],
            order_by: vec![BossTargetOrderKeyData::MinPathCost],
            lock_policy: BossTargetLockPolicyData::LockUntilInvalid,
        }
    }

    fn result_transition(
        id: &str,
        priority: i32,
        task_id: &str,
        result: BossTaskResultData,
        target: BossPatternTransitionTargetData,
    ) -> BossPatternTransitionData {
        BossPatternTransitionData {
            id: BossTransitionId::new(id),
            priority,
            condition: BossConditionData::TaskResult {
                task_id: BossTaskId::new(task_id),
                result,
            },
            target,
            loop_guard: None,
        }
    }

    pub(crate) fn valid_catalog() -> BossRuntimeCatalogData {
        let acquire = BossPatternStateData {
            id: BossPatternStateId::new("acquire"),
            tasks: vec![BossTaskData::AcquireTarget {
                id: BossTaskId::new("acquire_target"),
                query: BossTargetQueryValueData::Parameter(BossParameterId::new("target_query")),
            }],
            transitions: vec![
                result_transition(
                    "acquired",
                    20,
                    "acquire_target",
                    BossTaskResultData::Acquired,
                    BossPatternTransitionTargetData::State(BossPatternStateId::new("approach")),
                ),
                result_transition(
                    "no_target",
                    10,
                    "acquire_target",
                    BossTaskResultData::NoEligibleTarget,
                    BossPatternTransitionTargetData::Outcome(BossPatternOutcomeId::new(
                        "target_missing",
                    )),
                ),
            ],
        };
        let approach = BossPatternStateData {
            id: BossPatternStateId::new("approach"),
            tasks: vec![BossTaskData::MoveToTarget {
                id: BossTaskId::new("move_to_target"),
                timeout_ms: BossDurationValueData::Parameter(BossParameterId::new("timeout_ms")),
            }],
            transitions: vec![
                result_transition(
                    "reached_target",
                    40,
                    "move_to_target",
                    BossTaskResultData::Reached,
                    BossPatternTransitionTargetData::State(BossPatternStateId::new("rejoin")),
                ),
                result_transition(
                    "path_unavailable",
                    30,
                    "move_to_target",
                    BossTaskResultData::PathUnavailable,
                    BossPatternTransitionTargetData::State(BossPatternStateId::new("rejoin")),
                ),
                result_transition(
                    "target_invalidated",
                    20,
                    "move_to_target",
                    BossTaskResultData::TargetInvalidated,
                    BossPatternTransitionTargetData::State(BossPatternStateId::new("rejoin")),
                ),
                result_transition(
                    "approach_timeout",
                    10,
                    "move_to_target",
                    BossTaskResultData::TimedOut,
                    BossPatternTransitionTargetData::State(BossPatternStateId::new("rejoin")),
                ),
            ],
        };
        let rejoin = BossPatternStateData {
            id: BossPatternStateId::new("rejoin"),
            tasks: vec![BossTaskData::RejoinForwardRoute {
                id: BossTaskId::new("rejoin_route"),
                timeout_ms: BossDurationValueData::Parameter(BossParameterId::new("timeout_ms")),
            }],
            transitions: vec![
                result_transition(
                    "rejoined",
                    30,
                    "rejoin_route",
                    BossTaskResultData::Reached,
                    BossPatternTransitionTargetData::Outcome(BossPatternOutcomeId::new("complete")),
                ),
                result_transition(
                    "rejoin_unavailable",
                    20,
                    "rejoin_route",
                    BossTaskResultData::PathUnavailable,
                    BossPatternTransitionTargetData::Outcome(BossPatternOutcomeId::new(
                        "route_unavailable",
                    )),
                ),
                result_transition(
                    "rejoin_timeout",
                    10,
                    "rejoin_route",
                    BossTaskResultData::TimedOut,
                    BossPatternTransitionTargetData::Outcome(BossPatternOutcomeId::new(
                        "route_unavailable",
                    )),
                ),
            ],
        };
        let pattern = BossPatternDefinitionData {
            id: BossPatternId::new("track_and_rejoin"),
            parameters: vec![
                BossParameterDefinitionData {
                    id: BossParameterId::new("target_query"),
                    parameter_type: BossParameterTypeData::TargetQuery,
                },
                BossParameterDefinitionData {
                    id: BossParameterId::new("timeout_ms"),
                    parameter_type: BossParameterTypeData::DurationMs {
                        min_ms: 1,
                        max_ms: 60_000,
                    },
                },
            ],
            outcomes: vec![
                BossPatternOutcomeId::new("complete"),
                BossPatternOutcomeId::new("target_missing"),
                BossPatternOutcomeId::new("route_unavailable"),
            ],
            entry_state_id: BossPatternStateId::new("acquire"),
            states: vec![acquire, approach, rejoin],
        };
        let candidate = BossPatternCandidateData {
            pattern_id: pattern.id.clone(),
            arguments: vec![
                BossPatternArgumentData {
                    parameter_id: BossParameterId::new("target_query"),
                    value: BossParameterValueData::TargetQuery(target_query()),
                },
                BossPatternArgumentData {
                    parameter_id: BossParameterId::new("timeout_ms"),
                    value: BossParameterValueData::DurationMs(5_000),
                },
            ],
            cooldown: Some(BossPatternCooldownData {
                group_id: BossCooldownGroupId::new("tracking"),
                duration_ms: 2_000,
            }),
        };
        let behavior = BossActorBehaviorDefinitionData {
            id: BossActorBehaviorId::new("tracking_behavior"),
            entry_node_id: BossControlNodeId::new("pattern_call"),
            nodes: vec![
                BossControlNodeData::PatternCall {
                    id: BossControlNodeId::new("pattern_call"),
                    selector: BossPatternSelectorData::FixedSequence {
                        candidates: vec![candidate],
                        repeat: false,
                    },
                    outcome_routes: pattern
                        .outcomes
                        .iter()
                        .cloned()
                        .map(|outcome_id| BossPatternOutcomeRouteData {
                            pattern_id: pattern.id.clone(),
                            outcome_id,
                            target_node_id: BossControlNodeId::new("route_combat"),
                        })
                        .collect(),
                    no_eligible_node_id: BossControlNodeId::new("route_combat"),
                },
                BossControlNodeData::State {
                    id: BossControlNodeId::new("route_combat"),
                    tasks: vec![BossTaskData::RouteCombat {
                        id: BossTaskId::new("route_combat_task"),
                    }],
                    transitions: vec![],
                },
            ],
        };
        let encounter = BossEncounterDefinitionData {
            id: BossEncounterRuntimeId::new("tracking_encounter"),
            actor_slots: vec![BossActorSlotId::new("primary")],
            entry_phase_id: BossEncounterPhaseId::new("phase_one"),
            phases: vec![BossEncounterPhaseData {
                id: BossEncounterPhaseId::new("phase_one"),
                actor_behaviors: vec![BossPhaseActorBehaviorData {
                    actor_slot_id: BossActorSlotId::new("primary"),
                    behavior_id: behavior.id.clone(),
                }],
                transitions: vec![],
            }],
        };
        BossRuntimeCatalogData {
            patterns: vec![pattern],
            actor_behaviors: vec![behavior],
            encounters: vec![encounter],
            signals: vec![],
        }
    }

    pub(crate) fn summon_catalog_fixture() -> (BossRuntimeCatalogData, SummonedUnitCatalogData) {
        let mut catalog = valid_catalog();
        let handle = SummonBatchHandleId::new("fixture_batch");
        let summon_task = BossTaskData::Summon {
            id: BossTaskId::new("summon_fixture"),
            definition_id: SummonedUnitDefinitionId::new("fixture_minion"),
            count: 2,
            batch_handle_id: handle.clone(),
            placement: BossSummonPlacementData::AroundSummoner { radius_tiles: 2 },
            occupancy_policy: BossSummonOccupancyPolicyData::RequireUnoccupied,
            on_insufficient_space: BossSummonInsufficientSpaceData::Fail,
            route_id: Some("fixture_route".to_string()),
            lifetime: None,
            removal_signal: None,
        };
        let result = |id: &str, priority: i32, result, target| {
            result_transition(id, priority, "summon_fixture", result, target)
        };
        catalog.patterns.push(BossPatternDefinitionData {
            id: BossPatternId::new("summon_fixture_pattern"),
            parameters: vec![],
            outcomes: vec![
                BossPatternOutcomeId::new("complete"),
                BossPatternOutcomeId::new("failed"),
            ],
            entry_state_id: BossPatternStateId::new("summon"),
            states: vec![
                BossPatternStateData {
                    id: BossPatternStateId::new("summon"),
                    tasks: vec![summon_task],
                    transitions: vec![
                        result(
                            "spawned",
                            40,
                            BossTaskResultData::Spawned,
                            BossPatternTransitionTargetData::State(BossPatternStateId::new(
                                "wait_clear",
                            )),
                        ),
                        result(
                            "limit_reached",
                            30,
                            BossTaskResultData::LimitReached,
                            BossPatternTransitionTargetData::Outcome(BossPatternOutcomeId::new(
                                "failed",
                            )),
                        ),
                        result(
                            "no_space",
                            20,
                            BossTaskResultData::NoSpawnSpace,
                            BossPatternTransitionTargetData::Outcome(BossPatternOutcomeId::new(
                                "failed",
                            )),
                        ),
                        result(
                            "canceled",
                            10,
                            BossTaskResultData::Canceled,
                            BossPatternTransitionTargetData::Outcome(BossPatternOutcomeId::new(
                                "failed",
                            )),
                        ),
                    ],
                },
                BossPatternStateData {
                    id: BossPatternStateId::new("wait_clear"),
                    tasks: vec![],
                    transitions: vec![BossPatternTransitionData {
                        id: BossTransitionId::new("batch_cleared"),
                        priority: 10,
                        condition: BossConditionData::SummonBatchCleared {
                            batch_handle_id: handle,
                        },
                        target: BossPatternTransitionTargetData::Outcome(
                            BossPatternOutcomeId::new("complete"),
                        ),
                        loop_guard: None,
                    }],
                },
            ],
        });
        let summon_pattern = catalog.patterns.last().unwrap().clone();
        let summoning_behavior = BossActorBehaviorDefinitionData {
            id: BossActorBehaviorId::new("summoning_behavior"),
            entry_node_id: BossControlNodeId::new("summon_pattern_call"),
            nodes: vec![
                BossControlNodeData::PatternCall {
                    id: BossControlNodeId::new("summon_pattern_call"),
                    selector: BossPatternSelectorData::FixedSequence {
                        candidates: vec![BossPatternCandidateData {
                            pattern_id: summon_pattern.id.clone(),
                            arguments: vec![],
                            cooldown: None,
                        }],
                        repeat: false,
                    },
                    outcome_routes: summon_pattern
                        .outcomes
                        .iter()
                        .cloned()
                        .map(|outcome_id| BossPatternOutcomeRouteData {
                            pattern_id: summon_pattern.id.clone(),
                            outcome_id,
                            target_node_id: BossControlNodeId::new("summoner_route_combat"),
                        })
                        .collect(),
                    no_eligible_node_id: BossControlNodeId::new("summoner_route_combat"),
                },
                BossControlNodeData::State {
                    id: BossControlNodeId::new("summoner_route_combat"),
                    tasks: vec![BossTaskData::RouteCombat {
                        id: BossTaskId::new("summoner_route_combat_task"),
                    }],
                    transitions: vec![],
                },
            ],
        };
        catalog.actor_behaviors.push(summoning_behavior.clone());
        catalog.encounters[0].phases[0].actor_behaviors[0].behavior_id = summoning_behavior.id;
        let summoned = SummonedUnitCatalogData {
            objects: Vec::new(),
            definitions: vec![SummonedUnitDefinitionData {
                id: SummonedUnitDefinitionId::new("fixture_minion"),
                base_uuid: Uuid::from_u128(0x5355_4d4d_4f4e),
                display_name: "Fixture Minion".to_string(),
                max_health: 100,
                attack: 12,
                defense: 3,
                magic_resistance: 4,
                movement: MovementDef::default(),
                basic_attack: BasicAttackDef::default(),
                resonance: ResonanceDef::default(),
                skill_id: None,
                max_alive: 2,
                max_total: 4,
                on_summoner_removed: SummonerRemovalPolicyData::Persist,
                actor_behavior_id: BossActorBehaviorId::new("tracking_behavior"),
            }],
        };
        (catalog, summoned)
    }

    pub(crate) fn summon_task_mut(catalog: &mut BossRuntimeCatalogData) -> &mut BossTaskData {
        &mut catalog
            .patterns
            .iter_mut()
            .find(|pattern| pattern.id.as_str() == "summon_fixture_pattern")
            .expect("summon fixture pattern")
            .states[0]
            .tasks[0]
    }

    #[test]
    fn summon_catalog_compiles_strict_definition_handle_and_batch_clear_contracts() {
        let (catalog, summoned) = summon_catalog_fixture();
        let compiled = catalog
            .compile_with_summoned_units(&summoned)
            .expect("valid summon fixture");
        assert_eq!(compiled.summoned_unit_count(), 1);
        assert!(compiled
            .get_summoned_unit(&SummonedUnitDefinitionId::new("fixture_minion"))
            .is_some());
    }

    #[test]
    fn summon_catalog_rejects_invalid_count_placement_occupancy_and_wait_matrix() {
        let cases: Vec<(&str, Box<dyn Fn(&mut BossTaskData)>)> = vec![
            (
                "count must be greater than zero",
                Box::new(|task| {
                    let BossTaskData::Summon { count, .. } = task else {
                        unreachable!()
                    };
                    *count = 0;
                }),
            ),
            (
                "radius_tiles must be greater than zero",
                Box::new(|task| {
                    let BossTaskData::Summon { placement, .. } = task else {
                        unreachable!()
                    };
                    *placement = BossSummonPlacementData::AroundSummoner { radius_tiles: 0 };
                }),
            ),
            (
                "non-empty allowed set",
                Box::new(|task| {
                    let BossTaskData::Summon {
                        occupancy_policy, ..
                    } = task
                    else {
                        unreachable!()
                    };
                    *occupancy_policy =
                        BossSummonOccupancyPolicyData::AllowOccupied { allowed: vec![] };
                }),
            ),
            (
                "values must be unique",
                Box::new(|task| {
                    let BossTaskData::Summon {
                        occupancy_policy, ..
                    } = task
                    else {
                        unreachable!()
                    };
                    *occupancy_policy = BossSummonOccupancyPolicyData::AllowOccupied {
                        allowed: vec![
                            BossSummonOccupancyClassData::Summoner,
                            BossSummonOccupancyClassData::Summoner,
                        ],
                    };
                }),
            ),
            (
                "Any must be the only",
                Box::new(|task| {
                    let BossTaskData::Summon {
                        occupancy_policy, ..
                    } = task
                    else {
                        unreachable!()
                    };
                    *occupancy_policy = BossSummonOccupancyPolicyData::AllowOccupied {
                        allowed: vec![
                            BossSummonOccupancyClassData::Any,
                            BossSummonOccupancyClassData::OpposingCombatant,
                        ],
                    };
                }),
            ),
            (
                "Summoner is redundant",
                Box::new(|task| {
                    let BossTaskData::Summon {
                        occupancy_policy, ..
                    } = task
                    else {
                        unreachable!()
                    };
                    *occupancy_policy = BossSummonOccupancyPolicyData::AllowOccupied {
                        allowed: vec![
                            BossSummonOccupancyClassData::Summoner,
                            BossSummonOccupancyClassData::SameFaction,
                        ],
                    };
                }),
            ),
            (
                "AllowOccupied cannot use WaitUntilAvailable",
                Box::new(|task| {
                    let BossTaskData::Summon {
                        occupancy_policy,
                        on_insufficient_space,
                        ..
                    } = task
                    else {
                        unreachable!()
                    };
                    *occupancy_policy = BossSummonOccupancyPolicyData::AllowOccupied {
                        allowed: vec![BossSummonOccupancyClassData::Any],
                    };
                    *on_insufficient_space = BossSummonInsufficientSpaceData::WaitUntilAvailable {
                        timeout_ms: BossDurationValueData::Literal(10),
                    };
                }),
            ),
            (
                "duration must be greater than zero",
                Box::new(|task| {
                    let BossTaskData::Summon {
                        on_insufficient_space,
                        ..
                    } = task
                    else {
                        unreachable!()
                    };
                    *on_insufficient_space = BossSummonInsufficientSpaceData::WaitUntilAvailable {
                        timeout_ms: BossDurationValueData::Literal(0),
                    };
                }),
            ),
        ];
        for (expected, mutate) in cases {
            let (mut catalog, summoned) = summon_catalog_fixture();
            mutate(summon_task_mut(&mut catalog));
            let error = catalog
                .compile_with_summoned_units(&summoned)
                .expect_err("invalid summon contract must reject");
            assert!(
                error.cause.contains(expected),
                "expected '{expected}', got {error:?}"
            );
        }
    }

    #[test]
    fn summon_catalog_rejects_missing_definition_behavior_route_and_handle() {
        let (mut missing_definition, summoned) = summon_catalog_fixture();
        let BossTaskData::Summon { definition_id, .. } = summon_task_mut(&mut missing_definition)
        else {
            unreachable!()
        };
        *definition_id = SummonedUnitDefinitionId::new("missing_minion");
        assert!(missing_definition
            .compile_with_summoned_units(&summoned)
            .unwrap_err()
            .cause
            .contains("missing summoned-unit definition"));

        let (catalog, mut missing_behavior) = summon_catalog_fixture();
        missing_behavior.definitions[0].actor_behavior_id =
            BossActorBehaviorId::new("missing_behavior");
        assert!(catalog
            .compile_with_summoned_units(&missing_behavior)
            .unwrap_err()
            .cause
            .contains("missing actor behavior"));

        let (mut missing_route, summoned) = summon_catalog_fixture();
        let BossTaskData::Summon { route_id, .. } = summon_task_mut(&mut missing_route) else {
            unreachable!()
        };
        *route_id = None;
        assert!(missing_route
            .compile_with_summoned_units(&summoned)
            .unwrap_err()
            .cause
            .contains("requires an explicit route"));

        let (mut missing_handle, summoned) = summon_catalog_fixture();
        let pattern = missing_handle
            .patterns
            .iter_mut()
            .find(|pattern| pattern.id.as_str() == "summon_fixture_pattern")
            .unwrap();
        let BossConditionData::SummonBatchCleared { batch_handle_id } =
            &mut pattern.states[1].transitions[0].condition
        else {
            unreachable!()
        };
        *batch_handle_id = SummonBatchHandleId::new("unbound_batch");
        assert!(missing_handle
            .compile_with_summoned_units(&summoned)
            .unwrap_err()
            .cause
            .contains("unproduced batch handle"));
    }

    #[test]
    fn summon_catalog_rejects_recursive_behavior_and_unproduced_removal_signal() {
        let (mut recursive, mut summoned) = summon_catalog_fixture();
        let pattern = recursive
            .patterns
            .iter()
            .find(|pattern| pattern.id.as_str() == "summon_fixture_pattern")
            .unwrap();
        let recursive_behavior = BossActorBehaviorDefinitionData {
            id: BossActorBehaviorId::new("recursive_minion_behavior"),
            entry_node_id: BossControlNodeId::new("call_summon"),
            nodes: vec![
                BossControlNodeData::PatternCall {
                    id: BossControlNodeId::new("call_summon"),
                    selector: BossPatternSelectorData::FixedSequence {
                        candidates: vec![BossPatternCandidateData {
                            pattern_id: pattern.id.clone(),
                            arguments: vec![],
                            cooldown: None,
                        }],
                        repeat: false,
                    },
                    outcome_routes: pattern
                        .outcomes
                        .iter()
                        .cloned()
                        .map(|outcome_id| BossPatternOutcomeRouteData {
                            pattern_id: pattern.id.clone(),
                            outcome_id,
                            target_node_id: BossControlNodeId::new("idle"),
                        })
                        .collect(),
                    no_eligible_node_id: BossControlNodeId::new("idle"),
                },
                BossControlNodeData::State {
                    id: BossControlNodeId::new("idle"),
                    tasks: vec![BossTaskData::Wait {
                        id: BossTaskId::new("idle_wait"),
                        duration_ms: BossDurationValueData::Literal(1_000),
                    }],
                    transitions: vec![],
                },
            ],
        };
        summoned.definitions[0].actor_behavior_id = recursive_behavior.id.clone();
        recursive.actor_behaviors.push(recursive_behavior);
        let BossTaskData::Summon { route_id, .. } = summon_task_mut(&mut recursive) else {
            unreachable!()
        };
        *route_id = None;
        assert!(recursive
            .compile_with_summoned_units(&summoned)
            .unwrap_err()
            .cause
            .contains("must not contain a Summon task"));

        let (mut unproduced, summoned) = summon_catalog_fixture();
        unproduced.signals.push(BossSignalDefinitionData {
            id: BossSignalId::new("expire_minions"),
        });
        let BossTaskData::Summon { removal_signal, .. } = summon_task_mut(&mut unproduced) else {
            unreachable!()
        };
        *removal_signal = Some(BossSummonRemovalSignalData {
            signal_id: BossSignalId::new("expire_minions"),
            mode: BossSummonRemovalModeData::Despawn,
        });
        assert!(unproduced
            .compile_with_summoned_units(&summoned)
            .unwrap_err()
            .cause
            .contains("has no Boss Runtime producer"));

        let (mut unbound, summoned) = summon_catalog_fixture();
        let signal_id = BossSignalId::new("unbound_expire_minions");
        unbound.signals.push(BossSignalDefinitionData {
            id: signal_id.clone(),
        });
        unbound
            .actor_behaviors
            .push(BossActorBehaviorDefinitionData {
                id: BossActorBehaviorId::new("unbound_signal_producer"),
                entry_node_id: BossControlNodeId::new("emit"),
                nodes: vec![
                    BossControlNodeData::State {
                        id: BossControlNodeId::new("emit"),
                        tasks: vec![BossTaskData::EmitSignal {
                            id: BossTaskId::new("emit_unbound_signal"),
                            signal_id: signal_id.clone(),
                        }],
                        transitions: vec![BossControlTransitionData {
                            id: BossTransitionId::new("unbound_signal_emitted"),
                            priority: 10,
                            condition: BossConditionData::TaskResult {
                                task_id: BossTaskId::new("emit_unbound_signal"),
                                result: BossTaskResultData::Completed,
                            },
                            target_node_id: BossControlNodeId::new("idle"),
                        }],
                    },
                    BossControlNodeData::State {
                        id: BossControlNodeId::new("idle"),
                        tasks: vec![BossTaskData::RouteCombat {
                            id: BossTaskId::new("idle_route"),
                        }],
                        transitions: vec![],
                    },
                ],
            });
        let BossTaskData::Summon { removal_signal, .. } = summon_task_mut(&mut unbound) else {
            unreachable!()
        };
        *removal_signal = Some(BossSummonRemovalSignalData {
            signal_id,
            mode: BossSummonRemovalModeData::Despawn,
        });
        assert!(unbound
            .compile_with_summoned_units(&summoned)
            .unwrap_err()
            .cause
            .contains("not bound by any encounter"));
    }

    #[test]
    fn valid_catalog_compiles_to_indexed_graphs_and_isolated_typed_frames() {
        let mut catalog = valid_catalog();
        let mut second_behavior = catalog.actor_behaviors[0].clone();
        second_behavior.id = BossActorBehaviorId::new("tracking_behavior_slow");
        let BossControlNodeData::PatternCall { selector, .. } = &mut second_behavior.nodes[0]
        else {
            unreachable!()
        };
        let BossPatternSelectorData::FixedSequence { candidates, .. } = selector else {
            unreachable!()
        };
        candidates[0].arguments[1].value = BossParameterValueData::DurationMs(9_000);
        catalog.actor_behaviors.push(second_behavior);

        let compiled = catalog.compile().expect("valid catalog should compile");
        assert_eq!(compiled.pattern_count(), 1);
        assert_eq!(compiled.actor_behavior_count(), 2);
        assert_eq!(compiled.encounter_count(), 1);
        let first = compiled
            .get_actor_behavior(&BossActorBehaviorId::new("tracking_behavior"))
            .unwrap();
        let second = compiled
            .get_actor_behavior(&BossActorBehaviorId::new("tracking_behavior_slow"))
            .unwrap();
        let CompiledBossControlNode::PatternCall { candidates, .. } = &first.nodes[0] else {
            unreachable!()
        };
        let CompiledBossControlNode::PatternCall {
            candidates: second_candidates,
            ..
        } = &second.nodes[0]
        else {
            unreachable!()
        };
        assert_eq!(
            candidates[0].argument_frame.values[1],
            ResolvedBossParameterValue::DurationMs(5_000)
        );
        assert_eq!(
            second_candidates[0].argument_frame.values[1],
            ResolvedBossParameterValue::DurationMs(9_000)
        );
        assert!(Arc::ptr_eq(
            &candidates[0].pattern,
            &second_candidates[0].pattern
        ));
        assert!(!Arc::ptr_eq(
            &candidates[0].argument_frame,
            &second_candidates[0].argument_frame
        ));
    }

    #[test]
    fn catalog_rejects_duplicate_dangling_unsupported_and_bad_parameter_contracts() {
        let cases: Vec<(&str, Box<dyn Fn(&mut BossRuntimeCatalogData)>)> = vec![
            (
                "duplicate id",
                Box::new(|catalog| catalog.patterns.push(catalog.patterns[0].clone())),
            ),
            (
                "missing Pattern",
                Box::new(|catalog| {
                    let BossControlNodeData::PatternCall { selector, .. } =
                        &mut catalog.actor_behaviors[0].nodes[0]
                    else {
                        unreachable!()
                    };
                    let BossPatternSelectorData::FixedSequence { candidates, .. } = selector else {
                        unreachable!()
                    };
                    candidates[0].pattern_id = BossPatternId::new("missing_pattern");
                }),
            ),
            (
                "not implemented",
                Box::new(|catalog| {
                    let BossControlNodeData::PatternCall { selector, .. } =
                        &mut catalog.actor_behaviors[0].nodes[0]
                    else {
                        unreachable!()
                    };
                    let BossPatternSelectorData::FixedSequence { candidates, .. } =
                        selector.clone()
                    else {
                        unreachable!()
                    };
                    *selector = BossPatternSelectorData::PriorityEligible { candidates };
                }),
            ),
            (
                "arguments must bind",
                Box::new(|catalog| {
                    let BossControlNodeData::PatternCall { selector, .. } =
                        &mut catalog.actor_behaviors[0].nodes[0]
                    else {
                        unreachable!()
                    };
                    let BossPatternSelectorData::FixedSequence { candidates, .. } = selector else {
                        unreachable!()
                    };
                    candidates[0].arguments.pop();
                }),
            ),
            (
                "outside",
                Box::new(|catalog| {
                    let BossControlNodeData::PatternCall { selector, .. } =
                        &mut catalog.actor_behaviors[0].nodes[0]
                    else {
                        unreachable!()
                    };
                    let BossPatternSelectorData::FixedSequence { candidates, .. } = selector else {
                        unreachable!()
                    };
                    candidates[0].arguments[1].value = BossParameterValueData::DurationMs(90_000);
                }),
            ),
        ];
        for (expected, mutate) in cases {
            let mut catalog = valid_catalog();
            mutate(&mut catalog);
            let error = catalog.compile().expect_err("invalid catalog must reject");
            assert!(
                error.cause.contains(expected),
                "expected '{expected}' in {:?}",
                error
            );
        }
    }

    #[test]
    fn catalog_rejects_priority_result_capability_and_graph_defects() {
        let mut duplicate_priority = valid_catalog();
        duplicate_priority.patterns[0].states[0].transitions[1].priority = 20;
        assert!(duplicate_priority
            .compile()
            .unwrap_err()
            .cause
            .contains("not unique"));

        let mut missing_result = valid_catalog();
        missing_result.patterns[0].states[1].transitions.pop();
        assert!(missing_result
            .compile()
            .unwrap_err()
            .cause
            .contains("not exhaustive"));

        let mut conflict = valid_catalog();
        conflict.patterns[0].states[1]
            .tasks
            .push(BossTaskData::RejoinForwardRoute {
                id: BossTaskId::new("also_movement"),
                timeout_ms: BossDurationValueData::Literal(1_000),
            });
        assert!(conflict
            .compile()
            .unwrap_err()
            .cause
            .contains("intrinsic capability"));

        let mut unbounded = valid_catalog();
        let transition = &mut unbounded.patterns[0].states[2].transitions[0];
        transition.target =
            BossPatternTransitionTargetData::State(BossPatternStateId::new("acquire"));
        assert!(unbounded
            .compile()
            .unwrap_err()
            .cause
            .contains("without a bounded loop guard"));

        let mut cyclic_exhaustion = valid_catalog();
        let transition = &mut cyclic_exhaustion.patterns[0].states[2].transitions[0];
        transition.target =
            BossPatternTransitionTargetData::State(BossPatternStateId::new("acquire"));
        transition.loop_guard = Some(BossLoopGuardData {
            id: BossLoopId::new("retry_limit"),
            max_iterations: 2,
            exhausted_target: BossPatternTransitionTargetData::State(BossPatternStateId::new(
                "approach",
            )),
        });
        assert!(cyclic_exhaustion
            .compile()
            .unwrap_err()
            .cause
            .contains("without a bounded loop guard"));

        let mut zero_time = valid_catalog();
        zero_time.patterns[0].states = vec![BossPatternStateData {
            id: BossPatternStateId::new("spin"),
            tasks: vec![],
            transitions: vec![BossPatternTransitionData {
                id: BossTransitionId::new("spin_again"),
                priority: 1,
                condition: BossConditionData::Always,
                target: BossPatternTransitionTargetData::State(BossPatternStateId::new("spin")),
                loop_guard: Some(BossLoopGuardData {
                    id: BossLoopId::new("spin_limit"),
                    max_iterations: 2,
                    exhausted_target: BossPatternTransitionTargetData::Outcome(
                        BossPatternOutcomeId::new("complete"),
                    ),
                }),
            }],
        }];
        zero_time.patterns[0].entry_state_id = BossPatternStateId::new("spin");
        assert!(zero_time.compile().unwrap_err().cause.contains("zero-time"));
    }

    #[test]
    fn target_query_requires_explicit_reachability_for_path_cost() {
        let mut catalog = valid_catalog();
        let BossControlNodeData::PatternCall { selector, .. } =
            &mut catalog.actor_behaviors[0].nodes[0]
        else {
            unreachable!()
        };
        let BossPatternSelectorData::FixedSequence { candidates, .. } = selector else {
            unreachable!()
        };
        let BossParameterValueData::TargetQuery(query) = &mut candidates[0].arguments[0].value
        else {
            unreachable!()
        };
        query
            .filters
            .retain(|filter| *filter != BossTargetFilterData::Reachable);
        assert!(catalog.compile().unwrap_err().cause.contains("Reachable"));
    }

    #[test]
    fn persistent_continue_state_requires_a_bounded_loop_guard() {
        let mut catalog = valid_catalog();
        catalog.patterns[0].states[1]
            .transitions
            .push(BossPatternTransitionData {
                id: BossTransitionId::new("persistent_continue"),
                priority: 50,
                condition: BossConditionData::StatePredicate(
                    BossStatePredicateData::SelfHealthAtOrBelowPercent { percent: 100 },
                ),
                target: BossPatternTransitionTargetData::ContinueState,
                loop_guard: None,
            });

        let error = catalog.compile().unwrap_err();
        assert!(
            error
                .cause
                .contains("persistent ContinueState condition requires a bounded loop guard"),
            "unexpected compiler error: {}",
            error.cause
        );
    }

    #[test]
    fn catalog_rejects_id_reference_outcome_variant_and_argument_matrix() {
        let mut invalid_id = valid_catalog();
        invalid_id.patterns[0].id = BossPatternId::new("TrackAndRejoin");
        assert!(invalid_id
            .compile()
            .unwrap_err()
            .cause
            .contains("lower_snake_case"));

        let mut dangling_state = valid_catalog();
        dangling_state.patterns[0].states[0].transitions[0].target =
            BossPatternTransitionTargetData::State(BossPatternStateId::new("missing_state"));
        assert!(dangling_state
            .compile()
            .unwrap_err()
            .cause
            .contains("missing state"));

        let mut incomplete_outcomes = valid_catalog();
        let BossControlNodeData::PatternCall { outcome_routes, .. } =
            &mut incomplete_outcomes.actor_behaviors[0].nodes[0]
        else {
            unreachable!()
        };
        outcome_routes.pop();
        assert!(incomplete_outcomes
            .compile()
            .unwrap_err()
            .cause
            .contains("exact and exhaustive"));

        let mut wrong_argument_type = valid_catalog();
        let BossControlNodeData::PatternCall { selector, .. } =
            &mut wrong_argument_type.actor_behaviors[0].nodes[0]
        else {
            unreachable!()
        };
        let BossPatternSelectorData::FixedSequence { candidates, .. } = selector else {
            unreachable!()
        };
        candidates[0].arguments[1].value = BossParameterValueData::TargetQuery(target_query());
        assert!(wrong_argument_type
            .compile()
            .unwrap_err()
            .cause
            .contains("expected DurationMs"));

        let mut duplicate_argument = valid_catalog();
        let BossControlNodeData::PatternCall { selector, .. } =
            &mut duplicate_argument.actor_behaviors[0].nodes[0]
        else {
            unreachable!()
        };
        let BossPatternSelectorData::FixedSequence { candidates, .. } = selector else {
            unreachable!()
        };
        let repeated_argument = candidates[0].arguments[0].clone();
        candidates[0].arguments.push(repeated_argument);
        assert!(duplicate_argument
            .compile()
            .unwrap_err()
            .cause
            .contains("duplicate argument"));

        let mut unsupported_task = valid_catalog();
        let BossControlNodeData::State { tasks, .. } =
            &mut unsupported_task.actor_behaviors[0].nodes[1]
        else {
            unreachable!()
        };
        tasks[0] = BossTaskData::RepathToProtectedObjective {
            id: BossTaskId::new("future_repath"),
            timeout_ms: BossDurationValueData::Literal(1_000),
        };
        assert!(unsupported_task
            .compile()
            .unwrap_err()
            .cause
            .contains("not implemented in v1"));

        let mut incomplete_phase_binding = valid_catalog();
        incomplete_phase_binding.encounters[0]
            .actor_slots
            .push(BossActorSlotId::new("secondary"));
        assert!(incomplete_phase_binding
            .compile()
            .unwrap_err()
            .cause
            .contains("every declared actor slot"));
    }

    #[test]
    fn encounter_rejects_cross_phase_cooldown_conflicts_and_zero_time_cycles() {
        let mut cooldown_conflict = valid_catalog();
        let mut second_behavior = cooldown_conflict.actor_behaviors[0].clone();
        second_behavior.id = BossActorBehaviorId::new("tracking_behavior_fast");
        let BossControlNodeData::PatternCall { selector, .. } = &mut second_behavior.nodes[0]
        else {
            unreachable!()
        };
        let BossPatternSelectorData::FixedSequence { candidates, .. } = selector else {
            unreachable!()
        };
        candidates[0].cooldown.as_mut().unwrap().duration_ms = 1_000;
        cooldown_conflict
            .actor_behaviors
            .push(second_behavior.clone());
        cooldown_conflict.encounters[0].phases[0].transitions =
            vec![BossEncounterPhaseTransitionData {
                id: BossTransitionId::new("enter_phase_two"),
                priority: 10,
                condition: BossEncounterConditionData::ActorHealthAtOrBelowPercent {
                    actor_slot_id: BossActorSlotId::new("primary"),
                    percent: 50,
                },
                timing: BossPhaseTransitionTimingData::Immediate,
                target_phase_id: BossEncounterPhaseId::new("phase_two"),
            }];
        cooldown_conflict.encounters[0]
            .phases
            .push(BossEncounterPhaseData {
                id: BossEncounterPhaseId::new("phase_two"),
                actor_behaviors: vec![BossPhaseActorBehaviorData {
                    actor_slot_id: BossActorSlotId::new("primary"),
                    behavior_id: second_behavior.id.clone(),
                }],
                transitions: vec![],
            });
        assert!(cooldown_conflict
            .compile()
            .unwrap_err()
            .cause
            .contains("conflicting durations"));

        let mut phase_cycle = valid_catalog();
        phase_cycle.encounters[0].phases[0].transitions = vec![BossEncounterPhaseTransitionData {
            id: BossTransitionId::new("enter_phase_two"),
            priority: 10,
            condition: BossEncounterConditionData::ActorHealthAtOrBelowPercent {
                actor_slot_id: BossActorSlotId::new("primary"),
                percent: 50,
            },
            timing: BossPhaseTransitionTimingData::Immediate,
            target_phase_id: BossEncounterPhaseId::new("phase_two"),
        }];
        let phase_actor_behaviors = phase_cycle.encounters[0].phases[0].actor_behaviors.clone();
        phase_cycle.encounters[0]
            .phases
            .push(BossEncounterPhaseData {
                id: BossEncounterPhaseId::new("phase_two"),
                actor_behaviors: phase_actor_behaviors,
                transitions: vec![BossEncounterPhaseTransitionData {
                    id: BossTransitionId::new("return_phase_one"),
                    priority: 10,
                    condition: BossEncounterConditionData::ActorDefeated {
                        actor_slot_id: BossActorSlotId::new("primary"),
                    },
                    timing: BossPhaseTransitionTimingData::Immediate,
                    target_phase_id: BossEncounterPhaseId::new("phase_one"),
                }],
            });
        assert!(phase_cycle
            .compile()
            .unwrap_err()
            .cause
            .contains("zero-time phase cycle"));
    }
}
