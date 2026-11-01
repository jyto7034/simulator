mod composition_pleasure {
    use super::*;
    use crate::game::{
        ability::sustained::{HitStatusKind, HitStatusProfile, SustainedEffectDef},
        battle::{
            core::damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
            damage::{DamageSource, DamageSourceSnapshot, DamageType},
            types::UnitTargetTrait,
        },
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/pleasure.skill.yaml"
    ));
    fn setup(percent: u32) -> Fixture {
        let source = SOURCE
            .replace("porccubus_pleasure", "r11_weapon")
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
    fn hit(f: &mut Fixture, now: u64) -> u32 {
        let before = f.core.units[&f.enemy].stats.current_health;
        attack(f, now, true);
        f.core
            .step_battle_execution_until(&mut f.state, now + 80)
            .unwrap();
        before - f.core.units[&f.enemy].stats.current_health
    }
    fn records(c: &BattleCore) -> Vec<crate::game::ability::sustained::SustainedEffectView> {
        c.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|s| {
                s.hit_status
                    .as_ref()
                    .is_some_and(|p| matches!(p.kind, HitStatusKind::DamageTaken { .. }))
            })
            .collect()
    }
    fn status(
        c: &mut BattleCore,
        a: UnitInstanceId,
        t: UnitInstanceId,
        definition: &str,
        percent: u32,
        duration: u32,
        now: u64,
    ) {
        c.runtime_transaction(|c| {
            c.begin_sustained(
                a,
                t,
                SustainedEffectDef::HitStatus {
                    duration_ms: duration,
                    profile: HitStatusProfile {
                        definition: definition.into(),
                        kind: HitStatusKind::DamageTaken { percent },
                        modifier_source: None,
                        vulnerability: None,
                    },
                },
                now,
            )
        })
        .unwrap();
    }
    fn damage(c: &mut BattleCore, s: DamageSourceSnapshot, t: UnitInstanceId, now: u64) -> u64 {
        c.execute_committed_damage(CommittedDamageInput {
            target_id: t,
            source_snapshot: s,
            committed_at_ms: now,
            policy: CommittedDamagePolicy::GeneralCommand,
        })
        .unwrap()
        .actual_hp_loss
    }
    fn captured(f: &mut Fixture, now: u64) -> DamageSourceSnapshot {
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
    fn state(c: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn pleasure_schema_shared_source_roundtrip_rejects_invalid_percent_duration() {
        let a = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        assert_eq!(
            a.equipment_attacks()[0]
                .program
                .on_hit_status
                .as_ref()
                .unwrap()
                .profile
                .kind,
            HitStatusKind::DamageTaken { percent: 20 }
        );
        for (from, to) in [
            ("percent: 20", "percent: 0"),
            ("percent: 20", "percent: 2147483648"),
            ("percent: 20", "percent: 20, typo: 1"),
            ("duration_ms: 2000", "duration_ms: 0"),
        ] {
            let e =
                crate::authoring::composition::compile_yaml(&SOURCE.replace(from, to)).unwrap_err();
            assert!(
                e.source.definition.contains("AttackProgram.on_hit_status"),
                "{e:?}"
            );
        }
        let mut f = setup(20);
        assert_eq!(hit(&mut f, 10), 100);
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
                damage_taken_increase_percent: Some(20),
                ended: None,
                expires_at_ms: 2090,
                ..
            }
        ));
    }
    #[test]
    fn pleasure_first_hit_then_nonstack_refresh_expiry_and_no_periodic_damage() {
        for p in [20, 35] {
            let mut f = setup(p);
            assert_eq!(hit(&mut f, 10), 100);
            assert_eq!(hit(&mut f, 110), 100 + p);
            assert_eq!(hit(&mut f, 210), 100 + p);
            assert_eq!(records(&f.core).len(), 1);
            assert_eq!(records(&f.core)[0].expires_at_ms, 2290);
            let hp = f.core.units[&f.enemy].stats.current_health;
            f.core
                .step_battle_execution_until(&mut f.state, 2090)
                .unwrap();
            assert_eq!(records(&f.core).len(), 1);
            assert_eq!(f.core.units[&f.enemy].stats.current_health, hp);
            f.core
                .step_battle_execution_until(&mut f.state, 2289)
                .unwrap();
            assert_eq!(records(&f.core).len(), 1);
            f.core
                .step_battle_execution_until(&mut f.state, 2290)
                .unwrap();
            assert!(records(&f.core).is_empty());
            assert_eq!(f.core.units[&f.enemy].stats.current_health, hp);
            assert_eq!(hit(&mut f, 2300), 100);
        }
    }
    #[test]
    fn pleasure_allies_and_regular_damage_types_share_weakness_other_target_isolated() {
        let mut f = setup(20);
        assert_eq!(hit(&mut f, 10), 100);
        for actor in [f.a, f.b] {
            for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
                for source in [
                    DamageSource::BasicAttack,
                    DamageSource::Ability,
                    DamageSource::BuffTick,
                    DamageSource::Environment,
                ] {
                    let target = f.enemy;
                    f.core
                        .runtime_transaction(|c| {
                            let s = c
                                .damage_source_snapshot_for_unit(
                                    actor,
                                    target,
                                    source,
                                    kind,
                                    100,
                                    Default::default(),
                                    0,
                                    100,
                                    false,
                                )
                                .unwrap();
                            assert_eq!(damage(c, s, target, 100), 120);
                            crate::game::battle::runtime_control::poll()
                        })
                        .unwrap();
                }
            }
        }
        let s = f
            .core
            .damage_source_snapshot_for_unit(
                f.b,
                f.secondary,
                DamageSource::Ability,
                DamageType::Magic,
                100,
                Default::default(),
                0,
                100,
                false,
            )
            .unwrap();
        assert_eq!(damage(&mut f.core, s, f.secondary, 100), 100);
        assert_eq!(
            records(&f.core).len(),
            1,
            "incoming damage never generates weakness stacks or ticks"
        );
    }
    #[test]
    fn pleasure_target_strike_segments_freeze_before_new_weakness_and_next_strike_reads_it() {
        let mut f = setup(20);
        let source = captured(&mut f, 10);
        let target = f.enemy;
        assert!(source.equipment_hits.as_ref().is_some_and(|capture|
            capture.providers.iter().any(|provider| provider.program.on_hit_status.is_some())));
        for expected in [100, 120] {
            f.core
                .runtime_transaction(|c| {
                    let old = c.begin_strike_scope();
                    assert_eq!(damage(c, source.clone(), target, 90), expected);
                    assert_eq!(damage(c, source.clone(), target, 90), expected);
                    c.end_strike_scope(old);
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
        }
        assert_eq!(records(&f.core).len(), 1);
    }
    #[test]
    fn pleasure_same_definition_strongest_distinct_definition_adds_and_latest_source_refreshes() {
        let mut f = setup(20);
        let (a, b, t) = (f.a, f.b, f.enemy);
        status(&mut f.core, a, t, "same", 20, 1000, 0);
        status(&mut f.core, b, t, "same", 35, 400, 0);
        status(&mut f.core, b, t, "different", 10, 1000, 0);
        assert_eq!(f.core.hit_damage_taken_percent(t).unwrap(), 45);
        assert_eq!(records(&f.core).len(), 3);
        status(&mut f.core, a, t, "same", 15, 2000, 10);
        assert_eq!(records(&f.core).len(), 3);
        assert_eq!(f.core.hit_damage_taken_percent(t).unwrap(), 45);
        f.core
            .step_battle_execution_until(&mut f.state, 400)
            .unwrap();
        assert_eq!(f.core.hit_damage_taken_percent(t).unwrap(), 25);
        f.core
            .step_battle_execution_until(&mut f.state, 1000)
            .unwrap();
        assert_eq!(f.core.hit_damage_taken_percent(t).unwrap(), 15);
        f.core
            .step_battle_execution_until(&mut f.state, 2010)
            .unwrap();
        assert_eq!(f.core.hit_damage_taken_percent(t).unwrap(), 0);
    }
    #[test]
    fn pleasure_shielded_and_zero_hits_apply_without_poison_or_control_immunity() {
        let mut f = setup(20);
        f.core
            .units
            .get_mut(&f.enemy)
            .unwrap()
            .target_traits
            .extend([
                UnitTargetTrait::StunImmune,
                UnitTargetTrait::SlowImmune,
                UnitTargetTrait::RootImmune,
            ]);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: 500,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        assert_eq!(hit(&mut f, 10), 0);
        assert_eq!(hit(&mut f, 110), 0);
        assert_eq!(records(&f.core).len(), 1);
        let remaining: u32 = f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter_map(|x| x.shield.map(|s| s.remaining))
            .sum();
        assert_eq!(remaining, 280);
        let mut zero = setup(20);
        // Target selection still uses a useful real attack. An already accepted
        // released hit can have a zero component; test that consumption boundary.
        let mut shot = captured(&mut zero, 10);
        shot.base_damage = 0;
        zero.core
            .runtime_transaction(|c| {
                assert_eq!(damage(c, shot, zero.enemy, 90), 0);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(records(&zero.core).len(), 1);
    }
    #[test]
    fn pleasure_cancel_after_replacement_restores_damage_deadline_and_observation_then_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(20);
        assert_eq!(hit(&mut f, 10), 100);
        attack(&mut f, 110, true);
        f.core
            .step_battle_execution_until(&mut f.state, 189)
            .unwrap();
        let before = (
            state(&f.core),
            format!("{:?}", f.state),
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_attack_hit_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 190),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(
            before,
            (
                state(&f.core),
                format!("{:?}", f.state),
                serde_json::to_value(&f.core.authoring_observation).unwrap()
            )
        );
        f.core
            .step_battle_execution_until(&mut f.state, 190)
            .unwrap();
        assert_eq!(records(&f.core).len(), 1);
        assert_eq!(records(&f.core)[0].expires_at_ms, 2190);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9780);
    }
    #[test]
    fn pleasure_observer_clone_retained_refusal_and_retry_keep_same_authority() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(20);
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 89)
            .unwrap();
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        let mut plain_state = f.state.clone();
        let (mut probe, mut execution) = (f.core.clone(), f.state.clone());
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut execution, 90)
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
            .step_battle_execution_until(&mut plain_state, 90)
            .unwrap();
        assert_eq!(records(&f.core).len(), 1);
        assert_eq!(plain.hit_damage_taken_percent(f.enemy).unwrap(), 20);
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
    fn pleasure_released_projectile_survives_withdrawal_and_expires() {
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
        assert_eq!(f.core.hit_damage_taken_percent(f.enemy).unwrap(), 20);
        let expiry = records(&f.core)[0].expires_at_ms;
        f.core
            .step_battle_execution_until(&mut f.state, expiry)
            .unwrap();
        assert!(records(&f.core).is_empty());
    }
    #[test]
    fn pleasure_parry_invulnerability_and_target_death() {
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
    fn pleasure_area_magic_targets_and_two_equipped_copies() {
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
            assert_eq!(f.core.hit_damage_taken_percent(target).unwrap(), 20);
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
        assert_eq!(f.core.hit_damage_taken_percent(f.enemy).unwrap(), 20);
        f.core
            .step_battle_execution_until(&mut f.state, 2090)
            .unwrap();
        assert_eq!(f.core.hit_damage_taken_percent(f.enemy).unwrap(), 20);
        assert_eq!(f.core.hit_damage_taken_percent(f.secondary).unwrap(), 0);
    }

    #[test]
    fn pleasure_only_real_basic_hits_apply_the_authored_weakness() {
        for source in [
            DamageSource::Ability,
            DamageSource::BuffTick,
            DamageSource::Environment,
        ] {
            let mut f = setup(20);
            let mut shot = captured(&mut f, 10);
            shot.source = source;
            f.core
                .runtime_transaction(|c| {
                    damage(c, shot, f.enemy, 90);
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            assert!(records(&f.core).is_empty(), "{source:?}");
        }
        let mut f = setup(20);
        let mut shot = captured(&mut f, 10);
        shot.fact_source.reaction = true;
        f.core
            .runtime_transaction(|c| {
                damage(c, shot, f.enemy, 90);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert!(records(&f.core).is_empty());
    }
}
