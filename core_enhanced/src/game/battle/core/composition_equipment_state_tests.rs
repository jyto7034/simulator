mod composition_equipment_state {
    include!("composition_hp_guard_tests.rs");
    include!("composition_dot_recovery_tests.rs");
    include!("composition_dot_kill_tests.rs");
    include!("composition_stun_end_tests.rs");
    use super::*;
    use crate::game::{
        ability::{
            equipment_state::EquipmentStateView, state_program::StateScalar,
            sustained::SustainedEffectDef,
        },
        battle::{
            core::damage_runtime::{CommittedDamageInput, CommittedDamagePolicy},
            damage::{DamageSource, DamageSourceSnapshot},
        },
    };
    const STATE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/equipment_state.skill.yaml"
    ));
    fn setup() -> Fixture {
        setup_extended(|_| {}, |_| {})
    }
    fn setup_extended(
        configure_skill: impl FnOnce(&mut String),
        configure_scenario: impl FnOnce(&mut BattleScenario),
    ) -> Fixture {
        setup_source(STATE, configure_skill, configure_scenario)
    }
    fn setup_source(
        source: &str,
        configure_skill: impl FnOnce(&mut String),
        configure_scenario: impl FnOnce(&mut BattleScenario),
    ) -> Fixture {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| *doc = parse_yaml(source).unwrap(),
            configure_skill,
            configure_scenario,
            None,
        );
        for unit in f.core.units.values_mut() {
            unit.stats.max_health = 10_000;
            unit.stats.current_health = 10_000;
            unit.stats.attack = 100;
            unit.basic_attack.windup_ms = 1;
            if unit.owner == Side::Opponent {
                unit.basic_attack.range_policy =
                    crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
                unit.basic_attack.defense_tile_range = None;
                unit.basic_attack.range_role =
                    crate::game::data::equipment_data::WeaponRangeRole::Ranged;
                unit.basic_attack.ranged_reposition_ms = 0;
            }
        }
        f
    }
    fn states(f: &Fixture) -> Vec<EquipmentStateView> {
        f.core.sustained_effect_snapshot().unwrap().equipment_states
    }
    fn speed(f: &Fixture, who: UnitInstanceId) -> u64 {
        f.core.sustained_basic_interval(who, 1000)
    }
    fn launch(
        f: &mut Fixture,
        source: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
    ) -> DamageSourceSnapshot {
        f.core.units.get_mut(&source).unwrap().next_basic_attack_ms = now;
        f.core.units.get_mut(&source).unwrap().current_target = Some(target);
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: now,
                    attacker_instance_id: source,
                    target_instance_id: Some(target),
                    schedule_next: true,
                    cause: Default::default(),
                },
                now,
            )
            .unwrap();
        f.core
            .event_queue
            .iter()
            .find_map(|e| match e.as_ref() {
                BattleEvent::AttackResolve {
                    attacker_instance_id,
                    source_snapshot,
                    time_ms,
                    ..
                } if *attacker_instance_id == source && *time_ms == now + 1 => {
                    Some(source_snapshot.clone())
                }
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "real regular basic capture: {:?}",
                    f.core.event_queue_debug_snapshot()
                )
            })
    }
    fn basic(f: &mut Fixture, now: u64) {
        launch(f, f.enemy, f.a, now);
        f.core
            .step_battle_execution_until(&mut f.state, now + 2)
            .unwrap();
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
    fn composition_equipment_state_schema_roundtrip_shared_origin_and_lifetime_bounds() {
        let artifact = crate::authoring::composition::compile_yaml(STATE).unwrap();
        let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert_eq!(loaded.equipment_states().len(), 1);
        assert!(loaded.sources()["equipment_states[0].program"]
            .definition
            .ends_with("StateProgram"));
        for (old, new) in [
            ("owns_use: false", "owns_use: true"),
            ("end_on_source_exit: true", "end_on_source_exit: false"),
            ("DirectHpLost: { relation: Hostile, once_per_action: true }", "Expired"),
            ("Time: BattleElapsed", "Time: RemainingTime"),
            ("moment: Live", "moment: Snapshot"),
            (
                "name: last_stack, unit: Count",
                "name: missing, unit: Count",
            ),
        ] {
            let error =
                crate::authoring::composition::compile_yaml(&STATE.replace(old, new)).unwrap_err();
            assert!(
                error.source.definition.contains("StateProgram"),
                "{error:?}"
            );
        }
        let mut duplicate = parse_yaml(STATE).unwrap();
        let binding = duplicate["equipment_states"][0].clone();
        duplicate["equipment_states"]
            .as_array_mut()
            .unwrap()
            .push(binding);
        assert!(compile_json(&duplicate.to_string()).is_err());
    }
    #[test]
    fn composition_equipment_state_real_basics_cap_refresh_and_exact_expiry() {
        let mut f = setup();
        for (now, interval, stack) in [(10, 935, 1), (20, 878, 2), (30, 827, 3), (40, 827, 3)] {
            basic(&mut f, now);
            assert_eq!(speed(&f, f.a), interval);
            assert_eq!(speed(&f, f.b), 1000);
            let s = states(&f);
            assert_eq!(s.len(), 1);
            assert_eq!(s[0].state.fields[0].value, StateScalar::Integer(stack));
            let worn = f
                .core
                .items
                .values()
                .find(|i| i.owner_unit_instance == f.a)
                .unwrap()
                .source_snapshot();
            assert!(worn.persistent_copy_uuid.is_some());
            assert_eq!(s[0].persistent_copy_uuid, worn.persistent_copy_uuid);
            assert_eq!(s[0].runtime_item_instance_id, worn.runtime_item_instance_id);
        }
        assert_eq!(states(&f)[0].state.events, 4);
        f.core
            .step_battle_execution_until(&mut f.state, 5011)
            .unwrap();
        assert_eq!(
            speed(&f, f.a),
            827,
            "old expiry cannot erase refreshed reward"
        );
        f.core
            .step_battle_execution_until(&mut f.state, 5040)
            .unwrap();
        assert_eq!(speed(&f, f.a), 827);
        f.core
            .step_battle_execution_until(&mut f.state, 5041)
            .unwrap();
        assert_eq!(speed(&f, f.a), 1000);
        assert_eq!(
            states(&f)[0].state.events,
            4,
            "subscription outlives finite reward"
        );
        basic(&mut f, 5050);
        assert_eq!(speed(&f, f.a), 935);
        assert_eq!(states(&f)[0].state.fields[0].value, StateScalar::Integer(1));
    }
    #[test]
    fn composition_equipment_state_same_action_components_deduplicate_across_expiry() {
        let mut f = setup();
        let (enemy, target) = (f.enemy, f.a);
        let source = launch(&mut f, enemy, target, 10);
        let target = f.a;
        f.core
            .runtime_transaction(|c| {
                assert_eq!(damage(c, source.clone(), target, 11), 100);
                assert_eq!(damage(c, source.clone(), target, 12), 100);
                c.reconcile_sustained(5011)?;
                assert_eq!(c.sustained_basic_interval(target, 1000), 1000);
                assert_eq!(damage(c, source.clone(), target, 5012), 100);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(states(&f)[0].state.events, 1);
        assert_eq!(
            speed(&f, f.a),
            1000,
            "same action must not refresh after expiry"
        );
    }
    #[test]
    fn composition_equipment_state_shield_zero_and_non_direct_sources_do_not_stack() {
        let mut f = setup();
        let target = f.a;
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    target,
                    target,
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
        basic(&mut f, 10);
        assert!(states(&f).is_empty());
        let enemy = f.enemy;
        let source = launch(&mut f, enemy, target, 20);
        f.core
            .runtime_transaction(|c| {
                for kind in [DamageSource::BuffTick, DamageSource::Environment] {
                    let mut altered = source.clone();
                    altered.source = kind;
                    damage(c, altered, target, 21);
                }
                let mut altered = source.clone();
                altered.fact_source.reaction = true;
                damage(c, altered, target, 21);
                let mut altered = source.clone();
                altered.source_id = target;
                altered.source_side = Side::Player;
                damage(c, altered, target, 21);
                let mut altered = source.clone();
                altered.base_damage = 0;
                altered.minimum_damage = 0;
                assert_eq!(damage(c, altered, target, 21), 0);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert!(states(&f).is_empty());
        f.core
            .runtime_transaction(|c| {
                damage(c, source, target, 22);
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        assert_eq!(states(&f)[0].state.events, 1);
    }
    #[test]
    fn composition_equipment_state_unequip_corrosion_and_terminal_release_both_lifetimes() {
        for boundary in ["unequip", "corrosion", "terminal"] {
            let mut f = setup();
            basic(&mut f, 10);
            assert_eq!(speed(&f, f.a), 935);
            f.core
                .runtime_transaction(|c| {
                    match boundary {
                        "unequip" => {
                            let id = c
                                .items
                                .values()
                                .find(|i| i.owner_unit_instance == f.a)
                                .unwrap()
                                .instance_id;
                            c.items.remove(&id);
                        }
                        "corrosion" => c.units.get_mut(&f.a).unwrap().corroded = true,
                        _ => {
                            c.clear_sustained(20)?;
                        }
                    }
                    c.reconcile_sustained(20)
                })
                .unwrap();
            assert!(states(&f).is_empty(), "{boundary}");
            assert_eq!(speed(&f, f.a), 1000, "{boundary}");
            assert!(f
                .core
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .is_empty());
        }
    }
    #[test]
    fn composition_equipment_state_two_owners_and_observation_have_identical_execution() {
        let mut plain = setup();
        let mut observed = Fixture {
            core: plain.core.clone(),
            state: plain.state.clone(),
            a: plain.a,
            b: plain.b,
            enemy: plain.enemy,
            secondary: plain.secondary,
            run: plain.run.clone(),
        };
        observed.core.enable_authoring_observation();
        for f in [&mut plain, &mut observed] {
            basic(f, 10);
            basic(f, 20);
            launch(f, f.secondary, f.b, 30);
            f.core
                .step_battle_execution_until(&mut f.state, 32)
                .unwrap();
            assert_eq!(speed(f, f.a), 878);
            assert_eq!(speed(f, f.b), 935);
            assert_eq!(states(f).len(), 2);
        }
        assert_eq!(states(&plain), states(&observed));
        assert_eq!(
            plain.core.event_queue_debug_snapshot(),
            observed.core.event_queue_debug_snapshot()
        );
        assert_eq!(
            serde_json::to_value(plain.core.runtime_fact_summary()).unwrap(),
            serde_json::to_value(observed.core.runtime_fact_summary()).unwrap()
        );
        assert!(observed
            .core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_sourced_sustained
            .is_some());
    }
    #[test]
    fn composition_equipment_state_cancel_restores_hp_fields_dedup_and_reward() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup();
        f.core.enable_authoring_observation();
        let (enemy, target) = (f.enemy, f.a);
        launch(&mut f, enemy, target, 10);
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
        let result = control::with_test_cancellation("state_reaction_applied", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.step_battle_execution_until(&mut f.state, 12),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snapshot(&f), before);
        use crate::game::battle::runtime_control as native;
        let mut probe = f.core.clone();
        let mut probe_state = f.state.clone();
        let (result, admissions) = native::observe_retained_admissions(|| {
            probe.step_battle_execution_until(&mut probe_state, 12)
        });
        result.unwrap();
        let required = admissions
            .iter()
            .find(|(site, _, _)| *site == "equipment_state_record")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = required - 1;
        let before = snapshot(&f);
        let error = f
            .core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(native::BattleRuntimeFault {
                    code: native::RuntimeFaultCode::RetainedByteLimit,
                    site: "equipment_state_record",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(snapshot(&f), before);
        f.core.runtime_facts.limits = Default::default();
        f.core
            .step_battle_execution_until(&mut f.state, 12)
            .unwrap();
        assert_eq!(states(&f)[0].state.events, 1);
        assert_eq!(speed(&f, target), 935);
        assert_eq!(f.core.units[&target].stats.current_health, 9900);
    }
    #[test]
    fn composition_equipment_state_actual_enemy_auto_skill_multihit_counts_one_action() {
        let mut f = setup_extended(
            |authored| {
                let mut db = crate::game::data::skill_data::SkillDatabase::from_ron(
                    "state-enemy.ron",
                    &r12_control_ron("Damage(amount:100, damage_type:True)"),
                )
                .unwrap();
                db.skills[0].steps[0].effects.push(
                    ron::de::from_str(r#"(id:"second",op:Damage(amount:100,damage_type:True))"#)
                        .unwrap(),
                );
                *authored = ron::ser::to_string(&db).unwrap();
            },
            |scenario| {
                let BattleUnitSource::TestFixture { profile, .. } =
                    &mut scenario.groups[0].spawns[0].draft.source
                else {
                    unreachable!()
                };
                profile.skill_id = Some("r11_aura".into());
                profile.skill_activation_mode = crate::game::ability::SkillActivationMode::Auto;
                profile.resonance.start = 0;
                profile.resonance.max = 100;
            },
        );
        for (now, hp, events) in [(10, 9800, 1), (20, 9600, 2)] {
            f.core
                .runtime_transaction(|c| {
                    c.process_commands(
                        vec![
                            crate::game::battle::damage::BattleCommand::ModifyResonance {
                                target_id: f.enemy,
                                amount: 100,
                                allow_autocast_when_full: true,
                            },
                        ],
                        now,
                    );
                    c.schedule_pending_autocasts(now);
                    crate::game::battle::runtime_control::poll()
                })
                .unwrap();
            f.core
                .step_battle_execution_until(&mut f.state, now + 1)
                .unwrap();
            assert_eq!(f.core.units[&f.a].stats.current_health, hp);
            assert_eq!(states(&f)[0].state.events, events);
        }
        assert_eq!(speed(&f, f.a), 878);
    }
    #[test]
    fn composition_equipment_state_withdraw_and_redeploy_same_copy_start_new_history() {
        let mut f = setup();
        basic(&mut f, 10);
        basic(&mut f, 20);
        let original = f.a;
        let old_copy = states(&f)[0].persistent_copy_uuid;
        let owned = f.core.units[&original].source_owned_uuid;
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
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: original,
                time_ms: 30,
            })
            .unwrap();
        assert!(states(&f).is_empty());
        let crate::game::battle::core::sim::BattleLiveCommandOutcome::UnitDeployed {
            unit_id: fresh,
        } = f
            .core
            .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft,
                position: crate::game::resources::Position::new(2, 1),
                tile_kind: crate::game::battle::types::TacticalTileKind::Ground,
                facing: crate::game::battle::tile_range::FacingDirection::Right,
                instance_salt: 0xe411,
                time_ms: 1001,
                current_hp_policy: None,
            })
            .unwrap()
        else {
            unreachable!()
        };
        assert_ne!(fresh, original);
        f.a = fresh;
        let unit = f.core.units.get_mut(&fresh).unwrap();
        unit.stats.max_health = 10_000;
        unit.stats.current_health = 10_000;
        unit.stats.defense = 0;
        unit.stats.attack_interval_ms = 100_000;
        unit.next_basic_attack_ms = 100_000;
        assert_eq!(speed(&f, fresh), 1000);
        basic(&mut f, 1010);
        let current = states(&f);
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].owner, fresh);
        assert_eq!(current[0].persistent_copy_uuid, old_copy);
        assert_eq!(current[0].state.events, 1);
        assert_eq!(speed(&f, fresh), 935);
    }
    #[test]
    fn composition_equipment_state_actual_hp_cost_maximum_change_and_self_damage_are_excluded() {
        let mut f = setup_extended(
            |authored| {
                let mut db = crate::game::data::skill_data::SkillDatabase::from_ron(
                    "state-self.ron",
                    &r12_control_ron("HealthCost(amount:100,allow_lethal:false)")
                        .replace("target: CastTarget", "target: SelfUnit"),
                )
                .unwrap();
                db.skills[0].steps[0].target = crate::game::ability::SkillTarget::SelfUnit;
                for effect in [
                    r#"(id:"max-hp",op:ModifyStats(modifier:(stat:MaxHealth,kind:Flat,value:-500)))"#,
                    r#"(id:"self-damage",op:Damage(amount:100,damage_type:True))"#,
                ] {
                    db.skills[0].steps[0]
                        .effects
                        .push(ron::de::from_str(effect).unwrap());
                }
                // Give the action a real hostile purpose before paying its
                // self cost; an entirely harmful self-only skill is correctly
                // rejected by ordinary player usefulness admission.
                let mut attack = db.skills[0].steps[0].clone();
                attack.id = "hostile-purpose".into();
                attack.target = crate::game::ability::SkillTarget::CastTarget;
                attack.effects =
                    vec![
                        ron::de::from_str(r#"(id:"hit",op:Damage(amount:1,damage_type:True))"#)
                            .unwrap(),
                    ];
                db.skills[0].steps.insert(0, attack);
                *authored = ron::ser::to_string(&db).unwrap();
            },
            |_| {},
        );
        f.core.units.get_mut(&f.a).unwrap().resonance_current = 100;
        f.core
            .apply_live_command(BattleLiveCommand::ActivateSkill {
                unit_id: f.a,
                skill_id: "r11_aura".into(),
                target: None,
                time_ms: 10,
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 11)
            .unwrap();
        assert_eq!(f.core.units[&f.a].stats.max_health, 9500);
        assert_eq!(f.core.units[&f.a].stats.current_health, 9400);
        assert!(states(&f).is_empty());
        assert_eq!(speed(&f, f.a), 1000);
    }
    #[test]
    fn composition_equipment_state_actual_trigger_proc_is_excluded() {
        use crate::game::{
            battle::{
                cooldown::CooldownSource,
                damage::{BattleCommand, ProcRollIdentity},
            },
            stats::TriggerType,
        };
        let mut f = setup_extended(
            |authored| *authored = r12_control_ron("Damage(amount:100,damage_type:True)"),
            |_| {},
        );
        let activation_source = CooldownSource::Unit {
            unit_instance_id: f.enemy,
        };
        let skill_id: crate::game::ability::SkillId = "r11_aura".into();
        let identity = ProcRollIdentity {
            trigger_type: TriggerType::OnAttack,
            activation_source,
            ability_id: skill_id.clone(),
            binding_index: 0,
            caster_id: f.enemy,
            trigger_unit_id: f.enemy,
            counterpart_unit_id: Some(f.a),
            target_id: Some(f.a),
            occurrence_id: Uuid::from_u128(0xe412),
            occurrence_index: 0,
        };
        f.core
            .runtime_transaction(|c| {
                c.process_commands(
                    vec![BattleCommand::TriggerAbility {
                        skill_id,
                        caster_id: f.enemy,
                        target_id: Some(f.a),
                        activation_source,
                        equipment_source: None,
                        binding_index: 0,
                        proc_roll_identity: identity,
                        proc_chance_percent: 100,
                        internal_cooldown_ms: 0,
                        max_triggers_per_battle: Some(1),
                        allow_dead_caster: false,
                    }],
                    10,
                );
                crate::game::battle::runtime_control::poll()
            })
            .unwrap();
        f.core
            .step_battle_execution_until(&mut f.state, 11)
            .unwrap();
        assert_eq!(f.core.units[&f.a].stats.current_health, 9900);
        assert!(f
            .core
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .unwrap()
            .source
            .cast
            .as_ref()
            .unwrap()
            .proc_origin
            .is_some());
        assert!(states(&f).is_empty());
    }
    #[test]
    fn composition_equipment_state_lethal_hit_never_creates_or_retains_subscription() {
        for established in [false, true] {
            let mut f = setup();
            if established {
                basic(&mut f, 10);
                assert_eq!(states(&f).len(), 1);
            }
            f.core.units.get_mut(&f.a).unwrap().stats.current_health = 50;
            basic(&mut f, 20);
            assert!(f.core.units[&f.a].lifecycle.is_dead());
            assert!(states(&f).is_empty());
            assert!(f
                .core
                .sustained_effect_snapshot()
                .unwrap()
                .active
                .iter()
                .all(|s| s.target != f.a));
        }
    }
    #[test]
    fn composition_equipment_state_predicates_preserve_refresh_expiry_and_copy_isolation() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/skill_composition/predicate_state.skill.yaml"
        ));
        let mut f = setup_source(source, |_| {}, |_| {});
        for (now, interval, stack) in [(10, 935, 1), (20, 878, 2), (30, 827, 3), (40, 827, 3)] {
            basic(&mut f, now);
            assert_eq!(speed(&f, f.a), interval);
            assert_eq!(speed(&f, f.b), 1000);
            assert_eq!(
                states(&f)[0].state.fields[0].value,
                StateScalar::Integer(stack)
            );
        }
        f.core
            .step_battle_execution_until(&mut f.state, 5041)
            .unwrap();
        assert_eq!(speed(&f, f.a), 1000);
        basic(&mut f, 5050);
        assert_eq!(speed(&f, f.a), 935);
        assert_eq!(states(&f)[0].state.fields[0].value, StateScalar::Integer(1));
    }
    #[test]
    fn recipient_support_state_target_relation_count_reuses_actual_state_event_frame() {
        let mut doc=parse_yaml(STATE).unwrap();
        let program=&mut doc["values"][0]["value"]["StateProgram"];
        program["handlers"][0]["event"]=json!({"DirectHpLost":{"relation":"Hostile","once_per_action":false}});
        program["handlers"][0]["body"]=json!([{"Set":{"field":"last_stack","value":{"moment":"Live","root":{"Read":{"RelationCount":{"subject":"Target","relation":"BlockingHostiles"}}}}}}]);
        let rejected=doc.to_string().replace("\"subject\":\"Target\"","\"subject\":\"Source\"");assert!(compile_json(&rejected).is_err());
        let mut f=setup_source(&doc.to_string(),|_|{},|_|{});let(e,t)=(f.enemy,f.a);
        for (now,blocked) in [(10,true),(20,false)] {
            let source=launch(&mut f,e,t,now);
            f.core.block_state.enemy_to_blocker.clear();f.core.block_state.blocker_to_enemies.clear();
            if blocked {f.core.block_state.enemy_to_blocker.insert(e,t);f.core.block_state.blocker_to_enemies.insert(t,vec![e]);}
            f.core.runtime_transaction(|c|{damage(c,source,t,now+1);crate::game::battle::runtime_control::poll()}).unwrap();
            assert_eq!(states(&f)[0].state.fields[0].value,StateScalar::Integer(if blocked{1}else{0}));
        }
    }
}
