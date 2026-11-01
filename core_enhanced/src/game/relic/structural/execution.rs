//! Private, generation-local lowering. Public DTOs never contain these owners.
//! Structural addresses remain lossless; short local aliases only index the
//! ordinary skill/Flow executor and never identify a relic copy by themselves.
use super::support::SelectedSupportReach;
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
    pub effects: Arc<Vec<(EffectSite, usize)>>,
    /// Selection is fixed at cast acceptance; this is authored data-unit reach,
    /// not a TileRangePattern or an increased shared protection radius.
    pub selected_recipient_reach: Option<SelectedSupportReach>,
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
fn merge_selected_recipient_reach(
    selected: &mut Option<SelectedSupportReach>,
    contribution: Option<SelectedSupportReach>,
) -> Result<(), RelicPlanError> {
    let Some(reach) = contribution else {
        return Ok(());
    };
    // One provider's original and generated selected consumers must agree.
    if selected.as_ref().is_some_and(|old| *old != reach) {
        return Err(error("selected support reach conflict"));
    }
    *selected = Some(reach);
    Ok(())
}
fn overlay(
    op: &mut SkillEffectDef,
    effect: &EffectiveEffect,
) -> Result<Option<SelectedSupportReach>, RelicPlanError> {
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
                authored_reach_units: _,
            } = definition.recipients
            {
                reach = Some(
                    super::support::selection_recipe(effect)
                        .unwrap_or_else(|| super::support::shield_reach(definition)),
                );
                if let Some(SelectedSupportReach::Pattern { range, .. }) = &reach {
                    definition.support = Some(super::support::SupportPolicyDef {
                        range: Some(Arc::clone(range)),
                        conversion: None,
                        conversions: Vec::new(),
                    });
                }
            }
        }
        SkillEffectDef::Sustained {
            definition: SustainedEffectDef::ChannelAura { aura, .. },
        } => {
            if let Some(policy) = effect.recipes.iter().find_map(|r| match r {
                StructuralRecipeDef::ChannelHealthUpkeep { policy } => Some(*policy),
                _ => None,
            }) {
                aura.health_upkeep = Some(policy);
            }
            if effect
                .recipes
                .iter()
                .any(|r| matches!(r, StructuralRecipeDef::FixedRecipientSupport))
            {
                reach = Some(
                    super::support::selection_recipe(effect)
                        .unwrap_or(SelectedSupportReach::Radius(aura.radius_units)),
                );
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
    if super::support::selection_recipe(effect).is_some() && reach.is_none() {
        return Err(error(
            "support selection pattern requires an executable selected support consumer",
        ));
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
            condition: rename_condition(condition, nodes, effects, producer_steps, work)?,
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
        compiler::admit_owned_copy(amount, work)?;
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
    compiler::admit_owned_copy(original, work)?;
    compiler::admit_owned_copy(&steps, work)?;
    compiler::admit_owned_copy(raw, work)?;
    let original_flow = data
        .skill_data
        .compiled_flow(original.id.as_str())
        .map_err(error)?;
    compiler::admit_owned_copy(original_flow.as_ref(), work)?;
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
    prepared: Option<(&[super::SharedEffect], &[EffectiveProviderPlan])>,
) -> Result<Vec<EffectiveProviderPlan>, RelicPlanError> {
    work.reserve(
        plan.providers
            .len()
            .checked_mul(std::mem::size_of::<EffectiveProviderPlan>()),
        "effective_provider_owner",
    )?;
    let mut execution = Vec::with_capacity(plan.providers.len());
    for (provider, binding) in plan.providers.iter().enumerate() {
        if let Some((effects, baseline)) = prepared {
            if unchanged_provider(plan, provider, effects, work)? {
                let cached = baseline
                    .get(provider)
                    .ok_or_else(|| error("prepared lowering provider missing"))?;
                super::storage::admit_shared_clone(cached, work)?;
                execution.push(cached.clone());
                continue;
            }
        }
        let original = data
            .skill_data
            .get_by_id(binding.skill_id())
            .ok_or_else(|| error("provider skill unavailable"))?;
        super::storage::admit_shared_clone(original, work)?;
        let mut skill = original.clone();
        super::action::apply(&mut skill, provider, &plan.action_modifiers, work)?;
        let original_flow = data
            .skill_data
            .compiled_flow(skill.id.as_str())
            .map_err(error)?;
        let has_finite_grant = plan.grants.iter().any(|grant| {
            grant.provider == provider
                && matches!(grant.body.as_ref(), GrantBodyDef::FiniteFlow { .. })
        });
        let mut used_steps = Vec::new();
        let mut used_nodes = Vec::new();
        if has_finite_grant {
            work.reserve(
                skill
                    .steps
                    .len()
                    .checked_add(original_flow.nodes.len())
                    .and_then(|n| n.checked_mul(std::mem::size_of::<String>())),
                "effective_alias_tables",
            )?;
            used_steps.reserve_exact(skill.steps.len());
            used_nodes.reserve_exact(original_flow.nodes.len());
            for step in &skill.steps {
                compiler::admit_owned_copy(&step.id, work)?;
                used_steps.push(step.id.clone());
            }
            for node in &original_flow.nodes {
                compiler::admit_owned_copy(&node.id, work)?;
                used_nodes.push(node.id.clone());
            }
        }
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
            merge_selected_recipient_reach(&mut selected_recipient_reach, reach)?;
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
                        compiler::admit_owned_copy(declared, work)?;
                        let id = aliases[&declared.id].clone();
                        let mut effect =
                            AuthoredSkillEffect::new(id, compiler::primitive(&declared.op));
                        rename_amount(&mut effect.op, &aliases, work)?;
                        effect.tags = effective.tags.clone();
                        let reach = overlay(&mut effect.op, effective)?;
                        merge_selected_recipient_reach(&mut selected_recipient_reach, reach)?;
                        bind_generated(&skill, &grant.anchor.action_id, &mut effect);
                        address.push((effect.source().unwrap().site.clone(), index));
                        skill.steps[anchor_step].effects.push(effect);
                    }
                }
                GrantBodyDef::FiniteFlow { steps, flow, .. } => {
                    compiler::admit_owned_copy(steps, work)?;
                    compiler::admit_owned_copy(flow, work)?;
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
                            let reach = overlay(&mut effect.op, effective)?;
                            merge_selected_recipient_reach(&mut selected_recipient_reach, reach)?;
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
        work.charge(
            skill
                .steps
                .iter()
                .try_fold(1usize, |n, s| n.checked_add(s.id.len() + 1))
                .ok_or_else(|| error("flow reuse comparison overflow"))?,
            "effective_flow_reuse_check",
        )?;
        let flow = if branches.is_empty()
            && original_flow.matches_source(skill.flow.as_ref(), &skill.steps)
        {
            Arc::clone(&original_flow)
        } else {
            compiler::admit_owned_copy(original_flow.as_ref(), work)?;
            Arc::new(
                (*original_flow)
                    .clone()
                    .attach_branches(&skill.steps, &branches)
                    .map_err(|e| error(format!("{}.{}: {}", skill.id, e.path, e.cause)))?,
            )
        };
        let single_recipe_sites: Vec<_> = address
            .iter()
            .filter(|(_, index)| {
                plan.effects[*index].recipes.iter().any(|recipe| {
                    matches!(recipe, StructuralRecipeDef::PersonalShieldSingle { .. })
                })
            })
            .map(|(site, _)| site.clone())
            .collect();
        // Equipped declarations have already been resolved into plan receipts.
        // Keep the executable flag, not declarations whose original group
        // target may now be a converted selected-recipient effect.
        if let Some(policy) = &skill.action_policy {
            let crate::game::ability::action::ActionPolicy {
                external_interrupt_protected,
                modifiers,
            } = policy.as_ref();
            if !modifiers.is_empty() {
                work.reserve(
                    Some(std::mem::size_of::<crate::game::ability::action::ActionPolicy>() + 64),
                    "effective_resolved_action_policy",
                )?;
                skill.action_policy = Some(Arc::new(crate::game::ability::action::ActionPolicy {
                    external_interrupt_protected: *external_interrupt_protected,
                    modifiers: Vec::new(),
                }));
            }
        }
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
                .checked_mul(std::mem::size_of::<(EffectSite, usize)>())
                .and_then(|n| n.checked_add(std::mem::size_of::<Vec<(EffectSite, usize)>>() + 64)),
            "effective_address_map",
        )?;
        execution.push(EffectiveProviderPlan {
            skill: Arc::new(database.skills.into_iter().next().unwrap()),
            flow,
            amounts: Arc::new(amounts),
            effects: Arc::new(address),
            selected_recipient_reach,
            selected_recipient_excludes_caster,
        });
    }
    Ok(execution)
}
// Compare the complete inputs consumed by this provider's lowering. Exact
// indices preserve effective effect addresses; Eq also covers non-serde body
// and amount capabilities. Other providers and all global rules were already
// resolved by the caller and are never omitted from validation.
fn unchanged_provider(
    plan: &EffectivePlanComponents,
    provider: usize,
    baseline: &[super::SharedEffect],
    work: &mut Work,
) -> Result<bool, RelicPlanError> {
    work.charge(
        plan.grants
            .len()
            .checked_add(plan.action_modifiers.len())
            .and_then(|n| n.checked_add(plan.effects.len()))
            .and_then(|n| n.checked_add(baseline.len()))
            .ok_or_else(|| error("prepared lowering comparison overflow"))?,
        "effective_prepared_scan",
    )?;
    if plan.grants.iter().any(|g| g.provider == provider)
        || plan.action_modifiers.iter().any(|m| m.provider == provider)
    {
        return Ok(false);
    }
    let mut found = 0usize;
    for (index, effect) in plan
        .effects
        .iter()
        .enumerate()
        .filter(|(_, e)| e.provider == provider)
    {
        let Some(original) = baseline.get(index).filter(|e| e.provider == provider) else {
            return Ok(false);
        };
        work.charge(1, "effective_shared_identity")?;
        if !Arc::ptr_eq(effect, original) {
            super::storage::admit_comparison(effect, work)?;
            if effect != original {
                return Ok(false);
            }
        }
        found += 1;
    }
    Ok(found == baseline.iter().filter(|e| e.provider == provider).count())
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

#[cfg(test)]
mod prepared_lowering_tests {
    use super::*;

    #[test]
    fn publication_prepared_lowering_reuses_only_exact_original_inputs_including_hidden_fields() {
        let (data, granted) = crate::game::battle::core::relic_structural_tests::prepared_plan();
        let mut plan = granted.components.clone();
        plan.grants.clear();
        plan.effects.retain(|e| e.original);
        plan.execution.clear();
        let baseline = plan.effects.clone();
        let mut work = Work::new(u64::MAX);
        let fresh = lower(&plan, data.database(), &mut work, None).unwrap();
        let mut work = Work::new(u64::MAX);
        let reused = lower(&plan, data.database(), &mut work, Some((&baseline, &fresh))).unwrap();
        assert_eq!(fresh.len(), reused.len());
        assert!(Arc::ptr_eq(&fresh[0].skill, &reused[0].skill));
        assert!(Arc::ptr_eq(&fresh[0].amounts, &reused[0].amounts));
        assert_eq!(fresh[0].effects, reused[0].effects);
        let unchanged = |p: &EffectivePlanComponents| {
            unchanged_provider(p, 0, &baseline, &mut Work::new(u64::MAX)).unwrap()
        };
        assert!(unchanged(&plan));
        let wire = serde_json::to_value(&plan.effects).unwrap();
        let mut hidden_body = plan.clone();
        let EffectBody::Original { effect, .. } =
            &mut Arc::make_mut(&mut hidden_body.effects[0]).body
        else {
            unreachable!()
        };
        *effect += 1;
        assert_eq!(serde_json::to_value(&hidden_body.effects).unwrap(), wire);
        assert!(!unchanged(&hidden_body));
        let mut hidden_capability = plan.clone();
        Arc::make_mut(&mut hidden_capability.effects[0])
            .amount_capabilities
            .deadline = !hidden_capability.effects[0].amount_capabilities.deadline;
        assert_eq!(
            serde_json::to_value(&hidden_capability.effects).unwrap(),
            wire
        );
        assert!(!unchanged(&hidden_capability));
        let mut removed = plan.clone();
        Arc::make_mut(&mut removed.effects[0]).present = false;
        assert!(!unchanged(&removed));
        let mut moved_address = plan.clone();
        let mut unrelated = moved_address.effects[0].clone();
        Arc::make_mut(&mut unrelated).provider = 1;
        moved_address.effects.insert(0, unrelated);
        assert!(!unchanged(&moved_address));
        let mut with_grant = plan.clone();
        with_grant.grants = granted.grants.clone();
        assert!(!unchanged(&with_grant));
        assert!(
            unchanged(&plan),
            "rejected candidates leave the baseline unchanged"
        );
    }
}

fn rename_condition(
    condition: &FlowCondition,
    nodes: &BTreeMap<String, String>,
    effects: &BTreeMap<(String, String), String>,
    producer_steps: &BTreeMap<String, String>,
    work: &mut Work,
) -> Result<FlowCondition, RelicPlanError> {
    work.charge(1, "effective_condition_lower")?;
    Ok(match condition {
        FlowCondition::Predicate(_) => return Err(error("unresolved authored predicate")),
        FlowCondition::Logic(logic) => {
            FlowCondition::Logic(logic.try_map(|_, child| {
                rename_condition(child, nodes, effects, producer_steps, work)
            })?)
        }
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
}
