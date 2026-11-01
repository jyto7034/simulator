mod composition_equipment_death_burst {
    use super::*;
    use crate::game::battle::core::{
        equipment_death_burst::DeathBurstOperation, sim::BattleLiveCommand,
    };
    use crate::game::battle::damage::{
        BattleCommand, DamageSource, DamageSourceSnapshot, DamageType,
    };
    fn canonical_source() -> String {
        crate::authoring::composition::canonical_funeral_tests::equipment_fixture("accessory")
    }
    fn setup_burst() -> (Hunt, UnitInstanceId, UnitInstanceId) {
        let mut f = setup_source_with(false, true, &canonical_source(), |scenario| {
            scenario.groups[0].enemy_movement_plan = None;
            scenario.groups[0].spawns[0].unit_ref = ScenarioUnitRef::new("burst-initial");
            for index in 1..=2 {
                let mut other = scenario.groups[0].spawns[0].clone();
                other.unit_ref = ScenarioUnitRef::new(format!("burst-enemy-{index}"));
                other.instance_salt += 100 * index;
                other.position = crate::game::resources::Position::new(4 + index as i32, 3);
                scenario.groups[0].spawns.push(other);
            }
        });
        f.enemy = f.core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("burst-initial")];
        let second =
            f.core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("burst-enemy-1")];
        let third =
            f.core.scenario_runtime.spawned.unit_refs[&ScenarioUnitRef::new("burst-enemy-2")];
        for (id, x) in [(f.a, 1), (f.b, 0), (f.enemy, 4), (second, 5), (third, 6)] {
            place(&mut f.core, id, x, 3);
            let unit = f.core.units.get_mut(&id).unwrap();
            unit.stats.attack = 100;
            unit.stats.defense = 0;
            unit.stats.magic_resist = 0;
        }
        f.core.units.get_mut(&second).unwrap().stats.current_health = 100;
        f.core.units.get_mut(&third).unwrap().stats.current_health = 300;
        assert!(f.core.units[&f.a].is_active());
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
        assert_eq!(
            f.core
                .equipped_attack_providers(f.a, |p| p.death_burst.is_some())
                .unwrap()
                .len(),
            1
        );
        assert_eq!(f.core.sustained.death_bursts.entity_count(), 0);
        (f, second, third)
    }
    #[test]
    fn composition_equipment_death_burst_preflight_delayed_first_delivery_and_chain() {
        let (mut f, second, third) = setup_burst();
        hit(&mut f.core, f.a, f.enemy, 20000, 0);
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert_eq!(
            (
                witness.center.x,
                witness.center.y,
                witness.due_ms,
                witness.attack,
                witness.base_damage,
                witness.pending
            ),
            (4, 3, 500, 100, 100, 1)
        );
        assert!(matches!(witness.operation, DeathBurstOperation::Reserved));
        f.core.units.get_mut(&f.a).unwrap().stats.attack = 200;
        advance(&mut f, 499);
        assert_eq!(f.core.units[&second].stats.current_health, 100);
        advance(&mut f, 500);
        assert!(f.core.units[&second].lifecycle.is_dead());
        assert_eq!(f.core.units[&third].stats.current_health, 300);
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert!(matches!(witness.operation, DeathBurstOperation::Detonated));
        assert_eq!((witness.targets, witness.pending), (1, 1));
        advance(&mut f, 999);
        assert_eq!(f.core.units[&third].stats.current_health, 300);
        advance(&mut f, 1000);
        assert_eq!(f.core.units[&third].stats.current_health, 100);
        assert_eq!(f.core.units[&f.b].stats.current_health, 10000);
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert_eq!(
            (witness.attack, witness.base_damage, witness.pending),
            (200, 200, 0)
        );
        assert!(witness.field_path.ends_with("death_burst"));
        f.core
            .runtime_transaction(|core| core.equipment_death_burst_tick(1, 500))
            .unwrap();
        f.core
            .runtime_transaction(|core| core.equipment_death_burst_tick(2, 1000))
            .unwrap();
        assert_eq!(f.core.units[&third].stats.current_health, 100);
    }
    #[test]
    fn composition_equipment_death_burst_withdrawal_cancels_and_late_event_is_inert() {
        let (mut f, second, _) = setup_burst();
        hit(&mut f.core, f.a, f.enemy, 20000, 0);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 100,
            })
            .unwrap();
        assert!(!f.core.units[&f.a].is_active());
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert!(matches!(witness.operation, DeathBurstOperation::Cancelled));
        assert_eq!(witness.pending, 0);
        advance(&mut f, 500);
        f.core
            .runtime_transaction(|core| core.equipment_death_burst_tick(1, 500))
            .unwrap();
        assert_eq!(f.core.units[&second].stats.current_health, 100);
    }

    fn raw_hit(
        f: &mut Hunt,
        kind: DamageSource,
        reaction: bool,
    ) -> Result<DamageSourceSnapshot, GameError> {
        f.core.runtime_transaction(|core| {
            let mut source = core
                .damage_source_snapshot_for_unit(
                    f.a,
                    f.enemy,
                    kind,
                    DamageType::True,
                    20000,
                    Default::default(),
                    0,
                    0,
                    false,
                )
                .ok_or(GameError::InvalidAction)?;
            source.fact_source.reaction = reaction;
            core.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: f.enemy,
                    source_snapshot: source.clone(),
                }],
                0,
            );
            crate::game::battle::runtime_control::poll()?;
            Ok(source)
        })
    }
    #[test]
    fn composition_equipment_death_burst_exclusions_reach_real_death_and_assist_does_not_claim() {
        for (kind, reaction, corrosion) in [
            (DamageSource::BuffTick, false, false),
            (DamageSource::Environment, false, false),
            (DamageSource::Ability, true, false),
            (DamageSource::BasicAttack, true, false),
            (DamageSource::Ability, false, true),
        ] {
            let (mut f, _, _) = setup_burst();
            f.core.units.get_mut(&f.a).unwrap().corroded = corrosion;
            raw_hit(&mut f, kind, reaction).unwrap();
            assert!(
                f.core.units[&f.enemy].lifecycle.is_dead(),
                "excluded hit must actually kill"
            );
            assert_eq!(
                f.core.sustained.death_bursts.entity_count(),
                0,
                "{kind:?}/{reaction}/{corrosion}"
            );
        }
        let (mut f, _, _) = setup_burst();
        hit(&mut f.core, f.a, f.enemy, 100, 0);
        assert!(f.core.units[&f.enemy].is_active());
        assert_eq!(f.core.sustained.death_bursts.entity_count(), 0);
        hit(&mut f.core, f.b, f.enemy, 20000, 0);
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert_eq!(witness.actor, f.b);
        assert_eq!(witness.pending, 1);
    }
    #[test]
    fn composition_equipment_death_burst_duplicate_death_stays_single_before_and_after_delivery() {
        let (mut f, second, _) = setup_burst();
        f.core.units.get_mut(&second).unwrap().stats.current_health = 1000;
        let source = raw_hit(&mut f, DamageSource::BasicAttack, false).unwrap();
        let victim = f.enemy;
        for _ in 0..2 {
            f.core
                .runtime_transaction(|core| core.equipment_death_burst_kill(&source, victim, 0))
                .unwrap();
        }
        assert_eq!(f.core.sustained.death_bursts.entity_count(), 2);
        advance(&mut f, 500);
        assert_eq!(f.core.units[&second].stats.current_health, 900);
        f.core
            .runtime_transaction(|core| core.equipment_death_burst_kill(&source, victim, 500))
            .unwrap();
        assert_eq!(f.core.sustained.death_bursts.entity_count(), 1);
        advance(&mut f, 1000);
        assert_eq!(f.core.units[&second].stats.current_health, 900);
    }
    #[test]
    fn composition_equipment_death_burst_fixed_center_queries_current_targets() {
        let (mut f, second, third) = setup_burst();
        hit(&mut f.core, f.a, f.enemy, 20000, 0);
        advance(&mut f, 499);
        place(&mut f.core, second, 6, 3);
        place(&mut f.core, third, 4, 3);
        advance(&mut f, 500);
        assert_eq!(f.core.units[&second].stats.current_health, 100);
        assert_eq!(f.core.units[&third].stats.current_health, 200);
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert_eq!(
            (
                witness.center.x,
                witness.center.y,
                witness.targets,
                witness.pending
            ),
            (4, 3, 1, 0)
        );
    }
    #[test]
    fn composition_equipment_death_burst_owner_death_and_terminal_clear_pending_deliveries() {
        for owner_dies in [true, false] {
            let (mut f, second, _) = setup_burst();
            hit(&mut f.core, f.a, f.enemy, 20000, 0);
            if owner_dies {
                hit(&mut f.core, second, f.a, 20000, 100);
                assert!(f.core.units[&f.a].lifecycle.is_dead());
            } else {
                Arc::make_mut(&mut f.core.scenario).terminal_outcomes =
                    BattleScenario::fixture_survival_outcomes(100);
                advance(&mut f, 100);
                assert!(f.state.is_finished());
                f.core.finalize_battle_execution(&mut f.state).unwrap();
                assert_eq!(f.core.sustained.death_bursts.entity_count(), 0);
            }
            let witness = f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_equipment_death_burst
                .as_ref()
                .unwrap();
            assert!(matches!(witness.operation, DeathBurstOperation::Cancelled));
            assert_eq!(witness.pending, 0);
            f.core
                .runtime_transaction(|core| core.equipment_death_burst_tick(1, 500))
                .unwrap();
            assert_eq!(f.core.units[&second].stats.current_health, 100);
        }
    }
    #[test]
    fn composition_equipment_death_burst_reservation_and_hit_cancellation_restore_then_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let (mut f, second, third) = setup_burst();
        let snapshot = |f: &Hunt| {
            (
                f.core.transaction_debug_snapshot(),
                format!("{:?}", f.core.sustained.death_bursts),
                format!("{:?}", f.state),
            )
        };
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("death_burst_reserved", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || raw_hit(&mut f, DamageSource::Ability, false),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, snapshot(&f));
        raw_hit(&mut f, DamageSource::Ability, false).unwrap();
        advance(&mut f, 499);
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("death_burst_hit", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 500),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, snapshot(&f));
        assert_eq!(f.core.units[&second].stats.current_health, 100);
        advance(&mut f, 500);
        assert!(f.core.units[&second].lifecycle.is_dead());
        advance(&mut f, 1000);
        assert_eq!(f.core.units[&third].stats.current_health, 200);
    }
    #[test]
    fn composition_equipment_death_burst_retained_failure_restores_and_observer_is_optional() {
        use crate::game::battle::runtime_control as native;
        let (mut f, second, _) = setup_burst();
        let mut probe = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a: f.a,
            b: f.b,
            enemy: f.enemy,
        };
        let (result, admissions) = native::observe_retained_admissions(|| {
            raw_hit(&mut probe, DamageSource::Ability, false)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "death_burst_observation")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        let original = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let snapshot = |f: &Hunt| {
            (
                f.core.transaction_debug_snapshot(),
                format!("{:?}", f.core.sustained.death_bursts),
            )
        };
        let before = snapshot(&f);
        let error = raw_hit(&mut f, DamageSource::Ability, false).unwrap_err();
        assert!(
            format!("{error:?}").contains("RetainedByteLimit"),
            "{error:?}"
        );
        assert_eq!(before, snapshot(&f));
        f.core.runtime_facts.limits.max_retained_bytes = original;
        f.core.authoring_observation = None;
        raw_hit(&mut f, DamageSource::Ability, false).unwrap();
        advance(&mut f, 500);
        assert!(f.core.units[&second].lifecycle.is_dead());
        assert_eq!(f.core.sustained.death_bursts.entity_count(), 3);
    }

    #[test]
    fn composition_equipment_death_burst_death_time_samples_motion_instead_of_tick_endpoint() {
        use crate::game::battle::core::movement::types::{MovementGoal, WorldVec2};
        let (mut f, _, _) = setup_burst();
        advance(&mut f, 1000);
        let unit = f.core.units.get_mut(&f.enemy).unwrap();
        unit.body.position = WorldVec2::new(3.99, 1.5);
        unit.body.move_speed = 2.0;
        unit.stats.move_speed_units_per_ms = 2000;
        unit.blockable = false;
        let goals = std::collections::HashMap::from([(
            f.enemy,
            MovementGoal::MoveToPoint {
                point: WorldVec2::new(6.5, 1.5),
                stop_radius: 0.1,
            },
        )]);
        f.core.run_test_movement_tick(1000, 50, &goals);
        assert_eq!(
            f.core
                .sample_unit_body_at(f.enemy, 1001)
                .unwrap()
                .projected_tile(),
            crate::game::resources::Position::new(3, 1)
        );
        assert_eq!(
            f.core.units[&f.enemy].body.projected_tile(),
            crate::game::resources::Position::new(4, 1)
        );
        hit(&mut f.core, f.a, f.enemy, 20000, 1001);
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert_eq!(
            (witness.center.x, witness.center.y, witness.due_ms),
            (3, 1, 1501)
        );
    }
    #[test]
    fn composition_equipment_death_burst_actual_basic_attack_owns_reservation() {
        let (mut f, second, _) = setup_burst();
        place(&mut f.core, f.a, 2, 3);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 100;
        f.core
            .event_queue
            .discard_channel_periodic_starts(f.a)
            .unwrap();
        f.core.schedule_initial_attack_for_unit(f.a, 1);
        advance(&mut f, 1);
        assert_eq!(f.core.units[&f.a].current_target, Some(f.enemy));
        assert_eq!(f.core.units[&f.a].basic_attack_windup_until_ms, Some(201));
        advance(&mut f, 201);
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert_eq!(
            (witness.actor, witness.due_ms, witness.base_damage),
            (f.a, 701, 100)
        );
        advance(&mut f, 700);
        assert!(f.core.units[&second].is_active());
        advance(&mut f, 701);
        assert!(f.core.units[&second].lifecycle.is_dead());
    }
    #[test]
    fn composition_equipment_death_burst_real_fragment_path_explosion_starts_accessory_chain() {
        let graph = crate::authoring::node_graph::tests::tile_burst_fixture("");
        let mut f = setup_source_with_graph(
            true,
            true,
            &canonical_source(),
            &graph,
            &[
                "widened-horizon",
                "overlapping-explosions",
                "explosion-vulnerability",
                "deep-imprint",
                "funeral-butterfly-orb",
            ],
            |_| {},
        );
        place(&mut f.core, f.enemy, 6, 3);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 10;
        f.core.units.get_mut(&f.a).unwrap().stats.attack = 100;
        f.core.units.get_mut(&f.b).unwrap().corroded = true;
        super::super::super::compiled_action_fixture::cast(&mut f.core, f.a, 10);
        advance(&mut f, 109);
        assert!(f.core.units[&f.enemy].is_active());
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .is_none());
        advance(&mut f, 110);
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert_eq!(
            (
                witness.actor,
                witness.due_ms,
                witness.attack,
                witness.base_damage
            ),
            (f.a, 610, 100, 100)
        );
        assert_eq!((witness.center.x, witness.center.y), (6, 3));
        // The fragment path does not grant weapon marks, and neither does the accessory.
        assert!(f.core.weapon_marks_snapshot().unwrap().stores.is_empty());
        advance(&mut f, 610);
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert!(matches!(witness.operation, DeathBurstOperation::Detonated));
        assert_eq!((witness.targets, witness.pending), (0, 0));
    }

    #[test]
    fn composition_equipment_death_burst_stun_preserves_delivery_air_target_resists_and_clone_agrees(
    ) {
        use crate::game::ability::sustained::{HitStatusKind, HitStatusProfile};
        let (mut f, second, _) = setup_burst();
        let enemy = f.core.units.get_mut(&second).unwrap();
        enemy.mobility_kind = crate::game::battle::types::MobilityKind::Airborne;
        enemy.stats.magic_resist = 100;
        hit(&mut f.core, f.a, f.enemy, 20000, 0);
        let accepted = f
            .core
            .runtime_transaction(|core| {
                core.begin_committed_hit_status(
                    second,
                    f.a,
                    Side::Opponent,
                    1000,
                    HitStatusProfile {
                        definition: "burst-stun-boundary".into(),
                        kind: HitStatusKind::Stun,
                        modifier_source: None,
                        vulnerability: None,
                    },
                    100,
                )
            })
            .unwrap();
        assert!(accepted);
        assert_eq!(f.core.hit_stun_until(f.a, Some(100)), Some(1099));
        assert_eq!(f.core.hit_stun_until(f.a, Some(1100)), None);
        advance(&mut f, 499);
        let mut clone = Hunt {
            core: f.core.clone(),
            state: f.state.clone(),
            a: f.a,
            b: f.b,
            enemy: f.enemy,
        };
        advance(&mut f, 500);
        advance(&mut clone, 500);
        assert_eq!(f.core.units[&second].stats.current_health, 50);
        assert_eq!(clone.core.units[&second].stats.current_health, 50);
        assert_eq!(f.core.hit_stun_until(f.a, Some(500)), Some(1099));
        assert_eq!(
            serde_json::to_value(
                f.core
                    .authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_equipment_death_burst
                    .as_ref()
            )
            .unwrap(),
            serde_json::to_value(
                clone
                    .core
                    .authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_equipment_death_burst
                    .as_ref()
            )
            .unwrap()
        );
    }
    #[test]
    fn composition_equipment_death_burst_actual_summon_is_not_the_wearer() {
        use crate::game::battle::{
            core::spawn_kernel::{
                SpawnOccupancyClass, SpawnOccupancyPolicy, SpawnPlacement, SpawnRequest,
                SpawnRequestId, SpawnRequestSettlement, SpawnRequestSource,
            },
            types::{BattleUnitSourceIdentity, UnitCombatProfile},
        };
        let (mut f, _, _) = setup_burst();
        let target = f.enemy;
        let mut group = f.core.scenario.groups[1].clone();
        group.id = ScenarioGroupId::new("burst-friendly-summon");
        group.required_for_victory = false;
        group.enemy_movement_plan = None;
        group.spawns.truncate(1);
        let spawn = &mut group.spawns[0];
        spawn.unit_ref = ScenarioUnitRef::new("burst-friendly-summon");
        spawn.instance_salt = 900;
        spawn.draft.owned_uuid = Uuid::from_u128(0xe600);
        spawn.draft.source = BattleUnitSource::SummonedUnit {
            definition_id: "burst-friendly-summon".into(),
            base_uuid: Uuid::from_u128(0xe600),
            profile: UnitCombatProfile::for_employee(100, 100, 0, 0, 100),
            summoner_instance_id: f.a,
            summon_sequence: 0,
            batch_uuid: Uuid::from_u128(0xe601),
        };
        spawn.draft.equipment_layout = None;
        spawn.draft.equipped_items.clear();
        spawn.draft.equipped_item_enhancements.clear();
        spawn.draft.employee_start_condition = None;
        let request = SpawnRequest {
            id: SpawnRequestId("burst-friendly-summon".into()),
            source: SpawnRequestSource::Summon {
                summoner: f.a,
                definition_id: "burst-friendly-summon".into(),
                count: 1,
            },
            owner: Side::Player,
            occupancy: SpawnOccupancyPolicy::AllowOccupied(vec![SpawnOccupancyClass::Any]),
            placements: vec![SpawnPlacement {
                order: 0,
                position: spawn.position,
                terrain_walkable: true,
                occupants: vec![],
            }],
        };
        let SpawnRequestSettlement::Spawned { unit_ids } = f
            .core
            .execute_spawn_request(request, |core, _| {
                core.spawn_scenario_group_member(&group, 0)
            })
            .unwrap()
        else {
            panic!("summon must be admitted")
        };
        assert_eq!(unit_ids.len(), 1);
        let summoned = unit_ids[0];
        assert!(f.core.units[&summoned].is_active());
        assert_eq!(f.core.units[&summoned].owner, Side::Player);
        assert!(matches!(f.core.units[&summoned].source_identity,
            BattleUnitSourceIdentity::SummonedUnit { summoner_instance_id, .. } if summoner_instance_id == f.a));
        f.core
            .runtime_transaction(|core| {
                let source = core
                    .damage_source_snapshot_for_unit(
                        summoned,
                        target,
                        DamageSource::BasicAttack,
                        DamageType::True,
                        20000,
                        Default::default(),
                        0,
                        0,
                        false,
                    )
                    .ok_or(GameError::InvalidAction)?;
                core.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: target,
                        source_snapshot: source,
                    }],
                    0,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert!(
            !f.core.units[&target].is_active(),
            "actual summon damage must kill the encounter enemy"
        );
        assert_eq!(f.core.sustained.death_bursts.entity_count(), 0);
    }
    #[test]
    fn composition_equipment_death_burst_real_bloom_consumes_weapon_marks_then_starts_chain() {
        let graph = crate::authoring::node_graph::tests::tile_burst_fixture("-bloom");
        let mut f = setup_source_with_graph(
            true,
            true,
            &canonical_source(),
            &graph,
            &[
                "widened-horizon",
                "overlapping-explosions",
                "explosion-vulnerability",
                "deep-imprint",
                "funeral-butterfly-orb",
            ],
            |scenario| scenario.groups[0].enemy_movement_plan = None,
        );
        place(&mut f.core, f.a, 2, 3);
        place(&mut f.core, f.enemy, 4, 3);
        f.core.units.get_mut(&f.a).unwrap().stats.attack = 100;
        f.core.units.get_mut(&f.enemy).unwrap().stats.defense = 0;
        f.core.units.get_mut(&f.b).unwrap().corroded = true;
        f.core
            .event_queue
            .discard_channel_periodic_starts(f.a)
            .unwrap();
        f.core.schedule_initial_attack_for_unit(f.a, 1);
        advance(&mut f, 1);
        assert_eq!(f.core.units[&f.a].current_target, Some(f.enemy));
        assert_eq!(f.core.units[&f.a].basic_attack_windup_until_ms, Some(201));
        advance(&mut f, 201);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
        let copy = f
            .core
            .items
            .values()
            .find(|i| i.owner_unit_instance == f.a && i.base_uuid != ACCESSORY)
            .unwrap()
            .persistent_copy_uuid
            .unwrap();
        assert_eq!(
            f.core.sustained.weapon_marks.pools[&(copy, "butterflies".into())].units[&f.enemy],
            2
        );
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .is_none());
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 10;
        super::super::super::compiled_action_fixture::cast(&mut f.core, f.a, 210);
        assert!(!f
            .core
            .sustained
            .weapon_marks
            .pools
            .contains_key(&(copy, "butterflies".into())));
        advance(&mut f, 309);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 10);
        advance(&mut f, 310);
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert_eq!(
            (
                witness.actor,
                witness.due_ms,
                witness.attack,
                witness.base_damage
            ),
            (f.a, 810, 100, 100)
        );
        assert_eq!((witness.center.x, witness.center.y), (4, 3));
        advance(&mut f, 810);
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_equipment_death_burst
            .as_ref()
            .unwrap();
        assert!(matches!(witness.operation, DeathBurstOperation::Detonated));
        assert_eq!((witness.targets, witness.pending), (0, 0));
        assert!(f.core.weapon_marks_snapshot().unwrap().stores.is_empty());
    }
}
