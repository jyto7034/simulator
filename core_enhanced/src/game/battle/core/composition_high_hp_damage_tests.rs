mod high_hp_damage {
    use super::*;
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/high_hp_damage.skill.yaml"
    ));
    fn setup(edit: impl FnOnce(&mut Value)) -> Fixture {
        let source = SOURCE.replace("placeholder_clouded_monk_armor", "r11_weapon");
        let mut f = super::setup(|doc| {
            *doc = parse_yaml(&source).unwrap();
            edit(doc);
        });
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 800;
        f
    }
    fn damage(
        f: &mut Fixture,
        kind: DamageType,
        source: DamageSource,
        reaction: bool,
        now: u64,
    ) -> u32 {
        let mut shot = snapshot(&mut f.core, f.a, f.enemy, kind, source, now);
        shot.fact_source.reaction = reaction;
        f.core
            .runtime_transaction(|c| {
                let result = apply(c, shot, f.enemy, now);
                crate::game::battle::runtime_control::poll()?;
                Ok(result)
            })
            .unwrap()
    }
    #[test]
    fn high_hp_damage_schema_roundtrip_filter_errors_and_incoming_rejection() {
        let a = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let a = Artifact::from_ron(&a.to_ron().unwrap()).unwrap();
        let filter = a.equipment_attacks()[0]
            .program
            .conditional_damage
            .as_ref()
            .unwrap()
            .damage_filter
            .as_ref()
            .unwrap();
        assert_eq!(
            filter.damage_types,
            vec![DamageType::Physical, DamageType::Magic]
        );
        for bad in [
            SOURCE.replace("[Physical, Magic]", "[]"),
            SOURCE.replace("[Physical, Magic]", "[Physical, Physical]"),
            SOURCE.replace("scope: Wearer", "scope: Unknown"),
            SOURCE.replace("[Physical, Magic]", "[Fire]"),
        ] {
            let err = crate::authoring::composition::compile_yaml(&bad).unwrap_err();
            assert!(format!("{err:?}").contains("damage_filter"), "{err:?}");
        }
        let mut doc = parse_yaml(SOURCE).unwrap();
        let policy = program(&mut doc)["conditional_damage"].clone();
        doc["values"][0]["value"] = json!({"DefenseProgram":{"ConditionalReduction":policy}});
        doc["equipment_attacks"] = json!([]);
        doc["equipment_defenses"] =
            json!([{"equipment":"r11_weapon","program":{"Shared":"high-hp-damage"}}]);
        let err = compile_json(&doc.to_string()).unwrap_err();
        assert!(format!("{err:?}").contains("damage_filter"), "{err:?}");
    }
    #[test]
    fn high_hp_damage_boundary_current_maximum_and_authored_increase() {
        for increase in [20, 40] {
            let mut f = setup(|doc| {
                program(doc)["conditional_damage"]["percent"]["root"]["Const"]["value"] =
                    json!(increase)
            });
            for (hp, want) in [
                (799, 100),
                (800, 100 + increase),
                (1000, 100 + increase),
                (1, 100),
                (900, 100 + increase),
            ] {
                f.core.units.get_mut(&f.a).unwrap().stats.current_health = hp;
                assert_eq!(hit(&mut f, 1), want);
                assert_eq!(witness(&f.core).wearer_hp, hp);
                assert_eq!(witness(&f.core).true_percent, 0);
                assert_eq!(f.core.units[&f.a].stats.attack, 100);
            }
            f.core.units.get_mut(&f.a).unwrap().stats.max_health = 2000;
            assert_eq!(hit(&mut f, 2), 100);
            f.core.units.get_mut(&f.a).unwrap().stats.current_health = 1600;
            assert_eq!(hit(&mut f, 3), 100 + increase);
        }
    }
    #[test]
    fn high_hp_damage_physical_magic_direct_periodic_and_excluded_true_reaction_environment() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            for source in [
                DamageSource::BasicAttack,
                DamageSource::Ability,
                DamageSource::BuffTick,
                DamageSource::Environment,
            ] {
                for reaction in [false, true] {
                    let mut f = setup(|_| {});
                    let wanted = if kind != DamageType::True
                        && source != DamageSource::Environment
                        && !reaction
                    {
                        120
                    } else {
                        100
                    };
                    assert_eq!(
                        damage(&mut f, kind, source, reaction, 10),
                        wanted,
                        "{kind:?}/{source:?}/{reaction}"
                    );
                }
            }
        }
    }
    #[test]
    fn high_hp_damage_mixed_type_strike_freezes_hp_for_every_order_without_true_leak() {
        for first in [DamageType::True, DamageType::Magic, DamageType::Physical] {
            for high in [false, true] {
                let mut f = setup(|_| {});
                f.core.units.get_mut(&f.a).unwrap().stats.current_health =
                    if high { 800 } else { 799 };
                let mut kinds = vec![first];
                kinds.extend(
                    [DamageType::True, DamageType::Physical, DamageType::Magic]
                        .into_iter()
                        .filter(|kind| *kind != first),
                );
                let shots: Vec<_> = kinds
                    .iter()
                    .map(|kind| {
                        snapshot(&mut f.core, f.a, f.enemy, *kind, DamageSource::Ability, 10)
                    })
                    .collect();
                f.core
                    .runtime_transaction(|c| {
                        let scope = c.begin_strike_scope();
                        for (i, (kind, shot)) in kinds.iter().zip(shots).enumerate() {
                            assert_eq!(
                                apply(c, shot, f.enemy, 10),
                                if high && *kind != DamageType::True {
                                    120
                                } else {
                                    100
                                }
                            );
                            if i == 0 {
                                if high {
                                    let shot = snapshot(
                                        c,
                                        f.enemy,
                                        f.a,
                                        DamageType::True,
                                        DamageSource::Ability,
                                        10,
                                    );
                                    assert_eq!(apply(c, shot, f.a, 10), 100);
                                } else {
                                    c.apply_direct_heal_and_record(Some(f.b), f.a, 201, 10)?;
                                }
                            }
                        }
                        c.end_strike_scope(scope);
                        crate::game::battle::runtime_control::poll()
                    })
                    .unwrap();
                assert_eq!(hit(&mut f, 11), if high { 100 } else { 120 });
            }
        }
    }
    #[test]
    fn high_hp_damage_actual_healing_reentry_and_unmodified_heal_shield_attack() {
        let mut f = setup(|_| {});
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 700;
        assert_eq!(hit(&mut f, 10), 100);
        f.core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(f.b), f.a, 100, 11))
            .unwrap();
        assert_eq!(f.core.units[&f.a].stats.current_health, 800);
        assert_eq!(hit(&mut f, 12), 120);
        f.core
            .runtime_transaction(|c| c.apply_direct_heal_and_record(Some(f.a), f.a, 50, 13))
            .unwrap();
        assert_eq!(f.core.units[&f.a].stats.current_health, 850);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.a,
                    f.a,
                    SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: 100,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    14,
                )
            })
            .unwrap();
        let active = f.core.sustained_effect_snapshot().unwrap().active;
        assert!(active
            .iter()
            .any(|r| r.target == f.a && r.shield.as_ref().is_some_and(|s| s.capacity == 100)));
        assert_eq!(f.core.units[&f.a].stats.attack, 100);
    }
    #[test]
    fn high_hp_damage_authored_filter_changes_and_legacy_omission_keep_owned_types() {
        let mut f = setup(|doc| {
            program(doc)["conditional_damage"]["damage_filter"]["damage_types"] = json!(["Magic"])
        });
        assert_eq!(
            damage(
                &mut f,
                DamageType::Physical,
                DamageSource::Ability,
                false,
                10
            ),
            100
        );
        assert_eq!(
            damage(&mut f, DamageType::Magic, DamageSource::Ability, false, 11),
            120
        );
        let mut f = setup(|doc| {
            program(doc)["conditional_damage"]
                .as_object_mut()
                .unwrap()
                .remove("damage_filter");
        });
        assert_eq!(
            damage(&mut f, DamageType::True, DamageSource::Ability, true, 10),
            120
        );
        assert_eq!(
            (
                witness(&f.core).physical_percent,
                witness(&f.core).magic_percent,
                witness(&f.core).true_percent
            ),
            (20, 20, 20)
        );
        let mut f = setup(|doc| {
            program(doc)["conditional_damage"]["damage_filter"]["scope"] = json!("Owned")
        });
        assert_eq!(
            damage(
                &mut f,
                DamageType::Physical,
                DamageSource::Ability,
                true,
                10
            ),
            120
        );
    }
    #[test]
    fn high_hp_damage_independent_actor_exclusion_equipped_copy_and_corrosion() {
        use crate::game::battle::runtime_facts::{
            IndependentObjectOriginSnapshot, RuntimeFactSource,
        };
        let mut f = setup(|_| {});
        let mut shot = snapshot(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::Physical,
            DamageSource::Ability,
            10,
        );
        let object = UnitInstanceId(Uuid::from_u128(0x9400));
        shot.source_id = object;
        shot.fact_source.actor = Some(object);
        shot.fact_source.extended_observation = RuntimeFactSource::extended_observation(
            Some(std::sync::Arc::new(IndependentObjectOriginSnapshot {
                object_id: object,
                definition_id: crate::game::data::independent_object_data::IndependentObjectId(
                    "high-hp-child".into(),
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
        assert_eq!(
            f.core
                .runtime_transaction(|c| Ok(apply(c, shot, f.enemy, 10)))
                .unwrap(),
            100
        );
        assert!(!witness(&f.core).contributors[0].eligible);
        assert_eq!(hit(&mut f, 11), 120);
        let copy = witness(&f.core).contributors[0]
            .equipment
            .persistent_copy_uuid;
        assert!(copy.is_some());
        f.core.units.get_mut(&f.a).unwrap().corroded = true;
        assert_eq!(hit(&mut f, 12), 100);
    }
    #[test]
    fn high_hp_damage_cancel_restores_public_action_then_retries_same_source() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(|_| {});
        attack(&mut f, 10, true);
        f.core
            .step_battle_execution_until(&mut f.state, 89)
            .unwrap();
        let before = (state(&f.core), format!("{:?}", f.state));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("conditional_damage_evaluated", 1, || {
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
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9880);
    }
    #[test]
    fn high_hp_damage_adds_to_existing_increase_without_modifying_true_damage() {
        let mut f = setup(|_| {});
        for (kind, want) in [
            (DamageType::Physical, 145),
            (DamageType::Magic, 145),
            (DamageType::True, 125),
        ] {
            let mut shot = snapshot(&mut f.core, f.a, f.enemy, kind, DamageSource::Ability, 10);
            shot.modifiers.damage_amp_percent = 25;
            let got = f
                .core
                .runtime_transaction(|c| {
                    let value = apply(c, shot, f.enemy, 10);
                    crate::game::battle::runtime_control::poll()?;
                    Ok(value)
                })
                .unwrap();
            assert_eq!(got, want);
        }
    }
    #[test]
    fn high_hp_damage_appended_components_keep_type_and_actor_filter() {
        use crate::game::battle::{cooldown::CooldownSource, damage::DamageBonusSnapshot};
        for base in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            let mut f = setup(|_| {});
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
            let got = f
                .core
                .runtime_transaction(|c| {
                    let loss = apply(c, shot, f.enemy, 10);
                    crate::game::battle::runtime_control::poll()?;
                    Ok(loss)
                })
                .unwrap();
            // Own Physical/Magic bonuses120+120, own True100, other actor100.
            assert_eq!(
                got,
                if base == DamageType::True { 540 } else { 560 },
                "{base:?}"
            );
        }
    }
}
