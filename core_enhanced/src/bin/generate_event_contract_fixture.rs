use std::sync::Arc;

use game_core::application::{
    ApplicationAdminCommand, ApplicationCommand, ApplicationDocument, ApplicationExecutionContext,
    ApplicationResultKind, ApplicationSnapshotKind, CoreApplication,
};
use game_core::game::{
    data::{
        event_data::{
            EventChoiceCheckDefinition, EventChoiceDefinition, EventChoiceId, EventChoiceKeyId,
            EventDatabase, EventDefinition, EventId, EventPresentationPanel, EventSceneDefinition,
            EventSceneId, EventSceneNext, EventScenePresentation,
        },
        GameDataBase,
    },
    employee::{EmployeeDefinition, EventAptitude},
    map::GameMode,
};
use serde_json::json;
use uuid::Uuid;

fn panel(image_id: &str, text_ids: &[&str]) -> EventScenePresentation {
    EventScenePresentation {
        panels: vec![EventPresentationPanel {
            image_id: image_id.to_string(),
            text_ids: text_ids
                .iter()
                .map(|text_id| (*text_id).to_string())
                .collect(),
        }],
    }
}

fn fixture_event() -> EventDefinition {
    EventDefinition {
        id: EventId::new("employee_check_fixture"),
        entry_scene_id: EventSceneId::new("check"),
        scenes: vec![
            EventSceneDefinition {
                id: EventSceneId::new("check"),
                presentation: EventScenePresentation {
                    panels: vec![
                        EventPresentationPanel {
                            image_id: "event_fixture_arrival".to_string(),
                            text_ids: vec![
                                "event_fixture_arrival_01".to_string(),
                                "event_fixture_arrival_02".to_string(),
                            ],
                        },
                        EventPresentationPanel {
                            image_id: "event_fixture_check".to_string(),
                            text_ids: vec!["event_fixture_check_01".to_string()],
                        },
                    ],
                },
                next: EventSceneNext::EmployeeCheckChoices {
                    choices: vec![
                        EventChoiceDefinition {
                            id: EventChoiceId::new("inspect"),
                            label_id: "event_fixture_choice_inspect".to_string(),
                            check: EventChoiceCheckDefinition {
                                aptitude: EventAptitude::Analysis,
                                dc: 16,
                                required_choice_key: Some(EventChoiceKeyId::new(
                                    "sealed_archive_access",
                                )),
                            },
                            success_action: None,
                            success_scene_id: EventSceneId::new("success"),
                            failure_scene_id: EventSceneId::new("failure"),
                        },
                        EventChoiceDefinition {
                            id: EventChoiceId::new("repair"),
                            label_id: "event_fixture_choice_repair".to_string(),
                            check: EventChoiceCheckDefinition {
                                aptitude: EventAptitude::Maintenance,
                                dc: 12,
                                required_choice_key: None,
                            },
                            success_action: None,
                            success_scene_id: EventSceneId::new("success"),
                            failure_scene_id: EventSceneId::new("failure"),
                        },
                    ],
                },
            },
            EventSceneDefinition {
                id: EventSceneId::new("success"),
                presentation: panel("event_fixture_success", &["event_fixture_success_01"]),
                next: EventSceneNext::End,
            },
            EventSceneDefinition {
                id: EventSceneId::new("failure"),
                presentation: panel("event_fixture_failure", &["event_fixture_failure_01"]),
                next: EventSceneNext::End,
            },
        ],
    }
}

fn selected_starter_ids(game_data: &GameDataBase) -> Vec<String> {
    ["maintenance_tech", "records_clerk", "field_medic"]
        .into_iter()
        .map(|definition_id| {
            let definition: &EmployeeDefinition = game_data
                .employee_definition_data
                .get_by_id(definition_id)
                .expect("fixture starter definition must exist");
            assert!(definition.starter_eligible);
            definition.employee_definition_id.clone()
        })
        .collect()
}

fn main() {
    let mut game_data = game_core::game::data::ValidatedGameData::load_live_embedded()
        .expect("embedded live data must load for fixture")
        .into_fixture_database_arc();
    let mut events = game_data.event_data.events.clone();
    events.push(fixture_event());
    Arc::get_mut(&mut game_data)
        .expect("fixture generator must own the only GameDataBase Arc")
        .event_data = Arc::new(EventDatabase::new(events));

    let starter_ids = selected_starter_ids(game_data.as_ref());
    let player_id = Uuid::from_u128(0xE001);
    let mut application = CoreApplication::new(game_data, 0xD1CE);
    let start = application
        .execute(
            player_id,
            ApplicationCommand::from_document(ApplicationDocument::new(json!({
                "type": "start_new_game",
                "game_mode": GameMode::Expedition,
                "replace_active_run": false,
            })))
            .expect("fixture start command must validate"),
            ApplicationExecutionContext::default(),
        )
        .expect("fixture run should start");
    assert_eq!(start.kind(), ApplicationResultKind::RunStartFlowStarted);
    let visual_novel = &start.payload().as_value()["visual_novel"];
    let session_id = visual_novel["session_id"]
        .as_str()
        .expect("fixture run must open the pre-run briefing");
    let current_scene_id = visual_novel["current_scene_id"]
        .as_str()
        .expect("fixture briefing must expose its current scene");
    application
        .execute(
            player_id,
            ApplicationCommand::from_document(ApplicationDocument::new(json!({
                "type": "advance_visual_novel",
                "session_id": session_id,
                "current_scene_id": current_scene_id,
            })))
            .expect("fixture Visual Novel command must validate"),
            ApplicationExecutionContext::default(),
        )
        .expect("fixture briefing should close");
    application
        .execute(
            player_id,
            ApplicationCommand::from_document(ApplicationDocument::new(json!({
                "type": "select_starter_employees",
                "employee_definition_ids": starter_ids,
                "loan_copy_ids": [],
            })))
            .expect("fixture starter command must validate"),
            ApplicationExecutionContext::default(),
        )
        .expect("fixture starter selection should begin the run");
    application
        .execute_admin(
            ApplicationAdminCommand::from_document(ApplicationDocument::new(json!({
                "type": "admin_enter_event",
                "event_id": "employee_check_fixture",
            })))
            .expect("fixture admin command must validate"),
        )
        .expect("fixture Event should enter through the real Core projection");

    let snapshot = application
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("fixture root snapshot should serialize")
        .read_model()
        .as_value()
        .clone();
    let node_id = snapshot["selected_event"]["node_id"]
        .as_str()
        .expect("fixture Event node id must be a string");
    let inspect_candidates = snapshot["selected_event"]["event"]["interaction"]["choices"][0]
        ["employee_candidates"]
        .as_array()
        .expect("inspect candidates must be an array");
    let employee_uuid = inspect_candidates
        .iter()
        .find(|candidate| candidate["eligibility"]["enabled"] == true)
        .and_then(|candidate| candidate["employee_uuid"].as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("fixture inspect choice must have one eligible key provider");
    application
        .execute(
            player_id,
            ApplicationCommand::from_document(ApplicationDocument::new(json!({
                "type": "resolve_event_choice_check",
                "node_id": node_id,
                "event_id": "employee_check_fixture",
                "current_scene_id": "check",
                "choice_id": "inspect",
                "employee_uuid": employee_uuid,
            })))
            .expect("fixture resolve command must validate"),
            ApplicationExecutionContext::default(),
        )
        .expect("fixture resolve must use the real Core command path");
    let committed_snapshot = application
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("committed fixture root snapshot should serialize")
        .read_model()
        .as_value()
        .clone();
    let output = json!({
        "game_state": snapshot["game_state"].clone(),
        "game_state_context": snapshot["game_state_context"].clone(),
        "action_capabilities": snapshot["action_capabilities"].clone(),
        "selected_event": snapshot["selected_event"].clone(),
        "committed_selected_event": committed_snapshot["selected_event"].clone(),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("Event fixture must serialize")
    );
}
