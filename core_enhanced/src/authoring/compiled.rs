//! Borrowed compiled graph views. Cross-definition edges remain IDs so a
//! shared pattern/behavior is never recursively copied into every caller.

use crate::game::data::boss_runtime_data::*;
use serde::{ser::SerializeSeq, Serialize, Serializer};

pub(super) struct Projected<'a, T, U> {
    values: &'a [T],
    project: fn(&'a T) -> U,
}
impl<T, U: Serialize> Serialize for Projected<'_, T, U> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.values.len()))?;
        for value in self.values {
            seq.serialize_element(&(self.project)(value))?;
        }
        seq.end()
    }
}

#[derive(Serialize)]
pub(super) struct ActorView<'a> {
    id: &'a BossActorBehaviorId,
    entry_node_index: usize,
    nodes: Projected<'a, CompiledBossControlNode, NodeView<'a>>,
}
impl<'a> ActorView<'a> {
    pub(super) fn new(value: &'a CompiledBossActorBehaviorDefinition) -> Self {
        Self {
            id: &value.id,
            entry_node_index: value.entry_node_index,
            nodes: Projected {
                values: &value.nodes,
                project: NodeView::new,
            },
        }
    }
}

#[derive(Serialize)]
enum NodeView<'a> {
    State {
        id: &'a BossControlNodeId,
        tasks: &'a [CompiledBossTask],
        transitions: &'a [CompiledBossControlTransition],
    },
    PatternCall {
        id: &'a BossControlNodeId,
        candidates: Projected<'a, CompiledBossPatternCandidate, CandidateView<'a>>,
        repeat: bool,
        outcome_routes: &'a [CompiledBossPatternOutcomeRoute],
        no_eligible_node_index: usize,
    },
}
impl<'a> NodeView<'a> {
    fn new(value: &'a CompiledBossControlNode) -> Self {
        match value {
            CompiledBossControlNode::State {
                id,
                tasks,
                transitions,
            } => Self::State {
                id,
                tasks,
                transitions,
            },
            CompiledBossControlNode::PatternCall {
                id,
                candidates,
                repeat,
                outcome_routes,
                no_eligible_node_index,
            } => Self::PatternCall {
                id,
                candidates: Projected {
                    values: candidates,
                    project: CandidateView::new,
                },
                repeat: *repeat,
                outcome_routes,
                no_eligible_node_index: *no_eligible_node_index,
            },
        }
    }
}

#[derive(Serialize)]
struct CandidateView<'a> {
    pattern_id: &'a BossPatternId,
    argument_frame: &'a ResolvedBossArgumentFrame,
    cooldown: &'a Option<BossPatternCooldownData>,
}
impl<'a> CandidateView<'a> {
    fn new(value: &'a CompiledBossPatternCandidate) -> Self {
        Self {
            pattern_id: &value.pattern.id,
            argument_frame: &value.argument_frame,
            cooldown: &value.cooldown,
        }
    }
}

#[derive(Serialize)]
pub(super) struct EncounterView<'a> {
    id: &'a BossEncounterRuntimeId,
    actor_slots: &'a [BossActorSlotId],
    entry_phase_index: usize,
    phases: Projected<'a, CompiledBossEncounterPhase, PhaseView<'a>>,
}
impl<'a> EncounterView<'a> {
    pub(super) fn new(value: &'a CompiledBossEncounterDefinition) -> Self {
        Self {
            id: &value.id,
            actor_slots: &value.actor_slots,
            entry_phase_index: value.entry_phase_index,
            phases: Projected {
                values: &value.phases,
                project: PhaseView::new,
            },
        }
    }
}

#[derive(Serialize)]
struct PhaseView<'a> {
    id: &'a BossEncounterPhaseId,
    actor_behaviors: Projected<'a, CompiledBossPhaseActorBehavior, BindingView<'a>>,
    transitions: &'a [CompiledBossEncounterPhaseTransition],
}
impl<'a> PhaseView<'a> {
    fn new(value: &'a CompiledBossEncounterPhase) -> Self {
        Self {
            id: &value.id,
            actor_behaviors: Projected {
                values: &value.actor_behaviors,
                project: BindingView::new,
            },
            transitions: &value.transitions,
        }
    }
}

#[derive(Serialize)]
struct BindingView<'a> {
    actor_slot_index: usize,
    behavior_id: &'a BossActorBehaviorId,
}
impl<'a> BindingView<'a> {
    fn new(value: &'a CompiledBossPhaseActorBehavior) -> Self {
        Self {
            actor_slot_index: value.actor_slot_index,
            behavior_id: &value.behavior.id,
        }
    }
}
