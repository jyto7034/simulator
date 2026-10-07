//! Private, generation-local lowering. Public DTOs never contain these owners.
//! Structural addresses remain lossless; short local aliases only index the
//! ordinary skill/Flow executor and never identify a relic copy by themselves.
use super::{
    authoring::*, compiler, EffectBody, EffectiveEffect, EffectivePlanComponents, FaultCode,
    RelicPlanError, Work,
};
use crate::game::{
    ability::{
        authoring::{ActionSite, AuthoredSkillEffect, EffectOrigin, EffectSite, EffectSource},
        flow::{CompiledSkillFlow, FlowCondition, SkillFlowDef},
        sustained::{SupportCoefficientScope, SustainedEffectDef},
        SkillCastSelection, SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillStepCondition,
        SkillStepDef, SkillStepRepeat, SkillTarget, SkillTargetReach, StepTargetingMode,
    },
    data::{skill_data::SkillDatabase, GameDataBase},
    value_expr::ValueExprCatalog,
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone)]
pub(crate) struct EffectiveProviderPlan {
    pub skill: Arc<SkillDef>,
    pub flow: Arc<CompiledSkillFlow>,
    pub amounts: Arc<ValueExprCatalog>,
    pub effects: Vec<(EffectSite, usize)>,
    /// Selection is fixed at cast acceptance; this is authored data-unit reach,
    /// not a TileRangePattern or an increased shared protection radius.
    pub selected_recipient_reach: Option<u32>,
    pub selected_recipient_excludes_caster: bool,
}
fn error(cause: impl Into<String>) -> RelicPlanError {
    RelicPlanError::new(FaultCode::InvalidReference, cause)
}
fn value(effect: &EffectiveEffect, property: RelicProperty) -> Option<u32> {
    effect
        .properties
        .iter()
        .find(|p| p.property == property)
        .and_then(|p| p.final_value)
        .and_then(|v| u32::try_from(v.value()).ok())
}
fn overlay(
    op: &mut SkillEffectDef,
    effect: &EffectiveEffect,
) -> Result<Option<u32>, RelicPlanError> {
    let mut reach = None;
    match op {
        SkillEffectDef::PersonalShieldGrant { definition } => {
            if let Some(amount) = &effect.amount {
                definition.amount = Arc::clone(amount);
            }
            let original_reach = match definition.recipients {
                PersonalShieldRecipientsDef::PlayerEmployeesAtRelease { radius_units } => {
                    radius_units
                }
                PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease {
                    authored_reach_units,
                } => authored_reach_units,
            };
            if effect
                .recipes
                .iter()
                .any(|r| matches!(r, StructuralRecipeDef::PersonalShieldSingle { .. }))
            {
                definition.recipients =
                    PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease {
                        authored_reach_units: original_reach,
                    };
            } else if let (
                Some(radius),
                PersonalShieldRecipientsDef::PlayerEmployeesAtRelease { radius_units },
            ) = (
                value(effect, RelicProperty::SupportAreaRadius),
                &mut definition.recipients,
            ) {
                *radius_units = radius;
            }
            if let PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease {
                authored_reach_units,
            } = definition.recipients
            {
                reach = Some(authored_reach_units);
            }
        }
        SkillEffectDef::Sustained {
            definition: SustainedEffectDef::ChannelAura { aura, .. },
        } => {
            if effect
                .recipes
                .iter()
                .any(|r| matches!(r, StructuralRecipeDef::FixedRecipientSupport))
            {
                reach = Some(aura.radius_units);
            }
            if let Some(radius) = value(effect, RelicProperty::SupportAreaRadius) {
                aura.radius_units = radius;
            }
            if let Some(damage) = value(effect, RelicProperty::RegularBasicDamagePercent) {
                aura.basic_damage_percent = damage;
                aura.basic_scope = SupportCoefficientScope::RegularOnly;
            }
            if let Some(speed) = value(effect, RelicProperty::RegularBasicSpeedPoints) {
                aura.basic_speed_points = speed;
                aura.basic_scope = SupportCoefficientScope::RegularOnly;
            }
        }
        _ => {}
    }
    Ok(reach)
}
fn bind_generated(skill: &SkillDef, action: &str, effect: &mut AuthoredSkillEffect) {
    let origin = EffectOrigin::Added {
        source_definition_id: skill.id.clone(),
    };
    effect.origin = origin.clone();
    effect.source = Some(Arc::new(EffectSource {
        source_file: "effective:relic_structural".into(),
        site: EffectSite {
            action: ActionSite {
                definition_id: skill.id.clone(),
                action_id: action.into(),
            },
            effect_id: effect.id.clone(),
        },
        field_path: format!("action.{action}.effect.{}", effect.id),
        origin_definition_id: skill.id.clone(),
        origin,
    }));
}
fn fresh_alias(
    prefix: &str,
    used: &mut Vec<String>,
    work: &mut Work,
) -> Result<String, RelicPlanError> {
    let mut index = used.len();
    loop {
        work.charge(
            used.len()
                .checked_mul(128)
                .ok_or_else(|| error("alias work overflow"))?,
            "effective_alias_lookup",
        )?;
        work.reserve(Some(128), "effective_alias_text")?;
        let candidate = format!("{prefix}_{index}");
        if !used.iter().any(|id| *id == candidate) {
            used.push(candidate.clone());
            return Ok(candidate);
        }
        index = index
            .checked_add(1)
            .ok_or_else(|| error("alias counter overflow"))?;
    }
}
fn rename_flow(
    raw: &SkillFlowDef,
    nodes: &BTreeMap<String, String>,
    steps: &BTreeMap<String, String>,
    effects: &BTreeMap<(String, String), String>,
    producer_steps: &BTreeMap<String, String>,
    work: &mut Work,
) -> Result<SkillFlowDef, RelicPlanError> {
    work.charge(1, "effective_flow_lower")?;
    let id = nodes
        .get(raw.id())
        .ok_or_else(|| error("unknown local Flow node"))?
        .clone();
    let rename_condition = |condition: &FlowCondition| -> Result<FlowCondition, RelicPlanError> {
        Ok(match condition {
            FlowCondition::Always(v) => FlowCondition::Always(*v),
            FlowCondition::Compare {
                node_id,
                metric,
                comparison,
                value,
            } => FlowCondition::Compare {
                node_id: nodes.get(node_id).unwrap_or(node_id).clone(),
                metric: *metric,
                comparison: *comparison,
                value: *value,
            },
            FlowCondition::EffectCompare {
                node_id,
                effect_id,
                metric,
                comparison,
                value,
            } => {
                let local_effect = producer_steps
                    .get(node_id)
                    .and_then(|step| effects.get(&(step.clone(), effect_id.clone())));
                FlowCondition::EffectCompare {
                    node_id: nodes.get(node_id).unwrap_or(node_id).clone(),
                    effect_id: local_effect.unwrap_or(effect_id).clone(),
                    metric: *metric,
                    comparison: *comparison,
                    value: *value,
                }
            }
        })
    };
    Ok(match raw {
        SkillFlowDef::Action { step_id, wait, .. } => SkillFlowDef::Action {
            id,
            step_id: steps
                .get(step_id)
                .ok_or_else(|| error("unknown local Flow action"))?
                .clone(),
            wait: *wait,
        },
        SkillFlowDef::Wait { duration_ms, .. } => SkillFlowDef::Wait {
            id,
            duration_ms: *duration_ms,
        },
        SkillFlowDef::Sequence { children, .. } => SkillFlowDef::Sequence {
            id,
            children: children
                .iter()
                .map(|child| rename_flow(child, nodes, steps, effects, producer_steps, work))
                .collect::<Result<_, _>>()?,
        },
        SkillFlowDef::Parallel { branches, .. } => SkillFlowDef::Parallel {
            id,
            branches: branches
                .iter()
                .map(|child| rename_flow(child, nodes, steps, effects, producer_steps, work))
                .collect::<Result<_, _>>()?,
        },
        SkillFlowDef::Branch {
            condition,
            then_node,
            else_node,
            ..
        } => SkillFlowDef::Branch {
            id,
            condition: rename_condition(condition)?,
            then_node: Box::new(rename_flow(
                then_node,
                nodes,
                steps,
                effects,
                producer_steps,
                work,
            )?),
            else_node: Box::new(rename_flow(
                else_node,
                nodes,
                steps,
                effects,
                producer_steps,
                work,
            )?),
        },
        SkillFlowDef::Repeat {
            count,
            interval_ms,
            body,
            ..
        } => SkillFlowDef::Repeat {
            id,
            count: *count,
            interval_ms: *interval_ms,
            body: Box::new(rename_flow(
                body,
                nodes,
                steps,
                effects,
                producer_steps,
                work,
            )?),
        },
    })
}
fn collect_nodes(
    raw: &SkillFlowDef,
    used: &mut Vec<String>,
    nodes: &mut BTreeMap<String, String>,
    producer_steps: &mut BTreeMap<String, String>,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    let alias = fresh_alias("crsa_flow", used, work)?;
    if nodes.insert(raw.id().into(), alias).is_some() {
        return Err(error("duplicate local Flow node"));
    }
    match raw {
        SkillFlowDef::Action { id, step_id, .. } => {
            producer_steps.insert(id.clone(), step_id.clone());
        }
        SkillFlowDef::Sequence { children, .. }
        | SkillFlowDef::Parallel {
            branches: children, ..
        } => {
            for child in children {
                collect_nodes(child, used, nodes, producer_steps, work)?;
            }
        }
        SkillFlowDef::Branch {
            then_node,
            else_node,
            ..
        } => {
            collect_nodes(then_node, used, nodes, producer_steps, work)?;
            collect_nodes(else_node, used, nodes, producer_steps, work)?;
        }
        SkillFlowDef::Repeat { body, .. } => {
            collect_nodes(body, used, nodes, producer_steps, work)?
        }
        SkillFlowDef::Wait { .. } => {}
    }
    Ok(())
}
fn rename_amount(
    op: &mut SkillEffectDef,
    aliases: &BTreeMap<String, String>,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    use crate::game::value_expr::{ResultSelector, ValueExpr, ValueRef};
    fn walk(
        node: &mut ValueExpr,
        aliases: &BTreeMap<String, String>,
        work: &mut Work,
    ) -> Result<(), RelicPlanError> {
        work.charge(1, "effective_amount_reference_lower")?;
        match node {
            ValueExpr::Read(ValueRef::Result {
                selector: ResultSelector::EarlierEffect(id),
                ..
            }) => {
                work.charge(
                    aliases
                        .len()
                        .checked_mul(id.len() + 128)
                        .ok_or_else(|| error("reference work overflow"))?,
                    "effective_amount_alias_lookup",
                )?;
                if let Some(alias) = aliases.get(id) {
                    work.reserve(Some(alias.len()), "effective_amount_alias")?;
                    *id = alias.clone();
                }
            }
            ValueExpr::Add(children) => {
                for child in children {
                    walk(child, aliases, work)?;
                }
            }
            ValueExpr::Mul { value, scalar } => {
                walk(value, aliases, work)?;
                walk(scalar, aliases, work)?;
            }
            ValueExpr::Ratio {
                numerator,
                denominator,
                ..
            } => {
                walk(numerator, aliases, work)?;
                walk(denominator, aliases, work)?;
            }
            ValueExpr::Clamp { value, min, max } => {
                walk(value, aliases, work)?;
                walk(min, aliases, work)?;
                walk(max, aliases, work)?;
            }
            ValueExpr::Aggregate { member, .. } => walk(member, aliases, work)?,
            _ => {}
        }
        Ok(())
    }
    let rewrite = |amount: &crate::game::value_expr::AmountExpr, work: &mut Work| {
        compiler::admit_serialized_copy(amount, work)?;
        let mut replacement = amount.clone();
        walk(&mut replacement.root, aliases, work)?;
        Ok::<_, RelicPlanError>(replacement)
    };
    match op {
        SkillEffectDef::DamageExpr { amount, .. }
        | SkillEffectDef::HealExpr { amount }
        | SkillEffectDef::ShieldExpr { amount, .. } => *amount = rewrite(amount, work)?,
        SkillEffectDef::PersonalShieldGrant { definition } => {
            definition.amount = Arc::new(rewrite(&definition.amount, work)?)
        }
        _ => {}
    }
    Ok(())
}
/// Validate every declared local producer, including a currently dormant grant,
/// against the real original anchor scope. No forged validated generation is
/// created, and the parent's Action wait remains unchanged.
pub(super) fn validate_grant_flow(
    original: &SkillDef,
    anchor: &str,
    steps: &[GrantedFlowStepDef],
    raw: &Arc<SkillFlowDef>,
    data: &GameDataBase,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    compiler::admit_serialized_copy(original, work)?;
    compiler::admit_serialized_copy(&steps, work)?;
    compiler::admit_serialized_copy(raw, work)?;
    let original_flow = data
        .skill_data
        .compiled_flow(original.id.as_str())
        .map_err(error)?;
    compiler::admit_serialized_copy(original_flow.as_ref(), work)?;
    let mut skill = original.clone();
    let anchor = skill
        .steps
        .iter()
        .position(|step| step.id == anchor)
        .ok_or_else(|| error("unknown original anchor"))?;
    let mut used_steps = skill.steps.iter().map(|step| step.id.clone()).collect();
    let mut used_nodes = original_flow
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect();
    let mut step_names = BTreeMap::new();
    let mut effect_names = BTreeMap::new();
    for step in steps {
        let id = fresh_alias("crsa_action", &mut used_steps, work)?;
        step_names.insert(step.id.clone(), id.clone());
        let mut used = Vec::new();
        let mut aliases = BTreeMap::new();
        for effect in &step.effects {
            let alias = fresh_alias("crsa_effect", &mut used, work)?;
            aliases.insert(effect.id.clone(), alias.clone());
            effect_names.insert((step.id.clone(), effect.id.clone()), alias);
        }
        let mut effects = Vec::with_capacity(step.effects.len());
        for declared in &step.effects {
            let mut effect = AuthoredSkillEffect::new(
                aliases[&declared.id].clone(),
                compiler::primitive(&declared.op),
            );
            effect.tags = declared.tags.clone();
            rename_amount(&mut effect.op, &aliases, work)?;
            bind_generated(&skill, &id, &mut effect);
            effects.push(effect);
        }
        skill.steps.push(SkillStepDef {
            id,
            delay_ms: step.delay_ms,
            target: step.target.clone(),
            targeting: StepTargetingMode::ReuseCastTarget,
            when: SkillStepCondition::Always,
            repeat: SkillStepRepeat::Once,
            delivery: (*step.delivery).clone(),
            effects,
            presentation: Default::default(),
        });
    }
    let mut nodes = BTreeMap::new();
    let mut producers = BTreeMap::new();
    collect_nodes(raw, &mut used_nodes, &mut nodes, &mut producers, work)?;
    let flow = Arc::new(
        (*original_flow)
            .clone()
            .attach_branches(
                &skill.steps,
                &[(
                    anchor,
                    Arc::new(rename_flow(
                        raw,
                        &nodes,
                        &step_names,
                        &effect_names,
                        &producers,
                        work,
                    )?),
                )],
            )
            .map_err(|e| error(format!("{}.{}: {}", skill.id, e.path, e.cause)))?,
    );
    let database = SkillDatabase::from_bound_effective_plan(skill, flow, &data.skill_data, &[])
        .map_err(|e| error(format!("finite declaration: {e:?}")))?;
    let deadline = compiler::amount_caps(
        data,
        original,
        &original.steps[anchor].target,
        &original.steps[anchor].delivery,
        work,
    )?
    .deadline;
    let amounts = ValueExprCatalog::compile_effective_skill(
        &database,
        &data.resource_data,
        &data.runtime_safety().expression(),
        deadline,
        work,
    )
    .map_err(|e| error(format!("finite declaration amounts: {e:?}")))?;
    amounts
        .ensure_runtime_publishable()
        .map_err(|e| error(format!("finite declaration capability: {e:?}")))?;
    Ok(())
}
pub(super) fn lower(
    plan: &EffectivePlanComponents,
    data: &GameDataBase,
    work: &mut Work,
) -> Result<Vec<EffectiveProviderPlan>, RelicPlanError> {
    work.reserve(
        plan.providers
            .len()
            .checked_mul(std::mem::size_of::<EffectiveProviderPlan>()),
        "effective_provider_owner",
    )?;
    let mut execution = Vec::with_capacity(plan.providers.len());
    for (provider, binding) in plan.providers.iter().enumerate() {
        let original = data
            .skill_data
            .get_by_id(binding.skill_id())
            .ok_or_else(|| error("provider skill unavailable"))?;
        compiler::admit_serialized_copy(original, work)?;
        let mut skill = original.clone();
        let original_flow = data
            .skill_data
            .compiled_flow(skill.id.as_str())
            .map_err(error)?;
        compiler::admit_serialized_copy(original_flow.as_ref(), work)?;
        let mut used_steps: Vec<_> = skill.steps.iter().map(|step| step.id.clone()).collect();
        let mut used_nodes: Vec<_> = original_flow
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect();
        let mut address = Vec::new();
        let mut branches = Vec::new();
        let mut selected_recipient_reach = None;
        let mut selected_recipient_excludes_caster = false;
        for (index, effective) in plan
            .effects
            .iter()
            .enumerate()
            .filter(|(_, e)| e.present && e.provider == provider && e.original)
        {
            let EffectBody::Original { step, effect, .. } = &effective.body else {
                return Err(error("original body mismatch"));
            };
            selected_recipient_excludes_caster |= effective
                .recipes
                .iter()
                .any(|recipe| matches!(recipe, StructuralRecipeDef::FixedRecipientSupport));
            let reach = overlay(&mut skill.steps[*step].effects[*effect].op, effective)?;
            if let Some(reach) = reach {
                if selected_recipient_reach.is_some_and(|old| old != reach) {
                    return Err(error("selected support reach conflict"));
                }
                selected_recipient_reach = Some(reach);
            }
            address.push((
                skill.steps[*step].effects[*effect]
                    .source()
                    .ok_or_else(|| error("original source unavailable"))?
                    .site
                    .clone(),
                index,
            ));
        }
        for grant in plan
            .grants
            .iter()
            .filter(|grant| grant.provider == provider)
        {
            let anchor_step = skill
                .steps
                .iter()
                .position(|step| step.id == grant.anchor.action_id)
                .ok_or_else(|| error("unknown grant anchor"))?;
            match grant.body.as_ref() {
                GrantBodyDef::InlineEffects { effects, .. } => {
                    let mut used: Vec<_> = skill.steps[anchor_step]
                        .effects
                        .iter()
                        .map(|effect| effect.id.clone())
                        .collect();
                    let mut aliases = BTreeMap::new();
                    for declared in effects {
                        aliases.insert(
                            declared.id.clone(),
                            fresh_alias("crsa_effect", &mut used, work)?,
                        );
                    }
                    for declared in effects {
                        let index = find_generated(
                            plan,
                            provider,
                            grant,
                            &grant.anchor.action_id,
                            &declared.id,
                            work,
                        )?;
                        let effective = &plan.effects[index];
                        compiler::admit_serialized_copy(declared, work)?;
                        let id = aliases[&declared.id].clone();
                        let mut effect =
                            AuthoredSkillEffect::new(id, compiler::primitive(&declared.op));
                        rename_amount(&mut effect.op, &aliases, work)?;
                        effect.tags = effective.tags.clone();
                        let reach = overlay(&mut effect.op, effective)?;
                        if let Some(reach) = reach {
                            selected_recipient_reach = Some(reach);
                        }
                        bind_generated(&skill, &grant.anchor.action_id, &mut effect);
                        address.push((effect.source().unwrap().site.clone(), index));
                        skill.steps[anchor_step].effects.push(effect);
                    }
                }
                GrantBodyDef::FiniteFlow { steps, flow, .. } => {
                    compiler::admit_serialized_copy(steps, work)?;
                    compiler::admit_serialized_copy(flow, work)?;
                    let mut step_names = BTreeMap::new();
                    let mut effect_names = BTreeMap::new();
                    for step in steps {
                        let id = fresh_alias("crsa_action", &mut used_steps, work)?;
                        step_names.insert(step.id.clone(), id.clone());
                        let mut effects = Vec::with_capacity(step.effects.len());
                        let mut used = Vec::new();
                        let mut aliases = BTreeMap::new();
                        for declared in &step.effects {
                            let alias = fresh_alias("crsa_effect", &mut used, work)?;
                            aliases.insert(declared.id.clone(), alias.clone());
                            effect_names.insert((step.id.clone(), declared.id.clone()), alias);
                        }
                        for declared in &step.effects {
                            let index = find_generated(
                                plan,
                                provider,
                                grant,
                                &step.id,
                                &declared.id,
                                work,
                            )?;
                            let effective = &plan.effects[index];
                            let alias = aliases[&declared.id].clone();
                            let mut effect =
                                AuthoredSkillEffect::new(alias, compiler::primitive(&declared.op));
                            rename_amount(&mut effect.op, &aliases, work)?;
                            effect.tags = effective.tags.clone();
                            if let Some(reach) = overlay(&mut effect.op, effective)? {
                                selected_recipient_reach = Some(reach);
                            }
                            bind_generated(&skill, &id, &mut effect);
                            address.push((effect.source().unwrap().site.clone(), index));
                            effects.push(effect);
                        }
                        skill.steps.push(SkillStepDef {
                            id,
                            delay_ms: step.delay_ms,
                            target: step.target.clone(),
                            targeting: StepTargetingMode::ReuseCastTarget,
                            when: SkillStepCondition::Always,
                            repeat: SkillStepRepeat::Once,
                            delivery: (*step.delivery).clone(),
                            effects,
                            presentation: Default::default(),
                        });
                    }
                    let mut nodes = BTreeMap::new();
                    let mut producers = BTreeMap::new();
                    collect_nodes(flow, &mut used_nodes, &mut nodes, &mut producers, work)?;
                    branches.push((
                        anchor_step,
                        Arc::new(rename_flow(
                            flow,
                            &nodes,
                            &step_names,
                            &effect_names,
                            &producers,
                            work,
                        )?),
                    ));
                }
            }
        }
        if selected_recipient_reach.is_some() {
            // The exact data-unit qualification/reach is a private policy at
            // readiness and release; Global here never increases membership.
            skill.cast_targeting = SkillCastTargetingDef::explicit_with_selection(
                SkillCastSelection::PlayerUnit,
                SkillTarget::AllySingle,
                SkillTargetReach::Global,
                None,
                true,
            );
        }
        let flow = Arc::new(
            (*original_flow)
                .clone()
                .attach_branches(&skill.steps, &branches)
                .map_err(|e| error(format!("{}.{}: {}", skill.id, e.path, e.cause)))?,
        );
        let single_recipe_sites: Vec<_> = address
            .iter()
            .filter(|(_, index)| {
                plan.effects[*index].recipes.iter().any(|recipe| {
                    matches!(recipe, StructuralRecipeDef::PersonalShieldSingle { .. })
                })
            })
            .map(|(site, _)| site.clone())
            .collect();
        let database = SkillDatabase::from_bound_effective_plan(
            skill,
            Arc::clone(&flow),
            &data.skill_data,
            &single_recipe_sites,
        )
        .map_err(|e| error(format!("effective skill contract: {e:?}")))?;
        let deadline = plan
            .effects
            .iter()
            .any(|e| e.provider == provider && e.amount_capabilities.deadline);
        let amounts = ValueExprCatalog::compile_effective_skill(
            &database,
            &data.resource_data,
            &data.runtime_safety().expression(),
            deadline,
            work,
        )
        .map_err(|e| error(format!("effective amount catalog: {e:?}")))?;
        amounts
            .ensure_runtime_publishable()
            .map_err(|e| error(format!("effective amount capability: {e:?}")))?;
        for (_, entry) in amounts.entries() {
            work.reserve(
                Some(entry.plan().cost().plan_retained_bytes),
                "effective_amount_plan",
            )?;
        }
        work.reserve(
            address
                .len()
                .checked_mul(std::mem::size_of::<(EffectSite, usize)>()),
            "effective_address_map",
        )?;
        execution.push(EffectiveProviderPlan {
            skill: Arc::new(database.skills.into_iter().next().unwrap()),
            flow,
            amounts: Arc::new(amounts),
            effects: address,
            selected_recipient_reach,
            selected_recipient_excludes_caster,
        });
    }
    Ok(execution)
}
fn find_generated(
    plan: &EffectivePlanComponents,
    provider: usize,
    grant: &super::EffectiveGrant,
    action: &str,
    effect: &str,
    work: &mut Work,
) -> Result<usize, RelicPlanError> {
    work.charge(plan.effects.len(), "effective_generated_identity")?;
    plan.effects.iter().position(|candidate| candidate.present && candidate.provider == provider &&
        matches!(&candidate.key, super::EffectiveEffectKey::Generated { relic_copy_uuid, grant_rule_id,
            anchor, local_action_id, local_effect_id } if *relic_copy_uuid == grant.contribution.copy_uuid &&
            grant_rule_id == &grant.contribution.rule_id && anchor == &grant.anchor && local_action_id == action && local_effect_id == effect))
        .ok_or_else(|| error("active generated effect has no finite identity"))
}
