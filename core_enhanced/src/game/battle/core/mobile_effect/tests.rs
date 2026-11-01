use super::*;
use crate::game::battle::core::{
    boss_runtime::tests::compiled_action_fixture::{fixture_graph, place},
    sim::{BattleExecutionState, BattleLiveCommand},
};
const RELICS: [&str; 5] = [
    "explosion-vulnerability",
    "widened-horizon",
    "summon-vulnerability",
    "long-breath",
    "red-thread",
];
fn fixture(
    bloom: bool,
    relics: &[usize],
    edit: impl FnOnce(&mut serde_json::Value),
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut doc = crate::authoring::node_graph::tests::fusion_placement_fixture(bloom);
    edit(&mut doc);
    let (mut c, s, a, e, b) = fixture_graph(&RELICS, relics, doc);
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    Arc::make_mut(&mut c.scenario).battlefield.object_placement =
        Some(crate::game::battle::scenario::IndependentPlacementDomain {
            ground: (0..8)
                .flat_map(|x| (0..5).map(move |y| Position::new(x, y)))
                .collect(),
            reserved: vec![],
        });
    for id in [a, e, b] {
        let u = c.units.get_mut(&id).unwrap();
        u.threat_class = crate::game::battle::types::BattleUnitThreatClass::Normal;
        u.stats.max_health = 1000;
        u.stats.current_health = 1000;
        u.stats.defense = 0;
        u.stats.magic_resist = 0;
        u.stats.attack = 100;
    }
    place(&mut c, a, 1, 1);
    place(&mut c, b, 1, 3);
    place(&mut c, e, 7, 3);
    (c, s, a, e, b)
}
fn cast(c: &mut BattleCore, a: UnitInstanceId, tile: Position, now: u64) -> Result<(), GameError> {
    c.apply_live_command(BattleLiveCommand::ActivateSkill {
        unit_id: a,
        skill_id: "r11_aura".into(),
        target: Some(SkillCastTarget::Tile { position: tile }),
        time_ms: now,
    })
    .map(|_| ())
}
fn step(c: &mut BattleCore, s: &mut BattleExecutionState, now: u64) {
    c.step_battle_execution_until(s, now).unwrap();
}
#[test]
fn fusion_placement_runtime_normal_bloom_and_no_neighbor_fallback() {
    for bloom in [false, true] {
        let (mut c, mut s, a, _, _) = fixture(bloom, &[], |_| {});
        cast(&mut c, a, Position::new(3, 2), 0).unwrap();
        step(&mut c, &mut s, 0);
        let effects: Vec<_> = c
            .sustained
            .mobile
            .effects
            .values()
            .map(|e| (e.tile, e.units))
            .collect();
        assert_eq!(
            effects,
            [
                (Position::new(3, 2), if bloom { 2 } else { 1 }),
                (Position::new(3, 1), 1)
            ]
        );
        assert!(!c.resonance_use_has_obligations(a));
        let (mut c, mut s, a, _, _) = fixture(bloom, &[], |_| {});
        Arc::make_mut(&mut c.scenario)
            .battlefield
            .object_placement
            .as_mut()
            .unwrap()
            .ground = vec![Position::new(3, 2)];
        cast(&mut c, a, Position::new(3, 2), 0).unwrap();
        step(&mut c, &mut s, 0);
        assert_eq!(c.sustained.mobile.effects.len(), 1);
        assert_eq!(
            c.sustained.mobile.effects.values().next().unwrap().units,
            if bloom { 3 } else { 2 }
        );
    }
}
#[test]
fn fusion_placement_runtime_locked_leap_and_source_exit_preserve_explosion() {
    let (mut c, mut s, a, e, _) = fixture(false, &[], |_| {});
    place(&mut c, e, 4, 2);
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 0);
    let moving = c
        .sustained
        .mobile
        .effects
        .values()
        .find(|e| e.flight.is_some())
        .unwrap();
    assert_eq!(moving.flight.as_ref().unwrap().explodes, 200);
    assert_eq!(
        moving.flight.as_ref().unwrap().destination,
        Position::new(4, 2)
    );
    place(&mut c, e, 5, 2);
    c.withdraw_unit(a, 50).unwrap();
    step(&mut c, &mut s, 200);
    assert_eq!(c.units[&e].stats.current_health, 1000);
    assert_eq!(c.sustained.mobile.effects.len(), 1);
    step(&mut c, &mut s, 5000);
    assert!(c.sustained.mobile.effects.is_empty());
}
#[test]
fn fusion_placement_runtime_explosion_and_lethal_gift_generate_one_seed() {
    for gift in [false, true] {
        let (mut c, mut s, a, e, _) = fixture(false, if gift { &[4] } else { &[] }, |_| {});
        place(&mut c, e, 4, 2);
        c.units.get_mut(&e).unwrap().stats.current_health = 100;
        cast(&mut c, a, Position::new(3, 2), 0).unwrap();
        step(&mut c, &mut s, 200);
        assert!(!c.units[&e].is_active());
        let seeds: Vec<_> = c
            .sustained
            .mobile
            .effects
            .values()
            .filter(|e| e.tile == Position::new(4, 2))
            .collect();
        assert_eq!(seeds.len(), usize::from(gift));
        if gift {
            assert_eq!(seeds[0].units, 1);
        }
        assert!(c.sustained.mobile.seeds.is_empty());
    }
}

fn no_cost(v: &mut serde_json::Value) {
    for n in v["nodes"].as_array_mut().unwrap() {
        if n["kind"] == "skill-resonance-cost" {
            n["values"]["mode"] = serde_json::json!("zero");
        }
    }
}
#[test]
fn fusion_placement_runtime_recast_preserves_attack_birth_count_and_copy_partition() {
    let (mut c, mut s, a, _, b) = fixture(false, &[], no_cost);
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 0);
    c.units.get_mut(&a).unwrap().stats.attack = 500;
    for now in [100, 200] {
        cast(&mut c, a, Position::new(3, 2), now).unwrap();
        step(&mut c, &mut s, now);
    }
    assert_eq!(c.sustained.mobile.effects.len(), 2);
    for e in c.sustained.mobile.effects.values() {
        assert_eq!(e.units, 3);
        assert_eq!(e.created, 0);
        assert_eq!(e.expires, 5000);
        assert_eq!(e.context.template.source_attack, 100);
        assert_eq!(
            e.context.program.tiers[2]
                .profile
                .power
                .resolve(e.context.template.source_attack, e.units)
                .unwrap(),
            300
        );
    }
    cast(&mut c, b, Position::new(3, 2), 300).unwrap();
    step(&mut c, &mut s, 300);
    assert_eq!(c.sustained.mobile.effects.len(), 4);
    let snap = c.mobile_effects_snapshot().unwrap();
    assert_eq!(snap.effects.iter().filter(|e| e.owner == a).count(), 2);
    assert_eq!(snap.effects.iter().filter(|e| e.owner == b).count(), 2);
}
#[test]
fn fusion_placement_runtime_admission_rejects_occupied_missing_ground_and_tile_limit() {
    let (mut c, _, a, e, _) = fixture(false, &[], |_| {});
    let before = c.units[&a].resonance_current;
    assert!(cast(&mut c, a, Position::new(1, 1), 0).is_err());
    let occupied = c.units[&e].body.projected_tile();
    assert!(cast(&mut c, a, occupied, 0).is_err());
    assert_eq!(c.units[&a].resonance_current, before);
    assert!(c.sustained.mobile.effects.is_empty());
    Arc::make_mut(&mut c.scenario).battlefield.object_placement = None;
    assert!(cast(&mut c, a, Position::new(3, 2), 0).is_err());
    assert_eq!(c.units[&a].resonance_current, before);
    let (mut c, _, a, _, _) = fixture(false, &[], |v| {
        for n in v["nodes"].as_array_mut().unwrap() {
            if n["kind"] == "skill-fusion-family" {
                n["values"]["maxTiles"] = serde_json::json!(1);
            }
        }
    });
    assert!(cast(&mut c, a, Position::new(3, 2), 0).is_err());
    assert_eq!(c.units[&a].resonance_current, 100);
    assert!(c.mobile_effects_snapshot().unwrap().effects.is_empty());
}
#[test]
fn fusion_placement_runtime_release_survives_caster_exit_and_retains_public_lineage() {
    let (mut c, mut s, a, e, _) = fixture(false, &[], |_| {});
    place(&mut c, e, 4, 2);
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 0);
    c.withdraw_unit(a, 50).unwrap();
    step(&mut c, &mut s, 200);
    assert_eq!(c.units[&e].stats.current_health, 900);
    assert!(c
        .runtime_fact_summary()
        .latest_damage
        .as_ref()
        .unwrap()
        .source
        .mobile_effect()
        .is_some());
}
#[test]
fn fusion_placement_runtime_vulnerability_is_after_damage_refreshes_and_does_not_stack() {
    let (mut c, mut s, a, e, _) = fixture(false, &[2], no_cost);
    place(&mut c, e, 4, 2);
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 200);
    assert_eq!(c.units[&e].stats.current_health, 900);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 50);
    cast(&mut c, a, Position::new(3, 2), 250).unwrap();
    step(&mut c, &mut s, 450);
    assert_eq!(c.units[&e].stats.current_health, 750);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 50);
    step(&mut c, &mut s, 5450);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 0);
}
#[test]
fn fusion_placement_runtime_death_batch_merges_before_detection() {
    let (mut c, mut s, a, e, b) = fixture(false, &[4], |_| {});
    place(&mut c, e, 4, 2);
    place(&mut c, b, 4, 2);
    c.units.get_mut(&b).unwrap().owner = Side::Opponent;
    for id in [e, b] {
        c.units.get_mut(&id).unwrap().stats.current_health = 100;
    }
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 200);
    assert!(!c.units[&e].is_active());
    assert!(!c.units[&b].is_active());
    let children: Vec<_> = c
        .sustained
        .mobile
        .effects
        .values()
        .filter(|e| e.tile == Position::new(4, 2))
        .collect();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].units, 2);
    assert!(children[0].flight.is_none());
    c.runtime_transaction(|c| c.mobile_target_died(e, 200))
        .unwrap();
    step(&mut c, &mut s, 200);
    assert_eq!(
        c.sustained
            .mobile
            .effects
            .values()
            .find(|e| e.tile == Position::new(4, 2))
            .unwrap()
            .units,
        2
    );
}
#[test]
fn fusion_placement_runtime_long_breath_only_extends_seed_and_expiry_precedes_death() {
    let (mut c, mut s, a, e, _) = fixture(false, &[3, 4], |_| {});
    place(&mut c, e, 4, 2);
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 200);
    let snap = c.mobile_effects_snapshot().unwrap();
    assert_eq!(snap.seeds.len(), 1);
    assert_eq!(snap.seeds[0].expires_at_ms, 12700);
    assert_eq!(snap.effects[0].waiting_expires_at_ms, 5000);
    step(&mut c, &mut s, 12700);
    assert!(c.sustained.mobile.seeds.is_empty());
    c.runtime_transaction(|c| {
        c.finalize_unit_death(Some(a), e, Side::Opponent, 12700);
        control::poll("test_death")
    })
    .unwrap();
    step(&mut c, &mut s, 12700);
    assert!(c.sustained.mobile.effects.is_empty());
}
#[test]
fn fusion_placement_runtime_power_and_range_apply_once_to_concrete_variant() {
    let (mut c, mut s, a, e, b) = fixture(true, &[0, 1], |_| {});
    place(&mut c, e, 4, 2);
    place(&mut c, b, 5, 2);
    c.units.get_mut(&b).unwrap().owner = Side::Opponent;
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 200);
    assert_eq!(c.units[&e].stats.current_health, 700);
    assert_eq!(c.units[&b].stats.current_health, 700);
}

#[test]
fn fusion_placement_runtime_seed_generation_can_seed_next_generation_without_adjacent_clone() {
    let (mut c, mut s, a, e, b) = fixture(false, &[4], |_| {});
    place(&mut c, e, 4, 2);
    place(&mut c, b, 5, 2);
    c.units.get_mut(&b).unwrap().owner = Side::Opponent;
    for id in [e, b] {
        c.units.get_mut(&id).unwrap().stats.current_health = 100;
    }
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 200);
    assert!(!c.units[&e].is_active());
    assert!(c.units[&b].is_active());
    let child = c
        .sustained
        .mobile
        .effects
        .values()
        .find(|e| e.tile == Position::new(4, 2))
        .unwrap();
    assert_eq!(child.units, 1);
    let child_id = child.id;
    let origin = child.seed_origin.as_ref().unwrap().clone();
    assert_ne!(origin.parent_activation, child_id);
    let seed = child.context.program.seed.as_ref().unwrap();
    assert_eq!(
        origin.relic.copy_uuid,
        child.context.relic_copies[&seed.definition]
    );
    assert_eq!(origin.relic.definition, seed.relic_definition);
    assert!(c
        .mobile_effects_snapshot()
        .unwrap()
        .effects
        .iter()
        .any(|e| e.activation == child_id && e.seed_origin.as_ref() == Some(&origin)));
    assert_eq!(
        child.flight.as_ref().unwrap().destination,
        Position::new(5, 2)
    );
    step(&mut c, &mut s, 400);
    assert!(!c.units[&b].is_active());
    let children: Vec<_> = c
        .sustained
        .mobile
        .effects
        .values()
        .filter(|e| e.tile == Position::new(5, 2))
        .collect();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].units, 1);
    assert_eq!(
        children[0].seed_origin.as_ref().unwrap().parent_activation,
        child_id
    );
    assert_eq!(
        children[0].seed_origin.as_ref().unwrap().relic,
        origin.relic
    );
    let facts = c.runtime_fact_summary();
    let damage_origin = facts
        .latest_damage
        .as_ref()
        .unwrap()
        .source
        .mobile_effect()
        .unwrap();
    assert_eq!(damage_origin.activation, child_id);
    assert_eq!(damage_origin.seed_origin.as_ref(), Some(&origin));
    assert_eq!(c.sustained.mobile.effects.len(), 2);
}
#[test]
fn fusion_placement_runtime_cancellation_restores_install_and_death_seed_batch() {
    use crate::authoring::control::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["mobile_detect", "mobile_explosion_damage"] {
        let (mut c, mut s, a, e, _) = fixture(false, &[4], |_| {});
        place(&mut c, e, 4, 2);
        c.units.get_mut(&e).unwrap().stats.current_health = 100;
        cast(&mut c, a, Position::new(3, 2), 0).unwrap();
        let now = if site == "mobile_detect" {
            0
        } else {
            step(&mut c, &mut s, 199);
            200
        };
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            format!("{:?}", c.sustained.mobile),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, now),
            )
        });
        assert!(
            token.is_cancelled() && result.is_err(),
            "{site}: {result:?}"
        );
        assert_eq!(
            before,
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                format!("{:?}", c.sustained.mobile),
                format!("{s:?}")
            )
        );
        step(&mut c, &mut s, now);
        assert_eq!(c.sustained.mobile.effects.len(), 2);
    }
}
#[test]
fn fusion_placement_runtime_retained_exact_boundary_and_below_restore_and_clear() {
    use crate::game::battle::runtime_control as native;
    let (mut original, s, a, _, _) = fixture(false, &[], |_| {});
    cast(&mut original, a, Position::new(3, 2), 0).unwrap();
    let mut probe = controlled_clone(&original).unwrap();
    let mut c = controlled_clone(&original).unwrap();
    let mut probe_s = s.clone();
    let mut s = s;
    let (result, admissions) =
        native::observe_retained_admissions(|| probe.step_battle_execution_until(&mut probe_s, 0));
    result.unwrap();
    let (site, _, boundary) = *admissions.iter().max_by_key(|(_, _, n)| *n).unwrap();
    assert!(boundary > 1);
    c.runtime_facts.limits.max_retained_bytes = boundary - 1;
    let before = (
        c.transaction_debug_snapshot(),
        c.event_queue_debug_snapshot(),
        format!("{:?}", c.sustained.mobile),
        format!("{s:?}"),
    );
    let failure = c.step_battle_execution_until(&mut s, 0).unwrap_err();
    assert!(
        matches!(
            failure,
            GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                code: native::RuntimeFaultCode::RetainedByteLimit,
                ..
            })
        ),
        "{failure:?}"
    );
    assert_eq!(
        before,
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            format!("{:?}", c.sustained.mobile),
            format!("{s:?}")
        )
    );
    c.runtime_facts.limits.max_retained_bytes = boundary;
    step(&mut c, &mut s, 0);
    assert_eq!(c.sustained.mobile.effects.len(), 2);
    c.runtime_transaction(|c| c.clear_mobile_effects()).unwrap();
    assert_eq!(c.sustained.mobile.entity_count(), 0);
    assert_eq!(c.sustained.mobile.retained_extent(), Some(0));
    println!("fusion placement boundary {boundary} at {site}");
}

#[test]
fn fusion_placement_runtime_policy_cadence_count_cap_and_expiry_win_detection() {
    let (mut c, mut s, a, e, _) = fixture(false, &[], |v| {
        no_cost(v);
        for n in v["nodes"].as_array_mut().unwrap() {
            if n["kind"] == "skill-fusion-family" {
                n["values"]["maxUnits"] = serde_json::json!(3);
            }
        }
    });
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 0);
    assert_eq!(c.sustained.mobile.next_tick, Some(c.movement_tick_ms()));
    for time in [1, 2, 3] {
        cast(&mut c, a, Position::new(3, 2), time).unwrap();
        step(&mut c, &mut s, time);
    }
    assert!(c
        .sustained
        .mobile
        .effects
        .values()
        .all(|e| e.units == 3 && e.created == 0 && e.expires == 5000));
    step(&mut c, &mut s, 4999);
    place(&mut c, e, 4, 2);
    step(&mut c, &mut s, 5000);
    assert!(c.sustained.mobile.effects.is_empty());
    assert_eq!(c.units[&e].stats.current_health, 1000);
}

#[test]
fn integration_mobile_delayed_hp_keeps_original_cast_and_fragment() {
    let (mut c, mut s, a, e, _) = fixture(false, &[], |_| {});
    let mut off = controlled_clone(&c).unwrap();
    let mut off_s = s.clone();
    c.enable_authoring_observation();
    let copy = c.units[&a].fragment_source.as_ref().unwrap().copy_uuid;
    place(&mut c, e, 4, 2);
    cast(&mut c, a, Position::new(3, 2), 0).unwrap();
    step(&mut c, &mut s, 0);
    c.withdraw_unit(a, 50).unwrap();
    step(&mut c, &mut s, 200);
    assert_eq!(c.units[&e].stats.current_health, 900);
    let observations = c.authoring_observation().unwrap().unwrap();
    let hp = observations.latest_nonzero_sourced_hp.as_ref().unwrap();
    assert_eq!(hp.target_unit, e);
    assert_eq!(hp.fragment.as_ref().unwrap().copy_uuid, copy);
    assert_eq!(hp.source.as_ref().expect("independent mobile retains cast provenance").cast.skill_id.as_str(), "r11_aura");
    place(&mut off, e, 4, 2);
    cast(&mut off, a, Position::new(3, 2), 0).unwrap();
    step(&mut off, &mut off_s, 0);
    off.withdraw_unit(a, 50).unwrap();
    step(&mut off, &mut off_s, 200);
    assert_eq!(c.units[&e].stats.current_health, off.units[&e].stats.current_health);
    assert_eq!(serde_json::to_value(&c.event_log.entries).unwrap(), serde_json::to_value(&off.event_log.entries).unwrap());
    assert_eq!(serde_json::to_value(c.runtime_fact_snapshot().unwrap()).unwrap(), serde_json::to_value(off.runtime_fact_snapshot().unwrap()).unwrap());
}

#[test]
fn integration_mobile_same_skill_two_copies_interleaved_and_same_time_keep_each_origin() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    use crate::authoring::trace::{TraceAttempt, TraceLimits, TraceOptions};
    for delay in [0, 50] {
        let (base, base_state, _, enemy, _) = fixture(false, &[], |_| {});
        let mut native = None;
        for observe in [false, true] {
            // Clone the same admitted input so HashMap debug iteration order
            // does not become a false observation-mode difference. Expected
            // actor/copy/cast/time/damage values below remain independent.
            let mut c = controlled_clone(&base).unwrap();
            let mut s = base_state.clone();
            // Independent fixture input: player_0/1 use owned IDs 2/3 and
            // fragment copies d200/d201, assigned before BattleCore admission.
            let actor = |owned| c.units.values().find(|u| u.source_owned_uuid == Uuid::from_u128(owned)).unwrap().instance_id;
            let a = actor(2);
            let b = actor(3);
            assert_eq!(c.units[&a].fragment_source.as_ref().unwrap().copy_uuid, Uuid::from_u128(0xd200));
            assert_eq!(c.units[&b].fragment_source.as_ref().unwrap().copy_uuid, Uuid::from_u128(0xd201));
            assert_eq!(c.skill_cast_seq, 0, "fixture has not cast a skill");
            c.units.get_mut(&b).unwrap().stats.attack = 200;
            place(&mut c, enemy, 4, 2);
            let options = TraceOptions { limits: Some(TraceLimits::default()) };
            if observe { c.enable_authoring_observation_with_trace(options); }
            let _trace = observe.then(|| TraceAttempt::enter(options).unwrap());
            cast(&mut c, a, Position::new(3, 2), 0).unwrap();
            step(&mut c, &mut s, 0);
            cast(&mut c, b, Position::new(3, 2), delay).unwrap();
            step(&mut c, &mut s, 100);
            c.withdraw_unit(a, 100).unwrap();
            assert_eq!(c.units[&enemy].stats.current_health, 1000);
            // Public stepping commits complete timestamps. Place the rollback
            // comparison immediately before the failing timestamp, not before
            // earlier successful movement/mobile ticks.
            step(&mut c, &mut s, 199);
            if observe {
                let before = (
                    c.transaction_debug_snapshot(), c.event_queue_debug_snapshot(),
                    serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),
                    c.authoring_trace.view().ndjson.to_vec(), format!("{s:?}"),
                );
                let cancel = CancellationToken::default();
                let result = control::with_test_cancellation("mobile_explosion_damage", 1, ||
                    control::run(&ExecutionLimits::default(), &cancel, ExecutionPhase::TrialExecution,
                        || c.step_battle_execution_until(&mut s, 200)));
                assert!(cancel.is_cancelled() && result.is_err(), "{result:?}");
                assert_eq!(before, (
                    c.transaction_debug_snapshot(), c.event_queue_debug_snapshot(),
                    serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),
                    c.authoring_trace.view().ndjson.to_vec(), format!("{s:?}"),
                ));
            }
            step(&mut c, &mut s, 200 + delay);
            assert_eq!(c.units[&enemy].stats.current_health, 700);
            if observe {
                let trace = c.authoring_trace.view();
                assert!(trace.gap.is_none(), "all applications must be retained");
                let rows: Vec<serde_json::Value> = std::str::from_utf8(trace.ndjson).unwrap()
                    .lines().map(|row| serde_json::from_str(row).unwrap())
                    .filter(|row: &serde_json::Value| row["change"]["channel"] == "hp"
                        && row["target_unit"] == serde_json::json!(enemy)
                        && row["change"]["before"].as_u64().unwrap() > row["change"]["after"].as_u64().unwrap())
                    .collect();
                assert_eq!(rows.len(), 2, "every HP application, not only the latest: {rows:?}");
                for (actor, copy, cast_seq, time, damage) in [
                    (a, 0xd200, 0, 200, 100),
                    (b, 0xd201, 1, 200 + delay, 200),
                ] {
                    let matching: Vec<_> = rows.iter().filter(|row| row["source_unit"] == serde_json::json!(actor)).collect();
                    assert_eq!(matching.len(), 1, "{rows:?}");
                    let row = matching[0];
                    assert_eq!(row["time_ms"], time);
                    assert_eq!(row["change"]["before"].as_u64().unwrap() - row["change"]["after"].as_u64().unwrap(), damage);
                    assert_eq!(row["fragment"]["copy_uuid"], Uuid::from_u128(copy).to_string());
                    assert_eq!(row["fragment"]["definition_id"], "r09_fragment");
                    assert_eq!(row["source"]["caster_instance_id"], serde_json::json!(actor));
                    assert_eq!(row["source"]["cast_seq"], cast_seq);
                    assert_eq!(row["source"]["skill_id"], "r11_aura");
                    assert_eq!(row["source"]["authored"]["source_file"], "r11/effect.ron");
                    assert_eq!(row["source"]["authored"]["field_path"], "skills[1].steps[0].effects[0]");
                    assert!(row["equipment"].is_null(), "fragment effect does not borrow the ambient weapon");
                }
            }
            let outcome = (
                c.transaction_debug_snapshot(), c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_snapshot().unwrap()).unwrap(),
                c.event_log.to_json_string().unwrap(), format!("{s:?}"),
            );
            if let Some(expected) = &native { assert_eq!(expected, &outcome); } else { native = Some(outcome); }
        }
    }
}
