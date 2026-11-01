mod delayed_tile_contract {
    use super::*;
    fn fixture() -> Fixture {
        fixture_delay(400)
    }
    fn fixture_delay(delay_ms: u32) -> Fixture {
        let equipment: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        let mut profile = equipment
            .items
            .iter()
            .find(|w| w.id == "placeholder_king_of_greed_weapon")
            .unwrap()
            .weapon_profile
            .clone()
            .unwrap();
        profile.action = BasicAttackAction::DamageTileAfterDelay { delay_ms };
        profile.interval_ms = 100_000;
        profile.windup_ms = 80;
        profile.defense_tile_range = ron::de::from_str(
            "(include_anchor_tile:false,rows:[\"XXXXX\",\"XXXXX\",\"XX@XX\",\"XXXXX\",\"XXXXX\"])",
        )
        .unwrap();
        let mut f = fixture_profile(0, profile);
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .stats
            .attack_interval_ms = 100_000;
        place(&mut f.core, f.other, 3, 1);
        f
    }
    fn released(f: &Fixture) -> Vec<(u64, uuid::Uuid, Position, u64)> {
        f.core
            .event_log
            .entries
            .iter()
            .filter_map(|e| match e.event {
                BattleLogEvent::BasicAttackTileReleased {
                    release_id,
                    attacker_instance_id,
                    tile,
                    impact_at_ms,
                } if attacker_instance_id == f.actor => {
                    Some((e.time_ms, release_id, tile, impact_at_ms))
                }
                _ => None,
            })
            .collect()
    }
    fn impacts(f: &Fixture) -> Vec<(u64, uuid::Uuid, Position, u32)> {
        f.core
            .event_log
            .entries
            .iter()
            .filter_map(|e| match e.event {
                BattleLogEvent::BasicAttackTileImpacted {
                    release_id,
                    attacker_instance_id,
                    tile,
                    target_count,
                } if attacker_instance_id == f.actor => {
                    Some((e.time_ms, release_id, tile, target_count))
                }
                _ => None,
            })
            .collect()
    }
    #[test]
    fn basic_delayed_tile_native_schema_roundtrip_precise_errors() {
        use crate::game::data::abnormality_data::BasicAttackDef;
        let mut basic = BasicAttackDef {
            action: BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 },
            ..Default::default()
        };
        basic.try_validate_runtime_contract().unwrap();
        let ron = ron::ser::to_string(&basic).unwrap();
        let reopened: BasicAttackDef = ron::de::from_str(&ron).unwrap();
        assert!(matches!(
            reopened.action,
            BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 }
        ));
        assert!(reopened.action.is_damage());
        basic.action = BasicAttackAction::DamageTileAfterDelay { delay_ms: 0 };
        assert_eq!(
            basic
                .try_validate_runtime_contract()
                .unwrap_err()
                .field_suffix,
            "action.delay_ms"
        );
        basic.action = BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 };
        basic.delivery = ron::de::from_str("Projectile(speed_units_per_ms:6000)").unwrap();
        assert_eq!(
            basic.try_validate_runtime_contract().unwrap_err().cause,
            "DamageTileAfterDelay requires Instant delivery"
        );
        assert!(ron::de::from_str::<BasicAttackAction>(
            "DamageTileAfterDelay(delay_ms:400,follow_target:true)"
        )
        .is_err());
    }
    #[test]
    fn basic_delayed_tile_release_wait_impact_once_and_action_settlement() {
        let mut f = fixture();
        start(&mut f, 10);
        run(&mut f, 89);
        assert!(released(&f).is_empty());
        run(&mut f, 90);
        assert_eq!(hp(&f, f.primary), 1000);
        assert_eq!(hp(&f, f.other), 1000);
        let releases = released(&f);
        assert_eq!(releases.len(), 1);
        assert_eq!(
            (releases[0].0, releases[0].2, releases[0].3),
            (90, Position::new(3, 1), 490)
        );
        let action = f
            .core
            .event_queue
            .iter()
            .find_map(|e| match e.as_ref() {
                BattleEvent::BasicAttackTileImpact {
                    source_snapshot, ..
                } => source_snapshot.fact_source.action.clone(),
                _ => None,
            })
            .expect("native basic action finite child");
        let waiting = f.core.query_action(&action).unwrap();
        assert_eq!(waiting.released_at_ms, Some(90));
        assert_eq!(waiting.effects_settled_at_ms, None);
        assert_eq!(waiting.outstanding_deliveries, 1);
        run(&mut f, 489);
        assert_eq!(hp(&f, f.primary), 1000);
        run(&mut f, 490);
        assert_eq!(hp(&f, f.primary), 900);
        assert_eq!(hp(&f, f.other), 900);
        assert_eq!(
            impacts(&f),
            vec![(490, releases[0].1, Position::new(3, 1), 2)]
        );
        let settled = f.core.runtime_fact_view().latest_settled_action().unwrap();
        assert_eq!(settled.ordinal, action.ordinal());
        assert_eq!(settled.effects_settled_at_ms, Some(490));
        assert_eq!(settled.facts.damage_strikes, 2);
        run(&mut f, 600);
        assert_eq!(hp(&f, f.primary), 900);
    }
    #[test]
    fn basic_delayed_tile_impact_occupants_air_friend_and_empty_tile() {
        for (leave, enter, air, friendly, allowed, expected) in [
            (true, true, false, false, true, 900),
            (true, true, true, false, true, 900),
            (true, true, true, false, false, 1000),
            (true, true, false, true, true, 1000),
            (true, false, false, false, true, 1000),
        ] {
            let mut f = fixture();
            place(&mut f.core, f.other, 5, 3);
            f.core
                .units
                .get_mut(&f.actor)
                .unwrap()
                .basic_attack
                .air_capable = allowed;
            start(&mut f, 10);
            run(&mut f, 90);
            if leave {
                place(&mut f.core, f.primary, 5, 1);
            }
            if enter {
                place(&mut f.core, f.other, 3, 1);
            }
            if air {
                f.core.units.get_mut(&f.other).unwrap().mobility_kind = MobilityKind::Airborne;
            }
            if friendly {
                f.core.units.get_mut(&f.other).unwrap().owner = Side::Player;
            }
            run(&mut f, 490);
            assert_eq!(hp(&f, f.primary), 1000, "original target is not followed");
            assert_eq!(hp(&f, f.other), expected);
            assert_eq!(impacts(&f)[0].3, u32::from(expected == 900));
            assert_eq!(f.core.runtime_fact_usage().outstanding_deliveries, 0);
        }
    }
    #[test]
    fn basic_delayed_tile_release_and_impact_sample_moving_centers() {
        use crate::game::battle::core::movement::types::{ActiveMovementSegment, WorldVec2};
        let mut f = fixture();
        place(&mut f.core, f.other, 5, 3);
        f.core.active_movement_segments.insert(
            f.primary,
            ActiveMovementSegment {
                start: WorldVec2::from_tile_center(Position::new(3, 1)),
                target: WorldVec2::from_tile_center(Position::new(4, 1)),
                started_at_ms: 10,
                ends_at_ms: 90,
            },
        );
        assert!(f.core.resolve_basic_attack(f.actor, f.primary, 90));
        assert_eq!(released(&f)[0].2, Position::new(4, 1));
        f.core.active_movement_segments.remove(&f.primary);
        place(&mut f.core, f.primary, 6, 1);
        // Install a segment after the last ordinary movement tick so the
        // query exercises sampled geometry, without the engine replacing it.
        run(&mut f, 489);
        f.core.active_movement_segments.insert(
            f.other,
            ActiveMovementSegment {
                start: WorldVec2::from_tile_center(Position::new(5, 3)),
                target: WorldVec2::from_tile_center(Position::new(4, 1)),
                started_at_ms: 90,
                ends_at_ms: 490,
            },
        );
        run(&mut f, 490);
        assert_eq!(hp(&f, f.primary), 1000);
        assert_eq!(hp(&f, f.other), 900);
    }
    #[test]
    fn basic_delayed_tile_release_rejects_primary_that_left_current_range() {
        use crate::game::battle::core::movement::types::{ActiveMovementSegment, WorldVec2};
        let mut f = fixture();
        f.core.units.get_mut(&f.actor).unwrap().resonance_current = 0;
        f.core.active_movement_segments.insert(
            f.primary,
            ActiveMovementSegment {
                start: WorldVec2::from_tile_center(Position::new(3, 1)),
                target: WorldVec2::from_tile_center(Position::new(7, 3)),
                started_at_ms: 10,
                ends_at_ms: 90,
            },
        );
        assert!(!f.core.resolve_basic_attack(f.actor, f.primary, 90));
        assert!(released(&f).is_empty());
        assert_eq!(f.core.units[&f.actor].resonance_current, 0);
    }
    #[test]
    fn basic_delayed_tile_withdraw_before_release_cancels_after_release_preserves() {
        for withdraw in [20, 100] {
            let mut f = fixture();
            start(&mut f, 10);
            run(&mut f, withdraw);
            f.core
                .apply_live_command(BattleLiveCommand::WithdrawUnit {
                    unit_id: f.actor,
                    time_ms: withdraw,
                })
                .unwrap();
            run(&mut f, 490);
            assert_eq!(hp(&f, f.primary), if withdraw == 20 { 1000 } else { 900 });
            assert_eq!(hp(&f, f.other), if withdraw == 20 { 1000 } else { 900 });
            assert_eq!(impacts(&f).len(), usize::from(withdraw == 100));
        }
    }
    #[test]
    fn basic_delayed_tile_overlay_replaces_then_restores_native_action() {
        let mut f = fixture();
        let overlay=ron::de::from_str(r#"BasicAttackOverlay(duration_ms:1000,next_attack_only:true,
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
        assert!(released(&f).is_empty());
        assert_eq!(hp(&f, f.primary), 900);
        assert!(matches!(
            f.core.units[&f.actor].basic_attack.action,
            BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 }
        ));
        start(&mut f, 500);
        run(&mut f, 580);
        assert_eq!(hp(&f, f.other), 1000);
        run(&mut f, 980);
        assert_eq!(hp(&f, f.other), 900);
    }
    #[test]
    fn basic_delayed_tile_cancellation_restores_release_and_impact_then_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for (site, before_time, until) in [
            ("basic_delayed_tile_released", 89, 90),
            ("basic_delayed_tile_impacted", 489, 490),
        ] {
            let mut f = fixture();
            start(&mut f, 10);
            run(&mut f, before_time);
            let before = (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.state),
            );
            let token = CancellationToken::default();
            let result = control::with_test_cancellation(site, 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || f.core.step_battle_execution_until(&mut f.state, until),
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
            run(&mut f, 490);
            assert_eq!(hp(&f, f.primary), 900);
            assert_eq!(impacts(&f).len(), 1);
        }
    }
    #[test]
    fn basic_delayed_tile_impact_retained_refusal_restores_and_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = fixture();
        start(&mut f, 10);
        run(&mut f, 489);
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 490)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "basic_delayed_tile_targets")
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
            .step_battle_execution_until(&mut f.state, 490)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "basic_delayed_tile_targets",
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
        run(&mut f, 490);
        assert_eq!(hp(&f, f.primary), 900);
        assert_eq!(hp(&f, f.other), 900);
    }

    #[test]
    fn basic_delayed_tile_authored_delays_clone_and_observer_preserve_results() {
        for delay in [1, 250, 900] {
            let mut f = fixture_delay(delay);
            start(&mut f, 10);
            run(&mut f, 90);
            let mut observed = f.core.clone();
            let mut observed_state = f.state.clone();
            observed.enable_authoring_observation();
            let at = 90 + u64::from(delay);
            run(&mut f, at - 1);
            assert_eq!(hp(&f, f.primary), 1000);
            run(&mut f, at);
            observed
                .step_battle_execution_until(&mut observed_state, at)
                .unwrap();
            assert_eq!(hp(&f, f.primary), 900);
            assert_eq!(hp(&f, f.other), 900);
            assert_eq!(observed.units[&f.primary].stats.current_health, 900);
            assert_eq!(observed.units[&f.other].stats.current_health, 900);
            assert_eq!(
                serde_json::to_value(&f.core.event_log).unwrap(),
                serde_json::to_value(&observed.event_log).unwrap()
            );
            assert_eq!(
                serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
                serde_json::to_value(observed.runtime_fact_view()).unwrap()
            );
            assert_eq!(
                f.core.event_queue_debug_snapshot(),
                observed.event_queue_debug_snapshot()
            );
        }
    }
    #[test]
    fn basic_delayed_tile_shared_profile_wearers_keep_separate_tiles_and_actions() {
        let mut f = fixture();
        // Same profile, disjoint ranges: native target selection may override
        // an AttackStart hint when more than one enemy is eligible.
        place(&mut f.core, f.ally, 6, 3);
        place(&mut f.core, f.other, 7, 3);
        f.core.units.get_mut(&f.ally).unwrap().stats.attack = 100;
        f.core.units.get_mut(&f.ally).unwrap().next_basic_attack_ms = 10;
        start(&mut f, 10);
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: 10,
                    attacker_instance_id: f.ally,
                    target_instance_id: Some(f.other),
                    schedule_next: true,
                    cause: Default::default(),
                },
                10,
            )
            .unwrap();
        run(&mut f, 90);
        let launches: Vec<_> = f
            .core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match entry.event {
                BattleLogEvent::BasicAttackTileReleased {
                    release_id,
                    attacker_instance_id,
                    tile,
                    ..
                } => Some((release_id, attacker_instance_id, tile)),
                _ => None,
            })
            .collect();
        assert_eq!(launches.len(), 2);
        assert_ne!(launches[0].0, launches[1].0);
        assert_ne!(launches[0].1, launches[1].1);
        assert_ne!(launches[0].2, launches[1].2);
        run(&mut f, 490);
        assert_eq!(hp(&f, f.primary), 900);
        assert_eq!(hp(&f, f.other), 900);
        assert_eq!(f.core.runtime_fact_usage().outstanding_deliveries, 0);
    }
    #[test]
    fn basic_delayed_tile_terminal_clears_pending_impact() {
        let mut f = fixture();
        std::sync::Arc::make_mut(&mut f.core.scenario).terminal_outcomes =
            BattleScenario::fixture_survival_outcomes(200);
        start(&mut f, 10);
        run(&mut f, 490);
        assert_eq!(released(&f).len(), 1);
        assert!(f.state.is_finished());
        let before = (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            format!("{:?}", f.state),
        );
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("basic_tile_terminal_cleared", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.finalize_battle_execution(&mut f.state).map(|_| ()),
            )
        });
        assert!(result.is_err() && token.is_cancelled(), "{result:?}");
        assert_eq!(
            before,
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.state),
            )
        );
        f.core.finalize_battle_execution(&mut f.state).unwrap();
        assert!(impacts(&f).is_empty());
        assert!(!f
            .core
            .event_queue
            .iter()
            .any(|e| matches!(e.as_ref(), BattleEvent::BasicAttackTileImpact { .. })));
        assert_eq!(hp(&f, f.primary), 1000);
        assert_eq!(f.core.runtime_fact_usage().outstanding_deliveries, 0);
    }
}
