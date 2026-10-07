// F01: preparation completion does not transfer ownership of unreleased flow work.
fn f01_skills() -> String {
    R10_SKILLS.replace(
        "Repeat(id: \"strikes\", count: 3, interval_ms: 100, body: Action(id: \"strike\", step_id: \"hit\", wait: EffectsSettled))",
        "Sequence(id: \"strikes\", children: [Action(id: \"strike\", step_id: \"hit\", wait: EffectsSettled), Wait(id: \"pause\", duration_ms: 600)])",
    ).replace("Hp(300)", "Hp(100)")
     .replace("target: CastTarget, delivery: Instant", "target: CastTarget, delivery: Projectile(speed_units_per_ms: 25000)")
}

#[test]
fn f01_timeout_and_state_exit_cancel_future_flow_but_preserve_released_delivery() {
    for state_exit in [false, true] {
        for cancel_at in [50, 150, 500] {
            let boss = if state_exit {
                R08_BOSS.replace("tasks: [StartAction", &format!("tasks: [Wait(id: \"exit\", duration_ms: Literal({cancel_at})), StartAction"))
                    .replace("(id: \"on_completed\"", "(id: \"exit_timer\", priority: 200, condition: TaskResult(task_id: \"exit\", result: Completed), target: Outcome(\"canceled\")), (id: \"on_completed\"")
            } else {
                R08_BOSS.replace("timeout_ms: 1000", &format!("timeout_ms: {cancel_at}"))
            };
            let mut core = r08_core(&boss, &f01_skills());
            let mut execution = core.start_battle_execution().unwrap();
            core.step_battle_execution_until(&mut execution, 0).unwrap();
            let actor = actor_runtime(&core).unit_id.unwrap();
            let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
            let hp = core.units[&target].stats.current_health;
            core.units.get_mut(&actor).unwrap().stats.current_health = 500;
            core.step_battle_execution_until(&mut execution, cancel_at)
                .unwrap();
            assert_eq!(
                r08_node(&core),
                if state_exit { "canceled" } else { "timedout" }
            );
            assert!(core.active_skill_casts.is_empty());
            assert!(!core.event_queue.iter().any(|event| matches!(
                event.as_ref(),
                BattleEvent::SkillFlowWake { .. } | BattleEvent::SkillStep { .. }
            )));
            assert_eq!(core.runtime_fact_usage().public_result_reservations, 0);
            // Even a previously extracted control event cannot recreate the owner.
            core.event_queue.push(BattleEvent::SkillFlowWake {
                time_ms: cancel_at + 1,
                cast_seq: 0,
                generation: u64::MAX,
            });
            core.step_battle_execution_until(&mut execution, 1000)
                .unwrap();
            assert_eq!(
                core.units[&target].stats.current_health,
                hp - if cancel_at > 100 { 100 } else { 0 }
            );
            assert_eq!(core.units[&actor].stats.current_health, 500);
            assert!(core.active_projectiles.is_empty());
        }
    }
}

#[test]
fn f01_detach_preserves_future_flow_and_delivery() {
    let boss = R08_BOSS
        .replace("timeout_ms: 1000", "timeout_ms: 150")
        .replace("on_timeout: CancelUnreleased", "on_timeout: Detach");
    let mut core = r08_core(&boss, &f01_skills());
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    let actor = actor_runtime(&core).unit_id.unwrap();
    let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
    let hp = core.units[&target].stats.current_health;
    core.units.get_mut(&actor).unwrap().stats.current_health = 500;
    core.step_battle_execution_until(&mut execution, 150)
        .unwrap();
    assert_eq!(r08_node(&core), "timedout");
    assert!(!core.active_skill_casts.is_empty());
    core.step_battle_execution_until(&mut execution, 1000)
        .unwrap();
    assert_eq!(core.units[&target].stats.current_health, hp - 100);
    assert_eq!(core.units[&actor].stats.current_health, 515);
    assert!(core.active_skill_casts.is_empty() && core.active_projectiles.is_empty());
}

#[test]
fn f01_cancellation_faults_restore_boundary_and_retry_exactly() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    for (at, site) in [
        (50, "boss_action_pending_cancelled"),
        (150, "boss_action_cancel_scan"),
        (150, "boss_action_continuation_removed"),
        (150, "boss_action_queue_scan"),
        (150, "boss_action_queue_pruned"),
        (150, "boss_action_cancel_committed"),
    ] {
        let boss = R08_BOSS.replace("timeout_ms: 1000", &format!("timeout_ms: {at}"));
        let mut core = r08_core(&boss, &f01_skills());
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, at - 1)
            .unwrap();
        let mut clean = core.clone();
        let mut clean_execution = execution.clone();
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{execution:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.step_battle_execution_until(&mut execution, at),
            )
        });
        assert!(
            token.is_cancelled() && result.is_err(),
            "{site}: {result:?}"
        );
        assert_eq!(
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                format!("{execution:?}")
            ),
            before,
            "{site}"
        );
        core.step_battle_execution_until(&mut execution, 1000)
            .unwrap();
        clean
            .step_battle_execution_until(&mut clean_execution, 1000)
            .unwrap();
        assert_eq!(
            core.transaction_debug_snapshot(),
            clean.transaction_debug_snapshot(),
            "{site}"
        );
        assert_eq!(
            core.event_queue_debug_snapshot(),
            clean.event_queue_debug_snapshot(),
            "{site}"
        );
    }
}

#[test]
fn f01_exact_handle_isolation_preserves_other_cast_and_actor_work() {
    let mut core = r08_core(R08_BOSS, &f01_skills());
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 150)
        .unwrap();
    let actor = actor_runtime(&core).unit_id.unwrap();
    let (seq, cast) = core.active_skill_casts.iter().next().unwrap();
    let seq = *seq;
    let action = cast.fact_source.action.clone().unwrap();
    let mut unrelated = cast.clone();
    // Deliberately share the ordinal, but not the attempt/occurrence capability.
    let mut other = r08_core(R08_BOSS, &f01_skills());
    let mut other_execution = other.start_battle_execution().unwrap();
    other
        .step_battle_execution_until(&mut other_execution, 150)
        .unwrap();
    let foreign = other
        .active_skill_casts
        .values()
        .next()
        .unwrap()
        .fact_source
        .action
        .clone()
        .unwrap();
    assert_eq!(action.ordinal(), foreign.ordinal());
    assert_ne!(action, foreign);
    unrelated.fact_source.action = Some(foreign);
    core.active_skill_casts.insert(10000, unrelated);
    core.event_queue.push(BattleEvent::SkillFlowWake {
        time_ms: 1000,
        cast_seq: 10000,
        generation: 0,
    });
    let unit_before = format!("{:?}", core.units[&actor]);
    let carriers_before = format!("{:?}", core.active_projectiles);
    core.cancel_boss_action(150, actor, &action).unwrap();
    core.cancel_boss_action(150, actor, &action).unwrap();
    assert!(!core.active_skill_casts.contains_key(&seq));
    assert!(core.active_skill_casts.contains_key(&10000));
    assert!(core.event_queue.iter().any(|event| matches!(
        event.as_ref(),
        BattleEvent::SkillFlowWake {
            cast_seq: 10000,
            ..
        }
    )));
    assert_eq!(format!("{:?}", core.units[&actor]), unit_before);
    assert_eq!(format!("{:?}", core.active_projectiles), carriers_before);
}

#[test]
fn f01_released_area_keeps_ticks_and_settles_after_control_cancellation() {
    let boss = R08_BOSS.replace("timeout_ms: 1000", "timeout_ms: 150");
    let skills = f01_skills().replace("Projectile(speed_units_per_ms: 25000)",
        "TileArea(footprint: (include_anchor_tile: true, rows: [\"@\"]), area: (anchor: CastTarget, tile_origin: Anchor, duration_ms: 300, tick_interval_ms: Some(100)))");
    let mut core = r08_core(&boss, &skills);
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    let actor = actor_runtime(&core).unit_id.unwrap();
    let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
    let action = core.units[&actor]
        .pending_skill_cast
        .as_ref()
        .unwrap()
        .action
        .clone()
        .unwrap();
    let hp = core.units[&target].stats.current_health;
    core.units.get_mut(&actor).unwrap().stats.current_health = 500;
    core.step_battle_execution_until(&mut execution, 150)
        .unwrap();
    assert!(core.active_skill_casts.is_empty());
    assert_eq!(core.active_areas.len(), 1);
    assert_eq!(core.units[&target].stats.current_health, hp - 100);
    let snapshot = core.query_action(&action).unwrap();
    assert!(matches!(
        snapshot.terminal.unwrap().kind,
        ActionTerminalKind::Cancelled(_)
    ));
    assert!(snapshot.outstanding_deliveries > 0);
    core.step_battle_execution_until(&mut execution, 1000)
        .unwrap();
    assert_eq!(core.units[&target].stats.current_health, hp - 300);
    assert_eq!(core.units[&actor].stats.current_health, 500);
    assert!(core.active_areas.is_empty());
    assert_eq!(core.runtime_fact_usage().outstanding_deliveries, 0);
}

#[test]
fn f01_same_timestamp_timeout_matches_partitioned_execution() {
    // The authored timeout and preparation completion share timestamp 100.
    let boss = R08_BOSS.replace("timeout_ms: 1000", "timeout_ms: 100");
    let mut one = r08_core(&boss, &f01_skills());
    let mut one_execution = one.start_battle_execution().unwrap();
    one.step_battle_execution_until(&mut one_execution, 0)
        .unwrap();
    let mut split = one.clone();
    let mut split_execution = one_execution.clone();
    one.step_battle_execution_until(&mut one_execution, 1000)
        .unwrap();
    for time in [0, 99, 100, 101, 1000] {
        split
            .step_battle_execution_until(&mut split_execution, time)
            .unwrap();
    }
    assert_eq!(r08_node(&one), "timedout");
    assert!(one.active_skill_casts.is_empty());
    assert_eq!(
        one.transaction_debug_snapshot(),
        split.transaction_debug_snapshot()
    );
    assert_eq!(
        one.event_queue_debug_snapshot(),
        split.event_queue_debug_snapshot()
    );
}

#[test]
fn f01_legacy_delayed_leaf_and_extracted_stale_step_cannot_release_after_cancel() {
    let boss = R08_BOSS.replace("timeout_ms: 1000", "timeout_ms: 150");
    let mut core = r08_core(&boss, R08_SKILLS);
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 100)
        .unwrap();
    let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
    let hp = core.units[&target].stats.current_health;
    let stale = core
        .event_queue
        .iter()
        .find(|event| matches!(event.as_ref(), BattleEvent::SkillStep { .. }))
        .unwrap()
        .into_owned();
    core.step_battle_execution_until(&mut execution, 150)
        .unwrap();
    assert!(core.active_skill_casts.is_empty());
    assert!(!core
        .event_queue
        .iter()
        .any(|event| matches!(event.as_ref(), BattleEvent::SkillStep { .. })));
    core.event_queue.push(stale);
    core.step_battle_execution_until(&mut execution, 1000)
        .unwrap();
    assert_eq!(core.units[&target].stats.current_health, hp);
    assert_eq!(r08_node(&core), "timedout");
}
