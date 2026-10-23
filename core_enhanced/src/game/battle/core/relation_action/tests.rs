use super::*;
use crate::game::battle::{
    core::{
        boss_runtime::tests::compiled_action_fixture::{fixture_graph, hit, place},
        sim::{BattleExecutionState, BattleLiveCommand},
    },
    damage::DamageSource,
};
const RELICS: [&str; 10] = [
    "deep-wedge",
    "hasty-second-hand",
    "bottomless-cup",
    "double-harvest",
    "deep-imprint",
    "layered-shell",
    "widened-horizon",
    "unbroken-trance",
    "next",
    "broken-balance",
];
fn fixture(
    bloom: bool,
    equipped: &[usize],
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let doc = crate::authoring::node_graph::tests::relation_action_fixture(bloom);
    let (mut c, s, a, e, b) = fixture_graph(&RELICS, equipped, doc);
    c.units.get_mut(&b).unwrap().fragment_source = None;
    c.units.get_mut(&b).unwrap().skill_id = None;
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    place(&mut c, e, 3, 1);
    place(&mut c, b, 2, 2);
    for id in [a, e, b] {
        let u = c.units.get_mut(&id).unwrap();
        u.stats.max_health = 10000;
        u.stats.current_health = 10000;
        u.stats.defense = 0;
    }
    c.runtime_transaction(|c| {
        c.clear_relation_actions(0)?;
        c.reconcile_relation_actions(0)
    })
    .unwrap();
    (c, s, a, e, b)
}
fn key(c: &BattleCore, a: UnitInstanceId) -> Uuid {
    c.units[&a].fragment_source.as_ref().unwrap().copy_uuid
}
fn owner(c: &BattleCore, a: UnitInstanceId) -> &Owner {
    &c.sustained.relations.owners[&key(c, a)]
}
fn stock(c: &mut BattleCore, a: UnitInstanceId, n: u32) {
    let k = key(c, a);
    c.sustained.relations.owners.get_mut(&k).unwrap().stock = n;
}
fn cast(c: &mut BattleCore, a: UnitInstanceId, bloom: bool, now: u64) {
    c.apply_live_command(BattleLiveCommand::ActivateSkill {
        unit_id: a,
        skill_id: "r11_aura".into(),
        target: bloom.then_some(SkillCastTarget::Tile {
            position: Position::new(3, 1),
        }),
        time_ms: now,
    })
    .unwrap();
}
fn relation(c: &mut BattleCore, a: UnitInstanceId, t: UnitInstanceId, now: u64) {
    c.apply_live_command(BattleLiveCommand::SelectRelation {
        unit_id: a,
        entry: "blessing".into(),
        target: t,
        time_ms: now,
    })
    .unwrap();
}
fn attack(c: &mut BattleCore, a: UnitInstanceId, e: UnitInstanceId, now: u64) {
    c.units.get_mut(&a).unwrap().next_basic_attack_ms = now;
    c.event_queue.push(BattleEvent::AttackStart {
        time_ms: now,
        attacker_instance_id: a,
        target_instance_id: Some(e),
        schedule_next: true,
        cause: Default::default(),
    });
}
#[test]
fn relation_action_runtime_normal_consumption_and_post_stage_wounds() {
    for (n, remaining, damage, wound) in [
        (0, 0, 100, 0),
        (1, 0, 200, 1),
        (4, 0, 760, 4),
        (7, 3, 760, 4),
        (8, 0, 820, 8),
    ] {
        let (mut c, mut s, a, e, _) = fixture(false, &[]);
        stock(&mut c, a, n);
        let next = c.units[&a].next_basic_attack_ms;
        cast(&mut c, a, false, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        assert!(owner(&c, a).active.as_ref().unwrap().waiting_basic);
        assert_eq!(c.units[&a].next_basic_attack_ms, next);
        assert!(!c.channel_pauses_basic(a));
        attack(&mut c, a, e, 100);
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert_eq!(owner(&c, a).stock, remaining, "stock {n}");
        assert!(
            owner(&c, a).active.is_none(),
            "stock {n}: {:?}; windup {}",
            c.relation_actions_snapshot().unwrap(),
            c.units[&a].basic_attack.windup_ms
        );
        assert_eq!(
            10000 - c.units[&e].stats.current_health,
            damage,
            "stock {n}"
        );
        assert_eq!(c.hit_damage_taken_percent(e).unwrap(), wound * 5);
        assert!(c.event_log.entries.iter().any(|e| matches!(
            e.event,
            crate::game::battle::event_log::BattleLogEvent::HpChanged {
                damage_source: Some(DamageSource::BasicAttack),
                ..
            }
        )));
        assert!(!c.resonance_use_has_obligations(a));
    }
}
#[test]
fn relation_action_runtime_menu_toggles_restarts_period_and_preserves_once_shield() {
    let (mut c, mut s, a, _, b) = fixture(false, &[]);
    let next = c.units[&a].next_basic_attack_ms;
    let resource = c.units[&a].resonance_current;
    relation(&mut c, a, b, 0);
    c.step_battle_execution_until(&mut s, 999).unwrap();
    assert_eq!(owner(&c, a).stock, 0);
    c.step_battle_execution_until(&mut s, 1000).unwrap();
    assert_eq!(owner(&c, a).stock, 1);
    relation(&mut c, a, a, 1000);
    assert_eq!(c.units[&a].next_basic_attack_ms, next);
    assert_eq!(c.units[&a].resonance_current, resource);
    c.step_battle_execution_until(&mut s, 1500).unwrap();
    relation(&mut c, a, a, 1500);
    assert!(owner(&c, a).relation.is_none());
    c.step_battle_execution_until(&mut s, 2500).unwrap();
    assert_eq!(owner(&c, a).stock, 1);
    c.units.get_mut(&b).unwrap().stats.current_health = 1000;
    relation(&mut c, a, b, 2500);
    assert!(owner(&c, a).shield_used);
    let shields = c
        .sustained_effect_snapshot()
        .unwrap()
        .active
        .into_iter()
        .filter_map(|r| r.shield)
        .collect::<Vec<_>>();
    assert_eq!(shields.len(), 1);
    assert_eq!(shields[0].capacity, 5000);
    relation(&mut c, a, a, 2500);
    assert_eq!(
        c.sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .filter(|r| r.shield.is_some())
            .count(),
        1
    );
    assert_eq!(c.relation_reduction(a).unwrap(), 25);
    assert_eq!(c.relation_reduction(b).unwrap(), 0);
    c.apply_live_command(BattleLiveCommand::WithdrawUnit {
        unit_id: a,
        time_ms: 2500,
    })
    .unwrap();
    assert!(owner(&c, a).relation.is_none());
    assert!(owner(&c, a).shield_used);
    assert_eq!(owner(&c, a).stock, 1);
    assert!(c
        .sustained_effect_snapshot()
        .unwrap()
        .active
        .iter()
        .any(|r| r.target == b && r.shield.is_some()));
}
#[test]
fn relation_action_runtime_death_reserves_after_whole_combo_without_free_cast() {
    let (mut c, mut s, a, e, b) = fixture(false, &[]);
    stock(&mut c, a, 4);
    relation(&mut c, a, b, 0);
    cast(&mut c, a, false, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    attack(&mut c, a, e, 100);
    c.step_battle_execution_until(&mut s, 350).unwrap();
    hit(&mut c, e, b, 20000, 350);
    assert!(owner(&c, a).reserved_mode);
    assert!(!owner(&c, a).mode);
    assert!(owner(&c, a).active.is_some());
    c.step_battle_execution_until(&mut s, 2001).unwrap();
    assert!(owner(&c, a).mode);
    assert!(!owner(&c, a).reserved_mode);
    assert_eq!(owner(&c, a).stock, 0);
    assert!(owner(&c, a).active.is_none());
    assert_eq!(c.units[&a].stats.current_health, 10000);
}
#[test]
fn relation_action_runtime_despair_uses_fixed_total_and_posthit_local_ramp() {
    let (mut c, mut s, a, e, b) = fixture(false, &[]);
    stock(&mut c, a, 4);
    relation(&mut c, a, b, 0);
    hit(&mut c, e, b, 20000, 0);
    assert!(owner(&c, a).mode);
    assert_eq!(owner(&c, a).stock, 4);
    assert_eq!(c.units[&a].stats.current_health, 8000);
    cast(&mut c, a, false, 0);
    c.step_battle_execution_until(&mut s, 599).unwrap();
    assert!(owner(&c, a).active.is_some());
    assert_eq!(10000 - c.units[&e].stats.current_health, 910);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 0);
    c.step_battle_execution_until(&mut s, 600).unwrap();
    assert!(owner(&c, a).active.is_none());
    assert_eq!(owner(&c, a).stock, 4);
}
#[test]
fn relation_action_runtime_bloom_spends_at_release_and_hits_current_whole_area() {
    for n in [0, 4] {
        let (mut c, mut s, a, e, _) = fixture(true, &[]);
        stock(&mut c, a, n);
        cast(&mut c, a, true, 0);
        c.step_battle_execution_until(&mut s, 99).unwrap();
        assert_eq!(owner(&c, a).stock, n);
        assert_eq!(c.units[&e].stats.current_health, 10000);
        c.step_battle_execution_until(&mut s, 100).unwrap();
        assert_eq!(owner(&c, a).stock, 0);
        assert_eq!(c.units[&e].stats.current_health, 9900);
        c.step_battle_execution_until(&mut s, 501).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 10000 - 100 * (n + 1));
        assert!(owner(&c, a).active.is_none());
    }
}
#[test]
fn relation_action_runtime_bloom_timed_expiry_precedes_both_ticks() {
    let (mut c, mut s, a, e, b) = fixture(true, &[]);
    relation(&mut c, a, b, 0);
    hit(&mut c, e, b, 20000, 0);
    cast(&mut c, a, true, 0);
    c.step_battle_execution_until(&mut s, 499).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 10000);
    c.step_battle_execution_until(&mut s, 1999).unwrap();
    assert_eq!(c.units[&e].stats.current_health, 9685);
    assert_eq!(c.units[&a].stats.current_health, 9000);
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 15);
    c.step_battle_execution_until(&mut s, 2000).unwrap();
    assert!(owner(&c, a).active.is_none());
    assert_eq!(c.units[&e].stats.current_health, 9685);
    assert_eq!(c.units[&a].stats.current_health, 9000);
    c.step_battle_execution_until(&mut s, 6500).unwrap();
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 0);
}
fn buff(c: &mut BattleCore, e: UnitInstanceId, a: UnitInstanceId, name: &str, now: u64) {
    c.runtime_transaction(|c| {
        c.process_commands(
            vec![crate::game::battle::damage::BattleCommand::ApplyBuff {
                caster_id: e,
                target_id: a,
                buff_id: crate::game::battle::buffs::BuffId::from_name(name),
                duration_ms: 300,
            }],
            now,
        );
        control::poll("relation_fixture_buff")
    })
    .unwrap();
}
#[test]
fn relation_action_runtime_protection_preserves_only_running_actions_and_manual_stop_still_works() {
    for protected in [false, true] {
        for name in ["r11_stun", "r11_silence"] {
            let (mut c, mut s, a, e, b) = fixture(true, if protected { &[7] } else { &[] });
            relation(&mut c, a, b, 0);
            hit(&mut c, e, b, 20000, 0);
            cast(&mut c, a, true, 0);
            c.step_battle_execution_until(&mut s, 100).unwrap();
            buff(&mut c, e, a, name, 100);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(owner(&c, a).active.is_some(), protected, "{name}");
            if protected {
                let id = owner(&c, a).active.as_ref().unwrap().id;
                c.apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
                    unit_id: a,
                    skill_id: "r11_aura".into(),
                    activation: id,
                    target: None,
                    time_ms: 101,
                })
                .unwrap();
                assert!(owner(&c, a).active.is_none());
            }
            c.step_battle_execution_until(&mut s, 2500).unwrap();
            assert_eq!(c.units[&e].stats.current_health, 10000);
            assert_eq!(c.units[&a].stats.current_health, 10000);
            c.units.get_mut(&a).unwrap().resonance_current = 100;
            buff(&mut c, e, a, name, 2500);
            c.step_battle_execution_until(&mut s, 2500).unwrap();
            assert!(c
                .apply_live_command(BattleLiveCommand::ActivateSkill {
                    unit_id: a,
                    skill_id: "r11_aura".into(),
                    target: Some(SkillCastTarget::Tile {
                        position: Position::new(3, 1)
                    }),
                    time_ms: 2500
                })
                .is_err());
        }
    }
}
#[test]
fn relation_action_runtime_actual_relics_change_period_gain_capacity_and_shield_independently() {
    for (equipped, period, gain, cap) in [
        (&[1][..], 800, 1, Some(4)),
        (&[3][..], 1000, 2, Some(4)),
        (&[1, 2, 3][..], 800, 2, None),
    ] {
        let (mut c, mut s, a, _, b) = fixture(false, equipped);
        relation(&mut c, a, b, 0);
        c.step_battle_execution_until(&mut s, period - 1).unwrap();
        assert_eq!(owner(&c, a).stock, 0);
        c.step_battle_execution_until(&mut s, period).unwrap();
        assert_eq!(owner(&c, a).stock, gain);
        assert_eq!(owner(&c, a).program.stock_capacity, cap);
        c.step_battle_execution_until(&mut s, period * 3).unwrap();
        assert_eq!(
            owner(&c, a).stock,
            if cap.is_some() {
                (gain * 3).min(4)
            } else {
                gain * 3
            }
        );
        assert_eq!(
            owner(&c, a).relation.as_ref().unwrap().next_gain,
            period * 4
        );
    }
    let (mut c, _, a, _, b) = fixture(false, &[5]);
    c.units.get_mut(&b).unwrap().stats.current_health = 1000;
    relation(&mut c, a, b, 0);
    assert_eq!(
        c.sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .find_map(|r| r.shield)
            .unwrap()
            .capacity,
        7500
    );
}
#[test]
fn relation_action_runtime_wedge_and_imprint_have_distinct_real_relic_boundaries() {
    for (equipped, damage, wound) in [(&[0][..], 820, 20), (&[4][..], 820, 40)] {
        let (mut c, mut s, a, e, _) = fixture(false, equipped);
        stock(&mut c, a, 4);
        cast(&mut c, a, false, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        attack(&mut c, a, e, 100);
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert_eq!(10000 - c.units[&e].stats.current_health, damage);
        assert_eq!(c.hit_damage_taken_percent(e).unwrap(), wound);
    }
    for equipped in [&[][..], &[0][..]] {
        let (mut c, mut s, a, e, b) = fixture(true, equipped);
        relation(&mut c, a, b, 0);
        hit(&mut c, e, b, 20000, 0);
        cast(&mut c, a, true, 0);
        c.step_battle_execution_until(&mut s, 2000).unwrap();
        assert_eq!(
            10000 - c.units[&e].stats.current_health,
            315,
            "periodic area excludes direct multihit"
        );
    }
}
#[test]
fn relation_action_runtime_primary_death_stops_unstarted_costs_or_retargets_with_next() {
    for retarget in [false, true] {
        let (mut c, mut s, a, e, b) = fixture(false, if retarget { &[8] } else { &[] });
        c.units.get_mut(&b).unwrap().owner = crate::game::enums::Side::Opponent;
        place(&mut c, b, 4, 1);
        stock(&mut c, a, 4);
        c.units.get_mut(&e).unwrap().stats.current_health = 50;
        cast(&mut c, a, false, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        attack(&mut c, a, e, 100);
        c.step_battle_execution_until(&mut s, 102).unwrap();
        assert_eq!(owner(&c, a).stock, if retarget { 3 } else { 4 });
        assert_eq!(owner(&c, a).active.is_some(), retarget);
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert_eq!(owner(&c, a).stock, if retarget { 0 } else { 4 });
        assert!(owner(&c, a).active.is_none());
        assert_eq!(
            c.hit_damage_taken_percent(b).unwrap(),
            if retarget { 20 } else { 0 }
        );
        assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 0);
    }
}
#[test]
fn relation_action_runtime_widened_primary_and_control_vulnerability_require_real_hit() {
    for equipped in [&[][..], &[6][..], &[9][..]] {
        let (mut c, mut s, a, e, b) = fixture(false, equipped);
        c.units.get_mut(&b).unwrap().owner = crate::game::enums::Side::Opponent;
        place(&mut c, b, 3, 1);
        stock(&mut c, a, 0);
        cast(&mut c, a, false, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        attack(&mut c, a, e, 100);
        c.step_battle_execution_until(&mut s, 102).unwrap();
        let hp = [
            c.units[&e].stats.current_health,
            c.units[&b].stats.current_health,
        ];
        assert_eq!(
            hp.iter().filter(|hp| **hp == 9900).count(),
            if equipped == [6] { 2 } else { 1 }
        );
        let debuff =
            c.hit_damage_taken_percent(e).unwrap() + c.hit_damage_taken_percent(b).unwrap();
        assert_eq!(debuff, if equipped == [9] { 10 } else { 0 });
    }
    let (mut c, mut s, a, e, _) = fixture(false, &[9]);
    c.units
        .get_mut(&e)
        .unwrap()
        .target_traits
        .push(crate::game::battle::types::UnitTargetTrait::StunImmune);
    cast(&mut c, a, false, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    attack(&mut c, a, e, 100);
    c.step_battle_execution_until(&mut s, 102).unwrap();
    assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 0);
}
#[test]
fn relation_action_runtime_shield_responds_between_hits_and_entry_damage_bypasses_it() {
    let (mut c, mut s, a, e, b) = fixture(false, &[]);
    c.units.get_mut(&b).unwrap().stats.current_health = 1100;
    relation(&mut c, a, b, 0);
    hit(&mut c, e, b, 200, 0);
    assert!(owner(&c, a).shield_used);
    assert_eq!(c.units[&b].stats.current_health, 950);
    hit(&mut c, e, b, 1000, 0);
    assert_eq!(c.units[&b].stats.current_health, 950);
    stock(&mut c, a, 4);
    c.begin_sustained(
        a,
        a,
        crate::game::ability::sustained::SustainedEffectDef::Shield {
            duration_ms: 20000,
            amount: 5000,
            priority: 0,
            shared_radius_units: None,
        },
        0,
    )
    .unwrap();
    hit(&mut c, e, b, 20000, 0);
    assert!(owner(&c, a).mode);
    assert_eq!(c.units[&a].stats.current_health, 8000);
    assert_eq!(
        c.sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .find(|r| r.target == a)
            .unwrap()
            .shield
            .unwrap()
            .remaining,
        5000
    );
    c.step_battle_execution_until(&mut s, 1).unwrap();
    assert!(owner(&c, a).active.is_none());
}
#[test]
fn relation_action_runtime_cancellation_restores_menu_admission_cost_damage_and_completion() {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "relation_target_changed",
        "relation_action_admitted",
        "relation_stage_spent",
        "relation_strike_damage",
        "relation_action_finished",
        "relation_stock_gained",
    ] {
        let (mut c, mut s, a, _, b) = fixture(true, &[]);
        stock(&mut c, a, 2);
        let menu = site == "relation_target_changed";
        let admission = site == "relation_action_admitted";
        let gain = site == "relation_stock_gained";
        let now = if menu || admission {
            0
        } else if gain {
            1000
        } else if site == "relation_action_finished" {
            300
        } else {
            100
        };
        if gain {
            relation(&mut c, a, b, 0);
        } else if !menu && !admission {
            cast(&mut c, a, true, 0);
        }
        if now > 0 {
            c.step_battle_execution_until(&mut s, now - 1).unwrap();
        }
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            c.relation_actions_snapshot().unwrap(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    if menu {
                        c.apply_live_command(BattleLiveCommand::SelectRelation {
                            unit_id: a,
                            entry: "blessing".into(),
                            target: b,
                            time_ms: now,
                        })
                        .map(|_| ())
                    } else if admission {
                        c.apply_live_command(BattleLiveCommand::ActivateSkill {
                            unit_id: a,
                            skill_id: "r11_aura".into(),
                            target: Some(SkillCastTarget::Tile {
                                position: Position::new(3, 1),
                            }),
                            time_ms: now,
                        })
                        .map(|_| ())
                    } else {
                        c.step_battle_execution_until(&mut s, now).map(|_| ())
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
                c.relation_actions_snapshot().unwrap(),
                format!("{s:?}")
            ),
            before,
            "{site}"
        );
        if menu {
            relation(&mut c, a, b, now);
        } else if admission {
            cast(&mut c, a, true, now);
        } else {
            c.step_battle_execution_until(&mut s, now).unwrap();
        }
    }
}
#[test]
fn relation_action_runtime_clone_snapshot_stale_generations_and_terminal_cleanup() {
    let (mut c, mut s, a, _, b) = fixture(true, &[]);
    stock(&mut c, a, 4);
    relation(&mut c, a, b, 0);
    let generation = owner(&c, a).relation.as_ref().unwrap().generation;
    relation(&mut c, a, a, 0);
    let before = owner(&c, a).stock;
    c.runtime_transaction(|c| c.relation_gain(key(c, a), generation, 1000))
        .unwrap();
    assert_eq!(owner(&c, a).stock, before);
    cast(&mut c, a, true, 0);
    c.step_battle_execution_until(&mut s, 250).unwrap();
    let snapshot = c.relation_actions_snapshot().unwrap();
    assert_eq!(
        serde_json::from_str::<RelationActionsSnapshot>(&serde_json::to_string(&snapshot).unwrap())
            .unwrap(),
        snapshot
    );
    let (mut cloned, mut cloned_s) = (c.clone(), s.clone());
    c.step_battle_execution_until(&mut s, 600).unwrap();
    cloned
        .step_battle_execution_until(&mut cloned_s, 600)
        .unwrap();
    assert_eq!(
        c.relation_actions_snapshot().unwrap(),
        cloned.relation_actions_snapshot().unwrap()
    );
    assert_eq!(
        c.event_queue_debug_snapshot(),
        cloned.event_queue_debug_snapshot()
    );
    assert!(c
        .apply_live_command(BattleLiveCommand::SelectRelation {
            unit_id: a,
            entry: "blessing".into(),
            target: b,
            time_ms: 249
        })
        .is_err());
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(700);
    c.step_battle_execution_until(&mut s, 701).unwrap();
    c.finalize_battle_execution(&mut s).unwrap();
    assert!(c.relation_actions_snapshot().unwrap().owners.is_empty());
    assert!(!c.event_queue.iter().any(|e| matches!(
        e.as_ref(),
        BattleEvent::RelationGain { .. } | BattleEvent::RelationActionTick { .. }
    )));
}
#[test]
fn relation_action_runtime_external_primary_death_wakes_wait_and_does_not_consume_future_stages() {
    let (mut c, mut s, a, e, b) = fixture(false, &[]);
    stock(&mut c, a, 4);
    cast(&mut c, a, false, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    attack(&mut c, a, e, 100);
    c.step_battle_execution_until(&mut s, 250).unwrap();
    assert_eq!(owner(&c, a).stock, 2);
    hit(&mut c, b, e, 20000, 250);
    c.step_battle_execution_until(&mut s, 250).unwrap();
    assert!(owner(&c, a).active.is_none());
    assert_eq!(owner(&c, a).stock, 2);
    assert!(!c.resonance_use_has_obligations(a));
}
#[test]
fn relation_action_runtime_redeployment_preserves_copy_stock_mode_and_shield_use() {
    for despair in [false, true] {
        let (mut c, _, a, e, b) = fixture(false, &[]);
        stock(&mut c, a, 4);
        c.units.get_mut(&b).unwrap().stats.current_health = 1000;
        relation(&mut c, a, b, 0);
        if despair {
            hit(&mut c, e, b, 30000, 0);
            assert!(owner(&c, a).mode);
        }
        let copy = key(&c, a);
        let owned = c.units[&a].source_owned_uuid;
        let draft = c
            .scenario
            .groups
            .iter()
            .flat_map(|g| &g.spawns)
            .find(|s| s.draft.owned_uuid == owned)
            .unwrap()
            .draft
            .clone();
        c.apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: a,
            time_ms: 10,
        })
        .unwrap();
        let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id: new } =
            c.apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position: Position::new(2, 1),
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: FacingDirection::Right,
                instance_salt: 4242,
                time_ms: 10,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(key(&c, new), copy);
        assert_eq!(owner(&c, new).stock, 4);
        assert_eq!(owner(&c, new).mode, despair);
        assert!(owner(&c, new).shield_used);
        assert!(owner(&c, new).relation.is_none());
    }
}
#[test]
fn relation_action_runtime_self_lethal_entry_and_maintenance_cannot_recreate_action() {
    for entry in [true, false] {
        let (mut c, mut s, a, e, b) = fixture(true, &[]);
        if entry {
            stock(&mut c, a, 4);
        }
        c.units.get_mut(&a).unwrap().stats.current_health = if entry { 1500 } else { 500 };
        relation(&mut c, a, b, 0);
        hit(&mut c, e, b, 30000, 0);
        // Restore the fallen ally through deployment so the battle remains
        // live while checking that the dead owner's future ticks are gone.
        let owned = c.units[&b].source_owned_uuid;
        let draft = c
            .scenario
            .groups
            .iter()
            .flat_map(|g| &g.spawns)
            .find(|spawn| spawn.draft.owned_uuid == owned)
            .unwrap()
            .draft
            .clone();
        c.apply_live_command(BattleLiveCommand::DeployPlayerUnit {
            draft,
            position: Position::new(7, 3),
            tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
            facing: FacingDirection::Right,
            instance_salt: 5252,
            time_ms: 0,
            current_hp_policy: None,
        })
        .unwrap();
        if !entry {
            cast(&mut c, a, true, 0);
        }
        c.step_battle_execution_until(&mut s, 1001).unwrap();
        assert!(c.units[&a].lifecycle.is_dead());
        assert!(owner(&c, a).mode);
        assert!(owner(&c, a).active.is_none());
        assert!(owner(&c, a).relation.is_none());
        let hp = c.units[&e].stats.current_health;
        c.step_battle_execution_until(&mut s, 2500).unwrap();
        assert_eq!(c.units[&e].stats.current_health, hp);
        assert!(!c.event_queue.iter().any(|e| matches!(
            e.as_ref(),
            BattleEvent::RelationGain { .. } | BattleEvent::RelationActionTick { .. }
        )));
    }
}
#[test]
fn relation_action_runtime_mode_waits_for_an_ordinary_basic_attack_damage_to_finish() {
    let (mut c, mut s, a, e, b) = fixture(false, &[]);
    stock(&mut c, a, 4);
    relation(&mut c, a, b, 0);
    c.units.get_mut(&a).unwrap().basic_attack.windup_ms = 100;
    attack(&mut c, a, e, 100);
    c.step_battle_execution_until(&mut s, 150).unwrap();
    hit(&mut c, e, b, 30000, 150);
    assert!(!owner(&c, a).mode);
    c.step_battle_execution_until(&mut s, 200).unwrap();
    assert!(owner(&c, a).mode);
    let events: Vec<_> = c
        .event_log
        .entries
        .iter()
        .filter(|event| event.time_ms == 200)
        .filter_map(|event| match event.event {
            crate::game::battle::event_log::BattleLogEvent::HpChanged {
                target_instance_id,
                delta,
                ..
            } if delta < 0 => Some(target_instance_id),
            _ => None,
        })
        .collect();
    assert_eq!(events, vec![e, a]);
}
#[test]
fn relation_action_runtime_retarget_does_not_put_completed_stage_wounds_on_an_unhit_target() {
    let (mut c, mut s, a, e, b) = fixture(false, &[8]);
    c.units.get_mut(&b).unwrap().owner = crate::game::enums::Side::Opponent;
    place(&mut c, b, 4, 1);
    stock(&mut c, a, 4);
    c.units.get_mut(&e).unwrap().stats.current_health = 150;
    cast(&mut c, a, false, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    attack(&mut c, a, e, 100);
    c.step_battle_execution_until(&mut s, 202).unwrap();
    assert!(c.units[&e].lifecycle.is_dead());
    assert_eq!(c.units[&b].stats.current_health, 10000);
    assert_eq!(c.hit_damage_taken_percent(b).unwrap(), 0);
    assert_eq!(owner(&c, a).stock, 2);
    c.step_battle_execution_until(&mut s, 302).unwrap();
    assert_eq!(c.units[&b].stats.current_health, 9900);
    assert_eq!(c.hit_damage_taken_percent(b).unwrap(), 5);
}
