//! Generation-local immutable plans compiled from final winning skill effects.

use std::{collections::BTreeMap, sync::Arc};

use serde::Serialize;

use crate::game::{
    ability::{
        authoring::{EffectSite, EffectSource},
        DeliveryDef, ProjectileHitPolicy, SkillCastSelection, SkillEffectDef, SkillTarget,
    },
    data::{live_ron::LiveRonPhase, skill_data::SkillDatabase},
    resource::ResourceCatalog,
};

use super::{
    compile, evaluate, narrow_damage, narrow_heal, AmountExpr, AmountProvenance,
    CompiledAmountPlan, DeliveryCapabilities, EvalFrame, ExprDiagnostic, ExprLimits, ExprMeter,
    ResultSelector, TypedInteger, ValueExprFault, ValueExprFaultCode, ValueRef,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AmountPlanKind {
    Damage,
    Heal,
    Shield,
}

#[derive(Debug, Clone)]
pub struct CompiledAmountEntry {
    kind: AmountPlanKind,
    plan: Arc<CompiledAmountPlan>,
}

impl CompiledAmountEntry {
    pub fn kind(&self) -> AmountPlanKind {
        self.kind
    }

    pub fn plan(&self) -> &Arc<CompiledAmountPlan> {
        &self.plan
    }

    pub fn is_authored_expression(&self) -> bool {
        self.plan.provenance() == AmountProvenance::AuthoredExpression
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValueExprCatalog {
    entries: BTreeMap<EffectSite, CompiledAmountEntry>,
}

enum AmountInput<'a> {
    Authored(&'a AmountExpr),
    Legacy(AmountExpr),
}

impl AmountInput<'_> {
    fn as_expr(&self) -> &AmountExpr {
        match self {
            Self::Authored(expression) => expression,
            Self::Legacy(expression) => expression,
        }
    }
}

impl ValueExprCatalog {
    pub fn get(&self, site: &EffectSite) -> Option<&CompiledAmountEntry> {
        self.entries.get(site)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&EffectSite, &CompiledAmountEntry)> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains_authored_expressions(&self) -> bool {
        self.entries
            .values()
            .any(CompiledAmountEntry::is_authored_expression)
    }

    pub(crate) fn compile_skills(
        skills: &SkillDatabase,
        resources: &ResourceCatalog,
        limits: &ExprLimits,
    ) -> Result<Self, crate::game::data::live_ron::LiveRonError> {
        Self::compile_skills_with_deadline(skills, resources, limits, false, None)
    }
    pub(crate) fn compile_effective_skill(
        skills: &SkillDatabase,
        resources: &ResourceCatalog,
        limits: &ExprLimits,
        has_deadline: bool,
        budget: &mut crate::game::relic::structural::Work,
    ) -> Result<Self, crate::game::data::live_ron::LiveRonError> {
        Self::compile_skills_with_deadline(skills, resources, limits, has_deadline, Some(budget))
    }
    fn compile_skills_with_deadline(
        skills: &SkillDatabase,
        resources: &ResourceCatalog,
        limits: &ExprLimits,
        has_deadline: bool,
        mut budget: Option<&mut crate::game::relic::structural::Work>,
    ) -> Result<Self, crate::game::data::live_ron::LiveRonError> {
        let mut entries = BTreeMap::new();
        let reinput_skills: std::collections::BTreeSet<_> = skills
            .skills
            .iter()
            .flat_map(|s| &s.steps)
            .flat_map(|s| &s.effects)
            .filter_map(|e| {
                if let SkillEffectDef::Sustained {
                    definition:
                        crate::game::ability::sustained::SustainedEffectDef::BasicAttackOverlay {
                            goodbye_skill: Some(id),
                            ..
                        },
                } = &e.op
                {
                    Some(id.as_str())
                } else {
                    None
                }
            })
            .collect();
        for skill in &skills.skills {
            let flow = skills.compiled_flow(skill.id.as_str()).map_err(|cause| {
                crate::game::data::live_ron::LiveRonError::validation(
                    "data/skills/base.ron",
                    "skills",
                    Some(skill.id.to_string()),
                    Some("flow"),
                    cause,
                )
            })?;
            for (step_index, step) in skill.steps.iter().enumerate() {
                let prior_step_has_effect = flow.settled_effect_prefix[step_index];
                let mut capabilities =
                    delivery_capabilities(skill.cast_selection(), &step.target, &step.delivery);
                capabilities.deadline |= has_deadline || reinput_skills.contains(skill.id.as_str());
                for (effect_index, effect) in step.effects.iter().enumerate() {
                    let (kind, expression) = match &effect.op {
                        SkillEffectDef::Damage { amount, .. } => (
                            AmountPlanKind::Damage,
                            AmountInput::Legacy(AmountExpr::legacy_damage(*amount)),
                        ),
                        SkillEffectDef::Heal { amount } => (
                            AmountPlanKind::Heal,
                            AmountInput::Legacy(AmountExpr::legacy_heal(*amount)),
                        ),
                        SkillEffectDef::DamageExpr { amount, .. } => {
                            (AmountPlanKind::Damage, AmountInput::Authored(amount))
                        }
                        SkillEffectDef::HealExpr { amount } => {
                            (AmountPlanKind::Heal, AmountInput::Authored(amount))
                        }
                        SkillEffectDef::ShieldExpr { amount, .. } => {
                            (AmountPlanKind::Shield, AmountInput::Authored(amount))
                        }
                        SkillEffectDef::PersonalShieldGrant { definition } => (
                            AmountPlanKind::Shield,
                            AmountInput::Authored(&definition.amount),
                        ),
                        SkillEffectDef::ModifyDamage { .. }
                        | SkillEffectDef::ModifyResonance { .. }
                        | SkillEffectDef::ModifyTacticalResource { .. }
                        | SkillEffectDef::Resources { .. }
                        | SkillEffectDef::Sustained { .. }
                        | SkillEffectDef::CreateObject { .. }
                        | SkillEffectDef::HealthCost { .. }
                        | SkillEffectDef::Pull { .. }
                        | SkillEffectDef::RechargeShield { .. }
                        | SkillEffectDef::ModifyStats { .. }
                        | SkillEffectDef::ApplyBuff { .. }
                        | SkillEffectDef::InterruptCast
                        | SkillEffectDef::ExtraAttack { .. } => continue,
                    };
                    let source = effect.source().ok_or_else(|| {
                        crate::game::data::live_ron::LiveRonError::validation(
                            "data/skills/base.ron",
                            "skills",
                            Some(skill.id.to_string()),
                            Some(format!("action.{}.effects[{effect_index}]", step.id)),
                            "value expression compile requires a bound EffectSource",
                        )
                        .with_phase(LiveRonPhase::Compile)
                    })?;
                    let mut selected_limits = limits.clone();
                    if let Some(budget) = budget.as_ref() {
                        selected_limits.max_compile_work =
                            selected_limits.max_compile_work.min(budget.remaining());
                    }
                    let plan =
                        compile(expression.as_expr(), source, capabilities, &selected_limits)
                            .map_err(|error| compile_error(source, error))?;
                    if let Some(budget) = budget.as_mut() {
                        budget
                            .charge(
                                usize::try_from(plan.cost().compile_work)
                                    .map_err(|_| source_error(source, "amount work conversion"))?,
                                "effective_amount_compile",
                            )
                            .map_err(|error| source_error(source, error.to_string()))?;
                    }
                    validate_references(
                        source,
                        &plan,
                        resources,
                        &step.effects,
                        effect_index,
                        prior_step_has_effect,
                    )?;
                    validate_result_contract(source, kind, &plan, limits)?;
                    let site = source.site.clone();
                    if entries
                        .insert(
                            site.clone(),
                            CompiledAmountEntry {
                                kind,
                                plan: Arc::new(plan),
                            },
                        )
                        .is_some()
                    {
                        return Err(source_error(
                            source,
                            "duplicate EffectSite while constructing value expression catalog",
                        )
                        .with_code("value_expr_duplicate_site"));
                    }
                }
            }
        }
        Ok(Self { entries })
    }

    pub(crate) fn ensure_runtime_publishable(
        &self,
    ) -> Result<(), crate::game::data::live_ron::LiveRonError> {
        for entry in self
            .entries
            .values()
            .filter(|entry| entry.is_authored_expression())
        {
            use super::{CompiledOp, FiniteSource, Grouping, StatSelector, ValueSubject};
            let plan = entry.plan.as_ref();
            for segment in 0..plan.segment_count() {
                for op in plan.segment(segment).expect("compiled segment") {
                    let CompiledOp::Aggregate {
                        source,
                        grouping,
                        member_segment,
                        ..
                    } = op
                    else {
                        continue;
                    };
                    let resolved = !matches!(source, FiniteSource::AuthoredMembers(_));
                    let unsupported_owner = resolved && matches!(grouping, Grouping::ByOwner);
                    let unsupported_member = plan
                        .segment(*member_segment)
                        .expect("compiled member segment")
                        .iter()
                        .any(|op| match op {
                            CompiledOp::Read {
                                reference:
                                    ValueRef::Stat {
                                        subject: ValueSubject::AggregateMember,
                                        stat,
                                    },
                                ..
                            } => !resolved || !matches!(stat, StatSelector::Attack),
                            CompiledOp::Read {
                                reference:
                                    ValueRef::HitPoints {
                                        subject: ValueSubject::AggregateMember,
                                        ..
                                    },
                                ..
                            } => !resolved,
                            CompiledOp::Read {
                                reference: ValueRef::AggregateMember { .. },
                                ..
                            } => resolved,
                            _ => false,
                        });
                    if unsupported_owner || unsupported_member {
                        return Err(source_error(plan.source(),
                            "aggregate member/owner selector has no supported runtime input; use explicit resolved owner relations and supported member fields")
                            .with_code("content_capability_unsupported"));
                    }
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn delivery_capabilities(
    cast_selection: SkillCastSelection,
    target: &SkillTarget,
    delivery: &DeliveryDef,
) -> DeliveryCapabilities {
    let direct_target = matches!(
        target,
        SkillTarget::SelfUnit | SkillTarget::EnemySingle { .. } | SkillTarget::AllySingle
    ) || matches!(
        (cast_selection, target),
        (
            SkillCastSelection::AutomaticUnit | SkillCastSelection::PlayerUnit,
            SkillTarget::CastTarget
        )
    );
    let (target_at_release, target_at_consume, resolved_at_release, resolved_at_consume, deadline) =
        match delivery {
            DeliveryDef::Instant => (direct_target, direct_target, true, true, false),
            DeliveryDef::Projectile {
                hit_policy,
                max_lifetime_ms,
                ..
            } => match hit_policy {
                ProjectileHitPolicy::TargetLocked => (
                    direct_target,
                    direct_target,
                    direct_target,
                    direct_target,
                    max_lifetime_ms.is_some(),
                ),
                ProjectileHitPolicy::DirectionalCollision => {
                    (false, true, false, true, max_lifetime_ms.is_some())
                }
            },
            DeliveryDef::TileArea { area, .. } => (false, true, false, true, area.duration_ms > 0),
        };
    DeliveryCapabilities {
        snapshot: true,
        live: true,
        target_at_release,
        target_at_consume,
        resolved_targets_at_release: resolved_at_release,
        resolved_targets_at_consume: resolved_at_consume,
        resources: true,
        result_prefix: true,
        deadline,
        owner_grouping: true,
        projected_tile_grouping: true,
    }
}

fn validate_references(
    source: &EffectSource,
    plan: &CompiledAmountPlan,
    resources: &ResourceCatalog,
    effects: &[crate::game::ability::authoring::AuthoredSkillEffect],
    effect_index: usize,
    prior_step_has_effect: bool,
) -> Result<(), crate::game::data::live_ron::LiveRonError> {
    for reference in &plan.requirements().references {
        match reference {
            ValueRef::Resource {
                resource, owner, ..
            } => {
                let definition = resources
                    .resources
                    .iter()
                    .find(|definition| definition.id == *resource)
                    .ok_or_else(|| {
                        source_error(
                            source,
                            format!("unknown value expression resource '{}'", resource.as_str()),
                        )
                        .with_code("value_expr_missing_context")
                    })?;
                definition.validate_selector(*owner).map_err(|error| {
                    source_error(source, error.to_string()).with_code("value_expr_invalid_owner")
                })?;
            }
            ValueRef::Result { selector, .. } => {
                match selector {
                    ResultSelector::EarlierEffect(effect_id) => {
                        let prior = effects[..effect_index]
                            .iter()
                            .any(|effect| effect.id == *effect_id);
                        if !prior {
                            let is_current_or_future = effects[effect_index..]
                                .iter()
                                .any(|effect| effect.id == *effect_id);
                            let (code, cause) = if is_current_or_future {
                                (
                                "value_expr_future_result",
                                format!(
                                    "EarlierEffect('{effect_id}') must name a completed earlier effect"
                                ),
                            )
                            } else {
                                (
                                "value_expr_unavailable",
                                format!("EarlierEffect('{effect_id}') does not exist in this action"),
                            )
                            };
                            return Err(source_error(source, cause).with_code(code));
                        }
                    }
                    ResultSelector::CurrentActionPrefix
                        if effect_index == 0 && !prior_step_has_effect =>
                    {
                        return Err(source_error(
                            source,
                            "CurrentActionPrefix is not settled for the first effect",
                        )
                        .with_code("value_expr_not_settled"));
                    }
                    ResultSelector::CurrentActionPrefix => {}
                }
            }
            ValueRef::Stat { .. }
            | ValueRef::HitPoints { .. }
            | ValueRef::Time(_)
            | ValueRef::AggregateMember { .. } => {}
        }
    }
    Ok(())
}

fn validate_result_contract(
    source: &EffectSource,
    kind: AmountPlanKind,
    plan: &CompiledAmountPlan,
    limits: &ExprLimits,
) -> Result<(), crate::game::data::live_ron::LiveRonError> {
    let probe = TypedInteger::new(0, plan.result_unit().clone());
    match kind {
        AmountPlanKind::Damage | AmountPlanKind::Shield => {
            narrow_damage(&source.site, &probe).map(|_| ())
        }
        AmountPlanKind::Heal => narrow_heal(&source.site, &probe).map(|_| ()),
    }
    .map_err(|fault| fault_error(source, fault))?;

    if can_evaluate_without_external_context(plan) {
        let mut meter = ExprMeter::new(*limits)
            .map_err(|cause| source_error(source, cause).with_code("value_expr_budget_exceeded"))?;
        let evaluated = evaluate(plan, &EvalFrame::default(), &mut meter)
            .map_err(|fault| fault_error(source, fault))?;
        match kind {
            AmountPlanKind::Damage | AmountPlanKind::Shield => {
                narrow_damage(&source.site, &evaluated.value).map(|_| ())
            }
            AmountPlanKind::Heal => narrow_heal(&source.site, &evaluated.value).map(|_| ()),
        }
        .map_err(|fault| fault_error(source, fault))?;
    }
    Ok(())
}

pub(crate) fn can_evaluate_without_external_context(plan: &CompiledAmountPlan) -> bool {
    // The compiler rejects member reads outside an aggregate. With no resolved
    // target source, a bare AggregateMember read is therefore supplied by an
    // AuthoredMembers source. Member Stat/HitPoints reads remain distinct refs
    // and still require runtime member context.
    !plan.requirements().resolved_targets
        && plan
            .requirements()
            .references
            .iter()
            .all(|reference| matches!(reference, ValueRef::AggregateMember { .. }))
}

fn compile_error(
    source: &EffectSource,
    error: ExprDiagnostic,
) -> crate::game::data::live_ron::LiveRonError {
    let mut result = source_error(source, error.to_string()).with_code(fault_code(error.code));
    result.expected = error
        .expected_unit
        .as_ref()
        .and_then(|unit| serde_json::to_value(unit).ok());
    result.actual = error
        .actual_unit
        .as_ref()
        .and_then(|unit| serde_json::to_value(unit).ok());
    result
}

fn fault_error(
    source: &EffectSource,
    fault: ValueExprFault,
) -> crate::game::data::live_ron::LiveRonError {
    let mut result = source_error(source, fault.to_string()).with_code(fault_code(fault.code));
    result.expected = fault
        .expected_unit
        .as_ref()
        .and_then(|unit| serde_json::to_value(unit).ok());
    result.actual = fault
        .actual_unit
        .as_ref()
        .and_then(|unit| serde_json::to_value(unit).ok());
    result
}

fn source_error(
    source: &EffectSource,
    cause: impl Into<String>,
) -> crate::game::data::live_ron::LiveRonError {
    let mut error = crate::game::data::live_ron::LiveRonError::validation(
        &source.source_file,
        "skills",
        Some(source.site.action.definition_id.to_string()),
        Some(format!("{}.op", source.field_path)),
        cause,
    )
    .with_phase(LiveRonPhase::Compile);
    error.action_id = Some(source.site.action.action_id.clone());
    error.effect_id = Some(source.site.effect_id.clone());
    error
}

fn fault_code(code: ValueExprFaultCode) -> &'static str {
    match code {
        ValueExprFaultCode::UnitMismatch => "value_expr_unit_mismatch",
        ValueExprFaultCode::MissingRounding => "value_expr_missing_rounding",
        ValueExprFaultCode::DivideByZero => "value_expr_divide_by_zero",
        ValueExprFaultCode::Overflow => "value_expr_overflow",
        ValueExprFaultCode::InvalidClamp => "value_expr_invalid_clamp",
        ValueExprFaultCode::MissingContext => "value_expr_missing_context",
        ValueExprFaultCode::UnsupportedMoment => "value_expr_unsupported_moment",
        ValueExprFaultCode::InvalidOwner => "value_expr_invalid_owner",
        ValueExprFaultCode::StaleLease => "value_expr_stale_lease",
        ValueExprFaultCode::DifferentAttempt => "value_expr_different_attempt",
        ValueExprFaultCode::NotSettled => "value_expr_not_settled",
        ValueExprFaultCode::FutureResult => "value_expr_future_result",
        ValueExprFaultCode::Unavailable => "value_expr_unavailable",
        ValueExprFaultCode::BudgetExceeded => "value_expr_budget_exceeded",
        ValueExprFaultCode::Cancelled => "value_expr_cancelled",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        data::skill_data::SkillDatabase,
        resource::ResourceCatalog,
        value_expr::{AmountProvenance, EvalFrame, ExprMeter},
    };

    #[test]
    fn actual_finite_set_capability_follows_delivery_moment() {
        let instant = delivery_capabilities(
            SkillCastSelection::PlayerTile,
            &SkillTarget::CastTarget,
            &DeliveryDef::Instant,
        );
        assert!(!instant.target_at_release);
        assert!(instant.resolved_targets_at_release);
        assert!(instant.resolved_targets_at_consume);

        let directional = delivery_capabilities(
            SkillCastSelection::PlayerTile,
            &SkillTarget::CastTarget,
            &DeliveryDef::Projectile {
                speed_units_per_ms: 1,
                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                allow_targetless_cast: true,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
        );
        assert!(!directional.resolved_targets_at_release);
        assert!(directional.resolved_targets_at_consume);
        let seed = compile_input(&skill_ron(
            r#"(id: "seed", op: DamageExpr(amount: (
                moment: Snapshot, root: Const((value: 1, unit: Magnitude))),
                damage_type: True))"#,
        ))
        .unwrap();
        let source = seed.entries().next().unwrap().1.plan().source().clone();
        let expression = AmountExpr::authored(
            super::super::EvalMoment::Snapshot,
            super::super::ValueExpr::Aggregate {
                source: super::super::FiniteSource::ResolvedTargets,
                grouping: super::super::Grouping::WholeSet,
                reduction: super::super::Reduction::Sum,
                outer_reduction: None,
                member: Box::new(super::super::ValueExpr::Read(ValueRef::Stat {
                    subject: super::super::ValueSubject::AggregateMember,
                    stat: super::super::StatSelector::Attack,
                })),
            },
        );
        assert_eq!(
            compile(&expression, &source, directional, &ExprLimits::default())
                .unwrap_err()
                .code,
            ValueExprFaultCode::UnsupportedMoment,
        );
    }

    fn skill_ron(effects: &str) -> String {
        format!(
            r#"SkillDatabase(
                skills: [SkillDef(
                    id: "expr_skill", name: "Expression Skill", kind: Targeted,
                    focus_time_ms: 0,
                    use_lifetime: Action,
                    cast_targeting: Explicit(
                        selection: automatic_unit,
                        target: EnemySingle(rule: Nearest),
                        reach: global,
                        air_capable: false,
                    ),
                    steps: [SkillStepDef(
                        id: "strike", delay_ms: 0,
                        target: EnemySingle(rule: Nearest), delivery: Instant,
                        effects: [{effects}],
                    )],
                )],
            )"#
        )
    }

    fn compile_input(
        input: &str,
    ) -> Result<ValueExprCatalog, crate::game::data::live_ron::LiveRonError> {
        let database = SkillDatabase::from_ron("packs/custom/data/skills/expr.ron", input)?;
        ValueExprCatalog::compile_skills(
            &database,
            &ResourceCatalog { resources: vec![] },
            &ExprLimits::default(),
        )
    }

    fn value(op: &str) -> TypedInteger {
        let input = skill_ron(op);
        let catalog = compile_input(&input).expect("expression must parse and compile");
        let (_, entry) = catalog.entries().next().expect("compiled amount plan");
        let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
        evaluate(entry.plan(), &EvalFrame::default(), &mut meter)
            .unwrap()
            .value
    }

    #[test]
    fn actual_skill_ron_compiles_expression_and_preserves_literal_lowering() {
        let expression = r#"(id: "damage", op: DamageExpr(
            amount: (moment: Snapshot, root: Add([
                Const((value: 2, unit: Magnitude)),
                Const((value: 3, unit: Magnitude)),
            ])),
            damage_type: True,
        ))"#;
        let input = skill_ron(expression);
        let catalog = compile_input(&input).unwrap();
        let (_, entry) = catalog.entries().next().unwrap();
        assert_eq!(entry.kind(), AmountPlanKind::Damage);
        assert_eq!(
            entry.plan().provenance(),
            AmountProvenance::AuthoredExpression
        );
        assert_eq!(entry.plan().cost().nodes, 3);
        assert_eq!(
            value(expression),
            TypedInteger::new(5, super::super::ValueUnit::Magnitude)
        );
        assert_eq!(
            entry.plan().source().source_file,
            "packs/custom/data/skills/expr.ron"
        );
        assert_eq!(entry.plan().source().site.action.action_id, "strike");
        assert_eq!(entry.plan().source().site.effect_id, "damage");

        let literals = skill_ron(
            r#"(id: "damage", op: Damage(amount: 17, damage_type: True)),
                (id: "heal", op: Heal(amount: -9))"#,
        );
        let catalog = compile_input(&literals).unwrap();
        let provenances = catalog
            .entries()
            .map(|(_, entry)| entry.plan().provenance())
            .collect::<Vec<_>>();
        assert_eq!(
            provenances,
            vec![
                AmountProvenance::LegacyDamageLiteral,
                AmountProvenance::LegacyHealLiteral,
            ]
        );
        assert!(!catalog.contains_authored_expressions());
        catalog.ensure_runtime_publishable().unwrap();
    }

    #[test]
    fn compile_rejections_keep_definition_action_effect_and_field_source() {
        let unit_mismatch = skill_ron(
            r#"(id: "bad_unit", op: DamageExpr(
                amount: (moment: Snapshot, root: Add([
                    Const((value: 2, unit: Magnitude)),
                    Const((value: 3, unit: Milliseconds)),
                ])), damage_type: True,
            ))"#,
        );
        let error = compile_input(&unit_mismatch).unwrap_err();
        assert_eq!(error.code, "value_expr_unit_mismatch");
        assert_eq!(error.source_file, "packs/custom/data/skills/expr.ron");
        assert_eq!(error.object_id.as_deref(), Some("expr_skill"));
        assert_eq!(error.action_id.as_deref(), Some("strike"));
        assert_eq!(error.effect_id.as_deref(), Some("bad_unit"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("skills[0].steps[0].effects[0].op")
        );
        assert!(error.expected.is_some());
        assert!(error.actual.is_some());

        let unknown_resource = skill_ron(
            r#"(id: "bad_resource", op: HealExpr(
                amount: (moment: Live, root: Read(Resource(
                    resource: "missing", owner: SourceEmployee, field: Balance,
                ))),
            ))"#,
        );
        let error = compile_input(&unknown_resource).unwrap_err();
        assert_eq!(error.code, "value_expr_missing_context");
        assert_eq!(error.effect_id.as_deref(), Some("bad_resource"));
        assert!(error
            .cause
            .contains("unknown value expression resource 'missing'"));
    }

    #[test]
    fn parse_rejection_and_supported_publication_are_explicit() {
        let missing_rounding = skill_ron(
            r#"(id: "bad_rounding", op: DamageExpr(
                amount: (moment: Snapshot, root: Ratio(
                    numerator: Const((value: 5, unit: Magnitude)),
                    denominator: Const((value: 2, unit: Scalar)),
                )), damage_type: True,
            ))"#,
        );
        let error = SkillDatabase::from_ron("packs/custom/data/skills/expr.ron", &missing_rounding)
            .unwrap_err();
        assert_eq!(error.code, "value_expr_missing_rounding");
        assert_eq!(error.object_id.as_deref(), Some("expr_skill"));
        assert_eq!(error.action_id.as_deref(), Some("strike"));
        assert_eq!(error.effect_id.as_deref(), Some("bad_rounding"));

        let expression = skill_ron(
            r#"(id: "blocked", op: HealExpr(
                amount: (moment: Live, root: Const((value: 4, unit: Magnitude))),
            ))"#,
        );
        let catalog = compile_input(&expression).unwrap();
        catalog.ensure_runtime_publishable().unwrap();
    }

    #[test]
    fn authored_aggregate_static_results_are_evaluated_and_narrowed() {
        let damage = skill_ron(
            r#"(id: "negative_sum", op: DamageExpr(
                amount: (moment: Snapshot, root: Aggregate(
                    source: AuthoredMembers([
                        (identity: "a", value: (value: -1, unit: Magnitude), owner: None, projected_tile: None),
                    ]),
                    grouping: WholeSet, reduction: Sum, outer_reduction: None,
                    member: Read(AggregateMember(unit: Magnitude)),
                )), damage_type: True,
            ))"#,
        );
        let error = compile_input(&damage).unwrap_err();
        assert_eq!(error.code, "value_expr_overflow");
        assert_eq!(error.source_file, "packs/custom/data/skills/expr.ron");
        assert_eq!(error.object_id.as_deref(), Some("expr_skill"));
        assert_eq!(error.action_id.as_deref(), Some("strike"));
        assert_eq!(error.effect_id.as_deref(), Some("negative_sum"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("skills[0].steps[0].effects[0].op")
        );

        let heal = skill_ron(
            r#"(id: "wide_heal", op: HealExpr(
                amount: (moment: Snapshot, root: Aggregate(
                    source: AuthoredMembers([
                        (identity: "a", value: (value: 2147483648, unit: Magnitude), owner: None, projected_tile: None),
                    ]),
                    grouping: WholeSet, reduction: Sum, outer_reduction: None,
                    member: Read(AggregateMember(unit: Magnitude)),
                )),
            ))"#,
        );
        let error = compile_input(&heal).unwrap_err();
        assert_eq!(error.code, "value_expr_overflow");
        assert_eq!(error.effect_id.as_deref(), Some("wide_heal"));
    }

    #[test]
    fn normal_and_nested_authored_aggregates_remain_static_and_valid() {
        let simple = r#"(id: "static_sum", op: DamageExpr(
            amount: (moment: Snapshot, root: Aggregate(
                source: AuthoredMembers([
                    (identity: "a", value: (value: 2, unit: Magnitude), owner: None, projected_tile: None),
                    (identity: "b", value: (value: 3, unit: Magnitude), owner: None, projected_tile: None),
                ]),
                grouping: WholeSet, reduction: Sum, outer_reduction: None,
                member: Read(AggregateMember(unit: Magnitude)),
            )), damage_type: True,
        ))"#;
        assert_eq!(
            value(simple),
            TypedInteger::new(5, super::super::ValueUnit::Magnitude)
        );

        let nested = r#"(id: "nested_sum", op: HealExpr(
            amount: (moment: Snapshot, root: Aggregate(
                source: AuthoredMembers([
                    (identity: "outer", value: (value: 99, unit: Magnitude), owner: None, projected_tile: None),
                ]),
                grouping: WholeSet, reduction: Sum, outer_reduction: None,
                member: Aggregate(
                    source: AuthoredMembers([
                        (identity: "inner_a", value: (value: 4, unit: Magnitude), owner: None, projected_tile: None),
                        (identity: "inner_b", value: (value: 5, unit: Magnitude), owner: None, projected_tile: None),
                    ]),
                    grouping: WholeSet, reduction: Sum, outer_reduction: None,
                    member: Read(AggregateMember(unit: Magnitude)),
                ),
            )),
        ))"#;
        assert_eq!(
            value(nested),
            TypedInteger::new(9, super::super::ValueUnit::Magnitude)
        );
    }

    #[test]
    fn authored_aggregate_static_fault_is_reported_by_the_shared_evaluator() {
        let input = skill_ron(
            r#"(id: "divide_by_zero", op: DamageExpr(
                amount: (moment: Snapshot, root: Aggregate(
                    source: AuthoredMembers([
                        (identity: "a", value: (value: 0, unit: Scalar), owner: None, projected_tile: None),
                    ]),
                    grouping: WholeSet, reduction: Sum, outer_reduction: None,
                    member: Ratio(
                        numerator: Const((value: 7, unit: Magnitude)),
                        denominator: Read(AggregateMember(unit: Scalar)),
                        rounding: TowardZero,
                    ),
                )), damage_type: True,
            ))"#,
        );
        let error = compile_input(&input).unwrap_err();
        assert_eq!(error.code, "value_expr_divide_by_zero");
        assert_eq!(error.object_id.as_deref(), Some("expr_skill"));
        assert_eq!(error.action_id.as_deref(), Some("strike"));
        assert_eq!(error.effect_id.as_deref(), Some("divide_by_zero"));
    }

    #[test]
    fn expressions_that_need_external_state_are_not_statically_evaluated() {
        let source_stat = skill_ron(
            r#"(id: "source_stat", op: DamageExpr(
                amount: (moment: Snapshot, root: Read(Stat(subject: Source, stat: Attack))),
                damage_type: True,
            ))"#,
        );
        let catalog = compile_input(&source_stat).expect("source stat remains runtime-dynamic");
        catalog.ensure_runtime_publishable().unwrap();

        let member_stat = skill_ron(
            r#"(id: "member_stat", op: DamageExpr(
                amount: (moment: Snapshot, root: Aggregate(
                    source: AuthoredMembers([
                        (identity: "a", value: (value: 1, unit: Magnitude), owner: None, projected_tile: None),
                    ]),
                    grouping: WholeSet, reduction: Sum, outer_reduction: None,
                    member: Read(Stat(subject: AggregateMember, stat: Attack)),
                )), damage_type: True,
            ))"#,
        );
        let catalog =
            compile_input(&member_stat).expect("member stat needs external member context");
        assert_eq!(
            catalog.ensure_runtime_publishable().unwrap_err().code,
            "content_capability_unsupported"
        );

        let resolved_members = skill_ron(
            r#"(id: "resolved_members", op: DamageExpr(
                amount: (moment: Snapshot, root: Aggregate(
                    source: ResolvedTargets,
                    grouping: WholeSet, reduction: Sum, outer_reduction: None,
                    member: Read(AggregateMember(unit: Magnitude)),
                )), damage_type: True,
            ))"#,
        );
        let catalog =
            compile_input(&resolved_members).expect("resolved targets need a runtime frame");
        assert_eq!(
            catalog.ensure_runtime_publishable().unwrap_err().code,
            "content_capability_unsupported"
        );
    }
}
