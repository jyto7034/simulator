mod composition_release_gap {
    use super::*;
    use crate::game::{
        ability::DeliveryDef,
        battle::{
            damage::{
                BattleCommand, DamageBonusSnapshot, DamageSource, DamageSourceSnapshot, DamageType,
            },
            runtime_control,
            tile_range::TileRangePolicy,
        },
        data::abnormality_data::BasicAttackAction,
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/release_gap.skill.yaml"
    ));
    fn setup() -> Fixture {
        let mut f = fixture(0, false, |d| *d = parse_yaml(SOURCE).unwrap());
        for u in f.core.units.values_mut() {
            u.stats.max_health = 100_000;
            u.stats.current_health = 100_000;
            u.stats.defense = 0;
            u.stats.magic_resist = 0;
        }
        let u = f.core.units.get_mut(&f.a).unwrap();
        u.stats.attack = 100;
        u.basic_attack.delivery = DeliveryDef::Instant;
        u.basic_attack.windup_ms = 200;
        u.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
        u.basic_attack.damage_type = DamageType::Physical;
        // The common fixture has processed its setup bucket. Model a fresh mount
        // at zero explicitly; the real mount/deploy path is covered by World.
        f.core.equipment_attack_state.release_gaps.clear();
        f.core
            .runtime_transaction(|c| c.reconcile_release_gaps(0))
            .unwrap();
        f.core.enable_authoring_observation();
        f.core
            .bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: 1,
                    battle_uuid: Uuid::from_u128(0xe102),
                },
            )
            .unwrap();
        f
    }
    fn advance(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn witness(f: &Fixture) -> &crate::game::battle::core::release_gap::ReleaseGapWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_release_gap
            .as_ref()
            .unwrap()
    }
    fn snap(c: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            c.transaction_debug_snapshot(),
            c.event_queue_debug_snapshot(),
            serde_json::to_value(c.runtime_fact_view()).unwrap(),
        )
    }
    fn admitted(f: &mut Fixture, regular: bool) -> DamageSourceSnapshot {
        attack(f, 1000, regular);
        f.core
            .event_queue
            .iter()
            .filter_map(|e| match e.as_ref() {
                BattleEvent::AttackResolve {
                    attacker_instance_id,
                    source_snapshot,
                    ..
                } if *attacker_instance_id == f.a => Some(source_snapshot.clone()),
                _ => None,
            })
            .max_by_key(|s| s.committed_at_ms)
            .unwrap()
    }
    fn hit(c: &mut BattleCore, target: UnitInstanceId, s: DamageSourceSnapshot, now: u64) -> u32 {
        let hp = c.units[&target].stats.current_health;
        c.runtime_transaction(|c| {
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: s,
                }],
                now,
            );
            runtime_control::poll()
        })
        .unwrap();
        hp - c.units[&target].stats.current_health
    }
    #[test]
    fn composition_release_gap_shared_roundtrip_bounds_and_accessory() {
        let a = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        let p = a.equipment_attacks()[0].program.release_gap.unwrap();
        assert_eq!((p.per_second_percent, p.max_percent), (20, 100));
        for field in ["per_second_percent", "max_percent"] {
            for bad in [0, 10001] {
                let mut d = parse_yaml(SOURCE).unwrap();
                program(&mut d)["release_gap"][field] = json!(bad);
                let e = compile_json(&d.to_string()).unwrap_err();
                assert!(format!("{e:?}").contains(field), "{e:?}");
            }
        }
        let f = setup();
        let mut db = (*f.core.game_data.equipment_data).clone();
        let item = db.items.iter_mut().find(|i| i.id == "r11_weapon").unwrap();
        item.equipment_type = crate::game::data::equipment_data::EquipmentType::Accessory;
        item.weapon_profile = None;
        crate::game::ability::equipment_attack::EquipmentAttackCatalog::compile(
            &db,
            &ResourceCatalog { resources: vec![] },
            &RelicDatabase::default(),
            &Default::default(),
        )
        .unwrap();
    }
    #[test]
    fn composition_release_gap_actual_release_counts_aim_and_resets_once() {
        let mut f = setup();
        let before = snap(&f.core);
        for (now, pct, cap) in [
            (49, 0, false),
            (50, 1, false),
            (999, 19, false),
            (1000, 20, false),
            (5000, 100, true),
            (u64::MAX, 100, true),
        ] {
            let q = f.core.release_gap_snapshot(f.a, now).unwrap().unwrap();
            assert_eq!((q.percent, q.capped), (pct, cap));
        }
        assert_eq!(before, snap(&f.core));
        attack(&mut f, 1000, true);
        advance(&mut f, 1199);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 100000);
        assert_eq!(
            f.core
                .release_gap_snapshot(f.a, 1199)
                .unwrap()
                .unwrap()
                .since_ms,
            0
        );
        advance(&mut f, 1200);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 99876);
        assert_eq!(
            (
                witness(&f).since_ms,
                witness(&f).time_ms,
                witness(&f).percent
            ),
            (0, 1200, 24)
        );
        assert_eq!(
            f.core
                .release_gap_snapshot(f.a, 1200)
                .unwrap()
                .unwrap()
                .percent,
            0
        );
        assert_eq!(
            f.core
                .release_gap_snapshot(f.b, 1200)
                .unwrap()
                .unwrap()
                .percent,
            24
        );
        attack(&mut f, 2000, true);
        advance(&mut f, 2200);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 99756);
        assert_eq!(
            (
                witness(&f).since_ms,
                witness(&f).elapsed_ms,
                witness(&f).percent
            ),
            (1200, 1000, 20)
        );
        assert_eq!(f.core.units[&f.a].stats.attack, 100);
    }
    #[test]
    fn composition_release_gap_cap_no_carry_and_nonregular_sources_excluded() {
        let mut f = setup();
        let mut s = admitted(&mut f, false);
        f.core
            .runtime_transaction(|c| c.capture_release_gap(&mut s, 6000))
            .unwrap();
        assert!(s.release_gap.is_none());
        assert_eq!(hit(&mut f.core, f.enemy, s, 6000), 100);
        assert_eq!(
            f.core
                .release_gap_snapshot(f.a, 6000)
                .unwrap()
                .unwrap()
                .since_ms,
            0
        );
        let mut f = setup();
        let mut s = admitted(&mut f, true);
        s.source = DamageSource::Ability;
        f.core
            .runtime_transaction(|c| c.capture_release_gap(&mut s, 6000))
            .unwrap();
        assert!(s.release_gap.is_none());
        s.source = DamageSource::BasicAttack;
        s.fact_source.reaction = true;
        f.core
            .runtime_transaction(|c| c.capture_release_gap(&mut s, 6000))
            .unwrap();
        assert!(s.release_gap.is_none());
        s.fact_source.reaction = false;
        f.core
            .runtime_transaction(|c| c.capture_release_gap(&mut s, 6000))
            .unwrap();
        assert_eq!(hit(&mut f.core, f.enemy, s, 7000), 200);
        assert_eq!(
            f.core
                .release_gap_snapshot(f.a, 6050)
                .unwrap()
                .unwrap()
                .percent,
            1
        );
        assert_eq!(witness(&f).elapsed_ms, 6000);
    }
    #[test]
    fn composition_release_gap_cancelled_windup_preserves_clock() {
        let mut f = setup();
        attack(&mut f, 1000, true);
        f.core.units.get_mut(&f.a).unwrap().basic_attack_epoch += 1;
        advance(&mut f, 1200);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_release_gap
            .is_none());
        assert_eq!(
            f.core
                .release_gap_snapshot(f.a, 1200)
                .unwrap()
                .unwrap()
                .since_ms,
            0
        );
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 100000);
        attack(&mut f, 2000, true);
        advance(&mut f, 2200);
        assert_eq!(witness(&f).percent, 44);
    }
    #[test]
    fn composition_release_gap_shared_capture_multitarget_and_delayed_tiles() {
        for delayed in [false, true] {
            let mut f = setup();
            super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 3, 1);
            f.core.units.get_mut(&f.a).unwrap().basic_attack.action = if delayed {
                BasicAttackAction::DamageTileAfterDelay { delay_ms: 400 }
            } else {
                ron::de::from_str("DamageArea(footprint:(include_anchor_tile:true,rows:[\"X@X\"]),secondary_percent:50)").unwrap()
            };
            attack(&mut f, 1000, true);
            advance(&mut f, 1200);
            assert_eq!(witness(&f).percent, 24);
            if delayed {
                assert_eq!(f.core.units[&f.enemy].stats.current_health, 100000);
                advance(&mut f, 1600);
            }
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 99876);
            assert_eq!(
                f.core.units[&f.secondary].stats.current_health,
                if delayed { 99876 } else { 99938 }
            );
            assert_eq!(witness(&f).time_ms, 1200);
        }
    }
    #[test]
    fn composition_release_gap_projectile_freezes_bonus_and_survives_withdrawal() {
        for miss in [false, true] {
            let mut f = setup();
            f.core.units.get_mut(&f.a).unwrap().basic_attack.delivery =
                ron::de::from_str("Projectile(speed_units_per_ms:1000)").unwrap();
            attack(&mut f, 1000, true);
            advance(&mut f, 1200);
            assert_eq!(witness(&f).percent, 24);
            assert!(!f.core.projectiles.is_empty());
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 100000);
            if miss {
                f.core.units.get_mut(&f.enemy).unwrap().lifecycle =
                    crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn;
            } else {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: f.a,
                        time_ms: 1201,
                    })
                    .unwrap();
                assert!(f.core.release_gap_snapshot(f.a, 1201).unwrap().is_none());
            }
            advance(&mut f, 3000);
            assert!(f.core.projectiles.is_empty());
            if !miss {
                assert_eq!(f.core.units[&f.enemy].stats.current_health, 99876);
            } else {
                assert_eq!(f.core.units[&f.enemy].stats.current_health, 100000);
                let q = f.core.release_gap_snapshot(f.a, 3000).unwrap().unwrap();
                assert_eq!((q.since_ms, q.percent), (1200, 36));
            }
            assert_eq!(witness(&f).time_ms, 1200);
        }
    }
    #[test]
    fn composition_release_gap_primary_only_type_preserved_and_shared_hit_capture() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            let mut f = setup();
            let mut s = admitted(&mut f, true);
            s.damage_type = kind;
            s.base_damage = 250; // Prepared raw multiplier has already applied.
            s.modifiers.base_damage_amp_percent = 10;
            s.on_attack_bonus_damage.push(DamageBonusSnapshot {
                flat: 100,
                percent: 0,
                damage_type: kind,
                equipment_source: None,
                source: crate::game::battle::cooldown::CooldownSource::Unit {
                    unit_instance_id: f.a,
                },
            });
            f.core
                .runtime_transaction(|c| c.capture_release_gap(&mut s, 2500))
                .unwrap();
            // 250 * (100 + 50 + 10)% + an unchanged 100 appended component.
            assert_eq!(hit(&mut f.core, f.enemy, s.clone(), 3000), 500);
            assert_eq!(hit(&mut f.core, f.secondary, s, 4000), 500);
            assert_eq!(
                f.core
                    .release_gap_snapshot(f.a, 4000)
                    .unwrap()
                    .unwrap()
                    .since_ms,
                2500
            );
        }
    }
    #[test]
    fn composition_release_gap_lifecycle_cleanup_and_reentry_start_zero() {
        for mode in 0..4 {
            let mut f = setup();
            let a = f.a;
            match mode {
                0 => f.core.units.get_mut(&a).unwrap().corroded = true,
                1 => f.core.items.retain(|_, i| i.owner_unit_instance != a),
                2 => {
                    let shot = f
                        .core
                        .damage_source_snapshot_for_unit(
                            f.enemy,
                            a,
                            DamageSource::Ability,
                            DamageType::True,
                            100000,
                            Default::default(),
                            0,
                            2000,
                            false,
                        )
                        .unwrap();
                    assert_eq!(hit(&mut f.core, a, shot, 2000), 100000);
                }
                _ => {
                    f.core
                        .runtime_transaction(|c| c.clear_sustained(2000))
                        .unwrap();
                }
            }
            f.core
                .runtime_transaction(|c| c.reconcile_release_gaps(2000))
                .unwrap();
            assert!(
                f.core.release_gap_snapshot(a, 3000).unwrap().is_none(),
                "mode {mode}"
            );
            assert!(!f
                .core
                .equipment_attack_state
                .release_gaps
                .values()
                .any(|c| c.owner == a));
            if mode == 0 {
                f.core.units.get_mut(&a).unwrap().corroded = false;
                f.core
                    .runtime_transaction(|c| c.reconcile_release_gaps(3000))
                    .unwrap();
                assert_eq!(
                    f.core
                        .release_gap_snapshot(a, 3000)
                        .unwrap()
                        .unwrap()
                        .percent,
                    0
                );
                assert_eq!(
                    f.core
                        .release_gap_snapshot(a, 4000)
                        .unwrap()
                        .unwrap()
                        .percent,
                    20
                );
            }
        }
    }
    #[test]
    fn composition_release_gap_cancel_and_retained_failure_rollback_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        attack(&mut f, 1000, true);
        advance(&mut f, 1199);
        let before = snap(&f.core);
        let state_before = format!("{:?}", f.state);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("release_gap_captured", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 1200),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, snap(&f.core));
        assert_eq!(state_before, format!("{:?}", f.state));
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = runtime_control::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 1200)
        });
        result.unwrap();
        let needed = admissions
            .iter()
            .filter(|(site, _, _)| *site == "release_gap_observation")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = needed - 1;
        let before = snap(&f.core);
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 1200)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(runtime_control::BattleRuntimeFault {
                    code: runtime_control::RuntimeFaultCode::RetainedByteLimit,
                    site: "release_gap_observation",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(before, snap(&f.core));
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        advance(&mut f, 1200);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 99876);
    }
    #[test]
    fn composition_release_gap_observation_off_clone_preserves_ordered_result() {
        let mut f = setup();
        attack(&mut f, 1000, true);
        advance(&mut f, 1199);
        let mut clone = f.core.clone();
        let mut state = f.state.clone();
        clone.authoring_observation = None;
        advance(&mut f, 1200);
        clone.step_battle_execution_until(&mut state, 1200).unwrap();
        assert_eq!(snap(&f.core), snap(&clone));
        assert_eq!(
            serde_json::to_value(&f.core.event_log).unwrap(),
            serde_json::to_value(&clone.event_log).unwrap()
        );
    }
    #[test]
    fn composition_release_gap_stun_keeps_time_redeploy_same_copy_starts_zero() {
        use crate::game::ability::sustained::{
            HitStatusKind, HitStatusProfile, SustainedEffectDef,
        };
        let mut f = setup();
        let a = f.a;
        let owned = f.core.units[&a].source_owned_uuid;
        let position = f.core.units[&a].body.projected_tile();
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
        let copy = f
            .core
            .release_gap_snapshot(a, 0)
            .unwrap()
            .unwrap()
            .equipment
            .persistent_copy_uuid;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    a,
                    SustainedEffectDef::HitStatus {
                        duration_ms: 1000,
                        profile: HitStatusProfile {
                            definition: "release-gap-stun".into(),
                            kind: HitStatusKind::Stun,
                            modifier_source: None,
                            vulnerability: None,
                        },
                    },
                    1000,
                )
            })
            .unwrap();
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(1500))
            .unwrap();
        assert_eq!(
            f.core
                .release_gap_snapshot(a, 1500)
                .unwrap()
                .unwrap()
                .percent,
            30
        );
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 3000,
            })
            .unwrap();
        assert!(f.core.release_gap_snapshot(a, 4000).unwrap().is_none());
        let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed {
            unit_id: fresh,
        } = f
            .core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position,
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: crate::game::battle::tile_range::FacingDirection::Right,
                instance_salt: 0xe1021,
                time_ms: 5000,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            unreachable!()
        };
        assert_ne!(a, fresh);
        let q = f.core.release_gap_snapshot(fresh, 5000).unwrap().unwrap();
        assert_eq!(q.equipment.persistent_copy_uuid, copy);
        assert_eq!((q.since_ms, q.percent), (5000, 0));
        assert_eq!(
            f.core
                .release_gap_snapshot(fresh, 6000)
                .unwrap()
                .unwrap()
                .percent,
            20
        );
    }
}
