// Included in compiled_support to reuse the real fragment/inventory fixture.
mod composition_channel_support {
    use super::*;
    use crate::authoring::composition::{compile_json, parse_yaml, Artifact};
    use serde_json::{json, Value};

    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/channel_support.skill.yaml"
    ));
    fn source() -> Value {
        let mut doc = parse_yaml(YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        doc
    }
    fn fixture(
        equipped: bool,
        edit: impl FnOnce(&mut Value),
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let mut doc = source();
        edit(&mut doc);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        fixture_from_native(artifact.load_native().unwrap(), equipped, false)
    }
    fn followup(doc: &mut Value) {
        let first = doc["skills"][0]["body"].clone();
        doc["skills"][0]["body"] = json!({"Sequence":{"id":"then-recover","children":[first,
            {"Action":{"id":"recover","target":{"Literal":{"Target":"SelfUnit"}},
                "delivery":{"Literal":{"Delivery":"Instant"}},"effects":[{"Heal":{"id":"recover","tags":["recovery"],
                    "amount":{"Literal":{"Amount":{"moment":"Snapshot","root":{"Const":{"value":37,"unit":"Magnitude"}}}}}}}],
                "wait":"EffectsSettled"}}
        ]}});
    }

    #[test]
    fn composition_channel_support_source_diagnostics_and_owned_lifetime() {
        let doc = source();
        let a = compile_json(&doc.to_string()).unwrap();
        let native = a.load_native().unwrap();
        assert!(matches!(&native.skills[0].use_lifetime,
            crate::game::ability::SkillUseLifetime::ActionAndEffects { effects } if effects.len() == 1));
        let origin = a
            .origin_for(native.skills[0].steps[0].effects[0].source().unwrap())
            .unwrap();
        assert!(origin.calls.iter().any(|p| p.starts_with("skills[0]")));
        assert!(origin.inputs.iter().any(|p| p.starts_with("values[0]")));
        for unused in [false, true] {
            let mut invalid = doc.clone();
            invalid["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]
                ["ChannelAura"]["duration_ms"] = json!(0);
            if unused {
                invalid["skills"][0]["body"]["Call"]["arguments"][0]["value"] =
                    json!({"Literal":{"Effects":[]}});
            }
            let error = compile_json(&invalid.to_string()).unwrap_err();
            assert!(
                error.source.definition.ends_with("ChannelAura"),
                "{error:?}"
            );
            assert!(error.reason.contains("positive"), "{error:?}");
            if !unused {
                assert!(!error.source.calls.is_empty(), "{error:?}");
            }
        }
        let mut released = doc;
        released["definitions"][0]["body"]["Action"]["wait"] = json!("Released");
        assert!(compile_json(&released.to_string())
            .unwrap_err()
            .reason
            .contains("EffectsSettled"));
    }

    #[test]
    fn composition_channel_support_only_natural_end_runs_followup() {
        for ending in ["expiry", "stop", "stun", "selected-exit"] {
            let (mut c, mut s, a, b) = fixture(ending == "selected-exit", followup);
            c.units.get_mut(&a).unwrap().stats.current_health = 500;
            cast(&mut c, a, (ending == "selected-exit").then_some(b), 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert!(c.channel_active(a));
            assert_eq!(c.units[&a].stats.current_health, 500);
            match ending {
                "stop" => cast(&mut c, a, None, 2),
                "stun" => {
                    let enemy = c
                        .units
                        .values()
                        .find(|u| u.owner == Side::Opponent)
                        .unwrap()
                        .instance_id;
                    c.process_event_for_test(
                        BattleEvent::ApplyBuff {
                            time_ms: 2,
                            caster_instance_id: enemy,
                            target_instance_id: a,
                            buff_id: crate::game::battle::buffs::BuffId::from_name("r11_stun"),
                            duration_ms: 50,
                            cause: Default::default(),
                        },
                        2,
                    )
                    .unwrap();
                }
                "selected-exit" => {
                    place(&mut c, b, 7, 3);
                    c.reconcile_sustained(2).unwrap();
                }
                _ => (),
            }
            c.step_battle_execution_until(&mut s, 201).unwrap();
            assert!(!c.channel_active(a), "{ending}");
            assert!(!c.resonance_use_has_obligations(a), "{ending}");
            assert_eq!(
                c.units[&a].stats.current_health,
                if ending == "expiry" { 537 } else { 500 },
                "{ending}"
            );
        }
    }

    #[test]
    fn composition_channel_support_inventory_conversion_and_released_regular_snapshot() {
        for (equipped, percent) in [(false, 20), (false, 35), (true, 20)] {
            let (mut c, mut s, a, b) = fixture(equipped, |doc| {
                doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]
                    ["ChannelAura"]["aura"]["basic_damage_percent"] = json!(percent);
            });
            cast(&mut c, a, equipped.then_some(b), 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            let bonus = if equipped { (40, 20) } else { (percent, 15) };
            assert_eq!(c.aura_basic_bonus_for_occurrence(b, true).unwrap(), bonus);
            assert_eq!(c.aura_basic_bonus_for_occurrence(b, false).unwrap(), (0, 0));
            assert_eq!(c.aura_basic_bonus_for_occurrence(a, true).unwrap(), (0, 0));
            assert_eq!(c.units[&b].stats.attack, 100);
            let enemy = c
                .units
                .values()
                .find(|u| u.owner == Side::Opponent)
                .unwrap()
                .instance_id;
            let hp = c.units[&enemy].stats.current_health;
            let (_, snapshot) = crate::game::battle::core::relic_structural_tests::flow_cases::start_basic_for_support(&mut c, b, enemy, true, 2);
            assert_eq!(snapshot.base_damage, 100 + bonus.0);
            cast(&mut c, a, None, 3);
            c.step_battle_execution_until(&mut s, 23).unwrap();
            assert_eq!(c.units[&enemy].stats.current_health, hp - 100 - bonus.0);
        }
    }

    #[test]
    fn composition_channel_support_area_reentry_and_selection_rejection() {
        let (mut c, mut s, a, b) = fixture(false, |_| {});
        cast(&mut c, a, None, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        place(&mut c, b, 7, 3);
        c.reconcile_sustained(2).unwrap();
        assert!(c.channel_active(a));
        assert_eq!(c.aura_basic_bonus_for_occurrence(b, true).unwrap(), (0, 0));
        place(&mut c, b, 3, 1);
        c.reconcile_sustained(3).unwrap();
        assert_eq!(
            c.aura_basic_bonus_for_occurrence(b, true).unwrap(),
            (20, 15)
        );
        let (mut c, _, a, b) = fixture(true, |_| {});
        place(&mut c, b, 7, 3);
        for target in [None, Some(a), Some(b)] {
            let before = (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
            );
            assert!(c
                .apply_live_command(BattleLiveCommand::ActivateSkill {
                    unit_id: a,
                    skill_id: "r11_aura".into(),
                    target: target
                        .map(|unit_instance_id| SkillCastTarget::Unit { unit_instance_id }),
                    time_ms: 0
                })
                .is_err());
            assert_eq!(
                (
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot()
                ),
                before
            );
            assert_eq!(c.units[&a].resonance_current, 100);
        }
    }

    #[test]
    fn composition_channel_support_reactive_equipment_survives_and_keeps_its_proc_budget() {
        let mut doc = source();
        let mut reaction = doc["skills"][0].clone();
        reaction["id"] = json!("reactive-recovery");
        reaction["admission"] = Value::Null;
        followup(&mut doc);
        reaction["body"] = doc["skills"][0]["body"]["Sequence"]["children"][1].clone();
        doc["skills"][0]["body"] = source()["skills"][0]["body"].clone();
        doc["skills"].as_array_mut().unwrap().push(reaction);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let (mut c, mut s, a, _) = fixture_from_native_with_weapon(
            artifact.load_native().unwrap(),
            false,
            false,
            |weapon| {
                weapon.ability_activations = vec![crate::game::ability::AbilityActivationBinding {
                    ability_id: "reactive-recovery".into(),
                    activation: crate::game::ability::AbilityActivationDef::TriggerProc {
                        trigger: crate::game::stats::TriggerType::OnHit,
                        proc_chance_percent: 100,
                        internal_cooldown_ms: 10,
                        max_triggers_per_battle: Some(2),
                    },
                }];
            },
        );
        let enemy = c
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        crate::game::battle::core::relic_structural_tests::flow_cases::prepare_support_attacker(
            &mut c, enemy,
        );
        c.units.get_mut(&enemy).unwrap().stats.attack = 10;
        c.units.get_mut(&enemy).unwrap().basic_attack.windup_ms = 1;
        c.units
            .get_mut(&enemy)
            .unwrap()
            .basic_attack
            .ranged_reposition_ms = 0;
        c.units.get_mut(&enemy).unwrap().basic_attack.range_role =
            crate::game::data::equipment_data::WeaponRangeRole::Ranged;
        place(&mut c, enemy, 2, 0);
        c.units.get_mut(&enemy).unwrap().current_target = Some(a);
        let strike = |c: &mut BattleCore, s: &mut BattleExecutionState, time| {
            c.units.get_mut(&enemy).unwrap().next_basic_attack_ms = 0;
            c.process_event_for_test(
                BattleEvent::AttackStart {
                    time_ms: time,
                    attacker_instance_id: enemy,
                    target_instance_id: Some(a),
                    schedule_next: true,
                    cause: Default::default(),
                },
                time,
            )
            .unwrap();
            c.step_battle_execution_until(s, time + 2).unwrap();
        };
        c.units.get_mut(&a).unwrap().stats.current_health = 500;
        cast(&mut c, a, None, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        for (time, expected) in [(2, 527), (4, 517), (20, 544), (40, 534)] {
            strike(&mut c, &mut s, time);
            assert_eq!(c.units[&a].stats.current_health, expected, "hit at {time}");
            assert!(c.channel_active(a));
            assert!(c.resonance_use_has_obligations(a));
        }
        cast(&mut c, a, None, 44);
        strike(&mut c, &mut s, 45);
        assert_eq!(
            c.units[&a].stats.current_health, 524,
            "two actual equipped procs exhausted the shared battle budget"
        );
        assert!(!c.channel_active(a));
        assert!(!c.resonance_use_has_obligations(a));
    }

    #[test]
    fn composition_channel_support_stronger_source_ends_and_restores_other_owner() {
        for strong_first in [false, true] {
            let artifact = compile_json(&source().to_string()).unwrap();
            let (mut c, mut s, a, b) =
                fixture_from_native(artifact.load_native().unwrap(), true, true);
            let recipient = c
                .units
                .values()
                .find(|u| u.owner == Side::Player && u.instance_id != a && u.instance_id != b)
                .unwrap()
                .instance_id;
            place(&mut c, b, 2, 2);
            place(&mut c, recipient, 3, 1);
            for strong in [strong_first, !strong_first] {
                if strong {
                    cast(&mut c, a, Some(recipient), 0);
                } else {
                    cast(&mut c, b, None, 0);
                }
            }
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(
                c.aura_basic_bonus_for_occurrence(recipient, true).unwrap(),
                (40, 20)
            );
            cast(&mut c, a, None, 2);
            assert!(
                c.channel_active(b),
                "cancellation is scoped to the actual cast owner"
            );
            assert_eq!(
                c.aura_basic_bonus_for_occurrence(recipient, true).unwrap(),
                (20, 15)
            );
            cast(&mut c, b, None, 3);
            assert_eq!(
                c.aura_basic_bonus_for_occurrence(recipient, true).unwrap(),
                (0, 0)
            );
        }
    }

    #[test]
    fn composition_channel_support_cancelled_transaction_restores_start_stop_and_expiry() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        for (site, ending) in [
            ("sustained_started", "start"),
            ("flow_cancelled", "stop"),
            ("sustained_ended", "stop"),
            ("sustained_observation_committed", "expiry"),
        ] {
            let (mut c, mut s, a, _) = fixture(false, followup);
            c.units.get_mut(&a).unwrap().stats.current_health = 500;
            c.enable_authoring_observation();
            c.bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: 1,
                    battle_uuid: Uuid::from_u128(0xc532),
                },
            )
            .unwrap();
            cast(&mut c, a, None, 0);
            c.step_battle_execution_until(&mut s, if ending == "start" { 0 } else { 200 })
                .unwrap();
            let snapshot = |c: &BattleCore, s: &BattleExecutionState| {
                (
                    c.transaction_debug_snapshot(),
                    c.event_queue_debug_snapshot(),
                    serde_json::to_value(c.runtime_fact_view()).unwrap(),
                    format!("{s:?}"),
                    serde_json::to_value(c.authoring_observation().unwrap()).unwrap(),
                )
            };
            let before = snapshot(&c, &s);
            let time = if ending == "start" { 1 } else { 201 };
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || {
                        if ending == "stop" {
                            c.apply_live_command(BattleLiveCommand::ActivateSkill {
                                unit_id: a,
                                skill_id: "r11_aura".into(),
                                target: None,
                                time_ms: 200,
                            })
                            .map(|_| ())
                        } else {
                            c.step_battle_execution_until(&mut s, time).map(|_| ())
                        }
                    },
                )
            });
            assert!(
                result.is_err() && token.is_cancelled(),
                "{site}/{ending}: {result:?}"
            );
            assert_eq!(snapshot(&c, &s), before, "{site}/{ending}");
            if ending == "stop" {
                cast(&mut c, a, None, 200);
            } else {
                c.step_battle_execution_until(&mut s, time).unwrap();
            }
            assert_eq!(c.channel_active(a), ending == "start");
            if ending == "stop" {
                c.step_battle_execution_until(&mut s, 201).unwrap();
                assert_eq!(c.units[&a].stats.current_health, 500);
            }
        }
    }

    #[test]
    fn composition_channel_support_stop_preserves_released_sibling_delivery() {
        let (mut c, mut s, a, _) = fixture(false, |doc| {
            let channel = doc["skills"][0]["body"].clone();
            let amount = json!({"Literal":{"Amount":{"moment":"Snapshot","root":{"Const":{"value":17,"unit":"Magnitude"}}}}});
            let damage = json!({"Damage":{"id":"released-damage","tags":["released"],"damage_type":"True","amount":amount}});
            let delivery = json!({"Literal":{"Delivery":{"TileArea":{
                "footprint":{"rows":["XXXXXXX","XXXXXXX","XXXXXXX","XXX@XXX","XXXXXXX","XXXXXXX","XXXXXXX"],"include_anchor_tile":true},
                "area":{"anchor":"Caster","tile_origin":"Caster","tracking":"GroundFixed","hit_targets":"Enemies","first_tick_delay_ms":20}
            }}}});
            let impact = json!({"Action":{"id":"impact","target":{"Literal":{"Target":"SelfUnit"}},"delivery":delivery,"effects":[damage],"wait":"Released"}});
            doc["skills"][0]["body"] =
                json!({"Parallel":{"id":"support-and-impact","branches":[channel,impact]}});
        });
        let enemy = c
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        place(&mut c, enemy, 4, 1);
        let hp = c.units[&enemy].stats.current_health;
        cast(&mut c, a, None, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        cast(&mut c, a, None, 2);
        assert!(!c.channel_active(a));
        assert!(
            !c.resonance_use_has_obligations(a),
            "released area is independent of the declared ChannelAura use lease"
        );
        assert_eq!(
            c.active_areas.len(),
            1,
            "released carrier remains after its control owner stops"
        );
        c.step_battle_execution_until(&mut s, 21).unwrap();
        assert_eq!(c.units[&enemy].stats.current_health, hp - 17);
        assert!(!c.resonance_use_has_obligations(a));
    }
}
