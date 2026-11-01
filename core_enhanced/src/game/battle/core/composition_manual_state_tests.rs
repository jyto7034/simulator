mod composition_manual_state {
    use super::compiled_action_fixture::{cast, fixture_content_with_weapon, hit, place};
    use super::*;
    use crate::{
        authoring::{
            composition::{compile_json, parse_yaml},
            node_graph::NativeGraphContent,
        },
        game::{
            ability::state_program::StateScalar,
            battle::core::sim::{BattleExecutionState, BattleLiveCommand},
        },
    };
    use serde_json::{json, Value};
    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/manual_state.skill.yaml"
    ));
    type Fixture = (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    );
    fn fixture(edit: impl FnOnce(&mut Value)) -> Fixture {
        let mut doc = parse_yaml(YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        edit(&mut doc);
        let native = compile_json(&doc.to_string())
            .unwrap()
            .load_native()
            .unwrap();
        let (mut c, s, a, e, b) = fixture_content_with_weapon(
            &["piercing-damage-increase"],
            &[0],
            NativeGraphContent {
                skills: native,
                equipment: vec![],
                mark_weapons: vec![],
            },
            |weapon| r12_lifesteal_weapon(weapon, 17),
        );
        c.units.get_mut(&b).unwrap().owner = Side::Opponent;
        c.units.get_mut(&b).unwrap().fragment_source = None;
        c.units.get_mut(&b).unwrap().skill_id = None;
        place(&mut c, e, 4, 1);
        place(&mut c, b, 6, 1);
        for id in [a, e, b] {
            let u = c.units.get_mut(&id).unwrap();
            u.stats.max_health = 1000;
            u.stats.current_health = 1000;
            u.stats.defense = 0;
        }
        c.units.get_mut(&a).unwrap().stats.current_health = 500;
        (c, s, a, e, b)
    }
    fn program(doc: &mut Value) -> &mut Value {
        &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["State"]
            ["program"]
    }
    fn activation(c: &BattleCore) -> u64 {
        c.sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .find(|s| s.state.is_some())
            .unwrap()
            .activation
    }
    fn input(a: UnitInstanceId, activation: u64, now: u64) -> BattleLiveCommand {
        BattleLiveCommand::ReinputSustainedSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            activation,
            target: None,
            time_ms: now,
        }
    }
    #[test]
    fn composition_manual_state_preserves_weapon_cycle_contacts_and_child_source() {
        let (mut c, mut s, a, e, b) = fixture(|_| {});
        c.enable_authoring_observation_with_options(Default::default(), true);
        c.bind_authoring_observation(
            crate::game::battle::core::authoring_observation::ObservationKey {
                entry_ordinal: 1,
                battle_uuid: Uuid::from_u128(0xa61),
            },
        )
        .unwrap();
        let original = serde_json::to_value(&c.units[&a].basic_attack).unwrap();
        let next = c.units[&a].next_basic_attack_ms;
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        assert_eq!(c.units[&a].next_basic_attack_ms, next);
        assert!(c.resonance_use_has_obligations(a));
        for now in [100, 200] {
            c.units.get_mut(&a).unwrap().next_basic_attack_ms = now;
            c.event_queue.push(BattleEvent::AttackStart {
                time_ms: now,
                attacker_instance_id: a,
                target_instance_id: Some(e),
                schedule_next: true,
                cause: Default::default(),
            });
            c.step_battle_execution_until(&mut s, now + 1).unwrap();
        }
        assert_eq!(c.units[&e].stats.current_health, 920);
        assert_eq!(c.units[&b].stats.current_health, 900);
        assert_eq!(c.units[&a].stats.current_health, 534);
        let next = c.units[&a].next_basic_attack_ms;
        let costs = c.runtime_fact_summary().cost_mutations;
        let activation = activation(&c);
        assert!(c
            .validate_manual_skill_activation(501, a, &"r11_aura".into(), None)
            .is_ok());
        cast(&mut c, a, 501); // The normal input reaches the authored handler.
        assert_eq!(c.units[&e].stats.current_health, 880);
        assert_eq!(c.units[&b].stats.current_health, 860);
        assert_eq!(c.units[&a].stats.current_health, 534);
        assert_eq!(c.units[&a].next_basic_attack_ms, next);
        assert_eq!(
            serde_json::to_value(&c.units[&a].basic_attack).unwrap(),
            original
        );
        assert_eq!(c.runtime_fact_summary().cost_mutations, costs);
        assert!(!c.resonance_use_has_obligations(a));
        let amount = c
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_amount
            .as_ref()
            .unwrap();
        assert_eq!(
            amount.source.cast.as_ref().unwrap().skill_id.as_str(),
            "authored-finish"
        );
        assert!(amount.source.equipment.is_none());
        assert_eq!(amount.source.reinput().unwrap().remaining_ms, 1500);
        assert_eq!(amount.source.reinput().unwrap().activation, activation);
        let before = c.transaction_debug_snapshot();
        assert!(c.apply_live_command(input(a, activation, 501)).is_err());
        assert_eq!(c.transaction_debug_snapshot(), before);
    }
    #[test]
    fn composition_manual_state_millisecond_capture_and_natural_expiry_are_distinct() {
        for (now, damage) in [(2, 45), (501, 40), (502, 40), (1002, 30), (1999, 11)] {
            let (mut c, mut s, a, e, _) = fixture(|_| {});
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, now).unwrap();
            let activation = activation(&c);
            c.apply_live_command(input(a, activation, now)).unwrap();
            assert_eq!(c.units[&e].stats.current_health, 1000 - damage);
            assert!(c.basic_line_profile(a).is_none());
        }
        let (mut c, mut s, a, e, _) = fixture(|_| {});
        let original = serde_json::to_value(&c.units[&a].basic_attack).unwrap();
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        let activation = activation(&c);
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert_eq!(c.units[&e].stats.current_health, 1000);
        assert_eq!(
            serde_json::to_value(&c.units[&a].basic_attack).unwrap(),
            original
        );
        assert!(!c.resonance_use_has_obligations(a));
        assert!(c.apply_live_command(input(a, activation, 2001)).is_err());
    }
    #[test]
    fn composition_manual_state_control_gates_and_stale_identity_preserve_activation() {
        for buff in ["r11_silence", "r11_stun"] {
            let (mut c, mut s, a, e, b) = fixture(|_| {});
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            let activation = activation(&c);
            c.process_event_for_test(
                BattleEvent::ApplyBuff {
                    time_ms: 1,
                    caster_instance_id: e,
                    target_instance_id: a,
                    buff_id: crate::game::battle::buffs::BuffId::from_name(buff),
                    duration_ms: 500,
                    cause: Default::default(),
                },
                1,
            )
            .unwrap();
            c.step_battle_execution_until(&mut s, 250).unwrap();
            let before = c.transaction_debug_snapshot();
            assert!(c
                .validate_manual_skill_activation(250, a, &"r11_aura".into(), None)
                .is_err());
            assert!(c.apply_live_command(input(a, activation, 250)).is_err());
            assert!(c.apply_live_command(input(b, activation, 250)).is_err());
            assert!(c.apply_live_command(input(a, activation + 1, 250)).is_err());
            assert_eq!(c.transaction_debug_snapshot(), before);
            assert!(c.basic_line_profile(a).is_some());
            c.step_battle_execution_until(&mut s, 502).unwrap();
            c.apply_live_command(input(a, activation, 502)).unwrap();
            assert_eq!(c.units[&e].stats.current_health, 960);
        }
    }
    #[test]
    fn composition_manual_state_second_use_defines_repeated_input_heal_and_field_termination() {
        let (mut c, mut s, a, e, _) = fixture(|doc| {
            let p = program(doc);
            p["basic_attack"] = Value::Null;
            p["fields"] = json!([{ "name":"charges", "value":{"Integer":{"unit":"Count","initial":2,"min":0,"max":2}} }]);
            p["handlers"][0]["body"] = json!([
                {"Set":{"field":"charges","value":{"moment":"Live","root":{"Add":[{"Read":{"StateField":{"name":"charges","unit":"Count"}}},{"Const":{"value":-1,"unit":"Count"}}]}}}},
                {"HealOwner":{"amount":{"moment":"Live","root":{"Const":{"value":20,"unit":"Magnitude"}}}}},
                {"If":{"condition":{"Compare":{"left":{"moment":"Live","root":{"Read":{"StateField":{"name":"charges","unit":"Count"}}}},"comparison":"Equal","right":{"moment":"Live","root":{"Const":{"value":0,"unit":"Count"}}}}},"then_ops":["End"],"else_ops":[]}}
            ]);
            doc["skills"].as_array_mut().unwrap().truncate(1);
            doc["values"].as_array_mut().unwrap().truncate(1);
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        let activation = activation(&c);
        let costs = c.runtime_fact_summary().cost_mutations;
        cast(&mut c, a, 100);
        let view = c.sustained_effect_snapshot().unwrap();
        let active = view
            .active
            .iter()
            .find(|v| v.activation == activation)
            .unwrap();
        assert_eq!(
            active.state.as_ref().unwrap().fields[0].value,
            StateScalar::Integer(1)
        );
        assert_eq!(active.expires_at_ms, 2001);
        assert_eq!(c.units[&a].stats.current_health, 520);
        cast(&mut c, a, 200);
        assert!(c.sustained_effect_snapshot().unwrap().active.is_empty());
        assert_eq!(c.units[&a].stats.current_health, 540);
        assert_eq!(c.units[&e].stats.current_health, 1000);
        assert_eq!(c.runtime_fact_summary().cost_mutations, costs);
        assert!(!c.resonance_use_has_obligations(a));
        assert!(c.apply_live_command(input(a, activation, 201)).is_err());
    }

    #[test]
    fn composition_manual_state_handoff_failure_restores_fields_overlay_queue_and_retry() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "reinput_after_consume",
            "reinput_after_invoke",
            "reinput_after_execution",
        ] {
            let (mut c, mut s, a, e, _) = fixture(|doc| {
                program(doc)["fields"] = json!([{ "name":"accepted", "value":{"Bool":false} }]);
                program(doc)["handlers"][0]["body"]
                    .as_array_mut()
                    .unwrap()
                    .insert(0, json!({"SetBool":{"field":"accepted","value":true}}));
            });
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 501).unwrap();
            let command = input(a, activation(&c), 501);
            let before = (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
            );
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || c.apply_live_command(command.clone()),
                )
            });
            assert!(
                result.is_err() && token.is_cancelled(),
                "{site}: {result:?}"
            );
            assert_eq!(
                (
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot()
                ),
                before,
                "{site}"
            );
            c.apply_live_command(command).unwrap();
            assert_eq!(c.units[&e].stats.current_health, 960);
        }
        let (mut c, mut s, a, _, _) = fixture(|doc| {
            doc["values"][1]["value"]["Amount"]["root"] = json!({"Mul":{
                "value":{"Const":{"value":10,"unit":"Magnitude"}},
                "scalar":{"Ratio":{"numerator":{"Const":{"value":1,"unit":"Magnitude"}},"denominator":{"Read":{"Stat":{"subject":"Source","stat":"SpellPower"}}},"rounding":"Floor"}}
            }});
        });
        c.units.get_mut(&a).unwrap().stats.spell_power = 0;
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 501).unwrap();
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
        );
        let error = c
            .apply_live_command(input(a, activation(&c), 501))
            .unwrap_err();
        assert!(
            matches!(error, GameError::ValueExpressionFault(_)),
            "{error:?}"
        );
        assert_eq!(
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot()
            ),
            before
        );
    }

    #[test]
    fn composition_manual_state_source_change_defense_and_exit_do_not_grant_finish() {
        let (mut c, mut s, a, e, _) = fixture(|_| {});
        c.initialize_deployed_tactical_state(
            a,
            crate::game::battle::types::TacticalTileKind::Ground,
            Default::default(),
            0,
        )
        .unwrap();
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1501).unwrap();
        let activation = activation(&c);
        let original = c.units[&a].fragment_source.clone();
        Arc::make_mut(
            c.units
                .get_mut(&a)
                .unwrap()
                .fragment_source
                .as_mut()
                .unwrap(),
        )
        .copy_uuid = Uuid::from_u128(0xbadd);
        let before = c.transaction_debug_snapshot();
        assert!(c.apply_live_command(input(a, activation, 1501)).is_err());
        assert_eq!(c.transaction_debug_snapshot(), before);
        c.units.get_mut(&a).unwrap().fragment_source = original;
        c.apply_live_command(BattleLiveCommand::SetEngagementStance {
            unit_id: a,
            stance: crate::game::battle::types::EngagementStance::Defense,
            time_ms: 1501,
        })
        .unwrap();
        assert!(c.apply_live_command(input(a, activation, 1501)).is_err());
        assert!(c.basic_line_profile(a).is_some());
        c.apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: a,
            time_ms: 1501,
        })
        .unwrap();
        assert!(c.basic_line_profile(a).is_none());
        assert_eq!(c.units[&e].stats.current_health, 1000);
        assert!(c.apply_live_command(input(a, activation, 1501)).is_err());
    }

    #[test]
    fn composition_manual_state_owner_death_removes_overlay_and_finish_without_child() {
        let (mut c, mut s, a, e, b) = fixture(|_| {});
        let original = serde_json::to_value(&c.units[&a].basic_attack).unwrap();
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 501).unwrap();
        let activation = activation(&c);
        let cast_sequence = c.skill_cast_seq;
        hit(&mut c, e, a, 1000, 501);
        assert_eq!(c.units[&a].stats.current_health, 0);
        assert!(c.basic_line_profile(a).is_none());
        assert_eq!(
            serde_json::to_value(&c.units[&a].basic_attack).unwrap(),
            original
        );
        assert!(!c.resonance_use_has_obligations(a));
        assert_eq!(c.units[&e].stats.current_health, 1000);
        assert_eq!(c.units[&b].stats.current_health, 1000);
        assert_eq!(c.skill_cast_seq, cast_sequence);
        assert!(c.apply_live_command(input(a, activation, 501)).is_err());
        assert!(c
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .all(|s| s.activation != activation));
    }

    #[test]
    fn composition_manual_state_observed_reinput_is_not_child_execution_authority() {
        use crate::game::battle::runtime_facts::{CommittedReinputContext, RuntimeFactSource};
        let (mut c, _, a, e, _) = fixture(|_| {});
        let prior = std::mem::replace(
            &mut c.runtime_fact_source,
            RuntimeFactSource {
                actor: Some(a),
                extended_observation: RuntimeFactSource::extended_observation(
                    None,
                    Some(Arc::new(CommittedReinputContext {
                        activation: 1,
                        committed_at_ms: 501,
                        remaining_ms: 1500,
                    })),
                )
                .unwrap(),
                ..Default::default()
            },
        );
        let sequence = c.skill_cast_seq;
        c.invoke_ability(
            501,
            a,
            "authored-finish",
            Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                unit_instance_id: a,
            }),
            Default::default(),
            false,
        );
        c.runtime_fact_source = prior;
        assert_eq!(c.skill_cast_seq, sequence);
        assert_eq!(c.units[&e].stats.current_health, 1000);
        assert_eq!(c.runtime_fact_summary().damage_hp_lost, 0);
    }
}
