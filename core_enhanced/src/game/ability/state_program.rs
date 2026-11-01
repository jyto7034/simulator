//! Authored, typed fields and lifetime-bound event reactions. No definition ID selects
//! behavior. Instances are owned by the battle's existing sustained effects.
use super::{
    authoring::{ActionSite, EffectSite, EffectSource},
    flow::FlowComparison,
};
use crate::game::value_expr::{
    self as expr, AmountExpr, CompiledAmountPlan, DeliveryCapabilities, ExprLimits, ValueRef,
    ValueSubject, ValueUnit,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, mem::size_of, sync::Arc};

pub const MAX_STATE_FIELDS: usize = 64;
pub const MAX_STATE_HANDLERS: usize = 32;
pub const MAX_STATE_OPERATIONS: usize = 256;
pub const MAX_STATE_DEPTH: usize = 16;

/// The first child handoff executes atomically in the input transaction. Keep
/// canonical authored Action flow; never erase a more complex flow to fit it.
pub(crate) fn immediate_child(skill: &super::SkillDef) -> bool {
    use super::{
        flow::SkillFlowDef, resonance::*, DeliveryDef, SkillStepCondition, SkillStepRepeat,
    };
    let [step] = skill.steps.as_slice() else {
        return false;
    };
    skill.focus_time_ms == 0
        && matches!(skill.use_lifetime, super::SkillUseLifetime::Action)
        && skill.resonance_admission.is_some_and(|a| {
            a.requirement == ResonanceRequirement::None && a.cost == ResonanceCost::Zero
        })
        && step.delay_ms == 0
        && step.when == SkillStepCondition::Always
        && step.repeat == SkillStepRepeat::Once
        && step.effects.iter().all(|effect| {
            matches!(
                &effect.op,
                super::SkillEffectDef::Damage { .. }
                    | super::SkillEffectDef::DamageExpr { .. }
                    | super::SkillEffectDef::Heal { .. }
                    | super::SkillEffectDef::HealExpr { .. }
                    | super::SkillEffectDef::Pull { .. }
                    | super::SkillEffectDef::PersonalShieldGrant { .. }
                    | super::SkillEffectDef::Sustained {
                        definition: super::sustained::SustainedEffectDef::HitStatus { .. }
                    }
            )
        })
        && skill.flow.as_deref().is_none_or(
            |flow| matches!(flow, SkillFlowDef::Action { step_id, .. } if step_id == &step.id),
        )
        && matches!(
            &step.delivery,
            DeliveryDef::Instant
                | DeliveryDef::TileArea {
                    area: super::SkillTileAreaDeliveryDef {
                        duration_ms: 0,
                        emission: None,
                        first_tick_delay_ms: 0,
                        ..
                    },
                    ..
                }
        )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateProgram {
    pub id: String,
    pub owns_use: bool,
    pub end_on_source_exit: bool,
    pub replacement: StateReplacement,
    /// A lease on the existing basic-attack consumer, released with this state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basic_attack: Option<Arc<super::sustained::BasicAttackOverlayProfile>>,
    pub fields: Vec<StateField>,
    pub handlers: Vec<StateHandler>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateReplacement {
    ReplaceSameSource,
    Stack,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateField {
    pub name: String,
    pub value: StateFieldDef,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StateFieldDef {
    Integer {
        unit: ValueUnit,
        initial: i64,
        min: i64,
        max: i64,
    },
    Bool(bool),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateRelation {
    Any,
    Hostile,
    Friendly,
}
/// Encounter authority is checked against the finalized victim's runtime
/// identity, never a content name or a shared base UUID.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateKillEligibility {
    #[default]
    AnyHostile,
    EncounterCombatant,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StateEvent {
    HpLost {
        relation: StateRelation,
        once_per_action: bool,
    },
    /// Positive HP loss from a direct basic/ability attack, excluding reactions,
    /// proc origins, periodic and environmental damage.
    DirectHpLost {
        relation: StateRelation,
        once_per_action: bool,
    },
    /// A normal basic action's base damage finalized a hostile death. Only
    /// activations captured when that attack started may receive this event.
    BasicAttackKill {
        primary_only: bool,
        once_per_action: bool,
        #[serde(default)]
        eligibility: StateKillEligibility,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        threat: Option<crate::game::battle::types::BattleUnitThreatClass>,
    },
    /// An equipment-owned clock, first firing one positive interval after activation.
    Periodic {
        interval_ms: u32,
    },
    /// The wearing combatant finalized a hostile combatant's death by direct
    /// or periodic damage. Delivery is once per victim instance and handler.
    OwnerKill,
    /// A direct base component finalized this wearer's hostile victim.
    OwnerDirectKill { source: StateDirectKillSource },
    /// A new active skill use was admitted; reinput and procs do not emit this.
    OwnerSkillCast,
    /// The last actually applied Stun contribution from this wearer ended on a live enemy.
    OwnerStunEnded,
    Expired,
    ManualInput,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateDirectKillSource {
    BasicAttack,
    ActiveSkill,
    BasicOrActiveSkill,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateHandler {
    pub id: String,
    pub event: StateEvent,
    pub body: Vec<StateOp>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StateCondition {
    Always(bool),
    /// Captured before lethal damage or this damage's explicit DOT consumption.
    /// Available only in an OwnerKill callback, never a later live lookup.
    EventVictimHadOwnerDot,
    Predicate(super::condition::PredicateCall),
    Logic(super::condition::ConditionLogic<StateCondition>),
    Compare {
        left: AmountExpr,
        comparison: FlowComparison,
        right: AmountExpr,
    },
    Bool {
        field: String,
        equals: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StateOp {
    /// One deterministic chance at this operation site in an equipped kill event.
    Chance { percent: AmountExpr, then_ops: Vec<StateOp>, else_ops: Vec<StateOp> },
    /// Return an immutable modifier for the currently admitted active skill use.
    AddCastDamage { percent: AmountExpr },
    Set {
        field: String,
        value: AmountExpr,
    },
    SetBool {
        field: String,
        value: bool,
    },
    If {
        condition: StateCondition,
        then_ops: Vec<StateOp>,
        else_ops: Vec<StateOp>,
    },
    HealOwner {
        amount: AmountExpr,
    },
    /// A positive kill reward. Equipped owner-kill events defer until the use
    /// settles. BasicAttackKill attempts it immediately without autocast or
    /// storing blocked credit. Both use the ordinary cap and gain locks.
    GrantOwnerResonance {
        amount: AmountExpr,
    },
    /// An independent finite contribution on the state recipient. Replacement
    /// is per source; property composition remains TargetState's strongest rule.
    ApplyOwnerState {
        duration: AmountExpr,
        profile: super::sustained::TargetStateProfile,
        replacement: StateReplacement,
    },
    /// A finite independent status on the live enemy carried by OwnerStunEnded.
    /// The event target cannot be stored in a state field or used in another event.
    ApplyEventTargetStatus {
        duration: AmountExpr,
        profile: super::sustained::HitStatusProfile,
    },
    /// Mutate declared resources using this activation's real source and the
    /// state recipient as TargetEmployee. No new lifetime is opened.
    Resources {
        operations: Vec<crate::game::resource::ResourceOperation>,
    },
    End,
    /// Consume this activation and invoke a distinct immediate child using the
    /// remaining time captured at input acceptance. Only valid in ManualInput.
    EndAndInvoke {
        skill: super::SkillId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateScalar {
    Integer(i128),
    Bool(bool),
}
crate::authoring::clone_shape::copy_shape!(
    StateScalar,
    StateReplacement,
    StateEvent,
    StateKillEligibility,
    StateDirectKillSource,
    StateRelation
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateFieldView {
    pub name: String,
    pub unit: Option<ValueUnit>,
    pub value: StateScalar,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateView {
    pub definition: String,
    pub fields: Vec<StateFieldView>,
    pub events: u64,
}

#[derive(Debug)]
pub(crate) struct CompiledStateField {
    pub name: String,
    pub unit: Option<ValueUnit>,
    pub min: i128,
    pub max: i128,
    pub initial: StateScalar,
}
#[derive(Debug)]
pub(crate) enum CompiledStateCondition {
    Always(bool),
    EventVictimHadOwnerDot,
    Logic(super::condition::ConditionLogic<CompiledStateCondition>),
    Compare {
        left: Arc<CompiledAmountPlan>,
        comparison: FlowComparison,
        right: Arc<CompiledAmountPlan>,
    },
    Bool {
        field: usize,
        equals: bool,
    },
}
#[derive(Debug)]
pub(crate) enum CompiledStateOp {
    Chance { percent: Arc<CompiledAmountPlan>, roll_site: u64, then_ops: Vec<CompiledStateOp>, else_ops: Vec<CompiledStateOp> },
    AddCastDamage { percent: Arc<CompiledAmountPlan> },
    Set {
        field: usize,
        value: Arc<CompiledAmountPlan>,
    },
    SetBool {
        field: usize,
        value: bool,
    },
    If {
        condition: CompiledStateCondition,
        then_ops: Vec<CompiledStateOp>,
        else_ops: Vec<CompiledStateOp>,
    },
    HealOwner {
        amount: Arc<CompiledAmountPlan>,
    },
    GrantOwnerResonance {
        amount: Arc<CompiledAmountPlan>,
    },
    ApplyOwnerState {
        duration: Arc<CompiledAmountPlan>,
        profile: Arc<super::sustained::TargetStateProfile>,
        replacement: StateReplacement,
    },
    ApplyEventTargetStatus {
        duration: Arc<CompiledAmountPlan>,
        profile: Arc<super::sustained::HitStatusProfile>,
    },
    Resources {
        operations: Arc<Vec<crate::game::resource::ResourceOperation>>,
        source: Arc<EffectSource>,
    },
    End,
    EndAndInvoke {
        skill: super::SkillId,
    },
}
#[derive(Debug)]
pub(crate) struct CompiledStateHandler {
    pub event: StateEvent,
    pub body: Vec<CompiledStateOp>,
}
#[derive(Debug)]
pub(crate) struct CompiledStateProgram {
    pub id: String,
    pub end_on_source_exit: bool,
    pub replacement: StateReplacement,
    pub fields: Vec<CompiledStateField>,
    pub handlers: Vec<CompiledStateHandler>,
    pub periodic_interval_ms: Option<u32>,
    pub retained_bytes: usize,
    pub reaction_bytes: usize,
    pub compile_work: u64,
    pub has_expressions: bool,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id.trim() == id && !id.chars().any(char::is_control)
}
fn allowed_unit(unit: &ValueUnit) -> bool {
    matches!(
        unit,
        ValueUnit::Count
            | ValueUnit::Magnitude
            | ValueUnit::Scalar
            | ValueUnit::Milliseconds
            | ValueUnit::AttackSpeedPoints
    )
}
struct Compiler<'a> {
    program: &'a StateProgram,
    source: Option<&'a EffectSource>,
    limits: &'a ExprLimits,
    fields: &'a [CompiledStateField],
    operations: usize,
    conditions: usize,
    expression_nodes: usize,
    retained: usize,
    work: u64,
    deadline: bool,
    reaction: bool,
    equipped: bool,
    state_path: &'static str,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum StateLifetime {
    Finite,
    Prepared,
    Equipped,
    Reaction,
}
impl StateProgram {
    pub(crate) fn check_resources(
        &self,
        mut check: impl FnMut(&crate::game::resource::ResourceOperation) -> Result<(), (String, String)>,
    ) -> Result<(), (String, String)> {
        fn block(
            ops: &[StateOp],
            check: &mut impl FnMut(
                &crate::game::resource::ResourceOperation,
            ) -> Result<(), (String, String)>,
        ) -> Result<(), (String, String)> {
            for (index, op) in ops.iter().enumerate() {
                let result = match op {
                    StateOp::Resources { operations } => {
                        operations.iter().enumerate().try_for_each(|(i, op)| {
                            check(op).map_err(|(path, reason)| {
                                (format!("Resources.operations[{i}].{path}"), reason)
                            })
                        })
                    }
                    StateOp::If {
                        then_ops, else_ops, ..
                    } | StateOp::Chance { then_ops, else_ops, .. } => {
                        let branch = if matches!(op, StateOp::Chance { .. }) { "Chance" } else { "If" };
                        block(then_ops, check)
                            .map_err(|(path, reason)| (format!("{branch}.then_ops{path}"), reason))
                            .and_then(|()| block(else_ops, check)
                                .map_err(|(path, reason)| (format!("{branch}.else_ops{path}"), reason)))
                    },
                    _ => Ok(()),
                };
                result.map_err(|(path, reason)| (format!("[{index}].{path}"), reason))?;
            }
            Ok(())
        }
        for (index, handler) in self.handlers.iter().enumerate() {
            block(&handler.body, &mut check)
                .map_err(|(path, reason)| (format!("handlers[{index}].body{path}"), reason))?;
        }
        Ok(())
    }
    pub(crate) fn has_manual_input(&self) -> bool {
        self.handlers
            .iter()
            .any(|h| h.event == StateEvent::ManualInput)
    }
    pub(crate) fn visit_children<'a>(&'a self, mut visit: impl FnMut(&'a super::SkillId)) {
        fn block<'a>(ops: &'a [StateOp], visit: &mut impl FnMut(&'a super::SkillId)) {
            for op in ops {
                match op {
                    StateOp::EndAndInvoke { skill } => visit(skill),
                    StateOp::If {
                        then_ops, else_ops, ..
                    } | StateOp::Chance { then_ops, else_ops, .. } => {
                        block(then_ops, visit);
                        block(else_ops, visit);
                    }
                    _ => {}
                }
            }
        }
        for handler in &self.handlers {
            block(&handler.body, &mut visit);
        }
    }
    /// Preserve the offending authored operation without serializing the
    /// program or allocating paths for successful reference checks.
    pub(crate) fn check_children(
        &self,
        mut check: impl FnMut(&super::SkillId) -> Result<(), String>,
    ) -> Result<(), (String, String)> {
        fn block(
            ops: &[StateOp],
            check: &mut impl FnMut(&super::SkillId) -> Result<(), String>,
        ) -> Result<(), (String, String)> {
            for (index, op) in ops.iter().enumerate() {
                let result = match op {
                    StateOp::EndAndInvoke { skill } => {
                        check(skill).map_err(|reason| ("EndAndInvoke.skill".into(), reason))
                    }
                    StateOp::If {
                        then_ops, else_ops, ..
                    } | StateOp::Chance { then_ops, else_ops, .. } => {
                        let branch = if matches!(op, StateOp::Chance { .. }) { "Chance" } else { "If" };
                        block(then_ops, check)
                            .map_err(|(path, reason)| (format!("{branch}.then_ops{path}"), reason))
                            .and_then(|()| block(else_ops, check)
                                .map_err(|(path, reason)| (format!("{branch}.else_ops{path}"), reason)))
                    },
                    _ => Ok(()),
                };
                result.map_err(|(path, reason)| (format!("[{index}].{path}"), reason))?;
            }
            Ok(())
        }
        for (index, handler) in self.handlers.iter().enumerate() {
            block(&handler.body, &mut check)
                .map_err(|(path, reason)| (format!("handlers[{index}].body{path}"), reason))?;
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.compile().map(|_| ())
    }
    pub(crate) fn compile(&self) -> Result<Arc<CompiledStateProgram>, String> {
        self.compile_bound(None, &ExprLimits::default())
    }
    pub(crate) fn compile_bound(
        &self,
        source: Option<&EffectSource>,
        limits: &ExprLimits,
    ) -> Result<Arc<CompiledStateProgram>, String> {
        self.compile_lifetime(source, limits, StateLifetime::Finite)
    }
    pub(crate) fn compile_prepared(
        &self,
        source: Option<&EffectSource>,
        limits: &ExprLimits,
    ) -> Result<Arc<CompiledStateProgram>, String> {
        if self.owns_use
            || !self.end_on_source_exit
            || self.replacement != StateReplacement::ReplaceSameSource
            || self.basic_attack.is_some()
            || self
                .handlers
                .iter()
                .any(|h| !matches!(h.event, StateEvent::BasicAttackKill { .. }))
        {
            return Err("PreparedState requires independent use, source-exit cleanup, ReplaceSameSource, no basic overlay and only BasicAttackKill handlers".into());
        }
        self.compile_lifetime(source, limits, StateLifetime::Prepared)
    }
    pub(crate) fn compile_equipped(
        &self,
        resources: &crate::game::resource::ResourceCatalog,
        source: Option<&EffectSource>,
        limits: &ExprLimits,
    ) -> Result<Arc<CompiledStateProgram>, String> {
        if self.owns_use
            || !self.end_on_source_exit
            || self.replacement != StateReplacement::ReplaceSameSource
            || self.basic_attack.is_some()
            || self.handlers.iter().any(|h| {
                !matches!(
                    h.event,
                    StateEvent::HpLost { .. }
                        | StateEvent::DirectHpLost { .. }
                        | StateEvent::Periodic { .. }
                        | StateEvent::OwnerKill
                        | StateEvent::OwnerDirectKill { .. }
                        | StateEvent::OwnerSkillCast
                        | StateEvent::OwnerStunEnded
                )
            })
        {
            return Err("Equipment state requires independent use, source-exit cleanup, ReplaceSameSource, no basic overlay and only HpLost, DirectHpLost, Periodic, OwnerKill, OwnerDirectKill, OwnerSkillCast or OwnerStunEnded handlers".into());
        }
        fn check(ops: &[StateOp], depth: usize, count: &mut usize) -> Result<(), String> {
            *count = count
                .checked_add(ops.len())
                .ok_or("equipment state operation overflow")?;
            if depth > MAX_STATE_DEPTH || *count > MAX_STATE_OPERATIONS {
                return Err("equipment state operation budget exceeded".into());
            }
            for op in ops {
                match op {
                    StateOp::AddCastDamage { .. } | StateOp::Set { .. } | StateOp::SetBool { .. } | StateOp::HealOwner { .. } | StateOp::GrantOwnerResonance { .. } | StateOp::ApplyEventTargetStatus { .. } => (),
                    StateOp::Resources { .. } => (),
                    StateOp::ApplyOwnerState { profile, .. } if profile.end_on_source_exit => (),
                    StateOp::If { then_ops, else_ops, .. } | StateOp::Chance { then_ops, else_ops, .. } => {
                        check(then_ops, depth + 1, count)?;
                        check(else_ops, depth + 1, count)?;
                    }
                    _ => return Err("Equipment state allows fields, conditions, owner rewards and finite event-target statuses only".into()),
                }
            }
            Ok(())
        }
        let mut count = 0;
        for handler in &self.handlers {
            check(&handler.body, 0, &mut count)?;
        }
        let program = self.compile_lifetime(source, limits, StateLifetime::Equipped)?;
        equipped_resources::validate_equipped_resources(&program, resources)?;
        Ok(program)
    }
    /// Compile an atomic callback with no activation, fields or deadline. The
    /// caller validates its event and operations; only the operation compiler
    /// is shared with persistent state programs.
    pub(crate) fn compile_reaction(
        &self,
        source: &EffectSource,
        limits: &ExprLimits,
    ) -> Result<Arc<CompiledStateProgram>, String> {
        self.compile_lifetime(Some(source), limits, StateLifetime::Reaction)
    }
    fn compile_lifetime(
        &self,
        source: Option<&EffectSource>,
        limits: &ExprLimits,
        lifetime: StateLifetime,
    ) -> Result<Arc<CompiledStateProgram>, String> {
        let deadline = lifetime == StateLifetime::Finite;
        let reaction = lifetime == StateLifetime::Reaction;
        let state_path = match lifetime {
            StateLifetime::Finite => ".op.Sustained.definition.State.program.",
            StateLifetime::Prepared => ".op.Sustained.definition.PreparedState.program.",
            StateLifetime::Equipped => ".",
            // Reaction paths are appended by effect_source at the callback body.
            StateLifetime::Reaction => "",
        };
        let mut periodic_interval_ms = None;
        for handler in &self.handlers {
            if matches!(handler.event, StateEvent::OwnerKill | StateEvent::OwnerDirectKill { .. } | StateEvent::OwnerSkillCast | StateEvent::OwnerStunEnded)
                && lifetime != StateLifetime::Equipped {
                return Err("OwnerKill, OwnerDirectKill, OwnerSkillCast and OwnerStunEnded require equipment lifetime".into());
            }
            if let StateEvent::Periodic { interval_ms } = handler.event {
                if interval_ms == 0
                    || lifetime != StateLifetime::Equipped
                    || periodic_interval_ms.is_some()
                {
                    return Err("Periodic requires equipment lifetime, one handler and a positive interval_ms".into());
                }
                periodic_interval_ms = Some(interval_ms);
            }
        }
        limits.clone().validate().map_err(str::to_string)?;
        let manual_count = self
            .handlers
            .iter()
            .filter(|h| h.event == StateEvent::ManualInput)
            .count();
        if manual_count > 1
            || (manual_count != 0
                && (!self.owns_use
                    || !self.end_on_source_exit
                    || self.replacement != StateReplacement::ReplaceSameSource))
        {
            return Err("manual input requires one handler, use ownership, source-exit cleanup and ReplaceSameSource".into());
        }
        if let Some(profile) = &self.basic_attack {
            profile.validate()?;
            if profile.followup_on_input {
                return Err(
                    "state basic attack uses a ManualInput handler, not the legacy followup flag"
                        .into(),
                );
            }
        }
        if !valid_id(&self.id)
            || self.fields.len() > MAX_STATE_FIELDS
            || self.handlers.is_empty()
            || self.handlers.len() > MAX_STATE_HANDLERS
        {
            return Err(
                "state program requires bounded identity, fields and 1..32 handlers".into(),
            );
        }
        let mut names = BTreeSet::new();
        let mut fields = Vec::with_capacity(self.fields.len());
        for field in &self.fields {
            if !valid_id(&field.name) || !names.insert(field.name.as_str()) {
                return Err(format!(
                    "state {}: invalid or duplicate field {}",
                    self.id, field.name
                ));
            }
            let (unit, initial, min, max) = match &field.value {
                StateFieldDef::Bool(value) => (None, StateScalar::Bool(*value), 0, 0),
                StateFieldDef::Integer {
                    unit,
                    initial,
                    min,
                    max,
                } => {
                    if !allowed_unit(unit)
                        || min > initial
                        || initial > max
                        || (unit != &ValueUnit::Scalar && *min < 0)
                    {
                        return Err(format!(
                            "state {}.{}: invalid type or field bounds",
                            self.id, field.name
                        ));
                    }
                    (
                        Some(unit.clone()),
                        StateScalar::Integer(i128::from(*initial)),
                        i128::from(*min),
                        i128::from(*max),
                    )
                }
            };
            fields.push(CompiledStateField {
                name: field.name.clone(),
                unit,
                initial,
                min,
                max,
            });
        }
        let retained = size_of::<CompiledStateProgram>()
            + self.id.len()
            + fields.capacity() * size_of::<CompiledStateField>()
            + fields.iter().map(|f| f.name.capacity()).sum::<usize>()
            + self.handlers.len() * size_of::<CompiledStateHandler>();
        let mut compiler = Compiler {
            program: self,
            source,
            limits,
            fields: &fields,
            operations: 0,
            conditions: 0,
            expression_nodes: 0,
            retained,
            work: (self.fields.len() + self.handlers.len()) as u64,
            deadline,
            reaction,
            equipped: lifetime == StateLifetime::Equipped,
            state_path,
        };
        let mut handlers = Vec::with_capacity(self.handlers.len());
        names.clear();
        for (index, handler) in self.handlers.iter().enumerate() {
            if !valid_id(&handler.id) || !names.insert(handler.id.as_str()) {
                return Err(format!(
                    "state {}: invalid or duplicate handler {}",
                    self.id, handler.id
                ));
            }
            handlers.push(CompiledStateHandler {
                event: handler.event,
                body: compiler.block(
                    &handler.body,
                    handler.event,
                    &format!("handlers[{index}].body"),
                    0,
                )?,
            });
        }
        let retained_bytes = compiler.retained;
        let compile_work = compiler.work;
        let has_expressions = compiler.expression_nodes != 0;
        // Plan storage includes every expression's source metadata, so this
        // also bounds copies retained by planned reaction commands.
        let reaction_bytes = retained_bytes
            + compiler.operations
                * (std::mem::size_of::<crate::game::battle::damage::BattleCommand>()
                    + std::mem::size_of::<(i32, Arc<EffectSource>)>()
                    + 128)
            + fields.len() * size_of::<StateScalar>();
        if retained_bytes > limits.max_retained_bytes || compiler.work > limits.max_compile_work {
            return Err("state program exceeds compiled storage limit".into());
        }
        Ok(Arc::new(CompiledStateProgram {
            id: self.id.clone(),
            end_on_source_exit: self.end_on_source_exit,
            replacement: self.replacement,
            fields,
            handlers,
            retained_bytes,
            reaction_bytes,
            compile_work,
            has_expressions,
            periodic_interval_ms,
        }))
    }
}
impl Compiler<'_> {
    fn field(&self, name: &str) -> Result<usize, String> {
        self.fields
            .iter()
            .position(|f| f.name == name)
            .ok_or_else(|| format!("state {}: unknown field {name}", self.program.id))
    }
    fn effect_source(&self, path: &str) -> EffectSource {
        let mut source = self.source.cloned().unwrap_or_else(|| EffectSource {
            source_file: format!("state:{}", self.program.id),
            site: EffectSite {
                action: ActionSite {
                    definition_id: self.program.id.clone().into(),
                    action_id: "reaction".into(),
                },
                effect_id: path.into(),
            },
            field_path: path.into(),
            origin_definition_id: self.program.id.clone().into(),
            origin: Default::default(),
        });
        if self.source.is_some() {
            if self.reaction {
                source
                    .field_path
                    .push_str(path.strip_prefix("handlers[0].body").unwrap_or(path));
            } else {
                source.field_path.push_str(self.state_path);
                source.field_path.push_str(path);
            }
        }
        source
    }
    fn expression(
        &mut self,
        value: &AmountExpr,
        event: StateEvent,
        path: &str,
    ) -> Result<Arc<CompiledAmountPlan>, String> {
        if value.moment != expr::EvalMoment::Live {
            return Err(format!(
                "{path}: state reactions require explicit Live evaluation at the event"
            ));
        }
        let source = self.effect_source(path);
        let mut capabilities = DeliveryCapabilities::constant_only();
        capabilities.state_context = true;
        capabilities.resources = self.equipped;
        capabilities.target_at_release = true;
        capabilities.target_at_consume = true;
        capabilities.deadline = self.deadline;
        let mut limits = self.limits.clone();
        limits.max_compile_work = limits
            .max_compile_work
            .checked_sub(self.work)
            .filter(|v| *v > 0)
            .ok_or("state compile work exhausted")?;
        let plan = expr::compile(value, &source, capabilities, &limits)
            .map_err(|e| format!("{path}: {e}"))?;
        for reference in &plan.requirements().references {
            match reference {
                ValueRef::StateField { name, unit } => {
                    if self.fields[self.field(name)?].unit.as_ref() != Some(unit) {
                        return Err(format!("{path}: field {name} has a different type"));
                    }
                }
                ValueRef::Resource { .. } if self.equipped => {},
                ValueRef::EventHpLost
                    if matches!(
                        event,
                        StateEvent::HpLost { .. } | StateEvent::DirectHpLost { .. }
                    ) => {}
                ValueRef::HitPoints {
                    subject: ValueSubject::Target,
                    ..
                }
                | ValueRef::Stat {
                    subject: ValueSubject::Target,
                    ..
                }
                | ValueRef::RelationCount { subject: ValueSubject::Target, .. }
                | ValueRef::Time(_) => {}
                _ => {
                    return Err(format!(
                        "{path}: unavailable state event context {reference:?}"
                    ))
                }
            }
        }
        if plan.requirements().resolved_targets {
            return Err(format!(
                "{path}: state event has no resolved target collection"
            ));
        }
        self.expression_nodes = self
            .expression_nodes
            .checked_add(plan.cost().nodes)
            .ok_or("state expression size overflow")?;
        self.retained = self
            .retained
            .checked_add(
                plan.cost().plan_retained_bytes
                    + size_of::<CompiledAmountPlan>()
                    + 2 * size_of::<usize>(),
            )
            .ok_or("state storage overflow")?;
        self.work = self
            .work
            .checked_add(plan.cost().compile_work)
            .ok_or("state work overflow")?;
        if self.expression_nodes > self.limits.max_nodes
            || self.retained > self.limits.max_retained_bytes
        {
            return Err(format!("{path}: state expression budget exceeded"));
        }
        Ok(Arc::new(plan))
    }
    fn condition(
        &mut self,
        condition: &StateCondition,
        event: StateEvent,
        path: &str,
        depth: usize,
    ) -> Result<CompiledStateCondition, String> {
        self.conditions += 1;
        self.work += 1;
        if depth > 1 {
            self.retained += size_of::<CompiledStateCondition>();
        }
        if depth > MAX_STATE_DEPTH
            || self.conditions > MAX_STATE_OPERATIONS
            || self.work > self.limits.max_compile_work
            || self.retained > self.limits.max_retained_bytes
        {
            return Err(format!("{path}: state condition budget exceeded"));
        }
        Ok(match condition {
            StateCondition::Always(value) => CompiledStateCondition::Always(*value),
            StateCondition::EventVictimHadOwnerDot => {
                if !matches!(event, StateEvent::OwnerKill | StateEvent::OwnerDirectKill { .. }) {
                    return Err(format!("{path}: EventVictimHadOwnerDot requires OwnerKill"));
                }
                CompiledStateCondition::EventVictimHadOwnerDot
            }
            StateCondition::Predicate(_) => {
                return Err(format!(
                    "{path}: unresolved authored predicate; compile the composition module first"
                ))
            }
            StateCondition::Logic(logic) => {
                if logic.children().is_empty() {
                    return Err(format!("{path}: boolean group must be nonempty"));
                }
                CompiledStateCondition::Logic(logic.try_map(|i, child| {
                    self.condition(
                        child,
                        event,
                        &format!("{path}.Logic{}", logic.child_suffix(i)),
                        depth + 1,
                    )
                })?)
            }
            StateCondition::Compare {
                left,
                comparison,
                right,
            } => {
                let left = self.expression(left, event, &format!("{path}.left"))?;
                let right = self.expression(right, event, &format!("{path}.right"))?;
                if left.result_unit() != right.result_unit() {
                    return Err(format!("{path}: comparison type mismatch"));
                }
                CompiledStateCondition::Compare {
                    left,
                    comparison: *comparison,
                    right,
                }
            }
            StateCondition::Bool { field, equals } => {
                let field = self.field(field)?;
                if self.fields[field].unit.is_some() {
                    return Err(format!("{path}: boolean condition requires Bool field"));
                }
                CompiledStateCondition::Bool {
                    field,
                    equals: *equals,
                }
            }
        })
    }
    fn block(
        &mut self,
        ops: &[StateOp],
        event: StateEvent,
        path: &str,
        depth: usize,
    ) -> Result<Vec<CompiledStateOp>, String> {
        self.operations = self
            .operations
            .checked_add(ops.len())
            .ok_or("state operation size overflow")?;
        self.work = self
            .work
            .checked_add(ops.len() as u64)
            .ok_or("state work overflow")?;
        if self.work > self.limits.max_compile_work {
            return Err("state compile work exhausted".into());
        }
        if depth > MAX_STATE_DEPTH || self.operations > MAX_STATE_OPERATIONS {
            return Err(format!("{path}: state operation budget exceeded"));
        }
        self.retained += ops.len() * size_of::<CompiledStateOp>();
        let mut result = Vec::with_capacity(ops.len());
        for (index, op) in ops.iter().enumerate() {
            let path = format!("{path}[{index}]");
            if matches!(
                result.last(),
                Some(CompiledStateOp::End | CompiledStateOp::EndAndInvoke { .. })
            ) {
                return Err(format!("{path}: unreachable operation after End"));
            }
            result.push(match op {
                StateOp::AddCastDamage { percent } => {
                    if event != StateEvent::OwnerSkillCast {
                        return Err(format!("{path}: AddCastDamage requires OwnerSkillCast"));
                    }
                    let percent = self.expression(percent, event, &format!("{path}.percent"))?;
                    if percent.result_unit() != &ValueUnit::Scalar {
                        return Err(format!("{path}.percent: cast damage requires Scalar percent"));
                    }
                    CompiledStateOp::AddCastDamage { percent }
                }
                StateOp::Chance { percent, then_ops, else_ops } => {
                    if !matches!(event, StateEvent::OwnerKill | StateEvent::OwnerDirectKill { .. }) {
                        return Err(format!("{path}: Chance requires an equipped kill event"));
                    }
                    let percent = self.expression(percent, event, &format!("{path}.percent"))?;
                    if percent.result_unit() != &ValueUnit::Scalar {
                        return Err(format!("{path}.percent: chance requires Scalar percent"));
                    }
                    // Stable authored operation path, independent from observations and success counts.
                    let roll_site = path.bytes().fold(0xcbf29ce484222325u64, |h,b| (h ^ u64::from(b)).wrapping_mul(0x100000001b3));
                    CompiledStateOp::Chance { percent, roll_site,
                        then_ops: self.block(then_ops, event, &format!("{path}.then"), depth + 1)?,
                        else_ops: self.block(else_ops, event, &format!("{path}.else"), depth + 1)? }
                }
                StateOp::Set { field, value } => {
                    let field = self.field(field)?;
                    let value = self.expression(value, event, &path)?;
                    if self.fields[field].unit.as_ref() != Some(value.result_unit()) {
                        return Err(format!("{path}: field assignment type mismatch"));
                    }
                    CompiledStateOp::Set { field, value }
                }
                StateOp::SetBool { field, value } => {
                    let field = self.field(field)?;
                    if self.fields[field].unit.is_some() {
                        return Err(format!("{path}: boolean assignment to numeric field"));
                    }
                    CompiledStateOp::SetBool {
                        field,
                        value: *value,
                    }
                }
                StateOp::HealOwner { amount } => {
                    let amount = self.expression(amount, event, &path)?;
                    if amount.result_unit() != &ValueUnit::Magnitude {
                        return Err(format!("{path}: healing requires Magnitude"));
                    }
                    CompiledStateOp::HealOwner { amount }
                }
                StateOp::GrantOwnerResonance { amount } => {
                    if !matches!(event, StateEvent::OwnerKill | StateEvent::OwnerDirectKill { .. } | StateEvent::BasicAttackKill { .. }) {
                        return Err(format!("{path}: GrantOwnerResonance requires an owner-kill or BasicAttackKill event"));
                    }
                    let amount = self.expression(amount, event, &path)?;
                    if amount.result_unit() != &ValueUnit::Magnitude {
                        return Err(format!("{path}: resonance reward requires Magnitude"));
                    }
                    CompiledStateOp::GrantOwnerResonance { amount }
                }
                StateOp::ApplyOwnerState { duration, profile, replacement } => {
                    profile.validate().map_err(|e| format!("{path}.profile: {e}"))?;
                    let duration = self.expression(duration, event, &format!("{path}.duration"))?;
                    if duration.result_unit() != &ValueUnit::Milliseconds {
                        return Err(format!("{path}.duration: state duration requires Milliseconds"));
                    }
                    self.retained += size_of::<super::sustained::TargetStateProfile>()
                        + 2 * size_of::<usize>() + profile.definition.len();
                    if self.retained > self.limits.max_retained_bytes {
                        return Err(format!("{path}: state profile budget exceeded"));
                    }
                    CompiledStateOp::ApplyOwnerState {
                        duration, profile: Arc::new(profile.clone()), replacement: *replacement,
                    }
                }
                StateOp::ApplyEventTargetStatus { duration, profile } => {
                    profile.validate_authored_provenance()
                        .map_err(|reason| format!("{path}.profile: {reason}"))?;
                    if event != StateEvent::OwnerStunEnded {
                        return Err(format!("{path}: ApplyEventTargetStatus requires OwnerStunEnded"));
                    }
                    profile.validate().map_err(|e| format!("{path}.profile: {e}"))?;
                    let duration = self.expression(duration, event, &format!("{path}.duration"))?;
                    if duration.result_unit() != &ValueUnit::Milliseconds {
                        return Err(format!("{path}.duration: status duration requires Milliseconds"));
                    }
                    self.retained = self.retained
                        .checked_add(size_of::<super::sustained::HitStatusProfile>() + 2 * size_of::<usize>())
                        .and_then(|n| n.checked_add(profile.retained_extent()?))
                        .ok_or_else(|| format!("{path}: status profile size overflow"))?;
                    if self.retained > self.limits.max_retained_bytes {
                        return Err(format!("{path}: status profile budget exceeded"));
                    }
                    CompiledStateOp::ApplyEventTargetStatus {
                        duration, profile: Arc::new(profile.clone()),
                    }
                }
                StateOp::Resources { operations } => {
                    use crate::game::resource::{store::ResourceLimits, ResourceOperation};
                    if operations.is_empty() || operations.len() > ResourceLimits::default().batch {
                        return Err(format!("{path}: Resources requires a nonempty bounded operation batch"));
                    }
                    let source = self.effect_source(&path);
                    let origin_bytes = match &source.origin {
                        super::authoring::EffectOrigin::Original => 0,
                        super::authoring::EffectOrigin::Added { source_definition_id } => {
                            source_definition_id.as_str().len()
                        }
                    };
                    self.retained += operations.len() * size_of::<ResourceOperation>()
                        + operations.iter().map(|op| op.resource.as_str().len()).sum::<usize>()
                        + size_of::<Vec<ResourceOperation>>() + 4 * size_of::<usize>()
                        + size_of::<EffectSource>() + source.source_file.capacity()
                        + source.field_path.capacity() + source.site.action.definition_id.as_str().len()
                        + source.site.action.action_id.capacity() + source.site.effect_id.capacity()
                        + source.origin_definition_id.as_str().len() + origin_bytes;
                    self.work += operations.len() as u64;
                    if self.retained > self.limits.max_retained_bytes || self.work > self.limits.max_compile_work {
                        return Err(format!("{path}: state resource budget exceeded"));
                    }
                    CompiledStateOp::Resources {
                        operations: Arc::new(operations.clone()), source: Arc::new(source),
                    }
                }
                StateOp::If {
                    condition,
                    then_ops,
                    else_ops,
                } => {
                    let condition = self.condition(condition, event, &format!("{path}.condition"), 1)?;
                    CompiledStateOp::If {
                        condition,
                        then_ops: self.block(
                            then_ops,
                            event,
                            &format!("{path}.then"),
                            depth + 1,
                        )?,
                        else_ops: self.block(
                            else_ops,
                            event,
                            &format!("{path}.else"),
                            depth + 1,
                        )?,
                    }
                }
                StateOp::End => CompiledStateOp::End,
                StateOp::EndAndInvoke { skill } => {
                    if event != StateEvent::ManualInput || !valid_id(skill.as_str()) {
                        return Err(format!("{path}: EndAndInvoke requires ManualInput and a bounded skill reference"));
                    }
                    self.retained += skill.as_str().len();
                    CompiledStateOp::EndAndInvoke { skill: skill.clone() }
                }
            });
        }
        Ok(result)
    }
}

mod equipped_resources;
