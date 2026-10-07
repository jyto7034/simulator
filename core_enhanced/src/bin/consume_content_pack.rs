//! Minimal external-pack runtime probe. General authoring CLI belongs to R03.
use game_core::{
    application::{
        ApplicationCommand, ApplicationCommandKind as Kind, ApplicationDocument,
        ApplicationExecutionContext, ApplicationSnapshotKind, CoreApplication,
    },
    game::{
        data::content_pack::{load, ContentSelection, InputLimits},
        map::GameMode,
    },
};
use serde_json::{json, Value};
use std::path::PathBuf;
use uuid::Uuid;

fn command(
    kind: Kind,
    value: Value,
) -> Result<ApplicationCommand, game_core::application::ApplicationError> {
    ApplicationCommand::try_new(kind, ApplicationDocument::new(value))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments.len() < 2 {
        return Err("usage: consume_content_pack TRUSTED_ROOT PACK_RELATIVE_DIRECTORY...".into());
    }
    let selection = ContentSelection {
        packs: arguments[1..].iter().map(PathBuf::from).collect(),
        winners: vec![],
    };
    let generation = load(
        &PathBuf::from(&arguments[0]),
        &selection,
        &InputLimits::default(),
    )?;
    let identity = generation.identity().clone();
    let receipt = generation.source_receipt().clone();
    let seed = 0xCA02_0001;
    let player = Uuid::from_u128(0xCA02_0001);
    let mut application = CoreApplication::new(generation, seed);
    let mut result = application.execute(
        player,
        command(
            Kind::StartNewGame,
            json!({"game_mode": GameMode::Standard, "replace_active_run": false}),
        )?,
        ApplicationExecutionContext::default(),
    )?;
    let mut briefing_scripts = Vec::new();
    for _ in 0..128 {
        let snapshot = application.snapshot(ApplicationSnapshotKind::Run)?;
        let state = snapshot.read_model().as_value();
        match state["game_state"].as_str() {
            Some("pre_run_briefing") => {
                let vn = &result.payload().as_value()["visual_novel"];
                if vn["session_id"].is_null() || vn["current_scene_id"].is_null() {
                    return Err("briefing did not return a current session and scene".into());
                }
                briefing_scripts.push(vn["script_id"].clone());
                result = application.execute(player, command(Kind::AdvanceVisualNovel,
                    json!({"session_id": vn["session_id"], "current_scene_id": vn["current_scene_id"]}))?,
                    ApplicationExecutionContext::default())?;
            }
            Some("selecting_starter_employees") => {
                let context = &state["game_state_context"];
                let count = context["required_count"]
                    .as_u64()
                    .ok_or("missing starter count")? as usize;
                let candidates = context["candidates"]
                    .as_array()
                    .ok_or("missing starter candidates")?;
                if candidates.len() < count {
                    return Err("insufficient starter candidates".into());
                }
                let ids: Vec<_> = candidates
                    .iter()
                    .take(count)
                    .map(|candidate| candidate["employee_definition_id"].clone())
                    .collect();
                result = application.execute(
                    player,
                    command(
                        Kind::SelectStarterEmployees,
                        json!({"employee_definition_ids": ids}),
                    )?,
                    ApplicationExecutionContext::default(),
                )?;
            }
            Some("viewing_map") => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "content_identity": identity,
                        "source_receipt": receipt,
                        "run_seed": seed,
                        "briefing_scripts": briefing_scripts,
                        "run_snapshot": state,
                    }))?
                );
                return Ok(());
            }
            _ => return Err("unexpected state in the bounded briefing/starter probe".into()),
        }
    }
    Err("briefing/starter probe exceeded 128 commands".into())
}
