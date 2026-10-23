use super::*;
use crate::game::resources::Position;
use crate::game::battle::core::{
    boss_runtime::tests::compiled_action_fixture::{cast, fixture_graph, hit, place},
    sim::BattleExecutionState,
};
const RELICS: [&str; 6] = [
    "hastened-pulse",
    "growth-medium",
    "sealed-culture",
    "widened-horizon",
    "explosion-vulnerability",
    "overlapping-explosions",
];
fn fixture(
    mods: &[usize],
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let (mut c, s, a, e, b) = fixture_graph(
        &RELICS,
        mods,
        crate::authoring::node_graph::tests::presence_fixture(),
    );
    place(&mut c, e, 3, 1);
    c.units.get_mut(&e).unwrap().stats.current_health = 10000;
    c.units.get_mut(&e).unwrap().stats.max_health = 10000;
    (c, s, a, e, b)
}
fn key(c: &BattleCore, a: UnitInstanceId) -> uuid::Uuid {
    c.units[&a].fragment_source.as_ref().unwrap().copy_uuid
}
fn hosts(c: &BattleCore, a: UnitInstanceId) -> usize {
    c.sustained
        .presence
        .networks
        .get(&key(c, a))
        .map_or(0, |r| r.hosts.len())
}
fn power(c: &BattleCore, a: UnitInstanceId, now: u64) -> u32 {
    let r = &c.sustained.presence.networks[&key(c, a)];
    r.power
        .amount(
            r.power
                .value(&r.capture.program.marks.power, r.hosts.len(), now)
                .unwrap(),
            MarkRatio {
                numerator: 1,
                denominator: 1,
            },
        )
        .unwrap()
}
fn strike(
    c: &mut BattleCore,
    a: UnitInstanceId,
    e: UnitInstanceId,
    now: u64,
    amount: u32,
) -> DamageSourceSnapshot {
    c.runtime_transaction(|c| {
        let action = c.runtime_facts.accept(
            a,
            None,
            RuntimeFactSource {
                actor: Some(a),
                ..Default::default()
            },
            now,
            false,
        )?;
        let old = std::mem::replace(
            &mut c.runtime_fact_source,
            RuntimeFactSource {
                actor: Some(a),
                action: Some(action),
                ..Default::default()
            },
        );
        let mut snap = c
            .damage_source_snapshot_for_unit(
                a,
                e,
                DamageSource::BasicAttack,
                crate::game::battle::damage::DamageType::True,
                amount,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        c.bind_presence(&mut snap)?;
        c.runtime_fact_source = old;
        c.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
            target_id: e,
            source_snapshot: snap.clone(),
            committed_at_ms: now,
            policy: super::super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
        });
        control::poll("fixture_presence_strike")?;
        Ok(snap)
    })
    .unwrap()
}
fn enemy(c: &mut BattleCore, e: UnitInstanceId, x: i32, y: i32) -> UnitInstanceId {
    let id = UnitInstanceId(uuid::Uuid::from_u128(0xda00 + c.units.len() as u128));
    let mut u = c.units[&e].clone();
    u.instance_id = id;
    u.current_target = None;
    c.units.insert(id, u);
    Arc::make_mut(&mut c.scenario_runtime.spawned)
        .unit_refs
        .insert(
            crate::game::battle::scenario::ScenarioUnitRef::new(format!("presence_{}", id)),
            id,
        );
    place(c, id, x, y);
    id
}
#[test]
fn presence_runtime_actual_regular_attack_binds_fragment_and_triggered_attack_does_not() {
    for regular in [true, false] {
        let (mut c, mut s, a, e, _) = fixture(&[]);
        c.units.get_mut(&a).unwrap().next_basic_attack_ms = 10;
        c.process_event(
            BattleEvent::AttackStart {
                time_ms: 10,
                attacker_instance_id: a,
                target_instance_id: Some(e),
                schedule_next: regular,
                cause: Default::default(),
            },
            10,
        )
        .unwrap();
        c.step_battle_execution_until(&mut s, 20).unwrap();
        assert_eq!(hosts(&c, a), usize::from(regular));
    }
}
#[test]
fn presence_runtime_periodic_samples_continuous_power_and_cadence_modifier_does_not_accelerate_growth(
) {
    for mods in [vec![], vec![0, 0], vec![1]] {
        let (mut c, mut s, a, e, _) = fixture(&mods);
        strike(&mut c, a, e, 10, 1);
        let hp = c.units[&e].stats.current_health;
        c.step_battle_execution_until(&mut s, 1010).unwrap();
        assert_eq!(power(&c, a, 1010), if mods == [1] { 15 } else { 10 });
        assert_eq!(
            hp - c.units[&e].stats.current_health,
            if mods == [0, 0] {
                15
            } else if mods == [1] {
                15
            } else {
                10
            }
        );
    }
}
#[test]
fn presence_runtime_signed_period_changes_match_authored_time_contract() {
    for (percent, interval) in [(-50.0, 500), (50.0, 1500), (100.0, 2000), (-99.999, 1)] {
        let mut doc = crate::authoring::node_graph::tests::presence_fixture();
        doc["nodes"].as_array_mut().unwrap().iter_mut()
            .find(|n| n["kind"] == "skill-modifier-duration").unwrap()["values"]["percent"] = serde_json::json!(percent);
        let (mut c, mut s, a, e, _) = fixture_graph(&RELICS, &[0, 0], doc);
        place(&mut c, e, 3, 1);
        strike(&mut c, a, e, 10, 1);
        let network = &c.sustained.presence.networks[&key(&c, a)];
        assert_eq!(network.capture.program.marks.periodic_interval_ms, interval);
        assert_eq!(network.capture.applied.len(), 1, "one authored contribution despite two equipped copies");
        if interval >= 500 {
            let hp = c.units[&e].stats.current_health;
            c.step_battle_execution_until(&mut s, 10 + u64::from(interval) - 1).unwrap();
            assert_eq!(c.units[&e].stats.current_health, hp);
            c.step_battle_execution_until(&mut s, 10 + u64::from(interval)).unwrap();
            assert_eq!(hp - c.units[&e].stats.current_health, interval / 100);
        }
    }
}
#[test]
fn presence_period_changes_sum_before_one_ceiling_and_reject_nonpositive_result() {
    let ratio = |n, d| MarkRatio { numerator: n, denominator: d };
    assert_eq!(modified_period(1001, &[(true, ratio(25, 1)), (true, ratio(25, 1))]).unwrap(), 501);
    assert_eq!(modified_period(1001, &[(false, ratio(50, 1)), (true, ratio(25, 1))]).unwrap(), 1252);
    assert_eq!(modified_period(1001, &[(true, ratio(1, 3)), (false, ratio(1, 3))]).unwrap(), 1001);
    assert!(modified_period(1000, &[(true, ratio(60, 1)), (true, ratio(40, 1))]).is_err());
}
#[test]
fn presence_runtime_hit_dedup_death_precedence_and_candidate_order() {
    let (mut c, _, a, e, b) = fixture(&[]);
    let near = enemy(&mut c, e, 4, 1);
    let farther = enemy(&mut c, e, 5, 1);
    let first = strike(&mut c, a, e, 10, 1);
    c.runtime_transaction(|c| c.presence_hit(&first, e, false, 10))
        .unwrap();
    assert_eq!(hosts(&c, a), 1);
    strike(&mut c, a, e, 100, 1);
    assert!(c.sustained.presence.networks[&key(&c, a)]
        .hosts
        .contains_key(&near));
    assert_eq!(hosts(&c, a), 2);
    hit(&mut c, b, e, 20000, 200);
    assert!(c.units[&e].lifecycle.is_dead());
    assert_eq!(hosts(&c, a), 2);
    assert!(c.sustained.presence.networks[&key(&c, a)]
        .hosts
        .contains_key(&farther));
}
#[test]
fn presence_runtime_new_lethal_target_never_infects_and_lethal_host_spreads_once() {
    let (mut c, _, a, e, _) = fixture(&[]);
    let near = enemy(&mut c, e, 4, 1);
    let far = enemy(&mut c, e, 5, 1);
    strike(&mut c, a, e, 10, 20000);
    assert_eq!(hosts(&c, a), 0);
    strike(&mut c, a, near, 20, 1);
    strike(&mut c, a, near, 30, 20000);
    assert_eq!(hosts(&c, a), 1);
    assert!(c.sustained.presence.networks[&key(&c, a)]
        .hosts
        .contains_key(&far));
}
#[test]
fn presence_runtime_admission_consumes_snapshot_empty_rejects_and_burst_uses_host_multiplier_once()
{
    let (mut c, mut s, a, e, _) = fixture(&[]);
    let resonance = c.units[&a].resonance_current;
    let attempt = crate::game::battle::core::sim::BattleLiveCommand::ActivateSkill {
        unit_id: a,
        skill_id: "r11_aura".into(),
        target: None,
        time_ms: 0,
    };
    assert!(c.apply_live_command(attempt).is_err());
    assert_eq!(c.units[&a].resonance_current, resonance);
    strike(&mut c, a, e, 10, 1);
    let hp = c.units[&e].stats.current_health;
    cast(&mut c, a, 1010);
    assert_eq!(hosts(&c, a), 0);
    assert_eq!(c.units[&a].resonance_current, 0);
    c.step_battle_execution_until(&mut s, 1010).unwrap();
    assert_eq!(hp - c.units[&e].stats.current_health, 30);
    assert!(c.sustained.presence.pending.is_empty());
}
#[test]
fn presence_runtime_wave_dedup_and_explosion_overlap_include_own_damage_only_once() {
    for (mods, expected) in [(vec![3], 60), (vec![3, 5], 80), (vec![3, 4, 5], 96)] {
        let (mut c, mut s, a, e, _) = fixture(&mods);
        let e2 = enemy(&mut c, e, 4, 1);
        strike(&mut c, a, e, 10, 1);
        strike(&mut c, a, e2, 10, 1);
        let hp = c.units[&e].stats.current_health;
        let hp2 = c.units[&e2].stats.current_health;
        cast(&mut c, a, 1010);
        c.step_battle_execution_until(&mut s, 1010).unwrap();
        assert_eq!(hp - c.units[&e].stats.current_health, expected);
        assert_eq!(hp2 - c.units[&e2].stats.current_health, expected);
    }
}
#[test]
fn presence_runtime_withdrawal_clears_source_and_stale_ticks_do_not_damage() {
    let (mut c, mut s, a, e, _) = fixture(&[]);
    strike(&mut c, a, e, 10, 1);
    let hp = c.units[&e].stats.current_health;
    c.units.get_mut(&a).unwrap().lifecycle = super::super::types::RuntimeUnitLifecycle::Withdrawn;
    c.runtime_transaction(|c| c.reconcile_sustained(100))
        .unwrap();
    assert_eq!(hosts(&c, a), 0);
    c.step_battle_execution_until(&mut s, 1010).unwrap();
    assert_eq!(c.units[&e].stats.current_health, hp);
}
#[test]
fn presence_runtime_independent_fragment_networks_and_burst_suppresses_other_network_death_spread()
{
    let (mut c, mut s, a, e, b) = fixture(&[]);
    let e2 = enemy(&mut c, e, 4, 1);
    strike(&mut c, a, e, 10, 1);
    strike(&mut c, b, e, 10, 1);
    assert_eq!(hosts(&c, a), 1);
    assert_eq!(hosts(&c, b), 1);
    c.units.get_mut(&e).unwrap().stats.current_health = 20;
    cast(&mut c, a, 1010);
    c.step_battle_execution_until(&mut s, 1010).unwrap();
    assert!(c.units[&e].lifecycle.is_dead());
    assert_eq!(hosts(&c, b), 0);
    assert!(c.units[&e2].is_active());
}

#[test]
fn presence_runtime_zero_chance_records_one_attempt_without_creating_network_or_timer() {
    let mut doc = crate::authoring::node_graph::tests::presence_fixture();
    doc["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["kind"] == "skill-mark-weapon-hit")
        .unwrap()["values"]["chance"] = serde_json::json!(0);
    let (mut c, _, a, e, _) = fixture_graph(&RELICS, &[], doc);
    let snap = strike(&mut c, a, e, 10, 1);
    c.runtime_transaction(|c| c.presence_hit(&snap, e, false, 10))
        .unwrap();
    assert!(c.presence_marks_snapshot(10).unwrap().networks.is_empty());
    assert_eq!(c.sustained.presence.attempts.len(), 1);
    assert!(!c
        .event_queue
        .iter()
        .any(|e| matches!(e.as_ref(), BattleEvent::PresenceTick { .. })));
}
#[test]
fn presence_runtime_non_basic_and_reaction_sources_have_no_subscription() {
    let (mut c, _, a, e, _) = fixture(&[]);
    for source in [
        DamageSource::Ability,
        DamageSource::BuffTick,
        DamageSource::Environment,
        DamageSource::BasicAttack,
    ] {
        let mut snap = c
            .damage_source_snapshot_for_unit(
                a,
                e,
                source,
                crate::game::battle::damage::DamageType::True,
                1,
                DamageModifiers::default(),
                0,
                10,
                false,
            )
            .unwrap();
        if source == DamageSource::BasicAttack {
            snap.fact_source.reaction = true;
        }
        c.bind_presence(&mut snap).unwrap();
        assert!(snap.presence_capture.is_none());
    }
}
#[test]
fn presence_runtime_authored_air_boss_abnormality_summon_and_temporary_filters_are_independent() {
    for flag in ["air", "boss", "abnormality", "summon", "temporary"] {
        for allowed in [true, false] {
            let mut doc = crate::authoring::node_graph::tests::presence_fixture();
            let filter = &mut doc["nodes"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["kind"] == "skill-mark-filter")
                .unwrap()["values"];
            for f in ["air", "boss", "abnormality", "summon", "temporary"] {
                filter[f] = serde_json::json!(true);
            }
            filter[flag] = serde_json::json!(allowed);
            let (mut c, _, a, e, _) = fixture_graph(&RELICS, &[], doc);
            let u = c.units.get_mut(&e).unwrap();
            match flag {
                "air" => u.mobility_kind = MobilityKind::Airborne,
                "boss" => u.threat_class = BattleUnitThreatClass::Boss,
                "abnormality" => {
                    u.source_identity = BattleUnitSourceIdentity::Abnormality {
                        abnormality_id: "fixture".into(),
                        base_uuid: u.base_uuid,
                    }
                }
                "summon" => {
                    u.source_identity = BattleUnitSourceIdentity::SummonedUnit {
                        definition_id: "fixture".into(),
                        base_uuid: u.base_uuid,
                        summoner_instance_id: a,
                        summon_sequence: 0,
                        batch_uuid: uuid::Uuid::nil(),
                    }
                }
                _ => {
                    Arc::make_mut(&mut c.scenario_runtime.spawned)
                        .unit_refs
                        .retain(|_, id| *id != e);
                }
            }
            strike(&mut c, a, e, 10, 1);
            assert_eq!(hosts(&c, a), usize::from(allowed), "{flag}={allowed}");
        }
    }
}
#[test]
fn presence_runtime_empty_population_decay_preservation_and_reinfection_use_exact_clock() {
    for preserve in [false, true] {
        let (mut c, _, a, e, b) = fixture(if preserve { &[2] } else { &[] });
        strike(&mut c, a, e, 10, 1);
        hit(&mut c, b, e, 20000, 1010);
        assert_eq!(hosts(&c, a), 0);
        assert_eq!(power(&c, a, 2010), 10);
        assert_eq!(power(&c, a, 3010), if preserve { 10 } else { 8 });
        let fresh = enemy(&mut c, e, 3, 1);
        c.units.get_mut(&fresh).unwrap().lifecycle =
            super::super::types::RuntimeUnitLifecycle::Active;
        c.units.get_mut(&fresh).unwrap().stats.current_health = 10000;
        strike(&mut c, a, fresh, 3010, 1);
        assert_eq!(power(&c, a, 3510), if preserve { 15 } else { 13 });
    }
}
#[test]
fn presence_runtime_same_process_clone_and_failed_transaction_preserve_hosts_power_and_deadlines() {
    let (mut c, mut state, a, e, _) = fixture(&[0, 1]);
    strike(&mut c, a, e, 10, 1);
    let before = c.presence_marks_snapshot(510).unwrap();
    let hp = c.units[&e].stats.current_health;
    let queued = c.event_queue.len();
    let result: Result<(), GameError> = c.runtime_transaction(|c| {
        let r = &c.sustained.presence.networks[&key(c, a)];
        let generation = r.hosts[&e].generation;
        c.presence_tick(key(c, a), e, generation, 510)?;
        Err(GameError::InvalidAction)
    });
    assert!(result.is_err());
    assert_eq!(c.presence_marks_snapshot(510).unwrap(), before);
    assert_eq!(c.units[&e].stats.current_health, hp);
    assert_eq!(c.event_queue.len(), queued);
    let mut clone = c.clone();
    let mut clone_state = state.clone();
    c.step_battle_execution_until(&mut state, 1010).unwrap();
    clone
        .step_battle_execution_until(&mut clone_state, 1010)
        .unwrap();
    assert_eq!(
        c.presence_marks_snapshot(1010).unwrap(),
        clone.presence_marks_snapshot(1010).unwrap()
    );
    assert_eq!(
        c.units[&e].stats.current_health,
        clone.units[&e].stats.current_health
    );
    let snapshot = c.presence_marks_snapshot(1010).unwrap();
    assert_eq!(snapshot.networks[0].applied_modifiers.len(), 2);
    let json = serde_json::to_string(&snapshot).unwrap();
    assert_eq!(
        serde_json::from_str::<PresenceMarksSnapshot>(&json).unwrap(),
        snapshot
    );
}
#[test]
fn presence_runtime_consumed_generation_cannot_damage_a_rebuilt_network() {
    let (mut c, mut s, a, e, _) = fixture(&[]);
    strike(&mut c, a, e, 10, 1);
    let stale = c
        .event_queue
        .iter()
        .find(|e| matches!(e.as_ref(), BattleEvent::PresenceTick { .. }))
        .unwrap()
        .into_owned();
    cast(&mut c, a, 510);
    c.step_battle_execution_until(&mut s, 510).unwrap();
    strike(&mut c, a, e, 520, 1);
    let hp = c.units[&e].stats.current_health;
    c.process_event(stale, 1010).unwrap();
    assert_eq!(c.units[&e].stats.current_health, hp);
    assert_eq!(hosts(&c, a), 1);
}

#[test]
fn presence_runtime_unrepresentable_burst_rolls_back_cost_consumption_and_queue() {
    let mut doc = crate::authoring::node_graph::tests::presence_fixture();
    doc["nodes"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["kind"] == "skill-mark-burst-plan")
        .unwrap()["values"]["coefficient"] = serde_json::json!(1000000000);
    let (mut c, _, a, e, _) = fixture_graph(&RELICS, &[], doc);
    strike(&mut c, a, e, 10, 1);
    let before = c.presence_marks_snapshot(1010).unwrap();
    let resonance = c.units[&a].resonance_current;
    let queued = c.event_queue.len();
    let result = c.apply_live_command(
        crate::game::battle::core::sim::BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            target: None,
            time_ms: 1010,
        },
    );
    assert!(result.is_err());
    assert_eq!(c.presence_marks_snapshot(1010).unwrap(), before);
    assert_eq!(c.units[&a].resonance_current, resonance);
    assert_eq!(c.event_queue.len(), queued);
}

#[test]
fn presence_runtime_immediate_release_still_runs_corrosion_start_gate_first() {
    let (mut c, mut s, a, e, _) = fixture(&[]);
    strike(&mut c, a, e, 10, 1);
    c.units.get_mut(&a).unwrap().employee_trauma =
        Some(c.game_data.run_policy.corrosion.trauma_threshold);
    assert!(c.corrosion_ready(a, &"r11_aura".into(), 1010));
    cast(&mut c, a, 1010);
    c.step_battle_execution_until(&mut s, 1010).unwrap();
    assert!(c.units[&a].corroded);
    assert!(c.forced_pause_remaining_ms().is_some());
    assert!(c.sustained.presence.pending.is_empty());
    assert_eq!(hosts(&c, a), 0);
}

#[test]
fn presence_runtime_fractional_bonuses_add_before_rounding_and_authored_blast_dedup_is_supported() {
    let mut doc = crate::authoring::node_graph::tests::presence_fixture();
    for n in doc["nodes"].as_array_mut().unwrap() {
        if n["kind"] == "skill-explosion-rule" {
            n["values"]["bonus"] = serde_json::json!(12.5);
            n["values"]["dedup"] = serde_json::json!("blast");
        }
        if n["kind"] == "skill-modifier-explosion-power" {
            n["values"]["percent"] = serde_json::json!(7.5);
        }
    }
    let (mut c, mut s, a, e, _) = fixture_graph(&RELICS, &[3, 4], doc);
    let e2 = enemy(&mut c, e, 4, 1);
    place(&mut c, e, 3, 1);
    strike(&mut c, a, e, 10, 1);
    strike(&mut c, a, e2, 10, 1);
    let hp = c.units[&e].stats.current_health;
    cast(&mut c, a, 1010);
    c.step_battle_execution_until(&mut s, 1010).unwrap();
    assert_eq!(hp - c.units[&e].stats.current_health, 96);
}

#[test]
fn presence_runtime_moving_host_uses_admission_position_and_takes_own_burst_once() {
    use crate::game::battle::core::movement::types::{MovementGoal, WorldVec2};
    for mods in [vec![], vec![5]] {
        for (now, tile) in [(1001, 3), (1010, 4)] {
            for radius in [0.35, 0.75] {
                let (mut c, mut s, a, e, _) = fixture(&mods);
                strike(&mut c, a, e, 10, 1);
                c.step_battle_execution_until(&mut s, 1000).unwrap();
                let unit = c.units.get_mut(&e).unwrap();
                unit.body.position = WorldVec2::new(3.99, 1.5);
                unit.body.radius = radius;
                unit.body.move_speed = 2.0;
                unit.stats.move_speed_units_per_ms = 2_000;
                unit.blockable = false;
                let goals = std::collections::HashMap::from([(
                    e,
                    MovementGoal::MoveToPoint {
                        point: WorldVec2::new(7.5, 1.5),
                        stop_radius: 0.1,
                    },
                )]);
                c.run_test_movement_tick(1000, 50, &goals);
                assert_eq!(c.sample_unit_body_at(e, now).unwrap().projected_tile(), Position::new(tile, 1));
                assert_eq!(c.units[&e].body.projected_tile(), Position::new(4, 1));
                let near = enemy(&mut c, e, tile, 1);
                let other = enemy(&mut c, e, if tile == 3 { 4 } else { 3 }, 1);
                let hp = c.units[&e].stats.current_health;
                let near_hp = c.units[&near].stats.current_health;
                let other_hp = c.units[&other].stats.current_health;
                cast(&mut c, a, now);
                assert_eq!(c.sustained.presence.pending[0].centers[0].1, Position::new(tile, 1));
                c.step_battle_execution_until(&mut s, now).unwrap();
                assert_eq!(hp - c.units[&e].stats.current_health, 30, "mods {mods:?}, time {now}, radius {radius}");
                assert_eq!(near_hp - c.units[&near].stats.current_health, 10);
                assert_eq!(c.units[&other].stats.current_health, other_hp);
            }
        }
    }
}

#[test]
fn presence_runtime_spread_samples_both_moving_host_and_candidate_at_hit_time() {
    use crate::game::battle::core::movement::types::{MovementGoal, WorldVec2};
    for moving_host in [true, false] {
        let mut doc = crate::authoring::node_graph::tests::presence_fixture();
        doc["nodes"].as_array_mut().unwrap().iter_mut()
            .find(|n| n["kind"] == "skill-mark-spread").unwrap()["values"]["radius"] = serde_json::json!(0.25);
        let (mut c, mut s, a, e, _) = fixture_graph(&RELICS, &[], doc);
        place(&mut c, e, 3, 1);
        let candidate = enemy(&mut c, e, 6, 1);
        strike(&mut c, a, e, 1, 1);
        c.step_battle_execution_until(&mut s, 1000).unwrap();
        let (moving, stationary) = if moving_host { (e, candidate) } else { (candidate, e) };
        for id in [moving, stationary] {
            let unit = c.units.get_mut(&id).unwrap();
            unit.body.radius = 0.01;
            unit.blockable = false;
            unit.body.position = WorldVec2::new(if id == moving { 3.99 } else { 3.75 }, 1.5);
        }
        c.units.get_mut(&moving).unwrap().body.move_speed = 2.0;
        c.units.get_mut(&moving).unwrap().stats.move_speed_units_per_ms = 2_000;
        let goals = std::collections::HashMap::from([(
            moving,
            MovementGoal::MoveToPoint { point: WorldVec2::new(7.5, 1.5), stop_radius: 0.1 },
        )]);
        c.run_test_movement_tick(1000, 50, &goals);
        assert!(c.units[&moving].body.position.x - c.units[&stationary].body.position.x > 0.25);
        strike(&mut c, a, e, 1001, 1);
        assert!(c.sustained.presence.networks[&key(&c, a)].hosts.contains_key(&candidate), "moving host: {moving_host}");
        assert_eq!(hosts(&c, a), 2);
    }
}
