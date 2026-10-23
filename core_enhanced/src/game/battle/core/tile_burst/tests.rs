use super::*;
use crate::game::battle::core::{
    boss_runtime::tests::compiled_action_fixture::{cast, fixture_graph, hit, place},
    sim::{BattleExecutionState, BattleLiveCommand},
};
const RELICS: [&str; 5] = [
    "widened-horizon",
    "overlapping-explosions",
    "explosion-vulnerability",
    "deep-imprint",
    "funeral-butterfly-orb",
];

#[test]
fn tile_burst_moving_stacks_use_admission_and_death_time_positions() {
    use crate::game::battle::core::movement::types::{MovementGoal, WorldVec2};
    for dies in [false, true] {
        let (mut c, mut s, a, e, b) = fixture("-bloom", &[]);
        strike(&mut c, a, e, 1, 10);
        c.step_battle_execution_until(&mut s, 1000).unwrap();
        let unit = c.units.get_mut(&e).unwrap();
        unit.body.position = WorldVec2::new(3.99, 1.5);
        unit.body.move_speed = 2.0;
        unit.stats.move_speed_units_per_ms = 2_000;
        unit.blockable = false;
        let goals = std::collections::HashMap::from([(
            e,
            MovementGoal::MoveToPoint { point: WorldVec2::new(7.5, 1.5), stop_radius: 0.1 },
        )]);
        c.run_test_movement_tick(1000, 50, &goals);
        assert_eq!(c.sample_unit_body_at(e, 1001).unwrap().projected_tile(), Position::new(3, 1));
        assert_eq!(c.units[&e].body.projected_tile(), Position::new(4, 1));
        if dies {
            hit(&mut c, b, e, 20000, 1001);
            assert!(c.units[&e].is_dead());
            let pool = &c.sustained.weapon_marks.pools[&(copy(&c, a), "butterflies".into())];
            assert!(pool.units.is_empty());
            assert_eq!(pool.tiles.len(), 1);
            assert_eq!(pool.tiles[&(3, 1)], 2);
        } else {
            cast(&mut c, a, 1001);
            let centers = &c.sustained.tile_bursts.pending[0].plan.centers;
            assert_eq!(centers.len(), 1);
            assert_eq!(centers[0].tile, Position::new(3, 1));
            assert_eq!(stacks(&c, a), 0);
        }
    }
}
fn fixture(
    form: &str,
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
        crate::authoring::node_graph::tests::tile_burst_fixture(form),
    );
    place(&mut c, e, 3, 1);
    c.units.get_mut(&e).unwrap().stats.current_health = 10000;
    c.units.get_mut(&e).unwrap().stats.max_health = 10000;
    (c, s, a, e, b)
}
fn copy(c: &BattleCore, a: UnitInstanceId) -> uuid::Uuid {
    let i = c
        .items
        .values()
        .find(|i| i.owner_unit_instance == a)
        .unwrap();
    i.persistent_copy_uuid.unwrap_or(i.instance_id)
}
fn stacks(c: &BattleCore, a: UnitInstanceId) -> u64 {
    c.sustained
        .weapon_marks
        .pools
        .get(&(copy(c, a), "butterflies".into()))
        .map_or(0, |p| p.units.values().chain(p.tiles.values()).sum())
}
fn strike(
    c: &mut BattleCore,
    a: UnitInstanceId,
    e: UnitInstanceId,
    damage: u32,
    now: u64,
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
        let prior = std::mem::replace(
            &mut c.runtime_fact_source,
            RuntimeFactSource {
                actor: Some(a),
                action: Some(action),
                ..Default::default()
            },
        );
        let mut snapshot = c
            .damage_source_snapshot_for_unit(
                a,
                e,
                DamageSource::BasicAttack,
                crate::game::battle::damage::DamageType::True,
                damage,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        c.bind_weapon_marks(&mut snapshot)?;
        c.runtime_fact_source = prior;
        c.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
            target_id: e,
            source_snapshot: snapshot.clone(),
            committed_at_ms: now,
            policy: super::super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
        })
        .ok_or(GameError::InvalidAction)?;
        Ok(snapshot)
    })
    .unwrap()
}
#[test]
fn tile_burst_weapon_actual_regular_launch_and_upgraded_stage_choose_grant_once() {
    for (regular, upgraded, expected) in [(true, false, 2), (true, true, 1), (false, false, 0)] {
        let (mut c, mut s, a, e, _) = fixture("-bloom", &[]);
        if upgraded {
            for i in c.items.values_mut().filter(|i| i.owner_unit_instance == a) {
                i.weapon_stage = crate::game::ability::weapon_marks::WeaponStage::Upgraded;
            }
        }
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
        assert_eq!(stacks(&c, a), expected);
    }
}
#[test]
fn tile_burst_weapon_lethal_grant_transfers_all_once_and_other_damage_cannot_grant() {
    let (mut c, _s, a, e, b) = fixture("-bloom", &[3, 3]);
    let first = strike(&mut c, a, e, 1, 10);
    assert_eq!(
        stacks(&c, a),
        4,
        "duplicate relic copies multiply only once"
    );
    c.runtime_transaction(|c| c.weapon_mark_hit(&first, e))
        .unwrap();
    assert_eq!(stacks(&c, a), 4);
    hit(&mut c, b, e, 1, 11);
    assert_eq!(stacks(&c, a), 4);
    strike(&mut c, a, e, 20000, 12);
    let pool = &c.sustained.weapon_marks.pools[&(copy(&c, a), "butterflies".into())];
    assert!(pool.units.is_empty());
    assert_eq!(pool.tiles[&(3, 1)], 8);
    c.runtime_transaction(|c| c.weapon_mark_death(e, 12)).unwrap();
    assert_eq!(stacks(&c, a), 8);
}
#[test]
fn tile_burst_weapon_other_death_cause_and_fragment_withdraw_preserve_copy_pool() {
    let (mut c, _s, a, e, b) = fixture("-bloom", &[]);
    strike(&mut c, a, e, 1, 10);
    c.units.get_mut(&a).unwrap().fragment_source = None;
    c.units.get_mut(&a).unwrap().lifecycle = super::super::types::RuntimeUnitLifecycle::Withdrawn;
    c.runtime_transaction(|c| c.reconcile_sustained(11))
        .unwrap();
    assert_eq!(stacks(&c, a), 2);
    hit(&mut c, b, e, 20000, 12);
    assert_eq!(
        c.sustained.weapon_marks.pools[&(copy(&c, a), "butterflies".into())].tiles[&(3, 1)],
        2
    );
}
#[test]
fn tile_burst_no_stacks_refuses_without_cost_and_snapshot_consumes_only_own_copy() {
    let (mut c, mut s, a, e, b) = fixture("-bloom", &[]);
    let resonance = c.units[&a].resonance_current;
    assert!(c
        .apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            target: None,
            time_ms: 10
        })
        .is_err());
    assert_eq!(c.units[&a].resonance_current, resonance);
    strike(&mut c, a, e, 1, 10);
    strike(&mut c, b, e, 1, 10);
    cast(&mut c, a, 20);
    assert_eq!(stacks(&c, a), 0);
    assert_eq!(stacks(&c, b), 2);
    c.step_battle_execution_until(&mut s, 20).unwrap();
    strike(&mut c, a, e, 1, 21);
    assert_eq!(stacks(&c, a), 2);
    let hp = c.units[&e].stats.current_health;
    c.step_battle_execution_until(&mut s, 120).unwrap();
    assert_eq!(hp - c.units[&e].stats.current_health, 10);
    c.step_battle_execution_until(&mut s, 220).unwrap();
    assert_eq!(hp - c.units[&e].stats.current_health, 20);
    assert_eq!(stacks(&c, a), 2);
    assert_eq!(stacks(&c, b), 2);
    assert!(c.sustained.tile_bursts.released.is_empty());
}
#[test]
fn tile_burst_rank_discontinuity_and_released_carrier_survive_owner_withdrawal() {
    for (n, expected) in [(2, 20), (3, 15), (6, 40), (9, 75)] {
        let (mut c, mut s, a, e, _) = fixture("-bloom", &[]);
        let key = (copy(&c, a), "butterflies".into());
        c.sustained
            .weapon_marks
            .pools
            .entry(key)
            .or_default()
            .tiles
            .insert((3, 1), n);
        cast(&mut c, a, 10);
        c.step_battle_execution_until(&mut s, 10).unwrap();
        c.units.get_mut(&a).unwrap().lifecycle =
            super::super::types::RuntimeUnitLifecycle::Withdrawn;
        c.runtime_transaction(|c| c.reconcile_sustained(11))
            .unwrap();
        let hp = c.units[&e].stats.current_health;
        c.step_battle_execution_until(&mut s, 410).unwrap();
        assert_eq!(
            hp - c.units[&e].stats.current_health,
            expected,
            "stacks {n}"
        );
        assert!(c.sustained.tile_bursts.released.is_empty());
    }
}
#[test]
fn tile_burst_path_captures_all_declared_routes_and_queries_current_enemy_position() {
    let (mut c, mut s, a, e, _) = fixture("", &[]);
    place(&mut c, e, 7, 3);
    cast(&mut c, a, 10);
    c.step_battle_execution_until(&mut s, 10).unwrap();
    let carrier = c.sustained.tile_bursts.released.values().next().unwrap();
    assert_eq!(
        carrier
            .plan
            .centers
            .iter()
            .map(|c| c.tile)
            .collect::<Vec<_>>(),
        vec![
            Position::new(3, 1),
            Position::new(4, 1),
            Position::new(6, 3)
        ]
    );
    place(&mut c, e, 6, 3);
    let hp = c.units[&e].stats.current_health;
    c.step_battle_execution_until(&mut s, 109).unwrap();
    assert_eq!(c.units[&e].stats.current_health, hp);
    c.step_battle_execution_until(&mut s, 110).unwrap();
    assert_eq!(hp - c.units[&e].stats.current_health, 10);
}
#[test]
fn tile_burst_wave_overlap_modifiers_use_actual_equipment_and_maximum_power() {
    for (mods, expected) in [(vec![0], 20), (vec![0, 1], 30), (vec![0, 1, 2], 45)] {
        let (mut c, mut s, a, e, _) = fixture("-bloom", &mods);
        let pool = c
            .sustained
            .weapon_marks
            .pools
            .entry((copy(&c, a), "butterflies".into()))
            .or_default();
        pool.tiles.insert((3, 1), 6);
        pool.tiles.insert((4, 1), 2);
        cast(&mut c, a, 10);
        c.step_battle_execution_until(&mut s, 10).unwrap();
        let hp = c.units[&e].stats.current_health;
        c.step_battle_execution_until(&mut s, 110).unwrap();
        assert_eq!(hp - c.units[&e].stats.current_health, expected);
        c.step_battle_execution_until(&mut s, 210).unwrap();
        assert_eq!(hp - c.units[&e].stats.current_health, expected * 2);
    }
}
#[test]
fn tile_burst_orb_actual_relic_replaces_rounds_and_host_expiry_does_not_advance_battle() {
    for mods in [vec![], vec![4, 1, 2]] {
        let (mut c, mut s, a, e, _) = fixture("-orb", &mods);
        c.sustained
            .weapon_marks
            .pools
            .entry((copy(&c, a), "butterflies".into()))
            .or_default()
            .tiles
            .insert((3, 1), 6);
        place(&mut c, e, 4, 2);
        cast(&mut c, a, 10);
        c.step_battle_execution_until(&mut s, 10).unwrap();
        if mods.is_empty() {
            assert_eq!(c.forced_pause_remaining_ms(), None);
            continue;
        }
        assert_eq!(c.forced_pause_remaining_ms(), Some(500));
        assert_eq!(
            c.forced_pause_reason(),
            crate::game::behavior::ActionDisabledReasonCode::SkillPresentationPause
        );
        let hp = c.units[&e].stats.current_health;
        assert!(c.ensure_not_forced_paused().is_err());
        c.advance_forced_pause_host_elapsed(499).unwrap();
        assert_eq!(c.units[&e].stats.current_health, hp);
        c.advance_forced_pause_host_elapsed(99999).unwrap();
        assert_eq!(c.forced_pause_remaining_ms(), None);
        assert_eq!(hp - c.units[&e].stats.current_health, 33);
        assert!(c.sustained.tile_bursts.released.is_empty());
        assert!(!c.advance_forced_pause_host_elapsed(1).unwrap());
        c.step_battle_execution_until(&mut s, 200).unwrap();
        assert_eq!(hp - c.units[&e].stats.current_health, 33);
    }
}

#[test]
fn tile_burst_consumption_damage_and_orb_expiry_faults_restore_and_retry() {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "tile_burst_stacks_consumed",
        "tile_burst_damage",
        "tile_burst_wave_complete",
    ] {
        let orb = site == "tile_burst_wave_complete";
        let (mut c, mut s, a, e, _) = fixture(
            if orb { "-orb" } else { "-bloom" },
            if orb { &[4] } else { &[] },
        );
        let key = (copy(&c, a), "butterflies".into());
        c.sustained
            .weapon_marks
            .pools
            .entry(key)
            .or_default()
            .tiles
            .insert((3, 1), 6);
        if site != "tile_burst_stacks_consumed" {
            cast(&mut c, a, 10);
            c.step_battle_execution_until(&mut s, 10).unwrap();
        }
        if site == "tile_burst_damage" {
            c.step_battle_execution_until(&mut s, 109).unwrap();
        }
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            c.weapon_marks_snapshot().unwrap(),
            format!("{:?}", c.sustained.tile_bursts),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    if site == "tile_burst_stacks_consumed" {
                        c.apply_live_command(BattleLiveCommand::ActivateSkill {
                            unit_id: a,
                            skill_id: "r11_aura".into(),
                            target: None,
                            time_ms: 10,
                        })
                        .map(|_| ())
                    } else if orb {
                        c.advance_forced_pause_host_elapsed(500).map(|_| ())
                    } else {
                        c.step_battle_execution_until(&mut s, 110).map(|_| ())
                    }
                },
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
                c.weapon_marks_snapshot().unwrap(),
                format!("{:?}", c.sustained.tile_bursts),
                format!("{s:?}")
            ),
            before,
            "{site}"
        );
        if site == "tile_burst_stacks_consumed" {
            cast(&mut c, a, 10);
            assert_eq!(stacks(&c, a), 0);
        } else if orb {
            c.advance_forced_pause_host_elapsed(500).unwrap();
            assert_eq!(c.forced_pause_remaining_ms(), None);
        } else {
            let hp = c.units[&e].stats.current_health;
            c.step_battle_execution_until(&mut s, 110).unwrap();
            assert_eq!(hp - c.units[&e].stats.current_health, 20);
        }
    }
}
#[test]
fn tile_burst_clone_and_terminal_cleanup_own_pending_events_and_pause() {
    for orb in [false, true] {
        let (mut c, mut s, a, e, _) = fixture(
            if orb { "-orb" } else { "-bloom" },
            if orb { &[4] } else { &[] },
        );
        let key = (copy(&c, a), "butterflies".into());
        c.sustained
            .weapon_marks
            .pools
            .entry(key)
            .or_default()
            .tiles
            .insert((3, 1), 6);
        let snapshot = c.weapon_marks_snapshot().unwrap();
        assert_eq!(
            serde_json::from_str::<crate::game::ability::weapon_marks::WeaponMarksSnapshot>(
                &serde_json::to_string(&snapshot).unwrap()
            )
            .unwrap(),
            snapshot
        );
        cast(&mut c, a, 10);
        c.step_battle_execution_until(&mut s, 10).unwrap();
        let published = c.tile_bursts_snapshot().unwrap();
        assert_eq!(
            serde_json::from_str::<crate::game::ability::tile_burst::TileBurstsSnapshot>(
                &serde_json::to_string(&published).unwrap()
            )
            .unwrap(),
            published
        );
        assert_eq!(published.released[0].orb_relic_copy.is_some(), orb);
        let mut clone = c.clone();
        let mut clone_s = s.clone();
        if orb {
            c.advance_forced_pause_host_elapsed(500).unwrap();
            clone.advance_forced_pause_host_elapsed(500).unwrap();
        } else {
            c.step_battle_execution_until(&mut s, 210).unwrap();
            clone
                .step_battle_execution_until(&mut clone_s, 210)
                .unwrap();
        }
        assert_eq!(
            c.units[&e].stats.current_health,
            clone.units[&e].stats.current_health
        );
        assert_eq!(
            format!("{:?}", c.sustained.tile_bursts),
            format!("{:?}", clone.sustained.tile_bursts)
        );
        let (mut c, mut s, a, _, _) = fixture(
            if orb { "-orb" } else { "-bloom" },
            if orb { &[4] } else { &[] },
        );
        let key = (copy(&c, a), "butterflies".into());
        c.sustained
            .weapon_marks
            .pools
            .entry(key)
            .or_default()
            .tiles
            .insert((3, 1), 6);
        cast(&mut c, a, 10);
        c.step_battle_execution_until(&mut s, 10).unwrap();
        c.runtime_transaction(|c| c.clear_sustained(10)).unwrap();
        assert!(c.sustained.tile_bursts.released.is_empty());
        assert!(c.sustained.weapon_marks.pools.is_empty());
        assert_eq!(c.forced_pause_remaining_ms(), None);
        assert!(!c.advance_forced_pause_host_elapsed(1000).unwrap());
    }
}

#[test]
fn tile_burst_unrepresentable_rank_refuses_before_cost_and_keeps_stack_store() {
    let (mut c, _, a, _, _) = fixture("-bloom", &[]);
    let key = (copy(&c, a), "butterflies".into());
    c.sustained
        .weapon_marks
        .pools
        .entry(key)
        .or_default()
        .tiles
        .insert((3, 1), u64::MAX);
    let before = (
        c.units[&a].resonance_current,
        c.weapon_marks_snapshot().unwrap(),
        c.event_queue_debug_snapshot(),
    );
    assert!(c
        .apply_live_command(BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            target: None,
            time_ms: 10
        })
        .is_err());
    assert_eq!(
        (
            c.units[&a].resonance_current,
            c.weapon_marks_snapshot().unwrap(),
            c.event_queue_debug_snapshot()
        ),
        before
    );
}
