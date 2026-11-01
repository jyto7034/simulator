mod composition_conditional_damage {
    include!("composition_high_hp_damage_tests.rs");
    include!("composition_direct_penetration_tests.rs");
    include!("composition_magic_penetration_tests.rs");
    use super::*;
    use crate::game::{
        ability::sustained::{ControlKind, HitStatusKind, HitStatusProfile, SustainedEffectDef},
        battle::{
            damage::{BattleCommand, DamageSource, DamageSourceSnapshot, DamageType},
            types::UnitTargetTrait,
        },
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/conditional_damage.skill.yaml"
    ));
    fn setup(edit: impl FnOnce(&mut Value)) -> Fixture {
        let source = SOURCE.replace("ice_crown", "r11_weapon");
        let mut f = fixture(0, false, |doc| {
            *doc = parse_yaml(&source).unwrap();
            edit(doc);
        });
        for unit in f.core.units.values_mut() {
            unit.stats.magic_resist = 0;
        }
        for id in [f.enemy, f.secondary] {
            let u = f.core.units.get_mut(&id).unwrap();
            u.stats.current_health = 10000;
            u.stats.max_health = 10000;
        }
        let u = f.core.units.get_mut(&f.a).unwrap();
        u.stats.attack = 100;
        u.basic_attack.delivery = crate::game::ability::DeliveryDef::Instant;
        u.basic_attack.windup_ms = 80;
        u.basic_attack.damage_type = DamageType::Physical;
        u.basic_attack.range_policy =
            crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
        f.core.enable_authoring_observation();
        f
    }
    fn status(
        core: &mut BattleCore,
        owner: UnitInstanceId,
        target: UnitInstanceId,
        kind: HitStatusKind,
        now: u64,
    ) {
        core.runtime_transaction(|c| {
            c.begin_sustained(
                owner,
                target,
                SustainedEffectDef::HitStatus {
                    duration_ms: 1000,
                    profile: HitStatusProfile {
                        definition: "conditional-test-status".into(),
                        kind,
                        modifier_source: None,
                        vulnerability: None,
                    },
                },
                now,
            )
        })
        .unwrap();
    }
    fn snapshot(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        kind: DamageType,
        source: DamageSource,
        now: u64,
    ) -> DamageSourceSnapshot {
        core.damage_source_snapshot_for_unit(
            actor,
            target,
            source,
            kind,
            100,
            Default::default(),
            0,
            now,
            false,
        )
        .unwrap()
    }
    fn apply(
        core: &mut BattleCore,
        snapshot: DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> u32 {
        let before = core.units[&target].stats.current_health;
        core.process_commands(
            vec![BattleCommand::ApplyDamage {
                target_id: target,
                source_snapshot: snapshot,
            }],
            now,
        );
        before - core.units[&target].stats.current_health
    }
    fn hit(f: &mut Fixture, now: u64) -> u32 {
        let shot = snapshot(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::Physical,
            DamageSource::BasicAttack,
            now,
        );
        f.core
            .runtime_transaction(|c| {
                let damage = apply(c, shot, f.enemy, now);
                crate::game::battle::runtime_control::poll()?;
                Ok(damage)
            })
            .unwrap()
    }
    fn witness(
        core: &BattleCore,
    ) -> &crate::game::battle::core::conditional_damage::ConditionalDamageWitness {
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_conditional_damage
            .as_ref()
            .unwrap()
    }
    fn hp_condition() -> Value {
        let amount = |field: &str, multiplier: u32| {
            json!({"moment":"Live","root":{"Mul":{
                "value":{"Read":{"HitPoints":{"subject":"Source","field":field}}},
                "scalar":{"Const":{"value":multiplier,"unit":"Scalar"}}
            }}})
        };
        json!({"Compare":{"left":amount("Current",100),"comparison":"AtLeast","right":amount("Max",80)}})
    }
    #[test]
    fn distant_defense_distance_predicate_also_drives_outgoing_authored_damage() {
        use crate::game::battle::core::movement::types::WorldVec2;
        let mut f = setup(|doc| {
            doc["values"][0]["value"]["AttackProgram"]["conditional_damage"]["condition"] =
                json!({"Distance":{"comparison":"Greater","units":2000000}});
        });
        f.core.units.get_mut(&f.enemy).unwrap().stats.defense = 0;
        f.core
            .units
            .get_mut(&f.a)
            .unwrap()
            .set_world_position(WorldVec2::new(0.5, 0.5));
        for (now, x, expected) in [(1, 2.5, 100), (2, 3.5, 120), (3, 1.5, 100)] {
            f.core
                .units
                .get_mut(&f.enemy)
                .unwrap()
                .set_world_position(WorldVec2::new(x, 0.5));
            assert_eq!(hit(&mut f, now), expected);
            assert_eq!(
                witness(&f.core).percent,
                if expected == 120 { 20 } else { 0 }
            );
            assert!(witness(&f.core).distance_squared.is_some());
        }
    }
    #[test]
    fn conditional_damage_roundtrip_rejects_wrong_context_unit_empty_and_deep_logic() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let reopened = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(reopened.equipment_attacks()[0]
            .program
            .conditional_damage
            .is_some());
        for (from, to) in [
            ("moment: Live", "moment: Snapshot"),
            ("unit: Scalar", "unit: Milliseconds"),
            (
                "condition: MovementImpaired",
                "condition: { Logic: { All: [] } }",
            ),
            ("condition: MovementImpaired", "condition: Unknown"),
        ] {
            assert!(
                crate::authoring::composition::compile_yaml(&SOURCE.replace(from, to)).is_err()
            );
        }
        let mut doc = parse_yaml(SOURCE).unwrap();
        let mut condition = json!("MovementImpaired");
        for _ in 0..8 {
            condition = json!({"Logic":{"Not":condition}});
        }
        program(&mut doc)["conditional_damage"]["condition"] = condition;
        assert!(compile_json(&doc.to_string()).is_err());
        program(&mut doc)["conditional_damage"]["condition"] = hp_condition();
        assert!(compile_json(&doc.to_string()).is_ok());
        program(&mut doc)["conditional_damage"]["condition"]["Compare"]["right"] =
            json!({"moment":"Live","root":{"Const":{"value":80,"unit":"Scalar"}}});
        let error = compile_json(&doc.to_string()).unwrap_err();
        assert!(
            error.source.definition.contains("conditional_damage"),
            "{error:?}"
        );
    }
    #[test]
    fn conditional_damage_admitted_ally_slow_root_freeze_stun_and_expiry() {
        for kind in [
            HitStatusKind::Slow { percent: 20 },
            HitStatusKind::Root,
            HitStatusKind::Freeze,
            HitStatusKind::Stun,
        ] {
            let mut f = setup(|_| {});
            assert_eq!(hit(&mut f, 1), 100);
            status(&mut f.core, f.b, f.enemy, kind, 10);
            assert_eq!(hit(&mut f, 11), 120, "{kind:?}");
            assert_eq!(witness(&f.core).target_movement_impaired, Some(true));
            assert_eq!(witness(&f.core).wearer, f.a);
            assert_eq!(hit(&mut f, 1009), 120);
            assert_eq!(hit(&mut f, 1010), 100);
            assert_eq!(witness(&f.core).target_movement_impaired, Some(false));
        }
    }
    #[test]
    fn conditional_damage_control_immunity_and_nonmovement_states_do_not_qualify() {
        for (kind, immune) in [
            (
                HitStatusKind::Slow { percent: 20 },
                UnitTargetTrait::SlowImmune,
            ),
            (HitStatusKind::Root, UnitTargetTrait::RootImmune),
            (HitStatusKind::Stun, UnitTargetTrait::StunImmune),
            (HitStatusKind::Freeze, UnitTargetTrait::StunImmune),
        ] {
            let mut f = setup(|_| {});
            f.core
                .units
                .get_mut(&f.enemy)
                .unwrap()
                .target_traits
                .push(immune);
            status(&mut f.core, f.b, f.enemy, kind, 10);
            assert_eq!(hit(&mut f, 11), 100);
            assert_eq!(witness(&f.core).target_movement_impaired, Some(false));
        }
        let mut f = setup(|_| {});
        status(&mut f.core, f.b, f.enemy, HitStatusKind::BasicLock, 10);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    f.b,
                    f.enemy,
                    SustainedEffectDef::Control {
                        duration_ms: 1000,
                        kind: ControlKind::Taunt { priority: 1 },
                    },
                    10,
                )
            })
            .unwrap();
        assert_eq!(hit(&mut f, 11), 100);
        // The fixture already has zero raw movement speed: it is not a tagged control.
        assert_eq!(witness(&f.core).target_movement_impaired, Some(false));
    }
    #[test]
    fn conditional_damage_two_controls_apply_once_and_targets_remain_independent() {
        let mut f = setup(|_| {});
        status(
            &mut f.core,
            f.b,
            f.enemy,
            HitStatusKind::Slow { percent: 20 },
            10,
        );
        status(&mut f.core, f.a, f.enemy, HitStatusKind::Stun, 10);
        assert_eq!(hit(&mut f, 11), 120);
        let shot = snapshot(
            &mut f.core,
            f.a,
            f.secondary,
            DamageType::Physical,
            DamageSource::BasicAttack,
            12,
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| Ok(apply(c, shot, f.secondary, 12)))
                .unwrap(),
            100
        );
        assert_eq!(witness(&f.core).target, f.secondary);
    }
    #[test]
    fn conditional_damage_same_authored_compare_and_boolean_composition_tracks_current_hp() {
        let mut f = setup(|doc| {
            let p = &mut program(doc)["conditional_damage"];
            p["condition"] = json!({"Logic":{"All":["MovementImpaired",hp_condition()]}});
            p["percent"]["root"]["Const"]["value"] = json!(30);
        });
        status(
            &mut f.core,
            f.b,
            f.enemy,
            HitStatusKind::Slow { percent: 20 },
            10,
        );
        assert_eq!(hit(&mut f, 11), 100);
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 800;
        assert_eq!(hit(&mut f, 12), 130);
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 799;
        assert_eq!(hit(&mut f, 13), 100);
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 1000;
        assert_eq!(hit(&mut f, 14), 130);
        assert_eq!(witness(&f.core).wearer_hp, 1000);
        let mut other = setup(|doc| {
            program(doc)["conditional_damage"]["condition"] =
                json!({"Logic":{"Any":[hp_condition(),{"Logic":{"Not":"MovementImpaired"}}]}})
        });
        assert_eq!(hit(&mut other, 1), 120);
        status(
            &mut other.core,
            other.b,
            other.enemy,
            HitStatusKind::Slow { percent: 20 },
            10,
        );
        assert_eq!(hit(&mut other, 11), 100);
    }
    #[test]
    fn conditional_damage_first_target_strike_freezes_before_later_hp_or_status_change() {
        let mut f = setup(|_| {});
        let first = snapshot(
            &mut f.core,
            f.a,
            f.enemy,
            DamageType::Physical,
            DamageSource::BasicAttack,
            10,
        );
        let second = first.clone();
        f.core
            .runtime_transaction(|c| {
                let scope = c.begin_strike_scope();
                assert_eq!(apply(c, first, f.enemy, 10), 100);
                c.begin_sustained(
                    f.b,
                    f.enemy,
                    SustainedEffectDef::Control {
                        duration_ms: 1000,
                        kind: ControlKind::Root,
                    },
                    10,
                )?;
                assert_eq!(apply(c, second, f.enemy, 10), 100);
                c.end_strike_scope(scope);
                Ok(())
            })
            .unwrap();
        assert_eq!(hit(&mut f, 11), 120);
        let mut hp = setup(|doc| program(doc)["conditional_damage"]["condition"] = hp_condition());
        let first = snapshot(
            &mut hp.core,
            hp.a,
            hp.enemy,
            DamageType::Physical,
            DamageSource::BasicAttack,
            10,
        );
        let second = first.clone();
        hp.core
            .runtime_transaction(|c| {
                let scope = c.begin_strike_scope();
                assert_eq!(apply(c, first, hp.enemy, 10), 100);
                c.units.get_mut(&hp.a).unwrap().stats.current_health = 1000;
                assert_eq!(apply(c, second, hp.enemy, 10), 100);
                c.end_strike_scope(scope);
                Ok(())
            })
            .unwrap();
        assert_eq!(hit(&mut hp, 11), 120);
    }
    fn state(core: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn conditional_damage_public_cancel_preserves_action_then_retries() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(|_| {});
        status(
            &mut f.core,
            f.b,
            f.enemy,
            HitStatusKind::Slow { percent: 20 },
            0,
        );
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
    fn conditional_damage_clone_and_observation_preserve_damage_and_queue() {
        let mut f = setup(|_| {});
        status(
            &mut f.core,
            f.b,
            f.enemy,
            HitStatusKind::Slow { percent: 20 },
            0,
        );
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
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9880);
        assert_eq!(plain.units[&f.enemy].stats.current_health, 9880);
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
    fn conditional_damage_withdraw_after_release_keeps_projectile_without_live_bonus() {
        let mut f = setup(|_| {});
        status(
            &mut f.core,
            f.b,
            f.enemy,
            HitStatusKind::Slow { percent: 20 },
            0,
        );
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
    }

    #[test]
    fn conditional_damage_retained_refusal_restores_and_retries_with_finite_limit() {
        use crate::game::battle::runtime_control as native;
        let mut f = setup(|_| {});
        status(
            &mut f.core,
            f.b,
            f.enemy,
            HitStatusKind::Slow { percent: 20 },
            0,
        );
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
            .find(|(site, _, _)| *site == "conditional_damage_observation")
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
                    site: "conditional_damage_observation",
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
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 9880);
    }
    #[test]
    fn conditional_damage_checked_object_creator_and_direct_employee_take_correct_copy() {
        use crate::game::battle::runtime_facts::{
            IndependentObjectOriginSnapshot, RuntimeFactSource,
        };
        let mut f = setup(|_| {});
        status(
            &mut f.core,
            f.b,
            f.enemy,
            HitStatusKind::Slow { percent: 20 },
            0,
        );
        let object = UnitInstanceId(Uuid::from_u128(0x770b));
        for (direct, matching, expected) in
            [(true, false, 145), (false, true, 145), (false, false, 125)]
        {
            let before = f.core.units[&f.enemy].stats.current_health;
            f.core
                .runtime_transaction(|c| {
                    let mut snapshot = c
                        .damage_source_snapshot_for_unit(
                            f.a,
                            f.enemy,
                            DamageSource::Ability,
                            DamageType::True,
                            100,
                            crate::game::battle::damage::DamageModifiers {
                                damage_amp_percent: 25,
                                ..Default::default()
                            },
                            0,
                            10,
                            false,
                        )
                        .unwrap();
                    if !direct {
                        snapshot.source_id = object;
                        snapshot.fact_source.actor = Some(object);
                    }
                    snapshot.fact_source.extended_observation =
                        RuntimeFactSource::extended_observation(
                            Some(std::sync::Arc::new(IndependentObjectOriginSnapshot {
                                object_id: if matching {
                                    object
                                } else {
                                    UnitInstanceId(Uuid::from_u128(0x770c))
                                },
                                definition_id:
                                    crate::game::data::independent_object_data::IndependentObjectId(
                                        "fixture-owned".into(),
                                    ),
                                creator: if direct { f.b } else { f.a },
                                creation_action: None,
                                creation_cast: None,
                                creation_fragment: None,
                                creation_equipment: None,
                                creation_authored: None,
                            })),
                            None,
                        )?;
                    c.process_commands(
                        vec![BattleCommand::ApplyDamage {
                            target_id: f.enemy,
                            source_snapshot: snapshot,
                        }],
                        10,
                    );
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            assert_eq!(
                before - f.core.units[&f.enemy].stats.current_health,
                expected
            );
            if expected == 145 {
                assert_eq!(witness(&f.core).wearer, f.a);
                assert_eq!(witness(&f.core).percent, 20);
            }
        }
    }

    #[test]
    fn conditional_damage_all_damage_types_sources_and_owner_isolation() {
        for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
            for (source, reaction, expected) in [
                (DamageSource::BasicAttack, false, 120),
                (DamageSource::Ability, false, 120),
                (DamageSource::BuffTick, false, 120),
                (DamageSource::Environment, true, 120),
                (DamageSource::Environment, false, 100),
            ] {
                let mut f = setup(|_| {});
                status(&mut f.core, f.b, f.enemy, HitStatusKind::Root, 0);
                let mut shot = snapshot(&mut f.core, f.a, f.enemy, kind, source, 10);
                shot.fact_source.reaction = reaction;
                let actual = f
                    .core
                    .runtime_transaction(|c| {
                        let result = apply(c, shot, f.enemy, 10);
                        crate::game::battle::runtime_control::poll()?;
                        Ok(result)
                    })
                    .unwrap();
                assert_eq!(actual, expected, "{kind:?}/{source:?}/{reaction}");
            }
        }
        let mut f = setup(|doc| program(doc)["conditional_damage"]["condition"] = hp_condition());
        f.core.units.get_mut(&f.a).unwrap().stats.current_health = 1000;
        f.core.units.get_mut(&f.b).unwrap().stats.current_health = 700;
        assert_eq!(hit(&mut f, 10), 120);
        let a_copy = witness(&f.core).contributors[0].equipment.clone();
        let shot = snapshot(
            &mut f.core,
            f.b,
            f.enemy,
            DamageType::Physical,
            DamageSource::Ability,
            11,
        );
        assert_eq!(
            f.core
                .runtime_transaction(|c| Ok(apply(c, shot, f.enemy, 11)))
                .unwrap(),
            100
        );
        assert_eq!(witness(&f.core).wearer, f.b);
        assert_ne!(
            serde_json::to_value(a_copy).unwrap(),
            serde_json::to_value(&witness(&f.core).contributors[0].equipment).unwrap()
        );
        f.core.units.get_mut(&f.a).unwrap().corroded = true;
        assert_eq!(hit(&mut f, 12), 100);
    }
    #[test]
    fn conditional_damage_legacy_stun_and_silence_keep_existing_expiry_semantics() {
        for (name, expected) in [("r11_stun", 120), ("r11_silence", 100)] {
            let mut f = setup(|_| {});
            f.core
                .runtime_transaction(|c| {
                    c.process_commands(
                        vec![BattleCommand::ApplyBuff {
                            caster_id: f.b,
                            target_id: f.enemy,
                            buff_id: crate::game::battle::buffs::BuffId::from_name(name),
                            duration_ms: 100,
                        }],
                        10,
                    );
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            f.core
                .step_battle_execution_until(&mut f.state, 10)
                .unwrap();
            assert_eq!(hit(&mut f, 11), expected, "{name}");
            // Legacy control remains active at its inclusive deadline.
            assert_eq!(hit(&mut f, 110), expected, "{name}");
            assert_eq!(hit(&mut f, 111), 100, "{name}");
        }
    }

    #[test]
    fn conditional_damage_out_of_range_runtime_expression_is_atomic() {
        for percent in [-1, 10001] {
            let mut f = setup(|doc| {
                program(doc)["conditional_damage"]["percent"]["root"]["Const"]["value"] =
                    json!(percent)
            });
            status(&mut f.core, f.b, f.enemy, HitStatusKind::Root, 0);
            attack(&mut f, 10, true);
            f.core
                .step_battle_execution_until(&mut f.state, 89)
                .unwrap();
            let before = (state(&f.core), format!("{:?}", f.state));
            let error = f
                .core
                .step_battle_execution_until(&mut f.state, 90)
                .unwrap_err();
            let detail = format!("{error:?}");
            assert!(
                detail.contains("conditional_damage") && detail.contains("Overflow"),
                "{detail}"
            );
            assert_eq!(before, (state(&f.core), format!("{:?}", f.state)));
            assert_eq!(f.core.units[&f.enemy].stats.current_health, 10000);
        }
    }
}
