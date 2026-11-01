use super::*;
use crate::{
    authoring::node_graph::{compile, tests::delayed_area_fixture, PortableGraph},
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
    fixture_with(delayed_area_fixture())
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
    let graph = compile(&document.to_string(), "delayed".into()).unwrap();
    let db = PortableGraph::from_ron(&graph.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let (mut core, state, actor, enemy, ally) =
        native_action_fixture(db.get_by_id("delayed").unwrap().clone(), Default::default());
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
    core.request_skill_action(0, actor, "delayed".into(), None)
        .unwrap()
}
#[test]
fn delayed_area_live_graph_waits_for_release_then_impacts_once_and_heals_once() {
    let (mut core, mut state, actor, enemy, ally) = fixture();
    core.units.get_mut(&ally).unwrap().owner = Side::Opponent;
    core.units
        .get_mut(&ally)
        .unwrap()
        .set_world_position(WorldVec2::from_tile_center(Position::new(2, 2)));
    let action = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 99).unwrap();
    assert!(core.active_areas.is_empty());
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert_eq!(core.active_areas.len(), 1);
    assert!(!core.resonance_use_has_obligations(actor));
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    core.step_battle_execution_until(&mut state, 299).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    core.step_battle_execution_until(&mut state, 302).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert_eq!(core.units[&ally].stats.current_health, 850);
    assert_eq!(core.units[&actor].stats.current_health, 700);
    assert!(core.active_areas.is_empty());
    let result = core.query_action(&action).unwrap();
    assert_eq!(result.facts.damage_hp_lost, 300);
    assert_eq!(result.facts.heal_mutations, 1);
    assert_eq!(result.facts.resource_spent, 100);
}
#[test]
fn delayed_area_keeps_accepted_tile_after_original_target_moves_or_dies() {
    for kill in [false, true] {
        let (mut core, mut state, actor, enemy, ally) = fixture();
        begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        if kill {
            joint_committed_damage(&mut core, actor, enemy, 1000, 2);
        } else {
            core.units
                .get_mut(&enemy)
                .unwrap()
                .set_world_position(WorldVec2::from_tile_center(Position::new(4, 4)));
        }
        core.units.get_mut(&ally).unwrap().owner = Side::Opponent;
        core.units
            .get_mut(&ally)
            .unwrap()
            .set_world_position(WorldVec2::from_tile_center(Position::new(2, 1)));
        core.step_battle_execution_until(&mut state, 302).unwrap();
        assert_eq!(core.units[&ally].stats.current_health, 850, "kill={kill}");
        assert_eq!(core.units[&actor].stats.current_health, 700);
        if !kill {
            assert_eq!(core.units[&enemy].stats.current_health, 1000);
        }
    }
}
#[test]
fn delayed_area_cancels_before_release_but_survives_caster_death_after_release() {
    for after_release in [false, true] {
        let (mut core, mut state, actor, enemy, _) = fixture();
        begin(&mut core, actor);
        let time = if after_release { 101 } else { 10 };
        core.step_battle_execution_until(&mut state, time).unwrap();
        joint_committed_damage(&mut core, enemy, actor, 1000, time);
        core.step_battle_execution_until(&mut state, 302).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if after_release { 850 } else { 1000 }
        );
        assert!(!core.units[&actor].is_active());
        assert!(core.active_areas.is_empty());
    }
}

#[test]
fn delayed_area_effective_hits_include_shields_but_exclude_immunity_and_misses() {
    use crate::game::ability::sustained::SustainedEffectDef;
    for outcome in ["shield", "immune", "miss"] {
        let (mut core, mut state, actor, enemy, _) = fixture();
        let action = begin(&mut core, actor);
        core.step_battle_execution_until(&mut state, 101).unwrap();
        match outcome {
            "shield" => core
                .begin_sustained(
                    enemy,
                    enemy,
                    SustainedEffectDef::Shield {
                        duration_ms: 1000,
                        amount: 150,
                        priority: 0,
                        shared_radius_units: None,
                    },
                    101,
                )
                .unwrap(),
            "immune" => {
                core.units
                    .get_mut(&enemy)
                    .unwrap()
                    .incoming_damage_modifiers
                    .damage_reduction_percent = 100
            }
            "miss" => core
                .units
                .get_mut(&enemy)
                .unwrap()
                .set_world_position(WorldVec2::from_tile_center(Position::new(4, 4))),
            _ => unreachable!(),
        }
        core.step_battle_execution_until(&mut state, 302).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 1000, "{outcome}");
        assert_eq!(
            core.units[&actor].stats.current_health,
            if outcome == "shield" { 700 } else { 500 },
            "{outcome}"
        );
        assert_eq!(
            core.query_action(&action).unwrap().facts.heal_mutations,
            u64::from(outcome == "shield")
        );
    }
}
#[test]
fn delayed_area_captures_power_at_release_and_heals_from_impact_time_max_hp() {
    let (mut core, mut state, actor, enemy, _) = fixture();
    begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    let unit = core.units.get_mut(&actor).unwrap();
    unit.stats.attack = 800;
    unit.stats.max_health = 2000;
    core.step_battle_execution_until(&mut state, 302).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert_eq!(core.units[&actor].stats.current_health, 900);
}
#[test]
fn delayed_area_selects_route_progress_then_spawn_order_and_refuses_no_target() {
    use crate::game::battle::scenario::EnemyMovementPlan;
    for tie in [false, true] {
        let (mut core, _, actor, enemy, ally) = fixture();
        core.units.get_mut(&enemy).unwrap().enemy_movement_plan =
            Some(EnemyMovementPlan::PathAlongCells {
                cells: vec![Position::new(0, 1), Position::new(2, 1)],
            });
        core.units.get_mut(&enemy).unwrap().spawn_order = 9;
        let unit = core.units.get_mut(&ally).unwrap();
        unit.owner = Side::Opponent;
        unit.spawn_order = 2;
        if tie {
            unit.set_world_position(WorldVec2::from_tile_center(Position::new(2, 2)));
            unit.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
                cells: vec![Position::new(0, 2), Position::new(4, 2)],
            });
        } else {
            unit.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
                cells: vec![
                    Position::new(0, 4),
                    Position::new(4, 4),
                    Position::new(4, 0),
                ],
            });
        }
        begin(&mut core, actor);
        assert_eq!(
            core.units[&actor]
                .pending_skill_cast
                .as_ref()
                .unwrap()
                .cast_target,
            Some(SkillCastTarget::Unit {
                unit_instance_id: ally
            }),
            "tie={tie}"
        );
    }
    let (mut core, _, actor, enemy, _) = fixture();
    core.units.get_mut(&enemy).unwrap().owner = Side::Player;
    let before = core.transaction_debug_snapshot();
    assert!(core
        .request_skill_action(0, actor, "delayed".into(), None)
        .is_err());
    assert_eq!(core.transaction_debug_snapshot(), before);
}
#[test]
fn delayed_area_hard_cc_cancels_preparation_but_preserves_released_delivery() {
    use crate::game::battle::{buffs::BuffId, event_log::BattleEventCause};
    for released in [false, true] {
        let (mut core, mut state, actor, enemy, _) = fixture();
        begin(&mut core, actor);
        let now = if released { 101 } else { 10 };
        core.step_battle_execution_until(&mut state, now).unwrap();
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: now,
                caster_instance_id: enemy,
                target_instance_id: actor,
                buff_id: BuffId::from_name("stun"),
                duration_ms: 500,
                cause: BattleEventCause::default(),
            },
            now,
        )
        .unwrap();
        core.step_battle_execution_until(&mut state, 302).unwrap();
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if released { 850 } else { 1000 }
        );
        assert_eq!(
            core.units[&actor].stats.current_health,
            if released { 700 } else { 500 }
        );
        assert_eq!(core.units[&actor].resonance_current, 0);
    }
}
#[test]
fn delayed_area_registration_and_hit_reaction_faults_restore_state_and_retry() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["area_registration_complete", "area_hit_reaction_applied"] {
        let (mut core, mut state, actor, enemy, _) = fixture();
        begin(&mut core, actor);
        let time = if site == "area_registration_complete" {
            100
        } else {
            300
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
        core.step_battle_execution_until(&mut state, 302).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 850);
        assert_eq!(core.units[&actor].stats.current_health, 700);
    }
}

#[test]
fn delayed_area_zero_preparation_and_delay_is_one_instant_delivery() {
    let mut document = delayed_area_fixture();
    for node in document["nodes"].as_array_mut().unwrap() {
        if node["kind"] == "skill-seconds" {
            node["values"]["value"] = serde_json::json!(0);
        }
    }
    let (mut core, mut state, actor, enemy, _) = fixture_with(document);
    let action = begin(&mut core, actor);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert_eq!(core.units[&actor].stats.current_health, 700);
    assert_eq!(core.query_action(&action).unwrap().facts.heal_mutations, 1);
    assert!(core.active_areas.is_empty());
}
