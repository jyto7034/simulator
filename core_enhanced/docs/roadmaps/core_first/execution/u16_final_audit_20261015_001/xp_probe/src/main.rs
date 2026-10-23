//! Replay a recorded public host workload, measuring public Core operations.
//! No issuer, approval bypass, private state mutation, or persistence claim.
use game_core::{
    application::{ApplicationBattleAuthority, ApplicationCommand, ApplicationExecutionContext,
        ApplicationResult, ApplicationSnapshotKind, CoreApplication},
    game::data::content_pack,
    runtime_observation::{begin_lab_window, OperationSummary},
    runtime_safety::RuntimeSafetySettings,
};
use game_server::runtime_measurement::{begin_process_window, process_heap, process_rss, HeapWindow};
use serde::Serialize;
use serde_json::{json, Value};
use std::{error::Error, fs, io::Write, path::Path, sync::Arc, time::Instant};
use uuid::Uuid;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[global_allocator]
static ALLOCATOR: game_server::runtime_measurement::CountingAllocator<'static, std::alloc::System> =
    game_server::runtime_measurement::process_allocator();

#[derive(Serialize)]
struct Stage {
    index: usize,
    kind: &'static str,
    seconds: f64,
    heap: HeapWindow,
    native: OperationSummary,
    accepted: bool,
}

fn measured<T>(index: usize, kind: &'static str, f: impl FnOnce() -> Result<T>) -> Result<(Result<T>, Stage)> {
    let observation = begin_lab_window().ok_or("observation already active")?;
    let heap = begin_process_window().map_err(|e| -> Box<dyn Error> { e.into() })?;
    let start = Instant::now();
    let result = f();
    let seconds = start.elapsed().as_secs_f64();
    let heap = heap.finish();
    let native = observation.finish().ok_or("native observation missing")?;
    assert!(!heap.boundary_updates_unavailable && heap.retiring_hooks_at_end == 0
        && !heap.baseline.counter_overflowed && !heap.end.counter_overflowed);
    let accepted = result.is_ok();
    Ok((result, Stage { index, kind, seconds, heap, native, accepted }))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 3, "config trace.jsonl new-output.json");
    let config: Value = serde_json::from_slice(&fs::read(&args[0])?)?;
    let rows: Vec<Value> = fs::read_to_string(&args[1])?.lines()
        .map(serde_json::from_str).collect::<std::result::Result<_, _>>()?;
    assert!(!rows.is_empty() && rows.len() <= 4096, "finite observed workload");
    let selection = serde_json::from_slice(&fs::read(config["base_selection_path"].as_str().ok_or("selection")?)?)?;
    let settings = Arc::new(RuntimeSafetySettings::from_ron("unchanged trusted host settings",
        &fs::read(config["runtime_safety_path"].as_str().ok_or("settings")?)?)?);
    let load_start = Instant::now();
    let data = content_pack::load_with_runtime_safety(
        Path::new(config["content_root"].as_str().ok_or("content")?), &selection,
        &content_pack::InputLimits::default(), settings)?;
    let load_seconds = load_start.elapsed().as_secs_f64();
    let mut stages = Vec::with_capacity(rows.len() * 2);
    let mut output = fs::OpenOptions::new().write(true).create_new(true).open(format!("{}.trace.jsonl", args[2]))?;
    let baseline = process_heap();
    let rss_before = process_rss();
    let mut app = CoreApplication::new(data.clone(), 12077);
    let player = Uuid::from_u128(0xc0de1200000000000000000000001001);
    let mut failure = None;
    let mut operations = 0;
    for (index, row) in rows.iter().enumerate() {
        let request = &row["request"];
        // Real IO fault injection belongs to the normal host. Skip its rejected
        // candidate here; the next identical request replays its actual commit.
        if request["fail"] == true { continue; }
        let run = app.snapshot(ApplicationSnapshotKind::Run)?;
        let run = run.read_model().as_value();
        let context = ApplicationExecutionContext {
            expected_battle_authority: if run["game_state"] == "in_battle" {
                Some(ApplicationBattleAuthority {
                    battle_uuid: serde_json::from_value(run["game_state_context"]["battle_uuid"].clone())?,
                    battle_state_revision: run["game_state_context"]["battle_state_revision"].as_u64().ok_or("revision")?,
                })
            } else { None },
            ..Default::default()
        };
        let (result, stage) = measured(index, "operation", || -> Result<Option<ApplicationResult>> {
            Ok(match request["op"].as_str().ok_or("op")? {
                "command" => Some(app.execute(player, ApplicationCommand::from_document(request["command"].clone().into())?, context)?),
                "tick" => Some(app.advance_active_battle_by(request["delta_ms"].as_u64().ok_or("delta")?, 0)?),
                "reconnect" => Some(app.active_battle_reconnect(0)?),
                "snapshot" => None,
                _ => return Err("unknown recorded operation".into()),
            })
        })?;
        let accepted = result.is_ok();
        stages.push(stage);
        let (snapshots, stage) = measured(index, "snapshots", || {
            Ok((app.snapshot(ApplicationSnapshotKind::Run)?, app.snapshot(ApplicationSnapshotKind::Inventory)?))
        })?;
        stages.push(stage);
        let (run, inventory) = snapshots?;
        let run = run.read_model().as_value();
        let inventory = inventory.read_model().as_value();
        let expected = &row["response"];
        let matches = accepted == (request["reject"] != true)
            && run["game_state"] == expected["run"]["game_state"]
            // XP output is independently checked by the audit runner below.
            // Authored XP rewards differ from the historical fixture.
            && *inventory == expected["inventory"];
        serde_json::to_writer(&mut output, &json!({"index":index,"request":request,"accepted":accepted,
            "matches_recorded_host_gameplay":matches,"error":result.as_ref().err().map(|e|e.to_string()),
            "run":run,"inventory":inventory}))?;
        writeln!(&mut output)?;
        operations += 1;
        if !matches { failure = Some(index); break; }
    }
    let rss_after_operations = process_rss();
    drop(app);
    let after = process_heap();
    let rss_after_drop = process_rss();
    let recovered = baseline.live_requested_bytes == after.live_requested_bytes;
    fs::write(&args[2], serde_json::to_vec_pretty(&json!({
        "passed":failure.is_none() && recovered,"failure_index":failure,"operations":operations,
        "scope":"audit variant: supported RON XP curve/reward; normal public CoreApplication, no host approval or persistence claim",
        "source_trace":args[1],"content_identity":data.identity(),"source_receipt":data.source_receipt(),
        "load_seconds":load_seconds,"stages":stages,"baseline":baseline,"after":after,
        "exact_requested_bytes_recovered":recovered,"rss_before":rss_before,
        "rss_after_operations":rss_after_operations,"rss_after_drop":rss_after_drop,
    }))?)?;
    if failure.is_some() || !recovered { return Err("diagnostic mismatch or retained bytes; raw evidence preserved".into()); }
    Ok(())
}
