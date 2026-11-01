mod composition_equipment_attack {
    include!("composition_release_gap_tests.rs");

    include!("composition_chance_stun_tests.rs");
    include!("composition_palm_slow_tests.rs");
    include!("composition_attack_continuity_tests.rs");
    include!("composition_multi_target_tests.rs");
    include!("composition_kill_cast_tests.rs");
    include!("composition_area_batch_tests.rs");
    include!("composition_nearby_ally_damage_tests.rs");
    include!("composition_conditional_damage_tests.rs");
    include!("composition_basic_slow_tests.rs");
    include!("composition_pleasure_tests.rs");
    include!("composition_ready_wait_tests.rs");
    include!("composition_nearby_defense_tests.rs");
    include!("composition_primary_hp_tests.rs");
    include!("composition_lifesteal_tests.rs");
    include!("composition_mountain_tests.rs");
    include!("composition_magazine_tests.rs");
    include!("composition_weapon_stages_tests.rs");
    include!("composition_shared_cohort_tests.rs");
    use super::*;
    use crate::{
        authoring::composition::{compile_json, parse_yaml, Artifact},
        game::{
            battle::{
                core::sim::{BattleExecutionState, BattleLiveCommand},
                types::BattleUnitThreatClass,
            },
            data::relic_data::RelicDatabase,
            resource::{ResourceCatalog, ResourceOwner, RunResourceEntry, RunResourceState},
        },
    };
    use serde_json::{json, Value};
    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/equipment_attack.skill.yaml"
    ));
    struct Fixture {
        core: BattleCore,
        state: BattleExecutionState,
        a: UnitInstanceId,
        b: UnitInstanceId,
        enemy: UnitInstanceId,
        secondary: UnitInstanceId,
        run: RunResourceState,
    }
    fn program(doc: &mut Value) -> &mut Value {
        &mut doc["values"][0]["value"]["AttackProgram"]
    }
    fn fixture(initial: u32, relic: bool, edit: impl FnOnce(&mut Value)) -> Fixture {
        fixture_copies(initial, relic, false, edit)
    }
    fn fixture_copies(
        initial: u32,
        relic: bool,
        swap: bool,
        edit: impl FnOnce(&mut Value),
    ) -> Fixture {
        fixture_copies_with_setup(initial, relic, swap, edit, |_| {}, |_| {}, None)
    }
    fn fixture_copies_with_setup(
        initial: u32,
        relic: bool,
        swap: bool,
        edit: impl FnOnce(&mut Value),
        configure_skill: impl FnOnce(&mut String),
        configure_scenario: impl FnOnce(&mut BattleScenario),
        fragments: Option<crate::game::data::skill_fragment_data::SkillFragmentDatabase>,
    ) -> Fixture {
        fixture_copies_with_setup_and_relics(
            initial, relic, swap, edit, configure_skill, configure_scenario, fragments, &[],
        )
    }
    fn fixture_copies_with_setup_and_relics(
        initial: u32,
        relic: bool,
        swap: bool,
        edit: impl FnOnce(&mut Value),
        configure_skill: impl FnOnce(&mut String),
        configure_scenario: impl FnOnce(&mut BattleScenario),
        fragments: Option<crate::game::data::skill_fragment_data::SkillFragmentDatabase>,
        extra_relics: &[&str],
    ) -> Fixture {
        fixture_copies_with_equipment(initial, relic, swap, edit, configure_skill,
            configure_scenario, fragments, extra_relics, |_| {})
    }
    fn fixture_copies_with_equipment(
        initial: u32,
        relic: bool,
        swap: bool,
        edit: impl FnOnce(&mut Value),
        configure_skill: impl FnOnce(&mut String),
        configure_scenario: impl FnOnce(&mut BattleScenario),
        fragments: Option<crate::game::data::skill_fragment_data::SkillFragmentDatabase>,
        extra_relics: &[&str],
        configure_equipment: impl FnOnce(&mut Vec<crate::game::data::equipment_data::EquipmentMetadata>),
    ) -> Fixture {
        let mut doc = parse_yaml(YAML).unwrap();
        edit(&mut doc);
        let artifact = compile_json(&doc.to_string()).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let skills = artifact.load_native().unwrap();
        let mut authored = if skills.skills.is_empty() {
            r12_control_ron("Heal(amount:1)")
        } else {
            ron::ser::to_string(&skills).unwrap()
        };
        configure_skill(&mut authored);
        let mut relics: RelicDatabase = ron::de::from_str(r#"(relics:[(id:"harvest",uuid:"00000000-0000-0000-0000-00000000d100",name:"Harvest fixture",description:"Authored condition",rarity:"fixture",allow_duplicates:true,effects:[(id:"unused",selector:(all_tags:["unused_fixture"]),percent:1)],dismantle_equipment_dust:1)])"#).unwrap();
        // Catalog dependencies are available for validation, without granting ownership.
        for (index, id) in extra_relics.iter().enumerate() {
            let mut definition = relics.relics[0].clone();
            definition.id = (*id).into();
            definition.uuid = Uuid::from_u128(0xd101 + index as u128);
            relics.relics.push(definition);
        }
        let (mut core, state, a, b) = r12_fragment_equipped_effect_core_with_equipment(
            &authored,
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(8000);
                scenario.groups[0].enemy_movement_plan = None;
                let mut secondary = scenario.groups[0].spawns[0].clone();
                secondary.unit_ref =
                    crate::game::battle::scenario::ScenarioUnitRef::new("equipment_secondary");
                secondary.instance_salt += 100;
                secondary.position = crate::game::resources::Position::new(6, 3);
                scenario.groups[0].spawns.push(secondary);
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    spawn.draft.equipped_items[0].persistent_copy_uuid = Some(Uuid::from_u128(
                        0xe100 + if swap { 1 - i as u128 } else { i as u128 },
                    ));
                }
                configure_scenario(scenario);
            },
            |_| {},
            fragments,
            configure_equipment,
            |builder| {
                builder
                    .with_resources(ResourceCatalog {
                        resources: artifact.resources().to_vec(),
                    })
                    .with_equipment_attacks(artifact.equipment_attacks().to_vec())
                    .with_equipment_defenses(artifact.equipment_defenses().to_vec())
                    .with_equipment_states(artifact.equipment_states().to_vec())
                    .with_equipment_supports(artifact.equipment_supports().to_vec())
                    .with_relics(relics)
            },
        );
        let mut enemies: Vec<_> = core
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .map(|u| (u.spawn_order, u.instance_id))
            .collect();
        enemies.sort();
        let enemy = enemies[0].1;
        let secondary = enemies[1].1;
        for unit in core.units.values_mut() {
            unit.stats.attack_interval_ms = 100_000;
            unit.next_basic_attack_ms = 100_000;
            unit.stats.move_speed_units_per_ms = 0;
            unit.stats.defense = 0;
            unit.stats.current_health = 1000;
            unit.stats.max_health = 1000;
            unit.threat_class = BattleUnitThreatClass::Normal;
        }
        core.units.get_mut(&a).unwrap().stats.current_health = 500;
        for (id, x, y) in [(a, 2, 1), (b, 7, 3), (enemy, 3, 1), (secondary, 6, 3)] {
            super::compiled_action_fixture::place(&mut core, id, x, y);
        }
        let owners = core
            .items
            .values()
            .filter_map(|i| i.persistent_copy_uuid)
            .map(ResourceOwner::EquipmentCopy)
            .collect();
        let run = RunResourceState {
            revision: 0,
            closed: false,
            entries: if initial == 0 {
                vec![]
            } else {
                vec![RunResourceEntry {
                    resource: artifact.resources()[0].id.clone(),
                    owner: owner(&core, a),
                    balance: initial,
                    revision: 0,
                }]
            },
        };
        core.mount_run_resources(&run, &owners).unwrap();
        if relic {
            let mut inventory = crate::game::relic::RelicInventory::default();
            let mut ids = crate::game::managers::uuid_manager::UuidManager::new(0xd300);
            let copy = inventory
                .grant(&core.game_data.relic_data.relics[0], &mut ids)
                .unwrap();
            inventory
                .equip(
                    copy,
                    core.units[&a].source_owned_uuid,
                    0,
                    &core.game_data.relic_data,
                )
                .unwrap();
            let owner = crate::game::battle::core::relic_numeric::RelicNumericOwner::capture(
                &inventory,
                core.game_data.clone(),
                core.content_identity.as_ref().unwrap().clone(),
            )
            .unwrap();
            core.install_relic_numeric_owner(owner).unwrap();
        }
        Fixture {
            core,
            state,
            a,
            b,
            enemy,
            secondary,
            run,
        }
    }
    fn owner(core: &BattleCore, actor: UnitInstanceId) -> ResourceOwner {
        ResourceOwner::EquipmentCopy(
            core.items
                .values()
                .find(|i| i.owner_unit_instance == actor)
                .unwrap()
                .persistent_copy_uuid
                .unwrap(),
        )
    }
    fn balance_owner(core: &BattleCore, owner: ResourceOwner) -> u32 {
        let view = core.typed_resources().unwrap();
        let ordinal = view
            .scopes()
            .find(|(_, l, _, _)| *l == crate::game::resource::ResourceLifetime::Run)
            .unwrap()
            .0;
        view.value(
            &crate::game::resource::ResourceId::new("weapon_mass".into()).unwrap(),
            owner,
            ordinal,
        )
        .unwrap()
        .balance
    }
    fn balance(f: &Fixture, actor: UnitInstanceId) -> u32 {
        balance_owner(&f.core, owner(&f.core, actor))
    }
    fn attack(f: &mut Fixture, now: u64, regular: bool) {
        f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = now;
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: now,
                    attacker_instance_id: f.a,
                    target_instance_id: Some(f.enemy),
                    schedule_next: regular,
                    cause: Default::default(),
                },
                now,
            )
            .unwrap();
    }
    const SPEED_YAML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/equipment_speed.skill.yaml"));
    fn speed_fixture(edit: impl FnOnce(&mut Value)) -> Fixture {
        fixture(0, false, |doc| {
            *doc = parse_yaml(SPEED_YAML).unwrap();
            edit(doc);
        })
    }
    #[test]
    fn composition_equipment_speed_live_hp_caps_healing_max_hp_and_isolation() {
        let mut f = speed_fixture(|_| {});
        for (hp, expected) in [(1000, 1000), (750, 667), (500, 500), (1, 500), (900, 834)] {
            f.core.units.get_mut(&f.a).unwrap().stats.current_health = hp;
            assert_eq!(f.core.sustained_basic_interval(f.a, 1000), expected);
            assert_eq!(f.core.sustained_basic_interval(f.b, 1000), 1000);
        }
        f.core.units.get_mut(&f.a).unwrap().stats.max_health = 2000;
        assert_eq!(f.core.sustained_basic_interval(f.a, 1000), 500);
        f.core.units.get_mut(&f.a).unwrap().corroded = true;
        assert_eq!(f.core.sustained_basic_interval(f.a, 1000), 1000);
        // Same engine, different authored HP dependency; no content ID branch.
        let other = speed_fixture(|doc| {
            let text = doc.to_string().replace("Missing", "Current");
            *doc = serde_json::from_str(&text).unwrap();
        });
        assert_eq!(other.core.sustained_basic_interval(other.b, 1000), 500);
    }
    #[test]
    fn composition_equipment_speed_regular_start_keeps_windup_and_next_reservation() {
        let mut f = speed_fixture(|_| {});
        f.core.units.get_mut(&f.a).unwrap().stats.attack_interval_ms = 1000;
        f.core.units.get_mut(&f.a).unwrap().basic_attack.windup_ms = 100;
        attack(&mut f, 10, true);
        assert_eq!(f.core.units[&f.a].next_basic_attack_ms, 510);
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 1000;
        f.core.step_battle_execution_until(&mut f.state, 109).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 1000);
        f.core.step_battle_execution_until(&mut f.state, 110).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 900);
        assert_eq!(f.core.units[&f.a].next_basic_attack_ms, 510);
        f.core.step_battle_execution_until(&mut f.state, 510).unwrap();
        assert_eq!(f.core.units[&f.a].next_basic_attack_ms, 1510);
        assert_eq!(f.core.sustained.entity_count(), 0);
    }
    #[test]
    fn composition_equipment_speed_diagnostics_and_accessory_publication() {
        use crate::game::ability::equipment_attack::EquipmentAttackCatalog;
        for (old, new) in [("moment: Live", "moment: Snapshot"),
            ("AttackSpeedPoints", "Milliseconds"), ("subject: Source", "subject: Target")] {
            let error = crate::authoring::composition::compile_yaml(&SPEED_YAML.replace(old, new)).unwrap_err();
            assert!(error.source.definition.contains("basic_speed_points"), "{error:?}");
        }
        let f = speed_fixture(|_| {});
        let mut equipment = (*f.core.game_data.equipment_data).clone();
        let item = equipment.items.iter_mut().find(|i| i.id == "r11_weapon").unwrap();
        item.equipment_type = crate::game::data::equipment_data::EquipmentType::Accessory;
        item.weapon_profile = None;
        EquipmentAttackCatalog::compile(&equipment, &ResourceCatalog { resources: vec![] },
            &RelicDatabase::default(), &crate::game::value_expr::ExprLimits::default()).unwrap();
    }
    #[test]
    fn composition_equipment_speed_fault_and_cancellation_restore_attack_admission() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = speed_fixture(|_| {});
        queue_attack(&mut f);
        let snapshot = |f: &Fixture| (f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(), format!("{:?}", f.state));
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_basic_speed_evaluated", 1, || {
            control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || {
                f.core.step_battle_execution_until(&mut f.state, 12)
            })
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, snapshot(&f));
        f.core.step_battle_execution_until(&mut f.state, 12).unwrap();
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        let mut invalid = speed_fixture(|doc| {
            program(doc)["basic_speed_points"]["root"] = json!({"Mul": {
                "value": {"Const":{"value":-1,"unit":"AttackSpeedPoints"}},
                "scalar":{"Ratio":{
                    "numerator":{"Read":{"HitPoints":{"subject":"Source","field":"Missing"}}},
                    "denominator":{"Const":{"value":1,"unit":"Magnitude"}},"rounding":"Floor"}}
            }});
        });
        queue_attack(&mut invalid);
        let before = snapshot(&invalid);
        let result = invalid.core.step_battle_execution_until(&mut invalid.state, 12);
        assert!(matches!(result, Err(GameError::ValueExpressionFault(_))), "{result:?}");
        assert_eq!(before, snapshot(&invalid));
    }
    #[test]
    fn composition_equipment_speed_observation_preserves_inputs_and_gameplay() {
        let mut f = speed_fixture(|_| {});
        f.core.enable_authoring_observation();
        f.core.units.get_mut(&f.a).unwrap().stats.attack_interval_ms = 1000;
        let mut plain = Fixture { core: f.core.clone(), state: f.state.clone(), a: f.a,
            b: f.b, enemy: f.enemy, secondary: f.secondary, run: f.run.clone() };
        plain.core.authoring_observation = None;
        attack(&mut f, 10, true);
        attack(&mut plain, 10, true);
        assert_eq!(f.core.transaction_debug_snapshot(), plain.core.transaction_debug_snapshot());
        assert_eq!(f.core.event_queue_debug_snapshot(), plain.core.event_queue_debug_snapshot());
        let before = f.core.authoring_observation.as_ref().unwrap().latest_equipment_speed.as_ref().unwrap().clone();
        assert_eq!((before.time_ms, before.current_hp, before.max_hp), (10, 500, 1000));
        assert_eq!((before.equipment_points, before.final_interval_ms), (100, 500));
        assert_eq!(before.contributors.len(), 1);
        assert_eq!(before.contributors[0].equipment.persistent_copy_uuid,
            Some(match owner(&f.core, f.a) { ResourceOwner::EquipmentCopy(id) => id, _ => unreachable!() }));
        assert!(before.contributors[0].field_path.ends_with("program.basic_speed_points"));
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 1000;
        assert_eq!(before.current_hp, 500, "the observation is an immutable admission snapshot");
    }
    #[test]
    fn composition_equipment_attack_real_copy_bonus_reward_and_save() {
        let mut f = fixture(3, false, |_| {});
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 870);
        assert_eq!(balance(&f, f.a), 3);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 100;
        attack(&mut f, 20, true);
        f.core
            .step_battle_execution_until(&mut f.state, 22)
            .unwrap();
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        assert_eq!(balance(&f, f.a), 5);
        assert_eq!(balance(&f, f.b), 0);
        assert_eq!(f.core.units[&f.a].stats.current_health, 507);
        assert_eq!(
            f.core.sustained.entity_count(),
            0,
            "attachment must not install a permanent state"
        );
        let mut saved = f.run.clone();
        f.core
            .capture_run_resources()
            .unwrap()
            .unwrap()
            .apply(&mut saved)
            .unwrap();
        let reopened: RunResourceState =
            ron::de::from_str(&ron::ser::to_string(&saved).unwrap()).unwrap();
        assert_eq!(saved, reopened);
        assert_eq!(f.run.entries[0].balance, 3, "entry savepoint is immutable");
        let mut next = fixture_copies(0, false, true, |_| {});
        next.core.runtime_facts.resources = None;
        let owners = next
            .core
            .items
            .values()
            .filter_map(|i| i.persistent_copy_uuid)
            .map(ResourceOwner::EquipmentCopy)
            .collect();
        next.core.mount_run_resources(&reopened, &owners).unwrap();
        assert_eq!(balance(&next, next.a), 0);
        assert_eq!(
            balance(&next, next.b),
            5,
            "same persistent copy transferred to the other employee"
        );
    }
    #[test]
    fn composition_equipment_attack_area_uses_admission_balance_and_primary_only() {
        use crate::game::{ability::DeliveryDef, battle::tile_range::TileRangePattern};
        for (primary_hp, secondary_hp, expected_mass, expected_secondary) in
            [(110, 1000, 5, 888), (1000, 110, 3, 0), (110, 110, 5, 0)]
        {
            let mut f = fixture(3, false, |_| {});
            super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 2);
            f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = primary_hp;
            f.core
                .units
                .get_mut(&f.secondary)
                .unwrap()
                .stats
                .current_health = secondary_hp;
            f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery = DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                    include_anchor_tile: true,
                },
                area: Default::default(),
            };
            attack(&mut f, 10, true);
            f.core
                .step_battle_execution_until(&mut f.state, 12)
                .unwrap();
            assert_eq!(balance(&f, f.a), expected_mass);
            assert_eq!(
                f.core.units[&f.secondary].stats.current_health,
                expected_secondary
            );
        }
    }
    #[test]
    fn composition_equipment_attack_relic_new_grants_and_data_only_combination() {
        for (relic, class, grant) in [
            (false, BattleUnitThreatClass::Normal, 2),
            (true, BattleUnitThreatClass::Normal, 4),
            (true, BattleUnitThreatClass::Elite, 10),
            (true, BattleUnitThreatClass::Boss, 18),
        ] {
            let mut f = fixture(3, relic, |doc| {
                let reactions = program(doc)["reactions"].as_array_mut().unwrap();
                let extra: Vec<_> = reactions
                    .clone()
                    .into_iter()
                    .map(|mut r| {
                        r["id"] = json!(format!("extra-{}", r["id"].as_str().unwrap()));
                        r["requires_relic"] = json!("harvest");
                        r["body"].as_array_mut().unwrap().truncate(1);
                        r
                    })
                    .collect();
                reactions.extend(extra);
            });
            f.core.units.get_mut(&f.enemy).unwrap().threat_class = class;
            attack(&mut f, 10, true);
            f.core
                .step_battle_execution_until(&mut f.state, 12)
                .unwrap();
            assert_eq!(
                f.core.units[&f.enemy].stats.current_health, 870,
                "relic cannot multiply prior balance or damage"
            );
            f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
            attack(&mut f, 20, true);
            f.core
                .step_battle_execution_until(&mut f.state, 22)
                .unwrap();
            assert_eq!(balance(&f, f.a), 3 + grant);
        }
        // A different body, still using the frozen operation compiler/runtime.
        let mut f = fixture(3, false, |doc| {
            program(doc)["reactions"][0]["body"] = json!([{"If": {
                "condition":{"Compare":{"left":{"moment":"Live","root":{"Read":{"HitPoints":{"subject":"Target","field":"Missing"}}}},"comparison":"AtLeast","right":{"moment":"Live","root":{"Const":{"value":100,"unit":"Magnitude"}}}}},
                "then_ops":[{"Resources":{"operations":[{"resource":"weapon_mass","owner":"SourceEquipment","operation":{"Decay":1}}]}},{"HealOwner":{"amount":{"moment":"Live","root":{"Const":{"value":19,"unit":"Magnitude"}}}}}],
                "else_ops":[]
            }}]);
        });
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap();
        assert_eq!(balance(&f, f.a), 2);
        assert_eq!(f.core.units[&f.a].stats.current_health, 519);
    }
    #[test]
    fn composition_equipment_attack_excludes_non_normal_and_unqualified_kills() {
        use crate::game::battle::types::BattleUnitSourceIdentity;
        for case in 0..3 {
            let mut f = fixture(3, false, |_| {});
            if case == 1 {
                let u = f.core.units.get_mut(&f.enemy).unwrap();
                u.source_identity = BattleUnitSourceIdentity::SummonedUnit {
                    definition_id: "temporary".into(),
                    base_uuid: u.base_uuid,
                    summoner_instance_id: f.a,
                    summon_sequence: 1,
                    batch_uuid: Uuid::from_u128(12),
                };
            } else if case == 2 {
                Arc::make_mut(&mut f.core.scenario_runtime.spawned)
                    .unit_refs
                    .retain(|_, id| *id != f.enemy);
            }
            f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 10;
            attack(&mut f, 10, case != 0);
            f.core
                .step_battle_execution_until(&mut f.state, 12)
                .unwrap();
            assert!(f.core.units[&f.enemy].lifecycle.is_dead());
            assert_eq!(balance(&f, f.a), 3);
            assert_eq!(f.core.units[&f.a].stats.current_health, 500);
        }
    }
    #[test]
    fn composition_equipment_attack_release_survives_withdrawal() {
        use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
        let mut f = fixture(3, true, |doc| {
            program(doc)["reactions"][0]["requires_relic"] = json!("harvest");
        });
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 110;
        f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery = DeliveryDef::Projectile {
            speed_units_per_ms: 1000,
            hit_policy: ProjectileHitPolicy::TargetLocked,
            allow_targetless_cast: false,
            max_range_tiles: None,
            max_lifetime_ms: None,
            max_kills: None,
            max_pierces: None,
            collision: Default::default(),
        };
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap();
        assert!(
            f.core
                .projectiles
                .values()
                .any(|p| p.source_snapshot.source_id == f.a),
            "no actor projectile: hp={} events={:?}",
            f.core.units[&f.enemy].stats.current_health,
            f.core.event_log.entries
        );
        for projectile in f
            .core
            .projectiles
            .values()
            .filter(|p| p.source_snapshot.source_id == f.a)
        {
            let capture = projectile
                .source_snapshot
                .equipment_attack
                .as_ref()
                .expect("released attack must carry authored equipment program");
            assert_eq!(capture.bonus, 30);
            assert!(capture.relic_copies[0].is_some());
            assert_eq!(capture.primary, f.enemy);
            assert_eq!(projectile.fired_at_ms + projectile.max_travel_ms, 1011);
        }
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 110);
        let copy = owner(&f.core, f.a);
        // Admission owns the prerequisite: later live equipment lookup must
        // not erase the condition from a released carrier.
        f.core.relic_numeric_owner = None;
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 12,
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 1200)
            .unwrap();
        assert!(
            f.core.units[&f.enemy].lifecycle.is_dead(),
            "remaining={} projectiles={} events={:?}",
            f.core.units[&f.enemy].stats.current_health,
            f.core.projectiles.len(),
            f.core.event_log.entries
        );
        assert_eq!(balance_owner(&f.core, copy), 5);
        assert_eq!(
            f.core.units[&f.a].stats.current_health, 500,
            "withdrawn employee is not healed"
        );
    }
    #[test]
    fn composition_equipment_attack_cancellation_restores_and_retries_once() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "equipment_attack_damage",
            "state_reaction_applied",
            "equipment_attack_reacted",
        ] {
            let mut f = fixture(3, false, |_| {});
            f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
            attack(&mut f, 10, true);
            f.core
                .step_battle_execution_until(&mut f.state, 10)
                .unwrap();
            let snapshot = |f: &Fixture| {
                (
                    f.core.transaction_debug_snapshot(),
                    f.core.event_queue_debug_snapshot(),
                    serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
                    format!("{:?}", f.state),
                    balance(f, f.a),
                )
            };
            let before = snapshot(&f);
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || f.core.step_battle_execution_until(&mut f.state, 12),
                )
            });
            assert!(result.is_err() && token.is_cancelled(), "{site}");
            assert_eq!(before, snapshot(&f), "{site}");
            f.core
                .step_battle_execution_until(&mut f.state, 12)
                .unwrap();
            assert_eq!(balance(&f, f.a), 5);
            assert_eq!(f.core.units[&f.a].stats.current_health, 507);
        }
    }
    #[test]
    fn composition_equipment_attack_authored_replacement_and_observation_match() {
        let mut results = Vec::new();
        for observe in [false, true] {
            let mut f = fixture_copies_with_setup_and_relics(3, false, false, |doc| {
                let mut state = parse_yaml(include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/fixtures/skill_composition/basic_kill.skill.yaml"
                )))
                .unwrap();
                state["skills"][0]["id"] = json!("r11_aura");
                state["values"][0]["value"]["Effects"][0]["op"]["Sustained"]["definition"]
                    ["State"]["program"]["handlers"][0]["body"] = json!([]);
                doc["values"]
                    .as_array_mut()
                    .unwrap()
                    .push(state["values"][0].clone());
                doc["definitions"] = state["definitions"].clone();
                doc["skills"] = state["skills"].clone();
            }, |_| {}, |_| {}, None, &["piercing-damage-increase"]);
            if observe {
                f.core
                    .enable_authoring_observation_with_options(Default::default(), true);
                f.core
                    .bind_authoring_observation(
                        crate::game::battle::core::authoring_observation::ObservationKey {
                            entry_ordinal: 1,
                            battle_uuid: Uuid::from_u128(0xb16b),
                        },
                    )
                    .unwrap();
            }
            super::compiled_action_fixture::cast(&mut f.core, f.a, 0);
            f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
            f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 60;
            attack(&mut f, 10, true);
            f.core
                .step_battle_execution_until(&mut f.state, 12)
                .unwrap();
            assert!(
                f.core.units[&f.enemy].lifecycle.is_dead(),
                "40 replacement + 30 stored mass"
            );
            assert_eq!(balance(&f, f.a), 5);
            assert_eq!(f.core.units[&f.a].stats.current_health, 507);
            if observe {
                let witness = f
                    .core
                    .authoring_observation()
                    .unwrap()
                    .unwrap()
                    .latest_nonzero_sourced_hp
                    .as_ref()
                    .unwrap();
                assert_eq!(witness.change.hp_delta(), 7);
                assert_eq!(
                    witness.equipment.unwrap().persistent_copy_uuid,
                    Some(match owner(&f.core, f.a) {
                        ResourceOwner::EquipmentCopy(id) => id,
                        _ => unreachable!(),
                    })
                );
            }
            results.push((
                serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
                balance(&f, f.a),
                f.core.sustained_effect_snapshot().unwrap(),
            ));
        }
        assert_eq!(results[0], results[1]);
    }
    #[test]
    fn composition_equipment_attack_amount_saturates_and_forbidden_authority_is_diagnostic() {
        let mut f = fixture(u32::MAX - 1, false, |_| {});
        use crate::game::{ability::DeliveryDef, battle::tile_range::TileRangePattern};
        super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 2);
        f.core
            .units
            .get_mut(&f.secondary)
            .unwrap()
            .stats
            .current_health = 2_000_000_000;
        f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery = DeliveryDef::TileArea {
            footprint: TileRangePattern {
                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
                include_anchor_tile: true,
            },
            area: Default::default(),
        };
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap();
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        assert_eq!(balance(&f, f.a), u32::MAX);
        assert!(
            f.core.units[&f.secondary].lifecycle.is_dead(),
            "scale wide bonus before the final damage clamp"
        );
        for (old, new, part) in [
            ("SourceEquipment", "SourceEmployee", "owner"),
            ("lifetime: Run", "lifetime: Deployment", "resource"),
            ("moment: Snapshot", "moment: Live", "bonus"),
            ("secondary_percent: 40", "secondary_percent: 101", "bonus"),
        ] {
            let error =
                crate::authoring::composition::compile_yaml(&YAML.replace(old, new)).unwrap_err();
            assert!(error.source.definition.contains(part), "{error:?}");
        }
        let mut doc = parse_yaml(YAML).unwrap();
        program(&mut doc)["reactions"][0]["body"] = json!(["End"]);
        let error = compile_json(&doc.to_string()).unwrap_err();
        assert!(error.reason.contains("no local fields"), "{error:?}");
        doc = parse_yaml(YAML).unwrap();
        let binding = doc["equipment_attacks"][0].clone();
        doc["equipment_attacks"]
            .as_array_mut()
            .unwrap()
            .push(binding);
        assert!(compile_json(&doc.to_string())
            .unwrap_err()
            .reason
            .contains("duplicate equipment"));
    }
    #[test]
    fn composition_equipment_attack_publication_references_and_conflicts() {
        use crate::game::ability::equipment_attack::EquipmentAttackCatalog;
        let f = fixture(3, false, |_| {});
        for case in 0..6 {
            let mut equipment = (*f.core.game_data.equipment_data).clone();
            let weapon = equipment
                .items
                .iter_mut()
                .find(|i| i.id == "r11_weapon")
                .unwrap();
            match case {
                0=>equipment.equipment_attacks[0].equipment="absent_weapon".into(),
                1=>weapon.equipment_type=crate::game::data::equipment_data::EquipmentType::Armor,
                2=>weapon.weapon_profile=None,
                3=>equipment.equipment_attacks[0].program.reactions[0].requires_relic=Some("absent_relic".into()),
                4=>equipment.equipment_attacks.push(equipment.equipment_attacks[0].clone()),
                5=>weapon.weapon_profile.as_mut().unwrap().kill_growth=Some(ron::de::from_str(r#"(resource:"weapon_mass",normal:2,elite:5,abnormality:9,damage_per_stack:10,secondary_percent:40)"#).unwrap()),
                _=>unreachable!(),
            }
            let error = EquipmentAttackCatalog::compile(
                &equipment,
                &f.core.game_data.resource_data,
                &f.core.game_data.relic_data,
                &Default::default(),
            )
            .unwrap_err();
            let diagnostic = format!("{error:?}");
            assert!(
                diagnostic.contains(if case == 3 {
                    "requires_relic"
                } else {
                    "equipment"
                }),
                "{case}: {diagnostic}"
            );
        }
    }
    fn queue_attack(f: &mut Fixture) {
        f.core.units.get_mut(&f.a).unwrap().next_basic_attack_ms = 10;
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
        f.core.event_queue.push(BattleEvent::AttackStart {
            time_ms: 10,
            attacker_instance_id: f.a,
            target_instance_id: Some(f.enemy),
            schedule_next: true,
            cause: Default::default(),
        });
        f.core.step_battle_execution_until(&mut f.state, 9).unwrap();
    }
    fn snapshot(f: &Fixture) -> impl PartialEq + std::fmt::Debug {
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
            format!("{:?}", f.state),
            balance(f, f.a),
        )
    }
    #[test]
    fn composition_equipment_attack_admission_cancellation_and_retained_refusal_retry() {
        use crate::{
            authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase},
            game::battle::runtime_control as native,
        };
        for site in [
            "equipment_attack_capture",
            "equipment_attack_frame",
            "equipment_attack_bound",
        ] {
            let mut f = fixture(3, false, |_| {});
            queue_attack(&mut f);
            let before = snapshot(&f);
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || f.core.step_battle_execution_until(&mut f.state, 12),
                )
            });
            assert!(
                result.is_err() && token.is_cancelled(),
                "{site}: {result:?}"
            );
            assert_eq!(before, snapshot(&f), "{site}");
            f.core
                .step_battle_execution_until(&mut f.state, 12)
                .unwrap();
            assert_eq!(balance(&f, f.a), 5);
        }
        let mut f = fixture(3, false, |_| {});
        queue_attack(&mut f);
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 12)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "equipment_attack_capture")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = snapshot(&f);
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "equipment_attack_capture",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(before, snapshot(&f));
        f.core.runtime_facts.limits = Default::default();
        f.core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap();
        assert_eq!(balance(&f, f.a), 5);
    }

    #[test]
    fn composition_equipment_attack_missing_persistent_copy_refuses_without_fallback() {
        let mut f = fixture(3, false, |_| {});
        queue_attack(&mut f);
        let item = f
            .core
            .items
            .values_mut()
            .find(|i| i.owner_unit_instance == f.a)
            .unwrap();
        let copy = item.persistent_copy_uuid.take().unwrap();
        let before = (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
            format!("{:?}", f.state),
        );
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(
                    crate::game::battle::runtime_control::BattleRuntimeFault {
                        code: crate::game::battle::runtime_control::RuntimeFaultCode::Resource(_),
                        ..
                    }
                )
            ),
            "{error:?}"
        );
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
                format!("{:?}", f.state)
            )
        );
        f.core
            .items
            .values_mut()
            .find(|i| i.owner_unit_instance == f.a)
            .unwrap()
            .persistent_copy_uuid = Some(copy);
        f.core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap();
        assert_eq!(balance(&f, f.a), 5);
    }
    include!("composition_source_direct_tests.rs");
    include!("composition_threshold_control_tests.rs");
    include!("composition_equipment_state_tests.rs");
    include!("composition_adaptive_shield_tests.rs");
}
