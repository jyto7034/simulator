//! U13 diagnostic: retain transient selection/entry within one node visit.
//! Default-feature external approved-host consumer. Invoke each stable operation in a
//! fresh process over the same disposable persistence roots.
use game_server as host_server;
#[path = "../../../shared/normal_host_support.rs"]
mod support;
use support::*;
use game_server::mod_admission::ApprovedPersistence;
use game_core::{application::{ApplicationSnapshotKind,ApplicationExecutionContext,CoreApplication},mod_validation::RequiredConfiguration};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, sync::Arc};

struct ApprovedHost<'a> {
    app:CoreApplication, required:RequiredConfiguration, revision:Option<uuid::Uuid>,
    session:&'a game_server::mod_admission::AdmissionSession, persistence:&'a ApprovedPersistence,
}
impl u12_independent_lifecycle_probe::Host for ApprovedHost<'_> {
    fn app(&self)->&CoreApplication {&self.app}
    fn command(&mut self, document:Value,context:ApplicationExecutionContext)->Result<Value> {
        let result=execute_with_context(self.session,self.persistence,&mut self.app,&mut self.required,&mut self.revision,document,context)?;
        Ok(result["payload"].clone())
    }
    fn tick(&mut self,delta_ms:u64,after_seq:u64)->Result<Option<Value>> {
        let before=self.app.export_active_run_checkpoint()?;
        let prepared=self.app.prepare_active_battle_for_host_tick(delta_ms,after_seq)?;
        let (result,pending)=self.persistence.publish(prepared,&self.required,&mut self.revision,before.as_ref(),false)?;
        if pending {return Err("actual route host tick publication recovery_pending".into());}
        Ok(result.map(|r|r.payload().as_value().clone()))
    }
    fn restart(&mut self)->Result<()> {
        let (app,required,revision)=restore(self.session,self.persistence)?;
        self.app=app;self.required=required;self.revision=revision;Ok(())
    }
}

fn main() -> Result<()> {
    let admin_args:Vec<String>=std::env::args().skip(1).collect();
    if let Some(report)=game_server::mod_admission::administration(&admin_args)? {
        println!("{}",serde_json::to_string_pretty(&report)?);return Ok(());
    }
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 5 || args.len() > 6 { return Err("usage: normal_host_lifecycle <start|continue|command|reject|walk> <config> <active-root> <profile-root> <new-evidence-dir> [command-or-walk-options.json]".into()); }
    let operation = args[0].to_str().ok_or("operation UTF-8")?;
    let active = PathBuf::from(&args[2]);
    let profile = PathBuf::from(&args[3]);
    let out = PathBuf::from(&args[4]);
    fs::create_dir(&out)?;
    let session = open(&PathBuf::from(&args[1]))?;
    save(&out.join("admission.json"), &session.report)?;
    let persistence = ApprovedPersistence::open_roots(active.clone(), profile.clone(), Arc::clone(&session.host))?;
    let (mut app, mut required, mut revision) = restore(&session, &persistence)?;
    let before = roots(&app)?;
    let profile_before = snapshot(&app, ApplicationSnapshotKind::PlayerProfile)?;
    save(&out.join("before.json"), &before)?;
    if operation=="walk" {
        let options:Value=serde_json::from_slice(&fs::read(args.get(5).ok_or("walk options file required")?)?)?;
        let mode=options["game_mode"].as_str().ok_or("game_mode required")?;
        let boundary=options["boundary"].as_str().ok_or("boundary required")?;
        let mut host=ApprovedHost {app,required,revision,session:&session,persistence:&persistence};
        let result=u12_independent_lifecycle_probe::walk(&mut host,mode,boundary,&out.join("trace.jsonl"));
        save(&out.join("RESULT.json"),&json!({"passed":result.is_ok(),"operation":operation,"mode":mode,"boundary":boundary,
            "result":result.as_ref().ok(),"error":result.as_ref().err().map(ToString::to_string),"required":host.required,
            "revision":host.revision,"roots":roots(&host.app)?}))?;
        result?;return Ok(());
    }
    match operation {
        "start" => {
            execute(&session, &persistence, &mut app, &mut required, &mut revision,
                json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))?;
            finish_start(&session, &persistence, &mut app, &mut required, &mut revision, &out)?;
            assert_eq!(snapshot(&app, ApplicationSnapshotKind::PlayerProfile)?["lifecycle"]["expedition_sequence"].as_u64().unwrap(),
                profile_before["lifecycle"]["expedition_sequence"].as_u64().unwrap() + 1);
        }
        "continue" => { assert!(app.export_active_run_checkpoint()?.is_some()); }
        "facility-node" => {
            let options: Value = serde_json::from_slice(&fs::read(args.get(5).ok_or("node options required")?)?)?;
            let run=snapshot(&app,ApplicationSnapshotKind::Run)?;
            let slot=options["slot_id"].as_str().ok_or("slot_id required")?;
            let node=run["map"]["nodes"].as_array().ok_or("nodes required")?.iter().find(|n|n["slot_id"]==slot).ok_or("slot missing")?;
            assert_eq!(node["state"],"Available");
            let mut trace=Vec::new();
            for command in [json!({"type":"select_map_node","node_id":node["id"]}),json!({"type":"confirm_enter_node"})] {
                let result=execute(&session,&persistence,&mut app,&mut required,&mut revision,command.clone())?;
                trace.push(json!({"command":command,"result":result,"run_after":snapshot(&app,ApplicationSnapshotKind::Run)?}));
            }
            if !snapshot(&app,ApplicationSnapshotKind::Run)?["run_progression"]["pending_continuation"].is_null() {
                let command=json!({"type":"choose_expedition_continuation","choice":"Continue"});
                let result=execute(&session,&persistence,&mut app,&mut required,&mut revision,command.clone())?;
                trace.push(json!({"command":command,"result":result,"run_after":snapshot(&app,ApplicationSnapshotKind::Run)?}));
            }
            if let Some(finish)=options["finish"].as_str() {
                assert!(matches!(finish,"complete_node"|"exit_shop"));
                let command=json!({"type":finish});
                let result=execute(&session,&persistence,&mut app,&mut required,&mut revision,command.clone())?;
                trace.push(json!({"command":command,"result":result,"run_after":snapshot(&app,ApplicationSnapshotKind::Run)?}));
            }
            save(&out.join("node_trace.json"),&trace)?;
        }
        "command" | "reject" => {
            let document: Value = serde_json::from_slice(&fs::read(args.get(5).ok_or("command file required")?)?)?;
            let files_before = (file_bytes(&active)?, file_bytes(&profile)?);
            let result = execute(&session, &persistence, &mut app, &mut required, &mut revision, document.clone());
            if operation == "reject" {
                assert!(result.is_err());
                assert_eq!(roots(&app)?, before);
                assert_eq!((file_bytes(&active)?, file_bytes(&profile)?), files_before);
                save(&out.join("rejected.json"), &json!({"command":document,"error":result.unwrap_err().to_string(),"unchanged":true}))?;
            } else { save(&out.join("accepted.json"), &json!({"command":document,"result":result?}))?; }
        }
        _ => return Err("unknown operation".into()),
    }
    save(&out.join("RESULT.json"), &json!({"passed":true,"operation":operation,"required":required,"revision":revision,"roots":roots(&app)?}))?;
    Ok(())
}
