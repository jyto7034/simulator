use super::*;
use crate::game::battle::core::{
    boss_runtime::tests::compiled_action_fixture::{cast, fixture_graph, place},
    sim::BattleExecutionState,
};
use crate::game::{
    battle::damage::{BattleCommand, DamageType},
    enums::Side,
};
fn fixture(
    equipped: &[usize],
    capacity: u32,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut doc = crate::authoring::node_graph::tests::parry_fixture();
    for n in doc["nodes"].as_array_mut().unwrap() {
        if n["kind"] == "skill-guard-charge" {
            n["values"]["capacity"] = serde_json::json!(capacity)
        }
        if n["kind"] == "skill-resonance-cost" {
            n["values"]["mode"] = serde_json::json!("zero")
        }
        if n["kind"] == "skill-resonance-requirement" {
            n["values"]["mode"] = serde_json::json!("none")
        }
    }
    let (mut c, s, a, e, b) = fixture_graph(&["awakened-black-swan"], equipped, doc);
    for id in [a, e, b] {
        let u = c.units.get_mut(&id).unwrap();
        u.stats.attack = 100;
        u.stats.current_health = 1000;
        u.stats.max_health = 1000;
        u.stats.magic_resist = 0;
        u.stats.defense = 0;
    }
    (c, s, a, e, b)
}
fn hit(
    c: &mut BattleCore,
    e: UnitInstanceId,
    a: UnitInstanceId,
    amount: u32,
    source: DamageSource,
    reaction: bool,
    now: u64,
) -> Result<(), GameError> {
    c.runtime_transaction(|c| {
        let mut snapshot = c
            .damage_source_snapshot_for_unit(
                e,
                a,
                source,
                DamageType::True,
                amount,
                Default::default(),
                0,
                now,
                false,
            )
            .unwrap();
        snapshot.fact_source.reaction = reaction;
        c.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id: a,
                source_snapshot: snapshot,
            }],
            now,
        );
        crate::game::battle::runtime_control::poll()
    })
}
fn stored(c: &BattleCore, a: UnitInstanceId) -> u32 {
    c.guard_charge_view(a).unwrap().map_or(0, |v| v.stored)
}
#[test]
fn guard_charge_credits_only_own_actual_reduction_after_other_reductions() {
    for (other, expected, loss) in [(50, 30, 20), (90, 10, 0), (100, 0, 0)] {
        let (mut c, _, a, e, _) = fixture(&[0], 1000);
        c.units
            .get_mut(&a)
            .unwrap()
            .incoming_damage_modifiers
            .damage_reduction_percent = other;
        hit(&mut c, e, a, 100, DamageSource::Ability, false, 0).unwrap();
        assert_eq!(stored(&c, a), expected);
        assert_eq!(c.units[&a].stats.current_health, 1000 - loss);
    }
}
#[test]
fn guard_charge_requires_equipped_copy_caps_once_and_does_not_consume_on_reflection() {
    for copies in [vec![], vec![0], vec![0, 0]] {
        let (mut c, _, a, e, b) = fixture(&copies, 40);
        hit(&mut c, e, a, 100, DamageSource::Ability, false, 0).unwrap();
        hit(&mut c, e, a, 100, DamageSource::Ability, false, 1).unwrap();
        assert_eq!(stored(&c, a), if copies.is_empty() { 0 } else { 40 });
        assert_eq!(c.units[&e].stats.current_health, 930); // two ordinary reflections
        assert_eq!(stored(&c, b), 0);
        let view = c
            .sustained_effect_snapshot()
            .unwrap()
            .passive_guards
            .into_iter()
            .find(|v| v.unit == a)
            .unwrap();
        assert_eq!(view.charge.is_some(), !copies.is_empty());
    }
}
#[test]
fn guard_charge_excludes_environment_reaction_self_cost_and_parry_immunity() {
    let (mut c, mut s, a, e, _) = fixture(&[0], 1000);
    for (source, reaction, attacker) in [
        (DamageSource::Environment, false, e),
        (DamageSource::Ability, true, e),
        (DamageSource::Ability, false, a),
    ] {
        hit(&mut c, attacker, a, 100, source, reaction, 0).unwrap();
        assert_eq!(stored(&c, a), 0);
    }
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    assert!(c
        .sustained
        .records
        .values()
        .any(|r| r.target == a && r.parry.is_some()));
    hit(&mut c, e, a, 100, DamageSource::Ability, false, 1).unwrap();
    hit(&mut c, e, a, 100, DamageSource::Ability, false, 2).unwrap();
    assert_eq!(stored(&c, a), 0);
    c.runtime_transaction(|c| {
        c.process_commands(
            vec![BattleCommand::PayHealthCost {
                caster_id: a,
                amount: 1,
                allow_lethal: false,
            }],
            3,
        );
        crate::game::battle::runtime_control::poll()
    })
    .unwrap();
    assert_eq!(stored(&c, a), 0);
}
#[test]
fn guard_charge_failure_preserves_then_success_consumes_adds_primary_and_splashes_once() {
    let (mut c, mut s, a, e, b) = fixture(&[0], 1000);
    hit(&mut c, e, a, 100, DamageSource::Ability, false, 0).unwrap();
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 31).unwrap();
    assert_eq!(stored(&c, a), 30);
    hit(&mut c, e, a, 100, DamageSource::Ability, false, 32).unwrap();
    assert_eq!(stored(&c, a), 30); // failed guard is disabled during its recovery
    c.step_battle_execution_until(&mut s, 61).unwrap();
    c.units.get_mut(&b).unwrap().owner = Side::Opponent;
    c.units.get_mut(&b).unwrap().fragment_source = None;
    place(&mut c, b, 4, 2);
    cast(&mut c, a, 61);
    c.step_battle_execution_until(&mut s, 62).unwrap();
    assert!(c
        .sustained
        .records
        .values()
        .any(|r| r.target == a && r.parry.is_some()));
    let hp = c.units[&e].stats.current_health;
    hit(&mut c, e, a, 100, DamageSource::Ability, false, 62).unwrap();
    assert_eq!(stored(&c, a), 0);
    assert_eq!(c.units[&e].stats.current_health, hp - 310); // 250 + 30 * 2
    assert_eq!(c.units[&b].stats.current_health, 985); // 30 * 0.5; primary excluded
    let fact = c.runtime_fact_summary().latest_damage.as_ref().unwrap();
    let origin = fact.source.guard_release().unwrap();
    assert_eq!(
        (origin.consumed, origin.added_power, origin.primary),
        (30, 15, false)
    );
    assert!(fact.source.reaction);
    assert_eq!(
        origin.relic_copy,
        c.equipped_normal_relic_copy(a, "awakened-black-swan")
            .unwrap()
    );
    assert_eq!(
        fact.source.fragment.as_ref().unwrap().copy_uuid,
        c.units[&a].fragment_source.as_ref().unwrap().copy_uuid
    );
    // A real released counter's descendant keeps its lineage even when the
    // independent reaction boundary resets `reaction` to false.
    let inherited = fact.source.independent_origin().unwrap();
    assert!(!inherited.reaction);
    c.step_battle_execution_until(&mut s, 600).unwrap();
    let enemy_hp = c.units[&e].stats.current_health;
    let prior = std::mem::replace(&mut c.runtime_fact_source, inherited);
    hit(&mut c, e, a, 100, DamageSource::Ability, false, 600).unwrap();
    c.runtime_fact_source = prior;
    assert_eq!(stored(&c, a), 0);
    assert_eq!(c.units[&e].stats.current_health, enemy_hp);
    assert!(c
        .runtime_fact_summary()
        .latest_damage
        .as_ref()
        .unwrap()
        .source
        .guard_release()
        .is_some());
}
#[test]
fn guard_charge_finite_shield_does_not_steal_guard_credit_and_exit_clears_it() {
    for mode in 0..4 {
        let (mut c, _, a, e, _) = fixture(&[0], 1000);
        c.begin_sustained(
            a,
            a,
            SustainedEffectDef::Shield {
                duration_ms: 1000,
                amount: 100,
                priority: 0,
                shared_radius_units: None,
            },
            0,
        )
        .unwrap();
        hit(&mut c, e, a, 100, DamageSource::Ability, false, 0).unwrap();
        assert_eq!(stored(&c, a), 30);
        assert_eq!(c.units[&a].stats.current_health, 1000);
        match mode {
            0 => {
                c.apply_live_command(
                    crate::game::battle::core::sim::BattleLiveCommand::WithdrawUnit {
                        unit_id: a,
                        time_ms: 1,
                    },
                )
                .unwrap();
            }
            1 => c.units.get_mut(&a).unwrap().corroded = true,
            2 => c.units.get_mut(&a).unwrap().fragment_source = None,
            _ => {
                c.clear_sustained(1).unwrap();
            }
        }
        c.reconcile_sustained(1).unwrap();
        assert!(c.sustained.guard_charges.is_empty());
    }
}
#[test]
fn guard_charge_cancellation_restores_meter_damage_queue_and_retry() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "guard_charge_after_accumulate",
        "guard_charge_after_consume",
        "guard_counter_damage",
    ] {
        let (mut c, mut s, a, e, _) = fixture(&[0], 1000);
        if site != "guard_charge_after_accumulate" {
            hit(&mut c, e, a, 100, DamageSource::Ability, false, 0).unwrap();
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert!(c
                .sustained
                .records
                .values()
                .any(|r| r.target == a && r.parry.is_some()));
        }
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || hit(&mut c, e, a, 100, DamageSource::Ability, false, 1),
            )
        });
        assert!(
            token.is_cancelled() && result.is_err(),
            "{site}: {result:?}"
        );
        assert_eq!(
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                format!("{s:?}")
            ),
            before,
            "{site}"
        );
        hit(&mut c, e, a, 100, DamageSource::Ability, false, 1).unwrap();
        assert_eq!(
            stored(&c, a),
            if site == "guard_charge_after_accumulate" {
                30
            } else {
                0
            }
        );
    }
}

#[test]
fn guard_charge_retained_boundary_is_exact_and_retry_does_not_duplicate_accumulation() {
    use crate::game::battle::runtime_control as native;
    let (original, _, a, e, _) = fixture(&[0], 1000);
    let mut probe = crate::authoring::clone_shape::controlled_clone(&original).unwrap();
    let mut c = crate::authoring::clone_shape::controlled_clone(&original).unwrap();
    let (result, admissions) = native::observe_retained_admissions(|| {
        hit(&mut probe, e, a, 100, DamageSource::Ability, false, 0)
    });
    result.unwrap();
    let (site, _, boundary) = *admissions.iter().max_by_key(|(_, _, n)| *n).unwrap();
    c.runtime_facts.limits.max_retained_bytes = boundary - 1;
    let before = (
        c.transaction_debug_snapshot(),
        c.event_queue_debug_snapshot(),
    );
    assert!(matches!(
        hit(&mut c, e, a, 100, DamageSource::Ability, false, 0),
        Err(GameError::BattleRuntimeFault(native::BattleRuntimeFault {
            code: native::RuntimeFaultCode::RetainedByteLimit,
            ..
        }))
    ));
    assert_eq!(
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot()
        ),
        before
    );
    c.runtime_facts.limits.max_retained_bytes = boundary;
    hit(&mut c, e, a, 100, DamageSource::Ability, false, 0).unwrap();
    assert_eq!(stored(&c, a), 30);
    println!("guard charge retained boundary {boundary} at {site}");
}
