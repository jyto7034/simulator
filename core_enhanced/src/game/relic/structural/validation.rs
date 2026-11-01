use super::{authoring::*, capabilities, compose::NumericProgram, FaultCode, RelicPlanError};
use crate::game::{
    ability::authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource},
    data::{relic_data::RelicDefinition, GameDataBase},
    resource::ResourceOwnerSelector,
    value_expr::{self, AmountExpr, DeliveryCapabilities, ValueRef, ValueSubject, ValueUnit},
};

pub(crate) fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
fn invalid(cause: &str) -> RelicPlanError {
    RelicPlanError::new(FaultCode::InvalidReference, cause)
}
pub(crate) fn validate_rules(
    definition: &str,
    rules: &[RelicRuleDef],
) -> Result<(), RelicPlanError> {
    crate::authoring::control::charge_serialized(&rules, "relic_rule_validation")
        .map_err(RelicPlanError::control)?;
    for (i, rule) in rules.iter().enumerate() {
        let validate = || {
            if !valid_id(&rule.id) || rules[..i].iter().any(|r| r.id == rule.id) {
                return Err(invalid("invalid or duplicate rule ID"));
            }
            selector(&rule.selector)?;
            condition(&rule.when, 0)?;
            match &rule.apply {
                RelicRuleActionDef::Subscribe(_) | RelicRuleActionDef::Periodic { .. } => {
                    super::causal::validate_action(&rule.apply)?;
                    if matches!(&rule.apply, RelicRuleActionDef::Periodic { .. }) {
                        fn event_dependent(c: &RelicConditionDef) -> bool {
                            match c {
                                RelicConditionDef::SameEffect(_) => true,
                                RelicConditionDef::All(v) | RelicConditionDef::Any(v) => {
                                    v.iter().any(event_dependent)
                                }
                                RelicConditionDef::Not(c) => event_dependent(c),
                                _ => false,
                            }
                        }
                        if rule.selector.effect != EffectPredicateDef::default()
                            || event_dependent(&rule.when)
                        {
                            return Err(invalid("independent periodic requires owner-only predicates; no triggering effect exists"));
                        }
                    }
                }
                RelicRuleActionDef::Wedge { .. } => {
                    if rule.selector.effect.all_tags.is_empty()
                        || rule
                            .selector
                            .effect
                            .op
                            .is_some_and(|op| op != EffectOperation::Damage)
                        || rule
                            .selector
                            .effect
                            .has_property
                            .is_some_and(|p| p != RelicProperty::DamageAmount)
                    {
                        return Err(invalid("Wedge requires explicit eligible damage tags"));
                    }
                }
                RelicRuleActionDef::Grant(body) => {
                    let mut ids = std::collections::BTreeSet::new();
                    match body {
                        GrantBodyDef::InlineEffects { action_id, effects } => {
                            if !valid_id(action_id) {
                                return Err(invalid("grant requires original action ID"));
                            }
                            effects_valid(effects, &mut ids)?;
                        }
                        GrantBodyDef::FiniteFlow {
                            anchor_action_id,
                            steps,
                            flow,
                        } => {
                            if !valid_id(anchor_action_id) || steps.is_empty() {
                                return Err(invalid("finite flow requires anchor and steps"));
                            }
                            let mut step_ids = std::collections::BTreeSet::new();
                            for step in steps {
                                if !valid_id(&step.id) || !step_ids.insert(&step.id) {
                                    return Err(invalid("duplicate local Flow step ID"));
                                }
                                let mut local_ids = std::collections::BTreeSet::new();
                                effects_valid(&step.effects, &mut local_ids)?;
                                step.delivery
                                    .try_validate_projectile_scalar_contract()
                                    .map_err(|e| invalid(e.cause))?;
                            }
                            // The real Flow compiler runs against the local steps in compiler.rs.
                            let _ = flow;
                        }
                    }
                }
                RelicRuleActionDef::Edit {
                    property,
                    operation,
                } => {
                    if *property == RelicProperty::DamageFootprint {
                        return Err(RelicPlanError::new(
                            FaultCode::UnsupportedOperation,
                            "footprint is a typed recipe",
                        ));
                    }
                    let descriptor = capabilities::structural_capabilities()
                        .iter()
                        .find(|d| d.property == *property)
                        .ok_or_else(|| invalid("unknown property"))?;
                    NumericProgram::default().include(*operation, descriptor)?;
                }
                RelicRuleActionDef::Recipe(recipe) => match recipe {
                    StructuralRecipeDef::ChannelHealthUpkeep { policy } => policy.validate().map_err(|e| invalid(&e))?,
                    StructuralRecipeDef::SupportSelectionPattern { range, .. } => {
                        range.validate().map_err(|e| invalid(&e))?
                    }
                    StructuralRecipeDef::DamageFootprint { footprint } => {
                        footprint.validate().map_err(|e| invalid(&e))?
                    }
                    StructuralRecipeDef::RegularBasicSupport {
                        damage_percent,
                        speed_points,
                    } if *damage_percent > 10000 || *speed_points > 10000 => {
                        return Err(RelicPlanError::new(
                            FaultCode::OutOfRange,
                            "regular coefficient outside existing aura range",
                        ))
                    }
                    _ => {}
                },
            }
            Ok(())
        };
        validate()
            .map_err(|e: RelicPlanError| e.at(definition, &rule.id, format!("rules[{i}]")))?;
    }
    Ok(())
}
fn tags(v: &[String]) -> Result<(), RelicPlanError> {
    for (i, tag) in v.iter().enumerate() {
        crate::authoring::control::charge(
            (tag.len()
                .checked_mul(i + 1)
                .ok_or_else(|| invalid("tag work overflow"))?) as u64,
            "relic_rule_tag_scan",
        )
        .map_err(RelicPlanError::control)?;
        if !valid_id(tag) || v[..i].contains(tag) {
            return Err(invalid("invalid or repeated tag"));
        }
    }
    Ok(())
}
fn predicate(p: &EffectPredicateDef) -> Result<(), RelicPlanError> {
    tags(&p.all_tags)
}
fn selector(p: &StructuralSelectorDef) -> Result<(), RelicPlanError> {
    if p.provider.form_id.is_some() && p.provider.fragment_id.is_none() {
        return Err(invalid("form requires exact fragment"));
    }
    for id in p
        .provider
        .equipment_id
        .iter()
        .chain(&p.provider.fragment_id)
        .chain(&p.provider.form_id)
    {
        if !valid_id(id) {
            return Err(invalid("invalid provider ID"));
        }
    }
    predicate(&p.effect)
}
fn condition(c: &RelicConditionDef, depth: usize) -> Result<(), RelicPlanError> {
    crate::authoring::control::charge(1, "relic_condition_validation")
        .map_err(RelicPlanError::control)?;
    if depth >= value_expr::current_parse_limits().max_depth {
        return Err(RelicPlanError::new(
            FaultCode::BudgetExceeded,
            "condition depth",
        ));
    }
    match c {
        RelicConditionDef::Always => Ok(()),
        RelicConditionDef::SameEffect(p) => predicate(p),
        RelicConditionDef::Exists(s) => selector(s),
        RelicConditionDef::All(v) | RelicConditionDef::Any(v) => {
            for c in v {
                condition(c, depth + 1)?;
            }
            Ok(())
        }
        RelicConditionDef::Not(c) => condition(c, depth + 1),
    }
}
fn effects_valid<'a>(
    effects: &'a [GrantedEffectDef],
    ids: &mut std::collections::BTreeSet<&'a str>,
) -> Result<(), RelicPlanError> {
    if effects.is_empty() {
        return Err(invalid("grant requires at least one finite primitive"));
    }
    for e in effects {
        if !valid_id(&e.id) || !ids.insert(&e.id) {
            return Err(invalid("duplicate or invalid local effect ID"));
        }
        tags(&e.tags)?;
        if let GrantPrimitiveDef::PersonalShieldGrant { definition } = &e.op {
            shield(definition)?;
        }
        if let GrantPrimitiveDef::HealthCost { amount, .. } = e.op {
            if amount == 0 || amount > i32::MAX as u32 {
                return Err(invalid("HP cost must fit its existing positive consumer"));
            }
        }
    }
    Ok(())
}
pub(crate) fn shield(d: &PersonalShieldGrantDef) -> Result<(), RelicPlanError> {
    if let Some(support) = &d.support {
        support.validate(true).map_err(|e| invalid(&e))?;
    }
    if d.duration_ms == 0
        || (d.support.as_ref().and_then(|s| s.range.as_ref()).is_none() && matches!(
            d.recipients,
            PersonalShieldRecipientsDef::PlayerEmployeesAtRelease { radius_units: 0 }
                | PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease {
                    authored_reach_units: 0
                }
        ))
    {
        return Err(RelicPlanError::new(
            FaultCode::OutOfRange,
            "personal grant requires positive duration/reach",
        ));
    }
    if let Some(program) = &d.source_creation {
        let descriptor = capabilities::descriptor(
            EffectOperation::PersonalShieldGrant,
            RelicProperty::PersonalShieldInitialAmount,
        )
        .unwrap();
        let mut result = NumericProgram::default();
        for op in &program.operations {
            result.include(*op, descriptor)?;
        }
        result.validate_dynamic(descriptor)?;
    }
    Ok(())
}
pub(crate) fn provider_references(
    rules: &[RelicRuleDef],
    data: &GameDataBase,
) -> Result<(), RelicPlanError> {
    fn check(s: &StructuralSelectorDef, data: &GameDataBase) -> Result<(), RelicPlanError> {
        if let Some(id) = &s.provider.equipment_id {
            if data.equipment_data.get_by_id(id).is_none() {
                return Err(invalid("unknown equipment"));
            }
        }
        if let Some(id) = &s.provider.fragment_id {
            let f = data
                .skill_fragment_data
                .get_by_id(&id.clone().into())
                .ok_or_else(|| invalid("unknown fragment"))?;
            if let Some(form) = &s.provider.form_id {
                if !matches!(&f.effect, crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill { forms, .. } if forms.contains_key(form))
                {
                    return Err(invalid("unknown fragment form"));
                }
            }
        }
        Ok(())
    }
    fn nested(c: &RelicConditionDef, data: &GameDataBase) -> Result<(), RelicPlanError> {
        match c {
            RelicConditionDef::Exists(s) => check(s, data),
            RelicConditionDef::All(v) | RelicConditionDef::Any(v) => {
                for c in v {
                    nested(c, data)?;
                }
                Ok(())
            }
            RelicConditionDef::Not(c) => nested(c, data),
            _ => Ok(()),
        }
    }
    for r in rules {
        if let RelicRuleActionDef::Subscribe(s) = &r.apply {
            check(
                &StructuralSelectorDef {
                    provider: ProviderPredicateDef {
                        equipment_id: s.source.equipment_id.clone(),
                        fragment_id: s.source.fragment_id.clone(),
                        form_id: s.source.form_id.clone(),
                    },
                    effect: Default::default(),
                },
                data,
            )
            .map_err(|e| e.at("", &r.id, "apply.Subscribe.source"))?;
        }
        check(&r.selector, data)
            .and_then(|()| nested(&r.when, data))
            .map_err(|e| e.at("", &r.id, "selector/when"))?;
    }
    Ok(())
}
pub(crate) fn amount(
    expression: &AmountExpr,
    site: EffectSite,
    source_only: bool,
    data: &GameDataBase,
    caps: DeliveryCapabilities,
    work: &mut super::Work,
) -> Result<Option<i128>, RelicPlanError> {
    let source = EffectSource {
        source_file: "typed:effective_plan".into(),
        field_path: "amount".into(),
        origin_definition_id: site.action.definition_id.clone(),
        site,
        origin: EffectOrigin::Original,
    };
    let mut limits = data.runtime_safety().expression();
    limits.max_compile_work = limits.max_compile_work.min(work.remaining());
    let plan = value_expr::compile(expression, &source, caps, &limits)
        .map_err(|e| RelicPlanError::new(FaultCode::InvalidReference, e.to_string()))?;
    work.charge(
        usize::try_from(plan.cost().compile_work).map_err(|_| invalid("amount work conversion"))?,
        "relic_structural_amount_proof",
    )?;
    if plan.result_unit() != &ValueUnit::Magnitude {
        return Err(RelicPlanError::new(
            FaultCode::UnitMismatch,
            "amount result requires Magnitude",
        ));
    }
    if source_only
        && (plan.requirements().resolved_targets
            || plan.requirements().references.iter().any(|r| {
                matches!(
                    r,
                    ValueRef::Stat {
                        subject: ValueSubject::Target | ValueSubject::AggregateMember,
                        ..
                    } | ValueRef::HitPoints {
                        subject: ValueSubject::Target | ValueSubject::AggregateMember,
                        ..
                    } | ValueRef::Resource {
                        owner: ResourceOwnerSelector::TargetEmployee,
                        ..
                    }
                )
            }))
    {
        return Err(RelicPlanError::new(
            FaultCode::InvalidPreserveContract,
            "personal creation amount cannot read recipients/member set",
        ));
    }
    for r in &plan.requirements().references {
        if let ValueRef::Resource {
            resource, owner, ..
        } = r
        {
            let definition = data
                .resource_data
                .resources
                .iter()
                .find(|r| r.id == *resource)
                .ok_or_else(|| invalid("unknown amount resource"))?;
            definition
                .validate_selector(*owner)
                .map_err(|e| invalid(&e.to_string()))?;
        }
    }
    if value_expr::can_evaluate_without_external_context(&plan) {
        let mut meter = value_expr::ExprMeter::new(plan.evaluation_limits())
            .map_err(|cause| RelicPlanError::new(FaultCode::BudgetExceeded, cause))?;
        let evaluated = value_expr::evaluate(&plan, &value_expr::EvalFrame::default(), &mut meter)
            .map_err(|e| RelicPlanError::new(FaultCode::InvalidReference, e.to_string()))?;
        Ok(Some(evaluated.value.value))
    } else {
        Ok(None)
    }
}
pub(crate) fn rules_for(
    d: &RelicDefinition,
    variant: super::super::RelicVariant,
) -> &[RelicRuleDef] {
    match variant {
        super::super::RelicVariant::Normal => &d.rules,
        super::super::RelicVariant::Corrupt => d.corruption.as_ref().map_or(&d.rules, |v| &v.rules),
    }
}
pub(crate) fn effect_site(
    skill: &crate::game::ability::SkillId,
    action: &str,
    effect: &str,
) -> EffectSite {
    EffectSite {
        action: ActionSite {
            definition_id: skill.clone(),
            action_id: action.into(),
        },
        effect_id: effect.into(),
    }
}
