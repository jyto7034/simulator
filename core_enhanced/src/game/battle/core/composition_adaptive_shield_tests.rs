mod composition_adaptive_shield {
    use super::*;
    use crate::game::{
        ability::sustained::SustainedEffectDef,
        battle::{
            core::{
                damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
                equipment_defense::AdaptiveShieldWitness,
            },
            damage::{DamageSource, DamageSourceSnapshot, DamageType},
        },
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/adaptive_shield.skill.yaml"
    ));
    fn setup_with(edit: impl FnOnce(&mut Value)) -> Fixture {
        let mut f = fixture(0, false, |doc| {
            *doc =
                crate::authoring::composition::canonical_overlay_tests::equipment_fixture("armor");
            edit(doc);
        });
        for u in f.core.units.values_mut() {
            u.stats.magic_resist = 0;
        }
        f.core.enable_authoring_observation();
        f.core
            .bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: 1,
                    battle_uuid: Uuid::from_u128(0xb450),
                },
            )
            .unwrap();
        f
    }
    fn setup() -> Fixture {
        setup_with(|_| {})
    }
    #[test]
    fn shield_beneficiary_typed_pool_applies_to_all_outgoing_types_until_depletion() {
        let mut f = setup_with(|doc| {
            doc["equipment_supports"] = json!([{"equipment":"r11_weapon","program":{"Literal":{"SupportProgram":{
                "heal_percent":0,"shield_percent":0,"shield_beneficiary_damage_percent":20
            }}}}]);
        });
        attack(&mut f, 0, true);
        f.core.step_battle_execution_until(&mut f.state, 2).unwrap();
        assert_eq!(
            pools(&f.core, f.a),
            vec![(Some(DamageType::Physical), 200, 200, 5001)]
        );
        let (actor, target) = (f.a, f.enemy);
        let apply = |c: &mut BattleCore, kind, now| {
            let shot = c
                .damage_source_snapshot_for_unit(
                    actor,
                    target,
                    DamageSource::Ability,
                    kind,
                    100,
                    Default::default(),
                    0,
                    now,
                    false,
                )
                .unwrap();
            let before = c.units[&target].stats.current_health;
            hit(c, shot, target, now)?;
            Ok(before - c.units[&target].stats.current_health)
        };
        for kind in [DamageType::Magic, DamageType::True] {
            assert_eq!(
                f.core.runtime_transaction(|c| apply(c, kind, 3)).unwrap(),
                120
            );
        }
        direct(&mut f, DamageType::Physical, 200, 4);
        assert_eq!(
            f.core
                .runtime_transaction(|c| apply(c, DamageType::True, 5))
                .unwrap(),
            100
        );
    }
    fn witness(core: &BattleCore) -> &AdaptiveShieldWitness {
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_adaptive_shield
            .as_ref()
            .unwrap()
    }
    fn pools(core: &BattleCore, owner: UnitInstanceId) -> Vec<(Option<DamageType>, u32, u32, u64)> {
        core.sustained_effect_snapshot()
            .unwrap()
            .active
            .into_iter()
            .filter(|e| e.target == owner)
            .filter_map(|e| {
                e.shield
                    .map(|s| (s.damage_type, s.capacity, s.remaining, e.expires_at_ms))
            })
            .collect()
    }
    fn captured(f: &mut Fixture, now: u64) -> DamageSourceSnapshot {
        attack(f, now, true);
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
            .expect("real normal attack admission")
    }
    fn hit(
        core: &mut BattleCore,
        source: DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        core.execute_committed_damage(CommittedDamageInput {
            target_id: target,
            source_snapshot: source,
            committed_at_ms: now,
            policy: CommittedDamagePolicy::GeneralCommand,
        })
        .ok_or(crate::game::behavior::GameError::InvalidAction)?;
        crate::game::battle::runtime_control::poll()
    }
    fn direct(f: &mut Fixture, kind: DamageType, amount: u32, now: u64) {
        let source = f
            .core
            .damage_source_snapshot_for_unit(
                f.secondary,
                f.a,
                DamageSource::Ability,
                kind,
                amount,
                Default::default(),
                0,
                now,
                false,
            )
            .unwrap();
        f.core
            .runtime_transaction(|c| hit(c, source, f.a, now))
            .unwrap();
    }
    fn grant(f: &mut Fixture, now: u64) {
        let source = captured(f, now);
        f.core
            .runtime_transaction(|c| hit(c, source, f.enemy, now + 1))
            .unwrap();
    }
    #[test]
    fn composition_adaptive_shield_schema_roundtrip_and_precise_errors() {
        let doc = parse_yaml(SOURCE).unwrap();
        let artifact = compile_json(&doc.to_string()).unwrap();
        let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(matches!(
            loaded.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::AdaptiveBasicShield {
                max_hp_percent: 20,
                duration_ms: 5000,
                cooldown_ms: 15000,
                priority: 0
            }
        ));
        for (field, value) in [
            ("max_hp_percent", 0),
            ("max_hp_percent", 10001),
            ("duration_ms", 0),
            ("cooldown_ms", 0),
        ] {
            let mut invalid = doc.clone();
            invalid["values"][0]["value"]["DefenseProgram"]["AdaptiveBasicShield"][field] =
                json!(value);
            let error = compile_json(&invalid.to_string()).unwrap_err();
            assert!(
                error
                    .source
                    .definition
                    .ends_with(&format!("DefenseProgram.AdaptiveBasicShield.{field}")),
                "{error:?}"
            );
        }
    }
    #[test]
    fn composition_adaptive_shield_actual_attack_and_matching_damage_from_another_source() {
        let mut f = setup();
        attack(&mut f, 0, true);
        f.core.step_battle_execution_until(&mut f.state, 2).unwrap();
        let w = witness(&f.core);
        assert_eq!(
            (
                w.actor,
                w.primary,
                w.damage_type,
                w.capacity,
                w.time_ms,
                w.expires_at_ms,
                w.ready_at_ms
            ),
            (f.a, f.enemy, DamageType::Physical, 200, 1, 5001, 15001)
        );
        assert!(w.equipment.persistent_copy_uuid.is_some());
        assert!(w.attack_source.action.is_some());
        direct(&mut f, DamageType::Physical, 60, 3);
        assert_eq!(f.core.units[&f.a].stats.current_health, 500);
        direct(&mut f, DamageType::Magic, 70, 4);
        direct(&mut f, DamageType::True, 30, 5);
        assert_eq!(f.core.units[&f.a].stats.current_health, 400);
        assert_eq!(
            pools(&f.core, f.a),
            vec![(Some(DamageType::Physical), 200, 140, 5001)]
        );
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(5001))
            .unwrap();
        assert!(pools(&f.core, f.a).is_empty());
    }
    #[test]
    fn composition_adaptive_shield_mixed_components_and_ordinary_priority() {
        use crate::game::battle::{cooldown::CooldownSource, damage::DamageBonusSnapshot};
        let mut f = setup();
        f.core
            .units
            .get_mut(&f.enemy)
            .unwrap()
            .basic_attack
            .damage_type = DamageType::Magic;
        grant(&mut f, 0);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.a,
                    f.a,
                    SustainedEffectDef::IndependentShield {
                        amount: 70,
                        duration_ms: 5000,
                        priority: -1,
                    },
                    2,
                )
            })
            .unwrap();
        let mut source = f
            .core
            .damage_source_snapshot_for_unit(
                f.secondary,
                f.a,
                DamageSource::Ability,
                DamageType::Physical,
                80,
                Default::default(),
                0,
                3,
                false,
            )
            .unwrap();
        source.on_attack_bonus_damage.push(DamageBonusSnapshot {
            source: CooldownSource::Unit {
                unit_instance_id: f.secondary,
            },
            equipment_source: None,
            flat: 60,
            percent: 0,
            damage_type: DamageType::Magic,
        });
        f.core
            .runtime_transaction(|c| hit(c, source, f.a, 3))
            .unwrap();
        assert_eq!(f.core.units[&f.a].stats.current_health, 490);
        assert_eq!(
            pools(&f.core, f.a),
            vec![(Some(DamageType::Magic), 200, 140, 5001)]
        );
        let fact = f
            .core
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .unwrap();
        assert_eq!(fact.protection.primary_absorbed(), 70);
        assert_eq!(
            fact.protection
                .shields
                .iter()
                .map(|s| (s.component_index, s.absorbed))
                .collect::<Vec<_>>(),
            vec![(Some(0), 70), (Some(1), 60)]
        );
    }
    #[test]
    fn composition_adaptive_shield_primary_only_and_lethal_primary_keeps_its_type() {
        let mut f = setup();
        f.core
            .units
            .get_mut(&f.enemy)
            .unwrap()
            .basic_attack
            .damage_type = DamageType::Magic;
        let source = captured(&mut f, 0);
        assert_eq!(source.basic_primary, Some(f.enemy));
        f.core
            .runtime_transaction(|c| hit(c, source.clone(), f.secondary, 1))
            .unwrap();
        assert!(pools(&f.core, f.a).is_empty());
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
        f.core
            .runtime_transaction(|c| hit(c, source, f.enemy, 2))
            .unwrap();
        assert!(f.core.units[&f.enemy].lifecycle.is_dead());
        assert_eq!(witness(&f.core).damage_type, DamageType::Magic);
        assert_eq!(witness(&f.core).primary, f.enemy);
    }
    #[test]
    fn composition_adaptive_shield_action_once_cooldown_boundary_and_current_max_hp() {
        let mut f = setup();
        let source = captured(&mut f, 0);
        f.core
            .runtime_transaction(|c| hit(c, source.clone(), f.enemy, 1))
            .unwrap();
        let first = witness(&f.core).activation;
        f.core
            .runtime_transaction(|c| hit(c, source, f.enemy, 15001))
            .unwrap();
        assert_eq!(
            witness(&f.core).activation,
            first,
            "same live action cannot proc after cooldown"
        );
        f.core.units.get_mut(&f.a).unwrap().stats.max_health = 103;
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 103;
        let fresh = captured(&mut f, 15001);
        f.core
            .runtime_transaction(|c| hit(c, fresh, f.enemy, 15002))
            .unwrap();
        assert_eq!(witness(&f.core).capacity, 20);
        assert_eq!(witness(&f.core).max_hp, 103);
        assert_eq!(witness(&f.core).ready_at_ms, 30002);
    }
    #[test]
    fn composition_adaptive_shield_unsupported_type_and_triggered_or_skill_hits_do_not_proc() {
        for mode in 0..5 {
            let mut f = setup();
            if mode == 0 {
                f.core
                    .units
                    .get_mut(&f.enemy)
                    .unwrap()
                    .basic_attack
                    .damage_type = DamageType::True;
            }
            if mode == 4 {
                f.core.units.get_mut(&f.enemy).unwrap().basic_attack.action =
                    crate::game::data::abnormality_data::BasicAttackAction::Heal {
                        flat: 1,
                        spell_power_percent: 0,
                        footprint: None,
                    };
            }
            let mut source = captured(&mut f, 0);
            if mode == 1 {
                source.basic_primary = None;
            }
            if mode == 2 {
                source.source = DamageSource::Ability;
            }
            if mode == 3 {
                source.fact_source.reaction = true;
            }
            f.core
                .runtime_transaction(|c| hit(c, source, f.enemy, 1))
                .unwrap();
            assert!(pools(&f.core, f.a).is_empty(), "mode {mode}");
        }
    }
    #[test]
    fn composition_adaptive_shield_zero_hit_and_blocked_hit_are_distinct() {
        let mut f = setup();
        let mut source = captured(&mut f, 0);
        source.base_damage = 0;
        source.minimum_damage = 0;
        f.core
            .runtime_transaction(|c| hit(c, source, f.enemy, 1))
            .unwrap();
        assert_eq!(witness(&f.core).capacity, 200);
        let mut f = setup();
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
                    0,
                )
            })
            .unwrap();
        let source = captured(&mut f, 0);
        f.core
            .runtime_transaction(|c| hit(c, source.clone(), f.enemy, 1))
            .unwrap();
        assert!(f.core.sustained_invulnerable(f.enemy, 2).unwrap());
        f.core
            .runtime_transaction(|c| hit(c, source.clone(), f.secondary, 2))
            .unwrap();
        assert!(
            pools(&f.core, f.a).is_empty(),
            "only secondary hit cannot activate"
        );
        f.core
            .runtime_transaction(|c| hit(c, source, f.enemy, 2))
            .unwrap();
        assert!(
            pools(&f.core, f.a).is_empty(),
            "primary remains invulnerable"
        );
    }
    #[test]
    fn composition_adaptive_shield_active_pool_never_refreshes_and_corrosion_clears() {
        let mut f = setup_with(|d| {
            d["values"][0]["value"]["DefenseProgram"]["AdaptiveBasicShield"]["cooldown_ms"] =
                json!(1)
        });
        grant(&mut f, 0);
        let first = witness(&f.core).activation;
        grant(&mut f, 10);
        assert_eq!(witness(&f.core).activation, first);
        assert_eq!(
            pools(&f.core, f.a),
            vec![(Some(DamageType::Physical), 200, 200, 5001)]
        );
        f.core.units.get_mut(&f.a).unwrap().corroded = true;
        f.core
            .runtime_transaction(|c| c.reconcile_sustained(12))
            .unwrap();
        assert!(pools(&f.core, f.a).is_empty());
        f.core.units.get_mut(&f.a).unwrap().corroded = false;
        grant(&mut f, 20);
        assert_ne!(witness(&f.core).activation, first);
        f.core
            .runtime_transaction(|c| c.clear_sustained(22))
            .unwrap();
        assert!(pools(&f.core, f.a).is_empty());
    }
    #[test]
    fn composition_adaptive_shield_clone_observer_and_cancel_restore() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        let source = captured(&mut f, 0);
        let before = f.core.transaction_debug_snapshot();
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("adaptive_shield_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    f.core
                        .runtime_transaction(|c| hit(c, source.clone(), f.enemy, 1))
                },
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, f.core.transaction_debug_snapshot());
        assert_eq!(
            obs,
            serde_json::to_value(&f.core.authoring_observation).unwrap()
        );
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        plain
            .runtime_transaction(|c| hit(c, source.clone(), f.enemy, 1))
            .unwrap();
        f.core
            .runtime_transaction(|c| hit(c, source, f.enemy, 1))
            .unwrap();
        assert_eq!(
            plain.transaction_debug_snapshot(),
            f.core.transaction_debug_snapshot()
        );
        assert_eq!(pools(&plain, f.a), pools(&f.core, f.a));
    }
    #[test]
    fn composition_adaptive_shield_withdraw_cancel_restores_state_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        grant(&mut f, 0);
        assert!(!f.core.equipment_defense.is_empty());
        let snapshot = |core: &BattleCore| {
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                format!("{:?}", core.equipment_defense),
                serde_json::to_value(&core.authoring_observation).unwrap(),
                pools(core, f.a),
            )
        };
        let before = snapshot(&f.core);
        let command = BattleLiveCommand::WithdrawUnit {
            unit_id: f.a,
            time_ms: 2,
        };
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_defense_remove_owner", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.apply_live_command(command.clone()),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snapshot(&f.core), before);
        f.core.apply_live_command(command).unwrap();
        assert!(f.core.equipment_defense.is_empty());
        assert!(pools(&f.core, f.a).is_empty());
    }
    #[test]
    fn composition_adaptive_shield_depletion_exact_cooldown_and_zero_rounding() {
        let mut f = setup();
        grant(&mut f, 0);
        direct(&mut f, DamageType::Physical, 200, 2);
        assert!(pools(&f.core, f.a).is_empty());
        grant(&mut f, 14999);
        assert!(pools(&f.core, f.a).is_empty());
        grant(&mut f, 15000);
        assert_eq!(witness(&f.core).time_ms, 15001);
        let mut f = setup();
        f.core.units.get_mut(&f.a).unwrap().stats.max_health = 4;
        grant(&mut f, 0);
        assert!(pools(&f.core, f.a).is_empty());
        f.core.units.get_mut(&f.a).unwrap().stats.max_health = 5;
        grant(&mut f, 10);
        assert_eq!(witness(&f.core).capacity, 1);
    }
    #[test]
    fn composition_adaptive_shield_full_absorption_counts_and_survival_applies_after_components() {
        let mut f = setup();
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.enemy,
                    f.enemy,
                    SustainedEffectDef::IndependentShield {
                        amount: 500,
                        duration_ms: 5000,
                        priority: 0,
                    },
                    0,
                )
            })
            .unwrap();
        grant(&mut f, 0);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 1000);
        assert_eq!(witness(&f.core).capacity, 200);
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 10;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.a,
                    f.a,
                    SustainedEffectDef::Survival {
                        duration_ms: 5000,
                        policy: crate::game::ability::sustained::SurvivalPolicy::PreventDeathOnce,
                    },
                    2,
                )
            })
            .unwrap();
        direct(&mut f, DamageType::Magic, 20, 3);
        assert_eq!(f.core.units[&f.a].stats.current_health, 1);
        assert_eq!(pools(&f.core, f.a)[0].2, 200);
        assert!(f
            .core
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .unwrap()
            .protection
            .survival
            .is_some());
    }
    #[test]
    fn composition_adaptive_shield_actual_projectile_preserves_primary_and_requires_live_wearer() {
        use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
        for withdraw in [false, true] {
            let mut f = setup();
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
            attack(&mut f, 10, true);
            f.core
                .step_battle_execution_until(&mut f.state, 12)
                .unwrap();
            assert!(
                f.core
                    .projectiles
                    .values()
                    .any(|p| p.source_snapshot.source_id == f.a
                        && p.source_snapshot.basic_primary == Some(f.enemy)),
                "projectiles={:?}; hp={}; basic={:?}; events={:?}",
                f.core.projectiles,
                f.core.units[&f.enemy].stats.current_health,
                f.core.units[&f.a].basic_attack,
                f.core.event_log.entries
            );
            if withdraw {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: f.a,
                        time_ms: 12,
                    })
                    .unwrap();
            }
            f.core
                .units
                .get_mut(&f.enemy)
                .unwrap()
                .basic_attack
                .damage_type = DamageType::Magic;
            f.core
                .step_battle_execution_until(&mut f.state, 1200)
                .unwrap();
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 900);
            if withdraw {
                assert!(pools(&f.core, f.a).is_empty());
            } else {
                assert_eq!(witness(&f.core).damage_type, DamageType::Magic);
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: f.a,
                        time_ms: 1200,
                    })
                    .unwrap();
                assert!(
                    pools(&f.core, f.a).is_empty(),
                    "actual withdrawal removes the personal typed pool"
                );
            }
        }
    }
    #[test]
    fn composition_adaptive_shield_hello_line_primary_once_and_goodbye_excluded() {
        for hello_attack in [false, true] {
            let mut f = fixture_copies_with_setup(
                0,
                false,
                false,
                |doc| *doc = parse_yaml(SOURCE).unwrap(),
                |authored| {
                    *authored=r11_hello_goodbye_ron("Snapshot").replace(
                    "defense_tile_range: None, delivery: Instant)",
                    "defense_tile_range: None, delivery: Instant, line: Some((length_units:6500000,width_units:500000,power:40,weapon_effects:true,piercing_relics:false,piercing:None)))")
                },
                |_| {},
                None,
            );
            f.core.enable_authoring_observation();
            f.core
                .units
                .get_mut(&f.enemy)
                .unwrap()
                .basic_attack
                .damage_type = DamageType::Magic;
            super::super::compiled_action_fixture::place(&mut f.core, f.secondary, 4, 1);
            f.core
                .apply_live_command(BattleLiveCommand::ActivateSkill {
                    unit_id: f.a,
                    skill_id: "r11_aura".into(),
                    target: None,
                    time_ms: 0,
                })
                .unwrap();
            f.core.step_battle_execution_until(&mut f.state, 1).unwrap();
            let activation = f.core.sustained_effect_snapshot().unwrap().active[0].activation;
            if hello_attack {
                attack(&mut f, 10, true);
                f.core
                    .step_battle_execution_until(&mut f.state, 12)
                    .unwrap();
                assert_eq!(f.core.units[&f.enemy].stats.current_health, 960);
                assert_eq!(f.core.units[&f.secondary].stats.current_health, 960);
                assert_eq!(pools(&f.core, f.a).len(), 1);
                assert_eq!(witness(&f.core).damage_type, DamageType::Magic);
                assert_eq!(witness(&f.core).primary, f.enemy);
            }
            let previous = f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_adaptive_shield
                .as_ref()
                .map(|w| w.activation);
            f.core
                .apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
                    unit_id: f.a,
                    skill_id: "r11_aura".into(),
                    activation,
                    target: None,
                    time_ms: 20,
                })
                .unwrap();
            assert_eq!(
                f.core
                    .authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_adaptive_shield
                    .as_ref()
                    .map(|w| w.activation),
                previous
            );
            if !hello_attack {
                assert!(pools(&f.core, f.a).is_empty());
            }
            assert!(f
                .core
                .runtime_fact_summary()
                .latest_damage
                .as_ref()
                .unwrap()
                .source
                .cast
                .as_ref()
                .is_some_and(|c| c.skill_id.as_str() == "r11_goodbye"));
        }
    }
    #[test]
    fn composition_adaptive_shield_retained_refusal_restores_pool_gate_and_retry() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup();
        let source = captured(&mut f, 0);
        let mut probe = f.core.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.runtime_transaction(|c| hit(c, source.clone(), f.enemy, 1))
        });
        result.unwrap();
        let required = admissions
            .iter()
            .filter(|(site, _, _)| *site == "adaptive_shield_witness")
            .map(|(_, _, n)| *n)
            .max()
            .unwrap();
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = f.core.transaction_debug_snapshot();
        let obs = serde_json::to_value(&f.core.authoring_observation).unwrap();
        let error = f
            .core
            .runtime_transaction(|c| hit(c, source.clone(), f.enemy, 1))
            .unwrap_err();
        assert!(
            matches!(
                error,
                crate::game::behavior::GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(f.core.transaction_debug_snapshot(), before);
        assert_eq!(
            serde_json::to_value(&f.core.authoring_observation).unwrap(),
            obs
        );
        f.core.runtime_facts.limits = Default::default();
        f.core
            .runtime_transaction(|c| hit(c, source, f.enemy, 1))
            .unwrap();
        assert_eq!(witness(&f.core).capacity, 200);
    }
}
