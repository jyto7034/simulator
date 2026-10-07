// Public authored input -> compiled catalog -> ordinary battle execution.
const R08_BOSS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r08/boss.ron"));
const R08_SKILLS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r08/skills.ron"));

fn r08_core(boss: &str, skills: &str) -> BattleCore {
    r08_configured_core(boss, skills, |_| {})
}
fn r08_configured_core(boss: &str, skills: &str, configure: impl FnOnce(&mut BattleScenario)) -> BattleCore {
    let catalog: crate::game::data::boss_runtime_data::BossRuntimeCatalogData = ron::de::from_str(boss).expect("public boss RON");
    let database = catalog.compile().expect("compiled boss graph");
    let skills = SkillDatabase::from_ron("r08/skills.ron", skills).expect("public skill RON");
    database.get_summoned_unit_skill_validation(&skills).expect("cross-database skill reference");
    let mut setup = ResolvedBossEncounterSetup {
        definition: database.get_encounter(&crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new("r08_encounter")).unwrap().clone(),
        actors: vec![ResolvedBossActorSetup { actor_slot_index: 0, unit_ref: ScenarioUnitRef::new("boss_0") }],
        enemy_stat_scale: Default::default(), summon_spawn_zones: vec![], summon_routes: vec![],
    };
    let two = setup.definition.actor_slots.len() == 2;
    if two { setup.actors.push(ResolvedBossActorSetup { actor_slot_index: 1, unit_ref: ScenarioUnitRef::new("boss_1") }); }
    let mut scenario = fixture_scenario_with_setup(Some(setup));
    if two {
        let mut other = scenario.groups[0].spawns[0].clone();
        other.unit_ref = ScenarioUnitRef::new("boss_1"); other.instance_salt = 1;
        other.draft = fixture_draft(4, 1000, 0); other.position = Position::new(6, 2);
        scenario.groups[0].spawns.push(other);
    }
    configure(&mut scenario);
    BattleCore::new_from_scenario(scenario, GameDataBuilder::empty().with_skills(skills).build_arc(), 17)
}
fn r08_node(core: &BattleCore) -> String {
    { let view = &core.boss_actor_runtime_debug_snapshots()[0]; view.state_id.clone().or_else(|| view.control_node_id.clone()).unwrap() }
}
#[test]
fn r08_public_ron_release_and_settlement_have_distinct_waiters() {
    for (milestone, completion) in [("Released", 100), ("EffectsSettled", 300)] {
        let mut core = r08_core(&R08_BOSS.replace("EffectsSettled", milestone), R08_SKILLS);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        assert_eq!(r08_node(&core), "cast");
        let actor = actor_runtime(&core).unit_id.unwrap();
        let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        let action = core.units[&actor].pending_skill_cast.as_ref().unwrap().action.clone().unwrap();
        assert_eq!(core.query_action(&action).unwrap().accepted_at_ms, 0);
        assert_eq!(core.units[&actor].owner, Side::Opponent);
        assert_eq!(core.units[&actor].facing_direction, Some(FacingDirection::Left));
        let origin = core.units[&actor].body.position;
        let hp = core.units[&target].stats.current_health;
        core.step_battle_execution_until(&mut execution, completion - 1).unwrap();
        assert_eq!(r08_node(&core), "cast");
        assert_eq!(core.units[&target].stats.current_health, hp);
        core.step_battle_execution_until(&mut execution, completion).unwrap();
        assert_eq!(r08_node(&core), "completed");
        assert_eq!(core.runtime_fact_usage().public_result_reservations, 0);
        core.step_battle_execution_until(&mut execution, 300).unwrap();
        assert_eq!(core.units[&target].stats.current_health, hp - 100);
        assert_eq!(core.units[&actor].body.position, origin);
        assert_eq!(core.runtime_fact_summary().damage_hp_lost, 100);
    }
}
#[test]
fn r08_public_timeout_cancels_unreleased_or_detaches_explicitly() {
    for (policy, expected_loss) in [("CancelUnreleased", 0), ("Detach", 100)] {
        let source = R08_BOSS.replace("timeout_ms: 1000", "timeout_ms: 50").replace("on_timeout: CancelUnreleased", &format!("on_timeout: {policy}"));
        let mut core = r08_core(&source, R08_SKILLS);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        let actor = actor_runtime(&core).unit_id.unwrap();
        let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        let hp = core.units[&target].stats.current_health;
        core.step_battle_execution_until(&mut execution, 50).unwrap();
        assert_eq!(r08_node(&core), "timedout");
        assert_eq!(core.runtime_fact_usage().public_result_reservations, 0);
        core.step_battle_execution_until(&mut execution, 1000).unwrap();
        assert_eq!(r08_node(&core), "timedout", "old deadline cannot alter the next activation");
        assert_eq!(core.units[&target].stats.current_health, hp - expected_loss);
        assert!(core.units[&actor].pending_skill_cast.is_none());
    }
}
#[test]
fn r08_public_invalid_catalogs_are_rejected_before_runtime() {
    use crate::game::data::boss_runtime_data::BossRuntimeCatalogData;
    for source in [R08_BOSS.replace("timeout_ms: 1000", "timeout_ms: 0"), R08_BOSS.replace("result: Canceled", "result: Completed"), R08_BOSS.replace("tasks: [StartAction", "tasks: [RouteCombat(id: \"conflict\"), StartAction")] {
        let catalog: BossRuntimeCatalogData = ron::de::from_str(&source).unwrap();
        assert!(catalog.compile().is_err());
    }
    let skills = SkillDatabase::from_ron("r08/skills.ron", R08_SKILLS).unwrap();
    for source in [R08_BOSS.replace("r08_strike", "missing_skill"), R08_BOSS.replace("target: LockedUnit", "target: SelfUnit")] {
        let catalog: BossRuntimeCatalogData = ron::de::from_str(&source).unwrap();
        assert!(catalog.compile().unwrap().get_summoned_unit_skill_validation(&skills).is_err());
    }
}

#[test]
fn r08_public_state_exit_preserves_only_released_projectile() {
    for (exit_ms, damage) in [(50, 0), (150, 100)] {
        let boss = R08_BOSS.replace("tasks: [StartAction", &format!("tasks: [Wait(id: \"exit\", duration_ms: Literal({exit_ms})), StartAction"))
            .replace("(id: \"on_completed\"", "(id: \"exit_timer\", priority: 200, condition: TaskResult(task_id: \"exit\", result: Completed), target: Outcome(\"canceled\")), (id: \"on_completed\"");
        let skills = R08_SKILLS.replace("delay_ms: 200", "delay_ms: 0").replace("delivery: Instant", "delivery: Projectile(speed_units_per_ms: 25000)");
        let mut core = r08_core(&boss, &skills);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        let hp = core.units[&target].stats.current_health;
        core.step_battle_execution_until(&mut execution, exit_ms).unwrap();
        assert_eq!(r08_node(&core), "canceled");
        assert_eq!(core.runtime_fact_usage().public_result_reservations, 0);
        assert_eq!(core.active_projectiles.is_empty(), exit_ms == 50);
        core.step_battle_execution_until(&mut execution, 1000).unwrap();
        assert_eq!(core.units[&target].stats.current_health, hp - damage);
        assert!(core.active_projectiles.is_empty());
    }
}
#[test]
fn r08_native_fault_rolls_back_boss_action_and_retry_matches_clean_start() {
    use crate::game::battle::runtime_control::RuntimeFactLimits;
    let mut clean = r08_core(R08_BOSS, R08_SKILLS);
    let mut clean_execution = clean.start_battle_execution().unwrap();
    clean.step_battle_execution_until(&mut clean_execution, 0).unwrap();
    for max_work in [1, 20, 100] {
        let mut limited = r08_core(R08_BOSS, R08_SKILLS).with_runtime_fact_limits(RuntimeFactLimits { max_work, ..Default::default() }).unwrap();
        let before = limited.transaction_debug_snapshot();
        let result = limited.start_battle_execution();
        assert!(result.is_err());
        assert_eq!(limited.transaction_debug_snapshot(), before);
        // A new operation budget does not preserve a failed candidate's sequence/RNG/action.
        let mut retry = limited.with_runtime_fact_limits(RuntimeFactLimits::default()).unwrap();
        let mut execution = retry.start_battle_execution().unwrap();
        retry.step_battle_execution_until(&mut execution, 0).unwrap();
        let units = |core: &BattleCore| core.units.iter().map(|(id, unit)| (*id, format!("{unit:?}"))).collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(units(&retry), units(&clean));
        assert_eq!(retry.boss_actor_runtime_debug_snapshots(), clean.boss_actor_runtime_debug_snapshots());
        assert_eq!(serde_json::to_value(retry.runtime_fact_summary()).unwrap(), serde_json::to_value(clean.runtime_fact_summary()).unwrap());
        assert_eq!(serde_json::to_value(&retry.event_log.entries).unwrap(), serde_json::to_value(&clean.event_log.entries).unwrap());
    }
}

#[test]
fn r08_second_action_admission_fault_rolls_back_entire_boss_boundary() {
    use crate::game::battle::runtime_control::{RuntimeFactLimits, RuntimeFaultCode};
    let boss = R08_BOSS.replace("actor_slots: [\"primary\"]", "actor_slots: [\"primary\", \"secondary\"]")
        .replace("actor_behaviors: [(actor_slot_id: \"primary\", behavior_id: \"r08_behavior\")]", "actor_behaviors: [(actor_slot_id: \"primary\", behavior_id: \"r08_behavior\"), (actor_slot_id: \"secondary\", behavior_id: \"r08_behavior\")]");
    let mut core = r08_core(&boss, R08_SKILLS).with_runtime_fact_limits(RuntimeFactLimits { max_retained_results: 1, ..Default::default() }).unwrap();
    let mut execution = core.start_battle_execution().unwrap();
    let before = core.transaction_debug_snapshot();
    let error = core.step_battle_execution_until(&mut execution, 0).unwrap_err();
    assert!(matches!(error, GameError::BattleRuntimeFault(f) if f.code == RuntimeFaultCode::RetainedResultLimit));
    assert_eq!(core.transaction_debug_snapshot(), before);
    assert_eq!(core.runtime_fact_usage().accepted_actions, 0);
    let mut retry = core.with_runtime_fact_limits(RuntimeFactLimits::default()).unwrap();
    retry.step_battle_execution_until(&mut execution, 0).unwrap();
    assert_eq!(retry.runtime_fact_usage().public_result_reservations, 2);
    assert_eq!(retry.boss_actor_runtime_debug_snapshots().len(), 2);
    retry.step_battle_execution_until(&mut execution, 300).unwrap();
    assert_eq!(retry.runtime_fact_summary().damage_hp_lost, 200);
    assert_eq!(retry.runtime_fact_usage().public_result_reservations, 0);
}
#[test]
fn r08_repeated_state_reclaims_handles_and_ignores_old_deadlines() {
    use crate::game::battle::runtime_control::RuntimeFactLimits;
    let boss = R08_BOSS.replace("target: Outcome(\"completed\")", "target: State(\"cast\"), loop_guard: Some((id: \"repeat\", max_iterations: 4, exhausted_target: Outcome(\"completed\")))");
    let mut core = r08_core(&boss, R08_SKILLS).with_runtime_fact_limits(RuntimeFactLimits { max_retained_results: 1, ..Default::default() }).unwrap();
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 3000).unwrap();
    assert_eq!(r08_node(&core), "completed");
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 500);
    assert_eq!(core.runtime_fact_usage().public_result_reservations, 0);
    assert_eq!(core.runtime_fact_usage().completed_results, 0);
}
#[test]
fn r08_public_rejection_and_target_loss_follow_distinct_routes() {
    let skills = R08_SKILLS.replace("reach: global", r#"reach: pattern, target_range: Some((include_anchor_tile: false, rows: [".X.", ".@.", "..."]))"#);
    let mut rejected = r08_core(R08_BOSS, &skills);
    let mut execution = rejected.start_battle_execution().unwrap();
    rejected.step_battle_execution_until(&mut execution, 0).unwrap();
    assert_eq!(r08_node(&rejected), "requestrejected");
    assert_eq!(rejected.runtime_fact_usage().accepted_actions, 0);
    assert_eq!(rejected.runtime_fact_usage().public_result_reservations, 0);

    let mut core = r08_core(R08_BOSS, R08_SKILLS);
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 50).unwrap();
    let actor = actor_runtime(&core).unit_id.unwrap();
    let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
    core.apply_live_command(super::super::sim::BattleLiveCommand::WithdrawUnit { unit_id: target, time_ms: 50 }).unwrap();
    core.step_battle_execution_until(&mut execution, 100).unwrap();
    assert_eq!(r08_node(&core), "targetinvalidated");
    assert!(core.units[&actor].pending_skill_cast.is_none());
    assert_eq!(core.runtime_fact_usage().public_result_reservations, 0);
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0);
}
const R10_SKILLS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r10/skills.ron"));
#[test]
fn r10_public_repeat_result_branch_and_interval_are_finite() {
    for interval in [0, 100, 200] {
        let skills = R10_SKILLS.replace("interval_ms: 100", &format!("interval_ms: {interval}"));
        let mut core = r08_core(R08_BOSS, &skills);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        let actor = actor_runtime(&core).unit_id.unwrap();
        let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        core.units.get_mut(&actor).unwrap().stats.current_health = 500;
        let hp = core.units[&target].stats.current_health;
        let finish = 100 + interval * 2;
        if interval > 0 {
            core.step_battle_execution_until(&mut execution, finish - 1).unwrap();
            assert_eq!(core.units[&target].stats.current_health, hp - 200);
            assert_eq!(core.units[&actor].stats.current_health, 500);
            assert_eq!(r08_node(&core), "cast");
        }
        core.step_battle_execution_until(&mut execution, finish).unwrap();
        assert_eq!(core.units[&target].stats.current_health, hp - 300);
        assert_eq!(core.units[&actor].stats.current_health, 515);
        assert_eq!(r08_node(&core), "completed");
        assert!(core.active_skill_casts.is_empty());
        assert_eq!(core.runtime_fact_summary().damage_hp_lost, 300);
        assert_eq!(core.runtime_fact_summary().healing_delta, 15);
        let steps: Vec<_> = core.event_log.entries.iter().filter_map(|entry| match &entry.event {
            BattleLogEvent::AbilityStepTriggered { step_id, .. } => Some(step_id.as_str()), _ => None,
        }).collect();
        assert_eq!(steps, ["hit", "hit", "hit", "heal"]);
    }
}
#[test]
fn r10_public_branch_uses_real_hp_loss_and_never_executes_unselected_leaf() {
    let skills = R10_SKILLS.replace("Damage(amount: 100", "Damage(amount: 0");
    let mut core = r08_core(R08_BOSS, &skills);
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    let actor = actor_runtime(&core).unit_id.unwrap();
    core.units.get_mut(&actor).unwrap().stats.current_health = 500;
    core.step_battle_execution_until(&mut execution, 300).unwrap();
    assert_eq!(r08_node(&core), "completed");
    assert_eq!(core.units[&actor].stats.current_health, 500);
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0);
    assert!(core.active_skill_casts.is_empty());
}
#[test]
fn r10_public_flow_rejects_bad_slots_units_bounds_and_overflow() {
    for (old, new) in [
        ("node_id: \"strikes\"", "node_id: \"recovery\""),
        ("node_id: \"strikes\"", "node_id: \"strike\""),
        ("value: Hp(300)", "value: Count(300)"),
        ("count: 3", "count: 0"),
        ("count: 3", "count: 1000001"),
        ("interval_ms: 100", "interval_ms: 18446744073709551615"),
        ("step_id: \"hit\"", "step_id: \"missing\""),
        ("id: \"miss\"", "id: \"strike\""),
        ("wait: EffectsSettled", "wait: Released"),
    ] {
        let source = R10_SKILLS.replace(old, new);
        assert!(SkillDatabase::from_ron("r10/invalid.ron", &source).is_err(), "{old} => {new}");
    }
}

#[test]
fn r10_public_wait_parallel_join_and_released_delivery_cancellation() {
    let skills = R10_SKILLS.replace("Repeat(id: \"strikes\", count: 3, interval_ms: 100, body: Action(id: \"strike\", step_id: \"hit\", wait: EffectsSettled))", "Parallel(id: \"strikes\", branches: [Action(id: \"strike\", step_id: \"hit\", wait: EffectsSettled), Wait(id: \"timer\", duration_ms: 400)])")
        .replace("Hp(300)", "Hp(100)").replace("target: CastTarget, delivery: Instant", "target: CastTarget, delivery: Projectile(speed_units_per_ms: 25000)");
    for cancel in [false, true] {
        let mut core = r08_core(R08_BOSS, &skills);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        let actor = actor_runtime(&core).unit_id.unwrap();
        let target = actor_runtime(&core).target_lock.as_ref().unwrap().target_id;
        core.units.get_mut(&actor).unwrap().stats.current_health = 500;
        let hp = core.units[&target].stats.current_health;
        let handle = core.units[&actor].pending_skill_cast.as_ref().unwrap().action.clone().unwrap();
        core.step_battle_execution_until(&mut execution, 150).unwrap();
        assert!(!core.active_projectiles.is_empty());
        if cancel {
            core.process_commands(vec![crate::game::battle::damage::BattleCommand::InterruptCast { source_id: target, target_id: actor }], 150);
        }
        // A stale activation cannot release another leaf or advance the timer.
        core.event_queue.push(BattleEvent::SkillFlowWake { time_ms: 151, cast_seq: 0, generation: u64::MAX });
        core.step_battle_execution_until(&mut execution, 499).unwrap();
        assert_eq!(core.units[&target].stats.current_health, hp - 100);
        assert_eq!(core.units[&actor].stats.current_health, 500);
        if !cancel { assert!(core.query_action(&handle).unwrap().terminal.is_none()); }
        core.step_battle_execution_until(&mut execution, 500).unwrap();
        assert_eq!(core.units[&actor].stats.current_health, if cancel { 500 } else { 515 });
        assert_eq!(r08_node(&core), if cancel { "canceled" } else { "completed" });
        assert!(core.active_skill_casts.is_empty() && core.active_projectiles.is_empty());
    }
}

#[test]
fn r10_public_flow_faults_restore_timestamp_and_retry_exactly() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["flow_state_initialize", "flow_occurrence", "flow_repeat_reset", "flow_continuation_committed"] {
        let mut core = r08_core(R08_BOSS, R10_SKILLS);
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 99).unwrap();
        let mut clean = core.clone(); let mut clean_execution = execution.clone();
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}"));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.step_battle_execution_until(&mut execution, 100)));
        assert!(token.is_cancelled() && result.is_err(), "{site}: {result:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}")), before, "{site}");
        core.step_battle_execution_until(&mut execution, 300).unwrap();
        clean.step_battle_execution_until(&mut clean_execution, 300).unwrap();
        assert_eq!(serde_json::to_value(core.runtime_fact_summary()).unwrap(), serde_json::to_value(clean.runtime_fact_summary()).unwrap());
        assert_eq!(core.event_queue_debug_snapshot(), clean.event_queue_debug_snapshot());
    }
}

#[test]
fn r10_public_repeat_spends_empty_opportunity_without_catchup() {
    let boss = R08_BOSS.replace("timeout_ms: 1000", "timeout_ms: 50").replace("on_timeout: CancelUnreleased", "on_timeout: Detach");
    let skills = R10_SKILLS.replace("target: CastTarget, delivery:", "target: EnemySingle(rule: Nearest), targeting: RetargetOnStep, delivery:");
    let mut core = r08_configured_core(&boss, &skills, |scenario| {
        let mut later = scenario.groups[1].clone();
        later.id = ScenarioGroupId::new("later_player"); later.spawns.truncate(1);
        later.spawns[0].draft = fixture_draft(4, 1000, 0);
        later.spawns[0].unit_ref = ScenarioUnitRef::new("later_player");
        scenario.events.push(ScenarioEvent { id: ScenarioEventId::new("later_spawn"), trigger: ScenarioTrigger::AtTimeMs(250), action: ScenarioAction::SpawnGroup { group_id: later.id.clone() } });
        scenario.groups.push(later);
    });
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 100).unwrap();
    let actor = actor_runtime(&core).unit_id.unwrap();
    core.units.get_mut(&actor).unwrap().stats.current_health = 500;
    let players: Vec<_> = core.units.values().filter(|unit| unit.owner == Side::Player).map(|unit| unit.instance_id).collect();
    for unit_id in players { core.apply_live_command(super::super::sim::BattleLiveCommand::WithdrawUnit { unit_id, time_ms: 150 }).unwrap(); }
    core.step_battle_execution_until(&mut execution, 200).unwrap();
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 100);
    core.step_battle_execution_until(&mut execution, 300).unwrap();
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 200);
    assert_eq!(core.units[&actor].stats.current_health, 500);
    assert!(core.active_skill_casts.is_empty());
    let later = core.units.values().find(|unit| unit.owner == Side::Player && unit.is_active()).unwrap();
    assert_eq!(later.stats.current_health, 900);
}

#[test]
fn r10_public_area_three_targets_then_one_self_heal() {
    let skills = R10_SKILLS.replace("count: 3", "count: 1").replace("target: CastTarget, delivery: Instant", "target: CastTarget, delivery: TileArea(footprint: (include_anchor_tile: true, rows: [\"XXX\", \"X@X\", \"XXX\"]), area: (anchor: CastTarget, tile_origin: Anchor))");
    let mut core = r08_configured_core(R08_BOSS, &skills, |scenario| {
        let mut third = scenario.groups[1].spawns[0].clone();
        third.draft = fixture_draft(4, 1000, 0); third.unit_ref = ScenarioUnitRef::new("third");
        third.position = Position::new(2, 1); third.instance_salt = 2;
        scenario.groups[1].spawns.push(third);
    });
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    let actor = actor_runtime(&core).unit_id.unwrap();
    core.units.get_mut(&actor).unwrap().stats.current_health = 500;
    core.step_battle_execution_until(&mut execution, 100).unwrap();
    assert_eq!(core.units[&actor].stats.current_health, 515);
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 300);
    assert_eq!(core.runtime_fact_summary().healing_delta, 15);
    assert_eq!(core.units.values().filter(|unit| unit.owner == Side::Player && unit.stats.current_health == 900).count(), 3);
    let fact = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
    assert_eq!(fact.source.authored.as_ref().unwrap().site.action.action_id, "hit");
    assert_eq!(fact.source.flow.as_ref().unwrap().occurrence, 0);
    let heal = core.runtime_fact_summary().latest_heal.as_ref().unwrap();
    assert_eq!(heal.source.authored.as_ref().unwrap().site.action.action_id, "heal");
    assert_eq!(heal.source.flow.as_ref().unwrap().occurrence, 1);
}

#[test]
fn r10_large_repeat_and_control_admission_faults_preserve_state() {
    use crate::game::battle::runtime_control::RuntimeFactLimits;
    for limits in [RuntimeFactLimits { max_work: 50_000, ..Default::default() }, RuntimeFactLimits { max_control_steps: 1, ..Default::default() }] {
        let skills = R10_SKILLS.replace("count: 3", "count: 1000000").replace("interval_ms: 100", "interval_ms: 0").replace("Damage(amount: 100", "Damage(amount: 0");
        let mut core = r08_core(R08_BOSS, &skills).with_runtime_fact_limits(limits).unwrap();
        let mut execution = core.start_battle_execution().unwrap();
        core.step_battle_execution_until(&mut execution, 99).unwrap();
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}"));
        let error = core.step_battle_execution_until(&mut execution, 100).expect_err("large finite flow must respect native budgets");
        assert!(matches!(error, crate::game::behavior::GameError::BattleRuntimeFault(_)), "{error:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}")), before);
    }
    let mut source = "Action(id: \"leaf\", step_id: \"hit\", wait: EffectsSettled)".to_owned();
    for depth in 0..65 { source = format!("Sequence(id: \"n{depth}\", children: [{source}])"); }
    let skills = R08_SKILLS.replace(" steps:", &format!(" flow: Some({source}), steps:"));
    assert!(SkillDatabase::from_ron("r10/deep.ron", &skills).is_err());
}

fn r11_modifier_skills() -> String {
    R08_SKILLS.replace("use_lifetime: Action", r#"use_lifetime: ActionAndEffects(effects: [(step: "hit", effect: "A"), (step: "hit", effect: "B")])"#)
        .replace("target: CastTarget, delivery: Instant", "target: SelfUnit, delivery: Instant")
        .replace("(id: \"damage\", op: Damage(amount: 100, damage_type: True))", r#"
            (id: "A", op: Sustained(definition: Modifiers(duration_ms: 100, modifiers: [(stat: Attack, kind: Flat, value: 50), (stat: Attack, kind: Percent, value: 20)]))),
            (id: "B", op: Sustained(definition: Modifiers(duration_ms: 200, modifiers: [(stat: Attack, kind: Percent, value: 30)])))
        "#)
}
#[test]
fn r11_public_modifiers_compose_expire_and_preserve_permanent_base() {
    let mut core = r08_core(R08_BOSS, &r11_modifier_skills());
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    let actor = actor_runtime(&core).unit_id.unwrap();
    core.units.get_mut(&actor).unwrap().stats.attack = 100;
    core.step_battle_execution_until(&mut execution, 300).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 225);
    assert_eq!(core.sustained.len(), 2);
    assert_eq!(r08_node(&core), "cast", "finite effect retains action settlement");
    core.process_commands_with_modifier_source(vec![crate::game::battle::damage::BattleCommand::ApplyModifier {
        target_id: actor, modifier: crate::game::stats::StatModifier { stat: crate::game::stats::StatId::Attack, kind: crate::game::stats::StatModifierKind::Flat, value: 10 },
    }], 350, Some(actor));
    assert_eq!(core.units[&actor].stats.attack, 240);
    core.step_battle_execution_until(&mut execution, 399).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 240);
    core.step_battle_execution_until(&mut execution, 400).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 143);
    core.step_battle_execution_until(&mut execution, 500).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 110);
    assert_eq!(core.sustained.entity_count(), 0);
    assert_eq!(r08_node(&core), "completed");
}
#[test]
fn r11_public_modifier_repeated_lifetimes_do_not_drift() {
    let boss = R08_BOSS.replace("target: Outcome(\"completed\")", "target: State(\"cast\"), loop_guard: Some((id: \"repeat\", max_iterations: 9, exhausted_target: Outcome(\"completed\")))");
    let mut core = r08_core(&boss, &r11_modifier_skills());
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    let actor = actor_runtime(&core).unit_id.unwrap();
    core.units.get_mut(&actor).unwrap().stats.attack = 101;
    core.step_battle_execution_until(&mut execution, 6000).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 101);
    assert_eq!(core.sustained.entity_count(), 0);
    assert_eq!(r08_node(&core), "completed");
}
#[test]
fn r11_public_modifier_faults_restore_source_stats_queue_and_retry() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["sustained_record_storage", "sustained_started", "sustained_ended"] {
        let mut core = r08_core(R08_BOSS, &r11_modifier_skills());
        let mut execution = core.start_battle_execution().unwrap();
        let (before_time, fault_time) = if site == "sustained_ended" { (399, 400) } else { (299, 300) };
        core.step_battle_execution_until(&mut execution, before_time).unwrap();
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}"));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.step_battle_execution_until(&mut execution, fault_time)));
        assert!(token.is_cancelled() && result.is_err(), "{site}: {result:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}")), before);
        core.step_battle_execution_until(&mut execution, 500).unwrap();
        assert_eq!(core.sustained.entity_count(), 0);
        assert_eq!(r08_node(&core), "completed");
    }
}

const R11_AURA: &str = r#"SkillDatabase(skills: [SkillDef(
 id: "r11_aura", name: "Finite employee channel", kind: Targeted, focus_time_ms: 0,
 use_lifetime: ActionAndEffects(effects: [(step: "aura", effect: "channel")]),
 cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: true),
 steps: [SkillStepDef(id: "aura", delay_ms: 0, target: SelfUnit, delivery: Instant,
 effects: [(id: "channel", op: Sustained(definition: ChannelAura(duration_ms: 1000, aura: (family: "fetus", priority: 0, radius_units: 2000000, basic_damage_percent: 20, basic_speed_points: 15))))])]
)])"#;
fn r11_aura_core() -> (BattleCore, super::super::sim::BattleExecutionState, UnitInstanceId, UnitInstanceId) {
    r11_employee_effect_core(R11_AURA)
}
fn r11_employee_effect_core(authored: &str) -> (BattleCore, super::super::sim::BattleExecutionState, UnitInstanceId, UnitInstanceId) {
    r11_configured_effect_core(authored, |_| {})
}
fn r11_configured_effect_core(authored: &str, configure: impl FnOnce(&mut BattleScenario)) -> (BattleCore, super::super::sim::BattleExecutionState, UnitInstanceId, UnitInstanceId) {
    r11_equipped_effect_core(authored, configure, |_| {})
}
fn r11_equipped_effect_core(authored: &str, configure: impl FnOnce(&mut BattleScenario), configure_weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata)) -> (BattleCore, super::super::sim::BattleExecutionState, UnitInstanceId, UnitInstanceId) {
    r12_fragment_equipped_effect_core(authored, configure, configure_weapon, None)
}
fn r12_fragment_equipped_effect_core(authored: &str, configure: impl FnOnce(&mut BattleScenario), configure_weapon: impl FnOnce(&mut crate::game::data::equipment_data::EquipmentMetadata), fragments: Option<crate::game::data::skill_fragment_data::SkillFragmentDatabase>) -> (BattleCore, super::super::sim::BattleExecutionState, UnitInstanceId, UnitInstanceId) {
    let skills = SkillDatabase::from_ron("r11/effect.ron", authored).unwrap();
    let buffs: crate::game::battle::buffs::BuffDatabase = ron::de::from_str(r#"(buffs: [
        (name: "r11_silence", kind: Silence, tick_interval_ms: 0, max_stacks: 1, reapply_policy: RefreshDuration),
        (name: "r11_stun", kind: Stun, tick_interval_ms: 0, max_stacks: 1, reapply_policy: RefreshDuration)
    ])"#).unwrap();
    let weapon_id = Uuid::from_u128(0x1101);
    let mut weapon = crate::game::data::equipment_data::EquipmentMetadata {
        id: "r11_weapon".into(), uuid: weapon_id, name: "R11 weapon".into(),
        equipment_type: crate::game::data::equipment_data::EquipmentType::Weapon,
        rarity: crate::game::enums::RiskLevel::ZAYIN, price: 0, allow_duplicate_equip: true,
        bound: false, cannot_unequip_reason: "equipment_bound".into(),
        triggered_effects: Default::default(), ability_activations: Vec::new(),
        weapon_profile: Some(crate::game::data::equipment_data::WeaponCombatProfile { interval_ms: 1000, windup_ms: 1, ..Default::default() }), event_check_modifiers: Vec::new(), event_choice_keys: Vec::new(),
    };
    configure_weapon(&mut weapon);
    let weapon_id = weapon.uuid;
    let equipment = crate::game::data::equipment_data::EquipmentDatabase::new(vec![weapon.clone()]);
    equipment.validate_effect_and_activation_contract(&skills, &buffs).unwrap();
    let mut scenario = fixture_scenario_with_setup(None);
    for spawn in &mut scenario.groups[1].spawns {
        let BattleUnitSource::TestFixture { mut profile, .. } = spawn.draft.source.clone() else { unreachable!() };
        profile.skill_id = Some("r11_aura".into());
        profile.skill_activation_mode = SkillActivationMode::Manual;
        profile.stats.attack = 100;
        spawn.draft.source = BattleUnitSource::Employee(profile);
        spawn.draft.equipped_items = vec![crate::game::battle::types::BattleEquipmentBinding { base_uuid: weapon_id, persistent_copy_uuid: None, slot: None }];
        spawn.draft.employee_start_condition = Some(crate::game::battle::types::EmployeeBattleStartCondition { trauma: 0, battle_hp_bonus_percent: 0 });
    }
    configure(&mut scenario);
    let builder = GameDataBuilder::empty().with_skills(skills).with_equipment(vec![weapon]).with_buffs(buffs);
    let builder = if let Some(fragments) = fragments { builder.with_skill_fragments(fragments) } else { builder };
    let mut core = BattleCore::new_from_scenario(scenario, builder.build_arc(), 17);
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 0).unwrap();
    let mut players: Vec<_> = core.units.values().filter(|u| u.owner == Side::Player).map(|u| u.instance_id).collect(); players.sort();
    for id in &players { let u = core.units.get_mut(id).unwrap(); u.resonance_current = u.resonance_max; }
    (core, execution, players[0], players[1])
}
#[test]
fn r11_public_channel_aura_membership_damage_identity_and_reinput_stop() {
    use super::super::sim::BattleLiveCommand;
    let (mut core, mut execution, caster, member) = r11_aura_core();
    let block = core.units[&caster].block_capacity;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    assert!(core.channel_active(caster));
    assert!(core.resonance_use_has_obligations(caster));
    assert_eq!(core.units[&caster].resonance_current, 0);
    assert_eq!(core.aura_basic_bonus(caster), (0, 0));
    assert_eq!(core.aura_basic_bonus(member), (20, 15));
    assert_eq!(core.units[&member].stats.attack, 100, "skill Attack must stay unchanged");
    assert_eq!(core.units[&caster].block_capacity, block);
    assert_eq!(core.sustained_basic_interval(member, 1000), 869);
    assert!(core.tactical_relocation_disabled_reason(caster, 1).is_some());
    assert!(core.stance_change_disabled_reason(caster, crate::game::battle::types::EngagementStance::Defense, 1).is_some());
    let enemy = core.units.values().find(|u| u.owner == Side::Opponent).unwrap().instance_id;
    for (source, expected) in [(crate::game::battle::damage::DamageSource::BasicAttack, 120), (crate::game::battle::damage::DamageSource::Ability, 100)] {
        let snapshot = core.damage_source_snapshot_for_unit(member, enemy, source, crate::game::battle::damage::DamageType::True, 100, Default::default(), 0, 1, false).unwrap();
        assert_eq!(snapshot.base_damage, expected);
    }
    let resonance = core.units[&caster].resonance_current;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 1 }).unwrap();
    assert!(!core.channel_active(caster));
    assert!(!core.resonance_use_has_obligations(caster));
    assert_eq!(core.runtime_fact_summary().resource_spent, 100);
    assert_eq!(core.aura_basic_bonus(member), (0, 0));
    assert_eq!(core.units[&caster].resonance_current, resonance);
    assert!(core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 1 }).is_err());
}
#[test]
fn r11_channel_rejects_empty_start_and_withdraw_reclaims_membership() {
    use super::super::sim::BattleLiveCommand;
    let (mut core, mut execution, caster, member) = r11_aura_core();
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    core.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: member, time_ms: 1 }).unwrap();
    assert_eq!(core.aura_basic_bonus(member), (0, 0));
    core.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: caster, time_ms: 1 }).unwrap();
    assert_eq!(core.sustained.entity_count(), 0);
    let (mut empty, _, caster, member) = r11_aura_core();
    empty.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: member, time_ms: 0 }).unwrap();
    let before = empty.transaction_debug_snapshot();
    assert!(empty.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).is_err());
    assert_eq!(empty.transaction_debug_snapshot(), before);
}

#[test]
fn r11_channel_silence_damage_preserve_hard_cc_interrupt_end_only_own_lease() {
    use super::super::sim::BattleLiveCommand;
    use crate::game::battle::{buffs::BuffId, damage::{BattleCommand, DamageSource, DamageType}};
    for hard_cc in [false, true] {
        let (mut core, mut execution, caster, member) = r11_aura_core();
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 1).unwrap();
        core.process_event_for_test(BattleEvent::ApplyBuff { time_ms: 1, caster_instance_id: member, target_instance_id: caster, buff_id: BuffId::from_name("r11_silence"), duration_ms: 500, cause: Default::default() }, 1).unwrap();
        assert!(core.channel_active(caster));
        let enemy = core.units.values().find(|u| u.owner == Side::Opponent).unwrap().instance_id;
        let damage = core.damage_source_snapshot_for_unit(enemy, caster, DamageSource::Ability, DamageType::True, 1, Default::default(), 0, 1, false).unwrap();
        core.process_commands_with_modifier_source(vec![BattleCommand::ApplyDamage { target_id: caster, source_snapshot: damage }], 1, Some(enemy));
        assert!(core.channel_active(caster));
        core.units.get_mut(&caster).unwrap().action_locks.lock_movement_until(9000);
        if hard_cc {
            core.process_event_for_test(BattleEvent::ApplyBuff { time_ms: 1, caster_instance_id: enemy, target_instance_id: caster, buff_id: BuffId::from_name("r11_stun"), duration_ms: 500, cause: Default::default() }, 1).unwrap();
        } else {
            core.process_commands_with_modifier_source(vec![BattleCommand::InterruptCast { source_id: enemy, target_id: caster }], 1, Some(enemy));
        }
        assert!(!core.channel_active(caster));
        assert_eq!(core.aura_basic_bonus(member), (0, 0));
        assert_eq!(core.units[&caster].action_locks.movement_until_ms, 9000);
        let snapshot = core.sustained_effect_snapshot().unwrap();
        assert_eq!((snapshot.started, snapshot.ended), (1, 1));
        assert_eq!(snapshot.latest_end.unwrap().reason, if hard_cc { crate::game::ability::sustained::SustainedEndReason::HardCrowdControl } else { crate::game::ability::sustained::SustainedEndReason::Interrupted });
    }
}

#[test]
fn r11_structured_effects_settled_waits_for_sustained_children() {
    let skills = r11_modifier_skills().replace(" steps:", " flow: Some(Sequence(id: \"flow\", children: [Action(id: \"modify\", step_id: \"hit\", wait: EffectsSettled), Wait(id: \"after\", duration_ms: 100)])), steps:");
    let mut core = r08_core(R08_BOSS, &skills);
    let mut execution = core.start_battle_execution().unwrap();
    core.step_battle_execution_until(&mut execution, 499).unwrap();
    assert_eq!(r08_node(&core), "cast");
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    core.step_battle_execution_until(&mut execution, 599).unwrap();
    assert_eq!(r08_node(&core), "cast", "continuation Wait starts after the last sustained child closes at500");
    core.step_battle_execution_until(&mut execution, 600).unwrap();
    assert_eq!(r08_node(&core), "completed");
    assert!(core.active_skill_casts.is_empty());
}

fn r11_overlay_ron(delivery: &str, duration: u32, once: bool) -> String {
    R11_AURA.replace("ChannelAura(duration_ms: 1000, aura: (family: \"fetus\", priority: 0, radius_units: 2000000, basic_damage_percent: 20, basic_speed_points: 15))",
        &format!("BasicAttackOverlay(duration_ms: {duration}, next_attack_only: {once}, profile: (range_policy: whole_field_valid_tiles, defense_tile_range: None, delivery: {delivery}))"))
}
#[test]
fn r11_overlay_public_area_once_uses_weapon_and_restores_without_cycle_reset() {
    use super::super::sim::BattleLiveCommand;
    let delivery = r#"TileArea(footprint: (include_anchor_tile: true, rows: ["XXXXXXXXX", "XXXX@XXXX", "XXXXXXXXX"]), area: (anchor: CastTarget, tile_origin: Anchor))"#;
    let authored = r11_overlay_ron(delivery, 1500, true);
    let (mut core, mut execution, caster, _) = r11_configured_effect_core(&authored, |scenario| {
        let mut second = scenario.groups[0].spawns[0].clone();
        second.unit_ref = ScenarioUnitRef::new("overlay_second"); second.instance_salt = 33;
        second.draft = fixture_draft(44, 1000, 0); second.position = Position::new(5, 2);
        scenario.groups[0].spawns.push(second);
    });
    let original = serde_json::to_value(&core.units[&caster].basic_attack).unwrap();
    let source = core.units[&caster].source_identity.clone();
    let block = core.units[&caster].block_capacity;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    core.step_battle_execution_until(&mut execution, 1400).unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().ended, 1);
    assert!(!core.resonance_use_has_obligations(caster));
    assert_eq!(core.sustained_effect_snapshot().unwrap().latest_end.unwrap().reason, crate::game::ability::sustained::SustainedEndReason::AttackReleased);
    assert_eq!(serde_json::to_value(&core.units[&caster].basic_attack).unwrap(), original);
    assert_eq!(core.units[&caster].source_identity, source);
    assert_eq!(core.units[&caster].block_capacity, block);
    assert!((2001..2100).contains(&core.units[&caster].next_basic_attack_ms));
    let enemies: Vec<_> = core.units.values().filter(|u| u.owner == Side::Opponent).collect();
    assert_eq!(enemies.len(), 2);
    for enemy in enemies { assert_eq!(enemy.stats.current_health, 900); }
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 200);
    assert!(core.projectiles.is_empty());
}
#[test]
fn r11_overlay_directional_release_survives_expiry_and_pierces_once_per_enemy() {
    use super::super::sim::BattleLiveCommand;
    let delivery = "Projectile(speed_units_per_ms: 10000, hit_policy: DirectionalCollision, max_range_tiles: Some(7), max_lifetime_ms: Some(1000), max_pierces: Some(1), collision: (radius_units: 125000, piercing: true))";
    let authored = r11_overlay_ron(delivery, 1200, false);
    let (mut core, mut execution, _, _) = r11_configured_effect_core(&authored, |scenario| {
        scenario.groups[0].spawns[0].draft = fixture_draft(1, 1000, 0);
        let mut second = scenario.groups[0].spawns[0].clone();
        second.unit_ref = ScenarioUnitRef::new("overlay_second"); second.instance_salt = 33;
        second.draft = fixture_draft(44, 1000, 0); second.position = Position::new(4, 1);
        scenario.groups[0].spawns.push(second);
    });
    let caster = core.units.values().find(|u| u.owner == Side::Player && u.body.position.project_to_tile() == Position::new(1, 1)).unwrap().instance_id;
    let original = serde_json::to_value(&core.units[&caster].basic_attack).unwrap();
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1201).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert_eq!(serde_json::to_value(&core.units[&caster].basic_attack).unwrap(), original);
    assert_eq!(core.projectiles.len(), 1, "released owner outlives overlay");
    assert!(!core.resonance_use_has_obligations(caster));
    core.add_resonance(caster, 1, 1201, false);
    assert_eq!(core.units[&caster].resonance_current, 1);
    let released = core.projectiles.values().next().unwrap();
    assert_eq!(released.source_snapshot.source, crate::game::battle::damage::DamageSource::BasicAttack);
    assert!(released.source_snapshot.fact_source.cast.is_none());
    core.step_battle_execution_until(&mut execution, 1800).unwrap();
    for enemy in core.units.values().filter(|u| u.owner == Side::Opponent) { assert_eq!(enemy.stats.current_health, 900); }
    assert!(core.projectiles.is_empty());
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 200);
    assert!((2001..2100).contains(&core.units[&caster].next_basic_attack_ms));
}
#[test]
fn r11_overlay_public_contract_rejects_unimplemented_or_unbounded_delivery() {
    for delivery in [
        "Projectile(speed_units_per_ms: 10000, hit_policy: DirectionalCollision)",
        "Projectile(speed_units_per_ms: 10000, hit_policy: DirectionalCollision, max_range_tiles: Some(7), max_lifetime_ms: Some(1000), allow_targetless_cast: true)",
        r#"TileArea(footprint: (rows: ["X@X"]), area: (duration_ms: 100, tick_interval_ms: Some(10)))"#,
    ] { assert!(SkillDatabase::from_ron("r11/reject.ron", &r11_overlay_ron(delivery, 1500, false)).is_err()); }
}

fn r11_hello_goodbye_ron(moment: &str) -> String {
    let hello = r11_overlay_ron("Instant", 2000, false).replace("next_attack_only: false,", "next_attack_only: false, goodbye_skill: Some(\"r11_goodbye\"),");
    let child = format!(r#"SkillDef(id: "r11_goodbye", name: "Goodbye", kind: Targeted, focus_time_ms: 0,
      use_lifetime: Action,
      cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: true),
      steps: [SkillStepDef(id: "goodbye", delay_ms: 0, target: SelfUnit,
        delivery: TileArea(footprint: (include_anchor_tile: true, rows: ["XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXX@XXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX","XXXXXXXXXXXXXXX"]), area: (anchor: Caster, tile_origin: Caster)),
        effects: [(id: "remaining_damage", op: DamageExpr(amount: (moment: {moment}, root: Mul(value: Const((value: 10, unit: Magnitude)), scalar: Ratio(numerator: Read(Time(RemainingTime)), denominator: Const((value: 100, unit: Milliseconds)), rounding: Floor))), damage_type: True))])])"#);
    hello.replace("\n)])", &format!("\n), {child}])"))
}
#[test]
fn r11_hello_reinput_captures_remaining_time_once_without_resonance_or_basic_source() {
    use super::super::sim::BattleLiveCommand;
    for moment in ["Snapshot", "Live"] {
        let (mut core, mut execution, caster, _) = r11_employee_effect_core(&r11_hello_goodbye_ron(moment));
        core.enable_authoring_observation_with_options(Default::default(), true);
        core.bind_authoring_observation(super::super::authoring_observation::ObservationKey { entry_ordinal: 1, battle_uuid: Uuid::from_u128(0x1111) }).unwrap();
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 501).unwrap();
        let view = core.sustained_effect_snapshot().unwrap();
        let activation = view.active[0].activation;
        assert_eq!(view.active[0].expires_at_ms, 2001);
        let resonance = core.units[&caster].resonance_current;
        let cost = core.runtime_fact_summary().cost_mutations;
        let command = BattleLiveCommand::ReinputSustainedSkill { unit_id: caster, skill_id: "r11_aura".into(), activation, target: None, time_ms: 501 };
        core.apply_live_command(command.clone()).unwrap();
        assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
        assert_eq!(core.units[&caster].resonance_current, resonance);
        assert_eq!(core.runtime_fact_summary().cost_mutations, cost);
        assert_eq!(core.runtime_fact_summary().damage_hp_lost, 150);
        let amount = core.authoring_observation.as_ref().unwrap().latest_amount.as_ref().unwrap();
        assert_eq!(amount.source.cast.as_ref().unwrap().skill_id.as_str(), "r11_goodbye");
        assert!(amount.source.equipment.is_none());
        assert_eq!(amount.source.reinput().unwrap().remaining_ms, 1500);
        assert_eq!(amount.source.reinput().unwrap().committed_at_ms, 501);
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
        assert!(core.apply_live_command(command).is_err());
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
    }
}
#[test]
fn r11_hello_expiry_same_time_stale_activation_and_child_fault_are_atomic() {
    use super::super::sim::BattleLiveCommand;
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let authored = r11_hello_goodbye_ron("Snapshot").replace("duration_ms: 2000", "duration_ms: 500");
    let (mut core, mut execution, caster, _) = r11_employee_effect_core(&authored);
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 500).unwrap();
    let activation = core.sustained_effect_snapshot().unwrap().active[0].activation;
    assert!(core.apply_live_command(BattleLiveCommand::ReinputSustainedSkill { unit_id: caster, skill_id: "r11_aura".into(), activation, target: None, time_ms: 501 }).is_err());
    core.step_battle_execution_until(&mut execution, 501).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0, "natural expiry never invokes Goodbye");
    for site in ["reinput_after_consume", "reinput_after_invoke", "reinput_after_execution"] {
        let (mut core, mut execution, caster, _) = r11_employee_effect_core(&r11_hello_goodbye_ron("Live"));
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 501).unwrap();
        let activation = core.sustained_effect_snapshot().unwrap().active[0].activation;
        let command = BattleLiveCommand::ReinputSustainedSkill { unit_id: caster, skill_id: "r11_aura".into(), activation, target: None, time_ms: 501 };
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.apply_live_command(command.clone())));
        assert!(token.is_cancelled() && result.is_err(), "{site}: {result:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
        assert!(core.apply_live_command(BattleLiveCommand::ReinputSustainedSkill { unit_id: caster, skill_id: "r11_aura".into(), activation: activation + 1, target: None, time_ms: 501 }).is_err());
        core.apply_live_command(command).unwrap();
        assert_eq!(core.runtime_fact_summary().damage_hp_lost, 150);
    }
}
#[test]
fn r11_hello_actual_child_expression_error_restores_live_overlay_and_all_facts() {
    use super::super::sim::BattleLiveCommand;
    let authored = r11_hello_goodbye_ron("Live").replace(
        "Ratio(numerator: Read(Time(RemainingTime)), denominator: Const((value: 100, unit: Milliseconds)), rounding: Floor)",
        "Ratio(numerator: Const((value: 1, unit: Magnitude)), denominator: Read(Stat(subject: Source, stat: SpellPower)), rounding: Floor)");
    let (mut core, mut execution, caster, _) = r11_employee_effect_core(&authored);
    assert_eq!(core.units[&caster].stats.spell_power, 0);
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 501).unwrap();
    let activation = core.sustained_effect_snapshot().unwrap().active[0].activation;
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
    let error = core.apply_live_command(BattleLiveCommand::ReinputSustainedSkill { unit_id: caster, skill_id: "r11_aura".into(), activation, target: None, time_ms: 501 }).unwrap_err();
    assert!(matches!(error, crate::game::behavior::GameError::ValueExpressionFault(crate::game::value_expr::ValueExprFault { code: crate::game::value_expr::ValueExprFaultCode::DivideByZero, .. })), "{error:?}");
    assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
}
#[test]
fn r11_overlay_expiry_during_windup_uses_restored_weapon_without_consumption() {
    use super::super::sim::BattleLiveCommand;
    // The normal attack starts after cast recovery1001; a long original windup
    // crosses overlay expiry1201. Restored short range cannot release at the far target.
    let authored = r11_overlay_ron("Instant", 1200, true);
    let (mut core, mut execution, caster, _) = r11_configured_effect_core(&authored, |_| {});
    core.units.get_mut(&caster).unwrap().basic_attack.windup_ms = 400;
    let original = serde_json::to_value(&core.units[&caster].basic_attack).unwrap();
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1150).unwrap();
    assert!(core.units[&caster].basic_attack_windup_until_ms.is_some());
    let next = core.units[&caster].next_basic_attack_ms;
    core.step_battle_execution_until(&mut execution, 1500).unwrap();
    assert_eq!(core.units[&caster].next_basic_attack_ms, next);
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0);
    assert_eq!(core.sustained_effect_snapshot().unwrap().latest_end.unwrap().reason, crate::game::ability::sustained::SustainedEndReason::Expired);
    assert_eq!(serde_json::to_value(&core.units[&caster].basic_attack).unwrap(), original);
    assert!(core.projectiles.is_empty());
}
#[test]
fn r11_hello_old_activation_cannot_consume_a_new_activation() {
    use super::super::sim::BattleLiveCommand;
    let authored = r11_hello_goodbye_ron("Snapshot").replace("duration_ms: 2000", "duration_ms: 500");
    let (mut core, mut execution, caster, _) = r11_employee_effect_core(&authored);
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let old = core.sustained_effect_snapshot().unwrap().active[0].activation;
    core.step_battle_execution_until(&mut execution, 600).unwrap();
    core.units.get_mut(&caster).unwrap().resonance_current = core.units[&caster].resonance_max;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 600 }).unwrap();
    core.step_battle_execution_until(&mut execution, 601).unwrap();
    let fresh = core.sustained_effect_snapshot().unwrap();
    assert_ne!(fresh.active[0].activation, old);
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
    assert!(core.apply_live_command(BattleLiveCommand::ReinputSustainedSkill { unit_id: caster, skill_id: "r11_aura".into(), activation: old, target: None, time_ms: 601 }).is_err());
    assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
    assert_eq!(core.sustained_effect_snapshot().unwrap(), fresh);
}

#[test]
fn r11_aura_overlap_uses_priority_then_creation_and_removes_only_own_source() {
    use super::super::sim::BattleLiveCommand;
    for (priority, family, expected) in [(1, "fetus", (40, 30)), (0, "fetus", (20, 15)), (0, "other", (60, 45))] {
        let second = R11_AURA.replace("r11_aura", "r11_second")
            .replace("priority: 0", &format!("priority: {priority}"))
            .replace("family: \"fetus\"", &format!("family: \"{family}\""))
            .replace("basic_damage_percent: 20", "basic_damage_percent: 40")
            .replace("basic_speed_points: 15", "basic_speed_points: 30");
        let authored = format!("{}, {}", R11_AURA.strip_suffix("])" ).unwrap(), second.strip_prefix("SkillDatabase(skills: [").unwrap());
        let (mut core, mut execution, _, _) = r11_configured_effect_core(&authored, |scenario| {
            let mut third = scenario.groups[1].spawns[0].clone();
            third.unit_ref = ScenarioUnitRef::new("aura_member"); third.instance_salt = 77;
            third.draft.owned_uuid = Uuid::from_u128(77); third.position = Position::new(2, 1);
            scenario.groups[1].spawns.push(third);
            let BattleUnitSource::Employee(profile) = &mut scenario.groups[1].spawns[1].draft.source else { unreachable!() };
            profile.skill_id = Some("r11_second".into());
        });
        let at = |core: &BattleCore, p| core.units.values().find(|u| u.owner == Side::Player && u.body.position.project_to_tile() == p).unwrap().instance_id;
        let a = at(&core, Position::new(1, 1)); let b = at(&core, Position::new(1, 2)); let member = at(&core, Position::new(2, 1));
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 1).unwrap();
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: b, skill_id: "r11_second".into(), target: None, time_ms: 1 }).unwrap();
        core.step_battle_execution_until(&mut execution, 2).unwrap();
        assert_eq!(core.aura_basic_bonus(member), expected);
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 2 }).unwrap();
        assert_eq!(core.aura_basic_bonus(member), (40, 30));
        assert!(core.channel_active(b));
        core.step_battle_execution_until(&mut execution, 1002).unwrap();
        assert_eq!(core.aura_basic_bonus(member), (0, 0));
        assert_eq!(core.sustained_effect_snapshot().unwrap().ended, 2);
    }
}

#[test]
fn r11_aura_live_deploy_withdraw_redeploy_and_nonemployee_exclusion() {
    use super::super::sim::{BattleLiveCommand, BattleLiveCommandOutcome};
    use crate::game::battle::types::TacticalTileKind;
    let (mut core, mut execution, caster, _) = r11_aura_core();
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let original = core.scenario.groups[1].spawns[0].draft.clone();
    for variant in 0..2 {
        let mut draft = original.clone(); draft.owned_uuid = Uuid::from_u128(900 + variant);
        match variant {
            0 => {},
            1 => { draft.employee_start_condition = None; },
            _ => unreachable!(),
        }
        for salt in 0..2 {
            let BattleLiveCommandOutcome::UnitDeployed { unit_id } = core.apply_live_command(BattleLiveCommand::DeployPlayerUnit {
                draft: draft.clone(), position: Position::new(2, 1), tile_kind: TacticalTileKind::Ground,
                facing: FacingDirection::Right, instance_salt: (100 + variant * 10 + salt) as u32, time_ms: 1, current_hp_policy: None,
            }).unwrap_or_else(|e| panic!("variant {variant} salt {salt}: {e:?}")) else { unreachable!() };
            assert_eq!(core.aura_basic_bonus(unit_id), if variant == 0 { (20, 15) } else { (0, 0) }, "variant {variant}");
            core.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id, time_ms: 1 }).unwrap();
            assert_eq!(core.aura_basic_bonus(unit_id), (0, 0));
        }
    }
    assert_eq!(core.sustained_effect_snapshot().unwrap().started, 1);
}

#[test]
fn r11_aura_scenario_summon_and_defense_object_never_become_members() {
    use super::super::sim::BattleLiveCommand;
    let (mut core, mut execution, _, _) = r11_configured_effect_core(R11_AURA, |scenario| {
        for kind in 0..2 {
            let mut spawn = scenario.groups[1].spawns[0].clone();
            spawn.unit_ref = ScenarioUnitRef::new(format!("excluded_{kind}"));
            spawn.instance_salt = 200 + kind;
            spawn.draft.owned_uuid = Uuid::from_u128(200 + u128::from(kind));
            spawn.position = Position::new(2, (kind * 2) as i32);
            let BattleUnitSource::Employee(profile) = spawn.draft.source else { unreachable!() };
            spawn.draft.source = if kind == 0 {
                BattleUnitSource::DefenseObject { base_uuid: spawn.draft.owned_uuid, profile }
            } else {
                BattleUnitSource::SummonedUnit { definition_id: "excluded".into(), base_uuid: spawn.draft.owned_uuid, profile,
                    summoner_instance_id: UnitInstanceId(Uuid::from_u128(2)), summon_sequence: 1, batch_uuid: Uuid::from_u128(2000) }
            };
            scenario.groups[1].spawns.push(spawn);
        }
    });
    let caster = core.units.values().find(|u| u.owner == Side::Player && u.body.position.project_to_tile() == Position::new(1, 1)).unwrap().instance_id;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let view = core.sustained_effect_snapshot().unwrap();
    assert_eq!(view.active[0].members.len(), 1);
    for unit in core.units.values().filter(|u| u.owner == Side::Player && u.body.position.project_to_tile().x == 2) {
        assert!(!view.active[0].members.contains(&unit.instance_id));
        assert_eq!(core.aura_basic_bonus(unit.instance_id), (0, 0));
    }
}

#[test]
fn r11_aura_member_admission_failure_rolls_back_live_deployment_and_retries() {
    use super::super::sim::{BattleLiveCommand, BattleLiveCommandOutcome};
    use crate::game::battle::types::TacticalTileKind;
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let (mut core, mut execution, caster, _) = r11_aura_core();
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let mut draft = core.scenario.groups[1].spawns[0].draft.clone(); draft.owned_uuid = Uuid::from_u128(991);
    let command = BattleLiveCommand::DeployPlayerUnit { draft, position: Position::new(2, 1), tile_kind: TacticalTileKind::Ground, facing: FacingDirection::Right, instance_salt: 991, time_ms: 1, current_hp_policy: None };
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), core.sustained_effect_snapshot().unwrap());
    let token = CancellationToken::default();
    let result = control::with_test_cancellation("aura_membership", 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.apply_live_command(command.clone())));
    assert!(token.is_cancelled() && result.is_err());
    assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), core.sustained_effect_snapshot().unwrap()), before);
    let BattleLiveCommandOutcome::UnitDeployed { unit_id } = core.apply_live_command(command).unwrap() else { unreachable!() };
    assert_eq!(core.aura_basic_bonus(unit_id), (20, 15));
}

#[test]
fn r11_aura_normal_tactical_exit_reentry_and_channel_commands_preserve_state() {
    use super::super::sim::BattleLiveCommand;
    use crate::game::battle::types::{TacticalTileKind, EngagementStance};
    let authored = R11_AURA.replace("duration_ms: 1000", "duration_ms: 1000000");
    let (mut core, mut execution, caster, _) = r11_configured_effect_core(&authored, |scenario| {
        scenario.groups[0].spawns[0].draft = fixture_draft(1, 1_000_000, 0);
    });
    let mut draft = core.scenario.groups[1].spawns[0].draft.clone(); draft.owned_uuid = Uuid::from_u128(889);
    let super::super::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id: member } = core.apply_live_command(BattleLiveCommand::DeployPlayerUnit {
        draft, position: Position::new(2, 1), tile_kind: TacticalTileKind::Ground, facing: FacingDirection::Right,
        instance_salt: 889, time_ms: 0, current_hp_policy: None,
    }).unwrap() else { unreachable!() };
    let ready = core.tactical_states[&member].stance_cooldown_until_ms.max(core.tactical_states[&member].relocation_cooldown_until_ms);
    core.step_battle_execution_until(&mut execution, ready).unwrap();
    core.apply_live_command(BattleLiveCommand::SetEngagementStance { unit_id: member, stance: EngagementStance::Defense, time_ms: ready }).unwrap();
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: ready }).unwrap();
    core.step_battle_execution_until(&mut execution, ready + 1).unwrap();
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
    for command in [
        BattleLiveCommand::SetEngagementStance { unit_id: caster, stance: EngagementStance::Defense, time_ms: ready + 1 },
        BattleLiveCommand::TacticalRelocateUnit { unit_id: caster, destination: Position::new(7, 3), tile_kind: TacticalTileKind::Ground, facing: FacingDirection::Right, time_ms: ready + 1 },
        BattleLiveCommand::TacticalSwapUnits { first_unit_id: member, second_unit_id: caster, first_destination_tile_kind: TacticalTileKind::Ground, second_destination_tile_kind: TacticalTileKind::Ground, first_facing: FacingDirection::Right, second_facing: FacingDirection::Right, time_ms: ready + 1 },
    ] {
        assert!(core.apply_live_command(command).is_err());
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
    }
    let origin = core.units[&member].body.position.project_to_tile();
    let cooldown = core.game_data.run_policy.tactical_relocation.cooldown_ms;
    for round in 0..3 {
        let time_ms = ready + 1 + round * 2 * (cooldown + 1);
        core.step_battle_execution_until(&mut execution, time_ms).unwrap();
        core.apply_live_command(BattleLiveCommand::TacticalRelocateUnit { unit_id: member, destination: Position::new(7, 3), tile_kind: TacticalTileKind::Ground, facing: FacingDirection::Right, time_ms }).unwrap();
        assert_eq!(core.aura_basic_bonus(member), (0, 0));
        let time_ms = time_ms + cooldown + 1;
        core.step_battle_execution_until(&mut execution, time_ms).unwrap();
        core.apply_live_command(BattleLiveCommand::TacticalRelocateUnit { unit_id: member, destination: origin, tile_kind: TacticalTileKind::Ground, facing: FacingDirection::Right, time_ms }).unwrap();
        assert_eq!(core.aura_basic_bonus(member), (20, 15));
        assert_eq!(core.units[&member].stats.attack, 100);
    }
    assert!(core.channel_active(caster));
}

fn r12_shield_ron(shared: bool, expression: &str) -> String {
    format!(r#"SkillDatabase(skills: [
      SkillDef(id: "r11_aura", name: "Source-owned shield", kind: Targeted, focus_time_ms: 0,
        use_lifetime: ActionAndEffects(effects: [(step: "shield", effect: "pool")]),
        cast_targeting: Explicit(selection: self, target: SelfUnit, reach: pattern, air_capable: true),
        steps: [SkillStepDef(id: "shield", delay_ms: 0, target: SelfUnit, delivery: Instant,
            effects: [(id: "pool", op: ShieldExpr(amount: (moment: Snapshot, root: {expression}), duration_ms: 1000, priority: 0, shared_radius_units: {radius}))])]),
      SkillDef(id: "r08_strike", name: "Two real enemy strikes", kind: Targeted, focus_time_ms: 0,
        use_lifetime: Action,
        cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: true),
        steps: [
          SkillStepDef(id: "first", delay_ms: 10, target: CastTarget, delivery: Instant,
            effects: [(id: "hit", op: Damage(amount: 800, damage_type: True))]),
          SkillStepDef(id: "second", delay_ms: 20, target: EnemySingle(rule: LowestHealthEnemy), targeting: RetargetOnStep, delivery: Instant,
            effects: [(id: "hit", op: Damage(amount: 700, damage_type: True))])])
    ])"#, radius = if shared { "Some(2000000)" } else { "None" })
}
fn r12_shield_core(shared: bool, expression: &str) -> (BattleCore, super::super::sim::BattleExecutionState, UnitInstanceId, UnitInstanceId) {
    r12_configured_shield_core(&r12_shield_ron(shared, expression), |_| {})
}
fn r12_configured_shield_core(authored: &str, configure: impl FnOnce(&mut BattleScenario)) -> (BattleCore, super::super::sim::BattleExecutionState, UnitInstanceId, UnitInstanceId) {
    let boss = r08_core(R08_BOSS, R08_SKILLS).scenario.boss_runtime.clone();
    let (core, execution, _, _) = r11_configured_effect_core(authored, |scenario| {
        scenario.boss_runtime = boss;
        let BattleUnitSource::Employee(profile) = &mut scenario.groups[1].spawns[0].draft.source else { unreachable!() };
        profile.stats.max_health = 1100; profile.stats.current_health = 1100;
        configure(scenario);
    });
    let a = core.units.values().find(|u| u.owner == Side::Player && u.body.position.project_to_tile() == Position::new(1, 1)).unwrap().instance_id;
    let b = core.units.values().find(|u| u.owner == Side::Player && u.body.position.project_to_tile() == Position::new(1, 2)).unwrap().instance_id;
    (core, execution, a, b)
}

#[test]
fn r12_public_shared_pool600_absorbs400_then200_and_applies_only150_hp_loss() {
    use super::super::sim::BattleLiveCommand;
    let (mut core, mut execution, a, b) = r12_shield_core(true, "Const((value: 600, unit: Magnitude))");
    for id in [a, b] { core.units.get_mut(&id).unwrap().incoming_damage_modifiers.damage_reduction_percent = 50; }
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let view = core.sustained_effect_snapshot().unwrap();
    assert_eq!(view.active[0].members.len(), 2);
    assert_eq!(view.active[0].shield.unwrap().capacity, 600);
    core.step_battle_execution_until(&mut execution, 11).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1100);
    assert_eq!(core.sustained_effect_snapshot().unwrap().active[0].shield.unwrap().remaining, 200);
    let first = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
    assert_eq!((first.protection.mitigated_damage, first.protection.absorbed, first.actual_hp_loss()), (400, 400, 0));
    assert_eq!(first.segments[0].calculated_damage, 400);
    assert_eq!(first.protection.shields[0].source.cast.as_ref().unwrap().skill_id.as_str(), "r11_aura");
    core.step_battle_execution_until(&mut execution, 31).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1100);
    assert_eq!(core.units[&b].stats.current_health, 850);
    let last = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
    assert_eq!((last.protection.mitigated_damage, last.protection.absorbed, last.actual_hp_loss()), (350, 200, 150));
    let view = core.sustained_effect_snapshot().unwrap();
    assert!(view.active.is_empty());
    assert_eq!(view.ended, 1);
    assert_eq!(view.latest_end.unwrap().reason, crate::game::ability::sustained::SustainedEndReason::Depleted);
}

#[test]
fn r12_shield_consumption_faults_restore_pool_hp_facts_queue_and_retry() {
    use super::super::sim::BattleLiveCommand;
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["shield_receipt_vector", "shield_receipt_source", "shield_receipt_storage", "shield_after_absorption", "sustained_ended"] {
        let (mut core, mut execution, caster, _) = r12_shield_core(false, "Const((value: 600, unit: Magnitude))");
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 1).unwrap();
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), core.sustained_effect_snapshot().unwrap(), format!("{execution:?}"));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.step_battle_execution_until(&mut execution, 11)));
        assert!(token.is_cancelled() && result.is_err(), "{site}: {result:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), core.sustained_effect_snapshot().unwrap(), format!("{execution:?}")), before);
        core.step_battle_execution_until(&mut execution, 11).unwrap();
        assert_eq!(core.units[&caster].stats.current_health, 900);
        assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
        assert_eq!(core.runtime_fact_summary().latest_damage.as_ref().unwrap().protection.absorbed, 600);
    }
}

fn r12_add_use_effects(authored: &str, skill: &str, refs: &[(&str, &str)]) -> String {
    let mut result = authored.to_owned();
    let begin = result.find(&format!("id: \"{skill}\"")).unwrap();
    let descriptor = begin + result[begin..].find("use_lifetime:").unwrap();
    let refs = refs.iter().map(|(step, effect)| format!("(step: \"{step}\", effect: \"{effect}\")")).collect::<Vec<_>>().join(", ");
    if result[descriptor..].starts_with("use_lifetime: ActionAndEffects") {
        let end = descriptor + result[descriptor..].find(']').unwrap();
        result.insert_str(end, &format!(", {refs}"));
    } else {
        let end = descriptor + "use_lifetime: Action".len();
        result.replace_range(descriptor..end, &format!("use_lifetime: ActionAndEffects(effects: [{refs}])"));
    }
    result
}
fn r12_append_pool_effect(authored: &str, effects: &str, refs: &[&str]) -> String {
    let mut result = authored.to_owned();
    let begin = result.find("shared_radius_units:").unwrap();
    let end = begin + result[begin..].find("])").unwrap();
    result.insert_str(end, &format!(", {effects}"));
    r12_add_use_effects(&result, "r11_aura", &refs.iter().map(|id| ("shield", *id)).collect::<Vec<_>>())
}
fn r12_append_step(authored: &str, skill: &str, step: &str) -> String {
    let mut result = authored.to_owned();
    let begin = result.find(&format!("id: \"{skill}\"")).unwrap();
    let steps = begin + result[begin..].find("steps:").unwrap();
    let open = steps + result[steps..].find('[').unwrap();
    let mut depth = 0;
    let mut quoted = false;
    let mut escaped = false;
    let mut end = None;
    for (offset, ch) in result[open..].char_indices() {
        if quoted {
            if escaped { escaped = false; }
            else if ch == '\\' { escaped = true; }
            else if ch == '"' { quoted = false; }
            continue;
        }
        match ch {
            '"' => quoted = true,
            '[' => depth += 1,
            ']' => { depth -= 1; if depth == 0 { end = Some(open + offset); break; } }
            _ => {}
        }
    }
    let end = end.unwrap();
    result.insert_str(end, &format!(", {step}"));
    result
}

#[test]
fn r12_personal_then_shared_priority_preserves_single_mitigation_and_target_hp() {
    use super::super::sim::BattleLiveCommand;
    let authored = r12_append_pool_effect(&r12_shield_ron(true, "Const((value: 200, unit: Magnitude))"), r#"
        (id: "high", op: Sustained(definition: Shield(duration_ms: 1000, amount: 100, priority: 10, shared_radius_units: Some(2000000)))),
        (id: "personal", op: Sustained(definition: Shield(duration_ms: 1000, amount: 50, priority: -100, shared_radius_units: None)))"#, &["high", "personal"]);
    let (mut core, mut execution, a, b) = r12_configured_shield_core(&authored, |_| {});
    core.units.get_mut(&a).unwrap().incoming_damage_modifiers.damage_reduction_percent = 50;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 11).unwrap();
    let fact = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
    assert_eq!((fact.protection.mitigated_damage, fact.protection.absorbed, fact.actual_hp_loss()), (400, 350, 50));
    let consumed: Vec<_> = fact.protection.shields.iter().map(|s| (s.activation, s.absorbed)).collect();
    assert_eq!(consumed, vec![(3, 50), (2, 100), (1, 200)]);
    assert_eq!(core.units[&a].stats.current_health, 1050);
    assert_eq!(core.units[&b].stats.current_health, 1000);
    assert_eq!(core.sustained_effect_snapshot().unwrap().ended, 3);
}

#[test]
fn r12_health_cost_is_separate_from_damage_heal_shield_and_hp_floor() {
    use super::super::sim::BattleLiveCommand;
    use crate::game::battle::runtime_facts::{FactResource, CostReason};
    for (amount, lethal) in [(1099, false), (1100, false), (1100, true)] {
        let authored = r12_append_pool_effect(&r12_shield_ron(false, "Const((value: 600, unit: Magnitude))"), &format!(r#"
          (id: "floor", op: Sustained(definition: Survival(duration_ms: 1000, policy: FloorOne))),
          (id: "prevent", op: Sustained(definition: Survival(duration_ms: 1000, policy: PreventDeathOnce))),
          (id: "payment", op: HealthCost(amount: {amount}, allow_lethal: {lethal}))"#), &["floor", "prevent"]);
        let (mut core, mut execution, a, _) = r12_configured_shield_core(&authored, |_| {});
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}"));
        let result = core.step_battle_execution_until(&mut execution, 1);
        if amount == 1100 && !lethal {
            assert!(matches!(result, Err(GameError::BattleRuntimeFault(f)) if f.code == crate::game::battle::runtime_control::RuntimeFaultCode::HealthCostUnavailable));
            assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}")), before);
            assert_eq!(core.sustained_effect_snapshot().unwrap().started, 0);
            continue;
        }
        result.unwrap();
        let facts = core.runtime_fact_summary();
        let cost = facts.latest_cost.as_ref().unwrap();
        assert_eq!(cost.resource, FactResource::Health);
        assert_eq!(cost.reason, CostReason::AuthoredHealth);
        assert_eq!(cost.actual_spent(), i64::from(amount));
        assert_eq!((facts.damage_hp_lost, facts.heal_mutations), (0, 0));
        if lethal {
            assert!(core.units.get(&a).is_none_or(|u| !u.is_active()));
            assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
            assert!(facts.latest_death.as_ref().unwrap().lethal_damage.is_none());
        } else {
            assert_eq!(core.units[&a].stats.current_health, 1);
            assert_eq!(core.sustained_effect_snapshot().unwrap().active[0].shield.unwrap().remaining, 600);
        }
    }
}

#[test]
fn r12_floor_precedes_one_use_death_prevention_then_ordinary_death_is_recorded() {
    use super::super::sim::BattleLiveCommand;
    use crate::game::ability::sustained::SurvivalPolicy;
    let mut authored = r12_append_pool_effect(&r12_shield_ron(false, "Const((value: 1, unit: Magnitude))"), r#"
       (id: "floor", op: Sustained(definition: Survival(duration_ms: 40, policy: FloorOne))),
       (id: "prevent", op: Sustained(definition: Survival(duration_ms: 1000, policy: PreventDeathOnce)))"#, &["floor", "prevent"]);
    for (id, delay) in [("third", 60), ("fourth", 70)] {
        authored = r12_append_step(&authored, "r08_strike", &format!(r#"SkillStepDef(id: "{id}", delay_ms: {delay}, target: EnemySingle(rule: LowestHealthEnemy), targeting: RetargetOnStep, delivery: Instant, effects: [(id: "hit", op: Damage(amount: 700, damage_type: True))])"#));
    }
    let (mut core, mut execution, a, _) = r12_configured_shield_core(&authored, |_| {});
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 31).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1);
    assert_eq!(core.runtime_fact_summary().latest_damage.as_ref().unwrap().protection.survival.as_ref().unwrap().policy, SurvivalPolicy::FloorOne);
    assert!(core.sustained_effect_snapshot().unwrap().active.iter().any(|r| r.survival == Some(SurvivalPolicy::PreventDeathOnce)));
    core.step_battle_execution_until(&mut execution, 61).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1);
    let fact = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
    assert_eq!(fact.actual_hp_loss(), 0);
    assert_eq!(fact.protection.survival.as_ref().unwrap().policy, SurvivalPolicy::PreventDeathOnce);
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    core.step_battle_execution_until(&mut execution, 71).unwrap();
    assert!(core.units.get(&a).is_none_or(|u| !u.is_active()));
    let death = core.runtime_fact_summary().latest_death.as_ref().unwrap();
    assert_eq!(death.lethal_damage.as_ref().unwrap().actual_hp_loss(), 1);
}

#[test]
fn r12_heal_and_health_cost_leave_shield_unchanged_only_recharge_changes_balance() {
    use super::super::sim::BattleLiveCommand;
    for (amount, after, final_hp) in [(100, 300, 950), (999, 600, 1000)] {
        let mut authored = r12_shield_ron(true, "Const((value: 600, unit: Magnitude))");
        authored = r12_append_step(&authored, "r11_aura", r#"SkillStepDef(id: "recover", delay_ms: 12, target: SelfUnit, delivery: Instant, effects: [(id: "cost", op: HealthCost(amount: 100, allow_lethal: false)), (id: "heal", op: Heal(amount: 100))])"#);
        authored = r12_append_step(&authored, "r11_aura", &format!(r#"SkillStepDef(id: "recharge", delay_ms: 13, target: SelfUnit, delivery: Instant, effects: [(id: "refill", op: RechargeShield(source_skill: "r11_aura", amount: {amount}))])"#));
        let (mut core, mut execution, a, b) = r12_configured_shield_core(&authored, |_| {});
        for id in [a, b] { core.units.get_mut(&id).unwrap().incoming_damage_modifiers.damage_reduction_percent = 50; }
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 13).unwrap();
        assert_eq!(core.units[&a].stats.current_health, 1100);
        assert_eq!(core.runtime_fact_summary().healing_delta, 100);
        assert_eq!(core.sustained_effect_snapshot().unwrap().active[0].shield.unwrap().remaining, 200);
        core.step_battle_execution_until(&mut execution, 14).unwrap();
        let snapshot = core.sustained_effect_snapshot().unwrap();
        assert_eq!(snapshot.active[0].shield.unwrap().capacity, 600);
        assert_eq!(snapshot.active[0].shield.unwrap().remaining, after);
        let receipt = snapshot.latest_recharge.unwrap();
        assert_eq!((receipt.before, receipt.after), (200, after));
        assert_eq!(receipt.authored_site.unwrap().effect_id, "refill");
        core.step_battle_execution_until(&mut execution, 31).unwrap();
        assert_eq!(core.units[&b].stats.current_health, final_hp);
        assert_eq!(core.runtime_fact_summary().heal_mutations, 1);
    }
}

fn r12_control_ron(op: &str) -> String {
    format!(r#"SkillDatabase(skills: [SkillDef(id: "r11_aura", name: "Authored enemy control", kind: Targeted, focus_time_ms: 0,
      use_lifetime: Action,
      cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: true),
      steps: [SkillStepDef(id: "control", delay_ms: 0, target: CastTarget, delivery: Instant, effects: [(id: "control", op: {op})])]
    )])"#)
}
fn r12_ranged_enemy(scenario: &mut BattleScenario) {
    let BattleUnitSource::TestFixture { profile, .. } = &mut scenario.groups[0].spawns[0].draft.source else { unreachable!() };
    profile.basic_attack.range_policy = crate::game::battle::tile_range::TileRangePolicy::WholeFieldValidTiles;
    profile.basic_attack.defense_tile_range = None;
    profile.basic_attack.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
    profile.basic_attack.ranged_reposition_ms = 0;
    profile.weapon_profile.as_mut().unwrap().range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
    profile.stats.attack = 50;
    profile.stats.move_speed_units_per_ms = 0;
}

#[test]
fn r12_slow_and_root_change_real_enemy_movement_without_stat_drift() {
    use super::super::sim::BattleLiveCommand;
    for (kind, factor) in [("Slow(percent: 50)", 0.5f32), ("Root", 0.0)] {
        let authored = r12_control_ron(&format!("Sustained(definition: Control(duration_ms: 200, kind: {kind}))"));
        let (mut plain, mut plain_execution, _, _) = r11_employee_effect_core(&authored);
        let (mut core, mut execution, caster, _) = r11_employee_effect_core(&authored);
        let enemy = core.units.values().find(|u| u.owner == Side::Opponent).unwrap().instance_id;
        let start = core.units[&enemy].body.position;
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        plain.step_battle_execution_until(&mut plain_execution, 200).unwrap();
        core.step_battle_execution_until(&mut execution, 200).unwrap();
        let full = start.x - plain.units[&enemy].body.position.x;
        let controlled = start.x - core.units[&enemy].body.position.x;
        assert!(full > 0.05, "movement fixture must actually advance");
        assert!((controlled - full * factor).abs() < 0.005, "{kind}: full {full}, actual {controlled}");
        assert_eq!(core.units[&enemy].block_capacity, plain.units[&enemy].block_capacity);
        core.units.get_mut(&enemy).unwrap().action_locks.lock_movement_until(9000);
        core.step_battle_execution_until(&mut execution, 300).unwrap();
        assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 1000);
        assert_eq!(core.units[&enemy].action_locks.movement_until_ms, 9000);
        assert!(!core.rooted(enemy));
    }
}

#[test]
fn r12_root_keeps_basic_attack_capability_and_taunt_changes_actual_attack_target() {
    use super::super::sim::BattleLiveCommand;
    for kind in ["Root", "Taunt(priority: 0)"] {
        let authored = r12_control_ron(&format!("Sustained(definition: Control(duration_ms: 1500, kind: {kind}))"));
        let (mut core, mut execution, a, b) = r11_configured_effect_core(&authored, r12_ranged_enemy);
        let enemy = core.units.values().find(|u| u.owner == Side::Opponent).unwrap().instance_id;
        let original = core.choose_attack_target_in_range(enemy).unwrap();
        let caster = if original == a { b } else { a };
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 2).unwrap();
        let first_hp = core.units[&caster].stats.current_health;
        core.step_battle_execution_until(&mut execution, 1100).unwrap();
        let hit = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
        assert_eq!(hit.source.actor, Some(enemy));
        assert!(hit.time_ms >= 1000);
        assert!(hit.source.cast.is_none(), "control cannot turn a basic attack into skill damage");
        if kind == "Root" {
            assert!(core.rooted(enemy));
            assert_eq!(hit.target, original);
        } else {
            assert_eq!(hit.target, caster);
            assert!(core.units[&caster].stats.current_health < first_hp);
            core.step_battle_execution_until(&mut execution, 2200).unwrap();
            assert_eq!(core.taunt_source(enemy), None);
            assert_eq!(core.runtime_fact_summary().latest_damage.as_ref().unwrap().target, original);
        }
    }
}

#[test]
fn r12_pull_uses_valid_unoccupied_cells_and_fault_restores_body_queue_and_ledger() {
    use super::super::sim::BattleLiveCommand;
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let authored = r12_control_ron("Pull(max_distance_tiles: 2)");
    for obstacle in [false, true] {
        let (mut core, mut execution, caster, _) = r11_configured_effect_core(&authored, |scenario| {
            if obstacle { scenario.battlefield.obstacles.push(Position::new(5, 1)); }
        });
        let enemy = core.units.values().find(|u| u.owner == Side::Opponent).unwrap().instance_id;
        let before = core.units[&enemy].body.position;
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 0).unwrap();
        if !obstacle {
            let snapshot = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}"));
            let token = CancellationToken::default();
            let result = control::with_test_cancellation("pull_after_displacement", 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.step_battle_execution_until(&mut execution, 1)));
            assert!(token.is_cancelled() && result.is_err());
            assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}")), snapshot);
        }
        core.step_battle_execution_until(&mut execution, 1).unwrap();
        let after = core.units[&enemy].body.position;
        let expected_x = if obstacle { before.x } else { before.project_to_tile().x as f32 - 2.0 + 0.5 };
        assert_eq!(after.x, expected_x);
        assert_eq!(after.y, before.y);
        let displaced = core.event_log.entries.iter().filter(|e| matches!(e.event, crate::game::battle::event_log::BattleLogEvent::UnitDisplaced { .. })).count();
        assert_eq!(displaced, usize::from(!obstacle));
    }
}

fn r12_parry_ron(window: u32, attachments: bool) -> String {
    let mut authored = r12_shield_ron(false, "Const((value: 600, unit: Magnitude))");
    let start = authored.find("ShieldExpr(").unwrap();
    let end = start + authored[start..].find("shared_radius_units: None)").unwrap() + "shared_radius_units: None)".len();
    authored.replace_range(start..end, &format!("Sustained(definition: Parry(duration_ms: 200, window_ms: {window}, invulnerable_ms: 40, miss_recovery_ms: 20, reduction_percent: 30, counter_skill: \"r12_counter\"))"));
    authored = authored.replace("target: EnemySingle(rule: LowestHealthEnemy), targeting: RetargetOnStep", "target: CastTarget");
    let counter = r#", SkillDef(id: "r12_counter", name: "Exact attacker counter", kind: Targeted, focus_time_ms: 0,
      use_lifetime: Action,
      cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: true),
      steps: [SkillStepDef(id: "counter", delay_ms: 0, target: CastTarget, delivery: Instant,
        effects: [(id: "counter_hit", op: Damage(amount: 100, damage_type: True))])])
    "#;
    let end = authored.rfind("])").unwrap();
    authored.insert_str(end, counter);
    if attachments {
        authored = authored.replacen("Damage(amount: 800, damage_type: True))]", r#"Damage(amount: 800, damage_type: True)),
            (id: "bound_root", attached_to: Some("hit"), op: Sustained(definition: Control(duration_ms: 100, kind: Root))),
            (id: "independent", op: ModifyStats(modifier: (stat: Attack, kind: Flat, value: -10))),
            (id: "second_segment", op: Damage(amount: 100, damage_type: True))]"#, 1);
    }
    authored
}
#[test]
fn r12_parry_suppresses_bound_attachment_counters_once_and_preserves_unrelated_effect() {
    use super::super::sim::BattleLiveCommand;
    use crate::game::battle::runtime_facts::DeflectionKind;
    let (mut core, mut execution, a, _) = r12_configured_shield_core(&r12_parry_ron(30, true), |_| {});
    let enemy = core.units.values().find(|u| u.owner == Side::Opponent).unwrap().instance_id;
    let enemy_hp = core.units[&enemy].stats.current_health;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 11).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1100);
    assert_eq!(core.units[&a].stats.attack, 90, "unbound operation retains its authored order");
    assert!(!core.rooted(a));
    assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 100);
    let facts = core.runtime_fact_summary();
    assert_eq!(facts.damage_hp_lost, 100);
    let counter = facts.latest_nonzero_damage.as_ref().unwrap();
    assert_eq!(counter.source.cast.as_ref().unwrap().skill_id.as_str(), "r12_counter");
    assert!(counter.source.reaction);
    assert_eq!(facts.latest_damage.as_ref().unwrap().protection.deflection.as_ref().unwrap().kind, DeflectionKind::Invulnerability);
    core.step_battle_execution_until(&mut execution, 21).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1100);
    assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 100);
    let parry = core.sustained_effect_snapshot().unwrap().active[0].parry.clone().unwrap();
    assert_eq!(parry.invulnerable_until_ms, Some(51));
}
#[test]
fn r12_parry_miss_disables_only_own_reduction_then_recovers_without_rearming() {
    use super::super::sim::BattleLiveCommand;
    let authored = r12_parry_ron(5, false).replace("Damage(amount: 700", "Damage(amount: 100");
    let authored = r12_append_step(&authored, "r08_strike", r#"SkillStepDef(id: "recovered", delay_ms: 30, target: CastTarget, delivery: Instant, effects: [(id: "hit", op: Damage(amount: 100, damage_type: True))])"#);
    let (mut core, mut execution, a, _) = r12_configured_shield_core(&authored, |_| {});
    core.units.get_mut(&a).unwrap().incoming_damage_modifiers.damage_reduction_percent = 20;
    core.units.get_mut(&a).unwrap().stats.defense = 42;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 11).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 460);
    core.step_battle_execution_until(&mut execution, 21).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 380);
    core.step_battle_execution_until(&mut execution, 31).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 330);
    assert_eq!(core.units[&a].incoming_damage_modifiers.damage_reduction_percent, 20);
    assert_eq!(core.units[&a].stats.defense, 42);
    assert_eq!(core.sustained_effect_snapshot().unwrap().active[0].parry.as_ref().unwrap().invulnerable_until_ms, None);
    assert!(core.runtime_fact_summary().latest_damage.as_ref().unwrap().protection.deflection.is_none());
}
#[test]
fn r12_parry_and_atomic_counter_faults_restore_hp_activation_queue_and_observation() {
    use super::super::sim::BattleLiveCommand;
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["parry_receipt_source", "parry_receipt_storage", "parry_after_deflection", "parry_after_counter_invoke", "parry_after_counter_execution"] {
        let (mut core, mut execution, a, _) = r12_configured_shield_core(&r12_parry_ron(30, true), |_| {});
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 10).unwrap();
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}"));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.step_battle_execution_until(&mut execution, 11)));
        assert!(token.is_cancelled() && result.is_err(), "{site}: {result:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}")), before, "{site}");
        core.step_battle_execution_until(&mut execution, 11).unwrap();
        assert_eq!(core.units[&a].stats.current_health, 1100);
        assert_eq!(core.runtime_fact_summary().damage_hp_lost, 100);
    }
}
#[test]
fn r12_parry_attachment_and_counter_publication_rejects_ambiguous_bindings() {
    let valid = r12_parry_ron(30, true);
    SkillDatabase::from_ron("r12/parry.ron", &valid).unwrap();
    for bad in [valid.replace("attached_to: Some(\"hit\")", "attached_to: Some(\"second_segment\")"),
        valid.replace("counter_skill: \"r12_counter\"", "counter_skill: \"r11_aura\""),
        valid.replace("invulnerable_ms: 40", "invulnerable_ms: 400"),
        valid.replace("attached_to: Some(\"hit\"), op: Sustained(definition: Control(duration_ms: 100, kind: Root))", "attached_to: Some(\"hit\"), op: Heal(amount: 1)")]
    { assert!(SkillDatabase::from_ron("r12/invalid_parry.ron", &bad).is_err()); }
}

fn r12_lifesteal_weapon(weapon: &mut crate::game::data::equipment_data::EquipmentMetadata, cap: u32) {
    weapon.triggered_effects = ron::de::from_str(&format!(r#"{{OnAttack: [(target: SelfUnit, effect: BasicLifesteal(percent: 25, max_heal: {cap}))]}}"#)).unwrap();
}
fn r12_with_initial_health_cost(authored: &str) -> String {
    r12_append_step(authored, "r11_aura", r#"SkillStepDef(id: "cost", delay_ms: 0, target: SelfUnit, delivery: Instant, effects: [(id: "pay", op: HealthCost(amount: 500, allow_lethal: false))])"#)
}
#[test]
fn r12_basic_lifesteal_sums_actual_area_hp_before_rounding_and_one_cap() {
    use super::super::sim::BattleLiveCommand;
    let delivery = r#"TileArea(footprint: (include_anchor_tile: true, rows: ["XXXXXXXXX", "XXXX@XXXX", "XXXXXXXXX"]), area: (anchor: CastTarget, tile_origin: Anchor))"#;
    for (hp, cap, expected) in [(3, 100, 1), (1000, 30, 30), (1000, 100, 50)] {
        let authored = r12_with_initial_health_cost(&r11_overlay_ron(delivery, 1500, true));
        let (mut core, mut execution, caster, _) = r11_equipped_effect_core(&authored, |scenario| {
            scenario.groups[0].spawns[0].draft = fixture_draft(1, hp, 0);
            let mut second = scenario.groups[0].spawns[0].clone();
            second.unit_ref = ScenarioUnitRef::new("lifesteal_second"); second.instance_salt = 33;
            second.draft = fixture_draft(44, hp, 0); second.position = Position::new(5, 2);
            scenario.groups[0].spawns.push(second);
        }, |weapon| r12_lifesteal_weapon(weapon, cap));
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 1400).unwrap();
        assert_eq!(core.units[&caster].stats.current_health, 500 + expected);
        let facts = core.runtime_fact_summary();
        assert_eq!(facts.damage_hp_lost, u64::from(hp.min(100)) * 2);
        assert_eq!((facts.heal_mutations, facts.healing_delta), (1, i64::from(expected)));
        let heal = facts.latest_heal.as_ref().unwrap();
        assert_eq!(heal.source.equipment.unwrap().definition_uuid, Uuid::from_u128(0x1101));
        assert!(heal.source.cast.is_none());
    }
}
#[test]
fn r12_directional_basic_lifesteal_uses_release_provider_after_overlay_expiry() {
    use super::super::sim::BattleLiveCommand;
    let delivery = "Projectile(speed_units_per_ms: 10000, hit_policy: DirectionalCollision, max_range_tiles: Some(7), max_lifetime_ms: Some(1000), max_pierces: Some(1), collision: (radius_units: 125000, piercing: true))";
    let authored = r12_with_initial_health_cost(&r11_overlay_ron(delivery, 1200, false));
    let (mut core, mut execution, _, _) = r11_equipped_effect_core(&authored, |scenario| {
        scenario.groups[0].spawns[0].draft = fixture_draft(1, 1000, 0);
        let mut second = scenario.groups[0].spawns[0].clone();
        second.unit_ref = ScenarioUnitRef::new("lifesteal_second"); second.instance_salt = 33;
        second.draft = fixture_draft(44, 1000, 0); second.position = Position::new(4, 1);
        scenario.groups[0].spawns.push(second);
    }, |weapon| r12_lifesteal_weapon(weapon, 35));
    let caster = core.units.values().find(|u| u.owner == Side::Player && u.body.position.project_to_tile() == Position::new(1, 1)).unwrap().instance_id;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1201).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert_eq!(core.units[&caster].stats.current_health, 500, "heal awaits the whole released projectile");
    core.step_battle_execution_until(&mut execution, 1800).unwrap();
    assert!(core.projectiles.is_empty());
    assert_eq!(core.units[&caster].stats.current_health, 535);
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 200);
    assert_eq!(core.runtime_fact_summary().heal_mutations, 1);
}
#[test]
fn r12_goodbye_is_not_basic_lifesteal_and_settlement_fault_rolls_back_the_attack() {
    use super::super::sim::BattleLiveCommand;
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let authored = r12_with_initial_health_cost(&r11_hello_goodbye_ron("Snapshot"));
    let (mut core, mut execution, caster, _) = r11_equipped_effect_core(&authored, |_| {}, |weapon| r12_lifesteal_weapon(weapon, 100));
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let mut probe = core.clone();
    let mut probe_execution = execution.clone();
    probe.step_battle_execution_until(&mut probe_execution, 1400).unwrap();
    assert_eq!(probe.runtime_fact_summary().heal_mutations, 1);
    let hit_time = probe.runtime_fact_summary().latest_heal.as_ref().unwrap().time_ms;
    core.step_battle_execution_until(&mut execution, hit_time - 1).unwrap();
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}"));
    let token = CancellationToken::default();
    let result = control::with_test_cancellation("basic_lifesteal_after_heal", 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.step_battle_execution_until(&mut execution, hit_time)));
    assert!(token.is_cancelled() && result.is_err());
    assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}")), before);
    core.step_battle_execution_until(&mut execution, hit_time).unwrap();
    assert_eq!(core.units[&caster].stats.current_health, 525);
    let activation = core.sustained_effect_snapshot().unwrap().active[0].activation;
    core.step_battle_execution_until(&mut execution, hit_time + 10).unwrap();
    core.apply_live_command(BattleLiveCommand::ReinputSustainedSkill { unit_id: caster, skill_id: "r11_aura".into(), activation, target: None, time_ms: hit_time + 10 }).unwrap();
    assert_eq!(core.units[&caster].stats.current_health, 525);
    assert_eq!(core.runtime_fact_summary().heal_mutations, 1);
    assert_eq!(core.runtime_fact_summary().latest_damage.as_ref().unwrap().source.cast.as_ref().unwrap().skill_id.as_str(), "r11_goodbye");
}

fn r12_guard_core(window: u32, chance: u32, segments: bool) -> (BattleCore, super::super::sim::BattleExecutionState, UnitInstanceId, UnitInstanceId) {
    r12_guard_core_custom(window, chance, segments, |s| s, |_| {})
}
fn r12_guard_core_custom(window: u32, chance: u32, segments: bool, edit: impl FnOnce(String) -> String, configure: impl FnOnce(&mut BattleScenario)) -> (BattleCore, super::super::sim::BattleExecutionState, UnitInstanceId, UnitInstanceId) {
    let mut authored = r12_parry_ron(window, false).replace("reduction_percent: 30", "reduction_percent: 0").replace("Damage(amount: 800", "Damage(amount: 100").replace("Damage(amount: 700", "Damage(amount: 100");
    if segments { authored = authored.replacen("Damage(amount: 100, damage_type: True))]", r#"Damage(amount: 100, damage_type: True)), (id: "segment2", op: Damage(amount: 100, damage_type: True))]"#, 1); }
    authored = r12_append_step(&authored, "r08_strike", r#"SkillStepDef(id: "after_recovery", delay_ms: 30, target: CastTarget, delivery: Instant, effects: [(id: "hit", op: Damage(amount: 100, damage_type: True))])"#);
    let authored = edit(authored);
    let fragments: crate::game::data::skill_fragment_data::SkillFragmentDatabase = ron::de::from_str(&include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r09/fragments.ron")).replace("r08_strike", "r11_aura").replace("ActiveSkill(forms:", &format!("ActiveSkill(passive_guard: Some((reduction_percent: 30, reflection_chance_percent: {chance}, reflection_percent: 50)), forms:"))).unwrap();
    let boss = r08_core(R08_BOSS, R08_SKILLS).scenario.boss_runtime.clone();
    let (core, execution, _, _) = r12_fragment_equipped_effect_core(&authored, |scenario| {
        scenario.boss_runtime = boss;
        for (i, spawn) in scenario.groups[1].spawns.iter_mut().enumerate() {
            let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else { unreachable!() };
            profile.stats.max_health = 1100; profile.stats.current_health = 1100;
            profile.fragment_source = Some(crate::game::skill_fragment::FragmentSourceSnapshot { copy_uuid: Uuid::from_u128(0x1200 + i as u128), definition_id: "r09_fragment".into(), form_id: "base".into(), skill_id: "r11_aura".into() });
        }
        configure(scenario);
    }, |_| {}, Some(fragments));
    let a = core.units.values().find(|u| u.owner == Side::Player && u.body.position.project_to_tile() == Position::new(1, 1)).unwrap().instance_id;
    let enemy = actor_runtime(&core).unit_id.unwrap();
    (core, execution, a, enemy)
}
#[test]
fn r12_passive_guard_at_zero_charge_reduces_reflects_once_per_strike_and_keeps_copy() {
    let (mut core, mut execution, a, enemy) = r12_guard_core(5, 100, true);
    core.units.get_mut(&a).unwrap().resonance_current = 0;
    let enemy_hp = core.units[&enemy].stats.current_health;
    let block = core.units[&a].block_capacity;
    core.step_battle_execution_until(&mut execution, 11).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 960, "two segments each lose 70 HP");
    assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 35, "only first positive segment reflects");
    assert_eq!(core.units[&a].block_capacity, block);
    assert!(!core.channel_active(a));
    let facts = core.runtime_fact_summary();
    assert_eq!(facts.damage_hp_lost, 175);
    let snapshot = core.sustained_effect_snapshot().unwrap();
    assert_eq!(snapshot.passive_guards.len(), 2);
    assert_eq!(snapshot.started, 0);
    let guard = facts.latest_damage.as_ref().unwrap().protection.passive_guard.as_ref().unwrap();
    assert_eq!(guard.roll_percent, None, "second segment cannot roll again");
    assert_eq!(guard.source.fragment.as_ref().unwrap().copy_uuid, Uuid::from_u128(0x1200));
    core.step_battle_execution_until(&mut execution, 21).unwrap();
    let reflected = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
    assert_eq!(reflected.source.fragment.as_ref().unwrap().copy_uuid, Uuid::from_u128(0x1200));
    assert!(reflected.source.reaction && reflected.source.cast.is_none());
    assert_eq!(reflected.target, enemy);
    assert!(reflected.protection.passive_guard.is_none());
}
#[test]
fn r12_passive_guard_miss_half_open_window_removes_only_own_reduction() {
    use super::super::sim::BattleLiveCommand;
    let (mut core, mut execution, a, _) = r12_guard_core(5, 0, false);
    core.units.get_mut(&a).unwrap().incoming_damage_modifiers.damage_reduction_percent = 20;
    core.units.get_mut(&a).unwrap().stats.defense = 42;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 11).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1020);
    let fact = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
    assert!(fact.protection.passive_guard.as_ref().unwrap().reduction_suppressed);
    core.step_battle_execution_until(&mut execution, 21).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 940);
    core.step_battle_execution_until(&mut execution, 31).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 890);
    assert!(!core.runtime_fact_summary().latest_damage.as_ref().unwrap().protection.passive_guard.as_ref().unwrap().reduction_suppressed);
    assert_eq!(core.units[&a].stats.defense, 42);
    assert_eq!(core.units[&a].incoming_damage_modifiers.damage_reduction_percent, 20);
}
#[test]
fn r12_passive_guard_success_and_invulnerability_never_reflect() {
    use super::super::sim::BattleLiveCommand;
    let (mut core, mut execution, a, enemy) = r12_guard_core(30, 100, true);
    let enemy_hp = core.units[&enemy].stats.current_health;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 31).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1100);
    assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 100);
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 100);
    let fact = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
    assert!(fact.protection.passive_guard.as_ref().unwrap().roll_percent.is_none());
    assert!(fact.protection.deflection.is_some());
    assert!(core.sustained_effect_snapshot().unwrap().passive_guards.iter().all(|g| g.reduction_suppressed_intervals_ms.is_empty()));
}
#[test]
fn r12_passive_guard_reflection_failure_rolls_back_damage_and_exact_replay() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["passive_guard_receipt_source", "passive_guard_receipt_storage", "passive_guard_after_reflection"] {
        let (mut core, mut execution, a, enemy) = r12_guard_core(5, 100, false);
        core.step_battle_execution_until(&mut execution, 10).unwrap();
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}"));
        let enemy_hp = core.units[&enemy].stats.current_health;
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.step_battle_execution_until(&mut execution, 11)));
        assert!(token.is_cancelled() && result.is_err(), "{site}: {result:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}")), before);
        core.step_battle_execution_until(&mut execution, 11).unwrap();
        assert_eq!(core.units[&a].stats.current_health, 1030);
        assert_eq!(core.units[&enemy].stats.current_health, enemy_hp - 35);
    }
}

#[test]
fn r12_shield_zero_actual_hp_result_expression_cannot_reuse_absorbed_damage() {
    use super::super::sim::BattleLiveCommand;
    for (shield, expected) in [(200, 0), (50, 50)] {
        let authored = r12_shield_ron(false, &format!("Const((value: {shield}, unit: Magnitude))")).replace("Damage(amount: 800", "Damage(amount: 100").replacen("Damage(amount: 100, damage_type: True))]", r#"Damage(amount: 100, damage_type: True)), (id: "actual_only", op: HealExpr(amount: (moment: Live, root: Read(Result(selector: EarlierEffect("hit"), field: DamageHpLost)))))]"#, 1);
        let (mut core, mut execution, a, _) = r12_configured_shield_core(&authored, |_| {});
        core.enable_authoring_observation_with_options(Default::default(), true);
        core.bind_authoring_observation(crate::game::battle::core::authoring_observation::ObservationKey { entry_ordinal: 1, battle_uuid: Uuid::from_u128(0x1212) }).unwrap();
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 11).unwrap();
        let facts = core.runtime_fact_summary();
        assert_eq!(facts.damage_hp_lost, expected);
        assert_eq!(facts.healing_delta, expected as i64);
        assert_eq!(core.units[&a].stats.current_health, 1100);
        let hit = facts.latest_damage.as_ref().unwrap();
        assert_eq!((hit.protection.mitigated_damage, hit.protection.absorbed, hit.actual_hp_loss()), (100, 100 - expected as u32, expected as u32));
        assert_eq!(core.authoring_observation().unwrap().unwrap().latest_amount.as_ref().unwrap().evaluated.value.value, expected as i128);
    }
}
#[test]
fn r12_passive_guard_original_attacker_gone_never_retargets_counter_or_reflection() {
    use super::super::sim::BattleLiveCommand;
    for parry in [false, true] {
        let (mut core, mut execution, a, enemy) = r12_guard_core_custom(100, 100, false, |s| {
            let s = s.replace("delay_ms: 10, target: CastTarget, delivery: Instant", "delay_ms: 0, target: CastTarget, delivery: Projectile(speed_units_per_ms: 100000, hit_policy: TargetLocked)");
            r12_append_step(&s, "r08_strike", r#"SkillStepDef(id: "attacker_exit", delay_ms: 1, target: SelfUnit, delivery: Instant, effects: [(id: "exit_cost", op: HealthCost(amount: 1000, allow_lethal: true))])"#)
        }, |scenario| {
            let mut second = scenario.groups[0].spawns[0].clone();
            second.unit_ref = ScenarioUnitRef::new("guard_second_enemy"); second.instance_salt = 22;
            second.draft = fixture_draft(22, 10000, 0); second.position = Position::new(7, 3);
            scenario.groups[0].spawns.push(second);
        });
        if parry { core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap(); }
        core.step_battle_execution_until(&mut execution, 100).unwrap();
        assert!(core.units.get(&enemy).is_none_or(|u| !u.is_active()));
        let fact = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
        assert_eq!(fact.target, a);
        if parry {
            assert_eq!(fact.actual_hp_loss(), 0);
            assert_eq!(fact.protection.deflection.as_ref().unwrap().counter, crate::game::battle::runtime_facts::CounterOutcome::AttackerUnavailable);
        } else {
            assert_eq!(fact.actual_hp_loss(), 70);
            assert_eq!(fact.protection.passive_guard.as_ref().unwrap().reflection, crate::game::battle::runtime_facts::ReflectionOutcome::AttackerUnavailable);
        }
    }
}
#[test]
fn r12_passive_guard_health_cost_bypasses_protection_and_self_damage_request_rejects() {
    use super::super::sim::BattleLiveCommand;
    for self_damage in [false, true] {
        let (mut core, mut execution, a, enemy) = r12_guard_core_custom(30, 100, false, |s| {
            let op = if self_damage { "Damage(amount: 100, damage_type: True)" } else { "HealthCost(amount: 100, allow_lethal: false)" };
            r12_append_step(&s, "r11_aura", &format!(r#"SkillStepDef(id: "self", delay_ms: 0, target: SelfUnit, delivery: Instant, effects: [(id: "self_op", op: {op})])"#))
        }, |_| {});
        let enemy_hp = core.units[&enemy].stats.current_health;
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot());
        let result = core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 });
        if self_damage {
            assert!(matches!(result, Err(GameError::ActionUnavailable { reason: crate::game::behavior::ActionDisabledReasonCode::NoValidTarget, .. })));
            assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot()), before);
        } else {
            result.unwrap();
            core.step_battle_execution_until(&mut execution, 1).unwrap();
            assert_eq!(core.units[&a].stats.current_health, 1000);
            assert_eq!(core.units[&enemy].stats.current_health, enemy_hp);
            assert!(core.runtime_fact_summary().latest_damage.is_none());
            assert_eq!(core.runtime_fact_summary().latest_cost.as_ref().unwrap().actual_spent(), 100);
        }
    }
}

#[test]
fn r12_native_protection_limits_are_hard_and_retry_restores_same_result() {
    use super::super::sim::BattleLiveCommand;
    use crate::game::battle::runtime_control::{RuntimeFactLimits, RuntimeFaultCode};
    for (limits, code) in [
        (RuntimeFactLimits { max_work: 1, ..Default::default() }, RuntimeFaultCode::WorkLimit),
        (RuntimeFactLimits { max_clone_slots: 1, ..Default::default() }, RuntimeFaultCode::CloneLimit),
        (RuntimeFactLimits { max_retained_bytes: 1, ..Default::default() }, RuntimeFaultCode::RetainedByteLimit),
    ] {
        let (mut core, mut execution, a, _) = r12_shield_core(false, "Const((value: 600, unit: Magnitude))");
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 10).unwrap();
        core.runtime_facts.limits = limits;
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}"));
        assert!(matches!(core.step_battle_execution_until(&mut execution, 11), Err(GameError::BattleRuntimeFault(f)) if f.code == code));
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), format!("{execution:?}")), before);
        core.runtime_facts.limits = RuntimeFactLimits::default();
        core.step_battle_execution_until(&mut execution, 11).unwrap();
        assert_eq!(core.units[&a].stats.current_health, 900);
        assert_eq!(core.runtime_fact_summary().latest_damage.as_ref().unwrap().protection.absorbed, 600);
    }
}

#[test]
fn r12_shield_initial_scale_once_membership_recast_and_old_expiry() {
    use super::super::sim::{BattleLiveCommand, BattleLiveCommandOutcome};
    use crate::game::battle::types::TacticalTileKind;
    let expression = "Mul(value: Const((value: 600, unit: Magnitude)), scalar: Ratio(numerator: Read(Stat(subject: Source, stat: Attack)), denominator: Const((value: 100, unit: Magnitude)), rounding: Floor))";
    let authored = r12_shield_ron(true, expression).replace("duration_ms: 1000", "duration_ms: 100").replace("Damage(amount: 800", "Damage(amount: 100").replace("Damage(amount: 700", "Damage(amount: 100");
    let authored = r12_append_step(&authored, "r11_aura", r#"SkillStepDef(id: "grow", delay_ms: 30, target: SelfUnit, delivery: Instant, effects: [(id: "attack", op: ModifyStats(modifier: (stat: Attack, kind: Flat, value: 100))), (id: "refill", op: ModifyResonance(amount: 10000))])"#);
    let (mut core, mut execution, a, _) = r12_configured_shield_core(&authored, |scenario| {
        let BattleUnitSource::Employee(profile) = &mut scenario.groups[1].spawns[0].draft.source else { unreachable!() };
        profile.stats.attack = 200;
        profile.resonance.gain_lock_ms = 0;
    });
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 11).unwrap();
    let pool = core.sustained_effect_snapshot().unwrap().active[0].shield.unwrap();
    assert_eq!((pool.capacity, pool.remaining), (1200, 1100));
    let mut draft = core.scenario.groups[1].spawns[0].draft.clone(); draft.owned_uuid = Uuid::from_u128(0x1234);
    for salt in 0..2 {
        let BattleLiveCommandOutcome::UnitDeployed { unit_id } = core.apply_live_command(BattleLiveCommand::DeployPlayerUnit { draft: draft.clone(), position: Position::new(2, 1), tile_kind: TacticalTileKind::Ground, facing: FacingDirection::Right, instance_salt: 0x1234 + salt, time_ms: 11, current_hp_policy: None }).unwrap() else { unreachable!() };
        for _ in 0..2 {
            let view = core.sustained_effect_snapshot().unwrap();
            assert_eq!(view.active[0].members.len(), 3);
            assert_eq!(view.active[0].shield.unwrap(), pool);
        }
        core.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id, time_ms: 11 }).unwrap();
        assert_eq!(core.sustained_effect_snapshot().unwrap().active[0].shield.unwrap(), pool);
    }
    core.step_battle_execution_until(&mut execution, 51).unwrap();
    assert_eq!(core.units[&a].stats.attack, 300);
    assert_eq!(core.sustained_effect_snapshot().unwrap().active[0].shield.unwrap().capacity, 1200);
    assert_eq!(core.units[&a].resonance_current, 0, "refill inside the active use was discarded");
    assert!(core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 51 }).is_err());
    let enemy = core.units.values().find(|unit| unit.owner == Side::Opponent).unwrap().instance_id;
    let remaining = core.sustained_effect_snapshot().unwrap().active[0].shield.unwrap().remaining;
    let damage = core.damage_source_snapshot_for_unit(enemy, a, crate::game::battle::damage::DamageSource::Ability,
        crate::game::battle::damage::DamageType::True, remaining, Default::default(), 0, 51, false).unwrap();
    core.process_commands_with_modifier_source(vec![crate::game::battle::damage::BattleCommand::ApplyDamage { target_id: a, source_snapshot: damage }], 51, Some(enemy));
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    core.add_resonance(a, 100, 51, false);
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 51 }).unwrap();
    core.step_battle_execution_until(&mut execution, 52).unwrap();
    let view = core.sustained_effect_snapshot().unwrap();
    assert_eq!(view.active.len(), 1);
    assert_eq!(view.active[0].shield.unwrap().capacity, 1800);
    let new_id = view.active[0].activation;
    core.step_battle_execution_until(&mut execution, 101).unwrap();
    let view = core.sustained_effect_snapshot().unwrap();
    assert_eq!(view.ended, 1);
    assert_eq!(view.active.len(), 1);
    assert_eq!(view.active[0].activation, new_id);
    assert_eq!(view.active[0].shield.unwrap().remaining, 1800);
    core.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: a, time_ms: 101 }).unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().ended, 2);
    core.step_battle_execution_until(&mut execution, 152).unwrap();
    let view = core.sustained_effect_snapshot().unwrap();
    assert_eq!(view.ended, 2);
    assert!(view.active.is_empty());
}

#[test]
fn r12_same_timestamp_area_shield_consumption_is_stable_under_spawn_order() {
    use super::super::sim::BattleLiveCommand;
    for reverse in [false, true] {
        let authored = r12_shield_ron(true, "Const((value: 600, unit: Magnitude))").replace("id: \"first\", delay_ms: 10, target: CastTarget, delivery: Instant", r#"id: "first", delay_ms: 10, target: CastTarget, delivery: TileArea(footprint: (include_anchor_tile: true, rows: ["XXXXX", "XXXXX", "XX@XX", "XXXXX", "XXXXX"]), area: (anchor: CastTarget, tile_origin: Anchor))"#);
        let (mut core, mut execution, a, b) = r12_configured_shield_core(&authored, |scenario| { if reverse { scenario.groups[1].spawns.reverse(); } });
        let mut targets = [a, b]; targets.sort();
        for id in targets { core.units.get_mut(&id).unwrap().incoming_damage_modifiers.damage_reduction_percent = 50; }
        let first_hp = core.units[&targets[0]].stats.current_health;
        let second_hp = core.units[&targets[1]].stats.current_health;
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 11).unwrap();
        assert_eq!(core.units[&targets[0]].stats.current_health, first_hp);
        assert_eq!(core.units[&targets[1]].stats.current_health, second_hp - 200);
        assert_eq!(core.runtime_fact_summary().damage_hp_lost, 200);
        assert_eq!(core.sustained_effect_snapshot().unwrap().ended, 1);
    }
}

#[test]
fn r12_passive_guard_miss_and_recovery_equal_timestamp_boundaries() {
    use super::super::sim::BattleLiveCommand;
    let (mut core, mut execution, a, _) = r12_guard_core_custom(5, 0, false,
        |s| s.replace("delay_ms: 10", "delay_ms: 5").replace("delay_ms: 20", "delay_ms: 25"), |_| {});
    core.units.get_mut(&a).unwrap().incoming_damage_modifiers.damage_reduction_percent = 20;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 5).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1100);
    core.step_battle_execution_until(&mut execution, 6).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 1020, "window end is already a miss");
    core.step_battle_execution_until(&mut execution, 26).unwrap();
    assert_eq!(core.units[&a].stats.current_health, 970, "recovery end restores own30 plus other20");
}

#[test]
fn r12_parry_basic_on_hit_commands_are_suppressed_before_dispatch() {
    use super::super::sim::BattleLiveCommand;
    for window in [5, 2000] {
        let authored = r12_parry_ron(window, false).replace("duration_ms: 200,", "duration_ms: 3000,").replace("invulnerable_ms: 40", "invulnerable_ms: 900");
        let (mut core, mut execution, _, _) = r11_equipped_effect_core(&authored, r12_ranged_enemy, |weapon| { weapon.triggered_effects = ron::de::from_str(r#"{OnHit: [(target: SelfUnit, effect: Modifier((stat: Attack, kind: Flat, value: 7)))]}"#).unwrap(); });
        // Resolve the real first basic target before arming that employee.
        let mut probe = core.clone(); let mut probe_execution = execution.clone();
        probe.step_battle_execution_until(&mut probe_execution, 1500).unwrap();
        let target = probe.event_log.entries.iter().find_map(|e| match &e.event {
            crate::game::battle::event_log::BattleLogEvent::HpChanged { target_instance_id, reason: crate::game::battle::event_log::HpChangeReason::BasicAttack, source_instance_id: Some(source), .. } if probe.units.get(source).is_some_and(|u| u.owner == Side::Opponent) => Some(*target_instance_id), _ => None,
        }).unwrap();
        // The opening basic hit at t1 precedes this cast. Parry starts at t2
        // and can suppress only the next actual hit, never retroactively.
        core.step_battle_execution_until(&mut execution, 1).unwrap();
        let attack_before = core.units[&target].stats.attack;
        let hp_before = core.units[&target].stats.current_health;
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: target, skill_id: "r11_aura".into(), target: None, time_ms: 1 }).unwrap();
        core.step_battle_execution_until(&mut execution, 1500).unwrap();
        if window == 2000 {
            assert_eq!(core.units[&target].stats.attack, attack_before);
            assert_eq!(core.units[&target].stats.current_health, hp_before);
            assert!(core.runtime_fact_summary().latest_damage.as_ref().unwrap().protection.deflection.is_some() || core.runtime_fact_summary().latest_damage.as_ref().unwrap().source.reaction);
        } else { assert!(core.units[&target].stats.attack > attack_before); }
    }
}

#[test]
fn r12_corrosion_bypasses_enemy_shield_floor_parry_with_live_boss_flow_and_channel() {
    use super::super::sim::BattleLiveCommand;
    let (mut core, mut execution, a, enemy) = r12_guard_core_custom(30, 100, false, |s| {
        let s = s.replacen("steps: [SkillStepDef(id: \"shield\"", "flow: Some(Action(id: \"guard_flow\", step_id: \"shield\", wait: EffectsSettled)), steps: [SkillStepDef(id: \"shield\"", 1);
        let s = s.replacen("counter_skill: \"r12_counter\")))]", r#"counter_skill: "r12_counter"))),
          (id: "shared", op: Sustained(definition: Shield(duration_ms: 1000, amount: 600, priority: 0, shared_radius_units: Some(2000000)))),
          (id: "channel", op: Sustained(definition: ChannelAura(duration_ms: 1000, aura: (family: "cross_wp", priority: 0, radius_units: 2000000, basic_damage_percent: 20, basic_speed_points: 15))))]"#, 1);
        let s = r12_add_use_effects(&s, "r11_aura", &[("shield", "shared"), ("shield", "channel")]);
        let s = r12_append_step(&s, "r08_strike", r#"SkillStepDef(id: "enemy_guard", delay_ms: 0, target: SelfUnit, delivery: Instant, effects: [
          (id: "shield", op: Sustained(definition: Shield(duration_ms: 1000, amount: 10000, priority: 0, shared_radius_units: None))),
          (id: "floor", op: Sustained(definition: Survival(duration_ms: 1000, policy: FloorOne))),
          (id: "parry", op: Sustained(definition: Parry(duration_ms: 200, window_ms: 30, invulnerable_ms: 40, miss_recovery_ms: 20, reduction_percent: 0, counter_skill: "r12_counter")))])"#);
        r12_add_use_effects(&s, "r08_strike", &[("enemy_guard", "shield"), ("enemy_guard", "floor"), ("enemy_guard", "parry")])
    }, |scenario| { scenario.groups[1].spawns[1].draft.employee_start_condition.as_mut().unwrap().trauma = 80; });
    let b = core.units.values().find(|u| u.owner == Side::Player && u.instance_id != a).unwrap().instance_id;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: a, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    let before = core.sustained_effect_snapshot().unwrap();
    assert!(core.channel_active(a));
    assert!(before.active.iter().any(|v| v.caster == a && v.shield.is_some()));
    assert_eq!(before.active.iter().filter(|v| v.caster == enemy).count(), 3);
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: b, skill_id: "r11_aura".into(), target: None, time_ms: 1 }).unwrap();
    core.step_battle_execution_until(&mut execution, 2).unwrap();
    let facts = core.runtime_fact_summary();
    assert_eq!(facts.latest_death.as_ref().unwrap().cause, crate::game::battle::runtime_facts::DeathCause::Corrosion);
    assert!(facts.latest_death.as_ref().unwrap().lethal_damage.is_none());
    assert_eq!(facts.damage_hp_lost, 0);
    assert!(core.units.get(&enemy).is_none_or(|u| !u.is_active()));
    assert!(core.units[&b].corroded);
    assert_eq!(core.units[&b].employee_trauma, Some(100));
    assert_eq!(core.forced_pause_remaining_ms(), Some(1000));
    assert!(core.sustained_effect_snapshot().unwrap().active.iter().all(|v| v.caster != enemy));
    core.advance_forced_pause_host_elapsed(1000).unwrap();
    let outcome = core.step_battle_execution_until(&mut execution, 100).unwrap();
    assert!(matches!(outcome, super::super::sim::BattleStepOutcome::Finished(_)));
    core.finalize_battle_execution(&mut execution).unwrap();
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
}

#[test]
fn r12_shielded_employee_attack_grants_no_charge_and_lifesteals_only_hp_loss() {
    use super::super::sim::BattleLiveCommand;
    for shield in [1, 1000] {
        let mut authored = r12_with_initial_health_cost(&r11_overlay_ron("Instant", 2000, false));
        let enemy = format!(r#", SkillDef(id: "r08_strike", name: "Enemy shield", kind: Targeted, focus_time_ms: 0,
          use_lifetime: ActionAndEffects(effects: [(step: "protect", effect: "shield")]),
          cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: true),
          steps: [SkillStepDef(id: "protect", delay_ms: 0, target: SelfUnit, delivery: Instant,
            effects: [(id: "shield", op: Sustained(definition: Shield(duration_ms: 5000, amount: {shield}, priority: 0, shared_radius_units: None)))])])"#);
        let end = authored.rfind("])").unwrap(); authored.insert_str(end, &enemy);
        let boss = r08_core(R08_BOSS, R08_SKILLS).scenario.boss_runtime.clone();
        let (mut core, mut execution, caster, _) = r11_equipped_effect_core(&authored, |scenario| { scenario.boss_runtime = boss; }, |weapon| r12_lifesteal_weapon(weapon, 100));
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 1400).unwrap();
        let loss = if shield == 1 { 99 } else { 0 };
        assert_eq!(core.runtime_fact_summary().damage_hp_lost, loss);
        assert_eq!(core.units[&caster].stats.current_health, 500 + loss as u32 / 4);
        assert_eq!(core.runtime_fact_summary().heal_mutations, u64::from(loss > 0));
        let damage = core.runtime_fact_summary().latest_damage.as_ref().unwrap();
        assert_eq!(damage.actual_hp_loss(), loss as u32);
        assert_eq!(damage.protection.absorbed, 100 - loss as u32);
        assert_eq!(core.units[&caster].resonance_current, 0, "active timed overlay blocks the global pulse as well as all attack gains");
    }
}


#[test]
fn r11_channel_excludes_actual_field_work_and_starts_after_stop() {
    use super::super::sim::BattleLiveCommand;
    use crate::game::battle::{scenario::{FieldWorkCellDef, FieldWorkCellId, FieldWorkObjectDef, FieldWorkObjectId, FieldWorkPresentationKind, FieldWorkCompletionEffect}, types::TacticalTileKind};
    let (mut core, mut execution, _, _) = r11_configured_effect_core(R11_AURA, |scenario| {
        scenario.groups[0].spawns[0].draft = fixture_draft(1, 100000, 0);
        scenario.tactical_plan.field_work_objects.push(FieldWorkObjectDef {
            id: FieldWorkObjectId::new("channel_work"), presentation_kind: FieldWorkPresentationKind::EmergencyRepairTerminal,
            position: Position::new(1, 1), required_work_ms: 1000000,
            completion_effect: FieldWorkCompletionEffect::RepairProtectedUnit { unit_ref: ScenarioUnitRef::new("player_0"), flat_hp: 1 },
            work_cells: vec![FieldWorkCellDef { id: FieldWorkCellId::new("worker"), position: Position::new(1, 1), tile_kind: TacticalTileKind::Ground }, FieldWorkCellDef { id: FieldWorkCellId::new("second"), position: Position::new(1, 2), tile_kind: TacticalTileKind::Ground }],
        });
    });
    let caster = core.units.values().find(|u| u.owner == Side::Player && u.body.projected_tile() == Position::new(1, 1)).unwrap().instance_id;
    let member = core.units.values().find(|u| u.owner == Side::Player && u.instance_id != caster).unwrap().instance_id;
    // Scenario employee spawns have no tactical state. Normal deployment creates
    // the worker capability before channel activation.
    let mut draft = core.scenario.groups[1].spawns[0].draft.clone();
    core.apply_live_command(BattleLiveCommand::WithdrawUnit { unit_id: caster, time_ms: 0 }).unwrap();
    draft.owned_uuid = Uuid::from_u128(0x11f1);
    let BattleUnitSource::Employee(profile) = &mut draft.source else { unreachable!() };
    profile.resonance.start = profile.resonance.max;
    let super::super::sim::BattleLiveCommandOutcome::UnitDeployed { unit_id: worker } = core.apply_live_command(BattleLiveCommand::DeployPlayerUnit { draft, position: Position::new(2, 1), tile_kind: TacticalTileKind::Ground, facing: FacingDirection::Right, instance_salt: 0x11f1, time_ms: 0, current_hp_policy: None }).unwrap() else { unreachable!() };
    assert!(!core.unit_is_field_working(worker));
    let full_at = core.tactical_states[&worker].stance_cooldown_until_ms.max(core.tactical_states[&worker].relocation_cooldown_until_ms).max(1);
    core.step_battle_execution_until(&mut execution, full_at).unwrap();
    assert!(core.units[&member].is_active());
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: worker, skill_id: "r11_aura".into(), target: None, time_ms: full_at }).unwrap();
    core.step_battle_execution_until(&mut execution, full_at + 1).unwrap();
    assert!(core.channel_active(worker));
    assert!(!core.unit_is_field_working(worker));
    let before = core.transaction_debug_snapshot();
    assert!(core.apply_live_command(BattleLiveCommand::TacticalRelocateUnit { unit_id: worker, destination: Position::new(1, 1), tile_kind: TacticalTileKind::Ground, facing: FacingDirection::Right, time_ms: full_at + 1 }).is_err());
    assert_eq!(core.transaction_debug_snapshot(), before);
    let work = core.field_work_object_snapshots()[0].accumulated_work_ms;
    core.step_battle_execution_until(&mut execution, full_at + 500).unwrap();
    assert_eq!(core.field_work_object_snapshots()[0].accumulated_work_ms, work);
    assert!(!core.event_log.entries.iter().any(|e| e.time_ms > full_at + 1 && e.time_ms <= full_at + 500 && matches!(e.event, BattleLogEvent::HpChanged { source_instance_id: Some(source), reason: crate::game::battle::event_log::HpChangeReason::BasicAttack, .. } if source == worker)));
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: worker, skill_id: "r11_aura".into(), target: None, time_ms: full_at + 500 }).unwrap();
    assert!(!core.channel_active(worker));
    core.apply_live_command(BattleLiveCommand::SetEngagementStance { unit_id: worker, stance: crate::game::battle::types::EngagementStance::Defense, time_ms: full_at + 500 }).unwrap();
    core.apply_live_command(BattleLiveCommand::TacticalRelocateUnit { unit_id: worker, destination: Position::new(1, 1), tile_kind: TacticalTileKind::Ground, facing: FacingDirection::Right, time_ms: full_at + 500 }).unwrap();
    assert!(core.unit_is_field_working(worker));
    core.step_battle_execution_until(&mut execution, full_at + 600).unwrap();
    assert!(core.field_work_object_snapshots()[0].accumulated_work_ms > work);
}

#[test]
fn r11_channel_actual_enemy_lethal_hit_clears_only_dead_source() {
    use super::super::sim::BattleLiveCommand;
    let (mut core, mut execution, _, _) = r11_configured_effect_core(R11_AURA, |scenario| {
        r12_ranged_enemy(scenario);
        let BattleUnitSource::TestFixture { profile, .. } = &mut scenario.groups[0].spawns[0].draft.source else { unreachable!() };
        profile.basic_attack.windup_ms = 100;
        profile.basic_attack.damage_type = crate::game::battle::damage::DamageType::True;
        let BattleUnitSource::Employee(profile) = &mut scenario.groups[1].spawns[0].draft.source else { unreachable!() };
        profile.stats.max_health = 40; profile.stats.current_health = 40;
    });
    let caster = core.units.values().find(|u| u.owner == Side::Player && u.stats.max_health == 40).unwrap().instance_id;
    let member = core.units.values().find(|u| u.owner == Side::Player && u.instance_id != caster).unwrap().instance_id;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1).unwrap();
    assert!(core.channel_active(caster)); assert_eq!(core.aura_basic_bonus(member), (20, 15));
    core.step_battle_execution_until(&mut execution, 100).unwrap();
    assert!(!core.units[&caster].is_active());
    assert!(core.units[&member].is_active());
    assert!(!core.channel_active(caster)); assert_eq!(core.aura_basic_bonus(member), (0, 0));
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert_eq!(core.sustained_effect_snapshot().unwrap().ended, 1);
}

#[test]
fn r11_overlay_compatible_weapons_preserve_type_identity_lifesteal_and_cycle() {
    use super::super::sim::BattleLiveCommand;
    use crate::game::battle::damage::DamageType;
    for (i, damage_type) in [DamageType::Physical, DamageType::Magic, DamageType::True].into_iter().enumerate() {
        let weapon_uuid = Uuid::from_u128(0x11c0 + i as u128);
        let authored = r12_with_initial_health_cost(&r11_overlay_ron("Instant", 1500, true));
        let (mut core, mut execution, caster, _) = r11_equipped_effect_core(&authored, |scenario| {
            for spawn in &mut scenario.groups[1].spawns {
                let BattleUnitSource::Employee(profile) = &mut spawn.draft.source else { unreachable!() };
                profile.stats.spell_power = 100;
            }
        }, |weapon| {
            weapon.uuid = weapon_uuid; weapon.id = format!("compatible_{i}");
            weapon.weapon_profile.as_mut().unwrap().damage_type = damage_type;
            r12_lifesteal_weapon(weapon, 100);
        });
        let original = serde_json::to_value(&core.units[&caster].basic_attack).unwrap();
        core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
        core.step_battle_execution_until(&mut execution, 1400).unwrap();
        let facts = core.runtime_fact_summary();
        assert_eq!(facts.damage_hp_lost, 100);
        assert_eq!(facts.healing_delta, 25);
        let heal = facts.latest_heal.as_ref().unwrap();
        assert_eq!(heal.source.equipment.unwrap().definition_uuid, weapon_uuid);
        assert!(heal.source.cast.is_none());
        assert!(core.event_log.entries.iter().any(|e| matches!(e.event, BattleLogEvent::HpChanged { source_instance_id: Some(source), reason: crate::game::battle::event_log::HpChangeReason::BasicAttack, damage_type: Some(kind), delta: -100, .. } if source == caster && kind == damage_type)));
        assert_eq!(serde_json::to_value(&core.units[&caster].basic_attack).unwrap(), original);
        assert!((2001..2100).contains(&core.units[&caster].next_basic_attack_ms));
        assert_eq!(core.sustained_effect_snapshot().unwrap().ended, 1);
    }
}


#[test]
fn r11_channel_blocks_real_periodic_attacks_and_stop_resumes_them() {
    use super::super::sim::BattleLiveCommand;
    let authored = R11_AURA.replace("duration_ms: 1000", "duration_ms: 10000");
    let make = || r11_configured_effect_core(&authored, |scenario| {
        scenario.groups[0].spawns[0].position = Position::new(2, 1);
        scenario.groups[0].spawns[0].draft = fixture_draft(1, 10000, 0);
    });
    let (mut plain, mut plain_execution, _, _) = make();
    let (mut core, mut execution, _, _) = make();
    let caster = core.units.values().find(|u| u.owner == Side::Player && u.body.projected_tile() == Position::new(1, 1)).unwrap().instance_id;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    plain.step_battle_execution_until(&mut plain_execution, 2500).unwrap();
    core.step_battle_execution_until(&mut execution, 2500).unwrap();
    let starts = |core: &BattleCore, after| core.event_log.entries.iter().filter(|e| e.time_ms > after && matches!(e.event, BattleLogEvent::AttackStart { attacker_instance_id, .. } if attacker_instance_id == caster)).count();
    assert!(starts(&plain, 1) >= 2, "control actually attacks on the periodic schedule");
    assert_eq!(starts(&core, 1), 0);
    assert!(core.channel_active(caster));
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 2500 }).unwrap();
    core.step_battle_execution_until(&mut execution, 2501).unwrap();
    assert_eq!(starts(&core, 2499), 1, "stop resumes one attack without duplicates");
    assert!(core.event_log.entries.iter().any(|e| e.time_ms >= 2500 && matches!(e.event, BattleLogEvent::HpChanged { source_instance_id: Some(source), reason: crate::game::battle::event_log::HpChangeReason::BasicAttack, delta: -100, .. } if source == caster)));
}

#[test]
fn r11_aura_expiry_keeps_already_released_projectile_damage_snapshot() {
    use super::super::sim::BattleLiveCommand;
    let authored = R11_AURA.replace("duration_ms: 1000", "duration_ms: 1100");
    let (mut core, mut execution, _, _) = r11_equipped_effect_core(&authored, |scenario| {
        scenario.groups[0].spawns[0].draft = fixture_draft(1, 10000, 0);
    }, |weapon| {
        let profile = weapon.weapon_profile.as_mut().unwrap();
        profile.range_role = crate::game::data::equipment_data::WeaponRangeRole::Ranged;
        profile.ranged_reposition_ms = 0;
        profile.defense_tile_range = ron::de::from_str(r#"(include_anchor_tile: true, rows: ["XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXX@XXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX", "XXXXXXXXXXXXXXX"])"#).unwrap();
        profile.delivery = ron::de::from_str("Projectile(speed_units_per_ms: 5000)").unwrap();
    });
    let caster = core.units.values().find(|u| u.owner == Side::Player && u.body.projected_tile() == Position::new(1, 2)).unwrap().instance_id;
    let member = core.units.values().find(|u| u.owner == Side::Player && u.instance_id != caster).unwrap().instance_id;
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 1002).unwrap();
    assert_eq!(core.aura_basic_bonus(member), (20, 15));
    assert!(core.event_log.entries.iter().any(|e| e.time_ms >= 1000 && matches!(e.event, BattleLogEvent::BasicAttackProjectileLaunched { attacker_instance_id, .. } if attacker_instance_id == member)));
    core.step_battle_execution_until(&mut execution, 1101).unwrap();
    assert_eq!(core.aura_basic_bonus(member), (0, 0));
    core.step_battle_execution_until(&mut execution, 2300).unwrap();
    assert!(core.event_log.entries.iter().any(|e| e.time_ms > 1101 && matches!(e.event, BattleLogEvent::HpChanged { source_instance_id: Some(source), reason: crate::game::battle::event_log::HpChangeReason::BasicAttack, delta: -120, .. } if source == member)), "released 20% bonus survives source expiry");
}

#[test]
fn native_u07_reinput_wrapper_late_fault_restores_consumed_activation_cast_queue_and_retry() {
    use super::super::sim::BattleLiveCommand;
    use crate::game::battle::runtime_control as native;
    let (mut core, mut execution, caster, _) = r11_employee_effect_core(&r11_hello_goodbye_ron("Live"));
    core.apply_live_command(BattleLiveCommand::ActivateSkill { unit_id: caster, skill_id: "r11_aura".into(), target: None, time_ms: 0 }).unwrap();
    core.step_battle_execution_until(&mut execution, 501).unwrap();
    let pools = core.sustained_effect_snapshot().unwrap();
    let activation = pools.active[0].activation;
    let command = BattleLiveCommand::ReinputSustainedSkill { unit_id: caster, skill_id: "r11_aura".into(), activation, target: None, time_ms: 501 };
    let mut probe = core.clone();
    let (success, admissions) = native::observe_retained_admissions(|| probe.apply_live_command(command.clone()));
    success.unwrap();
    assert_eq!(probe.runtime_fact_summary().damage_hp_lost, 150);
    let wrappers: Vec<_> = admissions.iter().filter(|(site, _, _)| *site == "extended_source_observation").collect();
    assert_eq!(wrappers.len(), 3, "reinput owner, accepted child, released child");
    core.runtime_facts.limits.max_retained_bytes = wrappers[2].2 - 1;
    let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), core.skill_cast_seq, core.runtime_fact_source.clone());
    let failure = core.apply_live_command(command.clone()).unwrap_err();
    assert!(matches!(failure, GameError::BattleRuntimeFault(native::BattleRuntimeFault { code: native::RuntimeFaultCode::RetainedByteLimit, site: "extended_source_observation", .. })), "{failure:?}");
    assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), core.skill_cast_seq, core.runtime_fact_source.clone()), before);
    assert_eq!(core.sustained_effect_snapshot().unwrap(), pools);
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0);
    core.runtime_facts.limits = Default::default();
    core.apply_live_command(command).unwrap();
    assert_eq!(core.runtime_fact_summary().damage_hp_lost, 150);
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
}
