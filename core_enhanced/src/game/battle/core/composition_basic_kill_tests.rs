mod composition_basic_kill {
    use super::compiled_action_fixture::{cast, fixture_content_with_data, hit, place};
    use super::*;
    use crate::game::battle::damage::DamageType;
    use crate::{
        authoring::{
            composition::{compile_json, parse_yaml, Artifact},
            node_graph::NativeGraphContent,
        },
        game::{
            ability::state_program::{StateScalar, StateView},
            battle::core::sim::BattleExecutionState,
        },
    };
    use serde_json::{json, Value};
    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/basic_kill.skill.yaml"
    ));
    type Fixture = (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    );
    fn program(doc: &mut Value) -> &mut Value {
        &mut doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]["State"]
            ["program"]
    }
    fn fixture(edit: impl FnOnce(&mut Value)) -> Fixture {
        fixture_weapon(edit, |_| {})
    }
    fn fixture_weapon(
        edit: impl FnOnce(&mut Value),
        weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
    ) -> Fixture {
        fixture_weapon_with_setup(edit, weapon, |_| {})
    }
    fn fixture_weapon_with_setup(
        edit: impl FnOnce(&mut Value),
        weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
        configure: impl FnOnce(&mut BattleScenario),
    ) -> Fixture {
        let mut doc = parse_yaml(YAML).unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        edit(&mut doc);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let (mut c, s, a, e, b) = fixture_content_with_data(
            &["piercing-damage-increase"],
            &[],
            NativeGraphContent {
                skills: artifact.load_native().unwrap(),
                equipment: vec![],
                mark_weapons: vec![],
            },
            weapon,
            crate::game::resource::ResourceCatalog {
                resources: artifact.resources().to_vec(),
            },
            |b| b.with_equipment_attacks(artifact.equipment_attacks().to_vec())
                .with_equipment_supports(artifact.equipment_supports().to_vec()),
            configure,
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
    fn states(c: &BattleCore) -> Vec<StateView> {
        c.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter_map(|r| r.state)
            .collect()
    }
    fn start(c: &mut BattleCore, a: UnitInstanceId, e: UnitInstanceId, now: u64, regular: bool) {
        c.units.get_mut(&a).unwrap().next_basic_attack_ms = now;
        c.event_queue.push(BattleEvent::AttackStart {
            time_ms: now,
            attacker_instance_id: a,
            target_instance_id: Some(e),
            schedule_next: regular,
            cause: Default::default(),
        });
    }
    #[test]
    fn composition_basic_kill_multi_target_dedup_and_primary_are_authored() {
        for (primary, once, primary_hp, expected) in [
            (false, true, 30, 1),
            (false, false, 30, 2),
            (true, false, 30, 1),
            (true, false, 1000, 0),
        ] {
            let (mut c, mut s, a, e, b) = fixture(|doc| {
                program(doc)["handlers"][0]["event"] =
                    json!({"BasicAttackKill":{"primary_only":primary,"once_per_action":once}});
            });
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            c.units.get_mut(&e).unwrap().stats.current_health = primary_hp;
            c.units.get_mut(&b).unwrap().stats.current_health = 30;
            start(&mut c, a, e, 100, true);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert!(c.units[&b].lifecycle.is_dead());
            assert_eq!(states(&c)[0].events, expected);
            assert_eq!(
                states(&c)[0].fields[0].value,
                StateScalar::Integer(expected as i128)
            );
            assert_eq!(
                c.units[&a].stats.current_health,
                500 + 100 * expected as u32
            );
        }
    }
    #[test]
    fn composition_basic_kill_triggered_attack_is_excluded() {
        let (mut c, mut s, a, e, b) = fixture(|_| {});
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        for target in [e, b] {
            c.units.get_mut(&target).unwrap().stats.current_health = 30;
        }
        start(&mut c, a, e, 100, false);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        assert!(c.units[&e].lifecycle.is_dead());
        assert_eq!(states(&c)[0].events, 0);
        assert_eq!(c.units[&a].stats.current_health, 500);
    }
    #[test]
    fn composition_basic_kill_base_and_equipment_bonus_after_absorption() {
        use crate::game::{
            ability::sustained::SustainedEffectDef,
            stats::{Effect, TriggerEffectTarget, TriggerType, TriggeredEffect},
        };
        for (hp, shield, expected) in [(30, 0, 1), (60, 0, 0), (30, 20, 0), (30, 10, 1)] {
            let (mut c, mut s, a, e, b) = fixture_weapon(
                |_| {},
                |w| {
                    w.triggered_effects.insert(
                        TriggerType::OnAttack,
                        vec![TriggeredEffect::targeted(
                            TriggerEffectTarget::CounterpartUnit,
                            Effect::BonusDamage {
                                flat: 100,
                                percent: 0,
                                damage_type: DamageType::True,
                            },
                        )],
                    );
                },
            );
            place(&mut c, b, 6, 3);
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            c.units.get_mut(&e).unwrap().stats.current_health = hp;
            if shield > 0 {
                c.begin_sustained(
                    e,
                    e,
                    SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: shield,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    1,
                )
                .unwrap();
            }
            start(&mut c, a, e, 100, true);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert!(c.units[&e].lifecycle.is_dead());
            assert_eq!(states(&c)[0].events, expected, "hp={hp}, shield={shield}");
            assert_eq!(
                c.units[&a].stats.current_health,
                500 + 100 * expected as u32
            );
        }
    }
    #[test]
    fn composition_basic_kill_projectile_subscription_lifetime() {
        use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
        use crate::game::battle::core::sim::BattleLiveCommand;
        for mode in ["live", "late", "replace", "expire", "withdraw"] {
            let (mut c, mut s, a, e, b) = fixture(|doc| {
                program(doc)["basic_attack"] = Value::Null;
                program(doc)["owns_use"] = json!(false);
                doc["skills"][0]["admission"]["requirement"] = json!("None");
                doc["skills"][0]["admission"]["cost"] = json!("Zero");
                if mode == "expire" {
                    doc["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]
                        ["State"]["duration_ms"] = json!(500);
                }
            });
            place(&mut c, b, 6, 3);
            c.units.get_mut(&a).unwrap().basic_attack.range_policy =
                crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
            c.units.get_mut(&a).unwrap().basic_attack.delivery = DeliveryDef::Projectile {
                speed_units_per_ms: 2000,
                hit_policy: ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            };
            c.units.get_mut(&a).unwrap().stats.attack = 100;
            c.units.get_mut(&e).unwrap().stats.current_health = 30;
            if mode != "late" {
                cast(&mut c, a, 0);
            }
            c.step_battle_execution_until(&mut s, 1).unwrap();
            start(&mut c, a, e, 100, true);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert!(!c.projectiles.is_empty(), "{mode}");
            if mode == "late" || mode == "replace" {
                cast(&mut c, a, 200);
            } else if mode == "withdraw" {
                c.apply_live_command(BattleLiveCommand::WithdrawUnit {
                    unit_id: a,
                    time_ms: 200,
                })
                .unwrap();
            }
            c.step_battle_execution_until(&mut s, 1500).unwrap();
            assert!(c.units[&e].lifecycle.is_dead(), "{mode}");
            if mode == "expire" || mode == "withdraw" {
                assert!(states(&c).is_empty(), "{mode}");
            } else {
                assert_eq!(
                    states(&c)[0].events,
                    if mode == "live" { 1 } else { 0 },
                    "{mode}"
                );
                assert_eq!(
                    c.units[&a].stats.current_health,
                    if mode == "live" { 600 } else { 500 },
                    "{mode}"
                );
            }
        }
    }
    #[test]
    fn composition_basic_kill_reaction_cancellation_restores_death_fields_and_heal() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut c, mut s, a, e, _) = fixture(|_| {});
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 30;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 100).unwrap();
        let before = (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
            format!("{s:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("state_reaction_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || c.step_battle_execution_until(&mut s, 101),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
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
        assert!(c.units[&e].lifecycle.is_dead());
        assert_eq!(states(&c)[0].events, 1);
        assert_eq!(c.units[&a].stats.current_health, 600);
    }

    #[test]
    fn composition_basic_kill_two_actions_and_observation_modes_keep_rewards() {
        let mut outcomes = Vec::new();
        for observe in [false, true] {
            let (mut c, mut s, a, e, b) = fixture(|_| {});
            if observe {
                c.enable_authoring_observation_with_options(Default::default(), true);
                c.bind_authoring_observation(
                    crate::game::battle::core::authoring_observation::ObservationKey {
                        entry_ordinal: 1,
                        battle_uuid: Uuid::from_u128(0xb13),
                    },
                )
                .unwrap();
            }
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            c.units.get_mut(&e).unwrap().stats.current_health = 30;
            c.units.get_mut(&b).unwrap().stats.current_health = 70;
            start(&mut c, a, e, 100, true);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(states(&c)[0].events, 1);
            assert_eq!(c.units[&b].stats.current_health, 30);
            start(&mut c, a, b, 200, true);
            c.step_battle_execution_until(&mut s, 201).unwrap();
            assert_eq!(states(&c)[0].events, 2);
            assert_eq!(c.units[&a].stats.current_health, 700);
            outcomes.push((
                c.units[&a].stats.current_health,
                states(&c),
                serde_json::to_value(c.runtime_fact_view()).unwrap(),
            ));
        }
        assert_eq!(outcomes[0], outcomes[1]);
    }

    #[test]
    fn composition_basic_kill_handlers_and_stacked_activations_are_independent() {
        let (mut c, mut s, a, e, b) = fixture(|doc| {
            program(doc)["owns_use"] = json!(false);
            program(doc)["replacement"] = json!("Stack");
            let mut second = program(doc)["handlers"][0].clone();
            second["id"] = json!("second-reward");
            program(doc)["handlers"]
                .as_array_mut()
                .unwrap()
                .push(second);
            doc["skills"][0]["admission"]["requirement"] = json!("None");
            doc["skills"][0]["admission"]["cost"] = json!("Zero");
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        cast(&mut c, a, 2);
        c.step_battle_execution_until(&mut s, 3).unwrap();
        assert_eq!(states(&c).len(), 2);
        for id in [e, b] {
            c.units.get_mut(&id).unwrap().stats.current_health = 30;
        }
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 101).unwrap();
        for state in states(&c) {
            assert_eq!(state.events, 2);
            assert_eq!(state.fields[0].value, StateScalar::Integer(2));
        }
        assert_eq!(c.units[&a].stats.current_health, 900);
    }

    #[test]
    fn composition_basic_kill_independent_damage_and_ended_subscription_do_not_repeat() {
        for independent in [false, true] {
            let (mut c, mut s, a, e, b) = fixture(|doc| {
                program(doc)["handlers"][0]["event"]["BasicAttackKill"]["once_per_action"] =
                    json!(false);
                program(doc)["handlers"][0]["body"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("End"));
            });
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            for id in [e, b] {
                c.units.get_mut(&id).unwrap().stats.current_health = 30;
            }
            if independent {
                hit(&mut c, a, e, 30, 100);
                assert!(c.units[&e].lifecycle.is_dead());
                assert_eq!(states(&c)[0].events, 0);
                assert_eq!(c.units[&a].stats.current_health, 500);
            } else {
                start(&mut c, a, e, 100, true);
                c.step_battle_execution_until(&mut s, 101).unwrap();
                assert!(c.units[&e].lifecycle.is_dead() && c.units[&b].lifecycle.is_dead());
                assert!(states(&c).is_empty());
                assert_eq!(c.units[&a].stats.current_health, 600);
            }
        }
    }

    #[test]
    fn composition_basic_kill_invalid_field_rolls_back_death_and_latches_fault() {
        let (mut c, mut s, a, e, _) = fixture(|doc| {
            program(doc)["fields"][0]["value"]["Integer"]["max"] = json!(0);
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 30;
        start(&mut c, a, e, 100, true);
        let error = c.step_battle_execution_until(&mut s, 101).unwrap_err();
        assert!(
            matches!(error, GameError::ValueExpressionFault(_)),
            "{error:?}"
        );
        assert_eq!(c.units[&e].stats.current_health, 30);
        assert!(c.units[&e].is_active());
        assert_eq!(c.units[&a].stats.current_health, 500);
        assert_eq!(states(&c)[0].events, 0);
        assert_eq!(states(&c)[0].fields[0].value, StateScalar::Integer(0));
        assert!(c.step_battle_execution_until(&mut s, 201).is_err());
    }
    fn resource_fixture(lifetime: &str, edit: impl FnOnce(&mut Value)) -> Fixture {
        let (mut c, s, a, e, b) = fixture(|doc| {
            *doc = parse_yaml(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/skill_composition/state_resources.skill.yaml"
            )))
            .unwrap();
            doc["skills"][0]["id"] = json!("r11_aura");
            doc["resources"][0]["lifetime"] = json!(lifetime);
            edit(doc);
        });
        // A raw BattleScenario has no World inventory. Mount its explicit
        // fixture inventory through the same Run boundary as World.
        let owners = c.units.values().flat_map(|u| {
            std::iter::once(crate::game::resource::ResourceOwner::Employee(u.source_owned_uuid))
                .filter(|_| !u.source_owned_uuid.is_nil())
                .chain(u.fragment_source.as_ref().map(|f| crate::game::resource::ResourceOwner::FragmentCopy(f.copy_uuid)))
        }).collect();
        c.mount_run_resources(&Default::default(), &owners).unwrap();
        (c, s, a, e, b)
    }
    fn resource_balances(c: &BattleCore) -> Vec<u32> {
        c.typed_resources()
            .unwrap()
            .entries()
            .filter(|(k, _)| k.resource.as_str() == "kill_meter")
            .map(|(_, v)| v.balance)
            .collect()
    }
    #[test]
    fn composition_state_resource_owned_scopes_expire_with_real_state() {
        use crate::game::resource::ResourceOwner;
        for lifetime in [
            "Run",
            "Attempt",
            "Deployment",
            "Activation",
            "Chain",
            "Delivery",
        ] {
            let (mut c, mut s, a, e, _) = resource_fixture(lifetime, |doc| {
                if lifetime == "Activation" {
                    doc["definitions"][0]["body"]["Action"]["wait"] = json!("EffectsSettled");
                }
            });
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            c.units.get_mut(&e).unwrap().stats.current_health = 30;
            start(&mut c, a, e, 100, true);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(resource_balances(&c), vec![4], "{lifetime}");
            assert_eq!(states(&c)[0].events, 1);
            assert_eq!(c.units[&a].stats.current_health, 600);
            let copy = c.units[&a].fragment_source.as_ref().unwrap().copy_uuid;
            assert!(c
                .typed_resources()
                .unwrap()
                .entries()
                .all(|(k, _)| k.owner == ResourceOwner::FragmentCopy(copy)));
            c.step_battle_execution_until(&mut s, 2001).unwrap();
            assert!(states(&c).is_empty());
            assert_eq!(
                resource_balances(&c),
                if matches!(lifetime, "Run" | "Attempt" | "Deployment") {
                    vec![4]
                } else {
                    vec![]
                },
                "{lifetime}"
            );
        }
    }
    #[test]
    fn composition_state_resource_cancellation_restores_death_fields_heal_and_balance() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        // Before/after resources and after heal must all restore the same input boundary.
        for checkpoint in [1, 2] {
            let (mut c, mut s, a, e, _) = resource_fixture("Attempt", |_| {});
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
            let result =
                control::with_test_cancellation("state_reaction_applied", checkpoint, || {
                    control::run(
                        &ExecutionLimits::default(),
                        &token,
                        ExecutionPhase::TrialExecution,
                        || c.step_battle_execution_until(&mut s, 101),
                    )
                });
            assert!(result.is_err() && token.is_cancelled());
            assert_eq!(snapshot(&c, &s), before);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(resource_balances(&c), vec![4]);
            assert_eq!(states(&c)[0].events, 1);
            assert_eq!(c.units[&a].stats.current_health, 600);
        }
    }
    #[test]
    fn composition_state_resource_failed_batch_rolls_back_the_whole_kill() {
        let (mut c, mut s, a, e, _) = resource_fixture("Attempt", |doc| {
            program(doc)["handlers"][0]["body"][1]["If"]["then_ops"][0]["Resources"]
                ["operations"][2]["operation"] = json!({"Consume":9});
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 30;
        start(&mut c, a, e, 100, true);
        c.step_battle_execution_until(&mut s, 100).unwrap();
        let before = (
            states(&c),
            resource_balances(&c),
            c.units[&a].stats.current_health,
            c.units[&e].stats.current_health,
        );
        let result = c.step_battle_execution_until(&mut s, 101);
        assert!(
            matches!(result, Err(GameError::BattleRuntimeFault(ref fault))
            if fault.code == crate::game::battle::runtime_control::RuntimeFaultCode::Resource(crate::game::resource::store::ResourceFaultCode::Insufficient)),
            "{result:?}"
        );
        assert_eq!(
            (
                states(&c),
                resource_balances(&c),
                c.units[&a].stats.current_health,
                c.units[&e].stats.current_health
            ),
            before
        );
        assert!(!c.units[&e].lifecycle.is_dead());
        assert!(c.step_battle_execution_until(&mut s, 201).is_err());
    }
    #[test]
    fn composition_state_resource_expired_handler_uses_its_own_live_delivery_then_cleans_up() {
        let (mut c, mut s, a, _, _) = resource_fixture("Delivery", |doc| {
            program(doc)["handlers"][0]["event"] = json!("Expired");
        });
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        assert!(resource_balances(&c).is_empty());
        c.step_battle_execution_until(&mut s, 2001).unwrap();
        assert_eq!(c.units[&a].stats.current_health, 600);
        assert!(states(&c).is_empty());
        assert!(resource_balances(&c).is_empty());
    }
    #[test]
    fn composition_state_resource_independent_state_owns_chain_and_delivery_but_not_activation() {
        for lifetime in ["Chain", "Delivery", "Activation"] {
            let (mut c, mut s, a, e, _) = resource_fixture(lifetime, |doc| {
                program(doc)["owns_use"] = json!(false);
            });
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            c.units.get_mut(&e).unwrap().stats.current_health = 30;
            start(&mut c, a, e, 100, true);
            let result = c.step_battle_execution_until(&mut s, 101);
            if lifetime == "Activation" {
                assert!(result.is_err());
                assert_eq!(states(&c)[0].events, 0);
                assert!(resource_balances(&c).is_empty());
                assert!(c.units[&e].is_active());
            } else {
                result.unwrap();
                assert_eq!(resource_balances(&c), vec![4]);
                c.step_battle_execution_until(&mut s, 2001).unwrap();
                assert!(resource_balances(&c).is_empty());
            }
        }
    }
    #[test]
    fn composition_state_resource_authored_second_event_order_and_observation_parity() {
        let mut outcomes = Vec::new();
        for observe in [false, true] {
            let (mut c, mut s, a, e, b) = resource_fixture("Attempt", |doc| {
                program(doc)["handlers"][0]["event"]["BasicAttackKill"]["once_per_action"] =
                    json!(false);
                program(doc)["handlers"].as_array_mut().unwrap().push(json!({
                    "id":"expire-cost", "event":"Expired", "body":[{"Resources":{"operations":[
                        {"resource":"kill_meter","owner":"SourceFragment","operation":{"Consume":3}}
                    ]}}]
                }));
            });
            if observe {
                c.enable_authoring_observation_with_options(Default::default(), true);
                c.bind_authoring_observation(
                    crate::game::battle::core::authoring_observation::ObservationKey {
                        entry_ordinal: 1,
                        battle_uuid: Uuid::from_u128(0xb14),
                    },
                )
                .unwrap();
            }
            cast(&mut c, a, 0);
            c.step_battle_execution_until(&mut s, 1).unwrap();
            for id in [e, b] {
                c.units.get_mut(&id).unwrap().stats.current_health = 30;
            }
            start(&mut c, a, e, 100, true);
            c.step_battle_execution_until(&mut s, 101).unwrap();
            assert_eq!(resource_balances(&c), vec![8]);
            assert_eq!(states(&c)[0].events, 2);
            c.step_battle_execution_until(&mut s, 2001).unwrap();
            assert_eq!(resource_balances(&c), vec![5]);
            outcomes.push((
                resource_balances(&c),
                c.units[&a].stats.current_health,
                serde_json::to_value(c.runtime_fact_view()).unwrap(),
            ));
        }
        assert_eq!(outcomes[0], outcomes[1]);
    }
    #[test]
    fn composition_state_resource_released_action_cannot_revive_activation_scope() {
        let (mut c, mut s, a, e, _) = resource_fixture("Activation", |_| {});
        cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1).unwrap();
        c.units.get_mut(&e).unwrap().stats.current_health = 30;
        start(&mut c, a, e, 100, true);
        let result = c.step_battle_execution_until(&mut s, 101);
        assert!(
            matches!(result, Err(GameError::BattleRuntimeFault(ref fault))
            if fault.code == crate::game::battle::runtime_control::RuntimeFaultCode::Resource(crate::game::resource::store::ResourceFaultCode::StaleScope)),
            "{result:?}"
        );
        assert!(resource_balances(&c).is_empty());
        assert_eq!(states(&c)[0].events, 0);
        assert!(c.units[&e].is_active());
    }
    #[test]
    fn composition_state_resource_source_and_recipient_owners_stay_distinct_after_source_exit() {
        use crate::game::{battle::{core::sim::BattleLiveCommand, event_log::SkillCastTarget}, resource::ResourceOwner};
        for withdraw in [false, true] {
            let (mut c, mut s, a, _, b) = resource_fixture("Attempt", |doc| {
                doc["resources"][0]["owner"] = json!("Employee");
                doc["definitions"][0]["body"]["Action"]["target"]["Literal"]["Target"] = json!("CastTarget");
                doc["skills"][0]["kind"] = json!("Targeted");
                doc["skills"][0]["cast_targeting"]["Explicit"] = json!({
                    "selection":"player_unit","target":"AllySingle","reach":"global","air_capable":true
                });
                let p = program(doc);
                p["basic_attack"] = Value::Null;
                p["owns_use"] = json!(false);
                p["end_on_source_exit"] = json!(false);
                p["handlers"][0]["event"] = json!("Expired");
                p["handlers"][0]["body"].as_array_mut().unwrap().truncate(2);
                p["handlers"][0]["body"][1]["If"]["then_ops"][0]["Resources"]["operations"] = json!([
                    {"resource":"kill_meter","owner":"TargetEmployee","operation":{"Grant":4}},
                    {"resource":"kill_meter","owner":"SourceEmployee","operation":{"Grant":7}}
                ]);
            });
            c.units.get_mut(&b).unwrap().owner = Side::Player;
            c.apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id:a, skill_id:"r11_aura".into(), target:Some(SkillCastTarget::Unit{unit_instance_id:b}), time_ms:0,
            }).unwrap();
            c.step_battle_execution_until(&mut s, 1).unwrap();
            assert_eq!(states(&c).len(), 1);
            if withdraw {
                c.apply_live_command(BattleLiveCommand::WithdrawUnit{unit_id:a,time_ms:500}).unwrap();
            }
            c.step_battle_execution_until(&mut s, 2001).unwrap();
            let entries: Vec<_> = c.typed_resources().unwrap().entries().collect();
            assert_eq!(entries.len(), 2);
            for (id, expected) in [(a,7),(b,4)] {
                let owner = ResourceOwner::Employee(c.units[&id].source_owned_uuid);
                assert_eq!(entries.iter().find(|(k,_)| k.owner == owner).unwrap().1.balance, expected);
            }
            assert!(states(&c).is_empty());
        }
    }

    include!("composition_state_reward_tests.rs");
    include!("composition_prepared_state_tests.rs");
    include!("composition_headshot_resonance_tests.inc.rs");
    include!("composition_kill_eligibility_tests.rs");
}
