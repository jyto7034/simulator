use std::sync::Arc;

use game_core::game::{
    behavior::{BehaviorResult, PlayerBehavior},
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
    world::{AdminCommand, GameCore},
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
    let mut game_data =
        GameDataBase::load_live_embedded().expect("embedded live data must load for fixture");
    let mut events = game_data.event_data.events.clone();
    events.push(fixture_event());
    Arc::get_mut(&mut game_data)
        .expect("fixture generator must own the only GameDataBase Arc")
        .event_data = Arc::new(EventDatabase::new(events));

    let starter_ids = selected_starter_ids(game_data.as_ref());
    let player_id = Uuid::from_u128(0xE001);
    let mut core = GameCore::new(game_data, 0xD1CE);
    let start = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Standard,
                replace_active_run: false,
            },
        )
        .expect("fixture run should start");
    let BehaviorResult::RunStartFlowStarted {
        visual_novel: Some(visual_novel),
        ..
    } = start
    else {
        panic!("fixture run must open the pre-run briefing")
    };
    core.execute(
        player_id,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: visual_novel.session_id,
            current_scene_id: game_core::game::data::visual_novel_data::VisualNovelSceneId::new(
                visual_novel.current_scene_id,
            ),
        },
    )
    .expect("fixture briefing should close");
    core.execute(
        player_id,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids: starter_ids,
        },
    )
    .expect("fixture starter selection should begin the run");
    core.execute_admin_command(AdminCommand::AdminEnterEvent {
        event_id: "employee_check_fixture".to_string(),
    })
    .expect("fixture Event should enter through the real Core projection");

    let snapshot = core
        .get_run_snapshot_json()
        .expect("fixture root snapshot should serialize");
    let node_id = game_core::game::map::MapNodeId::new(
        Uuid::parse_str(
            snapshot["selected_event"]["node_id"]
                .as_str()
                .expect("fixture Event node id must be a string"),
        )
        .expect("fixture Event node id must be a UUID"),
    );
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
    core.execute(
        player_id,
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id,
            event_id: EventId::new("employee_check_fixture"),
            current_scene_id: EventSceneId::new("check"),
            choice_id: EventChoiceId::new("inspect"),
            employee_uuid,
        },
    )
    .expect("fixture resolve must use the real Core command path");
    let committed_snapshot = core
        .get_run_snapshot_json()
        .expect("committed fixture root snapshot should serialize");
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
