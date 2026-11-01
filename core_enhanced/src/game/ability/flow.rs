//! Authored finite control flow and its immutable, indexed execution plan.
use super::{authoring::EffectSite, SkillStepDef};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub const MAX_FLOW_DEPTH: usize = 64;
pub const MAX_FLOW_NODES: usize = 4096;
pub const MAX_FLOW_REPEAT: u32 = 1_000_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowMilestone {
    Released,
    #[default]
    EffectsSettled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowMetric {
    ResolvedTargets,
    DamagedTargets,
    KilledTargets,
    HpLost,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowLiteral {
    Count(u64),
    Hp(u64),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowComparison {
    Equal,
    Greater,
    AtLeast,
    Less,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FlowCondition {
    Predicate(super::condition::PredicateCall),
    Logic(super::condition::ConditionLogic<FlowCondition>),
    Always(bool),
    Compare {
        node_id: String,
        metric: FlowMetric,
        comparison: FlowComparison,
        value: FlowLiteral,
    },
    /// The named effect of one settled producer occurrence. Ordinary Compare
    /// continues to observe the complete Action result.
    EffectCompare {
        node_id: String,
        effect_id: String,
        metric: FlowMetric,
        comparison: FlowComparison,
        value: FlowLiteral,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillFlowDef {
    Action {
        id: String,
        step_id: String,
        #[serde(default)]
        wait: FlowMilestone,
    },
    Wait {
        id: String,
        duration_ms: u64,
    },
    Sequence {
        id: String,
        children: Vec<SkillFlowDef>,
    },
    Parallel {
        id: String,
        branches: Vec<SkillFlowDef>,
    },
    Branch {
        id: String,
        condition: FlowCondition,
        then_node: Box<SkillFlowDef>,
        else_node: Box<SkillFlowDef>,
    },
    Repeat {
        id: String,
        count: u32,
        interval_ms: u64,
        body: Box<SkillFlowDef>,
    },
}
impl SkillFlowDef {
    pub fn id(&self) -> &str {
        match self {
            Self::Action { id, .. }
            | Self::Wait { id, .. }
            | Self::Sequence { id, .. }
            | Self::Parallel { id, .. }
            | Self::Branch { id, .. }
            | Self::Repeat { id, .. } => id,
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub enum CompiledFlowCondition {
    Logic(super::condition::ConditionLogic<CompiledFlowCondition>),
    Always(bool),
    Compare {
        producer: usize,
        metric: FlowMetric,
        comparison: FlowComparison,
        value: FlowLiteral,
    },
    EffectCompare {
        producer: usize,
        effect: EffectSite,
        metric: FlowMetric,
        comparison: FlowComparison,
        value: FlowLiteral,
    },
}
#[derive(Debug, Clone, Serialize)]
pub enum CompiledFlowNodeKind {
    Action {
        step_index: usize,
        wait: FlowMilestone,
    },
    Wait {
        duration_ms: u64,
    },
    Sequence {
        children: Vec<usize>,
    },
    Parallel {
        branches: Vec<usize>,
    },
    Branch {
        condition: CompiledFlowCondition,
        then_node: usize,
        else_node: usize,
    },
    Repeat {
        count: u32,
        interval_ms: u64,
        body: usize,
    },
}
#[derive(Debug, Clone, Serialize)]
pub struct CompiledFlowNode {
    pub id: String,
    pub kind: CompiledFlowNodeKind,
    pub result_available_on_completion: bool,
    pub guaranteed_effect_prefix: bool,
    /// Producers whose settled results are guaranteed before this node.
    pub result_scope: Vec<usize>,
}
#[derive(Debug, Clone, Serialize)]
pub struct CompiledSkillFlow {
    pub nodes: Vec<CompiledFlowNode>,
    pub root: usize,
    pub legacy: bool,
    /// Finite structural branches start once at each original anchor's settled
    /// occurrence. They do not change the original root or Action wait.
    pub side_branches: Vec<CompiledSideBranch>,
    pub authored_delay_bound_ms: u64,
    pub settled_effect_prefix: Vec<bool>,
    #[serde(skip)]
    pub source: Option<Arc<SkillFlowDef>>,
    #[serde(skip)]
    pub step_ids: Vec<(String, u32, bool, bool)>,
}
#[derive(Debug, Clone, Serialize)]
pub struct CompiledSideBranch {
    pub anchor: usize,
    pub root: usize,
}
#[derive(Debug, Clone)]
pub struct FlowCompileError {
    pub path: String,
    pub cause: String,
}
fn error(path: &str, cause: impl Into<String>) -> FlowCompileError {
    FlowCompileError {
        path: path.into(),
        cause: cause.into(),
    }
}
type Scope = BTreeMap<String, usize>;
struct Compiler<'a> {
    steps: &'a [SkillStepDef],
    nodes: Vec<CompiledFlowNode>,
    ids: BTreeSet<String>,
    steps_used: BTreeSet<usize>,
    predecessors: Vec<bool>,
    definition_bytes: usize,
}
// Walk borrowed input before allocating tables or copying scopes. This prices
// static node/edge/string owners; transient scope copies are admitted below.
fn preflight(
    raw: &SkillFlowDef,
    depth: usize,
    count: &mut usize,
    conditions: &mut usize,
    bytes: &mut usize,
) -> Result<(), FlowCompileError> {
    use crate::authoring::control::{self, Resource};
    control::charge(1 + raw.id().len() as u64, "flow_shape_preflight")
        .map_err(|e| error("flow", format!("{e:?}")))?;
    *count += 1;
    if depth > MAX_FLOW_DEPTH || *count > MAX_FLOW_NODES {
        return Err(error("flow", "finite flow depth/node limit exceeded"));
    }
    let extra = 512usize
        .checked_add(raw.id().len().saturating_mul(4))
        .ok_or_else(|| error("flow", "flow storage overflow"))?;
    control::admit(
        Resource::MetadataBytes,
        *bytes,
        extra,
        "flow_compiled_storage",
    )
    .map_err(|e| error("flow", format!("{e:?}")))?;
    *bytes = bytes
        .checked_add(extra)
        .ok_or_else(|| error("flow", "flow storage overflow"))?;
    match raw {
        SkillFlowDef::Sequence { children, .. }
        | SkillFlowDef::Parallel {
            branches: children, ..
        } => {
            for child in children {
                preflight(child, depth + 1, count, conditions, bytes)?;
            }
        }
        SkillFlowDef::Branch {
            condition,
            then_node,
            else_node,
            ..
        } => {
            preflight_condition(condition, 1, conditions, bytes)?;
            preflight(then_node, depth + 1, count, conditions, bytes)?;
            preflight(else_node, depth + 1, count, conditions, bytes)?;
        }
        SkillFlowDef::Repeat { body, .. } => preflight(body, depth + 1, count, conditions, bytes)?,
        _ => (),
    }
    Ok(())
}
fn preflight_condition(
    raw: &FlowCondition,
    depth: usize,
    count: &mut usize,
    bytes: &mut usize,
) -> Result<(), FlowCompileError> {
    use crate::authoring::control::{self, Resource};
    control::charge(1, "flow_condition_preflight")
        .map_err(|e| error("flow.condition", format!("{e:?}")))?;
    *count += 1;
    if depth > MAX_FLOW_DEPTH || *count > MAX_FLOW_NODES {
        return Err(error(
            "flow.condition",
            "finite condition depth/node limit exceeded",
        ));
    }
    let text = match raw {
        FlowCondition::Predicate(_) => {
            return Err(error("flow.condition", "unresolved authored predicate"))
        }
        FlowCondition::Compare { node_id, .. } => node_id.len(),
        FlowCondition::EffectCompare {
            node_id, effect_id, ..
        } => node_id.len().saturating_add(effect_id.len()),
        _ => 0,
    };
    let extra = if depth > 1 {
        256usize.saturating_add(text)
    } else {
        0
    };
    control::admit(
        Resource::MetadataBytes,
        *bytes,
        extra,
        "flow_condition_storage",
    )
    .map_err(|e| error("flow.condition", format!("{e:?}")))?;
    *bytes = bytes
        .checked_add(extra)
        .ok_or_else(|| error("flow.condition", "condition storage overflow"))?;
    if let FlowCondition::Logic(logic) = raw {
        if logic.children().is_empty() {
            return Err(error("flow.condition", "boolean group must be nonempty"));
        }
        for child in logic.children() {
            preflight_condition(child, depth + 1, count, bytes)?;
        }
    }
    Ok(())
}
impl Compiler<'_> {
    fn condition(
        &self,
        condition: &FlowCondition,
        path: &str,
        before: &Scope,
    ) -> Result<CompiledFlowCondition, FlowCompileError> {
        crate::authoring::control::charge(1, "flow_condition_compile")
            .map_err(|e| error(path, format!("{e:?}")))?;
        Ok(match condition {
            FlowCondition::Predicate(_) => {
                return Err(error(
                    path,
                    "unresolved authored predicate; compile the composition module first",
                ))
            }
            FlowCondition::Logic(logic) => {
                if logic.children().is_empty() {
                    return Err(error(path, "boolean group must be nonempty"));
                }
                CompiledFlowCondition::Logic(logic.try_map(|i, child| {
                    self.condition(
                        child,
                        &format!("{path}.Logic{}", logic.child_suffix(i)),
                        before,
                    )
                })?)
            }
            FlowCondition::Always(value) => CompiledFlowCondition::Always(*value),
            FlowCondition::EffectCompare {
                node_id,
                effect_id,
                metric,
                comparison,
                value,
            } => {
                let producer = *before.get(node_id).ok_or_else(|| {
                    error(
                        &format!("{path}.node_id"),
                        "effect result is unknown, forward, out of scope, or not EffectsSettled",
                    )
                })?;
                let CompiledFlowNodeKind::Action { step_index, .. } = self.nodes[producer].kind
                else {
                    return Err(error(path, "effect result requires an Action producer"));
                };
                crate::authoring::control::charge(
                    self.steps[step_index].effects.len() as u64,
                    "flow_effect_result_reference",
                )
                .map_err(|e| error(path, format!("{e:?}")))?;
                let effect = self.steps[step_index]
                    .effects
                    .iter()
                    .find(|effect| effect.id == *effect_id)
                    .and_then(|effect| effect.source())
                    .ok_or_else(|| error(path, "unknown producer effect"))?;
                if !matches!(
                    (metric, value),
                    (FlowMetric::HpLost, FlowLiteral::Hp(_))
                        | (
                            FlowMetric::ResolvedTargets
                                | FlowMetric::DamagedTargets
                                | FlowMetric::KilledTargets,
                            FlowLiteral::Count(_)
                        )
                ) {
                    return Err(error(path, "effect result/literal units do not match"));
                }
                crate::authoring::control::charge_serialized(
                    &effect.site,
                    "flow_effect_result_site",
                )
                .map_err(|e| error(path, format!("{e:?}")))?;
                CompiledFlowCondition::EffectCompare {
                    producer,
                    effect: effect.site.clone(),
                    metric: *metric,
                    comparison: *comparison,
                    value: *value,
                }
            }
            FlowCondition::Compare {
                node_id,
                metric,
                comparison,
                value,
            } => {
                let producer = *before.get(node_id).ok_or_else(|| {
                    error(
                        &format!("{path}.node_id"),
                        "result is unknown, forward, out of scope, or not EffectsSettled",
                    )
                })?;
                if !matches!(
                    (metric, value),
                    (FlowMetric::HpLost, FlowLiteral::Hp(_))
                        | (
                            FlowMetric::ResolvedTargets
                                | FlowMetric::DamagedTargets
                                | FlowMetric::KilledTargets,
                            FlowLiteral::Count(_)
                        )
                ) {
                    return Err(error(
                        &format!("{path}.value"),
                        "result/literal units do not match",
                    ));
                }
                CompiledFlowCondition::Compare {
                    producer,
                    metric: *metric,
                    comparison: *comparison,
                    value: *value,
                }
            }
        })
    }
    fn node(
        &mut self,
        raw: &SkillFlowDef,
        path: &str,
        depth: usize,
        before: &Scope,
        outer_scope_bytes: usize,
    ) -> Result<(usize, Scope, u64, bool), FlowCompileError> {
        crate::authoring::control::charge(1, "skill_flow_compile")
            .map_err(|e| error(path, format!("{e:?}")))?;
        if depth > MAX_FLOW_DEPTH || self.nodes.len() >= MAX_FLOW_NODES {
            return Err(error(path, "finite flow depth/node limit exceeded"));
        }
        let local_scope_bytes = before
            .keys()
            .try_fold(0usize, |bytes, key| {
                bytes.checked_add(128 + key.len().saturating_mul(4))
            })
            .ok_or_else(|| error(path, "flow scope storage overflow"))?;
        let scope_bytes = outer_scope_bytes
            .checked_add(local_scope_bytes)
            .ok_or_else(|| error(path, "flow scope storage overflow"))?;
        crate::authoring::control::admit(
            crate::authoring::control::Resource::MetadataBytes,
            self.definition_bytes,
            scope_bytes,
            "flow_scope_storage",
        )
        .map_err(|e| error(path, format!("{e:?}")))?;
        let id = raw.id();
        if id.is_empty() || id.trim() != id || id.len() > 128 || !self.ids.insert(id.into()) {
            return Err(error(
                &format!("{path}.id"),
                "flow node ID must be unique, nonempty, trimmed, at most 128 bytes",
            ));
        }
        let index = self.nodes.len();
        crate::authoring::control::admit(
            crate::authoring::control::Resource::Definitions,
            index,
            1,
            "skill_flow_node_allocation",
        )
        .map_err(|e| error(path, format!("{e:?}")))?;
        self.nodes.push(CompiledFlowNode {
            id: id.into(),
            kind: CompiledFlowNodeKind::Wait { duration_ms: 0 },
            result_available_on_completion: false,
            guaranteed_effect_prefix: false,
            result_scope: {
                let mut indices: Vec<_> = before.values().copied().collect();
                indices.sort_unstable();
                indices.dedup();
                indices
            },
        });
        crate::authoring::control::charge(
            before.keys().map(|key| key.len() as u64 + 1).sum(),
            "flow_scope_copy",
        )
        .map_err(|e| error(path, format!("{e:?}")))?;
        let mut after = before.clone();
        let (kind, duration, settled) = match raw {
            SkillFlowDef::Action { step_id, wait, .. } => {
                crate::authoring::control::charge(
                    self.steps.len() as u64,
                    "skill_flow_step_reference",
                )
                .map_err(|e| error(path, format!("{e:?}")))?;
                let step_index = self
                    .steps
                    .iter()
                    .position(|step| &step.id == step_id)
                    .ok_or_else(|| {
                        error(
                            &format!("{path}.step_id"),
                            format!("unknown action '{step_id}'"),
                        )
                    })?;
                if matches!(
                    self.steps[step_index].when,
                    super::SkillStepCondition::IfPreviousStepDealtDamage
                ) {
                    return Err(error(path, "structured flow uses explicit settled result conditions; implicit previous occurrence is ambiguous"));
                }
                if *wait == FlowMilestone::Released
                    && self.steps[step_index].effects.iter().any(|e| {
                        matches!(
                            &e.op,
                            super::SkillEffectDef::Sustained {
                                definition: super::sustained::SustainedEffectDef::ChannelAura { .. }
                            }
                        )
                    })
                {
                    return Err(error(
                        path,
                        "ChannelAura continuation must wait for EffectsSettled",
                    ));
                }
                self.predecessors[step_index] = before
                    .values()
                    .any(|index| self.nodes[*index].guaranteed_effect_prefix);
                if !self.steps_used.insert(step_index) {
                    return Err(error(&format!("{path}.step_id"), "one authored action may have one flow producer; use Repeat for repeated occurrences"));
                }
                (
                    CompiledFlowNodeKind::Action {
                        step_index,
                        wait: *wait,
                    },
                    u64::from(self.steps[step_index].delay_ms),
                    *wait == FlowMilestone::EffectsSettled,
                )
            }
            SkillFlowDef::Wait { duration_ms, .. } => (
                CompiledFlowNodeKind::Wait {
                    duration_ms: *duration_ms,
                },
                *duration_ms,
                true,
            ),
            SkillFlowDef::Sequence { children, .. } => {
                let mut indices = Vec::new();
                let mut duration = 0_u64;
                let mut settled = true;
                for (n, child) in children.iter().enumerate() {
                    let (child, scope, delta, ready) = self.node(
                        child,
                        &format!("{path}.children[{n}]"),
                        depth + 1,
                        &after,
                        scope_bytes,
                    )?;
                    indices.push(child);
                    after = scope;
                    settled &= ready;
                    duration = duration
                        .checked_add(delta)
                        .ok_or_else(|| error(path, "flow time overflow"))?;
                }
                (
                    CompiledFlowNodeKind::Sequence { children: indices },
                    duration,
                    settled,
                )
            }
            SkillFlowDef::Parallel { branches, .. } => {
                let mut indices = Vec::new();
                let mut duration = 0;
                let mut settled = true;
                for (n, branch) in branches.iter().enumerate() {
                    let (branch, scope, delta, ready) = self.node(
                        branch,
                        &format!("{path}.branches[{n}]"),
                        depth + 1,
                        before,
                        scope_bytes,
                    )?;
                    indices.push(branch);
                    after.extend(scope);
                    duration = duration.max(delta);
                    settled &= ready;
                }
                (
                    CompiledFlowNodeKind::Parallel { branches: indices },
                    duration,
                    settled,
                )
            }
            SkillFlowDef::Branch {
                condition,
                then_node,
                else_node,
                ..
            } => {
                let condition = self.condition(condition, &format!("{path}.condition"), before)?;
                let (yes, _, yd, yr) = self.node(
                    then_node,
                    &format!("{path}.then_node"),
                    depth + 1,
                    before,
                    scope_bytes,
                )?;
                let (no, _, nd, nr) = self.node(
                    else_node,
                    &format!("{path}.else_node"),
                    depth + 1,
                    before,
                    scope_bytes,
                )?;
                (
                    CompiledFlowNodeKind::Branch {
                        condition,
                        then_node: yes,
                        else_node: no,
                    },
                    yd.max(nd),
                    yr && nr,
                )
            }
            SkillFlowDef::Repeat {
                count,
                interval_ms,
                body,
                ..
            } => {
                if *count == 0 || *count > MAX_FLOW_REPEAT {
                    return Err(error(
                        &format!("{path}.count"),
                        "repeat count must be 1..=1000000",
                    ));
                }
                let (body, _, bd, ready) = self.node(
                    body,
                    &format!("{path}.body"),
                    depth + 1,
                    before,
                    scope_bytes,
                )?;
                let duration = bd
                    .checked_mul(u64::from(*count))
                    .and_then(|d| {
                        interval_ms
                            .checked_mul(u64::from(count - 1))
                            .and_then(|gap| d.checked_add(gap))
                    })
                    .ok_or_else(|| error(path, "repeat time overflow"))?;
                (
                    CompiledFlowNodeKind::Repeat {
                        count: *count,
                        interval_ms: *interval_ms,
                        body,
                    },
                    duration,
                    ready,
                )
            }
        };
        if settled {
            after.insert(id.into(), index);
        }
        self.nodes[index].guaranteed_effect_prefix = match &kind {
            CompiledFlowNodeKind::Action { step_index, .. } => {
                !self.steps[*step_index].effects.is_empty()
            }
            CompiledFlowNodeKind::Sequence { children }
            | CompiledFlowNodeKind::Parallel { branches: children } => children
                .iter()
                .any(|index| self.nodes[*index].guaranteed_effect_prefix),
            CompiledFlowNodeKind::Branch {
                then_node,
                else_node,
                ..
            } => {
                self.nodes[*then_node].guaranteed_effect_prefix
                    && self.nodes[*else_node].guaranteed_effect_prefix
            }
            CompiledFlowNodeKind::Repeat { body, .. } => self.nodes[*body].guaranteed_effect_prefix,
            CompiledFlowNodeKind::Wait { .. } => false,
        };
        self.nodes[index].kind = kind;
        self.nodes[index].result_available_on_completion = settled;
        Ok((index, after, duration, settled))
    }
}
impl CompiledSkillFlow {
    pub fn compile(
        source: Option<&Arc<SkillFlowDef>>,
        steps: &[SkillStepDef],
    ) -> Result<Self, FlowCompileError> {
        crate::authoring::control::admit(
            crate::authoring::control::Resource::Definitions,
            0,
            steps.len(),
            "flow_action_table",
        )
        .map_err(|e| error("steps", format!("{e:?}")))?;
        crate::authoring::control::charge(steps.len() as u64, "flow_action_table")
            .map_err(|e| error("steps", format!("{e:?}")))?;
        if steps.len() >= MAX_FLOW_NODES {
            return Err(error("steps", "finite flow action limit exceeded"));
        }
        let mut bytes = steps
            .iter()
            .try_fold(0usize, |bytes, step| {
                bytes.checked_add(256 + step.id.len().saturating_mul(4))
            })
            .ok_or_else(|| error("steps", "flow action storage overflow"))?;
        crate::authoring::control::admit(
            crate::authoring::control::Resource::MetadataBytes,
            0,
            bytes,
            "flow_action_storage",
        )
        .map_err(|e| error("steps", format!("{e:?}")))?;
        if let Some(raw) = source {
            preflight(raw, 1, &mut 0, &mut 0, &mut bytes)?;
            let mut compiler = Compiler {
                steps,
                nodes: Vec::new(),
                ids: BTreeSet::new(),
                steps_used: BTreeSet::new(),
                predecessors: vec![false; steps.len()],
                definition_bytes: bytes,
            };
            let (root, _, duration, _) = compiler.node(raw, "flow", 1, &Scope::new(), 0)?;
            if compiler.steps_used.len() != steps.len() {
                return Err(error(
                    "flow",
                    "each authored action must have exactly one flow producer",
                ));
            }
            Ok(Self {
                settled_effect_prefix: compiler.predecessors,
                nodes: compiler.nodes,
                root,
                legacy: false,
                side_branches: Vec::new(),
                authored_delay_bound_ms: duration,
                source: Some(raw.clone()),
                step_ids: steps
                    .iter()
                    .map(|step| {
                        (
                            step.id.clone(),
                            step.delay_ms,
                            !step.effects.is_empty(),
                            matches!(
                                step.when,
                                super::SkillStepCondition::IfPreviousStepDealtDamage
                            ),
                        )
                    })
                    .collect(),
            })
        } else {
            if steps.len() + 1 > MAX_FLOW_NODES {
                return Err(error("steps", "finite flow node limit exceeded"));
            }
            crate::authoring::control::charge(steps.len() as u64 + 1, "legacy_skill_flow_compile")
                .map_err(|e| error("steps", format!("{e:?}")))?;
            let mut nodes = Vec::with_capacity(steps.len() + 1);
            nodes.push(CompiledFlowNode {
                id: "legacy".into(),
                kind: CompiledFlowNodeKind::Parallel {
                    branches: (1..=steps.len()).collect(),
                },
                result_available_on_completion: true,
                guaranteed_effect_prefix: steps.iter().any(|step| !step.effects.is_empty()),
                result_scope: Vec::new(),
            });
            nodes.extend(
                steps
                    .iter()
                    .enumerate()
                    .map(|(step_index, step)| CompiledFlowNode {
                        id: step.id.clone(),
                        kind: CompiledFlowNodeKind::Action {
                            step_index,
                            wait: FlowMilestone::EffectsSettled,
                        },
                        result_available_on_completion: true,
                        guaranteed_effect_prefix: !step.effects.is_empty(),
                        result_scope: Vec::new(),
                    }),
            );
            Ok(Self {
                settled_effect_prefix: {
                    let mut prior = false;
                    steps
                        .iter()
                        .map(|step| {
                            let result = prior;
                            prior |= !step.effects.is_empty();
                            result
                        })
                        .collect()
                },
                nodes,
                root: 0,
                legacy: true,
                side_branches: Vec::new(),
                authored_delay_bound_ms: steps
                    .iter()
                    .map(|step| u64::from(step.delay_ms))
                    .max()
                    .unwrap_or(0),
                source: None,
                step_ids: steps
                    .iter()
                    .map(|step| {
                        (
                            step.id.clone(),
                            step.delay_ms,
                            !step.effects.is_empty(),
                            matches!(
                                step.when,
                                super::SkillStepCondition::IfPreviousStepDealtDamage
                            ),
                        )
                    })
                    .collect(),
            })
        }
    }
    /// A private producer graft. Every anchor belongs to the original plan;
    /// generated actions are never entered into the spawn universe.
    pub(crate) fn attach_branches(
        mut self,
        steps: &[SkillStepDef],
        branches: &[(usize, Arc<SkillFlowDef>)],
    ) -> Result<Self, FlowCompileError> {
        let original_node_count = self.nodes.len();
        let original_steps = self.step_ids.len();
        let mut bytes = 0usize;
        for (_, raw) in branches {
            preflight(raw, 1, &mut 0, &mut 0, &mut bytes)?;
        }
        crate::authoring::control::charge(self.nodes.len() as u64, "flow_side_scope_prepare")
            .map_err(|e| error("flow", format!("{e:?}")))?;
        let ids = self.nodes.iter().map(|node| node.id.clone()).collect();
        let mut compiler = Compiler {
            steps,
            nodes: self.nodes,
            ids,
            steps_used: (0..original_steps).collect(),
            predecessors: self.settled_effect_prefix,
            definition_bytes: bytes,
        };
        compiler.predecessors.resize(steps.len(), false);
        let mut side_branches = Vec::with_capacity(branches.len());
        for (anchor_step, raw) in branches {
            crate::authoring::control::charge(
                original_node_count as u64,
                "flow_side_anchor_lookup",
            )
            .map_err(|e| error("flow", format!("{e:?}")))?;
            let anchor = compiler.nodes[..original_node_count].iter().position(|node|
                matches!(node.kind, CompiledFlowNodeKind::Action { step_index, .. } if step_index == *anchor_step))
                .ok_or_else(|| error("flow", "side branch anchor is not an original Action"))?;
            let mut scope = Scope::new();
            for index in &compiler.nodes[anchor].result_scope {
                scope.insert(compiler.nodes[*index].id.clone(), *index);
            }
            scope.insert(compiler.nodes[anchor].id.clone(), anchor);
            // The structural address is the authored Action site, which may
            // have a different flow node ID. Ambiguous names are rejected.
            let action_id = &steps[*anchor_step].id;
            if scope.get(action_id).is_some_and(|index| *index != anchor) {
                return Err(error("flow", "anchor result alias is ambiguous"));
            }
            scope.insert(action_id.clone(), anchor);
            // The graft starts only after this exact anchor occurrence has
            // settled, independently of the original parent's Action wait.
            let original_available = compiler.nodes[anchor].result_available_on_completion;
            compiler.nodes[anchor].result_available_on_completion = true;
            let result = compiler.node(raw, "flow.side", 1, &scope, 0);
            compiler.nodes[anchor].result_available_on_completion = original_available;
            let (root, _, _, _) = result?;
            side_branches.push(CompiledSideBranch { anchor, root });
        }
        if compiler.steps_used.len() != steps.len() {
            return Err(error(
                "flow",
                "each generated Action requires one finite flow producer",
            ));
        }
        self.nodes = compiler.nodes;
        self.settled_effect_prefix = compiler.predecessors;
        self.side_branches = side_branches;
        self.step_ids = steps
            .iter()
            .map(|step| {
                (
                    step.id.clone(),
                    step.delay_ms,
                    !step.effects.is_empty(),
                    matches!(
                        step.when,
                        super::SkillStepCondition::IfPreviousStepDealtDamage
                    ),
                )
            })
            .collect();
        Ok(self)
    }
    pub fn matches_source(
        &self,
        source: Option<&Arc<SkillFlowDef>>,
        steps: &[SkillStepDef],
    ) -> bool {
        let same = match (&self.source, source) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b) || a == b,
            _ => false,
        };
        same && self
            .step_ids
            .iter()
            .map(|(id, delay, effects, previous)| (id.as_str(), *delay, *effects, *previous))
            .eq(steps.iter().map(|step| {
                (
                    step.id.as_str(),
                    step.delay_ms,
                    !step.effects.is_empty(),
                    matches!(
                        step.when,
                        super::SkillStepCondition::IfPreviousStepDealtDamage
                    ),
                )
            }))
    }
}
