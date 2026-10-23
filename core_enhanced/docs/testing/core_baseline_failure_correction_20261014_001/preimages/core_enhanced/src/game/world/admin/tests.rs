use std::sync::Arc;
use uuid::Uuid;

use super::{AdminCommand, GameCore, ADMIN_NODE_NS};
use crate::game::world::RunState;
use crate::game::{
    behavior::{BehaviorResult, GameError, PlayerBehavior},
    data::{
        consumable_data::{
            ConsumableDatabase, ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata,
            ConsumableTargetPolicy, ConsumableTier,
        },
        equipment_data::{
            EquipmentDatabase, EquipmentMaterialMetadata, EquipmentMaterialType, EquipmentMetadata,
            EquipmentType,
        },
        reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata},
        shop_data::{
            ShopContext, ShopDatabase, ShopMerchandise, ShopMerchandiseEntry, ShopMetadata,
            ShopPoolMetadata,
        },
        skill_fragment_data::{
            SkillFragmentAcquisitionSource, SkillFragmentDatabase, SkillFragmentEffectDef,
            SkillFragmentId, SkillFragmentMetadata, SkillFragmentRarity,
        },
        GameDataBase, GameDataBuilder,
    },
    enums::{RewardMode, RiskLevel},
    map::{
        FacilityTerminalResolution, GameMode, MapNode, MapNodeCategory, MapNodeId, MapNodeKindId,
        MapNodePayload, MapNodeState, MapNodeVisibility, MapProgression, MapSlotId, MapTemplateId,
        MapTerminalNode, NodeSession, RunMap, RunProgression, FACILITY_TEMPLATE_VERSION,
    },
    resources::GameState,
    reward::RewardEffect,
    stats::TriggeredEffects,
};

fn core_with_run(game_data: Arc<GameDataBase>) -> GameCore {
    let mut core = GameCore::new(game_data, 123);
    let boss_id = MapNodeId::new(Uuid::from_u128(0xB055));
    let map = RunMap {
        map_template_id: MapTemplateId::new("standard_final_a"),
        facility_template_version: FACILITY_TEMPLATE_VERSION,
        nodes: vec![MapNode {
            id: boss_id,
            slot_id: MapSlotId::new("final_chamber"),
            kind_id: MapNodeKindId::new("boss"),
            category: MapNodeCategory::Boss,
            state: MapNodeState::Unavailable,
            visibility: MapNodeVisibility::Concealed,
            payload: MapNodePayload::Encounter { encounter_id: None },
            omen: None,
            combat_difficulty: None,
        }],
        start_node_ids: vec![],
        terminals: vec![MapTerminalNode {
            node_id: boss_id,
            resolution: FacilityTerminalResolution::RunFinalBoss,
        }],
    };
    let run_progression = RunProgression::new(123, GameMode::Expedition, 1);
    core.state.run = Some(RunState::new(
        map,
        MapProgression::default(),
        run_progression,
    ));
    core.transition_to(GameState::ViewingMap).unwrap();
    core
}

fn empty_core_with_run() -> GameCore {
    core_with_run(GameDataBuilder::empty().build_arc())
}

fn core_with_admin_map_content() -> GameCore {
    let shop_equipment = EquipmentMetadata {
        id: "admin_shop_armor".to_string(),
        uuid: Uuid::from_u128(10_002),
        name: "Admin Shop Armor".to_string(),
        equipment_type: EquipmentType::Armor,
        rarity: RiskLevel::ZAYIN,
        price: 10,
        allow_duplicate_equip: true,
        bound: false,
        cannot_unequip_reason: "equipment_bound".to_string(),
        triggered_effects: TriggeredEffects::default(),
        ability_activations: vec![],
        weapon_profile: None,
        event_check_modifiers: vec![],
        event_choice_keys: vec![],
    };
    let game_data = GameDataBuilder::empty()
        .with_equipment(vec![shop_equipment])
        .with_shops(ShopDatabase::new_with_pools(
            vec![ShopMetadata {
                id: "admin_shop".to_string(),
                name: "Admin Shop".to_string(),
                uuid: Uuid::from_u128(10_001),
                context: ShopContext::UnknownExploration,
                offer_count: 1,
                default_buy_percent: 100,
                preset_refs: vec![],
                local_entries: vec![ShopMerchandiseEntry {
                    merchandise: ShopMerchandise::Equipment {
                        equipment_id: "admin_shop_armor".to_string(),
                    },
                    quantity: 1,
                    weight: 1,
                    repeatable: false,
                    buy_percent: None,
                    tags: vec![],
                }],
                refresh_policy: None,
                sell_policy: None,
            }],
            vec![ShopPoolMetadata {
                id: "admin_shops".to_string(),
                shop_ids: vec!["admin_shop".to_string()],
            }],
        ))
        .with_rewards(RewardDatabase::new_with_pools(
            vec![RewardMetadata {
                id: "admin_reward".to_string(),
                uuid: Uuid::from_u128(20_001),
                name: "Admin Reward".to_string(),
                description: "Admin reward".to_string(),
                icon: "test".to_string(),
                effects: vec![RewardEffect::GrantEnkephalin { amount: 7 }],
            }],
            vec![RewardPoolMetadata {
                id: "admin_rewards".to_string(),
                reward_ids: vec!["admin_reward".to_string()],
            }],
        ))
        .build_arc();
    core_with_run(game_data)
}

fn core_with_admin_grant_catalog_data() -> GameCore {
    let equipment = EquipmentMetadata {
        id: "fixture_armor".to_string(),
        uuid: Uuid::from_u128(30_001),
        name: "Fixture Armor".to_string(),
        equipment_type: EquipmentType::Armor,
        rarity: RiskLevel::TETH,
        price: 10,
        allow_duplicate_equip: true,
        bound: false,
        cannot_unequip_reason: "equipment_bound".to_string(),
        triggered_effects: TriggeredEffects::default(),
        ability_activations: Vec::new(),
        weapon_profile: None,
        event_check_modifiers: vec![],
        event_choice_keys: vec![],
    };
    let material = EquipmentMaterialMetadata {
        id: "fixture_dust".to_string(),
        uuid: Uuid::from_u128(30_002),
        name: "Fixture Dust".to_string(),
        description: "fixture material".to_string(),
        material_type: EquipmentMaterialType::Generic,
        rarity: RiskLevel::ZAYIN,
        equipment_type: None,
    };
    let consumable = ConsumableMetadata {
        id: "fixture_ration".to_string(),
        uuid: Uuid::from_u128(40_001),
        name: "Fixture Ration".to_string(),
        description: "fixture consumable".to_string(),
        tier: ConsumableTier::Common,
        rarity: RiskLevel::ZAYIN,
        price: 5,
        target_policy: ConsumableTargetPolicy::SingleEmployee,
        duration_policy: ConsumableDurationPolicy::NextCombatNode,
        effect: ConsumableEffect::DeathPrevent,
        live_pool: true,
    };
    let fragment = SkillFragmentMetadata {
        corrosion_class: crate::game::data::skill_fragment_data::FragmentCorrosionClass::Ordinary,
        id: SkillFragmentId::from("fixture_fragment"),
        uuid: Uuid::from_u128(60_001),
        name: "Fixture Fragment".to_string(),
        description: "fixture fragment".to_string(),
        rarity: SkillFragmentRarity::Common,
        equip_limit: crate::game::data::skill_fragment_data::SkillFragmentEquipLimit::OwnedCopies,
        origin: None,
        sources: vec![SkillFragmentAcquisitionSource::RareReward],
        dependencies: Vec::new(),
        compatibility: Default::default(),
        effect: SkillFragmentEffectDef::BasicAttackModifier {
            attack_bonus: 1,
            attack_interval_ms_reduction: 0,
        },
    };
    let game_data = GameDataBuilder::empty()
        .with_equipment_data(Arc::new(EquipmentDatabase::with_materials(
            vec![equipment],
            vec![material],
        )))
        .with_consumable_data(Arc::new(ConsumableDatabase::new(vec![consumable])))
        .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(vec![fragment]))
        .build_arc();
    core_with_run(game_data)
}

fn fixture_state_fingerprint(
    core: &GameCore,
) -> (
    usize,
    Option<MapNodeId>,
    Vec<MapNodeId>,
    Option<NodeSession>,
    Uuid,
) {
    let run = core.state.run.as_ref().unwrap();
    (
        run.map.nodes.len(),
        run.map_progression.current_node_id,
        run.map_progression.available_node_ids.clone(),
        core.state.node_session.clone(),
        core.state.uuid_manager.peek(ADMIN_NODE_NS),
    )
}

#[test]
fn admin_enter_save_point_requires_an_active_run() {
    let mut core = GameCore::new(GameDataBuilder::empty().build_arc(), 123);

    let err = core
        .execute_admin_command(AdminCommand::AdminEnterSavePoint)
        .unwrap_err();

    assert!(matches!(err, GameError::MissingResource("RunState")));
}

#[test]
fn admin_save_point_fixture_uses_authored_slot_and_produces_a_valid_checkpoint() {
    let game_data = crate::game::data::ValidatedGameData::load_live_embedded()
        .expect("embedded live gameplay RON must be valid")
        .into_fixture_database_arc();
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    let start = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .expect("run start should open the HQ briefing");
    let BehaviorResult::RunStartFlowStarted {
        visual_novel: Some(visual_novel),
        ..
    } = start
    else {
        panic!("run start should expose the HQ briefing");
    };
    core.execute(
        player_id,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: visual_novel.session_id,
            current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                visual_novel.current_scene_id,
            ),
        },
    )
    .expect("HQ briefing should advance to starter selection");
    let starter_count = core.run_policy().setup.starter_employee_count;
    let employee_definition_ids = core
        .state
        .starter_candidates
        .iter()
        .take(starter_count)
        .map(|definition| definition.employee_definition_id.clone())
        .collect::<Vec<_>>();
    core.execute(
        player_id,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids,
            loan_copy_ids: Vec::new(),
        },
    )
    .expect("starter selection should create the run");

    let output = core
        .execute_admin_command(AdminCommand::AdminEnterSavePoint)
        .expect("admin SavePoint fixture should enter");

    assert_eq!(output.result_type, "AdminEnteredSavePoint");
    let run = core.state.run.as_ref().expect("run state");
    assert_eq!(run.map.map_template_id.as_str(), "standard_branching_a");
    let save_point_id = run
        .map_progression
        .current_node_id
        .expect("admin SavePoint is current");
    assert!(run.map_progression.available_node_ids.is_empty());
    assert!(!run
        .map_progression
        .completed_node_ids
        .contains(&save_point_id));
    let save_point_node = run.map.node(save_point_id).expect("SavePoint node");
    assert_eq!(save_point_node.slot_id.as_str(), "save_archive");
    assert_eq!(save_point_node.state, MapNodeState::Unavailable);
    assert_eq!(save_point_node.visibility, MapNodeVisibility::Revealed);
    let template = core.facility_template_for_map(&run.map).unwrap();
    assert!(run
        .map_progression
        .validate_checkpoint_state(&run.map, &template));

    core.execute(player_id, PlayerBehavior::CompleteNode)
        .expect("admin SavePoint should complete and save a checkpoint");
    let payload = core
        .state
        .run_checkpoint
        .payload
        .as_ref()
        .expect("SavePoint completion checkpoint");
    core.validate_run_checkpoint_payload(payload)
        .expect("admin SavePoint checkpoint must satisfy the authored facility contract");
}

#[test]
fn admin_enter_maintenance_creates_valid_fixture_state() {
    let mut core = empty_core_with_run();

    let output = core
        .execute_admin_command(AdminCommand::AdminEnterMaintenance)
        .unwrap();

    assert_eq!(output.result_type, "AdminEnteredMaintenance");
    assert!(matches!(core.get_state(), GameState::InNode { .. }));
    let run = core.state.run.as_ref().unwrap();
    let fixture_id = run
        .map_progression
        .current_node_id
        .expect("admin fixture is current");
    assert_eq!(run.map_progression.available_node_ids, vec![fixture_id]);
    assert!(run.map_progression.completed_node_ids.is_empty());
    assert!(run.map.node(fixture_id).is_some_and(|node| {
        node.slot_id.as_str() == "admin_fixture"
            && node.category == MapNodeCategory::Maintenance
            && node.state == MapNodeState::Unavailable
            && node.visibility == MapNodeVisibility::Revealed
    }));
    let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(selected_event["type"], "maintenance");
    assert!(selected_event["maintenance_options"].is_object());
}

#[test]
fn admin_enter_event_uses_live_typed_event_projection() {
    let game_data = crate::game::data::ValidatedGameData::load_live_embedded()
        .expect("embedded live gameplay RON must be valid")
        .into_fixture_database_arc();
    let mut core = core_with_run(game_data);
    let output = core
        .execute_admin_command(AdminCommand::AdminEnterEvent {
            event_id: "white_night_confession_01".to_string(),
        })
        .expect("live Event fixture should enter");

    assert_eq!(output.result_type, "AdminEnteredEvent");
    let snapshot = core.get_run_snapshot_json().expect("snapshot");
    assert_eq!(snapshot["selected_event"]["type"], "event");
    assert_eq!(
        snapshot["selected_event"]["event"]["event_id"],
        "white_night_confession_01"
    );
    assert_eq!(
        snapshot["selected_event"]["event"]["interaction"]["type"],
        "advance"
    );
    assert!(snapshot["action_capabilities"]
        .as_array()
        .is_some_and(|capabilities| capabilities.iter().any(|capability| {
            capability["action"] == "AdvanceEventScene" && capability["enabled"] == true
        })));
}

#[test]
fn failed_admin_shop_entry_does_not_mutate_fixture_state() {
    let mut core = empty_core_with_run();
    let before = fixture_state_fingerprint(&core);

    let err = core
        .execute_admin_command(AdminCommand::AdminEnterShop {
            shop_id: Some("missing_shop".to_string()),
            shop_pool_id: None,
        })
        .unwrap_err();

    assert!(matches!(err, GameError::InvalidStaticData(_)));
    assert_eq!(fixture_state_fingerprint(&core), before);
}

#[test]
fn failed_admin_reward_entry_does_not_mutate_fixture_state() {
    let mut core = empty_core_with_run();
    let before = fixture_state_fingerprint(&core);

    let err = core
        .execute_admin_command(AdminCommand::AdminEnterReward {
            reward_pool_id: Some("missing_rewards".to_string()),
            mode: Some(RewardMode::ClaimAll),
            can_skip: None,
        })
        .unwrap_err();

    assert!(matches!(err, GameError::InvalidStaticData(_)));
    assert_eq!(fixture_state_fingerprint(&core), before);
}

#[test]
fn admin_enter_shop_creates_shop_fixture_snapshot() {
    let mut core = core_with_admin_map_content();

    let output = core
        .execute_admin_command(AdminCommand::AdminEnterShop {
            shop_id: Some("admin_shop".to_string()),
            shop_pool_id: None,
        })
        .unwrap();

    assert_eq!(output.result_type, "AdminEnteredShop");
    assert!(matches!(core.get_state(), GameState::InShop { .. }));
    let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(selected_event["type"], "shop");
    assert_eq!(selected_event["id"], "admin_shop");
}

#[test]
fn admin_enter_reward_applies_requested_mode_to_snapshot() {
    let mut core = core_with_admin_map_content();

    let output = core
        .execute_admin_command(AdminCommand::AdminEnterReward {
            reward_pool_id: Some("admin_rewards".to_string()),
            mode: Some(RewardMode::ChooseOne),
            can_skip: Some(true),
        })
        .unwrap();

    assert_eq!(output.result_type, "AdminEnteredReward");
    assert!(matches!(core.get_state(), GameState::InReward { .. }));
    assert_eq!(output.payload["mode"], "ChooseOne");
    let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(selected_event["type"], "reward");
    assert_eq!(selected_event["mode"], "ChooseOne");
    assert_eq!(selected_event["can_skip"], true);
    let rewards = selected_event["rewards"].as_array().expect("reward array");
    assert!(!rewards.is_empty());
    assert!(rewards[0]["uuid"].is_string());
    assert!(rewards[0]["id"].is_string());
    assert!(rewards[0].get("reward_id").is_none());
}

#[test]
fn admin_enter_headquarters_contact_creates_fixture_snapshot() {
    let mut core = core_with_admin_map_content();

    let output = core
        .execute_admin_command(AdminCommand::AdminEnterHeadquartersContact {
            shop_pool_id: Some("admin_shops".to_string()),
            candidate_count: Some(2),
        })
        .unwrap();

    assert_eq!(output.result_type, "AdminEnteredHeadquartersContact");
    assert!(matches!(core.get_state(), GameState::InNode { .. }));
    let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(selected_event["type"], "headquarters_contact");
}

#[test]
fn admin_grant_missing_equipment_is_rejected_without_inventory_change() {
    let mut core = empty_core_with_run();
    let roots_before = serde_json::json!({
        "inventory": &core.state.inventory,
        "skill_fragments": &core.state.skill_fragments,
        "roster": &core.state.roster,
        "uuid_manager": &core.state.uuid_manager,
        "enkephalin": &core.state.enkephalin,
    });

    let err = core
        .execute_admin_command(AdminCommand::AdminGrantEquipment {
            definition_id: "missing_equipment".to_string(),
            count: 1,
        })
        .unwrap_err();

    assert!(matches!(err, GameError::InvalidStaticData(_)));
    assert_eq!(core.state.inventory.equipments.len(), 0);
    assert_eq!(
        serde_json::json!({
            "inventory": &core.state.inventory,
            "skill_fragments": &core.state.skill_fragments,
            "roster": &core.state.roster,
            "uuid_manager": &core.state.uuid_manager,
            "enkephalin": &core.state.enkephalin,
        }),
        roots_before
    );
}

#[test]
fn admin_grant_late_duplicate_uuid_failure_preserves_all_five_roots() {
    let mut core = core_with_admin_grant_catalog_data();
    let uuid_manager_before_setup = core.state.uuid_manager.clone();
    let setup = core
        .execute_admin_command(AdminCommand::AdminGrantEquipment {
            definition_id: "fixture_armor".to_string(),
            count: 2,
        })
        .unwrap();
    let setup_uuids = setup.payload["item_uuids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|uuid| Uuid::parse_str(uuid.as_str().unwrap()).unwrap())
        .collect::<Vec<_>>();
    core.state
        .inventory
        .equipments
        .remove_item(setup_uuids[0])
        .unwrap();
    core.state.uuid_manager = uuid_manager_before_setup;
    let roots_before = serde_json::json!({
        "inventory": &core.state.inventory,
        "skill_fragments": &core.state.skill_fragments,
        "roster": &core.state.roster,
        "uuid_manager": &core.state.uuid_manager,
        "enkephalin": &core.state.enkephalin,
    });

    let error = core
        .execute_admin_command(AdminCommand::AdminGrantEquipment {
            definition_id: "fixture_armor".to_string(),
            count: 2,
        })
        .unwrap_err();

    assert!(matches!(error, GameError::InvalidAction));
    assert_eq!(
        serde_json::json!({
            "inventory": &core.state.inventory,
            "skill_fragments": &core.state.skill_fragments,
            "roster": &core.state.roster,
            "uuid_manager": &core.state.uuid_manager,
            "enkephalin": &core.state.enkephalin,
        }),
        roots_before
    );
}

#[test]
fn admin_consumable_grants_share_one_definition_stack_and_report_post_quantity() {
    let mut core = core_with_admin_grant_catalog_data();

    let first = core
        .execute_admin_command(AdminCommand::AdminGrantConsumable {
            definition_id: "fixture_ration".to_string(),
            count: 3,
        })
        .unwrap();
    assert_eq!(first.result_type, "AdminGrantedConsumable");
    assert_eq!(
        first.payload["item_uuid"],
        Uuid::from_u128(40_001).to_string()
    );
    assert_eq!(first.payload["quantity"], 3);
    assert_eq!(core.state.inventory.consumables.len(), 1);
    assert_eq!(core.state.inventory.occupied_slot_count(), 1);

    let second = core
        .execute_admin_command(AdminCommand::AdminGrantConsumable {
            definition_id: "fixture_ration".to_string(),
            count: 2,
        })
        .unwrap();
    assert_eq!(second.payload["quantity"], 5);
    assert_eq!(core.state.inventory.consumables.len(), 1);
    assert_eq!(core.state.inventory.occupied_slot_count(), 1);
    assert_eq!(core.state.inventory.reserved_slot_count(), 50);
    assert_eq!(
        core.get_inventory_snapshot_json().unwrap()["consumables"][0]["quantity"],
        5
    );
}

#[test]
fn admin_equipment_grant_crosses_former_cap_and_grows_reserve_without_inventory_full() {
    let mut core = core_with_admin_grant_catalog_data();
    let skill_fragments_before = serde_json::to_value(&core.state.skill_fragments).unwrap();
    let roster_before = serde_json::to_value(&core.state.roster).unwrap();
    let enkephalin_before = serde_json::to_value(&core.state.enkephalin).unwrap();
    let mut expected_uuid_manager = core.state.uuid_manager.clone();
    let expected_item_uuids = (0..51)
        .map(|_| expected_uuid_manager.next_owned_equipment())
        .collect::<Vec<_>>();
    let expected_next_uuid = expected_uuid_manager
        .peek(crate::game::managers::uuid_manager::UuidManager::NS_OWNED_EQUIPMENT);

    let output = core
        .execute_admin_command(AdminCommand::AdminGrantEquipment {
            definition_id: "fixture_armor".to_string(),
            count: 51,
        })
        .unwrap();

    assert_eq!(output.result_type, "AdminGrantedEquipment");
    assert_eq!(
        output.payload["item_uuids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|uuid| Uuid::parse_str(uuid.as_str().unwrap()).unwrap())
            .collect::<Vec<_>>(),
        expected_item_uuids
    );
    assert_eq!(core.state.inventory.equipments.len(), 51);
    assert_eq!(core.state.inventory.occupied_slot_count(), 51);
    assert_eq!(core.state.inventory.reserved_slot_count(), 60);
    assert_eq!(
        serde_json::to_value(&core.state.skill_fragments).unwrap(),
        skill_fragments_before
    );
    assert_eq!(
        serde_json::to_value(&core.state.roster).unwrap(),
        roster_before
    );
    assert_eq!(
        serde_json::to_value(&core.state.enkephalin).unwrap(),
        enkephalin_before
    );
    assert_eq!(
        core.state
            .uuid_manager
            .peek(crate::game::managers::uuid_manager::UuidManager::NS_OWNED_EQUIPMENT),
        expected_next_uuid
    );
}

#[test]
fn admin_dump_grant_catalog_lists_grantable_live_data() {
    let mut core = core_with_admin_grant_catalog_data();

    let output = core
        .execute_admin_command(AdminCommand::AdminDumpGrantCatalog)
        .unwrap();

    assert_eq!(output.result_type, "AdminGrantCatalog");
    assert_eq!(output.payload["schema_version"], 1);
    assert_eq!(output.payload["equipment"][0]["id"], "fixture_armor");
    assert_eq!(output.payload["equipment"][0]["equipment_type"], "Armor");
    assert_eq!(
        output.payload["equipment"][0]["grant_command"],
        "admin_grant_equipment"
    );
    assert_eq!(output.payload["equipment"][0]["amount_mode"], "count");
    assert_eq!(output.payload["consumables"][0]["id"], "fixture_ration");
    assert_eq!(
        output.payload["consumables"][0]["grant_command"],
        "admin_grant_consumable"
    );
    assert!(output.payload["skill_fragments"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["id"] == "fixture_fragment"
            && entry["grant_command"] == "admin_grant_skill_fragment"));
    assert_eq!(
        output.payload["equipment_materials"][0]["grant_command"],
        "admin_grant_equipment_material"
    );
    assert!(output.payload["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["id"] == "fragment_dust"
            && entry["grant_command"] == "admin_grant_fragment_dust"
            && entry["amount_mode"] == "amount"));
    assert!(output.payload["resources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["id"] == "enkephalin"
            && entry["grant_command"] == "admin_set_enkephalin"
            && entry["amount_mode"] == "set"));
}

#[test]
fn admin_zero_amount_grants_are_rejected() {
    let mut core = empty_core_with_run();

    let material_err = core
        .execute_admin_command(AdminCommand::AdminGrantEquipmentMaterial {
            material_id: "equipment_dust".to_string(),
            amount: 0,
        })
        .unwrap_err();
    assert!(matches!(material_err, GameError::InvalidAction));

    let dust_err = core
        .execute_admin_command(AdminCommand::AdminGrantFragmentDust { amount: 0 })
        .unwrap_err();
    assert!(matches!(dust_err, GameError::InvalidAction));
}

#[test]
fn admin_set_enkephalin_updates_resource_snapshot() {
    let mut core = empty_core_with_run();

    let output = core
        .execute_admin_command(AdminCommand::AdminSetEnkephalin { amount: 777 })
        .unwrap();

    assert_eq!(output.result_type, "AdminSetEnkephalin");
    assert_eq!(core.get_enkephalin(), 777);
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["resources"]["enkephalin"], 777);
}

#[test]
fn admin_set_employee_condition_updates_both_axes_and_rejects_out_of_range_values() {
    let mut core = empty_core_with_run();
    let employee_uuid = Uuid::from_u128(0x00C0_11D1_710A);
    core.state.roster.add(crate::game::employee::Employee::new(
        employee_uuid,
        "admin_condition_employee",
        "Admin Condition Employee",
    ));

    let output = core
        .execute_admin_command(AdminCommand::AdminSetEmployeeCondition {
            employee_uuid,
            physical_damage: 5,
            trauma: 100,
        })
        .unwrap();

    assert_eq!(output.result_type, "AdminSetEmployeeCondition");
    assert_eq!(output.payload["physical_damage"], 5);
    assert_eq!(output.payload["trauma"], 100);
    assert_eq!(
        core.state.roster.get(&employee_uuid).unwrap().condition,
        crate::game::employee::EmployeeCondition::new(5, 100)
    );

    let error = core
        .execute_admin_command(AdminCommand::AdminSetEmployeeCondition {
            employee_uuid,
            physical_damage: 7,
            trauma: 0,
        })
        .unwrap_err();
    assert!(matches!(error, GameError::InvalidAction));
    assert_eq!(
        core.state.roster.get(&employee_uuid).unwrap().condition,
        crate::game::employee::EmployeeCondition::new(5, 100)
    );
}
