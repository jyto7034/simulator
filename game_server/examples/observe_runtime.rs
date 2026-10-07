//! Local observation/parity driver through the production public Core boundary.
//! Usage: observe_runtime PACK_ROOT SELECTION OBSERVABILITY_RON MODE STEPS PAUSE_MS
//! MODE: off, none, detached, queue_full, stalled, on. No saves or remote state are changed.
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
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
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
fn battle_command(app: &mut CoreApplication, player: Uuid, kind: &str) -> ApplicationResult {
    let projection = app.active_battle_reconnect(0).unwrap();
    let update = &projection.payload().as_value()["battle_update"];
    let authority = ApplicationBattleAuthority {
        battle_uuid: serde_json::from_value(update["battle_uuid"].clone()).unwrap(),
        battle_state_revision: update["battle_state_revision"].as_u64().unwrap(),
    };
    app.execute(
        player,
        command(json!({"type":kind})),
        ApplicationExecutionContext {
            expected_battle_authority: Some(authority),
            ..Default::default()
        },
    )
    .unwrap()
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
fn output<T: serde::Serialize>(step: &str, result: &T, elapsed: Duration) {
    println!(
        "{}",
        json!({"step":step,"operation_wall_ns":elapsed.as_nanos(),"result":result})
    );
}
fn main() -> Result<(), Error> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 7 {
        return Err(
            "usage: observe_runtime PACK_ROOT SELECTION OBSERVABILITY_RON MODE STEPS PAUSE_MS"
                .into(),
        );
    }
    let selection: ContentSelection = serde_json::from_slice(&std::fs::read(&args[2])?)?;
    let data = content_pack::load(Path::new(&args[1]), &selection, &InputLimits::default())?;
    let mode = args[4].as_str();
    let steps: usize = args[5].parse()?;
    let pause: u64 = args[6].parse()?;
    if !["off", "none", "detached", "queue_full", "stalled", "on"].contains(&mode) {
        return Err("unknown observation mode".into());
    }
    #[cfg(not(feature = "runtime-observation"))]
    if mode != "off" {
        return Err("this binary has no runtime-observation feature".into());
    }
    #[cfg(feature = "runtime-observation")]
    let service = if mode == "on" || mode == "stalled" {
        game_server::observability::Observability::start(
            game_server::observability::ObservationConfig::from_path(Path::new(&args[3]))?,
        )?
    } else {
        None
    };
    // Hold an actual exporter epoch reader to exercise retirement backpressure.
    #[cfg(feature = "runtime-observation")]
    let _stalled_reader = (mode == "stalled").then(crossbeam_epoch::pin);
    #[cfg(feature = "runtime-observation")]
    let mut held_receiver = None;
    #[cfg(feature = "runtime-observation")]
    let mut standalone_sender = None;
    let player = Uuid::from_u128(0x5202);
    let mut app = waiting(data.clone(), player, 23)?;
    #[cfg(feature = "runtime-observation")]
    {
        if let Some((obs, _)) = &service {
            obs.attach(&mut app);
        }
        if mode == "detached" || mode == "queue_full" {
            let (sender, receiver) = game_core::runtime_observation::channel();
            let (observer, _) = sender.session(1);
            app.set_runtime_observer(Some(observer));
            standalone_sender = Some(sender);
            if mode == "queue_full" {
                held_receiver = Some(receiver);
            }
        }
    }
    let start = Instant::now();
    let entry = app
        .prepare_non_battle_command(
            player,
            command(json!({"type":"confirm_enter_node"})),
            ApplicationExecutionContext::default(),
        )?
        .commit();
    output("entry", &entry, start.elapsed());
    // Real public authority rejection; no game state or native allowance changes.
    let start = Instant::now();
    let rejection = execute(&mut app, player, json!({"type":"pause_battle"})).unwrap_err();
    output("authority_rejection", &rejection, start.elapsed());
    #[cfg(feature = "runtime-observation")]
    if let Some((obs, _)) = &service {
        for _ in 0..100 {
            let detail = obs.detail_json();
            if let Some(session) = detail["sessions"]
                .as_array()
                .and_then(|rows| rows.iter().find(|row| !row["current"]["battle"].is_null()))
                .and_then(|row| row["session"].as_u64())
            {
                obs.select(Some(session)).map_err(std::io::Error::other)?;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    // A second immutable profile demonstrates an actual native rejection. This
    // is a local driver input, never an observability setting or live reload.
    let tiny_text = std::str::from_utf8(game_core::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON)?
        .replacen("max_work: 10000000", "max_work: 1000", 1);
    let tiny = Arc::new(game_core::runtime_safety::RuntimeSafetySettings::from_ron(
        "driver tiny native profile",
        tiny_text.as_bytes(),
    )?);
    let tiny_data = content_pack::load_with_runtime_safety(
        Path::new(&args[1]),
        &selection,
        &InputLimits::default(),
        tiny,
    )?;
    let mut refused = waiting(tiny_data, Uuid::from_u128(0x5203), 23)?;
    #[cfg(feature = "runtime-observation")]
    {
        if let Some((obs, _)) = &service {
            obs.attach(&mut refused);
        }
        if let Some(sender) = &standalone_sender {
            let (observer, _) = sender.session(2);
            refused.set_runtime_observer(Some(observer));
        }
    }
    let start = Instant::now();
    let second_entry = execute(
        &mut refused,
        Uuid::from_u128(0x5203),
        json!({"type":"confirm_enter_node"}),
    )?;
    output("second_entry", &second_entry, start.elapsed());
    let start = Instant::now();
    let failure = refused.advance_active_battle_by(60_000, 0).unwrap_err();
    output("native_rejection", &failure, start.elapsed());
    for kind in ["pause_battle", "resume_battle"] {
        let start = Instant::now();
        let result = battle_command(&mut app, player, kind);
        output(kind, &result, start.elapsed());
    }
    let start = Instant::now();
    let concentrated = app.advance_active_battle_by(2000, 0);
    output("concentrated_tick", &concentrated, start.elapsed());
    for step in 0..steps {
        if app.has_active_battle() {
            let start = Instant::now();
            let result = app.advance_active_battle_by(10, 0);
            output(&format!("tick_{step}"), &result, start.elapsed());
        }
        if pause != 0 {
            std::thread::sleep(Duration::from_millis(pause));
        }
    }
    output(
        "final_run",
        &app.snapshot(ApplicationSnapshotKind::Run)?,
        Duration::ZERO,
    );
    if app.has_active_battle() {
        output(
            "final_battle",
            &app.active_battle_reconnect(0)?,
            Duration::ZERO,
        );
    }
    if app.has_active_battle() {
        let start = Instant::now();
        let result = battle_command(&mut app, player, "retreat_battle");
        output("end_battle", &result, start.elapsed());
    }
    drop(refused);
    if pause != 0 {
        std::thread::sleep(Duration::from_secs(3));
    }
    #[cfg(feature = "runtime-observation")]
    {
        if let Some(receiver) = held_receiver {
            let counters = receiver.counters();
            println!(
                "{}",
                json!({"observation":{"queue_dropped":counters.dropped,"commits":counters.commits}})
            );
        }
        if let Some((obs, _)) = &service {
            println!("{}", json!({"observation":obs.detail_json()}));
        }
    }
    Ok(())
}
