//! Generation-bound finite programs. These are ordinary skill/Flow consumers;
//! event subscription, participant admission and periodic roots belong to the
//! battle, and cannot be reconstructed from their public explanations.
use super::{
    authoring::*, compiler, execution::EffectiveProviderPlan, validation, FaultCode,
    PlanContribution, RelicPlanError, Work,
};
use crate::game::{
    ability::{
        authoring::{ActionSite, AuthoredSkillEffect, EffectOrigin, EffectSite, EffectSource},
        flow::{CompiledSkillFlow, FlowMilestone, SkillFlowDef},
        SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillId, SkillStepCondition, SkillStepDef,
        SkillStepRepeat, SkillTarget, SkillTargetReach, SkillUseLifetime, StepTargetingMode,
    },
    data::{relic_data::RelicDamageSelector, skill_data::SkillDatabase, GameDataBase},
    value_expr::ValueExprCatalog,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Debug, Clone, Serialize)]
pub(crate) enum CausalPayloadKind {
    Program,
    Fork,
    Join { then_program: usize },
}
#[derive(Clone)]
pub(crate) struct EffectiveCausalProgram {
    pub id: String,
    /// Existing identity namespace, retained as bounded scalar observation so
    /// a branch named `then` remains distinct from the Join continuation.
    pub site_kind:u8,
    pub site_index:usize,
    pub contribution: PlanContribution,
    pub event: Option<CausalEventDef>,
    pub execution: EffectiveProviderPlan,
    pub targets: Vec<CausalTargetDef>,
    pub(super) retained_bytes: usize,
}
impl EffectiveCausalProgram {
    pub(crate) fn retained_extent(&self) -> usize {
        self.retained_bytes
    }
}
#[derive(Clone, Serialize)]
pub(crate) struct EffectiveCausalRule {
    pub contribution: PlanContribution,
    pub selector: StructuralSelectorDef,
    pub condition: RelicConditionDef,
    pub event: Option<CausalEventDef>,
    pub source: RelicDamageSelector,
    pub interval_ms: Option<u64>,
    pub payload: CausalPayloadDef,
    pub kind: CausalPayloadKind,
    #[serde(skip)]
    pub programs: Vec<Arc<EffectiveCausalProgram>>,
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct EffectiveWedgeRule {
    pub contribution: PlanContribution,
    pub selector: StructuralSelectorDef,
    pub condition: RelicConditionDef,
    pub increment_percent: u32,
    pub cap_percent: Option<u32>,
    pub effect_sites: Vec<EffectSite>,
}
fn invalid(cause: impl Into<String>) -> RelicPlanError {
    RelicPlanError::new(FaultCode::InvalidReference, cause)
}
#[derive(Clone, Copy)]
enum StaticTruth {
    False,
    True,
    Unknown,
}
pub(crate) fn owner_provider_matches(
    p: &ProviderPredicateDef,
    providers: &[super::ProspectiveProvider],
) -> bool {
    (p.equipment_id.is_none() && p.fragment_id.is_none() && p.form_id.is_none())
        || providers
            .iter()
            .any(|actual| super::closure::provider_matches(p, actual))
}
fn static_condition(
    c: &RelicConditionDef,
    effects: &[super::SharedEffect],
    providers: &[super::ProspectiveProvider],
    has_event_effect: bool,
    work: &mut Work,
) -> Result<StaticTruth, RelicPlanError> {
    use StaticTruth::*;
    work.charge(1, "causal_static_condition")?;
    Ok(match c {
        RelicConditionDef::Always => True,
        RelicConditionDef::SameEffect(_) => {
            if has_event_effect {
                Unknown
            } else {
                False
            }
        }
        RelicConditionDef::Exists(s) => {
            let mut found = false;
            for e in effects.iter().filter(|e| e.present) {
                if super::closure::matches(s, e, providers, true, work)? {
                    found = true;
                    break;
                }
            }
            if found {
                True
            } else {
                False
            }
        }
        RelicConditionDef::All(v) => {
            let mut unknown = false;
            let mut failed = false;
            for c in v {
                match static_condition(c, effects, providers, has_event_effect, work)? {
                    False => {
                        failed = true;
                        break;
                    }
                    Unknown => unknown = true,
                    True => {}
                }
            }
            if failed {
                False
            } else if unknown {
                Unknown
            } else {
                True
            }
        }
        RelicConditionDef::Any(v) => {
            let mut unknown = false;
            let mut found = false;
            for c in v {
                match static_condition(c, effects, providers, has_event_effect, work)? {
                    True => {
                        found = true;
                        break;
                    }
                    Unknown => unknown = true,
                    False => {}
                }
            }
            if found {
                True
            } else if unknown {
                Unknown
            } else {
                False
            }
        }
        RelicConditionDef::Not(c) => {
            match static_condition(c, effects, providers, has_event_effect, work)? {
                True => False,
                False => True,
                Unknown => Unknown,
            }
        }
    })
}
pub(super) fn rule_state(
    rule: &RelicRuleDef,
    effects: &[super::SharedEffect],
    providers: &[super::ProspectiveProvider],
    work: &mut Work,
) -> Result<super::dto::RelicRuleState, RelicPlanError> {
    use super::dto::{
        RelicDormantReason as Dormant, RelicPendingReason as Pending, RelicRuleState as State,
    };
    work.charge(providers.len(), "causal_static_provider")?;
    if !owner_provider_matches(&rule.selector.provider, providers) {
        return Ok(State::Dormant {
            reason: Dormant::ProviderUnavailable,
        });
    }
    let has_event_effect = matches!(
        &rule.apply,
        RelicRuleActionDef::Subscribe(CausalSubscriptionDef {
            event: CausalEventDef::DamageHpLost { .. },
            ..
        })
    );
    let condition = static_condition(&rule.when, effects, providers, has_event_effect, work)?;
    if matches!(condition, StaticTruth::False) {
        return Ok(State::Dormant {
            reason: Dormant::ConditionUnmet,
        });
    }
    if !has_event_effect && rule.selector.effect != EffectPredicateDef::default() {
        return Ok(State::Dormant {
            reason: Dormant::NoMatchingEffect,
        });
    }
    let source_pending = matches!(&rule.apply,RelicRuleActionDef::Subscribe(s) if s.source!=RelicDamageSelector::default());
    if matches!(condition, StaticTruth::Unknown)
        || rule.selector.effect != EffectPredicateDef::default()
        || source_pending
    {
        Ok(State::Pending {
            reason: Pending::ActualEventPredicate,
        })
    } else {
        Ok(State::Active)
    }
}

pub(super) fn validate_action(action: &RelicRuleActionDef) -> Result<(), RelicPlanError> {
    match action {
        RelicRuleActionDef::Subscribe(s) => {
            let mut tags = BTreeSet::new();
            for tag in &s.source.all_tags {
                if !validation::valid_id(tag) || !tags.insert(tag) {
                    return Err(invalid("invalid or duplicate causal source tag"));
                }
            }
            for id in s
                .source
                .equipment_id
                .iter()
                .chain(&s.source.fragment_id)
                .chain(&s.source.form_id)
            {
                if !validation::valid_id(id) {
                    return Err(invalid("invalid causal source provider"));
                }
            }
            if !matches!(s.event, CausalEventDef::DamageHpLost { .. })
                && s.source != RelicDamageSelector::default()
            {
                return Err(invalid("damage source selector requires DamageHpLost"));
            }
            if s.source.form_id.is_some() && s.source.fragment_id.is_none() {
                return Err(invalid("causal form requires fragment"));
            }
            validate_payload(&s.payload)
        }
        RelicRuleActionDef::Periodic {
            interval_ms,
            payload,
        } => {
            if *interval_ms == 0 {
                return Err(invalid("independent periodic interval must be positive"));
            }
            validate_payload(payload)
        }
        RelicRuleActionDef::Wedge { .. } => Ok(()),
        _ => Err(invalid("not a causal action")),
    }
}
fn validate_payload(payload: &CausalPayloadDef) -> Result<(), RelicPlanError> {
    match payload {
        CausalPayloadDef::Program(p) => validate_program(p),
        CausalPayloadDef::Fork { branches } | CausalPayloadDef::Join { branches, .. } => {
            if branches.is_empty() {
                return Err(invalid("causal branches must be nonempty"));
            }
            let mut ids = BTreeSet::new();
            for branch in branches {
                if !validation::valid_id(&branch.id) || !ids.insert(&branch.id) {
                    return Err(invalid("invalid or duplicate causal participant ID"));
                }
                validate_program(&branch.program)?;
            }
            if let CausalPayloadDef::Join { then, .. } = payload {
                validate_program(then)?;
            }
            Ok(())
        }
    }
}
fn validate_program(p: &CausalProgramDef) -> Result<(), RelicPlanError> {
    if p.steps.is_empty() {
        return Err(invalid("causal program must have finite steps"));
    }
    let mut steps = BTreeSet::new();
    for step in &p.steps {
        if !validation::valid_id(&step.id) || !steps.insert(&step.id) {
            return Err(invalid("invalid or duplicate causal step ID"));
        }
        step.delivery
            .try_validate_projectile_scalar_contract()
            .map_err(|e| invalid(e.cause))?;
        if step.effects.is_empty() {
            return Err(invalid("causal step must have effects"));
        }
        let mut effects = BTreeSet::new();
        for e in &step.effects {
            if !validation::valid_id(&e.id) || !effects.insert(&e.id) {
                return Err(invalid("invalid or duplicate causal effect ID"));
            }
            let mut tags = BTreeSet::new();
            for tag in &e.tags {
                if !validation::valid_id(tag) || !tags.insert(tag) {
                    return Err(invalid("invalid or duplicate causal effect tag"));
                }
            }
            match &e.op {
                CausalPrimitiveDef::Damage { amount, .. } | CausalPrimitiveDef::Heal { amount }
                    if *amount < 0 =>
                {
                    return Err(invalid("causal magnitude must be nonnegative"))
                }
                CausalPrimitiveDef::ApplyBuff {
                    buff_id,
                    duration_ms,
                } if !validation::valid_id(buff_id) || *duration_ms == 0 => {
                    return Err(invalid(
                        "causal buff requires identity and finite positive duration",
                    ))
                }
                CausalPrimitiveDef::Resources { operations } if operations.is_empty() => {
                    return Err(invalid("causal Resources must have operations"))
                }
                _ => {}
            }
        }
    }
    Ok(())
}
fn primitive(p: &CausalPrimitiveDef) -> SkillEffectDef {
    match p {
        CausalPrimitiveDef::Damage {
            amount,
            damage_type,
        } => SkillEffectDef::Damage {
            amount: *amount,
            damage_type: *damage_type,
        },
        CausalPrimitiveDef::DamageExpr {
            amount,
            damage_type,
        } => SkillEffectDef::DamageExpr {
            amount: (**amount).clone(),
            damage_type: *damage_type,
        },
        CausalPrimitiveDef::Heal { amount } => SkillEffectDef::Heal { amount: *amount },
        CausalPrimitiveDef::HealExpr { amount } => SkillEffectDef::HealExpr {
            amount: (**amount).clone(),
        },
        CausalPrimitiveDef::HealthCost {
            amount,
            allow_lethal,
        } => SkillEffectDef::HealthCost {
            amount: *amount,
            allow_lethal: *allow_lethal,
        },
        CausalPrimitiveDef::PersonalShieldGrant { definition } => {
            SkillEffectDef::PersonalShieldGrant {
                definition: definition.clone(),
            }
        }
        CausalPrimitiveDef::ModifyResonance { amount } => {
            SkillEffectDef::ModifyResonance { amount: *amount }
        }
        CausalPrimitiveDef::Resources { operations } => SkillEffectDef::Resources {
            operations: operations.clone(),
        },
        CausalPrimitiveDef::ApplyBuff {
            buff_id,
            duration_ms,
        } => SkillEffectDef::ApplyBuff {
            buff_id: buff_id.clone(),
            duration_ms: *duration_ms,
        },
    }
}
fn compile_program(
    c: &PlanContribution,
    event: Option<CausalEventDef>,
    id: &str,
    namespace: (&str, usize),
    p: &CausalProgramDef,
    data: &GameDataBase,
    work: &mut Work,
) -> Result<Arc<EffectiveCausalProgram>, RelicPlanError> {
    compiler::admit_owned_copy(p, work)?;
    compiler::admit_owned_copy(c, work)?;
    let identity_input = (c.copy_uuid, &c.rule_id, id, namespace);
    let identity_bytes = compiler::serialized_size(&identity_input, work)?;
    work.reserve(
        identity_bytes
            .checked_mul(2)
            .and_then(|n| n.checked_add(128)),
        "causal_program_identity",
    )?;
    let identity = serde_json::to_vec(&identity_input).map_err(|e| invalid(e.to_string()))?;
    work.charge(identity.len().div_ceil(32), "causal_program_identity_hash")?;
    let skill_id = SkillId::from(format!("crce_{:x}", Sha256::digest(&identity)));
    let mut steps = Vec::new();
    work.reserve(
        p.steps
            .len()
            .checked_mul(std::mem::size_of::<SkillStepDef>()),
        "causal_program_steps",
    )?;
    for raw in &p.steps {
        let mut effects = Vec::new();
        work.reserve(
            raw.effects
                .len()
                .checked_mul(std::mem::size_of::<AuthoredSkillEffect>()),
            "causal_program_effects",
        )?;
        for raw_effect in &raw.effects {
            if let CausalPrimitiveDef::ApplyBuff { buff_id, .. } = &raw_effect.op {
                work.charge(data.buff_data.buffs.len(), "causal_buff_reference")?;
                if !data.buff_data.buffs.iter().any(|b| b.name == *buff_id) {
                    return Err(invalid(format!("unknown causal buff {buff_id}")));
                }
            }
            if let CausalPrimitiveDef::Resources { operations } = &raw_effect.op {
                for op in operations {
                    work.charge(
                        data.resource_data.resources.len(),
                        "causal_resource_reference",
                    )?;
                    data.resource_data
                        .resources
                        .iter()
                        .find(|d| d.id == op.resource)
                        .ok_or_else(|| invalid("unknown causal resource"))?
                        .validate_selector(op.owner)
                        .map_err(|e| invalid(e.to_string()))?;
                }
            }
            let mut e = AuthoredSkillEffect::new(raw_effect.id.clone(), primitive(&raw_effect.op));
            e.tags = raw_effect.tags.clone();
            e.origin = EffectOrigin::Added {
                source_definition_id: skill_id.clone(),
            };
            e.source = Some(Arc::new(EffectSource {
                source_file: "effective:relic_causal".into(),
                site: EffectSite {
                    action: ActionSite {
                        definition_id: skill_id.clone(),
                        action_id: raw.id.clone(),
                    },
                    effect_id: raw_effect.id.clone(),
                },
                field_path: format!(
                    "rules.{}.program.{id}.step.{}.effect.{}",
                    c.rule_id, raw.id, raw_effect.id
                ),
                origin_definition_id: skill_id.clone(),
                origin: e.origin.clone(),
            }));
            effects.push(e);
        }
        // The native context resolver supplies the captured role target. It
        // never invokes automatic acquisition for a missing explicit role.
        steps.push(SkillStepDef {
            id: raw.id.clone(),
            delay_ms: raw.delay_ms,
            target: if raw.target == CausalTargetDef::Owner {
                SkillTarget::SelfUnit
            } else {
                SkillTarget::CastTarget
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: SkillStepCondition::Always,
            repeat: SkillStepRepeat::Once,
            delivery: (*raw.delivery).clone(),
            effects,
            presentation: Default::default(),
        });
    }
    let flow_source = p.flow.clone().or_else(|| {
        Some(Arc::new(SkillFlowDef::Sequence {
            id: "crce_sequence".into(),
            children: p
                .steps
                .iter()
                .enumerate()
                .map(|(n, step)| SkillFlowDef::Action {
                    id: format!("crce_action_{n}"),
                    step_id: step.id.clone(),
                    wait: FlowMilestone::EffectsSettled,
                })
                .collect(),
        }))
    });
    let skill = SkillDef {
        action_policy: None,
        resonance_admission: None,
        id: skill_id,
        name: format!("relic causal {} {id}", c.rule_id),
        use_lifetime: SkillUseLifetime::Action,
        flow: flow_source,
        kind: Default::default(),
        cast_targeting: SkillCastTargetingDef::explicit(
            SkillTarget::SelfUnit,
            SkillTargetReach::Pattern,
            None,
            true,
        ),
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps,
    };
    // This upper bound covers the synthesized IDs/source paths and default
    // Flow AST as well as the raw payload. The native Flow compiler separately
    // preflights its node/action tables before allocating them.
    compiler::admit_owned_copy(&skill, work)?;
    let flow = Arc::new(
        CompiledSkillFlow::compile(skill.flow.as_ref(), &skill.steps)
            .map_err(|e| invalid(format!("causal flow {}: {}", e.path, e.cause)))?,
    );
    compiler::admit_owned_copy(flow.as_ref(), work)?;
    let db =
        SkillDatabase::from_bound_effective_plan(skill, Arc::clone(&flow), &data.skill_data, &[])
            .map_err(|e| invalid(format!("causal skill contract: {e:?}")))?;
    let amounts = ValueExprCatalog::compile_effective_skill(
        &db,
        &data.resource_data,
        &data.runtime_safety().expression(),
        false,
        work,
    )
    .map_err(|e| invalid(format!("causal amounts: {e:?}")))?;
    amounts
        .ensure_runtime_publishable()
        .map_err(|e| invalid(format!("causal amount capability: {e:?}")))?;
    for (_, entry) in amounts.entries() {
        work.reserve(
            Some(entry.plan().cost().plan_retained_bytes),
            "causal_amount_plan",
        )?;
    }
    work.reserve(
        Some(std::mem::size_of::<EffectiveCausalProgram>() + 128),
        "causal_program_owner",
    )?;
    work.reserve(Some(std::mem::size_of::<Vec<(EffectSite,usize)>>() + 64), "effective_causal_address_owner")?;
    let mut program = EffectiveCausalProgram {
        id: id.into(),
        site_kind:match namespace.0{"program"=>0,"branch"=>1,"continuation"=>2,_=>return Err(invalid("unknown private causal namespace"))},
        site_index:namespace.1,
        contribution: c.clone(),
        event,
        targets: p.steps.iter().map(|s| s.target).collect(),
        retained_bytes: 0,
        execution: EffectiveProviderPlan {
            skill: Arc::new(
                db.skills
                    .into_iter()
                    .next()
                    .ok_or_else(|| invalid("causal skill absent"))?,
            ),
            flow,
            amounts: Arc::new(amounts),
            effects: Arc::new(Vec::new()),
            selected_recipient_reach: None,
            selected_recipient_excludes_caster: false,
        },
    };
    program.retained_bytes = super::storage::measure_causal_program(&program, work)?;
    Ok(Arc::new(program))
}
pub(super) fn compile_rule(
    rule: &RelicRuleDef,
    c: PlanContribution,
    data: &GameDataBase,
    work: &mut Work,
) -> Result<EffectiveCausalRule, RelicPlanError> {
    validate_action(&rule.apply)?;
    let (event, source, interval_ms, payload) = match &rule.apply {
        RelicRuleActionDef::Subscribe(s) => (Some(s.event), s.source.clone(), None, &s.payload),
        RelicRuleActionDef::Periodic {
            interval_ms,
            payload,
        } => (None, Default::default(), Some(*interval_ms), payload),
        _ => return Err(invalid("not a subscriber or periodic rule")),
    };
    compiler::admit_owned_copy(rule, work)?;
    let mut programs = Vec::new();
    let kind = match payload {
        CausalPayloadDef::Program(p) => {
            programs.push(compile_program(
                &c,
                event,
                "program",
                ("program", 0),
                p,
                data,
                work,
            )?);
            CausalPayloadKind::Program
        }
        CausalPayloadDef::Fork { branches } | CausalPayloadDef::Join { branches, .. } => {
            for (index, b) in branches.iter().enumerate() {
                programs.push(compile_program(
                    &c,
                    event,
                    &b.id,
                    ("branch", index),
                    &b.program,
                    data,
                    work,
                )?);
            }
            if let CausalPayloadDef::Join { then, .. } = payload {
                let then_program = programs.len();
                programs.push(compile_program(
                    &c,
                    event,
                    "then",
                    ("continuation", then_program),
                    then,
                    data,
                    work,
                )?);
                CausalPayloadKind::Join { then_program }
            } else {
                CausalPayloadKind::Fork
            }
        }
    };
    work.reserve(
        Some(std::mem::size_of::<EffectiveCausalRule>()),
        "causal_rule_owner",
    )?;
    Ok(EffectiveCausalRule {
        contribution: c,
        selector: rule.selector.clone(),
        condition: rule.when.clone(),
        event,
        source,
        interval_ms,
        payload: payload.clone(),
        kind,
        programs,
    })
}
