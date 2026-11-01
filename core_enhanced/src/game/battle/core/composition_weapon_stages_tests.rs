mod composition_weapon_stages {
    use super::*;
    use crate::game::{
        ability::weapon_marks::WeaponStage,
        battle::tile_range::FacingDirection,
        data::equipment_data::{WeaponArchetype, WeaponRangeRole},
        resources::item_slot::{EquipmentSlotAddress, SlotLayoutKind},
    };
    fn canonical_source() -> String {
        crate::authoring::composition::canonical_funeral_tests::equipment_fixture("weapon")
    }
    const WEAPON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/butterfly.weapon.ron"
    ));
    struct Gun {
        core: BattleCore,
        state: BattleExecutionState,
        actor: UnitInstanceId,
        enemies: [UnitInstanceId; 3],
    }
    fn setup(stage: WeaponStage, deep_imprint: bool) -> Gun {
        let artifact = crate::authoring::composition::compile_yaml(&canonical_source()).unwrap();
        let mut native =
            crate::authoring::composition::canonical_funeral_tests::battle_fixture_content("");
        native.mark_weapons[0].definition.filter.air = true;
        let relics = [
            "widened-horizon",
            "overlapping-explosions",
            "explosion-vulnerability",
            "deep-imprint",
            "funeral-butterfly-orb",
        ];
        let (mut core, state, actor, _, ally) =
            super::super::compiled_action_fixture::fixture_content_with_data(
                &relics,
                if deep_imprint { &[3] } else { &[] },
                native,
                |weapon| weapon.weapon_profile = Some(ron::de::from_str(WEAPON).unwrap()),
                ResourceCatalog { resources: vec![] },
                |builder| builder.with_equipment_attacks(artifact.equipment_attacks().to_vec()),
                |scenario| {
                    scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(120000);
                    scenario.groups[0].enemy_movement_plan = None;
                    scenario.battlefield.walkable_paths = (0..8)
                        .flat_map(|x| {
                            (0..4).map(move |y| crate::game::resources::Position::new(x, y))
                        })
                        .collect();
                    scenario.groups[0].spawns[0].unit_ref = ScenarioUnitRef::new("stage-enemy-0");
                    scenario.groups[0].spawns[0].position =
                        crate::game::resources::Position::new(4, 1);
                    for index in 1..3 {
                        let mut enemy = scenario.groups[0].spawns[0].clone();
                        enemy.unit_ref = ScenarioUnitRef::new(format!("stage-enemy-{index}"));
                        enemy.instance_salt += 100 * index;
                        enemy.position = crate::game::resources::Position::new(4 + index as i32, 1);
                        scenario.groups[0].spawns.push(enemy);
                    }
                    for spawn in &mut scenario.groups[1].spawns {
                        spawn.draft.equipment_layout = Some(SlotLayoutKind::ByType);
                        let binding = &mut spawn.draft.equipped_items[0];
                        binding.weapon_stage = stage;
                        binding.slot = Some(EquipmentSlotAddress::Weapon);
                        binding.persistent_copy_uuid =
                            Some(Uuid::from_u128(0xe5500 + spawn.instance_salt as u128));
                        if !deep_imprint {
                            let BattleUnitSource::Employee(profile) = &mut spawn.draft.source
                            else {
                                panic!()
                            };
                            profile.fragment_source = None;
                            profile.skill_id = None;
                        }
                    }
                },
            );
        let enemies = std::array::from_fn(|i| {
            core.scenario_runtime.spawned.unit_refs
                [&ScenarioUnitRef::new(format!("stage-enemy-{i}"))]
        });
        // The shared fixture starts at 0; discard its uncommitted automatic
        // windups before measuring the explicitly scheduled attack at 1.
        for unit in core.units.values_mut() {
            unit.basic_attack_epoch += 1;
            unit.basic_attack_windup_until_ms = None;
            unit.pending_basic_attack = false;
            unit.action_locks = Default::default();
        }
        assert!(core.projectiles.is_empty());
        for (i, enemy) in enemies.iter().enumerate() {
            super::super::compiled_action_fixture::place(&mut core, *enemy, 4 + i as i32, 1);
            let unit = core.units.get_mut(enemy).unwrap();
            unit.stats.max_health = 10000;
            unit.stats.current_health = 10000;
            unit.stats.defense = 0;
            unit.stats.magic_resist = 0;
        }
        super::super::compiled_action_fixture::place(&mut core, actor, 2, 1);
        super::super::compiled_action_fixture::place(&mut core, ally, 0, 3);
        for spawn in &core.scenario.groups[1].spawns {
            let profile = spawn.draft.combat_profile(&core.game_data).unwrap();
            assert_eq!(
                profile.deployment_affinity,
                crate::game::battle::types::DeploymentAffinity::PlatformOnly
            );
            assert_eq!(profile.block_capacity, 0);
        }
        let unit = core.units.get_mut(&actor).unwrap();
        unit.facing_direction = Some(FacingDirection::Right);
        unit.stats.attack_interval_ms = unit.basic_attack.interval_ms;
        unit.stats.attack = 100;
        assert_eq!(unit.block_capacity, 0);
        assert_eq!(unit.basic_attack.range_role, WeaponRangeRole::Ranged);
        assert!(core
            .items
            .values()
            .any(|i| i.owner_unit_instance == actor && i.weapon_stage == stage));
        core.enable_authoring_observation();
        Gun {
            core,
            state,
            actor,
            enemies,
        }
    }
    fn advance(f: &mut Gun, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn release_one(f: &mut Gun) {
        assert_eq!(f.core.units[&f.actor].basic_attack_windup_until_ms, None);
        assert_eq!(
            f.core.select_basic_attack_target(f.actor, None, None),
            Some(f.enemies[0])
        );
        f.core
            .event_queue
            .discard_channel_periodic_starts(f.actor)
            .unwrap();
        f.core.schedule_initial_attack_for_unit(f.actor, 1);
        advance(f, 1);
        let release = 1 + f.core.units[&f.actor].basic_attack.windup_ms as u64;
        assert_eq!(
            f.core.units[&f.actor].basic_attack_windup_until_ms,
            Some(release)
        );
        advance(f, release);
        assert_eq!(
            f.core.projectiles.len(),
            1,
            "actual release must create a flight"
        );
        f.core
            .event_queue
            .discard_channel_periodic_starts(f.actor)
            .unwrap();
    }
    fn marks(f: &Gun, target: UnitInstanceId) -> u64 {
        f.core
            .weapon_marks_snapshot()
            .unwrap()
            .stores
            .iter()
            .flat_map(|s| &s.units)
            .filter(|(id, _)| *id == target)
            .map(|(_, n)| n)
            .sum()
    }
    fn full_snapshot(f: &Gun) -> String {
        format!(
            "{:?}\n{:?}\n{:?}\n{:?}\n{}",
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            f.state,
            f.core.weapon_marks_snapshot().unwrap(),
            serde_json::to_string(&f.core.authoring_observation).unwrap()
        )
    }
    #[test]
    fn composition_weapon_stages_real_periodic_release_uses_selected_cadence() {
        for (stage, expected) in [
            (WeaponStage::Base, vec![201]),
            (WeaponStage::Upgraded, vec![51, 301, 551]),
        ] {
            let mut f = setup(stage, false);
            f.core
                .event_queue
                .discard_channel_periodic_starts(f.actor)
                .unwrap();
            f.core.schedule_initial_attack_for_unit(f.actor, 1);
            advance(&mut f, 551);
            let fired: Vec<_> = f.core.event_log.entries.iter().filter_map(|entry| {
                match entry.event {
                    crate::game::battle::event_log::BattleLogEvent::BasicAttackProjectileLaunched {attacker_instance_id, fired_at_ms, ..} if attacker_instance_id == f.actor => Some(fired_at_ms),
                    _ => None,
                }
            }).collect();
            assert_eq!(fired, expected);
        }
    }
    #[test]
    fn composition_weapon_stages_public_retained_failure_restores_hit_then_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(WeaponStage::Upgraded, false);
        release_one(&mut f);
        advance(&mut f, 203);
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 204)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "weapon_stage_observation")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        let prior = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = full_snapshot(&f);
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 204)
            .unwrap_err();
        assert!(
            matches!(
                error,
                crate::game::behavior::GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "weapon_stage_observation",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(before, full_snapshot(&f));
        f.core.runtime_facts.limits.max_retained_bytes = prior;
        advance(&mut f, 204);
        assert_eq!(f.core.units[&f.enemies[0]].stats.current_health, 9900);
        assert_eq!(marks(&f, f.enemies[0]), 1);
    }
    #[test]
    fn composition_weapon_stages_authoring_and_native_admission_reject_invalid_shots() {
        use crate::authoring::composition::{compile_json, compile_yaml, parse_yaml, Artifact};
        let valid = compile_yaml(&canonical_source()).unwrap();
        let restored = Artifact::from_ron(&valid.to_ron().unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(valid.equipment_attacks()).unwrap(),
            serde_json::to_value(restored.equipment_attacks()).unwrap()
        );
        for (pointer, value, field) in [
            ("/interval_ms", json!(0), "interval_ms"),
            (
                "/push_distance_units",
                json!(32000001),
                "push_distance_units",
            ),
            (
                "/delivery/Projectile/speed_units_per_ms",
                json!(0),
                "speed_units_per_ms",
            ),
            (
                "/delivery/Projectile/max_range_tiles",
                json!(0),
                "max_range_tiles",
            ),
            (
                "/delivery/Projectile/max_lifetime_ms",
                json!(0),
                "max_lifetime_ms",
            ),
        ] {
            let mut doc = parse_yaml(&canonical_source()).unwrap();
            *doc["values"][0]["value"]["AttackProgram"]["weapon_stages"]["upgraded"]
                .pointer_mut(pointer)
                .unwrap() = value;
            let error = compile_json(&doc.to_string()).unwrap_err();
            assert!(format!("{error:?}").contains(field), "{error:?}");
        }
        let mut doc = parse_yaml(&canonical_source()).unwrap();
        let magazine = parse_yaml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/magazine.skill.yaml"
        )))
        .unwrap();
        doc["values"][0]["value"]["AttackProgram"]["magazine"] =
            magazine["values"][0]["value"]["AttackProgram"]["magazine"].clone();
        let error = compile_json(&doc.to_string()).unwrap_err();
        assert!(
            format!("{error:?}").contains("cannot both own projectile delivery"),
            "{error:?}"
        );
        let f = setup(WeaponStage::Upgraded, false);
        let mut equipment = f.core.game_data.equipment_data.as_ref().clone();
        equipment
            .items
            .iter_mut()
            .find(|i| i.id == "r11_weapon")
            .unwrap()
            .weapon_profile
            .as_mut()
            .unwrap()
            .mark_weapon = None;
        let error = crate::game::ability::equipment_attack::EquipmentAttackCatalog::compile(
            &equipment,
            &ResourceCatalog { resources: vec![] },
            &crate::game::data::relic_data::RelicDatabase::default(),
            &Default::default(),
        )
        .unwrap_err();
        assert!(
            format!("{error:?}").contains("explicit mark-weapon evolution"),
            "{error:?}"
        );
    }
    #[test]
    fn composition_weapon_stages_deep_imprint_doubles_fresh_marks_only() {
        for (stage, expected) in [(WeaponStage::Base, 4), (WeaponStage::Upgraded, 2)] {
            let mut f = setup(stage, true);
            release_one(&mut f);
            advance(&mut f, 800);
            assert_eq!(marks(&f, f.enemies[0]), expected);
            super::super::compiled_action_fixture::hit(
                &mut f.core,
                f.actor,
                f.enemies[0],
                20000,
                801,
            );
            assert!(f.core.units[&f.enemies[0]].lifecycle.is_dead());
            assert_eq!(marks(&f, f.enemies[0]), 0);
            let snapshot = f.core.weapon_marks_snapshot().unwrap();
            let transferred: u64 = snapshot
                .stores
                .iter()
                .flat_map(|s| &s.tiles)
                .map(|(_, n)| n)
                .sum();
            assert_eq!(
                transferred, expected,
                "death transfer must not double again"
            );
        }
    }
    #[test]
    fn composition_weapon_stages_airborne_and_large_targets_keep_damage_without_push() {
        for airborne in [true, false] {
            let mut f = setup(WeaponStage::Upgraded, false);
            let target = f.enemies[0];
            let unit = f.core.units.get_mut(&target).unwrap();
            if airborne {
                unit.mobility_kind = crate::game::battle::types::MobilityKind::Airborne;
            } else {
                unit.body.radius = 0.6;
            }
            let from = unit.body.position.to_data_units();
            release_one(&mut f);
            advance(&mut f, 800);
            assert_eq!(f.core.units[&target].stats.current_health, 9900);
            assert_eq!(marks(&f, target), 1);
            assert_eq!(f.core.units[&target].body.position.to_data_units(), from);
        }
    }
    #[test]
    fn composition_weapon_stages_push_respects_terrain_and_body_contact() {
        use crate::game::battle::core::movement::types::WorldVec2;
        for terrain in [true, false] {
            let mut f = setup(WeaponStage::Upgraded, false);
            let target = f.enemies[0];
            let radius = f.core.units[&target].body.radius;
            super::super::compiled_action_fixture::place(&mut f.core, f.enemies[2], 6, 3);
            if terrain {
                super::super::compiled_action_fixture::place(&mut f.core, f.enemies[1], 5, 3);
                f.core
                    .battlefield
                    .add_static_obstacle(crate::game::resources::Position::new(5, 1))
                    .unwrap();
                f.core.units.get_mut(&target).unwrap().body.position =
                    WorldVec2::new(5.0 - radius, 1.5);
            } else {
                let other_radius = f.core.units[&f.enemies[1]].body.radius;
                f.core.units.get_mut(&f.enemies[1]).unwrap().body.position =
                    WorldVec2::new(4.5 + radius + other_radius, 1.5);
            }
            let from = f.core.units[&target].body.position.to_data_units();
            release_one(&mut f);
            // Finish the first impact before any later enemy can move.
            advance(&mut f, 250);
            assert_eq!(f.core.units[&target].stats.current_health, 9900);
            assert_eq!(marks(&f, target), 1);
            assert_eq!(f.core.units[&target].body.position.to_data_units(), from);
        }
    }
    #[test]
    fn composition_weapon_stages_short_push_clips_to_remaining_clearance() {
        use crate::game::battle::core::movement::types::WorldVec2;
        for terrain in [true, false] {
            let mut f = setup(WeaponStage::Upgraded, false);
            let target = f.enemies[0];
            let radius = f.core.units[&target].body.radius;
            super::super::compiled_action_fixture::place(&mut f.core, f.enemies[2], 6, 3);
            if terrain {
                super::super::compiled_action_fixture::place(&mut f.core, f.enemies[1], 5, 3);
                f.core
                    .battlefield
                    .add_static_obstacle(crate::game::resources::Position::new(5, 1))
                    .unwrap();
                f.core.units.get_mut(&target).unwrap().body.position =
                    WorldVec2::new(5.0 - radius - 0.05, 1.5);
            } else {
                let other_radius = f.core.units[&f.enemies[1]].body.radius;
                f.core.units.get_mut(&f.enemies[1]).unwrap().body.position =
                    WorldVec2::new(4.5 + radius + other_radius + 0.05, 1.5);
            }
            let from = f.core.units[&target].body.position.to_data_units();
            release_one(&mut f);
            advance(&mut f, 250);
            let to = f.core.units[&target].body.position.to_data_units();
            assert_eq!(f.core.units[&target].stats.current_health, 9900);
            assert_eq!(marks(&f, target), 1);
            assert!(
                (to.0 - from.0 - 50000).abs() <= 2,
                "terrain={terrain}, from={from:?}, to={to:?}"
            );
            assert_eq!(to.1, from.1);
        }
    }
    #[test]
    fn composition_weapon_stages_withdraw_before_release_cancels_but_flight_survives() {
        for released in [false, true] {
            let mut f = setup(WeaponStage::Upgraded, false);
            if released {
                release_one(&mut f);
            } else {
                f.core.schedule_initial_attack_for_unit(f.actor, 1);
                advance(&mut f, 1);
                assert_eq!(
                    f.core.units[&f.actor].basic_attack_windup_until_ms,
                    Some(51)
                );
            }
            f.core
                .apply_live_command(BattleLiveCommand::WithdrawUnit {
                    unit_id: f.actor,
                    time_ms: if released { 52 } else { 2 },
                })
                .unwrap();
            advance(&mut f, 1700);
            assert!(
                f.core.projectiles.is_empty(),
                "released flights have finite range/life"
            );
            for target in f.enemies {
                assert_eq!(
                    f.core.units[&target].stats.current_health,
                    if released { 9900 } else { 10000 }
                );
                assert_eq!(marks(&f, target), if released { 1 } else { 0 });
            }
        }
    }
    #[test]
    fn composition_weapon_stages_clone_and_observation_off_preserve_continuation() {
        let mut f = setup(WeaponStage::Upgraded, false);
        release_one(&mut f);
        let mut clone = Gun {
            core: f.core.clone(),
            state: f.state.clone(),
            actor: f.actor,
            enemies: f.enemies,
        };
        let mut silent = Gun {
            core: f.core.clone(),
            state: f.state.clone(),
            actor: f.actor,
            enemies: f.enemies,
        };
        silent.core.authoring_observation = None;
        advance(&mut f, 1700);
        advance(&mut clone, 1700);
        advance(&mut silent, 1700);
        assert_eq!(full_snapshot(&f), full_snapshot(&clone));
        assert_eq!(
            f.core.event_queue_debug_snapshot(),
            silent.core.event_queue_debug_snapshot()
        );
        for target in f.enemies {
            assert_eq!(
                f.core.units[&target].stats.current_health,
                silent.core.units[&target].stats.current_health
            );
            assert_eq!(
                f.core.units[&target].body.position.to_data_units(),
                silent.core.units[&target].body.position.to_data_units()
            );
            assert_eq!(marks(&f, target), marks(&silent, target));
        }
    }
    #[test]
    fn composition_weapon_stages_public_cancel_restores_damage_marks_push_and_queue() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for site in [
            "weapon_mark_granted",
            "shot_push_interrupt",
            "weapon_stage_after_hit",
        ] {
            let mut f = setup(WeaponStage::Upgraded, false);
            release_one(&mut f);
            // This fixture's first collision dispatches at 204. Earlier empty
            // flight events are separate successful transactions.
            advance(&mut f, 203);
            assert_eq!(f.core.units[&f.enemies[0]].stats.current_health, 10000);
            let before = full_snapshot(&f);
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || f.core.step_battle_execution_until(&mut f.state, 300),
                )
            });
            assert!(
                result.is_err() && token.is_cancelled(),
                "{site}: {result:?}"
            );
            assert_eq!(before, full_snapshot(&f), "{site}");
            advance(&mut f, 300);
            assert_eq!(f.core.units[&f.enemies[0]].stats.current_health, 9900);
            assert_eq!(marks(&f, f.enemies[0]), 1);
            assert_eq!(
                f.core.units[&f.enemies[0]].body.position.to_data_units(),
                (4600000, 1500000)
            );
        }
    }
    #[test]
    fn composition_weapon_stages_motion_push_uses_collision_time_not_tick_endpoint() {
        use crate::game::battle::core::movement::types::{MovementGoal, WorldVec2};
        let mut f = setup(WeaponStage::Upgraded, false);
        release_one(&mut f);
        advance(&mut f, 200);
        let target = f.enemies[0];
        let unit = f.core.units.get_mut(&target).unwrap();
        unit.body.move_speed = 2.0;
        unit.stats.move_speed_units_per_ms = 2000;
        unit.blockable = false;
        let goals = std::collections::HashMap::from([(
            target,
            MovementGoal::MoveToPoint {
                point: WorldVec2::new(6.5, 1.5),
                stop_radius: 0.1,
            },
        )]);
        f.core.run_test_movement_tick(200, 50, &goals);
        let endpoint = f.core.units[&target].body.position.to_data_units();
        let samples: Vec<_> = (200..=230)
            .map(|t| {
                f.core
                    .sample_unit_body_at(target, t)
                    .unwrap()
                    .position
                    .to_data_units()
            })
            .collect();
        let projectile_id = *f.core.projectiles.keys().next().unwrap();
        f.core
            .runtime_transaction(|c| {
                c.process_event(
                    BattleEvent::BasicAttackProjectileAdvance {
                        time_ms: 230,
                        projectile_id,
                        cause: Default::default(),
                    },
                    230,
                )
            })
            .unwrap();
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_weapon_stage
            .as_ref()
            .unwrap();
        assert_eq!(witness.target, target);
        assert!(
            witness.time_ms > 200 && witness.time_ms < 230,
            "{witness:?}"
        );
        assert_eq!(
            witness.from_data_units,
            samples[(witness.time_ms - 200) as usize]
        );
        assert_ne!(witness.from_data_units, endpoint);
        assert_eq!(
            witness.to_data_units,
            (
                witness.from_data_units.0 + 100000,
                witness.from_data_units.1
            )
        );
        assert_eq!(witness.actual_hp_loss, 100);
    }
    #[test]
    fn composition_weapon_stages_preflight_actual_base_and_upgraded_projectile_consumers() {
        for stage in [WeaponStage::Base, WeaponStage::Upgraded] {
            let mut f = setup(stage, false);
            let interval = if stage == WeaponStage::Base {
                1000
            } else {
                250
            };
            assert_eq!(f.core.units[&f.actor].stats.attack_interval_ms, interval);
            let from = f
                .enemies
                .map(|id| f.core.units[&id].body.position.to_data_units());
            release_one(&mut f);
            advance(&mut f, 800);
            for (i, enemy) in f.enemies.iter().enumerate() {
                let hit = stage == WeaponStage::Upgraded || i == 0;
                assert_eq!(
                    f.core.units[enemy].stats.current_health,
                    if hit { 9900 } else { 10000 }
                );
                assert_eq!(
                    marks(&f, *enemy),
                    if hit {
                        if stage == WeaponStage::Base {
                            2
                        } else {
                            1
                        }
                    } else {
                        0
                    }
                );
                let to = f.core.units[enemy].body.position.to_data_units();
                assert_eq!(
                    to,
                    (
                        from[i].0
                            + if stage == WeaponStage::Upgraded {
                                100000
                            } else {
                                0
                            },
                        from[i].1
                    )
                );
            }
            let witness = f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_weapon_stage
                .as_ref()
                .unwrap();
            assert_eq!(witness.stage, stage);
            assert_eq!(witness.interval_ms, interval);
            assert_eq!(witness.actual_hp_loss, 100);
            assert!(witness.equipment.persistent_copy_uuid.is_some());
            assert!(witness.field_path.ends_with(if stage == WeaponStage::Base {
                "weapon_stages.base"
            } else {
                "weapon_stages.upgraded"
            }));
            let item = f
                .core
                .items
                .values()
                .find(|i| i.owner_unit_instance == f.actor)
                .unwrap();
            assert_eq!(
                f.core
                    .game_data
                    .equipment_data
                    .get_by_uuid(&item.base_uuid)
                    .unwrap()
                    .weapon_profile
                    .as_ref()
                    .unwrap()
                    .weapon_archetype,
                WeaponArchetype::Gun
            );
        }
    }
}
