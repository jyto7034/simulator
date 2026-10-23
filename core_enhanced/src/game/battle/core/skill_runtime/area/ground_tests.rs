use super::*;
use crate::{
    authoring::node_graph::{compile, tests::ground_fixture, PortableGraph},
    game::battle::{
        core::{
            relic_causal_tests::joint_committed_damage,
            relic_structural_tests::flow_cases::place_stationary,
            sim::{BattleExecutionState, BattleLiveCommand},
            tests::native_action_fixture,
        },
        runtime_facts::ActionHandle,
        types::BattleUnitSourceIdentity,
    },
};
fn fixture(
    doc: serde_json::Value,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let compiled = compile(&doc.to_string(), "ground".into()).unwrap();
    let db = PortableGraph::from_ron(&compiled.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let (mut core, state, actor, enemy, ally) =
        native_action_fixture(db.get_by_id("ground").unwrap().clone(), Default::default());
    let unit = core.units.get_mut(&actor).unwrap();
    unit.source_identity = BattleUnitSourceIdentity::Employee {
        employee_uuid: actor.as_uuid(),
        base_uuid: unit.base_uuid,
    };
    for (id, x, y) in [(actor, 1, 1), (enemy, 2, 1), (ally, 3, 1)] {
        place_stationary(&mut core, id, x, y);
        let unit = core.units.get_mut(&id).unwrap();
        unit.stats.max_health = 1000;
        unit.stats.current_health = if id == ally { 500 } else { 1000 };
    }
    (core, state, actor, enemy, ally)
}
fn begin(core: &mut BattleCore, actor: UnitInstanceId) -> ActionHandle {
    core.request_skill_action(
        0,
        actor,
        "ground".into(),
        Some(SkillCastTarget::Tile {
            position: Position::new(2, 1),
        }),
    )
    .unwrap()
}
fn set_first(doc: &mut serde_json::Value, value: &str) {
    for node in doc["nodes"].as_array_mut().unwrap() {
        if node["kind"] == "skill-ground-periodic" {
            node["values"]["first"] = serde_json::json!(value);
        }
    }
}
#[test]
fn compiled_ground_ticks_are_half_open_from_creation_and_release_unlocks() {
    for after in [false, true] {
        let mut doc = ground_fixture(false);
        if after {
            set_first(&mut doc, "after-interval");
        }
        let (mut core, mut state, actor, enemy, _) = fixture(doc);
        let action = begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        assert!(!core.resonance_use_has_obligations(actor));
        assert_eq!(core.units[&actor].resonance_current, 0);
        let area = core.active_areas.values().next().unwrap();
        assert_eq!(area.expires_at_ms - area.spawned_at_ms, 300);
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if after { 1000 } else { 950 }
        );
        let first_pulse = if after { 101 } else { 1 };
        core.step_battle_execution_until(&mut state, first_pulse)
            .unwrap();
        let first = core
            .query_action(&action)
            .unwrap()
            .facts
            .latest_damage
            .unwrap()
            .source
            .clone();
        core.step_battle_execution_until(&mut state, first_pulse + 100)
            .unwrap();
        let second = core
            .query_action(&action)
            .unwrap()
            .facts
            .latest_damage
            .unwrap()
            .source
            .clone();
        assert_eq!(first.action, second.action);
        assert_eq!(first.actor, Some(actor));
        // Public facts deliberately strip execution delivery capabilities.
        assert!(first.delivery.is_none() && second.delivery.is_none());
        let first_root = first.causal().unwrap().context.as_ref().unwrap().root;
        let second_root = second.causal().unwrap().context.as_ref().unwrap().root;
        assert_ne!(first_root, second_root);
        core.step_battle_execution_until(&mut state, 301).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if after { 900 } else { 850 }
        );
        assert!(core.active_areas.is_empty());
        assert_eq!(
            core.query_action(&action).unwrap().facts.damage_hp_lost,
            if after { 100 } else { 150 }
        );
        assert_eq!(core.runtime_fact_view().usage().outstanding_deliveries, 0);
    }
}
#[test]
fn compiled_ground_delayed_first_can_expire_without_any_tick() {
    let mut doc = ground_fixture(false);
    set_first(&mut doc, "after-interval");
    for node in doc["nodes"].as_array_mut().unwrap() {
        if node["kind"] == "skill-seconds" {
            node["values"]["value"] = serde_json::json!(0.1);
        }
    }
    let (mut core, mut state, actor, enemy, _) = fixture(doc);
    let action = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(core.active_areas.len(), 1);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert!(core.active_areas.is_empty());
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    assert_eq!(core.query_action(&action).unwrap().facts.damage_hp_lost, 0);
}
#[test]
fn compiled_ground_empty_tile_admission_and_current_tick_membership() {
    let (mut core, mut state, actor, enemy, _) = fixture(ground_fixture(false));
    place_stationary(&mut core, enemy, 4, 4);
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(core.active_areas.len(), 1);
    place_stationary(&mut core, enemy, 3, 1);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 950);
    place_stationary(&mut core, enemy, 4, 4);
    core.step_battle_execution_until(&mut state, 301).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 950);
}
#[test]
fn compiled_ground_selected_asymmetric_pattern_never_rotates_or_follows_caster() {
    for facing in [
        FacingDirection::Up,
        FacingDirection::Right,
        FacingDirection::Down,
        FacingDirection::Left,
    ] {
        let (mut core, mut state, actor, enemy, ally) = fixture(ground_fixture(false));
        core.units.get_mut(&actor).unwrap().facing_direction = Some(facing);
        place_stationary(&mut core, enemy, 3, 1);
        place_stationary(&mut core, ally, 2, 2);
        core.units.get_mut(&ally).unwrap().owner = Side::Opponent;
        begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        place_stationary(&mut core, actor, 0, 4);
        core.step_battle_execution_until(&mut state, 301).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 850, "{facing:?}");
        assert_eq!(core.units[&ally].stats.current_health, 500, "{facing:?}");
    }
}
#[test]
fn compiled_ground_heal_filters_caster_and_caps_actual_healing() {
    for include in [false, true] {
        let mut doc = ground_fixture(true);
        for node in doc["nodes"].as_array_mut().unwrap() {
            if node["kind"] == "skill-ground-filter" {
                node["values"]["caster"] =
                    serde_json::json!(if include { "include" } else { "exclude" });
            }
        }
        let (mut core, mut state, actor, enemy, ally) = fixture(doc);
        place_stationary(&mut core, actor, 2, 1);
        place_stationary(&mut core, enemy, 3, 1);
        place_stationary(&mut core, ally, 2, 1);
        core.units.get_mut(&actor).unwrap().stats.current_health = 900;
        core.units.get_mut(&ally).unwrap().stats.current_health = 970;
        let action = begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 301).unwrap();
        assert_eq!(
            core.units[&actor].stats.current_health,
            if include { 1000 } else { 900 }
        );
        assert_eq!(core.units[&ally].stats.current_health, 1000);
        assert_eq!(core.units[&enemy].stats.current_health, 1000);
        let facts = core.query_action(&action).unwrap().facts;
        assert!(facts.heal_mutations >= 1);
        assert_eq!(facts.healing_delta, if include { 130 } else { 30 });
    }
}
#[test]
fn compiled_ground_range_rejection_is_atomic_and_released_area_survives_source_exit() {
    for withdraw in [false, true] {
        let (mut core, mut state, actor, enemy, _) = fixture(ground_fixture(false));
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
        );
        assert!(core
            .request_skill_action(
                0,
                actor,
                "ground".into(),
                Some(SkillCastTarget::Tile {
                    position: Position::new(4, 4)
                })
            )
            .is_err());
        assert_eq!(
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot()
            ),
            before
        );
        begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        if withdraw {
            core.apply_live_command(BattleLiveCommand::WithdrawUnit {
                unit_id: actor,
                time_ms: 2,
            })
            .unwrap();
        } else {
            joint_committed_damage(&mut core, enemy, actor, 1000, 2);
        }
        core.step_battle_execution_until(&mut state, 301).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 850);
        assert!(core.active_areas.is_empty());
    }
}
#[test]
fn compiled_ground_tick_cancellation_restores_state_queue_facts_and_retry() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for heal in [false, true] {
        let (mut core, mut state, actor, _, _) = fixture(ground_fixture(heal));
        begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        let tick = core
            .active_areas
            .values()
            .next()
            .unwrap()
            .next_tick_ms
            .unwrap();
        core.step_battle_execution_until(&mut state, tick - 1)
            .unwrap();
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
            format!("{state:?}"),
        );
        let token = CancellationToken::default();
        let result = control::with_test_cancellation("area_step_resolved", 1, || {
            control::run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::TrialExecution,
                || core.step_battle_execution_until(&mut state, tick),
            )
        });
        assert!(token.is_cancelled() && result.is_err(), "{result:?}");
        assert_eq!(
            (
                core.transaction_debug_snapshot(),
                core.event_queue_debug_snapshot(),
                serde_json::to_value(core.runtime_fact_view()).unwrap(),
                format!("{state:?}")
            ),
            before
        );
        core.step_battle_execution_until(&mut state, 301).unwrap();
        assert!(core.active_areas.is_empty());
    }
}

#[test]
fn compiled_ground_independent_recasts_keep_distinct_instances_and_results() {
    let (mut core, mut state, actor, enemy, _) = fixture(ground_fixture(false));
    let first = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    core.add_resonance(actor, 100, 1, false);
    let second = core
        .request_skill_action(
            2,
            actor,
            "ground".into(),
            Some(SkillCastTarget::Tile {
                position: Position::new(2, 1),
            }),
        )
        .unwrap();
    core.step_battle_execution_until(&mut state, 3).unwrap();
    assert_eq!(core.active_areas.len(), 2);
    core.step_battle_execution_until(&mut state, 301).unwrap();
    assert_eq!(core.active_areas.len(), 1);
    core.step_battle_execution_until(&mut state, 303).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 700);
    assert_eq!(core.query_action(&first).unwrap().facts.damage_hp_lost, 150);
    assert_eq!(
        core.query_action(&second).unwrap().facts.damage_hp_lost,
        150
    );
    assert!(core.active_areas.is_empty());
    assert_eq!(core.runtime_fact_view().usage().outstanding_deliveries, 0);
}
#[test]
fn compiled_ground_terminal_cleans_instances_and_expiry_cancellation_retries() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    use crate::game::battle::scenario::BattleScenario;
    for heal in [false, true] {
        let (mut core, mut state, actor, enemy, _) = fixture(ground_fixture(heal));
        std::sync::Arc::make_mut(&mut core.scenario).terminal_outcomes =
            BattleScenario::fixture_survival_outcomes(50);
        begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 51).unwrap();
        assert!(state.is_finished());
        let result = core.finalize_battle_execution(&mut state).unwrap();
        assert!(core.active_areas.is_empty());
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if heal { 1000 } else { 950 }
        );
        assert_eq!(
            result.runtime_facts.view().usage().outstanding_deliveries,
            0
        );
    }
    for site in ["area_registration_complete", "area_expired"] {
        let (mut core, mut state, actor, _, _) = fixture(ground_fixture(false));
        begin(&mut core, actor);
        let time = if site == "area_registration_complete" {
            1
        } else {
            301
        };
        if time > 0 {
            core.step_battle_execution_until(&mut state, time - 1)
                .unwrap();
        }
        let before = (
            core.transaction_debug_snapshot(),
            core.event_queue_debug_snapshot(),
            serde_json::to_value(core.runtime_fact_view()).unwrap(),
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
                serde_json::to_value(core.runtime_fact_view()).unwrap(),
                format!("{state:?}")
            ),
            before,
            "{site}"
        );
        core.step_battle_execution_until(&mut state, 301).unwrap();
        assert!(core.active_areas.is_empty());
    }
}

fn status_fixture() -> (BattleCore, BattleExecutionState, UnitInstanceId, UnitInstanceId, UnitInstanceId) {
    let (mut core, state, actor, enemy, ally) = fixture(crate::authoring::node_graph::tests::ground_status_fixture());
    core.units.get_mut(&enemy).unwrap().stats.move_speed_units_per_ms = 100;
    (core, state, actor, enemy, ally)
}
#[test]
fn compiled_ground_status_entry_dwell_and_end_are_half_open() {
    let (mut core, mut state, actor, enemy, _) = status_fixture();
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 80);
    assert!(!core.unit_control_state(enemy, 100).unwrap().hard_cc_active());
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert!(core.unit_control_state(enemy, 101).unwrap().hard_cc_active());
    assert!(!core.resonance_use_has_obligations(actor));
    core.step_battle_execution_until(&mut state, 301).unwrap();
    assert!(!core.unit_control_state(enemy, 301).unwrap().hard_cc_active());
    assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 100);
    assert!(core.active_areas.is_empty());
    assert_eq!(core.runtime_fact_view().usage().outstanding_deliveries, 0);
}
#[test]
fn compiled_ground_status_reentry_restarts_dwell_and_removes_exit_contributions() {
    let (mut core, mut state, actor, enemy, _) = status_fixture();
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    core.step_battle_execution_until(&mut state, 50).unwrap();
    place_stationary(&mut core, enemy, 4, 4);
    core.reconcile_sustained(50).unwrap();
    assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 100);
    core.step_battle_execution_until(&mut state, 90).unwrap();
    place_stationary(&mut core, enemy, 2, 1);
    core.reconcile_sustained(90).unwrap();
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert!(!core.unit_control_state(enemy, 101).unwrap().hard_cc_active());
    core.step_battle_execution_until(&mut state, 190).unwrap();
    assert!(core.unit_control_state(enemy, 190).unwrap().hard_cc_active());
    place_stationary(&mut core, enemy, 4, 4);
    core.reconcile_sustained(191).unwrap();
    assert!(!core.unit_control_state(enemy, 191).unwrap().hard_cc_active());
}
#[test]
fn compiled_ground_status_overlap_ends_only_its_contribution() {
    for overlap in ["sum", "strongest"] {
        let mut doc = crate::authoring::node_graph::tests::ground_status_fixture();
        for node in doc["nodes"].as_array_mut().unwrap() { if node["kind"] == "skill-ground-status" && node["values"]["effect"] == "slow" { node["values"]["overlap"] = serde_json::json!(overlap); } }
        let (mut core, mut state, actor, enemy, _) = fixture(doc);
        core.units.get_mut(&enemy).unwrap().stats.move_speed_units_per_ms = 100;
        begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        core.add_resonance(actor, 100, 1, false);
        core.request_skill_action(2, actor, "ground".into(), Some(SkillCastTarget::Tile { position: Position::new(2, 1) })).unwrap();
        core.step_battle_execution_until(&mut state, 3).unwrap();
        assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, if overlap == "sum" { 60 } else { 80 });
        core.step_battle_execution_until(&mut state, 301).unwrap();
        assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 80);
        assert!(core.unit_control_state(enemy, 301).unwrap().hard_cc_active());
        core.step_battle_execution_until(&mut state, 303).unwrap();
        assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 100);
        assert!(!core.unit_control_state(enemy, 303).unwrap().hard_cc_active());
    }
}
#[test]
fn compiled_ground_status_area_lifetime_survives_exit_and_caster_death() {
    let mut doc = crate::authoring::node_graph::tests::ground_status_fixture();
    for node in doc["nodes"].as_array_mut().unwrap() { if node["kind"] == "skill-ground-status" { node["values"]["lifetime"] = serde_json::json!("area"); } }
    let (mut core, mut state, actor, enemy, _) = fixture(doc);
    core.units.get_mut(&enemy).unwrap().stats.move_speed_units_per_ms = 100;
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    place_stationary(&mut core, enemy, 4, 4);
    core.reconcile_sustained(102).unwrap();
    assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 80);
    assert!(core.unit_control_state(enemy, 102).unwrap().hard_cc_active());
    place_stationary(&mut core, enemy, 2, 1);
    core.reconcile_sustained(103).unwrap();
    assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 80);
    joint_committed_damage(&mut core, enemy, actor, 1000, 104);
    assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 80);
    core.step_battle_execution_until(&mut state, 301).unwrap();
    assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 100);
}
#[test]
fn compiled_ground_status_damage_taken_changes_actual_damage_and_target_death_cleans() {
    let mut doc = crate::authoring::node_graph::tests::ground_status_fixture();
    for node in doc["nodes"].as_array_mut().unwrap() { if node["kind"] == "skill-ground-status" && node["values"]["effect"] == "slow" { node["values"]["effect"] = serde_json::json!("damage-taken"); } }
    let (mut core, mut state, actor, enemy, _) = fixture(doc);
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    joint_committed_damage(&mut core, actor, enemy, 100, 2);
    assert_eq!(core.units[&enemy].stats.current_health, 880);
    joint_committed_damage(&mut core, actor, enemy, 1000, 3);
    assert!(!core.has_ground_contribution(enemy));
}

#[test]
fn compiled_ground_status_dwell_at_expiry_never_stuns() {
    let mut doc = crate::authoring::node_graph::tests::ground_status_fixture();
    for node in doc["nodes"].as_array_mut().unwrap() { if node["kind"] == "skill-seconds" { node["values"]["value"] = serde_json::json!(0.3); } }
    let (mut core, mut state, actor, enemy, _) = fixture(doc);
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 300).unwrap();
    assert!(!core.unit_control_state(enemy, 300).unwrap().hard_cc_active());
    core.step_battle_execution_until(&mut state, 301).unwrap();
    assert!(!core.unit_control_state(enemy, 301).unwrap().hard_cc_active());
    assert!(!core.has_ground_contribution(enemy));
}
#[test]
fn compiled_ground_status_explicit_clear_follows_the_same_entry_flow() {
    let mut doc = crate::authoring::node_graph::tests::ground_status_fixture();
    let entry = doc["nodes"].as_array().unwrap().iter().find(|n| n["kind"] == "skill-ground-enter").unwrap()["id"].clone();
    let apply_id = doc["edges"].as_array().unwrap().iter().find(|e| e["source"]["node"] == entry && e["source"]["port"] == "out").unwrap()["target"]["node"].clone();
    let mut clear = doc["nodes"].as_array().unwrap().iter().find(|n| n["id"] == apply_id).unwrap().clone();
    let clear_id = serde_json::json!("00100000-0000-4000-8000-000000000001");
    clear["id"] = clear_id.clone(); clear["kind"] = serde_json::json!("skill-ground-clear-status");
    let connections: Vec<_> = doc["edges"].as_array().unwrap().iter().filter(|e| e["target"]["node"] == apply_id).cloned().collect();
    for (index, mut edge) in connections.into_iter().enumerate() {
        edge["id"] = serde_json::json!(format!("00200000-0000-4000-8000-{index:012}"));
        edge["target"]["node"] = clear_id.clone();
        if edge["target"]["port"] == "in" { edge["source"]["node"] = apply_id.clone(); }
        doc["edges"].as_array_mut().unwrap().push(edge);
    }
    doc["nodes"].as_array_mut().unwrap().push(clear);
    let (mut core, mut state, actor, enemy, _) = fixture(doc);
    core.units.get_mut(&enemy).unwrap().stats.move_speed_units_per_ms = 100;
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 100);
    assert!(!core.has_ground_contribution(enemy));
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert!(core.unit_control_state(enemy, 101).unwrap().hard_cc_active());
}

#[test]
fn compiled_ground_status_cancel_rolls_back_relations_stats_and_deadlines() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for (site, time) in [("ground_reconciled", 1), ("ground_reconciled", 101), ("area_expired", 301)] {
        let (mut core, mut state, actor, enemy, _) = status_fixture();
        begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, time - 1).unwrap();
        let before = (core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), serde_json::to_value(core.runtime_fact_view()).unwrap(), format!("{state:?}"), format!("{:?}", core.units[&enemy].stats));
        let token = CancellationToken::default();
        let result = control::with_test_cancellation(site, 1, || control::run(&ExecutionLimits::default(), &token, ExecutionPhase::TrialExecution, || core.step_battle_execution_until(&mut state, time)));
        assert!(token.is_cancelled() && result.is_err(), "{site}@{time}: {result:?}");
        assert_eq!((core.transaction_debug_snapshot(), core.event_queue_debug_snapshot(), serde_json::to_value(core.runtime_fact_view()).unwrap(), format!("{state:?}"), format!("{:?}", core.units[&enemy].stats)), before);
        core.step_battle_execution_until(&mut state, 301).unwrap();
        assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 100);
        assert_eq!(core.runtime_fact_view().usage().outstanding_deliveries, 0);
    }
}
#[test]
fn compiled_ground_status_terminal_restores_stats_and_releases_ownership() {
    let (mut core, mut state, actor, enemy, _) = status_fixture();
    std::sync::Arc::make_mut(&mut core.scenario).terminal_outcomes = crate::game::battle::scenario::BattleScenario::fixture_survival_outcomes(150);
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 151).unwrap();
    assert!(state.is_finished());
    let result = core.finalize_battle_execution(&mut state).unwrap();
    assert!(core.active_areas.is_empty());
    assert_eq!(core.units[&enemy].stats.move_speed_units_per_ms, 100);
    assert!(!core.unit_control_state(enemy, 151).unwrap().hard_cc_active());
    assert_eq!(result.runtime_facts.view().usage().outstanding_deliveries, 0);
}
