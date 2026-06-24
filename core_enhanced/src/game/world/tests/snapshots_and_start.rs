use super::*;

#[test]
fn selected_shop_snapshot_includes_display_item_metadata() {
    let blade = equipment_meta(0x10, "raw_blade_core", EquipmentType::Weapon);
    let lens = artifact_meta(0x20, "artifact_echo_lens");
    let game_data = game_data_with_display_items(vec![blade.clone()], vec![lens.clone()]);
    let mut core = GameCore::new(game_data, 123);

    core.state.active_node_content = Some(ActiveNodeContent::Shop(ShopSessionState {
        id: "artifact_shop".to_string(),
        name: "Artifact Merchant".to_string(),
        uuid: Uuid::from_u128(0x30),
        shop_type: crate::game::data::shop_data::ShopType::Shop,
        can_reroll: true,
        visible_items: vec![blade.uuid],
        hidden_items: vec![lens.uuid],
    }));

    let selected = core
        .get_selected_event_snapshot_json()
        .unwrap()
        .expect("selected event snapshot");

    assert_eq!(selected["visible_items"][0]["uuid"], json!(blade.uuid));
    assert_eq!(selected["visible_items"][0]["kind"], "equipment");
    assert_eq!(
        selected["visible_items"][0]["definition_id"],
        "raw_blade_core"
    );
    assert_eq!(selected["visible_items"][0]["id"], "raw_blade_core");
    assert_eq!(selected["visible_items"][0]["equipment_type"], "weapon");
    assert!(selected["visible_items"][0]["icon"].is_null());
    assert_eq!(selected["hidden_items"][0]["kind"], "artifact");
    assert_eq!(
        selected["hidden_items"][0]["definition_id"],
        "artifact_echo_lens"
    );
    assert_eq!(selected["hidden_items"][0]["id"], "artifact_echo_lens");
    assert_eq!(
        selected["hidden_items"][0]["description"],
        "artifact_echo_lens description"
    );
    assert!(selected["hidden_items"][0]["icon"].is_null());
    assert_eq!(selected["visible_item_uuids"][0], json!(blade.uuid));
    assert_eq!(selected["hidden_item_uuids"][0], json!(lens.uuid));
}

#[test]
fn selling_owned_artifact_reports_not_removable_instead_of_missing_item() {
    let lens = artifact_meta(0x20, "artifact_echo_lens");
    let game_data = game_data_with_display_items(vec![], vec![lens.clone()]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);

    core.state
        .inventory
        .add_item_owned(
            lens.uuid,
            crate::game::data::Item::Artifact(Arc::new(lens.clone())),
        )
        .unwrap();
    core.state.active_node_content = Some(ActiveNodeContent::Shop(ShopSessionState {
        id: "artifact_shop".to_string(),
        name: "Artifact Merchant".to_string(),
        uuid: Uuid::from_u128(0x30),
        shop_type: crate::game::data::shop_data::ShopType::Shop,
        can_reroll: true,
        visible_items: vec![],
        hidden_items: vec![lens.uuid],
    }));
    core.transition_to(GameState::InShop {
        shop_uuid: Uuid::from_u128(0x30),
    })
    .unwrap();

    let err = core
        .execute(
            player_id,
            PlayerBehavior::SellItem {
                item_uuid: lens.uuid,
            },
        )
        .unwrap_err();

    assert!(matches!(err, GameError::InventoryItemNotRemovable));
}

#[test]
fn game_core_rejects_disallowed_actions_and_updates_allowed_actions_on_state_transition() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);

    assert!(matches!(core.get_state(), GameState::NotStarted));
    assert!(core.is_action_allowed(&PlayerBehavior::StartNewGame));
    assert!(!core.is_action_allowed(&PlayerBehavior::RequestMapData));

    let err = core
        .execute(player_id, PlayerBehavior::RequestMapData)
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));

    let res = core
        .execute(player_id, PlayerBehavior::StartNewGame)
        .unwrap();
    let BehaviorResult::StartNewGame {
        candidates,
        required_count,
    } = res
    else {
        panic!("start should expose starter candidates");
    };
    assert_eq!(required_count, run_policy().setup.starter_employee_count);
    assert_eq!(candidates.len(), 6);
    assert!(matches!(
        core.get_state(),
        GameState::SelectingStarterEmployees
    ));
    assert_eq!(
        core.get_allowed_actions(),
        vec![ActionKind::SelectStarterEmployees]
    );

    let selected_ids = candidates
        .into_iter()
        .take(required_count)
        .map(|candidate| candidate.id)
        .collect::<Vec<_>>();
    let selected = core
        .execute(
            player_id,
            PlayerBehavior::SelectStarterEmployees {
                candidate_ids: selected_ids,
            },
        )
        .unwrap();
    assert!(matches!(
        selected,
        BehaviorResult::StarterEmployeesSelected { .. }
    ));
    assert!(matches!(core.get_state(), GameState::ViewingMap));

    let allowed = core.get_allowed_actions();
    assert!(allowed.contains(&ActionKind::RequestMapData));
    assert!(allowed.contains(&ActionKind::SelectMapNode));
    assert!(allowed.contains(&ActionKind::EquipItem));
    assert!(allowed.contains(&ActionKind::MoveRosterUnit));
}

#[test]
fn start_new_game_creates_starter_employee_roster_and_order() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);

    start_new_game_with_default_starters(&mut core, player_id);

    let roster = core.roster().expect("employee roster exists");
    assert_eq!(roster.len(), run_policy().setup.starter_employee_count);

    let employee_ids = roster.available_employee_ids();
    assert_eq!(
        employee_ids.len(),
        run_policy().setup.starter_employee_count
    );
    let roster_order = core.roster_order().expect("roster order exists");
    for employee_id in employee_ids {
        assert!(roster_order.slot_of(employee_id).is_some());
        let employee = roster.get(&employee_id).unwrap();
        let equipped: Vec<_> = employee.loadout.item_slot.iter().collect();
        assert_eq!(equipped.len(), 1);
        let equipped_ref = equipped[0];
        let owned = core
            .inventory()
            .unwrap()
            .equipments
            .get_item(&equipped_ref.instance_uuid)
            .expect("starter equipment should exist in inventory");
        assert_eq!(owned.meta.id, "standard_armor");
        assert_eq!(owned.equipped_to, Some(employee_id));
    }
}

#[test]
fn employee_roster_snapshot_exposes_status_loadout_and_roster_slot() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_id = core.roster().unwrap().available_employee_ids()[0];

    let snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let employees = snapshot["employees"].as_array().unwrap();
    let employee = employees
        .iter()
        .find(|employee| employee["uuid"] == json!(employee_id))
        .expect("starter employee is exposed in roster snapshot");

    assert_eq!(employee["life_state"], "Alive");
    assert_eq!(employee["availability"], "Available");
    assert_eq!(employee["available_for_combat"], true);
    assert_eq!(employee["trauma"], 0);
    assert!(employee.get("field_position").is_none());
    assert_eq!(employee["roster_slot"], 0);
    let equipped_items = employee["equipped_items"].as_array().unwrap();
    assert_eq!(equipped_items.len(), 1);
    assert_eq!(equipped_items[0]["definition_id"], "standard_armor");
    assert_eq!(
        employee["combat_profile"]["deployment_affinity"],
        "ground_only"
    );
    assert_eq!(
        employee["combat_profile"]["effective_deployment_affinity"],
        "ground_only"
    );
    assert!(snapshot["available_employee_ids"]
        .as_array()
        .unwrap()
        .contains(&json!(employee_id)));
}

#[test]
fn employee_roster_snapshot_surfaces_effective_profile_errors() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_id = core.roster().unwrap().available_employee_ids()[0];
    let stale_item_uuid = Uuid::from_u128(0xEFFE_C710);

    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_id).unwrap();
        employee
            .loadout
            .item_slot
            .equip(
                crate::game::resources::item_slot::EquippedRef {
                    instance_uuid: stale_item_uuid,
                    base_uuid: Uuid::from_u128(0xBADC_0DE),
                    equipment_type: EquipmentType::Weapon,
                },
                true,
            )
            .unwrap();
    }

    let snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let employee = snapshot["employees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|employee| employee["uuid"] == json!(employee_id))
        .expect("starter employee is exposed in roster snapshot");
    let combat_profile = &employee["combat_profile"];

    assert_eq!(
        combat_profile["effective_profile_error"]["code"],
        "inventory_item_not_found"
    );
    assert!(combat_profile["effective_profile_error"]["message"]
        .as_str()
        .unwrap()
        .contains("InventoryItemNotFound"));
    assert_eq!(combat_profile["effective_stats"], Value::Null);
    assert_eq!(combat_profile["effective_weapon_profile"], Value::Null);
    assert_eq!(combat_profile["effective_deployment_affinity"], Value::Null);
}

#[test]
fn run_snapshot_exposes_current_flow_after_start() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let snapshot = core.get_run_snapshot_json().unwrap();

    assert_eq!(snapshot["game_state"], "viewing_map");
    assert_eq!(snapshot["game_state_context"]["type"], "viewing_map");
    assert_eq!(snapshot["run_progression"]["act_index"], 0);
    assert_eq!(
        snapshot["run_progression"]["max_acts"],
        run_policy().setup.default_max_acts
    );
    assert!(snapshot["allowed_actions"]
        .as_array()
        .unwrap()
        .contains(&json!("SelectMapNode")));
    let map_nodes = snapshot["map"]["nodes"].as_array().unwrap();
    assert!(!map_nodes.is_empty());
    assert_eq!(snapshot["map"]["map_template_id"], "act_01_floor_a");
    let mut slot_ids = std::collections::HashSet::new();
    for node in map_nodes {
        let slot_id = node["slot_id"]
            .as_str()
            .expect("map node exposes slot_id");
        assert!(!slot_id.is_empty());
        assert!(slot_ids.insert(slot_id.to_string()));
        assert!(node.get("kind_id").is_some());
        assert!(node.get("category").is_some());
        assert!(node.get("state").is_some());
    }
    let map_edges = snapshot["map"]["edges"].as_array().unwrap();
    assert!(!map_edges.is_empty());
    for edge in map_edges {
        assert!(edge.get("from_node_id").is_some());
        assert!(edge.get("to_node_id").is_some());
        assert!(edge.get("from").is_none());
        assert!(edge.get("to").is_none());
    }
    assert!(map_nodes
        .iter()
        .any(|node| node["state"] == json!("Available")));
    assert!(snapshot["map"].get("available_node_ids").is_none());
    assert!(snapshot["map"].get("completed_node_ids").is_none());
    assert!(snapshot["map_progression"]
        .get("available_node_ids")
        .is_none());
    assert!(snapshot["map_progression"]
        .get("completed_node_ids")
        .is_none());
    assert_eq!(
        snapshot["roster"]["employees"].as_array().unwrap().len(),
        run_policy().setup.starter_employee_count
    );
    assert_eq!(snapshot["current_node_session"], Value::Null);
    assert_eq!(snapshot["resources"]["enkephalin"], 500);
}

#[test]
fn run_snapshot_exposes_current_node_session_after_node_entry() {
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

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let snapshot = core.get_run_snapshot_json().unwrap();

    assert_eq!(snapshot["game_state"], "node_confirm");
    assert_eq!(snapshot["game_state_context"]["node_id"], json!(node_id));
    assert_eq!(snapshot["current_node_session"]["node_id"], json!(node_id));
    assert_eq!(
        snapshot["current_node_session"]["payload"]["Support"]["support_mode"],
        "LimitedChoice"
    );
    assert!(snapshot["allowed_actions"]
        .as_array()
        .unwrap()
        .contains(&json!("ConfirmEnterNode")));
    assert!(snapshot["allowed_actions"]
        .as_array()
        .unwrap()
        .contains(&json!("SelectMapNode")));
}
