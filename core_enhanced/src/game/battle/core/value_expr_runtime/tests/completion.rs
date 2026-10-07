use super::*;
use crate::authoring::{encode_record_for_test, AmountQuery};
use crate::game::battle::core::authoring_observation::ObservationKey;

fn observed(
    effects: &str,
) -> (
    BattleCore,
    UnitInstanceId,
    UnitInstanceId,
    crate::game::ability::SkillDef,
) {
    let (mut core, actor, target, skill, _) = ordered_fixture(effects);
    core.enable_authoring_observation_with_options(Default::default(), true);
    core.bind_authoring_observation(ObservationKey {
        entry_ordinal: 91,
        battle_uuid: Uuid::from_u128(0xc0814),
    })
    .unwrap();
    (core, actor, target, skill)
}
fn run(
    core: &mut BattleCore,
    actor: UnitInstanceId,
    target: UnitInstanceId,
    skill: &crate::game::ability::SkillDef,
) -> Result<crate::game::battle::core::types::SkillStepResult, GameError> {
    core.runtime_transaction(|c| {
        c.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[0],
            &[target],
            10,
            10,
            None,
            None,
        )
    })
}

#[test]
fn r06_spell_power_only_changes_spell_power_amount_and_preserves_damage_kind() {
    for (stat, expected_pair) in [("Attack", [10_u32, 10]), ("SpellPower", [20, 35])] {
        for (kind, expected_kind) in [
            (
                "Physical",
                crate::game::battle::damage::DamageType::Physical,
            ),
            ("Magic", crate::game::battle::damage::DamageType::Magic),
        ] {
            let effects = format!(
                "(id: \"single_read\", op: DamageExpr(amount: (moment: Live, root: Read(Stat(subject: Source, stat: {stat}))), damage_type: {kind}))"
            );
            let (mut baseline, actor, target, skill) = observed(&effects);
            baseline.units.get_mut(&actor).unwrap().stats.spell_power = 20;
            baseline.units.get_mut(&target).unwrap().stats.defense = 0;
            baseline.units.get_mut(&target).unwrap().stats.magic_resist = 0;
            assert_eq!(baseline.units[&actor].stats.attack, 10);
            assert_eq!(baseline.units[&target].stats.current_health, 80);
            let mut changed = baseline.clone();
            changed.units.get_mut(&actor).unwrap().stats.spell_power = 35;
            let mut restored = changed.units[&actor].clone();
            restored.stats.spell_power = 20;
            assert_eq!(
                format!("{restored:?}"),
                format!("{:?}", baseline.units[&actor])
            );
            assert_eq!(
                format!("{:?}", changed.units[&target]),
                format!("{:?}", baseline.units[&target])
            );
            for (mut core, spell_power, expected) in [
                (baseline, 20, expected_pair[0]),
                (changed, 35, expected_pair[1]),
            ] {
                let action = core.runtime_fact_source.action.clone().unwrap();
                run(&mut core, actor, target, &skill).unwrap();
                let record = core
                    .authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_amount
                    .as_ref()
                    .unwrap();
                assert_eq!(record.amount, RuntimeAmount::Damage(expected));
                assert_eq!(
                    record.evaluated.value,
                    TypedInteger::new(i128::from(expected), ValueUnit::Magnitude)
                );
                assert_eq!(core.units[&target].stats.current_health, 80 - expected);
                let action = core.runtime_facts.query(&action).unwrap();
                let damage = action.facts.latest_damage.as_ref().unwrap();
                assert_eq!(damage.segments.len(), 1);
                assert_eq!(damage.segments[0].damage_type, expected_kind);
                assert_eq!(damage.segments[0].raw_damage, expected);
                assert_eq!((damage.hp_before, damage.hp_after), (80, 80 - expected));
                assert_eq!(core.units[&actor].stats.attack, 10);
                eprintln!("R06_SPELL_ONLY stat={stat} kind={kind} spell_power={spell_power} amount={expected} hp=80->{}", 80 - expected);
            }
        }
    }
}

#[test]
fn r06_hp_max_reads_source_and_target_independently_of_current_and_missing() {
    for (subject, max, current, missing) in [
        (ValueSubject::Source, 100, 70, 30),
        (ValueSubject::Target, 80, 60, 20),
    ] {
        for moment in [EvalMoment::Snapshot, EvalMoment::Live] {
            for (field, expected) in [
                (HitPointField::Max, max),
                (HitPointField::Current, current),
                (HitPointField::Missing, missing),
            ] {
                let p = plan(
                    moment,
                    "hp_direct",
                    ValueExpr::Read(ValueRef::HitPoints { subject, field }),
                );
                let (mut core, actor, target) = core_with_units(vec![]);
                core.units.get_mut(&actor).unwrap().stats.current_health = 70;
                core.units.get_mut(&target).unwrap().stats.current_health = 60;
                assert_eq!(core.units[&actor].stats.max_health, 100);
                assert_eq!(core.units[&target].stats.max_health, 80);
                let source = occurrence(&mut core, actor, &p);
                let capture = core
                    .capture_value_expr_plan(
                        AmountPlanKind::Damage,
                        p,
                        &source,
                        Some(target),
                        10,
                        None,
                    )
                    .unwrap();
                let result = core
                    .evaluate_value_expr_capture(&capture, Some(target), 20)
                    .unwrap();
                assert_eq!(result.amount, RuntimeAmount::Damage(expected));
                assert_eq!(
                    result.evaluated.value,
                    TypedInteger::new(i128::from(expected), ValueUnit::Magnitude)
                );
                eprintln!("R06_HP_DIRECT {subject:?} {moment:?} {field:?} value={expected}");
            }
        }
    }
}

#[test]
fn r06_c08_record_replays_after_source_removal_and_rejects_corrupt_bindings() {
    let (mut core, actor, target, skill) = observed(
        r#"(id:"hit",op:DamageExpr(amount:(moment:Live,root:Const((value:2,unit:Magnitude))),damage_type:True))"#,
    );
    run(&mut core, actor, target, &skill).unwrap();
    let mut record = core
        .authoring_observation
        .as_mut()
        .unwrap()
        .latest_amount
        .take()
        .unwrap();
    let query = AmountQuery::Exact {
        battle_key: record.battle_key.into(),
        evaluation_ordinal: record.evaluation_ordinal,
    };
    let before = encode_record_for_test(&record, query, true).unwrap();
    core.units.remove(&actor);
    core.items.clear();
    core.active_skill_casts.clear();
    assert_eq!(
        before,
        encode_record_for_test(&record, query, true).unwrap()
    );
    let r = Arc::get_mut(&mut record).unwrap();
    let generation = Arc::clone(&r.generation);
    Arc::make_mut(&mut r.generation).semantic_sha256.push('0');
    assert!(crate::authoring::validate_record_generation_for_test(&generation, r).is_err());
    r.generation = generation;
    let original = r.fingerprint;
    r.fingerprint[0] ^= 1;
    assert!(encode_record_for_test(r, query, true).is_err());
    r.fingerprint = original;
    let site = r.site.clone();
    r.site.effect_id.push_str("_wrong");
    assert!(encode_record_for_test(r, query, false).is_err());
    r.site = site;
    let source = Arc::clone(&r.source);
    Arc::make_mut(&mut r.source).equipment = Some(
        crate::game::battle::runtime_facts::EquipmentSourceSnapshot {
            definition_uuid: Uuid::nil(),
            runtime_item_instance_id: Uuid::from_u128(3),
            persistent_copy_uuid: Some(Uuid::from_u128(4)),
        },
    );
    assert!(encode_record_for_test(r, query, true).is_err());
    r.source = source;
    let ordinal = r.evaluation_ordinal;
    r.evaluation_ordinal = 0;
    assert!(encode_record_for_test(r, query, true).is_err());
    r.evaluation_ordinal = ordinal;
    assert_eq!(before, encode_record_for_test(r, query, true).unwrap());
}

#[test]
fn r06_c10_previous_record_survives_later_effect_fault_and_clean_retry() {
    let (mut core, actor, target, skill) = observed(
        r#"
        (id:"hit",op:Damage(amount:2,damage_type:True)),
        (id:"division",op:HealExpr(amount:(moment:Live,root:Mul(value:Const((value:1,unit:Magnitude)),scalar:Ratio(numerator:Const((value:10,unit:Magnitude)),denominator:Read(Stat(subject:Source,stat:SpellPower)),rounding:TowardZero)))))
    "#,
    );
    let mut first = skill.clone();
    first.steps[0].effects.truncate(1);
    run(&mut core, actor, target, &first).unwrap();
    let saved = Arc::clone(
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_amount
            .as_ref()
            .unwrap(),
    );
    let snapshot = (
        core.transaction_debug_snapshot(),
        core.event_queue_debug_snapshot(),
        format!("{:?}", core.runtime_facts),
    );
    assert!(run(&mut core, actor, target, &skill).is_err());
    assert_eq!(
        snapshot,
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{:?}", core.runtime_facts)
        )
    );
    let state = core.authoring_observation.as_ref().unwrap();
    assert_eq!(state.amount_evaluations, 1);
    assert!(Arc::ptr_eq(&saved, state.latest_amount.as_ref().unwrap()));
    core.units.get_mut(&actor).unwrap().stats.spell_power = 10;
    let hp = core.units[&target].stats.current_health;
    run(&mut core, actor, target, &skill).unwrap();
    assert_eq!(core.units[&target].stats.current_health, hp - 1);
    assert_eq!(
        core.authoring_observation
            .as_ref()
            .unwrap()
            .amount_evaluations,
        3
    );
}

#[test]
fn r06_c14_actual_i128_trace_is_exact_in_both_query_documents() {
    let big = (1_i128 << 100) + 1;
    let effects = r#"(id:"big",op:DamageExpr(amount:(moment:Live,root:Add([Add([Mul(value:Const((value:1125899906842624,unit:Magnitude)),scalar:Mul(value:Const((value:1125899906842624,unit:Scalar)),scalar:Ratio(numerator:Read(Stat(subject:Source,stat:Attack)),denominator:Read(Stat(subject:Source,stat:Attack)),rounding:TowardZero))),Const((value:1,unit:Magnitude))]),Mul(value:Const((value:1125899906842624,unit:Magnitude)),scalar:Const((value:-1125899906842624,unit:Scalar)))])),damage_type:True))"#;
    let (mut core, actor, target, skill) = observed(&effects);
    run(&mut core, actor, target, &skill).unwrap();
    let record = core
        .authoring_observation
        .as_ref()
        .unwrap()
        .latest_amount
        .as_ref()
        .unwrap();
    assert_eq!(record.amount, RuntimeAmount::Damage(1));
    assert!(record
        .evaluated
        .trace
        .iter()
        .any(|node| node.value.value == big));
    for explain in [false, true] {
        let text = encode_record_for_test(
            record,
            AmountQuery::Exact {
                battle_key: record.battle_key.into(),
                evaluation_ordinal: 1,
            },
            explain,
        )
        .unwrap();
        assert!(text.contains("1267650600228229401496703205377"));
        assert!(
            text.find("1267650600228229401496703205377").unwrap()
                < text.find("-1267650600228229401496703205376").unwrap()
        );
        eprintln!("R06_C14 {text}");
    }
}

#[test]
fn r06_spell_power_and_points_snapshot_live_frames_use_original_inputs() {
    for stat in [StatSelector::SpellPower, StatSelector::AttackSpeedPoints] {
        for moment in [EvalMoment::Snapshot, EvalMoment::Live] {
            let p = plan(
                moment,
                "stat",
                ValueExpr::Read(ValueRef::Stat {
                    subject: ValueSubject::Source,
                    stat,
                }),
            );
            let (mut core, actor, _) = core_with_units(vec![]);
            core.units.get_mut(&actor).unwrap().stats.spell_power = 20;
            core.units
                .get_mut(&actor)
                .unwrap()
                .basic_attack_speed_percent = 150;
            let source = occurrence(&mut core, actor, &p);
            let capture = core
                .capture_value_expr_plan(
                    AmountPlanKind::Damage,
                    Arc::clone(&p),
                    &source,
                    None,
                    10,
                    None,
                )
                .unwrap();
            core.units.get_mut(&actor).unwrap().stats.spell_power = 35;
            core.units
                .get_mut(&actor)
                .unwrap()
                .basic_attack_speed_percent = 200;
            // Points intentionally cannot narrow to Damage; inspect the actual prepared frame instead.
            let frame = if moment == EvalMoment::Snapshot {
                capture.snapshot_frame.as_ref().unwrap().as_ref().clone()
            } else {
                core.build_value_expr_frame_with_prefix(&p, &source, None, 20, None, None, 0)
                    .unwrap()
            };
            let expected = match (stat, moment) {
                (StatSelector::SpellPower, EvalMoment::Snapshot) => 20,
                (StatSelector::SpellPower, _) => 35,
                (_, EvalMoment::Snapshot) => 150,
                _ => 200,
            };
            assert_eq!(raw(&p, &frame).value, expected);
            assert_eq!(core.units[&actor].stats.attack, 10);
        }
    }
}

#[test]
fn r06_amount_record_is_independent_of_event_log_off_and_real_eviction() {
    use crate::game::battle::event_log::{BattleLogEvent, BATTLE_HISTORY_MAX_EVENTS};
    let effects = r#"(id:"hit",op:DamageExpr(amount:(moment:Live,root:Add([Read(Stat(subject:Source,stat:Attack)),Const((value:1,unit:Magnitude))])),damage_type:True))"#;
    let mut baseline = None;
    for mode in 0..3 {
        let (mut core, actor, target, skill) = observed(effects);
        if mode == 1 {
            core.event_log.close_for_sequence_exhaustion();
        }
        run(&mut core, actor, target, &skill).unwrap();
        let record = Arc::clone(
            core.authoring_observation
                .as_ref()
                .unwrap()
                .latest_amount
                .as_ref()
                .unwrap(),
        );
        let query = AmountQuery::Exact {
            battle_key: record.battle_key.into(),
            evaluation_ordinal: 1,
        };
        let before = encode_record_for_test(&record, query, true).unwrap();
        if mode == 2 {
            let latest = core.event_log.latest_seq();
            for _ in 0..=BATTLE_HISTORY_MAX_EVENTS {
                core.record_event_log(
                    20,
                    BattleLogEvent::BattleStart {
                        width: 4,
                        height: 4,
                    },
                );
            }
            assert!(core.event_log.floor_seq() > latest);
        }
        assert_eq!(
            before,
            encode_record_for_test(&record, query, true).unwrap()
        );
        if let Some(previous) = &baseline {
            assert_eq!(&before, previous);
        } else {
            baseline = Some(before);
        }
        assert_eq!(
            core.authoring_observation
                .as_ref()
                .unwrap()
                .amount_evaluations,
            1
        );
    }
}

#[test]
fn r06_actual_later_trace_cancel_preserves_record_and_native_transaction_then_retry() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let effects = r#"(id:"hit",op:DamageExpr(amount:(moment:Live,root:Add([Read(Stat(subject:Source,stat:Attack)),Const((value:1,unit:Magnitude))])),damage_type:True))"#;
    let (mut core, actor, target, skill) = observed(effects);
    run(&mut core, actor, target, &skill).unwrap();
    let saved = Arc::clone(
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_amount
            .as_ref()
            .unwrap(),
    );
    let before = (
        core.transaction_debug_snapshot(),
        core.event_queue_debug_snapshot(),
        format!("{:?}", core.runtime_facts),
    );
    let token = CancellationToken::default();
    let result = control::with_test_cancellation("value_expr_trace", 2, || {
        control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialExecution,
            || run(&mut core, actor, target, &skill),
        )
    });
    assert!(token.is_cancelled());
    assert!(result.is_err());
    assert_eq!(
        before,
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{:?}", core.runtime_facts)
        )
    );
    assert!(Arc::ptr_eq(
        &saved,
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_amount
            .as_ref()
            .unwrap()
    ));
    assert_eq!(
        core.authoring_observation
            .as_ref()
            .unwrap()
            .amount_evaluations,
        1
    );
    let hp = core.units[&target].stats.current_health;
    run(&mut core, actor, target, &skill).unwrap();
    assert_eq!(core.units[&target].stats.current_health, hp - 11);
    assert_eq!(
        core.authoring_observation
            .as_ref()
            .unwrap()
            .amount_evaluations,
        2
    );
}

#[test]
fn r06_record_replacement_peak_is_inclusive_and_failure_preserves_previous_owner() {
    use crate::game::battle::runtime_control::{self as native, RuntimeFactLimits};
    let effects = r#"(id:"hit",op:DamageExpr(amount:(moment:Live,root:Const((value:1,unit:Magnitude))),damage_type:True))"#;
    let (mut core, actor, target, skill) = observed(effects);
    run(&mut core, actor, target, &skill).unwrap();
    let prior = Arc::clone(
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_amount
            .as_ref()
            .unwrap(),
    );
    let mut measured = core.clone();
    let (result, admissions) =
        native::observe_retained_admissions(|| run(&mut measured, actor, target, &skill));
    result.unwrap();
    let (_, old, peak) = admissions
        .iter()
        .find(|(site, _, _)| *site == "recorded_amount_replacement_peak")
        .copied()
        .unwrap();
    assert!(peak - old >= prior.retained_extent() * 2);
    let required = admissions.last().unwrap().2;
    let before = core.transaction_debug_snapshot();
    assert!(native::run(
        RuntimeFactLimits {
            max_retained_bytes: peak - 1,
            ..Default::default()
        },
        || run(&mut core, actor, target, &skill)
    )
    .is_err());
    assert_eq!(before, core.transaction_debug_snapshot());
    assert!(Arc::ptr_eq(
        &prior,
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_amount
            .as_ref()
            .unwrap()
    ));
    native::run(
        RuntimeFactLimits {
            max_retained_bytes: required,
            ..Default::default()
        },
        || run(&mut core, actor, target, &skill),
    )
    .unwrap();
    assert_eq!(
        core.authoring_observation
            .as_ref()
            .unwrap()
            .amount_evaluations,
        2
    );
    assert_eq!(
        core.units[&target].stats.current_health,
        measured.units[&target].stats.current_health
    );
}
