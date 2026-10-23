use super::*;
use crate::game::behavior::{ActionCapabilityDto, ActionDisabledReasonCode, ActionScope};

#[test]
fn run_snapshot_exposes_inactive_visual_novel_as_explicit_null() {
    let core = GameCore::new(empty_game_data(), 123);

    let snapshot = core
        .get_run_snapshot_json()
        .expect("run snapshot should serialize");

    assert!(
        snapshot
            .as_object()
            .expect("run snapshot should be an object")
            .contains_key("visual_novel"),
        "visual_novel must be an explicit root field even when inactive"
    );
    assert!(
        snapshot["visual_novel"].is_null(),
        "inactive VN overlay should serialize as visual_novel: null"
    );
}

#[test]
fn live_game_data_loads_run_start_visual_novel_triggers() {
    let game_data = crate::game::data::ValidatedGameData::load_live_embedded()
        .expect("embedded live gameplay RON must be valid");

    let standard = game_data
        .visual_novel_trigger_data
        .get_by_id("run_start_standard_hq_briefing")
        .expect("standard run-start trigger should load from live RON");
    let endless = game_data
        .visual_novel_trigger_data
        .get_by_id("run_start_endless_hq_briefing")
        .expect("endless run-start trigger should load from live RON");

    assert_eq!(standard.vn_id.as_str(), "hq_briefing_standard_intro");
    assert_eq!(endless.vn_id.as_str(), "hq_briefing_endless_intro");
}

#[test]
fn start_new_game_opens_pre_run_hq_visual_novel_without_run_or_map() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);

    let result = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .unwrap();
    let BehaviorResult::RunStartFlowStarted {
        game_mode,
        visual_novel: Some(visual_novel),
    } = result
    else {
        panic!("start new game should return pre-run VN state");
    };

    assert_eq!(game_mode, GameMode::Expedition);
    assert_eq!(
        visual_novel.source,
        crate::game::behavior::VisualNovelSourceDto::HqBriefing
    );
    assert_eq!(visual_novel.vn_id, "hq_briefing_standard_intro");
    assert_eq!(visual_novel.current_scene_id, "start");
    assert_eq!(visual_novel.script_id, "hq_briefing_standard_intro");
    assert!(visual_novel.blocking);
    assert!(core.state.run.is_none());
    assert!(core.state.starter_candidates.is_empty());
    assert!(matches!(
        core.get_state(),
        GameState::PreRunBriefing {
            game_mode: GameMode::Expedition
        }
    ));

    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["game_state"], "pre_run_briefing");
    assert_eq!(snapshot["game_state_context"]["type"], "pre_run_briefing");
    assert_eq!(snapshot["game_state_context"]["game_mode"], "Expedition");
    assert!(snapshot["run_progression"].is_null());
    assert!(snapshot["map"].is_null());
    assert_eq!(snapshot["visual_novel"]["source"]["type"], "hq_briefing");
    assert_eq!(
        snapshot["action_capabilities"],
        json!([{
            "action": "AdvanceVisualNovel",
            "scope": "gameplay",
            "enabled": true
        }])
    );

    assert!(matches!(
        core.execute(
            player_id,
            PlayerBehavior::SelectStarterEmployees {
                employee_definition_ids: vec![],
                loan_copy_ids: vec![],
            }
        )
        .unwrap_err(),
        GameError::ActionUnavailable {
            action: ActionKind::SelectStarterEmployees,
            reason: ActionDisabledReasonCode::ContextIrrelevant
        }
    ));
}

#[test]
fn run_start_session_blocking_is_owned_only_by_the_matching_trigger() {
    let mut triggers = test_hq_briefing_visual_novel_trigger_database();
    triggers
        .triggers
        .iter_mut()
        .find(|trigger| {
            matches!(
                trigger.source,
                VisualNovelTriggerSource::RunStart {
                    game_mode: GameMode::Expedition
                }
            )
        })
        .expect("standard run-start trigger")
        .blocking = false;
    let game_data = GameDataBuilder::empty()
        .with_employee_definitions(test_employee_definition_database())
        .with_visual_novels(test_hq_briefing_visual_novel_database())
        .with_visual_novel_triggers(triggers)
        .with_equipment(equipment_with_starter_assets(Vec::new()))
        .with_skill_fragment_data(Arc::new(SkillFragmentDatabase::new(vec![
            SkillFragmentMetadata::starter_basic_attack(),
        ])))
        .build_arc();
    let mut core = GameCore::new(game_data, 123);

    let result = core
        .execute(
            Uuid::from_u128(1),
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .expect("run start should activate the matching trigger");

    let BehaviorResult::RunStartFlowStarted {
        visual_novel: Some(visual_novel),
        ..
    } = result
    else {
        panic!("run start should return the trigger-owned VN session");
    };
    assert!(!visual_novel.blocking);
    assert!(!core.state.visual_novel.active.as_ref().unwrap().blocking);
}

#[test]
fn missing_run_start_visual_novel_trigger_fails_start_new_game() {
    let game_data = GameDataBuilder::empty()
        .with_employee_definitions(test_employee_definition_database())
        .with_visual_novels(test_hq_briefing_visual_novel_database())
        .with_equipment(equipment_with_starter_assets(Vec::new()))
        .with_skill_fragment_data(Arc::new(SkillFragmentDatabase::new(vec![
            SkillFragmentMetadata::starter_basic_attack(),
        ])))
        .build_arc();
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    let before = non_battle_state_without_profile_fingerprint(&core);
    let before_profile = core.player_profile();
    let before_run_seed = core.run_seed;

    let first_error = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .unwrap_err();

    assert!(
        matches!(&first_error, GameError::InvalidStaticData(message) if message.contains("missing RunStart visual novel trigger"))
    );
    assert_eq!(non_battle_state_without_profile_fingerprint(&core), before);
    assert_eq!(core.player_profile(), before_profile);
    assert_eq!(core.run_seed, before_run_seed);

    let retry_error = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .unwrap_err();
    assert_eq!(format!("{retry_error:?}"), format!("{first_error:?}"));
    assert_eq!(non_battle_state_without_profile_fingerprint(&core), before);
    assert_eq!(core.player_profile(), before_profile);
    assert_eq!(core.run_seed, before_run_seed);
}

#[test]
fn pre_run_visual_novel_choice_works_without_run_state_and_opens_starter_selection() {
    use crate::game::data::visual_novel_data::{
        VisualNovelChoiceDefinition, VisualNovelChoiceId, VisualNovelChoiceNext,
        VisualNovelChoicePreview, VisualNovelSpeakerDefinition, VisualNovelSpeakerPosition,
    };

    let mut standard_hq = test_hq_briefing_visual_novel_definition("hq_briefing_standard_intro");
    standard_hq.scenes[0].text_kind = VisualNovelTextKind::Dialogue;
    standard_hq.scenes[0].speakers = vec![VisualNovelSpeakerDefinition {
        speaker_id: "operator".to_string(),
        portrait_id: None,
        position: VisualNovelSpeakerPosition::Center,
        active: true,
    }];
    standard_hq.scenes[0].next = VisualNovelSceneNext::Choices {
        choices: vec![VisualNovelChoiceDefinition {
            id: VisualNovelChoiceId::new("begin"),
            label_id: "hq_briefing_choice_begin".to_string(),
            preview: VisualNovelChoicePreview::default(),
            next: VisualNovelChoiceNext::End,
        }],
    };
    let game_data = test_game_data_builder()
        .with_visual_novels(VisualNovelDatabase::new(vec![
            standard_hq,
            test_hq_briefing_visual_novel_definition("hq_briefing_endless_intro"),
        ]))
        .build_arc();
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);

    let BehaviorResult::RunStartFlowStarted {
        visual_novel: Some(visual_novel),
        ..
    } = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .unwrap()
    else {
        panic!("start should project pre-run VN");
    };
    assert!(core.state.run.is_none());
    assert_eq!(visual_novel.choices.len(), 1);

    let selected = core
        .execute(
            player_id,
            PlayerBehavior::SelectVisualNovelChoice {
                session_id: visual_novel.session_id,
                current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                    visual_novel.current_scene_id,
                ),
                choice_id: VisualNovelChoiceId::new("begin"),
            },
        )
        .unwrap();

    assert!(matches!(
        selected,
        BehaviorResult::VisualNovelState { visual_novel: None }
    ));
    assert!(core.state.run.is_none());
    assert!(matches!(
        core.get_state(),
        GameState::SelectingStarterEmployees
    ));
    assert_eq!(core.state.starter_candidates.len(), 6);
}

#[test]
fn run_snapshot_projects_active_visual_novel_session() {
    let mut core = GameCore::new(live_game_data_from_ron(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    core.state.visual_novel.active = Some(crate::game::visual_novel::VisualNovelSession {
        session_id: "vn_session_1".to_string(),
        source: crate::game::visual_novel::VisualNovelSource::Map,
        vn_id: crate::game::data::visual_novel_data::VisualNovelId::new(
            "white_night_confession_intro",
        ),
        current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
            "white_night_speaks",
        ),
        committed_choice_id: None,
        blocking: true,
        presentation_mode: crate::game::visual_novel::VisualNovelSessionPresentationMode::Cinematic,
    });

    let snapshot = core
        .get_run_snapshot_json()
        .expect("run snapshot should serialize");
    let visual_novel = &snapshot["visual_novel"];

    assert_eq!(visual_novel["source"]["type"], "map");
    assert_eq!(visual_novel["session_id"], "vn_session_1");
    assert_eq!(visual_novel["vn_id"], "white_night_confession_intro");
    assert_eq!(visual_novel["current_scene_id"], "white_night_speaks");
    assert_eq!(visual_novel["background_id"], "white_night_chapel");
    assert_eq!(visual_novel["script_id"], "white_night_confession_intro");
    assert_eq!(visual_novel["speakers"][0]["speaker_id"], "white_night");
    assert_eq!(
        visual_novel["speakers"][0]["portrait_id"],
        "white_night_veil"
    );
    assert_eq!(visual_novel["speakers"][0]["position"], "center");
    assert_eq!(visual_novel["speakers"][0]["active"], true);
    assert_eq!(visual_novel["blocking"], true);
    assert_eq!(visual_novel["presentation_mode"], "cinematic");
}

#[test]
fn blocking_visual_novel_gates_background_actions_and_advances_to_end() {
    let mut core = GameCore::new(live_game_data_from_ron(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    core.state.visual_novel.active = Some(crate::game::visual_novel::VisualNovelSession {
        session_id: "vn_session_advance".to_string(),
        source: crate::game::visual_novel::VisualNovelSource::Map,
        vn_id: crate::game::data::visual_novel_data::VisualNovelId::new(
            "white_night_confession_intro",
        ),
        current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new("start"),
        committed_choice_id: None,
        blocking: true,
        presentation_mode: crate::game::visual_novel::VisualNovelSessionPresentationMode::Cinematic,
    });

    assert!(action_capability_enabled(
        &core,
        ActionKind::AdvanceVisualNovel
    ));
    assert!(action_capability(&core, ActionKind::SelectVisualNovelChoice).is_none());
    assert!(matches!(
        core.execute(player_id, PlayerBehavior::RequestMapData)
            .unwrap_err(),
        GameError::ActionUnavailable {
            action: ActionKind::RequestMapData,
            reason: ActionDisabledReasonCode::ContextIrrelevant
        }
    ));

    let advanced = core
        .execute(
            player_id,
            PlayerBehavior::AdvanceVisualNovel {
                session_id: "vn_session_advance".to_string(),
                current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                    "start",
                ),
            },
        )
        .unwrap();
    let BehaviorResult::VisualNovelState {
        visual_novel: Some(visual_novel),
    } = advanced
    else {
        panic!("advance should project the next VN scene");
    };
    assert_eq!(visual_novel.current_scene_id, "white_night_speaks");

    let ended = core
        .execute(
            player_id,
            PlayerBehavior::AdvanceVisualNovel {
                session_id: "vn_session_advance".to_string(),
                current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                    "white_night_speaks",
                ),
            },
        )
        .unwrap();
    let BehaviorResult::VisualNovelState { visual_novel: None } = ended else {
        panic!("advancing an End scene should close the VN overlay");
    };
    assert!(core.state.visual_novel.active.is_none());
    assert!(action_capability_enabled(&core, ActionKind::RequestMapData));
}

#[test]
fn visual_novel_choice_command_advances_global_session() {
    use crate::game::data::visual_novel_data::{
        VisualNovelBackground, VisualNovelChoiceDefinition, VisualNovelChoiceId,
        VisualNovelChoiceNext, VisualNovelChoicePreview, VisualNovelDatabase,
        VisualNovelDefinition, VisualNovelId, VisualNovelPresentationMode,
        VisualNovelSceneDefinition, VisualNovelSceneId, VisualNovelSceneNext,
        VisualNovelSpeakerDefinition, VisualNovelSpeakerPosition, VisualNovelTextKind,
    };

    let game_data = test_game_data_builder()
        .with_visual_novels(VisualNovelDatabase::new(vec![
            test_hq_briefing_visual_novel_definition("hq_briefing_standard_intro"),
            test_hq_briefing_visual_novel_definition("hq_briefing_endless_intro"),
            VisualNovelDefinition {
                id: VisualNovelId::new("choice_vn"),
                entry_scene_id: VisualNovelSceneId::new("choice"),
                default_presentation_mode: VisualNovelPresentationMode::Cinematic,
                scenes: vec![
                    VisualNovelSceneDefinition {
                        id: VisualNovelSceneId::new("choice"),
                        text_kind: VisualNovelTextKind::Dialogue,
                        background: VisualNovelBackground::CurrentScreenDimmed,
                        script_id: "choice_script".to_string(),
                        speakers: vec![VisualNovelSpeakerDefinition {
                            speaker_id: "operator".to_string(),
                            portrait_id: None,
                            position: VisualNovelSpeakerPosition::Center,
                            active: true,
                        }],
                        next: VisualNovelSceneNext::Choices {
                            choices: vec![VisualNovelChoiceDefinition {
                                id: VisualNovelChoiceId::new("accept"),
                                label_id: "choice_accept".to_string(),
                                preview: VisualNovelChoicePreview::default(),
                                next: VisualNovelChoiceNext::Scene {
                                    scene_id: VisualNovelSceneId::new("after_choice"),
                                },
                            }],
                        },
                    },
                    VisualNovelSceneDefinition {
                        id: VisualNovelSceneId::new("after_choice"),
                        text_kind: VisualNovelTextKind::Narration,
                        background: VisualNovelBackground::CurrentScreenDimmed,
                        script_id: "after_choice_script".to_string(),
                        speakers: vec![],
                        next: VisualNovelSceneNext::End,
                    },
                ],
            },
        ]))
        .build_arc();
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    core.state.visual_novel.active = Some(crate::game::visual_novel::VisualNovelSession {
        session_id: "vn_session_choice".to_string(),
        source: crate::game::visual_novel::VisualNovelSource::Map,
        vn_id: VisualNovelId::new("choice_vn"),
        current_scene_id: VisualNovelSceneId::new("choice"),
        committed_choice_id: None,
        blocking: true,
        presentation_mode: crate::game::visual_novel::VisualNovelSessionPresentationMode::Cinematic,
    });

    let selected = core
        .execute(
            player_id,
            PlayerBehavior::SelectVisualNovelChoice {
                session_id: "vn_session_choice".to_string(),
                current_scene_id: VisualNovelSceneId::new("choice"),
                choice_id: VisualNovelChoiceId::new("accept"),
            },
        )
        .unwrap();
    let BehaviorResult::VisualNovelState {
        visual_novel: Some(visual_novel),
    } = selected
    else {
        panic!("choice command should project the next VN scene");
    };

    assert_eq!(visual_novel.current_scene_id, "after_choice");
    assert_eq!(visual_novel.script_id, "after_choice_script");
    assert!(visual_novel.background_id.is_none());
    assert!(visual_novel.choices.is_empty());
}

#[test]
fn selected_shop_snapshot_includes_typed_offer_metadata_without_hidden_generation_state() {
    let blade = equipment_meta(0x10, "raw_blade_core", EquipmentType::Weapon);
    let game_data = game_data_with_display_items(vec![blade.clone()]);
    let mut core = GameCore::new(game_data, 123);

    core.state.active_node_content = Some(ActiveNodeContent::Shop(ShopSessionState {
        id: "typed_shop".to_string(),
        name: "Typed Shop".to_string(),
        uuid: Uuid::from_u128(0x30),
        context: crate::game::data::shop_data::ShopContext::UnknownExploration,
        node_id: MapNodeId::new(Uuid::from_u128(0x31)),
        offer_count: 1,
        refresh_policy: None,
        sell_policy: None,
        refresh_uses: 0,
        page_index: 0,
        offers: vec![crate::game::resources::ShopOfferState {
            offer_id: Uuid::from_u128(0x32),
            merchandise: crate::game::data::shop_data::ShopMerchandise::Equipment {
                equipment_id: blade.id.clone(),
            },
            item_uuid: blade.uuid,
            quantity: 1,
            total_price: 70,
            repeatable: false,
            tags: vec!["equipment".to_string()],
        }],
        purchased_merchandise: std::collections::HashSet::new(),
    }));

    let selected = core
        .get_selected_event_snapshot_json()
        .unwrap()
        .expect("selected event snapshot");

    assert_eq!(
        selected["offers"][0]["offer_id"],
        json!(Uuid::from_u128(0x32))
    );
    assert_eq!(selected["offers"][0]["quantity"], 1);
    assert_eq!(selected["offers"][0]["total_price"], 70);
    assert_eq!(selected["offers"][0]["item"]["uuid"], json!(blade.uuid));
    assert_eq!(selected["offers"][0]["item"]["kind"], "equipment");
    assert_eq!(
        selected["offers"][0]["item"]["definition_id"],
        "raw_blade_core"
    );
    assert_eq!(selected["offers"][0]["item"]["id"], "raw_blade_core");
    assert_eq!(selected["offers"][0]["item"]["equipment_type"], "weapon");
    assert!(selected["offers"][0]["item"]["icon"].is_null());
    assert!(selected.get("visible_items").is_none());
    assert!(selected.get("visible_item_uuids").is_none());
    assert!(selected.get("hidden_items").is_none());
    assert!(selected.get("hidden_item_uuids").is_none());
}

#[test]
fn game_core_projects_context_capabilities_and_revalidates_on_state_transition() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);

    assert!(matches!(core.get_state(), GameState::NotStarted));
    assert!(action_capability_enabled(&core, ActionKind::StartNewGame));
    assert!(action_capability(&core, ActionKind::RequestMapData).is_none());

    let err = core
        .execute(player_id, PlayerBehavior::RequestMapData)
        .unwrap_err();
    assert!(matches!(
        err,
        GameError::ActionUnavailable {
            action: ActionKind::RequestMapData,
            reason: ActionDisabledReasonCode::ContextIrrelevant
        }
    ));

    let res = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .unwrap();
    let BehaviorResult::RunStartFlowStarted {
        game_mode,
        visual_novel: Some(visual_novel),
    } = res
    else {
        panic!("start should expose pre-run VN");
    };
    assert_eq!(game_mode, GameMode::Expedition);
    assert!(matches!(
        core.get_state(),
        GameState::PreRunBriefing {
            game_mode: GameMode::Expedition
        }
    ));
    assert!(action_capability_enabled(
        &core,
        ActionKind::AdvanceVisualNovel
    ));
    assert!(action_capability(&core, ActionKind::SelectVisualNovelChoice).is_none());

    let ended = core
        .execute(
            player_id,
            PlayerBehavior::AdvanceVisualNovel {
                session_id: visual_novel.session_id,
                current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                    visual_novel.current_scene_id,
                ),
            },
        )
        .unwrap();
    assert!(matches!(
        ended,
        BehaviorResult::VisualNovelState { visual_novel: None }
    ));
    assert!(matches!(
        core.get_state(),
        GameState::SelectingStarterEmployees
    ));
    assert_eq!(core.state.starter_candidates.len(), 6);
    assert_eq!(
        core.get_action_capabilities(),
        vec![ActionCapabilityDto::enabled(
            ActionKind::SelectStarterEmployees,
            ActionScope::Gameplay,
        )]
    );

    let selected_ids = core
        .state
        .starter_candidates
        .iter()
        .take(run_policy().setup.starter_employee_count)
        .map(|definition| definition.employee_definition_id.clone())
        .collect::<Vec<_>>();
    let selected = core
        .execute(
            player_id,
            PlayerBehavior::SelectStarterEmployees {
                employee_definition_ids: selected_ids,
                loan_copy_ids: vec![],
            },
        )
        .unwrap();
    assert!(matches!(
        selected,
        BehaviorResult::StarterEmployeesSelected { .. }
    ));
    assert!(matches!(core.get_state(), GameState::ViewingMap));

    assert!(action_capability_enabled(&core, ActionKind::RequestMapData));
    assert!(action_capability_enabled(&core, ActionKind::SelectMapNode));
    assert!(action_capability_enabled(&core, ActionKind::EquipItem));
    assert!(action_capability_enabled(&core, ActionKind::UnEquipItem));
    assert!(action_capability(&core, ActionKind::UseConsumableItem).is_some());
    assert!(action_capability(&core, ActionKind::EquipSkillFragment).is_some());
    assert!(action_capability(&core, ActionKind::UnequipSkillFragment).is_some());
}

#[test]
fn fresh_profile_can_start_expedition_without_tutorial_unlock() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    assert_eq!(core.state.run.as_ref().unwrap().run_progression.game_mode, GameMode::Expedition);
    assert!(serde_json::to_value(&core.state.player_profile).unwrap().get("standard_tutorial_completed").is_none());
}

#[test]
fn start_new_game_uses_requested_game_mode_when_run_is_created() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);

    let start = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .unwrap();
    let BehaviorResult::RunStartFlowStarted {
        game_mode,
        visual_novel: Some(visual_novel),
    } = start
    else {
        panic!("start should expose pre-run VN");
    };
    assert_eq!(game_mode, GameMode::Expedition);
    assert!(matches!(
        core.get_state(),
        GameState::PreRunBriefing {
            game_mode: GameMode::Expedition
        }
    ));

    core.execute(
        player_id,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: visual_novel.session_id,
            current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                visual_novel.current_scene_id,
            ),
        },
    )
    .unwrap();

    let employee_definition_ids = core
        .state
        .starter_candidates
        .iter()
        .take(run_policy().setup.starter_employee_count)
        .map(|definition| definition.employee_definition_id.clone())
        .collect::<Vec<_>>();
    core.execute(
        player_id,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids,
            loan_copy_ids: vec![],
        },
    )
    .unwrap();

    let run = core.state.run.as_ref().expect("run should be created");
    assert_eq!(run.run_progression.game_mode, GameMode::Expedition);

    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["run_progression"]["game_mode"], "Expedition");
}

#[test]
fn employee_definition_profile_json_preserves_authored_and_run_identities() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    let start = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .unwrap();
    let BehaviorResult::RunStartFlowStarted {
        visual_novel: Some(visual_novel),
        ..
    } = start
    else {
        panic!("start should expose pre-run VN");
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
    .unwrap();

    let selecting = core.get_run_snapshot_json().unwrap();
    let candidate = &selecting["game_state_context"]["candidates"][0];
    assert_eq!(candidate["employee_definition_id"], "candidate_0");
    assert_eq!(candidate["starter_eligible"], true);
    assert_eq!(candidate["profile"]["personality"]["risk"], "Cautious");
    assert_eq!(candidate["profile"]["personality"]["social"], "Obedient");
    assert_eq!(candidate["profile"]["combat_aptitude"], "Balanced");
    assert_eq!(candidate["profile"]["base_combat_stats"]["max_hp"], 100);
    assert_eq!(candidate["profile"]["basic_attack_speed_percent"], 100);
    assert!(candidate.get("id").is_none());
    assert!(candidate.get("starter_loadout").is_none());

    let employee_definition_ids = core
        .state
        .starter_candidates
        .iter()
        .take(run_policy().setup.starter_employee_count)
        .map(|definition| definition.employee_definition_id.clone())
        .collect::<Vec<_>>();
    let command = serde_json::to_value(PlayerBehavior::SelectStarterEmployees {
        employee_definition_ids: employee_definition_ids.clone(),
        loan_copy_ids: vec![],
    })
    .unwrap();
    assert_eq!(
        command["employee_definition_ids"],
        json!(employee_definition_ids)
    );
    assert!(command.get("candidate_ids").is_none());

    core.execute(
        player_id,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids,
            loan_copy_ids: vec![],
        },
    )
    .unwrap();
    let running = core.get_run_snapshot_json().unwrap();
    let employee = &running["roster"]["employees"][0];
    assert!(employee["uuid"].is_string());
    assert_eq!(employee["employee_definition_id"], "candidate_0");
    assert_eq!(employee["profile"], candidate["profile"]);
    assert!(employee.get("trust").is_none());
    assert!(employee.get("psychology").is_none());
    assert_eq!(employee["role"], candidate["role"]);
}

#[test]
fn start_new_game_omits_removed_abnormality_research_snapshot() {
    let game_data = live_game_data_from_ron();
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);

    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);

    let snapshot = core.get_run_snapshot_json().unwrap();
    assert!(snapshot.get("abnormality_research").is_none());
}

#[test]
fn run_checkpoint_preserves_encounter_progression_graph() {
    let mut core = GameCore::new(live_game_data_from_ron(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let reward_source_id = core.game_data.pve_data.reward_sources[0]
        .reward_source_id
        .clone();
    {
        let progression = &mut core.state.run.as_mut().unwrap().encounter_progression;
        progression
            .completed_reward_source_ids
            .insert(reward_source_id.clone());
    }
    let event_node_id = force_first_available_node(&mut core, MapNodeCategory::Event, "event_story",
        MapNodePayload::Event {event_id:Some(crate::game::data::event_data::EventId::new("white_night_confession_01"))});
    {
        let run = core.state.run.as_mut().unwrap();
        run.boss_omen.cycle_index = 1;
        run.boss_omen.remaining_chain_ids = core.game_data.boss_omen_data.chains.iter().map(|chain|chain.id.clone()).collect();
        run.boss_omen.active = Some(crate::game::boss_omen::ActiveBossOmenState {
            chain_id: crate::game::data::boss_omen_data::BossOmenChainId::new("white_night"),
            boss_abnormality_id: "o-01-45_white_night".to_string(),
            next_step_index: 1,
            completed_steps: vec![crate::game::data::boss_omen_data::BossOmenStepId::new(
                "confession_01",
            )],
        });
        run.event_sessions.insert(
            event_node_id,
            crate::game::resources::EventSessionState {
                node_id: event_node_id,
                event_id: crate::game::data::event_data::EventId::new("white_night_confession_01"),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
                relic_corruption_selections: None,
                committed_check: Some(crate::game::resources::EventCommittedCheckState {
                    source_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
                    choice_id: crate::game::data::event_data::EventChoiceId::new("listen"),
                    employee_uuid: core.state.roster.iter().next().unwrap().uuid,
                    raw_roll: 14,
                    aptitude: crate::game::employee::EventAptitude::Analysis,
                    dc: 12,
                    modifier_components: vec![],
                    modifier_total: 0,
                    total_roll: 14,
                    choice_key_evidence: None,
                    outcome: crate::game::data::event_data::EventCheckOutcome::Success,
                }),
                started_combat: Some(crate::game::resources::EventStartedCombatState {
                    encounter_id: "suppress_white_night".to_string(),
                    primary_abnormality_id: Some("o-01-45_white_night".to_string()),
                }),
            },
        );
    }
    core.state.visual_novel.active = Some(crate::game::visual_novel::VisualNovelSession {
        session_id: "checkpoint_vn_session".to_string(),
        source: crate::game::visual_novel::VisualNovelSource::EventNode {
            node_id: event_node_id,
        },
        vn_id: crate::game::data::visual_novel_data::VisualNovelId::new(
            "white_night_confession_intro",
        ),
        current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new("start"),
        committed_choice_id: None,
        blocking: true,
        presentation_mode: crate::game::visual_novel::VisualNovelSessionPresentationMode::Cinematic,
    });
    core.save_run_checkpoint_for_test().unwrap();
    core.validate_run_checkpoint_payload(core.state.run_checkpoint.payload.as_ref().unwrap()).unwrap();
    {
        let progression = &mut core.state.run.as_mut().unwrap().encounter_progression;
        progression.completed_reward_source_ids.clear();
    }
    core.state.run.as_mut().unwrap().boss_omen.active = None;
    core.state.run.as_mut().unwrap().event_sessions.clear();
    core.state.visual_novel.active = Some(crate::game::visual_novel::VisualNovelSession {
        session_id: "mutated_vn_session".to_string(),
        source: crate::game::visual_novel::VisualNovelSource::Map,
        vn_id: crate::game::data::visual_novel_data::VisualNovelId::new(
            "white_night_confession_intro",
        ),
        current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
            "white_night_speaks",
        ),
        committed_choice_id: None,
        blocking: false,
        presentation_mode: crate::game::visual_novel::VisualNovelSessionPresentationMode::Compact,
    });

    core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();

    let restored_run = core.state.run.as_ref().unwrap();
    assert!(restored_run
        .completed_reward_source_ids
        .contains(&reward_source_id));
    assert!(restored_run.revival.eggs.is_empty());
    let restored_omen = restored_run
        .boss_omen
        .active
        .as_ref()
        .expect("checkpoint should restore active boss omen");
    assert_eq!(restored_omen.chain_id.as_str(), "white_night");
    assert_eq!(restored_omen.completed_steps.len(), 1);
    let restored_session = restored_run
        .event_sessions
        .get(&event_node_id)
        .expect("checkpoint should restore event session");
    assert_eq!(
        restored_session
            .committed_check
            .as_ref()
            .map(|check| check.choice_id.as_str()),
        Some("listen")
    );
    assert!(restored_session.started_combat.is_some());
    let active_vn = core
        .state
        .visual_novel
        .active
        .as_ref()
        .expect("top-level VN should remain active after checkpoint load");
    assert_eq!(active_vn.session_id, "mutated_vn_session");
    assert_eq!(active_vn.current_scene_id.as_str(), "white_night_speaks");
    assert!(!active_vn.blocking);
}

#[test]
fn run_checkpoint_rewinds_only_checkpoint_scoped_visual_novel_trigger_history() {
    let base = live_game_data_from_ron();
    let mut triggers = base.visual_novel_trigger_data.triggers.clone();
    for (id, policy) in [
        (
            "persistent_before_checkpoint",
            VisualNovelTriggerCheckpointPolicy::PersistAcrossCheckpointLoad,
        ),
        (
            "persistent_after_checkpoint",
            VisualNovelTriggerCheckpointPolicy::PersistAcrossCheckpointLoad,
        ),
        (
            "rewind_before_checkpoint",
            VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        ),
        (
            "rewind_after_checkpoint",
            VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        ),
    ] {
        let mut trigger = triggers[0].clone();
        trigger.id = VisualNovelTriggerId::new(id);
        trigger.source = VisualNovelTriggerSource::MapNodeEntered {
            category: MapNodeCategory::SavePoint,
            kind_id: MapNodeKindId::new("save_point"),
        };
        trigger.repeat = VisualNovelTriggerRepeat::OncePerRun;
        trigger.checkpoint_policy = policy;
        triggers.push(trigger);
    }
    let mut core = GameCore::new(
        GameDataBuilder::from_validated(&crate::game::data::ValidatedGameData::from(base))
            .with_visual_novel_triggers(VisualNovelTriggerDatabase::new(triggers))
            .build_arc(),
        123,
    );
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);

    let persistent_before = VisualNovelTriggerConsumedKey {
        trigger_id: VisualNovelTriggerId::new("persistent_before_checkpoint"),
        scope: VisualNovelTriggerConsumedScope::Run,
    };
    let persistent_after = VisualNovelTriggerConsumedKey {
        trigger_id: VisualNovelTriggerId::new("persistent_after_checkpoint"),
        scope: VisualNovelTriggerConsumedScope::Run,
    };
    let rewind_before = VisualNovelTriggerConsumedKey {
        trigger_id: VisualNovelTriggerId::new("rewind_before_checkpoint"),
        scope: VisualNovelTriggerConsumedScope::Run,
    };
    let rewind_after = VisualNovelTriggerConsumedKey {
        trigger_id: VisualNovelTriggerId::new("rewind_after_checkpoint"),
        scope: VisualNovelTriggerConsumedScope::Run,
    };

    core.state
        .visual_novel_triggers
        .persistent_consumed
        .insert(persistent_before.clone());
    core.state
        .visual_novel_triggers
        .checkpoint_rewind_consumed
        .insert(rewind_before.clone());
    core.save_run_checkpoint_for_test().unwrap();

    core.state
        .visual_novel_triggers
        .persistent_consumed
        .insert(persistent_after.clone());
    core.state
        .visual_novel_triggers
        .checkpoint_rewind_consumed
        .insert(rewind_after);

    core.refresh_continue_after_operation(super::super::application::ApplicationRoute::Behavior(
        ActionKind::CompleteNode,
    ))
    .unwrap();
    let current_history = core.state.visual_novel_triggers.clone();
    for _ in 0..3 {
        let export = core.export_active_run_checkpoint().unwrap().unwrap();
        let mut restarted = core_with_profile_for_continue(&core, 999);
        restarted
            .continue_from_active_run_checkpoint(
                player_id,
                &export.core_payload_json,
                export.loads_used,
                export.run_peak_certified_power,
            )
            .unwrap();
        assert_eq!(restarted.state.visual_novel_triggers, current_history);
        assert_eq!(
            restarted.export_active_run_checkpoint().unwrap().unwrap(),
            export
        );
        core = restarted;
    }

    core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();

    assert!(
        core.state
            .visual_novel_triggers
            .persistent_consumed
            .contains(&persistent_before),
        "persistent trigger history from before the checkpoint should remain"
    );
    assert!(
        core.state
            .visual_novel_triggers
            .persistent_consumed
            .contains(&persistent_after),
        "persistent trigger history added after the checkpoint should survive checkpoint load"
    );
    assert_eq!(
        core.state
            .visual_novel_triggers
            .checkpoint_rewind_consumed
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![rewind_before],
        "checkpoint-scoped trigger history should rewind to the saved payload"
    );
}

#[test]
fn employee_roster_snapshot_exposes_gameplay_status_and_loadout_without_display_slot() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_id = core.roster().unwrap().combat_eligible_employee_ids()[0];

    let snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let employees = snapshot["employees"].as_array().unwrap();
    let employee = employees
        .iter()
        .find(|employee| employee["uuid"] == json!(employee_id))
        .expect("starter employee is exposed in roster snapshot");

    assert_eq!(employee["condition"]["physical_damage"], 0);
    assert_eq!(employee["condition"]["physical_state"], "Normal");
    assert_eq!(employee["condition"]["trauma"], 0);
    assert_eq!(employee["condition"]["mental_state"], "Stable");
    assert_eq!(employee["eligibility"]["combat"]["eligible"], true);
    assert!(employee["eligibility"]["combat"]
        .get("disabled_reason")
        .is_none());
    assert!(employee.get("life_state").is_none());
    assert!(employee.get("availability").is_none());
    assert!(employee.get("available_for_combat").is_none());
    assert!(employee.get("health").is_none());
    assert!(employee.get("field_position").is_none());
    assert!(
        employee.get("roster_slot").is_none(),
        "display order is owned by Unity and must not be projected by Core"
    );
    let equipped_items = employee["equipped_items"].as_array().unwrap();
    assert_eq!(equipped_items.len(), 2);
    assert_eq!(
        equipped_items[0]["definition_id"],
        "standard_training_sword"
    );
    assert_eq!(equipped_items[1]["definition_id"], "standard_armor");
    assert!(employee["combat_profile"]
        .get("deployment_affinity")
        .is_none());
    assert_eq!(
        employee["combat_profile"]["effective_deployment_affinity"],
        "ground_only"
    );
    assert!(snapshot["combat_eligible_employee_ids"]
        .as_array()
        .unwrap()
        .contains(&json!(employee_id)));
}

#[test]
fn employee_roster_snapshot_projects_derived_condition_and_typed_reason_priority() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_id = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_id)
        .unwrap()
        .condition = EmployeeCondition::new(5, 100);

    let snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let employee = snapshot["employees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|employee| employee["uuid"] == json!(employee_id))
        .unwrap();

    assert_eq!(employee["condition"]["physical_state"], "Critical");
    assert_eq!(employee["condition"]["mental_state"], "Breakdown");
    assert_eq!(employee["eligibility"]["combat"]["eligible"], false);
    assert_eq!(
        employee["eligibility"]["combat"]["disabled_reason"],
        "CriticalPhysicalDamage"
    );
    assert_eq!(employee["eligibility"]["loadout"]["eligible"], true);
    assert_eq!(
        employee["eligibility"]["physical_treatment"]["eligible"],
        true
    );
    assert!(!snapshot["combat_eligible_employee_ids"]
        .as_array()
        .unwrap()
        .contains(&json!(employee_id)));
}

#[test]
fn weaponless_employee_remains_safezone_visible_and_rearms_from_authored_profile() {
    let mut core = GameCore::new(empty_game_data(), 0xA11C_E002);
    let player_id = Uuid::from_u128(0xA11C_E002);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let weapon_instance_uuid = force_unequip_employee_weapon(&mut core, employee_uuid);

    let snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let employee = snapshot["employees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|employee| employee["uuid"] == json!(employee_uuid))
        .expect("weaponless employee remains in the safe-zone roster");
    assert_eq!(employee["eligibility"]["combat"]["eligible"], true);
    assert_eq!(
        employee["combat_profile"]["effective_profile_error"]["code"],
        "weapon_required"
    );
    assert_eq!(
        employee["combat_profile"]["effective_weapon_profile"],
        Value::Null
    );
    assert!(!core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .is_battle_deployable());

    core.handle_equip_item(weapon_instance_uuid, employee_uuid)
        .expect("safe-zone weapon attachment must remain available");
    let rearmed = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert!(rearmed.is_battle_deployable());
    let snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let employee = snapshot["employees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|employee| employee["uuid"] == json!(employee_uuid))
        .unwrap();
    assert_eq!(
        employee["combat_profile"]["effective_weapon_profile"]["weapon_archetype"],
        "Sword"
    );
    assert_eq!(
        employee["combat_profile"]["effective_deployment_affinity"],
        "ground_only"
    );
    assert!(employee["combat_profile"]["effective_profile_error"].is_null());
}

#[test]
fn employee_roster_snapshot_surfaces_effective_profile_errors() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_id = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let stale_item_uuid = Uuid::from_u128(0xEFFE_C710);

    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_id).unwrap();
        let starter_weapon_uuid = employee
            .loadout
            .item_slot
            .iter()
            .find(|equipped| equipped.equipment_type == EquipmentType::Weapon)
            .map(|equipped| equipped.instance_uuid)
            .expect("starter employee is armed");
        employee
            .loadout
            .item_slot
            .remove_by_instance(starter_weapon_uuid)
            .unwrap();
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
        .contains("inventory item not found"));
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
    assert_eq!(snapshot["run_progression"]["game_mode"], "Expedition");
    assert_eq!(snapshot["run_progression"]["floor_index"], 0);
    assert!(snapshot["run_progression"]["max_floors"].is_null());
    assert!(snapshot["run_progression"]["current_floor_seed"].is_u64());
    assert_eq!(snapshot["map"]["map_template_id"], "expedition_development_v1");
    assert!(
        snapshot["action_capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|capability| capability["action"] == "SelectMapNode"
                && capability["enabled"] == true)
    );
    let map_nodes = snapshot["map"]["nodes"].as_array().unwrap();
    assert!(!map_nodes.is_empty());
    assert!(map_nodes.iter().all(|node| {
        node.get("id").is_some()
            && node.get("slot_id").is_some()
            && node.get("kind_id").is_some()
            && node.get("category").is_some()
            && node.get("state").is_some()
            && node.get("visibility").is_some()
    }));
    assert!(map_nodes
        .iter()
        .any(|node| node["state"] == json!("Available")));
    assert!(map_nodes
        .iter()
        .all(|node| node["visibility"] == json!("Revealed")));
    assert!(snapshot["map"].get("edges").is_none());
    assert!(snapshot["map"].get("current_node_id").is_none());
    assert!(snapshot["map"].get("available_node_ids").is_none());
    assert!(snapshot["map"].get("completed_node_ids").is_none());
    assert!(snapshot.get("map_progression").is_none());
    assert!(snapshot["map_navigation"]["current_node_id"].is_string());
    let selectable_node_ids = snapshot["map_navigation"]["selectable_node_ids"]
        .as_array()
        .unwrap();
    assert!(!selectable_node_ids.is_empty());
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
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let snapshot = core.get_run_snapshot_json().unwrap();

    assert_eq!(snapshot["game_state"], "node_confirm");
    assert_eq!(snapshot["game_state_context"]["node_id"], json!(node_id));
    assert_eq!(snapshot["map"]["map_template_id"], "expedition_development_v1");
    assert_eq!(
        snapshot["map_navigation"]["current_node_id"],
        core.state
            .run
            .as_ref()
            .unwrap()
            .map_progression
            .current_node_id
            .map(|id| json!(id))
            .unwrap()
    );
    assert_eq!(snapshot["current_node_session"]["node_id"], json!(node_id));
    assert_eq!(snapshot["current_node_session"]["payload"], "SavePoint");
    assert_eq!(
        snapshot["game_state_context"]["trauma_recovery_preview"]["recovery"]["CurrentPercent"]
            ["percent"],
        15
    );
    assert!(snapshot["action_capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|capability| capability["action"] == "ConfirmEnterNode"
            && capability["enabled"] == true));
    assert!(
        snapshot["action_capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|capability| capability["action"] == "SelectMapNode"
                && capability["enabled"] == true)
    );
}
