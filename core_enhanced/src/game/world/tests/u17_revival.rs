//! Focused owner contracts. Authored public progression is exercised separately.
use super::*;
use crate::game::{material_economy::MaterialPayment, revival::RevivalStage};

fn setup() -> (GameCore, Uuid, MapNodeId, String) {
    setup_with_deadline(false)
}
fn setup_with_deadline(deadline: bool) -> (GameCore, Uuid, MapNodeId, String) {
    let base = GameCore::new(game_data_with_pve_encounters(), 1717);
    let data = base.game_data.clone();
    let mut pve = data.pve_data.as_ref().clone();
    let source = pve
        .reward_sources
        .iter_mut()
        .find(|s| s.reward_source_id == "source_boss_risk")
        .unwrap();
    source.revival.as_mut().unwrap().catalyst_amount = 2;
    let source_id = source.reward_source_id.clone();
    if deadline {
        let encounter = pve
            .encounters
            .iter_mut()
            .find(|e| e.id == "boss_risk_encounter")
            .unwrap();
        let outcome = encounter
            .terminal_outcomes
            .iter_mut()
            .find(|o| o.verdict == crate::game::battle::types::BattleVerdict::Defeat)
            .unwrap();
        outcome.condition =
            crate::game::data::pve_data::PveTerminalConditionData::TimeReached { time_ms: 6000 };
    }
    let data = GameDataBuilder::from_validated(&data)
        .with_pve(pve)
        .build_arc();
    let mut core = GameCore::new(data, 1717);
    let player = Uuid::from_u128(17);
    start_new_game_with_mode_and_default_starters(&mut core, player, GameMode::Expedition);
    core.apply_grant_effects(&[RewardEffect::GrantEquipmentMaterial {
        material_id: "revival_catalyst".into(),
        amount: 2,
    }])
    .unwrap();
    core.state
        .player_profile
        .lifecycle
        .vault
        .materials
        .insert("revival_catalyst".into(), 2);
    let run = core.state.run.as_mut().unwrap();
    run.completed_reward_source_ids.insert(source_id.clone());
    run.revival.grant_first_egg(&source_id).unwrap();
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        MapNodePayload::Encounter {
            encounter_id: Some("normal_corroded_encounter".into()),
        },
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    (core, player, node, source_id)
}
fn hatch(
    node_id: MapNodeId,
    reward_source_id: &str,
    carried: u32,
    warehouse: u32,
) -> PlayerBehavior {
    PlayerBehavior::HatchEncounter {
        node_id,
        reward_source_id: reward_source_id.into(),
        stage: RevivalStage::Stage2,
        payment: vec![MaterialPayment {
            material_id: "revival_catalyst".into(),
            carried,
            warehouse,
        }],
    }
}
fn roots(core: &GameCore) -> Value {
    json!({"snapshot":core.state.run.as_ref().map(|_|core.get_run_snapshot_json().unwrap()), "profile":core.state.player_profile,
        "inventory":core.state.inventory,"uuid":core.state.uuid_manager,"progress":core.state.run.as_ref().map(|r|&r.encounter_progression),"continue":core.export_active_run_checkpoint().unwrap().map(|c|c.core_payload_json)})
}
#[test]
fn u17_hatch_payment_is_explicit_atomic_and_does_not_enter_combat() {
    for (carried, warehouse) in [(2, 0), (1, 1), (0, 2)] {
        let (mut core, player, node, source) = setup();
        let before = roots(&core);
        let quote = core.revival_snapshot().unwrap();
        let option = quote
            .hatch_options
            .iter()
            .find(|o| o.reward_source_id == source)
            .unwrap();
        assert!(option.can_hatch);
        assert_eq!(
            (
                option.catalyst_required,
                option.catalyst_carried,
                option.catalyst_warehouse
            ),
            (2, 2, 2)
        );
        for bad in [
            hatch(node, &source, 1, 0),
            hatch(node, &source, 0, 3),
            hatch(node, "wrong_source", 2, 0),
        ] {
            assert!(core.execute(player, bad).is_err());
            assert_eq!(roots(&core), before);
        }
        let deterrence = core.run_state().unwrap().run_progression.deterrence;
        core.execute(player, hatch(node, &source, carried, warehouse))
            .unwrap();
        assert!(matches!(core.get_state(),GameState::NodeConfirm {node_id,..} if node_id == node));
        assert!(core.state.active_battle.is_none());
        assert!(core.run_state().unwrap().revival.eggs.is_empty());
        assert_eq!(
            core.run_state().unwrap().revival.nodes[&node].stage,
            RevivalStage::Stage2
        );
        assert!(core.run_state().unwrap().abnormality_attempts.is_empty());
        assert_eq!(
            core.run_state().unwrap().run_progression.deterrence,
            deterrence
        );
        assert_eq!(
            core.state
                .inventory
                .equipment_materials
                .amount("revival_catalyst"),
            2 - carried
        );
        assert_eq!(
            core.state
                .player_profile
                .lifecycle
                .vault
                .materials
                .get("revival_catalyst")
                .copied()
                .unwrap_or(0),
            2 - warehouse
        );
        let after = roots(&core);
        assert!(core
            .execute(player, hatch(node, &source, carried, warehouse))
            .is_err());
        assert_eq!(roots(&core), after);
    }
}
#[test]
fn u17_hatch_before_entry_survives_continue_and_cancel() {
    let (mut core, player, node, source) = setup();
    core.execute(player, hatch(node, &source, 1, 1)).unwrap();
    core.execute(player, PlayerBehavior::CancelSelectedNode)
        .unwrap();
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restarted = core_with_profile_for_continue(&core, 1718);
    restarted
        .continue_from_active_run_checkpoint(
            player,
            &saved.core_payload_json,
            saved.loads_used,
            saved.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(
        restarted.run_state().unwrap().revival.nodes[&node].stage,
        RevivalStage::Stage2
    );
    assert_eq!(
        restarted
            .state
            .inventory
            .equipment_materials
            .amount("revival_catalyst"),
        1
    );
    assert_eq!(
        restarted.state.player_profile.lifecycle.vault.materials["revival_catalyst"],
        1
    );
}

#[test]
fn u17_manual_retreat_keeps_hatch_and_two_retreats_leave_final_attempt() {
    let (mut core, player, node, source) = setup();
    core.execute(player, hatch(node, &source, 1, 1)).unwrap();
    let initial = core.run_state().unwrap().run_progression.deterrence;
    for retreat in 1..=2 {
        core.execute(player, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        assert!(matches!(core.get_state(), GameState::InBattle { .. }));
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
        let run = core.run_state().unwrap();
        assert_eq!(run.abnormality_attempt_state(node).retreats_used, retreat);
        assert!(!run.revival.nodes[&node].resolved);
        assert_eq!(
            run.run_progression.deterrence,
            initial - 1,
            "entry cost paid once"
        );
        assert_eq!(
            core.state
                .inventory
                .equipment_materials
                .amount("revival_catalyst"),
            1
        );
        assert_eq!(
            core.state.player_profile.lifecycle.vault.materials["revival_catalyst"],
            1
        );
    }
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let before = roots(&core);
    assert!(
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).is_err()
    );
    assert_eq!(roots(&core), before);
}

#[test]
fn u17_unfinished_normal_entry_then_hatch_rolls_back_all_sources_on_continue() {
    let (mut core, player, node, source) = setup();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(core.hatchable_node(), Some(node));
    core.execute(player, hatch(node, &source, 1, 1)).unwrap();
    assert!(core.run_state().unwrap().revival.eggs.is_empty());
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restarted = core_with_profile_for_continue(&core, 1718);
    restarted
        .continue_from_active_run_checkpoint(
            player,
            &saved.core_payload_json,
            saved.loads_used,
            saved.run_peak_certified_power,
        )
        .unwrap();
    assert!(restarted.run_state().unwrap().revival.nodes.is_empty());
    assert_eq!(
        restarted.run_state().unwrap().revival.eggs.get(&source),
        Some(&RevivalStage::Stage2)
    );
    assert_eq!(
        restarted
            .state
            .inventory
            .equipment_materials
            .amount("revival_catalyst"),
        2
    );
    assert_eq!(
        restarted.state.player_profile.lifecycle.vault.materials["revival_catalyst"],
        2
    );
    assert!(
        matches!(&restarted.run_state().unwrap().map.node(node).unwrap().payload,MapNodePayload::Encounter {encounter_id:Some(id)} if id=="normal_corroded_encounter")
    );
}

#[test]
fn u17_savepoint_before_and_after_hatch_restores_egg_node_and_source_payments() {
    let (mut core, player, node, source) = setup();
    core.save_run_checkpoint_for_test().unwrap();
    core.execute(player, hatch(node, &source, 1, 1)).unwrap();
    core.execute(player, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();
    assert!(core.run_state().unwrap().revival.nodes.is_empty());
    assert_eq!(
        core.run_state().unwrap().revival.eggs.get(&source),
        Some(&RevivalStage::Stage2)
    );
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("revival_catalyst"),
        2
    );
    assert_eq!(
        core.state.player_profile.lifecycle.vault.materials["revival_catalyst"],
        2
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    core.execute(player, hatch(node, &source, 1, 1)).unwrap();
    core.save_run_checkpoint_for_test().unwrap();
    let paid = core.run_state().unwrap().revival.nodes[&node]
        .payment
        .clone();
    core.execute(player, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();
    assert_eq!(core.run_state().unwrap().revival.nodes[&node].payment, paid);
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("revival_catalyst"),
        1
    );
    assert_eq!(
        core.state.player_profile.lifecycle.vault.materials["revival_catalyst"],
        1
    );
}

#[test]
fn u17_actual_defeat_consumes_hatched_node_without_success_or_refund() {
    let (mut core, player, node, source) = setup_with_deadline(true);
    core.execute(player, hatch(node, &source, 1, 1)).unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    // A supported authored deadline reaches defeat through real battle ticks.
    for _ in 0..40 {
        if core.state.active_battle.is_none() {
            break;
        }
        core.advance_active_battle_by(250).unwrap();
    }
    assert!(
        matches!(core.get_state(), GameState::CombatResult { .. }),
        "finite undefended defeat: {:?}",
        core.get_state()
    );
    assert_eq!(
        core.get_selected_event_snapshot_json().unwrap().unwrap()["verdict"],
        "defeat"
    );
    let run = core.run_state().unwrap();
    assert!(run.revival.nodes[&node].resolved);
    assert!(run.revival.eggs.is_empty());
    assert_eq!(run.map.node(node).unwrap().state, MapNodeState::Completed);
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("revival_catalyst"),
        1
    );
    assert_eq!(
        core.state.player_profile.lifecycle.vault.materials["revival_catalyst"],
        1
    );
    assert_eq!(
        core.state
            .inventory
            .equipment_materials
            .amount("part_big_bad_wolf"),
        0
    );
    assert_eq!(
        core.state
            .skill_fragments
            .count(&"fragment_boss_risk_abno".into()),
        0
    );
}

#[test]
fn u17_real_v16_host_save_is_rejected_before_decode_and_preserved() {
    let bytes=include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),"/docs/goals/core_manual_encounter_revival/execution/single_goal/test_parts/legacy_v16/core_payload.json"));
    let original = bytes.to_vec();
    let (source, player, _, _) = setup();
    let mut core = core_with_profile_for_continue(&source, 1718);
    let before = roots(&core);
    for _ in 0..2 {
        let error = GameCore::inspect_checkpoint_requirements(bytes, 64 * 1024 * 1024).unwrap_err();
        assert!(
            matches!(error,GameError::InvalidStaticData(ref message) if message.contains("unsupported read-only continuation version 16"))
        );
        let error = core
            .continue_from_active_run_checkpoint(player, bytes, 0, 0)
            .unwrap_err();
        assert!(
            matches!(error,GameError::InvalidStaticData(ref message) if message.contains("unsupported active-run core payload version 16")),
            "{error:?}"
        );
        assert_eq!(roots(&core), before);
        assert_eq!(bytes.as_slice(), original);
    }
}

#[cfg(feature = "runtime-observation")]
#[test]
fn u17_hatch_native_exact_limits_and_short_retry_preserve_all_roots() {
    use crate::game::battle::runtime_control::{self, RuntimeFaultCode};
    let (source, player, node, id) = setup();
    let before = roots(&source);
    let limits = source.runtime_safety().native_runtime();
    let mut complete = source.clone();
    let window = crate::runtime_observation::begin_lab_window().unwrap();
    runtime_control::run(limits, || complete.execute(player, hatch(node, &id, 1, 1))).unwrap();
    let measured = window.finish().unwrap();
    assert!(measured.work > 1 && measured.clone_slots > 1);
    let expected = roots(&complete);
    for work in [true, false] {
        let mut exact = limits;
        if work {
            exact.max_work = measured.work;
        } else {
            exact.max_clone_slots = usize::try_from(measured.clone_slots).unwrap();
        }
        let mut candidate = source.clone();
        runtime_control::run(exact, || candidate.execute(player, hatch(node, &id, 1, 1))).unwrap();
        assert_eq!(roots(&candidate), expected);
        let mut short = exact;
        if work {
            short.max_work -= 1;
        } else {
            short.max_clone_slots -= 1;
        }
        let mut rejected = source.clone();
        for _ in 0..2 {
            let error =
                runtime_control::run(short, || rejected.execute(player, hatch(node, &id, 1, 1)))
                    .unwrap_err();
            assert!(
                matches!(error,GameError::BattleRuntimeFault(ref fault) if fault.code==if work {RuntimeFaultCode::WorkLimit}else{RuntimeFaultCode::CloneLimit})
            );
            assert_eq!(roots(&rejected), before);
        }
        runtime_control::run(limits, || rejected.execute(player, hatch(node, &id, 1, 1))).unwrap();
        assert_eq!(roots(&rejected), expected);
    }
}
