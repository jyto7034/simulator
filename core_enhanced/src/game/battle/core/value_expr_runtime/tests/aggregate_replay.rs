//! C06: independent aggregate views through actual consumption and recorded replay.
use super::*;
use crate::game::battle::{
    core::{commands::amount_command_observation, types::SkillStepResult},
    damage::{DamageSource, DamageSourceSnapshot, DamageType},
};
use crate::game::value_expr::TraceEntry;

fn fixture(effects: &str) -> (OwnerFixture, crate::game::ability::SkillDef) {
    let mut fixture = owner_fixture(Some(ordered_skill(effects)));
    let core = &mut fixture.core;
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
            .find(|(_, e)| e.is_authored_expression())
            .unwrap()
            .1
            .plan(),
    );
    core.runtime_fact_source = occurrence(core, fixture.employee_a, &plan);
    core.enable_authoring_observation_with_options(Default::default(), true);
    core.bind_authoring_observation(super::super::super::authoring_observation::ObservationKey {
        entry_ordinal: 16,
        battle_uuid: Uuid::from_u128(0xf0306),
    })
    .unwrap();
    (fixture, skill)
}

fn release(
    f: &mut OwnerFixture,
    step: &crate::game::ability::SkillStepDef,
    targets: &[UnitInstanceId],
) -> DamageSourceSnapshot {
    let mut source = f.core.runtime_fact_source.clone();
    source.allows_finite_children = true;
    let source = f.core.reserve_finite_source(source).unwrap();
    let carrier = f
        .core
        .build_value_expr_delivery_capture(step, &source, Some(targets), 10, None)
        .unwrap()
        .unwrap();
    let mut template = f
        .core
        .damage_source_snapshot_for_unit(
            f.employee_a,
            targets[0],
            DamageSource::Ability,
            DamageType::True,
            0,
            Default::default(),
            0,
            10,
            false,
        )
        .unwrap();
    template.fact_source = source;
    template.value_expr_capture = Some(carrier);
    template
}

fn consume(
    f: &mut OwnerFixture,
    skill: &crate::game::ability::SkillDef,
    targets: &[UnitInstanceId],
    template: &DamageSourceSnapshot,
) -> Result<SkillStepResult, GameError> {
    let actor = f.employee_a;
    f.core.runtime_transaction(|core| {
        let prior = std::mem::replace(&mut core.runtime_fact_source, template.fact_source.clone());
        let result = core.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[0],
            targets,
            20,
            10,
            Some(template),
            None,
        );
        core.runtime_fact_source = prior;
        result
    })
}

fn queries(core: &BattleCore, label: &str, expected: u32) -> (String, String) {
    let observer = core.authoring_observation.as_ref().unwrap();
    let record = observer.latest_amount.as_ref().unwrap();
    assert_eq!(record.amount, RuntimeAmount::Damage(expected));
    assert_eq!(
        record.evaluated.value,
        TypedInteger::new(i128::from(expected), ValueUnit::Magnitude)
    );
    let before = (
        core.transaction_debug_snapshot(),
        core.event_queue_debug_snapshot(),
        format!("{:?}", core.runtime_facts),
        format!("{:?}", core.event_log),
    );
    let encode = |explain| {
        let (document, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::Inspection,
            || {
                crate::authoring::encode_record_for_test(
                    record,
                    crate::authoring::AmountQuery::Exact {
                        battle_key: record.battle_key.into(),
                        evaluation_ordinal: record.evaluation_ordinal,
                    },
                    explain,
                )
            },
        )
        .unwrap();
        (document.unwrap(), usage)
    };
    let preview = encode(false);
    let explain = encode(true);
    assert_eq!(preview, encode(false));
    assert_eq!(explain, encode(true));
    for (document, kind) in [
        (&preview.0, "amount_preview"),
        (&explain.0, "amount_explanation"),
    ] {
        let value: serde_json::Value = serde_json::from_str(document).unwrap();
        assert_eq!(value["kind"], kind);
        assert_eq!(value["status"], "evaluated");
        assert_eq!(
            value["narrowed_base_amount"],
            serde_json::to_value(record.amount).unwrap()
        );
        assert_eq!(
            value["trace"],
            serde_json::to_value(&record.evaluated.trace).unwrap()
        );
        assert_eq!(
            value["identity"]["evaluation_ordinal"],
            record.evaluation_ordinal
        );
        assert_eq!(value["release_time_ms"], record.release_time_ms);
        assert_eq!(value["consume_time_ms"], record.consume_time_ms);
    }
    assert_eq!(
        before,
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{:?}", core.runtime_facts),
            format!("{:?}", core.event_log)
        )
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
    assert_eq!(observer.amount_evaluations, record.evaluation_ordinal);
    eprintln!(
        "C06_REPLAY {}",
        serde_json::json!({"label": label, "expected": expected, "moment": record.moment, "preview": preview.0, "explain": explain.0})
    );
    (preview.0, explain.0)
}

fn aggregate(source: &str, owner: &str, reduction: &str) -> String {
    format!("Aggregate(source: {source}, grouping: ByResolvedOwner({owner}), reduction: {reduction}, outer_reduction: Some(Sum), member: Read(Stat(subject: AggregateMember, stat: Attack)))")
}

fn slots(
    f: &mut OwnerFixture,
    unit: UnitInstanceId,
) -> &mut super::super::super::types::BattleEquipmentSlots {
    Arc::make_mut(
        f.core
            .units
            .get_mut(&unit)
            .unwrap()
            .equipment_slots
            .as_mut()
            .unwrap(),
    )
}

fn set_attacks(f: &mut OwnerFixture, values: [u32; 4]) {
    for (unit, attack) in [f.employee_a, f.employee_b, f.summon, f.enemy]
        .into_iter()
        .zip(values)
    {
        f.core.units.get_mut(&unit).unwrap().stats.attack = attack;
    }
}

type Member = (UnitInstanceId, GroupOwnerDomain, String, i128);

fn assert_combined_frame(f: &OwnerFixture, live: bool) {
    let record = f
        .core
        .authoring_observation
        .as_ref()
        .unwrap()
        .latest_amount
        .as_ref()
        .unwrap();
    let [a, b, summon, enemy] = if live { [4, 9, 13, 6] } else { [3, 5, 11, 7] };
    let employee_id = |n| Uuid::from_u128(n).hyphenated().to_string();
    let mut weapon: Vec<Member> = vec![(
        f.employee_a,
        GroupOwnerDomain::EquipmentCopy,
        f.weapon_x.to_string(),
        a,
    )];
    if !live {
        weapon.push((
            f.employee_b,
            GroupOwnerDomain::EquipmentCopy,
            f.weapon_z.to_string(),
            b,
        ));
    }
    let rows = vec![
        vec![
            (f.employee_a, GroupOwnerDomain::Side, "player".into(), a),
            (f.employee_b, GroupOwnerDomain::Side, "player".into(), b),
            (f.summon, GroupOwnerDomain::Side, "player".into(), summon),
            (f.enemy, GroupOwnerDomain::Side, "opponent".into(), enemy),
        ],
        vec![
            (
                f.employee_a,
                GroupOwnerDomain::Employee,
                employee_id(0x5001),
                a,
            ),
            (
                f.employee_b,
                GroupOwnerDomain::Employee,
                employee_id(0x5002),
                b,
            ),
        ],
        weapon,
        vec![(
            f.employee_a,
            GroupOwnerDomain::EquipmentCopy,
            f.armor_y.to_string(),
            a,
        )],
    ];
    let values = if live { [19, 13, 4, 4] } else { [18, 8, 8, 3] };
    let reference = ValueRef::Stat {
        subject: ValueSubject::AggregateMember,
        stat: StatSelector::Attack,
    };
    let mut trace = Vec::new();
    // Canonical predicate nodes consume IDs even though they emit no value trace.
    for (((node, read_node), mut expected), value) in [(1, 2), (3, 5), (6, 9), (10, 13)]
        .into_iter()
        .zip(rows)
        .zip(values)
    {
        expected.sort_by_key(|row| row.0.as_uuid().to_string());
        let members = record.frame.resolved_target_view(node).unwrap();
        assert_eq!(members.len(), expected.len());
        for (member, (unit, domain, owner, attack)) in members.iter().zip(expected) {
            assert_eq!(member.identity, unit.as_uuid().to_string());
            assert_eq!(
                member.owner,
                Some(crate::game::value_expr::GroupOwner {
                    domain,
                    identity: owner
                })
            );
            assert_eq!(member.value, None);
            let value = TypedInteger::new(attack, ValueUnit::Magnitude);
            assert_eq!(
                member.values,
                BTreeMap::from([(reference.clone(), value.clone())])
            );
            trace.push(TraceEntry {
                node_index: read_node,
                member_identity: Some(unit.as_uuid().to_string()),
                value,
            });
        }
        trace.push(TraceEntry {
            node_index: node,
            member_identity: None,
            value: TypedInteger::new(value, ValueUnit::Magnitude),
        });
    }
    trace.push(TraceEntry {
        node_index: 0,
        member_identity: None,
        value: TypedInteger::new(if live { 40 } else { 37 }, ValueUnit::Magnitude),
    });
    assert_eq!(record.evaluated.trace, trace);
    assert!(record.frame.resolved_target_view(0).is_none());
    assert!(record.frame.resolved_target_view(9).is_none());
}

#[test]
fn d07_f03_c06_four_owner_views_preserve_snapshot_live_and_recorded_node_member_trace() {
    for any3 in [false, true] {
        let (weapon, armor) = if any3 {
            ("Any3(0)", "Any3(1)")
        } else {
            ("Weapon", "Armor")
        };
        let parts = [
            aggregate("ResolvedTargets", "Side", "Max"),
            aggregate(
                "ResolvedTargetsWhere([IsActualEmployee])",
                "ActualEmployee",
                "Max",
            ),
            aggregate(
                &format!("ResolvedTargetsWhere([IsActualEmployee, HasEquippedSlot({weapon})])"),
                &format!("EquippedSlot({weapon})"),
                "Max",
            ),
            aggregate(
                &format!("ResolvedTargetsWhere([IsActualEmployee, HasEquippedSlot({armor})])"),
                &format!("EquippedSlot({armor})"),
                "Max",
            ),
        ];
        for moment in [EvalMoment::Snapshot, EvalMoment::Live] {
            let effects = format!(
                r#"(id: "amount", op: DamageExpr(amount: (moment: {moment:?}, root: Add([{}])), damage_type: True))"#,
                parts.join(",")
            );
            let (mut f, skill) = fixture(&effects);
            if any3 {
                for unit in [f.employee_a, f.employee_b] {
                    let slots = slots(&mut f, unit);
                    slots.layout = SlotLayoutKind::Any3;
                    slots.slots[0].as_mut().unwrap().binding.slot =
                        Some(EquipmentSlotAddress::Any3(0));
                    if let Some(armor) = slots.slots[1].as_mut() {
                        armor.binding.slot = Some(EquipmentSlotAddress::Any3(1));
                    }
                }
            }
            // Deliberately different from canonical member order; aggregate filtering does not retarget damage.
            let targets = [f.enemy, f.employee_b, f.summon, f.employee_a];
            let template = release(&mut f, &skill.steps[0], &targets);
            set_attacks(&mut f, [4, 9, 13, 6]);
            let b = f.employee_b;
            slots(&mut f, b).slots[0] = None;
            let expected = if moment == EvalMoment::Snapshot {
                37
            } else {
                40
            };
            let (result, inputs) = amount_command_observation::observe(|| {
                consume(&mut f, &skill, &targets, &template)
            });
            let result = result.unwrap();
            assert_eq!(result.resolved_target_count, 4);
            assert_eq!(
                inputs,
                targets
                    .map(|unit| (unit, RuntimeAmount::Damage(expected)))
                    .to_vec()
            );
            for unit in targets {
                assert_eq!(f.core.units[&unit].stats.current_health, 100 - expected);
            }
            let observer = f.core.authoring_observation.as_ref().unwrap();
            assert_eq!(observer.amount_evaluations, 4);
            let record = observer.latest_amount.as_ref().unwrap();
            assert_eq!((record.release_time_ms, record.consume_time_ms), (10, 20));
            assert_eq!(record.consumption_target, Some(f.employee_a));
            assert_eq!(
                record.source.delivery.as_ref().unwrap().ordinal,
                template.fact_source.delivery.as_ref().unwrap().ordinal
            );
            let capture = template
                .value_expr_capture
                .as_ref()
                .unwrap()
                .capture_for(0, Some(f.employee_a))
                .unwrap();
            if moment == EvalMoment::Snapshot {
                assert!(Arc::ptr_eq(
                    &record.frame,
                    capture.snapshot_frame.as_ref().unwrap()
                ));
            } else {
                assert!(capture.snapshot_frame.is_none());
            }
            assert_combined_frame(&f, moment == EvalMoment::Live);
            let label = format!("combined_{any3}_{moment:?}");
            let first = queries(&f.core, &label, expected);
            set_attacks(&mut f, [40, 90, 130, 60]);
            let a = f.employee_a;
            slots(&mut f, a).slots[0] = None;
            slots(&mut f, a).slots[1] = None;
            assert_combined_frame(&f, moment == EvalMoment::Live);
            assert_eq!(
                first,
                queries(&f.core, &format!("{label}_after_mutation"), expected)
            );
        }
    }
}

#[test]
fn d07_f03_c06_filtered_empty_owner_view_records_zero_without_damage_commands() {
    let root = aggregate(
        "ResolvedTargetsWhere([IsActualEmployee, HasEquippedSlot(Accessory)])",
        "EquippedSlot(Accessory)",
        "Sum",
    );
    for moment in [EvalMoment::Snapshot, EvalMoment::Live] {
        let effects = format!(
            r#"(id: "empty", op: DamageExpr(amount: (moment: {moment:?}, root: {root}), damage_type: True))"#
        );
        let (mut f, skill) = fixture(&effects);
        let targets = [f.enemy, f.employee_b, f.summon, f.employee_a];
        let template = release(&mut f, &skill.steps[0], &targets);
        let (result, inputs) =
            amount_command_observation::observe(|| consume(&mut f, &skill, &targets, &template));
        assert_eq!(result.unwrap().resolved_target_count, 4);
        assert!(inputs.is_empty());
        for target in targets {
            assert_eq!(f.core.units[&target].stats.current_health, 100);
        }
        let observer = f.core.authoring_observation.as_ref().unwrap();
        assert_eq!(observer.amount_evaluations, 4);
        let record = observer.latest_amount.as_ref().unwrap();
        assert!(record.frame.resolved_target_view(0).unwrap().is_empty());
        assert_eq!(
            record.evaluated.trace,
            vec![TraceEntry {
                node_index: 0,
                member_identity: None,
                value: TypedInteger::new(0, ValueUnit::Magnitude)
            }]
        );
        queries(&f.core, &format!("empty_{moment:?}"), 0);
    }
}

#[test]
fn d07_f03_c06_ineligible_relations_refuse_and_preserve_prior_record_and_gameplay() {
    for case in ["employee", "empty_slot", "no_layout", "stale_filtered_copy"] {
        let (source, owner) = match case {
            "employee" => ("ResolvedTargets", "ActualEmployee"),
            "empty_slot" => ("ResolvedTargets", "EquippedSlot(Armor)"),
            "no_layout" => ("ResolvedTargets", "EquippedSlot(Weapon)"),
            _ => (
                "ResolvedTargetsWhere([IsActualEmployee, HasEquippedSlot(Weapon)])",
                "EquippedSlot(Weapon)",
            ),
        };
        let root = aggregate(source, owner, "Sum");
        let effects = format!(
            r#"(id: "prior", op: Damage(amount: 1, damage_type: True)),
            (id: "bad", op: DamageExpr(amount: (moment: Live, root: {root}), damage_type: True))"#
        );
        let (mut f, skill) = fixture(&effects);
        let mut seed = skill.steps[0].clone();
        seed.effects.truncate(1);
        f.core
            .execute_ordered_skill_effects(
                f.employee_a,
                &skill.id,
                &seed,
                &[f.employee_a],
                9,
                9,
                None,
                None,
            )
            .unwrap();
        let targets = match case {
            "employee" => vec![f.employee_a, f.summon],
            "empty_slot" => vec![f.employee_b],
            "no_layout" => vec![f.summon],
            _ => vec![f.employee_a],
        };
        let template = release(&mut f, &skill.steps[0], &targets);
        if case == "stale_filtered_copy" {
            let a = f.employee_a;
            slots(&mut f, a).slots[0]
                .as_mut()
                .unwrap()
                .runtime_item_instance_id = Uuid::nil();
        }
        let before = (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.core.runtime_facts),
            format!("{:?}", f.core.event_log),
        );
        let retained = Arc::clone(
            f.core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_amount
                .as_ref()
                .unwrap(),
        );
        let prior_queries = queries(&f.core, &format!("{case}_before"), 1);
        let (result, commands) =
            amount_command_observation::observe(|| consume(&mut f, &skill, &targets, &template));
        // The first effect actually ran in the candidate before the relation refused.
        assert_eq!(
            commands,
            targets
                .iter()
                .map(|id| (*id, RuntimeAmount::Damage(1)))
                .collect::<Vec<_>>()
        );
        let error = result.unwrap_err();
        let expected = if case == "no_layout" {
            ValueExprFaultCode::MissingContext
        } else {
            ValueExprFaultCode::InvalidOwner
        };
        assert!(
            matches!(&error, GameError::ValueExpressionFault(fault) if fault.code == expected),
            "{case}: {error:?}"
        );
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.core.runtime_facts),
                format!("{:?}", f.core.event_log)
            )
        );
        assert!(Arc::ptr_eq(
            &retained,
            f.core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_amount
                .as_ref()
                .unwrap()
        ));
        assert_eq!(
            f.core
                .authoring_observation
                .as_ref()
                .unwrap()
                .amount_evaluations,
            1
        );
        assert_eq!(prior_queries, queries(&f.core, &format!("{case}_after"), 1));
        eprintln!("C06_REFUSAL case={case} code={expected:?} prior_record_preserved=true");
    }
}
