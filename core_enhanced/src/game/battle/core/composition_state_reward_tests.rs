// Included inside composition_basic_kill to share its real YAML/published pack fixture.
const REWARD_YAML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/state_reward.skill.yaml"
));
fn reward_fixture(edit: impl FnOnce(&mut Value)) -> Fixture {
    fixture(|doc| {
        *doc = parse_yaml(REWARD_YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        edit(doc);
    })
}
fn reward_op(doc: &mut Value) -> &mut Value {
    &mut program(doc)["handlers"][0]["body"][1]["ApplyOwnerState"]
}
fn rewards(c: &BattleCore) -> Vec<crate::game::ability::sustained::SustainedEffectView> {
    c.sustained_effect_snapshot()
        .unwrap()
        .active
        .into_iter()
        .filter(|r| r.target_state.is_some())
        .collect()
}

#[test]
fn composition_state_reward_compile_rejects_units_profiles_and_roundtrips() {
    let doc = parse_yaml(REWARD_YAML).unwrap();
    let a = compile_json(&doc.to_string()).unwrap();
    Artifact::from_ron(&a.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    for (case, expected) in [
        (0, "Milliseconds"),
        (1, "nonempty"),
        (2, "nonempty"),
        (3, "Live"),
    ] {
        let mut d = doc.clone();
        match case {
            0 => reward_op(&mut d)["duration"]["root"]["Const"]["unit"] = json!("Count"),
            1 => reward_op(&mut d)["profile"]["attack_percent"] = json!(0),
            2 => reward_op(&mut d)["profile"]["definition"] = json!(""),
            _ => reward_op(&mut d)["duration"]["moment"] = json!("Snapshot"),
        }
        let err = compile_json(&d.to_string()).unwrap_err().to_string();
        assert!(err.contains(expected), "{case}: {err}");
        assert!(err.contains("handlers[0].body[1]"), "{err}");
    }
}

#[test]
fn composition_state_reward_parent_end_independent_deadline_and_future_attack() {
    let (mut c, mut s, a, e, b) = reward_fixture(|doc| {
        program(doc)["handlers"][0]["body"]
            .as_array_mut()
            .unwrap()
            .push(json!("End"));
    });
    cast(&mut c, a, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    c.units.get_mut(&e).unwrap().stats.current_health = 30;
    start(&mut c, a, e, 100, true);
    c.step_battle_execution_until(&mut s, 101).unwrap();
    assert!(c.units[&e].lifecycle.is_dead());
    // The sibling hit in the same line retains its captured 40 base power.
    assert_eq!(c.units[&b].stats.current_health, 960);
    assert_eq!(c.units[&a].stats.attack, 125);
    assert!(states(&c).is_empty());
    assert_eq!(rewards(&c).len(), 1);
    assert!(!c.resonance_use_has_obligations(a));
    c.units.get_mut(&a).unwrap().basic_attack.range_policy =
        crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
    start(&mut c, a, b, 300, true);
    c.step_battle_execution_until(&mut s, 301).unwrap();
    assert_eq!(c.units[&b].stats.current_health, 835);
    c.step_battle_execution_until(&mut s, 1601).unwrap();
    assert!(rewards(&c).is_empty());
    assert_eq!(c.units[&a].stats.attack, 100);
}

#[test]
fn composition_state_reward_same_source_replacement_and_preserved_contributions() {
    for (replacement, count) in [("ReplaceSameSource", 1), ("Stack", 2)] {
        let (mut c, mut s, a, e, b) = reward_fixture(|doc| {
            reward_op(doc)["replacement"] = json!(replacement);
            program(doc)["handlers"][0]["event"]["BasicAttackKill"]["once_per_action"] =
                json!(false);
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        for id in [e, b] {
            c.units.get_mut(&id).unwrap().stats.current_health = 30;
        }
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert_eq!(states(&c)[0].events, 2);
        assert_eq!(rewards(&c).len(), count, "{replacement}");
        // Stack preserves contributions; it does not add identical properties.
        assert_eq!(c.units[&a].stats.attack, 125);
        c.step_battle_execution_until(&mut s, 1601).unwrap();
        assert!(rewards(&c).is_empty());
        assert_eq!(c.units[&a].stats.attack, 100);
    }
}

#[test]
fn composition_state_reward_invalid_duration_rolls_back_death_and_fields() {
    for duration in [0u64, u32::MAX as u64 + 1] {
        let (mut c, mut s, a, e, _) = reward_fixture(|doc| {
            reward_op(doc)["duration"]["root"]["Const"]["value"] = json!(duration);
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 30;
        start(&mut c, a, e, 100, true);
        assert!(matches!(
            c.step_battle_execution_until(&mut s, 101),
            Err(GameError::ValueExpressionFault(_))
        ));
        assert!(c.units[&e].is_active());
        assert_eq!(c.units[&e].stats.current_health, 30);
        assert_eq!(c.units[&a].stats.attack, 100);
        assert_eq!(c.units[&a].stats.current_health, 500);
        assert_eq!(states(&c)[0].events, 0);
        assert!(rewards(&c).is_empty());
        assert!(c.step_battle_execution_until(&mut s, 102).is_err());
    }
}

#[test]
fn composition_state_reward_cancel_replacement_restores_transaction_and_retries_once() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for (site, count) in [
        ("state_reaction_applied", 1),
        ("state_reaction_applied", 2),
        ("sustained_ended", 1),
    ] {
        let (mut c, mut s, a, e, b) = reward_fixture(|doc| {
            program(doc)["handlers"][0]["event"]["BasicAttackKill"]["once_per_action"] =
                json!(false);
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        for id in [e, b] {
            c.units.get_mut(&id).unwrap().stats.current_health = 30;
        }
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 100).unwrap();
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, count, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, 101),
            )
        });
        assert!(
            result.is_err() && token.is_cancelled(),
            "{site}: {result:?}"
        );
        assert_eq!(
            before,
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_view()).unwrap(),
                format!("{s:?}")
            )
        );
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert_eq!(states(&c)[0].events, 2);
        assert_eq!(rewards(&c).len(), 1);
        assert_eq!(c.units[&a].stats.attack, 125);
        assert_eq!(c.units[&a].stats.current_health, 700);
    }
}

#[test]
fn composition_state_reward_expiry_composition_and_observation_parity() {
    let mut results = Vec::new();
    for observe in [false, true] {
        let (mut c, mut s, a, _, _) = reward_fixture(|doc| {
            program(doc)["handlers"][0]["event"] = json!("Expired");
            reward_op(doc)["profile"]["attack_percent"] = json!(0);
            reward_op(doc)["profile"]["defense_percent"] = json!(40);
            reward_op(doc)["profile"]["attack_speed_points"] = json!(50);
            reward_op(doc)["duration"]["root"] =
                json!({"Read":{"StateField":{"name":"reward_ms","unit":"Milliseconds"}}});
            program(doc)["fields"].as_array_mut().unwrap().push(json!({"name":"reward_ms","value":{"Integer":{"unit":"Milliseconds","initial":500,"min":1,"max":1000}}}));
        });
        c.units.get_mut(&a).unwrap().stats.defense = 100;
        if observe {
            c.enable_authoring_observation_with_options(Default::default(), true);
            c.bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: 1,
                    battle_uuid: Uuid::from_u128(0xb15),
                },
            )
            .unwrap();
        }
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert!(states(&c).is_empty());
        assert_eq!(rewards(&c).len(), 1);
        assert_eq!(c.units[&a].stats.defense, 140);
        assert_eq!(c.sustained_basic_interval(a, 1000), 667);
        c.step_battle_execution_until(&mut s, 2501).unwrap();
        assert_eq!(c.units[&a].stats.defense, 100);
        assert_eq!(c.sustained_basic_interval(a, 1000), 1000);
        results.push((
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
            c.units[&a].stats.current_health,
        ));
    }
    assert_eq!(results[0], results[1]);
}

fn target_reward(doc: &mut Value) {
    doc["definitions"][0]["body"]["Action"]["target"]["Literal"]["Target"] = json!("CastTarget");
    doc["skills"][0]["kind"] = json!("Targeted");
    doc["skills"][0]["cast_targeting"]["Explicit"] = json!({
        "selection":"player_unit", "target":"AllySingle", "reach":"global", "air_capable":true
    });
    doc["skills"][0]["admission"]["requirement"] = json!("None");
    doc["skills"][0]["admission"]["cost"] = json!("Zero");
    program(doc)["basic_attack"] = Value::Null;
    program(doc)["owns_use"] = json!(false);
    program(doc)["end_on_source_exit"] = json!(false);
    program(doc)["replacement"] = json!("Stack");
    program(doc)["handlers"][0]["event"] = json!("Expired");
}
fn cast_reward_at(c: &mut BattleCore, a: UnitInstanceId, b: UnitInstanceId, now: u64) {
    c.apply_live_command(
        crate::game::battle::core::sim::BattleLiveCommand::ActivateSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            target: Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                unit_instance_id: b,
            }),
            time_ms: now,
        },
    )
    .unwrap();
}

#[test]
fn composition_state_reward_source_exit_before_and_after_grant_and_recipient_exit() {
    use crate::game::battle::core::sim::BattleLiveCommand;
    for when in [500, 2100] {
        for end in [false, true] {
            let (mut c, mut s, a, _, b) = reward_fixture(|doc| {
                target_reward(doc);
                reward_op(doc)["profile"]["end_on_source_exit"] = json!(end);
            });
            c.units.get_mut(&b).unwrap().owner = Side::Player;
            cast_reward_at(&mut c, a, b, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: when,
            })
            .unwrap();
            c.step_battle_execution_until(&mut s, 2101).unwrap();
            assert_eq!(
                c.units[&b].stats.attack,
                if end { 100 } else { 125 },
                "when={when}, end={end}"
            );
            assert_eq!(rewards(&c).len(), usize::from(!end));
            assert!(states(&c).is_empty());
            if !end {
                let reward = &rewards(&c)[0];
                assert_eq!((reward.caster, reward.target), (a, b));
                assert_eq!(
                    reward.fragment_copy_uuid,
                    c.units[&a].fragment_source.as_ref().map(|f| f.copy_uuid)
                );
                assert!(reward.flow_occurrence.is_none());
                c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                    unit_id: b,
                    time_ms: 2200,
                })
                .unwrap();
                assert!(rewards(&c).is_empty());
            }
        }
    }
}

#[test]
fn composition_state_reward_different_sources_keep_independent_deadlines() {
    let (mut c, mut s, a, _, b) = reward_fixture(target_reward);
    c.units.get_mut(&b).unwrap().owner = Side::Player;
    c.units.get_mut(&b).unwrap().skill_id = Some("r11_aura".into());
    let mut fragment = c.units[&a]
        .fragment_source
        .as_ref()
        .unwrap()
        .as_ref()
        .clone();
    fragment.copy_uuid = Uuid::from_u128(0xb1502);
    c.units.get_mut(&b).unwrap().fragment_source = Some(Arc::new(fragment));
    cast_reward_at(&mut c, a, b, 0);
    c.step_battle_execution_until(&mut s, 1).unwrap();
    cast_reward_at(&mut c, b, b, 300);
    c.step_battle_execution_until(&mut s, 2301).unwrap();
    assert_eq!(rewards(&c).len(), 2);
    assert_eq!(c.units[&b].stats.attack, 125);
    c.step_battle_execution_until(&mut s, 3501).unwrap();
    assert_eq!(rewards(&c).len(), 1);
    assert_eq!(rewards(&c)[0].caster, b);
    assert_eq!(c.units[&b].stats.attack, 125);
    c.step_battle_execution_until(&mut s, 3801).unwrap();
    assert!(rewards(&c).is_empty());
    assert_eq!(c.units[&b].stats.attack, 100);
}

#[test]
fn composition_state_reward_attack_expression_uses_real_stat() {
    for percent in [25, 50] {
        let (mut c, mut s, a, e, b) = reward_fixture(|doc| {
            reward_op(doc)["profile"]["attack_percent"] = json!(percent);
            program(doc)["handlers"].as_array_mut().unwrap().push(json!({
                "id":"probe-input", "event":"ManualInput", "body":[{"EndAndInvoke":{"skill":"attack-probe"}}]
            }));
            let template = parse_yaml(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/skill_composition/manual_state.skill.yaml"
            )))
            .unwrap();
            let mut skill = template["skills"][1].clone();
            skill["id"] = json!("attack-probe");
            skill["body"]["Action"]["effects"][0]["Damage"]["amount"] = json!({"Literal":{"Amount":{
                "moment":"Snapshot", "root":{"Read":{"Stat":{"subject":"Source","stat":"Attack"}}}
            }}});
            skill["body"]["Action"]["effects"][0]["Damage"]["damage_type"] = json!("True");
            doc["skills"].as_array_mut().unwrap().push(skill);
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 30;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert_eq!(c.units[&a].stats.attack, 100 + percent);
        assert_eq!(c.units[&b].stats.current_health, 960);
        cast(&mut c, a, 300);
        c.step_battle_execution_until(&mut s, 301).unwrap();
        assert_eq!(c.units[&b].stats.current_health, 860 - percent);
    }
}

#[test]
fn composition_state_reward_replace_removes_stronger_old_value_and_stale_timer() {
    for (replacement, attack, count) in [("ReplaceSameSource", 125, 1), ("Stack", 150, 2)] {
        let (mut c, mut s, a, e, _) = reward_fixture(|doc| {
            reward_op(doc)["profile"]["attack_percent"] = json!(50);
            reward_op(doc)["duration"]["root"]["Const"]["value"] = json!(2000);
            reward_op(doc)["replacement"] = json!(replacement);
            let mut second = program(doc)["handlers"][0]["body"][1].clone();
            second["ApplyOwnerState"]["profile"]["attack_percent"] = json!(25);
            second["ApplyOwnerState"]["duration"]["root"]["Const"]["value"] = json!(500);
            program(doc)["handlers"][0]["body"]
                .as_array_mut()
                .unwrap()
                .insert(2, second);
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 30;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert_eq!(c.units[&a].stats.attack, attack);
        assert_eq!(rewards(&c).len(), count);
        c.step_battle_execution_until(&mut s, 601).unwrap();
        assert_eq!(c.units[&a].stats.attack, if count == 1 { 100 } else { 150 });
        c.step_battle_execution_until(&mut s, 2101).unwrap();
        assert_eq!(c.units[&a].stats.attack, 100);
        assert!(rewards(&c).is_empty());
    }
}
