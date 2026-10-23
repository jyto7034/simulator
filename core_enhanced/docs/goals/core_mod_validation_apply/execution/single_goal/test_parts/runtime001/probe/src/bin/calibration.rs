//! Collect actual fresh-process paired evidence for Root's numeric review.
//! No criterion is selected, adopted, serialized as authority, or applied here.
use game_core::{authoring::CancellationToken,
    game::data::content_pack::InputLimits,
    mod_validation::{Candidate,LabProfile,LabRun,LabSuite,TrustedBase,WorkerPair,WorkloadClass},
    runtime_safety::RuntimeSafetySettings};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest,Sha256};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path,sync::Arc};
use u11_independent_public_probe::file_state::FileState;

fn save(path:&Path,value:&impl Serialize)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn private_directory(path:&Path)->Result<(),Box<dyn Error>> {
    #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;
        let mut builder=fs::DirBuilder::new();builder.mode(0o700);builder.create(path)?;Ok(())}
    #[cfg(not(unix))] {let _=path;Err("actual private Unix worker environment required".into())}
}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=8&&args.len()!=9&&args.len()!=10 {return Err("usage: calibration FROZEN_FIXTURE_ROOT REGISTRY_JSON TRUSTED_SUITE_JSON DIAGNOSTIC_PROFILE_JSON ACTUAL_ENVIRONMENT_JSON PLAIN_WORKER INSTRUMENTED_WORKER NEW_OUTPUT_DIRECTORY [combined|normal|expanded|pathological [TRUSTED_SELECTION_JSON]]".into());}
    let root=Path::new(&args[0]);let out=Path::new(&args[7]);private_directory(out)?;
    let registry=fs::read(&args[1])?;let suite_bytes=fs::read(&args[2])?;
    let profile_bytes=fs::read(&args[3])?;let environment_bytes=fs::read(&args[4])?;
    let suite:LabSuite=serde_json::from_slice(&suite_bytes)?;
    let profile:LabProfile=serde_json::from_slice(&profile_bytes)?;
    let environment:serde_json::Value=serde_json::from_slice(&environment_bytes)?;
    assert!(profile.criterion.is_none(),"numeric adoption follows actual raw review");
    let scope=args.get(8).map(|s|s.to_str().ok_or("UTF-8 collection scope required")).transpose()?.unwrap_or("combined");
    match scope {
        "combined"=>for class in [WorkloadClass::Normal,WorkloadClass::Expanded,WorkloadClass::Pathological] {
            assert!(suite.cases.iter().any(|case|case.class==class),"combined calibration must retain every declared class");
        },
        "normal"|"expanded"|"pathological"=>{
            let class=match scope{"normal"=>WorkloadClass::Normal,"expanded"=>WorkloadClass::Expanded,_=>WorkloadClass::Pathological};
            assert!(!suite.cases.is_empty()&&suite.cases.iter().all(|case|case.class==class),"isolated actual process window must contain only its declared class");
        },
        _=>return Err("unknown actual collection scope".into()),
    }
    let environment_sha256=format!("{:x}",Sha256::digest(&environment_bytes));
    let pair=WorkerPair::open(Path::new(&args[5]).to_owned(),Path::new(&args[6]).to_owned(),
        format!("u11-reviewed-environment-sha256:{environment_sha256}"))?;
    u11_independent_public_probe::worker_evidence::preserve(&pair,&out.join("worker_identity"))?;
    let settings=Arc::new(RuntimeSafetySettings::from_ron("actual independent calibration profile",&profile.runtime_safety_ron)?);
    let selection=u11_independent_public_probe::normal_preparation::trusted_selection(
        args.get(9).map(|path|Path::new(path)),&out.join("trusted_base_context.json"))?;
    let base=TrustedBase::acquire(root,selection,
        &InputLimits::default(),settings,&CancellationToken::default())?;
    let candidate=Candidate::acquire(root,&registry,&base,&InputLimits::default(),&CancellationToken::default())?;
    save(&out.join("inputs.json"),&json!({"registry_sha256":format!("{:x}",Sha256::digest(&registry)),
        "suite_sha256":format!("{:x}",Sha256::digest(&suite_bytes)),"profile_sha256":format!("{:x}",Sha256::digest(&profile_bytes)),
        "environment_sha256":environment_sha256,"environment":environment,"suite":suite,"profile":profile,
        "pair":pair.identity(),"candidate":candidate.view(),"collection_scope":scope,"criterion_adopted":false}))?;
    let work=out.join("workers");private_directory(&work)?;
    let result=LabRun::calibrate(&candidate,&suite,&profile,&pair,&work,&CancellationToken::default());
    match result {
        Ok(calibration)=>{
            save(&out.join("actual_calibration.json"),&json!({"outcome":calibration.outcome(),
                "raw_evidence_directory":calibration.evidence_directory(),"criterion_adopted":false,"approval_claimed":false}))?;
            let inventory=FileState::capture(&work,65_536,128*1024*1024)?;
            save(&out.join("raw_evidence_inventory.json"),&inventory.report())?;
            assert!(calibration.outcome().metrics.is_complete(),"all actual paired phase windows must complete");
            assert!(calibration.outcome().coverage.is_complete(),"declared workload labels cannot replace actual required source/site consumption");
            assert!(!calibration.outcome().is_passed(),"diagnostic calibration has no adopted criterion");
            save(&out.join("RESULT.json"),&json!({"paired_calibration_collected":true,
                "required_coverage_complete":true,"numeric_criterion_selected":false,"approval_claimed":false,
                "collection_scope":scope,"separate_suite_identity":true,
                "scope":"actual paired calibration collection; per-case workload validity still requires independent gameplay oracles"}))?;
        }
        Err(error)=>{
            save(&out.join("failed_calibration.json"),&error)?;
            let inventory=FileState::capture(&work,65_536,128*1024*1024)?;
            save(&out.join("raw_evidence_inventory.json"),&inventory.report())?;
            return Err(error.into());
        }
    }
    println!("{}",json!({"paired_calibration_collected":true,"criterion_adopted":false,"output":out}));Ok(())
}
