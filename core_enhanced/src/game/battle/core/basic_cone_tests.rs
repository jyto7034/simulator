mod basic_cone_contract {
    use super::compiled_action_fixture::place;
    use super::*;
    use crate::game::{
        battle::{
            core::sim::{BattleExecutionState, BattleLiveCommand},
            types::MobilityKind,
        },
        data::abnormality_data::BasicAttackAction,
    };
    struct Fixture {
        core: BattleCore,
        state: BattleExecutionState,
        actor: UnitInstanceId,
        ally: UnitInstanceId,
        primary: UnitInstanceId,
        other: UnitInstanceId,
    }
    fn fixture() -> Fixture {
        fixture_extra(0)
    }
    fn fixture_extra(extra: u32) -> Fixture {
        fixture_profile(extra, ron::de::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/larynx.weapon.ron"))).unwrap())
    }
    fn fixture_profile(extra: u32, profile: crate::game::data::equipment_data::WeaponCombatProfile) -> Fixture {
        let (mut core, state, actor, ally) = r12_fragment_equipped_effect_core_with_data(
            &r12_control_ron("Heal(amount:1)"),
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(8000);
                scenario.groups[0].enemy_movement_plan = None;
                let mut second = scenario.groups[0].spawns[0].clone();
                second.unit_ref =
                    crate::game::battle::scenario::ScenarioUnitRef::new("cone-secondary");
                second.instance_salt += 100;
                second.position = Position::new(6, 3);
                scenario.groups[0].spawns.push(second.clone());
                for index in 0..extra {
                    let mut spare = second.clone();
                    spare.unit_ref = crate::game::battle::scenario::ScenarioUnitRef::new(format!(
                        "cone-spare-{index}"
                    ));
                    spare.instance_salt += index + 200;
                    spare.position = Position::new(7, index as i32);
                    scenario.groups[0].spawns.push(spare);
                }
            },
            |weapon| {
                weapon.weapon_profile = Some(profile.clone());
            },
            None,
            |builder| builder,
        );
        let mut enemies: Vec<_> = core
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .map(|u| (u.spawn_order, u.instance_id))
            .collect();
        enemies.sort();
        let primary = enemies[0].1;
        let other = enemies[1].1;
        for unit in core.units.values_mut() {
            unit.stats.attack_interval_ms = 100_000;
            unit.next_basic_attack_ms = 100_000;
            unit.stats.move_speed_units_per_ms = 0;
            unit.body.move_speed = 0.0;
            unit.stats.defense = 0;
            unit.stats.magic_resist = 0;
            unit.stats.current_health = 1000;
            unit.stats.max_health = 1000;
        }
        core.units.get_mut(&actor).unwrap().stats.attack = 100;
        core.units.get_mut(&actor).unwrap().stats.attack_interval_ms = 400;
        for (id, x, y) in [(actor, 2, 1), (ally, 7, 3), (primary, 3, 1), (other, 6, 2)] {
            place(&mut core, id, x, y);
        }
        for unit in core
            .units
            .values_mut()
            .filter(|u| u.owner == Side::Opponent)
        {
            unit.enemy_movement_plan = Some(
                crate::game::battle::scenario::EnemyMovementPlan::PathAlongCells {
                    cells: vec![Position::new(7, 1), Position::new(0, 1)],
                },
            );
        }
        Fixture {
            core,
            state,
            actor,
            ally,
            primary,
            other,
        }
    }
    fn start(f: &mut Fixture, now: u64) {
        f.core.units.get_mut(&f.actor).unwrap().next_basic_attack_ms = now;
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: now,
                    attacker_instance_id: f.actor,
                    target_instance_id: Some(f.primary),
                    schedule_next: true,
                    cause: Default::default(),
                },
                now,
            )
            .unwrap();
        assert_eq!(f.core.units[&f.actor].current_target, Some(f.primary));
    }
    fn run(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn hp(f: &Fixture, id: UnitInstanceId) -> u32 {
        f.core.units[&id].stats.current_health
    }
    #[test]
    fn basic_cone_native_profile_regular_release_widening_and_eligibility() {
        for (x, y, air, friendly, expected) in [
            (3, 2, false, false, 1000),
            (6, 2, false, false, 900),
            (7, 3, false, false, 900),
            (6, 3, false, false, 1000),
            (1, 1, false, false, 1000),
            (6, 2, true, false, 900),
            (6, 2, false, true, 1000),
        ] {
            let mut f = fixture();
            place(&mut f.core, f.other, x, y);
            if air {
                f.core.units.get_mut(&f.other).unwrap().mobility_kind = MobilityKind::Airborne;
            }
            if friendly {
                f.core.units.get_mut(&f.other).unwrap().owner = Side::Player;
            }
            assert!(matches!(
                f.core.units[&f.actor].basic_attack.action,
                BasicAttackAction::DamageCone { .. }
            ));
            start(&mut f, 10);
            assert_eq!(f.core.units[&f.actor].next_basic_attack_ms, 410);
            run(&mut f, 89);
            assert_eq!(hp(&f, f.primary), 1000);
            run(&mut f, 90);
            assert_eq!(hp(&f, f.primary), 900);
            assert_eq!(
                hp(&f, f.other),
                expected,
                "({x},{y}) air={air} friend={friendly}"
            );
            assert_eq!(hp(&f, f.ally), 1000);
        }
    }
    #[test]
    fn basic_cone_release_tracks_current_primary_and_requeries_secondary() {
        let mut f = fixture();
        start(&mut f, 10);
        place(&mut f.core, f.primary, 4, 3);
        place(&mut f.core, f.other, 5, 3);
        run(&mut f, 90);
        assert_eq!(hp(&f, f.primary), 900);
        assert_eq!(
            hp(&f, f.other),
            900,
            "diagonal cone includes a target outside the old horizontal footprint"
        );
        let mut f = fixture();
        start(&mut f, 10);
        place(&mut f.core, f.primary, 7, 3);
        run(&mut f, 90);
        assert_eq!(hp(&f, f.primary), 1000);
        assert_eq!(
            hp(&f, f.other),
            1000,
            "lost primary does not release an untargeted cone"
        );
    }
    #[test]
    fn basic_cone_withdraw_cancels_windup() {
        let mut f = fixture();
        start(&mut f, 10);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.actor,
                time_ms: 20,
            })
            .unwrap();
        run(&mut f, 90);
        assert_eq!(hp(&f, f.primary), 1000);
        assert_eq!(hp(&f, f.other), 1000);
    }
    #[test]
    fn basic_cone_target_scan_cancellation_rolls_back_and_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = fixture();
        start(&mut f, 10);
        run(&mut f, 89);
        let before = (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("basic_cone_target_scan", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 90),
            )
        });
        assert!(result.is_err() && token.is_cancelled(), "{result:?}");
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.state)
            )
        );
        run(&mut f, 90);
        assert_eq!(hp(&f, f.primary), 900);
        assert_eq!(hp(&f, f.other), 900);
    }
    #[test]
    fn basic_cone_overlay_replaces_then_restores_native_action() {
        let mut f = fixture();
        place(&mut f.core, f.other, 4, 2);
        let overlay = ron::de::from_str(
            r#"BasicAttackOverlay(duration_ms:1000,next_attack_only:true,
            profile:(range_policy:whole_field_valid_tiles,defense_tile_range:None,
                delivery:TileArea(footprint:(include_anchor_tile:true,rows:["XXX","X@X","XXX"]),
                    area:(anchor:CastTarget,tile_origin:Anchor))))"#,
        )
        .unwrap();
        f.core
            .begin_sustained(f.actor, f.actor, overlay, 0)
            .unwrap();
        assert!(matches!(
            f.core.units[&f.actor].basic_attack.action,
            BasicAttackAction::Damage
        ));
        start(&mut f, 10);
        run(&mut f, 90);
        assert_eq!(
            hp(&f, f.other),
            900,
            "replacement area includes a target outside native cone"
        );
        assert!(matches!(
            f.core.units[&f.actor].basic_attack.action,
            BasicAttackAction::DamageCone { .. }
        ));
        place(&mut f.core, f.other, 6, 2);
        start(&mut f, 500);
        run(&mut f, 580);
        assert_eq!(hp(&f, f.primary), 800);
        assert_eq!(hp(&f, f.other), 800);
    }
    #[test]
    fn basic_cone_progress_priority_ties_air_and_length_gate() {
        let mut f = fixture();
        // The other route is almost finished, but its traversed distance is smaller.
        f.core.units.get_mut(&f.other).unwrap().enemy_movement_plan = Some(
            crate::game::battle::scenario::EnemyMovementPlan::PathAlongCells {
                cells: vec![Position::new(5, 2), Position::new(6, 2)],
            },
        );
        assert_eq!(
            f.core.choose_attack_target_in_range(f.actor),
            Some(f.primary)
        );
        place(&mut f.core, f.other, 3, 1);
        f.core.units.get_mut(&f.other).unwrap().enemy_movement_plan =
            f.core.units[&f.primary].enemy_movement_plan.clone();
        assert_eq!(
            f.core.choose_attack_target_in_range(f.actor),
            Some(f.primary)
        );
        let spawn = f.core.units[&f.primary].spawn_order;
        f.core.units.get_mut(&f.other).unwrap().spawn_order = spawn;
        assert_eq!(
            f.core.choose_attack_target_in_range(f.actor),
            Some(f.primary.min(f.other))
        );
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .basic_attack
            .air_capable = false;
        for id in [f.primary, f.other] {
            f.core.units.get_mut(&id).unwrap().mobility_kind = MobilityKind::Airborne;
        }
        assert!(f.core.choose_attack_target_in_range(f.actor).is_none());
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .basic_attack
            .air_capable = true;
        assert!(f.core.choose_attack_target_in_range(f.actor).is_some());
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .basic_attack
            .range_policy = crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .basic_attack
            .defense_tile_range = None;
        place(&mut f.core, f.primary, 7, 3);
        place(&mut f.core, f.other, 7, 3);
        assert!(
            f.core.choose_attack_target_in_range(f.actor).is_none(),
            "even global acquisition cannot extend native cone length"
        );
    }
    #[test]
    fn basic_cone_hits_all_six_enemies_once_without_attenuation() {
        let mut f = fixture_extra(4);
        start(&mut f, 10);
        run(&mut f, 90);
        let enemies: Vec<_> = f
            .core
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .collect();
        assert_eq!(enemies.len(), 6);
        for enemy in enemies {
            assert_eq!(enemy.stats.current_health, 900);
        }
    }
    #[test]
    fn basic_cone_native_retained_refusal_is_atomic() {
        use crate::game::battle::runtime_control as native;
        let mut f = fixture_extra(4);
        start(&mut f, 10);
        run(&mut f, 89);
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 90)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "basic_cone_targets")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state),
        );
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "basic_cone_targets",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.state)
            )
        );
        f.core.runtime_facts.limits = Default::default();
        run(&mut f, 90);
        assert_eq!(hp(&f, f.primary), 900);
        assert_eq!(hp(&f, f.other), 900);
    }
    #[test]
    fn basic_cone_sampled_primary_exit_is_a_miss_without_release_gain() {
        use crate::game::battle::core::movement::types::{ActiveMovementSegment, WorldVec2};
        for (target, hit) in [
            (Position::new(7, 3), false),
            (Position::new(1, 1), false),
            (Position::new(4, 3), true),
        ] {
            let mut f = fixture();
            f.core.units.get_mut(&f.actor).unwrap().resonance_current = 0;
            f.core.active_movement_segments.insert(
                f.primary,
                ActiveMovementSegment {
                    start: WorldVec2::from_tile_center(Position::new(3, 1)),
                    target: WorldVec2::from_tile_center(target),
                    started_at_ms: 10,
                    ends_at_ms: 90,
                },
            );
            // The cached body still occupies its previous cell. This is the
            // native interpolation boundary used between movement events.
            assert_eq!(
                f.core.live_unit_projected_tile(f.primary),
                Some(Position::new(3, 1))
            );
            assert_eq!(
                f.core.resolve_basic_attack(f.actor, f.primary, 90),
                hit,
                "sampled primary {target:?}"
            );
            if !hit {
                assert_eq!(f.core.units[&f.actor].resonance_current, 0);
            }
            assert_eq!(hp(&f, f.primary), if hit { 900 } else { 1000 });
            assert_eq!(hp(&f, f.other), 1000);
        }
    }
    include!("basic_occupied_tiles_tests.rs");
    include!("basic_saw_tests.rs");
    include!("basic_delayed_tile_tests.rs");
    include!("basic_multi_target_tests.rs");
}
