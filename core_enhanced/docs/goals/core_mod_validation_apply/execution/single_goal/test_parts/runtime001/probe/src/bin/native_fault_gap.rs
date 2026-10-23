//! Bounded public-API diagnostic only. No issuer, token, approval or cap adoption.
//! Retains every attempted exact parent-derived profile and actual typed failure.
use game_core::{application::{ApplicationCommand,ApplicationExecutionContext,
    ApplicationSnapshotKind,ApplicationBattleAuthority,CoreApplication},authoring::{CancellationToken,
    ScenarioLimits,TrialScenario,WorldLabPreparation},game::{behavior::{GameStateContextDto,RunSnapshotDto},
    data::{ValidatedGameData,content_pack::InputLimits},map::{MapNodeCategory,MapNodeState}},
    mod_validation::{Candidate,LabSuite,TrustedBase},runtime_safety::RuntimeSafetySettings};
use game_server::mod_admission::AdmissionConfig;
use serde::Serialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fmt::Debug,fs,io::Read,path::{Path,PathBuf},sync::Arc};
use u11_independent_public_probe::{file_state::FileState,normal_preparation::{limits,save}};
use uuid::Uuid;
type Result<T>=std::result::Result<T,Box<dyn Error>>;
type Checked<T>=std::result::Result<T,Failure>;
const PARENT:u64=10_000_000;
const FIRST_FAILED:u64=1000;
const MAX_REFINEMENTS:usize=24;
const TICKS:usize=40; // Original actor window: 200 polls *20ms /100ms per host tick.
const WINDOW60_TICKS:usize=600;
const OBSERVED_PREREQUISITE_CAP:u64=17_761;

#[derive(Debug,Serialize)]
struct Failure {stage:String,typed:Value,debug:String,native_work_limit:bool}
fn native_code(value:&Value)->bool {
    if value.get("code")==Some(&json!("battle_runtime_work_limit")){return true;}
    match value {Value::Object(fields)=>fields.values().any(native_code),
        Value::Array(items)=>items.iter().any(native_code),_=>false}
}
fn failure(stage:&str,error:&(impl Serialize+Debug))->Failure {
    let typed=serde_json::to_value(error).expect("serializable public failure");
    let debug=format!("{error:?}");
    let native_work_limit=native_code(&typed)||debug.contains("BattleRuntimeFault { code: WorkLimit,");
    Failure{stage:stage.into(),typed,debug,native_work_limit}
}
fn diagnostic(stage:&str,message:impl ToString)->Failure {
    Failure{stage:stage.into(),typed:json!({"diagnostic_error":message.to_string()}),
        debug:message.to_string(),native_work_limit:false}
}
fn bounded(path:&Path,cap:usize)->Result<Vec<u8>> {
    let mut bytes=Vec::new();fs::File::open(path)?.take(cap as u64+1).read_to_end(&mut bytes)?;
    if bytes.len()>cap{return Err("diagnostic input exceeds finite byte cap".into());}Ok(bytes)
}
fn keep(path:&Path,value:&impl Serialize)->Checked<()> {save(path,value).map_err(|e|diagnostic("evidence",e))}
fn snapshot(app:&CoreApplication)->Checked<RunSnapshotDto> {
    let value=app.snapshot(ApplicationSnapshotKind::Run).map_err(|e|failure("normal.snapshot",&e))?;
    serde_json::from_value(value.read_model().as_value().clone()).map_err(|e|diagnostic("normal.snapshot_decode",e))
}
fn cursor(value:&Value,after:&mut u64){
    if let Some(seq)=value.pointer("/events_delta/to_seq").and_then(Value::as_u64){*after=(*after).max(seq);}
    match value {Value::Object(fields)=>for child in fields.values(){cursor(child,after)},
        Value::Array(items)=>for child in items{cursor(child,after)},_=>{}}
}
fn command(app:&mut CoreApplication,data:&ValidatedGameData,player:Uuid,value:Value,
    after:&mut u64,sequence:&mut usize,out:&Path)->Checked<Value> {
    app.export_active_run_checkpoint().map_err(|e|failure("normal.before_command_checkpoint",&e))?;
    let input=ApplicationCommand::from_document(value.clone().into()).map_err(|e|failure("normal.command_decode",&e))?;
    let generation=(value["type"]=="start_new_game").then(||data.clone());
    let context=ApplicationExecutionContext{source_command_id:Some(format!("diagnostic-{sequence}")),
        battle_event_after_seq:*after,expected_battle_authority:None};
    let result=app.prepare_command_with_generation(player,input,context,generation)
        .map(|prepared|prepared.commit());
    keep(&out.join(format!("command_{sequence:03}.json")),&json!({"input":value,"result":result}))?;
    *sequence+=1;
    let result=result.map_err(|e|failure("normal.command_prepare_commit",&e))?;
    cursor(result.payload().as_value(),after);Ok(result.payload().as_value().clone())
}
fn deploy_two(app:&mut CoreApplication,player:Uuid,mut payload:Value,after:&mut u64,out:&Path)->Checked<()> {
    let setup=payload["battle_setup_snapshot"].clone();
    let zones=setup["deployment_zones"].as_array().ok_or_else(||diagnostic("normal.deploy.setup","deployment zones missing"))?;
    let roster=app.snapshot(ApplicationSnapshotKind::EmployeeRoster).map_err(|e|failure("normal.deploy.roster",&e))?;
    let roster=roster.read_model().as_value();
    keep(&out.join("DEPLOY_ROSTER.json"),roster)?;
    let eligible=roster["combat_eligible_employee_ids"].as_array().ok_or_else(||diagnostic("normal.deploy.roster","eligible ids missing"))?;
    let employees=roster["employees"].as_array().ok_or_else(||diagnostic("normal.deploy.roster","employees missing"))?;
    let mut deployed=0;
    for employee in eligible.iter().take(3) {
        let update=&payload["battle_update"];let cp=&update["checkpoint"];
        let costs=cp["deployment"]["unit_deploy_costs"].as_array().ok_or_else(||diagnostic("normal.deploy.cost","costs missing"))?;
        let Some(cost)=costs.iter().find(|c|c["employee_uuid"]==*employee&&c["eligibility"]["enabled"]==true) else{continue};
        let cost=cost["effective_deploy_cost"].as_u64().ok_or_else(||diagnostic("normal.deploy.cost","numeric cost required"))?;
        let available=cp["tactical_resource"]["current"].as_u64().ok_or_else(||diagnostic("normal.deploy.cost","resource required"))?;
        if cost>available{continue;}
        let details=employees.iter().find(|e|e["uuid"]==*employee).ok_or_else(||diagnostic("normal.deploy.employee","employee missing"))?;
        let affinity=details["combat_profile"]["effective_deployment_affinity"].as_str()
            .ok_or_else(||diagnostic("normal.deploy.affinity","effective affinity required"))?;
        let units=cp["units"].as_array().ok_or_else(||diagnostic("normal.deploy.units","units missing"))?;
        let position=zones.iter().filter(|z|matches!((affinity,z["kind"].as_str()),
            ("any",Some("Ground"|"Platform"))|("ground_only",Some("Ground"))|("platform_only",Some("Platform"))))
            .filter_map(|z|z["cells"].as_array()).flatten()
            .find(|cell|!units.iter().any(|u|u["position"]==**cell))
            .cloned().ok_or_else(||diagnostic("normal.deploy.tile","unoccupied compatible returned tile required"))?;
        let authority:ApplicationBattleAuthority=serde_json::from_value(json!({"battle_uuid":update["battle_uuid"],
            "battle_state_revision":update["battle_state_revision"]})).map_err(|e|diagnostic("normal.deploy.authority",e))?;
        let value=json!({"type":"deploy_unit","employee_uuid":employee,"position":position,"facing":"up"});
        let command=ApplicationCommand::from_document(value.clone().into()).map_err(|e|failure("normal.deploy.decode",&e))?;
        let context=ApplicationExecutionContext{source_command_id:Some(format!("diagnostic-deploy-{deployed}")),
            battle_event_after_seq:*after,expected_battle_authority:Some(authority)};
        let result=app.prepare_command_with_generation(player,command,context,None).map(|prepared|prepared.commit());
        keep(&out.join(format!("deploy_{deployed}.json")),&json!({"input":value,"result":result,
            "returned_cost":cost,"resource_before":available,"affinity":affinity,"authority":authority,
            "fault_after":app.battle_fault(),"classification":"public deployment command; never a host-tick fault"}))?;
        let result=result.map_err(|e|failure("normal.deploy.prepare_commit",&e))?;
        payload=result.payload().as_value().clone();cursor(&payload,after);deployed+=1;
        if deployed==2{break;}
    }
    let cp=&payload["battle_update"]["checkpoint"];
    let count=cp["deployment"]["deployed_units"].as_array().map(Vec::len).unwrap_or(0);
    let combatants=cp["units"].as_array().map(|units|units.iter().filter(|u|u["owner"]=="Player"&&u["role"]=="combatant").count()).unwrap_or(0);
    keep(&out.join("DEPLOYED_READY.json"),&json!({"successful_deploy_commands":deployed,"actual_deployed_units":count,
        "actual_player_combatants":combatants,"final_payload":payload,"first_fault":app.battle_fault()}))?;
    if deployed!=2||count!=2||combatants!=2||app.battle_fault().is_some(){
        return Err(diagnostic("normal.deploy.ready","exactly two successful live player deployments required before ticking"));
    }
    Ok(())
}
fn normal_ready(data:ValidatedGameData,side:&str,out:&Path,deployed_window:bool)->Checked<(CoreApplication,u64)> {
    let player=Uuid::from_u128(if side=="a"{0x5203}else{0x5205});
    let mut app=CoreApplication::new(data.clone(),23);let mut after=0;let mut seq=0;
    command(&mut app,&data,player,json!({"type":"start_new_game","game_mode":"Standard",
        "replace_active_run":false}),&mut after,&mut seq,out)?;
    let mut ready=false;
    for _ in 0..128 {
        let run=snapshot(&app)?;
        let value=if let Some(vn)=run.visual_novel {
            if let Some(choice)=vn.choices.iter().find(|c|c.selectable){json!({"type":"select_visual_novel_choice",
                "session_id":vn.session_id,"current_scene_id":vn.current_scene_id,"choice_id":choice.choice_id})}
            else{json!({"type":"advance_visual_novel","session_id":vn.session_id,"current_scene_id":vn.current_scene_id})}
        }else if let GameStateContextDto::SelectingStarterEmployees{required_count,candidates}=run.game_state_context {
            let ids:Vec<_>=candidates.into_iter().take(required_count).map(|c|c.employee_definition_id).collect();
            if ids.len()!=required_count{return Err(diagnostic("normal.starters","missing exact starter candidates"));}
            json!({"type":"select_starter_employees","employee_definition_ids":ids})
        }else{
            if run.roster.combat_eligible_employee_ids.len()<2{return Err(diagnostic("normal.starters","missing live starter roster"));}
            ready=true;break;
        };
        command(&mut app,&data,player,value,&mut after,&mut seq,out)?;
    }
    if !ready{return Err(diagnostic("normal.starters","finite starter command bound exceeded"));}
    let mut selected=false;
    for _ in 0..128 {
        let map=snapshot(&app)?.map.ok_or_else(||diagnostic("normal.map","map required"))?;
        if let Some(node)=map.nodes.iter().find(|n|n.state==MapNodeState::Available&&n.category==MapNodeCategory::Combat){
            command(&mut app,&data,player,json!({"type":"select_map_node","node_id":node.id}),&mut after,&mut seq,out)?;
            selected=true;break;
        }
        let node=map.nodes.iter().find(|n|n.state==MapNodeState::Available&&n.category==MapNodeCategory::Start)
            .ok_or_else(||diagnostic("normal.map","seed23 has no adjacent Start/Combat node"))?;
        for value in [json!({"type":"select_map_node","node_id":node.id}),
            json!({"type":"confirm_enter_node"}),json!({"type":"complete_node"})] {
            command(&mut app,&data,player,value,&mut after,&mut seq,out)?;
        }
    }
    if !selected{return Err(diagnostic("normal.map","finite normal combat selection bound exceeded"));}
    let entered=command(&mut app,&data,player,json!({"type":"confirm_enter_node"}),&mut after,&mut seq,out)?;
    if deployed_window{deploy_two(&mut app,player,entered,&mut after,out)?;}
    if !app.has_active_battle()||app.battle_fault().is_some(){return Err(diagnostic("normal.ready","unfaulted active battle required"));}
    app.export_active_run_checkpoint().map_err(|e|failure("normal.ready_checkpoint",&e))?;
    keep(&out.join("READY.json"),&json!({"seed":23,"player":player,"actual_generation":app.content_identity(),
        "runtime_sha256":app.runtime_safety().effective_sha256(),"after_seq":after,"approval_claimed":false}))?;
    Ok((app,after))
}
fn prerequisites(cfg:&AdmissionConfig,suite:&LabSuite,settings:Arc<RuntimeSafetySettings>,side:&str,out:&Path,deployed_window:bool)
    ->Checked<(CoreApplication,u64)> {
    let selection=serde_json::from_slice(&bounded(&cfg.base_selection_path,65536)
        .map_err(|e|diagnostic("base_selection_read",e))?).map_err(|e|diagnostic("base_selection_decode",e))?;
    let base=TrustedBase::acquire(&cfg.content_root,selection,&InputLimits::default(),settings,&CancellationToken::default())
        .map_err(|e|failure("trusted_base",&e))?;
    keep(&out.join("TRUSTED_BASE.json"),&json!({"completed":true,"identity":base.generation().identity()}))?;
    let raw=bounded(&cfg.registry_path,1_048_576).map_err(|e|diagnostic("registry_read",e))?;
    let candidate=Candidate::acquire(&cfg.content_root,&raw,&base,&InputLimits::default(),&CancellationToken::default())
        .map_err(|e|failure("candidate",&e))?;
    keep(&out.join("CANDIDATE.json"),candidate.view())?;
    let case=&suite.cases[0];
    let scenario=TrialScenario::from_json_slice(&case.scenario,&ScenarioLimits::default()).map_err(|e|failure("world.scenario",&e))?;
    let recipe=case.preparation.clone().ok_or_else(||diagnostic("world.recipe","real World preparation required"))?;
    let trial=WorldLabPreparation::prepare(candidate.authoring_content(),scenario,recipe,
        limits(&candidate).map_err(|e|diagnostic("world.limits",e))?,&CancellationToken::default())
        .map_err(|e|failure("world.prepare",&e))?;
    keep(&out.join("WORLD_PREPARED.json"),&json!({"completed":true,"receipt":trial.lab_preparation(),
        "ready_document":trial.ready_receipt().document(),"accepted_work":trial.accepted_work(),
        "scope":"completed public preparation; cumulative authoring work is not a native-operation maximum"}))?;
    drop(trial);
    normal_ready(candidate.generation().clone(),side,out,deployed_window)
}
fn ticks(mut app:CoreApplication,mut after:u64,out:&Path,max_ticks:usize)->Checked<Value> {
    for index in 0..max_ticks {
        if !app.has_active_battle(){return Ok(json!({"fault":false,"completed_ticks":index,
            "completed_window_ms":index*100,"battle_ended":true}));}
        app.export_active_run_checkpoint().map_err(|e|failure("ticks.before_checkpoint",&e))?;
        let result=app.prepare_active_battle_for_host_tick(100,after).map(|prepared|prepared.commit());
        keep(&out.join(format!("tick_{index:03}.json")),&json!({"delta_ms":100,"after_seq":after,
            "tick_index":index,"attempted_tick_end_ms":(index+1)*100,
            "result":result,"first_fault":app.battle_fault()}))?;
        match result {
            Ok(Some(result))=>cursor(result.payload().as_value(),&mut after),
            Ok(None)=>{},
            Err(error)=>{
                let native=error.code()=="battle_runtime_work_limit"&&app.battle_fault().is_some();
                if !native{return Err(failure("ticks.unexpected_failure",&error));}
                return Ok(json!({"fault":true,"completed_ticks":index,"completed_window_ms":index*100,
                    "attempted_tick_end_ms":(index+1)*100,"error":error,"first_fault":app.battle_fault()}));
            }
        }
    }
    Ok(json!({"fault":false,"completed_ticks":max_ticks,"completed_window_ms":max_ticks*100,
        "battle_ended":!app.has_active_battle()}))
}
fn attempt(index:usize,cap:u64,parent:&str,configs:&[AdmissionConfig;2],suite:&LabSuite,out:&Path,max_ticks:usize,deployed_window:bool)->Result<(bool,bool)> {
    let directory=out.join(format!("attempt_{index:02}_native_{cap}"));fs::create_dir(&directory)?;
    let runtime=parent.replacen("native_runtime: (\n        max_work: 10000000,",
        &format!("native_runtime: (\n        max_work: {cap},"),1);
    fs::write(directory.join("runtime_safety.ron"),runtime.as_bytes())?;
    let settings=Arc::new(RuntimeSafetySettings::from_ron("actual native gap diagnostic",runtime.as_bytes())?);
    save(&directory.join("PROFILE.json"),&json!({"native_cap":cap,"raw_sha256":format!("{:x}",Sha256::digest(runtime.as_bytes())),
        "effective_sha256":settings.effective_sha256(),"same_profile_for_all_prerequisites_and_ticks":true}))?;
    let mut apps=Vec::new();let mut failures=Vec::new();
    for (side,cfg) in ["a","b"].into_iter().zip(configs) {
        let side_out=directory.join(side);fs::create_dir(&side_out)?;
        match prerequisites(cfg,suite,Arc::clone(&settings),side,&side_out,deployed_window) {
            Ok(app)=>apps.push((side,side_out,app)),
            Err(error)=>{save(&side_out.join("PREREQUISITE_FAILED.json"),&error)?;failures.push(error);}
        }
    }
    if !failures.is_empty(){
        save(&directory.join("RESULT.json"),&json!({"prerequisites_completed":false,"failures":failures,"approval_claimed":false}))?;
        if failures.iter().any(|f|!f.native_work_limit){return Err("non-native-work prerequisite failure; no cap inference permitted".into());}
        return Ok((false,false));
    }
    let mut actual=serde_json::Map::new();
    for (side,side_out,(app,after)) in apps {
        let result=ticks(app,after,&side_out,max_ticks);
        keep(&side_out.join("TICKS.json"),&result).map_err(|e|format!("{e:?}"))?;
        actual.insert(side.into(),result.map_err(|e|format!("{e:?}"))?);
    }
    let gap=actual["a"]["fault"]==true;
    save(&directory.join("RESULT.json"),&json!({"prerequisites_completed":true,"ticks":actual,
        "diagnostic_gap_observed":gap,"approval_claimed":false,"cap_adopted":false}))?;
    Ok((true,gap))
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    let deployed_window=args.len()==6&&args[5]=="--deployed-window60";
    let window60=deployed_window||(args.len()==6&&args[5]=="--window60");
    if args.len()!=5&&!window60{return Err("usage: native_fault_gap A_CONFIG B_CONFIG FINITE_WORLD_SUITE EXACT_PARENT_RON NEW_PRIVATE_OUTPUT [--window60|--deployed-window60]".into());}
    let max_ticks=if window60{WINDOW60_TICKS}else{TICKS};
    let out=PathBuf::from(&args[4]);
    #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;fs::DirBuilder::new().mode(0o700).create(&out)?;}
    #[cfg(not(unix))] {return Err("actual private Unix output required".into());}
    let configs=[AdmissionConfig::read(Path::new(&args[0]))?,AdmissionConfig::read(Path::new(&args[1]))?];
    if configs[0].content_root!=configs[1].content_root||configs[0].base_selection_path!=configs[1].base_selection_path {
        return Err("A/B must retain their exact common trusted base".into());
    }
    let original=FileState::capture(&configs[0].content_root,65536,256*1024*1024)?;
    let parent_bytes=bounded(Path::new(&args[3]),65536)?;let parent=std::str::from_utf8(&parent_bytes)?;
    if parent.matches("native_runtime: (\n        max_work: 10000000,").count()!=1{return Err("exact valid native10m parent required".into());}
    let original_low=parent.replacen("native_runtime: (\n        max_work: 10000000,",
        "native_runtime: (\n        max_work: 1000,",1);
    for cfg in &configs {
        if bounded(&cfg.runtime_safety_path,65536)?!=original_low.as_bytes(){
            return Err("original1000 context must derive from the exact supplied parent, including OBS clone settings".into());
        }
    }
    let mut a=serde_json::to_value(&configs[0])?;let mut b=serde_json::to_value(&configs[1])?;
    a.as_object_mut().unwrap().remove("registry_path");b.as_object_mut().unwrap().remove("registry_path");
    if a!=b{return Err("A/B must differ only in their actual registry selection".into());}
    let suite_raw=bounded(Path::new(&args[2]),16*1024*1024)?;let suite:LabSuite=serde_json::from_slice(&suite_raw)?;
    if suite.cases.len()!=1{return Err("original single-case finite World suite required".into());}
    let case=&suite.cases[0];let scenario:Value=serde_json::from_slice(&case.scenario)?;
    let recipe=case.preparation.as_ref().ok_or("World recipe required")?;
    if scenario["steps"]!=json!([])||recipe.employees.len()!=1||recipe.employees[0].count!=1||recipe.placements.len()!=1 {
        return Err("unchanged one-employee/one-deployment preparation with no execution required".into());
    }
    let mut preserved=Vec::new();
    for (index,path) in args[..4].iter().enumerate() {
        let raw=bounded(Path::new(path),16*1024*1024)?;
        fs::write(out.join(format!("original_input_{index}.raw")),&raw)?;
        preserved.push((PathBuf::from(path),raw));
    }
    for (index,path) in [configs[0].base_selection_path.clone(),configs[0].registry_path.clone(),configs[1].registry_path.clone()].into_iter().enumerate(){
        let raw=bounded(&path,1_048_576)?;fs::write(out.join(format!("original_selection_{index}.raw")),&raw)?;preserved.push((path,raw));
    }
    let mut timing_provenance=Vec::new();
    if window60 {
        for relative in ["official/data/pve/encounters.ron","official/data/enemies/corroded_wave_presets.ron"] {
            let path=configs[0].content_root.join(relative);let raw=bounded(&path,16*1024*1024)?;
            timing_provenance.push(json!({"path":path,"sha256":format!("{:x}",Sha256::digest(&raw))}));
        }
    }
    save(&out.join("INPUTS.json"),&json!({"a":configs[0],"b":configs[1],"suite":suite,
        "parent_raw_bytes":parent_bytes,"parent_sha256":format!("{:x}",Sha256::digest(&parent_bytes)),
        "original_source":original.report(),"scope":"public prerequisite/cap diagnostic, no issuer or approval",
        "mode":if deployed_window{"deployed_window60_parent_and_observed_prerequisite_cap"}else if window60{"window60_parent_and_observed_prerequisite_cap"}else{"original40_refinement"},
        "normal_seed":23,"host_tick_ms":100,"max_ticks":max_ticks,
        "max_refinements":if window60{0}else{MAX_REFINEMENTS},"timing_source_files":timing_provenance,
        "reviewed_timing":if window60{json!({"encounter":"normal_corroded_patrol",
            "wave":"archive_sentries","activation_at_time_ms":5000,"preset":"archive_sentry_pair",
            "generated_enemy_count":2,"original_window_ms":4000,"scope":"reviewed fixture timing; actual ticks retain runtime behavior"})}else{Value::Null}}))?;
    let outcome=(||->Result<Value>{
        let (parent_ready,parent_gap)=attempt(0,PARENT,parent,&configs,&suite,&out,max_ticks,deployed_window)?;
        if !parent_ready{return Err("valid parent failed a prerequisite; no lower-cap inference permitted".into());}
        if parent_gap{return Ok(json!({"status":"diagnostic_gap_observed","observed_cap":PARENT}));}
        if window60 {
            let (ready,gap)=attempt(1,OBSERVED_PREREQUISITE_CAP,parent,&configs,&suite,&out,max_ticks,deployed_window)?;
            return Ok(json!({"status":if gap{"diagnostic_gap_observed"}else if ready{
                if deployed_window{"no_gap_observed_in_bounded_deployed_window60"}else{"no_gap_observed_in_bounded_window60"}}else{"observed_prerequisite_cap_did_not_reproduce"},
                "observed_cap":OBSERVED_PREREQUISITE_CAP,"same_profile_prerequisites_completed":ready,
                "max_ticks_per_side":max_ticks,"universal_no_gap_claimed":false}));
        }
        let (low_ready,low_gap)=attempt(1,FIRST_FAILED,parent,&configs,&suite,&out,max_ticks,deployed_window)?;
        if low_gap{return Ok(json!({"status":"diagnostic_gap_observed","observed_cap":FIRST_FAILED}));}
        if low_ready{return Err("original1000 prerequisite failure did not reproduce; stop rather than assume bracket".into());}
        let(mut low,mut high)=(FIRST_FAILED,PARENT);
        for step in 0..MAX_REFINEMENTS {
            if high-low<=1{break;}
            let cap=low+(high-low)/2;
            let (ready,gap)=attempt(step+2,cap,parent,&configs,&suite,&out,max_ticks,deployed_window)?;
            if gap{return Ok(json!({"status":"diagnostic_gap_observed","observed_cap":cap,
                "previous_failed_prerequisite_cap":low,"previous_complete_prerequisite_cap":high}));}
            if ready{high=cap;}else{low=cap;}
        }
        Ok(json!({"status":"no_gap_observed_in_bounded_refinement","failed_prerequisite_cap":low,
            "complete_prerequisite_cap":high,"universal_no_gap_claimed":false}))
    })();
    original.ensure_unchanged(&FileState::capture(&configs[0].content_root,65536,256*1024*1024)?)?;
    for (path,raw) in &preserved {if fs::read(path)?!=*raw{return Err("original diagnostic input changed".into());}}
    save(&out.join("RESULT.json"),&json!({"diagnostic":outcome.as_ref().map_err(|e|e.to_string()),
        "mode":if deployed_window{"deployed_window60_parent_and_observed_prerequisite_cap"}else if window60{"window60_parent_and_observed_prerequisite_cap"}else{"original40_refinement"},
        "max_ticks_per_side":max_ticks,"host_tick_ms":100,
        "approval_claimed":false,"cap_adopted":false,"profile_only_change":"native_runtime.max_work",
        "original_unchanged":true,"native_operation_maximum_claimed":false}))?;
    outcome?;Ok(())
}
