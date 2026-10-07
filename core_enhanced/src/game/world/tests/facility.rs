use super::*;
use crate::game::behavior::{
    ActionCapabilityDto, ActionDisabledReasonCode, ActionScope, MaintenanceTargetKind,
};
use crate::game::data::Item;

fn maintenance_action_kinds() -> [ActionKind; 5] {
    [
        ActionKind::SelectSkillFragmentForm,
        ActionKind::AwakenSkillFragment,
        ActionKind::DismantleSkillFragment,
        ActionKind::DismantleEquipment,
        ActionKind::EnhanceEquipment,
    ]
}

#[test]
fn save_point_reduces_living_trauma_and_saves_checkpoint() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let mut ids = core.roster().unwrap().combat_eligible_employee_ids();
        ids.sort();
        let roster = core.roster_mut().unwrap();
        roster.get_mut(&ids[0]).unwrap().condition = EmployeeCondition::new(5, 80);
        roster.get_mut(&ids[1]).unwrap().condition.trauma = 100;
        roster.get_mut(&ids[2]).unwrap().condition =
            EmployeeCondition::new(MAX_PHYSICAL_DAMAGE, 50);
    }
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );

    let psychology_before = core
        .roster()
        .unwrap()
        .iter()
        .map(|employee| (employee.uuid, employee.psychology.clone()))
        .collect::<std::collections::HashMap<_, _>>();
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let BehaviorResult::NodePreview {
        trauma_recovery_preview: Some(preview),
        ..
    } = preview
    else {
        panic!("SavePoint NodePreview must include authoritative recovery projection");
    };
    assert_eq!(
        preview.recovery,
        crate::game::data::run_policy_data::TraumaRecovery::CurrentPercent { percent: 15 }
    );
    assert!(preview.physical_damage_unchanged);
    assert!(preview.replaces_run_checkpoint);
    assert_eq!(preview.employees.len(), 3);
    let save_point_state = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(matches!(
        save_point_state,
        BehaviorResult::SavePointState { .. }
    ));
    let selected_event = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(selected_event["type"], "save_point");
    assert_eq!(selected_event["node_id"], node_id.0.to_string());
    assert_eq!(
        selected_event["recovery_preview"]["recovery"]["CurrentPercent"]["percent"],
        15
    );
    let completion = core
        .execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();
    let BehaviorResult::NodeCompleted { .. } = completion else {
        panic!("SavePoint should return to planning");
    };

    let mut ids = core
        .roster()
        .unwrap()
        .iter()
        .map(|employee| employee.uuid)
        .collect::<Vec<_>>();
    ids.sort();
    let roster = core.roster().unwrap();
    assert_eq!(roster.get(&ids[0]).unwrap().condition.trauma, 68);
    assert_eq!(roster.get(&ids[0]).unwrap().condition.physical_damage, 5);
    assert_eq!(roster.get(&ids[1]).unwrap().condition.trauma, 85);
    assert_eq!(roster.get(&ids[2]).unwrap().condition.trauma, 50);
    for employee in roster.iter() {
        assert_eq!(
            &employee.psychology,
            psychology_before.get(&employee.uuid).unwrap(),
            "SavePoint recovery must not produce Psychology state"
        );
    }
    assert!(core.state.run_checkpoint.payload.is_some());
    assert!(action_capability_enabled(
        &core,
        ActionKind::LoadRunCheckpoint
    ));
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["run_checkpoint"]["exists"], true);
    assert_eq!(snapshot["run_checkpoint"]["loads_used"], 0);
    assert_eq!(snapshot["run_checkpoint"]["max_loads"], 3);
    assert_eq!(snapshot["run_checkpoint"]["remaining_loads"], 3);
    assert!(snapshot["run_checkpoint"].get("can_load").is_none());

    let trauma_after_completion = core
        .roster()
        .unwrap()
        .iter()
        .map(|employee| (employee.uuid, employee.condition.trauma))
        .collect::<std::collections::HashMap<_, _>>();
    assert!(core
        .execute(player_id, PlayerBehavior::CompleteNode)
        .is_err());
    for employee in core.roster().unwrap().iter() {
        assert_eq!(
            employee.condition.trauma, trauma_after_completion[&employee.uuid],
            "SavePoint completion must apply recovery exactly once"
        );
    }

    core.handle_load_run_checkpoint().unwrap();
}

#[test]
fn checkpoint_restores_inventory_reserve_high_water_and_consumable_quantity() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let consumable = consumable_meta(
        0xC0DE,
        "checkpoint_tonic",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 10 },
    );

    for index in 1..=58_u128 {
        core.state
            .inventory
            .add_item_owned(
                Uuid::from_u128(0xE000 + index),
                Item::Equipment(Arc::new(equipment_meta(
                    0xE100 + index,
                    &format!("checkpoint_equipment_{index}"),
                    EquipmentType::Accessory,
                ))),
            )
            .unwrap();
    }
    core.state
        .inventory
        .add_consumable(Arc::new(consumable.clone()), 7)
        .unwrap();
    assert_eq!(core.state.inventory.occupied_slot_count(), 65);
    assert_eq!(core.state.inventory.reserved_slot_count(), 70);
    core.save_run_checkpoint_for_test().unwrap();

    for index in 1..=20_u128 {
        core.state
            .inventory
            .remove_item(Uuid::from_u128(0xE000 + index))
            .unwrap();
    }
    core.state
        .inventory
        .consumables
        .remove_quantity("checkpoint_tonic", 6)
        .unwrap();
    assert_eq!(core.state.inventory.occupied_slot_count(), 45);
    assert_eq!(core.state.inventory.reserved_slot_count(), 70);

    core.handle_load_run_checkpoint().unwrap();
    assert_eq!(core.state.inventory.occupied_slot_count(), 65);
    assert_eq!(core.state.inventory.reserved_slot_count(), 70);
    assert_eq!(
        core.state
            .inventory
            .consumables
            .get_by_definition_id("checkpoint_tonic")
            .unwrap()
            .quantity,
        7
    );
}

#[test]
fn map_node_entered_trigger_opens_global_visual_novel_overlay() {
    let mut triggers = test_hq_briefing_visual_novel_trigger_database().triggers;
    triggers.push(VisualNovelTrigger {
        id: VisualNovelTriggerId::new("save_point_map_vn"),
        source: VisualNovelTriggerSource::MapNodeEntered {
            category: MapNodeCategory::SavePoint,
            kind_id: MapNodeKindId::new("save_point"),
        },
        vn_id: VisualNovelId::new("hq_briefing_standard_intro"),
        repeat: VisualNovelTriggerRepeat::OncePerNode,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        priority: 50,
        blocking: true,
    });
    let game_data = game_data_with_visual_novel_triggers(
        test_game_data_builder().build_arc(),
        VisualNovelTriggerDatabase::new(triggers),
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );

    select_and_confirm_map_node(&mut core, player_id, node_id);

    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["visual_novel"]["source"]["type"], "map");
    assert_eq!(
        snapshot["visual_novel"]["vn_id"],
        "hq_briefing_standard_intro"
    );
    assert_eq!(snapshot["visual_novel"]["current_scene_id"], "start");
}

#[test]
fn visual_novel_triggers_use_priority_then_queue_behind_active_session() {
    let mut triggers = test_hq_briefing_visual_novel_trigger_database().triggers;
    triggers.push(VisualNovelTrigger {
        id: VisualNovelTriggerId::new("save_point_low_priority_vn"),
        source: VisualNovelTriggerSource::MapNodeEntered {
            category: MapNodeCategory::SavePoint,
            kind_id: MapNodeKindId::new("save_point"),
        },
        vn_id: VisualNovelId::new("hq_briefing_standard_intro"),
        repeat: VisualNovelTriggerRepeat::OncePerNode,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        priority: 10,
        blocking: true,
    });
    triggers.push(VisualNovelTrigger {
        id: VisualNovelTriggerId::new("save_point_high_priority_vn"),
        source: VisualNovelTriggerSource::MapNodeEntered {
            category: MapNodeCategory::SavePoint,
            kind_id: MapNodeKindId::new("save_point"),
        },
        vn_id: VisualNovelId::new("hq_briefing_endless_intro"),
        repeat: VisualNovelTriggerRepeat::OncePerNode,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        priority: 20,
        blocking: true,
    });
    let game_data = game_data_with_visual_novel_triggers(
        test_game_data_builder().build_arc(),
        VisualNovelTriggerDatabase::new(triggers),
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );

    select_and_confirm_map_node(&mut core, player_id, node_id);

    let active = core
        .state
        .visual_novel
        .active
        .as_ref()
        .expect("higher-priority trigger should become active");
    assert_eq!(active.vn_id.as_str(), "hq_briefing_endless_intro");
    assert_eq!(core.state.visual_novel.queue.len(), 1);
    assert_eq!(
        core.state.visual_novel.queue[0].vn_id.as_str(),
        "hq_briefing_standard_intro"
    );

    let active_session_id = active.session_id.clone();
    core.execute(
        player_id,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: active_session_id,
            current_scene_id: VisualNovelSceneId::new("start"),
        },
    )
    .unwrap();

    let promoted = core
        .state
        .visual_novel
        .active
        .as_ref()
        .expect("queued VN should promote after active VN completes");
    assert_eq!(promoted.vn_id.as_str(), "hq_briefing_standard_intro");
    assert!(core.state.visual_novel.queue.is_empty());
}

#[test]
fn checkpoint_load_does_not_reuse_preserved_visual_novel_session_identity() {
    let mut triggers = test_hq_briefing_visual_novel_trigger_database().triggers;
    triggers.push(VisualNovelTrigger {
        id: VisualNovelTriggerId::new("checkpoint_rewind_nonblocking_vn"),
        source: VisualNovelTriggerSource::MapNodeEntered {
            category: MapNodeCategory::SavePoint,
            kind_id: MapNodeKindId::new("save_point"),
        },
        vn_id: VisualNovelId::new("hq_briefing_standard_intro"),
        repeat: VisualNovelTriggerRepeat::OncePerNode,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        priority: 50,
        blocking: false,
    });
    let game_data = game_data_with_visual_novel_triggers(
        test_game_data_builder().build_arc(),
        VisualNovelTriggerDatabase::new(triggers),
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    core.save_run_checkpoint_for_test().unwrap();

    let event =
        crate::game::world::visual_novel_triggers::VisualNovelTriggerEvent::MapNodeEntered {
            node_id,
            category: MapNodeCategory::SavePoint,
            kind_id: MapNodeKindId::new("save_point"),
        };
    assert_eq!(
        core.evaluate_visual_novel_triggers(event.clone()).unwrap(),
        1
    );
    let preserved_session_id = core
        .state
        .visual_novel
        .active
        .as_ref()
        .expect("nonblocking VN should remain top-level active")
        .session_id
        .clone();

    core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();
    assert_eq!(core.evaluate_visual_novel_triggers(event).unwrap(), 1);

    let queued_session_id = core
        .state
        .visual_novel
        .queue
        .first()
        .expect("rewound trigger should queue behind the preserved active session")
        .session_id
        .clone();
    assert_ne!(
        queued_session_id, preserved_session_id,
        "checkpoint load must not rewind the identity stream for top-level VN sessions"
    );

    core.execute(
        player_id,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: preserved_session_id.clone(),
            current_scene_id: VisualNovelSceneId::new("start"),
        },
    )
    .unwrap();
    let promoted = core
        .state
        .visual_novel
        .active
        .as_ref()
        .expect("queued checkpoint-replayed session should promote");
    assert_eq!(promoted.session_id, queued_session_id);
    assert!(core.state.visual_novel.queue.is_empty());

    assert!(matches!(
        core.execute(
            player_id,
            PlayerBehavior::AdvanceVisualNovel {
                session_id: preserved_session_id,
                current_scene_id: VisualNovelSceneId::new("start"),
            },
        ),
        Err(GameError::InvalidAction)
    ));
    assert_eq!(
        core.state
            .visual_novel
            .active
            .as_ref()
            .expect("stale command must not remove promoted session")
            .session_id,
        queued_session_id
    );
    assert!(core.state.visual_novel.queue.is_empty());
}

#[test]
fn load_run_checkpoint_restores_visible_state_and_anchor_but_not_load_count_or_peak() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
        employee.condition.physical_damage = 4;
        employee.condition.trauma = 80;
        employee.psychology.trust_score = 7;
    }
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();

    let checkpoint_trauma = core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .condition
        .trauma;
    let checkpoint_event_aptitude = core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .profile
        .event_aptitude;
    assert_eq!(checkpoint_trauma, 68);
    assert_eq!(
        core.roster()
            .unwrap()
            .get(&employee_uuid)
            .unwrap()
            .condition
            .physical_damage,
        4,
        "SavePoint must not heal physical damage"
    );
    assert_eq!(
        core.state
            .run_checkpoint
            .payload
            .as_ref()
            .unwrap()
            .run_progression
            .game_mode,
        GameMode::Standard
    );
    assert_eq!(core.state.run_checkpoint.loads_used, 0);

    let power_model_version = core
        .game_data
        .run_policy
        .combat_difficulty
        .power_model_version;
    let checkpoint_anchor = core
        .state
        .run_checkpoint
        .payload
        .as_ref()
        .unwrap()
        .current_floor_power_anchor;
    core.state.run_power_authority.power_model_version = power_model_version;
    core.state.run_power_authority.run_peak_certified_power = 900;
    core.state.run_power_authority.current_floor_power_anchor = 800;

    core.state.enkephalin.amount = 999;
    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_uuid).unwrap();
        employee.condition.physical_damage = 1;
        employee.condition.trauma = 3;
        employee.profile.event_aptitude = EventAptitude::Analysis;
        employee.psychology.trust_score = 50;
    }
    core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();

    let employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert_eq!(employee.condition.physical_damage, 4);
    assert_eq!(employee.condition.trauma, checkpoint_trauma);
    assert_eq!(employee.profile.event_aptitude, checkpoint_event_aptitude);
    assert_eq!(employee.psychology.trust_score, 7);
    assert!(employee.psychology.memories.is_empty());
    assert_eq!(core.state.enkephalin.amount, 500);
    assert_eq!(
        core.state.run.as_ref().unwrap().run_progression.game_mode,
        GameMode::Standard
    );
    assert_eq!(core.state.run_checkpoint.loads_used, 1);
    assert_eq!(
        core.state.run_power_authority.power_model_version,
        power_model_version
    );
    assert_eq!(core.state.run_power_authority.run_peak_certified_power, 900);
    assert_eq!(
        core.state.run_power_authority.current_floor_power_anchor,
        checkpoint_anchor
    );
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["run_progression"]["game_mode"], "Standard");
    assert_eq!(snapshot["run_checkpoint"]["loads_used"], 1);
    assert_eq!(snapshot["run_checkpoint"]["remaining_loads"], 2);
    assert!(snapshot["run_checkpoint"].get("can_load").is_none());
}

#[test]
fn active_run_checkpoint_round_trip_preserves_monotonic_loads_without_local_profile() {
    let game_data = empty_game_data();
    let mut source = GameCore::new(game_data.clone(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut source, player_id);
    let node_id = force_first_available_node(
        &mut source,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    source
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    source
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    source
        .execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();
    source
        .validate_run_checkpoint_payload(source.state.run_checkpoint.payload.as_ref().unwrap())
        .unwrap();
    source
        .execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();

    let power_model_version = source
        .game_data
        .run_policy
        .combat_difficulty
        .power_model_version;
    let checkpoint_anchor = source
        .state
        .run_checkpoint
        .payload
        .as_ref()
        .unwrap()
        .current_floor_power_anchor;
    source.state.run_power_authority.power_model_version = power_model_version;
    source.state.run_power_authority.run_peak_certified_power = 900;
    source.state.run_power_authority.current_floor_power_anchor = 800;

    let exported = source
        .export_active_run_checkpoint()
        .unwrap()
        .expect("completed SavePoint should be exportable");
    assert_eq!(exported.loads_used, 1);
    assert_eq!(exported.run_peak_certified_power, 900);
    assert!(!String::from_utf8_lossy(&exported.core_payload_json).contains("encounter_compendium"));
    let exported_document: serde_json::Value =
        serde_json::from_slice(&exported.core_payload_json).unwrap();
    assert_eq!(exported_document["version"], 11);
    assert_eq!(
        exported_document["continuation"]["current"]["power_model_version"],
        power_model_version
    );
    assert_eq!(
        exported_document["continuation"]["current"]["current_floor_power_anchor"],
        checkpoint_anchor
    );

    let mut old_core_document = exported_document.clone();
    old_core_document["version"] = serde_json::json!(3);
    let old_core_payload = serde_json::to_vec(&old_core_document).unwrap();
    let mut old_core = GameCore::new(game_data.clone(), 997);
    assert!(matches!(
        old_core.continue_from_active_run_checkpoint(
            Uuid::from_u128(3),
            &old_core_payload,
            exported.loads_used,
            exported.run_peak_certified_power,
        ),
        Err(GameError::InvalidStaticData(_))
    ));

    let mut v2_core_document = exported_document.clone();
    v2_core_document["version"] = serde_json::json!(2);
    let mut v2_core = GameCore::new(game_data.clone(), 996);
    assert!(matches!(
        v2_core.continue_from_active_run_checkpoint(
            Uuid::from_u128(4),
            &serde_json::to_vec(&v2_core_document).unwrap(),
            exported.loads_used,
            exported.run_peak_certified_power,
        ),
        Err(GameError::InvalidStaticData(_))
    ));

    let mut missing_tactical_profile_document = exported_document.clone();
    let equipment_items = missing_tactical_profile_document["continuation"]["current"]["inventory"]
        ["equipments"]["items"]
        .as_object_mut()
        .expect("active-run checkpoint equipment map");
    let training_weapon = equipment_items
        .values_mut()
        .find(|item| item["meta"]["id"] == "standard_training_sword")
        .expect("checkpoint must persist a starter training weapon");
    training_weapon["meta"]["weapon_profile"]
        .as_object_mut()
        .unwrap()
        .remove("deployment_affinity");
    let missing_tactical_profile_payload =
        serde_json::to_vec(&missing_tactical_profile_document).unwrap();
    let mut missing_tactical_profile = GameCore::new(game_data.clone(), 994);
    let error = missing_tactical_profile
        .continue_from_active_run_checkpoint(
            Uuid::from_u128(7),
            &missing_tactical_profile_payload,
            exported.loads_used,
            exported.run_peak_certified_power,
        )
        .expect_err("current-version payload missing weapon tactical data must reject");
    assert!(matches!(error, GameError::InvalidStaticData(_)));

    let mut legacy_core_document = old_core_document;
    legacy_core_document
        .as_object_mut()
        .unwrap()
        .remove("version");
    let legacy_core_payload = serde_json::to_vec(&legacy_core_document).unwrap();
    let mut legacy_core = GameCore::new(game_data.clone(), 996);
    assert!(matches!(
        legacy_core.continue_from_active_run_checkpoint(
            Uuid::from_u128(5),
            &legacy_core_payload,
            exported.loads_used,
            exported.run_peak_certified_power,
        ),
        Err(GameError::InvalidStaticData(_))
    ));

    let mut stale_model_document = exported_document;
    stale_model_document["continuation"]["current"]["power_model_version"] =
        serde_json::json!(power_model_version - 1);
    refresh_active_run_checksum(&mut stale_model_document);
    let stale_model_payload = serde_json::to_vec(&stale_model_document).unwrap();
    let mut stale_model = GameCore::new(game_data.clone(), 998);
    assert!(matches!(
        stale_model.continue_from_active_run_checkpoint(
            Uuid::from_u128(4),
            &stale_model_payload,
            exported.loads_used,
            exported.run_peak_certified_power,
        ),
        Err(GameError::InvalidStaticData(_))
    ));

    let mut mismatched_anchor_document: serde_json::Value =
        serde_json::from_slice(&exported.core_payload_json).unwrap();
    mismatched_anchor_document["continuation"]["current"]["current_floor_power_anchor"] =
        serde_json::json!(checkpoint_anchor + 1);
    refresh_active_run_checksum(&mut mismatched_anchor_document);
    let mismatched_anchor_payload = serde_json::to_vec(&mismatched_anchor_document).unwrap();
    let mut mismatched_anchor = GameCore::new(game_data.clone(), 995);
    assert!(matches!(
        mismatched_anchor.continue_from_active_run_checkpoint(
            Uuid::from_u128(6),
            &mismatched_anchor_payload,
            exported.loads_used,
            exported.run_peak_certified_power,
        ),
        Err(GameError::InvalidStaticData(_))
    ));

    let restored_player_id = Uuid::from_u128(2);
    let mut restored = GameCore::new(game_data, 999);
    let continued = restored
        .continue_from_active_run_checkpoint(
            restored_player_id,
            &exported.core_payload_json,
            exported.loads_used,
            exported.run_peak_certified_power,
        )
        .unwrap();
    assert!(matches!(continued, BehaviorResult::MapState { .. }));
    assert!(matches!(restored.get_state(), GameState::ViewingMap));
    assert_eq!(
        restored.state.player.as_ref().unwrap().id,
        restored_player_id
    );
    assert_eq!(restored.state.run_checkpoint.loads_used, 1);
    assert_eq!(
        restored.state.run_power_authority.power_model_version,
        power_model_version
    );
    assert_eq!(
        restored.state.run_power_authority.run_peak_certified_power,
        900
    );
    assert_eq!(
        restored
            .state
            .run_power_authority
            .current_floor_power_anchor,
        checkpoint_anchor
    );
    assert_eq!(
        restored.current_map_view().unwrap(),
        source.current_map_view().unwrap()
    );
    assert_eq!(
        restored
            .state
            .run
            .as_ref()
            .unwrap()
            .run_progression
            .run_seed,
        123
    );
}

#[test]
fn checkpoint_load_rewinds_run_streak_state_but_never_rolls_back_permanent_records() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();

    core.state
        .run
        .as_mut()
        .unwrap()
        .record_progress
        .current_deathless_floor_streak = 9;
    core.state.player_profile.highest_floor_reached = 12;
    core.state.player_profile.longest_deathless_floor_streak = 7;
    core.state.player_profile.standard_tutorial_completed = true;
    core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();

    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .record_progress
            .current_deathless_floor_streak,
        0,
        "run-local streak progress follows the checkpoint"
    );
    assert_eq!(core.state.player_profile.highest_floor_reached, 12);
    assert_eq!(core.state.player_profile.longest_deathless_floor_streak, 7);
    assert!(core.state.player_profile.standard_tutorial_completed);
}

#[test]
fn checkpoint_load_restores_floor_anchor_and_gate_replay_is_deterministic() {
    let mut core = GameCore::new(empty_game_data(), 321);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let gate_node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Gate,
        "gate_stairs",
        MapNodePayload::None,
    );
    let checkpoint_anchor = core.state.run_power_authority.current_floor_power_anchor;
    let checkpoint_map = core.state.run.as_ref().unwrap().map.clone();
    core.save_run_checkpoint_for_test().unwrap();

    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: gate_node_id,
        },
    )
    .unwrap();
    assert!(matches!(
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap(),
        BehaviorResult::FloorAdvanced { .. }
    ));
    let first_next_anchor = core.state.run_power_authority.current_floor_power_anchor;
    let first_next_map = core.state.run.as_ref().unwrap().map.clone();
    assert!(first_next_anchor > checkpoint_anchor);

    core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();
    assert_eq!(
        core.state.run_power_authority.current_floor_power_anchor, checkpoint_anchor,
        "checkpoint load must restore the anchor generation owned by its map"
    );
    assert_eq!(core.state.run.as_ref().unwrap().map, checkpoint_map);

    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: gate_node_id,
        },
    )
    .unwrap();
    assert!(matches!(
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap(),
        BehaviorResult::FloorAdvanced { .. }
    ));
    assert_eq!(
        core.state.run_power_authority.current_floor_power_anchor, first_next_anchor,
        "replaying the same checkpoint Gate must not compound the anchor rise"
    );
    assert_eq!(core.state.run.as_ref().unwrap().map, first_next_map);
}

#[test]
fn load_run_checkpoint_is_limited_to_three_uses_per_run() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
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

    assert_eq!(
        action_capability(&core, ActionKind::LoadRunCheckpoint)
            .expect("checkpoint recovery remains context-relevant")
            .disabled_reason_code,
        Some(ActionDisabledReasonCode::CheckpointUnavailable)
    );
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["run_checkpoint"]["loads_used"], 3);
    assert_eq!(snapshot["run_checkpoint"]["remaining_loads"], 0);
    assert!(snapshot["run_checkpoint"].get("can_load").is_none());
    assert!(matches!(
        core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
            .unwrap_err(),
        GameError::ActionUnavailable {
            action: ActionKind::LoadRunCheckpoint,
            reason: ActionDisabledReasonCode::CheckpointUnavailable
        }
    ));
}

#[test]
fn independent_maintenance_node_exposes_maintenance_actions() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    for action in maintenance_action_kinds() {
        assert_eq!(
            action_capability(&core, action),
            None,
            "Maintenance action must be absent outside Maintenance: {action:?}"
        );
    }
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );

    let result = select_and_confirm_map_node(&mut core, player_id, node_id);

    let BehaviorResult::MaintenanceState {
        maintenance_options,
        ..
    } = result
    else {
        panic!("expected Maintenance state");
    };
    assert!(!maintenance_options.materials.is_empty());
    for action in maintenance_action_kinds() {
        assert_eq!(
            action_capability(&core, action),
            Some(ActionCapabilityDto::disabled(
                action,
                ActionScope::Gameplay,
                ActionDisabledReasonCode::NoEligiblePayload,
            )),
            "Maintenance action without an executable payload must be typed disabled: {action:?}"
        );
    }

    let snapshot = core.get_run_snapshot_json().unwrap();
    let serialized_capabilities = snapshot["action_capabilities"].as_array().unwrap();
    for action in maintenance_action_kinds() {
        let serialized = serialized_capabilities
            .iter()
            .find(|capability| capability["action"] == format!("{action:?}"))
            .expect("Maintenance capability must serialize in Maintenance context");
        assert_eq!(serialized["scope"], "gameplay");
        assert_eq!(serialized["enabled"], false);
        assert_eq!(serialized["disabled_reason_code"], "no_eligible_payload");
    }

    core.state.visual_novel.active = Some(crate::game::visual_novel::VisualNovelSession {
        session_id: "maintenance_blocking_vn".to_string(),
        source: crate::game::visual_novel::VisualNovelSource::Map,
        vn_id: crate::game::data::visual_novel_data::VisualNovelId::new("maintenance_blocking_vn"),
        current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new("start"),
        committed_choice_id: None,
        blocking: true,
        presentation_mode: crate::game::visual_novel::VisualNovelSessionPresentationMode::Cinematic,
    });
    for action in maintenance_action_kinds() {
        assert_eq!(
            action_capability(&core, action),
            None,
            "blocking Visual Novel must hide Maintenance action: {action:?}"
        );
    }
    core.state.visual_novel.active = None;

    core.execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();
    for action in maintenance_action_kinds() {
        assert_eq!(
            action_capability(&core, action),
            None,
            "Maintenance action must disappear after node completion: {action:?}"
        );
    }
}

#[test]
fn live_maintenance_capabilities_follow_executable_operation_projection() {
    let game_data = crate::game::data::ValidatedGameData::load_live_embedded()
        .expect("embedded live gameplay RON must be valid");
    let equipment = game_data
        .equipment_data
        .get_by_id("standard_armor")
        .expect("live standard_armor")
        .clone();
    let fragment = game_data
        .skill_fragment_data
        .fragments
        .iter()
        .find(|fragment| {
            matches!(
                &fragment.effect,
                SkillFragmentEffectDef::ActiveSkill {
                    forms, ..
                } if forms.contains_key("awakened")
            )
        })
        .expect("live awakenable skill fragment")
        .clone();
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let owned_equipment_uuid = Uuid::from_u128(0xCA11_0001);
    core.inventory_mut()
        .unwrap()
        .equipments
        .add_item(OwnedEquipment::new(
            owned_equipment_uuid,
            Arc::new(equipment),
        ))
        .unwrap();
    core.inventory_mut()
        .unwrap()
        .equipment_materials
        .add("equipment_dust", 2)
        .unwrap();
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.state.skill_fragments.add_fragment_dust(100).unwrap();

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    let result = select_and_confirm_map_node(&mut core, player_id, node_id);
    let BehaviorResult::MaintenanceState {
        maintenance_options,
        ..
    } = result
    else {
        panic!("expected Maintenance state");
    };

    let equipment_preview = maintenance_options
        .items
        .iter()
        .find(|item| item.target_id == owned_equipment_uuid.to_string())
        .expect("live recipe-backed equipment preview");
    assert_eq!(
        equipment_preview.target_kind,
        MaintenanceTargetKind::Equipment
    );
    assert!(equipment_preview.operations.dismantle.can_execute);
    assert!(equipment_preview.operations.enhance.can_execute);

    let fragment_preview = maintenance_options
        .items
        .iter()
        .find(|item| item.target_id == copy_uuid.to_string())
        .expect("live skill fragment preview");
    assert_eq!(
        fragment_preview.target_kind,
        MaintenanceTargetKind::SkillFragment
    );
    assert!(!fragment_preview.operations.dismantle.can_execute);
    assert!(!fragment_preview.operations.enhance.can_execute);
    assert!(!fragment_preview.operations.awaken.can_execute);
    assert!(fragment_preview
        .fragment_forms
        .iter()
        .any(|f| f.form_id == "base" && f.can_select));

    for action in [
        ActionKind::SelectSkillFragmentForm,
        ActionKind::DismantleEquipment,
        ActionKind::EnhanceEquipment,
    ] {
        assert_eq!(
            action_capability(&core, action),
            Some(ActionCapabilityDto::enabled(action, ActionScope::Gameplay))
        );
    }

    let snapshot_before_invalid_command = core.get_run_snapshot_json().unwrap();
    let error = core
        .execute(
            player_id,
            PlayerBehavior::EnhanceEquipment {
                item_uuid: Uuid::from_u128(0xCA11_FFFF),
            },
        )
        .unwrap_err();
    assert!(matches!(error, GameError::InventoryItemNotFound));
    assert_eq!(
        core.get_run_snapshot_json().unwrap(),
        snapshot_before_invalid_command,
        "an enabled family capability must still preserve selected-payload validation and atomicity"
    );
}
