//! Actual native failure through public Core APIs, including terminal observation parity.
//! Usage: fault_runtime PACK_ROOT SELECTION OBSERVABILITY_RON MODE MAX_WORK
use game_core::{
    application::*,
    game::{
        behavior::{GameStateContextDto, RunSnapshotDto},
        data::{
            content_pack::{self, ContentSelection, InputLimits},
            ValidatedGameData,
        },
        map::{MapNodeCategory, MapNodeState},
    },
};
use serde_json::{json, Value};
#[cfg(feature = "runtime-observation")]
use std::time::Duration;
use std::{path::Path, sync::Arc};
use uuid::Uuid;
type Error = Box<dyn std::error::Error>;
fn command(value: Value) -> ApplicationCommand {
    ApplicationCommand::from_document(value.into()).expect("fixed public driver input")
}
fn execute(
    app: &mut CoreApplication,
    player: Uuid,
    value: Value,
) -> Result<ApplicationResult, ApplicationError> {
    app.execute(
        player,
        command(value),
        ApplicationExecutionContext::default(),
    )
}
fn snapshot(app: &CoreApplication) -> RunSnapshotDto {
    serde_json::from_value(
        app.snapshot(ApplicationSnapshotKind::Run)
            .unwrap()
            .read_model()
            .as_value()
            .clone(),
    )
    .unwrap()
}
fn waiting(data: ValidatedGameData, player: Uuid, seed: u64) -> Result<CoreApplication, Error> {
    let mut app = CoreApplication::new(data, seed);
    execute(
        &mut app,
        player,
        json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false}),
    )?;
    for _ in 0..128 {
        let view = snapshot(&app);
        if let Some(vn) = view.visual_novel {
            let value = if let Some(choice) = vn.choices.iter().find(|choice| choice.selectable) {
                json!({"type":"select_visual_novel_choice","session_id":vn.session_id,"current_scene_id":vn.current_scene_id,"choice_id":choice.choice_id})
            } else {
                json!({"type":"advance_visual_novel","session_id":vn.session_id,"current_scene_id":vn.current_scene_id})
            };
            execute(&mut app, player, value)?;
            continue;
        }
        if let GameStateContextDto::SelectingStarterEmployees {
            required_count,
            candidates,
        } = view.game_state_context
        {
            let ids: Vec<_> = candidates
                .into_iter()
                .take(required_count)
                .map(|row| row.employee_definition_id)
                .collect();
            execute(
                &mut app,
                player,
                json!({"type":"select_starter_employees","employee_definition_ids":ids}),
            )?;
            continue;
        }
        let map = view.map.ok_or("expected live map")?;
        if let Some(node) = map.nodes.iter().find(|node| {
            node.state == MapNodeState::Available && node.category == MapNodeCategory::Combat
        }) {
            execute(
                &mut app,
                player,
                json!({"type":"select_map_node","node_id":node.id}),
            )?;
            return Ok(app);
        }
        if let Some(node) = map.nodes.iter().find(|node| {
            node.state == MapNodeState::Available && node.category == MapNodeCategory::Start
        }) {
            execute(
                &mut app,
                player,
                json!({"type":"select_map_node","node_id":node.id}),
            )?;
            execute(&mut app, player, json!({"type":"confirm_enter_node"}))?;
            execute(&mut app, player, json!({"type":"complete_node"}))?;
            continue;
        }
        return Err("driver seed has no adjacent ordinary combat".into());
    }
    Err("driver frontmatter bound exceeded".into())
}
fn main() -> Result<(), Error> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err(
            "usage: fault_runtime PACK_ROOT SELECTION OBSERVABILITY_RON MODE MAX_WORK".into(),
        );
    }
    let selection: ContentSelection = serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let tiny_text = std::str::from_utf8(game_core::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON)?
        .replacen("max_work: 10000000", &format!("max_work: {}", args[5]), 1);
    let settings = Arc::new(game_core::runtime_safety::RuntimeSafetySettings::from_ron(
        "fault driver",
        tiny_text.as_bytes(),
    )?);
    let data = content_pack::load_with_runtime_safety(
        Path::new(&args[1]),
        &selection,
        &InputLimits::default(),
        settings,
    )?;
    let mode = args[4].as_str();
    let mut app = waiting(data, Uuid::from_u128(0x5203), 23)?;
    #[cfg(feature = "runtime-observation")]
    let service = if mode == "on" {
        game_server::observability::Observability::start(
            game_server::observability::ObservationConfig::from_path(Path::new(&args[3]))?,
        )?
    } else {
        None
    };
    #[cfg(feature = "runtime-observation")]
    let mut receiver = None;
    #[cfg(feature = "runtime-observation")]
    let mut reader = None;
    #[cfg(feature = "runtime-observation")]
    {
        if let Some((obs, _)) = &service {
            assert!(obs.attach(&mut app));
        }
        if mode == "queue_full" || mode == "detached" {
            let (sender, rx) = game_core::runtime_observation::channel();
            let (observer, rd) = sender.session(1);
            app.set_runtime_observer(Some(observer));
            reader = Some(rd);
            if mode == "queue_full" {
                receiver = Some(rx);
            }
        }
    }
    execute(
        &mut app,
        Uuid::from_u128(0x5203),
        json!({"type":"confirm_enter_node"}),
    )?;
    #[cfg(feature = "runtime-observation")]
    if let Some((obs, _)) = &service {
        let mut selected = false;
        for _ in 0..200 {
            let detail = obs.detail_json();
            if let Some(id) = detail["sessions"]
                .as_array()
                .and_then(|s| s.first())
                .and_then(|s| s["session"].as_u64())
            {
                obs.select(Some(id)).map_err(std::io::Error::other)?;
                selected = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(selected);
        std::thread::sleep(Duration::from_millis(50));
        println!("{}", json!({"before_observation":obs.detail_json()}));
    }
    #[cfg(feature = "runtime-observation")]
    if mode == "queue_full" {
        for _ in 0..game_core::runtime_observation::QUEUE_CAPACITY + 5 {
            assert_eq!(
                execute(
                    &mut app,
                    Uuid::from_u128(0x5203),
                    json!({"type":"pause_battle"})
                )
                .unwrap_err()
                .code(),
                "missing_battle_state_authority"
            );
        }
    }
    if mode == "small_ticks" {
        for n in 0..1000 {
            if !app.has_active_battle() {
                return Err("battle ended without fault".into());
            }
            if let Err(error) = app.advance_active_battle_for_host_tick(100, 0) {
                println!(
                    "{}",
                    json!({"tick":n,"error":error,"fault":app.battle_fault()})
                );
                assert!(app.battle_fault().is_some());
                return Ok(());
            }
        }
        return Err("no fault within tick bound".into());
    }
    let before = app.snapshot(ApplicationSnapshotKind::Run)?;
    let failure = app.advance_active_battle_by(60_000, 0).unwrap_err();
    assert_eq!(failure.code(), "battle_runtime_work_limit");
    let first = *app.battle_fault().unwrap();
    for _ in 0..64 {
        assert_eq!(
            app.advance_active_battle_for_host_tick(100, 0)
                .unwrap_err()
                .code(),
            "battle_faulted"
        );
        assert_eq!(app.battle_fault(), Some(&first));
    }
    let after = app.snapshot(ApplicationSnapshotKind::Run)?;
    let mut normalized = after.read_model().as_value().clone();
    normalized.as_object_mut().unwrap().remove("battle_fault");
    normalized["action_capabilities"] =
        before.read_model().as_value()["action_capabilities"].clone();
    normalized["game_state_context"]["can_retreat"] =
        before.read_model().as_value()["game_state_context"]["can_retreat"].clone();
    normalized["game_state_context"]["deployment"] =
        before.read_model().as_value()["game_state_context"]["deployment"].clone();
    assert_eq!(&normalized, before.read_model().as_value());
    println!(
        "{}",
        json!({"result":{"first_error":failure,"first_fault":first,"final_run":after,"repeat_count":64}})
    );
    #[cfg(feature = "runtime-observation")]
    {
        if let Some(rx) = receiver {
            assert!(rx.counters().dropped > 0);
            println!("{}", json!({"queue_dropped":rx.counters().dropped}));
        }
        if let Some(rd) = reader {
            assert!(rd.is_alive());
            assert!(rd.snapshot(false).unwrap().current.battle.is_none());
        }
        if let Some((obs, _)) = &service {
            let mut ended = false;
            for _ in 0..200 {
                let detail = obs.detail_json();
                if detail["selected_session"].is_null()
                    && detail["ring"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|r| r["boundary"] == "ended")
                {
                    assert!(detail["sessions"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|s| s["current"]["battle"].is_null()));
                    println!("{}", json!({"after_observation":detail}));
                    ended = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(
                ended,
                "collector must end selected faulted battle while owner stays alive"
            );
        }
    }
    Ok(())
}
