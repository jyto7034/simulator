mod composition_equipment_evasion {
    use super::*;
    use crate::game::battle::core::damage_runtime::{CommittedDamageInput, CommittedDamagePolicy};
    const EVASION: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/equipment_evasion.skill.yaml"
    ));
    fn setup(chance: u8) -> Fixture {
        setup_weapon(chance, |_| {})
    }
    fn setup_weapon(
        chance: u8,
        configure: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
    ) -> Fixture {
        setup_extended(chance, configure, |_| {})
    }
    fn setup_extended(
        chance: u8,
        configure: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata),
        configure_scenario: impl FnOnce(&mut BattleScenario),
    ) -> Fixture {
        fixture_extra_with_defense(
            "Damage(amount:100, damage_type:True)",
            false,
            None,
            false,
            &EVASION.replace("chance_percent: 30", &format!("chance_percent: {chance}")),
            configure,
            configure_scenario,
        )
    }
    fn evasion(f: &Fixture) -> &crate::game::battle::core::equipment_defense::BasicEvasionWitness {
        f.core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_basic_evasion
            .as_ref()
            .unwrap()
    }
    fn hit(
        core: &mut BattleCore,
        source: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
    ) -> crate::game::battle::core::types::CommandExecutionSummary {
        let snapshot = core
            .damage_source_snapshot_for_unit(
                source,
                target,
                DamageSource::BasicAttack,
                DamageType::True,
                10,
                Default::default(),
                0,
                now,
                false,
            )
            .unwrap();
        core.execute_committed_damage(CommittedDamageInput {
            target_id: target,
            source_snapshot: snapshot,
            committed_at_ms: now,
            policy: CommittedDamagePolicy::BasicAttack {
                occurrence_id: Uuid::from_u128(0xe390),
                occurrence_time_ms: now,
            },
        })
        .unwrap()
    }
    fn basic(f: &mut Fixture, now: u64) {
        f.core
            .runtime_transaction(|c| {
                hit(c, f.caster, f.target, now);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
    }
    #[test]
    fn composition_equipment_evasion_schema_shared_origin_and_bounds() {
        let doc = parse_yaml(EVASION).unwrap();
        let artifact = compile_json(&doc.to_string()).unwrap();
        let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(matches!(
            loaded.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::BasicAttackEvasion {
                chance_percent: 30
            }
        ));
        for chance in [0, 100] {
            let mut valid = doc.clone();
            valid["values"][0]["value"]["DefenseProgram"]["BasicAttackEvasion"]["chance_percent"] =
                json!(chance);
            compile_json(&valid.to_string()).unwrap();
        }
        let mut invalid = doc;
        invalid["values"][0]["value"]["DefenseProgram"]["BasicAttackEvasion"]["chance_percent"] =
            json!(101);
        let error = compile_json(&invalid.to_string()).unwrap_err();
        assert!(
            error
                .source
                .definition
                .ends_with("DefenseProgram.BasicAttackEvasion.chance_percent"),
            "{error:?}"
        );
    }
    #[test]
    fn composition_equipment_evasion_zero_and_full_chance_skill_exclusion() {
        for chance in [0, 100] {
            let mut f = setup(chance);
            basic(&mut f, 0);
            assert_eq!(
                f.core.units[&f.target].stats.current_health,
                if chance == 100 { 1000 } else { 990 }
            );
            assert_eq!(evasion(&f).evaded, chance == 100);
            assert_eq!(evasion(&f).chance_percent, chance);
            assert_eq!(
                evasion(&f).equipment.persistent_copy_uuid,
                Some(Uuid::from_u128(0xe370))
            );
            let strike = evasion(&f).strike;
            resolve(&mut f, 10);
            assert_eq!(
                f.core.units[&f.target].stats.current_health,
                if chance == 100 { 900 } else { 890 }
            );
            assert_eq!(evasion(&f).strike, strike);
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_skill_barrier
                .is_none());
        }
    }
    #[test]
    fn composition_equipment_evasion_segments_share_roll_new_hits_roll_again() {
        let mut f = setup(30);
        let mut outcomes = Vec::new();
        for now in 0..12 {
            let before = f.core.units[&f.target].stats.current_health;
            f.core
                .runtime_transaction(|c| {
                    let scope = c.begin_strike_scope();
                    let one = hit(c, f.caster, f.target, now);
                    let witness = c
                        .authoring_observation
                        .as_ref()
                        .unwrap()
                        .latest_basic_evasion
                        .clone()
                        .unwrap();
                    let two = hit(c, f.caster, f.target, now);
                    assert_eq!(one.actual_hp_loss, two.actual_hp_loss);
                    assert_eq!(
                        c.authoring_observation
                            .as_ref()
                            .unwrap()
                            .latest_basic_evasion
                            .as_ref()
                            .unwrap()
                            .strike,
                        witness.strike
                    );
                    c.end_strike_scope(scope);
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            let w = evasion(&f);
            assert_eq!(
                before - f.core.units[&f.target].stats.current_health,
                if w.evaded { 0 } else { 20 }
            );
            outcomes.push((w.strike, w.roll_percent, w.evaded));
        }
        assert!(outcomes.windows(2).all(|pair| pair[0].0 != pair[1].0));
        assert!(outcomes.iter().any(|o| o.2));
        assert!(outcomes.iter().any(|o| !o.2));
        assert_eq!(
            &outcomes[..5],
            &[
                (0, 47, false),
                (1, 80, false),
                (2, 29, true),
                (3, 86, false),
                (4, 32, false)
            ]
        );
    }
    #[test]
    fn composition_equipment_evasion_precedes_shield_and_observer_is_passive() {
        let mut observed = setup(30);
        let mut plain = Fixture {
            core: observed.core.clone(),
            state: observed.state.clone(),
            caster: observed.caster,
            target: observed.target,
            ally: observed.ally,
        };
        plain.core.authoring_observation = None;
        for now in 0..12 {
            basic(&mut observed, now);
            basic(&mut plain, now);
            assert_eq!(
                observed.core.transaction_debug_snapshot(),
                plain.core.transaction_debug_snapshot()
            );
            assert_eq!(
                observed.core.event_queue_debug_snapshot(),
                plain.core.event_queue_debug_snapshot()
            );
        }
        let mut f = setup(100);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.target,
                    f.target,
                    crate::game::ability::sustained::SustainedEffectDef::Shield {
                        duration_ms: 30000,
                        amount: 250,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    0,
                )
            })
            .unwrap();
        basic(&mut f, 0);
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
        let shield: u32 = f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .active
            .iter()
            .filter(|v| v.target == f.target)
            .filter_map(|v| v.shield.as_ref().map(|s| s.remaining))
            .sum();
        assert_eq!(shield, 250);
    }

    fn queue_basic(f: &mut Fixture, now: u64) {
        super::super::compiled_action_fixture::place(&mut f.core, f.caster, 2, 1);
        super::super::compiled_action_fixture::place(&mut f.core, f.target, 3, 1);
        super::super::compiled_action_fixture::place(&mut f.core, f.ally, 7, 3);
        f.core.units.get_mut(&f.caster).unwrap().resonance_current = 0;
        f.core
            .units
            .get_mut(&f.caster)
            .unwrap()
            .next_basic_attack_ms = now;
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: now,
                    attacker_instance_id: f.caster,
                    target_instance_id: Some(f.target),
                    schedule_next: false,
                    cause: Default::default(),
                },
                now,
            )
            .unwrap();
        assert_eq!(
            f.core.units[&f.caster].current_target,
            Some(f.target),
            "actor={:?}; range={:?}",
            f.core.units[&f.caster],
            f.core.basic_action_snapshot(f.caster, true)
        );
    }
    #[test]
    fn composition_equipment_evasion_actual_melee_projectile_suppresses_hit_effects() {
        use crate::game::stats::{Effect, TriggerEffectTarget, TriggerType, TriggeredEffect};
        for projectile in [false, true] {
            for chance in [0, 100] {
                let mut f = setup_weapon(chance, |weapon| {
                    weapon.triggered_effects.insert(
                        TriggerType::OnAttack,
                        vec![TriggeredEffect::targeted(
                            TriggerEffectTarget::CounterpartUnit,
                            Effect::ApplyBuff {
                                buff_id: "r11_stun".into(),
                                duration_ms: 200,
                            },
                        )],
                    );
                    weapon.triggered_effects.insert(
                        TriggerType::OnHit,
                        vec![TriggeredEffect::targeted(
                            TriggerEffectTarget::CounterpartUnit,
                            Effect::BonusDamage {
                                flat: 8,
                                percent: 0,
                                damage_type: DamageType::True,
                            },
                        )],
                    );
                    if projectile {
                        weapon.weapon_profile.as_mut().unwrap().delivery =
                            crate::game::ability::DeliveryDef::Projectile {
                                speed_units_per_ms:
                                    crate::game::battle::core::movement::types::DATA_UNITS_PER_WORLD
                                        as u32,
                                hit_policy: crate::game::ability::ProjectileHitPolicy::TargetLocked,
                                allow_targetless_cast: false,
                                max_range_tiles: None,
                                max_lifetime_ms: None,
                                max_kills: None,
                                max_pierces: None,
                                collision: Default::default(),
                            };
                    }
                });
                queue_basic(&mut f, 1);
                f.core.step_battle_execution_until(&mut f.state, 5).unwrap();
                let w = evasion(&f);
                assert_eq!(w.evaded, chance == 100);
                assert_eq!(
                    f.core.units[&f.target].stats.current_health,
                    if chance == 100 { 1000 } else { 900 }
                );
                assert_eq!(
                    f.core.units[&f.caster].stats.current_health,
                    if chance == 100 { 1000 } else { 992 }
                );
                assert_eq!(
                    f.core.buffs.is_empty(),
                    chance == 100,
                    "{projectile}/{chance}"
                );
                assert_eq!(
                    f.core.units[&f.caster].resonance_current, 0,
                    "employees do not gain release/hit resonance under the current rule"
                );
                assert!(f.core.projectiles.is_empty());
            }
        }
    }
    #[test]
    fn composition_equipment_evasion_hostile_release_keeps_resonance() {
        let mut f = setup_extended(
            100,
            |_| {},
            |scenario| {
                let BattleUnitSource::TestFixture { profile, .. } =
                    &mut scenario.groups[0].spawns[0].draft.source
                else {
                    unreachable!()
                };
                // Fixed-defense melee enemies attack blockers/endpoints. This ranged
                // source can acquire the nearby employee without forging a block.
                profile.basic_attack.range_role =
                    crate::game::data::equipment_data::WeaponRangeRole::Ranged;
                profile.basic_attack.ranged_reposition_ms = 0;
            },
        );
        std::mem::swap(&mut f.caster, &mut f.target);
        f.core.units.get_mut(&f.caster).unwrap().stats.attack = 100;
        queue_basic(&mut f, 1);
        f.core.step_battle_execution_until(&mut f.state, 5).unwrap();
        assert!(evasion(&f).evaded);
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
        assert_eq!(f.core.units[&f.caster].resonance_current, 10);
    }
    #[test]
    fn composition_equipment_evasion_tick_environment_self_ally_and_reaction_excluded() {
        for source in [DamageSource::BuffTick, DamageSource::Environment] {
            let mut f = setup(100);
            raw_damage(&mut f, source, 0);
            assert_eq!(f.core.units[&f.target].stats.current_health, 990);
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_basic_evasion
                .is_none());
        }
        for target_self in [true, false] {
            let mut f = setup(100);
            let target = if target_self { f.caster } else { f.ally };
            f.core
                .runtime_transaction(|c| {
                    hit(c, f.caster, target, 0);
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            assert!(f
                .core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_basic_evasion
                .is_none());
        }
        let mut f = setup(100);
        f.core
            .runtime_transaction(|c| {
                let mut snapshot = c
                    .damage_source_snapshot_for_unit(
                        f.caster,
                        f.target,
                        DamageSource::BasicAttack,
                        DamageType::True,
                        10,
                        Default::default(),
                        0,
                        0,
                        false,
                    )
                    .unwrap();
                snapshot.fact_source.reaction = true;
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: f.target,
                        source_snapshot: snapshot,
                    }],
                    0,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(f.core.units[&f.target].stats.current_health, 990);
        assert!(f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_basic_evasion
            .is_none());
    }
    #[test]
    fn composition_equipment_evasion_precedes_parry_but_invulnerability_does_not_roll() {
        use crate::game::ability::sustained::SustainedEffectDef;
        let mut f = setup(100);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.target,
                    f.target,
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
        basic(&mut f, 0);
        assert_eq!(f.core.units[&f.caster].stats.current_health, 1000);
        assert!(
            !f.core.sustained_invulnerable(f.target, 0).unwrap(),
            "evasion must not consume parry"
        );
        resolve(&mut f, 1); // Skill bypasses evasion, consumes parry and starts its invulnerability.
        assert!(f.core.sustained_invulnerable(f.target, 2).unwrap());
        let strike = evasion(&f).strike;
        basic(&mut f, 3);
        assert_eq!(
            evasion(&f).strike,
            strike,
            "invulnerable hit must not roll evasion"
        );
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
    }
    #[test]
    fn composition_equipment_evasion_highest_chance_and_stable_copy_selection() {
        use crate::game::ability::equipment_defense::{
            EquipmentDefenseBinding, EquipmentDefenseProgram,
        };
        let mut f = setup(30);
        let original = f.core.game_data.equipment_data.items[0].clone();
        let mut stronger = original.clone();
        stronger.uuid = Uuid::from_u128(0xe391);
        stronger.id = "stronger_evasion".into();
        stronger.equipment_type = crate::game::data::equipment_data::EquipmentType::Armor;
        stronger.weapon_profile = None;
        let mut equal = stronger.clone();
        equal.uuid = Uuid::from_u128(0xe396);
        equal.id = "equal_evasion".into();
        equal.equipment_type = crate::game::data::equipment_data::EquipmentType::Accessory;
        let mut equipment = crate::game::data::equipment_data::EquipmentDatabase::new(vec![
            original, stronger, equal,
        ]);
        equipment.equipment_defenses = vec![
            EquipmentDefenseBinding {
                equipment: "r11_weapon".into(),
                program: EquipmentDefenseProgram::BasicAttackEvasion { chance_percent: 30 },
                source: None,
            },
            EquipmentDefenseBinding {
                equipment: "stronger_evasion".into(),
                program: EquipmentDefenseProgram::BasicAttackEvasion {
                    chance_percent: 100,
                },
                source: None,
            },
        ];
        equipment.equipment_defenses.push(EquipmentDefenseBinding {
            equipment: "equal_evasion".into(),
            program: EquipmentDefenseProgram::BasicAttackEvasion {
                chance_percent: 100,
            },
            source: None,
        });
        f.core.game_data = GameDataBuilder::empty()
            .with_equipment_data(std::sync::Arc::new(equipment))
            .with_skills((*f.core.game_data.skill_data).clone())
            .with_buffs((*f.core.game_data.buff_data).clone())
            .build_arc();
        let mut copy = f
            .core
            .items
            .values()
            .find(|i| i.owner_unit_instance == f.target)
            .unwrap()
            .clone();
        copy.base_uuid = Uuid::from_u128(0xe391);
        copy.persistent_copy_uuid = Some(Uuid::from_u128(0xe392));
        let lower = Uuid::from_u128(0xe393);
        for (id, base) in [
            (Uuid::from_u128(0xe394), Uuid::from_u128(0xe396)),
            (lower, Uuid::from_u128(0xe391)),
        ] {
            copy.instance_id = id;
            copy.base_uuid = base;
            f.core.items.insert(id, copy.clone());
        }
        basic(&mut f, 0);
        assert_eq!(evasion(&f).chance_percent, 100);
        assert_eq!(evasion(&f).equipment.runtime_item_instance_id, lower);
        assert_eq!(evasion(&f).definition_id, "stronger_evasion");
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
    }
    #[test]
    fn composition_equipment_evasion_cancel_and_retained_refusal_restore_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::runtime_control as native;
        let mut f = setup(100);
        queue_basic(&mut f, 1);
        let snapshot = |f: &Fixture| {
            (
                f.core.transaction_debug_snapshot(),
                f.core.event_queue_debug_snapshot(),
                format!("{:?}", f.state),
                serde_json::to_value(&f.core.authoring_observation).unwrap(),
            )
        };
        let before = snapshot(&f);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("equipment_evasion_resolved", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 2),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snapshot(&f), before);
        let mut probe = f.core.clone();
        let mut state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut state, 2)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "basic_evasion_observation")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = snapshot(&f);
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 2)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "basic_evasion_observation",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(snapshot(&f), before);
        f.core.runtime_facts.limits = Default::default();
        f.core.step_battle_execution_until(&mut f.state, 2).unwrap();
        assert_eq!(f.core.units[&f.target].stats.current_health, 1000);
        assert!(evasion(&f).evaded);
        assert_eq!(
            evasion(&f).roll_percent,
            probe
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_basic_evasion
                .as_ref()
                .unwrap()
                .roll_percent
        );
    }

    #[test]
    fn composition_equipment_evasion_native_cone_checks_each_target() {
        for chance in [0, 100] {
            let mut f = setup_extended(
                chance,
                |weapon| {
                    weapon.weapon_profile = Some(
                        ron::de::from_str(include_str!(concat!(
                            env!("CARGO_MANIFEST_DIR"),
                            "/tests/fixtures/skill_composition/larynx.weapon.ron"
                        )))
                        .unwrap(),
                    );
                },
                |scenario| {
                    let mut spare = scenario.groups[0].spawns[0].clone();
                    spare.unit_ref = crate::game::battle::scenario::ScenarioUnitRef::new(
                        "evasion-area-secondary",
                    );
                    spare.instance_salt += 10;
                    spare.position = Position::new(7, 3);
                    spare.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0xe395));
                    scenario.groups[0].spawns.push(spare);
                },
            );
            let other = f
                .core
                .units
                .values()
                .find(|u| u.owner == Side::Opponent && u.instance_id != f.target)
                .unwrap()
                .instance_id;
            super::super::compiled_action_fixture::place(&mut f.core, other, 4, 1);
            f.core.units.get_mut(&f.caster).unwrap().stats.attack = 100;
            queue_basic(&mut f, 1);
            f.core
                .step_battle_execution_until(&mut f.state, 81)
                .unwrap();
            for target in [f.target, other] {
                assert_eq!(
                    f.core.units[&target].stats.current_health,
                    if chance == 100 { 1000 } else { 900 },
                    "{target}/{chance}"
                );
            }
            assert_eq!(evasion(&f).evaded, chance == 100);
        }
    }
}
