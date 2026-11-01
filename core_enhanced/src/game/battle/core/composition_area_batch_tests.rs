mod composition_area_batch {
    use super::*;
    use crate::game::{
        ability::DeliveryDef,
        battle::{damage::DamageType, tile_range::TileRangePolicy},
        data::abnormality_data::BasicAttackAction,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/area_batch.skill.yaml"
    ));

    fn setup(edit: impl FnOnce(&mut Value)) -> Fixture {
        let mut f = fixture(0, false, |doc| {
            *doc = parse_yaml(SOURCE).unwrap();
            edit(doc);
        });
        for u in f.core.units.values_mut() {
            u.stats.magic_resist = 0;
            u.stats.defense = 0;
            u.stats.attack_interval_ms = 100_000;
            u.next_basic_attack_ms = 100_000;
        }
        super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 1);
        let u = f.core.units.get_mut(&f.a).unwrap();
        u.stats.attack = 100;
        u.basic_attack.delivery = DeliveryDef::Instant;
        u.basic_attack.windup_ms = 80;
        u.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
        u.basic_attack.damage_type = DamageType::Physical;
        u.basic_attack.air_capable = true;
        f.core.enable_authoring_observation();
        f
    }
    fn witness(core: &BattleCore) -> &crate::game::battle::core::area_damage::AreaDamageWitness {
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_area_damage
            .as_ref()
            .unwrap()
    }
    fn state(core: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
        )
    }
    fn run(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }

    #[test]
    fn composition_area_batch_schema_shared_roundtrip_and_precise_context_errors() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(artifact.equipment_attacks()[0]
            .program
            .area_damage_percent
            .is_some());
        for bad in [
            SOURCE.replace("moment: Live", "moment: Snapshot"),
            SOURCE.replace("grouping: WholeSet", "grouping: ByProjectedTile"),
            SOURCE.replace("value: 50, unit: Scalar", "value: 50, unit: Milliseconds"),
        ] {
            let error = crate::authoring::composition::compile_yaml(&bad).unwrap_err();
            assert!(
                error.source.definition.contains("area_damage_percent"),
                "{error:?}"
            );
        }
        let f = setup(|_| {});
        let mut equipment = (*f.core.game_data.equipment_data).clone();
        let item = equipment
            .items
            .iter_mut()
            .find(|i| i.id == "r11_weapon")
            .unwrap();
        item.equipment_type = crate::game::data::equipment_data::EquipmentType::Accessory;
        item.weapon_profile = None;
        crate::game::ability::equipment_attack::EquipmentAttackCatalog::compile(
            &equipment,
            &ResourceCatalog { resources: vec![] },
            &RelicDatabase::default(),
            &crate::game::value_expr::ExprLimits::default(),
        )
        .unwrap();
    }
    #[test]
    fn composition_area_batch_zero_single_unique_enemy_count_and_authored_cap() {
        for (coefficient, members, expected) in [(10, 0, 0), (10, 1, 0), (10, 2, 10), (100, 2, 50)]
        {
            let mut f = setup(|doc| {
                program(doc)["area_damage_percent"]["root"]["Clamp"]["value"]["Ratio"]
                    ["numerator"]["Mul"]["scalar"]["Const"]["value"] = json!(coefficient);
            });
            let targets = match members {
                0 => vec![f.a, f.b],
                1 => vec![f.enemy, f.enemy, f.a],
                _ => vec![f.enemy, f.secondary, f.enemy, f.secondary, f.a, f.b],
            };
            let percent = f
                .core
                .runtime_transaction(|core| {
                    core.area_damage_percent(f.a, Side::Player, &Default::default(), &targets, 10)
                })
                .unwrap();
            assert_eq!(percent, expected);
            assert_eq!(witness(&f.core).target_count, members);
            assert_eq!(witness(&f.core).contributors.len(), 1);
        }
    }
    #[test]
    fn composition_area_batch_all_native_area_shapes_share_one_bonus() {
        let area: BasicAttackAction = ron::de::from_str(
            "DamageArea(footprint:(include_anchor_tile:true,rows:[\"X@X\"]),secondary_percent:100)",
        )
        .unwrap();
        for action in [
            area,
            BasicAttackAction::DamageCone {
                length_tiles: 5,
                end_half_width_tiles: 2,
            },
            BasicAttackAction::DamageOccupiedTiles,
            BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 },
        ] {
            let delay = if matches!(action, BasicAttackAction::DamageTileAfterDelay { .. }) {
                400
            } else {
                0
            };
            let mut f = setup(|_| {});
            f.core.units.get_mut(&f.a).unwrap().basic_attack.action = action;
            attack(&mut f, 10, true);
            run(&mut f, 89 + delay);
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 1000);
            run(&mut f, 90 + delay);
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 890);
            assert_eq!(f.core.units[&f.secondary].stats.current_health, 890);
            assert_eq!(
                (witness(&f.core).target_count, witness(&f.core).percent),
                (2, 10)
            );
            assert!(witness(&f.core).action.is_some());
        }
    }
    #[test]
    fn composition_area_batch_delayed_impact_recounts_current_air_eligible_occupants() {
        for allowed in [true, false] {
            let mut f = setup(|_| {});
            let u = f.core.units.get_mut(&f.a).unwrap();
            u.basic_attack.action = BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 };
            u.basic_attack.air_capable = allowed;
            super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 6, 3);
            attack(&mut f, 10, true);
            run(&mut f, 90);
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_area_damage
                .is_none());
            super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 1);
            f.core.units.get_mut(&f.secondary).unwrap().mobility_kind =
                crate::game::battle::types::MobilityKind::Airborne;
            run(&mut f, 490);
            assert_eq!(
                f.core.units[&f.enemy].stats.current_health,
                if allowed { 890 } else { 900 }
            );
            assert_eq!(
                f.core.units[&f.secondary].stats.current_health,
                if allowed { 890 } else { 1000 }
            );
            assert_eq!(witness(&f.core).target_count, if allowed { 2 } else { 1 });
        }
    }
    #[test]
    fn composition_area_batch_cancel_restores_entire_damage_batch_and_retries_once() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(|_| {});
        f.core.units.get_mut(&f.a).unwrap().basic_attack.action =
            BasicAttackAction::DamageOccupiedTiles;
        attack(&mut f, 10, true);
        run(&mut f, 89);
        let before = (state(&f.core), format!("{:?}", f.state));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("area_damage_evaluated", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 90),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, (state(&f.core), format!("{:?}", f.state)));
        run(&mut f, 90);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 890);
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 890);
    }
    #[test]
    fn composition_area_batch_clone_and_observation_do_not_change_damage_or_queue() {
        let mut f = setup(|_| {});
        f.core.units.get_mut(&f.a).unwrap().basic_attack.action =
            BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 };
        attack(&mut f, 10, true);
        run(&mut f, 90);
        let mut unobserved = f.core.clone();
        let mut execution = f.state.clone();
        unobserved.authoring_observation = None;
        run(&mut f, 490);
        unobserved
            .step_battle_execution_until(&mut execution, 490)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 890);
        assert_eq!(unobserved.units[&f.enemy].stats.current_health, 890);
        assert_eq!(
            serde_json::to_value(&f.core.event_log).unwrap(),
            serde_json::to_value(&unobserved.event_log).unwrap()
        );
        assert_eq!(
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
            serde_json::to_value(unobserved.runtime_fact_view()).unwrap()
        );
        assert_eq!(
            f.core.event_queue_debug_snapshot(),
            unobserved.event_queue_debug_snapshot()
        );
    }
    #[test]
    fn composition_area_batch_lethal_first_member_does_not_recompute_the_same_batch() {
        let mut f = setup(|_| {});
        f.core.units.get_mut(&f.a).unwrap().basic_attack.action =
            BasicAttackAction::DamageOccupiedTiles;
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
        attack(&mut f, 10, true);
        run(&mut f, 90);
        assert!(f.core.units[&f.enemy].is_dead());
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 890);
        assert_eq!(
            (witness(&f.core).target_count, witness(&f.core).percent),
            (2, 10)
        );
    }
    #[test]
    fn composition_area_batch_same_time_wearers_and_single_hits_never_merge() {
        let mut f = setup(|_| {});
        for id in [f.a, f.b] {
            let u = f.core.units.get_mut(&id).unwrap();
            u.stats.attack = 100;
            u.basic_attack.action = BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 };
            u.basic_attack.delivery = DeliveryDef::Instant;
            u.basic_attack.windup_ms = 80;
            u.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
            u.basic_attack.damage_type = DamageType::Physical;
        }
        attack(&mut f, 10, true);
        f.core.units.get_mut(&f.b).unwrap().next_basic_attack_ms = 10;
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: 10,
                    attacker_instance_id: f.b,
                    target_instance_id: Some(f.enemy),
                    schedule_next: true,
                    cause: Default::default(),
                },
                10,
            )
            .unwrap();
        run(&mut f, 490);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 780);
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 780);
        assert_eq!(witness(&f.core).contributors.len(), 1);
        assert_eq!(witness(&f.core).percent, 10);
        let mut single = setup(|_| {});
        single
            .core
            .units
            .get_mut(&single.a)
            .unwrap()
            .basic_attack
            .action = BasicAttackAction::Damage;
        attack(&mut single, 10, true);
        run(&mut single, 90);
        assert_eq!(single.core.units[&single.enemy].stats.current_health, 900);
        assert!(single
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_area_damage
            .is_none());
    }
    fn graph_fixture(
        doc: Value,
        relics: &[&str],
    ) -> (
        BattleCore,
        BattleExecutionState,
        UnitInstanceId,
        UnitInstanceId,
        UnitInstanceId,
    ) {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let graph =
            crate::authoring::node_graph::compile(&doc.to_string(), "r11_aura".into()).unwrap();
        let native =
            crate::authoring::node_graph::PortableGraph::from_ron(&graph.to_ron().unwrap())
                .unwrap()
                .load_content()
                .unwrap();
        let (mut c, s, a, _, b) = super::super::compiled_action_fixture::fixture_content_with_data(
            relics,
            &[],
            native,
            |_| {},
            crate::game::resource::ResourceCatalog { resources: vec![] },
            |builder| builder.with_equipment_attacks(artifact.equipment_attacks().to_vec()),
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(120000);
                scenario.groups[0].enemy_movement_plan = None;
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    spawn.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0xe740 + i as u128));
                }
                let mut secondary = scenario.groups[0].spawns[0].clone();
                secondary.unit_ref =
                    crate::game::battle::scenario::ScenarioUnitRef::new("batch_second");
                secondary.instance_salt += 100;
                secondary.position = crate::game::resources::Position::new(5, 2);
                scenario.groups[0].spawns.push(secondary);
            },
        );
        c.units.get_mut(&b).unwrap().fragment_source = None;
        c.units.get_mut(&b).unwrap().skill_id = None;
        let mut enemies: Vec<_> = c
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .map(|u| (u.spawn_order, u.instance_id))
            .collect();
        enemies.sort();
        let e = enemies[0].1;
        let second = enemies[1].1;
        for id in [a, e, b, second] {
            let u = c.units.get_mut(&id).unwrap();
            u.stats.attack = 100;
            u.stats.current_health = 1000;
            u.stats.max_health = 1000;
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
            u.stats.move_speed_units_per_ms = 0;
            u.next_basic_attack_ms = 100000;
            u.stats.attack_interval_ms = 100000;
        }
        super::super::compiled_action_fixture::place(&mut c, a, 2, 1);
        super::super::compiled_action_fixture::place(&mut c, e, 3, 1);
        super::super::compiled_action_fixture::place(&mut c, second, 3, 1);
        super::super::compiled_action_fixture::place(&mut c, b, 7, 3);
        c.enable_authoring_observation();
        (c, s, a, e, second)
    }
    #[test]
    fn composition_area_batch_prepared_volley_each_shot_recounts_and_does_not_multiply_stock() {
        let (mut c, mut s, a, e, second) = graph_fixture(
            crate::authoring::node_graph::tests::prepared_volley_fixture(),
            &[
                "long-breath", "widened-horizon", "double-harvest", "unbroken-trance",
                "deep-wedge", "king-of-greed-stunning-treasures", "king-of-greed-endless-treasury",
            ],
        );
        let copy = c.units[&a].fragment_source.as_ref().unwrap().copy_uuid;
        super::super::compiled_action_fixture::cast(&mut c, a, 0);
        c.step_battle_execution_until(&mut s, 1750).unwrap();
        let activation = c.prepared_volley_reinput(a, &"r11_aura".into()).unwrap();
        c.apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            activation,
            target: None,
            time_ms: 1750,
        })
        .unwrap();
        let mut first_time = None;
        for now in 1751..=3000 {
            c.step_battle_execution_until(&mut s, now).unwrap();
            if c.prepared_volleys_snapshot().unwrap().settled.len() == 1 {
                first_time = Some(now);
                break;
            }
        }
        assert!(first_time.is_some());
        assert_eq!(c.units[&e].stats.current_health, 890);
        assert_eq!(c.units[&second].stats.current_health, 890);
        assert_eq!(witness(&c).target_count, 2);
        super::super::compiled_action_fixture::place(&mut c, second, 7, 3);
        c.step_battle_execution_until(&mut s, 4000).unwrap();
        assert_eq!(c.prepared_volleys_snapshot().unwrap().settled.len(), 2);
        assert_eq!(c.units[&e].stats.current_health, 790);
        assert_eq!(c.units[&second].stats.current_health, 890);
        assert_eq!(witness(&c).target_count, 1);
        let view = c.prepared_volleys_snapshot().unwrap();
        assert_eq!(
            view.owners.iter().find(|o| o.copy == copy).unwrap().balance,
            8
        );
    }

    #[test]
    fn composition_area_batch_shield_and_deflection_stay_in_predefense_count() {
        use crate::game::ability::sustained::SustainedEffectDef;
        for shield in [true, false] {
            let mut f = setup(|_| {});
            f.core.units.get_mut(&f.a).unwrap().basic_attack.action =
                BasicAttackAction::DamageOccupiedTiles;
            let effect = if shield {
                SustainedEffectDef::Shield {
                    duration_ms: 1000,
                    amount: 1000,
                    priority: 0,
                    shared_radius_units: None,
                }
            } else {
                SustainedEffectDef::Parry {
                    duration_ms: 1000,
                    window_ms: 300,
                    invulnerable_ms: 400,
                    miss_recovery_ms: 200,
                    reduction_percent: 0,
                    counter_skill: "r11_aura".into(),
                    end_on_resolution: false,
                    fragment_guard: None,
                    guard_charge: None,
                }
            };
            f.core
                .runtime_transaction(|c| c.begin_sustained(f.enemy, f.enemy, effect, 0))
                .unwrap();
            attack(&mut f, 10, true);
            run(&mut f, 90);
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 1000);
            assert_eq!(f.core.units[&f.secondary].stats.current_health, 890);
            assert_eq!(
                (witness(&f.core).target_count, witness(&f.core).percent),
                (2, 10)
            );
        }
    }
    #[test]
    fn composition_area_batch_released_tile_survives_withdraw_without_inactive_equipment_bonus() {
        let mut f = setup(|_| {});
        f.core.units.get_mut(&f.a).unwrap().basic_attack.action =
            BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 };
        attack(&mut f, 10, true);
        run(&mut f, 90);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 100,
            })
            .unwrap();
        run(&mut f, 490);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 900);
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 900);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_area_damage
            .is_none());
    }

    #[test]
    fn composition_area_batch_general_tile_skill_recounts_each_periodic_judgment() {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| *doc = parse_yaml(SOURCE).unwrap(),
            |skill| {
                *skill=skill.replace("Heal(amount:1)","Damage(amount:100,damage_type:Physical)")
                .replace("delivery: Instant",r#"delivery: TileArea(footprint:(include_anchor_tile:true,rows:["XXX","X@X","XXX"]),
                    area:(anchor:CastTarget,tile_origin:Anchor,duration_ms:101,tick_interval_ms:Some(100)))"#)
            },
            |_| {},
            None,
        );
        super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 1);
        f.core.enable_authoring_observation();
        super::super::compiled_action_fixture::cast(&mut f.core, f.a, 10);
        run(&mut f, 11);
        assert_eq!(
            f.core.units[&f.enemy].stats.current_health,
            890,
            "events {:?}; queue {:?}",
            f.core.event_log,
            f.core.event_queue_debug_snapshot()
        );
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 890);
        assert_eq!(witness(&f.core).target_count, 2);
        super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 6, 3);
        run(&mut f, 111);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 790);
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 890);
        assert_eq!(witness(&f.core).target_count, 1);
    }

    #[test]
    fn composition_area_batch_same_time_explosions_stay_separate_unless_authored_as_one_wave() {
        for (dedup, expected) in [("blast", 10), ("wave", 11)] {
            let mut doc = crate::authoring::node_graph::tests::tile_burst_fixture("");
            for node in doc["nodes"].as_array_mut().unwrap() {
                if node["kind"] == "skill-explosion-rule" {
                    node["values"]["dedup"] = json!(dedup);
                }
            }
            let (mut c, mut s, a, e, second) = graph_fixture(
                doc,
                &[
                    "widened-horizon",
                    "overlapping-explosions",
                    "explosion-vulnerability",
                    "deep-imprint",
                    "funeral-butterfly-orb",
                ],
            );
            super::super::compiled_action_fixture::place(&mut c, second, 4, 1);
            super::super::compiled_action_fixture::cast(&mut c, a, 10);
            c.step_battle_execution_until(&mut s, 110).unwrap();
            assert_eq!(c.units[&e].stats.current_health, 1000 - expected, "{dedup}");
            assert_eq!(
                c.units[&second].stats.current_health,
                1000 - expected,
                "{dedup}"
            );
        }
    }
}
