mod direct_penetration {
    use super::*;
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/direct_penetration.skill.yaml"
    ));
    fn setup(percent: u32) -> Fixture {
        let source = SOURCE.replace("placeholder_clouded_monk_accessory", "r11_weapon");
        let mut f = super::setup(|doc| {
            *doc = parse_yaml(&source).unwrap();
            program(doc)["direct_physical_penetration_percent"] = json!(percent);
        });
        for id in [f.enemy, f.secondary] {
            let u = f.core.units.get_mut(&id).unwrap();
            u.stats.defense = 100;
            u.stats.magic_resist = 100;
        }
        f
    }
    fn hit_shot(f: &mut Fixture, shot: DamageSourceSnapshot) -> u32 {
        f.core
            .runtime_transaction(|c| {
                let amount = apply(c, shot, f.enemy, 10);
                crate::game::battle::runtime_control::poll()?;
                Ok(amount)
            })
            .unwrap()
    }
    #[test]
    fn direct_penetration_authoring_roundtrip_and_percentage_diagnostics() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let artifact = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(
            artifact.equipment_attacks()[0]
                .program
                .direct_physical_penetration_percent,
            Some(25)
        );
        for value in ["101", "-1", "25.5", "nullx"] {
            let bad = SOURCE.replace("percent: 25", &format!("percent: {value}"));
            let err = crate::authoring::composition::compile_yaml(&bad).unwrap_err();
            assert!(
                format!("{err:?}").contains("direct_physical_penetration_percent"),
                "{err:?}"
            );
        }
        for percent in [0, 25, 50, 100] {
            let mut f = setup(percent);
            let want = match percent {
                0 => 50,
                25 => 57,
                50 => 66,
                _ => 100,
            };
            assert_eq!(hit(&mut f, 10), want);
        }
    }
    #[test]
    fn direct_penetration_types_direct_sources_and_reaction_exclusions() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            for source in [
                DamageSource::BasicAttack,
                DamageSource::Ability,
                DamageSource::BuffTick,
                DamageSource::Environment,
            ] {
                for reaction in [false, true] {
                    let mut f = setup(50);
                    let mut shot = snapshot(&mut f.core, f.a, f.enemy, kind, source, 10);
                    shot.fact_source.reaction = reaction;
                    let want = if kind == DamageType::True {
                        100
                    } else if kind == DamageType::Physical
                        && !reaction
                        && matches!(source, DamageSource::BasicAttack | DamageSource::Ability)
                    {
                        66
                    } else {
                        50
                    };
                    assert_eq!(
                        hit_shot(&mut f, shot),
                        want,
                        "{kind:?}/{source:?}/{reaction}"
                    );
                }
            }
        }
    }
    #[test]
    fn direct_penetration_mixed_components_keep_type_and_actor() {
        use crate::game::battle::{cooldown::CooldownSource, damage::DamageBonusSnapshot};
        for base in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            let mut f = setup(50);
            let mut shot = snapshot(&mut f.core, f.a, f.enemy, base, DamageSource::Ability, 10);
            for (actor, kind) in [
                (f.a, DamageType::Physical),
                (f.a, DamageType::Magic),
                (f.a, DamageType::True),
                (f.b, DamageType::Physical),
            ] {
                shot.on_attack_bonus_damage.push(DamageBonusSnapshot {
                    source: CooldownSource::Unit {
                        unit_instance_id: actor,
                    },
                    equipment_source: None,
                    flat: 100,
                    percent: 0,
                    damage_type: kind,
                });
            }
            let base_amount = match base {
                DamageType::Physical => 66,
                DamageType::Magic => 50,
                DamageType::True => 100,
            };
            assert_eq!(hit_shot(&mut f, shot), base_amount + 266, "{base:?}");
        }
    }
    #[test]
    fn direct_penetration_resistance_then_percentage_then_flat_and_nonpositive_preserved() {
        for (armor, base_percent, flat, want) in [
            (100, 25, 10, 86), //100 ->25 ->15, floor(10000/115).
            (100, 75, 0, 100), //50+75 is capped at100.
            (40, 0, 0, 83),    //Current reduced DEF40 ->20; no target debuff.
            (0, 0, 0, 100),
            (-100, 0, 0, 150), //Do not weaken existing negative-defense benefit.
        ] {
            let mut f = setup(50);
            f.core.units.get_mut(&f.enemy).unwrap().stats.defense = armor;
            let mut shot = snapshot(
                &mut f.core,
                f.a,
                f.enemy,
                DamageType::Physical,
                DamageSource::Ability,
                10,
            );
            shot.modifiers.armor_penetration_percent = base_percent;
            shot.modifiers.armor_penetration_flat = flat;
            assert_eq!(hit_shot(&mut f, shot), want, "DEF{armor}");
            assert_eq!(f.core.units[&f.enemy].stats.defense, armor);
        }
    }
    #[test]
    fn direct_penetration_equipment_removal_and_corrosion_disable_new_hits() {
        let mut f = setup(50);
        assert_eq!(hit(&mut f, 1), 66);
        f.core.units.get_mut(&f.a).unwrap().corroded = true;
        assert_eq!(hit(&mut f, 2), 50);
        f.core.units.get_mut(&f.a).unwrap().corroded = false;
        assert_eq!(hit(&mut f, 3), 66);
        f.core
            .items
            .retain(|_, item| item.owner_unit_instance != f.a);
        assert_eq!(hit(&mut f, 4), 50);
    }
    #[test]
    fn direct_penetration_cancel_restores_attack_and_allows_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(50);
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 89)
            .unwrap();
        let before = (state(&f.core), format!("{:?}", f.state));
        let token = CancellationToken::default();
        let result =
            control::with_test_cancellation("direct_penetration_evaluated", 1, || {
                control::run(
                    &ExecutionLimits::default(),
                    &token,
                    ExecutionPhase::TrialExecution,
                    || f.core.step_battle_execution_until(&mut f.state, 90),
                )
            });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, (state(&f.core), format!("{:?}", f.state)));
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9934);
    }
    #[test]
    fn direct_penetration_proc_guard_and_independent_actor_are_excluded() {
        use crate::game::battle::{
            cooldown::CooldownSource,
            runtime_facts::{IndependentObjectOriginSnapshot, RuntimeFactSource},
            skill_source::{SkillCastExecutionSource, SkillProcOrigin},
        };
        let mut f = setup(50);
        for case in 0..3 {
            let mut shot = snapshot(
                &mut f.core,
                f.a,
                f.enemy,
                DamageType::Physical,
                DamageSource::Ability,
                10,
            );
            if case == 0 {
                shot.fact_source.cast = Some(std::sync::Arc::new(SkillCastExecutionSource {
                    skill_id: "penetration-proc".into(),
                    caster_instance_id: f.a,
                    cast_seq: 1,
                    ability_event_seq: 1,
                    cause: Default::default(),
                    proc_origin: Some(SkillProcOrigin {
                        event_seq: 1,
                        activation_source: CooldownSource::Unit {
                            unit_instance_id: f.a,
                        },
                        binding_index: 0,
                    }),
                }));
            } else if case == 1 {
                shot.fact_source = shot
                    .fact_source
                    .with_guard_release(crate::game::ability::guard_charge::GuardReleaseOrigin {
                        relic_copy: Uuid::from_u128(95),
                        relic_definition: "penetration-guard".into(),
                        definition: "guard".into(),
                        consumed: 100,
                        added_power: 100,
                        primary: true,
                    })
                    .unwrap();
            } else {
                let object = UnitInstanceId(Uuid::from_u128(0x9500));
                shot.source_id = object;
                shot.fact_source.actor = Some(object);
                shot.fact_source.extended_observation = RuntimeFactSource::extended_observation(
                    Some(std::sync::Arc::new(IndependentObjectOriginSnapshot {
                        object_id: object,
                        definition_id:
                            crate::game::data::independent_object_data::IndependentObjectId(
                                "penetration-child".into(),
                            ),
                        creator: f.a,
                        creation_action: None,
                        creation_cast: None,
                        creation_fragment: None,
                        creation_equipment: None,
                        creation_authored: None,
                    })),
                    None,
                )
                .unwrap();
            }
            assert_eq!(hit_shot(&mut f, shot), 50, "excluded source {case}");
        }
        assert_eq!(hit(&mut f, 11), 66);
    }
    #[test]
    fn direct_penetration_retained_refusal_restores_and_retries_with_finite_limit() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(50);
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 89)
            .unwrap();
        let mut probe = f.core.clone();
        let mut execution = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut execution, 90)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "direct_penetration_observation")
            .unwrap()
            .2;
        let limit = f.core.runtime_facts.limits.max_retained_bytes;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = (state(&f.core), format!("{:?}", f.state));
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "direct_penetration_observation",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(before, (state(&f.core), format!("{:?}", f.state)));
        f.core.runtime_facts.limits.max_retained_bytes = limit;
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9934);
    }
    #[test]
    fn direct_penetration_clone_and_observation_preserve_damage_and_queue() {
        let mut f = setup(50);
        attack(&mut f, 10, true);
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        let mut execution = f.state.clone();
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        plain
            .step_battle_execution_until(&mut execution, 90)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9934);
        assert_eq!(plain.units[&f.enemy].stats.current_health, 9934);
        assert_eq!(
            serde_json::to_value(&f.core.event_log).unwrap(),
            serde_json::to_value(&plain.event_log).unwrap()
        );
        assert_eq!(
            serde_json::to_value(f.core.runtime_fact_view()).unwrap(),
            serde_json::to_value(plain.runtime_fact_view()).unwrap()
        );
        assert_eq!(
            f.core.event_queue_debug_snapshot(),
            plain.event_queue_debug_snapshot()
        );
    }
    #[test]
    fn direct_penetration_withdraw_after_release_keeps_projectile_without_live_bonus() {
        let mut f = setup(50);
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
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9950);
    }

    #[test]
    fn direct_penetration_multiple_copies_sum_before_combined_cap() {
        let mut f = setup(75);
        for item in f.core.items.values_mut() {
            item.owner_unit_instance = f.a;
        }
        let mut shot = snapshot(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::Physical,
            DamageSource::Ability,
            10,
        );
        shot.modifiers.armor_penetration_percent = -50;
        assert_eq!(hit_shot(&mut f, shot), 100); //75+75-50=100, not early cap100-50.
        let w = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_direct_penetration
            .as_ref()
            .unwrap();
        assert_eq!(w.percent, 150);
        assert_eq!(w.contributors.len(), 2);
        assert_ne!(
            w.contributors[0].equipment.persistent_copy_uuid,
            w.contributors[1].equipment.persistent_copy_uuid
        );
    }
}
