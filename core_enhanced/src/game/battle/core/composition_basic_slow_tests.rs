mod composition_basic_slow {
    use super::*;
    use crate::game::{
        ability::sustained::{HitStatusKind, HitStatusProfile, SustainedEffectDef},
        battle::{damage::DamageType, types::UnitTargetTrait},
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/basic_slow.skill.yaml"
    ));
    fn setup(percent: u32) -> Fixture {
        let source = SOURCE
            .replace("black_swan_umbrella", "r11_weapon")
            .replace("percent: 20", &format!("percent: {percent}"));
        let mut f = fixture(0, false, |doc| *doc = parse_yaml(&source).unwrap());
        for u in f.core.units.values_mut() {
            u.stats.max_health = 10000;
            u.stats.current_health = 10000;
            u.stats.magic_resist = 0;
        }
        let u = f.core.units.get_mut(&f.a).unwrap();
        u.stats.attack = 100;
        u.basic_attack.delivery = crate::game::ability::DeliveryDef::Instant;
        u.basic_attack.windup_ms = 80;
        u.basic_attack.damage_type = DamageType::Magic;
        u.basic_attack.range_policy =
            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        f.core.enable_authoring_observation();
        f
    }
    fn hit(f: &mut Fixture, now: u64) {
        attack(f, now, true);
        f.core
            .step_battle_execution_until(&mut f.state, now + 80)
            .unwrap();
    }
    fn slow(
        core: &mut BattleCore,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        definition: &str,
        percent: u32,
        duration_ms: u32,
        now: u64,
    ) {
        core.runtime_transaction(|c| {
            c.begin_sustained(
                caster,
                target,
                SustainedEffectDef::HitStatus {
                    duration_ms,
                    profile: HitStatusProfile {
                        definition: definition.into(),
                        kind: HitStatusKind::BasicAttackSlow { percent },
                        modifier_source: None,
                        vulnerability: None,
                    },
                },
                now,
            )
        })
        .unwrap();
    }
    fn records(core: &BattleCore) -> Vec<crate::game::ability::sustained::SustainedEffectView> {
        core.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|s| {
                s.hit_status
                    .as_ref()
                    .is_some_and(|p| p.kind.basic_speed_reduction().is_some())
            })
            .collect()
    }
    fn interval(core: &BattleCore, who: UnitInstanceId, now: u64) -> u64 {
        core.basic_interval_with_equipment(who, 1000, 0, now)
    }
    fn state(core: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn basic_slow_schema_roundtrip_bounds_and_actual_source() {
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
            HitStatusKind::BasicAttackSlow { percent: 20 }
        );
        for (from, to) in [
            ("percent: 20", "percent: 0"),
            ("percent: 20", "percent: 100"),
            ("percent: 20", "percent: 20, typo: 1"),
            ("duration_ms: 2000", "duration_ms: 0"),
        ] {
            assert!(
                crate::authoring::composition::compile_yaml(&SOURCE.replace(from, to)).is_err(),
                "{to}"
            );
        }
        let mut f = setup(20);
        hit(&mut f, 10);
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
        assert!(matches!(
            w.change,
            crate::authoring::ApplicationChange::Sustained {
                basic_speed_reduction_percent: Some(20),
                ended: None,
                expires_at_ms: 2090,
                ..
            }
        ));
    }
    #[test]
    fn basic_slow_same_source_input_changes_rate_without_movement_or_hard_control() {
        for (percent, wanted) in [(20, 1250), (35, 1539)] {
            let mut f = setup(percent);
            let before = f.core.units[&f.enemy].stats;
            hit(&mut f, 10);
            assert_eq!(interval(&f.core, f.enemy, 90), wanted);
            assert_eq!(
                f.core.units[&f.enemy].stats.move_speed_units_per_ms,
                before.move_speed_units_per_ms
            );
            assert_eq!(
                f.core.units[&f.enemy].stats.attack_interval_ms,
                before.attack_interval_ms
            );
            assert!(!f.core.target_movement_impaired(f.enemy, 90).unwrap());
            assert!(!f
                .core
                .unit_control_state(f.enemy, 90)
                .unwrap()
                .hard_cc_active());
            assert_eq!(f.core.basic_speed_reduction(f.secondary, 90).unwrap(), 0);
            assert_eq!(
                f.core.basic_speed_reduction(f.enemy, 2089).unwrap(),
                percent
            );
            assert_eq!(f.core.basic_speed_reduction(f.enemy, 2090).unwrap(), 0);
        }
    }
    #[test]
    fn basic_slow_refresh_replaces_one_source_and_one_expiry() {
        let mut f = setup(20);
        hit(&mut f, 10);
        let first = records(&f.core)[0].activation;
        hit(&mut f, 200);
        let r = records(&f.core);
        assert_eq!(r.len(), 1);
        assert_ne!(r[0].activation, first);
        assert_eq!(r[0].expires_at_ms, 2280);
        assert_eq!(interval(&f.core, f.enemy, 2090), 1250);
        assert_eq!(f.core.event_queue.iter().filter(|e|matches!(e.as_ref(),BattleEvent::SustainedExpiry{activation,..} if *activation==first)).count(),0);
        f.core
            .step_battle_execution_until(&mut f.state, 2280)
            .unwrap();
        assert!(records(&f.core).is_empty());
        assert_eq!(interval(&f.core, f.enemy, 2280), 1000);
        slow(&mut f.core, f.a, f.enemy, "terminal", 20, 20000, 2300);
        f.core
            .step_battle_execution_until(&mut f.state, 8001)
            .unwrap();
        f.core.finalize_battle_execution(&mut f.state).unwrap();
        assert!(records(&f.core).is_empty());
    }
    #[test]
    fn basic_slow_strongest_definition_fallback_other_definitions_cap_and_copy_isolation() {
        let mut f = setup(20);
        slow(&mut f.core, f.a, f.enemy, "shared", 20, 2000, 0);
        slow(&mut f.core, f.b, f.enemy, "shared", 35, 1000, 0);
        assert_eq!(records(&f.core).len(), 2);
        assert_eq!(interval(&f.core, f.enemy, 999), 1539);
        assert_eq!(interval(&f.core, f.enemy, 1000), 1250);
        slow(&mut f.core, f.a, f.enemy, "other", 90, 1000, 1000);
        assert_eq!(f.core.basic_speed_reduction(f.enemy, 1000).unwrap(), 99);
        assert_eq!(interval(&f.core, f.enemy, 1000), 100000);
        assert_eq!(interval(&f.core, f.secondary, 1000), 1000);
        assert_eq!(interval(&f.core, f.enemy, 2000), 1000);
    }
    #[test]
    fn basic_slow_regular_next_start_uses_rate_and_keeps_committed_schedule() {
        let mut f = setup(20);
        slow(&mut f.core, f.a, f.enemy, "cycle", 20, 2000, 0);
        let e = f.core.units.get_mut(&f.enemy).unwrap();
        e.stats.attack_interval_ms = 1000;
        e.next_basic_attack_ms = 100;
        e.basic_attack.range_policy =
            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        // Match the existing ranged enemy fixture: this cadence test must not
        // depend on a melee block relation or a pending reposition.
        e.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
        e.basic_attack.defense_tile_range = None;
        e.basic_attack.ranged_reposition_ms = 0;
        e.ranged_reposition_until_ms = 0;
        e.basic_attack.windup_ms = 80;
        let target_query = f.core.basic_action_snapshot(f.enemy, true).unwrap();
        assert!(
            target_query.selected_target_id.is_some(),
            "{target_query:?}"
        );
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: 100,
                    attacker_instance_id: f.enemy,
                    target_instance_id: Some(f.a),
                    schedule_next: true,
                    cause: Default::default(),
                },
                100,
            )
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].next_basic_attack_ms, 1350);
        let queue = f.core.event_queue_debug_snapshot();
        slow(&mut f.core, f.a, f.enemy, "cycle", 35, 2000, 110);
        assert_eq!(f.core.units[&f.enemy].next_basic_attack_ms, 1350);
        assert_ne!(queue, f.core.event_queue_debug_snapshot()); // Only finite status expiry changed.
        let next = f.core.units[&f.enemy].next_basic_attack_ms;
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: 120,
                    attacker_instance_id: f.enemy,
                    target_instance_id: Some(f.a),
                    schedule_next: false,
                    cause: Default::default(),
                },
                120,
            )
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].next_basic_attack_ms, next);
        assert_eq!(
            f.core
                .basic_interval_with_equipment(f.enemy, 1000, 100, 120),
            770
        ); // +100 points, then -35%, ceil.
           // The infallible interval helper must not hide a latched cancellation.
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let before = state(&f.core);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("basic_slow_query", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    f.core.runtime_transaction(|c| {
                        c.process_event(
                            BattleEvent::AttackStart {
                                time_ms: 1350,
                                attacker_instance_id: f.enemy,
                                target_instance_id: Some(f.a),
                                schedule_next: true,
                                cause: Default::default(),
                            },
                            1350,
                        )
                    })
                },
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, state(&f.core));
        f.core
            .runtime_transaction(|c| {
                c.process_event(
                    BattleEvent::AttackStart {
                        time_ms: 1350,
                        attacker_instance_id: f.enemy,
                        target_instance_id: Some(f.a),
                        schedule_next: true,
                        cause: Default::default(),
                    },
                    1350,
                )
            })
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].next_basic_attack_ms, 2889);
    }
    #[test]
    fn basic_slow_control_immunities_do_not_imply_attack_rate_immunity() {
        let mut f = setup(20);
        f.core
            .units
            .get_mut(&f.enemy)
            .unwrap()
            .target_traits
            .extend([
                UnitTargetTrait::SlowImmune,
                UnitTargetTrait::RootImmune,
                UnitTargetTrait::StunImmune,
            ]);
        hit(&mut f, 10);
        assert_eq!(interval(&f.core, f.enemy, 90), 1250);
    }
    #[test]
    fn basic_slow_public_cancel_of_replacement_restores_and_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(20);
        hit(&mut f, 10);
        attack(&mut f, 100, true);
        f.core
            .step_battle_execution_until(&mut f.state, 179)
            .unwrap();
        let before = (state(&f.core), format!("{:?}", f.state));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_attack_hit_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 180),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, (state(&f.core), format!("{:?}", f.state)));
        f.core
            .step_battle_execution_until(&mut f.state, 180)
            .unwrap();
        assert_eq!(records(&f.core).len(), 1);
        assert_eq!(records(&f.core)[0].expires_at_ms, 2180);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9800);
    }
    #[test]
    fn basic_slow_clone_observation_and_retained_refusal_preserve_public_effect() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(20);
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 89)
            .unwrap();
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        let mut execution = f.state.clone();
        let mut probe = f.core.clone();
        let mut probe_state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut probe_state, 90)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "sustained_record_storage")
            .unwrap()
            .2;
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = (state(&f.core), format!("{:?}", f.state));
        assert!(f
            .core
            .step_battle_execution_until(&mut f.state, 90)
            .is_err());
        assert_eq!(before, (state(&f.core), format!("{:?}", f.state)));
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        plain
            .step_battle_execution_until(&mut execution, 90)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
        assert_eq!(interval(&plain, f.enemy, 90), 1250);
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
    fn basic_slow_released_projectile_survives_withdrawal_and_expires() {
        let mut f = setup(20);
        f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery =
            ron::de::from_str("Projectile(speed_units_per_ms:6000)").unwrap();
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 91,
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 800)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9900);
        assert_eq!(interval(&f.core, f.enemy, 800), 1250);
        let expiry = records(&f.core)[0].expires_at_ms;
        f.core
            .step_battle_execution_until(&mut f.state, expiry)
            .unwrap();
        assert!(records(&f.core).is_empty());
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
        core: &mut BattleCore,
        source: crate::game::battle::damage::DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> u64 {
        use crate::game::battle::core::damage_runtime::{
            CommittedDamageInput, CommittedDamagePolicy,
        };
        core.execute_committed_damage(CommittedDamageInput {
            target_id: target,
            source_snapshot: source,
            committed_at_ms: now,
            policy: CommittedDamagePolicy::GeneralCommand,
        })
        .unwrap()
        .actual_hp_loss
    }
    #[test]
    fn basic_slow_zero_shield_and_nonbasic_source_gate() {
        let mut f = setup(20);
        let mut source = captured(&mut f, 10);
        source.base_damage = 0;
        f.core
            .runtime_transaction(|c| {
                assert_eq!(committed(c, source, f.enemy, 11), 0);
                Ok(())
            })
            .unwrap();
        assert_eq!(interval(&f.core, f.enemy, 11), 1250);
        let mut f = setup(20);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Shield {
                        duration_ms: 1000,
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
        assert_eq!(interval(&f.core, f.enemy, 90), 1250);
        for kind in [
            crate::game::battle::damage::DamageSource::Ability,
            crate::game::battle::damage::DamageSource::BuffTick,
            crate::game::battle::damage::DamageSource::Environment,
        ] {
            let mut f = setup(20);
            let mut source = captured(&mut f, 10);
            source.source = kind;
            f.core
                .runtime_transaction(|c| {
                    committed(c, source, f.enemy, 11);
                    Ok(())
                })
                .unwrap();
            assert!(records(&f.core).is_empty(), "{kind:?}");
        }
    }
    #[test]
    fn basic_slow_parry_invulnerability_and_target_death() {
        let mut f = setup(20);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Parry {
                        duration_ms: 500,
                        window_ms: 200,
                        invulnerable_ms: 200,
                        miss_recovery_ms: 20,
                        reduction_percent: 0,
                        counter_skill: "r11_aura".into(),
                        end_on_resolution: false,
                        fragment_guard: None,
                        guard_charge: None,
                    },
                    0,
                )
            })
            .unwrap();
        hit(&mut f, 10);
        assert!(records(&f.core).is_empty());
        hit(&mut f, 110);
        assert!(records(&f.core).is_empty());
        hit(&mut f, 600);
        assert_eq!(records(&f.core).len(), 1);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
        hit(&mut f, 800);
        assert!(records(&f.core).is_empty());
    }
    #[test]
    fn basic_slow_area_magic_targets_and_two_equipped_copies() {
        use crate::game::{
            battle::tile_range::TileRangePattern, data::abnormality_data::BasicAttackAction,
        };
        let mut f = setup(20);
        super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 2);
        f.core.units.get_mut(&f.a).unwrap().basic_attack.action = BasicAttackAction::DamageArea {
            footprint: TileRangePattern {
                include_anchor_tile: true,
                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
            },
            secondary_percent: 100,
        };
        hit(&mut f, 10);
        assert_eq!(records(&f.core).len(), 2);
        for target in [f.enemy, f.secondary] {
            assert_eq!(f.core.units[&target].stats.current_health, 9900);
            assert_eq!(interval(&f.core, target, 90), 1250);
        }
        let old = f.a;
        super::super::compiled_action_fixture::place(&mut f.core, old, 7, 3);
        super::super::compiled_action_fixture::place(&mut f.core, f.b, 2, 1);
        f.a = f.b;
        f.core.units.get_mut(&f.a).unwrap().basic_attack.windup_ms = 80;
        f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery =
            crate::game::ability::DeliveryDef::Instant;
        hit(&mut f, 200);
        assert_eq!(records(&f.core).len(), 3);
        assert_eq!(interval(&f.core, f.enemy, 280), 1250);
        assert_eq!(interval(&f.core, f.enemy, 2090), 1250);
        assert_eq!(interval(&f.core, f.secondary, 2090), 1000);
    }
}
