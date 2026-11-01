mod composition_channel_opening {
    use super::*;
    use crate::game::ability::sustained::SustainedEffectDef;
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/channel_opening_guard.skill.yaml"
    ));
    fn setup() -> Fixture {
        fixture_extra_with_defense(
            "Damage(amount:100,damage_type:True)",
            false,
            None,
            false,
            &crate::authoring::composition::canonical_channel_tests::equipment_fixture("armor").to_string(),
            |_| {},
            |_| {},
        )
    }
    fn aura(duration: u32, upkeep: bool) -> SustainedEffectDef {
        ron::de::from_str(&format!(r#"ChannelAura(duration_ms:{duration},aura:(family:"opening-test",priority:0,radius_units:10000000,basic_damage_percent:1,basic_speed_points:0,health_upkeep:{}))"#,
            if upkeep { "Some((interval_ms:250,max_health_percent:10))" } else { "None" })).unwrap()
    }
    fn start(f: &mut Fixture, now: u64, duration: u32, upkeep: bool) {
        let actor = f.caster;
        f.core
            .runtime_transaction(|c| c.begin_sustained(actor, actor, aura(duration, upkeep), now))
            .unwrap();
        assert_eq!(f.core.channel_started_at(actor, now).unwrap(), Some(now));
    }
    fn damage(
        f: &mut Fixture,
        actor: UnitInstanceId,
        source: DamageSource,
        kind: DamageType,
        now: u64,
    ) -> u32 {
        let target = f.caster;
        let before = f.core.units[&target].stats.current_health;
        f.core
            .runtime_transaction(|c| {
                let snapshot = c
                    .damage_source_snapshot_for_unit(
                        actor,
                        target,
                        source,
                        kind,
                        100,
                        Default::default(),
                        0,
                        now,
                        false,
                    )
                    .unwrap();
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: target,
                        source_snapshot: snapshot,
                    }],
                    now,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        before - f.core.units[&target].stats.current_health
    }
    #[test]
    fn composition_channel_opening_schema_roundtrip_and_rejects_invalid_shared_values() {
        let doc = parse_yaml(SOURCE).unwrap();
        let a = compile_json(&doc.to_string()).unwrap();
        let b = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert!(matches!(
            b.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::ChannelOpeningGuard {
                duration_ms: 2000,
                reduction_percent: 50
            }
        ));
        for (field, value) in [
            ("duration_ms", 0),
            ("reduction_percent", 0),
            ("reduction_percent", 101),
        ] {
            let mut bad = doc.clone();
            bad["values"][0]["value"]["DefenseProgram"]["ChannelOpeningGuard"][field] =
                json!(value);
            let error = compile_json(&bad.to_string()).unwrap_err();
            assert!(error.to_string().contains(field), "{error:?}");
        }
    }
    #[test]
    fn composition_channel_opening_exact_window_all_damage_kinds_and_natural_end() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            let mut f = setup();
            let e = f.target;
            assert_eq!(damage(&mut f, e, DamageSource::BasicAttack, kind, 0), 100);
            start(&mut f, 10, 4000, false);
            assert_eq!(damage(&mut f, e, DamageSource::BasicAttack, kind, 10), 50);
            assert_eq!(damage(&mut f, e, DamageSource::Ability, kind, 2009), 50);
            assert_eq!(damage(&mut f, e, DamageSource::BuffTick, kind, 2010), 100);
            assert!(
                f.core.channel_active(f.caster),
                "guard expiry must not end channel"
            );
            assert_eq!(f.core.channel_started_at(f.caster, 4010).unwrap(), None);
            assert_eq!(damage(&mut f, e, DamageSource::Ability, kind, 4010), 100);
        }
    }
    #[test]
    fn composition_channel_opening_stop_and_new_channel_reset_without_residual_guard() {
        let mut f = setup();
        let e = f.target;
        let a = f.caster;
        start(&mut f, 10, 4000, false);
        assert_eq!(
            damage(&mut f, e, DamageSource::Ability, DamageType::True, 10),
            50
        );
        f.core
            .runtime_transaction(|c| c.stop_channel(a, 11))
            .unwrap();
        assert_eq!(f.core.channel_started_at(a, 11).unwrap(), None);
        assert_eq!(
            damage(&mut f, e, DamageSource::Ability, DamageType::True, 11),
            100
        );
        start(&mut f, 12, 4000, false);
        assert_eq!(
            damage(&mut f, e, DamageSource::Ability, DamageType::True, 12),
            50
        );
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 13,
            })
            .unwrap();
        assert!(!f.core.channel_active(a));
        assert_eq!(f.core.channel_started_at(a, 13).unwrap(), None);
    }
    #[test]
    fn composition_channel_opening_self_friendly_environment_and_upkeep_are_unreduced() {
        let mut f = setup();
        let a = f.caster;
        let b = f.ally;
        let e = f.target;
        start(&mut f, 10, 4000, true);
        assert_eq!(
            damage(&mut f, a, DamageSource::Ability, DamageType::True, 10),
            100
        );
        assert_eq!(
            damage(&mut f, b, DamageSource::Ability, DamageType::True, 11),
            100
        );
        assert_eq!(
            damage(&mut f, e, DamageSource::Environment, DamageType::True, 12),
            100
        );
        let hp = f.core.units[&a].stats.current_health;
        f.core
            .step_battle_execution_until(&mut f.state, 260)
            .unwrap();
        assert_eq!(
            hp - f.core.units[&a].stats.current_health,
            100,
            "10% max HP cost is not damage"
        );
        assert_eq!(
            damage(&mut f, e, DamageSource::BuffTick, DamageType::True, 261),
            50
        );
    }
    #[test]
    fn composition_channel_opening_cancelled_start_and_damage_restore_exact_state_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        let a = f.caster;
        let before = f.core.transaction_debug_snapshot();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("sustained_started", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    f.core
                        .runtime_transaction(|c| c.begin_sustained(a, a, aura(4000, false), 10))
                },
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        start(&mut f, 10, 4000, false);
        let before = f.core.transaction_debug_snapshot();
        let token = CancellationToken::default();
        let e = f.target;
        let result = control::with_test_cancellation("channel_opening_evaluated", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    f.core.runtime_transaction(|c| {
                        let source = c
                            .damage_source_snapshot_for_unit(
                                e,
                                a,
                                DamageSource::Ability,
                                DamageType::True,
                                100,
                                Default::default(),
                                0,
                                11,
                                false,
                            )
                            .unwrap();
                        c.process_commands(
                            vec![BattleCommand::ApplyDamage {
                                target_id: a,
                                source_snapshot: source,
                            }],
                            11,
                        );
                        crate::game::battle::runtime_control::poll()
                    })
                },
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            damage(&mut f, e, DamageSource::Ability, DamageType::True, 11),
            50
        );
    }
    #[test]
    fn composition_channel_opening_all_authored_channel_families_use_actual_release() {
        use crate::authoring::node_graph::{compile, tests as graphs, PortableGraph};
        use crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::fixture_content_with_data;
        for family in ["ground", "counter", "volley", "relation"] {
            let doc = match family {
                "ground" => graphs::channel_ground_fixture(),
                "counter" => graphs::counter_channel_fixture(false),
                "volley" => graphs::prepared_volley_fixture(),
                _ => graphs::relation_action_fixture(true),
            };
            let graph = compile(&doc.to_string(), "r11_aura".into()).unwrap();
            let native = PortableGraph::from_ron(&graph.to_ron().unwrap())
                .unwrap()
                .load_content()
                .unwrap();
            let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
            let relics = [
                "long-breath",
                "widened-horizon",
                "unbroken-trance",
                "broken-balance",
                "tightening-shackles",
                "double-harvest",
                "hasty-second-hand",
                "healing-increase",
                "lingering-warmth",
                "deep-wedge",
                "king-of-greed-stunning-treasures",
                "king-of-greed-endless-treasury",
                "bottomless-cup",
                "deep-imprint",
                "layered-shell",
                "next",
            ];
            let (mut core, state, caster, target, ally) = fixture_content_with_data(
                &relics,
                &[],
                native,
                |_| {},
                crate::game::resource::ResourceCatalog { resources: vec![] },
                |b| b.with_equipment_defenses(artifact.equipment_defenses().to_vec()),
                |_| {},
            );
            core.units.get_mut(&ally).unwrap().fragment_source = None;
            core.units.get_mut(&ally).unwrap().skill_id = None;
            for u in core.units.values_mut() {
                u.stats.current_health = 10000;
                u.stats.max_health = 10000;
                u.stats.defense = 0;
                u.stats.magic_resist = 0;
                u.stats.move_speed_units_per_ms = 0;
            }
            core.runtime_transaction(|c| {
                c.clear_counter_channels(0)?;
                c.reconcile_counter_channels(0)?;
                c.clear_prepared_volleys(0)?;
                c.reconcile_prepared_volleys(0)?;
                c.clear_relation_actions(0)?;
                c.reconcile_relation_actions(0)
            })
            .unwrap();
            let mut f = Fixture {
                core,
                state,
                caster,
                target,
                ally,
            };
            assert_eq!(
                f.core.channel_started_at(caster, 0).unwrap(),
                None,
                "{family}"
            );
            f.core
                .apply_live_command(BattleLiveCommand::ActivateSkill {
                    unit_id: caster,
                    skill_id: "r11_aura".into(),
                    target: matches!(family, "relation" | "counter").then_some(
                        crate::game::battle::event_log::SkillCastTarget::Tile {
                            position: Position::new(3, 1),
                        },
                    ),
                    time_ms: 0,
                })
                .unwrap();
            f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
            assert_eq!(
                f.core.channel_started_at(caster, 1).unwrap(),
                Some(if family == "ground" { 1 } else { 0 }),
                "{family}"
            );
            assert_eq!(
                damage(&mut f, target, DamageSource::Ability, DamageType::True, 2),
                50,
                "{family}"
            );
            f.core
                .runtime_transaction(|c| {
                    c.stop_channel_with_reason(
                        caster,
                        3,
                        crate::game::ability::sustained::SustainedEndReason::HardCrowdControl,
                    )
                })
                .unwrap_or_else(|e| panic!("{family}: {e:?}"));
            assert_eq!(
                f.core.channel_started_at(caster, 3).unwrap(),
                None,
                "{family}"
            );
            assert_eq!(
                damage(&mut f, target, DamageSource::Ability, DamageType::True, 3),
                100,
                "{family}"
            );
        }
    }

    #[test]
    fn composition_channel_opening_parallel_leases_preserve_first_start_after_oldest_expires() {
        let mut doc = parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/channel_ground.skill.yaml"
        )))
        .unwrap();
        doc["skills"][0]["id"] = json!("r11_aura");
        doc["skills"][0]["prepare_ms"] = json!(100);
        doc["definitions"][0]["body"]["Action"]["modifiers"] = json!([]);
        doc["values"][0]["value"]["Delivery"]["TileArea"]["area"]["duration_ms"] = json!(1500);
        let mut longer = doc["values"][0].clone();
        longer["name"] = json!("second-ground");
        longer["value"]["Delivery"]["TileArea"]["area"]["duration_ms"] = json!(4000);
        doc["values"].as_array_mut().unwrap().push(longer);
        let mut first = doc["skills"][0]["body"].clone();
        first["Call"]["id"] = json!("first");
        let mut second = first.clone();
        second["Call"]["id"] = json!("second");
        second["Call"]["arguments"][0]["value"] = json!({"Shared":"second-ground"});
        doc["skills"][0]["body"] = json!({"Parallel":{"id":"overlap","branches":[first,{"Sequence":{"id":"later","children":[{"Wait":{"id":"delay","duration":{"Literal":{"Milliseconds":1000}}}},second]}}]}});
        let guard = parse_yaml(SOURCE).unwrap();
        doc["values"]
            .as_array_mut()
            .unwrap()
            .push(guard["values"][0].clone());
        doc["equipment_defenses"] = guard["equipment_defenses"].clone();
        let artifact = compile_json(&doc.to_string()).unwrap();
        let authored = ron::ser::to_string(&artifact.load_native().unwrap()).unwrap();
        let (mut core, state, caster, ally) = r12_fragment_equipped_effect_core_with_data(
            &authored,
            |s| {
                s.groups[0].enemy_movement_plan = None;
            },
            |_| {},
            None,
            |b| b.with_equipment_defenses(artifact.equipment_defenses().to_vec()),
        );
        let target = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for u in core.units.values_mut() {
            u.stats.current_health = 1000;
            u.stats.max_health = 1000;
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
            u.stats.move_speed_units_per_ms = 0;
            u.next_basic_attack_ms = 100000;
            u.stats.attack_interval_ms = 100000;
        }
        let mut f = Fixture {
            core,
            state,
            caster,
            target,
            ally,
        };
        f.core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: caster,
                skill_id: "r11_aura".into(),
                target: None,
                time_ms: 0,
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 99)
            .unwrap();
        assert_eq!(f.core.channel_started_at(caster, 99).unwrap(), None);
        assert_eq!(
            damage(&mut f, target, DamageSource::Ability, DamageType::True, 99),
            100
        );
        f.core
            .step_battle_execution_until(&mut f.state, 101)
            .unwrap();
        assert_eq!(f.core.channel_started_at(caster, 101).unwrap(), Some(100));
        assert_eq!(
            damage(&mut f, target, DamageSource::Ability, DamageType::True, 101),
            50
        );
        f.core
            .step_battle_execution_until(&mut f.state, 1101)
            .unwrap();
        assert_eq!(f.core.active_areas.len(), 2);
        assert_eq!(f.core.channel_started_at(caster, 1101).unwrap(), Some(100));
        f.core
            .step_battle_execution_until(&mut f.state, 1601)
            .unwrap();
        assert_eq!(f.core.active_areas.len(), 1);
        assert_eq!(f.core.channel_started_at(caster, 1601).unwrap(), Some(100));
        assert_eq!(
            damage(
                &mut f,
                target,
                DamageSource::Ability,
                DamageType::True,
                2099
            ),
            50
        );
        assert_eq!(
            damage(
                &mut f,
                target,
                DamageSource::Ability,
                DamageType::True,
                2100
            ),
            100
        );
        assert!(f.core.channel_active(caster));
        f.core
            .step_battle_execution_until(&mut f.state, 5101)
            .unwrap();
        assert_eq!(f.core.channel_started_at(caster, 5101).unwrap(), None);
    }
    #[test]
    fn composition_channel_opening_duplicate_providers_apply_once_and_keep_other_reduction() {
        let catalog: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        let mut weapon = catalog
            .get_by_id("standard_training_sword")
            .unwrap()
            .clone();
        weapon.id = "r11_weapon".into();
        weapon.uuid = Uuid::from_u128(0x1101);
        weapon.triggered_effects.clear();
        weapon.ability_activations.clear();
        let mut armor = catalog
            .get_by_id("placeholder_burrowing_heaven_armor")
            .unwrap()
            .clone();
        armor.id = "opening_fixture_armor".into();
        let armor_id = armor.uuid;
        let mut doc = parse_yaml(SOURCE).unwrap();
        doc["equipment_defenses"].as_array_mut().unwrap().push(
            json!({"equipment":"opening_fixture_armor","program":{"Shared":"opening-protection"}}),
        );
        let artifact = compile_json(&doc.to_string()).unwrap();
        let (mut core, state, caster, ally) = r12_fragment_equipped_effect_core_with_data(
            &r12_control_ron("Damage(amount:100,damage_type:True)"),
            |s| {
                for (index, spawn) in s.groups[1].spawns.iter_mut().enumerate() {
                    let mut second = spawn.draft.equipped_items[0];
                    second.base_uuid = armor_id;
                    second.persistent_copy_uuid = Some(Uuid::from_u128(0xe650 + index as u128));
                    spawn.draft.equipped_items.push(second);
                }
            },
            |_| {},
            None,
            |b| {
                b.with_equipment(vec![weapon, armor])
                    .with_equipment_defenses(artifact.equipment_defenses().to_vec())
            },
        );
        let target = core
            .units
            .values()
            .find(|u| u.owner == Side::Opponent)
            .unwrap()
            .instance_id;
        for u in core.units.values_mut() {
            u.stats.current_health = 1000;
            u.stats.max_health = 1000;
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
        }
        let mut f = Fixture {
            core,
            state,
            caster,
            target,
            ally,
        };
        assert_eq!(
            f.core
                .items
                .values()
                .filter(|i| i.owner_unit_instance == f.caster)
                .count(),
            2
        );
        start(&mut f, 10, 4000, false);
        let e = f.target;
        assert_eq!(
            damage(&mut f, e, DamageSource::Ability, DamageType::True, 10),
            50
        );
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .incoming_damage_modifiers
            .damage_reduction_percent = 20;
        assert_eq!(
            damage(&mut f, e, DamageSource::Ability, DamageType::True, 11),
            30
        );
    }
    #[test]
    fn composition_channel_opening_observer_off_and_cloned_continuation_match() {
        let mut observed = setup();
        start(&mut observed, 10, 4000, false);
        let mut plain = Fixture {
            core: observed.core.clone(),
            state: observed.state.clone(),
            caster: observed.caster,
            target: observed.target,
            ally: observed.ally,
        };
        plain.core.authoring_observation = None;
        let enemy = observed.target;
        for (now, expected) in [(11, 50), (2009, 50), (2010, 100), (4010, 100)] {
            assert_eq!(
                damage(
                    &mut observed,
                    enemy,
                    DamageSource::Ability,
                    DamageType::True,
                    now
                ),
                expected
            );
            assert_eq!(
                damage(
                    &mut plain,
                    enemy,
                    DamageSource::Ability,
                    DamageType::True,
                    now
                ),
                expected
            );
            assert_eq!(
                observed.core.transaction_debug_snapshot(),
                plain.core.transaction_debug_snapshot()
            );
            assert_eq!(
                observed.core.event_queue_debug_snapshot(),
                plain.core.event_queue_debug_snapshot()
            );
            assert_eq!(
                serde_json::to_value(observed.core.runtime_fact_summary()).unwrap(),
                serde_json::to_value(plain.core.runtime_fact_summary()).unwrap()
            );
        }
        assert!(plain.core.authoring_observation.is_none());
        assert_eq!(
            observed
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_channel_opening
                .as_ref()
                .unwrap()
                .time_ms,
            2009,
            "an expired window keeps only the last eligible historical observation"
        );
    }

    #[test]
    fn composition_channel_opening_native_retained_refusal_preserves_observation_and_retry() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        start(&mut f, 10, 4000, false);
        let a = f.caster;
        let e = f.target;
        let source = f
            .core
            .damage_source_snapshot_for_unit(
                e,
                a,
                DamageSource::Ability,
                DamageType::True,
                100,
                Default::default(),
                0,
                11,
                false,
            )
            .unwrap();
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|c| c.channel_opening_reduction(a, &source, 11))
        });
        assert_eq!(result.unwrap(), 50);
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "channel_opening_observation")
            .unwrap()
            .2;
        let original = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        let observation = serde_json::to_value(&f.core.authoring_observation).unwrap();
        assert!(f
            .core
            .runtime_transaction(|c| c.channel_opening_reduction(a, &source, 11))
            .is_err());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            observation
        );
        f.core.runtime_facts.limits.max_retained_bytes = original;
        assert_eq!(
            f.core
                .runtime_transaction(|c| c.channel_opening_reduction(a, &source, 11))
                .unwrap(),
            50
        );
    }
}
