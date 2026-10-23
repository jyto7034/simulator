//! Setup observations only: one unchanged requested roster per bounded process.
//! No clock step is executed, no scale pass or approval is selected.
use game_core::{authoring::{CancellationToken,LabPlacement,TrialSession,WorldLabPreparation},
    game::data::content_pack::{ContentSelection,InputLimits},mod_validation::{Candidate,TrustedBase},
    runtime_safety::{RuntimeSafetySettings,EMBEDDED_RUNTIME_SAFETY_RON}};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fmt::Debug,fs,io::Read,path::Path,sync::Arc,time::Instant};
use u11_independent_public_probe::{file_state::FileState,normal_preparation as normal};
type Result<T>=std::result::Result<T,Box<dyn Error>>;
fn bounded(path:&Path,cap:usize)->Result<Vec<u8>> {
    let mut bytes=Vec::new();fs::File::open(path)?.take(cap as u64+1).read_to_end(&mut bytes)?;
    if bytes.len()>cap{return Err("diagnostic input exceeds finite bound".into());}Ok(bytes)
}
fn memory()->Result<Value> {
    let raw=String::from_utf8(bounded(Path::new("/proc/self/status"),65536)?)?;
    let field=|name:&str|->Result<u64>{
        let fields:Vec<_>=raw.lines().find(|line|line.starts_with(name)).ok_or("RSS field absent")?.split_whitespace().collect();
        if fields.len()!=3||fields[2]!="kB"{return Err("unexpected RSS unit".into());}
        Ok(fields[1].parse::<u64>()?.checked_mul(1024).ok_or("RSS overflow")?)
    };
    Ok(json!({"current_rss_bytes":field("VmRSS:")?,"process_lifetime_peak_rss_bytes":field("VmHWM:")?,
        "peak_scope":"Cumulative process high-water mark including earlier phases and diagnostic bookkeeping; not per-phase allocations."}))
}
fn measured<T,E:Debug>(out:&Path,label:&str,f:impl FnOnce()->std::result::Result<T,E>,
    success:impl FnOnce(&T)->Value,failure:impl FnOnce(&E)->Value)->Result<std::result::Result<T,E>> {
    let before=memory()?;let start=Instant::now();let result=f();let elapsed=start.elapsed().as_nanos();let after=memory()?;
    let observation=match &result{Ok(value)=>success(value),Err(error)=>failure(error)};
    normal::save(&out.join(format!("{label}.json")),&json!({"phase":label,"elapsed_ns":elapsed,
        "memory_before":before,"memory_after":after,"result":observation}))?;Ok(result)
}
fn error<E:Debug>(error:&E)->Value{json!({"status":"failed","error_debug":format!("{error:?}"),"usage_available":false,"usage":null})}
fn prepared(trial:&TrialSession)->Value {
    let ready=trial.ready_receipt();let raw=ready.document();
    let parsed=serde_json::from_str::<Value>(raw);
    let (usage,parse_error)=match parsed {
        Ok(document)=>(document.get("usage").cloned().unwrap_or(Value::Null),None),
        Err(error)=>(Value::Null,Some(error.to_string())),
    };
    let clone_peak=usage.get("clone_slots_peak").and_then(Value::as_u64);
    json!({"status":"prepared","lab_preparation":trial.lab_preparation(),"accepted_work":trial.accepted_work(),
        "accepted_work_scope":"Public TrialSession cumulative accepted work through preparation.",
        "copied_bytes":trial.copied_bytes(),"receipt_bytes":trial.receipt_bytes(),
        "actual_ready_receipt_document":raw,"ready_receipt_parse_error":parse_error,
        "usage_available":!usage.is_null(),"actual_ready_receipt_usage":usage,
        "clone_peak_available":clone_peak.is_some(),"clone_slots_peak":clone_peak,
        "clone_peak_scope":"Actual public preparation-ready receipt usage.clone_slots_peak; largest charged owned-copy shape during preparation, not cumulative copied slots.",
        "gameplay_executed":false})
}
fn prepare_failure(failure:&game_core::authoring::TrialFailure)->Value {
    json!({"status":"failed","failure":failure,"error_debug":format!("{failure:?}"),
        "attempt_usage_available":failure.attempt_usage().is_some(),"attempt_usage":failure.attempt_usage(),
        "clone_peak_available":false,"clone_peak":null,
        "clone_peak_unavailable_reason":"Failed preparation has no completed ready receipt; crossing diagnostics and any partial attempt_usage do not establish the complete required owned-copy shape."})
}
fn run(root:&Path,registry:&[u8],settings:Arc<RuntimeSafetySettings>,count:u32,deployed:usize,out:&Path)->Result<Value> {
    let base=measured(out,"01_trusted_base",||TrustedBase::acquire(root,
        ContentSelection{packs:vec!["official".into()],winners:vec![]},&InputLimits::default(),settings,&CancellationToken::default()),
        |base|json!({"status":"completed","identity":base.generation().identity(),"usage_available":false,
            "usage_unavailable_reason":"TrustedBase exposes no acquisition usage."}),error)?;
    let Ok(base)=base else{return Ok(json!({"stopped_at":"trusted_base"}));};
    let candidate=measured(out,"02_candidate",||Candidate::acquire(root,registry,&base,&InputLimits::default(),&CancellationToken::default()),
        |candidate|json!({"status":"completed","candidate":candidate.view(),"validation_usage":candidate.authoring_content().validation_usage()}),error)?;
    let Ok(candidate)=candidate else{return Ok(json!({"stopped_at":"candidate"}));};
    let selection=&candidate.required_configuration().selection;
    let mut packs:Vec<_>=selection.packs.iter().map(|p|p.to_string_lossy().into_owned()).collect();packs.sort();
    if packs!=["a_skills","b_skills","host_a_item","host_b_item","official"]||!selection.winners.is_empty(){
        return Err("diagnostic requires the existing host_ab selected pack closure".into());
    }
    let scenario=measured(out,"03_normal_prefix",||normal::prefix(&candidate,&out.join("normal_prefix")),
        |scenario|json!({"status":"completed","scenario":scenario,"usage_available":false}),error)?;
    let Ok(scenario)=scenario else{return Ok(json!({"stopped_at":"normal_prefix"}));};
    // Exactly the existing host_load recipe: records_clerk and two distinct
    // equipment instances per employee, normal_corroded_patrol/Combat.
    let mut recipe=normal::recipe("host_load",count)?;
    normal::save(&out.join("fixed_input.json"),&json!({"requested_employee_count":count,
        "requested_deployments":deployed,"recipe_without_deployments":recipe,"scenario":scenario,
        "scope":"Setup only, no gameplay clock execution; same existing normal prefix and host_load recipe."}))?;
    let checked=normal::checked(&scenario)?;let limits=normal::limits(&candidate)?;
    let discovery=measured(out,"04_discovery_prepare",||WorldLabPreparation::prepare(candidate.authoring_content(),
        checked,recipe.clone(),limits,&CancellationToken::default()),prepared,prepare_failure)?;
    let Ok(discovery)=discovery else{return Ok(json!({"stopped_at":"discovery_prepare","deployed_path":"not_executed_prerequisite_failed"}));};
    let receipt=discovery.lab_preparation().ok_or("actual preparation receipt missing")?;
    if receipt.employee_ids.len()!=count as usize||receipt.deployed_count!=0{return Err("discovery receipt differs from requested exact roster/zero deployment".into());}
    let view=measured(out,"05_discovery_reconnect",||normal::reconnect(&discovery),
        |view|json!({"status":"completed","actual_public_view":view,"usage_available":false}),error)?;
    drop(discovery); // No discovery TrialSession is retained across deployed preparation.
    let Ok(view)=view else{return Ok(json!({"stopped_at":"discovery_reconnect","deployed_path":"not_executed_prerequisite_failed"}));};
    let geometry=measured(out,"06_placement_geometry",||->Result<(bool,Value)>{
        let (cells,capacity)=normal::unoccupied_ground_cells(&view)?;
        if cells.len()<deployed{return Ok((false,json!({"status":"constraint","constraint":"insufficient_unoccupied_ground_cells",
            "requested_deployments":deployed,"actual_capacity":capacity,"counts_unchanged":true})));}
        let mut evidence=Vec::new();let mut intersections=0usize;
        for (employee_index,position) in cells.into_iter().take(deployed).enumerate() {
            let (facing,intersects,report)=normal::route_facing(&candidate,&recipe,&view,position)?;
            intersections+=usize::from(intersects);evidence.push(report);
            recipe.placements.push(LabPlacement{employee_index,position,facing});
        }
        Ok((intersections==deployed,json!({"status":if intersections==deployed{"ready"}else{"constraint"},
            "constraint":if intersections==deployed{Value::Null}else{json!("not_all_requested_positions_intersect_route")},
            "requested_deployments":deployed,"route_intersecting_deployments":intersections,"actual_capacity":capacity,
            "facing_geometry":evidence,"counts_unchanged":true})))
    },|(_,report)|report.clone(),error)?;
    normal::save(&out.join("actual_deployed_recipe.json"),&recipe)?;
    drop(view);
    let Ok((ready,constraint))=geometry else{return Ok(json!({"stopped_at":"placement_geometry","deployed_path":"not_executed_geometry_error"}));};
    if !ready{return Ok(json!({"stopped_at":"placement_constraint","constraint":constraint,"deployed_path":"not_executed_exact_requested_geometry_unavailable"}));}
    let checked=normal::checked(&scenario)?;let limits=normal::limits(&candidate)?;
    let actual=measured(out,"07_deployed_prepare",||WorldLabPreparation::prepare(candidate.authoring_content(),
        checked,recipe,limits,&CancellationToken::default()),prepared,prepare_failure)?;
    if let Ok(trial)=&actual {
        let receipt=trial.lab_preparation().ok_or("actual deployed receipt missing")?;
        if receipt.employee_ids.len()!=count as usize||receipt.deployed_count!=deployed{return Err("actual deployed receipt differs from unchanged requested counts".into());}
    }
    Ok(json!({"stopped_at":"deployed_prepare","deployed_path":if actual.is_ok(){"prepared"}else{"failed"},"gameplay_executed":false}))
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=6{return Err("usage: setup_cost ROOT REGISTRY_JSON ADOPTED_PARENT_RON ACTUAL_PROFILE_RON COUNT NEW_OUTPUT".into());}
    let root=Path::new(&args[0]);let out=Path::new(&args[5]);fs::create_dir(out)?;
    let count:u32=args[4].to_str().ok_or("UTF-8 count required")?.parse()?;
    let deployed=match count{16=>1,500=>8,1000=>16,_=>return Err("only unchanged declared counts 16/500/1000 are allowed".into())};
    let original=FileState::capture(root,65536,256*1024*1024)?;
    normal::save(&out.join("source_before.json"),&original.report())?;
    let registry=bounded(Path::new(&args[1]),1_048_576)?;
    let parent=bounded(Path::new(&args[2]),65_536)?;let actual=bounded(Path::new(&args[3]),65_536)?;
    let embedded=std::str::from_utf8(EMBEDDED_RUNTIME_SAFETY_RON)?;
    let expected_parent=embedded.replacen("authoring_execution: (\n        max_work: 10000000,",
        "authoring_execution: (\n        max_work: 100000000,",1);
    if expected_parent==embedded||parent!=expected_parent.as_bytes(){return Err("parent must be exact adopted CMVA100m profile bytes".into());}
    let parent_text=std::str::from_utf8(&parent)?;
    if parent_text.matches("max_clone_slots: 65536,").count()!=1{return Err("unique authoring clone field required".into());}
    let observation=parent_text.replacen("max_clone_slots: 65536,","max_clone_slots: 1000000,",1);
    let kind=if actual==parent{"adopted_parent_baseline"}else if actual==observation.as_bytes(){"observation_only_clone_1000000"}
        else{return Err("actual profile must equal parent or exact single-field clone observation".into());};
    let baseline=RuntimeSafetySettings::from_ron("adopted parent",&parent)?;
    let settings=Arc::new(RuntimeSafetySettings::from_ron("actual setup observation",&actual)?);
    let mut expected=serde_json::to_value(baseline.authoring_execution())?;
    if kind=="observation_only_clone_1000000"{expected["max_clone_slots"]=json!(1_000_000);}
    if serde_json::to_value(settings.authoring_execution())?!=expected{return Err("parsed authoring profile differs from exact allowed change".into());}
    for(label,path,bytes)in [("registry",&args[1],&registry),("parent_profile",&args[2],&parent),("actual_profile",&args[3],&actual)] {
        normal::save(&out.join(format!("{label}_input.json")),&json!({"path":Path::new(path),"raw_bytes":bytes,
            "sha256":format!("{:x}",Sha256::digest(bytes))}))?;
    }
    normal::save(&out.join("context.json"),&json!({"schema":"u11.setup-cost-diagnostic/v1","profile_kind":kind,
        "requested_employee_count":count,"requested_deployments":deployed,"all_other_profile_bytes_unchanged":true,
        "authoring_execution":settings.authoring_execution(),"raw_sha256":settings.raw_sha256(),"effective_sha256":settings.effective_sha256(),
        "observation_profile_adopted":false,"criterion_selected":false,"scale_pass_claimed":false,"gameplay_executed":false,
        "debug_assertions":cfg!(debug_assertions),"timing_scope":"Actual build setup observation, not release calibration.",
        "external_timeout_seconds":300,"external_kill_after_seconds":10}))?;
    let result=run(root,&registry,settings,count,deployed,out);
    normal::save(&out.join("actual_result.json"),&match &result{Ok(value)=>value.clone(),Err(error)=>error_value(error)})?;
    let after=FileState::capture(root,65536,256*1024*1024)?;
    normal::save(&out.join("source_after.json"),&after.report())?;original.ensure_unchanged(&after)?;
    normal::save(&out.join("RESULT.json"),&json!({"diagnostic_completed":result.is_ok(),"source_unchanged":true,
        "scale_pass_claimed":false,"observation_profile_adopted":false,"criterion_selected":false,"gameplay_executed":false}))?;
    result.map(|_|())
}
fn error_value(error:&Box<dyn Error>)->Value{json!({"status":"diagnostic_error","error":format!("{error:?}")})}
