mod occupied_tiles_contract {
    use super::*;
    fn fixture() -> Fixture {
        let equipment: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        let mut profile = equipment
            .items
            .iter()
            .find(|w| w.id == "placeholder_burrowing_heaven_weapon")
            .unwrap()
            .weapon_profile
            .clone()
            .unwrap();
        // Explicit test timing and mask; actual action/delivery come from the catalog.
        profile.interval_ms = 400;
        profile.windup_ms = 80;
        profile.defense_tile_range = ron::de::from_str(
            "(include_anchor_tile:false,rows:[\"XXXXX\",\"XXXXX\",\"XX@XX\",\"XXXXX\",\"XXXXX\"])",
        )
        .unwrap();
        let mut f = fixture_profile(0, profile);
        place(&mut f.core, f.primary, 3, 1);
        place(&mut f.core, f.other, 4, 1);
        f
    }
    fn tiles(f: &Fixture) -> Vec<Position> {
        f.core
            .event_log
            .entries
            .iter()
            .rev()
            .find_map(|entry| match &entry.event {
                BattleLogEvent::BasicAttackTilesReleased {
                    attacker_instance_id,
                    affected_tiles,
                } if *attacker_instance_id == f.actor => Some(affected_tiles.clone()),
                _ => None,
            })
            .expect("a committed tile release")
    }
    #[test]
    fn basic_occupied_tiles_native_cadence_repeats_one_release_per_attack() {
        let mut f = fixture();
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .basic_attack
            .interval_ms = 1800;
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .basic_attack
            .windup_ms = 280;
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .stats
            .attack_interval_ms = 1800;
        start(&mut f, 10);
        run(&mut f, 289);
        assert_eq!(hp(&f, f.primary), 1000);
        run(&mut f, 290);
        assert_eq!(hp(&f, f.primary), 900);
        assert_eq!(hp(&f, f.other), 900);
        run(&mut f, 2090);
        assert_eq!(hp(&f, f.primary), 800);
        assert_eq!(hp(&f, f.other), 800);
        let releases: Vec<_> = f.core.event_log.entries.iter().filter_map(|e| matches!(e.event,
            BattleLogEvent::BasicAttackTilesReleased { attacker_instance_id, .. } if attacker_instance_id == f.actor).then_some(e.time_ms)).collect();
        assert_eq!(releases, vec![290, 2090]);
    }
    #[test]
    fn basic_occupied_tiles_samples_movement_at_release() {
        use crate::game::battle::core::movement::types::{ActiveMovementSegment, WorldVec2};
        for primary in [true, false] {
            let mut f = fixture();
            let id = if primary { f.primary } else { f.other };
            let origin = if primary {
                Position::new(3, 1)
            } else {
                Position::new(4, 1)
            };
            f.core.active_movement_segments.insert(
                id,
                ActiveMovementSegment {
                    start: WorldVec2::from_tile_center(origin),
                    target: WorldVec2::from_tile_center(Position::new(7, 3)),
                    started_at_ms: 10,
                    ends_at_ms: 90,
                },
            );
            assert_eq!(f.core.live_unit_projected_tile(id), Some(origin));
            assert_eq!(
                f.core.resolve_basic_attack(f.actor, f.primary, 90),
                !primary
            );
            assert_eq!(hp(&f, id), 1000);
            if !primary {
                assert_eq!(tiles(&f), vec![Position::new(3, 1)]);
            }
        }
    }
    #[test]
    fn basic_occupied_tiles_release_eligibility_dedup_and_fixed_positions() {
        for (x, y, air, friendly, allowed_air, expected) in [
            (3, 1, false, false, true, 900), // Two enemies, one visual tile.
            (4, 1, false, false, true, 900),
            (6, 1, false, false, true, 1000),
            (4, 1, true, false, false, 1000),
            (4, 1, true, false, true, 900),
            (4, 1, false, true, true, 1000),
        ] {
            let mut f = fixture();
            f.core
                .units
                .get_mut(&f.actor)
                .unwrap()
                .basic_attack
                .air_capable = allowed_air;
            place(&mut f.core, f.other, x, y);
            if air {
                f.core.units.get_mut(&f.other).unwrap().mobility_kind = MobilityKind::Airborne;
            }
            if friendly {
                f.core.units.get_mut(&f.other).unwrap().owner = Side::Player;
            }
            start(&mut f, 10);
            run(&mut f, 89);
            assert_eq!(hp(&f, f.primary), 1000);
            run(&mut f, 90);
            assert_eq!(hp(&f, f.primary), 900);
            assert_eq!(
                hp(&f, f.other),
                expected,
                "({x},{y}) air={air} friendly={friendly}"
            );
            assert_eq!(hp(&f, f.ally), 1000);
            let captured = tiles(&f);
            let expected_tiles = if expected == 900 && x != 3 {
                vec![Position::new(3, 1), Position::new(x, y)]
            } else {
                vec![Position::new(3, 1)]
            };
            assert_eq!(captured, expected_tiles);
            place(&mut f.core, f.other, 7, 3);
            run(&mut f, 100);
            assert_eq!(
                tiles(&f),
                captured,
                "released visuals never follow later movement"
            );
            assert!(!f.core.event_log.entries.iter().any(|e| matches!(e.event, BattleLogEvent::BasicAttackProjectileLaunched { attacker_instance_id, .. } if attacker_instance_id == f.actor)));
        }
    }
    #[test]
    fn basic_occupied_tiles_requeries_at_release_and_lost_primary_misses() {
        let mut f = fixture();
        place(&mut f.core, f.other, 7, 3);
        start(&mut f, 10);
        place(&mut f.core, f.other, 4, 1);
        run(&mut f, 90);
        assert_eq!(hp(&f, f.other), 900);
        assert_eq!(tiles(&f), vec![Position::new(3, 1), Position::new(4, 1)]);
        let mut f = fixture();
        start(&mut f, 10);
        place(&mut f.core, f.primary, 7, 3);
        run(&mut f, 90);
        assert_eq!(hp(&f, f.primary), 1000);
        assert_eq!(hp(&f, f.other), 1000);
        assert!(!f
            .core
            .event_log
            .entries
            .iter()
            .any(|e| matches!(e.event, BattleLogEvent::BasicAttackTilesReleased { .. })));
    }
    #[test]
    fn basic_occupied_tiles_withdraw_cancels_windup() {
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
    fn basic_occupied_tiles_scan_cancellation_rolls_back_and_retries() {
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
        let result = control::with_test_cancellation("basic_occupied_tiles_scan", 1, || {
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
    fn basic_occupied_tiles_overlay_restores_native_action() {
        let mut f = fixture();
        let overlay = ron::de::from_str(r#"BasicAttackOverlay(duration_ms:1000,next_attack_only:true,
            profile:(range_policy:whole_field_valid_tiles,defense_tile_range:None,delivery:Instant))"#).unwrap();
        f.core
            .begin_sustained(f.actor, f.actor, overlay, 0)
            .unwrap();
        assert!(matches!(
            f.core.units[&f.actor].basic_attack.action,
            BasicAttackAction::Damage
        ));
        start(&mut f, 10);
        run(&mut f, 90);
        assert_eq!(hp(&f, f.other), 1000);
        assert!(matches!(
            f.core.units[&f.actor].basic_attack.action,
            BasicAttackAction::DamageOccupiedTiles
        ));
        start(&mut f, 500);
        run(&mut f, 580);
        assert_eq!(hp(&f, f.other), 900);
    }
    #[test]
    fn basic_occupied_tiles_native_retained_refusal_is_atomic() {
        use crate::game::battle::runtime_control as native;
        let mut f = fixture();
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
            .find(|(site, _, _)| *site == "basic_occupied_tiles")
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
                    site: "basic_occupied_tiles",
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
}
