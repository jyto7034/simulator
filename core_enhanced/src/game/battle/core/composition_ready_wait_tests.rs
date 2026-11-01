mod composition_ready_wait {
    use super::*;
    use crate::game::{
        ability::{
            sustained::{HitStatusKind, HitStatusProfile, SustainedEffectDef},
            SkillActivationMode,
        },
        battle::{
            damage::{BattleCommand, DamageSource, DamageType},
            runtime_control,
        },
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/ready_wait.skill.yaml"
    ));
    fn setup(rate: u32, cap: u32) -> Fixture {
        setup_edit(rate, cap, |_| {})
    }
    fn setup_edit(rate: u32, cap: u32, edit: impl FnOnce(&mut Value)) -> Fixture {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| {
                *doc = parse_yaml(SOURCE).unwrap();
                let p = &mut doc["values"][0]["value"]["DefenseProgram"]["ReadyWaitDamage"];
                p["per_second_percent"] = json!(rate);
                p["max_percent"] = json!(cap);
                edit(doc);
            },
            |ron| *ron = r12_control_ron("Damage(amount:100,damage_type:True)"),
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(100000);
            },
            None,
        );
        f.core.equipment_defense = Default::default();
        for unit in f.core.units.values_mut() {
            unit.stats.max_health = 10000;
            unit.stats.current_health = 10000;
            unit.stats.magic_resist = 0;
        }
        for id in [f.a, f.b] {
            let unit = f.core.units.get_mut(&id).unwrap();
            unit.resonance_current = unit.resonance_max;
            unit.next_action_time = 0;
        }
        f.core.enable_authoring_observation();
        f.core
            .bind_authoring_observation(
                crate::game::battle::core::authoring_observation::ObservationKey {
                    entry_ordinal: 1,
                    battle_uuid: Uuid::from_u128(0x7200),
                },
            )
            .unwrap();
        refresh(&mut f.core, 0).unwrap();
        f
    }
    fn refresh(core: &mut BattleCore, now: u64) -> Result<(), GameError> {
        core.runtime_transaction(|c| c.reconcile_sustained(now))
    }
    fn step(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn cast(f: &mut Fixture, actor: UnitInstanceId, now: u64) {
        step(f, now);
        f.core
            .request_skill_action(now, actor, "r11_aura".into(), None)
            .unwrap();
    }
    fn witness(
        core: &BattleCore,
    ) -> &crate::game::battle::core::equipment_defense::ReadyWaitWitness {
        core.authoring_observation
            .as_ref()
            .unwrap()
            .latest_ready_wait
            .as_ref()
            .unwrap()
    }
    fn hit(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        source: DamageSource,
        kind: DamageType,
        now: u64,
    ) -> u32 {
        let before = core.units[&target].stats.current_health;
        core.runtime_transaction(|c| {
            let snapshot = c
                .damage_source_snapshot_for_unit(
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
                .unwrap();
            c.process_commands(
                vec![BattleCommand::ApplyDamage {
                    target_id: target,
                    source_snapshot: snapshot,
                }],
                now,
            );
            runtime_control::poll()
        })
        .unwrap();
        before - core.units[&target].stats.current_health
    }
    fn snap(core: &BattleCore) -> impl PartialEq + std::fmt::Debug {
        (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
        )
    }
    #[test]
    fn composition_ready_wait_roundtrip_and_precise_positive_bounds() {
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let restored = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        assert!(matches!(
            restored.equipment_defenses()[0].program,
            crate::game::ability::equipment_defense::EquipmentDefenseProgram::ReadyWaitDamage {
                per_second_percent: 10,
                max_percent: 50,
                duration_ms: 15000
            }
        ));
        for (field, value) in [
            ("per_second_percent", 0),
            ("per_second_percent", 10001),
            ("max_percent", 0),
            ("max_percent", 10001),
            ("duration_ms", 0),
        ] {
            let mut doc = parse_yaml(SOURCE).unwrap();
            doc["values"][0]["value"]["DefenseProgram"]["ReadyWaitDamage"][field] = json!(value);
            let e = compile_json(&doc.to_string()).unwrap_err();
            assert!(e.source.definition.ends_with(field), "{e:?}");
        }
    }
    #[test]
    fn composition_ready_wait_zero_fractional_linear_cap_and_independent_wearers() {
        for (at, expected) in [(0, 0), (999, 9), (2500, 25), (5000, 50), (8000, 50)] {
            let mut f = setup(10, 50);
            let a = f.a;
            cast(&mut f, a, at);
            let w = witness(&f.core);
            assert_eq!(
                (
                    w.time_ms,
                    w.percent,
                    w.contributors[0].waited_ms,
                    w.contributors[0].until_ms
                ),
                (at, expected, at, at + 15000)
            );
            assert_eq!(w.actor, a);
            assert_eq!(
                Some(w.contributors[0].equipment.persistent_copy_uuid.unwrap()),
                match owner(&f.core, a) {
                    ResourceOwner::EquipmentCopy(id) => Some(id),
                    _ => None,
                }
            );
        }
        let mut f = setup(10, 50);
        let (a, b) = (f.a, f.b);
        cast(&mut f, a, 2000);
        assert_eq!(witness(&f.core).percent, 20);
        cast(&mut f, b, 3000);
        assert_eq!(witness(&f.core).percent, 30);
        assert_ne!(owner(&f.core, a), owner(&f.core, b));
    }
    #[test]
    fn composition_ready_wait_actual_damage_types_periodic_components_and_half_open_expiry() {
        let mut f = setup(10, 50);
        let (a, e) = (f.a, f.enemy);
        let attack = f.core.units[&a].stats.attack;
        cast(&mut f, a, 5000);
        step(&mut f, 5001);
        assert_eq!(f.core.units[&a].stats.attack, attack);
        for source in [
            DamageSource::BasicAttack,
            DamageSource::Ability,
            DamageSource::BuffTick,
        ] {
            for kind in [DamageType::Physical, DamageType::Magic, DamageType::True] {
                assert_eq!(
                    hit(&mut f.core, a, e, source, kind, 5002),
                    150,
                    "{source:?}/{kind:?}"
                );
            }
        }
        assert_eq!(
            hit(
                &mut f.core,
                a,
                e,
                DamageSource::Environment,
                DamageType::True,
                5002
            ),
            100
        );
        assert_eq!(
            hit(
                &mut f.core,
                a,
                e,
                DamageSource::Ability,
                DamageType::True,
                19999
            ),
            150
        );
        assert_eq!(
            hit(
                &mut f.core,
                a,
                e,
                DamageSource::Ability,
                DamageType::True,
                20000
            ),
            100
        );
        assert_eq!(
            witness(&f.core).percent,
            50,
            "last consumption is not a live-buff projection"
        );
    }
    #[test]
    fn composition_ready_wait_resource_loss_target_loss_and_hard_cc_restart_waiting() {
        let mut f = setup(10, 50);
        let (a, e) = (f.a, f.enemy);
        step(&mut f, 1000);
        f.core.units.get_mut(&a).unwrap().resonance_current = 0;
        refresh(&mut f.core, 1000).unwrap();
        assert!(f
            .core
            .request_skill_action(1000, a, "r11_aura".into(), None)
            .is_err());
        step(&mut f, 2000);
        let max = f.core.units[&a].resonance_max;
        f.core.units.get_mut(&a).unwrap().resonance_current = max;
        refresh(&mut f.core, 2000).unwrap();
        cast(&mut f, a, 3000);
        assert_eq!(witness(&f.core).percent, 10);
        let mut f = setup(10, 50);
        let a = f.a;
        step(&mut f, 1000);
        for id in [f.enemy, f.secondary] {
            f.core.units.get_mut(&id).unwrap().owner = Side::Player;
        }
        refresh(&mut f.core, 1000).unwrap();
        assert!(f
            .core
            .request_skill_action(1000, a, "r11_aura".into(), None)
            .is_err());
        step(&mut f, 2000);
        f.core.units.get_mut(&f.enemy).unwrap().owner = Side::Opponent;
        refresh(&mut f.core, 2000).unwrap();
        cast(&mut f, a, 3000);
        assert_eq!(witness(&f.core).percent, 10);
        let mut f = setup(10, 50);
        let a = f.a;
        step(&mut f, 1000);
        f.core
            .runtime_transaction(|c| {
                c.begin_sustained(
                    e,
                    a,
                    SustainedEffectDef::HitStatus {
                        duration_ms: 1000,
                        profile: HitStatusProfile {
                            definition: "ready-wait-stun".into(),
                            kind: HitStatusKind::Stun,
                            modifier_source: None,
                            vulnerability: None,
                        },
                    },
                    1000,
                )
            })
            .unwrap();
        refresh(&mut f.core, 1000).unwrap();
        assert!(f
            .core
            .request_skill_action(1500, a, "r11_aura".into(), None)
            .is_err());
        step(&mut f, 2000);
        // Ending the stun restores readiness, not the old pre-stun waiting time.
        cast(&mut f, a, 3000);
        assert_eq!(witness(&f.core).percent, 10);
    }
    #[test]
    fn composition_ready_wait_rejected_cast_preserves_wait_and_recast_replaces_window() {
        let mut f = setup(10, 50);
        let a = f.a;
        step(&mut f, 2000);
        let before = snap(&f.core);
        assert!(f
            .core
            .request_skill_action(2000, a, "missing".into(), None)
            .is_err());
        assert_eq!(snap(&f.core), before);
        cast(&mut f, a, 3000);
        assert_eq!(witness(&f.core).percent, 30);
        step(&mut f, 3001);
        f.core.units.get_mut(&a).unwrap().resonance_current = f.core.units[&a].resonance_max;
        refresh(&mut f.core, 3001).unwrap();
        cast(&mut f, a, 4001);
        assert_eq!(
            (
                witness(&f.core).percent,
                witness(&f.core).contributors[0].until_ms
            ),
            (10, 19001)
        );
        assert_eq!(
            hit(
                &mut f.core,
                a,
                f.enemy,
                DamageSource::Ability,
                DamageType::True,
                4002
            ),
            110
        );
    }
    #[test]
    fn composition_ready_wait_corrosion_withdrawal_and_terminal_remove_copy_state() {
        let mut f = setup(10, 50);
        let (a, e) = (f.a, f.enemy);
        cast(&mut f, a, 5000);
        step(&mut f, 5001);
        let mut off = f.core.clone();
        off.units.get_mut(&a).unwrap().corroded = true;
        assert_eq!(
            hit(
                &mut off,
                a,
                e,
                DamageSource::Ability,
                DamageType::True,
                5002
            ),
            100
        );
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 5002,
            })
            .unwrap();
        refresh(&mut f.core, 5002).unwrap();
        f.core
            .runtime_transaction(|c| c.clear_sustained(5003))
            .unwrap();
        assert!(f.core.equipment_defense.is_empty());
    }
    #[test]
    fn composition_ready_wait_consume_cancellation_retained_failure_and_observer_off_clone() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(10, 50);
        let a = f.a;
        step(&mut f, 3000);
        let before = snap(&f.core);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("ready_wait_activated", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || {
                    f.core
                        .request_skill_action(3000, a, "r11_aura".into(), None)
                },
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snap(&f.core), before);
        let mut probe = f.core.clone();
        let (result, admissions) = runtime_control::observe_retained_admissions(|| {
            probe.request_skill_action(3000, a, "r11_aura".into(), None)
        });
        result.unwrap();
        let needed = admissions
            .iter()
            .find(|(site, _, _)| *site == "ready_wait_observation")
            .unwrap()
            .2;
        f.core.runtime_facts.limits.max_retained_bytes = needed - 1;
        let before = snap(&f.core);
        let error = f
            .core
            .request_skill_action(3000, a, "r11_aura".into(), None)
            .unwrap_err();
        assert!(
            matches!(
                error,
                GameError::BattleRuntimeFault(runtime_control::BattleRuntimeFault {
                    site: "ready_wait_observation",
                    ..
                })
            ),
            "{error:?}"
        );
        assert_eq!(snap(&f.core), before);
        f.core.runtime_facts.limits = Default::default();
        let mut off = f.core.clone();
        off.authoring_observation = None;
        f.core
            .request_skill_action(3000, a, "r11_aura".into(), None)
            .unwrap();
        off.request_skill_action(3000, a, "r11_aura".into(), None)
            .unwrap();
        assert_eq!(snap(&f.core), snap(&off));
        assert_eq!(witness(&f.core).percent, 30);
    }
    #[test]
    fn composition_ready_wait_released_projectile_reads_impact_window_and_survives_withdrawal() {
        use crate::game::ability::{DeliveryDef, ProjectileHitPolicy};
        for (release, withdraw, expected) in
            [(6000, false, 150), (19500, false, 100), (6000, true, 100)]
        {
            let mut f = setup(10, 50);
            let (a, e) = (f.a, f.enemy);
            cast(&mut f, a, 5000);
            step(&mut f, 5001);
            let u = f.core.units.get_mut(&a).unwrap();
            u.stats.attack = 100;
            u.basic_attack.delivery = DeliveryDef::Projectile {
                speed_units_per_ms: 1000,
                hit_policy: ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            };
            u.next_basic_attack_ms = release;
            f.core.event_queue.push(BattleEvent::AttackStart {
                time_ms: release,
                attacker_instance_id: a,
                target_instance_id: Some(e),
                schedule_next: true,
                cause: Default::default(),
            });
            step(&mut f, release + 2);
            assert!(f
                .core
                .projectiles
                .values()
                .any(|p| p.source_snapshot.basic_primary == Some(e)));
            let before = f.core.units[&e].stats.current_health;
            if withdraw {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: a,
                        time_ms: release + 2,
                    })
                    .unwrap();
            }
            step(&mut f, release + 1200);
            assert_eq!(
                before - f.core.units[&e].stats.current_health,
                expected,
                "{release}/{withdraw}"
            );
        }
    }
    #[test]
    fn composition_ready_wait_applies_to_added_damage_component_without_changing_attack() {
        let mut f = setup(10, 50);
        let (a, e) = (f.a, f.enemy);
        cast(&mut f, a, 5000);
        step(&mut f, 5001);
        let before = f.core.units[&e].stats.current_health;
        f.core
            .runtime_transaction(|c| {
                let mut source = c
                    .damage_source_snapshot_for_unit(
                        a,
                        e,
                        DamageSource::Ability,
                        DamageType::True,
                        100,
                        Default::default(),
                        0,
                        5002,
                        false,
                    )
                    .unwrap();
                source.on_attack_bonus_damage.push(
                    crate::game::battle::damage::DamageBonusSnapshot {
                        source: crate::game::battle::cooldown::CooldownSource::Unit {
                            unit_instance_id: a,
                        },
                        equipment_source: None,
                        flat: 20,
                        percent: 0,
                        damage_type: DamageType::True,
                    },
                );
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: e,
                        source_snapshot: source,
                    }],
                    5002,
                );
                runtime_control::poll()
            })
            .unwrap();
        assert_eq!(before - f.core.units[&e].stats.current_health, 180);
    }
    #[test]
    fn composition_ready_wait_automatic_cast_uses_actual_start_and_no_voluntary_wait() {
        let mut f = setup(10, 50);
        let a = f.a;
        f.core.units.get_mut(&a).unwrap().skill_activation_mode = SkillActivationMode::Auto;
        f.core.event_queue.push(BattleEvent::AutoCastStart {
            time_ms: 0,
            caster_instance_id: a,
            cause: Default::default(),
        });
        step(&mut f, 1);
        assert_eq!((witness(&f.core).time_ms, witness(&f.core).percent), (0, 0));
    }
    #[test]
    fn composition_ready_wait_prepared_volley_charge_start_reinput_and_cancel_keep_fixed_deadline()
    {
        use crate::authoring::node_graph::{compile, tests as graphs, PortableGraph};
        use crate::game::battle::core::boss_runtime::tests::compiled_action_fixture::fixture_content_with_data;
        let graph = compile(
            &graphs::prepared_volley_fixture().to_string(),
            "r11_aura".into(),
        )
        .unwrap();
        let native = PortableGraph::from_ron(&graph.to_ron().unwrap())
            .unwrap()
            .load_content()
            .unwrap();
        let artifact = crate::authoring::composition::compile_yaml(SOURCE).unwrap();
        let relics = [
            "long-breath",
            "widened-horizon",
            "double-harvest",
            "unbroken-trance",
            "deep-wedge",
            "king-of-greed-stunning-treasures",
            "king-of-greed-endless-treasury",
        ];
        let (mut core, mut state, a, e, b) = fixture_content_with_data(
            &relics,
            &[],
            native,
            |_| {},
            crate::game::resource::ResourceCatalog { resources: vec![] },
            |builder| builder.with_equipment_defenses(artifact.equipment_defenses().to_vec()),
            |scenario| {
                scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(60000);
                for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
                    spawn.draft.equipped_items[0].persistent_copy_uuid =
                        Some(Uuid::from_u128(0x7201 + i as u128));
                }
            },
        );
        core.units.get_mut(&b).unwrap().fragment_source = None;
        core.units.get_mut(&b).unwrap().skill_id = None;
        for u in core.units.values_mut() {
            u.stats.current_health = 100000;
            u.stats.max_health = 100000;
            u.stats.attack_interval_ms = 100000;
            u.next_basic_attack_ms = 100000;
            u.stats.move_speed_units_per_ms = 0;
            u.stats.defense = 0;
        }
        core.enable_authoring_observation();
        core.runtime_transaction(|c| {
            c.clear_prepared_volleys(0)?;
            c.reconcile_prepared_volleys(0)?;
            c.reconcile_sustained(0)
        })
        .unwrap();
        core.step_battle_execution_until(&mut state, 3000).unwrap();
        core.request_skill_action(3000, a, "r11_aura".into(), None)
            .unwrap();
        core.step_battle_execution_until(&mut state, 3001).unwrap();
        assert_eq!(core.channel_started_at(a, 3001).unwrap(), Some(3000));
        assert_eq!(
            (
                witness(&core).percent,
                witness(&core).contributors[0].until_ms
            ),
            (30, 18000)
        );
        let mut cancelled = core.clone();
        cancelled
            .runtime_transaction(|c| c.stop_channel(a, 3002))
            .unwrap();
        assert_eq!(
            hit(
                &mut cancelled,
                a,
                e,
                DamageSource::Ability,
                DamageType::True,
                3003
            ),
            130
        );
        core.step_battle_execution_until(&mut state, 4000).unwrap();
        let activation = core.prepared_volley_reinput(a, &"r11_aura".into()).unwrap();
        core.apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
            unit_id: a,
            skill_id: "r11_aura".into(),
            activation,
            target: None,
            time_ms: 4000,
        })
        .unwrap();
        assert_eq!(
            (
                witness(&core).time_ms,
                witness(&core).contributors[0].until_ms
            ),
            (3000, 18000)
        );
        assert_eq!(
            hit(
                &mut core,
                a,
                e,
                DamageSource::Ability,
                DamageType::True,
                17999
            ),
            130
        );
        assert_eq!(
            hit(
                &mut core,
                a,
                e,
                DamageSource::Ability,
                DamageType::True,
                18000
            ),
            100
        );
    }
    #[test]
    fn composition_ready_wait_owner_kill_resonance_settlement_starts_wait_at_reward_time() {
        let mut f = setup_edit(10, 50, |doc| {
            doc["version"] = json!(2);
            doc["values"].as_array_mut().unwrap().push(json!({"name":"kill-charge","value":{"StateProgram":{
                "id":"kill-charge","owns_use":false,"end_on_source_exit":true,"replacement":"ReplaceSameSource","fields":[],
                "handlers":[{"id":"charge","event":"OwnerKill","body":[{"GrantOwnerResonance":{"amount":{"moment":"Live","root":{"Const":{"value":100,"unit":"Magnitude"}}}}}]}]
            }}}));
            doc["equipment_states"] =
                json!([{"equipment":"r11_weapon","program":{"Shared":"kill-charge"}}]);
        });
        let (a, e) = (f.a, f.enemy);
        f.core.units.get_mut(&a).unwrap().resonance_current = 0;
        refresh(&mut f.core, 0).unwrap();
        step(&mut f, 1000);
        f.core.units.get_mut(&e).unwrap().stats.current_health = 1;
        hit(
            &mut f.core,
            a,
            e,
            DamageSource::Ability,
            DamageType::True,
            1000,
        );
        f.core
            .runtime_transaction(|c| {
                c.reconcile_sustained(1000)?;
                c.settle_equipment_resonance_rewards(1000)
            })
            .unwrap();
        assert_eq!(
            f.core.units[&a].resonance_current,
            f.core.units[&a].resonance_max
        );
        f.core
            .request_skill_action(2000, a, "r11_aura".into(), None)
            .unwrap();
        assert_eq!(
            (
                witness(&f.core).contributors[0].waited_ms,
                witness(&f.core).percent
            ),
            (1000, 10)
        );
    }
    #[test]
    fn composition_ready_wait_owned_reflection_classification_is_distinct_from_environment() {
        let mut f = setup(10, 50);
        let (a, e) = (f.a, f.enemy);
        cast(&mut f, a, 5000);
        step(&mut f, 5001);
        for (reaction, expected) in [(false, 100), (true, 150)] {
            let before = f.core.units[&e].stats.current_health;
            f.core
                .runtime_transaction(|c| {
                    let mut source = c
                        .damage_source_snapshot_for_unit(
                            a,
                            e,
                            DamageSource::Environment,
                            DamageType::True,
                            100,
                            Default::default(),
                            0,
                            5002,
                            false,
                        )
                        .unwrap();
                    source.fact_source.reaction = reaction;
                    c.process_commands(
                        vec![BattleCommand::ApplyDamage {
                            target_id: e,
                            source_snapshot: source,
                        }],
                        5002,
                    );
                    runtime_control::poll()
                })
                .unwrap();
            assert_eq!(before - f.core.units[&e].stats.current_health, expected);
        }
    }
}
