mod composition_kill_cast {
    include!("composition_kill_speed_tests.inc.rs");
    use super::*;
    use crate::game::{
        ability::state_program::StateScalar,
        battle::{
            damage::{BattleCommand, DamageSource, DamageType},
            runtime_control,
        },
    };
    const SOURCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/kill_cast_state.skill.yaml"
    ));
    fn setup(armor: bool, initial: u32, chance: u32, area: bool) -> Fixture {
        fixture_edit(armor, initial, chance, area, |_| {})
    }
    fn fixture_edit(
        armor: bool,
        initial: u32,
        chance: u32,
        area: bool,
        edit: impl FnOnce(&mut Value),
    ) -> Fixture {
        fixture_edit_skill(armor, initial, chance, area, edit, |_| {})
    }
    fn fixture_edit_skill(
        armor: bool,
        initial: u32,
        chance: u32,
        area: bool,
        edit: impl FnOnce(&mut Value),
        configure: impl FnOnce(&mut String),
    ) -> Fixture {
        let mut f = fixture_copies_with_setup(
            0,
            false,
            false,
            |doc| {
                *doc = parse_yaml(SOURCE).unwrap();
                doc["values"][1]["value"]["StateProgram"]["fields"][0]["value"]["Integer"]
                    ["initial"] = json!(initial);
                doc["values"][1]["value"]["StateProgram"]["handlers"][0]["body"][0]["Chance"]
                    ["percent"]["root"]["Const"]["value"] = json!(chance);
                if armor {
                    doc["equipment_states"][0]["program"]["Shared"] = json!("direct-kill-heal");
                }
                edit(doc);
            },
            |skill| {
                *skill = r12_control_ron("Damage(amount:100,damage_type:True)")
                    .replace("focus_time_ms: 0", "focus_time_ms: 10");
                if area {
                    *skill=skill.replace("delivery: Instant",r#"delivery: TileArea(footprint:(include_anchor_tile:true,rows:["XXX","X@X","XXX"]),area:(anchor:CastTarget,tile_origin:Anchor,duration_ms:201,tick_interval_ms:Some(100)))"#);
                }
                configure(skill);
            },
            |s| s.terminal_outcomes = BattleScenario::fixture_survival_outcomes(100000),
            None,
        );
        for u in f.core.units.values_mut() {
            u.stats.attack = 100;
            u.stats.magic_resist = 0;
            u.basic_attack.windup_ms = 1;
        }
        for id in [f.a, f.b] {
            let u = f.core.units.get_mut(&id).unwrap();
            u.resonance_current = u.resonance_max;
            u.next_action_time = 0;
        }
        f.core.enable_authoring_observation();
        f
    }
    fn run(f: &mut Fixture, now: u64) {
        f.core
            .step_battle_execution_until(&mut f.state, now)
            .unwrap();
    }
    fn cast(f: &mut Fixture, actor: UnitInstanceId, now: u64) {
        run(f, now);
        f.core
            .request_skill_action(now, actor, "r11_aura".into(), None)
            .unwrap();
    }
    fn hearts(f: &Fixture, actor: UnitInstanceId) -> i128 {
        f.core
            .sustained_effect_snapshot()
            .unwrap()
            .equipment_states
            .iter()
            .find(|s| s.owner == actor)
            .and_then(|s| s.state.fields.iter().find(|v| v.name == "hearts"))
            .map_or(0, |v| match v.value {
                StateScalar::Integer(n) => n,
                _ => panic!("numeric field"),
            })
    }
    fn basic_kill(f: &mut Fixture, actor: UnitInstanceId, target: UnitInstanceId, now: u64) {
        run(f, now);
        f.core.units.get_mut(&target).unwrap().stats.current_health = 1;
        let u = f.core.units.get_mut(&actor).unwrap();
        u.current_target = Some(target);
        u.next_basic_attack_ms = now;
        u.next_action_time = now;
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: now,
                    attacker_instance_id: actor,
                    target_instance_id: Some(target),
                    schedule_next: false,
                    cause: Default::default(),
                },
                now,
            )
            .unwrap();
        run(f, now + 1);
        assert!(
            f.core.units[&target].lifecycle.is_dead(),
            "real basic must finalize victim"
        );
    }
    #[test]
    fn composition_kill_cast_schema_roundtrip_and_event_unit_diagnostics() {
        let doc = parse_yaml(SOURCE).unwrap();
        let artifact = compile_json(&doc.to_string()).unwrap();
        Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
        for case in 0..3 {
            let mut bad = doc.clone();
            let p = &mut bad["values"][1]["value"]["StateProgram"];
            match case {
                0 => p["handlers"][1]["event"] = json!("OwnerKill"),
                1 => {
                    p["handlers"][0]["body"][0]["Chance"]["percent"]["root"]["Const"]["unit"] =
                        json!("Count")
                }
                _ => p["handlers"][0]["event"] = json!("OwnerSkillCast"),
            }
            let error = format!("{:?}", compile_json(&bad.to_string()).unwrap_err());
            assert!(
                error.contains("handlers")
                    && (error.contains("Chance")
                        || error.contains("Scalar")
                        || error.contains("AddCastDamage")),
                "{error}"
            );
        }
    }
    #[test]
    fn composition_kill_cast_nested_predicate_and_unknown_condition_keep_branch_path() {
        for allowed in [false, true] {
            let mut f = fixture_edit(false, 0, 100, false, |d| {
                d["predicates"] = json!([{"name":"allow-heart","context":"State","parameters":[],"body":{"Always":allowed}}]);
                let body = &mut d["values"][1]["value"]["StateProgram"]["handlers"][0]["body"][0]
                    ["Chance"]["then_ops"];
                let grant = body.take();
                *body = json!([{"If":{"condition":{"Predicate":{"name":"allow-heart","arguments":[]}},"then_ops":grant,"else_ops":[]}}]);
            });
            let (actor, victim) = (f.a, f.enemy);
            basic_kill(&mut f, actor, victim, 10);
            assert_eq!(hearts(&f, actor), i128::from(allowed));
            assert_eq!(hearts(&f, f.b), 0);
        }
        let mut bad = parse_yaml(SOURCE).unwrap();
        bad["values"][1]["value"]["StateProgram"]["handlers"][0]["body"][0]["Chance"]["then_ops"] = json!([{"If":{"condition":{"Predicate":{"name":"missing-condition","arguments":[]}},"then_ops":[],"else_ops":[]}}]);
        let error = format!("{:?}", compile_json(&bad.to_string()).unwrap_err());
        assert!(
            error.contains("handlers[0].body[0].then_ops[0].condition")
                && error.contains("missing-condition"),
            "{error}"
        );
    }
    #[test]
    fn composition_kill_cast_armor_actual_active_heals_per_victim_and_basic_is_excluded() {
        let mut f = setup(true, 0, 100, false);
        let (a, e) = (f.a, f.enemy);
        let before = f.core.units[&a].stats.current_health;
        basic_kill(&mut f, a, e, 10);
        assert_eq!(f.core.units[&a].stats.current_health, before);
        let mut f = setup(true, 0, 100, true);
        let (a, e, s) = (f.a, f.enemy, f.secondary);
        super::super::compiled_action_fixture::place(&mut f.core, s, 3, 1);
        for id in [e, s] {
            f.core.units.get_mut(&id).unwrap().stats.current_health = 50;
        }
        let before = f.core.units[&a].stats.current_health;
        cast(&mut f, a, 10);
        run(&mut f, 20);
        assert!(f.core.units[&e].lifecycle.is_dead() && f.core.units[&s].lifecycle.is_dead());
        assert_eq!(f.core.units[&a].stats.current_health, before + 14);
        assert_eq!(
            f.core
                .sustained_effect_snapshot()
                .unwrap()
                .equipment_states
                .iter()
                .find(|r| r.owner == a)
                .unwrap()
                .state
                .events,
            2
        );
    }
    #[test]
    fn composition_kill_cast_probability_extremes_and_cap_are_authored() {
        for chance in [0, 100] {
            let mut f = fixture_edit(false, 0, chance, false, |d| {
                d["values"][1]["value"]["StateProgram"]["handlers"][0]["body"][0]["Chance"]
                    ["then_ops"][0]["Set"]["value"]["root"]["Clamp"]["max"]["Const"]["value"] =
                    json!(1)
            });
            let (a, e, s) = (f.a, f.enemy, f.secondary);
            basic_kill(&mut f, a, e, 10);
            assert_eq!(hearts(&f, a), if chance == 0 { 0 } else { 1 });
            super::super::compiled_action_fixture::place(&mut f.core, s, 3, 1);
            basic_kill(&mut f, a, s, 30);
            assert_eq!(hearts(&f, a), if chance == 0 { 0 } else { 1 });
            assert_eq!(hearts(&f, f.b), 0);
        }
    }
    #[test]
    fn composition_kill_cast_one_stack_fixed_all_ticks_and_midcast_gain_is_for_next_cast() {
        let mut f = setup(false, 1, 100, true);
        let (a, e, s) = (f.a, f.enemy, f.secondary);
        super::super::compiled_action_fixture::place(&mut f.core, s, 3, 1);
        f.core.units.get_mut(&e).unwrap().stats.current_health = 50;
        f.core.units.get_mut(&s).unwrap().stats.current_health = 10000;
        f.core.units.get_mut(&s).unwrap().stats.max_health = 10000;
        cast(&mut f, a, 10);
        assert_eq!(hearts(&f, a), 0, "consume exactly one before focus");
        let action = f.core.units[&a]
            .pending_skill_cast
            .as_ref()
            .unwrap()
            .action
            .as_ref()
            .unwrap();
        let source = f.core.runtime_facts.accepted_source(action).unwrap();
        let capture = source.cast_damage().unwrap();
        assert_eq!(capture.percent, 20);
        assert_eq!(capture.actor, a);
        assert_eq!(capture.contributors.len(), 1);
        assert_eq!(
            capture.contributors[0].equipment.persistent_copy_uuid,
            match owner(&f.core, a) {
                ResourceOwner::EquipmentCopy(id) => Some(id),
                _ => None,
            }
        );
        run(&mut f, 20);
        assert_eq!(hearts(&f, a), 1);
        assert_eq!(f.core.units[&s].stats.current_health, 9880);
        run(&mut f, 220);
        assert_eq!(f.core.units[&s].stats.current_health, 9640);
        assert_eq!(hearts(&f, a), 1);
        let unit = f.core.units.get_mut(&a).unwrap();
        unit.resonance_current = unit.resonance_max;
        cast(&mut f, a, 250);
        assert_eq!(hearts(&f, a), 0);
        run(&mut f, 260);
        assert_eq!(f.core.units[&s].stats.current_health, 9520);
    }
    #[test]
    fn composition_kill_cast_empty_stacks_and_two_owners_remain_independent() {
        for initial in [0, 3] {
            let mut f = setup(false, initial, 100, false);
            let (a, b, e) = (f.a, f.b, f.enemy);
            cast(&mut f, a, 10);
            run(&mut f, 20);
            assert_eq!(
                f.core.units[&e].stats.current_health,
                if initial == 0 { 900 } else { 880 }
            );
            assert_eq!(hearts(&f, a), if initial == 0 { 0 } else { 2 });
            cast(&mut f, b, 30);
            assert_eq!(hearts(&f, b), if initial == 0 { 0 } else { 2 });
            assert_eq!(hearts(&f, a), if initial == 0 { 0 } else { 2 });
        }
    }
    #[test]
    fn composition_kill_cast_periodic_reaction_and_unattributed_ability_do_not_grant() {
        for kind in [
            DamageSource::BuffTick,
            DamageSource::Environment,
            DamageSource::Ability,
        ] {
            let mut f = setup(false, 0, 100, false);
            let (a, e) = (f.a, f.enemy);
            f.core
                .runtime_transaction(|c| {
                    let mut snapshot = c
                        .damage_source_snapshot_for_unit(
                            a,
                            e,
                            kind,
                            DamageType::True,
                            20000,
                            Default::default(),
                            0,
                            10,
                            false,
                        )
                        .unwrap();
                    if kind == DamageSource::Environment {
                        snapshot.fact_source.reaction = true;
                    }
                    c.process_commands(
                        vec![BattleCommand::ApplyDamage {
                            target_id: e,
                            source_snapshot: snapshot,
                        }],
                        10,
                    );
                    runtime_control::poll()
                })
                .unwrap();
            assert!(f.core.units[&e].lifecycle.is_dead());
            assert_eq!(hearts(&f, a), 0);
        }
    }
    #[test]
    fn composition_kill_cast_rejected_cast_preserves_and_started_withdraw_does_not_refund() {
        let mut f = setup(false, 2, 100, false);
        let a = f.a;
        assert!(f
            .core
            .request_skill_action(0, a, "missing-skill".into(), None)
            .is_err());
        cast(&mut f, a, 10);
        assert_eq!(hearts(&f, a), 1);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 11,
            })
            .unwrap();
        run(&mut f, 20);
        assert!(f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .equipment_states
            .iter()
            .all(|s| s.owner != a));
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 1000);
    }
    #[test]
    fn composition_kill_cast_probability_replays_with_clone_and_observation_noise() {
        let base = setup(false, 0, 47, false);
        let mut seen = std::collections::BTreeSet::new();
        for seed in 0..12 {
            let mut results = Vec::new();
            for observe in [false, true] {
                let mut f = Fixture {
                    core: base.core.clone(),
                    state: base.state.clone(),
                    a: base.a,
                    b: base.b,
                    enemy: base.enemy,
                    secondary: base.secondary,
                    run: base.run.clone(),
                };
                f.core.seed = seed;
                if !observe {
                    f.core.authoring_observation = None;
                } else {
                    f.core.record_event_log(
                        0,
                        crate::game::battle::event_log::BattleLogEvent::BattleStart {
                            width: 8,
                            height: 8,
                        },
                    );
                }
                let (a, e) = (f.a, f.enemy);
                basic_kill(&mut f, a, e, 10);
                let value = hearts(&f, a);
                seen.insert(value);
                results.push(value);
            }
            assert_eq!(results[0], results[1]);
        }
        assert_eq!(seen, std::collections::BTreeSet::from([0, 1]));
    }
    #[test]
    fn composition_kill_cast_released_area_keeps_fixed_bonus_after_withdraw() {
        let mut f = setup(false, 2, 100, true);
        let (a, e) = (f.a, f.enemy);
        cast(&mut f, a, 10);
        run(&mut f, 20);
        assert_eq!(hearts(&f, a), 1);
        assert_eq!(f.core.units[&e].stats.current_health, 880);
        f.core
            .apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: a,
                time_ms: 21,
            })
            .unwrap();
        run(&mut f, 220);
        assert_eq!(f.core.units[&e].stats.current_health, 640);
        assert!(f
            .core
            .sustained_effect_snapshot()
            .unwrap()
            .equipment_states
            .iter()
            .all(|s| s.owner != a));
    }
    #[test]
    fn composition_kill_cast_nondamage_consumes_and_reinput_inherits_without_new_consumption() {
        let mut f = fixture_edit_skill(
            false,
            2,
            100,
            false,
            |_| {},
            |s| *s = s.replace("Damage(amount:100,damage_type:True)", "Heal(amount:1)"),
        );
        let a = f.a;
        cast(&mut f, a, 10);
        assert_eq!(hearts(&f, a), 1);
        run(&mut f, 20);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 1000);
        let mut f = fixture_edit_skill(
            false,
            2,
            100,
            false,
            |_| {},
            |s| *s = r11_hello_goodbye_ron("Snapshot"),
        );
        let a = f.a;
        f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 10000;
        f.core.units.get_mut(&f.enemy).unwrap().stats.max_health = 10000;
        cast(&mut f, a, 10);
        run(&mut f, 11);
        assert_eq!(hearts(&f, a), 1);
        let activation = f.core.sustained_effect_snapshot().unwrap().active[0].activation;
        f.core
            .apply_live_command(BattleLiveCommand::ReinputSustainedSkill {
                unit_id: a,
                skill_id: "r11_aura".into(),
                activation,
                target: None,
                time_ms: 20,
            })
            .unwrap();
        assert_eq!(hearts(&f, a), 1);
        let latest = f
            .core
            .runtime_fact_summary()
            .latest_damage
            .as_ref()
            .unwrap();
        assert_eq!(
            latest.source.cast.as_ref().unwrap().skill_id.as_str(),
            "r11_goodbye"
        );
        assert_eq!(latest.source.cast_damage().unwrap().percent, 20);
    }
    #[test]
    fn composition_kill_cast_cancel_after_consume_restores_public_transaction_and_retry() {
        use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut f = setup(false, 2, 100, false);
        let a = f.a;
        run(&mut f, 10);
        let snap = |c: &BattleCore| {
            (
                c.transaction_debug_snapshot(),
                c.event_queue_debug_snapshot(),
                serde_json::to_value(c.runtime_fact_view()).unwrap(),
            )
        };
        let before = snap(&f.core);
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("action_accept", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || f.core.request_skill_action(10, a, "r11_aura".into(), None),
            )
        });
        assert!(result.is_err() && token.is_cancelled());
        assert_eq!(snap(&f.core), before);
        f.core
            .request_skill_action(10, a, "r11_aura".into(), None)
            .unwrap();
        assert_eq!(hearts(&f, a), 1);
        run(&mut f, 20);
        assert_eq!(f.core.units[&f.enemy].stats.current_health, 880);
    }
    #[test]
    fn composition_kill_cast_appended_bonus_finisher_does_not_qualify_as_direct_kill() {
        let mut f = setup(false, 0, 100, false);
        let (a, e) = (f.a, f.enemy);
        f.core.units.get_mut(&a).unwrap().next_basic_attack_ms = 10;
        f.core
            .process_event(
                BattleEvent::AttackStart {
                    time_ms: 10,
                    attacker_instance_id: a,
                    target_instance_id: Some(e),
                    schedule_next: false,
                    cause: Default::default(),
                },
                10,
            )
            .unwrap();
        let mut source = f
            .core
            .event_queue
            .iter()
            .find_map(|event| match event.as_ref() {
                BattleEvent::AttackResolve {
                    source_snapshot,
                    attacker_instance_id,
                    ..
                } if *attacker_instance_id == a => Some(source_snapshot.clone()),
                _ => None,
            })
            .unwrap();
        source.base_damage = 10;
        source
            .on_attack_bonus_damage
            .push(crate::game::battle::damage::DamageBonusSnapshot {
                source: crate::game::battle::cooldown::CooldownSource::Unit {
                    unit_instance_id: a,
                },
                equipment_source: None,
                flat: 10000,
                percent: 0,
                damage_type: DamageType::True,
            });
        f.core.runtime_transaction(|c|{
            c.execute_committed_damage(crate::game::battle::core::damage_runtime::CommittedDamageInput{target_id:e,source_snapshot:source,committed_at_ms:11,policy:crate::game::battle::core::damage_runtime::CommittedDamagePolicy::GeneralCommand}).ok_or(GameError::InvalidAction)?;
            runtime_control::poll()
        }).unwrap();
        assert!(f.core.units[&e].lifecycle.is_dead());
        assert_eq!(hearts(&f, a), 0);
    }
}
