mod multi_target_contract {
    use super::*;
    fn fixture(extra: u32) -> Fixture {
        fixture_targets(extra, 3)
    }
    fn fixture_targets(extra: u32, max_targets: u32) -> Fixture {
        let catalog: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        let mut profile = catalog
            .items
            .iter()
            .find(|w| w.id == "placeholder_laetitia_weapon")
            .unwrap()
            .weapon_profile
            .clone()
            .unwrap();
        profile.interval_ms = 100_000;
        profile.action = BasicAttackAction::DamageTargets { max_targets };
        profile.windup_ms = 80;
        profile.defense_tile_range = ron::de::from_str(
            "(include_anchor_tile:false,rows:[\"XXXXX\",\"XXXXX\",\"XX@XX\",\"XXXXX\",\"XXXXX\"])",
        )
        .unwrap();
        let mut f = fixture_profile(extra, profile);
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .stats
            .attack_interval_ms = 100_000;
        let ids: Vec<_> = f
            .core
            .units
            .values()
            .filter(|u| u.owner == Side::Opponent)
            .map(|u| u.instance_id)
            .collect();
        for id in ids {
            place(&mut f.core, id, 3, 1);
        }
        f
    }
    fn launches(f: &Fixture) -> Vec<(uuid::Uuid, UnitInstanceId)> {
        f.core
            .event_log
            .entries
            .iter()
            .filter_map(|e| match e.event {
                BattleLogEvent::BasicAttackProjectileLaunched {
                    projectile_id,
                    attacker_instance_id,
                    target_instance_id,
                    ..
                } if attacker_instance_id == f.actor => Some((projectile_id, target_instance_id)),
                _ => None,
            })
            .collect()
    }
    #[test]
    fn basic_multi_target_schema_count_delivery_and_roundtrip() {
        use crate::game::data::abnormality_data::BasicAttackDef;
        let mut basic = BasicAttackDef {
            action: BasicAttackAction::DamageTargets { max_targets: 3 },
            delivery: ron::de::from_str("Projectile(speed_units_per_ms:6000)").unwrap(),
            ..Default::default()
        };
        basic.try_validate_runtime_contract().unwrap();
        let reopened: BasicAttackDef =
            ron::de::from_str(&ron::ser::to_string(&basic).unwrap()).unwrap();
        assert!(matches!(
            reopened.action,
            BasicAttackAction::DamageTargets { max_targets: 3 }
        ));
        for n in [0, 129] {
            basic.action = BasicAttackAction::DamageTargets { max_targets: n };
            assert_eq!(
                basic
                    .try_validate_runtime_contract()
                    .unwrap_err()
                    .field_suffix,
                "action.max_targets"
            );
        }
        basic.action = BasicAttackAction::DamageTargets { max_targets: 3 };
        basic.delivery = crate::game::ability::DeliveryDef::Instant;
        assert_eq!(
            basic
                .try_validate_runtime_contract()
                .unwrap_err()
                .field_suffix,
            "action"
        );
        assert!(ron::de::from_str::<BasicAttackAction>(
            "DamageTargets(max_targets:3,retarget:true)"
        )
        .is_err());
    }
    #[test]
    fn basic_multi_target_one_two_three_four_distinct_same_tile_and_one_action() {
        for count in 1u32..=4 {
            let mut f = fixture(count.saturating_sub(2));
            if count == 1 {
                f.core.units.get_mut(&f.other).unwrap().owner = Side::Player;
            }
            start(&mut f, 10);
            run(&mut f, 90);
            let shots = launches(&f);
            assert_eq!(shots.len(), count.min(3) as usize);
            assert_eq!(
                shots
                    .iter()
                    .map(|(id, _)| id)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                shots.len()
            );
            assert_eq!(
                shots
                    .iter()
                    .map(|(_, id)| id)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                shots.len()
            );
            assert_eq!(shots[0].1, f.primary);
            assert_eq!(f.core.basic_projectile_batches.len(), 1);
            let source = f
                .core
                .projectiles
                .values()
                .next()
                .unwrap()
                .source_snapshot
                .fact_source
                .clone();
            assert!(f
                .core
                .projectiles
                .values()
                .all(|p| p.source_snapshot.fact_source.action == source.action));
            run(&mut f, 500);
            for (_, id) in shots {
                assert_eq!(hp(&f, id), 900);
            }
            assert!(f.core.basic_projectile_batches.is_empty());
            assert!(f.core.projectiles.is_empty());
        }
    }
    #[test]
    fn basic_multi_target_authored_profile_changes_limit_before_load() {
        for max_targets in [1, 2, 4] {
            let mut f = fixture_targets(2, max_targets);
            start(&mut f, 10);
            run(&mut f, 500);
            assert_eq!(launches(&f).len(), max_targets as usize);
            for (_, id) in launches(&f) {
                assert_eq!(hp(&f, id), 900);
            }
        }
    }
    #[test]
    fn basic_multi_target_release_range_air_and_no_replacement_after_target_gone() {
        let mut f = fixture(1);
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .basic_attack
            .air_capable = false;
        f.core.units.get_mut(&f.other).unwrap().mobility_kind = MobilityKind::Airborne;
        start(&mut f, 10);
        run(&mut f, 90);
        assert_eq!(launches(&f).len(), 2);
        assert!(!launches(&f).iter().any(|(_, id)| *id == f.other));
        f.core
            .units
            .get_mut(&f.primary)
            .unwrap()
            .stats
            .current_health = 0;
        run(&mut f, 500);
        assert_eq!(hp(&f, f.other), 1000);
        assert!(f.core.basic_projectile_batches.is_empty());
        assert_eq!(launches(&f).len(), 2);
        let mut f = fixture(0);
        place(&mut f.core, f.other, 7, 3);
        start(&mut f, 10);
        run(&mut f, 90);
        assert_eq!(launches(&f).len(), 1);
    }
    #[test]
    fn basic_multi_target_source_withdraw_before_and_after_release() {
        for time in [20, 100] {
            let mut f = fixture(1);
            start(&mut f, 10);
            run(&mut f, time);
            f.core
                .apply_live_command(BattleLiveCommand::WithdrawUnit {
                    unit_id: f.actor,
                    time_ms: time,
                })
                .unwrap();
            run(&mut f, 600);
            assert_eq!(hp(&f, f.primary), if time == 20 { 1000 } else { 900 });
            assert_eq!(hp(&f, f.other), if time == 20 { 1000 } else { 900 });
            assert!(f.core.basic_projectile_batches.is_empty());
        }
    }
    #[test]
    fn basic_multi_target_batch_cancellation_restores_and_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        for (site, before, until) in [
            ("basic_projectile_member_released", 89, 90),
            ("basic_projectile_batch_settled", 198, 199),
        ] {
            let mut f = fixture(1);
            start(&mut f, 10);
            run(&mut f, before);
            let prior = (
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
                prior,
                (
                    f.core.transaction_debug_snapshot(),
                    f.core.event_queue_debug_snapshot(),
                    format!("{:?}", f.state)
                )
            );
            run(&mut f, 500);
            assert_eq!(hp(&f, f.primary), 900);
            assert_eq!(launches(&f).len(), 3);
            assert!(f.core.basic_projectile_batches.is_empty());
        }
    }
    #[test]
    fn basic_multi_target_clone_observation_and_terminal_cleanup() {
        let mut f = fixture(1);
        start(&mut f, 10);
        run(&mut f, 90);
        let mut clone = f.core.clone();
        let mut state = f.state.clone();
        clone.enable_authoring_observation();
        clone.step_battle_execution_until(&mut state, 500).unwrap();
        run(&mut f, 500);
        assert_eq!(
            clone.units[&f.primary].stats.current_health,
            hp(&f, f.primary)
        );
        assert_eq!(clone.units[&f.other].stats.current_health, hp(&f, f.other));
        assert!(clone.basic_projectile_batches.is_empty());
        let mut f = fixture(1);
        start(&mut f, 10);
        run(&mut f, 90);
        f.core.clear_basic_projectile_batches(91).unwrap();
        assert!(f.core.basic_projectile_batches.is_empty());
        assert!(f.core.projectiles.is_empty());
        run(&mut f, 500);
        assert_eq!(hp(&f, f.primary), 1000);
    }
    #[test]
    fn basic_multi_target_action_stays_open_until_last_flight() {
        let mut f = fixture(0);
        place(&mut f.core, f.other, 4, 1);
        start(&mut f, 10);
        run(&mut f, 90);
        let action = f
            .core
            .projectiles
            .values()
            .next()
            .unwrap()
            .source_snapshot
            .fact_source
            .action
            .clone()
            .unwrap();
        let released = f.core.query_action(&action).unwrap();
        assert_eq!(released.released_at_ms, Some(90));
        assert_eq!(released.outstanding_deliveries, 1);
        run(&mut f, 250);
        assert_eq!(hp(&f, f.primary), 900);
        assert_eq!(hp(&f, f.other), 1000);
        assert_eq!(
            f.core.query_action(&action).unwrap().effects_settled_at_ms,
            None
        );
        run(&mut f, 500);
        let settled = f.core.runtime_fact_view().latest_settled_action().unwrap();
        assert_eq!(settled.ordinal, action.ordinal());
        assert_eq!(settled.facts.damage_strikes, 2);
        assert_eq!(settled.outstanding_deliveries, 0);
        assert!(settled.effects_settled_at_ms.is_some_and(|t| t > 250));
    }
    #[test]
    fn basic_multi_target_taunt_and_secondary_priority() {
        let mut f = fixture(2);
        f.core
            .begin_sustained(
                f.primary,
                f.actor,
                ron::de::from_str("Control(duration_ms:1000,kind:Taunt(priority:1))").unwrap(),
                0,
            )
            .unwrap();
        start(&mut f, 10);
        run(&mut f, 90);
        assert_eq!(launches(&f).len(), 1);
        assert_eq!(launches(&f)[0].1, f.primary);
        let mut f = fixture(2);
        f.core.units.get_mut(&f.actor).unwrap().basic_attack.action =
            BasicAttackAction::DamageTargets { max_targets: 2 };
        f.core
            .units
            .get_mut(&f.actor)
            .unwrap()
            .basic_attack
            .targeting_profile =
            crate::game::data::equipment_data::TargetingProfile::LowMagicResistFirst;
        for unit in f
            .core
            .units
            .values_mut()
            .filter(|u| u.owner == Side::Opponent)
        {
            unit.stats.magic_resist = 50;
        }
        f.core.units.get_mut(&f.other).unwrap().stats.magic_resist = 0;
        f.core.units.get_mut(&f.primary).unwrap().stats.magic_resist = -10;
        f.core.units.get_mut(&f.actor).unwrap().current_target = Some(f.primary);
        start(&mut f, 10);
        run(&mut f, 90);
        assert_eq!(
            launches(&f).iter().map(|(_, id)| *id).collect::<Vec<_>>(),
            vec![f.primary, f.other]
        );
    }
    #[test]
    fn basic_multi_target_secondary_position_is_sampled_at_release() {
        use crate::game::battle::core::movement::types::{ActiveMovementSegment, WorldVec2};
        let mut f = fixture(0);
        f.core.active_movement_segments.insert(
            f.other,
            ActiveMovementSegment {
                start: WorldVec2::from_tile_center(Position::new(3, 1)),
                target: WorldVec2::from_tile_center(Position::new(7, 3)),
                started_at_ms: 10,
                ends_at_ms: 90,
            },
        );
        assert!(f.core.resolve_basic_attack(f.actor, f.primary, 90));
        assert_eq!(launches(&f).len(), 1);
    }
    #[test]
    fn basic_multi_target_batch_retained_refusal_restores_and_retries() {
        use crate::game::battle::runtime_control as native;
        let mut f = fixture(1);
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
            .find(|(site, _, _)| *site == "basic_projectile_batch")
            .unwrap()
            .2;
        let original_limit = f.core.runtime_facts.limits.max_retained_bytes;
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
                    site: "basic_projectile_batch",
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
        f.core.runtime_facts.limits.max_retained_bytes = original_limit;
        run(&mut f, 500);
        assert_eq!(hp(&f, f.primary), 900);
        assert_eq!(launches(&f).len(), 3);
    }
    #[test]
    fn basic_multi_target_one_preparation_amplifies_every_member_once() {
        let mut f = fixture(1);
        f.core.begin_sustained(f.actor,f.actor,ron::de::from_str(
            "PreparedBasic(profile:(definition:\"single-use\",numerator:2,denominator:1,kill_reward:None))"
        ).unwrap(),0).unwrap();
        start(&mut f, 10);
        run(&mut f, 500);
        for (_, id) in launches(&f) {
            assert_eq!(hp(&f, id), 800);
        }
        start(&mut f, 1000);
        run(&mut f, 1500);
        assert_eq!(hp(&f, f.primary), 700);
        assert_eq!(hp(&f, f.other), 700);
    }
}
