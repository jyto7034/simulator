//! Bounded continuation interpreter. Authored nodes and released occurrences have
//! separate identities: resetting a repeat body never invalidates its deliveries.
use super::super::{types::SkillStepResult, BattleCore};
use crate::game::ability::authoring::EffectSite;
use crate::game::battle::runtime_facts::RuntimeFactSource;
use crate::{
    authoring::control,
    game::{
        ability::{flow::*, SkillDef},
        battle::{
            enums::BattleEvent,
            event_log::{BattleEventCause, SkillCastTarget},
        },
        behavior::GameError,
    },
};
use std::{collections::BTreeMap, sync::Arc};
mod structural;
use structural::SideOccurrence;
mod results;
use results::ScopedResultWitness;

#[derive(Debug, Clone, Default)]
pub(in crate::game::battle::core) struct NodeState {
    phase: u8,
    cursor: u32,
    occurrence: Option<usize>,
    deadline: Option<u64>,
    dispatched: bool,
    result: SkillStepResult,
}
crate::authoring::clone_shape::struct_shape!(NodeState {
    phase,
    cursor,
    occurrence,
    deadline,
    dispatched,
    result
});
#[derive(Debug, Clone)]
pub(in crate::game::battle::core) struct FlowState {
    pub plan: Arc<CompiledSkillFlow>,
    pub nodes: Vec<NodeState>,
    target: Option<SkillCastTarget>,
    next_occurrence: usize,
    wake: Option<u64>,
    generation: u64,
    pub complete: bool,
    root: usize,
    // Only the original interpreter owns side branches. Child interpreters
    // cannot spawn, and their control does not extend the original use.
    side_occurrences: Option<Box<Vec<SideOccurrence>>>,
    // Cold scope exists only for explicit effect-result conditions. Its keys
    // include the producer occurrence; repeated hits and sibling effects remain
    // distinct from Action aggregates.
    effect_results: Option<Box<BTreeMap<(usize, EffectSite), SkillStepResult>>>,
    scope_results: Option<Box<BTreeMap<usize, ScopedResultWitness>>>,
}
crate::authoring::clone_shape::struct_shape!(FlowState {
    plan,
    nodes,
    target,
    next_occurrence,
    wake,
    generation,
    complete,
    root,
    side_occurrences,
    effect_results,
    scope_results
});
fn add_time(now: u64, delay: u64) -> Result<u64, GameError> {
    now.checked_add(delay).ok_or_else(|| {
        crate::game::battle::runtime_control::reject(
            crate::game::battle::runtime_control::BattleRuntimeFault {
                code: crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
                site: "flow_schedule_time",
                limit: None,
                requested: None,
            },
        )
    })
}
impl FlowState {
    pub fn accepted_target(&self) -> Option<SkillCastTarget> {
        self.target
    }
    /// Completion of a delivery may gate later authored control, but a final
    /// projectile's impact is not itself a remaining use action.
    pub fn has_use_control(&self) -> bool {
        fn pending(flow: &FlowState, node: usize) -> bool {
            let state = &flow.nodes[node];
            if state.phase == 2 {
                return false;
            }
            match &flow.plan.nodes[node].kind {
                CompiledFlowNodeKind::Action { .. } => !state.dispatched,
                CompiledFlowNodeKind::Wait { .. } => true,
                CompiledFlowNodeKind::Sequence { children } => children
                    .iter()
                    .skip(state.cursor as usize)
                    .any(|child| pending(flow, *child)),
                CompiledFlowNodeKind::Parallel { branches } => {
                    branches.iter().any(|child| pending(flow, *child))
                }
                CompiledFlowNodeKind::Branch {
                    then_node,
                    else_node,
                    ..
                } => {
                    state.phase == 0
                        || pending(
                            flow,
                            if state.cursor == 1 {
                                *then_node
                            } else {
                                *else_node
                            },
                        )
                }
                CompiledFlowNodeKind::Repeat { count, body, .. } => {
                    state.cursor.saturating_add(1) < *count || pending(flow, *body)
                }
            }
        }
        !self.complete && pending(self, self.root)
    }
    pub fn retains_occurrence(&self, occurrence: usize) -> bool {
        self.nodes
            .iter()
            .any(|state| state.occurrence == Some(occurrence))
            || self.side_occurrences.as_ref().is_some_and(|sides| {
                sides.iter().any(|side| {
                    side.anchor_occurrence == occurrence || side.flow.retains_occurrence(occurrence)
                })
            })
    }
    pub fn new(
        plan: Arc<CompiledSkillFlow>,
        target: Option<SkillCastTarget>,
    ) -> Result<Self, GameError> {
        let root = plan.root;
        Self::new_at(plan, target, root, true)
    }
    fn new_at(
        plan: Arc<CompiledSkillFlow>,
        target: Option<SkillCastTarget>,
        root: usize,
        allow_sides: bool,
    ) -> Result<Self, GameError> {
        let count = plan.nodes.len();
        control::charge(count as u64, "flow_state_initialize")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            count
                .checked_mul(std::mem::size_of::<NodeState>())
                .and_then(|n| n.checked_add(std::mem::size_of::<Self>())),
            "flow_state_storage",
        )?;
        let scoped = plan.nodes.iter().any(|node| {
            matches!(
                &node.kind,
                CompiledFlowNodeKind::Branch {
                    condition: CompiledFlowCondition::EffectCompare { .. },
                    ..
                }
            )
        });
        if scoped {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<
                    BTreeMap<(usize, EffectSite), SkillStepResult>,
                >()),
                "flow_effect_scope",
            )?;
        }
        let side_occurrences = if allow_sides && !plan.side_branches.is_empty() {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<Vec<SideOccurrence>>()),
                "flow_side_owner",
            )?;
            Some(Box::new(Vec::new()))
        } else {
            None
        };
        let scope_results = if !plan.side_branches.is_empty() {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(std::mem::size_of::<BTreeMap<usize, ScopedResultWitness>>()),
                "flow_result_scope",
            )?;
            Some(Box::new(BTreeMap::new()))
        } else {
            None
        };
        Ok(Self {
            plan,
            nodes: vec![NodeState::default(); count],
            target,
            next_occurrence: 0,
            wake: None,
            generation: 0,
            complete: false,
            root,
            side_occurrences,
            effect_results: scoped.then(|| Box::new(BTreeMap::new())),
            scope_results,
        })
    }
    fn reset(&mut self, node: usize, core: &mut BattleCore, cast: u64) -> Result<(), GameError> {
        control::charge(1, "flow_repeat_reset")?;
        if let Some(occurrence) = self.nodes[node].occurrence {
            let retained_by_side = self.side_occurrences.as_ref().is_some_and(|sides| {
                sides
                    .iter()
                    .any(|side| side.anchor_occurrence == occurrence)
            });
            if let Some(results) = &mut self.effect_results {
                control::charge(results.len() as u64, "flow_effect_scope_prune")?;
                if !retained_by_side {
                    results.retain(|(key, _), _| *key != occurrence);
                }
            }
            if let Some(cast) = core.active_skill_casts.get_mut(&cast) {
                if cast
                    .step_progress
                    .get(&occurrence)
                    .is_some_and(|p| p.is_terminal())
                    && !retained_by_side
                {
                    cast.step_progress.remove(&occurrence);
                    cast.total_steps -= 1;
                }
            }
        }
        self.nodes[node] = NodeState::default();
        // The immutable plan owner is cheap to copy and is never unrolled.
        let plan = self.plan.clone();
        match &plan.nodes[node].kind {
            CompiledFlowNodeKind::Sequence { children }
            | CompiledFlowNodeKind::Parallel { branches: children } => {
                for &child in children {
                    self.reset(child, core, cast)?;
                }
            }
            CompiledFlowNodeKind::Branch {
                then_node,
                else_node,
                ..
            } => {
                self.reset(*then_node, core, cast)?;
                self.reset(*else_node, core, cast)?;
            }
            CompiledFlowNodeKind::Repeat { body, .. } => self.reset(*body, core, cast)?,
            _ => (),
        }
        Ok(())
    }
    pub(crate) fn effect_scope_extent(&self) -> Option<usize> {
        let side_extent = self.side_extent()?.checked_add(self.scope_extent()?)?;
        let Some(results) = &self.effect_results else {
            return Some(side_extent);
        };
        control::charge(results.len() as u64, "flow_effect_scope_extent").ok()?;
        results.iter().try_fold(
            std::mem::size_of_val(results.as_ref()).checked_add(side_extent)?,
            |bytes, ((_, site), _)| {
                bytes
                    .checked_add(
                        std::mem::size_of::<((usize, EffectSite), SkillStepResult)>() + 64,
                    )?
                    .checked_add(site.action.definition_id.as_str().len())?
                    .checked_add(site.action.action_id.capacity())?
                    .checked_add(site.effect_id.capacity())
            },
        )
    }
    pub(crate) fn record_effect_result(
        &mut self,
        occurrence: usize,
        effect: &EffectSite,
        result: &SkillStepResult,
    ) -> Result<(), GameError> {
        if let Some(sides) = &mut self.side_occurrences {
            let first_generated = self
                .plan
                .side_branches
                .iter()
                .map(|b| b.root)
                .min()
                .expect("side root");
            for side in sides.iter_mut() {
                control::charge(side.flow.nodes.len() as u64, "flow_side_effect_owner")?;
                if side.flow.owns_child_occurrence(occurrence, first_generated) {
                    return side.flow.record_effect_result(occurrence, effect, result);
                }
            }
        }
        let Some(results) = &mut self.effect_results else {
            return Ok(());
        };
        control::charge(results.len() as u64 + 1, "flow_effect_scope_record")?;
        // Do not allocate witness history for unrelated effects.
        control::charge(self.plan.nodes.len() as u64, "flow_effect_scope_selection")?;
        if !self.plan.nodes.iter().any(|node| matches!(&node.kind,
            CompiledFlowNodeKind::Branch { condition: CompiledFlowCondition::EffectCompare { effect: site, .. }, .. } if site == effect)) {
            return Ok(());
        }
        crate::authoring::control::charge_serialized(effect, "flow_effect_scope_key")?;
        if !results
            .keys()
            .any(|(key, site)| *key == occurrence && site == effect)
        {
            let bytes = std::mem::size_of::<((usize, EffectSite), SkillStepResult)>()
                .checked_add(64)
                .and_then(|n| n.checked_add(effect.action.definition_id.as_str().len()))
                .and_then(|n| n.checked_add(effect.action.action_id.len()))
                .and_then(|n| n.checked_add(effect.effect_id.len()));
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                bytes,
                "flow_effect_scope_entry",
            )?;
        }
        results
            .entry((occurrence, effect.clone()))
            .or_default()
            .merge(result);
        Ok(())
    }
    fn condition(&self, condition: &CompiledFlowCondition) -> Result<bool, GameError> {
        match condition {
            CompiledFlowCondition::Always(value) => Ok(*value),
            CompiledFlowCondition::EffectCompare {
                producer,
                effect,
                metric,
                comparison,
                value,
            } => {
                let state = &self.nodes[*producer];
                if state.phase != 2 {
                    return Err(GameError::InvalidStaticData(
                        "flow effect result is not settled".into(),
                    ));
                }
                let result = match (state.occurrence, self.effect_results.as_ref()) {
                    (Some(occurrence), Some(results)) => {
                        control::charge(results.len() as u64, "flow_effect_scope_read")?;
                        results
                            .iter()
                            .find(|((key, site), _)| *key == occurrence && site == effect)
                            .map(|(_, result)| result)
                    }
                    _ => None,
                };
                let actual = result.map_or(0, |result| match metric {
                    FlowMetric::ResolvedTargets => result.resolved_target_count as u64,
                    FlowMetric::DamagedTargets => result.actual_damage_target_count as u64,
                    FlowMetric::KilledTargets => result.killed_target_count as u64,
                    FlowMetric::HpLost => result.actual_hp_loss,
                });
                let (FlowLiteral::Count(expected) | FlowLiteral::Hp(expected)) = value;
                Ok(match comparison {
                    FlowComparison::Equal => actual == *expected,
                    FlowComparison::Greater => actual > *expected,
                    FlowComparison::AtLeast => actual >= *expected,
                    FlowComparison::Less => actual < *expected,
                })
            }
            CompiledFlowCondition::Compare {
                producer,
                metric,
                comparison,
                value,
            } => {
                let state = &self.nodes[*producer];
                if state.phase != 2 {
                    return Err(GameError::InvalidStaticData(
                        "flow result is not settled".into(),
                    ));
                }
                let actual = match metric {
                    FlowMetric::ResolvedTargets => state.result.resolved_target_count as u64,
                    FlowMetric::DamagedTargets => state.result.actual_damage_target_count as u64,
                    FlowMetric::KilledTargets => state.result.killed_target_count as u64,
                    FlowMetric::HpLost => state.result.actual_hp_loss,
                };
                let (FlowLiteral::Count(expected) | FlowLiteral::Hp(expected)) = value;
                Ok(match comparison {
                    FlowComparison::Equal => actual == *expected,
                    FlowComparison::Greater => actual > *expected,
                    FlowComparison::AtLeast => actual >= *expected,
                    FlowComparison::Less => actual < *expected,
                })
            }
        }
    }
    fn drive(
        &mut self,
        node: usize,
        core: &mut BattleCore,
        cast: u64,
        skill: &SkillDef,
        now: u64,
    ) -> Result<bool, GameError> {
        control::charge(1, "flow_node_visit")?;
        crate::game::battle::runtime_control::charge(1, "flow_node_visit")?;
        if self.nodes[node].phase == 2 {
            return Ok(true);
        }
        let plan = self.plan.clone();
        match &plan.nodes[node].kind {
            CompiledFlowNodeKind::Action { step_index, wait } => {
                if self.nodes[node].phase == 0 {
                    let step = &skill.steps[*step_index];
                    let occurrence = self.next_occurrence;
                    self.next_occurrence =
                        occurrence
                            .checked_add(1)
                            .ok_or(GameError::InvalidStaticData(
                                "flow occurrence overflow".into(),
                            ))?;
                    core.admit_authoring_runtime_growth(self.nodes.len() + 1, "flow_occurrence")?;
                    let owner = core
                        .active_skill_casts
                        .get_mut(&cast)
                        .ok_or(GameError::InvalidAction)?;
                    owner.total_steps = owner
                        .total_steps
                        .checked_add(1)
                        .ok_or(GameError::InvalidAction)?;
                    core.event_queue.push(BattleEvent::SkillStep {
                        time_ms: add_time(now, u64::from(step.delay_ms))?,
                        cast_seq: cast,
                        step_index: occurrence,
                        caster_instance_id: owner.caster_instance_id,
                        skill_id: skill.id.clone(),
                        step_id: step.id.clone(),
                        cast_target: self.target,
                        cause: BattleEventCause::Parent {
                            seq: owner.ability_event_seq,
                        },
                    });
                    self.nodes[node].occurrence = Some(occurrence);
                    self.nodes[node].phase = 1;
                    self.seed_action_scope(node, occurrence, core, cast)?;
                    self.prepare_sides(node, occurrence, core, cast)?;
                    return Ok(false);
                }
                let progress = core
                    .active_skill_casts
                    .get(&cast)
                    .and_then(|c| c.step_progress.get(&self.nodes[node].occurrence.unwrap()));
                let ready = self.nodes[node].dispatched
                    && (plan.legacy
                        || *wait == FlowMilestone::Released
                        || progress.is_some_and(|p| p.is_terminal()));
                if !ready {
                    return Ok(false);
                }
                if let Some(progress) = progress {
                    self.nodes[node].result = progress.result.clone();
                }
                // Capture the exact original occurrence before a Repeat can
                // reset it in this same drive. Released carriers may settle later.
                self.start_settled_sides(core, cast)?;
            }
            CompiledFlowNodeKind::Wait { duration_ms } => {
                let deadline = match self.nodes[node].deadline {
                    Some(deadline) => deadline,
                    None => {
                        let deadline = add_time(now, *duration_ms)?;
                        self.nodes[node].deadline = Some(deadline);
                        deadline
                    }
                };
                if now < deadline {
                    self.wake = Some(self.wake.map_or(deadline, |w| w.min(deadline)));
                    return Ok(false);
                }
            }
            CompiledFlowNodeKind::Sequence { children } => {
                while (self.nodes[node].cursor as usize) < children.len() {
                    let child = children[self.nodes[node].cursor as usize];
                    if !self.drive(child, core, cast, skill, now)? {
                        return Ok(false);
                    }
                    let result = self.nodes[child].result.clone();
                    self.nodes[node].result.merge(&result);
                    self.nodes[node].cursor += 1;
                }
            }
            CompiledFlowNodeKind::Parallel { branches } => {
                let mut all = true;
                for &branch in branches {
                    all &= self.drive(branch, core, cast, skill, now)?;
                }
                if !all {
                    return Ok(false);
                }
                // Aggregate in authored order, independently of delivery completion order.
                for &branch in branches {
                    let result = self.nodes[branch].result.clone();
                    self.nodes[node].result.merge(&result);
                }
            }
            CompiledFlowNodeKind::Branch {
                condition,
                then_node,
                else_node,
            } => {
                if self.nodes[node].phase == 0 {
                    self.nodes[node].cursor = u32::from(self.condition(condition)?);
                    self.nodes[node].phase = 1;
                }
                let child = if self.nodes[node].cursor == 1 {
                    *then_node
                } else {
                    *else_node
                };
                if !self.drive(child, core, cast, skill, now)? {
                    return Ok(false);
                }
                self.nodes[node].result = self.nodes[child].result.clone();
            }
            CompiledFlowNodeKind::Repeat {
                count,
                interval_ms,
                body,
            } => {
                if let Some(deadline) = self.nodes[node].deadline {
                    if now < deadline {
                        self.wake = Some(self.wake.map_or(deadline, |w| w.min(deadline)));
                        return Ok(false);
                    }
                    self.nodes[node].deadline = None;
                }
                if !self.drive(*body, core, cast, skill, now)? {
                    return Ok(false);
                }
                let result = self.nodes[*body].result.clone();
                self.nodes[node].result.merge(&result);
                self.nodes[node].cursor += 1;
                if self.nodes[node].cursor < *count {
                    self.reset(*body, core, cast)?;
                    let deadline = add_time(now, *interval_ms)?;
                    self.nodes[node].deadline = Some(deadline);
                    self.wake = Some(self.wake.map_or(deadline, |w| w.min(deadline)));
                    return Ok(false);
                }
            }
        }
        self.nodes[node].phase = 2;
        Ok(true)
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn skill_flow_occurrence(
        &self,
        cast: u64,
        occurrence: usize,
    ) -> Option<crate::game::battle::runtime_facts::FlowOccurrence> {
        let flow = self.active_skill_casts.get(&cast)?.flow.as_ref()?;
        if flow.plan.legacy && flow.plan.side_branches.is_empty() {
            return None;
        }
        let node_index = flow.occurrence_node(occurrence)?;
        Some(crate::game::battle::runtime_facts::FlowOccurrence {
            node_index,
            occurrence,
        })
    }
    pub(in crate::game::battle::core) fn cancel_structured_flows(
        &mut self,
        actor: crate::game::battle::ids::UnitInstanceId,
        now: u64,
        reason: crate::game::battle::runtime_facts::ActionCancelReason,
    ) {
        if control::charge(self.active_skill_casts.len() as u64, "flow_cancel_scan").is_err() {
            return;
        }
        loop {
            let next = self
                .active_skill_casts
                .iter()
                .filter(|(_, cast)| {
                    cast.caster_instance_id == actor
                        && !cast.allow_dead_caster
                        && !cast.control_cancelled
                        && cast
                            .flow
                            .as_ref()
                            .is_some_and(|flow| !flow.complete || flow.has_pending_sides())
                })
                .map(|(seq, _)| *seq)
                .min();
            let Some(seq) = next else {
                break;
            };
            if control::charge(self.active_skill_casts.len() as u64 + 1, "flow_cancel_next")
                .is_err()
            {
                return;
            }
            let mut flow = self
                .active_skill_casts
                .get_mut(&seq)
                .expect("selected flow owner")
                .flow
                .take();
            if let Some(flow) = flow.as_mut() {
                flow.complete = true;
                if flow.cancel_sides(self, now).is_err() {
                    return;
                }
            }
            let cast = self
                .active_skill_casts
                .get_mut(&seq)
                .expect("selected flow owner");
            cast.flow = flow;
            cast.control_cancelled = true;
            cast.deferred_steps.clear();
            cast.step_progress.retain(|_, progress| progress.started);
            // Released carriers retain their cast context and actual leases.
            cast.total_steps = cast.step_progress.len();
            let action = cast.fact_source.action.clone();
            self.cancel_pending_action(action.as_ref(), now, reason);
            self.cleanup_finished_skill_cast(seq);
        }
    }

    pub(in crate::game::battle::core) fn wake_skill_flow(&mut self, cast: u64, now: u64) {
        let Some(flow) = self
            .active_skill_casts
            .get_mut(&cast)
            .and_then(|c| c.flow.as_mut())
        else {
            return;
        };
        if (flow.plan.legacy && flow.plan.side_branches.is_empty())
            || (flow.complete && !flow.has_pending_sides())
            || flow.wake.is_some_and(|wake| wake <= now)
        {
            return;
        }
        flow.generation = match flow.generation.checked_add(1) {
            Some(value) => value,
            None => return,
        };
        flow.wake = Some(now);
        self.event_queue.push(BattleEvent::SkillFlowWake {
            time_ms: now,
            cast_seq: cast,
            generation: flow.generation,
        });
    }
    pub(in crate::game::battle::core) fn flow_step_dispatched(
        &mut self,
        cast: u64,
        occurrence: usize,
        now: u64,
    ) -> Result<(), GameError> {
        if let Some(flow) = self
            .active_skill_casts
            .get_mut(&cast)
            .and_then(|c| c.flow.as_mut())
        {
            let _ = flow.mark_dispatched(occurrence)?;
        }
        if self
            .active_skill_casts
            .get(&cast)
            .and_then(|c| c.flow.as_ref())
            .is_some_and(|f| f.plan.legacy)
        {
            self.drive_skill_flow(cast, now, None)?;
        } else {
            self.wake_skill_flow(cast, now);
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn drive_skill_flow(
        &mut self,
        cast: u64,
        now: u64,
        generation: Option<u64>,
    ) -> Result<(), GameError> {
        let Some(owner) = self.active_skill_casts.get_mut(&cast) else {
            return Ok(());
        };
        let Some(mut flow) = owner.flow.take() else {
            return Ok(());
        };
        if (flow.complete && !flow.has_pending_sides())
            || generation.is_some_and(|g| g != flow.generation || flow.wake != Some(now))
        {
            owner.flow = Some(flow);
            return Ok(());
        }
        let skill_id = owner.skill_id.clone();
        let action = owner.fact_source.action.clone();
        let source = owner.fact_source.clone();
        flow.wake = None;
        let operation = (|| {
            let skill = self
                .cloned_skill_for_source(&skill_id, &source)?
                .ok_or(GameError::InvalidAction)?;
            let original_complete = flow.complete;
            flow.start_settled_sides(self, cast)?;
            if !flow.complete {
                flow.complete = flow.drive(flow.root, self, cast, &skill, now)?;
            }
            flow.drive_sides(self, cast, &skill, now)?;
            flow.prune_result_scopes(self, cast)?;
            if flow.complete && !original_complete && !flow.plan.legacy {
                if let Some(action) = &action {
                    self.runtime_facts.dispatched(action, 0, now)?;
                }
            }
            if let Some(wake) = flow.wake {
                flow.generation = flow
                    .generation
                    .checked_add(1)
                    .ok_or(GameError::InvalidAction)?;
                self.event_queue.push(BattleEvent::SkillFlowWake {
                    time_ms: wake,
                    cast_seq: cast,
                    generation: flow.generation,
                });
            }
            control::poll("flow_continuation_committed")
        })();
        if let Some(owner) = self.active_skill_casts.get_mut(&cast) {
            owner.flow = Some(flow);
        }
        operation?;
        self.cleanup_finished_skill_cast(cast);
        Ok(())
    }
}
