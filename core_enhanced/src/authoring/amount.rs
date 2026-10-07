//! Borrowed static descriptions of generation-local compiled amount plans.

use serde::{
    ser::{Error as _, SerializeSeq},
    Serialize, Serializer,
};

use super::{
    control::{self, Resource},
    query::{control_error, request_error},
    AuthoringError,
};
use crate::game::{
    ability::{
        authoring::{AuthoredSkillEffect, EffectSite},
        SkillEffectDef,
    },
    value_expr::{
        can_evaluate_without_external_context, AmountPlanKind, AmountProvenance,
        CompiledAmountEntry, CompiledAmountPlan, CompiledOp, ContextRequirements, EvalMoment,
        PlanCost, ValueUnit, VALUE_EXPR_COMPILER_VERSION,
    },
};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum StaticEvaluationContext {
    ContextFree,
    NeedsRuntimeContext,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) struct StaticEvaluationExplanation {
    pub(super) context: StaticEvaluationContext,
    pub(super) performed: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AmountPublication {
    LegacySupported,
    AuthoredSupported,
}

#[derive(Debug, Serialize)]
pub(super) struct CompiledAmountExplanation<'a> {
    representation: &'static str,
    kind: AmountPlanKind,
    compiler_version: u32,
    provenance: AmountProvenance,
    moment: EvalMoment,
    result_unit: &'a ValueUnit,
    source: &'a crate::game::ability::authoring::EffectSource,
    semantic_fingerprint: String,
    requirements: &'a ContextRequirements,
    cost: PlanCost,
    segments: CompiledSegments<'a>,
    evaluation: StaticEvaluationExplanation,
    publication: AmountPublication,
}

#[derive(Debug, Clone, Copy)]
struct CompiledSegments<'a>(&'a CompiledAmountPlan);

#[derive(Serialize)]
struct CompiledSegment<'a> {
    index: usize,
    ops: &'a [CompiledOp],
}

impl Serialize for CompiledSegments<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.segment_count()))?;
        for index in 0..self.0.segment_count() {
            let ops = self
                .0
                .segment(index)
                .ok_or_else(|| S::Error::custom("compiled amount segment is missing"))?;
            sequence.serialize_element(&CompiledSegment { index, ops })?;
        }
        sequence.end()
    }
}

pub(super) fn charge_catalog_lookup(
    catalog_len: usize,
    site: &EffectSite,
) -> Result<(), AuthoringError> {
    let site_bytes = site
        .action
        .definition_id
        .as_str()
        .len()
        .checked_add(site.action.action_id.len())
        .and_then(|value| value.checked_add(site.effect_id.len()))
        .and_then(|value| value.checked_add(3))
        .ok_or_else(cost_overflow)?;
    let comparisons = if catalog_len == 0 {
        1
    } else {
        usize::BITS as usize - catalog_len.leading_zeros() as usize + 1
    };
    let work = site_bytes
        .checked_mul(comparisons)
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(cost_overflow)?;
    control::charge(work, "explain_amount_catalog_lookup").map_err(control_error)
}

pub(super) fn explain_effect<'a>(
    site: &EffectSite,
    effect: &AuthoredSkillEffect,
    entry: Option<&'a CompiledAmountEntry>,
) -> Result<Option<CompiledAmountExplanation<'a>>, AuthoringError> {
    let expected_kind = match &effect.op {
        SkillEffectDef::Damage { .. } | SkillEffectDef::DamageExpr { .. } => {
            Some(AmountPlanKind::Damage)
        }
        SkillEffectDef::Heal { .. } | SkillEffectDef::HealExpr { .. } => Some(AmountPlanKind::Heal),
        SkillEffectDef::ShieldExpr { .. } | SkillEffectDef::PersonalShieldGrant { .. } => {
            Some(AmountPlanKind::Shield)
        }
        SkillEffectDef::CreateObject { .. }
        | SkillEffectDef::ModifyDamage { .. }
        | SkillEffectDef::ModifyResonance { .. }
        | SkillEffectDef::ModifyTacticalResource { .. }
        | SkillEffectDef::Resources { .. }
        | SkillEffectDef::Sustained { .. }
        | SkillEffectDef::HealthCost { .. }
        | SkillEffectDef::Pull { .. }
        | SkillEffectDef::RechargeShield { .. }
        | SkillEffectDef::ModifyStats { .. }
        | SkillEffectDef::ApplyBuff { .. }
        | SkillEffectDef::InterruptCast
        | SkillEffectDef::ExtraAttack { .. } => None,
    };
    let Some(expected_kind) = expected_kind else {
        return if entry.is_none() {
            Ok(None)
        } else {
            Err(inconsistent(
                "A non-amount effect unexpectedly owns a compiled amount plan.",
            ))
        };
    };
    let entry = entry.ok_or_else(|| {
        request_error(
            "authoring_amount_plan_unavailable",
            "site",
            "This amount effect has no compiled amount plan in the published generation.",
        )
    })?;
    let plan = entry.plan().as_ref();
    if entry.kind() != expected_kind {
        return Err(inconsistent(
            "The compiled amount kind does not match the published effect kind.",
        ));
    }
    if &plan.source().site != site {
        return Err(inconsistent(
            "The compiled amount source site does not match the requested effect site.",
        ));
    }
    let effect_source = effect.source().ok_or_else(|| {
        inconsistent("The published amount effect has no reconstructed source attribution.")
    })?;
    if effect_source.as_ref() != plan.source() {
        return Err(inconsistent(
            "The compiled amount source does not match the published effect source.",
        ));
    }

    describe_plan(entry.kind(), entry.is_authored_expression(), plan).map(Some)
}

pub(super) fn describe_plan(
    kind: AmountPlanKind,
    is_authored_expression: bool,
    plan: &CompiledAmountPlan,
) -> Result<CompiledAmountExplanation<'_>, AuthoringError> {
    let mut op_count = 0usize;
    for index in 0..plan.segment_count() {
        let ops = plan.segment(index).ok_or_else(|| {
            inconsistent("The compiled amount plan has a missing segment in its index space.")
        })?;
        op_count = op_count.checked_add(ops.len()).ok_or_else(cost_overflow)?;
        control::charge_serialized(&ops, "explain_amount_segment_scan").map_err(control_error)?;
    }
    let structure_work = plan
        .segment_count()
        .checked_add(op_count)
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(cost_overflow)?;
    control::charge(structure_work, "explain_amount_plan_structure").map_err(control_error)?;
    control::charge_serialized(plan.requirements(), "explain_amount_requirements")
        .map_err(control_error)?;
    control::charge_serialized(plan.source(), "explain_amount_source").map_err(control_error)?;
    control::charge_serialized(&plan.cost(), "explain_amount_cost").map_err(control_error)?;

    // The fingerprint is the only newly owned plan payload in the descriptor.
    // Admit its exact fixed length before allocating the String.
    control::admit(Resource::MetadataBytes, 0, 64, "explain_amount_fingerprint")
        .and_then(|()| control::charge(64, "explain_amount_fingerprint"))
        .map_err(control_error)?;
    let semantic_fingerprint = plan.semantic_fingerprint_hex();
    debug_assert_eq!(semantic_fingerprint.len(), 64);

    let descriptor = CompiledAmountExplanation {
        representation: "compiled_amount_plan",
        kind,
        compiler_version: VALUE_EXPR_COMPILER_VERSION,
        provenance: plan.provenance(),
        moment: plan.moment(),
        result_unit: plan.result_unit(),
        source: plan.source(),
        semantic_fingerprint,
        requirements: plan.requirements(),
        cost: plan.cost(),
        segments: CompiledSegments(plan),
        evaluation: StaticEvaluationExplanation {
            context: if can_evaluate_without_external_context(plan) {
                StaticEvaluationContext::ContextFree
            } else {
                StaticEvaluationContext::NeedsRuntimeContext
            },
            performed: false,
        },
        publication: if is_authored_expression {
            AmountPublication::AuthoredSupported
        } else {
            AmountPublication::LegacySupported
        },
    };
    control::metadata_size(&descriptor).map_err(control_error)?;
    Ok(descriptor)
}

fn inconsistent(reason: &str) -> AuthoringError {
    request_error("authoring_amount_plan_inconsistent", "site", reason)
}

fn cost_overflow() -> AuthoringError {
    request_error(
        "authoring_amount_plan_cost_overflow",
        "site",
        "Compiled amount explanation cost overflowed.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authoring::{CancellationToken, ExecutionLimits, ExecutionPhase},
        game::{
            ability::{
                authoring::{ActionSite, EffectOrigin, EffectSource},
                SkillId,
            },
            resource::{ResourceId, ResourceOwnerSelector},
            resources::item_slot::EquipmentSlotAddress,
            value_expr::{
                compile, AmountExpr, AuthoredMember, DeliveryCapabilities, ExprLimits,
                FiniteSource, Grouping, HitPointField, Reduction, ResolvedOwnerRelation,
                ResolvedTargetPredicate, ResultField, ResultSelector, StatSelector, TimeValue,
                TypedInteger, ValueExpr, ValueRef, ValueSubject, ValueUnit,
            },
        },
    };

    fn source() -> EffectSource {
        EffectSource {
            source_file: "packs/staged/data/skills/amount.ron".into(),
            site: EffectSite {
                action: ActionSite {
                    definition_id: SkillId::from("static_explain"),
                    action_id: "strike".into(),
                },
                effect_id: "amount".into(),
            },
            field_path: "skills[0].steps[0].effects[0].op".into(),
            origin_definition_id: SkillId::from("static_explain"),
            origin: EffectOrigin::Original,
        }
    }

    fn plan(root: ValueExpr) -> CompiledAmountPlan {
        compile(
            &AmountExpr::authored(EvalMoment::Snapshot, root),
            &source(),
            DeliveryCapabilities::all(),
            &ExprLimits::default(),
        )
        .unwrap()
    }

    fn explain_json(plan: &CompiledAmountPlan) -> serde_json::Value {
        let (value, _) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::Inspection,
            || {
                serde_json::to_value(describe_plan(AmountPlanKind::Damage, true, plan).unwrap())
                    .unwrap()
            },
        )
        .unwrap();
        value
    }

    #[test]
    fn static_explanation_context_classification_uses_the_catalog_rule_without_evaluating() {
        let constant = plan(ValueExpr::Const(TypedInteger::new(
            123,
            ValueUnit::Magnitude,
        )));
        let constant_json = explain_json(&constant);
        assert_eq!(constant_json["evaluation"]["context"], "context_free");
        assert_eq!(constant_json["evaluation"]["performed"], false);

        let wide = plan(ValueExpr::Const(TypedInteger::new(
            i128::MAX,
            ValueUnit::Magnitude,
        )));
        let (encoded, _) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::Inspection,
            || {
                serde_json::to_string(&describe_plan(AmountPlanKind::Damage, true, &wide).unwrap())
                    .unwrap()
            },
        )
        .unwrap();
        assert!(encoded.contains(&i128::MAX.to_string()));

        let authored = plan(ValueExpr::Aggregate {
            source: FiniteSource::AuthoredMembers(vec![AuthoredMember::scalar(
                "member",
                TypedInteger::new(7, ValueUnit::Magnitude),
            )]),
            grouping: Grouping::WholeSet,
            reduction: Reduction::Sum,
            outer_reduction: None,
            member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
                unit: ValueUnit::Magnitude,
            })),
        });
        assert_eq!(
            explain_json(&authored)["evaluation"]["context"],
            "context_free"
        );

        let resource = ResourceId::new("explain_resource".into()).unwrap();
        let dynamic = [
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Target,
                stat: StatSelector::Attack,
            }),
            ValueExpr::Aggregate {
                source: FiniteSource::AuthoredMembers(vec![AuthoredMember::scalar(
                    "member",
                    TypedInteger::new(1, ValueUnit::Magnitude),
                )]),
                grouping: Grouping::WholeSet,
                reduction: Reduction::Sum,
                outer_reduction: None,
                member: Box::new(ValueExpr::Read(ValueRef::Stat {
                    subject: ValueSubject::AggregateMember,
                    stat: StatSelector::Attack,
                })),
            },
            ValueExpr::Aggregate {
                source: FiniteSource::ResolvedTargets,
                grouping: Grouping::WholeSet,
                reduction: Reduction::Sum,
                outer_reduction: None,
                member: Box::new(ValueExpr::Read(ValueRef::HitPoints {
                    subject: ValueSubject::AggregateMember,
                    field: HitPointField::Current,
                })),
            },
            ValueExpr::Read(ValueRef::Resource {
                resource,
                owner: ResourceOwnerSelector::SourceEmployee,
                field: crate::game::value_expr::ResourceValueField::Balance,
            }),
            ValueExpr::Read(ValueRef::Time(TimeValue::BattleElapsed)),
            ValueExpr::Read(ValueRef::Result {
                selector: ResultSelector::CurrentActionPrefix,
                field: ResultField::DamageHpLost,
            }),
        ];
        for expression in dynamic {
            let dynamic_plan = plan(expression);
            let value = explain_json(&dynamic_plan);
            assert_eq!(
                value["evaluation"]["context"], "needs_runtime_context",
                "{value}"
            );
            assert_eq!(value["evaluation"]["performed"], false);
        }
    }

    #[test]
    fn static_explanation_preserves_relations_filters_and_member_segment_indices() {
        let resolved = |relation, predicates| ValueExpr::Aggregate {
            source: FiniteSource::ResolvedTargetsWhere(predicates),
            grouping: Grouping::ByResolvedOwner(relation),
            reduction: Reduction::Sum,
            outer_reduction: Some(Reduction::Max),
            member: Box::new(ValueExpr::Read(ValueRef::HitPoints {
                subject: ValueSubject::AggregateMember,
                field: HitPointField::Current,
            })),
        };
        let nested = ValueExpr::Aggregate {
            source: FiniteSource::AuthoredMembers(vec![AuthoredMember::scalar(
                "outer",
                TypedInteger::new(1, ValueUnit::Magnitude),
            )]),
            grouping: Grouping::WholeSet,
            reduction: Reduction::Sum,
            outer_reduction: None,
            member: Box::new(ValueExpr::Aggregate {
                source: FiniteSource::AuthoredMembers(vec![AuthoredMember::scalar(
                    "inner",
                    TypedInteger::new(2, ValueUnit::Magnitude),
                )]),
                grouping: Grouping::WholeSet,
                reduction: Reduction::Sum,
                outer_reduction: None,
                member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
                    unit: ValueUnit::Magnitude,
                })),
            }),
        };
        let plan = plan(ValueExpr::Add(vec![
            resolved(
                ResolvedOwnerRelation::Side,
                vec![ResolvedTargetPredicate::IsActualEmployee],
            ),
            resolved(
                ResolvedOwnerRelation::ActualEmployee,
                vec![ResolvedTargetPredicate::HasEquippedSlot(
                    EquipmentSlotAddress::Weapon,
                )],
            ),
            resolved(
                ResolvedOwnerRelation::EquippedSlot(EquipmentSlotAddress::Armor),
                vec![
                    ResolvedTargetPredicate::IsActualEmployee,
                    ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Armor),
                ],
            ),
            nested,
        ]));
        let value = explain_json(&plan);
        assert_eq!(
            value["requirements"],
            serde_json::to_value(plan.requirements()).unwrap()
        );
        let segments = value["segments"].as_array().unwrap();
        assert_eq!(segments.len(), plan.segment_count());
        for (index, segment) in segments.iter().enumerate() {
            assert_eq!(segment["index"], index);
            assert_eq!(
                segment["ops"],
                serde_json::to_value(plan.segment(index).unwrap()).unwrap()
            );
        }
        let member_segments = segments
            .iter()
            .flat_map(|segment| segment["ops"].as_array().unwrap())
            .filter_map(|op| op.get("Aggregate"))
            .map(|aggregate| aggregate["member_segment"].as_u64().unwrap() as usize)
            .collect::<Vec<_>>();
        assert!(member_segments.len() >= 5);
        assert!(member_segments.iter().all(|index| *index < segments.len()));
        assert_eq!(value["evaluation"]["context"], "needs_runtime_context");
    }

    #[test]
    fn long_static_explanation_obeys_work_metadata_and_cancellation() {
        let members = (0..512)
            .map(|index| {
                AuthoredMember::scalar(
                    format!("member_{index:04}_{}", "x".repeat(96)),
                    TypedInteger::new(index, ValueUnit::Magnitude),
                )
            })
            .collect();
        let plan = plan(ValueExpr::Aggregate {
            source: FiniteSource::AuthoredMembers(members),
            grouping: Grouping::WholeSet,
            reduction: Reduction::Sum,
            outer_reduction: None,
            member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
                unit: ValueUnit::Magnitude,
            })),
        });
        let rejected = control::run(
            &ExecutionLimits {
                max_work: 128,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::Inspection,
            || describe_plan(AmountPlanKind::Damage, true, &plan),
        )
        .unwrap_err();
        assert_eq!(rejected.code, "authoring_work_limit");

        let rejected = control::run(
            &ExecutionLimits {
                max_metadata_bytes: 64,
                ..Default::default()
            },
            &CancellationToken::default(),
            ExecutionPhase::Inspection,
            || describe_plan(AmountPlanKind::Damage, true, &plan),
        )
        .unwrap_err();
        assert_eq!(rejected.code, "authoring_metadata_limit");

        let cancelled = CancellationToken::default();
        cancelled.cancel();
        let rejected = control::run(
            &ExecutionLimits::default(),
            &cancelled,
            ExecutionPhase::Inspection,
            || describe_plan(AmountPlanKind::Damage, true, &plan),
        )
        .unwrap_err();
        assert_eq!(rejected.code, "authoring_cancelled");
    }
}
