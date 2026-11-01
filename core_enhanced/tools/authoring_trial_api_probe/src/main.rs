//! Standalone default-release consumer. Fixture selection is confined here.
use game_core::{
    application::{
        ApplicationCommand, ApplicationDocument, ApplicationExecutionContext,
        ApplicationSnapshotKind, CoreApplication,
    },
    authoring::{
        self, AuthoringContent, CancellationToken, ExplainQuery, TrialScenario, TrialSession,
    },
    game::data::content_pack::ContentSelection,
};
use serde_json::{json, Value};
use std::{fs, path::Path};
use uuid::Uuid;

fn command(app: &mut CoreApplication, player: Uuid, input: Value) -> Value {
    serde_json::to_value(
        app.execute(
            player,
            ApplicationCommand::from_document(ApplicationDocument::new(input)).unwrap(),
            ApplicationExecutionContext::default(),
        )
        .unwrap(),
    )
    .unwrap()
}
fn scenario(content: &AuthoringContent) -> Value {
    let player = Uuid::from_u128(0xCA03_0001);
    let mut app = CoreApplication::new(content.token().clone(), 77);
    let start = command(
        &mut app,
        player,
        json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false}),
    );
    command(
        &mut app,
        player,
        json!({"type":"advance_visual_novel","session_id":start["payload"]["visual_novel"]["session_id"],
        "current_scene_id":start["payload"]["visual_novel"]["current_scene_id"]}),
    );
    let snapshot =
        serde_json::to_value(app.snapshot(ApplicationSnapshotKind::Run).unwrap()).unwrap();
    let context = &snapshot["read_model"]["game_state_context"];
    let count = context["required_count"].as_u64().unwrap() as usize;
    let ids: Vec<_> = context["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .take(count)
        .map(|entry| entry["employee_definition_id"].clone())
        .collect();
    assert_eq!(ids.len(), count);
    let selected = command(
        &mut app,
        player,
        json!({"type":"select_starter_employees","employee_definition_ids":ids}),
    );
    let node = selected["payload"]["map"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["state"] == "Available" && node["category"] == "Combat")
        .unwrap();
    let bindings: Vec<_> = (0..count).map(|index| json!({"from":{"step":2,
        "pointer":format!("/read_model/game_state_context/candidates/{index}/employee_definition_id")},
        "to":format!("/command/employee_definition_ids/{index}"),"value_type":"string"})).collect();
    json!({"initial":{"player_id":player,"run_seed":77,"profile":"default_memory"},"preparation_steps":[
        {"kind":"command","command":{"type":"start_new_game","game_mode":"Standard","replace_active_run":false}},
        {"kind":"command","command":{"type":"advance_visual_novel","session_id":null,"current_scene_id":null},"bindings":[
            {"from":{"step":0,"pointer":"/payload/visual_novel/session_id"},"to":"/command/session_id","value_type":"string"},
            {"from":{"step":0,"pointer":"/payload/visual_novel/current_scene_id"},"to":"/command/current_scene_id","value_type":"string"}]},
        {"kind":"snapshot","snapshot":"run"},
        {"kind":"command","command":{"type":"select_starter_employees","employee_definition_ids":vec![Value::Null;count]},"bindings":bindings},
        {"kind":"command","command":{"type":"select_map_node","node_id":null},"bindings":[
            {"from":{"step":3,"pointer":format!("/payload/map/nodes/{node}/id")},"to":"/command/node_id","value_type":"string"}]},
        {"kind":"command","command":{"type":"confirm_enter_node"}},
        {"kind":"command","command":{"type":"pause_battle"},"context":{"expected_battle_authority":{"battle_uuid":null,"battle_state_revision":null}},"bindings":[
            {"from":{"step":5,"pointer":"/payload/battle_uuid"},"to":"/context/expected_battle_authority/battle_uuid","value_type":"string"},
            {"from":{"step":5,"pointer":"/payload/battle_update/battle_state_revision"},"to":"/context/expected_battle_authority/battle_state_revision","value_type":"number"}]},
        {"kind":"clock","clock":{"mode":"host_tick","delta_ms":1,"after_seq":0}},
        {"kind":"assert","from":{"step":5,"pointer":"/kind"},"assertion":{"test":"type","value_type":"string"}}
    ],"steps":[{"kind":"snapshot","snapshot":"run"},{"kind":"clock","clock":{"mode":"next_event_bucket"}}]})
}
fn receipts(session: &TrialSession) -> String {
    let mut result = session.receipt_documents().collect::<Vec<_>>().join("\n");
    result.push('\n');
    result
}
fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    assert_eq!(args.len(), 3);
    let root = Path::new(&args[0]);
    let output = Path::new(&args[2]);
    fs::create_dir(output).unwrap();
    let selection = ContentSelection {
        packs: vec![(&args[1]).into()],
        winners: vec![],
    };
    let cancel = CancellationToken::default();
    let limits = authoring::AuthoringLimits::default();
    let load = || authoring::validate(root, &selection, &limits, &cancel).unwrap();
    let content = load();
    let validated = authoring::validation_receipt(&content, &limits.execution, &cancel).unwrap();
    let mut exact = limits.execution.clone();
    exact.max_result_bytes = validated.document().len();
    exact.max_work = validated.usage().work;
    assert_eq!(
        authoring::validation_receipt(&content, &exact, &cancel)
            .unwrap()
            .document(),
        validated.document()
    );
    exact.max_work -= 1;
    assert!(matches!(
        authoring::validation_receipt(&content, &exact, &cancel),
        Err(authoring::AuthoringError::Execution { .. })
    ));
    exact.max_work = limits.execution.max_work;
    exact.max_result_bytes -= 1;
    assert!(matches!(
        authoring::validation_receipt(&content, &exact, &cancel),
        Err(authoring::AuthoringError::Execution { .. })
    ));
    exact = limits.execution.clone();
    exact.max_definitions = content.token().source_receipt().files.len() - 1;
    assert!(
        matches!(authoring::validation_receipt(&content, &exact, &cancel),
        Err(authoring::AuthoringError::Execution { diagnostic }) if diagnostic.site == "validation_receipt_files")
    );
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(matches!(
        authoring::validation_receipt(&content, &limits.execution, &cancelled),
        Err(authoring::AuthoringError::Execution { .. })
    ));
    assert_eq!(
        authoring::validation_receipt(&content, &limits.execution, &cancel)
            .unwrap()
            .document(),
        validated.document()
    );
    fs::write(
        output.join("selection.json"),
        serde_json::to_vec(&selection).unwrap(),
    )
    .unwrap();
    fs::write(
        output.join("validate.json"),
        authoring::validation_receipt(&content, &limits.execution, &cancel)
            .unwrap()
            .document(),
    )
    .unwrap();
    let description = authoring::describe(&content, &limits.execution, &cancel).unwrap();
    fs::write(output.join("describe.json"), description.document()).unwrap();
    let described: Value = serde_json::from_str(description.document()).unwrap();
    let query: ExplainQuery =
        serde_json::from_value(json!({"query":"definition","key":described["definitions"][0][0]}))
            .unwrap();
    fs::write(
        output.join("explain.json"),
        serde_json::to_vec(&query).unwrap(),
    )
    .unwrap();
    fs::write(
        output.join("explain_result.json"),
        authoring::explain(&content, &query, &limits.execution, &cancel)
            .unwrap()
            .document(),
    )
    .unwrap();
    let document = scenario(&content);
    write_json(&output.join("scenario.json"), &document);
    let parse = |value: &Value| {
        TrialScenario::from_json_slice(&serde_json::to_vec(value).unwrap(), &Default::default())
            .unwrap()
    };
    let session =
        authoring::simulate(content, parse(&document), Default::default(), &cancel).unwrap();
    assert!(session.is_complete());
    assert_eq!(session.next_step_index(), 11);
    assert!(session.step(7).unwrap().public_result().is_none());
    assert_eq!(session.receipt_documents().count(), 13);
    assert!(session.lifecycle().is_ready());
    assert_eq!(
        session
            .observation()
            .unwrap()
            .active()
            .unwrap()
            .key()
            .unwrap(),
        session.trace().unwrap().active().unwrap().key()
    );
    fs::write(output.join("receipts.ndjson"), receipts(&session)).unwrap();
    let mut partial = document.clone();
    partial["steps"] = json!([{"kind":"snapshot","snapshot":"run"},{"kind":"command","command":{"type":"unknown_command"}}]);
    write_json(&output.join("partial.json"), &partial);
    let failure =
        authoring::simulate(load(), parse(&partial), Default::default(), &cancel).unwrap_err();
    assert_eq!(failure.accepted().unwrap().next_step_index(), 10);
    fs::write(
        output.join("partial_receipts.ndjson"),
        receipts(failure.accepted().unwrap()),
    )
    .unwrap();
    fs::write(
        output.join("partial_failure.json"),
        authoring::render_trial_failure(failure.failure(), &Default::default(), &cancel)
            .unwrap()
            .document(),
    )
    .unwrap();
    let mut unprepared = document;
    unprepared["preparation_steps"] = json!([]);
    write_json(&output.join("unprepared.json"), &unprepared);
    let failure =
        authoring::simulate(load(), parse(&unprepared), Default::default(), &cancel).unwrap_err();
    assert!(failure.accepted().is_none());
    fs::write(
        output.join("preparation_failure.json"),
        authoring::render_trial_failure(failure.failure(), &Default::default(), &cancel)
            .unwrap()
            .document(),
    )
    .unwrap();
    println!(
        "{}",
        json!({"passed":true,"external_public_consumer":true,"accepted_steps":11,"receipts":13,"partial_accepted_steps":10})
    );
}
