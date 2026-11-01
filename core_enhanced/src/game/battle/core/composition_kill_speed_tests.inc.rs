// Data-only combination of the direct-kill listener and finite state reward.
const KILL_SPEED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/kill_speed.skill.yaml"
));
fn kill_speed_fixture(area: bool) -> Fixture {
    fixture_edit(false, 0, 100, area, |d| {
        *d = parse_yaml(KILL_SPEED).unwrap()
    })
}
fn kill_speed_rewards(f: &Fixture) -> Vec<crate::game::ability::sustained::SustainedEffectView> {
    f.core
        .sustained_effect_snapshot()
        .unwrap()
        .active
        .into_iter()
        .filter(|r| {
            r.target_state
                .as_ref()
                .is_some_and(|p| p.definition == "delivery-coat-speed")
        })
        .collect()
}
fn kill_speed_interval(f: &Fixture, owner: UnitInstanceId) -> u64 {
    f.core.sustained_basic_interval(owner, 1000)
}
#[test]
fn composition_kill_speed_schema_shared_roundtrip_and_duration_diagnostic() {
    let doc = parse_yaml(KILL_SPEED).unwrap();
    let artifact = compile_json(&doc.to_string()).unwrap();
    let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    loaded.load_native().unwrap();
    assert_eq!(loaded.equipment_states().len(), 1);
    assert!(loaded.sources()["equipment_states[0].program"]
        .definition
        .ends_with("StateProgram"));
    let mut bad = doc.clone();
    bad["values"][0]["value"]["StateProgram"]["handlers"][0]["body"][0]["ApplyOwnerState"]
        ["duration"]["root"]["Const"]["unit"] = json!("Count");
    let error = compile_json(&bad.to_string()).unwrap_err().to_string();
    assert!(
        error.contains("duration") && error.contains("Milliseconds"),
        "{error}"
    );
}
#[test]
fn composition_kill_speed_actual_basics_refresh_not_stack_and_old_expiry_is_harmless() {
    let mut f = kill_speed_fixture(false);
    let (a, e, s, b) = (f.a, f.enemy, f.secondary, f.b);
    basic_kill(&mut f, a, e, 10);
    assert_eq!(kill_speed_interval(&f, a), 667);
    assert_eq!(kill_speed_interval(&f, b), 1000);
    assert_eq!(kill_speed_rewards(&f).len(), 1);
    super::super::compiled_action_fixture::place(&mut f.core, s, 3, 1);
    basic_kill(&mut f, a, s, 100);
    assert_eq!(kill_speed_interval(&f, a), 667);
    assert_eq!(kill_speed_rewards(&f).len(), 1);
    run(&mut f, 3011);
    assert_eq!(kill_speed_interval(&f, a), 667);
    run(&mut f, 3100);
    assert_eq!(kill_speed_interval(&f, a), 667);
    run(&mut f, 3101);
    assert_eq!(kill_speed_interval(&f, a), 1000);
    assert!(kill_speed_rewards(&f).is_empty());
    let states = f.core.sustained_effect_snapshot().unwrap().equipment_states;
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].state.events, 2);
    assert!(states[0].persistent_copy_uuid.is_some());
}
#[test]
fn composition_kill_speed_direct_active_multikill_same_time_refresh_and_owner_only() {
    let mut f = kill_speed_fixture(true);
    let (a, e, s, b) = (f.a, f.enemy, f.secondary, f.b);
    super::super::compiled_action_fixture::place(&mut f.core, s, 3, 1);
    for id in [e, s] {
        f.core.units.get_mut(&id).unwrap().stats.current_health = 50;
    }
    cast(&mut f, a, 10);
    run(&mut f, 20);
    assert!(f.core.units[&e].lifecycle.is_dead() && f.core.units[&s].lifecycle.is_dead());
    assert_eq!(kill_speed_rewards(&f).len(), 1);
    assert_eq!(kill_speed_interval(&f, a), 667);
    assert_eq!(kill_speed_interval(&f, b), 1000);
    assert_eq!(
        f.core.sustained_effect_snapshot().unwrap().equipment_states[0]
            .state
            .events,
        2
    );
    run(&mut f, 3020);
    assert_eq!(kill_speed_interval(&f, a), 1000);
}
#[test]
fn composition_kill_speed_indirect_and_ally_kills_do_not_grant_owner_reward() {
    for kind in [
        DamageSource::BuffTick,
        DamageSource::Environment,
        DamageSource::Ability,
    ] {
        let mut f = kill_speed_fixture(false);
        let (a, e) = (f.a, f.enemy);
        f.core
            .runtime_transaction(|c| {
                let mut source = c
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
                    source.fact_source.reaction = true;
                }
                c.process_commands(
                    vec![BattleCommand::ApplyDamage {
                        target_id: e,
                        source_snapshot: source,
                    }],
                    10,
                );
                runtime_control::poll()
            })
            .unwrap();
        assert!(f.core.units[&e].lifecycle.is_dead());
        assert!(kill_speed_rewards(&f).is_empty(), "{kind:?}");
    }
    let mut f = kill_speed_fixture(false);
    let (a, b, e) = (f.a, f.b, f.enemy);
    super::super::compiled_action_fixture::place(&mut f.core, b, 2, 1);
    super::super::compiled_action_fixture::place(&mut f.core, e, 3, 1);
    basic_kill(&mut f, b, e, 10);
    assert_eq!(kill_speed_interval(&f, a), 1000);
    assert_eq!(kill_speed_interval(&f, b), 667);
}
#[test]
fn composition_kill_speed_exit_corrosion_unequip_death_and_terminal_release() {
    for boundary in ["withdraw", "corrosion", "unequip", "death", "terminal"] {
        let mut f = kill_speed_fixture(false);
        let (a, e) = (f.a, f.enemy);
        basic_kill(&mut f, a, e, 10);
        assert_eq!(kill_speed_interval(&f, a), 667);
        if boundary == "withdraw" {
            f.core
                .apply_live_command(BattleLiveCommand::WithdrawUnit {
                    unit_id: a,
                    time_ms: 20,
                })
                .unwrap();
        } else {
            let secondary = f.secondary;
            f.core
                .runtime_transaction(|c| {
                    match boundary {
                        "corrosion" => c.units.get_mut(&a).unwrap().corroded = true,
                        "unequip" => {
                            let id = c
                                .items
                                .values()
                                .find(|i| i.owner_unit_instance == a)
                                .unwrap()
                                .instance_id;
                            c.items.remove(&id);
                        }
                        "death" => {
                            let source = c
                                .damage_source_snapshot_for_unit(
                                    secondary,
                                    a,
                                    DamageSource::Environment,
                                    DamageType::True,
                                    20000,
                                    Default::default(),
                                    0,
                                    20,
                                    false,
                                )
                                .unwrap();
                            c.process_commands(
                                vec![BattleCommand::ApplyDamage {
                                    target_id: a,
                                    source_snapshot: source,
                                }],
                                20,
                            );
                        }
                        _ => c.clear_sustained(20)?,
                    }
                    c.reconcile_sustained(20)
                })
                .unwrap();
        }
        assert!(kill_speed_rewards(&f).is_empty(), "{boundary}");
        assert!(
            f.core
                .sustained_effect_snapshot()
                .unwrap()
                .equipment_states
                .is_empty(),
            "{boundary}"
        );
        assert_eq!(kill_speed_interval(&f, a), 1000, "{boundary}");
    }
}
#[test]
fn composition_kill_speed_cancel_and_retained_failure_restore_lethal_and_retry() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    let mut f = kill_speed_fixture(false);
    let (a, e) = (f.a, f.enemy);
    run(&mut f, 10);
    f.core.units.get_mut(&e).unwrap().stats.current_health = 1;
    let u = f.core.units.get_mut(&a).unwrap();
    u.current_target = Some(e);
    u.next_basic_attack_ms = 10;
    u.next_action_time = 10;
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
            || f.core.step_battle_execution_until(&mut f.state, 11),
        )
    });
    assert!(result.is_err() && token.is_cancelled());
    assert_eq!(snapshot(&f), before);
    let mut probe = f.core.clone();
    let mut state = f.state.clone();
    let (result, admissions) = runtime_control::observe_retained_admissions(|| {
        probe.step_battle_execution_until(&mut state, 11)
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
        .step_battle_execution_until(&mut f.state, 11)
        .unwrap_err();
    assert!(
        matches!(
            error,
            GameError::BattleRuntimeFault(runtime_control::BattleRuntimeFault {
                code: runtime_control::RuntimeFaultCode::RetainedByteLimit,
                ..
            })
        ),
        "{error:?}"
    );
    assert_eq!(snapshot(&f), before);
    f.core.runtime_facts.limits = Default::default();
    run(&mut f, 11);
    assert!(f.core.units[&e].lifecycle.is_dead());
    assert_eq!(kill_speed_rewards(&f).len(), 1);
    assert_eq!(kill_speed_interval(&f, a), 667);
}
#[test]
fn composition_kill_speed_clone_and_observation_preserve_ordered_execution() {
    let mut f = kill_speed_fixture(true);
    f.core.units.get_mut(&f.enemy).unwrap().stats.current_health = 1;
    let mut other = Fixture {
        core: f.core.clone(),
        state: f.state.clone(),
        a: f.a,
        b: f.b,
        enemy: f.enemy,
        secondary: f.secondary,
        run: f.run.clone(),
    };
    other.core.authoring_observation = None;
    for x in [&mut f, &mut other] {
        let a = x.a;
        cast(x, a, 10);
        run(x, 21);
        assert_eq!(kill_speed_interval(x, a), 667);
    }
    assert_eq!(
        f.core.transaction_debug_snapshot(),
        other.core.transaction_debug_snapshot()
    );
    assert_eq!(
        f.core.event_queue_debug_snapshot(),
        other.core.event_queue_debug_snapshot()
    );
    assert_eq!(
        serde_json::to_value(f.core.runtime_fact_summary()).unwrap(),
        serde_json::to_value(other.core.runtime_fact_summary()).unwrap()
    );
    assert_eq!(kill_speed_rewards(&f), kill_speed_rewards(&other));
}
