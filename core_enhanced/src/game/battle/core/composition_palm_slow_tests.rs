mod composition_palm_slow {
    use super::*;
    use crate::game::{
        ability::sustained::{HitStatusKind, HitStatusProfile, SustainedEffectDef},
        battle::{
            damage::{DamageSource, DamageType},
            types::{MobilityKind, UnitTargetTrait},
        },
        data::abnormality_data::BasicAttackAction,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/palm_slow.skill.yaml"
    ));
    fn profile() -> crate::game::data::equipment_data::WeaponCombatProfile {
        let data: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        data.get_by_id("placeholder_clouded_monk_weapon")
            .unwrap()
            .weapon_profile
            .clone()
            .unwrap()
    }
    fn setup(percent: u32) -> Fixture {
        let source = SOURCE
            .replace("placeholder_clouded_monk_weapon", "r11_weapon")
            .replace("percent: 20", &format!("percent: {percent}"));
        let mut f = fixture(0, false, |doc| *doc = parse_yaml(&source).unwrap());
        let profile = profile();
        for u in f.core.units.values_mut() {
            u.stats.max_health = 10000;
            u.stats.current_health = 10000;
            u.stats.magic_resist = 0;
            u.stats.move_speed_units_per_ms = 1000;
        }
        let a = f.core.units.get_mut(&f.a).unwrap();
        a.stats.attack = 100;
        a.basic_attack.action = profile.action;
        a.basic_attack.damage_type = profile.damage_type;
        a.basic_attack.delivery = profile.delivery;
        a.basic_attack.windup_ms = profile.windup_ms;
        a.basic_attack.air_capable = profile.air_capable;
        a.basic_attack.range_policy =
            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        f.core.enable_authoring_observation();
        f
    }
    fn run(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn hit(f: &mut Fixture, now: u64) {
        attack(f, now, true);
        run(f, now + 400);
    }
    fn place(f: &mut Fixture, who: UnitInstanceId, x: i32, y: i32) {
        super::super::compiled_action_fixture::place(&mut f.core, who, x, y);
    }
    fn records(core: &BattleCore) -> Vec<crate::game::ability::sustained::SustainedEffectView> {
        core.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|r| {
                r.hit_status
                    .as_ref()
                    .is_some_and(|p| matches!(p.kind, HitStatusKind::Slow { .. }))
            })
            .collect()
    }
    fn speed(f: &Fixture, who: UnitInstanceId) -> u32 {
        f.core.units[&who].stats.move_speed_units_per_ms
    }
    fn state(f: &Fixture) -> impl PartialEq + std::fmt::Debug {
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot(),
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
            format!("{:?}", f.state),
        )
    }
    fn captured(f: &mut Fixture, now: u64) -> crate::game::battle::damage::DamageSourceSnapshot {
        attack(f, now, true);
        f.core
            .event_queue
            .iter()
            .find_map(|e| match e.as_ref() {
                BattleEvent::AttackResolve {
                    attacker_instance_id,
                    source_snapshot,
                    ..
                } if *attacker_instance_id == f.a => Some(source_snapshot.clone()),
                _ => None,
            })
            .unwrap()
    }
    fn committed(
        f: &mut Fixture,
        source: crate::game::battle::damage::DamageSourceSnapshot,
        now: u64,
    ) {
        use crate::game::battle::core::damage_runtime::{
            CommittedDamageInput, CommittedDamagePolicy,
        };
        f.core
            .runtime_transaction(|c| {
                c.execute_committed_damage(CommittedDamageInput {
                    target_id: f.enemy,
                    source_snapshot: source,
                    committed_at_ms: now,
                    policy: CommittedDamagePolicy::GeneralCommand,
                })
                .ok_or(GameError::InvalidAction)
                .map(|_| ())
            })
            .unwrap();
    }
    #[test]
    fn palm_slow_schema_catalog_and_roundtrip() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let reopened = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(
            reopened.equipment_attacks()[0]
                .program
                .on_hit_status
                .as_ref()
                .unwrap()
                .profile
                .kind,
            HitStatusKind::Slow { percent: 20 }
        );
        let p = profile();
        p.try_validate_runtime_contract("palm").unwrap();
        assert!(matches!(
            p.action,
            BasicAttackAction::DamageArea {
                secondary_percent: 100,
                ..
            }
        ));
        assert_eq!(p.damage_type, DamageType::Physical);
        assert_eq!(p.block_capacity, 0);
        assert!(!p.air_capable);
        assert_eq!(
            p.weapon_archetype,
            crate::game::data::equipment_data::WeaponArchetype::Staff
        );
        for (from, to) in [
            ("percent: 20", "percent: 101"),
            ("duration_ms: 3000", "duration_ms: 0"),
            ("percent: 20", "percent: 20, typo: 1"),
        ] {
            assert!(
                crate::authoring::composition::compile_yaml(&SOURCE.replace(from, to)).is_err(),
                "{to}"
            );
        }
    }
    #[test]
    fn palm_slow_area_once_per_ground_target_and_movement_only_with_authored_rate() {
        for (percent, expected) in [(20, 800), (35, 650)] {
            let mut f = setup(percent);
            let target = f.secondary;
            place(&mut f, target, 3, 1); // Two units on one occupied tile, two distinct strikes.
            let prior_interval = f.core.units[&f.enemy].stats.attack_interval_ms;
            hit(&mut f, 10);
            assert_eq!(records(&f.core).len(), 2);
            for target in [f.enemy, f.secondary] {
                assert_eq!(f.core.units[&target].stats.current_health, 9900);
                assert_eq!(speed(&f, target), expected);
                assert_eq!(
                    f.core.basic_interval_with_equipment(target, 1000, 0, 410),
                    1000
                );
                assert_eq!(
                    f.core.units[&target].stats.attack_interval_ms,
                    prior_interval
                );
                assert!(!f
                    .core
                    .unit_control_state(target, 410)
                    .unwrap()
                    .hard_cc_active());
            }
            assert_eq!(f.core.units[&f.a].stats.current_health, 10000);
            let w = f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_sourced_sustained
                .as_ref()
                .unwrap();
            assert!(w.equipment.is_some());
            assert!(w
                .authored
                .as_ref()
                .unwrap()
                .field_path
                .contains("on_hit_status"));
        }
        let mut f = setup(20);
        let target = f.secondary;
        place(&mut f, target, 3, 1);
        f.core.units.get_mut(&target).unwrap().mobility_kind = MobilityKind::Airborne;
        hit(&mut f, 10);
        assert_eq!(f.core.units[&target].stats.current_health, 10000);
        assert_eq!(speed(&f, target), 1000);
        assert_eq!(records(&f.core).len(), 1);
    }
    #[test]
    fn palm_slow_refresh_removes_old_expiry_and_restores_movement_on_deadline() {
        let mut f = setup(20);
        hit(&mut f, 10);
        let first = records(&f.core)[0].activation;
        hit(&mut f, 600);
        let r = records(&f.core);
        assert_eq!(r.len(), 1);
        assert_ne!(r[0].activation, first);
        assert_eq!(r[0].expires_at_ms, 4000);
        assert!(!f.core.event_queue.iter().any(|e|matches!(e.as_ref(),BattleEvent::SustainedExpiry{activation,..} if *activation==first)));
        run(&mut f, 3410);
        assert_eq!(speed(&f, f.enemy), 800);
        run(&mut f, 3999);
        assert_eq!(speed(&f, f.enemy), 800);
        run(&mut f, 4000);
        assert_eq!(speed(&f, f.enemy), 1000);
        assert!(records(&f.core).is_empty());
    }
    #[test]
    fn palm_slow_other_source_strongest_fallback_and_fragment_lifetime_preserved() {
        let mut f = setup(20);
        hit(&mut f, 10);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.b,
                    f.enemy,
                    SustainedEffectDef::HitStatus {
                        duration_ms: 1000,
                        profile: HitStatusProfile {
                            definition: "other-slow".into(),
                            kind: HitStatusKind::Slow { percent: 35 },
                            modifier_source: None,
                            vulnerability: None,
                        },
                    },
                    500,
                )
            })
            .unwrap();
        assert_eq!(speed(&f, f.enemy), 650);
        hit(&mut f, 600);
        assert_eq!(records(&f.core).len(), 2);
        assert_eq!(speed(&f, f.enemy), 650);
        run(&mut f, 1500);
        assert_eq!(speed(&f, f.enemy), 800);
        run(&mut f, 4000);
        assert_eq!(speed(&f, f.enemy), 1000);
        // Unowned/fragment-style repeated contributions retain their original contract.
        for now in [4100, 4200] {
            f.core
                .runtime_transaction(|c| {
                    c.begin_sustained(
                        f.b,
                        f.enemy,
                        SustainedEffectDef::HitStatus {
                            duration_ms: 1000,
                            profile: HitStatusProfile {
                                definition: "fragment-style".into(),
                                kind: HitStatusKind::Slow { percent: 20 },
                                modifier_source: None,
                                vulnerability: None,
                            },
                        },
                        now,
                    )
                })
                .unwrap();
        }
        assert_eq!(records(&f.core).len(), 2);
    }
    #[test]
    fn palm_slow_explicit_refresh_preserves_existing_equipment_independent_deadlines() {
        let independent = SOURCE
            .replace("          refresh_movement_slow: true\n", "")
            .replace("placeholder_clouded_monk_weapon", "r11_weapon");
        let mut f = fixture(0, false, |doc| *doc = parse_yaml(&independent).unwrap());
        f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery =
            crate::game::ability::DeliveryDef::Instant;
        f.core.units.get_mut(&f.a).unwrap().basic_attack.windup_ms = 400;
        f.core
            .units
            .get_mut(&f.enemy)
            .unwrap()
            .stats
            .move_speed_units_per_ms = 1000;
        hit(&mut f, 10);
        hit(&mut f, 600);
        let r = records(&f.core);
        assert_eq!(r.len(), 2);
        assert!(r.iter().any(|r| r.expires_at_ms == 3410));
        assert!(r.iter().any(|r| r.expires_at_ms == 4000));
        assert_eq!(speed(&f, f.enemy), 800);
        run(&mut f, 3410);
        assert_eq!(speed(&f, f.enemy), 800);
        run(&mut f, 4000);
        assert_eq!(speed(&f, f.enemy), 1000);
        let invalid = SOURCE.replace("Slow: { percent: 20 }", "BasicAttackSlow: { percent: 20 }");
        let error = crate::authoring::composition::compile_yaml(&invalid).unwrap_err();
        assert!(format!("{error:?}").contains("refresh_movement_slow"));
    }
    #[test]
    fn palm_slow_current_target_center_and_late_secondary_entry() {
        let mut f = setup(20);
        attack(&mut f, 10, true);
        let target = f.enemy;
        place(&mut f, target, 4, 1);
        let target = f.secondary;
        place(&mut f, target, 5, 2);
        run(&mut f, 410);
        assert_eq!(f.core.units[&f.secondary].stats.current_health, 9900);
        assert_eq!(speed(&f, f.secondary), 800);
        let mut f = setup(20);
        let target = f.secondary;
        place(&mut f, target, 3, 2);
        attack(&mut f, 10, true);
        place(&mut f, target, 7, 3);
        run(&mut f, 410);
        assert_eq!(f.core.units[&target].stats.current_health, 10000);
        assert_eq!(speed(&f, target), 1000);
    }
    #[test]
    fn palm_slow_immunity_shield_zero_damage_and_direct_basic_gate() {
        let mut f = setup(20);
        f.core
            .units
            .get_mut(&f.enemy)
            .unwrap()
            .target_traits
            .push(UnitTargetTrait::SlowImmune);
        hit(&mut f, 10);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
        assert_eq!(speed(&f, f.enemy), 1000);
        assert!(records(&f.core).is_empty());
        let mut f = setup(20);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Shield {
                        duration_ms: 2000,
                        amount: 1000,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        hit(&mut f, 10);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
        assert_eq!(speed(&f, f.enemy), 800);
        let mut f = setup(20);
        let mut snapshot = captured(&mut f, 10);
        snapshot.base_damage = 0;
        committed(&mut f, snapshot, 11);
        assert_eq!(speed(&f, f.enemy), 800);
        for kind in [
            DamageSource::Ability,
            DamageSource::BuffTick,
            DamageSource::Environment,
        ] {
            let mut f = setup(20);
            let mut snapshot = captured(&mut f, 10);
            snapshot.source = kind;
            committed(&mut f, snapshot, 11);
            assert!(records(&f.core).is_empty(), "{kind:?}");
        }
        let mut f = setup(20);
        let mut snapshot = captured(&mut f, 10);
        snapshot.fact_source.reaction = true;
        committed(&mut f, snapshot, 11);
        assert!(records(&f.core).is_empty());
    }
    #[test]
    fn palm_slow_withdraw_before_hit_cancels_and_landed_status_keeps_finite_lifetime() {
        for after_hit in [false, true] {
            let mut f = setup(20);
            attack(&mut f, 10, true);
            run(&mut f, if after_hit { 410 } else { 100 });
            f.core
                .apply_live_command(BattleLiveCommand::WithdrawUnit {
                    unit_id: f.a,
                    time_ms: if after_hit { 411 } else { 101 },
                })
                .unwrap();
            run(&mut f, 500);
            assert_eq!(
                f.core.units[&f.enemy].stats.current_health,
                if after_hit { 9900 } else { 10000 }
            );
            assert_eq!(speed(&f, f.enemy), if after_hit { 800 } else { 1000 });
            run(&mut f, 3410);
            assert_eq!(speed(&f, f.enemy), 1000);
            assert!(records(&f.core).is_empty());
        }
    }
    #[test]
    fn palm_slow_refresh_cancel_rolls_back_damage_speed_and_expiry_then_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(20);
        hit(&mut f, 10);
        attack(&mut f, 600, true);
        run(&mut f, 999);
        let before = state(&f);
        let cancel = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_attack_hit_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &cancel,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 1000),
            )
        });
        assert!(result.is_err() && cancel.is_cancelled());
        assert_eq!(before, state(&f));
        run(&mut f, 1000);
        assert_eq!(records(&f.core).len(), 1);
        assert_eq!(records(&f.core)[0].expires_at_ms, 4000);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9800);
        assert_eq!(speed(&f, f.enemy), 800);
    }
    #[test]
    fn palm_slow_retained_refusal_and_observer_free_clone_preserve_result() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(20);
        attack(&mut f, 10, true);
        run(&mut f, 409);
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        let mut plain_state = f.state.clone();
        let mut probe = f.core.clone();
        let mut probe_state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut probe_state, 410)
        });
        result.unwrap();
        let required = admissions.iter().map(|(_, _, n)| *n).max().unwrap();
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = state(&f);
        assert!(f
            .core
            .step_battle_execution_until(&mut f.state, 410)
            .is_err());
        assert_eq!(before, state(&f));
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        run(&mut f, 410);
        plain
            .step_battle_execution_until(&mut plain_state, 410)
            .unwrap();
        assert_eq!(plain.units[&f.enemy].stats.move_speed_units_per_ms, 800);
        assert_eq!(
            serde_json::to_value(&f.core.event_log).unwrap(),
            serde_json::to_value(&plain.event_log).unwrap()
        );
        assert_eq!(
            f.core.event_queue_debug_snapshot(),
            plain.event_queue_debug_snapshot()
        );
    }

    #[test]
    fn palm_slow_distinct_copies_keep_deadlines_and_compose_with_general_control() {
        use crate::game::ability::sustained::ControlKind;
        let mut f = setup(20);
        hit(&mut f, 10);
        let first_copy = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_sourced_sustained
            .as_ref()
            .unwrap()
            .equipment
            .unwrap();
        let old = f.a;
        place(&mut f, old, 7, 3);
        f.a = f.b;
        let actor = f.a;
        place(&mut f, actor, 2, 1);
        let a = f.core.units.get_mut(&actor).unwrap();
        a.basic_attack.delivery = crate::game::ability::DeliveryDef::Instant;
        a.basic_attack.windup_ms = 400;
        a.basic_attack.range_policy =
            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        hit(&mut f, 600);
        let second_copy = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_sourced_sustained
            .as_ref()
            .unwrap()
            .equipment
            .unwrap();
        assert_ne!(
            first_copy.persistent_copy_uuid,
            second_copy.persistent_copy_uuid
        );
        assert_eq!(records(&f.core).len(), 2);
        assert_eq!(speed(&f, f.enemy), 800);
        // General stat/control penalties retain their established additive layer.
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    actor,
                    f.enemy,
                    SustainedEffectDef::Control {
                        duration_ms: 500,
                        kind: ControlKind::Slow { percent: 30 },
                    },
                    1100,
                )
            })
            .unwrap();
        assert_eq!(speed(&f, f.enemy), 500);
        run(&mut f, 1600);
        assert_eq!(speed(&f, f.enemy), 800);
        run(&mut f, 3410);
        assert_eq!(records(&f.core).len(), 1);
        assert_eq!(speed(&f, f.enemy), 800);
        run(&mut f, 4000);
        assert!(records(&f.core).is_empty());
        assert_eq!(speed(&f, f.enemy), 1000);
    }
}
