mod composition_control {
    use super::composition_duration_modifiers::equip_named;
    use super::*;
    use crate::authoring::composition::{compile_json, parse_yaml, Artifact};
    use crate::game::ability::sustained::ControlKind;
    use crate::game::battle::core::movement::types::{MovementGoal, WorldVec2};
    use serde_json::{json, Value};

    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/control.skill.yaml"
    ));
    fn control(doc: &mut Value) -> &mut Value {
        &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["Control"]
    }
    fn native(edit: impl FnOnce(&mut Value)) -> SkillDatabase {
        let mut doc = parse_yaml(YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        edit(&mut doc);
        let artifact = compile_json(&doc.to_string()).unwrap();
        Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap()
    }
    fn fixture() -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let (mut c, s, a, b) = fixture_from_native(native(|_| {}), false, false);
        let enemy = c
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for (id, x, y) in [(a, 2, 1), (b, 5, 2), (enemy, 6, 1)] {
            place(&mut c, id, x, y);
            let u = c.units.get_mut(&id).unwrap();
            u.stats.current_health = 1000;
            u.stats.max_health = 1000;
            u.next_basic_attack_ms = 1000;
            u.basic_attack_windup_until_ms = None;
            u.action_locks = Default::default();
        }
        c.units.get_mut(&a).unwrap().block_capacity = 1;
        c.units.get_mut(&a).unwrap().block_radius_units = 1.0;
        c.units.get_mut(&b).unwrap().block_capacity = 1;
        c.units.get_mut(&b).unwrap().block_radius_units = 1.5;
        let u = c.units.get_mut(&enemy).unwrap();
        u.stats.move_speed_units_per_ms = 1000;
        u.body.move_speed = 1.0;
        u.basic_attack.range_policy =
            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        u.basic_attack.defense_tile_range = None;
        u.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
        c.refresh_block_state();
        (c, s, a, b, enemy)
    }

    #[test]
    fn composition_control_used_unused_validation_roundtrip_and_source_inputs() {
        let doc = parse_yaml(YAML).unwrap();
        let artifact = compile_json(&doc.to_string()).unwrap();
        let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let skill = &loaded.load_native().unwrap().skills[0];
        assert!(matches!(
            skill.use_lifetime,
            crate::game::ability::SkillUseLifetime::Action
        ));
        assert!(artifact
            .sources()
            .iter()
            .any(|(_, trace)| trace.inputs.iter().any(|p| p == "values[0].value")));
        for unused in [false, true] {
            for mode in ["duration", "speed", "range", "slow"] {
                let mut d = doc.clone();
                match mode {
                    "duration" => control(&mut d)["duration_ms"] = json!(0),
                    "speed" => {
                        control(&mut d)["kind"]["ApproachTaunt"]["profile"]["speed_percent"] =
                            json!(0)
                    }
                    "range" => {
                        control(&mut d)["kind"]["ApproachTaunt"]["profile"]["attack_range_units"] =
                            json!(0)
                    }
                    _ => control(&mut d)["kind"] = json!({"Slow":{"percent":100}}),
                }
                if unused {
                    d["skills"][0]["body"]["Call"]["arguments"][0]["value"] =
                        json!({"Literal":{"Effects":[]}});
                }
                let error = compile_json(&d.to_string()).unwrap_err();
                assert!(
                    error
                        .source
                        .definition
                        .starts_with("values[0].value.Effects[0].op.Sustained.definition.Control"),
                    "{mode}/{unused}: {error:?}"
                );
                if mode == "duration" {
                    assert!(error.source.definition.ends_with("duration_ms"));
                }
            }
        }
    }

    #[test]
    fn composition_control_approach_moves_unblocks_limits_attacks_and_ends_without_use_lock() {
        let (mut c, mut s, a, b, e) = fixture();
        assert_eq!(c.blocked_by(e), Some(b));
        cast(&mut c, a, None, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        assert_eq!(c.taunt_source(e), Some(a));
        assert_eq!(c.blocked_by(e), None);
        assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 2000);
        assert!(!c.channel_active(a) && !c.resonance_use_has_obligations(a));
        assert_eq!(
            c.units[&e].stats.current_health, 1000,
            "control causes no direct damage"
        );
        assert!(!c.is_basic_attack_target_in_range(e, a));
        assert!(!c.is_basic_attack_target_in_range(e, b));
        assert!(
            matches!(c.build_continuous_attack_goals().get(&e),Some(MovementGoal::MoveToPoint{point,..}) if *point==c.units[&a].body.position)
        );
        let initial = c.units[&e].body.position.x;
        c.step_battle_execution_until(&mut s, 500).unwrap();
        assert!(c.units[&e].body.position.x < initial - 0.5);
        assert_eq!(c.blocked_by(e), None);
        assert_eq!(c.units[&a].stats.current_health, 1000);
        c.active_movement_segments.remove(&e);
        let center = c.units[&a].body.position;
        c.units
            .get_mut(&e)
            .unwrap()
            .set_world_position(WorldVec2::new(center.x + 1.001, center.y));
        assert!(!c.is_basic_attack_target_in_range(e, a));
        c.units
            .get_mut(&e)
            .unwrap()
            .set_world_position(WorldVec2::new(center.x + 1.0, center.y));
        assert!(c.is_basic_attack_target_in_range(e, a));
        // Hard CC on the source after release does not cancel this target-owned state.
        crate::game::battle::core::relic_structural_tests::flow_cases::hard_cc_cancel(
            &mut c, a, 500,
        );
        assert_eq!(c.taunt_source(e), Some(a));
        c.step_battle_execution_until(&mut s, 1502).unwrap();
        assert_eq!(c.taunt_source(e), None);
        assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
    }

    #[test]
    fn composition_control_immune_replacement_exit_and_cancel_are_atomic() {
        let (mut c, mut s, a, b, e) = fixture();
        c.units
            .get_mut(&e)
            .unwrap()
            .target_traits
            .push(crate::game::battle::types::UnitTargetTrait::TauntImmune);
        cast(&mut c, a, None, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        assert_eq!(c.taunt_source(e), None);
        assert_eq!(c.blocked_by(e), Some(b));
        assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
        assert!(c.sustained_effect_snapshot().unwrap().active.is_empty());
        let (mut c, mut s, a, b, e) = fixture();
        cast(&mut c, a, None, 0);
        c.step_battle_execution_until(&mut s, 0).unwrap();
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            format!("{s:?}"),
        );
        use crate::authoring::{
            control as ctl, CancellationToken, ExecutionLimits, ExecutionPhase,
        };
        let token = CancellationToken::default();
        let result = ctl::with_test_cancellation("approach_taunt_cancelled_attacks", 1, || {
            ctl::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, 1),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                format!("{s:?}")
            ),
            before
        );
        assert_eq!(c.blocked_by(e), Some(b));
        c.step_battle_execution_until(&mut s, 100).unwrap();
        assert_eq!(c.taunt_source(e), Some(a));
        cast(&mut c, b, None, 100);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert_eq!(c.taunt_source(e), Some(b));
        assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 2000);
        assert_eq!(c.sustained_effect_snapshot().unwrap().active.len(), 1);
        c.apply_live_command(BattleLiveCommand::WithdrawUnit {
            unit_id: b,
            time_ms: 102,
        })
        .unwrap();
        c.step_battle_execution_until(&mut s, 102).unwrap();
        assert_eq!(c.taunt_source(e), None);
        assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
        c.step_battle_execution_until(&mut s, 1602).unwrap();
        assert_eq!(c.taunt_source(e), None);
    }

    #[test]
    fn composition_control_other_kinds_share_action_and_change_actual_movement() {
        for (kind, factor) in [
            (json!({"Slow":{"percent":50}}), 0.5f32),
            (json!("Root"), 0.0),
        ] {
            let native = native(|doc| {
                control(doc)["kind"] = kind.clone();
                control(doc)["duration_ms"] = json!(201);
                doc["definitions"][0]["body"]["Action"]
                    .as_object_mut()
                    .unwrap()
                    .remove("modifiers");
            });
            let ron = ron::ser::to_string(&native).unwrap();
            let (mut plain, mut ps, _, _) = r11_employee_effect_core(&ron);
            let (mut c, mut s, a, _) = r11_employee_effect_core(&ron);
            let e = c
                .units
                .values()
                .find(|u| u.owner == Side::Opponent)
                .unwrap()
                .instance_id;
            let start = c.units[&e].body.position;
            cast(&mut c, a, None, 0);
            plain.step_battle_execution_until(&mut ps, 200).unwrap();
            c.step_battle_execution_until(&mut s, 200).unwrap();
            let full = start.x - plain.units[&e].body.position.x;
            let controlled = start.x - c.units[&e].body.position.x;
            assert!(full > 0.05, "fixture movement must actually advance");
            assert!(
                (controlled - full * factor).abs() < 0.005,
                "{kind}: {full} vs {controlled}"
            );
            assert!(!c.resonance_use_has_obligations(a));
            c.units
                .get_mut(&e)
                .unwrap()
                .action_locks
                .lock_movement_until(9000);
            c.step_battle_execution_until(&mut s, 300).unwrap();
            assert_eq!(c.units[&e].stats.move_speed_units_per_ms, 1000);
            assert_eq!(c.units[&e].action_locks.movement_until_ms, 9000);
            assert!(!c.rooted(e));
        }
    }

    #[test]
    fn composition_control_duration_uses_real_equipped_owner_once() {
        for mode in ["none", "owned", "other", "one", "two"] {
            let (mut c, mut s, a, b, e) = fixture();
            let owners = match mode {
                "owned" => vec![None],
                "other" => vec![Some(b)],
                "one" => vec![Some(a)],
                "two" => vec![Some(a), Some(a)],
                _ => vec![],
            };
            equip_named(&mut c, a, &owners, false, "long-breath");
            cast(&mut c, a, None, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            let view = c
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .into_iter()
                .find(|r| matches!(r.control, Some(ControlKind::ApproachTaunt { .. })))
                .unwrap();
            assert_eq!(
                view.expires_at_ms,
                1 + if matches!(mode, "one" | "two") {
                    1877
                } else {
                    1501
                },
                "{mode}"
            );
            assert!(!c.resonance_use_has_obligations(a));
            c.step_battle_execution_until(&mut s, view.expires_at_ms - 1)
                .unwrap();
            assert_eq!(c.taunt_source(e), Some(a));
            c.step_battle_execution_until(&mut s, view.expires_at_ms)
                .unwrap();
            assert_eq!(c.taunt_source(e), None);
        }
    }
}
