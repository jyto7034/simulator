use std::sync::Arc;
use uuid::Uuid;

use super::{AdminCommand, GameCore, ADMIN_NODE_NS};
use crate::game::world::RunState;
use crate::game::{
    behavior::{ActionKind, GameError},
    data::{
        artifact_data::{ArtifactDatabase, ArtifactMetadata},
        consumable_data::{
            ConsumableDatabase, ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata,
            ConsumableTargetPolicy, ConsumableTier,
        },
        equipment_data::{
            EquipmentDatabase, EquipmentMaterialMetadata, EquipmentMaterialType, EquipmentMetadata,
            EquipmentType,
        },
        reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata},
        shop_data::{ShopDatabase, ShopMetadata, ShopPoolMetadata, ShopType},
        skill_fragment_data::{
            SkillFragmentAcquisitionSource, SkillFragmentDatabase, SkillFragmentEffectDef,
            SkillFragmentId, SkillFragmentMetadata, SkillFragmentRarity,
        },
        GameDataBase, GameDataBuilder,
    },
    enums::{RewardMode, RiskLevel},
    map::{
        MapNode, MapNodeCategory, MapNodeId, MapNodeKindId, MapNodePayload, MapNodeState,
        MapProgression, NodeSession, RunMap, RunProgression,
    },
    resources::GameState,
    reward::RewardEffect,
    stats::TriggeredEffects,
};

fn core_with_run(game_data: Arc<GameDataBase>) -> GameCore {
    let mut core = GameCore::new(game_data, 123);
    let boss_id = MapNodeId::new(Uuid::from_u128(0xB055));
    let map = RunMap {
        nodes: vec![MapNode {
            id: boss_id,
            depth: 1,
            lane: 0,
            kind_id: MapNodeKindId::new("boss"),
            category: MapNodeCategory::Boss,
            state: MapNodeState::Hidden,
            outgoing: vec![],
            payload: MapNodePayload::Encounter { encounter_id: None },
        }],
        start_node_ids: vec![],
        boss_node_id: boss_id,
    };
    let run_progression = RunProgression::new(123, 1);
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
    let game_data = GameDataBuilder::empty()
        .with_shops(ShopDatabase::new_with_pools(
            vec![ShopMetadata {
                id: "admin_shop".to_string(),
                name: "Admin Shop".to_string(),
                uuid: Uuid::from_u128(10_001),
                shop_type: ShopType::Shop,
                can_reroll: false,
                visible_items: vec![],
                hidden_items: vec![],
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
    let artifact = ArtifactMetadata {
        id: "fixture_artifact".to_string(),
        uuid: Uuid::from_u128(50_001),
        name: "Fixture Artifact".to_string(),
        description: "fixture artifact".to_string(),
        rarity: RiskLevel::HE,
        price: 20,
        triggered_effects: TriggeredEffects::default(),
        ability_activations: Vec::new(),
    };
    let fragment = SkillFragmentMetadata {
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
        .with_equipment_data(Arc::new(EquipmentDatabase::with_materials_and_recipes(
            vec![equipment],
            vec![material],
            Vec::new(),
        )))
        .with_consumable_data(Arc::new(ConsumableDatabase::new(vec![consumable])))
        .with_artifact_data(Arc::new(ArtifactDatabase::new(vec![artifact])))
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
fn admin_enter_support_requires_an_active_run() {
    let mut core = GameCore::new(GameDataBuilder::empty().build_arc(), 123);

    let err = core
        .execute_admin_command(AdminCommand::AdminEnterMaintenance)
        .unwrap_err();

    assert!(matches!(err, GameError::MissingResource("RunState")));
}

#[test]
fn admin_enter_maintenance_creates_valid_fixture_state() {
    let mut core = empty_core_with_run();

    let output = core
        .execute_admin_command(AdminCommand::AdminEnterMaintenance)
        .unwrap();

    assert_eq!(output.result_type, "AdminEnteredMaintenance");
    assert!(matches!(core.get_state(), GameState::InNode { .. }));
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::DismantleEquipment));
    let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(selected_event["type"], "maintenance");
    assert!(selected_event["maintenance_options"].is_object());
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

    let err = core
        .execute_admin_command(AdminCommand::AdminGrantEquipment {
            definition_id: "missing_equipment".to_string(),
            count: 1,
        })
        .unwrap_err();

    assert!(matches!(err, GameError::InvalidStaticData(_)));
    assert_eq!(core.state.inventory.equipments.len(), 0);
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
    assert_eq!(output.payload["artifacts"][0]["id"], "fixture_artifact");
    assert_eq!(output.payload["artifacts"][0]["amount_mode"], "single");
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
