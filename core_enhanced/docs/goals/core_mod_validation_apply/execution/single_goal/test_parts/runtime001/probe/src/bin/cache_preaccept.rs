//! Real filesystem create failure after actual calibrated validation succeeds.
//! The source cache is read-only; all obstruction occurs in a disposable copy.
use game_core::{authoring::CancellationToken,
    game::data::content_pack::InputLimits,
    mod_validation::{ApprovedHost,Candidate,JobState,LabProfile,LabSuite,TrustedBase,ValidationIssuer,WorkerPair},
    runtime_safety::RuntimeSafetySettings};
use serde::Serialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path,sync::Arc};
use u11_independent_public_probe::{file_state::FileState,secret_oracle::require_secret_absent};
fn save(path:&Path,value:&impl Serialize)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn state(path:&Path)->std::io::Result<FileState>{FileState::capture(path,65_536,256*1024*1024)}
fn private_dir(path:&Path)->Result<(),Box<dyn Error>> {
    #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;
        let mut b=fs::DirBuilder::new();b.mode(0o700);b.create(path)?;Ok(())}
    #[cfg(not(unix))] {let _=path;Err("private Unix environment required".into())}
}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=10&&args.len()!=11{return Err("usage: cache_preaccept FROZEN_ROOT A_REGISTRY B_REGISTRY SUITE DIAGNOSTIC_PROFILE ACTUAL_ENVIRONMENT PLAIN_WORKER INSTRUMENTED_WORKER ORIGINAL_ACCEPTED_CACHE NEW_OUTPUT [TRUSTED_SELECTION_JSON]".into());}
    let root=Path::new(&args[0]);let source=Path::new(&args[8]);let out=Path::new(&args[9]);private_dir(out)?;
    let original=state(source)?;
    let suite:LabSuite=serde_json::from_slice(&fs::read(&args[3])?)?;
    let profile:LabProfile=serde_json::from_slice(&fs::read(&args[4])?)?;assert!(profile.criterion.is_none());
    let environment=format!("u11-reviewed-environment-sha256:{:x}",Sha256::digest(fs::read(&args[5])?));
    let pair=WorkerPair::open(Path::new(&args[6]).to_owned(),Path::new(&args[7]).to_owned(),environment)?;
    u11_independent_public_probe::worker_evidence::preserve(&pair,&out.join("worker_identity"))?;
    let settings=Arc::new(RuntimeSafetySettings::from_ron("preaccept actual profile",&profile.runtime_safety_ron)?);
    let selection=u11_independent_public_probe::normal_preparation::trusted_selection(
        args.get(10).map(|path|Path::new(path)),&out.join("trusted_base_context.json"))?;
    let base=TrustedBase::acquire(root,selection,
        &InputLimits::default(),settings,&CancellationToken::default())?;
    let a=Candidate::acquire(root,&fs::read(&args[1])?,&base,&InputLimits::default(),&CancellationToken::default())?;
    let b=Candidate::acquire(root,&fs::read(&args[2])?,&base,&InputLimits::default(),&CancellationToken::default())?;
    assert_ne!(a.view().identity,b.view().identity);
    let cache=out.join("disposable_private");private_dir(&cache)?;
    let mut b_records=Vec::new();
    for entry in fs::read_dir(source)? {let entry=entry?;let ty=entry.file_type()?;
        if ty.is_dir()&&entry.file_name()=="workers" {continue;}
        if !ty.is_file()||entry.metadata()?.len()>8*1024*1024 {return Err("unexpected source cache entry".into());}
        let bytes=fs::read(entry.path())?;
        let mut options=OpenOptions::new();options.write(true).create_new(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
        options.open(cache.join(entry.file_name()))?.write_all(&bytes)?;
        if entry.file_name().to_string_lossy().ends_with(".accepted.json") {
            let envelope:Value=serde_json::from_slice(&bytes)?;
            let payload:Vec<u8>=serde_json::from_value(envelope["payload"].clone())?;
            let record:Value=serde_json::from_slice(&payload)?;
            if record["result"]["candidate_identity"]==json!(b.view().identity) {
                b_records.push((cache.join(entry.file_name()),record["key"].as_str().expect("actual key").to_owned()));
            }
        }
    }
    assert_eq!(b_records.len(),1,"one actual accepted B record required");
    let (old_b,key)=b_records.pop().unwrap();
    // Remove only the test copy's B acceptance, forcing a genuine fresh Lab.
    fs::remove_file(old_b)?;private_dir(&cache.join("workers"))?;
    let issuer=ValidationIssuer::open(&cache,suite,profile,pair)?;
    let job_a=issuer.begin(a)?;let control=issuer.run(job_a)?;
    save(&out.join("a_control_hit.json"),&control)?;
    assert_eq!(control.state,JobState::Passed);assert!(control.cache_hit);
    let host=ApprovedHost::new(Arc::clone(&issuer),issuer.approval(job_a)?)?;
    let host_before=serde_json::to_value(host.query()?)?;
    let job_b=issuer.begin(b)?;let nonce=serde_json::to_value(job_b)?;
    let nonce=nonce.as_str().expect("public JobId serialization exposes UUID identifier");
    let obstruction=cache.join(format!("{key}-{nonce}.partial"));private_dir(&obstruction)?;
    let mut protected=Vec::new();
    for entry in fs::read_dir(&cache)? {let entry=entry?;
        if entry.file_type()?.is_file(){protected.push((entry.path(),state(&entry.path())?));}}
    let result=issuer.run(job_b)?;
    save(&out.join("actual_passed_validation_blocked_publication.json"),&result)?;
    assert_eq!(result.state,JobState::Incomplete);assert!(!result.cache_hit);
    assert!(result.error.is_some());assert!(result.outcome.as_ref().expect("real Lab outcome").is_passed(),
        "test must reach publication after a real complete calibrated pass");
    assert!(issuer.approval(job_b).is_err());assert_eq!(serde_json::to_value(host.query()?)?,host_before);
    for (path,before) in &protected {before.ensure_unchanged(&state(path)?)?;}
    let accepted=cache.join(format!("{key}-{nonce}.accepted.json"));assert!(!accepted.exists());
    assert!(obstruction.is_dir(),"filesystem obstruction is an actual directory, not a fake I/O return");
    save(&out.join("private_after_hashes.json"),&state(&cache)?.report())?;
    drop(host);drop(issuer);original.ensure_unchanged(&state(source)?)?;
    let mut artifacts=Vec::new();for entry in fs::read_dir(out)? {let entry=entry?;if entry.file_type()?.is_file(){artifacts.push(entry.path());}}
    save(&out.join("key_absence.json"),&require_secret_absent(source,&artifacts,64*1024*1024)?)?;
    save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"real preaccept create failure after actual calibrated pass",
        "validation_passed":true,"new_approval_issued":false,"prior_host_selection_unchanged":true,
        "prior_accepted_files_unchanged":true,"original_source_cache_unchanged":true,
        "post_rename_durability_failure_claimed":false}))?;
    println!("{}",json!({"passed":true,"output":out}));Ok(())
}
