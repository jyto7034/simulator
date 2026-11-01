const KILL_ELIGIBILITY_YAML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/kill_eligibility.skill.yaml"
));
fn eligibility_fixture(edit: impl FnOnce(&mut Value)) -> Fixture {
    let (mut core, state, actor, enemy, other) = resource_fixture("Run", |doc| {
        *doc = parse_yaml(KILL_ELIGIBILITY_YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        edit(doc);
    });
    for id in [enemy, other] {
        core.units.get_mut(&id).unwrap().threat_class =
            crate::game::battle::types::BattleUnitThreatClass::Normal;
    }
    (core, state, actor, enemy, other)
}
#[test]
fn composition_kill_eligibility_class_selects_authored_rewards() {
    use crate::game::battle::types::BattleUnitThreatClass as Class;
    for (class, grant, heal) in [
        (Class::Normal, 2, 7),
        (Class::Elite, 5, 11),
        (Class::Boss, 9, 17),
    ] {
        let (mut c, mut s, a, e, b) = eligibility_fixture(|_| {});
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().threat_class = class;
        for id in [e, b] {
            c.units.get_mut(&id).unwrap().stats.current_health = 30;
        }
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert!(c.units[&e].lifecycle.is_dead() && c.units[&b].lifecycle.is_dead());
        assert_eq!(resource_balances(&c), vec![grant]);
        assert_eq!(states(&c)[0].events, 1);
        assert_eq!(states(&c)[0].fields[0].value, StateScalar::Integer(1));
        assert_eq!(c.units[&a].stats.current_health, 500 + heal);
    }
}
#[test]
fn composition_kill_eligibility_rejects_temporary_and_unregistered_identities() {
    use crate::game::battle::types::{
        BattleUnitRole, BattleUnitSourceIdentity, IndependentUnitKind,
    };
    for case in 0..6 {
        let (mut c, mut s, a, e, b) = eligibility_fixture(|_| {});
        place(&mut c, b, 6, 3);
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        let u = c.units.get_mut(&e).unwrap();
        let base_uuid = u.base_uuid;
        match case {
            0 => {
                u.source_identity = BattleUnitSourceIdentity::SummonedUnit {
                    definition_id: "same-enemy".into(),
                    base_uuid,
                    summoner_instance_id: a,
                    summon_sequence: 1,
                    batch_uuid: Uuid::from_u128(12),
                }
            }
            1 => {
                u.source_identity = BattleUnitSourceIdentity::IndependentObject {
                    definition_id: "same-enemy".into(),
                    base_uuid,
                    creator_instance_id: a,
                    kind: IndependentUnitKind::Actor,
                }
            }
            2 => u.source_identity = BattleUnitSourceIdentity::DefenseObject { base_uuid },
            3 => u.role = BattleUnitRole::DefenseObject,
            4 => u.role = BattleUnitRole::IndependentWall,
            _ => (),
        }
        u.stats.current_health = 30;
        if case == 5 {
            std::sync::Arc::make_mut(&mut c.scenario_runtime.spawned)
                .unit_refs
                .retain(|_, id| *id != e);
        }
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        if case == 4 {
            // Independent walls are rejected by normal attack admission;
            // this path must not manufacture a death or a reward.
            assert!(c.units[&e].is_active());
            assert_eq!(c.units[&e].stats.current_health, 30);
        } else {
            assert!(c.units[&e].lifecycle.is_dead(), "case {case}");
        }
        assert!(
            resource_balances(&c).iter().all(|value| *value == 0),
            "case {case}"
        );
        assert_eq!(states(&c)[0].events, 0, "case {case}");
        assert_eq!(c.units[&a].stats.current_health, 500, "case {case}");
    }
}
#[test]
fn composition_kill_eligibility_filters_before_action_dedup() {
    use crate::game::battle::types::BattleUnitThreatClass as Class;
    for wrong_class in [false, true] {
        let (mut c, mut s, a, e, b) = eligibility_fixture(|doc| {
            let handlers = program(doc)["handlers"].as_array_mut().unwrap();
            handlers.remove(2);
            handlers.remove(0); // Keep only Elite; first rejected victim must not consume it.
            handlers[0]["event"]["BasicAttackKill"]["primary_only"] = json!(false);
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        for id in [e, b] {
            let unit = c.units.get_mut(&id).unwrap();
            unit.stats.current_health = 30;
            unit.threat_class = Class::Elite;
        }
        if wrong_class {
            c.units.get_mut(&e).unwrap().threat_class = Class::Normal;
        } else {
            std::sync::Arc::make_mut(&mut c.scenario_runtime.spawned)
                .unit_refs
                .retain(|_, id| *id != e);
        }
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert!(c.units[&e].lifecycle.is_dead() && c.units[&b].lifecycle.is_dead());
        assert_eq!(resource_balances(&c), vec![5]);
        assert_eq!(states(&c)[0].events, 1);
    }
}
#[test]
fn composition_kill_eligibility_omission_preserves_any_hostile() {
    use crate::game::battle::types::BattleUnitSourceIdentity;
    let (mut c, mut s, a, e, b) = fixture(|_| {});
    place(&mut c, b, 6, 3);
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    let u = c.units.get_mut(&e).unwrap();
    u.source_identity = BattleUnitSourceIdentity::SummonedUnit {
        definition_id: "temporary".into(),
        base_uuid: u.base_uuid,
        summoner_instance_id: a,
        summon_sequence: 1,
        batch_uuid: Uuid::from_u128(12),
    };
    u.stats.current_health = 30;
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 101).unwrap();
    assert_eq!(states(&c)[0].events, 1);
    assert_eq!(c.units[&a].stats.current_health, 600);
}
#[test]
fn composition_kill_eligibility_prepared_attack_uses_same_filters() {
    use crate::game::battle::types::BattleUnitThreatClass as Class;
    for (class, eligible, heal) in [
        (Class::Normal, true, 7),
        (Class::Elite, true, 11),
        (Class::Boss, true, 17),
        (Class::Boss, false, 0),
    ] {
        let (mut c, mut s, a, e, b) = prepared_fixture(|doc| {
            let mut definition = parse_yaml(KILL_ELIGIBILITY_YAML).unwrap();
            let mut handlers = program(&mut definition)["handlers"].clone();
            for handler in handlers.as_array_mut().unwrap() {
                handler["body"].as_array_mut().unwrap().remove(1);
            }
            prepared_program(doc)["handlers"] = handlers;
        });
        place(&mut c, b, 6, 3);
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().threat_class = class;
        c.units.get_mut(&e).unwrap().stats.current_health = 240;
        if !eligible {
            std::sync::Arc::make_mut(&mut c.scenario_runtime.spawned)
                .unit_refs
                .retain(|_, id| *id != e);
        }
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert!(preparations(&c).is_empty());
        assert_eq!(c.units[&a].stats.current_health, 500 + heal);
    }
}
#[test]
fn composition_kill_eligibility_cancellation_restores_all_and_retry_once() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for (point, count) in [
        ("encounter_kill_source", 1),
        ("state_reaction_applied", 1),
        ("state_reaction_applied", 2),
    ] {
        let (mut c, mut s, a, e, b) = eligibility_fixture(|_| {});
        place(&mut c, b, 6, 3);
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 30;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 100).unwrap();
        let snapshot = |c: &BattleCore, s: &BattleExecutionState| {
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_view()).unwrap(),
                format!("{s:?}"),
                resource_balances(c),
            )
        };
        let before = snapshot(&c, &s);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(point, count, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, 101),
            )
        });
        assert!(result.is_err() && token.is_cancelled(), "{point}");
        assert_eq!(snapshot(&c, &s), before, "{point}");
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert_eq!(resource_balances(&c), vec![2]);
        assert_eq!(states(&c)[0].events, 1);
        assert_eq!(c.units[&a].stats.current_health, 507);
    }
}
#[test]
fn composition_kill_eligibility_rejects_unknown_filter_tags() {
    let doc = parse_yaml(KILL_ELIGIBILITY_YAML).unwrap();
    let artifact = compile_json(&doc.to_string()).unwrap();
    Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    for (field, value) in [
        ("eligibility", "PendingAuthority"),
        ("threat", "PendingRank"),
    ] {
        let mut invalid = doc.clone();
        program(&mut invalid)["handlers"][0]["event"]["BasicAttackKill"][field] = json!(value);
        let errors = compile_json(&invalid.to_string()).unwrap_err();
        assert!(format!("{errors:?}").contains(value));
    }
}

#[test]
fn composition_kill_eligibility_regular_attack_and_observation_are_equivalent() {
    let mut results = Vec::new();
    for observe in [false, true] {
        let (mut c, mut s, a, e, b) = eligibility_fixture(|doc| {
            program(doc)["basic_attack"] = Value::Null;
        });
        place(&mut c, b, 6, 3);
        c.units.get_mut(&a).unwrap().basic_attack.range_policy =
            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        if observe {
            c.enable_authoring_observation_with_options(Default::default(), true);
            c.bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: 1,
                    battle_uuid: Uuid::from_u128(0xb16),
                },
            )
            .unwrap();
        }
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 30;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert_eq!(resource_balances(&c), vec![2]);
        assert_eq!(states(&c)[0].events, 1);
        assert_eq!(c.units[&a].stats.current_health, 507);
        results.push((
            states(&c),
            resource_balances(&c),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
        ));
    }
    assert_eq!(results[0], results[1]);
}
