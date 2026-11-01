#[path = "../../../external_pack_probe.rs"]
mod external_pack_probe;

use game_core::application::{
    ApplicationCommand, ApplicationDocument, ApplicationExecutionContext,
    ApplicationResultDisposition, ApplicationResultKind, ApplicationSnapshotKind, CoreApplication,
};
use serde_json::{json, Value};
use uuid::Uuid;

fn command(value: Value) -> ApplicationCommand {
    ApplicationCommand::from_document(ApplicationDocument::new(value))
        .expect("application command must validate")
}

fn main() {
    let data = external_pack_probe::load_selected();
    let player_id = Uuid::from_u128(0xCA04_1001);
    let mut application = CoreApplication::new(data, 0xCA04_1001);

    let start = application
        .execute(
            player_id,
            command(json!({
                "type": "start_new_game",
                "game_mode": "Standard",
                "replace_active_run": false,
            })),
            ApplicationExecutionContext::default(),
        )
        .expect("application-owned run start must execute");
    assert_eq!(start.kind(), ApplicationResultKind::RunStartFlowStarted);
    let visual_novel = &start.payload().as_value()["visual_novel"];
    application
        .execute(
            player_id,
            command(json!({
                "type": "advance_visual_novel",
                "session_id": visual_novel["session_id"],
                "current_scene_id": visual_novel["current_scene_id"],
            })),
            ApplicationExecutionContext::default(),
        )
        .expect("live briefing must advance to starter selection");

    let selection = application
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("starter selection must project");
    let context = &selection.read_model().as_value()["game_state_context"];
    assert_eq!(context["type"], "selecting_starter_employees");
    let required_count = context["required_count"].as_u64().expect("required count") as usize;
    let employee_definition_ids = context["candidates"]
        .as_array()
        .expect("starter candidates")
        .iter()
        .take(required_count)
        .map(|candidate| candidate["employee_definition_id"].clone())
        .collect::<Vec<_>>();
    application
        .execute(
            player_id,
            command(json!({
                "type": "select_starter_employees",
                "employee_definition_ids": employee_definition_ids,
            })),
            ApplicationExecutionContext::default(),
        )
        .expect("starter selection must create a live run");

    let run = application
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("live map must project");
    let node_id = run.read_model().as_value()["map"]["nodes"]
        .as_array()
        .expect("map nodes")
        .iter()
        .find(|node| {
            node["state"] == "Available"
                && matches!(node["category"].as_str(), Some("Combat" | "Boss"))
        })
        .expect("live first map must expose an available combat route")["id"]
        .clone();
    application
        .execute(
            player_id,
            command(json!({ "type": "select_map_node", "node_id": node_id })),
            ApplicationExecutionContext::default(),
        )
        .expect("combat node selection must prepare the private kernel");
    application
        .execute(
            player_id,
            command(json!({ "type": "confirm_enter_node" })),
            ApplicationExecutionContext::default(),
        )
        .expect("combat confirmation must enter the private kernel");

    let battle = application
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("battle state must project");
    assert_eq!(battle.read_model().as_value()["game_state"], "in_battle");

    let reconnect = application
        .active_battle_reconnect(0)
        .expect("private bounded history must project through the application port");
    let reconnect_retry = application
        .active_battle_reconnect(0)
        .expect("repeated reconnect projection must be stable");
    assert_eq!(reconnect.kind(), ApplicationResultKind::BattleAdvanced);
    assert_eq!(
        reconnect.disposition(),
        ApplicationResultDisposition::ReadOnly
    );
    assert_eq!(reconnect, reconnect_retry);
    assert!(reconnect
        .payload()
        .as_value()
        .get("battle_update")
        .is_some());
}
