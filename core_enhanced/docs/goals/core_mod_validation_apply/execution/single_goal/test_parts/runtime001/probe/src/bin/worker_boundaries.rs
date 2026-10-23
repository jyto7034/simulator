//! Real paired calibration: private-root boundary and typed gameplay-fault parity.
//! Requires the expression fixture and its actual safe Staff cast suite.
//! This diagnostic cannot issue approval or choose a criterion.
use game_core::{authoring::{CancellationToken,AuthoringError},game::data::content_pack::{ContentSelection,InputLimits},
    mod_validation::{Candidate,LabProfile,LabRun,LabSuite,ModValidationErrorKind,TrustedBase,WorkerMode,WorkerPair,WorkerRequest,WorkerResponse},
    runtime_safety::{RuntimeSafetySettings}};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs,path::Path,sync::Arc};
use u11_independent_public_probe::{file_state::FileState,normal_preparation::{save,runtime_profile,prepare,reconnect},worker_evidence};
type Result<T>=std::result::Result<T,Box<dyn Error>>;
fn private(path:&Path)->Result<()> {
    #[cfg(unix)] {
        use std::os::unix::fs::{DirBuilderExt,PermissionsExt};
        fs::DirBuilder::new().mode(0o700).create(path)?;
        let meta=fs::symlink_metadata(path)?;
        assert!(meta.is_dir()&&!meta.file_type().is_symlink());assert_eq!(meta.permissions().mode()&0o777,0o700);
        Ok(())
    }
    #[cfg(not(unix))] {let _=path;Err("Unix actual private directory required".into())}
}
fn state(path:&Path)->Result<FileState>{Ok(FileState::capture(path,65_536,256*1024*1024)?)}
fn unit_hp(view:&Value,id:&Value)->Result<u64>{
    let units=view.pointer("/payload/battle_update/checkpoint/units").and_then(Value::as_array)
        .ok_or("actual checkpoint units required")?;
    let unit=units.iter().find(|unit|unit["unit_instance_id"]==*id).ok_or("same selected unit must remain live")?;
    unit["stats"]["current_health"].as_u64().ok_or("actual selected unit HP required".into())
}
fn prepared_for_cast(candidate:&Candidate,suite:&LabSuite,out:&Path,label:&str)->Result<game_core::authoring::TrialSession>{
    let case=&suite.cases[0];let document:Value=serde_json::from_slice(&case.scenario)?;
    let mut trial=prepare(candidate,&document,case.preparation.as_ref().unwrap())?;
    assert_eq!(trial.next_step_index(),6);
    save(&out.join(format!("{label}_prepared.json")),&reconnect(&trial)?)?;
    while trial.next_step_index()<12 {
        let index=trial.next_step_index();
        let result=trial.advance(&CancellationToken::default())
            .map(|step|step.public_result().cloned()).map_err(|e|format!("{e:?}"));
        save(&out.join(format!("{label}_prerequisite_{index}.json")),&json!({"step":index,
            "public_result":result.as_ref().ok().cloned().flatten(),
            "error":result.as_ref().err(),"reconnect":reconnect(&trial)?}))?;
        result.map_err(|e|format!("{label} actual prerequisite{index}: {e}"))?;
    }
    assert_eq!(trial.next_step_index(),12);
    Ok(trial)
}
fn cast_participants(view:&Value)->Result<(Value,Value)>{
    let checkpoint=&view["payload"]["battle_update"]["checkpoint"];
    let deployed=checkpoint["deployment"]["deployed_units"].as_array().ok_or("actual deployed caster required")?;
    assert_eq!(deployed.len(),1);let caster=deployed[0]["unit_instance_id"].clone();
    assert_eq!(deployed[0]["skill_readiness"]["resonance_current"],1);
    assert_eq!(deployed[0]["skill_readiness"]["selection"],"player_unit");
    let target=&checkpoint["units"][2];assert_eq!(target["owner"],"Opponent");
    assert!(target["stats"]["current_health"].as_u64().ok_or("live target HP required")?>10);
    let target=target["unit_instance_id"].clone();assert_ne!(caster,target);
    assert_eq!(checkpoint["tactical_focus"]["caster_unit_instance_id"],caster);
    assert_eq!(checkpoint["tactical_focus"]["skill_id"],"training_berserker");
    Ok((caster,target))
}
fn verify_safe_amount(candidate:&Candidate,suite:&LabSuite,out:&Path)->Result<()> {
    let mut trial=prepared_for_cast(candidate,suite,out,"safe")?;
    let before=reconnect(&trial)?;let(caster,target)=cast_participants(&before)?;
    let before_seq=before["payload"]["battle_update"]["checkpoint"]["at_seq"].as_u64().ok_or("actual before sequence required")?;
    trial.run_to_end(&CancellationToken::default()).map_err(|e|format!("safe hostile cast: {e:?}"))?;
    let after=reconnect(&trial)?;
    let hp_before=unit_hp(&before,&target)?;let hp_after=unit_hp(&after,&target)?;
    let events=after["payload"]["battle_update"]["events_delta"]["events"].as_array().ok_or("actual event delta required")?;
    let damage:Vec<_>=events.iter().filter(|e|e["seq"].as_u64().is_some_and(|seq|seq>before_seq)
        &&e["event"]["type"]=="HpChanged"&&e["event"]["target_instance_id"]==target).collect();
    save(&out.join("safe_actual_amount_oracle.json"),&json!({"before":before,"after":after,"caster":caster,"target":target,
        "hp_before":hp_before,"hp_after":hp_after,"damage_events":damage,"independent_expected_damage":10,
        "basis":"1*(100/10) from existing Staff +10 SpellPower; True damage to the actual selected Opponent"}))?;
    assert_eq!(hp_before.checked_sub(hp_after),Some(10),"safe Lab must consume the expression as ten HP damage to the same selected opponent");
    assert_eq!(damage.len(),1,"one actual target HP mutation after the cast baseline");
    assert_eq!(damage[0]["event"]["source_instance_id"],caster);
    assert_eq!(damage[0]["event"]["damage_source"],"Ability");
    assert_eq!(damage[0]["event"]["raw_damage"],10);assert_eq!(damage[0]["event"]["final_damage"],10);
    Ok(())
}
fn verify_typed_expression(candidate:&Candidate,suite:&LabSuite,out:&Path)->Result<()> {
    let mut trial=prepared_for_cast(candidate,suite,out,"negative")?;
    let pre_cast=reconnect(&trial)?;let(caster,target)=cast_participants(&pre_cast)?;
    let accepted=trial.advance(&CancellationToken::default()).map(|step|step.public_result().cloned())
        .map_err(|e|format!("ordinary Sword ActivateSkill must be accepted before release: {e:?}"))?;
    assert_eq!(trial.next_step_index(),13);
    let before=reconnect(&trial)?;
    save(&out.join("negative_accepted_pending_cast.json"),&json!({"pre_cast":pre_cast,"accepted":accepted,"before_clock":before}))?;
    let failure=trial.advance(&CancellationToken::default()).err().ok_or("ordinary Sword release clock must actually fault")?;
    assert_eq!(failure.position(),game_core::authoring::TrialPosition::Step{index:13});
    let typed=serde_json::to_value(failure.error())?;
    save(&out.join("direct_typed_expression_failure.json"),&json!({"before":before,"caster":caster,"target":target,"position":failure.position(),"failed_operation":"elapsed1ms after accepted ActivateSkill","failure":typed}))?;
    let AuthoringError::Application{diagnostic,..}=failure.error() else{return Err("actual application expression failure required".into());};
    assert_eq!(diagnostic.code(),"value_expression_fault");
    assert_eq!(diagnostic.details().as_value()["fault"]["code"],"divide_by_zero");
    assert_eq!(diagnostic.details().as_value()["fault"]["phase"],"evaluate");
    assert_eq!(failure.error().to_string(),"value_expression_fault");
    Ok(())
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=8&&args.len()!=9{return Err("usage: worker_boundaries SOURCE_ROOT REGISTRY SAFE_STAFF_CAST_SUITE PARENT_RUNTIME_RON ACTUAL_ENVIRONMENT PLAIN_WORKER INSTRUMENTED_WORKER NEW_PRIVATE_OUTPUT [PARENT_RUNTIME_SAFETY_RON]".into());}
    let root=Path::new(&args[0]);let out=Path::new(&args[7]);private(out)?;
    let original=state(root)?;
    let runtime=fs::read(&args[3])?;
    let (_parent_settings,parent_bytes)=runtime_profile(args.get(8).map(|p|Path::new(p)),&out.join("parent_runtime_context.json"))?;
    assert_eq!(runtime,parent_bytes,"expression successor must retain exact reviewed parent runtime bytes");
    let suite:LabSuite=serde_json::from_slice(&fs::read(&args[2])?)?;
    assert_eq!(suite.cases.len(),1);let case=&suite.cases[0];
    let mut required:Vec<_>=suite.required_definitions.iter().map(|k|format!("{:?}:{}",k.kind,k.id)).collect();required.sort();
    assert_eq!(required,vec!["Employee:records_clerk","PveEncounter:normal_corroded_patrol","RunPolicy:run_policy","Skill:training_berserker","SkillFragment:training_berserker"]);
    let recipe=case.preparation.as_ref().ok_or("real World preparation required")?;
    assert_eq!(recipe.employees.len(),1);assert_eq!(recipe.employees[0].count,1);assert_eq!(recipe.placements.len(),1);
    let document:Value=serde_json::from_slice(&case.scenario)?;
    assert_eq!(document["preparation_steps"].as_array().unwrap().len(),6);
    assert_eq!(document["steps"].as_array().unwrap().len(),9);
    assert_eq!(document["steps"][6]["command"]["type"],"activate_skill");
    assert_eq!(recipe.employees[0].equipment_ids.as_ref().unwrap(),&vec!["standard_training_healing_staff".to_owned(),"standard_armor".to_owned()]);
    assert_eq!(recipe.employees[0].active_fragment_id.as_deref(),Some("training_berserker"));
    let profile=LabProfile{revision:"u11-worker-boundary-expression-v1".into(),repeats:3,timeout_ms:300_000,
        max_output_bytes:16*1024*1024,runtime_safety_ron:runtime.clone(),criterion:None};
    let environment=format!("u11-reviewed-environment-sha256:{:x}",Sha256::digest(fs::read(&args[4])?));
    let pair=WorkerPair::open(Path::new(&args[5]).to_owned(),Path::new(&args[6]).to_owned(),environment)?;
    worker_evidence::preserve(&pair,&out.join("actual_worker_identity"))?;
    let settings=Arc::new(RuntimeSafetySettings::from_ron("actual paired expression boundary",&runtime)?);
    let base=TrustedBase::acquire(root,ContentSelection{packs:vec!["official".into()],winners:vec![]},&InputLimits::default(),settings,&CancellationToken::default())?;
    let candidate=Candidate::acquire(root,&fs::read(&args[1])?,&base,&InputLimits::default(),&CancellationToken::default())?;
    save(&out.join("actual_inputs.json"),&json!({"candidate":candidate.view(),"suite":suite,"profile":profile,"pair":pair.identity()}))?;
    verify_safe_amount(&candidate,&suite,out)?;
    // The expression and execution steps stay identical. Only the legal
    // loadout and its compatible deployment surface differ from safe +10.
    let mut failure_suite=suite.clone();failure_suite.revision="u11-actual-expression-fault-parity-v1".into();
    let recipe=failure_suite.cases[0].preparation.as_mut().unwrap();
    recipe.employees[0].equipment_ids=Some(vec!["standard_training_sword".into(),"standard_armor".into()]);
    recipe.placements[0].position=game_core::game::resources::Position::new(3,6);
    assert_eq!(failure_suite.cases[0].scenario,suite.cases[0].scenario);
    verify_typed_expression(&candidate,&failure_suite,out)?;
    if std::env::var_os("U11_EXPRESSION_DIRECT_PREFLIGHT_ONLY").as_deref()==Some(std::ffi::OsStr::new("1")) {
        original.ensure_unchanged(&state(root)?)?;
        save(&out.join("PREFLIGHT_RESULT.json"),&json!({"passed":true,"scope":"direct public hostile target preparation/preview/cast only",
            "safe_actual_hp_damage":10,"actual_sword_cause":"evaluate/divide_by_zero","paired_calibration_executed":false,"approval_claimed":false}))?;
        return Ok(());
    }
    let positive=out.join("private_positive");private(&positive)?;
    let result=LabRun::calibrate(&candidate,&suite,&profile,&pair,&positive,&CancellationToken::default());
    if let Err(error)=&result{save(&out.join("positive_failed.json"),error)?;}
    let calibration=result?;
    save(&out.join("actual_private_calibration.json"),&json!({"outcome":calibration.outcome(),"evidence":calibration.evidence_directory()}))?;
    assert!(calibration.outcome().metrics.is_complete()&&calibration.outcome().coverage.is_complete());
    let positive_before=state(&positive)?;
    #[cfg(unix)] {
        use std::os::unix::fs::{symlink,PermissionsExt};
        let wrong=out.join("wrong_mode");private(&wrong)?;fs::set_permissions(&wrong,fs::Permissions::from_mode(0o755))?;
        assert_eq!(fs::symlink_metadata(&wrong)?.permissions().mode()&0o777,0o755);
        let target=out.join("symlink_target");private(&target)?;
        let link=out.join("symlink_root");symlink(&target,&link)?;
        for(label,path)in [("wrong_mode",&wrong),("symlink",&link)] {
            let before=state(path)?;let target_before=state(&target)?;
            let error=LabRun::calibrate(&candidate,&suite,&profile,&pair,path,&CancellationToken::default())
                .err().ok_or("unsafe direct work root unexpectedly admitted")?;
            save(&out.join(format!("{label}_error.json")),&error)?;
            assert_eq!(error.kind,ModValidationErrorKind::Incomplete);assert_eq!(error.field_path,"worker.root");
            before.ensure_unchanged(&state(path)?)?;target_before.ensure_unchanged(&state(&target)?)?;
            save(&out.join(format!("{label}_unchanged.json")),&json!({"root":before.report(),"target":target_before.report(),"request_or_content_materialization":false}))?;
        }
    }
    positive_before.ensure_unchanged(&state(&positive)?)?;
    save(&out.join("actual_failure_suite.json"),&failure_suite)?;
    let failed=out.join("actual_failure_workers");private(&failed)?;
    let error=LabRun::calibrate(&candidate,&failure_suite,&profile,&pair,&failed,&CancellationToken::default())
        .err().ok_or("ordinary Sword cast did not produce the required real expression fault")?;
    save(&out.join("actual_paired_failure.json"),&error)?;
    assert_eq!(error.kind,ModValidationErrorKind::ContentFailed);assert_eq!(error.field_path,"lab.execute");
    assert_eq!(error.reason,"value_expression_fault");
    let dirs=fs::read_dir(&failed)?.collect::<std::result::Result<Vec<_>,_>>()?;assert_eq!(dirs.len(),1);
    assert!(dirs[0].file_type()?.is_dir());let work=dirs[0].path();
    let request_path=work.join("request.json");
    let request:WorkerRequest=serde_json::from_slice(&fs::read(&request_path)?)?;
    assert_eq!(request.schema_version,1);assert_eq!(request.candidate_identity,candidate.view().identity);
    assert_eq!(serde_json::to_value(&request.suite)?,serde_json::to_value(&failure_suite)?);assert_eq!(request.runtime_safety_ron,runtime);
    let mut observed=Vec::new();
    for repeat in 0..3 {
        for(mode,label,argument,build)in [(WorkerMode::Plain,"Plain","plain",&pair.identity().plain),
            (WorkerMode::Instrumented,"Instrumented","instrumented",&pair.identity().instrumented)] {
            let raw=work.join(format!("{repeat}-{label}.json"));
            let bytes=fs::read(&raw)?;assert!((bytes.len() as u64)<=profile.max_output_bytes);
            let response:WorkerResponse=serde_json::from_slice(&bytes)?;
            let WorkerResponse::Failed(report)=response else{return Err("complete response cannot stand in for a matching typed failure".into());};
            assert_eq!(report.schema_version,1);assert_eq!(report.nonce,request.nonce);assert_eq!(report.mode,mode);
            assert_eq!(&report.build,build);assert_eq!(report.candidate_identity,candidate.view().identity);assert_eq!(report.error,error);
            let process:Value=serde_json::from_slice(&fs::read(raw.with_extension("process.json"))?)?;
            assert_eq!(process["success"],true);assert_eq!(process["exit_code"],0);assert_eq!(process["child_reaped"],true);
            assert_eq!(process["stream_overflow"],false);assert!(process["supervisor_error"].is_null()&&process["stream_error"].is_null());
            assert_eq!(process["binary_sha256_before"],build.executable_sha256);assert_eq!(process["binary_sha256_after"],build.executable_sha256);
            assert_eq!(process["args"],json!(["--cmva-lab-worker",request_path,argument]));
            observed.push(json!({"repeat":repeat,"mode":mode,"raw":raw,"sha256":format!("{:x}",Sha256::digest(&bytes)),"process":process}));
        }
    }
    assert_eq!(observed.len(),6);
    positive_before.ensure_unchanged(&state(&positive)?)?;original.ensure_unchanged(&state(root)?)?;
    save(&out.join("RESULT.json"),&json!({"passed":true,"actual_private_positive":true,"safe_cast_independent_hp_damage":10,"direct_typed_expression_cause":"evaluate/divide_by_zero","private_positive_unchanged":true,
        "unsafe_roots_rejected_before_materialization":true,"actual_paired_failure":error,"actual_failed_envelopes":observed,
        "transport_crash_timeout_or_unparseable_report":false,"criterion_selected":false,"approval_claimed":false}))?;
    Ok(())
}
