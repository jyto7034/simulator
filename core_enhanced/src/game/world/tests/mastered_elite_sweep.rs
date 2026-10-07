use super::*;
use crate::game::behavior::{ActionDisabledReasonCode, MasteredEliteSweepRewardBranchDto};
use crate::game::data::consumable_data::{
    ConsumableDurationPolicy, ConsumableEffect, ConsumableTier,
};
use crate::game::employee::{ActiveConsumableModifier, MAX_PHYSICAL_DAMAGE};

const SWEEP_ENCOUNTER_ID: &str = "elite_risk_encounter";
const SWEEP_ABNORMALITY_ID: &str = "elite_risk_abno";
const SWEEP_SOURCE_ID: &str = "source_elite_risk";
const SWEEP_FRAGMENT_ID: &str = "fragment_elite_risk_abno";

fn prepare_mastered_elite_node(core: &mut GameCore) -> MapNodeId {
    let node_id = force_first_available_node(
        core,
        MapNodeCategory::Combat,
        "combat_elite",
        MapNodePayload::Encounter {
            encounter_id: Some(SWEEP_ENCOUNTER_ID.to_string()),
        },
    );
    let run = core.state.run.as_mut().expect("Endless run");
    if run.map.node(node_id).unwrap().combat_difficulty.is_none() {
        let difficulty = run
            .map
            .nodes
            .iter()
            .find_map(|node| node.combat_difficulty.clone())
            .expect("generated Endless floor must contain combat difficulty authority");
        run.map.node_mut(node_id).unwrap().combat_difficulty = Some(difficulty);
    }
    run.monolith
        .record_natural_victory(SWEEP_ABNORMALITY_ID)
        .unwrap();
    run.monolith.purchase(SWEEP_ABNORMALITY_ID, 3).unwrap();
    for value in [0x5EE2_u128, 0x5EE3] {
        let remanifestation_node_id = MapNodeId::new(Uuid::from_u128(value));
        run.monolith
            .begin_remanifestation(
                remanifestation_node_id,
                SWEEP_ABNORMALITY_ID,
                SWEEP_ENCOUNTER_ID,
            )
            .unwrap();
        run.monolith
            .resolve_remanifestation_victory(remanifestation_node_id, 4)
            .unwrap();
    }
    run.completed_reward_source_ids
        .insert(SWEEP_SOURCE_ID.to_string());
    run.mastered_elite_cycle
        .assigned_source_ids
        .push(SWEEP_SOURCE_ID.to_string());
    node_id
}

fn start_sweep_core(seed: u64) -> (GameCore, Uuid, MapNodeId) {
    let mut core = GameCore::new(game_data_with_pve_encounters(), seed);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);
    let node_id = prepare_mastered_elite_node(&mut core);
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    (core, player_id, node_id)
}

#[test]
fn mastered_elite_node_projects_direct_and_sweep_from_one_core_authority() {
    let (core, _, node_id) = start_sweep_core(0x5EE0_0001);

    let direct = action_capability(&core, ActionKind::ConfirmEnterNode).unwrap();
    let sweep = action_capability(&core, ActionKind::SweepMasteredElite).unwrap();
    assert!(direct.enabled);
    assert!(sweep.enabled);
    let snapshot = core.get_run_snapshot_json().unwrap();
    let resolution = &snapshot["game_state_context"]["mastered_elite_resolution"];
    assert_eq!(resolution["direct_battle"]["enabled"], json!(true));
    assert_eq!(resolution["sweep"]["enabled"], json!(true));
    assert_eq!(snapshot["game_state_context"]["node_id"], json!(node_id));
    assert!(core.state.active_battle.is_none());

    let result = {
        let mut direct_core = core;
        direct_core
            .execute(Uuid::from_u128(1), PlayerBehavior::ConfirmEnterNode)
            .unwrap()
    };
    assert!(matches!(result, BehaviorResult::BattleAdvanced { .. }));
}

#[test]
fn sweep_stages_stable_battle_independent_result_then_commits_every_effect_once() {
    let (mut core, player_id, node_id) = start_sweep_core(0x5EE0_0002);
    let employee_ids = core
        .state
        .roster
        .iter()
        .map(|employee| employee.uuid)
        .collect::<Vec<_>>();
    let dead_id = employee_ids[2];
    core.state
        .roster
        .get_mut(&dead_id)
        .unwrap()
        .condition
        .physical_damage = MAX_PHYSICAL_DAMAGE;
    let duration_employee_id = employee_ids[0];
    core.state
        .roster
        .get_mut(&duration_employee_id)
        .unwrap()
        .active_consumable_modifier = Some(ActiveConsumableModifier {
        source_item_uuid: Uuid::from_u128(0xC05E),
        definition_id: "test_sweep_duration".to_string(),
        name: "Test sweep duration".to_string(),
        tier: ConsumableTier::Common,
        duration_policy: ConsumableDurationPolicy::CombatNodes(2),
        remaining_combat_nodes: 2,
        effect: ConsumableEffect::DeployCostReduction { percent: 5 },
    });
    let enkephalin_before = core.state.enkephalin.amount;
    let roster_before = core.state.roster.clone();
    let profile_before = serde_json::to_value(&core.state.player_profile).unwrap();
    let compendium_before =
        serde_json::to_value(&core.state.player_profile.encounter_compendium).unwrap();
    let map_before = serde_json::to_value(&core.state.run.as_ref().unwrap().map).unwrap();

    let staged = core
        .execute(player_id, PlayerBehavior::SweepMasteredElite {})
        .unwrap();
    let BehaviorResult::MasteredEliteSweepResultStaged { result } = staged else {
        panic!("sweep selection must stage a typed result");
    };
    assert_eq!(result.node_id, node_id);
    assert_eq!(result.reward_source_id, SWEEP_SOURCE_ID);
    assert_eq!(
        result.reward_branch,
        MasteredEliteSweepRewardBranchDto::ReacquireFragment
    );
    assert!(result.enkephalin_after > enkephalin_before);
    let projected_enkephalin_after = result.enkephalin_after;
    assert_eq!(result.employee_experience_diffs.len(), 2);
    assert!(result
        .employee_experience_diffs
        .iter()
        .all(|diff| diff.amount == run_policy().growth.post_battle_survival_xp));
    assert!(result
        .employee_experience_diffs
        .iter()
        .all(|diff| diff.employee_uuid != dead_id));

    assert!(matches!(
        core.get_state(),
        GameState::MasteredEliteSweepResult { sweep_uuid }
            if sweep_uuid == result.sweep_uuid
    ));
    assert!(core.state.active_battle.is_none());
    assert!(core.state.active_node_content.is_none());
    assert_eq!(core.state.enkephalin.amount, enkephalin_before);
    assert_eq!(
        serde_json::to_value(&core.state.roster).unwrap(),
        serde_json::to_value(&roster_before).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&core.state.run.as_ref().unwrap().map).unwrap(),
        map_before
    );
    assert_eq!(
        core.state
            .skill_fragments
            .count(&SkillFragmentId::from(SWEEP_FRAGMENT_ID)),
        0
    );
    assert_eq!(
        action_capability(&core, ActionKind::CompleteMasteredEliteSweep)
            .unwrap()
            .enabled,
        true
    );

    let reconnect_a = core.get_run_snapshot_json().unwrap();
    let reconnect_b = core.get_run_snapshot_json().unwrap();
    assert_eq!(reconnect_a, reconnect_b);
    assert_eq!(
        reconnect_a["selected_event"]["type"],
        json!("mastered_elite_sweep_result")
    );
    assert!(reconnect_a["selected_event"].get("result_stats").is_none());
    assert!(reconnect_a["selected_event"].get("has_event_log").is_none());

    let completed = core
        .execute(player_id, PlayerBehavior::CompleteMasteredEliteSweep {})
        .unwrap();
    let BehaviorResult::MasteredEliteSweepCompleted {
        sweep_uuid,
        completion,
        ..
    } = completed
    else {
        panic!("sweep confirmation must return the typed completion");
    };
    assert_eq!(sweep_uuid, result.sweep_uuid);
    assert!(matches!(*completion, BehaviorResult::NodeCompleted { .. }));
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    assert!(core.state.pending_mastered_elite_sweep.is_none());
    assert!(core.state.active_battle.is_none());
    assert!(core.state.active_node_content.is_none());
    assert_eq!(core.state.enkephalin.amount, projected_enkephalin_after);
    assert_eq!(
        core.state
            .skill_fragments
            .count(&SkillFragmentId::from(SWEEP_FRAGMENT_ID)),
        1
    );
    let run = core.state.run.as_ref().unwrap();
    assert!(run.completed_reward_source_ids.contains(SWEEP_SOURCE_ID));
    assert!(!run
        .mastered_elite_cycle
        .assigned_source_ids
        .iter()
        .any(|source| source == SWEEP_SOURCE_ID));
    assert_eq!(
        run.monolith
            .progress(SWEEP_ABNORMALITY_ID)
            .unwrap()
            .victories,
        3
    );
    assert_eq!(
        run.map.node(node_id).unwrap().state,
        MapNodeState::Completed
    );
    for employee_id in &employee_ids[..2] {
        assert_eq!(
            core.state.roster.get(employee_id).unwrap().experience,
            run_policy().growth.post_battle_survival_xp
        );
        assert_eq!(
            core.state.roster.get(employee_id).unwrap().condition,
            roster_before.get(employee_id).unwrap().condition
        );
    }
    assert_eq!(core.state.roster.get(&dead_id).unwrap().experience, 0);
    assert_eq!(
        core.state
            .roster
            .get(&duration_employee_id)
            .unwrap()
            .active_consumable_modifier
            .as_ref()
            .unwrap()
            .remaining_combat_nodes,
        1
    );
    assert_eq!(
        serde_json::to_value(&core.state.player_profile).unwrap(),
        profile_before
    );
    assert_eq!(
        serde_json::to_value(&core.state.player_profile.encounter_compendium).unwrap(),
        compendium_before
    );

    assert!(matches!(
        core.execute(player_id, PlayerBehavior::CompleteMasteredEliteSweep {}),
        Err(GameError::ActionUnavailable {
            action: ActionKind::CompleteMasteredEliteSweep,
            reason: ActionDisabledReasonCode::ContextIrrelevant,
        })
    ));
    assert_eq!(core.state.enkephalin.amount, projected_enkephalin_after);
}

#[test]
fn sweep_rejects_without_deployable_employee_and_rolls_back_reward_overflow() {
    let (mut disabled, player_id, _) = start_sweep_core(0x5EE0_0003);
    for employee in disabled.state.roster.iter_mut() {
        employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE - 1;
    }
    let before = non_battle_state_without_profile_fingerprint(&disabled);
    let sweep = action_capability(&disabled, ActionKind::SweepMasteredElite).unwrap();
    assert!(!sweep.enabled);
    assert_eq!(
        sweep.disabled_reason_code,
        Some(ActionDisabledReasonCode::NoEligiblePayload)
    );
    assert!(disabled
        .execute(player_id, PlayerBehavior::SweepMasteredElite {})
        .is_err());
    assert_eq!(
        non_battle_state_without_profile_fingerprint(&disabled),
        before
    );

    let (mut overflow, player_id, _) = start_sweep_core(0x5EE0_0004);
    let fragment_id = SkillFragmentId::from(SWEEP_FRAGMENT_ID);
    let fragment = overflow
        .game_data
        .skill_fragment_data
        .get_by_id(&fragment_id)
        .unwrap()
        .clone();
    let _copy_uuid = overflow
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut overflow.state.uuid_manager,
            &overflow.state.skill_fragment_policy,
        )
        .unwrap();
    overflow
        .state
        .skill_fragments
        .add_fragment_dust(u32::MAX)
        .unwrap();
    let before = non_battle_state_without_profile_fingerprint(&overflow);
    assert!(overflow
        .execute(player_id, PlayerBehavior::SweepMasteredElite {})
        .is_err());
    assert_eq!(
        non_battle_state_without_profile_fingerprint(&overflow),
        before
    );
    assert!(matches!(
        overflow.get_state(),
        GameState::NodeConfirm { .. }
    ));
    assert!(overflow.state.pending_mastered_elite_sweep.is_none());
}

#[test]
fn owned_fragment_sweep_uses_repeat_only_and_stale_confirmation_preserves_staged_plan() {
    let (mut core, player_id, _) = start_sweep_core(0x5EE0_0006);
    let fragment_id = SkillFragmentId::from(SWEEP_FRAGMENT_ID);
    let fragment = core
        .game_data
        .skill_fragment_data
        .get_by_id(&fragment_id)
        .unwrap()
        .clone();
    let _copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();

    let staged = core
        .execute(player_id, PlayerBehavior::SweepMasteredElite {})
        .unwrap();
    let BehaviorResult::MasteredEliteSweepResultStaged { result } = staged else {
        panic!("owned fragment sweep must stage a result");
    };
    assert_eq!(
        result.reward_branch,
        MasteredEliteSweepRewardBranchDto::RepeatReward
    );
    assert_eq!(
        result.exclusive_effects,
        vec![RewardEffect::GrantFragmentDust { amount: 5 }]
    );
    assert!(result.skill_fragment_diffs.is_empty());

    core.state.node_session = None;
    let stale_before = non_battle_state_without_profile_fingerprint(&core);
    assert!(core
        .execute(player_id, PlayerBehavior::CompleteMasteredEliteSweep {})
        .is_err());
    assert_eq!(
        non_battle_state_without_profile_fingerprint(&core),
        stale_before
    );
    assert!(core.state.pending_mastered_elite_sweep.is_some());

    core.state.node_session = Some(crate::game::map::NodeSession {
        node_id: result.node_id,
        kind_id: core
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(result.node_id)
            .unwrap()
            .kind_id
            .clone(),
        category: MapNodeCategory::Combat,
        payload: MapNodePayload::Encounter {
            encounter_id: Some(SWEEP_ENCOUNTER_ID.to_string()),
        },
    });
    core.execute(player_id, PlayerBehavior::CompleteMasteredEliteSweep {})
        .unwrap();
    assert_eq!(core.state.skill_fragments.count(&fragment_id), 1);
    assert!(core.state.skill_fragments.fragment_dust() >= 5);
}

#[test]
fn checkpoint_load_after_committed_sweep_rolls_back_all_run_local_effects_together() {
    let (mut core, player_id, node_id) = start_sweep_core(0x5EE0_0007);
    let checkpoint = crate::game::world::state::RunCheckpointPayload::capture(
        &core.state,
        core.state.run.as_ref().unwrap(),
        core.game_data.identity(),
    )
    .unwrap();
    core.state.run_checkpoint.payload = Some(checkpoint);
    let enkephalin_before = core.state.enkephalin.amount;
    let experience_before = core
        .state
        .roster
        .iter()
        .map(|employee| (employee.uuid, employee.experience))
        .collect::<Vec<_>>();

    core.execute(player_id, PlayerBehavior::SweepMasteredElite {})
        .unwrap();
    core.execute(player_id, PlayerBehavior::CompleteMasteredEliteSweep {})
        .unwrap();
    assert_eq!(
        core.state
            .skill_fragments
            .count(&SkillFragmentId::from(SWEEP_FRAGMENT_ID)),
        1
    );
    assert!(core.state.enkephalin.amount > enkephalin_before);

    core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();

    assert_eq!(core.state.run_checkpoint.loads_used, 1);
    assert_eq!(core.state.enkephalin.amount, enkephalin_before);
    assert_eq!(
        core.state
            .skill_fragments
            .count(&SkillFragmentId::from(SWEEP_FRAGMENT_ID)),
        0
    );
    for (employee_uuid, expected_experience) in experience_before {
        assert_eq!(
            core.state.roster.get(&employee_uuid).unwrap().experience,
            expected_experience
        );
    }
    let run = core.state.run.as_ref().unwrap();
    assert_eq!(
        run.map.node(node_id).unwrap().state,
        MapNodeState::Available
    );
    assert!(run
        .mastered_elite_cycle
        .assigned_source_ids
        .iter()
        .any(|source| source == SWEEP_SOURCE_ID));
    assert!(run.completed_reward_source_ids.contains(SWEEP_SOURCE_ID));
    assert!(core.state.pending_mastered_elite_sweep.is_none());
    assert!(matches!(core.get_state(), GameState::ViewingMap));
}

#[test]
fn unavailable_placeholder_removal_then_sweep_reacquires_fresh_copy_without_resetting_mastery() {
    let (mut core, player_id, _) = start_sweep_core(0x5EE0_0005);
    core.execute(player_id, PlayerBehavior::SweepMasteredElite {})
        .unwrap();
    core.execute(player_id, PlayerBehavior::CompleteMasteredEliteSweep {})
        .unwrap();
    let fragment_id = SkillFragmentId::from(SWEEP_FRAGMENT_ID);
    core.state.skill_fragments.add_fragment_dust(1_000).unwrap();
    let copy_uuid = core
        .state
        .skill_fragments
        .copies()
        .find(|copy| copy.definition_id == fragment_id)
        .unwrap()
        .copy_uuid;
    let mastery_before = core.state.run.as_ref().unwrap().monolith.clone();

    let maintenance_node = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: maintenance_node,
        },
    )
    .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let before = serde_json::to_value(&core.state.skill_fragments).unwrap();
    assert!(core
        .execute(
            player_id,
            PlayerBehavior::DismantleSkillFragment {
                copy_uuid,
                confirmed: true
            }
        )
        .is_err());
    assert_eq!(
        serde_json::to_value(&core.state.skill_fragments).unwrap(),
        before
    );
    // This fixture owns a non-active placeholder without a recipe. Exercise the
    // reacquisition selector after exact removal, without inventing a dust recipe.
    core.state.skill_fragments.remove(copy_uuid).unwrap();
    assert_eq!(core.state.skill_fragments.count(&fragment_id), 0);
    assert!(core.state.skill_fragments.get(&copy_uuid).is_none());
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();

    let next_elite = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_elite",
        MapNodePayload::Encounter {
            encounter_id: Some(SWEEP_ENCOUNTER_ID.to_string()),
        },
    );
    core.state
        .run
        .as_mut()
        .unwrap()
        .mastered_elite_cycle
        .assigned_source_ids
        .push(SWEEP_SOURCE_ID.to_string());
    let dust_before_reacquire = core.state.skill_fragments.fragment_dust();
    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: next_elite,
        },
    )
    .unwrap();
    let staged = core
        .execute(player_id, PlayerBehavior::SweepMasteredElite {})
        .unwrap();
    assert!(matches!(
        staged,
        BehaviorResult::MasteredEliteSweepResultStaged {
            result: crate::game::behavior::MasteredEliteSweepResultDto {
                reward_branch: MasteredEliteSweepRewardBranchDto::ReacquireFragment,
                ..
            }
        }
    ));
    core.execute(player_id, PlayerBehavior::CompleteMasteredEliteSweep {})
        .unwrap();

    assert_eq!(core.state.skill_fragments.count(&fragment_id), 1);
    assert_eq!(
        core.state.skill_fragments.fragment_dust(),
        dust_before_reacquire + 3,
        "reacquisition suppresses only the exclusive repeat reward; authored base reward remains"
    );
    assert!(core.state.skill_fragments.get(&copy_uuid).is_none());
    let run = core.state.run.as_ref().unwrap();
    assert_eq!(run.monolith, mastery_before);
    assert!(run.completed_reward_source_ids.contains(SWEEP_SOURCE_ID));
}

#[test]
fn u04_unfinished_sweep_continue_discards_plan_and_confirmed_sweep_remains_committed() {
    fn restore(core: &GameCore, player: Uuid) -> GameCore {
        let checkpoint = core.export_active_run_checkpoint().unwrap().unwrap();
        let mut next = GameCore::new(core.game_data.clone(), 0);
        next.continue_from_active_run_checkpoint(
            player,
            &checkpoint.core_payload_json,
            checkpoint.loads_used,
            checkpoint.run_peak_certified_power,
        )
        .unwrap();
        next
    }
    let (mut core, player, node) = start_sweep_core(0x5EE0_0008);
    let before = restore(&core, player);
    core.execute(player, PlayerBehavior::SweepMasteredElite {})
        .unwrap();
    assert!(core.state.pending_mastered_elite_sweep.is_some());
    let mut restored = restore(&core, player);
    assert!(restored.state.pending_mastered_elite_sweep.is_none());
    assert_eq!(
        restored.get_run_snapshot_json().unwrap(),
        before.get_run_snapshot_json().unwrap()
    );
    assert_eq!(
        json!(restored.state.uuid_manager),
        json!(before.state.uuid_manager)
    );
    restored
        .execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    restored
        .execute(player, PlayerBehavior::SweepMasteredElite {})
        .unwrap();
    restored
        .execute(player, PlayerBehavior::CompleteMasteredEliteSweep {})
        .unwrap();
    let after = restore(&restored, player);
    assert_eq!(
        after.get_run_snapshot_json().unwrap(),
        restored.get_run_snapshot_json().unwrap()
    );
    assert_eq!(
        json!(after.state.inventory),
        json!(restored.state.inventory)
    );
    assert_eq!(
        json!(after.state.skill_fragments),
        json!(restored.state.skill_fragments)
    );
    assert_eq!(
        after
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node)
            .unwrap()
            .state,
        MapNodeState::Completed
    );
}
