use crate::{
    authoring::node_graph::{compile, tests::state_fixture, PortableGraph},
    game::{
        ability::sustained::{SustainedEffectDef, TargetStateProfile},
        battle::{
            core::{
                relic_causal_tests::joint_committed_damage, sim::BattleExecutionState,
                tests::native_action_fixture, BattleCore,
            },
            ids::UnitInstanceId,
            types::BattleUnitSourceIdentity,
        },
    },
};

fn fixture() -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let graph = compile(&state_fixture().to_string(), "state_test".into()).unwrap();
    let graph = PortableGraph::from_ron(&graph.to_ron().unwrap()).unwrap();
    let db = graph.load_native().unwrap();
    let (mut core, state, actor, enemy, ally) = native_action_fixture(
        db.get_by_id("state_test").unwrap().clone(),
        Default::default(),
    );
    let unit = core.units.get_mut(&actor).unwrap();
    unit.source_identity = BattleUnitSourceIdentity::Employee {
        employee_uuid: actor.as_uuid(),
        base_uuid: unit.base_uuid,
    };
    unit.resonance_lock_ms = 0;
    unit.stats.attack = 100;
    unit.stats.defense = 100;
    (core, state, actor, enemy, ally)
}

#[test]
fn target_state_graph_runtime_locks_use_preserves_cc_and_expires_without_healing() {
    use crate::game::battle::{buffs::BuffId, enums::BattleEvent, event_log::BattleEventCause};
    let (mut core, mut state, actor, enemy, _) = fixture();
    let action = core
        .request_skill_action(0, actor, "state_test".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(
        (
            core.units[&actor].stats.attack,
            core.units[&actor].stats.defense
        ),
        (150, 150)
    );
    assert_eq!(core.sustained_basic_interval(actor, 1000), 666);
    assert!(!core.channel_active(actor));
    assert!(core.resonance_use_has_obligations(actor));
    assert_eq!(
        core.query_action(&action).unwrap().facts.resource_spent,
        100
    );
    core.add_resonance(actor, 10, 0, false);
    assert_eq!(core.units[&actor].resonance_current, 0);
    core.process_event(
        BattleEvent::ApplyBuff {
            time_ms: 1,
            caster_instance_id: enemy,
            target_instance_id: actor,
            buff_id: BuffId::from_name("stun"),
            duration_ms: 400,
            cause: BattleEventCause::default(),
        },
        1,
    )
    .unwrap();
    core.step_battle_execution_until(&mut state, 500).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 150);
    joint_committed_damage(&mut core, enemy, actor, 2000, 500);
    assert_eq!(core.units[&actor].stats.current_health, 1);
    core.step_battle_execution_until(&mut state, 999).unwrap();
    assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
    core.step_battle_execution_until(&mut state, 1001).unwrap();
    assert_eq!(
        (
            core.units[&actor].stats.attack,
            core.units[&actor].stats.defense
        ),
        (100, 100)
    );
    assert_eq!(core.sustained_basic_interval(actor, 1000), 1000);
    assert_eq!(core.units[&actor].stats.current_health, 1);
    assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
    assert!(!core.resonance_use_has_obligations(actor));
    core.add_resonance(actor, 10, 1001, false);
    assert_eq!(core.units[&actor].resonance_current, 10);
    joint_committed_damage(&mut core, enemy, actor, 1, 1001);
    assert!(!core.units[&actor].is_active());
}

fn profile(amount: u32, end: bool) -> TargetStateProfile {
    TargetStateProfile {
        definition: "shared".into(),
        attack_percent: amount,
        defense_percent: amount,
        attack_speed_points: amount,
        hp_floor_one: false,
        end_on_source_exit: end,
    }
}

#[test]
fn target_state_allows_normal_basic_attack_while_resonance_use_is_locked() {
    use crate::game::battle::{
        enums::BattleEvent, event_log::BattleEventCause, tile_range::TileRangePolicy,
    };
    let (mut core, mut state, actor, enemy, _) = fixture();
    let unit = core.units.get_mut(&actor).unwrap();
    unit.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
    unit.basic_attack.windup_ms = 1;
    unit.next_basic_attack_ms = 10;
    unit.stats.attack_interval_ms = 100;
    core.units.get_mut(&enemy).unwrap().stats.current_health = 1000;
    core.units.get_mut(&enemy).unwrap().stats.max_health = 1000;
    core.request_skill_action(0, actor, "state_test".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 1).unwrap();
    core.event_queue.push(BattleEvent::AttackStart {
        time_ms: 10,
        attacker_instance_id: actor,
        target_instance_id: Some(enemy),
        schedule_next: true,
        cause: BattleEventCause::default(),
    });
    core.step_battle_execution_until(&mut state, 102).unwrap();
    assert!(core.resonance_use_has_obligations(actor));
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert_eq!(core.units[&actor].stats.attack, 150);
    assert_eq!(core.units[&actor].resonance_current, 0);
}

#[test]
fn target_state_copies_keep_strongest_properties_and_restore_surviving_contributions() {
    let (mut core, mut state, actor, _, ally) = fixture();
    for (caster, amount, duration) in [(actor, 20, 2000), (ally, 50, 1000)] {
        core.begin_sustained(
            caster,
            actor,
            SustainedEffectDef::TargetState {
                duration_ms: duration,
                profile: profile(amount, true),
            },
            0,
        )
        .unwrap();
    }
    assert_eq!(core.units[&actor].stats.attack, 150);
    assert_eq!(core.sustained_basic_interval(actor, 1000), 666);
    core.step_battle_execution_until(&mut state, 1000).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 120);
    assert_eq!(core.sustained_basic_interval(actor, 1000), 833);
    core.step_battle_execution_until(&mut state, 2000).unwrap();
    assert_eq!(core.units[&actor].stats.attack, 100);
}

#[test]
fn target_state_source_exit_policy_preserves_independent_state_until_target_or_deadline() {
    for end in [false, true] {
        let (mut core, mut state, actor, enemy, ally) = fixture();
        core.begin_sustained(
            ally,
            actor,
            SustainedEffectDef::TargetState {
                duration_ms: 1000,
                profile: profile(50, end),
            },
            0,
        )
        .unwrap();
        joint_committed_damage(&mut core, enemy, ally, 1000, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert_eq!(core.units[&actor].stats.attack, if end { 100 } else { 150 });
        core.step_battle_execution_until(&mut state, 1000).unwrap();
        assert_eq!(core.units[&actor].stats.attack, 100);
    }
}

#[test]
fn target_state_apply_and_expiry_faults_restore_atomic_state_and_retry() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in [
        "target_state_definition",
        "sustained_started",
        "sustained_ended",
    ] {
        let (mut core, mut state, actor, _, _) = fixture();
        core.request_skill_action(0, actor, "state_test".into(), None)
            .unwrap();
        core.step_battle_execution_until(&mut state, 0).unwrap();
        let time = if site == "sustained_ended" {
            core.step_battle_execution_until(&mut state, 1000).unwrap();
            1001
        } else {
            1
        };
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            format!("{state:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.step_battle_execution_until(&mut state, time),
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
                format!("{state:?}")
            ),
            before
        );
        core.step_battle_execution_until(&mut state, time).unwrap();
        assert_eq!(
            core.units[&actor].stats.attack,
            if time == 1 { 150 } else { 100 }
        );
    }
}
