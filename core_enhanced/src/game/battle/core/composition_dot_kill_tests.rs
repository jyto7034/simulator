// Included in composition_equipment_state; exercises the authored equipment path.
use crate::game::battle::damage::{DamageModifiers, DamageType};
const DOT_KILL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/equipment_dot_kill.skill.yaml"
));
fn kill_fixture() -> Fixture {
    let mut f = dot_fixture_source(&crate::authoring::composition::canonical_presence_tests::equipment_fixture("accessory"));
    for unit in f.core.units.values_mut() {
        unit.resonance_current = 0;
    }
    f
}
fn lethal(
    core: &mut BattleCore,
    owner: UnitInstanceId,
    victim: UnitInstanceId,
    kind: DamageSource,
    now: u64,
) -> Result<(), GameError> {
    core.runtime_transaction(|c| {
        let source = c
            .damage_source_snapshot_for_unit(
                owner,
                victim,
                kind,
                DamageType::True,
                20_000,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .unwrap();
        let result = c.execute_committed_damage(CommittedDamageInput {
            target_id: victim,
            source_snapshot: source,
            committed_at_ms: now,
            policy: CommittedDamagePolicy::GeneralCommand,
        });
        crate::game::battle::runtime_control::poll()?;
        result.ok_or(GameError::InvalidAction)?;
        Ok(())
    })
}
fn settle_kill(f: &mut Fixture, now: u64) {
    f.core
        .runtime_transaction(|c| {
            c.reconcile_sustained(now)?;
            c.settle_equipment_resonance_rewards(now)
        })
        .unwrap();
}
#[test]
fn composition_dot_kill_direct_duplicate_dot_once_and_copy_source() {
    let mut f = kill_fixture();
    let (a, b, e) = (f.a, f.b, f.enemy);
    for name in ["dot_one", "dot_two"] {
        dot_apply(&mut f, a, e, name, 0, 5000);
    }
    f.core.enable_authoring_observation();
    let snapshot = f
        .core
        .damage_source_snapshot_for_unit(
            a,
            e,
            DamageSource::Ability,
            DamageType::True,
            20_000,
            DamageModifiers::default(),
            0,
            10,
            false,
        )
        .unwrap();
    let replay = f
        .core
        .capture_equipment_kill(&snapshot, e, false, true, 10)
        .unwrap()
        .unwrap();
    lethal(&mut f.core, a, e, DamageSource::Ability, 10).unwrap();
    assert!(f.core.units[&e].lifecycle.is_dead());
    assert_eq!(
        f.core.units[&a].resonance_current, 0,
        "reward waits for settlement"
    );
    settle_kill(&mut f, 10);
    assert_eq!(f.core.units[&a].resonance_current, 7);
    assert_eq!(f.core.units[&b].resonance_current, 0);
    let view = states(&f).into_iter().find(|v| v.owner == a).unwrap();
    assert_eq!(view.state.events, 1);
    assert_eq!(
        view.persistent_copy_uuid,
        f.core
            .items
            .values()
            .find(|i| i.owner_unit_instance == a)
            .unwrap()
            .persistent_copy_uuid
    );
    // Replay the same finalized death through its typed input cannot grant twice.
    f.core
        .runtime_transaction(|c| c.dispatch_equipment_kill(replay, 10))
        .unwrap();
    settle_kill(&mut f, 10);
    assert_eq!(f.core.units[&a].resonance_current, 7);
    f.core.process_commands(
        vec![crate::game::battle::damage::BattleCommand::ApplyHeal {
            target_id: a,
            flat: 1,
            percent: 0,
            source_id: Some(a),
        }],
        11,
    );
    let observations = serde_json::to_value(&f.core.authoring_observation).unwrap();
    assert_eq!(observations["latest"]["change"]["channel"], "hp");
    assert_eq!(
        observations["latest_sourced_resonance"]["change"]["channel"],
        "resonance"
    );
    assert_eq!(
        observations["latest_sourced_resonance"]["authored"]["field_path"],
        "equipment_states[0].program.handlers[0].body[0].then[0]"
    );
}
#[test]
fn composition_dot_kill_other_owner_expired_zero_friendly_and_environment_excluded() {
    for case in 0..6 {
        let mut f = kill_fixture();
        let (a, b, e) = (f.a, f.b, f.enemy);
        let dot_owner = if case == 0 { b } else { a };
        let victim = if case == 4 { b } else { e };
        dot_apply(
            &mut f,
            dot_owner,
            victim,
            if case == 2 { "zero_dot" } else { "dot_one" },
            0,
            if case == 1 { 10 } else { 5000 },
        );
        let killer = if case == 3 { b } else { a };
        lethal(
            &mut f.core,
            killer,
            victim,
            if case == 5 {
                DamageSource::Environment
            } else {
                DamageSource::Ability
            },
            10,
        )
        .unwrap();
        settle_kill(&mut f, 10);
        assert_eq!(f.core.units[&a].resonance_current, 0, "case {case}");
        assert_eq!(f.core.units[&b].resonance_current, 0, "case {case}");
    }
}
#[test]
fn composition_dot_kill_real_basic_and_periodic_tick_reward() {
    let mut f = kill_fixture();
    let (a, e) = (f.a, f.enemy);
    dot_apply(&mut f, a, e, "dot_one", 0, 5000);
    f.core.units.get_mut(&e).unwrap().stats.current_health = 50;
    launch(&mut f, a, e, 10);
    f.core
        .step_battle_execution_until(&mut f.state, 12)
        .unwrap();
    assert!(f.core.units[&e].lifecycle.is_dead());
    assert_eq!(f.core.units[&a].resonance_current, 7);
    let mut f = kill_fixture();
    let (a, e) = (f.a, f.enemy);
    dot_apply(&mut f, a, e, "dot_one", 0, 5000);
    f.core.units.get_mut(&e).unwrap().stats.current_health = 1;
    f.core
        .step_battle_execution_until(&mut f.state, 1000)
        .unwrap();
    assert!(f.core.units[&e].lifecycle.is_dead());
    assert_eq!(
        f.core.units[&a].resonance_current, 8,
        "7 kill reward + ordinary one-second pulse"
    );
    assert_eq!(
        states(&f)
            .into_iter()
            .find(|v| v.owner == a)
            .unwrap()
            .state
            .events,
        1
    );
}
#[test]
fn composition_dot_kill_two_deaths_cap_and_lost_copy_pending_discard() {
    let mut f = kill_fixture();
    let (a, e, s) = (f.a, f.enemy, f.secondary);
    for target in [e, s] {
        dot_apply(&mut f, a, target, "dot_one", 0, 5000);
        lethal(&mut f.core, a, target, DamageSource::Ability, 10).unwrap();
    }
    settle_kill(&mut f, 10);
    assert_eq!(f.core.units[&a].resonance_current, 14);
    let mut f = kill_fixture();
    let (a, e) = (f.a, f.enemy);
    f.core.units.get_mut(&a).unwrap().resonance_current = 97;
    dot_apply(&mut f, a, e, "dot_one", 0, 5000);
    lethal(&mut f.core, a, e, DamageSource::Ability, 10).unwrap();
    settle_kill(&mut f, 10);
    assert_eq!(f.core.units[&a].resonance_current, 100);
    let mut f = kill_fixture();
    let (a, e) = (f.a, f.enemy);
    dot_apply(&mut f, a, e, "dot_one", 0, 5000);
    lethal(&mut f.core, a, e, DamageSource::Ability, 10).unwrap();
    let view = states(&f).into_iter().find(|v| v.owner == a).unwrap();
    f.core.items.remove(&view.runtime_item_instance_id);
    settle_kill(&mut f, 10);
    assert_eq!(f.core.units[&a].resonance_current, 0);
    assert!(states(&f).is_empty());
}
#[test]
fn composition_dot_kill_context_native_roundtrip_and_negative_reward_rejected() {
    let artifact = crate::authoring::composition::compile_yaml(DOT_KILL).unwrap();
    let loaded = Artifact::from_ron(&artifact.to_ron().unwrap()).unwrap();
    assert_eq!(loaded.equipment_states().len(), 1);
    for (old, new) in [
        (
            "event: OwnerKill",
            "event: { Periodic: { interval_ms: 1000 } }",
        ),
        ("unit: Magnitude", "unit: Milliseconds"),
        ("context: State", "context: Flow"),
    ] {
        let err =
            crate::authoring::composition::compile_yaml(&DOT_KILL.replace(old, new)).unwrap_err();
        assert!(!err.source.definition.is_empty(), "{err:?}");
    }
    let mut f = dot_fixture_source(&DOT_KILL.replace("value: 7", "value: -1"));
    let (a, e) = (f.a, f.enemy);
    dot_apply(&mut f, a, e, "dot_one", 0, 5000);
    let before = f.core.transaction_debug_snapshot();
    assert!(lethal(&mut f.core, a, e, DamageSource::Ability, 10).is_err());
    assert_eq!(f.core.transaction_debug_snapshot(), before);
}

#[test]
fn composition_dot_kill_pending_cleanup_and_hard_cc_do_not_refund_later() {
    for exit in 0..4 {
        let mut f = kill_fixture();
        let (a, e, s) = (f.a, f.enemy, f.secondary);
        dot_apply(&mut f, a, e, "dot_one", 0, 5000);
        lethal(&mut f.core, a, e, DamageSource::Ability, 10).unwrap();
        assert_eq!(states(&f).len(), 1);
        match exit {
            0 => {
                f.core
                    .apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: a,
                        time_ms: 11,
                    })
                    .unwrap();
            }
            1 => {
                f.core.units.get_mut(&a).unwrap().corroded = true;
            }
            2 => {
                lethal(&mut f.core, s, a, DamageSource::Environment, 11).unwrap();
            }
            _ => {
                f.core
                    .runtime_transaction(|c| c.clear_sustained(11))
                    .unwrap();
            }
        }
        settle_kill(&mut f, 11);
        assert!(states(&f).is_empty(), "exit {exit}");
        assert_eq!(f.core.units[&a].resonance_current, 0, "exit {exit}");
    }
    let mut f = kill_fixture();
    let (a, e, b) = (f.a, f.enemy, f.b);
    dot_apply(&mut f, a, e, "dot_one", 0, 5000);
    dot_apply(&mut f, b, a, "kill_stun", 1, 1000);
    lethal(&mut f.core, a, e, DamageSource::BuffTick, 10).unwrap();
    settle_kill(&mut f, 10);
    assert_eq!(f.core.units[&a].resonance_current, 0);
    settle_kill(&mut f, 1002);
    assert_eq!(
        f.core.units[&a].resonance_current, 0,
        "blocked gain is not banked until hard CC expires"
    );
}
#[test]
fn composition_dot_kill_reward_cancel_and_retained_limit_restore_death_and_dedup() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    use crate::game::battle::runtime_control as native;
    let mut f = kill_fixture();
    let (a, e) = (f.a, f.enemy);
    dot_apply(&mut f, a, e, "dot_one", 0, 5000);
    let before = (
        f.core.transaction_debug_snapshot(),
        f.core.event_queue_debug_snapshot(),
    );
    let token = CancellationToken::default();
    let result = control::with_test_cancellation("equipment_reward_queued", 1, || {
        control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialExecution,
            || lethal(&mut f.core, a, e, DamageSource::Ability, 10),
        )
    });
    assert!(result.is_err() && token.is_cancelled());
    assert_eq!(
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot()
        ),
        before
    );
    let mut probe = f.core.clone();
    let (result, admissions) =
        native::observe_retained_admissions(|| lethal(&mut probe, a, e, DamageSource::Ability, 10));
    result.unwrap();
    let required = admissions
        .iter()
        .find(|(site, _, _)| *site == "equipment_reward_storage")
        .unwrap()
        .2;
    f.core.runtime_facts.limits.max_retained_bytes = required - 1;
    let before = (
        f.core.transaction_debug_snapshot(),
        f.core.event_queue_debug_snapshot(),
    );
    assert!(lethal(&mut f.core, a, e, DamageSource::Ability, 10).is_err());
    assert_eq!(
        (
            f.core.transaction_debug_snapshot(),
            f.core.event_queue_debug_snapshot()
        ),
        before
    );
    f.core.runtime_facts.limits = Default::default();
    lethal(&mut f.core, a, e, DamageSource::Ability, 10).unwrap();
    settle_kill(&mut f, 10);
    assert_eq!(f.core.units[&a].resonance_current, 7);
}

#[test]
fn composition_dot_kill_last_enemy_reward_settles_before_terminal_cleanup() {
    let mut f = kill_fixture();
    let (a, e, s) = (f.a, f.enemy, f.secondary);
    Arc::make_mut(&mut f.core.scenario).terminal_outcomes =
        BattleScenario::fixture_required_group_outcomes();
    lethal(&mut f.core, a, s, DamageSource::Ability, 10).unwrap();
    dot_apply(&mut f, a, e, "dot_one", 10, 5000);
    f.core.units.get_mut(&e).unwrap().stats.current_health = 1;
    f.core.enable_authoring_observation();
    f.core
        .step_battle_execution_until(&mut f.state, 1010)
        .unwrap();
    assert!(
        f.state.is_finished(),
        "last real enemy completes the battle"
    );
    assert_eq!(f.core.units[&a].resonance_current, 8);
    f.core.finalize_battle_execution(&mut f.state).unwrap();
    assert!(
        states(&f).is_empty(),
        "terminal cleanup discards the equipment activation"
    );
    let observation = serde_json::to_value(&f.core.authoring_observation).unwrap();
    assert_eq!(
        observation["latest_sourced_resonance"]["change"]["after"],
        8
    );
}
