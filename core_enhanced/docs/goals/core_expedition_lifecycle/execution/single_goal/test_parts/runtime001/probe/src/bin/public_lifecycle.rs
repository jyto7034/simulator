//! Direct public Core API, default features and explicitly supplied external content.
//! This is a public boundary oracle; normal admission is a separate consumer.
use game_core::{application::{ApplicationCommand, ApplicationExecutionContext, ApplicationSnapshotKind, CoreApplication},
    game::data::ValidatedGameData};
use serde_json::{json, Value};
use std::{error::Error, fs, io::{Read,Write}, path::{Path,PathBuf}, sync::Arc};
use uuid::Uuid;
type Result<T> = std::result::Result<T, Box<dyn Error>>;
const PLAYER: Uuid = Uuid::from_u128(0xc0de1200000000000000000000002001);

struct DirectHost { app: CoreApplication, data: ValidatedGameData }
impl u12_independent_lifecycle_probe::Host for DirectHost {
    fn app(&self) -> &CoreApplication { &self.app }
    fn command(&mut self, document: Value, context: ApplicationExecutionContext) -> Result<Value> {
        let result=self.app.execute(PLAYER,ApplicationCommand::from_document(document.into())?,context)?;
        Ok(result.payload().as_value().clone())
    }
    fn tick(&mut self, delta_ms:u64, after_seq:u64) -> Result<Option<Value>> {
        Ok(self.app.prepare_active_battle_for_host_tick(delta_ms,after_seq)?.commit()
            .map(|result|result.payload().as_value().clone()))
    }
    fn restart(&mut self) -> Result<()> {
        let checkpoint=self.app.export_active_run_checkpoint()?.ok_or("Continue checkpoint required")?;
        let profile=snapshot(&self.app,ApplicationSnapshotKind::PlayerProfile)?;
        let mut next=CoreApplication::new(self.data.clone(),12077);
        next.install_authoritative_player_profile(profile.into())?;
        next.continue_from_active_run_checkpoint(PLAYER,&checkpoint)?;
        self.app=next;
        Ok(())
    }
}

fn roots(app: &CoreApplication) -> Result<Value> {
    Ok(json!({"snapshots":ApplicationSnapshotKind::ALL.iter().map(|kind| app.snapshot(*kind))
        .collect::<std::result::Result<Vec<_>, _>>()?,
        "continue":app.export_active_run_checkpoint()?.map(|checkpoint| checkpoint.core_payload_json().to_vec())}))
}
fn snapshot(app: &CoreApplication, kind: ApplicationSnapshotKind) -> Result<Value> {
    Ok(app.snapshot(kind)?.read_model().as_value().clone())
}
fn execute(app: &mut CoreApplication, command: Value) -> Result<Value> {
    Ok(serde_json::to_value(app.execute(PLAYER, ApplicationCommand::from_document(command.into())?, ApplicationExecutionContext::default())?)?)
}
fn reject(app: &mut CoreApplication, command: Value, evidence: &mut Vec<Value>) -> Result<()> {
    let before = roots(app)?;
    let result = execute(app, command.clone());
    assert!(result.is_err(), "negative command accepted: {command}");
    assert_eq!(roots(app)?, before);
    evidence.push(json!({"command":command,"rejected":true,"error":result.unwrap_err().to_string(),"all_roots_unchanged":true}));
    Ok(())
}
fn finish_briefing(app: &mut CoreApplication, evidence: &mut Vec<Value>) -> Result<Value> {
    for _ in 0..128 {
        let run = snapshot(app, ApplicationSnapshotKind::Run)?;
        let vn = &run["visual_novel"];
        if vn.is_null() { return Ok(run); }
        let command = if let Some(choice) = vn["choices"].as_array().and_then(|choices| choices.iter().find(|choice| choice["selectable"] == true)) {
            json!({"type":"select_visual_novel_choice","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"],"choice_id":choice["choice_id"]})
        } else {
            json!({"type":"advance_visual_novel","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"]})
        };
        let result = execute(app, command.clone())?;
        evidence.push(json!({"command":command,"result":result}));
    }
    Err("briefing did not terminate".into())
}

fn bounded(path:&Path)->Result<Vec<u8>> {
    let metadata=fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len()>65536 {return Err("regular input no larger than 64 KiB required".into());}
    let mut bytes=Vec::new();fs::File::open(path)?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len()>65536 {return Err("input exceeded 64 KiB while reading".into());}Ok(bytes)
}

fn main() -> Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if !(4..=6).contains(&args.len()) {return Err("usage: public_lifecycle <new-result.json> <content-root> <selection.json> <runtime-safety.ron> [Tutorial|Expedition] [hq|battle|save|floor|complete|defeat_then_abandon|failure_battle|failure_then_depart]".into());}
    let path=PathBuf::from(&args[0]);let content_root=PathBuf::from(&args[1]);
    let selection_path=PathBuf::from(&args[2]);let runtime_path=PathBuf::from(&args[3]);
    let selection:game_core::game::data::content_pack::ContentSelection=serde_json::from_slice(&bounded(&selection_path)?)?;
    let settings=Arc::new(game_core::runtime_safety::RuntimeSafetySettings::from_ron("U12 explicit direct host profile",&bounded(&runtime_path)?)?);
    let data=game_core::game::data::content_pack::load_with_runtime_safety(&content_root,&selection,
        &game_core::game::data::content_pack::InputLimits::default(),settings)?;
    let source=json!({"content_root":content_root,"selection_path":selection_path,"runtime_safety_path":runtime_path,
        "identity":data.identity(),"source_receipt":data.source_receipt(),"runtime_safety_sha256":data.runtime_safety().effective_sha256(),
        "approval_claimed":false});
    // Optional actual-route mode follows the explicit external inputs. The original protocol oracle remains the
    // default and is not silently replaced by a tactically chosen role set.
    if let Some(mode)=args.get(4) {
        let mode=mode.to_str().ok_or("UTF-8 mode required")?;
        let boundary=args.get(5).map(|value|value.to_str().ok_or("UTF-8 boundary required")).transpose()?.unwrap_or("save");
        let mut host=DirectHost {app:CoreApplication::new(data.clone(),12077),data};
        let trace_path=path.with_extension("trace.jsonl");
        let result=u12_independent_lifecycle_probe::walk(&mut host,mode,boundary,&trace_path);
        let document=json!({"passed":result.is_ok(),"scope":"actual direct public lifecycle route",
            "source":source,"mode":mode,"boundary":boundary,"trace":trace_path,"result":result.as_ref().ok(),
            "error":result.as_ref().err().map(ToString::to_string),"final_roots":roots(&host.app)?});
        let mut file=fs::OpenOptions::new().create_new(true).write(true).open(&path)?;
        file.write_all(&serde_json::to_vec_pretty(&document)?)?;file.write_all(b"\n")?;
        result?;
        return Ok(());
    }
    let mut app = CoreApplication::new(data, 12077);
    let mut evidence = Vec::new();
    for mode in ["Standard", "Endless"] {
        reject(&mut app, json!({"type":"start_new_game","game_mode":mode,"replace_active_run":false}), &mut evidence)?;
    }
    execute(&mut app, json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))?;
    let selecting = finish_briefing(&mut app, &mut evidence)?;
    assert_eq!(selecting["game_state"], "selecting_starter_employees");
    assert_eq!(selecting["game_state_context"]["required_count"], 3);
    let candidates = selecting["game_state_context"]["candidates"].as_array().ok_or("candidate list")?;
    assert_eq!(candidates.len(), 6);
    for role in ["Dealer", "Tank", "Healer"] {
        assert_eq!(candidates.iter().filter(|candidate| candidate["role"] == role).count(), 2);
    }
    let ids: Vec<_> = candidates.iter().take(3).map(|candidate| candidate["employee_definition_id"].clone()).collect();
    reject(&mut app, json!({"type":"select_starter_employees","employee_definition_ids":ids}), &mut evidence)?;
    reject(&mut app, json!({"type":"select_starter_employees","employee_definition_ids":[ids[0],ids[0],ids[1]],"loan_copy_ids":[]}), &mut evidence)?;
    execute(&mut app, json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":[]}))?;
    let first = snapshot(&app, ApplicationSnapshotKind::Run)?;
    assert_eq!(first["roster"]["employees"].as_array().ok_or("roster")?.len(), 3);
    for employee in first["roster"]["employees"].as_array().unwrap() {
        assert!(employee.get("psychology").is_none());
        assert!(employee.get("role").is_some());
    }
    let instance = snapshot(&app, ApplicationSnapshotKind::PlayerProfile)?["lifecycle"]["active_expedition"].clone();
    assert_eq!(instance["sequence"], 1);
    reject(&mut app, json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":true}), &mut evidence)?;
    execute(&mut app, json!({"type":"abandon_run"}))?;
    assert!(app.export_active_run_checkpoint()?.is_none());
    let profile_before_tutorial = snapshot(&app, ApplicationSnapshotKind::PlayerProfile)?;
    for _ in 0..2 {
        execute(&mut app, json!({"type":"start_new_game","game_mode":"Tutorial","replace_active_run":false}))?;
        let tutorial = finish_briefing(&mut app, &mut evidence)?;
        assert_eq!(tutorial["game_state"], "viewing_map");
        assert_eq!(tutorial["roster"]["employees"].as_array().ok_or("tutorial roster")?.len(), 3);
        execute(&mut app, json!({"type":"abandon_run"}))?;
        assert_eq!(snapshot(&app, ApplicationSnapshotKind::PlayerProfile)?, profile_before_tutorial);
    }
    let result = json!({"passed":true,"scope":"public default-feature Core lifecycle protocol",
        "source":source,"first_run":first,"first_instance":instance,"commands":evidence,"final_roots":roots(&app)?});
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&serde_json::to_vec_pretty(&result)?)?;
    file.write_all(b"\n")?;
    Ok(())
}
