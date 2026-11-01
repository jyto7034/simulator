use super::*;
use crate::{
    authoring::node_graph::{compile, tests::scheduled_area_fixture, PortableGraph},
    game::battle::{
        core::{
            relic_causal_tests::joint_committed_damage, sim::BattleExecutionState,
            tests::native_action_fixture,
        },
        runtime_facts::ActionHandle,
        types::BattleUnitSourceIdentity,
    },
};

fn fixture() -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    fixture_with(scheduled_area_fixture())
}
fn fixture_with(
    document: serde_json::Value,
) -> (
    BattleCore,
    BattleExecutionState,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let graph = compile(&document.to_string(), "scheduled".into()).unwrap();
    let db = PortableGraph::from_ron(&graph.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let (mut core, state, actor, enemy, ally) = native_action_fixture(
        db.get_by_id("scheduled").unwrap().clone(),
        Default::default(),
    );
    let unit = core.units.get_mut(&actor).unwrap();
    unit.source_identity = BattleUnitSourceIdentity::Employee {
        employee_uuid: actor.as_uuid(),
        base_uuid: unit.base_uuid,
    };
    unit.stats.attack = 100;
    unit.stats.max_health = 1000;
    unit.stats.current_health = 500;
    for id in [enemy, ally] {
        let unit = core.units.get_mut(&id).unwrap();
        unit.stats.max_health = 1000;
        unit.stats.current_health = 1000;
    }
    (core, state, actor, enemy, ally)
}
fn begin(core: &mut BattleCore, actor: UnitInstanceId) -> ActionHandle {
    core.request_skill_action(0, actor, "scheduled".into(), None)
        .unwrap()
}
#[test]
fn scheduled_area_releases_finite_children_with_fresh_power_and_one_action() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    let action = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(core.active_areas.len(), 2);
    assert!(!core.resonance_use_has_obligations(actor));
    assert!(core.units[&actor].action_locks.can_basic_attack(1));
    core.units.get_mut(&actor).unwrap().stats.attack = 200;
    core.units
        .get_mut(&actor)
        .unwrap()
        .basic_attack_speed_percent = 900;
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert_eq!(core.active_areas.len(), 3);
    core.units.get_mut(&actor).unwrap().stats.attack = 300;
    core.step_battle_execution_until(&mut state, 202).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert_eq!(core.active_areas.len(), 2);
    core.step_battle_execution_until(&mut state, 302).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 550);
    core.step_battle_execution_until(&mut state, 402).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 100);
    assert!(core.active_areas.is_empty());
    assert_eq!(
        serde_json::to_value(core.runtime_fact_view()).unwrap()["usage"]["outstanding_deliveries"],
        0
    );
    let result = core.query_action(&action).unwrap();
    assert_eq!(result.facts.damage_hp_lost, 900);
    assert_eq!(result.facts.resource_spent, 100);
}
#[test]
fn scheduled_area_consumes_empty_opportunities_without_catchup() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    core.units
        .get_mut(&enemy)
        .unwrap()
        .set_world_position(WorldVec2::from_tile_center(Position::new(4, 4)));
    let action = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert_eq!(core.active_areas.len(), 1);
    core.units
        .get_mut(&enemy)
        .unwrap()
        .set_world_position(WorldVec2::from_tile_center(Position::new(2, 1)));
    core.step_battle_execution_until(&mut state, 402).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert!(core.active_areas.is_empty());
    assert_eq!(
        serde_json::to_value(core.runtime_fact_view()).unwrap()["usage"]["outstanding_deliveries"],
        0
    );
    assert_eq!(
        core.query_action(&action).unwrap().facts.damage_hp_lost,
        150
    );
}
#[test]
fn scheduled_area_fixes_admission_search_center_and_each_release_target_tile() {
    let (mut core, mut state, actor, enemy, ally) = fixture();
    begin(&mut core, actor);
    core.units
        .get_mut(&actor)
        .unwrap()
        .set_world_position(WorldVec2::from_tile_center(Position::new(4, 4)));
    core.step_battle_execution_until(&mut state, 1).unwrap();
    let emitter = core
        .active_areas
        .values()
        .find(|r| r.emission.is_some())
        .unwrap();
    assert_eq!(emitter.tile_anchor, Position::new(1, 1));
    core.units
        .get_mut(&enemy)
        .unwrap()
        .set_world_position(WorldVec2::from_tile_center(Position::new(4, 4)));
    core.units.get_mut(&ally).unwrap().owner = Side::Opponent;
    core.units
        .get_mut(&ally)
        .unwrap()
        .set_world_position(WorldVec2::from_tile_center(Position::new(2, 1)));
    core.step_battle_execution_until(&mut state, 402).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    assert_eq!(core.units[&ally].stats.current_health, 550);
}
#[test]
fn scheduled_area_source_death_stops_future_drops_but_keeps_released_child() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    let action = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 50).unwrap();
    joint_committed_damage(&mut core, enemy, actor, 1000, 50);
    assert_eq!(core.active_areas.len(), 1);
    assert!(core.active_areas.values().all(|r| r.emission.is_none()));
    core.step_battle_execution_until(&mut state, 402).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert!(core.active_areas.is_empty());
    assert_eq!(
        serde_json::to_value(core.runtime_fact_view()).unwrap()["usage"]["outstanding_deliveries"],
        0
    );
    assert_eq!(
        core.query_action(&action).unwrap().facts.damage_hp_lost,
        150
    );
}
#[test]
fn scheduled_area_hard_cc_and_silence_preserve_released_schedule() {
    use crate::game::battle::{buffs::BuffId, event_log::BattleEventCause};
    for buff in ["stun", "silence"] {
        let (mut core, mut state, actor, enemy, _) = fixture();
        begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 50).unwrap();
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: 50,
                caster_instance_id: enemy,
                target_instance_id: actor,
                buff_id: BuffId::from_name(buff),
                duration_ms: 500,
                cause: BattleEventCause::default(),
            },
            50,
        )
        .unwrap();
        let control = core.unit_control_state(actor, 50).unwrap();
        assert!(if buff == "stun" {
            control.hard_cc_active()
        } else {
            control.silenced
        });
        core.step_battle_execution_until(&mut state, 402).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 550, "{buff}");
    }
}
#[test]
fn scheduled_area_recast_replaces_future_opportunities_only() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    let first = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 50).unwrap();
    core.units.get_mut(&actor).unwrap().resonance_current = 100;
    let second = core
        .request_skill_action(50, actor, "scheduled".into(), None)
        .unwrap();
    core.step_battle_execution_until(&mut state, 52).unwrap();
    assert_eq!(
        core.active_areas
            .values()
            .filter(|r| r.emission.is_some())
            .count(),
        1
    );
    assert_eq!(core.active_areas.len(), 3);
    core.step_battle_execution_until(&mut state, 452).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 400);
    assert!(core.active_areas.is_empty());
    assert_eq!(
        serde_json::to_value(core.runtime_fact_view()).unwrap()["usage"]["outstanding_deliveries"],
        0
    );
    assert_eq!(core.query_action(&first).unwrap().facts.damage_hp_lost, 150);
    assert_eq!(
        core.query_action(&second).unwrap().facts.damage_hp_lost,
        450
    );
}
#[test]
fn scheduled_area_zero_fall_delay_and_one_opportunity_release_once() {
    let mut doc = scheduled_area_fixture();
    for node in doc["nodes"].as_array_mut().unwrap() {
        if node["kind"] == "skill-count-positive" {
            node["values"]["value"] = serde_json::json!(1);
        }
        if node["kind"] == "skill-seconds" && !node["title"].as_str().unwrap().contains("간격") {
            node["values"]["value"] = serde_json::json!(0);
        }
    }
    let (mut core, mut state, actor, enemy, _) = fixture_with(doc);
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 3).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert!(core.active_areas.is_empty());
    assert_eq!(
        serde_json::to_value(core.runtime_fact_view()).unwrap()["usage"]["outstanding_deliveries"],
        0
    );
}
#[test]
fn scheduled_area_spawn_and_replacement_faults_restore_state_queue_and_facts() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["area_emission_complete", "area_emitter_stopped"] {
        let (mut core, mut state, actor, enemy, _) = fixture();
        begin(&mut core, actor);
        let time = if site == "area_emitter_stopped" {
            core.step_battle_execution_until(&mut state, 50).unwrap();
            core.units.get_mut(&actor).unwrap().resonance_current = 100;
            core.request_skill_action(50, actor, "scheduled".into(), None)
                .unwrap();
            51
        } else {
            1
        };
        core.step_battle_execution_until(&mut state, time - 1)
            .unwrap();
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
        core.step_battle_execution_until(&mut state, 452).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if time == 1 { 550 } else { 400 }
        );
    }
}
#[test]
fn scheduled_area_allows_actual_basic_attack_without_reclassifying_skill_hits() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    let action = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 50).unwrap();
    core.units.get_mut(&actor).unwrap().next_basic_attack_ms = 50;
    core.process_event(
        BattleEvent::AttackStart {
            time_ms: 50,
            attacker_instance_id: actor,
            target_instance_id: Some(enemy),
            schedule_next: true,
            cause: Default::default(),
        },
        50,
    )
    .unwrap();
    core.step_battle_execution_until(&mut state, 251).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 750);
    assert_eq!(
        core.query_action(&action).unwrap().facts.damage_hp_lost,
        150
    );
    core.step_battle_execution_until(&mut state, 402).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 450);
    assert_eq!(
        core.query_action(&action).unwrap().facts.damage_hp_lost,
        450
    );
}
#[test]
fn scheduled_area_withdrawal_stops_only_future_drops_and_terminal_retires_all_areas() {
    use crate::game::battle::{core::sim::BattleLiveCommand, scenario::BattleScenario};
    let (mut core, mut state, actor, enemy, _) = fixture();
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 50).unwrap();
    core.apply_live_command(BattleLiveCommand::WithdrawUnit {
        unit_id: actor,
        time_ms: 50,
    })
    .unwrap();
    assert_eq!(core.active_areas.len(), 1);
    core.step_battle_execution_until(&mut state, 402).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert_eq!(core.runtime_fact_view().usage().outstanding_deliveries, 0);

    let (mut core, mut state, actor, enemy, _) = fixture();
    std::sync::Arc::make_mut(&mut core.scenario).terminal_outcomes =
        BattleScenario::fixture_survival_outcomes(50);
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 51).unwrap();
    assert!(state.is_finished());
    let result = core.finalize_battle_execution(&mut state).unwrap();
    assert!(core.active_areas.is_empty());
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    assert_eq!(
        result.runtime_facts.view().usage().outstanding_deliveries,
        0
    );
}
#[test]
fn scheduled_area_selects_progress_then_spawn_and_unit_identity() {
    use crate::game::battle::scenario::EnemyMovementPlan;
    for mode in ["progress", "spawn", "identity"] {
        let (mut core, mut state, actor, enemy, ally) = fixture();
        let unit = core.units.get_mut(&enemy).unwrap();
        unit.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
            cells: vec![Position::new(0, 1), Position::new(2, 1)],
        });
        unit.spawn_order = 9;
        let unit = core.units.get_mut(&ally).unwrap();
        unit.owner = Side::Opponent;
        unit.spawn_order = if mode == "identity" { 9 } else { 2 };
        let position = Position::new(3, 2);
        unit.set_world_position(WorldVec2::from_tile_center(position));
        unit.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
            cells: vec![
                Position::new(if mode == "progress" { 0 } else { 1 }, 2),
                Position::new(4, 2),
            ],
        });
        begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        let child = core
            .active_areas
            .values()
            .find(|r| r.emission.is_none())
            .unwrap();
        assert_eq!(
            child.tile_anchor,
            if mode == "identity" {
                Position::new(2, 1)
            } else {
                position
            },
            "{mode}"
        );
    }
}
#[test]
fn scheduled_area_impact_footprint_is_separate_from_search_range() {
    let mut doc = scheduled_area_fixture();
    for node in doc["nodes"].as_array_mut().unwrap() {
        if node["kind"] == "skill-count-positive" {
            node["values"]["value"] = serde_json::json!(1);
        }
    }
    let (mut core, mut state, actor, enemy, ally) = fixture_with(doc);
    let unit = core.units.get_mut(&ally).unwrap();
    unit.owner = Side::Opponent;
    unit.set_world_position(WorldVec2::from_tile_center(Position::new(4, 1)));
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 402).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert_eq!(core.units[&ally].stats.current_health, 1000);
}

#[test]
fn scheduled_area_observation_records_actual_start_replacement_and_child_expiry() {
    use crate::authoring::AreaChange;
    use crate::game::ability::sustained::SustainedEndReason;
    let (mut core, mut state, actor, _, _) = fixture();
    core.enable_authoring_observation();
    core.bind_authoring_observation(
        crate::game::battle::core::authoring_observation::ObservationKey {
            entry_ordinal: 1,
            battle_uuid: Uuid::from_u128(0xb31),
        },
    )
    .unwrap();
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    let started = core
        .authoring_observation()
        .unwrap()
        .unwrap()
        .latest_area
        .as_ref()
        .unwrap()
        .clone();
    assert_eq!(started.change, AreaChange::Started);
    assert!(started.before.is_none() && started.after.is_none());
    assert_eq!(started.time_ms, 1);
    let emitter = core
        .active_areas
        .values()
        .find(|a| a.emission.is_some())
        .unwrap();
    let id = emitter.area_id;
    let step = emitter.step_id.clone();
    core.runtime_transaction(|core| {
        core.replace_area_emitter(actor, &"scheduled".into(), &step, 50)
    })
    .unwrap();
    let stopped = core
        .authoring_observation()
        .unwrap()
        .unwrap()
        .latest_area
        .as_ref()
        .unwrap();
    assert_eq!(stopped.area_id, id);
    assert_eq!(
        stopped.change,
        AreaChange::Ended {
            reason: SustainedEndReason::Replaced
        }
    );
    assert_eq!(core.active_areas.len(), 1);
    core.step_battle_execution_until(&mut state, 402).unwrap();
    let ended = core
        .authoring_observation()
        .unwrap()
        .unwrap()
        .latest_area
        .as_ref()
        .unwrap();
    assert_eq!(ended.area_id, started.area_id);
    assert_eq!(
        ended.change,
        AreaChange::Ended {
            reason: SustainedEndReason::Expired
        }
    );
    assert!(core.active_areas.is_empty());
}

include!("canonical_rudolta_tests.inc.rs");
