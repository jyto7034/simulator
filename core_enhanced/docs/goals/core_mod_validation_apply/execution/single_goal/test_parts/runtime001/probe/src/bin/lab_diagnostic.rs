//! Public diagnostic and missing-witness negatives. No paired metrics, approval,
//! calibrated pass, or normal gameplay coverage is claimed by this executable.
use game_core::{authoring::CancellationToken,
    game::data::content_pack::{ContentSelection,InputLimits},
    mod_validation::{Candidate,LabProfile,LabRun,LabSuite,ModValidationError,ModValidationErrorKind,TrustedBase},
    runtime_safety::{RuntimeSafetySettings,EMBEDDED_RUNTIME_SAFETY_RON}};
use serde::Serialize;
use serde_json::{json,Value};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path};

fn save(path:&Path,value:&impl Serialize)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn observe(result:&Result<LabRun,ModValidationError>)->Value {
    match result {Ok(run)=>json!({"returned_outcome":true,"outcome":run.outcome()}),
        Err(error)=>json!({"returned_outcome":false,"error":error})}
}
fn suite()->LabSuite {
    // A real Application snapshot from a fresh default-memory profile. It has
    // deliberately performed no battle/equipment trigger/skill consumption.
    let scenario=serde_json::to_vec(&json!({
        "initial":{"player_id":"c0de1100-0000-0000-0000-000000000401","run_seed":11,"profile":"default_memory"},
        "preparation_steps":[],"steps":[{"kind":"snapshot","snapshot":"inventory"}]
    })).unwrap();
    serde_json::from_value(json!({"revision":"u11-independent-missing-witness-v1",
        "cases":[{"id":"inventory_only","class":"normal","scenario":scenario}]})).unwrap()
}
fn profile()->LabProfile {
    LabProfile{revision:"u11-independent-diagnostic-v1".into(),repeats:1,timeout_ms:30_000,
        max_output_bytes:1024*1024,runtime_safety_ron:EMBEDDED_RUNTIME_SAFETY_RON.to_vec(),criterion:None}
}

fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=2{return Err("usage: lab_diagnostic FROZEN_FIXTURE_ROOT NEW_OUTPUT_DIRECTORY".into());}
    let root=Path::new(&args[0]);let out=Path::new(&args[1]);fs::create_dir(out)?;
    let base=TrustedBase::acquire(root,ContentSelection{packs:vec!["official".into()],winners:vec![]},
        &InputLimits::default(),RuntimeSafetySettings::embedded_shared(),&CancellationToken::default())?;
    let candidate=Candidate::acquire(root,&fs::read(root.join("registries/a.json"))?,&base,
        &InputLimits::default(),&CancellationToken::default())?;
    let suite=suite();let profile=profile();
    let result=LabRun::diagnose(&candidate,&suite,&profile,&CancellationToken::default());
    save(&out.join("unreached_actual_diagnostic.json"),&observe(&result))?;
    // This input never establishes a battle. The preparation boundary rejects
    // it before a coverage report exists; blaming the untouched skill would
    // turn an invalid experiment setup into a content failure.
    let setup_error=result.err().expect("inventory alone is not a prepared battle");
    assert_eq!(setup_error.kind,ModValidationErrorKind::Incomplete);
    assert_eq!(setup_error.field_path,"lab.prepare");
    assert!(setup_error.reason.contains("authoring_trial_battle_not_prepared"));

    let mut profiles=vec![];
    for (field,value,label) in [("repeats",0,"zero_repeats"),("repeats",32,"over_repeats"),
        ("timeout_ms",0,"zero_timeout"),("timeout_ms",300_001,"over_timeout"),
        ("max_output_bytes",0,"zero_output"),("max_output_bytes",16*1024*1024+1,"over_output")] {
        let mut invalid=profile.clone();
        match field {"repeats"=>invalid.repeats=value as u32,"timeout_ms"=>invalid.timeout_ms=value,
            "max_output_bytes"=>invalid.max_output_bytes=value,_=>unreachable!()}
        profiles.push((label,invalid));
    }
    let mut altered=profile.clone();altered.runtime_safety_ron.push(b'\n');profiles.push(("changed_exact_safety",altered));
    for (label,invalid) in profiles {
        let result=LabRun::diagnose(&candidate,&suite,&invalid,&CancellationToken::default());
        save(&out.join(format!("{label}.json")),&observe(&result))?;
        assert_eq!(result.err().expect("invalid trusted profile must be rejected").kind,ModValidationErrorKind::InvalidInput);
    }
    let mut duplicate=serde_json::to_value(&suite)?;let case=duplicate["cases"][0].clone();
    duplicate["cases"].as_array_mut().unwrap().push(case);
    let invalid:LabSuite=serde_json::from_value(duplicate)?;
    let result=LabRun::diagnose(&candidate,&invalid,&profile,&CancellationToken::default());
    save(&out.join("duplicate_case_id.json"),&observe(&result))?;
    assert_eq!(result.err().expect("duplicate suite case identity must fail").kind,ModValidationErrorKind::InvalidInput);
    let cancellation=CancellationToken::default();cancellation.cancel();
    let result=LabRun::diagnose(&candidate,&suite,&profile,&cancellation);
    save(&out.join("cancelled_diagnostic.json"),&observe(&result))?;
    assert_eq!(result.err().expect("cancelled diagnostic cannot run").kind,ModValidationErrorKind::Cancelled);
    save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"setup incomplete and invalid diagnostic request isolation",
        "approval_claimed":false,"paired_measurement_claimed":false,"normal_gameplay_coverage_claimed":false}))?;
    println!("{}",json!({"passed":true,"approval_claimed":false,"output":out}));Ok(())
}
