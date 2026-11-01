use crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::{
    primary_fragment_input, enable_complete_trace, hp_trace_rows, assert_fragment_trace_origin,
};
use super::*;
use crate::game::battle::core::{
    boss_runtime::tests::compiled_action_fixture::{cast, fixture_graph, hit, place},
    sim::{BattleExecutionState, BattleLiveCommand},
};
use serde_json::{json, Value};
include!("canonical_tests.inc.rs");
const RELICS: [&str; 7] = [
    "long-breath",
    "widened-horizon",
    "double-harvest",
    "unbroken-trance",
    "deep-wedge",
    "king-of-greed-stunning-treasures",
    "king-of-greed-endless-treasury",
];
fn fixture(
    relics: &[usize],
    edit: impl FnOnce(&mut Value),
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut doc = crate::authoring::node_graph::tests::prepared_volley_fixture();
    edit(&mut doc);
    let (mut c, s, a, e, b) = fixture_graph(&RELICS, relics, doc);
    c.units.get_mut(&b).unwrap().fragment_source = None;
    c.units.get_mut(&b).unwrap().skill_id = None;
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    for id in [a, e, b] {
        let u = c.units.get_mut(&id).unwrap();
        u.stats.max_health = 10000;
        u.stats.current_health = 10000;
        u.stats.defense = 0;
    }
    c.runtime_transaction(|c| {
        c.clear_prepared_volleys(0)?;
        c.sustained.volleys = Default::default();
        c.reconcile_prepared_volleys(0)
    })
    .unwrap();
    (c, s, a, e, b)
}
fn key(c: &BattleCore, a: UnitInstanceId) -> Uuid {
    c.units[&a].fragment_source.as_ref().unwrap().copy_uuid
}
fn step(c: &mut BattleCore, s: &mut BattleExecutionState, now: u64) {
    c.step_battle_execution_until(s, now).unwrap();
}
fn release(c: &mut BattleCore, a: UnitInstanceId, now: u64) {
    let activation = c.prepared_volley_reinput(a, &"r11_aura".into()).unwrap();
    c.apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
        unit_id: a,
        skill_id: "r11_aura".into(),
        activation,
        target: None,
        time_ms: now,
    })
    .unwrap();
}
fn value(doc: &mut Value, kind: &str, field: &str, val: Value) {
    for n in doc["nodes"].as_array_mut().unwrap() {
        if n["kind"] == kind {
            n["values"][field] = val.clone();
        }
    }
}
#[test]
fn prepared_volley_runtime_accelerates_pays_on_creation_and_decays_without_refund() {
    let (mut c, mut s, a, _, _) = fixture(&[], |_| {});
    let k = key(&c, a);
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 999);
    assert_eq!(c.sustained.volleys.owners[&k].balance, 10);
    step(&mut c, &mut s, 1000);
    assert_eq!(c.sustained.volleys.owners[&k].balance, 9);
    assert_eq!(
        c.sustained.volleys.owners[&k]
            .active
            .as_ref()
            .unwrap()
            .prepared,
        1
    );
    step(&mut c, &mut s, 1749);
    assert_eq!(c.sustained.volleys.owners[&k].balance, 9);
    step(&mut c, &mut s, 1750);
    assert_eq!(c.sustained.volleys.owners[&k].balance, 8);
    step(&mut c, &mut s, 2250);
    assert_eq!(
        c.sustained.volleys.owners[&k]
            .active
            .as_ref()
            .unwrap()
            .deadline,
        4250
    );
    step(&mut c, &mut s, 4250);
    assert_eq!(
        c.sustained.volleys.owners[&k]
            .active
            .as_ref()
            .unwrap()
            .prepared,
        2
    );
    step(&mut c, &mut s, 6250);
    assert!(c.sustained.volleys.owners[&k].active.is_none());
    assert_eq!(c.sustained.volleys.owners[&k].balance, 7);
}
#[test]
fn prepared_volley_runtime_reinput_serializes_flights_and_stops_after_frozen_count() {
    let (mut c, mut s, a, e, _) = fixture(&[], |_| {});
    let k = key(&c, a);
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1750);
    release(&mut c, a, 1750);
    assert_eq!(c.sustained.volleys.flights.len(), 1);
    assert_eq!(
        c.sustained.volleys.owners[&k]
            .active
            .as_ref()
            .unwrap()
            .prepared,
        1
    );
    step(&mut c, &mut s, 2000);
    assert_eq!(c.sustained.volleys.settled.len(), 0);
    step(&mut c, &mut s, 3000);
    let v = c.prepared_volleys_snapshot().unwrap();
    assert_eq!(v.settled.len(), 2);
    assert_eq!(v.settled[1].released_at_ms, v.settled[0].ended_at_ms);
    assert_eq!(c.units[&e].stats.current_health, 9800);
    assert!(c.sustained.volleys.owners[&k].active.is_none());
    assert_eq!(c.sustained.volleys.owners[&k].balance, 8);
}
#[test]
fn prepared_volley_runtime_marked_death_by_other_source_recovers_once_after_lethal_mark() {
    let (mut c, _, a, e, b) = fixture(&[2], |_| {});
    let k = key(&c, a);
    hit(&mut c, a, e, 1, 0);
    assert_eq!(c.prepared_volleys_snapshot().unwrap().marks.len(), 1);
    hit(&mut c, b, e, 9999, 1);
    assert_eq!(c.sustained.volleys.owners[&k].balance, 12);
    assert!(c.prepared_volleys_snapshot().unwrap().marks.is_empty());
    c.runtime_transaction(|c| c.volley_target_died(e, 1))
        .unwrap();
    assert_eq!(c.sustained.volleys.owners[&k].balance, 12);
    let (mut c, _, a, e, _) = fixture(&[], |_| {});
    let k = key(&c, a);
    hit(&mut c, a, e, 10000, 0);
    assert_eq!(c.sustained.volleys.owners[&k].balance, 11);
}
#[test]
fn prepared_volley_runtime_continuous_cost_ignores_stock_and_lethal_boundary_precedes_release() {
    let (mut c, mut s, a, e, _) = fixture(&[6], |d| value(d, "skill-seconds", "value", json!(1)));
    let k = key(&c, a);
    // Reach a legitimate debt carry from the previous battle; this use cannot borrow.
    let mut ledger = CreditLedgers::default();
    ledger.entries.insert(
        k,
        CreditCarry {
            definition: c.sustained.volleys.owners[&k]
                .program
                .ledger
                .definition
                .clone(),
            unpaid: 20,
            borrowed: true,
        },
    );
    c.mount_credit_ledgers(&ledger).unwrap();
    c.units.get_mut(&a).unwrap().stats.current_health = 500;
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 0);
    assert_eq!(c.sustained.volleys.flights.len(), 1);
    step(&mut c, &mut s, 1000);
    assert_eq!(c.units[&a].stats.current_health, 0);
    assert!(c.sustained.volleys.owners[&k].active.is_none());
    assert_eq!(c.sustained.volleys.owners[&k].balance, -10);
    assert!(!c.sustained.volleys.owners[&k].borrowed);
    assert_eq!(c.units[&e].stats.current_health, 9900);
    assert_eq!(c.prepared_volleys_snapshot().unwrap().settled.len(), 1);
}
#[test]
fn prepared_volley_runtime_direct_stop_discards_prepared_but_released_flight_finishes() {
    let (mut c, mut s, a, e, _) = fixture(&[], |_| {});
    let k = key(&c, a);
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1750);
    release(&mut c, a, 1750);
    cast(&mut c, a, 1750);
    assert!(c.sustained.volleys.owners[&k].active.is_none());
    assert_eq!(c.sustained.volleys.flights.len(), 1);
    step(&mut c, &mut s, 4000);
    assert_eq!(c.units[&e].stats.current_health, 9900);
    assert_eq!(c.prepared_volleys_snapshot().unwrap().settled.len(), 1);
    assert_eq!(c.sustained.volleys.owners[&k].balance, 8);
}
#[test]
fn prepared_volley_runtime_original_target_loss_does_not_retarget_in_flight() {
    let (mut c, mut s, a, e, b) = fixture(&[], |_| {});
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1000);
    release(&mut c, a, 1000);
    hit(&mut c, b, e, 10000, 1001);
    step(&mut c, &mut s, 1300);
    assert_eq!(
        c.prepared_volleys_snapshot().unwrap().settled[0].end,
        VolleyFlightEnd::TargetLost
    );
}
#[test]
fn prepared_volley_runtime_native_clone_and_dto_roundtrip_preserve_active_state() {
    let (mut c, mut s, a, _, _) = fixture(&[], |_| {});
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1750);
    release(&mut c, a, 1750);
    let copied = c.clone();
    assert_eq!(
        c.prepared_volleys_snapshot().unwrap(),
        copied.prepared_volleys_snapshot().unwrap()
    );
    let snapshot = c.prepared_volleys_snapshot().unwrap();
    assert_eq!(
        snapshot,
        serde_json::from_str::<PreparedVolleysSnapshot>(&serde_json::to_string(&snapshot).unwrap())
            .unwrap()
    );
}
#[test]
fn prepared_volley_runtime_empty_target_continuous_keeps_cost_and_finite_discards_remaining() {
    for endless in [false, true] {
        let (mut c, mut s, a, e, _) = fixture(if endless { &[6] } else { &[] }, |_| {});
        place(&mut c, e, 7, 1);
        let k = key(&c, a);
        cast(&mut c, a, 0);
        step(&mut c, &mut s, 1000);
        if !endless {
            release(&mut c, a, 1000);
        }
        step(&mut c, &mut s, 2000);
        assert!(c.sustained.volleys.flights.is_empty());
        assert_eq!(
            c.units[&a].stats.current_health,
            if endless { 9000 } else { 10000 }
        );
        assert_eq!(c.sustained.volleys.owners[&k].active.is_some(), endless);
    }
}

#[test]
fn prepared_volley_runtime_credit_floor_and_deadline_keep_borrow_history_after_repayment() {
    let (mut c, mut s, a, e, _) = fixture(&[], |d| {
        value(d, "skill-credit-ledger", "grant", json!(1));
        value(d, "skill-credit-ledger", "creditLimit", json!(1));
    });
    let k = key(&c, a);
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 9999);
    let o = &c.sustained.volleys.owners[&k];
    assert_eq!(o.balance, -1);
    assert!(o.borrowed);
    assert_eq!(o.active.as_ref().unwrap().prepared, 2);
    step(&mut c, &mut s, 10000);
    assert_eq!(
        c.sustained.volleys.owners[&k]
            .active
            .as_ref()
            .unwrap()
            .prepared,
        1
    );
    hit(&mut c, a, e, 10000, 10000);
    assert_eq!(c.sustained.volleys.owners[&k].balance, 0);
    assert!(c.sustained.volleys.owners[&k].borrowed);
    c.runtime_transaction(|c| c.clear_prepared_volleys(10000))
        .unwrap();
    let r = c.credit_settlements().unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].after.unpaid, 0);
    assert!(r[0].after.borrowed);
    let mut ledger = CreditLedgers::default();
    ledger.apply(&r).unwrap();
    assert!(ledger.apply(&r).is_err());
    c.runtime_transaction(|c| {
        c.sustained.volleys = Default::default();
        c.reconcile_prepared_volleys(10000)?;
        c.mount_credit_ledgers(&ledger)
    })
    .unwrap();
    assert_eq!(c.sustained.volleys.owners[&k].balance, 1);
    assert!(!c.sustained.volleys.owners[&k].can_borrow);
}
#[test]
fn prepared_volley_runtime_serial_selection_uses_fresh_marks_and_impact_barrier() {
    let (mut c, mut s, a, e, _) = fixture(&[], |_| {});
    let other = UnitInstanceId(Uuid::from_u128(0xabc101));
    let mut unit = c.units[&e].clone();
    unit.instance_id = other;
    unit.spawn_order += 1;
    c.units.insert(other, unit);
    place(&mut c, other, 5, 1);
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1750);
    release(&mut c, a, 1750);
    step(&mut c, &mut s, 3300);
    let settled = c.prepared_volleys_snapshot().unwrap().settled;
    assert_eq!(settled.len(), 2);
    assert_ne!(settled[0].target, settled[1].target);
    assert_eq!(settled[1].released_at_ms, settled[0].ended_at_ms);
    assert_eq!(c.units[&e].stats.current_health, 9900);
    assert_eq!(c.units[&other].stats.current_health, 9900);
}
#[test]
fn prepared_volley_runtime_shield_only_damage_and_expired_marks_do_not_recover() {
    for shield in [false, true] {
        let (mut c, _, a, e, b) = fixture(&[], |_| {});
        let k = key(&c, a);
        if shield {
            c.begin_sustained(
                e,
                e,
                crate::game::ability::sustained::SustainedEffectDef::Shield {
                    duration_ms: 10000,
                    amount: 1,
                    priority: 0,
                    shared_radius_units: None,
                },
                0,
            )
            .unwrap();
        }
        hit(&mut c, a, e, 1, 0);
        assert_eq!(
            c.prepared_volleys_snapshot().unwrap().marks.is_empty(),
            shield
        );
        hit(&mut c, b, e, 10000, 5000);
        assert_eq!(c.sustained.volleys.owners[&k].balance, 10);
    }
}
#[test]
fn prepared_volley_runtime_multihit_status_and_withdrawal_preserve_released_source() {
    let (mut c, mut s, a, e, _) = fixture(&[4, 5], |_| {});
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1750);
    release(&mut c, a, 1750);
    step(&mut c, &mut s, 3000);
    assert_eq!(c.units[&e].stats.current_health, 9794);
    assert!(c.hit_stun_until(e, Some(3000)).is_some());
    assert!(!c.resonance_use_has_obligations(a));
    let (mut c, mut s, a, e, _) = fixture(&[5], |_| {});
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1000);
    release(&mut c, a, 1000);
    c.apply_live_command(BattleLiveCommand::WithdrawUnit {
        unit_id: a,
        time_ms: 1000,
    })
    .unwrap();
    step(&mut c, &mut s, 1400);
    assert_eq!(c.units[&e].stats.current_health, 9900);
    assert!(c.hit_stun_until(e, Some(1400)).is_some());
    assert_eq!(c.prepared_volleys_snapshot().unwrap().marks.len(), 1);
}
#[test]
fn prepared_volley_runtime_external_interruption_protection_silence_and_manual_stop() {
    use crate::game::battle::{buffs::BuffId, damage::BattleCommand};
    for protected in [false, true] {
        for name in ["r11_stun", "r11_silence"] {
            let (mut c, mut s, a, e, _) = fixture(if protected { &[3] } else { &[] }, |_| {});
            let k = key(&c, a);
            cast(&mut c, a, 0);
            step(&mut c, &mut s, 1000);
            c.runtime_transaction(|c| {
                c.process_commands(
                    vec![BattleCommand::ApplyBuff {
                        caster_id: e,
                        target_id: a,
                        buff_id: BuffId::from_name(name),
                        duration_ms: 300,
                    }],
                    1000,
                );
                control::poll("volley_fixture_cc")
            })
            .unwrap();
            step(&mut c, &mut s, 1001);
            let stays = protected || name == "r11_silence";
            assert_eq!(
                c.sustained.volleys.owners[&k].active.is_some(),
                stays,
                "{protected} {name}"
            );
            if stays {
                let activation = c.sustained.volleys.owners[&k].active.as_ref().unwrap().id;
                assert!(c
                    .apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
                        unit_id: a,
                        skill_id: "r11_aura".into(),
                        activation,
                        target: None,
                        time_ms: 1001
                    })
                    .is_err());
                cast(&mut c, a, 1001);
                assert!(c.sustained.volleys.owners[&k].active.is_none());
            }
        }
    }
}
#[test]
fn prepared_volley_runtime_cancellation_restores_cost_queue_damage_and_delivery() {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "prepared_volley_started",
        "prepared_volley_portal_paid",
        "volley_projectile_released",
        "volley_damage_settled",
        "volley_impact_barrier",
    ] {
        let (mut c, mut s, a, _, _) = fixture(&[], |_| {});
        let starting = site == "prepared_volley_started";
        let payment = site == "prepared_volley_portal_paid";
        let releasing = site == "volley_projectile_released";
        cast(&mut c, a, 0);
        if !starting {
            step(&mut c, &mut s, if payment { 999 } else { 1000 });
            if !payment && !releasing {
                release(&mut c, a, 1000);
                // 2 tiles minus 0.25 projectile radius and 0.35 target radius, at 5 tiles/s: impact at 1280ms.
                step(&mut c, &mut s, 1279);
            }
        }
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            c.prepared_volleys_snapshot().unwrap(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let activation = c.prepared_volley_reinput(a, &"r11_aura".into());
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    if starting {
                        c.step_battle_execution_until(&mut s, 0).map(|_| ())
                    } else if releasing {
                        c.apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
                            unit_id: a,
                            skill_id: "r11_aura".into(),
                            activation: activation.unwrap(),
                            target: None,
                            time_ms: 1000,
                        })
                        .map(|_| ())
                    } else {
                        c.step_battle_execution_until(&mut s, if payment { 1000 } else { 1280 })
                            .map(|_| ())
                    }
                },
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
                c.prepared_volleys_snapshot().unwrap(),
                format!("{s:?}")
            ),
            "{site}"
        );
        if starting {
            step(&mut c, &mut s, 0);
        } else if releasing {
            release(&mut c, a, 1000);
        } else {
            step(&mut c, &mut s, if payment { 1000 } else { 1280 });
        }
    }
}

#[test]
fn prepared_volley_runtime_air_obstacle_range_lifetime_and_terminal_close_delivery() {
    for case in 0..5 {
        let (mut c, mut s, a, e, _) = fixture(&[], |d| {
            if case == 2 {
                value(d, "skill-projectile-profile", "range", json!(1));
            }
            if case == 3 {
                for n in d["nodes"].as_array_mut().unwrap() {
                    if n["title"].as_str().unwrap().contains("비행") {
                        n["values"]["value"] = json!(0.1);
                    }
                }
            }
        });
        c.units.get_mut(&e).unwrap().mobility_kind =
            crate::game::battle::types::MobilityKind::Airborne;
        if case == 1 {
            Arc::make_mut(&mut c.scenario)
                .battlefield
                .obstacles
                .push(crate::game::resources::Position::new(3, 1));
        }
        cast(&mut c, a, 0);
        step(&mut c, &mut s, 1000);
        release(&mut c, a, 1000);
        if case == 4 {
            c.runtime_transaction(|c| c.clear_prepared_volleys(1000))
                .unwrap();
        } else {
            step(&mut c, &mut s, 1600);
        }
        let v = c.prepared_volleys_snapshot().unwrap();
        assert_eq!(v.settled.len(), 1, "{case}");
        assert_eq!(
            v.settled[0].end,
            match case {
                0 => VolleyFlightEnd::Impact,
                1 => VolleyFlightEnd::Obstacle,
                2 => VolleyFlightEnd::Range,
                3 => VolleyFlightEnd::Lifetime,
                _ => VolleyFlightEnd::BattleEnded,
            },
            "{case}"
        );
        assert_eq!(
            c.units[&e].stats.current_health,
            if case == 0 { 9900 } else { 10000 }
        );
        assert!(!c.resonance_use_has_obligations(a), "{case}");
    }
}
#[test]
fn prepared_volley_runtime_duration_area_modifiers_and_redeployment_keep_copy_balance() {
    let (mut c, mut s, a, e, _) = fixture(&[0, 1], |d| {
        for n in d["nodes"].as_array_mut().unwrap() {
            if n["title"].as_str().unwrap().contains("확대 타일") {
                n["values"]["pattern"] = json!("XXX\nX@X\nXXX");
            }
        }
    });
    let other = UnitInstanceId(Uuid::from_u128(0xabc102));
    let mut unit = c.units[&e].clone();
    unit.instance_id = other;
    unit.spawn_order += 1;
    c.units.insert(other, unit);
    place(&mut c, other, 4, 2);
    let copy = key(&c, a);
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1000);
    release(&mut c, a, 1000);
    step(&mut c, &mut s, 1600);
    assert_eq!(c.units[&e].stats.current_health, 9900);
    assert_eq!(c.units[&other].stats.current_health, 9900);
    let v = c.prepared_volleys_snapshot().unwrap();
    assert_eq!(v.marks.len(), 2);
    assert!(v
        .marks
        .iter()
        .all(|m| m.2 == v.settled[0].ended_at_ms + 6250));
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
        time_ms: 1600,
    })
    .unwrap();
    let super::super::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id: new } = c
        .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
            draft,
            position: crate::game::resources::Position::new(2, 1),
            tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
            facing: crate::game::battle::tile_range::FacingDirection::Right,
            instance_salt: 4242,
            time_ms: 1600,
            current_hp_policy: None,
        })
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(key(&c, new), copy);
    assert_eq!(c.sustained.volleys.owners[&copy].balance, 9);
    assert_eq!(c.sustained.volleys.owners[&copy].caster, Some(new));
}

#[test]
fn prepared_volley_runtime_air_exclusion_applies_to_selection_and_impact_separately() {
    for selection in [false, true] {
        let (mut c, mut s, a, e, _) = fixture(&[], |d| {
            value(
                d,
                if selection {
                    "skill-enemy-selector"
                } else {
                    "skill-projectile-profile"
                },
                "air",
                json!("exclude"),
            )
        });
        c.units.get_mut(&e).unwrap().mobility_kind =
            crate::game::battle::types::MobilityKind::Airborne;
        cast(&mut c, a, 0);
        step(&mut c, &mut s, 1000);
        release(&mut c, a, 1000);
        step(&mut c, &mut s, 1600);
        assert_eq!(c.units[&e].stats.current_health, 10000);
        let v = c.prepared_volleys_snapshot().unwrap();
        assert!(v.marks.is_empty());
        if selection {
            assert!(v.settled.is_empty());
        } else {
            assert_eq!(v.settled[0].end, VolleyFlightEnd::TargetLost);
        }
    }
}

#[test]
fn basic_delayed_tile_with_prepared_volley_preserves_stock_and_marks_for_recovery() {
    use crate::game::battle::{enums::BattleEvent, event_log::BattleLogEvent};
    let equipment: crate::game::data::equipment_data::EquipmentDatabase = ron::de::from_str(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../game_resources/data/equipments/base.ron")),
    ).unwrap();
    let profile = equipment.get_by_id("placeholder_king_of_greed_weapon").unwrap().weapon_profile.clone().unwrap();
    let (mut c, mut s, a, e, b) = crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::fixture_graph_with_weapon(
        &RELICS, &[], crate::authoring::node_graph::tests::prepared_volley_fixture(),
        |weapon| weapon.weapon_profile = Some(profile),
    );
    c.units.get_mut(&b).unwrap().fragment_source = None;
    c.units.get_mut(&b).unwrap().skill_id = None;
    Arc::make_mut(&mut c.scenario).terminal_outcomes =
        crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(120000);
    for u in c.units.values_mut() {
        u.stats.max_health = 10000;
        u.stats.current_health = 10000;
        u.stats.defense = 0;
        u.stats.attack_interval_ms = 100000;
        u.next_basic_attack_ms = 100000;
        u.stats.move_speed_units_per_ms = 0;
        u.body.move_speed = 0.0;
    }
    c.units.get_mut(&a).unwrap().stats.attack = 100;
    for (id,x,y) in [(a,2,1),(e,3,1),(b,7,3)] { place(&mut c,id,x,y); }
    c.runtime_transaction(|c| {
        c.clear_prepared_volleys(0)?;
        c.sustained.volleys = Default::default();
        c.reconcile_prepared_volleys(0)
    }).unwrap();
    let copy = key(&c,a);
    assert_eq!(c.sustained.volleys.owners[&copy].balance,10);
    c.units.get_mut(&a).unwrap().next_basic_attack_ms = 10;
    c.process_event(BattleEvent::AttackStart {time_ms:10,attacker_instance_id:a,
        target_instance_id:Some(e),schedule_next:true,cause:Default::default()},10).unwrap();
    step(&mut c,&mut s,610);
    assert_eq!(c.sustained.volleys.owners[&copy].balance,10);
    assert!(c.sustained.volleys.owners[&copy].active.is_none());
    assert!(!c.sustained.volleys.owners[&copy].borrowed);
    assert_eq!(c.units[&e].stats.current_health,10000);
    step(&mut c,&mut s,1010);
    assert_eq!(c.sustained.volleys.owners[&copy].balance,10);
    assert!(c.units[&e].stats.current_health<10000);
    assert!(c.sustained.volleys.marks.contains_key(&(copy,e)));
    assert_eq!(c.event_log.entries.iter().filter(|entry| matches!(entry.event,
        BattleLogEvent::BasicAttackTileImpacted{attacker_instance_id,..} if attacker_instance_id==a)).count(),1);
    // The fragment's passive recovers from another actor's later kill. The bow
    // creates no treasure and pays no stock; its ordinary owned hit can mark.
    hit(&mut c,b,e,10000,1020);
    assert_eq!(c.sustained.volleys.owners[&copy].balance,11);
}

#[test]
fn integration_volley_delayed_hp_keeps_original_cast_and_fragment() {
    let (mut c, mut s, a, e, _) = fixture(&[], |_| {});
    let mut off = controlled_clone(&c).unwrap();
    let mut off_s = s.clone();
    let (_trace, trace_from) = enable_complete_trace(&mut c);
    let copy = primary_fragment_input(&c, a);
    cast(&mut c, a, 0);
    step(&mut c, &mut s, 1750);
    release(&mut c, a, 1750);
    step(&mut c, &mut s, 3000);
    assert_eq!(c.units[&e].stats.current_health, 9800);
    let observations = c.authoring_observation().unwrap().unwrap();
    let hp = observations.latest_nonzero_sourced_hp.as_ref().unwrap();
    assert_eq!(hp.target_unit, e);
    assert_eq!(hp.fragment.as_ref().unwrap().copy_uuid, copy);
    assert_eq!(hp.source.as_ref().expect("released volley retains cast provenance").cast.skill_id.as_str(), "r11_aura");
    assert_eq!(hp.source.as_ref().unwrap().cast.cast_seq, 0);
    assert_eq!(hp.source.as_ref().unwrap().cast.caster_instance_id, a);
    let rows = hp_trace_rows(&c, trace_from);
    assert_eq!(rows.len(), 2);
    for row in &rows { assert_fragment_trace_origin(row, a, Some(("r11_aura", 0))); }
    cast(&mut off, a, 0);
    step(&mut off, &mut off_s, 1750);
    release(&mut off, a, 1750);
    step(&mut off, &mut off_s, 3000);
    assert_eq!(c.units[&e].stats.current_health, off.units[&e].stats.current_health);
    assert_eq!(serde_json::to_value(&c.event_log.entries).unwrap(), serde_json::to_value(&off.event_log.entries).unwrap());
    assert_eq!(serde_json::to_value(c.runtime_fact_snapshot().unwrap()).unwrap(), serde_json::to_value(off.runtime_fact_snapshot().unwrap()).unwrap());
}
