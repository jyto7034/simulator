use super::*;
use std::collections::BTreeSet;

fn headquarters_payload() -> MapNodePayload {
    MapNodePayload::HeadquartersContact {
        shop_pool_id: Some("headquarters_basic_supplies".to_string()),
        candidate_count: 3,
    }
}

fn recruitment_queue(core: &GameCore) -> Vec<String> {
    core.state
        .run
        .as_ref()
        .expect("run should exist")
        .recruitment
        .queue_definition_ids()
        .map(str::to_string)
        .collect()
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
    let selected_event = core
        .get_selected_event_snapshot_json()
        .expect("selected event serializes")
        .expect("headquarters selected event");
    let candidates = selected_event["recruitment_candidates"]
        .as_array()
        .expect("candidate array");
    assert_eq!(candidates.len(), 3);
    assert!(candidates
        .iter()
        .all(|candidate| candidate["employee_definition_id"].is_string()));
    assert!(candidates
        .iter()
        .all(|candidate| candidate["profile"]["personality"]["risk"].is_string()));
    assert!(candidates
        .iter()
        .all(|candidate| candidate.get("id").is_none()));
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
    let employee_definition_id = recruitment_candidates[0].employee_definition_id.clone();

    let result = core
        .execute(
            player_id,
            PlayerBehavior::RecruitEmployee {
                employee_definition_id: employee_definition_id.clone(),
            },
        )
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::EmployeeRecruited {
            employee_definition_id: id,
            completion,
            ..
        } if id == employee_definition_id
            && matches!(*completion, BehaviorResult::NodeCompleted { .. })
    ));
    assert_eq!(core.roster().unwrap().len(), initial_roster_size + 1);
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    assert!(core.state.node_session.is_none());
}

#[test]
fn headquarters_session_is_fixed_and_recruitment_moves_exactly_one_definition_to_roster() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let original_queue = recruitment_queue(&core);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::HeadquartersContact,
        "headquarters_fixed_session",
        headquarters_payload(),
    );
    let entered = select_and_confirm_map_node(&mut core, player_id, node_id);
    let BehaviorResult::HeadquartersContactState {
        recruitment_candidates,
        ..
    } = entered
    else {
        panic!("headquarters should expose candidates");
    };
    let offered = recruitment_candidates
        .iter()
        .map(|candidate| candidate.employee_definition_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(offered, original_queue[..3]);
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .recruitment
            .active_session()
            .unwrap()
            .employee_definition_ids,
        offered
    );

    let first_snapshot = core.get_selected_event_snapshot_json().unwrap();
    let second_snapshot = core.get_selected_event_snapshot_json().unwrap();
    assert_eq!(
        first_snapshot, second_snapshot,
        "snapshot reads must not reroll"
    );

    let roster_size = core.roster().unwrap().len();
    assert!(core
        .execute(
            player_id,
            PlayerBehavior::RecruitEmployee {
                employee_definition_id: "not_offered".to_string(),
            },
        )
        .is_err());
    assert_eq!(core.roster().unwrap().len(), roster_size);
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .recruitment
            .active_session()
            .unwrap()
            .employee_definition_ids,
        offered
    );

    let selected = offered[1].clone();
    core.execute(
        player_id,
        PlayerBehavior::RecruitEmployee {
            employee_definition_id: selected.clone(),
        },
    )
    .unwrap();
    let expected_queue = original_queue[3..]
        .iter()
        .chain([&offered[0], &offered[2]])
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(recruitment_queue(&core), expected_queue);
    assert!(core
        .roster()
        .unwrap()
        .iter()
        .any(|employee| employee.employee_definition_id == selected));
    core.state
        .run
        .as_ref()
        .unwrap()
        .recruitment
        .validate_exact_ownership(
            &core.game_data.employee_definition_data.definitions,
            core.roster().unwrap(),
        )
        .unwrap();
}

#[test]
fn headquarters_non_recruit_action_returns_candidates_and_checkpoint_restores_queue_order() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let checkpoint_queue = recruitment_queue(&core);
    core.save_run_checkpoint().unwrap();
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::HeadquartersContact,
        "headquarters_checkpoint_queue",
        headquarters_payload(),
    );
    let entered = select_and_confirm_map_node(&mut core, player_id, node_id);
    let BehaviorResult::HeadquartersContactState {
        recruitment_candidates,
        ..
    } = entered
    else {
        panic!("headquarters should expose candidates");
    };
    let offered = recruitment_candidates
        .iter()
        .map(|candidate| candidate.employee_definition_id.clone())
        .collect::<Vec<_>>();

    core.execute(player_id, PlayerBehavior::RequestEmergencySupplies)
        .unwrap();
    let expected_rotated = checkpoint_queue[offered.len()..]
        .iter()
        .chain(offered.iter())
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(recruitment_queue(&core), expected_rotated);

    core.handle_load_run_checkpoint().unwrap();
    assert_eq!(recruitment_queue(&core), checkpoint_queue);
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .recruitment
        .active_session()
        .is_none());
}

#[test]
fn checkpoint_load_rejects_non_exact_recruitment_ownership_without_partial_restore() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    core.save_run_checkpoint().unwrap();
    let queue_before = recruitment_queue(&core);
    let payload = core
        .state
        .run_checkpoint
        .payload
        .as_mut()
        .expect("checkpoint payload");
    let node_id = MapNodeId::new(Uuid::from_u128(0xBAD));
    let removed_definition_id = payload
        .recruitment
        .begin_session(node_id, 1)
        .unwrap()
        .employee_definition_ids[0]
        .clone();
    payload
        .recruitment
        .recruit_from_active_session(&removed_definition_id)
        .unwrap();

    assert!(core.handle_load_run_checkpoint().is_err());
    assert_eq!(recruitment_queue(&core), queue_before);
    assert!(matches!(core.get_state(), GameState::ViewingMap));
}

#[test]
fn headquarters_shop_returns_all_candidates_to_queue_before_shop_exit() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let original_queue = recruitment_queue(&core);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::HeadquartersContact,
        "headquarters_shop_queue",
        headquarters_payload(),
    );
    let entered = select_and_confirm_map_node(&mut core, player_id, node_id);
    let BehaviorResult::HeadquartersContactState {
        recruitment_candidates,
        ..
    } = entered
    else {
        panic!("headquarters should expose candidates");
    };
    let offered = recruitment_candidates
        .iter()
        .map(|candidate| candidate.employee_definition_id.clone())
        .collect::<Vec<_>>();

    core.execute(player_id, PlayerBehavior::OpenHeadquartersShop)
        .unwrap();
    let expected = original_queue[offered.len()..]
        .iter()
        .chain(offered.iter())
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(recruitment_queue(&core), expected);
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .recruitment
        .active_session()
        .is_none());
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
fn checkpoint_payload_restores_exact_persisted_shop_session_page_and_purchase_history() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Shop,
        "shop_checkpoint",
        MapNodePayload::Shop {
            shop_id: Some("map_shop".to_string()),
            shop_pool_id: None,
        },
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let offer_id = core
        .state
        .active_node_content
        .as_ref()
        .unwrap()
        .as_shop()
        .unwrap()
        .offers[0]
        .offer_id;
    core.execute(player_id, PlayerBehavior::PurchaseItem { offer_id })
        .unwrap();
    let saved = core
        .state
        .run
        .as_ref()
        .unwrap()
        .shop_sessions
        .get(&node_id)
        .unwrap()
        .clone();
    assert!(saved.offers.is_empty());
    assert_eq!(saved.purchased_merchandise.len(), 1);
    core.save_run_checkpoint().unwrap();
    core.state.run.as_mut().unwrap().shop_sessions.clear();
    core.state.enkephalin.amount = 1;
    core.state.active_node_content = None;
    core.state.node_session = None;
    core.transition_to(GameState::ViewingMap).unwrap();
    core.handle_load_run_checkpoint().unwrap();

    let restored = core
        .state
        .run
        .as_ref()
        .unwrap()
        .shop_sessions
        .get(&node_id)
        .expect("checkpoint must restore Shop session");
    assert_eq!(restored.id, saved.id);
    assert_eq!(restored.page_index, saved.page_index);
    assert_eq!(restored.refresh_uses, saved.refresh_uses);
    assert_eq!(restored.offers.len(), saved.offers.len());
    assert_eq!(restored.purchased_merchandise, saved.purchased_merchandise);
}

#[test]
fn live_shop_pool_separates_standard_shop_from_endless_floor_four_wandering_merchant() {
    let payload = MapNodePayload::Shop {
        shop_id: None,
        shop_pool_id: Some("default_shops".to_string()),
    };
    let player_id = Uuid::from_u128(1);

    let mut standard = GameCore::new(live_game_data_from_ron(), 123);
    start_new_game_with_mode_and_default_starters(&mut standard, player_id, GameMode::Standard);
    let standard_shop = standard
        .resolve_map_shop(MapNodeId::new(Uuid::from_u128(0x5100)), &payload)
        .unwrap()
        .unwrap();
    assert_eq!(standard_shop.id, "unknown_exploration_shop");
    assert!(standard_shop
        .offers
        .iter()
        .all(|offer| !matches!(offer.merchandise, ShopMerchandise::Monolith { .. })));

    let mut endless = GameCore::new(live_game_data_from_ron(), 124);
    start_new_game_with_mode_and_default_starters(&mut endless, player_id, GameMode::Endless);
    endless
        .state
        .run
        .as_mut()
        .unwrap()
        .run_progression
        .mode_state = crate::game::map::RunProgressionModeState::Endless { floor_index: 3 };
    let elite_ids = endless
        .game_data
        .pve_data
        .encounters
        .iter()
        .filter(|encounter| encounter.encounter_class == PveEncounterClass::Elite)
        .filter_map(|encounter| encounter.primary_abnormality_id())
        .take(2)
        .map(str::to_string)
        .collect::<Vec<_>>();
    assert_eq!(elite_ids.len(), 2);
    for abnormality_id in &elite_ids {
        endless
            .state
            .run
            .as_mut()
            .unwrap()
            .monolith
            .record_natural_victory(abnormality_id)
            .unwrap();
    }
    let wandering = endless
        .resolve_map_shop(MapNodeId::new(Uuid::from_u128(0x5101)), &payload)
        .unwrap()
        .unwrap();
    assert_eq!(wandering.id, "wandering_monolith_shop");
    assert_eq!(wandering.offers.len(), 3);
    assert_eq!(
        wandering
            .offers
            .iter()
            .filter(|offer| matches!(offer.merchandise, ShopMerchandise::Monolith { .. }))
            .count(),
        2
    );
    endless
        .state
        .run
        .as_mut()
        .unwrap()
        .monolith
        .purchase(&elite_ids[0], 3)
        .unwrap();
    let saved_monolith = endless.state.run.as_ref().unwrap().monolith.clone();
    endless.save_run_checkpoint().unwrap();
    endless.state.run.as_mut().unwrap().monolith = Default::default();

    endless.handle_load_run_checkpoint().unwrap();

    assert_eq!(endless.state.run.as_ref().unwrap().monolith, saved_monolith);
}

#[test]
fn typed_monolith_stage_round_trips_checkpoint_continue_and_transaction_rollback() {
    let game_data = live_game_data_from_ron();
    let player_id = Uuid::from_u128(1);
    let mut source = GameCore::new(game_data.clone(), 125);
    start_new_game_with_mode_and_default_starters(&mut source, player_id, GameMode::Endless);

    let (encounter_id, abnormality_id) = source
        .game_data
        .pve_data
        .encounters
        .iter()
        .find_map(|encounter| {
            (encounter.encounter_class == PveEncounterClass::Elite)
                .then(|| {
                    encounter
                        .primary_abnormality_id()
                        .map(|abnormality_id| (encounter.id.clone(), abnormality_id.to_string()))
                })
                .flatten()
        })
        .expect("live data must contain a monolith-eligible Elite encounter");
    let node_id = source
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .nodes
        .iter()
        .find(|node| {
            node.category == MapNodeCategory::Combat
                && node.combat_difficulty.as_ref().is_some_and(|difficulty| {
                    matches!(
                        difficulty.class,
                        crate::game::map::MapNodeCombatDifficultyClass::Ordinary(_)
                    )
                })
        })
        .map(|node| node.id)
        .expect("live Endless map must contain an ordinary Combat node");

    {
        let run = source.state.run.as_mut().unwrap();
        run.monolith
            .record_natural_victory(&abnormality_id)
            .unwrap();
        run.monolith.purchase(&abnormality_id, 3).unwrap();
        let remanifestation = run
            .monolith
            .begin_remanifestation(node_id, &abnormality_id, &encounter_id)
            .unwrap();
        assert_eq!(remanifestation.stage, 2);

        let node = run.map.node_mut(node_id).unwrap();
        node.payload = MapNodePayload::Encounter {
            encounter_id: Some(encounter_id.clone()),
        };
        let difficulty = node.combat_difficulty.as_mut().unwrap();
        difficulty.class = crate::game::map::MapNodeCombatDifficultyClass::Role(
            crate::game::data::run_policy_data::CombatNodeDifficultyRole::MonolithRemanifestation,
        );
        difficulty.monolith_stage = Some(crate::game::map::MonolithRemanifestationStage::Stage2);
    }
    let tuning = source
        .run_policy()
        .resolved_tuning_for_manifested_node(
            source
                .state
                .run
                .as_ref()
                .unwrap()
                .map
                .node(node_id)
                .unwrap()
                .combat_difficulty
                .as_ref()
                .unwrap(),
        )
        .unwrap();
    {
        let difficulty = source
            .state
            .run
            .as_mut()
            .unwrap()
            .map
            .node_mut(node_id)
            .unwrap()
            .combat_difficulty
            .as_mut()
            .unwrap();
        difficulty.target_power = u64::try_from(
            (u128::from(difficulty.floor_power_anchor) * u128::from(tuning.target_power_percent))
                .div_ceil(100),
        )
        .unwrap();
    }

    source.save_run_checkpoint().unwrap();
    let exported = source
        .export_active_run_checkpoint()
        .unwrap()
        .expect("checkpoint must export");
    let document: serde_json::Value = serde_json::from_slice(&exported.core_payload_json).unwrap();
    let node_index = document["checkpoint"]["map"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["id"] == serde_json::json!(node_id))
        .unwrap();
    assert_eq!(
        document["checkpoint"]["map"]["nodes"][node_index]["combat_difficulty"]["monolith_stage"],
        "Stage2"
    );

    let mut legacy_missing_stage = document.clone();
    legacy_missing_stage["checkpoint"]["map"]["nodes"][node_index]["combat_difficulty"]
        .as_object_mut()
        .unwrap()
        .remove("monolith_stage");
    let mut legacy_core = GameCore::new(game_data.clone(), 126);
    assert!(matches!(
        legacy_core.continue_from_active_run_checkpoint(
            Uuid::from_u128(2),
            &serde_json::to_vec(&legacy_missing_stage).unwrap(),
            exported.loads_used,
            exported.run_peak_certified_power,
        ),
        Err(GameError::InvalidStaticData(_))
    ));

    let mut mismatched_stage = document;
    mismatched_stage["checkpoint"]["map"]["nodes"][node_index]["combat_difficulty"]
        ["monolith_stage"] = serde_json::json!("Stage3");
    let mut mismatched_core = GameCore::new(game_data.clone(), 127);
    assert!(matches!(
        mismatched_core.continue_from_active_run_checkpoint(
            Uuid::from_u128(3),
            &serde_json::to_vec(&mismatched_stage).unwrap(),
            exported.loads_used,
            exported.run_peak_certified_power,
        ),
        Err(GameError::InvalidStaticData(_))
    ));

    let mut continued = GameCore::new(game_data, 128);
    continued
        .continue_from_active_run_checkpoint(
            Uuid::from_u128(4),
            &exported.core_payload_json,
            exported.loads_used,
            exported.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(
        continued
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node_id)
            .unwrap()
            .combat_difficulty
            .as_ref()
            .unwrap()
            .monolith_stage,
        Some(crate::game::map::MonolithRemanifestationStage::Stage2)
    );

    source
        .state
        .run
        .as_mut()
        .unwrap()
        .map
        .node_mut(node_id)
        .unwrap()
        .combat_difficulty
        .as_mut()
        .unwrap()
        .monolith_stage = Some(crate::game::map::MonolithRemanifestationStage::Stage3);
    source.handle_load_run_checkpoint().unwrap();
    assert_eq!(
        source
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node_id)
            .unwrap()
            .combat_difficulty
            .as_ref()
            .unwrap()
            .monolith_stage,
        Some(crate::game::map::MonolithRemanifestationStage::Stage2)
    );

    let rollback = source.capture_active_run_transaction_rollback().unwrap();
    {
        let run = source.state.run.as_mut().unwrap();
        run.map
            .node_mut(node_id)
            .unwrap()
            .combat_difficulty
            .as_mut()
            .unwrap()
            .monolith_stage = Some(crate::game::map::MonolithRemanifestationStage::Stage3);
        run.monolith
            .remanifestation_nodes
            .get_mut(&node_id)
            .unwrap()
            .stage = 3;
    }
    source.restore_active_run_transaction_rollback(rollback);
    let restored_run = source.state.run.as_ref().unwrap();
    assert_eq!(
        restored_run
            .map
            .node(node_id)
            .unwrap()
            .combat_difficulty
            .as_ref()
            .unwrap()
            .monolith_stage,
        Some(crate::game::map::MonolithRemanifestationStage::Stage2)
    );
    assert_eq!(
        restored_run
            .monolith
            .remanifestation(node_id)
            .unwrap()
            .stage,
        2
    );
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
    let initial_enkephalin = core.get_enkephalin();
    let initial_equipment_uuids = core
        .state
        .inventory
        .equipments
        .iter()
        .map(|equipment| equipment.instance_uuid)
        .collect::<BTreeSet<_>>();
    let initial_reserved_slot_count = core.state.inventory.reserved_slot_count();

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
                    equipment_id: "missing_atomic_failure_equipment".to_string(),
                },
            ],
        }],
        false,
    );

    let result = core.apply_reward_session(&reward);

    assert!(matches!(result, Err(GameError::InvalidStaticData(_))));
    assert_eq!(core.get_enkephalin(), initial_enkephalin);
    assert_eq!(
        core.state
            .inventory
            .equipments
            .iter()
            .map(|equipment| equipment.instance_uuid)
            .collect::<BTreeSet<_>>(),
        initial_equipment_uuids
    );
    assert_eq!(
        core.state.inventory.reserved_slot_count(),
        initial_reserved_slot_count
    );
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
fn reward_consumable_quantity_uses_added_then_updated_definition_stack_diffs() {
    let consumable = consumable_meta(
        0xA730,
        "reward_stack_tonic",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 10 },
    );
    let mut core = GameCore::new(game_data_with_consumables(vec![consumable]), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let reward = |core: &GameCore, stage_uuid, reward_uuid, quantity| {
        core.build_reward_session(
            stage_uuid,
            RewardMode::ClaimAll,
            vec![RewardOption {
                id: format!("reward_stack_{quantity}"),
                uuid: reward_uuid,
                name: "Stack Reward".to_string(),
                description: String::new(),
                icon: String::new(),
                effects: vec![RewardEffect::GrantConsumable {
                    consumable_id: "reward_stack_tonic".to_string(),
                    quantity,
                }],
            }],
            false,
        )
    };

    let first = reward(&core, Uuid::from_u128(0xA731), Uuid::from_u128(0xA732), 3);
    let (_, first_diff, _, _, _) = core.apply_reward_session(&first).unwrap();
    assert_eq!(first_diff.added.len(), 1);
    assert!(first_diff.updated.is_empty());
    let crate::game::resources::InventoryItemDto::Consumable(first_stack) = &first_diff.added[0]
    else {
        panic!("first grant must add a consumable stack");
    };
    assert_eq!(first_stack.quantity, 3);

    let second = reward(&core, Uuid::from_u128(0xA733), Uuid::from_u128(0xA734), 2);
    let (_, second_diff, _, _, _) = core.apply_reward_session(&second).unwrap();
    assert!(second_diff.added.is_empty());
    assert_eq!(second_diff.updated.len(), 1);
    let crate::game::resources::InventoryItemDto::Consumable(second_stack) =
        &second_diff.updated[0]
    else {
        panic!("repeat grant must update the consumable stack");
    };
    assert_eq!(second_stack.quantity, 5);
    assert_eq!(core.state.inventory.consumables.len(), 1);
    assert_eq!(core.state.inventory.reserved_slot_count(), 50);
}

#[test]
fn reward_session_grant_experience_uses_explicit_alive_roster_target_and_reports_diffs() {
    let mut core = GameCore::new(game_data_with_map_content(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
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
                amount: 7,
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
    let mut expected_employee_ids = employee_ids;
    expected_employee_ids.sort_by_key(|uuid| uuid.as_u128());
    assert_eq!(
        experience_diffs
            .iter()
            .map(|diff| diff.employee_uuid)
            .collect::<Vec<_>>(),
        expected_employee_ids
    );
    assert_eq!(
        experience_diffs
            .iter()
            .map(|diff| diff.amount)
            .collect::<Vec<_>>(),
        vec![3, 2, 2]
    );
    assert!(experience_diffs.iter().all(|diff| {
        diff.level_before == 1
            && diff.level_after == 1
            && diff.experience_before == 0
            && diff.experience_after == diff.amount
    }));
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
