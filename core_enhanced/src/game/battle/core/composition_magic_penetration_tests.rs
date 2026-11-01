mod magic_penetration {
    use super::*;
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/magic_penetration.skill.yaml"
    ));
    fn setup(edit: impl FnOnce(&mut Value)) -> Fixture {
        let source = SOURCE.replace("mp01_probe_accessory", "r11_weapon");
        let mut f = super::setup(|doc| {
            *doc = parse_yaml(&source).unwrap();
            edit(program(doc));
        });
        for id in [f.enemy, f.secondary] {
            let u = f.core.units.get_mut(&id).unwrap();
            u.stats.defense = 100;
            u.stats.magic_resist = 100;
        }
        f.core.units.get_mut(&f.a).unwrap().basic_attack.damage_type = DamageType::Magic;
        f
    }
    fn shot(f: &mut Fixture) -> DamageSourceSnapshot {
        snapshot(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::Magic,
            DamageSource::Ability,
            10,
        )
    }
    fn apply_shot(f: &mut Fixture, shot: DamageSourceSnapshot) -> u32 {
        f.core
            .runtime_transaction(|c| {
                let amount = apply(c, shot, f.enemy, 10);
                crate::game::battle::runtime_control::poll()?;
                Ok(amount)
            })
            .unwrap()
    }
    fn hit(f: &mut Fixture) -> u32 {
        let s = shot(f);
        apply_shot(f, s)
    }
    #[test]
    fn magic_penetration_shared_authoring_roundtrip_and_each_scalar_diagnostic() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        let p = &restored.equipment_attacks()[0].program;
        assert_eq!(p.direct_magic_penetration_percent, Some(25));
        assert_eq!(p.direct_magic_penetration_flat, Some(10));
        assert_eq!(p.direct_physical_penetration_percent, None);
        for (field, valid, invalid) in [
            ("direct_magic_penetration_percent", 100u64, 101u64),
            ("direct_physical_penetration_percent", 100, 101),
            (
                "direct_magic_penetration_flat",
                i32::MAX as u64,
                i32::MAX as u64 + 1,
            ),
            (
                "direct_physical_penetration_flat",
                i32::MAX as u64,
                i32::MAX as u64 + 1,
            ),
        ] {
            let mut doc = parse_yaml(SOURCE).unwrap();
            *program(&mut doc) = json!({"reactions":[],field:valid});
            assert!(compile_json(&doc.to_string()).is_ok(), "{field}");
            program(&mut doc)[field] = json!(0);
            assert!(compile_json(&doc.to_string()).is_ok());
            for bad in [json!(invalid), json!(-1), json!(0.5), json!("unresolved")] {
                program(&mut doc)[field] = bad;
                let err = compile_json(&doc.to_string()).unwrap_err();
                assert!(format!("{err:?}").contains(field), "{field}: {err:?}");
            }
        }
        let old = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/direct_penetration.skill.yaml"
        ));
        let old = crate::authoring::composition::compile_yaml(old).unwrap();
        let serialized = serde_json::to_value(&old.equipment_attacks()[0].program).unwrap();
        assert!(serialized.get("direct_magic_penetration_percent").is_none());
        assert!(serialized.get("direct_physical_penetration_flat").is_none());
    }
    #[test]
    fn magic_penetration_percent_then_flat_live_resistance_and_combined_cap() {
        for (resistance, percent, flat, old_percent, old_flat, want) in [
            (100, 25, 10, 0, 0, 60), //100 ->75 ->65; floor(10000/165).
            (100, 50, 10, 0, 0, 71),
            (100, 25, 10, 25, 5, 74), //50% then15: resistance35.
            (100, 75, 0, 75, 0, 100),
            (40, 25, 10, 0, 0, 83),
            (0, 50, 10, 0, 0, 100),
            (-100, 50, 10, 0, 0, 150),
            (100, 100, 10, 0, 0, 110), //Existing fixed penetration may cross zero.
        ] {
            let mut f = setup(|p| {
                p["direct_magic_penetration_percent"] = json!(percent);
                p["direct_magic_penetration_flat"] = json!(flat);
            });
            f.core.units.get_mut(&f.enemy).unwrap().stats.magic_resist = resistance;
            let mut s = shot(&mut f);
            s.modifiers.magic_resist_penetration_percent = old_percent;
            s.modifiers.magic_resist_penetration_flat = old_flat;
            assert_eq!(
                apply_shot(&mut f, s),
                want,
                "resist{resistance} pct{percent} flat{flat}"
            );
            assert_eq!(f.core.units[&f.enemy].stats.magic_resist, resistance);
        }
        let mut f = setup(|p| {
            p["direct_magic_penetration_percent"] = json!(75);
            p["direct_magic_penetration_flat"] = json!(0);
        });
        for item in f.core.items.values_mut() {
            item.owner_unit_instance = f.a;
        }
        let mut s = shot(&mut f);
        s.modifiers.magic_resist_penetration_percent = -50;
        assert_eq!(apply_shot(&mut f, s), 100); //75+75-50; no early100 cap.
        let w = f
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_direct_penetration
            .as_ref()
            .unwrap();
        assert_eq!(w.magic_percent, 150);
        assert_eq!(w.percent, 0);
        assert_eq!(w.contributors.len(), 4);
    }
    #[test]
    fn magic_penetration_mixed_components_only_actual_actor_and_matching_type() {
        use crate::game::battle::{cooldown::CooldownSource, damage::DamageBonusSnapshot};
        for base in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            let mut f = setup(|p| {
                p["direct_physical_penetration_percent"] = json!(50);
                p["direct_physical_penetration_flat"] = json!(10);
            });
            let mut s = snapshot(&mut f.core, f.a, f.enemy, base, DamageSource::Ability, 10);
            for (actor, kind) in [
                (f.a, DamageType::Physical),
                (f.a, DamageType::Magic),
                (f.a, DamageType::True),
                (f.b, DamageType::Physical),
                (f.b, DamageType::Magic),
            ] {
                s.on_attack_bonus_damage.push(DamageBonusSnapshot {
                    source: CooldownSource::Unit {
                        unit_instance_id: actor,
                    },
                    equipment_source: None,
                    flat: 100,
                    percent: 0,
                    damage_type: kind,
                });
            }
            let base = match base {
                DamageType::Physical => 71,
                DamageType::Magic => 60,
                DamageType::True => 100,
            };
            assert_eq!(apply_shot(&mut f, s), base + 71 + 60 + 100 + 50 + 50);
        }
    }
    #[test]
    fn magic_penetration_direct_sources_reactions_and_other_types() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            for source in [
                DamageSource::BasicAttack,
                DamageSource::Ability,
                DamageSource::BuffTick,
                DamageSource::Environment,
            ] {
                for reaction in [false, true] {
                    let mut f = setup(|_| {});
                    let mut s = snapshot(&mut f.core, f.a, f.enemy, kind, source, 10);
                    s.fact_source.reaction = reaction;
                    let want = if kind == DamageType::True {
                        100
                    } else if kind == DamageType::Magic
                        && !reaction
                        && matches!(source, DamageSource::BasicAttack | DamageSource::Ability)
                    {
                        60
                    } else {
                        50
                    };
                    assert_eq!(
                        apply_shot(&mut f, s),
                        want,
                        "{kind:?}/{source:?}/{reaction}"
                    );
                }
            }
        }
    }
    #[test]
    fn magic_penetration_live_equipment_corrosion_and_withdrawn_projectile() {
        let mut f = setup(|_| {});
        assert_eq!(hit(&mut f), 60);
        f.core.units.get_mut(&f.a).unwrap().corroded = true;
        assert_eq!(hit(&mut f), 50);
        f.core.units.get_mut(&f.a).unwrap().corroded = false;
        assert_eq!(hit(&mut f), 60);
        f.core.items.retain(|_, i| i.owner_unit_instance != f.a);
        assert_eq!(hit(&mut f), 50);
        let mut f = setup(|_| {});
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
    fn magic_penetration_cancel_and_observation_limit_restore_entire_strike() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        use crate::game::battle::runtime_control as native;
        let mut f = setup(|_| {});
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 89)
            .unwrap();
        let before = (state(&f.core), format!("{:?}", f.state));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("direct_penetration_evaluated", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 90),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(before, (state(&f.core), format!("{:?}", f.state)));
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
        let mut plain = f.core.clone();
        plain.authoring_observation = None;
        let mut execution = f.state.clone();
        f.core
            .step_battle_execution_until(&mut f.state, 90)
            .unwrap();
        plain
            .step_battle_execution_until(&mut execution, 90)
            .unwrap();
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9940);
        assert_eq!(plain.units[&f.enemy].stats.current_health, 9940);
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
    fn magic_penetration_preserves_shield_type_immunity_and_invulnerability() {
        for mode in 0..3 {
            let mut f = setup(|_| {});
            if mode == 0 {
                f.core
                    .runtime_transaction(|c| {
                        c.begin_sustained(
                            f.enemy,
                            f.enemy,
                            SustainedEffectDef::Shield {
                                duration_ms: 1000,
                                amount: 100,
                                priority: 0,
                                shared_radius_units: None,
                            },
                            0,
                        )
                    })
                    .unwrap();
                assert_eq!(hit(&mut f), 0);
                assert_eq!(hit(&mut f), 20); //60+60 consumes100 shield before20HP.
            } else if mode == 1 {
                let mut s = shot(&mut f);
                s.modifiers.magic_damage_reduction_percent = 100;
                assert_eq!(apply_shot(&mut f, s), 0);
            } else {
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
                assert_eq!(hit(&mut f), 0);
                assert!(f.core.sustained_invulnerable(f.enemy, 11).unwrap());
                assert_eq!(hit(&mut f), 0);
            }
        }
    }
}
