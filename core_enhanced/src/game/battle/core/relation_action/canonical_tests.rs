use super::*;
use crate::authoring::composition::{canonical_knight_tests as source, compile_json, Artifact};
use crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::fixture_content_with_data;
use serde_json::{json, Value};

fn canonical(
    bloom: bool,
    equipped: &[&str],
    edit: impl FnOnce(&mut Value),
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let mut document = source::resolved(bloom);
    // Adapt only the technical fixture's skill identity. The published World
    // cases verify the unchanged canonical IDs and actual acquired copy.
    document["skills"][0]["id"] = json!("r11_aura");
    edit(source::program(&mut document));
    let artifact = compile_json(&document.to_string()).unwrap();
    let native = crate::authoring::node_graph::NativeGraphContent {
        skills: Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap(),
        equipment: vec![],
        mark_weapons: vec![],
    };
    let relics = source::relics();
    let indices: Vec<_> = equipped
        .iter()
        .map(|id| relics.relics.iter().position(|r| &r.id == id).unwrap())
        .collect();
    let (c, s, a, e, b) = fixture_content_with_data(
        &RELICS,
        &indices,
        native,
        |_| {},
        crate::game::resource::ResourceCatalog { resources: vec![] },
        |builder| builder.with_relics(relics),
        |_| {},
    );
    let (mut c, s, a, e, b) = configure_relation_fixture(c, s, a, e, b);
    c.enable_authoring_observation();
    (c, s, a, e, b)
}

#[test]
fn canonical_knight_real_catalog_period_gain_limit_and_shield_modifiers() {
    for bloom in [false, true] {
        for (relics, period, gain, cap) in [
            (vec![], 1000, 1, Some(4)),
            (vec!["hasty-second-hand"], 800, 1, Some(4)),
            (vec!["double-harvest"], 1000, 2, Some(4)),
            (
                vec!["hasty-second-hand", "bottomless-cup", "double-harvest"],
                800,
                2,
                None,
            ),
        ] {
            let (mut c, mut s, a, _, b) = canonical(bloom, &relics, |_| {});
            relation(&mut c, a, b, 0);
            c.step_battle_execution_until(&mut s, period - 1).unwrap();
            assert_eq!(owner(&c, a).stock, 0);
            c.step_battle_execution_until(&mut s, period).unwrap();
            assert_eq!(owner(&c, a).stock, gain);
            assert_eq!(owner(&c, a).program.stock_capacity, cap);
            c.step_battle_execution_until(&mut s, period * 3).unwrap();
            assert_eq!(
                owner(&c, a).stock,
                cap.map_or(gain * 3, |cap| (gain * 3).min(cap))
            );
        }
        let (mut c, _, a, _, b) = canonical(bloom, &["layered-shell"], |_| {});
        c.units.get_mut(&b).unwrap().stats.current_health = 1000;
        relation(&mut c, a, b, 0);
        let shield = c
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .find_map(|r| r.shield)
            .unwrap();
        assert_eq!(shield.capacity, 7500);
    }
}

#[test]
fn canonical_knight_direct_multihit_and_wound_catalog_keep_distinct_meanings() {
    for (relics, damage, wounds) in [
        (vec![], 760, 20),
        (vec!["deep-wedge"], 820, 20),
        (vec!["deep-imprint"], 820, 40),
    ] {
        let (mut c, mut s, a, e, _) = canonical(false, &relics, |_| {});
        stock(&mut c, a, 4);
        cast(&mut c, a, false, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        attack(&mut c, a, e, 100);
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert_eq!(10000 - c.units[&e].stats.current_health, damage);
        assert_eq!(c.hit_damage_taken_percent(e).unwrap(), wounds);
        let hp = c
            .authoring_observation()
            .unwrap()
            .unwrap()
            .latest_nonzero_sourced_hp
            .as_ref()
            .unwrap();
        assert_eq!(hp.fragment.as_ref().unwrap().copy_uuid, key(&c, a));
    }
}

#[test]
fn canonical_knight_bloom_periodic_damage_excludes_direct_multihit_and_expires_before_ticks() {
    for (relics, damage, wounds) in [
        (vec![], 315, 15),
        (vec!["deep-wedge"], 315, 15),
        (vec!["deep-imprint"], 330, 30),
    ] {
        let (mut c, mut s, a, e, b) = canonical(true, &relics, |_| {});
        relation(&mut c, a, b, 0);
        hit(&mut c, e, b, 20000, 0);
        cast(&mut c, a, true, 0);
        c.step_battle_execution_until(&mut s, 499).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 10000);
        c.step_battle_execution_until(&mut s, 1999).unwrap();
        assert_eq!(10000 - c.units[&e].stats.current_health, damage);
        assert_eq!(c.hit_damage_taken_percent(e).unwrap(), wounds);
        assert_eq!(c.units[&a].stats.current_health, 9000);
        c.step_battle_execution_until(&mut s, 2000).unwrap();
        assert!(owner(&c, a).active.is_none());
        assert_eq!(10000 - c.units[&e].stats.current_health, damage);
        assert_eq!(c.units[&a].stats.current_health, 9000);
        c.step_battle_execution_until(&mut s, 6500).unwrap();
        assert_eq!(c.hit_damage_taken_percent(e).unwrap(), 0);
    }
}

#[test]
fn canonical_knight_data_only_period_and_first_strike_edit_reaches_execution() {
    for (period, power) in [(1000, 100), (700, 200)] {
        let (mut c, mut s, a, e, b) = canonical(false, &[], |p| {
            p["period_ms"] = json!(period);
            p["plans"][0]["op"]["Strike"]["power"] = json!(power);
        });
        relation(&mut c, a, b, 0);
        c.step_battle_execution_until(&mut s, period - 1).unwrap();
        assert_eq!(owner(&c, a).stock, 0);
        c.step_battle_execution_until(&mut s, period).unwrap();
        assert_eq!(owner(&c, a).stock, 1);
        relation(&mut c, a, b, period);
        cast(&mut c, a, false, period);
        c.step_battle_execution_until(&mut s, period + 1).unwrap();
        attack(&mut c, a, e, period + 100);
        let hit_at = period + 100 + u64::from(c.units[&a].basic_attack.windup_ms);
        c.step_battle_execution_until(&mut s, hit_at).unwrap();
        assert_eq!(10000 - c.units[&e].stats.current_health, power);
    }
}

#[test]
fn canonical_knight_real_catalog_protection_preserves_running_action_and_manual_stop() {
    for protected in [false, true] {
        for name in ["r11_stun", "r11_silence"] {
            let (mut c, mut s, a, e, b) = canonical(true, if protected { &["unbroken-trance"] } else { &[] }, |_| {});
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
fn canonical_knight_real_catalog_retarget_preserves_remaining_costs_and_wounds() {
    for retarget in [false, true] {
        let (mut c, mut s, a, e, b) = canonical(false, if retarget { &["next"] } else { &[] }, |_| {});
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
fn canonical_knight_real_catalog_range_and_control_require_actual_hit() {
    for equipped in [&[][..], &["widened-horizon"][..], &["broken-balance"][..]] {
        let (mut c, mut s, a, e, b) = canonical(false, equipped, |_| {});
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
            if equipped == ["widened-horizon"] { 2 } else { 1 }
        );
        let debuff =
            c.hit_damage_taken_percent(e).unwrap() + c.hit_damage_taken_percent(b).unwrap();
        assert_eq!(debuff, if equipped == ["broken-balance"] { 10 } else { 0 });
    }
    let (mut c, mut s, a, e, _) = canonical(false, &["broken-balance"], |_| {});
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

fn assert_relation_intrinsic_origin(
    source: &crate::game::ability::authoring::EffectSource,
) {
    assert_eq!(source.source_file, "r11/effect.ron");
    assert_eq!(source.field_path, "skills[1].steps[0].effects[0]");
    assert_eq!(source.site.action.definition_id, "r11_aura");
    assert_eq!(source.origin_definition_id, "r11_aura");
}

fn expected_fixture_copy(c: &BattleCore, actor: UnitInstanceId) -> Uuid {
    // The scenario assigns these copies before BattleCore admission. Actor
    // selection is sorted by instance ID, so it need not be player_0.
    match c.units[&actor].source_owned_uuid.as_u128() {
        2 => Uuid::from_u128(0xd200),
        3 => Uuid::from_u128(0xd201),
        other => panic!("unexpected fixture employee {other}"),
    }
}

#[test]
fn integration_relation_intrinsic_shield_keeps_origin_through_source_exit_and_expiry() {
    let (mut c, mut state, a, _, b) = canonical(false, &[], |_| {});
    let expected_copy = expected_fixture_copy(&c, a);
    c.units.get_mut(&b).unwrap().stats.current_health = 1000;
    relation(&mut c, a, b, 0);
    let views = c.sustained_effect_snapshot().unwrap();
    let shield = views.active.iter().find(|r| r.target == b && r.shield.is_some()).unwrap();
    assert_eq!(shield.shield.as_ref().unwrap().capacity, 5000);
    assert_eq!(shield.expires_at_ms, 15000);
    assert_eq!(shield.fragment_copy_uuid, Some(expected_copy));
    assert_eq!(shield.authored_site.as_ref().expect("intrinsic shield lost its registered declaration").action.definition_id, "r11_aura");
    assert!(shield.skill_id.is_none() && shield.cast_sequence.is_none());
    let start = c.authoring_observation().unwrap().unwrap().latest_sourced_sustained.as_ref().unwrap();
    assert_relation_intrinsic_origin(start.authored.as_deref().unwrap());
    assert!(start.source.is_none() && start.equipment.is_none());
    assert_eq!(start.fragment.as_ref().unwrap().copy_uuid, expected_copy);
    assert_eq!((start.source_unit, start.target_unit, start.time_ms), (Some(a), b, 0));
    c.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: a, time_ms: 0 }).unwrap();
    c.step_battle_execution_until(&mut state, 14999).unwrap();
    assert!(c.sustained_effect_snapshot().unwrap().active.iter().any(|r| r.target == b && r.shield.is_some()));
    c.step_battle_execution_until(&mut state, 15000).unwrap();
    assert!(!c.sustained_effect_snapshot().unwrap().active.iter().any(|r| r.target == b && r.shield.is_some()));
    let end = c.authoring_observation().unwrap().unwrap().latest_sourced_sustained.as_ref().unwrap();
    assert_relation_intrinsic_origin(end.authored.as_deref().unwrap());
    assert!(end.source.is_none() && end.equipment.is_none());
    assert_eq!(end.fragment.as_ref().unwrap().copy_uuid, expected_copy);
    assert_eq!((end.source_unit, end.target_unit, end.time_ms), (Some(a), b, 15000));
}

#[test]
fn integration_relation_intrinsic_mode_and_periodic_damage_keep_own_origin() {
    for periodic in [false, true] {
        let (mut c, mut state, a, enemy, b) = canonical(periodic, &[], |_| {});
        let expected_copy = expected_fixture_copy(&c, a);
        if !periodic { stock(&mut c, a, 4); }
        relation(&mut c, a, b, 0);
        hit(&mut c, enemy, b, 20000, 0);
        assert!(owner(&c, a).mode);
        if periodic {
            cast(&mut c, a, true, 0);
            c.step_battle_execution_until(&mut state, 1001).unwrap();
        }
        let expected_loss = if periodic { 1000 } else { 2000 };
        assert_eq!(c.units[&a].stats.current_health, 10000 - expected_loss);
        let fact = c.runtime_fact_summary().latest_damage.as_ref().unwrap();
        assert_eq!(fact.target, a);
        assert_eq!(fact.actual_hp_loss(), expected_loss);
        assert_eq!(fact.source.actor, Some(a));
        assert_eq!(fact.source.fragment.as_ref().unwrap().copy_uuid, expected_copy);
        assert_relation_intrinsic_origin(fact.source.authored.as_deref().expect("intrinsic self damage lost its registered declaration"));
        assert!(fact.source.cast.is_none() && fact.source.action.is_none());
        assert!(fact.source.equipment.is_none() && fact.source.delivery.is_none());
        let observed = c.authoring_observation().unwrap().unwrap().latest_nonzero_sourced_hp.as_ref().unwrap();
        assert_eq!((observed.source_unit, observed.target_unit), (Some(a), a));
        assert_eq!(observed.change.hp_delta(), -i64::from(expected_loss));
        assert_relation_intrinsic_origin(observed.authored.as_deref().unwrap());
        assert!(observed.source.is_none() && observed.equipment.is_none());
        assert_eq!(observed.fragment.as_ref().unwrap().copy_uuid, expected_copy);
    }
}

#[test]
fn integration_relation_intrinsic_callbacks_restore_failed_attempt_and_ignore_observation_mode() {
    use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
    use crate::authoring::trace::{TraceAttempt, TraceLimits, TraceOptions};
    for (case, fault_site) in [(0, "relation_emergency_shield"), (1, "relation_mode_entered"), (2, "relation_self_damage")] {
        let (mut base, mut base_state, a, enemy, b) = canonical(case == 2, &[], |_| {});
        let expected_copy = expected_fixture_copy(&base, a);
        base.authoring_observation = None;
        if case == 0 {
            base.units.get_mut(&b).unwrap().stats.current_health = 1000;
        } else {
            if case == 1 { stock(&mut base, a, 4); }
            relation(&mut base, a, b, 0);
            if case == 2 {
                hit(&mut base, enemy, b, 20000, 0);
                cast(&mut base, a, true, 0);
                base.step_battle_execution_until(&mut base_state, 999).unwrap();
            }
        }
        let trigger = |c: &mut BattleCore, s: &mut BattleExecutionState| -> Result<(), GameError> {
            match case {
                0 => c.apply_live_command(BattleLiveCommand::SelectRelation {
                    unit_id: a, entry: "blessing".into(), target: b, time_ms: 0,
                }).map(|_| ()),
                // Same committed damage boundary as the fixture hit helper,
                // but return the actual error instead of unwrapping it.
                1 => c.runtime_transaction(|c| {
                    let source = c.damage_source_snapshot_for_unit(
                        enemy, b, DamageSource::Ability,
                        crate::game::battle::damage::DamageType::True,
                        20000, Default::default(), 0, 0, false,
                    ).ok_or(GameError::InvalidAction)?;
                    c.process_commands(vec![crate::game::battle::damage::BattleCommand::ApplyDamage {
                        target_id: b, source_snapshot: source,
                    }], 0);
                    crate::game::battle::runtime_control::poll()
                }),
                _ => c.step_battle_execution_until(s, 1000).map(|_| ()),
            }
        };
        let mut unobserved = None;
        for observe in [false, true] {
            let mut c = base.clone();
            let mut s = base_state.clone();
            let options = TraceOptions { limits: Some(TraceLimits::default()) };
            if observe { c.enable_authoring_observation_with_trace(options); }
            let _trace = observe.then(|| TraceAttempt::enter(options).unwrap());
            let snapshot = |c: &BattleCore, s: &BattleExecutionState| (
                c.transaction_debug_snapshot(), c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_snapshot().unwrap()).unwrap(),
                serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),
                c.authoring_trace.view().ndjson.to_vec(), c.event_log.to_json_string().unwrap(),
                format!("{s:?}"),
            );
            let before = snapshot(&c, &s);
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(fault_site, 1, || {
                control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution,
                    || trigger(&mut c, &mut s))
            });
            assert!(token.is_cancelled() && result.is_err(), "{case}: {result:?}");
            assert_eq!(before, snapshot(&c, &s), "{case}: failed attempt leaked state or witness");
            trigger(&mut c, &mut s).unwrap();
            if case == 0 {
                let shields = c.sustained_effect_snapshot().unwrap();
                let shields: Vec<_> = shields.active.iter().filter(|r| r.target == b && r.shield.is_some()).collect();
                assert_eq!(shields.len(), 1);
                assert_eq!(shields[0].shield.as_ref().unwrap().capacity, 5000);
                assert_eq!(shields[0].fragment_copy_uuid, Some(expected_copy));
                assert!(owner(&c, a).shield_used);
            } else {
                let loss = if case == 1 { 2000 } else { 1000 };
                assert_eq!(c.units[&a].stats.current_health, 10000 - loss);
                let damage = c.runtime_fact_summary().latest_damage.as_ref().unwrap();
                assert_eq!(damage.actual_hp_loss(), loss);
                assert_eq!(damage.source.fragment.as_ref().unwrap().copy_uuid, expected_copy);
                assert_relation_intrinsic_origin(damage.source.authored.as_deref().unwrap());
                assert!(damage.source.cast.is_none() && damage.source.action.is_none());
            }
            if observe {
                let trace = c.authoring_trace.view();
                assert!(trace.gap.is_none());
                let rows: Vec<Value> = std::str::from_utf8(trace.ndjson).unwrap().lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .filter(|row: &Value| row["fragment"]["copy_uuid"] == json!(expected_copy)
                        && row["change"]["channel"] == if case == 0 { "sustained" } else { "hp" })
                    .collect();
                assert_eq!(rows.len(), if case == 2 { 2 } else { 1 }, "{case}: {rows:?}");
                if case == 2 {
                    // The same timestamp also lands the channel's ordinary
                    // enemy hit. It keeps its cast; autonomous self damage does not.
                    let strike = rows.iter().find(|row| row["target_unit"] == json!(enemy)).unwrap();
                    assert_eq!(strike["change"]["before"], 9900);
                    assert_eq!(strike["change"]["after"], 9795);
                    assert_eq!(strike["source_unit"], json!(a));
                    assert_eq!(strike["source"]["cast_seq"], 0);
                    assert_eq!(strike["source"]["authored"]["source_file"], "r11/effect.ron");
                    assert_eq!(strike["source"]["authored"]["field_path"], "skills[1].steps[0].effects[0]");
                }
                let rows: Vec<_> = rows.iter().filter(|row| row["target_unit"] == json!(if case == 0 { b } else { a })).collect();
                assert_eq!(rows.len(), 1, "{case}: retry must publish exactly one intrinsic application: {rows:?}");
                assert_eq!(rows[0]["authored"]["source_file"], "r11/effect.ron");
                assert_eq!(rows[0]["authored"]["field_path"], "skills[1].steps[0].effects[0]");
                assert_eq!(rows[0]["source_unit"], json!(a));
                assert_eq!(rows[0]["target_unit"], json!(if case == 0 { b } else { a }));
                assert!(rows[0]["source"].is_null() && rows[0]["equipment"].is_null());
            }
            let outcome = (
                c.transaction_debug_snapshot(), c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_snapshot().unwrap()).unwrap(),
                c.event_log.to_json_string().unwrap(), format!("{s:?}"),
            );
            if let Some(expected) = &unobserved { assert_eq!(expected, &outcome, "{case}"); }
            else { unobserved = Some(outcome); }
        }
    }
}
