mod composition_primary_hp {
    use super::*;
    use crate::game::battle::{
        core::damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
        damage::{BattleCommand, DamageSource, DamageSourceSnapshot, DamageType},
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/primary_hp.skill.yaml"
    ));
    fn setup(edit: impl FnOnce(&mut Value)) -> Fixture {
        let mut f = fixture(0, false, |doc| {
            *doc = parse_yaml(SOURCE).unwrap();
            edit(doc);
        });
        for unit in f.core.units.values_mut() {
            unit.stats.magic_resist = 0;
        }
        f.core.enable_authoring_observation();
        f.core
            .bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: 1,
                    battle_uuid: Uuid::from_u128(0x7010),
                },
            )
            .unwrap();
        f
    }
    fn captured(f: &mut Fixture) -> DamageSourceSnapshot {
        attack(f, 10, true);
        let mut source = f
            .core
            .event_queue
            .iter()
            .filter_map(|event| match event.as_ref() {
                BattleEvent::AttackResolve {
                    attacker_instance_id,
                    source_snapshot,
                    ..
                } if *attacker_instance_id == f.a => Some(source_snapshot.clone()),
                _ => None,
            })
            .max_by_key(|s| s.committed_at_ms)
            .unwrap();
        // Explicit Native damage input; the action/primary came from real admission.
        source.base_damage = 100;
        source.damage_type = DamageType::Physical;
        source
    }
    fn hit(
        core: &mut BattleCore,
        source: DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        core.runtime_transaction(|core| {
            core.execute_committed_damage(CommittedDamageInput {
                target_id: target,
                source_snapshot: source,
                committed_at_ms: now,
                policy: CommittedDamagePolicy::GeneralCommand,
            })
            .ok_or(GameError::InvalidAction)?;
            crate::game::battle::runtime_control::poll()
        })
    }
    fn witness(
        core: &BattleCore,
    ) -> &crate::game::battle::core::equipment_primary_damage::PrimaryDamageWitness {
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_primary_damage
            .as_ref()
            .unwrap()
    }
    fn snap(core: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn composition_primary_hp_roundtrip_precise_diagnostics_and_nonweapon_provider() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(artifact.equipment_attacks()[0]
            .program
            .primary_damage_percent
            .is_some());
        for bad in [
            SOURCE.replace("moment: Live", "moment: Snapshot"),
            SOURCE.replace("subject: Target", "subject: Source"),
            SOURCE.replace("unit: Scalar", "unit: Milliseconds"),
        ] {
            let error = crate::authoring::composition::compile_yaml(&bad).unwrap_err();
            assert!(
                error.source.definition.contains("primary_damage_percent"),
                "{error:?}"
            );
        }
        let f = setup(|_| {});
        let mut equipment = (*f.core.game_data.equipment_data).clone();
        let item = equipment
            .items
            .iter_mut()
            .find(|i| i.id == "r11_weapon")
            .unwrap();
        item.equipment_type = crate::game::data::equipment_data::EquipmentType::Accessory;
        item.weapon_profile = None;
        crate::game::ability::equipment_attack::EquipmentAttackCatalog::compile(
            &equipment,
            &ResourceCatalog { resources: vec![] },
            &RelicDatabase::default(),
            &crate::game::value_expr::ExprLimits::default(),
        )
        .unwrap();
    }
    #[test]
    fn composition_primary_hp_live_ratio_rounding_cap_and_authored_variant() {
        for (current, expected, percent) in [
            (1000, 100, 0),
            (750, 125, 25),
            (667, 133, 33),
            (333, 150, 50),
        ] {
            let mut f = setup(|_| {});
            let source = captured(&mut f);
            f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = current;
            hit(&mut f.core, source, f.enemy, 11).unwrap();
            assert_eq!(
                f.core.units[&f.enemy].stats.current_health,
                current - expected
            );
            assert_eq!(witness(&f.core).percent, percent);
            assert_eq!(witness(&f.core).target_current_hp, current);
            assert_eq!(
                witness(&f.core).contributors[0]
                    .equipment
                    .persistent_copy_uuid,
                Some(match owner(&f.core, f.a) {
                    ResourceOwner::EquipmentCopy(id) => id,
                    _ => unreachable!(),
                })
            );
        }
        let mut f = setup(|doc| {
            *doc = parse_yaml(&SOURCE.replace("value: 100, unit", "value: 200, unit")).unwrap()
        });
        let source = captured(&mut f);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 750;
        hit(&mut f.core, source, f.enemy, 11).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 600);
    }
    #[test]
    fn composition_primary_hp_multistrike_healing_and_maximum_use_current_values() {
        let mut f = setup(|_| {});
        let source = captured(&mut f);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 800;
        hit(&mut f.core, source.clone(), f.enemy, 11).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 680);
        hit(&mut f.core, source.clone(), f.enemy, 12).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 548);
        f.core.process_commands(
            vec![BattleCommand::ApplyHeal {
                target_id: f.enemy,
                flat: 352,
                percent: 0,
                source_id: None,
            }],
            13,
        );
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 900);
        hit(&mut f.core, source.clone(), f.enemy, 14).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 790);
        f.core.units.get_mut(&f.enemy).unwrap().stats.max_health = 2000;
        hit(&mut f.core, source, f.enemy, 15).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 640);
    }
    #[test]
    fn composition_primary_hp_secondary_skill_proc_environment_and_unadmitted_excluded() {
        for mode in 0..7 {
            let mut f = setup(|_| {});
            let mut source = captured(&mut f);
            let target = if mode == 0 { f.secondary } else { f.enemy };
            f.core.units.get_mut(&target).unwrap().stats.current_health = 500;
            match mode {
                1 => source.source = DamageSource::Ability,
                2 => source.source = DamageSource::Environment,
                3 => source.fact_source.reaction = true,
                4 => source.fact_source.action = None,
                5 => source.basic_primary = None,
                6 => f.core.units.get_mut(&f.a).unwrap().corroded = true,
                _ => (),
            }
            hit(&mut f.core, source, target, 11).unwrap();
            assert_eq!(
                f.core.units[&target].stats.current_health, 400,
                "mode {mode}"
            );
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_primary_damage
                .is_none());
        }
    }
    #[test]
    fn composition_primary_hp_direct_component_only_and_existing_additive_modifier() {
        let mut f = setup(|_| {});
        let mut source = captured(&mut f);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 750;
        source.modifiers.base_damage_amp_percent = 20;
        source
            .on_attack_bonus_damage
            .push(crate::game::battle::damage::DamageBonusSnapshot {
                source: crate::game::battle::cooldown::CooldownSource::Unit {
                    unit_instance_id: f.a,
                },
                equipment_source: None,
                flat: 50,
                percent: 0,
                damage_type: DamageType::Magic,
            });
        hit(&mut f.core, source, f.enemy, 11).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 555); // 145 direct + 50 separate.
    }
    #[test]
    fn composition_primary_hp_cancel_and_invalid_expression_restore_damage_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(|_| {});
        let source = captured(&mut f);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 750;
        let before = snap(&f.core);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("primary_damage_evaluated", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || hit(&mut f.core, source.clone(), f.enemy, 11),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snap(&f.core), before);
        hit(&mut f.core, source, f.enemy, 11).unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 625);
        for value in [-1, 10001] {
            let mut bad = setup(|doc| {
                program(doc)["primary_damage_percent"]["root"] =
                    json!({"Const":{"value":value,"unit":"Scalar"}})
            });
            let source = captured(&mut bad);
            let before = snap(&bad.core);
            assert!(hit(&mut bad.core, source, bad.enemy, 11).is_err());
            assert_eq!(snap(&bad.core), before);
        }
    }
    #[test]
    fn composition_primary_hp_observer_clone_and_retained_refusal_preserve_authority() {
        let mut f = setup(|_| {});
        let source = captured(&mut f);
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 750;
        let mut off = f.core.clone();
        off.authoring_observation = None;
        let mut probe = f.core.clone();
        let (result, admissions) =
            crate::game::battle::runtime_control::observe_retained_admissions(|| {
                hit(&mut probe, source.clone(), f.enemy, 11)
            });
        result.unwrap();
        let needed = admissions
            .iter()
            .find(|(site, _, _)| *site == "primary_damage_observation")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = needed - 1;
        let before = snap(&f.core);
        let error = hit(&mut f.core, source.clone(), f.enemy, 11).unwrap_err();
        assert!(matches!(error, GameError::BattleRuntimeFault(crate::game::battle::runtime_control::BattleRuntimeFault {
            code: crate::game::battle::runtime_control::RuntimeFaultCode::RetainedByteLimit,
            site: "primary_damage_observation", ..
        })), "{error:?}");
        assert_eq!(snap(&f.core), before);
        f.core.runtime_facts.limits = Default::default();
        hit(&mut f.core, source.clone(), f.enemy, 11).unwrap();
        hit(&mut off, source, f.enemy, 11).unwrap();
        assert_eq!(snap(&f.core), snap(&off));
    }
    #[test]
    fn composition_primary_hp_actual_projectile_rechecks_target_and_preserves_release_after_withdraw(
    ) {
        use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
        for withdraw in [false, true] {
            let mut f = setup(|_| {});
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
            assert!(f
                .core
                .projectiles
                .values()
                .any(|p| p.source_snapshot.basic_primary == Some(f.enemy)));
            // HP changes after release; impact must read this current state.
            f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 750;
            if withdraw {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: f.a,
                        time_ms: 12,
                    })
                    .unwrap();
            }
            f.core
                .step_battle_execution_until(&mut f.state, 1200)
                .unwrap();
            assert_eq!(
                f.core.units[&f.enemy].stats.current_health,
                if withdraw { 650 } else { 625 }
            );
            if withdraw {
                assert!(f
                    .core
                    .authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_primary_damage
                    .is_none());
            } else {
                assert_eq!(witness(&f.core).percent, 25);
            }
        }
    }
}
