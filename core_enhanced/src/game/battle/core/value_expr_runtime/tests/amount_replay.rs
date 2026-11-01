//! C04/C05: real release/consumption records replay through both query encoders.
//! The private encoder does not exercise public Latest/Exact selection.
use super::*;
use crate::game::battle::damage::{DamageSource, DamageSourceSnapshot, DamageType};

fn observed_fixture(
    skills: SkillDatabase,
    resources: Vec<ResourceDefinition>,
) -> (
    BattleCore,
    UnitInstanceId,
    UnitInstanceId,
    crate::game::ability::SkillDef,
) {
    let (mut core, actor, target) = core_with_units_and_skills(resources, Some(skills));
    let skill = core
        .game_data
        .skill_data
        .get_by_id("wp02_runtime_frame")
        .unwrap()
        .clone();
    let plan = Arc::clone(
        core.game_data
            .value_expr_catalog()
            .entries()
            .find(|(_, entry)| entry.is_authored_expression())
            .unwrap()
            .1
            .plan(),
    );
    core.runtime_fact_source = occurrence(&mut core, actor, &plan);
    core.enable_authoring_observation_with_options(Default::default(), true);
    core.bind_authoring_observation(super::super::super::authoring_observation::ObservationKey {
        entry_ordinal: 14,
        battle_uuid: Uuid::from_u128(0xf0304),
    })
    .unwrap();
    (core, actor, target, skill)
}

fn release(
    core: &mut BattleCore,
    actor: UnitInstanceId,
    target: UnitInstanceId,
    step: &crate::game::ability::SkillStepDef,
    time: u64,
) -> DamageSourceSnapshot {
    let mut source = core.runtime_fact_source.clone();
    source.allows_finite_children = true;
    let source = core.reserve_finite_source(source).unwrap();
    assert!(source.delivery.is_some());
    let carrier = core
        .build_value_expr_delivery_capture(step, &source, Some(&[target]), time, None)
        .unwrap()
        .unwrap();
    let mut template = core
        .damage_source_snapshot_for_unit(
            actor,
            target,
            DamageSource::Ability,
            DamageType::True,
            0,
            Default::default(),
            0,
            time,
            false,
        )
        .unwrap();
    template.fact_source = source;
    template.value_expr_capture = Some(carrier);
    template
}

fn consume(
    core: &mut BattleCore,
    actor: UnitInstanceId,
    target: UnitInstanceId,
    skill: &crate::game::ability::SkillDef,
    step_index: usize,
    template: &DamageSourceSnapshot,
    release_time: u64,
    consume_time: u64,
) {
    core.runtime_transaction(|candidate| {
        // Match the delivery caller: the carrier and command execution share one source.
        let prior = std::mem::replace(
            &mut candidate.runtime_fact_source,
            template.fact_source.clone(),
        );
        let result = candidate.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[step_index],
            &[target],
            consume_time,
            release_time,
            Some(template),
            None,
        );
        candidate.runtime_fact_source = prior;
        result
    })
    .unwrap();
    let record = core
        .authoring_observation
        .as_ref()
        .unwrap()
        .latest_amount
        .as_ref()
        .unwrap();
    assert_eq!(record.release_time_ms, release_time);
    assert_eq!(record.consume_time_ms, consume_time);
    assert_eq!(record.release_target, Some(target));
    assert_eq!(record.consumption_target, Some(target));
    assert_eq!(
        record.source.delivery.as_ref().unwrap().ordinal,
        template.fact_source.delivery.as_ref().unwrap().ordinal
    );
    let captured = template
        .value_expr_capture
        .as_ref()
        .unwrap()
        .capture_for(record.effect_index, Some(target))
        .unwrap();
    if record.moment == EvalMoment::Snapshot {
        assert!(Arc::ptr_eq(
            &record.frame,
            captured.snapshot_frame.as_ref().unwrap()
        ));
    } else {
        assert!(captured.snapshot_frame.is_none());
    }
}

fn query_pair(
    core: &BattleCore,
    label: &str,
    expected: RuntimeAmount,
    reference: &ValueRef,
    expected_input: i128,
) -> (String, String) {
    let observer = core.authoring_observation.as_ref().unwrap();
    let record = observer.latest_amount.as_ref().unwrap();
    let ordinal = observer.amount_evaluations;
    assert_eq!(record.evaluation_ordinal, ordinal);
    assert_eq!(record.amount, expected);
    let raw_expected = match expected {
        RuntimeAmount::Damage(v) | RuntimeAmount::Shield(v) => i128::from(v),
        RuntimeAmount::Heal(v) => i128::from(v),
    };
    assert_eq!(
        record.evaluated.value,
        TypedInteger::new(raw_expected, ValueUnit::Magnitude)
    );
    let input = TypedInteger::new(expected_input, reference.unit());
    assert_eq!(record.frame.get(reference), Some(&input));
    assert!(record
        .evaluated
        .trace
        .iter()
        .any(|entry| entry.value == input));
    let before = (
        core.transaction_debug_snapshot(),
        core.event_queue_debug_snapshot(),
        format!("{:?}", core.runtime_facts),
    );
    let query = crate::authoring::AmountQuery::Exact {
        battle_key: record.battle_key.into(),
        evaluation_ordinal: record.evaluation_ordinal,
    };
    let encode = |explanation| {
        let (result, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::Inspection,
            || crate::authoring::encode_record_for_test(record, query, explanation),
        )
        .unwrap();
        (result.unwrap(), usage)
    };
    let preview = encode(false);
    let explanation = encode(true);
    assert_eq!(preview, encode(false));
    assert_eq!(explanation, encode(true));
    for (document, kind) in [
        (&preview.0, "amount_preview"),
        (&explanation.0, "amount_explanation"),
    ] {
        let parsed: serde_json::Value = serde_json::from_str(document).unwrap();
        assert_eq!(parsed["kind"], kind);
        assert_eq!(parsed["status"], "evaluated");
        assert_eq!(parsed["basis"], "recorded_evaluation");
        assert_eq!(
            parsed["moment"],
            serde_json::to_value(record.moment).unwrap()
        );
        assert_eq!(
            parsed["narrowed_base_amount"],
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(
            parsed["typed_base_amount"],
            serde_json::to_value(&record.evaluated.value).unwrap()
        );
        assert_eq!(
            parsed["trace"],
            serde_json::to_value(&record.evaluated.trace).unwrap()
        );
        assert_eq!(parsed["identity"]["evaluation_ordinal"], ordinal);
        assert_eq!(parsed["release_time_ms"], record.release_time_ms);
        assert_eq!(parsed["consume_time_ms"], record.consume_time_ms);
        if kind == "amount_explanation" {
            assert_eq!(parsed["static_plan"]["evaluation"]["performed"], false);
        }
    }
    assert_eq!(
        before,
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{:?}", core.runtime_facts)
        )
    );
    assert_eq!(
        core.authoring_observation
            .as_ref()
            .unwrap()
            .amount_evaluations,
        ordinal
    );
    assert!(Arc::ptr_eq(
        record,
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_amount
            .as_ref()
            .unwrap()
    ));
    eprintln!(
        "D07_F03_REPLAY {label} input={expected_input} expected={expected:?} preview={} explain={}",
        preview.0, explanation.0
    );
    (preview.0, explanation.0)
}

#[test]
fn d07_f03_c04_c05_attack_release_consume_and_later_queries_keep_distinct_moments() {
    let reference = ValueRef::Stat {
        subject: ValueSubject::Source,
        stat: StatSelector::Attack,
    };
    for (moment, expected) in [(EvalMoment::Snapshot, 10), (EvalMoment::Live, 20)] {
        let effects = format!(
            r#"(id: "amount", op: DamageExpr(amount: (moment: {moment:?},
            root: Read(Stat(subject: Source, stat: Attack))), damage_type: True))"#
        );
        let (mut core, actor, target, skill) = observed_fixture(ordered_skill(&effects), vec![]);
        assert_eq!(core.units[&actor].stats.attack, 10);
        let template = release(&mut core, actor, target, &skill.steps[0], 10);
        core.units.get_mut(&actor).unwrap().stats.attack = 20;
        let hp = core.units[&target].stats.current_health;
        consume(&mut core, actor, target, &skill, 0, &template, 10, 20);
        assert_eq!(core.units[&target].stats.current_health, hp - expected);
        let first = query_pair(
            &core,
            "attack_at_consume",
            RuntimeAmount::Damage(expected),
            &reference,
            i128::from(expected),
        );
        core.units.get_mut(&actor).unwrap().stats.attack = 30;
        assert_eq!(
            first,
            query_pair(
                &core,
                "attack_after_mutation",
                RuntimeAmount::Damage(expected),
                &reference,
                i128::from(expected)
            )
        );
    }
}

#[test]
fn d07_f03_c04_c05_target_hp_release_and_consumption_are_frozen_in_record() {
    let reference = ValueRef::HitPoints {
        subject: ValueSubject::Target,
        field: HitPointField::Current,
    };
    for (moment, input, expected) in [(EvalMoment::Snapshot, 80, 20), (EvalMoment::Live, 60, 15)] {
        let effects = format!(
            r#"(id: "amount", op: DamageExpr(amount: (moment: {moment:?},
            root: Ratio(numerator: Read(HitPoints(subject: Target, field: Current)),
                denominator: Const((value: 4, unit: Scalar)), rounding: TowardZero)), damage_type: True))"#
        );
        let (mut core, actor, target, skill) = observed_fixture(ordered_skill(&effects), vec![]);
        assert_eq!(core.units[&target].stats.current_health, 80);
        let template = release(&mut core, actor, target, &skill.steps[0], 10);
        core.units.get_mut(&target).unwrap().stats.current_health = 60;
        consume(&mut core, actor, target, &skill, 0, &template, 10, 20);
        assert_eq!(core.units[&target].stats.current_health, 60 - expected);
        let first = query_pair(
            &core,
            "target_hp_at_consume",
            RuntimeAmount::Damage(expected),
            &reference,
            input,
        );
        core.units.get_mut(&target).unwrap().stats.current_health = 30;
        assert_eq!(
            first,
            query_pair(
                &core,
                "target_hp_after_mutation",
                RuntimeAmount::Damage(expected),
                &reference,
                input
            )
        );
    }
}

fn resource_definition() -> ResourceDefinition {
    ResourceDefinition {
        id: ResourceId::new("meter".into()).unwrap(),
        owner: ResourceOwnerKind::Employee,
        lifetime: ResourceLifetime::Attempt,
        initial: 7,
        cap: 40,
    }
}

fn grant(core: &mut BattleCore, target: UnitInstanceId, amount: u32, time: u64) {
    core.apply_typed_resource_operations(
        &[ResourceOperation {
            resource: ResourceId::new("meter".into()).unwrap(),
            owner: ResourceOwnerSelector::TargetEmployee,
            operation: ResourceMutation::Grant(amount),
        }],
        &[target],
        time,
    )
    .unwrap();
}

const RESOURCE_AMOUNT: &str = r#"Mul(value: Const((value: 1, unit: Magnitude)), scalar: Ratio(
    numerator: Read(Resource(resource: "meter", owner: TargetEmployee, field: Balance)),
    denominator: Const((value: 1, unit: Resource("meter"))), rounding: TowardZero))"#;

#[test]
fn d07_f03_c04_c05_resource_mutations_use_release_or_consumption_balance_in_queries() {
    let reference = ValueRef::Resource {
        resource: ResourceId::new("meter".into()).unwrap(),
        owner: ResourceOwnerSelector::TargetEmployee,
        field: ResourceValueField::Balance,
    };
    for (moment, expected) in [(EvalMoment::Snapshot, 7), (EvalMoment::Live, 11)] {
        let effects = format!(
            r#"(id: "amount", op: DamageExpr(amount: (moment: {moment:?}, root: {RESOURCE_AMOUNT}), damage_type: True))"#
        );
        let (mut core, actor, target, skill) =
            observed_fixture(ordered_skill(&effects), vec![resource_definition()]);
        let template = release(&mut core, actor, target, &skill.steps[0], 10);
        grant(&mut core, target, 4, 15);
        let hp = core.units[&target].stats.current_health;
        consume(&mut core, actor, target, &skill, 0, &template, 10, 20);
        assert_eq!(core.units[&target].stats.current_health, hp - expected);
        let first = query_pair(
            &core,
            "resource_at_consume",
            RuntimeAmount::Damage(expected),
            &reference,
            i128::from(expected),
        );
        grant(&mut core, target, 5, 30);
        assert_eq!(
            first,
            query_pair(
                &core,
                "resource_after_mutation",
                RuntimeAmount::Damage(expected),
                &reference,
                i128::from(expected)
            )
        );
    }
}

#[test]
fn d07_f03_c04_c05_completed_action_results_keep_the_release_frontier_or_live_frontier() {
    let reference = ValueRef::Result {
        selector: ResultSelector::CurrentActionPrefix,
        field: ResultField::DamageHpLost,
    };
    for (moment, expected) in [(EvalMoment::Snapshot, 7), (EvalMoment::Live, 10)] {
        let skills = SkillDatabase::from_ron("fixture/d07_result_replay.ron", &format!(r#"SkillDatabase(skills: [SkillDef(
            id: "wp02_runtime_frame", name: "release", kind: Targeted, focus_time_ms: 0,
            use_lifetime: Action,
            cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: false),
            steps: [
                SkillStepDef(id: "first", delay_ms: 0, delivery: Instant, target: EnemySingle(rule: Nearest),
                    effects: [(id: "damage", op: Damage(amount: 7, damage_type: True))]),
                SkillStepDef(id: "deferred", delay_ms: 0, delivery: Instant, target: EnemySingle(rule: Nearest),
                    effects: [(id: "amount", op: HealExpr(amount: (moment: {moment:?},
                        root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost)))))]),
                SkillStepDef(id: "later", delay_ms: 0, delivery: Instant, target: EnemySingle(rule: Nearest),
                    effects: [(id: "damage", op: Damage(amount: 3, damage_type: True))])
            ])])"#)).unwrap();
        let (mut core, actor, target, skill) = observed_fixture(skills, vec![]);
        let action = core.runtime_fact_source.action.clone().unwrap();
        core.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[0],
            &[target],
            9,
            9,
            None,
            None,
        )
        .unwrap();
        let template = release(&mut core, actor, target, &skill.steps[1], 10);
        core.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[2],
            &[target],
            15,
            15,
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            core.runtime_facts
                .query(&action)
                .unwrap()
                .facts
                .damage_hp_lost,
            10
        );
        let hp = core.units[&target].stats.current_health;
        consume(&mut core, actor, target, &skill, 1, &template, 10, 20);
        assert_eq!(
            core.units[&target].stats.current_health,
            hp + expected as u32
        );
        assert_eq!(
            core.runtime_facts
                .query(&action)
                .unwrap()
                .facts
                .healing_delta,
            i64::from(expected)
        );
        let retained = Arc::clone(
            core.authoring_observation
                .as_ref()
                .unwrap()
                .latest_amount
                .as_ref()
                .unwrap(),
        );
        let first = query_pair(
            &core,
            "completed_result_at_consume",
            RuntimeAmount::Heal(expected),
            &reference,
            i128::from(expected),
        );
        // Mutate the same action's actual completed result without evaluating another amount.
        let later = skill.steps[2].clone();
        // Keep authored action settlement: disable only numerical observation for this later effect.
        core.authoring_observation
            .as_mut()
            .unwrap()
            .amount_observation_enabled = false;
        core.execute_ordered_skill_effects(actor, &skill.id, &later, &[target], 30, 30, None, None)
            .unwrap();
        core.authoring_observation
            .as_mut()
            .unwrap()
            .amount_observation_enabled = true;
        assert_eq!(
            core.runtime_facts
                .query(&action)
                .unwrap()
                .facts
                .damage_hp_lost,
            13
        );
        assert!(Arc::ptr_eq(
            &retained,
            core.authoring_observation
                .as_ref()
                .unwrap()
                .latest_amount
                .as_ref()
                .unwrap()
        ));
        assert_eq!(
            first,
            query_pair(
                &core,
                "completed_result_after_mutation",
                RuntimeAmount::Heal(expected),
                &reference,
                i128::from(expected)
            )
        );
    }
}

#[test]
fn d07_f03_c05_live_reads_prior_effect_hp_resource_and_actual_mitigated_result() {
    let cases = [
        (
            "hp",
            "Read(HitPoints(subject: Target, field: Missing))".to_string(),
            ValueRef::HitPoints {
                subject: ValueSubject::Target,
                field: HitPointField::Missing,
            },
            24,
        ),
        (
            "result",
            "Read(Result(selector: EarlierEffect(\"damage\"), field: DamageHpLost))".to_string(),
            ValueRef::Result {
                selector: ResultSelector::EarlierEffect("damage".into()),
                field: ResultField::DamageHpLost,
            },
            4,
        ),
        (
            "resource",
            RESOURCE_AMOUNT.to_string(),
            ValueRef::Resource {
                resource: ResourceId::new("meter".into()).unwrap(),
                owner: ResourceOwnerSelector::TargetEmployee,
                field: ResourceValueField::Balance,
            },
            11,
        ),
    ];
    for (label, root, reference, expected) in cases {
        let effects = format!(
            r#"
            (id: "damage", op: Damage(amount: 7, damage_type: Physical)),
            (id: "grant", op: Resources(operations: [(resource: "meter", owner: TargetEmployee, operation: Grant(4))])),
            (id: "amount", op: HealExpr(amount: (moment: Live, root: {root}))),
        "#
        );
        let (mut core, actor, target, skill) =
            observed_fixture(ordered_skill(&effects), vec![resource_definition()]);
        core.units.get_mut(&target).unwrap().stats.current_health = 60;
        core.units.get_mut(&target).unwrap().stats.defense = 75;
        let action = core.runtime_fact_source.action.clone().unwrap();
        let template = release(&mut core, actor, target, &skill.steps[0], 10);
        consume(&mut core, actor, target, &skill, 0, &template, 10, 20);
        assert_eq!(
            core.runtime_facts
                .query(&action)
                .unwrap()
                .facts
                .damage_hp_lost,
            4
        );
        assert_eq!(
            core.runtime_facts
                .query(&action)
                .unwrap()
                .facts
                .healing_delta,
            i64::from(expected)
        );
        assert_eq!(
            core.units[&target].stats.current_health,
            56 + expected as u32
        );
        assert_eq!(
            core.authoring_observation
                .as_ref()
                .unwrap()
                .amount_evaluations,
            2
        );
        let first = query_pair(
            &core,
            label,
            RuntimeAmount::Heal(expected),
            &reference,
            i128::from(expected),
        );
        core.units.get_mut(&target).unwrap().stats.current_health = 30;
        grant(&mut core, target, 5, 30);
        assert_eq!(
            first,
            query_pair(
                &core,
                label,
                RuntimeAmount::Heal(expected),
                &reference,
                i128::from(expected)
            )
        );
    }
}
