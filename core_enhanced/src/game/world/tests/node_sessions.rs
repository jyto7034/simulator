use super::*;

fn headquarters_payload() -> MapNodePayload {
    MapNodePayload::HeadquartersContact {
        shop_pool_id: Some("headquarters_basic_supplies".to_string()),
        candidate_count: 3,
    }
}

#[test]
fn headquarters_contact_enters_safe_choice_state_with_recruitment_candidates() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::HeadquartersContact,
        "headquarters_contact",
        headquarters_payload(),
    );

    let result = select_and_confirm_map_node(&mut core, player_id, node_id);

    assert!(matches!(
        result,
        BehaviorResult::HeadquartersContactState {
            options,
            recruitment_candidates,
            shop_pool_id: Some(pool_id),
            ..
        } if options.contains(&crate::game::map::HeadquartersContactOption::RecruitEmployee)
            && options.contains(&crate::game::map::HeadquartersContactOption::RequestEmergencySupplies)
            && options.contains(&crate::game::map::HeadquartersContactOption::OpenHeadquartersShop)
            && recruitment_candidates.len() == 3
            && pool_id == "headquarters_basic_supplies"
    ));
    assert!(matches!(core.get_state(), GameState::InNode { .. }));
    assert!(core.state.node_session.is_some());
    assert!(core
        .execute(player_id, PlayerBehavior::CompleteNode)
        .is_err());
}

#[test]
fn headquarters_recruit_employee_completes_node_and_adds_roster_member() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let initial_roster_size = core.roster().unwrap().len();
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::HeadquartersContact,
        "headquarters_contact",
        headquarters_payload(),
    );
    let result = select_and_confirm_map_node(&mut core, player_id, node_id);
    let BehaviorResult::HeadquartersContactState {
        recruitment_candidates,
        ..
    } = result
    else {
        panic!("headquarters node should expose recruitment candidates");
    };
    let candidate_id = recruitment_candidates[0].id.clone();

    let result = core
        .execute(
            player_id,
            PlayerBehavior::RecruitEmployee {
                candidate_id: candidate_id.clone(),
            },
        )
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::EmployeeRecruited {
            candidate_id: id,
            completion,
            ..
        } if id == candidate_id
            && matches!(*completion, BehaviorResult::NodeCompleted { .. })
    ));
    assert_eq!(core.roster().unwrap().len(), initial_roster_size + 1);
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    assert!(core.state.node_session.is_none());
}

#[test]
fn headquarters_shop_uses_headquarters_pool_and_exit_completes_node() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::HeadquartersContact,
        "headquarters_contact",
        headquarters_payload(),
    );
    select_and_confirm_map_node(&mut core, player_id, node_id);

    let result = core
        .execute(player_id, PlayerBehavior::OpenHeadquartersShop)
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::ShopState { shop, .. } if shop.id == "hq_supply_shop"
    ));
    assert!(matches!(core.get_state(), GameState::InShop { .. }));

    let result = core.execute(player_id, PlayerBehavior::ExitShop).unwrap();
    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    assert!(core.state.node_session.is_none());
}

#[test]
fn headquarters_emergency_supplies_completes_node_without_shop_or_recruitment() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let initial_enkephalin = core.get_enkephalin();
    let initial_roster_size = core.roster().unwrap().len();
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::HeadquartersContact,
        "headquarters_contact",
        headquarters_payload(),
    );
    select_and_confirm_map_node(&mut core, player_id, node_id);

    let result = core
        .execute(player_id, PlayerBehavior::RequestEmergencySupplies)
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::EmergencySuppliesGranted {
            enkephalin,
            completion,
            ..
        } if enkephalin == initial_enkephalin + run_policy().headquarters.emergency_enkephalin
            && matches!(*completion, BehaviorResult::NodeCompleted { .. })
    ));
    assert_eq!(core.roster().unwrap().len(), initial_roster_size);
    assert!(matches!(core.get_state(), GameState::ViewingMap));
}

#[test]
fn shop_map_node_enters_shop_and_exit_completes_node() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Shop,
        "shop_general",
        MapNodePayload::Shop {
            shop_id: Some("map_shop".to_string()),
            shop_pool_id: None,
        },
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::ShopState { shop, .. } if shop.id == "map_shop"
    ));
    assert!(matches!(core.get_state(), GameState::InShop { .. }));
    assert!(core.state.node_session.is_some());

    let result = core.execute(player_id, PlayerBehavior::ExitShop).unwrap();

    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    assert!(core.state.node_session.is_none());
}

#[test]
fn reward_map_node_enters_claimable_reward_and_exit_completes_node() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Reward,
        "reward_treasure",
        MapNodePayload::Reward {
            reward_pool_id: Some("default_treasures".to_string()),
        },
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    assert!(matches!(result, BehaviorResult::RewardState { .. }));
    assert!(matches!(core.get_state(), GameState::InReward { .. }));

    let result = core
        .execute(player_id, PlayerBehavior::ClaimReward)
        .unwrap();
    assert!(matches!(result, BehaviorResult::RewardGranted { .. }));
    assert!(matches!(
        core.get_state(),
        GameState::InRewardClaimed { .. }
    ));

    let result = core.execute(player_id, PlayerBehavior::ExitReward).unwrap();
    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    assert!(core.state.node_session.is_none());
}

#[test]
fn reward_claim_failure_does_not_partially_commit_prior_effects() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    core.state.inventory.equipments = crate::game::resources::EquipmentInventory::with_max_slots(0);
    let initial_enkephalin = core.get_enkephalin();

    let reward = core.build_reward_session(
        Uuid::from_u128(0xA700),
        RewardMode::ClaimAll,
        vec![RewardOption {
            id: "atomic_failure_reward".to_string(),
            uuid: Uuid::from_u128(0xA701),
            name: "Atomic Failure".to_string(),
            description: String::new(),
            icon: String::new(),
            effects: vec![
                RewardEffect::GrantEnkephalin { amount: 10 },
                RewardEffect::GrantEquipment {
                    equipment_id: "standard_armor".to_string(),
                },
            ],
        }],
        false,
    );

    let result = core.apply_reward_session(&reward);

    assert!(matches!(result, Err(GameError::InventoryFull)));
    assert_eq!(core.get_enkephalin(), initial_enkephalin);
    assert!(core.state.inventory.equipments.is_empty());
}

#[test]
fn reward_session_reports_skill_fragment_and_research_diffs() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let fragment_id = SkillFragmentId::from("starter_basic_attack_enhancement");

    let reward = core.build_reward_session(
        Uuid::from_u128(0xA710),
        RewardMode::ClaimAll,
        vec![RewardOption {
            id: "fragment_diff_reward".to_string(),
            uuid: Uuid::from_u128(0xA711),
            name: "Fragment Diff".to_string(),
            description: String::new(),
            icon: String::new(),
            effects: vec![
                RewardEffect::GrantSkillFragment {
                    fragment_id: fragment_id.clone(),
                },
                RewardEffect::GrantSkillFragmentResearch {
                    fragment_id: fragment_id.clone(),
                    amount: 1,
                },
            ],
        }],
        false,
    );

    let (_, _, fragment_diffs, research_diffs, experience_diffs) =
        core.apply_reward_session(&reward).unwrap();

    assert_eq!(fragment_diffs.len(), 1);
    assert_eq!(fragment_diffs[0].fragment_id, fragment_id);
    assert_eq!(fragment_diffs[0].count_before, 0);
    assert_eq!(fragment_diffs[0].count_after, 1);
    assert_eq!(research_diffs.len(), 1);
    assert_eq!(research_diffs[0].fragment_id, fragment_id);
    assert_eq!(research_diffs[0].research_progress_before, 0);
    assert_eq!(research_diffs[0].research_progress_after, 1);
    assert!(experience_diffs.is_empty());
}

#[test]
fn reward_session_grant_experience_uses_explicit_alive_roster_target_and_reports_diffs() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_ids = core.roster().unwrap().available_employee_ids();
    assert!(!employee_ids.is_empty());

    let reward = core.build_reward_session(
        Uuid::from_u128(0xA720),
        RewardMode::ClaimAll,
        vec![RewardOption {
            id: "xp_diff_reward".to_string(),
            uuid: Uuid::from_u128(0xA721),
            name: "XP Diff".to_string(),
            description: String::new(),
            icon: String::new(),
            effects: vec![RewardEffect::GrantExperience {
                amount: 6,
                target: ExperienceTargetPolicy::AliveRoster,
            }],
        }],
        false,
    );

    let (_, inventory_diff, fragment_diffs, research_diffs, experience_diffs) =
        core.apply_reward_session(&reward).unwrap();

    assert!(inventory_diff.added.is_empty());
    assert!(fragment_diffs.is_empty());
    assert!(research_diffs.is_empty());
    assert_eq!(experience_diffs.len(), employee_ids.len());
    assert!(experience_diffs
        .iter()
        .all(|diff| diff.experience_before == 0 && diff.experience_after > 0));
}

#[test]
fn reward_map_node_uses_declared_pool_and_avoids_forbidden_rewards() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Reward,
        "reward_treasure",
        MapNodePayload::Reward {
            reward_pool_id: Some("default_treasures".to_string()),
        },
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::RewardState { rewards, .. }
            if rewards.len() == 1
                && rewards[0].id == "map_reward"
    ));
}
