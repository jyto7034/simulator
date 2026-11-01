//! End-to-end authored file -> compiler -> portable RON -> native loader ->
//! production manual admission/time advancement. Setup alone uses test units.
use super::*;
use crate::authoring::composition::{compile_json, Artifact};
use crate::game::battle::{
    core::sim::{BattleExecutionState, BattleLiveCommand},
    runtime_facts::{ActionCancelReason, ActionHandle, ActionTerminalKind},
    types::BattleUnitSourceIdentity,
};
use serde_json::{json, Value};

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/woodsman_monk.json"
));

#[path = "canonical_area_strikes.rs"]
mod canonical_area_strikes;

fn fixture(
    source: &str,
    skill: &str,
) -> (
    BattleCore,
    BattleExecutionState,
    Artifact,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let compiled = compile_json(source).unwrap();
    fixture_artifact(compiled, skill)
}
fn fixture_artifact(
    compiled: Artifact,
    skill: &str,
) -> (
    BattleCore,
    BattleExecutionState,
    Artifact,
    UnitInstanceId,
    UnitInstanceId,
    UnitInstanceId,
) {
    let artifact = Artifact::from_ron(&compiled.to_ron().unwrap()).unwrap();
    let database = artifact.load_native().unwrap();
    let data = GameDataBuilder::live_defaults()
        .with_abnormalities(vec![])
        .with_skills(database)
        .build_arc();
    let mut scenario = BattleScenario::empty((6, 6));
    scenario.terminal_outcomes = BattleScenario::fixture_survival_outcomes(10_000);
    let mut core = BattleCore::new_from_scenario(scenario, data, 123);
    let state = core.start_battle_execution().unwrap();
    let actor = Uuid::from_u128(0xCA01).into();
    let enemy = Uuid::from_u128(0xCA02).into();
    let ally = Uuid::from_u128(0xCA03).into();
    for (id, side, tile) in [
        (actor, Side::Player, Position::new(1, 1)),
        (enemy, Side::Opponent, Position::new(3, 1)),
        (ally, Side::Player, Position::new(4, 4)),
    ] {
        let mut unit = runtime_unit(id, side);
        unit.stats.current_health = if side == Side::Player { 500 } else { 1000 };
        unit.stats.max_health = 1000;
        unit.stats.attack = 100;
        unit.stats.attack_interval_ms = 100_000;
        unit.next_basic_attack_ms = 100_000;
        unit.stats.move_speed_units_per_ms = 0;
        unit.skill_activation_mode = SkillActivationMode::Manual;
        if side == Side::Player {
            unit.source_identity = BattleUnitSourceIdentity::Employee {
                employee_uuid: id.as_uuid(),
                base_uuid: unit.base_uuid,
            };
            unit.skill_id = Some(skill.into());
            unit.resonance_current = 100;
        }
        core.insert_runtime_unit(id, unit);
        place_unit(&mut core, id, tile);
        sync_unit_to_battlefield_tile_center(&mut core, id);
    }
    // Execution uses the portable roundtrip; attribution uses the separately
    // retained compiler result, not trust asserted by imported metadata.
    (core, state, compiled, actor, enemy, ally)
}
fn begin(core: &mut BattleCore, actor: UnitInstanceId, time: u64) -> ActionHandle {
    core.request_skill_action(
        time,
        actor,
        core.units[&actor].skill_id.clone().unwrap(),
        None,
    )
    .unwrap()
}

#[test]
fn yaml_edit_and_recompile_keep_running_definition_and_native_provenance_pinned() {
    use crate::authoring::composition::compile_yaml;
    const YAML: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/woodsman_monk.skill.yaml"
    ));
    let old = compile_yaml(YAML).unwrap();
    let (mut first, mut first_state, old_artifact, actor, enemy, _) =
        fixture_artifact(old, "monk-authored");
    begin(&mut first, actor, 0);
    first
        .step_battle_execution_until(&mut first_state, 101)
        .unwrap();
    let edited = YAML
        .replacen("value: 3", "value: 5", 1)
        .replace("prepare_ms: 100", "prepare_ms: 150");
    assert_ne!(edited, YAML);
    let new = compile_yaml(&edited).unwrap();
    let (mut second, mut second_state, new_artifact, new_actor, new_enemy, _) =
        fixture_artifact(new, "monk-authored");
    begin(&mut second, new_actor, 0);
    first
        .step_battle_execution_until(&mut first_state, 302)
        .unwrap();
    second
        .step_battle_execution_until(&mut second_state, 302)
        .unwrap();
    assert_eq!(first.units[&enemy].stats.current_health, 850);
    assert_eq!(second.units[&new_enemy].stats.current_health, 1000);
    second
        .step_battle_execution_until(&mut second_state, 352)
        .unwrap();
    assert_eq!(second.units[&new_enemy].stats.current_health, 750);
    assert_eq!(first.units[&actor].stats.current_health, 700);
    assert_eq!(second.units[&new_actor].stats.current_health, 700);
    let effect = first
        .event_log
        .entries
        .iter()
        .filter_map(|entry| entry.skill_effect_source.as_ref())
        .next()
        .unwrap();
    assert!(old_artifact.origin_for(&effect.authored).is_some());
    assert!(new_artifact.origin_for(&effect.authored).is_none());
}
fn stun(core: &mut BattleCore, enemy: UnitInstanceId, actor: UnitInstanceId, now: u64) {
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
}
fn move_to(core: &mut BattleCore, id: UnitInstanceId, x: i32, y: i32) {
    core.units
        .get_mut(&id)
        .unwrap()
        .set_world_position(WorldVec2::from_tile_center(Position::new(x, y)));
}

#[test]
fn composition_woodsman_three_stages_read_live_targets_and_keep_status_lifetime() {
    let (mut core, mut state, artifact, actor, enemy, _) = fixture(SOURCE, "woodsman-authored");
    let action = begin(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 900);
    assert_eq!(
        core.unit_world_position_or_tile_center(enemy)
            .unwrap()
            .project_to_tile(),
        Position::new(2, 1)
    );
    assert!(core.resonance_use_has_obligations(actor));
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 800);
    assert!(core
        .unit_control_state(enemy, 101)
        .unwrap()
        .hard_cc_active());
    core.step_battle_execution_until(&mut state, 201).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 680);
    assert!(!core.resonance_use_has_obligations(actor));
    assert!(core
        .unit_control_state(enemy, 201)
        .unwrap()
        .hard_cc_active());
    core.step_battle_execution_until(&mut state, 401).unwrap();
    assert!(!core
        .unit_control_state(enemy, 401)
        .unwrap()
        .hard_cc_active());
    let receipt = core.take_settled_action_result(&action).unwrap();
    assert_eq!(receipt.action.facts.damage_hp_lost, 320);
    let damage_sources: Vec<_> = core
        .event_log
        .entries
        .iter()
        .filter_map(|entry| {
            if !matches!(entry.event, BattleLogEvent::HpChanged { .. }) {
                return None;
            }
            entry.skill_effect_source.as_ref()
        })
        .collect();
    assert_eq!(damage_sources.len(), 3);
    for (index, source) in damage_sources.iter().enumerate() {
        assert_eq!(source.cast.caster_instance_id, actor.into());
        let trace = artifact
            .origin_for(&source.authored)
            .expect("runtime source returns to authored definition");
        assert_eq!(trace.definition, "definitions[0].body.effects[0]");
        assert!(trace
            .calls
            .contains(&format!("skills[0].body.children[{index}]")));
    }
}

#[test]
fn composition_monk_delayed_area_keeps_one_recovery_for_multiple_actual_hits() {
    let (mut core, mut state, artifact, actor, enemy, other) = fixture(SOURCE, "monk-authored");
    core.units.get_mut(&other).unwrap().owner = Side::Opponent;
    core.units.get_mut(&other).unwrap().stats.current_health = 1000;
    move_to(&mut core, other, 3, 2);
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    assert_eq!(core.units[&other].stats.current_health, 1000);
    let action = begin(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 99).unwrap();
    assert!(core.active_areas.is_empty());
    assert_eq!(core.units[&enemy].stats.current_health, 1000);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert_eq!(core.active_areas.len(), 1);
    assert!(!core.resonance_use_has_obligations(actor));
    core.step_battle_execution_until(&mut state, 299).unwrap();
    assert_eq!(core.units[&actor].stats.current_health, 500);
    core.step_battle_execution_until(&mut state, 302).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 850);
    assert_eq!(core.units[&other].stats.current_health, 850);
    assert_eq!(core.units[&actor].stats.current_health, 700);
    let facts = &core.query_action(&action).unwrap().facts;
    assert_eq!(facts.damage_hp_lost, 300);
    assert_eq!(facts.heal_mutations, 1);
    assert_eq!(facts.resource_spent, 100);
    assert!(core.active_areas.is_empty());
    assert_eq!(core.runtime_fact_usage().outstanding_deliveries, 0);
    let source = core
        .event_log
        .entries
        .iter()
        .filter_map(|e| e.skill_effect_source.as_ref())
        .find(|s| s.authored.site.effect_id == "damage")
        .unwrap();
    assert!(artifact
        .origin_for(&source.authored)
        .unwrap()
        .calls
        .contains(&"skills[1].body".into()));
}

#[test]
fn composition_monk_preparation_cancel_and_released_survival_have_distinct_results() {
    for released in [false, true] {
        for reason in ["stun", "death", "withdraw"] {
            let (mut core, mut state, _, actor, enemy, _) = fixture(SOURCE, "monk-authored");
            let action = begin(&mut core, actor, 0);
            let now = if released { 101 } else { 10 };
            core.step_battle_execution_until(&mut state, now).unwrap();
            match reason {
                "stun" => stun(&mut core, enemy, actor, now),
                "death" => crate::game::battle::core::relic_causal_tests::joint_committed_damage(
                    &mut core, enemy, actor, 1000, now,
                ),
                _ => {
                    core.apply_live_command(BattleLiveCommand::WithdrawUnit {
                        unit_id: actor,
                        time_ms: now,
                    })
                    .unwrap();
                }
            }
            core.step_battle_execution_until(&mut state, 601).unwrap();
            assert_eq!(
                core.units[&enemy].stats.current_health,
                if released { 850 } else { 1000 },
                "{reason} released={released}"
            );
            assert_eq!(
                core.query_action(&action).unwrap().facts.heal_mutations,
                u64::from(released && reason == "stun")
            );
            assert!(core.active_areas.is_empty());
            assert_eq!(core.runtime_fact_usage().outstanding_deliveries, 0);
        }
    }
}

#[test]
fn composition_two_owners_cancel_and_recast_do_not_share_flow_state() {
    let (mut core, mut state, _, actor, enemy, ally) = fixture(SOURCE, "woodsman-authored");
    move_to(&mut core, ally, 1, 2);
    let first = begin(&mut core, actor, 0);
    let second = begin(&mut core, ally, 0);
    assert_ne!(first, second);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 800);
    stun(&mut core, enemy, actor, 1);
    core.step_battle_execution_until(&mut state, 502).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 570);
    assert_eq!(core.query_action(&first).unwrap().facts.damage_hp_lost, 100);
    assert!(matches!(
        core.query_action(&first).unwrap().terminal.unwrap().kind,
        ActionTerminalKind::Cancelled(ActionCancelReason::HardCrowdControl)
    ));
    assert_eq!(
        core.query_action(&second).unwrap().facts.damage_hp_lost,
        330
    );
    core.units.get_mut(&enemy).unwrap().stats.current_health = 1000;
    core.units.get_mut(&actor).unwrap().resonance_current = 100;
    move_to(&mut core, enemy, 3, 1);
    let fresh = begin(&mut core, actor, 600);
    assert_ne!(fresh, first);
    core.step_battle_execution_until(&mut state, 801).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 680);
    assert_eq!(core.query_action(&fresh).unwrap().facts.damage_hp_lost, 320);
    assert_eq!(core.query_action(&first).unwrap().facts.damage_hp_lost, 100);
}

fn conditional_repeat_source() -> Value {
    let mut doc: Value = serde_json::from_str(SOURCE).unwrap();
    // Data-only definition: one settled attack followed by one conditional heal.
    // This is a use-owned experimental composition, not the Monk's independent
    // delivery-owned recovery contract.
    doc["definitions"][0]["body"]["Action"]["wait"] = json!("EffectsSettled");
    let mut call = doc["skills"][0]["body"]["Sequence"]["children"][0].clone();
    call["Call"]["id"] = json!("probe");
    call["Call"]["arguments"][2]["value"] = json!({"Shared":"none"});
    doc["definitions"].as_array_mut().unwrap().push(json!({
        "name":"hit-and-recover", "parameters":[],
        "body":{"Sequence":{"id":"combination","children":[call,
            {"Branch":{"id":"decision","condition":{"Compare":{"node_id":"probe","metric":"DamagedTargets","comparison":"AtLeast","value":{"Count":1}}},
                "then_node":{"Action":{"id":"recover","target":{"Literal":{"Target":"SelfUnit"}},"delivery":{"Literal":{"Delivery":"Instant"}},"effects":[{"Heal":{"id":"heal","tags":[],"amount":{"Literal":{"Amount":{"moment":"Live","root":{"Const":{"value":25,"unit":"Magnitude"}}}}}}}],"wait":"EffectsSettled"}},
                "else_node":{"Wait":{"id":"no-hit","duration":{"Literal":{"Milliseconds":0}}}}
            }}
        ]}}
    }));
    doc["skills"][0]["body"] = json!({"Repeat":{"id":"three-opportunities","count":{"Literal":{"Count":3}},"interval":{"Literal":{"Milliseconds":100}},"body":{"Call":{"id":"attempt","definition":"hit-and-recover","arguments":[]}}}});
    doc
}

#[test]
fn composition_data_only_repeat_resets_previous_result_before_the_next_condition() {
    let doc = conditional_repeat_source();
    let (mut core, mut state, _, actor, enemy, _) = fixture(&doc.to_string(), "woodsman-authored");
    let action = begin(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 2).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 900);
    assert_eq!(core.units[&actor].stats.current_health, 525);
    move_to(&mut core, enemy, 5, 5);
    core.step_battle_execution_until(&mut state, 102).unwrap();
    assert_eq!(core.units[&actor].stats.current_health, 525);
    move_to(&mut core, enemy, 3, 1);
    core.step_battle_execution_until(&mut state, 205).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 800);
    assert_eq!(core.units[&actor].stats.current_health, 550);
    assert_eq!(core.query_action(&action).unwrap().facts.heal_mutations, 2);
    assert!(!core.resonance_use_has_obligations(actor));
}

#[test]
fn composition_data_only_parallel_calls_have_distinct_local_results_and_effect_sources() {
    let mut doc = conditional_repeat_source();
    doc["skills"][0]["body"] = json!({"Parallel":{"id":"both","branches":[
        {"Call":{"id":"left","definition":"hit-and-recover","arguments":[]}},
        {"Call":{"id":"right","definition":"hit-and-recover","arguments":[]}}
    ]}});
    let (mut core, mut state, artifact, actor, enemy, _) =
        fixture(&doc.to_string(), "woodsman-authored");
    begin(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 5).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 800);
    assert_eq!(core.units[&actor].stats.current_health, 550);
    let calls: std::collections::BTreeSet<_> = core
        .event_log
        .entries
        .iter()
        .filter_map(|entry| {
            let source = entry.skill_effect_source.as_ref()?;
            if source.authored.site.effect_id != "heal" {
                return None;
            }
            artifact
                .origin_for(&source.authored)
                .map(|trace| trace.calls[1].clone())
        })
        .collect();
    assert_eq!(
        calls,
        std::collections::BTreeSet::from([
            "skills[0].body.branches[0]".into(),
            "skills[0].body.branches[1]".into()
        ])
    );
}

#[test]
fn composition_monk_shield_only_hit_recovers_but_immunity_and_miss_do_not() {
    use crate::game::ability::sustained::SustainedEffectDef;
    for outcome in ["shield", "immune", "miss"] {
        let (mut core, mut state, _, actor, enemy, _) = fixture(SOURCE, "monk-authored");
        let action = begin(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 101).unwrap();
        assert_eq!(core.active_areas.len(), 1);
        match outcome {
            "shield" => {
                core.begin_sustained(
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
                .unwrap();
            }
            "immune" => {
                core.units
                    .get_mut(&enemy)
                    .unwrap()
                    .incoming_damage_modifiers
                    .damage_reduction_percent = 100
            }
            _ => move_to(&mut core, enemy, 5, 5),
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
        assert!(core.active_areas.is_empty());
    }
}

#[test]
fn composition_monk_keeps_admission_tile_release_power_and_live_recovery() {
    for dies in [false, true] {
        let (mut core, mut state, _, actor, enemy, other) = fixture(SOURCE, "monk-authored");
        begin(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 1).unwrap();
        if dies {
            crate::game::battle::core::relic_causal_tests::joint_committed_damage(
                &mut core, actor, enemy, 1000, 2,
            );
        } else {
            move_to(&mut core, enemy, 5, 5);
        }
        core.units.get_mut(&other).unwrap().owner = Side::Opponent;
        core.units.get_mut(&other).unwrap().stats.current_health = 1000;
        move_to(&mut core, other, 3, 1);
        core.step_battle_execution_until(&mut state, 101).unwrap();
        core.units.get_mut(&actor).unwrap().stats.attack = 800;
        core.units.get_mut(&actor).unwrap().stats.max_health = 2000;
        core.step_battle_execution_until(&mut state, 302).unwrap();
        assert_eq!(core.units[&other].stats.current_health, 850, "dies={dies}");
        assert_eq!(core.units[&actor].stats.current_health, 900, "dies={dies}");
    }
}

#[test]
fn composition_woodsman_requeries_after_original_target_death() {
    let (mut core, mut state, _, actor, enemy, other) = fixture(SOURCE, "woodsman-authored");
    core.units.get_mut(&enemy).unwrap().stats.current_health = 100;
    begin(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert!(!core.units[&enemy].is_active());
    core.units.get_mut(&other).unwrap().owner = Side::Opponent;
    core.units.get_mut(&other).unwrap().stats.current_health = 1000;
    move_to(&mut core, other, 3, 1);
    core.step_battle_execution_until(&mut state, 201).unwrap();
    assert_eq!(core.units[&other].stats.current_health, 790);
}

#[test]
fn composition_different_skills_share_definition_without_sharing_cancellation_or_receipts() {
    let (mut core, mut state, _, actor, enemy, ally) = fixture(SOURCE, "woodsman-authored");
    core.units.get_mut(&ally).unwrap().skill_id = Some("monk-authored".into());
    move_to(&mut core, ally, 1, 2);
    let woodsman = begin(&mut core, actor, 0);
    let monk = begin(&mut core, ally, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 900);
    stun(&mut core, enemy, actor, 1);
    core.step_battle_execution_until(&mut state, 302).unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 750);
    assert_eq!(core.units[&ally].stats.current_health, 700);
    assert_eq!(
        core.query_action(&woodsman).unwrap().facts.damage_hp_lost,
        100
    );
    assert_eq!(core.query_action(&monk).unwrap().facts.damage_hp_lost, 150);
    assert_eq!(
        core.query_action(&woodsman).unwrap().facts.heal_mutations,
        0
    );
    assert_eq!(core.query_action(&monk).unwrap().facts.heal_mutations, 1);
}

#[test]
fn composition_data_only_shared_value_and_order_changes_change_actual_results() {
    for variant in ["shared-value", "order"] {
        let mut doc: Value = serde_json::from_str(SOURCE).unwrap();
        if variant == "shared-value" {
            doc["values"][0]["value"]["Amount"]["root"] =
                json!({"Const":{"value":50,"unit":"Magnitude"}});
        } else {
            let steps = doc["skills"][0]["body"]["Sequence"]["children"]
                .as_array_mut()
                .unwrap();
            steps.rotate_right(1);
            for (i, step) in steps.iter_mut().enumerate() {
                step["Call"]["arguments"][3]["value"] =
                    json!({"Literal":{"Milliseconds":if i==0 {0} else {100}}});
            }
        }
        let (mut core, mut state, _, actor, enemy, _) =
            fixture(&doc.to_string(), "woodsman-authored");
        begin(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 201).unwrap();
        // Shared value: 50+50+110. Order: missing-HP strike first, 100+100+100.
        assert_eq!(
            core.units[&enemy].stats.current_health,
            if variant == "shared-value" { 790 } else { 700 },
            "{variant}"
        );
    }
}

#[test]
fn composition_native_fault_rolls_back_effects_and_flow_then_retries_once() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    for site in ["area_registration_complete", "area_hit_reaction_applied"] {
        let (mut core, mut state, _, actor, enemy, _) = fixture(SOURCE, "monk-authored");
        let action = begin(&mut core, actor, 0);
        let time = if site == "area_registration_complete" {
            100
        } else {
            300
        };
        core.step_battle_execution_until(&mut state, time - 1)
            .unwrap();
        let before = native_action_probe(&core, &state);
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
        assert_eq!(native_action_probe(&core, &state), before);
        core.step_battle_execution_until(&mut state, 302).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 850);
        assert_eq!(core.units[&actor].stats.current_health, 700);
        assert_eq!(core.query_action(&action).unwrap().facts.heal_mutations, 1);
    }
}

#[test]
fn composition_no_target_admission_preserves_cost_and_termination_retires_released_work() {
    let (mut core, _, _, actor, enemy, _) = fixture(SOURCE, "monk-authored");
    move_to(&mut core, enemy, 5, 5);
    let before = core.transaction_debug_snapshot();
    assert!(core
        .request_skill_action(0, actor, "monk-authored".into(), None)
        .is_err());
    assert_eq!(core.transaction_debug_snapshot(), before);
    assert_eq!(core.units[&actor].resonance_current, 100);

    let (mut core, mut state, _, actor, enemy, _) = fixture(SOURCE, "monk-authored");
    Arc::make_mut(&mut core.scenario).terminal_outcomes =
        BattleScenario::fixture_survival_outcomes(150);
    begin(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 101).unwrap();
    assert_eq!(core.active_areas.len(), 1);
    core.step_battle_execution_until(&mut state, 151).unwrap();
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
fn composition_withdraw_and_native_redeployment_reject_old_flow_ownership() {
    use crate::game::battle::core::sim::BattleLiveCommandOutcome;
    let (mut core, mut state, _, actor, enemy, _) = fixture(SOURCE, "woodsman-authored");
    Arc::make_mut(&mut core.scenario).terminal_outcomes =
        BattleScenario::fixture_survival_outcomes(40_000);
    let old = begin(&mut core, actor, 0);
    core.step_battle_execution_until(&mut state, 1).unwrap();
    core.apply_live_command(BattleLiveCommand::WithdrawUnit {
        unit_id: actor,
        time_ms: 1,
    })
    .unwrap();
    core.step_battle_execution_until(&mut state, 31_000)
        .unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 900);
    assert!(matches!(
        core.query_action(&old).unwrap().terminal.unwrap().kind,
        ActionTerminalKind::Cancelled(ActionCancelReason::Withdrawn)
    ));
    let mut draft = fixture_draft(actor.as_uuid(), Uuid::nil(), 1000, 100);
    let BattleUnitSource::TestFixture { profile, .. } = &mut draft.source else {
        unreachable!()
    };
    profile.skill_id = Some("woodsman-authored".into());
    profile.skill_activation_mode = SkillActivationMode::Manual;
    profile.basic_attack.interval_ms = 100_000;
    profile.resonance.max = 100;
    profile.resonance.start = 100;
    let outcome = core
        .apply_live_command(BattleLiveCommand::DeployPlayerUnit {
            draft,
            position: Position::new(1, 1),
            tile_kind: TacticalTileKind::Ground,
            facing: FacingDirection::Right,
            instance_salt: 17,
            time_ms: 31_001,
            current_hp_policy: None,
        })
        .unwrap();
    let BattleLiveCommandOutcome::UnitDeployed {
        unit_id: fresh_actor,
    } = outcome
    else {
        panic!("expected native deployment")
    };
    assert_ne!(actor, fresh_actor);
    assert!(core.units[&fresh_actor].resonance_use.is_none());
    // As in fixture(), supply an employee identity for resonance admission.
    // Deployment is native; this test does not cover World equipment validation.
    let fresh_unit = core.units.get_mut(&fresh_actor).unwrap();
    fresh_unit.source_identity = BattleUnitSourceIdentity::Employee {
        employee_uuid: actor.as_uuid(),
        base_uuid: fresh_unit.base_uuid,
    };
    // The fixture supplies charge for a new use; it does not restore old charge.
    core.units.get_mut(&fresh_actor).unwrap().resonance_current = 100;
    core.units
        .get_mut(&fresh_actor)
        .unwrap()
        .next_basic_attack_ms = 100_000;
    core.units.get_mut(&enemy).unwrap().stats.current_health = 1000;
    move_to(&mut core, enemy, 3, 1);
    let fresh = begin(&mut core, fresh_actor, 31_001);
    core.step_battle_execution_until(&mut state, 31_202)
        .unwrap();
    assert_eq!(core.units[&enemy].stats.current_health, 680);
    assert_eq!(core.query_action(&old).unwrap().facts.damage_hp_lost, 100);
    assert!(core.take_settled_action_result(&fresh).is_err());
    // The second hit's 300ms status still owns a live result at 31_202.
    core.step_battle_execution_until(&mut state, 31_402)
        .unwrap();
    assert_eq!(
        core.take_settled_action_result(&fresh)
            .unwrap()
            .action
            .facts
            .damage_hp_lost,
        320
    );
    assert!(
        core.query_action(&fresh).is_err(),
        "consumed result cannot become a new activation"
    );
}

#[test]
fn composition_predicates_nested_effect_results_are_isolated_per_parallel_call() {
    // Both occurrences deal 100. A threshold of 150 must fail for both: a
    // mistakenly shared result of 200 would heal on the second occurrence.
    for (minimum, expected_hp) in [(100, 550), (150, 500)] {
        let mut doc = conditional_repeat_source();
        doc["version"] = json!(2);
        doc["predicates"] = json!([{"name":"eligible-hit","context":"Flow","parameters":[
        {"name":"result","value_type":{"FlowResult":"Magnitude"}},
        {"name":"minimum","value_type":{"FlowLiteral":"Magnitude"},"range":{"min":1,"max":1000}}],
        "body":{"Logic":{"All":[
            {"Result":{"result":{"Parameter":"result"},"comparison":"AtLeast","value":{"Parameter":"minimum"}}},
            {"Logic":{"Not":{"Always":false}}}
        ]}}}]);
        let last = doc["definitions"].as_array().unwrap().len() - 1;
        doc["definitions"][last]["body"]["Sequence"]["children"][0] = json!({"Action":{
            "id":"probe", "target":{"Literal":{"Target":{"EnemySingle":{"rule":"Nearest"}}}},
            "delivery":{"Literal":{"Delivery":"Instant"}},"effects":[{"Damage":{"id":"damage","tags":[],"damage_type":"True",
                "amount":{"Literal":{"Amount":{"moment":"Live","root":{"Const":{"value":100,"unit":"Magnitude"}}}}}}}],"wait":"EffectsSettled"}});
        doc["definitions"][last]["body"]["Sequence"]["children"][1]["Branch"]["condition"] = json!({"Predicate":{"name":"eligible-hit","arguments":[
            {"name":"result","value":{"Literal":{"FlowResult":{"node_id":"probe","effect_id":"damage","metric":"HpLost"}}}},
            {"name":"minimum","value":{"Literal":{"FlowLiteral":{"Hp":minimum}}}}
        ]}});
        doc["skills"][0]["body"] = json!({"Parallel":{"id":"both","branches":[
            {"Call":{"id":"left","definition":"hit-and-recover","arguments":[]}},
            {"Call":{"id":"right","definition":"hit-and-recover","arguments":[]}}
        ]}});
        let (mut core, mut state, artifact, actor, enemy, _) =
            fixture(&doc.to_string(), "woodsman-authored");
        begin(&mut core, actor, 0);
        core.step_battle_execution_until(&mut state, 5).unwrap();
        assert_eq!(core.units[&enemy].stats.current_health, 800);
        assert_eq!(core.units[&actor].stats.current_health, expected_hp);
        let origins: std::collections::BTreeSet<_> = artifact
            .sources()
            .values()
            .filter(|trace| trace.definition.starts_with("predicates[0]"))
            .filter_map(|trace| trace.calls.get(1).cloned())
            .collect();
        assert!(origins.contains("skills[0].body.branches[0]"));
        assert!(origins.contains("skills[0].body.branches[1]"));
    }
}
