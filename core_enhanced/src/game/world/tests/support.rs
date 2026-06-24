use super::*;

#[test]
fn save_point_reduces_living_trauma_without_healing_hp_and_saves_checkpoint() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let mut ids = core.roster().unwrap().available_employee_ids();
        ids.sort();
        let roster = core.roster_mut().unwrap();
        roster.get_mut(&ids[0]).unwrap().health.set_current_hp(10);
        roster.get_mut(&ids[0]).unwrap().trauma = 80;
        roster.get_mut(&ids[1]).unwrap().trauma = 5;
        roster.get_mut(&ids[2]).unwrap().life_state = EmployeeLifeState::Dead;
        roster.get_mut(&ids[2]).unwrap().trauma = 50;
    }
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_save_point",
        MapNodePayload::Support {
            support_type: SupportNodeType::SavePoint,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        },
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let support_state = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(matches!(
        support_state,
        BehaviorResult::SupportState {
            support_type: Some(SupportNodeType::SavePoint),
            ..
        }
    ));
    let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(selected_event["support_type"], "SavePoint");
    assert!(selected_event.get("target_candidates").is_none());
    assert!(selected_event.get("selected_employee_uuid").is_none());
    assert!(selected_event.get("selected_medical_treatment").is_none());
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();

    let mut ids = core
        .roster()
        .unwrap()
        .iter()
        .map(|employee| employee.uuid)
        .collect::<Vec<_>>();
    ids.sort();
    let roster = core.roster().unwrap();
    assert_eq!(roster.get(&ids[0]).unwrap().health.current_hp, 10);
    assert_eq!(roster.get(&ids[0]).unwrap().trauma, 68);
    assert_eq!(roster.get(&ids[1]).unwrap().trauma, 5);
    assert_eq!(roster.get(&ids[2]).unwrap().trauma, 50);
    assert!(core.state.run_checkpoint.payload.is_some());
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::LoadRunCheckpoint));
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["run_checkpoint"]["exists"], true);
    assert_eq!(snapshot["run_checkpoint"]["loads_used"], 0);
    assert_eq!(snapshot["run_checkpoint"]["max_loads"], 3);
    assert_eq!(snapshot["run_checkpoint"]["remaining_loads"], 3);
    assert_eq!(snapshot["run_checkpoint"]["can_load"], true);
}

#[test]
fn load_run_checkpoint_restores_visible_run_state_but_not_load_count() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
        employee.trauma = 80;
    }
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_save_point",
        MapNodePayload::Support {
            support_type: SupportNodeType::SavePoint,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        },
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();

    let checkpoint_trauma = core.roster().unwrap().get(&employee_uuid).unwrap().trauma;
    assert_eq!(checkpoint_trauma, 68);
    assert_eq!(core.state.run_checkpoint.loads_used, 0);

    core.state.enkephalin.amount = 999;
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .trauma = 3;
    core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();

    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert_eq!(employee.trauma, checkpoint_trauma);
    assert_eq!(core.state.enkephalin.amount, 500);
    assert_eq!(core.state.run_checkpoint.loads_used, 1);
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["run_checkpoint"]["loads_used"], 1);
    assert_eq!(snapshot["run_checkpoint"]["remaining_loads"], 2);
    assert_eq!(snapshot["run_checkpoint"]["can_load"], true);
}

#[test]
fn load_run_checkpoint_is_limited_to_three_uses_per_run() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_save_point",
        MapNodePayload::Support {
            support_type: SupportNodeType::SavePoint,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        },
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();

    for expected_used in 1..=3 {
        core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
            .unwrap();
        assert_eq!(core.state.run_checkpoint.loads_used, expected_used);
    }

    assert!(!core
        .get_allowed_actions()
        .contains(&ActionKind::LoadRunCheckpoint));
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["run_checkpoint"]["loads_used"], 3);
    assert_eq!(snapshot["run_checkpoint"]["remaining_loads"], 0);
    assert_eq!(snapshot["run_checkpoint"]["can_load"], false);
    assert!(matches!(
        core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
            .unwrap_err(),
        GameError::InvalidAction
    ));
}

#[test]
fn rest_support_node_restores_all_living_employee_trauma() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let mut ids = core.roster().unwrap().available_employee_ids();
        ids.sort();
        let roster = core.roster_mut().unwrap();
        roster.get_mut(&ids[0]).unwrap().trauma = 30;
        roster.get_mut(&ids[1]).unwrap().trauma = 5;
        roster.get_mut(&ids[2]).unwrap().life_state = EmployeeLifeState::Dead;
        roster.get_mut(&ids[2]).unwrap().trauma = 50;
    }
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_rest",
        MapNodePayload::Support {
            support_type: SupportNodeType::Rest,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        },
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();

    let mut ids = core
        .roster()
        .unwrap()
        .iter()
        .map(|e| e.uuid)
        .collect::<Vec<_>>();
    ids.sort();
    let roster = core.roster().unwrap();
    assert_eq!(roster.get(&ids[0]).unwrap().trauma, 20);
    assert_eq!(roster.get(&ids[1]).unwrap().trauma, 0);
    assert_eq!(roster.get(&ids[2]).unwrap().trauma, 50);
}

#[test]
fn limited_choice_support_node_requires_choice_before_completion() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_choice",
        MapNodePayload::Support {
            support_type: SupportNodeType::SavePoint,
            support_mode: SupportNodeMode::LimitedChoice,
            choices: vec![SupportNodeType::SavePoint, SupportNodeType::Rest],
        },
    );

    let result = select_and_confirm_map_node(&mut core, player_id, node_id);
    assert!(matches!(
        result,
        BehaviorResult::SupportState {
            support_mode: SupportNodeMode::LimitedChoice,
            support_type: None,
            ..
        }
    ));
    assert!(matches!(
        core.execute(player_id, PlayerBehavior::CompleteNode)
            .unwrap_err(),
        GameError::InvalidAction
    ));

    let result = core
        .execute(
            player_id,
            PlayerBehavior::ChooseSupport {
                support_type: SupportNodeType::Rest,
            },
        )
        .unwrap();
    assert!(matches!(
        result,
        BehaviorResult::SupportState {
            selected_support_type: Some(SupportNodeType::Rest),
            ..
        }
    ));
    let result = core
        .execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();

    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    assert!(core.state.active_node_content.is_none());
}

#[test]
fn support_choice_does_not_expose_maintenance_actions() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_choice",
        MapNodePayload::Support {
            support_type: SupportNodeType::SavePoint,
            support_mode: SupportNodeMode::LimitedChoice,
            choices: vec![SupportNodeType::SavePoint, SupportNodeType::Rest],
        },
    );

    select_and_confirm_map_node(&mut core, player_id, node_id);
    core.execute(
        player_id,
        PlayerBehavior::ChooseSupport {
            support_type: SupportNodeType::Rest,
        },
    )
    .unwrap();
    assert!(!core
        .get_allowed_actions()
        .contains(&ActionKind::DismantleSkillFragment));
}

#[test]
fn independent_maintenance_node_exposes_maintenance_actions() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );

    let result = select_and_confirm_map_node(&mut core, player_id, node_id);

    assert!(matches!(
        result,
        BehaviorResult::MaintenanceState {
            maintenance_options,
            ..
        } if !maintenance_options.materials.is_empty()
    ));
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::DismantleSkillFragment));
}

#[test]
fn full_choice_support_node_exposes_all_active_support_effects() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_full_choice",
        MapNodePayload::Support {
            support_type: SupportNodeType::SavePoint,
            support_mode: SupportNodeMode::FullChoice,
            choices: vec![],
        },
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::SupportState {
            support_mode: SupportNodeMode::FullChoice,
            choices,
            ..
        } if choices == vec![
            SupportNodeType::SavePoint,
            SupportNodeType::Rest,
        ]
    ));
}
