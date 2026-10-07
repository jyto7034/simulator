use game_core::{
    application::{
        ApplicationCommand, ApplicationExecutionContext, ApplicationSnapshotKind, CoreApplication,
    },
    game::data::ValidatedGameData,
};
use serde_json::json;
use uuid::Uuid;

fn main() {
    let game_data = ValidatedGameData::load_live_embedded()
        .expect("embedded live employee definitions must be valid");
    let recruitment_definition = serde_json::to_value(
        game_data
            .employee_definition_data
            .recruitment_definitions()
            .into_iter()
            .next()
            .expect("live catalog must contain a recruitment definition"),
    )
    .expect("recruitment definition must serialize");
    let mut core = CoreApplication::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    let start = core
        .execute(
            player_id,
            ApplicationCommand::from_document(
                json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false})
                    .into(),
            )
            .unwrap(),
            ApplicationExecutionContext::default(),
        )
        .expect("run start must succeed");
    let visual_novel = &start.payload().as_value()["visual_novel"];
    core.execute(player_id, ApplicationCommand::from_document(json!({
        "type":"advance_visual_novel", "session_id":visual_novel["session_id"], "current_scene_id":visual_novel["current_scene_id"]
    }).into()).unwrap(), ApplicationExecutionContext::default()).expect("HQ briefing must advance to starter selection");

    let selecting = core
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("starter snapshot must serialize")
        .read_model()
        .as_value()
        .clone();
    let candidate = selecting["game_state_context"]["candidates"][0].clone();
    let required_count = selecting["game_state_context"]["required_count"]
        .as_u64()
        .expect("required_count must be numeric") as usize;
    let employee_definition_ids: Vec<_> = selecting["game_state_context"]["candidates"]
        .as_array()
        .expect("starter candidates must be an array")
        .iter()
        .take(required_count)
        .map(|value| {
            value["employee_definition_id"]
                .as_str()
                .expect("candidate identity must be a string")
                .to_string()
        })
        .collect();
    core.execute(
        player_id,
        ApplicationCommand::from_document(json!({"type":"select_starter_employees","employee_definition_ids":employee_definition_ids}).into()).unwrap(),
        ApplicationExecutionContext::default(),
    )
    .expect("starter selection must succeed");
    let running = core
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("roster snapshot must serialize")
        .read_model()
        .as_value()
        .clone();
    let employee = &running["roster"]["employees"][0];

    let fixture = json!({
        "generator": "game_server/examples/generate_employee_profile_contract_fixture.rs",
        "starter_state": {
            "game_state_context": {
                "type": "selecting_starter_employees",
                "required_count": 1,
                "candidates": [candidate],
            },
        },
        "recruitment_state": {
            "selected_event": {
                "type": "headquarters_contact",
                "recruitment_candidates": [recruitment_definition],
            },
        },
        "roster_state": {
            "roster": {
                "employees": [{
                    "uuid": employee["uuid"],
                    "employee_definition_id": employee["employee_definition_id"],
                    "name": employee["name"],
                    "profile": employee["profile"],
                    "psychology": employee["psychology"],
                    "condition": employee["condition"],
                    "eligibility": employee["eligibility"],
                }],
            },
        },
    });

    if let Some(path) = std::env::args().nth(1) {
        let checked_in: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&path).expect("checked-in fixture must be readable"),
        )
        .expect("checked-in fixture must be valid JSON");
        assert_eq!(
            checked_in, fixture,
            "checked-in fixture differs from typed Rust serialization: {path}"
        );
        return;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&fixture).expect("fixture must serialize")
    );
}
