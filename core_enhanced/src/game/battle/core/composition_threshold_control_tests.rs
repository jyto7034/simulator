mod composition_threshold_control {
    use super::*;
    use crate::game::{
        ability::sustained::{
            HitCounterOutcome, HitStatusKind, SustainedEffectDef, ThresholdControlKind,
        },
        battle::{
            core::damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
            damage::{DamageSourceSnapshot, DamageType},
            types::UnitTargetTrait,
        },
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/threshold_control.skill.yaml"
    ));
    fn setup_with(source: &str) -> Fixture {
        let source = source.replace("frost_splinter", "r11_weapon");
        let mut f = fixture(0, false, |doc| *doc = parse_yaml(&source).unwrap());
        f.core.enable_authoring_observation();
        f
    }
    fn setup() -> Fixture {
        setup_with(SOURCE)
    }
    fn counts(f: &Fixture) -> Vec<(UnitInstanceId, UnitInstanceId, u32, u64)> {
        f.core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter_map(|v| {
                if let HitStatusKind::ThresholdControl { stacks, .. } = v.hit_status?.kind {
                    Some((v.caster, v.target, stacks, v.expires_at_ms))
                } else {
                    None
                }
            })
            .collect()
    }
    fn outcome(f: &Fixture) -> HitCounterOutcome {
        let witness = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_sourced_hit_counter
            .as_ref()
            .unwrap();
        let crate::authoring::ApplicationChange::HitCounter { outcome } = witness.change else {
            panic!("counter witness");
        };
        outcome
    }
    fn basic(f: &mut Fixture, now: u64) -> u32 {
        let before = f.core.units[&f.enemy].stats.current_health;
        attack(f, now, true);
        f.core
            .step_battle_execution_until(&mut f.state, now + 2)
            .unwrap();
        before - f.core.units[&f.enemy].stats.current_health
    }
    fn captured(f: &mut Fixture, now: u64) -> DamageSourceSnapshot {
        attack(f, now, true);
        f.core
            .event_queue
            .iter()
            .find_map(|event| match event.as_ref() {
                BattleEvent::AttackResolve {
                    attacker_instance_id,
                    source_snapshot,
                    ..
                } if *attacker_instance_id == f.a => Some(source_snapshot.clone()),
                _ => None,
            })
            .unwrap()
    }
    fn damage(
        core: &mut BattleCore,
        source: DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> u64 {
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
    fn threshold_control_schema_shared_roundtrip_bounds_and_origin() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let reopened = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(matches!(
            reopened.equipment_attacks()[0]
                .program
                .on_hit_status
                .as_ref()
                .unwrap()
                .profile
                .kind,
            HitStatusKind::ThresholdControl {
                threshold: 5,
                control: ThresholdControlKind::Stun,
                ..
            }
        ));
        for (from, to) in [
            ("threshold: 5", "threshold: 0"),
            ("threshold: 5", "threshold: 1001"),
            ("stacks: 1", "stacks: 0"),
            ("stacks: 1", "stacks: 6"),
            ("control_ms: 1000", "control_ms: 0"),
            ("duration_ms: 5000", "duration_ms: 0"),
            ("control: Stun", "control: Unknown"),
        ] {
            assert!(
                crate::authoring::composition::compile_yaml(&SOURCE.replace(from, to)).is_err(),
                "{to}"
            );
        }
        assert!(crate::authoring::composition::compile_yaml(
            &SOURCE.replace("threshold: 5", "threshold: 5, ignored_typo: 1")
        )
        .is_err());
        let mut f = setup();
        basic(&mut f, 10);
        let w = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_sourced_hit_counter
            .as_ref()
            .unwrap();
        assert!(w
            .authored
            .as_ref()
            .unwrap()
            .field_path
            .contains("on_hit_status"));
        assert!(w.equipment.is_some());
    }
    #[test]
    fn threshold_control_five_hits_consume_stun_and_allow_reentry_without_slow() {
        let mut f = setup();
        for (n, now) in [10, 20, 30, 40].into_iter().enumerate() {
            assert_eq!(basic(&mut f, now), 100);
            assert_eq!(counts(&f)[0].2, n as u32 + 1);
            assert!(!f
                .core
                .unit_control_state(f.enemy, now + 2)
                .unwrap()
                .hard_cc_active());
            assert!(!outcome(&f).triggered);
        }
        assert_eq!(basic(&mut f, 50), 100);
        assert!(counts(&f).is_empty());
        let w = outcome(&f);
        assert_eq!((w.before, w.added, w.after, w.threshold), (4, 1, 0, 5));
        assert!(w.triggered && w.control_applied);
        assert_eq!(w.control_expires_at_ms, Some(1051));
        assert!(f
            .core
            .unit_control_state(f.enemy, 1050)
            .unwrap()
            .hard_cc_active());
        assert!(!f
            .core
            .unit_control_state(f.enemy, 1051)
            .unwrap()
            .hard_cc_active());
        basic(&mut f, 60);
        assert_eq!(counts(&f)[0].2, 1);
        f.core
            .step_battle_execution_until(&mut f.state, 1051)
            .unwrap();
        assert!(!f
            .core
            .unit_control_state(f.enemy, 1051)
            .unwrap()
            .hard_cc_active());
        assert_eq!(counts(&f)[0].2, 1);
    }
    #[test]
    fn threshold_control_authored_root_and_packet_remainder_use_same_runtime() {
        let mut f = setup_with(
            &SOURCE
                .replace("threshold: 5", "threshold: 3")
                .replace("stacks: 1", "stacks: 2")
                .replace("control: Stun", "control: Root"),
        );
        basic(&mut f, 10);
        assert_eq!(counts(&f)[0].2, 2);
        basic(&mut f, 20);
        let w = outcome(&f);
        assert_eq!((w.before, w.added, w.after), (2, 2, 1));
        assert!(w.triggered && w.control_applied);
        assert_eq!(w.control, ThresholdControlKind::Root);
        assert!(!f
            .core
            .unit_control_state(f.enemy, 22)
            .unwrap()
            .hard_cc_active());
        assert!(f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .any(|s| s.target == f.enemy
                && s.control == Some(crate::game::ability::sustained::ControlKind::Root)));
        basic(&mut f, 30);
        assert!(counts(&f).is_empty());
        assert!(outcome(&f).control_applied);
    }
    #[test]
    fn threshold_control_refresh_discards_old_expiry_and_has_bounded_storage() {
        let mut f = setup_with(&SOURCE.replace("threshold: 5", "threshold: 1000"));
        // Shield keeps the target alive across repeated real accepted attacks.
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Shield {
                        duration_ms: 20000,
                        amount: 20000,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        for now in 1..=100 {
            basic(&mut f, now * 10);
        }
        assert_eq!(counts(&f).len(), 1);
        assert_eq!(counts(&f)[0].2, 100);
        assert_eq!(counts(&f)[0].3, 6001);
        assert_eq!(
            f.core
                .event_queue
                .iter()
                .filter(|e| matches!(e.as_ref(), BattleEvent::SustainedExpiry { .. }))
                .count(),
            2
        );
        f.core
            .step_battle_execution_until(&mut f.state, 5011)
            .unwrap();
        assert_eq!(counts(&f)[0].2, 100);
        f.core
            .step_battle_execution_until(&mut f.state, 6001)
            .unwrap();
        assert!(counts(&f).is_empty());
        basic(&mut f, 6010);
        assert_eq!(counts(&f)[0].2, 1);
    }
    #[test]
    fn threshold_control_immunity_consumes_and_next_hits_start_new_counter() {
        for (control, immune) in [
            ("Stun", UnitTargetTrait::StunImmune),
            ("Root", UnitTargetTrait::RootImmune),
        ] {
            let mut f = setup_with(
                &SOURCE
                    .replace("threshold: 5", "threshold: 2")
                    .replace("control: Stun", &format!("control: {control}")),
            );
            f.core
                .units
                .get_mut(&f.enemy)
                .unwrap()
                .target_traits
                .push(immune);
            basic(&mut f, 10);
            basic(&mut f, 20);
            let w = outcome(&f);
            assert!(w.triggered && !w.control_applied);
            assert_eq!(w.after, 0);
            assert!(counts(&f).is_empty());
            assert_eq!(w.control_expires_at_ms, None);
            basic(&mut f, 30);
            assert_eq!(counts(&f)[0].2, 1);
        }
    }
    #[test]
    fn threshold_control_zero_shield_parry_invulnerability_and_lethal_hit() {
        let mut zero = setup();
        let mut source = captured(&mut zero, 10);
        source.base_damage = 0;
        zero.core
            .runtime_transaction(|c| {
                assert_eq!(damage(c, source, zero.enemy, 11), 0);
                Ok(())
            })
            .unwrap();
        assert_eq!(counts(&zero)[0].2, 1);
        let mut f = setup();
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Shield {
                        duration_ms: 12,
                        amount: 100,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        assert_eq!(basic(&mut f, 10), 0);
        assert_eq!(counts(&f)[0].2, 1);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::Parry {
                        duration_ms: 200,
                        window_ms: 30,
                        invulnerable_ms: 40,
                        miss_recovery_ms: 20,
                        reduction_percent: 0,
                        counter_skill: "r11_aura".into(),
                        end_on_resolution: false,
                        fragment_guard: None,
                        guard_charge: None,
                    },
                    15,
                )
            })
            .unwrap();
        assert_eq!(basic(&mut f, 20), 0);
        assert_eq!(counts(&f)[0].2, 1);
        assert_eq!(basic(&mut f, 30), 0);
        assert_eq!(counts(&f)[0].2, 1);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
        assert_eq!(basic(&mut f, 220), 1);
        assert!(counts(&f).is_empty());
    }
    #[test]
    fn threshold_control_area_targets_and_equipped_copies_are_isolated() {
        use crate::game::{
            battle::tile_range::TileRangePattern, data::abnormality_data::BasicAttackAction,
        };
        let mut f = setup();
        super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 2);
        f.core.units.get_mut(&f.a).unwrap().basic_attack.action = BasicAttackAction::DamageArea {
            footprint: TileRangePattern {
                include_anchor_tile: true,
                rows: vec!["XXX".into(), "X@X".into(), "XXX".into()],
            },
            secondary_percent: 100,
        };
        basic(&mut f, 10);
        assert_eq!(counts(&f).len(), 2);
        assert!(counts(&f).iter().all(|s| s.2 == 1));
        let old = f.a;
        super::super::compiled_action_fixture::place(&mut f.core, old, 7, 3);
        super::super::compiled_action_fixture::place(&mut f.core, f.b, 2, 1);
        f.a = f.b;
        basic(&mut f, 20);
        assert_eq!(counts(&f).len(), 3);
        for now in [30, 40, 50, 60] {
            basic(&mut f, now);
        }
        assert_eq!(counts(&f).len(), 2);
        assert!(counts(&f).iter().all(|s| s.0 == old && s.2 == 1));
    }
    #[test]
    fn threshold_control_cancellation_between_consumption_and_control_rolls_back() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        for now in [10, 20, 30, 40] {
            basic(&mut f, now);
        }
        let source = captured(&mut f, 50);
        let before = f.core.transaction_debug_snapshot();
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("hit_counter_consumed", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    f.core.runtime_transaction(|c| {
                        c.execute_committed_damage(CommittedDamageInput {
                            target_id: f.enemy,
                            source_snapshot: source.clone(),
                            committed_at_ms: 51,
                            policy: CommittedDamagePolicy::GeneralCommand,
                        });
                        crate::game::battle::runtime_control::poll()
                    })
                },
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core
            .runtime_transaction(|c| {
                damage(c, source, f.enemy, 51);
                Ok(())
            })
            .unwrap();
        assert!(outcome(&f).control_applied);
        assert!(counts(&f).is_empty());
    }
    #[test]
    fn threshold_control_catalog_profile_replaces_legacy_freeze() {
        let data: crate::game::data::equipment_data::EquipmentDatabase =
            ron::de::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../game_resources/data/equipments/base.ron"
            )))
            .unwrap();
        let item = data.get_by_id("frost_splinter").unwrap();
        let p = item.weapon_profile.as_ref().unwrap();
        assert_eq!(format!("{:?}", p.weapon_archetype), "Sword");
        assert_eq!(p.block_capacity, 0);
        assert_eq!(p.damage_type, DamageType::Magic);
        assert!(p.air_capable);
        assert!(matches!(
            p.action,
            crate::game::data::abnormality_data::BasicAttackAction::DamageArea {
                secondary_percent: 100,
                ..
            }
        ));
        assert!(!ron::ser::to_string(item).unwrap().contains("freeze"));
    }

    #[test]
    fn threshold_control_real_fragment_damage_uses_own_stack() {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| *doc = parse_yaml(&SOURCE.replace("frost_splinter", "r11_weapon")).unwrap(),
            |ron| *ron = r12_control_ron("Damage(amount:100,damage_type:True)"),
            |scenario| {
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else {
                        unreachable!()
                    };
                    profile.fragment_source =
                        Some(crate::game::skill_fragment::FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0xf400 + i as u128),
                            definition_id: "r09_fragment".into(),
                            form_id: "base".into(),
                            skill_id: "r11_aura".into(),
                        });
                }
            },
            Some(
                ron::de::from_str(
                    &include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/tests/fixtures/r09/fragments.ron"
                    ))
                    .replace("r08_strike", "r11_aura"),
                )
                .unwrap(),
            ),
        );
        assert_eq!(basic(&mut f, 10), 100);
        for (actor, expected, now) in [(f.a, 100, 20), (f.b, 100, 30)] {
            if actor == f.b {
                super::super::compiled_action_fixture::place(&mut f.core, f.a, 7, 3);
                super::super::compiled_action_fixture::place(&mut f.core, f.b, 2, 1);
            }
            let before = f.core.units[&f.enemy].stats.current_health;
            f.core.units.get_mut(&actor).unwrap().resonance_current = 100;
            f.core
                .apply_live_command(BattleLiveCommand::ActivateSkill {
                    unit_id: actor,
                    skill_id: "r11_aura".into(),
                    target: None,
                    time_ms: now,
                })
                .unwrap();
            f.core
                .step_battle_execution_until(&mut f.state, now + 1)
                .unwrap();
            assert_eq!(
                before - f.core.units[&f.enemy].stats.current_health,
                expected
            );
        }
        assert_eq!(
            counts(&f)[0].2,
            1,
            "fragment direct hits never grow weapon stacks"
        );
    }
    #[test]
    fn threshold_control_projectile_and_landed_state_survive_withdrawal() {
        use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
        let mut f = setup();
        let owned = f.core.units[&f.a].source_owned_uuid;
        let draft = f
            .core
            .scenario
            .groups
            .iter()
            .flat_map(|g| &g.spawns)
            .find(|s| s.draft.owned_uuid == owned)
            .unwrap()
            .draft
            .clone();
        let original = f.a;
        assert_eq!(basic(&mut f, 10), 100);
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
        attack(&mut f, 20, true);
        f.core
            .step_battle_execution_until(&mut f.state, 22)
            .unwrap();
        assert!(f
            .core
            .projectiles
            .values()
            .any(|p| p.source_snapshot.source_id == f.a));
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: f.a,
                time_ms: 22,
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 1200)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 800);
        assert_eq!(counts(&f)[0].2, 2);
        let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed {
            unit_id: fresh,
        } = f
            .core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position: crate::game::resources::Position::new(2, 1),
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: crate::game::battle::tile_range::FacingDirection::Right,
                instance_salt: 0xe410,
                time_ms: 1201,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            unreachable!()
        };
        assert_ne!(fresh, original);
        let unit = f.core.units.get_mut(&fresh).unwrap();
        unit.stats.attack_interval_ms = 100_000;
        unit.next_basic_attack_ms = 100_000;
        unit.stats.move_speed_units_per_ms = 0;
        f.a = fresh;
        assert_eq!(
            basic(&mut f, 1210),
            100,
            "same owned copy on a new battle unit cannot inherit old stacks"
        );
        let active = counts(&f);
        assert_eq!(active.len(), 2);
        assert_eq!(active.iter().find(|s| s.0 == original).unwrap().2, 2);
        assert_eq!(active.iter().find(|s| s.0 == fresh).unwrap().2, 1);
    }
    #[test]
    fn threshold_control_observation_clone_and_retained_refusal_preserve_results() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        for now in [10, 20, 30, 40] {
            basic(&mut f, now);
        }
        let mut source = captured(&mut f, 50);
        source.base_damage = 0;
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        plain
            .runtime_transaction(|c| {
                c.execute_committed_damage(CommittedDamageInput {
                    target_id: f.enemy,
                    source_snapshot: source.clone(),
                    committed_at_ms: 51,
                    policy: CommittedDamagePolicy::GeneralCommand,
                });
                native::poll()
            })
            .unwrap();
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|c| {
                c.execute_committed_damage(CommittedDamageInput {
                    target_id: f.enemy,
                    source_snapshot: source.clone(),
                    committed_at_ms: 51,
                    policy: CommittedDamagePolicy::GeneralCommand,
                });
                native::poll()
            })
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "hit_counter_replacements")
            .map(|(_, _, bytes)| *bytes)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        assert!(f
            .core
            .runtime_transaction(|c| {
                c.execute_committed_damage(CommittedDamageInput {
                    target_id: f.enemy,
                    source_snapshot: source.clone(),
                    committed_at_ms: 51,
                    policy: CommittedDamagePolicy::GeneralCommand,
                });
                native::poll()
            })
            .is_err());
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        f.core.runtime_facts.limits = Default::default();
        f.core
            .runtime_transaction(|c| {
                c.execute_committed_damage(CommittedDamageInput {
                    target_id: f.enemy,
                    source_snapshot: source,
                    committed_at_ms: 51,
                    policy: CommittedDamagePolicy::GeneralCommand,
                });
                native::poll()
            })
            .unwrap();
        assert_eq!(
            f.core.transaction_debug_snapshot(),
            plain.transaction_debug_snapshot()
        );
        assert!(outcome(&f).control_applied);
        assert_eq!(counts(&f).len(), 0);
        assert!(f
            .core
            .unit_control_state(f.enemy, 52)
            .unwrap()
            .hard_cc_active());
    }

    #[test]
    fn threshold_control_retrigger_keeps_existing_stun_until_last_expiry_and_terminal_clears() {
        let mut f = setup_with(&SOURCE.replace("threshold: 5", "threshold: 2"));
        for now in [10, 20, 30, 40] {
            basic(&mut f, now);
        }
        assert!(counts(&f).is_empty());
        assert_eq!(outcome(&f).control_expires_at_ms, Some(1041));
        f.core
            .step_battle_execution_until(&mut f.state, 1021)
            .unwrap();
        assert!(f
            .core
            .unit_control_state(f.enemy, 1021)
            .unwrap()
            .hard_cc_active());
        assert_eq!(
            f.core
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .filter(|r| r
                    .hit_status
                    .as_ref()
                    .is_some_and(|p| p.kind == HitStatusKind::Stun))
                .count(),
            1
        );
        f.core
            .step_battle_execution_until(&mut f.state, 1041)
            .unwrap();
        assert!(!f
            .core
            .unit_control_state(f.enemy, 1041)
            .unwrap()
            .hard_cc_active());
        basic(&mut f, 1050);
        assert_eq!(counts(&f)[0].2, 1);
        f.core
            .runtime_transaction(|c| c.clear_sustained(1052))
            .unwrap();
        assert!(counts(&f).is_empty());
        assert!(!f
            .core
            .event_queue
            .iter()
            .any(|e| matches!(e.as_ref(), BattleEvent::SustainedExpiry { .. })));
    }
}
