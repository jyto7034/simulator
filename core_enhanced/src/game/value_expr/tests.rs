use crate::game::ability::{
    authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource},
    SkillId,
};
use crate::game::resource::{ResourceId, ResourceOwnerSelector};
use crate::game::resources::item_slot::EquipmentSlotAddress;

use super::*;

fn source() -> EffectSource {
    EffectSource {
        source_file: "fixture/value_expr.ron".to_string(),
        site: EffectSite {
            action: ActionSite {
                definition_id: SkillId::new("value_expr_fixture"),
                action_id: "strike".to_string(),
            },
            effect_id: "damage".to_string(),
        },
        field_path: "steps[0].effects[0].op".to_string(),
        origin_definition_id: SkillId::new("value_expr_fixture"),
        origin: EffectOrigin::Original,
    }
}

fn r06_scoped_aggregate(values: Vec<TypedInteger>, member: ValueExpr) -> ValueExpr {
    ValueExpr::Aggregate {
        source: FiniteSource::AuthoredMembers(
            values
                .into_iter()
                .enumerate()
                .map(|(index, value)| AuthoredMember::scalar(format!("m{index}"), value))
                .collect(),
        ),
        grouping: Grouping::WholeSet,
        reduction: Reduction::Sum,
        outer_reduction: None,
        member: Box::new(member),
    }
}

fn r06_member(unit: ValueUnit) -> ValueExpr {
    ValueExpr::Read(ValueRef::AggregateMember { unit })
}

#[test]
fn r06_authored_member_direct_read_scope_and_resource_units_are_exact() {
    let mana = ValueUnit::Resource(ResourceId::new("mana".into()).unwrap());
    let rage = ValueUnit::Resource(ResourceId::new("rage".into()).unwrap());
    let mixed = vec![
        TypedInteger::new(1, ValueUnit::Magnitude),
        TypedInteger::new(2, ValueUnit::Milliseconds),
    ];
    let scalar_ratio = r06_scoped_aggregate(
        vec![TypedInteger::new(2, ValueUnit::Scalar)],
        ValueExpr::Ratio {
            numerator: Box::new(typed(8, ValueUnit::Magnitude)),
            denominator: Box::new(r06_member(ValueUnit::Scalar)),
            rounding: Rounding::TowardZero,
        },
    );
    let inner = r06_scoped_aggregate(
        vec![TypedInteger::new(3, ValueUnit::Magnitude)],
        r06_member(ValueUnit::Magnitude),
    );
    let cases = [
        (
            "resource_same",
            r06_scoped_aggregate(
                vec![TypedInteger::new(3, mana.clone())],
                r06_member(mana.clone()),
            ),
            TypedInteger::new(3, mana.clone()),
        ),
        (
            "unused_mixed_const",
            r06_scoped_aggregate(mixed.clone(), typed(7, ValueUnit::Magnitude)),
            TypedInteger::new(14, ValueUnit::Magnitude),
        ),
        (
            "scalar_read_magnitude_result",
            scalar_ratio.clone(),
            TypedInteger::new(4, ValueUnit::Magnitude),
        ),
        (
            "nested_parent_unused",
            r06_scoped_aggregate(mixed, scalar_ratio),
            TypedInteger::new(8, ValueUnit::Magnitude),
        ),
        (
            "nested_both_direct",
            r06_scoped_aggregate(
                vec![TypedInteger::new(2, ValueUnit::Scalar)],
                ValueExpr::Mul {
                    value: Box::new(inner),
                    scalar: Box::new(r06_member(ValueUnit::Scalar)),
                },
            ),
            TypedInteger::new(6, ValueUnit::Magnitude),
        ),
    ];
    for (name, expression, expected) in cases {
        assert_eq!(evaluated(expression).unwrap().value, expected, "{name}");
        eprintln!("R06_SCOPE_VALID {name} {expected:?}");
    }

    let negative = [
        (
            "resource_other_id",
            r06_scoped_aggregate(
                vec![TypedInteger::new(3, rage.clone())],
                r06_member(mana.clone()),
            ),
            mana,
            rage,
        ),
        (
            "incompatible_direct_reads",
            r06_scoped_aggregate(
                vec![TypedInteger::new(2, ValueUnit::Magnitude)],
                ValueExpr::Mul {
                    value: Box::new(r06_member(ValueUnit::Magnitude)),
                    scalar: Box::new(r06_member(ValueUnit::Scalar)),
                },
            ),
            ValueUnit::Scalar,
            ValueUnit::Magnitude,
        ),
        (
            "nested_child_mismatch_with_external_sibling",
            ValueExpr::Add(vec![
                ValueExpr::Read(ValueRef::Stat {
                    subject: ValueSubject::Source,
                    stat: StatSelector::Attack,
                }),
                r06_scoped_aggregate(
                    vec![TypedInteger::new(2, ValueUnit::Scalar)],
                    r06_scoped_aggregate(
                        vec![TypedInteger::new(3, ValueUnit::Milliseconds)],
                        r06_member(ValueUnit::Magnitude),
                    ),
                ),
            ]),
            ValueUnit::Magnitude,
            ValueUnit::Milliseconds,
        ),
    ];
    for (name, root, expected_unit, actual_unit) in negative {
        let error = compile(
            &AmountExpr::authored(EvalMoment::Live, root),
            &source(),
            DeliveryCapabilities::all(),
            &ExprLimits::default(),
        )
        .unwrap_err();
        assert_eq!(error.code, ValueExprFaultCode::UnitMismatch, "{name}");
        assert_eq!(error.phase, ValueExprPhase::Compile);
        assert_eq!(error.site, source().site);
        assert_eq!(error.expected_unit, Some(expected_unit));
        assert_eq!(error.actual_unit, Some(actual_unit));
        eprintln!("R06_SCOPE_REFUSAL {name} {error:?}");
    }

    // No payload exists to violate either direct read: this remains Sum's zero.
    let empty = r06_scoped_aggregate(
        vec![],
        ValueExpr::Mul {
            value: Box::new(r06_member(ValueUnit::Magnitude)),
            scalar: Box::new(r06_member(ValueUnit::Scalar)),
        },
    );
    assert_eq!(
        evaluated(empty).unwrap().value,
        TypedInteger::new(0, ValueUnit::Magnitude)
    );
}

#[test]
fn r06_authored_member_validation_compile_work_boundary_is_exact() {
    use std::mem::{align_of, size_of};
    let resource = ValueUnit::Resource(ResourceId::new("r".repeat(65)).unwrap());
    let expression = AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Aggregate {
            source: FiniteSource::AuthoredMembers(vec![
                AuthoredMember::scalar("a", TypedInteger::new(1, resource.clone())),
                AuthoredMember::scalar("b", TypedInteger::new(2, resource.clone())),
            ]),
            grouping: Grouping::WholeSet,
            reduction: Reduction::Sum,
            outer_reduction: None,
            member: Box::new(r06_member(resource.clone())),
        },
    );
    // Independent source-accounting arithmetic, fixed before observing PlanCost.
    // CompileValue stores ValueUnit, Option<i128>, u64, padded to their maximum alignment.
    let value_alignment = align_of::<ValueUnit>()
        .max(align_of::<Option<i128>>())
        .max(align_of::<u64>());
    let value_storage = (size_of::<ValueUnit>() + size_of::<Option<i128>>() + size_of::<u64>())
        .div_ceil(value_alignment)
        * value_alignment;
    let retained = 2 * (size_of::<EffectSource>() + 92)
        + 6 * size_of::<Vec<CompiledOp>>()
        + 8 * (size_of::<CompiledOp>() + value_storage)
        + 4 * size_of::<AuthoredMember>()
        + 264 // two copies of two (one-byte identity + 65-byte ResourceId) payloads
        + 260 // four copies of the 65-byte read payload
        + 16 * (size_of::<ValueRef>() + size_of::<usize>());
    // Source=ceil92/32=3, nodes=2, authored preprocessing=2+2*(1+3)+2=12,
    // reference=ceil65/32=3, new scan=1+2+2*ceil130/32=13; total prefix=33.
    let expected_work = 33 + retained.div_ceil(32) as u64;
    let limits = ExprLimits {
        max_compile_work: expected_work,
        ..Default::default()
    };
    let accepted = compile(&expression, &source(), DeliveryCapabilities::all(), &limits).unwrap();
    assert_eq!(accepted.cost().compile_work, expected_work);
    // Runtime remains: preprocessing12 + two reads + two reductions + root1 =17.
    assert_eq!(accepted.cost().static_work, 17);
    assert_eq!(accepted.cost().worst_case_work, 17);
    let mut meter = ExprMeter::new(limits).unwrap();
    assert_eq!(
        evaluate(&accepted, &EvalFrame::default(), &mut meter)
            .unwrap()
            .value,
        TypedInteger::new(3, resource)
    );
    let refused = compile(
        &expression,
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits {
            max_compile_work: expected_work - 1,
            ..limits
        },
    )
    .unwrap_err();
    assert_eq!(refused.code, ValueExprFaultCode::BudgetExceeded);
    assert_eq!(refused.phase, ValueExprPhase::Compile);
    assert_eq!(refused.site, source().site);
    eprintln!("R06_COMPILE_BOUNDARY retained={retained} compile_value_storage={value_storage} work={expected_work} below={} runtime_work=17", expected_work - 1);
}

#[test]
fn bounded_frame_keeps_sorted_logarithmic_reads_duplicates_and_capacity() {
    let attack = ValueRef::Stat {
        subject: ValueSubject::Source,
        stat: StatSelector::Attack,
    };
    let resource_id = ResourceId::new(format!("resource_{}", "x".repeat(100))).unwrap();
    let resource = ValueRef::Resource {
        resource: resource_id.clone(),
        owner: ResourceOwnerSelector::SourceEmployee,
        field: ResourceValueField::Balance,
    };
    let time = ValueRef::Time(TimeValue::BattleElapsed);
    let mut frame = EvalFrame::with_value_capacity(2);
    frame
        .insert_bounded(
            resource.clone(),
            TypedInteger::new(7, ValueUnit::Resource(resource_id)),
            2,
        )
        .unwrap();
    frame
        .insert_bounded(
            attack.clone(),
            TypedInteger::new(10, ValueUnit::Magnitude),
            2,
        )
        .unwrap();
    frame
        .insert_bounded(
            attack.clone(),
            TypedInteger::new(12, ValueUnit::Magnitude),
            2,
        )
        .unwrap();
    assert_eq!(frame.values.len(), 2);
    assert!(frame.values[0].0 < frame.values[1].0);
    for _ in 0..128 {
        assert_eq!(frame.get(&attack).unwrap().value, 12);
        assert_eq!(frame.get(&resource).unwrap().value, 7);
    }
    assert_eq!(
        frame.insert_bounded(
            time.clone(),
            TypedInteger::new(1, ValueUnit::Milliseconds),
            2
        ),
        Err("frame value capacity exhausted")
    );
    assert!(!frame.contains_key(&time));
}

fn typed(value: i128, unit: ValueUnit) -> ValueExpr {
    ValueExpr::Const(TypedInteger::new(value, unit))
}

fn compiled(root: ValueExpr) -> CompiledAmountPlan {
    compile(
        &AmountExpr::authored(EvalMoment::Live, root),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap()
}

fn evaluated(root: ValueExpr) -> Result<EvaluatedAmount, ValueExprFault> {
    let plan = compiled(root);
    let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
    evaluate(&plan, &EvalFrame::default(), &mut meter)
}

fn owner(identity: &str) -> GroupOwner {
    GroupOwner::new(GroupOwnerDomain::Employee, identity).unwrap()
}

fn ratio(numerator: i128, denominator: i128, rounding: Rounding) -> i128 {
    evaluated(ValueExpr::Ratio {
        numerator: Box::new(typed(numerator, ValueUnit::Scalar)),
        denominator: Box::new(typed(denominator, ValueUnit::Scalar)),
        rounding,
    })
    .unwrap()
    .value
    .value
}

#[test]
fn typed_add_and_ratio_reject_cross_unit_inputs() {
    let mismatch = compile(
        &AmountExpr::authored(
            EvalMoment::Live,
            ValueExpr::Add(vec![
                typed(1, ValueUnit::Milliseconds),
                typed(2, ValueUnit::Magnitude),
            ]),
        ),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(mismatch.code, ValueExprFaultCode::UnitMismatch);
    assert_eq!(mismatch.site, source().site);

    let a = ResourceId::new("resource_a".to_string()).unwrap();
    let b = ResourceId::new("resource_b".to_string()).unwrap();
    let mismatch = compile(
        &AmountExpr::authored(
            EvalMoment::Live,
            ValueExpr::Add(vec![
                typed(1, ValueUnit::Resource(a)),
                typed(2, ValueUnit::Resource(b)),
            ]),
        ),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(mismatch.code, ValueExprFaultCode::UnitMismatch);

    let result = evaluated(ValueExpr::Add(vec![
        typed(2, ValueUnit::Magnitude),
        typed(3, ValueUnit::Magnitude),
    ]))
    .unwrap();
    assert_eq!(result.value, TypedInteger::new(5, ValueUnit::Magnitude));
}

#[test]
fn integer_rounding_matches_the_signed_oracle() {
    assert_eq!(ratio(5, 2, Rounding::TowardZero), 2);
    assert_eq!(ratio(5, 2, Rounding::Floor), 2);
    assert_eq!(ratio(5, 2, Rounding::Ceil), 3);
    assert_eq!(ratio(5, 2, Rounding::NearestTiesEven), 2);

    assert_eq!(ratio(-5, 2, Rounding::TowardZero), -2);
    assert_eq!(ratio(-5, 2, Rounding::Floor), -3);
    assert_eq!(ratio(-5, 2, Rounding::Ceil), -2);
    assert_eq!(ratio(-5, 2, Rounding::NearestTiesEven), -2);
    assert_eq!(ratio(7, 2, Rounding::NearestTiesEven), 4);
}

#[test]
fn checked_compile_and_evaluation_report_exact_faults() {
    let overflow = compile(
        &AmountExpr::authored(
            EvalMoment::Live,
            ValueExpr::Add(vec![
                typed(i128::MAX, ValueUnit::Magnitude),
                typed(1, ValueUnit::Magnitude),
            ]),
        ),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(overflow.code, ValueExprFaultCode::Overflow);
    assert_eq!(overflow.node_index, 0);

    let divide_zero = compile(
        &AmountExpr::authored(
            EvalMoment::Live,
            ValueExpr::Ratio {
                numerator: Box::new(typed(1, ValueUnit::Scalar)),
                denominator: Box::new(typed(0, ValueUnit::Scalar)),
                rounding: Rounding::TowardZero,
            },
        ),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(divide_zero.code, ValueExprFaultCode::DivideByZero);

    let invalid_clamp = compile(
        &AmountExpr::authored(
            EvalMoment::Live,
            ValueExpr::Clamp {
                value: Box::new(typed(3, ValueUnit::Magnitude)),
                min: Box::new(typed(9, ValueUnit::Magnitude)),
                max: Box::new(typed(4, ValueUnit::Magnitude)),
            },
        ),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(invalid_clamp.code, ValueExprFaultCode::InvalidClamp);

    assert_eq!(
        evaluated(ValueExpr::Clamp {
            value: Box::new(typed(12, ValueUnit::Magnitude)),
            min: Box::new(typed(2, ValueUnit::Magnitude)),
            max: Box::new(typed(8, ValueUnit::Magnitude)),
        })
        .unwrap()
        .value,
        TypedInteger::new(8, ValueUnit::Magnitude)
    );
}

#[test]
fn dynamic_reads_use_typed_frames_and_report_missing_context_and_overflow() {
    let attack = ValueRef::Stat {
        subject: ValueSubject::Source,
        stat: StatSelector::Attack,
    };
    let plan = compiled(ValueExpr::Add(vec![
        ValueExpr::Read(attack.clone()),
        typed(1, ValueUnit::Magnitude),
    ]));

    let mut frame = EvalFrame::default();
    assert_eq!(
        frame.insert(
            attack.clone(),
            TypedInteger::new(3, ValueUnit::Milliseconds)
        ),
        Err("frame value unit does not match its reference")
    );
    frame
        .insert(attack.clone(), TypedInteger::new(41, ValueUnit::Magnitude))
        .unwrap();
    let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
    assert_eq!(
        evaluate(&plan, &frame, &mut meter).unwrap().value,
        TypedInteger::new(42, ValueUnit::Magnitude)
    );

    let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
    let missing = evaluate(&plan, &EvalFrame::default(), &mut meter).unwrap_err();
    assert_eq!(missing.code, ValueExprFaultCode::MissingContext);

    frame
        .insert(attack, TypedInteger::new(i128::MAX, ValueUnit::Magnitude))
        .unwrap();
    let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
    let overflow = evaluate(&plan, &frame, &mut meter).unwrap_err();
    assert_eq!(overflow.code, ValueExprFaultCode::Overflow);
    assert_eq!(overflow.phase, ValueExprPhase::Evaluate);
}

fn aggregate(
    members: Vec<AuthoredMember>,
    grouping: Grouping,
    reduction: Reduction,
    outer_reduction: Option<Reduction>,
) -> ValueExpr {
    ValueExpr::Aggregate {
        source: FiniteSource::AuthoredMembers(members),
        grouping,
        reduction,
        outer_reduction,
        member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        })),
    }
}

fn members() -> Vec<AuthoredMember> {
    vec![
        AuthoredMember {
            identity: "a".to_string(),
            value: TypedInteger::new(5, ValueUnit::Magnitude),
            owner: Some(owner("left")),
            projected_tile: Some(ProjectedTile { x: 0, y: 0 }),
        },
        AuthoredMember {
            identity: "b".to_string(),
            value: TypedInteger::new(3, ValueUnit::Magnitude),
            owner: Some(owner("left")),
            projected_tile: Some(ProjectedTile { x: 0, y: 0 }),
        },
        AuthoredMember {
            identity: "c".to_string(),
            value: TypedInteger::new(2, ValueUnit::Magnitude),
            owner: Some(owner("right")),
            projected_tile: Some(ProjectedTile { x: 1, y: 0 }),
        },
    ]
}

#[test]
fn finite_aggregate_has_explicit_empty_and_grouped_semantics() {
    for (reduction, expected) in [(Reduction::Sum, 10), (Reduction::Max, 5)] {
        assert_eq!(
            evaluated(aggregate(members(), Grouping::WholeSet, reduction, None))
                .unwrap()
                .value,
            TypedInteger::new(expected, ValueUnit::Magnitude)
        );
    }
    assert_eq!(
        evaluated(aggregate(
            members(),
            Grouping::WholeSet,
            Reduction::Count,
            None
        ))
        .unwrap()
        .value,
        TypedInteger::new(3, ValueUnit::Count)
    );
    assert_eq!(
        evaluated(aggregate(
            members(),
            Grouping::ByProjectedTile,
            Reduction::Max,
            Some(Reduction::Sum)
        ))
        .unwrap()
        .value,
        TypedInteger::new(7, ValueUnit::Magnitude)
    );
    assert_eq!(
        evaluated(aggregate(
            members(),
            Grouping::ByProjectedTile,
            Reduction::Count,
            Some(Reduction::Max)
        ))
        .unwrap()
        .value,
        TypedInteger::new(2, ValueUnit::Count)
    );

    for (reduction, unit) in [
        (Reduction::Count, ValueUnit::Count),
        (Reduction::Sum, ValueUnit::Magnitude),
    ] {
        assert_eq!(
            evaluated(aggregate(Vec::new(), Grouping::WholeSet, reduction, None))
                .unwrap()
                .value,
            TypedInteger::new(0, unit)
        );
    }
    let empty_max = evaluated(aggregate(
        Vec::new(),
        Grouping::WholeSet,
        Reduction::Max,
        None,
    ))
    .unwrap_err();
    assert_eq!(empty_max.code, ValueExprFaultCode::Unavailable);
}

#[test]
fn member_identity_order_is_canonical_for_fingerprint_and_trace() {
    let ordered = compiled(aggregate(
        members(),
        Grouping::WholeSet,
        Reduction::Sum,
        None,
    ));
    let mut reverse = members();
    reverse.reverse();
    let reversed = compiled(aggregate(reverse, Grouping::WholeSet, Reduction::Sum, None));
    assert_eq!(
        ordered.semantic_fingerprint(),
        reversed.semantic_fingerprint()
    );

    let mut left_meter = ExprMeter::new(ExprLimits::default()).unwrap();
    let mut right_meter = ExprMeter::new(ExprLimits::default()).unwrap();
    let left = evaluate(&ordered, &EvalFrame::default(), &mut left_meter).unwrap();
    let right = evaluate(&reversed, &EvalFrame::default(), &mut right_meter).unwrap();
    assert_eq!(left, right);
}

#[test]
fn compile_and_runtime_budgets_reject_before_partial_results() {
    let dynamic = ValueExpr::Add(vec![
        ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        }),
        typed(1, ValueUnit::Magnitude),
    ]);
    let aggregate = ValueExpr::Aggregate {
        source: FiniteSource::AuthoredMembers(members()),
        grouping: Grouping::WholeSet,
        reduction: Reduction::Sum,
        outer_reduction: None,
        member: Box::new(dynamic),
    };

    let mut node_limits = ExprLimits::default();
    node_limits.max_nodes = 3;
    let error = compile(
        &AmountExpr::authored(EvalMoment::Live, aggregate.clone()),
        &source(),
        DeliveryCapabilities::all(),
        &node_limits,
    )
    .unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::BudgetExceeded);

    let mut plan_limits = ExprLimits::default();
    plan_limits.max_work = 32;
    let plan = compile(
        &AmountExpr::authored(EvalMoment::Live, aggregate),
        &source(),
        DeliveryCapabilities::all(),
        &plan_limits,
    )
    .unwrap();

    let mut meter_limits = plan_limits;
    meter_limits.max_work = 6;
    let mut meter = ExprMeter::new(meter_limits).unwrap();
    let error = evaluate(&plan, &EvalFrame::default(), &mut meter).unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::BudgetExceeded);

    let mut cancelled = ExprMeter::new(plan_limits).unwrap();
    cancelled.cancel();
    let error = evaluate(&plan, &EvalFrame::default(), &mut cancelled).unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::Cancelled);
}

#[test]
fn compile_limits_accept_the_boundary_and_reject_one_past_it() {
    let three_nodes = ValueExpr::Add(vec![
        typed(1, ValueUnit::Magnitude),
        typed(2, ValueUnit::Magnitude),
    ]);
    for (limit, accepted) in [(2, false), (3, true), (4, true)] {
        let mut limits = ExprLimits::default();
        limits.max_nodes = limit;
        assert_eq!(
            compile(
                &AmountExpr::authored(EvalMoment::Live, three_nodes.clone()),
                &source(),
                DeliveryCapabilities::all(),
                &limits,
            )
            .is_ok(),
            accepted,
            "node limit {limit}"
        );
    }

    for (limit, accepted) in [(1, false), (2, true), (3, true)] {
        let mut limits = ExprLimits::default();
        limits.max_depth = limit;
        assert_eq!(
            compile(
                &AmountExpr::authored(EvalMoment::Live, three_nodes.clone()),
                &source(),
                DeliveryCapabilities::all(),
                &limits,
            )
            .is_ok(),
            accepted,
            "depth limit {limit}"
        );
    }

    let dynamic_add = ValueExpr::Add(vec![
        ValueExpr::Read(ValueRef::Stat {
            subject: ValueSubject::Source,
            stat: StatSelector::Attack,
        }),
        ValueExpr::Read(ValueRef::Stat {
            subject: ValueSubject::Source,
            stat: StatSelector::SpellPower,
        }),
    ]);
    for (limit, accepted) in [(1, false), (2, true), (3, true)] {
        let mut limits = ExprLimits::default();
        limits.max_stack = limit;
        assert_eq!(
            compile(
                &AmountExpr::authored(EvalMoment::Live, dynamic_add.clone()),
                &source(),
                DeliveryCapabilities::all(),
                &limits,
            )
            .is_ok(),
            accepted,
            "stack limit {limit}"
        );
    }

    for (limit, accepted) in [(2, false), (3, true), (4, true)] {
        let mut limits = ExprLimits::default();
        limits.max_work = limit;
        assert_eq!(
            compile(
                &AmountExpr::authored(EvalMoment::Live, dynamic_add.clone()),
                &source(),
                DeliveryCapabilities::all(),
                &limits,
            )
            .is_ok(),
            accepted,
            "work limit {limit}"
        );
    }

    let two_members = aggregate(
        members().into_iter().take(2).collect(),
        Grouping::WholeSet,
        Reduction::Sum,
        None,
    );
    for (limit, accepted) in [(1, false), (2, true), (3, true)] {
        let mut limits = ExprLimits::default();
        limits.max_dynamic_members = limit;
        assert_eq!(
            compile(
                &AmountExpr::authored(EvalMoment::Live, two_members.clone()),
                &source(),
                DeliveryCapabilities::all(),
                &limits,
            )
            .is_ok(),
            accepted,
            "member limit {limit}"
        );
    }
}

#[test]
fn nested_aggregate_cross_product_and_static_group_keys_are_admitted_up_front() {
    let many = (0..256)
        .map(|index| {
            AuthoredMember::scalar(
                format!("member_{index:03}"),
                TypedInteger::new(1, ValueUnit::Magnitude),
            )
        })
        .collect::<Vec<_>>();
    let nested = ValueExpr::Aggregate {
        source: FiniteSource::AuthoredMembers(many.clone()),
        grouping: Grouping::WholeSet,
        reduction: Reduction::Sum,
        outer_reduction: None,
        member: Box::new(aggregate(many, Grouping::WholeSet, Reduction::Sum, None)),
    };
    let error = compile(
        &AmountExpr::authored(EvalMoment::Live, nested),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::BudgetExceeded);
    assert!(error.cause.contains("worst-case work"));

    let missing_owner = aggregate(
        vec![AuthoredMember::scalar(
            "no_owner",
            TypedInteger::new(1, ValueUnit::Magnitude),
        )],
        Grouping::ByOwner,
        Reduction::Sum,
        Some(Reduction::Sum),
    );
    let error = compile(
        &AmountExpr::authored(EvalMoment::Live, missing_owner),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::InvalidOwner);

    let empty_identity = aggregate(
        vec![AuthoredMember::scalar(
            " ",
            TypedInteger::new(1, ValueUnit::Magnitude),
        )],
        Grouping::WholeSet,
        Reduction::Sum,
        None,
    );
    let error = compile(
        &AmountExpr::authored(EvalMoment::Live, empty_identity),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::InvalidOwner);

    let mixed_owner = aggregate(
        vec![
            AuthoredMember {
                identity: "employee".to_string(),
                value: TypedInteger::new(1, ValueUnit::Magnitude),
                owner: Some(owner("employee_1")),
                projected_tile: None,
            },
            AuthoredMember {
                identity: "side".to_string(),
                value: TypedInteger::new(2, ValueUnit::Magnitude),
                owner: Some(GroupOwner::new(GroupOwnerDomain::Side, "player").unwrap()),
                projected_tile: None,
            },
        ],
        Grouping::ByOwner,
        Reduction::Sum,
        Some(Reduction::Sum),
    );
    let error = compile(
        &AmountExpr::authored(EvalMoment::Live, mixed_owner),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::InvalidOwner);

    let resolved = ValueExpr::Aggregate {
        source: FiniteSource::ResolvedTargets,
        grouping: Grouping::ByOwner,
        reduction: Reduction::Sum,
        outer_reduction: Some(Reduction::Sum),
        member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        })),
    };
    let plan = compiled(resolved);
    let mut frame = EvalFrame::default();
    frame.set_resolved_targets(vec![
        EvalMember {
            identity: "employee".to_string(),
            value: Some(TypedInteger::new(1, ValueUnit::Magnitude)),
            owner: Some(owner("employee_1")),
            projected_tile: None,
            values: Default::default(),
        },
        EvalMember {
            identity: "side".to_string(),
            value: Some(TypedInteger::new(2, ValueUnit::Magnitude)),
            owner: Some(GroupOwner::new(GroupOwnerDomain::Side, "player").unwrap()),
            projected_tile: None,
            values: Default::default(),
        },
    ]);
    let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
    let error = evaluate(&plan, &frame, &mut meter).unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::InvalidOwner);
}

#[test]
fn capability_checks_prevent_unavailable_snapshot_context() {
    let expression = AmountExpr::authored(
        EvalMoment::Snapshot,
        ValueExpr::Read(ValueRef::Stat {
            subject: ValueSubject::Target,
            stat: StatSelector::Attack,
        }),
    );
    let error = compile(
        &expression,
        &source(),
        DeliveryCapabilities::constant_only(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::UnsupportedMoment);
}

#[test]
fn legacy_literal_lowering_and_final_narrowing_preserve_signed_contracts() {
    let damage_plan = compile(
        &AmountExpr::legacy_damage(i32::MIN),
        &source(),
        DeliveryCapabilities::constant_only(),
        &ExprLimits::default(),
    )
    .unwrap();
    let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
    let damage = evaluate(&damage_plan, &EvalFrame::default(), &mut meter).unwrap();
    assert_eq!(
        damage_plan.provenance(),
        AmountProvenance::LegacyDamageLiteral
    );
    assert_eq!(damage.value.value, 2_147_483_648);
    assert_eq!(
        narrow_damage(&source().site, &damage.value).unwrap(),
        2_147_483_648
    );

    let heal_plan = compile(
        &AmountExpr::legacy_heal(-5),
        &source(),
        DeliveryCapabilities::constant_only(),
        &ExprLimits::default(),
    )
    .unwrap();
    let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
    let heal = evaluate(&heal_plan, &EvalFrame::default(), &mut meter).unwrap();
    assert_eq!(heal_plan.provenance(), AmountProvenance::LegacyHealLiteral);
    assert_eq!(narrow_heal(&source().site, &heal.value).unwrap(), -5);

    let negative_damage = TypedInteger::new(-1, ValueUnit::Magnitude);
    assert_eq!(
        narrow_damage(&source().site, &negative_damage)
            .unwrap_err()
            .code,
        ValueExprFaultCode::Overflow
    );
    let large_heal = TypedInteger::new(i128::from(i32::MAX) + 1, ValueUnit::Magnitude);
    assert_eq!(
        narrow_heal(&source().site, &large_heal).unwrap_err().code,
        ValueExprFaultCode::Overflow
    );
}

#[test]
fn correction_compile_work_and_storage_boundaries_are_exact() {
    let expression = AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Add(vec![
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
            typed(1, ValueUnit::Magnitude),
        ]),
    );
    let admitted = compile(
        &expression,
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap();
    let cost = admitted.cost();

    for (limit, accepted) in [
        (cost.compile_work - 1, false),
        (cost.compile_work, true),
        (cost.compile_work + 1, true),
    ] {
        let mut limits = ExprLimits::default();
        limits.max_compile_work = limit;
        assert_eq!(
            compile(&expression, &source(), DeliveryCapabilities::all(), &limits,).is_ok(),
            accepted,
            "compile work limit {limit}"
        );
    }

    for (limit, accepted) in [
        (cost.compile_reserved_bytes - 1, false),
        (cost.compile_reserved_bytes, true),
        (cost.compile_reserved_bytes + 1, true),
    ] {
        let mut limits = ExprLimits::default();
        limits.max_retained_bytes = limit;
        assert_eq!(
            compile(&expression, &source(), DeliveryCapabilities::all(), &limits,).is_ok(),
            accepted,
            "compile retained limit {limit}"
        );
    }
}

#[test]
fn correction_raw_and_canonical_member_budgets_are_distinct_and_exact() {
    let plan = compiled(ValueExpr::Aggregate {
        source: FiniteSource::ResolvedTargets,
        grouping: Grouping::WholeSet,
        reduction: Reduction::Sum,
        outer_reduction: None,
        member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        })),
    });
    let duplicate = EvalMember {
        identity: "same".to_string(),
        value: Some(TypedInteger::new(7, ValueUnit::Magnitude)),
        owner: None,
        projected_tile: None,
        values: BTreeMap::new(),
    };
    let mut frame = EvalFrame::default();
    frame.set_resolved_targets(vec![duplicate; 8]);
    let mut reference_meter = ExprMeter::new(ExprLimits::default()).unwrap();
    let result = evaluate(&plan, &frame, &mut reference_meter).unwrap();
    let usage = reference_meter.usage();
    assert_eq!(result.value.value, 7);
    assert_eq!(usage.raw_members, 8);
    assert_eq!(usage.dynamic_members, 1);

    for (limit, accepted) in [
        (usage.work - 1, false),
        (usage.work, true),
        (usage.work + 1, true),
    ] {
        let mut limits = ExprLimits::default();
        limits.max_work = limit;
        limits.max_dynamic_members = 1;
        let mut meter = ExprMeter::new(limits).unwrap();
        assert_eq!(
            evaluate(&plan, &frame, &mut meter).is_ok(),
            accepted,
            "raw preprocessing work limit {limit}"
        );
    }

    for (limit, accepted) in [
        (usage.retained_bytes - 1, false),
        (usage.retained_bytes, true),
        (usage.retained_bytes + 1, true),
    ] {
        let mut limits = ExprLimits::default();
        limits.max_dynamic_members = 1;
        limits.max_retained_bytes = limit;
        let mut meter = ExprMeter::new(limits).unwrap();
        assert_eq!(
            evaluate(&plan, &frame, &mut meter).is_ok(),
            accepted,
            "evaluation retained limit {limit}"
        );
    }

    let mut unique_frame = EvalFrame::default();
    unique_frame.set_resolved_targets(vec![
        EvalMember {
            identity: "a".to_string(),
            value: Some(TypedInteger::new(1, ValueUnit::Magnitude)),
            owner: None,
            projected_tile: None,
            values: BTreeMap::new(),
        },
        EvalMember {
            identity: "b".to_string(),
            value: Some(TypedInteger::new(2, ValueUnit::Magnitude)),
            owner: None,
            projected_tile: None,
            values: BTreeMap::new(),
        },
    ]);
    for (limit, accepted) in [(1, false), (2, true), (3, true)] {
        let mut limits = ExprLimits::default();
        limits.max_dynamic_members = limit;
        let mut meter = ExprMeter::new(limits).unwrap();
        assert_eq!(
            evaluate(&plan, &unique_frame, &mut meter).is_ok(),
            accepted,
            "canonical member limit {limit}"
        );
    }
}

#[test]
fn correction_long_payloads_are_rejected_before_kernel_owned_clones() {
    let short_member = AuthoredMember {
        identity: "member".to_string(),
        value: TypedInteger::new(1, ValueUnit::Magnitude),
        owner: Some(owner("employee")),
        projected_tile: None,
    };
    let short_expression = AmountExpr::authored(
        EvalMoment::Live,
        aggregate(
            vec![short_member.clone()],
            Grouping::ByOwner,
            Reduction::Sum,
            Some(Reduction::Sum),
        ),
    );
    let short = compile(
        &short_expression,
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap();
    let mut exact = ExprLimits::default();
    exact.max_retained_bytes = short.cost().compile_reserved_bytes;
    assert!(compile(
        &short_expression,
        &source(),
        DeliveryCapabilities::all(),
        &exact,
    )
    .is_ok());

    let long_member = AuthoredMember {
        owner: Some(owner(&"x".repeat(131_072))),
        ..short_member
    };
    let long_expression = AmountExpr::authored(
        EvalMoment::Live,
        aggregate(
            vec![long_member],
            Grouping::ByOwner,
            Reduction::Sum,
            Some(Reduction::Sum),
        ),
    );
    let error = compile(
        &long_expression,
        &source(),
        DeliveryCapabilities::all(),
        &exact,
    )
    .unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::BudgetExceeded);
    assert!(error.to_string().len() < 1024);

    let plan = compiled(ValueExpr::Aggregate {
        source: FiniteSource::ResolvedTargets,
        grouping: Grouping::WholeSet,
        reduction: Reduction::Sum,
        outer_reduction: None,
        member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        })),
    });
    let mut frame = EvalFrame::default();
    frame.set_resolved_targets(vec![EvalMember {
        identity: "z".repeat(16_384),
        value: Some(TypedInteger::new(1, ValueUnit::Magnitude)),
        owner: None,
        projected_tile: None,
        values: BTreeMap::new(),
    }]);
    let mut limits = ExprLimits::default();
    limits.max_retained_bytes = 4096;
    let mut meter = ExprMeter::new(limits).unwrap();
    let error = evaluate(&plan, &frame, &mut meter).unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::BudgetExceeded);
    assert!(error.to_string().len() < 1024);
}

#[test]
fn correction_nested_stack_nodes_depth_and_task_frames_are_enforced() {
    let member_expr = ValueExpr::Add(vec![
        ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        }),
        typed(1, ValueUnit::Magnitude),
    ]);
    let root = ValueExpr::Add(vec![
        ValueExpr::Read(ValueRef::Stat {
            subject: ValueSubject::Source,
            stat: StatSelector::Attack,
        }),
        typed(1, ValueUnit::Magnitude),
        ValueExpr::Aggregate {
            source: FiniteSource::AuthoredMembers(vec![AuthoredMember::scalar(
                "member",
                TypedInteger::new(2, ValueUnit::Magnitude),
            )]),
            grouping: Grouping::WholeSet,
            reduction: Reduction::Sum,
            outer_reduction: None,
            member: Box::new(member_expr),
        },
    ]);
    let plan = compiled(root);
    assert_eq!(plan.cost().max_stack, 4);
    assert_eq!(plan.cost().max_task_frames, 3);
    let mut frame = EvalFrame::default();
    frame
        .insert(
            ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            },
            TypedInteger::new(10, ValueUnit::Magnitude),
        )
        .unwrap();

    for (limit, accepted) in [(3, false), (4, true), (5, true)] {
        let mut limits = ExprLimits::default();
        limits.max_stack = limit;
        let mut meter = ExprMeter::new(limits).unwrap();
        let result = evaluate(&plan, &frame, &mut meter);
        assert_eq!(result.is_ok(), accepted, "stack limit {limit}");
        if accepted {
            assert_eq!(result.unwrap().value.value, 14);
            assert_eq!(meter.usage().peak_stack_values, 4);
            assert_eq!(meter.usage().peak_task_frames, 3);
        }
    }

    for (required, field) in [
        (plan.cost().nodes, "nodes"),
        (plan.cost().max_depth, "depth"),
    ] {
        for (limit, accepted) in [
            (required - 1, false),
            (required, true),
            (required + 1, true),
        ] {
            let mut limits = ExprLimits::default();
            if field == "nodes" {
                limits.max_nodes = limit;
            } else {
                limits.max_depth = limit;
            }
            let mut meter = ExprMeter::new(limits).unwrap();
            assert_eq!(
                evaluate(&plan, &frame, &mut meter).is_ok(),
                accepted,
                "{field} limit {limit}"
            );
        }
    }
}

#[test]
fn correction_group_boundaries_and_cumulative_resource_capabilities_hold() {
    let plan = compiled(ValueExpr::Aggregate {
        source: FiniteSource::ResolvedTargets,
        grouping: Grouping::ByOwner,
        reduction: Reduction::Sum,
        outer_reduction: Some(Reduction::Sum),
        member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        })),
    });
    let mut frame = EvalFrame::default();
    frame.set_resolved_targets(vec![
        EvalMember {
            identity: "a".to_string(),
            value: Some(TypedInteger::new(1, ValueUnit::Magnitude)),
            owner: Some(owner("employee_a")),
            projected_tile: None,
            values: BTreeMap::new(),
        },
        EvalMember {
            identity: "b".to_string(),
            value: Some(TypedInteger::new(2, ValueUnit::Magnitude)),
            owner: Some(owner("employee_b")),
            projected_tile: None,
            values: BTreeMap::new(),
        },
    ]);
    for (limit, accepted) in [(1, false), (2, true), (3, true)] {
        let mut limits = ExprLimits::default();
        limits.max_groups = limit;
        let mut meter = ExprMeter::new(limits).unwrap();
        assert_eq!(
            evaluate(&plan, &frame, &mut meter).is_ok(),
            accepted,
            "group limit {limit}"
        );
    }

    let mana = ResourceId::new("mana".to_string()).unwrap();
    let target_resource = ValueRef::Resource {
        resource: mana.clone(),
        owner: ResourceOwnerSelector::TargetEmployee,
        field: ResourceValueField::Balance,
    };
    let source_resource = ValueRef::Resource {
        resource: mana,
        owner: ResourceOwnerSelector::SourceEquipment,
        field: ResourceValueField::Balance,
    };

    let mut resources_only = DeliveryCapabilities::constant_only();
    resources_only.resources = true;
    assert!(compile(
        &AmountExpr::authored(EvalMoment::Snapshot, ValueExpr::Read(source_resource)),
        &source(),
        resources_only,
        &ExprLimits::default(),
    )
    .is_ok());
    let missing_target = compile(
        &AmountExpr::authored(
            EvalMoment::Snapshot,
            ValueExpr::Read(target_resource.clone()),
        ),
        &source(),
        resources_only,
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(missing_target.code, ValueExprFaultCode::UnsupportedMoment);

    let mut target_only = DeliveryCapabilities::constant_only();
    target_only.target_at_release = true;
    let missing_resource = compile(
        &AmountExpr::authored(
            EvalMoment::Snapshot,
            ValueExpr::Read(target_resource.clone()),
        ),
        &source(),
        target_only,
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(missing_resource.code, ValueExprFaultCode::MissingContext);

    let mut snapshot_both = resources_only;
    snapshot_both.target_at_release = true;
    assert!(compile(
        &AmountExpr::authored(
            EvalMoment::Snapshot,
            ValueExpr::Read(target_resource.clone()),
        ),
        &source(),
        snapshot_both,
        &ExprLimits::default(),
    )
    .is_ok());

    let live_missing_consume = compile(
        &AmountExpr::authored(EvalMoment::Live, ValueExpr::Read(target_resource.clone())),
        &source(),
        snapshot_both,
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(
        live_missing_consume.code,
        ValueExprFaultCode::UnsupportedMoment
    );
    snapshot_both.target_at_consume = true;
    let live_plan = compile(
        &AmountExpr::authored(EvalMoment::Live, ValueExpr::Read(target_resource)),
        &source(),
        snapshot_both,
        &ExprLimits::default(),
    )
    .unwrap();

    for source_owner in [
        ResourceOwnerSelector::SourceEmployee,
        ResourceOwnerSelector::SourceEquipment,
    ] {
        for moment in [EvalMoment::Snapshot, EvalMoment::Live] {
            let reference = ValueRef::Resource {
                resource: ResourceId::new("source_mana".to_string()).unwrap(),
                owner: source_owner,
                field: ResourceValueField::Available,
            };
            let missing = compile(
                &AmountExpr::authored(moment, ValueExpr::Read(reference.clone())),
                &source(),
                DeliveryCapabilities::constant_only(),
                &ExprLimits::default(),
            )
            .unwrap_err();
            assert_eq!(missing.code, ValueExprFaultCode::MissingContext);
            assert!(compile(
                &AmountExpr::authored(moment, ValueExpr::Read(reference)),
                &source(),
                resources_only,
                &ExprLimits::default(),
            )
            .is_ok());
        }
    }

    for moment in [EvalMoment::Snapshot, EvalMoment::Live] {
        for (resources, target, accepted) in [
            (false, false, false),
            (false, true, false),
            (true, false, false),
            (true, true, true),
        ] {
            let mut capabilities = DeliveryCapabilities::constant_only();
            capabilities.resources = resources;
            match moment {
                EvalMoment::Snapshot => capabilities.target_at_release = target,
                EvalMoment::Live => capabilities.target_at_consume = target,
            }
            let reference = ValueRef::Resource {
                resource: ResourceId::new("target_mana".to_string()).unwrap(),
                owner: ResourceOwnerSelector::TargetEmployee,
                field: ResourceValueField::Balance,
            };
            assert_eq!(
                compile(
                    &AmountExpr::authored(moment, ValueExpr::Read(reference)),
                    &source(),
                    capabilities,
                    &ExprLimits::default(),
                )
                .is_ok(),
                accepted,
                "target resource moment={moment:?} resources={resources} target={target}"
            );
        }
    }

    let live_reference = ValueRef::Resource {
        resource: ResourceId::new("mana".to_string()).unwrap(),
        owner: ResourceOwnerSelector::TargetEmployee,
        field: ResourceValueField::Balance,
    };
    let mut positive_frame = EvalFrame::default();
    positive_frame
        .insert(
            live_reference,
            TypedInteger::new(
                23,
                ValueUnit::Resource(ResourceId::new("mana".to_string()).unwrap()),
            ),
        )
        .unwrap();
    let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
    assert_eq!(
        evaluate(&live_plan, &positive_frame, &mut meter)
            .unwrap()
            .value
            .value,
        23
    );
}

fn resolved_owner_aggregate(source: FiniteSource, relation: ResolvedOwnerRelation) -> ValueExpr {
    ValueExpr::Aggregate {
        source,
        grouping: Grouping::ByResolvedOwner(relation),
        reduction: Reduction::Sum,
        outer_reduction: Some(Reduction::Sum),
        member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        })),
    }
}

#[test]
fn resolved_predicates_are_bounded_normalized_and_fingerprinted_with_slot_meaning() {
    let employee = ResolvedTargetPredicate::IsActualEmployee;
    let weapon = ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Weapon);
    let left = compiled(resolved_owner_aggregate(
        FiniteSource::ResolvedTargetsWhere(vec![weapon, employee, weapon]),
        ResolvedOwnerRelation::EquippedSlot(EquipmentSlotAddress::Weapon),
    ));
    let right = compiled(resolved_owner_aggregate(
        FiniteSource::ResolvedTargetsWhere(vec![employee, weapon]),
        ResolvedOwnerRelation::EquippedSlot(EquipmentSlotAddress::Weapon),
    ));
    assert_eq!(left.semantic_fingerprint(), right.semantic_fingerprint());
    assert_eq!(
        left.requirements().resolved_target_predicates,
        vec![employee, weapon]
    );
    assert_eq!(
        left.requirements().resolved_owner_relations,
        vec![ResolvedOwnerRelation::EquippedSlot(
            EquipmentSlotAddress::Weapon
        )]
    );
    assert!(left.cost().nodes >= 4);

    let armor = compiled(resolved_owner_aggregate(
        FiniteSource::ResolvedTargetsWhere(vec![employee]),
        ResolvedOwnerRelation::EquippedSlot(EquipmentSlotAddress::Armor),
    ));
    assert_ne!(left.semantic_fingerprint(), armor.semantic_fingerprint());

    let unfiltered = compiled(ValueExpr::Aggregate {
        source: FiniteSource::ResolvedTargets,
        grouping: Grouping::WholeSet,
        reduction: Reduction::Sum,
        outer_reduction: None,
        member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        })),
    });
    let empty_filter = compiled(ValueExpr::Aggregate {
        source: FiniteSource::ResolvedTargetsWhere(Vec::new()),
        grouping: Grouping::WholeSet,
        reduction: Reduction::Sum,
        outer_reduction: None,
        member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        })),
    });
    assert_eq!(
        unfiltered.semantic_fingerprint(),
        empty_filter.semantic_fingerprint()
    );

    let invalid = compile(
        &AmountExpr::authored(
            EvalMoment::Live,
            resolved_owner_aggregate(
                FiniteSource::ResolvedTargetsWhere(vec![ResolvedTargetPredicate::HasEquippedSlot(
                    EquipmentSlotAddress::Any3(3),
                )]),
                ResolvedOwnerRelation::EquippedSlot(EquipmentSlotAddress::Any3(3)),
            ),
        ),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(invalid.code, ValueExprFaultCode::InvalidOwner);

    let authored = compile(
        &AmountExpr::authored(
            EvalMoment::Live,
            resolved_owner_aggregate(
                FiniteSource::AuthoredMembers(vec![AuthoredMember::scalar(
                    "member",
                    TypedInteger::new(1, ValueUnit::Magnitude),
                )]),
                ResolvedOwnerRelation::Side,
            ),
        ),
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(authored.code, ValueExprFaultCode::InvalidOwner);

    let boundary_expr = AmountExpr::authored(
        EvalMoment::Live,
        resolved_owner_aggregate(
            FiniteSource::ResolvedTargetsWhere(vec![employee, weapon]),
            ResolvedOwnerRelation::ActualEmployee,
        ),
    );
    let mut exact = ExprLimits::default();
    exact.max_nodes = 4;
    assert_eq!(
        compile(
            &boundary_expr,
            &source(),
            DeliveryCapabilities::all(),
            &exact,
        )
        .unwrap()
        .cost()
        .nodes,
        4
    );
    exact.max_nodes = 3;
    assert_eq!(
        compile(
            &boundary_expr,
            &source(),
            DeliveryCapabilities::all(),
            &exact,
        )
        .unwrap_err()
        .code,
        ValueExprFaultCode::BudgetExceeded
    );

    let encoded = ron::ser::to_string(&boundary_expr).unwrap();
    let decoded: AmountExpr = ron::de::from_str(&encoded).unwrap();
    assert_eq!(decoded, boundary_expr);
    assert_eq!(
        compile(
            &decoded,
            &source(),
            DeliveryCapabilities::all(),
            &ExprLimits::default(),
        )
        .unwrap()
        .semantic_fingerprint(),
        compile(
            &boundary_expr,
            &source(),
            DeliveryCapabilities::all(),
            &ExprLimits::default(),
        )
        .unwrap()
        .semantic_fingerprint()
    );
}

#[test]
fn resolved_relation_requires_its_exact_node_view_and_distinguishes_empty() {
    let plan = compiled(resolved_owner_aggregate(
        FiniteSource::ResolvedTargetsWhere(vec![ResolvedTargetPredicate::IsActualEmployee]),
        ResolvedOwnerRelation::ActualEmployee,
    ));
    let mut wrong = EvalFrame::default();
    wrong.set_resolved_target_views(vec![PreparedResolvedTargetView {
        node_index: 99,
        members: Arc::new(Vec::new()),
    }]);
    let mut meter = ExprMeter::new(plan.evaluation_limits()).unwrap();
    assert_eq!(
        evaluate(&plan, &wrong, &mut meter).unwrap_err().code,
        ValueExprFaultCode::MissingContext
    );

    let mut empty = EvalFrame::default();
    empty.set_resolved_target_views(vec![PreparedResolvedTargetView {
        node_index: 0,
        members: Arc::new(Vec::new()),
    }]);
    let mut meter = ExprMeter::new(plan.evaluation_limits()).unwrap();
    let evaluated = evaluate(&plan, &empty, &mut meter).unwrap();
    assert_eq!(evaluated.value, TypedInteger::new(0, ValueUnit::Magnitude));
    assert_eq!(meter.usage().raw_members, 0);
}

#[test]
fn correction_wide_add_rejects_before_child_traversal_and_keeps_left_order() {
    let wide = AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Add(
            (0..65_536)
                .map(|_| typed(1, ValueUnit::Magnitude))
                .collect(),
        ),
    );
    let mut limits = ExprLimits::default();
    limits.max_nodes = 4;
    let error = compile(&wide, &source(), DeliveryCapabilities::all(), &limits).unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::BudgetExceeded);
    assert!(error.cause.contains("direct children"));

    let ordered_overflow = AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Add(vec![
            typed(i128::MAX, ValueUnit::Magnitude),
            typed(1, ValueUnit::Magnitude),
            typed(-1, ValueUnit::Magnitude),
        ]),
    );
    let error = compile(
        &ordered_overflow,
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::Overflow);
}

#[test]
fn correction_streaming_fingerprint_is_byte_compatible_with_pre_fix_plan() {
    let mut fingerprint_source = source();
    fingerprint_source.site.action.definition_id = SkillId::new("fingerprint");
    fingerprint_source.site.action.action_id = "strike".to_string();
    fingerprint_source.site.effect_id = "damage".to_string();

    let expressions = [
        (
            AmountExpr::authored(
                EvalMoment::Live,
                ValueExpr::Const(TypedInteger::new(42, ValueUnit::Magnitude)),
            ),
            "39615ba8f80ab6d23c5e3b7456f417e27454bbf848bfc76bd65866765fb13caa",
        ),
        (
            AmountExpr::authored(
                EvalMoment::Live,
                ValueExpr::Add(vec![
                    ValueExpr::Read(ValueRef::Stat {
                        subject: ValueSubject::Source,
                        stat: StatSelector::Attack,
                    }),
                    ValueExpr::Read(ValueRef::Stat {
                        subject: ValueSubject::Source,
                        stat: StatSelector::SpellPower,
                    }),
                ]),
            ),
            "143e8af8e4716e6aaddf21cbdef6275a6d6543cbfda7791df39eb81ad14be2ae",
        ),
        (
            AmountExpr::authored(
                EvalMoment::Live,
                ValueExpr::Aggregate {
                    source: FiniteSource::AuthoredMembers(vec![
                        AuthoredMember::scalar("b", TypedInteger::new(2, ValueUnit::Magnitude)),
                        AuthoredMember::scalar("a", TypedInteger::new(1, ValueUnit::Magnitude)),
                    ]),
                    grouping: Grouping::WholeSet,
                    reduction: Reduction::Sum,
                    outer_reduction: None,
                    member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
                        unit: ValueUnit::Magnitude,
                    })),
                },
            ),
            "30673c776d40fef4e594b3f43d32043379440dbd19fbc23e1e4ec521df75b308",
        ),
    ];

    for (expression, pre_fix_fingerprint) in expressions {
        let plan = compile(
            &expression,
            &fingerprint_source,
            DeliveryCapabilities::all(),
            &ExprLimits::default(),
        )
        .unwrap();
        assert_eq!(plan.semantic_fingerprint_hex(), pre_fix_fingerprint);
    }
}

fn correction_002_authored_member(
    identity: String,
    owner_identity: Option<String>,
    tile: Option<ProjectedTile>,
) -> AuthoredMember {
    AuthoredMember {
        identity,
        value: TypedInteger::new(1, ValueUnit::Magnitude),
        owner: owner_identity.map(|identity| owner(&identity)),
        projected_tile: tile,
    }
}

fn assert_static_work_boundary(expression: &AmountExpr) -> (PlanCost, ExprUsage) {
    let default_limits = ExprLimits::default();
    let plan = compile(
        expression,
        &source(),
        DeliveryCapabilities::all(),
        &default_limits,
    )
    .unwrap();
    let cost = plan.cost();
    assert!(!cost.runtime_dynamic_work);
    assert_eq!(cost.static_work, cost.worst_case_work);

    let mut meter = ExprMeter::new(default_limits).unwrap();
    evaluate(&plan, &EvalFrame::default(), &mut meter).unwrap();
    let usage = meter.usage();
    assert!(
        usage.work <= cost.static_work,
        "actual {} exceeded static bound {}",
        usage.work,
        cost.static_work
    );

    for (limit, accepted) in [
        (cost.static_work - 1, false),
        (cost.static_work, true),
        (cost.static_work + 1, true),
    ] {
        let limits = ExprLimits {
            max_work: limit,
            ..default_limits
        };
        let compiled = compile(expression, &source(), DeliveryCapabilities::all(), &limits);
        assert_eq!(compiled.is_ok(), accepted, "static work limit {limit}");
        if let Ok(plan) = compiled {
            let mut meter = ExprMeter::new(limits).unwrap();
            assert!(evaluate(&plan, &EvalFrame::default(), &mut meter).is_ok());
            assert!(meter.usage().work <= limit);
        }
    }
    (cost, usage)
}

#[test]
fn correction_002_authored_work_is_a_static_executable_bound() {
    let one = AmountExpr::authored(
        EvalMoment::Live,
        aggregate(
            vec![correction_002_authored_member("0000".into(), None, None)],
            Grouping::WholeSet,
            Reduction::Sum,
            None,
        ),
    );
    let (cost, usage) = assert_static_work_boundary(&one);
    assert_eq!((cost.static_work, usage.work), (7, 7));

    for count in [0, 1, 2, 8, 32] {
        for identity_len in [0, 32, 128] {
            let expression = AmountExpr::authored(
                EvalMoment::Live,
                aggregate(
                    (0..count)
                        .map(|index| {
                            correction_002_authored_member(
                                format!("{}{:04}", "x".repeat(identity_len), index),
                                None,
                                None,
                            )
                        })
                        .collect(),
                    Grouping::WholeSet,
                    Reduction::Sum,
                    None,
                ),
            );
            let plan = compile(
                &expression,
                &source(),
                DeliveryCapabilities::all(),
                &ExprLimits::default(),
            )
            .unwrap();
            let mut meter = ExprMeter::new(ExprLimits::default()).unwrap();
            evaluate(&plan, &EvalFrame::default(), &mut meter).unwrap();
            assert_eq!(
                plan.cost().static_work,
                meter.usage().work,
                "count={count} identity_len={identity_len}"
            );
            assert_eq!(plan.cost().static_work, plan.cost().worst_case_work);
        }
    }

    let cases = [
        AmountExpr::authored(
            EvalMoment::Live,
            aggregate(Vec::new(), Grouping::WholeSet, Reduction::Sum, None),
        ),
        AmountExpr::authored(
            EvalMoment::Live,
            aggregate(
                vec![correction_002_authored_member(
                    format!("{}0000", "identity".repeat(512)),
                    None,
                    None,
                )],
                Grouping::WholeSet,
                Reduction::Sum,
                None,
            ),
        ),
        AmountExpr::authored(
            EvalMoment::Live,
            aggregate(
                (0..4)
                    .map(|index| {
                        correction_002_authored_member(
                            format!("owner_member_{index}"),
                            Some(format!("owner_{}", index % 2)),
                            None,
                        )
                    })
                    .collect(),
                Grouping::ByOwner,
                Reduction::Sum,
                Some(Reduction::Sum),
            ),
        ),
        AmountExpr::authored(
            EvalMoment::Live,
            aggregate(
                (0..4)
                    .map(|index| {
                        correction_002_authored_member(
                            format!("tile_member_{index}"),
                            None,
                            Some(ProjectedTile {
                                x: index % 2,
                                y: index / 2,
                            }),
                        )
                    })
                    .collect(),
                Grouping::ByProjectedTile,
                Reduction::Sum,
                Some(Reduction::Sum),
            ),
        ),
    ];
    for expression in &cases {
        assert_static_work_boundary(expression);
    }

    let resource = ResourceId::new(format!("resource_{}", "r".repeat(100))).unwrap();
    let resource_expression = AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Aggregate {
            source: FiniteSource::AuthoredMembers(vec![AuthoredMember::scalar(
                "resource_member",
                TypedInteger::new(3, ValueUnit::Resource(resource.clone())),
            )]),
            grouping: Grouping::WholeSet,
            reduction: Reduction::Sum,
            outer_reduction: None,
            member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
                unit: ValueUnit::Resource(resource),
            })),
        },
    );
    assert_static_work_boundary(&resource_expression);
}

#[test]
fn correction_002_grouped_and_nested_authored_work_stays_bounded() {
    let inner = aggregate(
        (0..3)
            .map(|index| {
                correction_002_authored_member(
                    format!("inner_{index}_{}", "i".repeat(96)),
                    Some(format!("inner_owner_{}", index % 2)),
                    None,
                )
            })
            .collect(),
        Grouping::ByOwner,
        Reduction::Sum,
        Some(Reduction::Sum),
    );
    let nested = AmountExpr::authored(
        EvalMoment::Live,
        ValueExpr::Aggregate {
            source: FiniteSource::AuthoredMembers(
                (0..2)
                    .map(|index| {
                        correction_002_authored_member(
                            format!("outer_{index}"),
                            None,
                            Some(ProjectedTile { x: index, y: 0 }),
                        )
                    })
                    .collect(),
            ),
            grouping: Grouping::ByProjectedTile,
            reduction: Reduction::Sum,
            outer_reduction: Some(Reduction::Sum),
            member: Box::new(inner),
        },
    );
    let (cost, usage) = assert_static_work_boundary(&nested);
    assert!(usage.work <= cost.static_work);

    let empty_grouped = AmountExpr::authored(
        EvalMoment::Live,
        aggregate(
            Vec::new(),
            Grouping::ByOwner,
            Reduction::Sum,
            Some(Reduction::Sum),
        ),
    );
    assert_static_work_boundary(&empty_grouped);
}

#[test]
fn correction_002_runtime_payload_work_is_explicitly_dynamic() {
    let resolved = compiled(ValueExpr::Aggregate {
        source: FiniteSource::ResolvedTargets,
        grouping: Grouping::WholeSet,
        reduction: Reduction::Sum,
        outer_reduction: None,
        member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
            unit: ValueUnit::Magnitude,
        })),
    });
    assert!(resolved.cost().runtime_dynamic_work);
    assert_eq!(
        resolved.cost().worst_case_work,
        ExprLimits::default().max_work
    );
    assert!(resolved.cost().static_work < resolved.cost().worst_case_work);

    let frame_read = compiled(ValueExpr::Read(ValueRef::Stat {
        subject: ValueSubject::Source,
        stat: StatSelector::Attack,
    }));
    assert!(frame_read.cost().runtime_dynamic_work);
    assert_eq!(
        frame_read.cost().worst_case_work,
        ExprLimits::default().max_work
    );

    let literal = compiled(typed(1, ValueUnit::Magnitude));
    assert!(!literal.cost().runtime_dynamic_work);
    assert_eq!(literal.cost().static_work, literal.cost().worst_case_work);
}

fn correction_002_nested_aggregates(count: usize) -> AmountExpr {
    let mut root = typed(1, ValueUnit::Magnitude);
    for _ in 0..count {
        root = ValueExpr::Aggregate {
            source: FiniteSource::AuthoredMembers(vec![AuthoredMember::scalar(
                "member",
                TypedInteger::new(1, ValueUnit::Magnitude),
            )]),
            grouping: Grouping::WholeSet,
            reduction: Reduction::Sum,
            outer_reduction: None,
            member: Box::new(root),
        };
    }
    AmountExpr::authored(EvalMoment::Live, root)
}

#[test]
fn correction_002_shape_is_iterative_at_default_and_extended_depth() {
    let default_boundary = correction_002_nested_aggregates(63);
    let default_plan = compile(
        &default_boundary,
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap();
    assert_eq!(default_plan.cost().max_depth, 64);

    let over_default = correction_002_nested_aggregates(64);
    let error = compile(
        &over_default,
        &source(),
        DeliveryCapabilities::all(),
        &ExprLimits::default(),
    )
    .unwrap_err();
    assert_eq!(error.code, ValueExprFaultCode::BudgetExceeded);

    let extended = std::mem::ManuallyDrop::new(correction_002_nested_aggregates(512));
    let generous = ExprLimits {
        max_depth: 513,
        max_nodes: 513,
        max_compile_work: 4 * 1024 * 1024,
        max_retained_bytes: 16 * 1024 * 1024,
        ..ExprLimits::default()
    };
    let plan = compile(&extended, &source(), DeliveryCapabilities::all(), &generous).unwrap();
    assert_eq!(plan.cost().max_depth, 513);
    assert_eq!(plan.cost().max_task_frames, 1025);
    let required = plan.cost().compile_reserved_bytes;

    for (limit, accepted) in [
        (required - 1, false),
        (required, true),
        (required + 1, true),
    ] {
        let limits = ExprLimits {
            max_retained_bytes: limit,
            ..generous
        };
        assert_eq!(
            compile(&extended, &source(), DeliveryCapabilities::all(), &limits).is_ok(),
            accepted,
            "shape scratch retained limit {limit}"
        );
    }

    let mut meter = ExprMeter::new(generous).unwrap();
    let result = evaluate(&plan, &EvalFrame::default(), &mut meter).unwrap();
    assert_eq!(result.value, TypedInteger::new(1, ValueUnit::Magnitude));
    assert_eq!(meter.usage().peak_task_frames, 1025);
}
