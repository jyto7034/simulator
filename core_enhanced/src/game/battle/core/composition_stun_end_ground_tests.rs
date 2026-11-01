mod composition_stun_end_ground {
    use super::compiled_action_fixture::{cast, fixture_content_with_data, place};
    use super::*;
    use crate::authoring::composition::{compile_json, parse_yaml, Artifact};
    use crate::game::{
        ability::sustained::{HitStatusKind, HitStatusProfile, SustainedEffectDef},
        battle::core::sim::BattleExecutionState,
    };
    use serde_json::{json, Value};
    fn fixture(
        edit: impl FnOnce(&mut Value),
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let mut doc = parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/channel_ground.skill.yaml"
        )))
        .unwrap();
        doc["version"] = json!(2);
        doc["skills"][0]["id"] = json!("r11_aura");
        doc["definitions"][0]["body"]["Action"]["modifiers"] = json!([]);
        let gear = crate::authoring::composition::canonical_channel_tests::equipment_fixture("accessory");
        doc["values"]
            .as_array_mut()
            .unwrap()
            .push(gear["values"][0].clone());
        doc["equipment_states"] = gear["equipment_states"].clone();
        edit(&mut doc);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let native = crate::authoring::node_graph::NativeGraphContent {
            skills: artifact.load_native().unwrap(),
            equipment: vec![],
            mark_weapons: vec![],
        };
        let (mut c, s, a, e, b) = fixture_content_with_data(
            &[],
            &[],
            native,
            |_| {},
            crate::game::resource::ResourceCatalog { resources: vec![] },
            |b| b.with_equipment_states(artifact.equipment_states().to_vec()),
            |_| {},
        );
        place(&mut c, e, 3, 1);
        (c, s, a, e, b)
    }
    fn area(d: &mut Value) -> &mut Value {
        &mut d["values"][0]["value"]["Delivery"]["TileArea"]["area"]
    }
    fn advance(c: &mut BattleCore, s: &mut BattleExecutionState, t: u64) {
        c.step_battle_execution_until(s, t).unwrap();
    }
    fn events(c: &BattleCore, a: UnitInstanceId) -> u64 {
        c.sustained_effect_snapshot()
            .unwrap()
            .equipment_states
            .iter()
            .filter(|s| s.owner == a)
            .map(|s| s.state.events)
            .sum()
    }
    #[test]
    fn composition_stun_end_ground_natural_manual_hard_cc_and_early_stop() {
        for mode in ["natural", "manual", "hard-cc", "early"] {
            let (mut c, mut s, a, e, b) = fixture(|_| {});
            cast(&mut c, a, 0);
            advance(&mut c, &mut s, if mode == "early" { 500 } else { 1001 });
            let end = if mode == "natural" {
                2002
            } else if mode == "early" {
                500
            } else {
                1001
            };
            match mode {
                "natural" => advance(&mut c, &mut s, end),
                "hard-cc" => {
                    c.runtime_transaction(|c| {
                        c.begin_sustained(
                            e,
                            a,
                            SustainedEffectDef::HitStatus {
                                duration_ms: 50,
                                profile: HitStatusProfile {
                                    definition: "interrupt".into(),
                                    kind: HitStatusKind::Stun,
                                    modifier_source: None,
                                    vulnerability: None,
                                },
                            },
                            end,
                        )
                    })
                    .unwrap();
                }
                _ => cast(&mut c, a, end),
            }
            assert!(!c.channel_active(a), "{mode}");
            assert_eq!(events(&c, a), u64::from(mode != "early"), "{mode}");
            assert_eq!(c.rooted(e), mode != "early", "{mode}");
            assert!(!c.unit_control_state(e, end).unwrap().hard_cc_active());
            if mode != "early" {
                // A real auto skill can start while the enemy is rooted.
                c.units.get_mut(&e).unwrap().skill_id = Some("r08_strike".into());
                c.units.get_mut(&e).unwrap().skill_activation_mode =
                    crate::game::ability::SkillActivationMode::Auto;
                c.units.get_mut(&e).unwrap().resonance_max = 100;
                c.units.get_mut(&e).unwrap().current_target = Some(b);
                c.runtime_transaction(|c| {
                    c.process_commands(
                        vec![
                            crate::game::battle::damage::BattleCommand::ModifyResonance {
                                target_id: e,
                                amount: 100,
                                allow_autocast_when_full: true,
                            },
                        ],
                        end + 1,
                    );
                    c.schedule_pending_autocasts(end + 1);
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
                advance(&mut c, &mut s, end + 302);
                assert_eq!(
                    c.units[&e].resonance_current, 0,
                    "root permits real skill admission: {mode}"
                );
                advance(&mut c, &mut s, end + 999);
                assert!(c.rooted(e), "{mode}");
                advance(&mut c, &mut s, end + 1000);
                assert!(!c.rooted(e), "{mode}");
                assert_eq!(events(&c, a), 1, "{mode}");
            }
        }
    }
    #[test]
    fn composition_stun_end_ground_clear_exit_and_cross_store_last_contribution() {
        for mode in ["clear", "exit", "overlap"] {
            let (mut c, mut s, a, e, _) = fixture(|d| {
                if mode == "clear" {
                    area(d)["ground_status"]["rules"].as_array_mut().unwrap().push(json!({"trigger":{"Dwell":{"duration_ms":1200}},"status":1,"action":"Clear"}));
                }
                if mode == "exit" {
                    area(d)["ground_status"]["statuses"][1]["lifetime"] = json!("UntilExit");
                }
            });
            cast(&mut c, a, 0);
            advance(&mut c, &mut s, 1001);
            assert!(c.unit_control_state(e, 1001).unwrap().hard_cc_active());
            let end = match mode {
                "clear" => {
                    advance(&mut c, &mut s, 1201);
                    1201
                }
                "exit" => {
                    place(&mut c, e, 7, 2);
                    c.runtime_transaction(|c| c.reconcile_sustained(1002))
                        .unwrap();
                    1002
                }
                _ => {
                    c.runtime_transaction(|c| {
                        c.begin_sustained(
                            a,
                            e,
                            SustainedEffectDef::HitStatus {
                                duration_ms: 2000,
                                profile: HitStatusProfile {
                                    definition: "extra-stun".into(),
                                    kind: HitStatusKind::Stun,
                                    modifier_source: None,
                                    vulnerability: None,
                                },
                            },
                            1001,
                        )
                    })
                    .unwrap();
                    advance(&mut c, &mut s, 2002);
                    assert_eq!(events(&c, a), 0);
                    assert!(!c.rooted(e));
                    advance(&mut c, &mut s, 3001);
                    3001
                }
            };
            assert_eq!(events(&c, a), 1, "{mode}");
            assert!(c.rooted(e), "{mode}");
            advance(&mut c, &mut s, end + 1000);
            assert!(!c.rooted(e), "{mode}");
            assert_eq!(events(&c, a), 1, "no duplicate area end: {mode}");
        }
    }
    #[test]
    fn composition_stun_end_ground_parallel_same_time_removal_emits_once() {
        for manual in [false, true] {
            let (mut c, mut s, a, e, _) = fixture(|d| {
                let mut first = d["skills"][0]["body"].clone();
                first["Call"]["id"] = json!("first");
                let mut second = first.clone();
                second["Call"]["id"] = json!("second");
                d["skills"][0]["body"] = json!({"Parallel":{"id":"overlap","branches":[first,second]}});
            });
            cast(&mut c, a, 0);
            advance(&mut c, &mut s, 1001);
            assert_eq!(c.active_areas.len(), 2);
            assert!(c.unit_control_state(e, 1001).unwrap().hard_cc_active());
            if manual { cast(&mut c, a, 1001); } else { advance(&mut c, &mut s, 2002); }
            assert!(c.active_areas.is_empty());
            assert!(c.rooted(e));
            assert_eq!(events(&c, a), 1);
        }
    }
    #[test]
    fn composition_stun_end_ground_real_movement_stops_and_resumes_after_root() {
        use crate::game::battle::core::movement::types::{MovementGoal, WorldVec2};
        let (mut c, mut s, a, e, _) = fixture(|_| {});
        c.units.get_mut(&e).unwrap().stats.move_speed_units_per_ms = 2000;
        c.units.get_mut(&e).unwrap().body.move_speed = 2.0;
        c.units.get_mut(&e).unwrap().blockable = false;
        cast(&mut c, a, 0);
        advance(&mut c, &mut s, 1001);
        cast(&mut c, a, 1001);
        assert!(c.rooted(e));
        let start = c.units[&e].body.position;
        let mut goals = std::collections::HashMap::new();
        goals.insert(
            e,
            MovementGoal::MoveToPoint {
                point: WorldVec2::new(6.5, 1.5),
                stop_radius: 0.1,
            },
        );
        c.run_test_movement_tick(1100, 50, &goals);
        assert_eq!(c.units[&e].body.position, start);
        advance(&mut c, &mut s, 2001);
        assert!(!c.rooted(e));
        c.run_test_movement_tick(2050, 50, &goals);
        assert_ne!(c.units[&e].body.position, start);
    }
}
