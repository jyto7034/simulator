//! C03: compile each literal as authored and observe the actual command boundary.
use super::*;
use crate::game::battle::core::commands::amount_command_observation;

fn legacy_case(literal: i32, expected: RuntimeAmount, expected_hp: u32) {
    let (op, provenance, expected_value) = match expected {
        RuntimeAmount::Shield(_) => panic!("shield has no legacy Damage/Heal literal"),
        RuntimeAmount::Damage(value) => (
            format!("Damage(amount: {literal}, damage_type: True)"),
            AmountProvenance::LegacyDamageLiteral,
            i128::from(value),
        ),
        RuntimeAmount::Heal(value) => (
            format!("Heal(amount: {literal})"),
            AmountProvenance::LegacyHealLiteral,
            i128::from(value),
        ),
    };
    // RON/public Damage rejects negatives. Use the existing typed authoring boundary,
    // and stage only negative Damage to exercise legacy runtime compatibility.
    let seed_op = match expected {
        RuntimeAmount::Shield(_) => panic!("shield has no legacy Damage/Heal literal"),
        RuntimeAmount::Damage(_) => "Damage(amount: 0, damage_type: True)",
        RuntimeAmount::Heal(_) => "Heal(amount: 0)",
    };
    let mut skills = ordered_skill(&format!("(id: \"literal\", op: {seed_op})"));
    let site = skills.skills[0].steps[0].effects[0]
        .source()
        .unwrap()
        .site
        .clone();
    let changed = skills.set_effect_amount(
        &site,
        crate::game::ability::authoring::EffectProperty::Amount,
        literal,
    );
    let staged = matches!(expected, RuntimeAmount::Damage(_)) && literal < 0;
    if staged {
        assert!(
            changed.is_err(),
            "public negative Damage must stay rejected"
        );
        assert_eq!(
            skills.skills[0].steps[0].effects[0].literal_amount(),
            Some(0)
        );
    } else {
        changed.unwrap();
    }
    let (mut core, actor, target) = core_with_units_and_skills(vec![], Some(skills));
    if staged {
        Arc::get_mut(&mut core.game_data)
            .unwrap()
            .stage_legacy_negative_damage_for_test(literal);
    }
    let skill = core
        .game_data
        .skill_data
        .get_by_id("wp02_runtime_frame")
        .unwrap()
        .clone();
    let entry = core
        .game_data
        .value_expr_catalog()
        .entries()
        .next()
        .unwrap()
        .1
        .clone();
    assert!(!entry.is_authored_expression());
    assert_eq!(entry.plan().provenance(), provenance);
    core.runtime_fact_source = occurrence(&mut core, actor, entry.plan());
    let action = core.runtime_fact_source.action.clone().unwrap();
    core.units.get_mut(&target).unwrap().stats.current_health = 40;
    assert_eq!(core.units[&target].stats.max_health, 80);

    let mut common_gameplay = None;
    // None: no host observer; Some(false): host observer without numerical records.
    for observe in [None, Some(false), Some(true)] {
        let mut candidate = core.clone();
        if let Some(amounts) = observe {
            candidate.enable_authoring_observation_with_options(Default::default(), amounts);
            candidate
                .bind_authoring_observation(
                    super::super::super::authoring_observation::ObservationKey {
                        entry_ordinal: 15,
                        battle_uuid: Uuid::from_u128(0xf0303),
                    },
                )
                .unwrap();
        }
        let (result, inputs) = amount_command_observation::observe(|| {
            candidate.runtime_transaction(|inner| {
                inner.execute_ordered_skill_effects(
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
        });
        let result = result.unwrap();
        let expected_inputs = if expected == RuntimeAmount::Damage(0) {
            vec![]
        } else {
            vec![(target, expected)]
        };
        assert_eq!(inputs, expected_inputs, "{op} observer={observe:?}");
        assert_eq!(result.applied_effect_count, expected_inputs.len());
        assert_eq!(candidate.units[&target].stats.current_health, expected_hp);
        let facts = &candidate.runtime_facts.query(&action).unwrap().facts;
        match expected {
            RuntimeAmount::Shield(_) => panic!("shield has no legacy Damage/Heal literal"),
            RuntimeAmount::Damage(0) => assert!(facts.latest_damage.is_none()),
            RuntimeAmount::Damage(_) => {
                let fact = facts.latest_damage.as_ref().unwrap();
                assert_eq!((fact.hp_before, fact.hp_after), (40, expected_hp));
                assert_eq!(fact.actual_hp_loss(), 40 - expected_hp);
            }
            RuntimeAmount::Heal(value) => {
                let fact = facts.latest_heal.as_ref().unwrap();
                assert_eq!(fact.requested_delta, i64::from(value));
                assert_eq!((fact.hp_before, fact.hp_after), (40, expected_hp));
                assert_eq!(fact.actual_delta(), i64::from(expected_hp) - 40);
            }
        }
        let gameplay = (
            candidate.transaction_debug_snapshot(),
            candidate.event_queue_debug_snapshot(),
            format!("{:?}", candidate.runtime_facts),
            format!("{:?}", candidate.event_log),
            format!("{result:?}"),
        );
        assert_eq!(
            common_gameplay.get_or_insert(gameplay.clone()),
            &gameplay,
            "{op} observer={observe:?}"
        );
        if observe == Some(true) {
            let observer = candidate.authoring_observation.as_ref().unwrap();
            let record = observer.latest_amount.as_ref().unwrap();
            assert_eq!(observer.amount_evaluations, 1);
            assert_eq!(record.evaluation_ordinal, 1);
            assert_eq!(record.amount, expected);
            assert_eq!(record.plan.provenance(), provenance);
            assert_eq!(
                record.evaluated.value,
                TypedInteger::new(expected_value, ValueUnit::Magnitude)
            );
            assert_eq!(record.evaluated.trace.len(), 1);
            assert_eq!(record.evaluated.trace[0].value, record.evaluated.value);
            let replay =
                evaluate_compiled_amount(&record.plan, record.kind(), &record.frame).unwrap();
            assert_eq!(replay.amount, expected);
            assert_eq!(replay.evaluated, record.evaluated);
            // Actual query encoders, not the public selector: C07/C08 remain separate.
            for explain in [false, true] {
                let encode = || {
                    control::run(
                        &ExecutionLimits::default(),
                        &CancellationToken::default(),
                        ExecutionPhase::Inspection,
                        || {
                            crate::authoring::encode_record_for_test(
                                record,
                                crate::authoring::AmountQuery::Exact {
                                    battle_key: record.battle_key.into(),
                                    evaluation_ordinal: 1,
                                },
                                explain,
                            )
                        },
                    )
                    .unwrap()
                };
                let (document, usage) = encode();
                let document = document.unwrap();
                let (repeat, repeat_usage) = encode();
                assert_eq!(document, repeat.unwrap());
                assert_eq!(usage, repeat_usage);
                let parsed: serde_json::Value = serde_json::from_str(&document).unwrap();
                assert_eq!(parsed["status"], "evaluated");
                assert_eq!(
                    parsed["narrowed_base_amount"],
                    serde_json::to_value(expected).unwrap()
                );
                assert_eq!(
                    parsed["trace"],
                    serde_json::to_value(&record.evaluated.trace).unwrap()
                );
            }
            assert_eq!(candidate.transaction_debug_snapshot(), gameplay.0);
            assert_eq!(candidate.event_queue_debug_snapshot(), gameplay.1);
            assert_eq!(format!("{:?}", candidate.runtime_facts), gameplay.2);
        } else if let Some(observer) = candidate.authoring_observation.as_ref() {
            assert!(observer.latest_amount.is_none());
            assert_eq!(observer.amount_evaluations, 0);
        }
        eprintln!("D07_F03_C03 literal={literal} expected={expected:?} observer={observe:?} command_inputs={inputs:?} hp_after={expected_hp} applied_effects={}", result.applied_effect_count);
    }
}

#[test]
fn d07_f03_c03_legacy_damage_boundaries_match_observation_off_and_on() {
    for (literal, magnitude, hp) in [
        (i32::MIN, 2_147_483_648, 0),
        (-7, 7, 33),
        (0, 0, 40),
        (7, 7, 33),
        (i32::MAX, 2_147_483_647, 0),
    ] {
        legacy_case(literal, RuntimeAmount::Damage(magnitude), hp);
    }
}

#[test]
fn d07_f03_c03_legacy_heal_boundaries_preserve_signed_commands_and_clamping() {
    for (literal, hp) in [(i32::MIN, 0), (-7, 33), (0, 40), (7, 47), (i32::MAX, 80)] {
        legacy_case(literal, RuntimeAmount::Heal(literal), hp);
    }
}
