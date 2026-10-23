//! Disposable private-cache mutations against actual accepted records.
//! Reads the original source without modification; never exports secret bytes.
use game_core::{authoring::CancellationToken,
    game::data::content_pack::InputLimits,
    mod_validation::{Candidate,JobState,LabProfile,LabSuite,ModValidationErrorKind,TrustedBase,ValidationIssuer,WorkerPair},
    runtime_safety::RuntimeSafetySettings};
use serde::Serialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::{Path,PathBuf},sync::Arc};
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
fn private_file(path:&Path,bytes:&[u8])->Result<(),Box<dyn Error>> {
    let mut options=OpenOptions::new();options.write(true).create_new(true);
    #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
    options.open(path)?.write_all(bytes)?;Ok(())
}
fn copy_cache(source:&Path,destination:&Path)->Result<(),Box<dyn Error>> {
    private_dir(destination)?;
    for entry in fs::read_dir(source)? {let entry=entry?;let ty=entry.file_type()?;
        if ty.is_dir()&&entry.file_name()=="workers" {continue;}
        if !ty.is_file(){return Err("source cache contains unexpected directory, symlink or special file".into());}
        if entry.metadata()?.len()>8*1024*1024 {return Err("source cache file exceeds probe bound".into());}
        private_file(&destination.join(entry.file_name()),&fs::read(entry.path())?)?;
    }
    private_dir(&destination.join("workers"))?;Ok(())
}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=9&&args.len()!=10{return Err("usage: cache_negative FROZEN_ROOT REGISTRY SUITE DIAGNOSTIC_PROFILE ACTUAL_ENVIRONMENT PLAIN_WORKER INSTRUMENTED_WORKER ORIGINAL_ACCEPTED_CACHE NEW_OUTPUT [TRUSTED_SELECTION_JSON]".into());}
    let root=Path::new(&args[0]);let source=Path::new(&args[7]);let out=Path::new(&args[8]);private_dir(out)?;
    let original=state(source)?;save(&out.join("source_before_hashes.json"),&original.report())?;
    let registry=fs::read(&args[1])?;let suite:LabSuite=serde_json::from_slice(&fs::read(&args[2])?)?;
    let profile:LabProfile=serde_json::from_slice(&fs::read(&args[3])?)?;assert!(profile.criterion.is_none());
    let environment_sha=format!("{:x}",Sha256::digest(fs::read(&args[4])?));
    let pair_sequence=std::cell::Cell::new(0u32);
    let pair=||->Result<WorkerPair,Box<dyn Error>> {
        let pair=WorkerPair::open(Path::new(&args[5]).to_owned(),Path::new(&args[6]).to_owned(),format!("u11-reviewed-environment-sha256:{environment_sha}"))?;
        let index=pair_sequence.get();pair_sequence.set(index+1);
        u11_independent_public_probe::worker_evidence::preserve(&pair,&out.join(format!("worker_identity_{index}")))?;
        Ok(pair)
    };
    let settings=Arc::new(RuntimeSafetySettings::from_ron("cache negative actual profile",&profile.runtime_safety_ron)?);
    let selection=u11_independent_public_probe::normal_preparation::trusted_selection(
        args.get(9).map(|path|Path::new(path)),&out.join("trusted_base_context.json"))?;
    let base=TrustedBase::acquire(root,selection,
        &InputLimits::default(),settings,&CancellationToken::default())?;
    let acquire=||Candidate::acquire(root,&registry,&base,&InputLimits::default(),&CancellationToken::default());
    let candidate=acquire()?;
    let mut matching:Vec<PathBuf>=Vec::new();let mut criteria=Vec::new();
    for entry in fs::read_dir(source)? {let entry=entry?;let name=entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".criterion.json"){criteria.push(entry.path());}
        if !name.ends_with(".accepted.json"){continue;}
        let envelope:Value=serde_json::from_slice(&fs::read(entry.path())?)?;
        let payload:Vec<u8>=serde_json::from_value(envelope["payload"].clone())?;
        let record:Value=serde_json::from_slice(&payload)?;
        if record["result"]["candidate_identity"]==json!(candidate.view().identity) {matching.push(entry.path());}
    }
    assert_eq!(matching.len(),1,"one actual candidate accepted record required for isolated mutation");
    assert_eq!(criteria.len(),1,"one actual adopted calibration context required");
    let accepted_name=matching[0].file_name().unwrap();let criterion_name=criteria[0].file_name().unwrap();
    // The control proves the copied private root and exact candidate/context are
    // usable before any failed authentication is attributed to a mutation.
    let control=out.join("control_private");copy_cache(source,&control)?;
    let issuer=ValidationIssuer::open(&control,suite.clone(),profile.clone(),pair()?)?;
    let id=issuer.begin(acquire()?)?;let before=state(&control)?;let result=issuer.run(id)?;
    save(&out.join("control_exact_hit.json"),&result)?;
    assert_eq!(result.state,JobState::Passed);assert!(result.cache_hit);assert!(issuer.approval(id).is_ok());
    before.ensure_unchanged(&state(&control)?)?;drop(issuer);
    for label in ["truncated_envelope","changed_tag","changed_payload","calibration_domain_transplant"] {
        let destination=out.join(format!("{label}_private"));copy_cache(source,&destination)?;
        let target=destination.join(accepted_name);
        let bytes=if label=="truncated_envelope" {b"{\"payload\":".to_vec()}
            else if label=="calibration_domain_transplant" {fs::read(destination.join(criterion_name))?}
            else {let mut value:Value=serde_json::from_slice(&fs::read(&target)?)?;
                let field=if label=="changed_tag"{"tag"}else{"payload"};
                let n=value[field][0].as_u64().expect("nonempty actual envelope field");
                value[field][0]=json!(n^1);serde_json::to_vec(&value)?};
        fs::write(&target,bytes)?;
        let issuer=ValidationIssuer::open(&destination,suite.clone(),profile.clone(),pair()?)?;
        let id=issuer.begin(acquire()?)?;let before=state(&destination)?;
        let result=issuer.run(id)?;save(&out.join(format!("{label}.json")),&result)?;
        assert_eq!(result.state,JobState::Incomplete);assert!(!result.cache_hit);
        assert!(issuer.approval(id).is_err());assert!(result.outcome.is_none());
        let error=result.error.expect("cache authentication failure must be visible");
        assert_eq!(error.kind,if label=="truncated_envelope" {ModValidationErrorKind::Incomplete}else{ModValidationErrorKind::ForeignIssuer});
        before.ensure_unchanged(&state(&destination)?)?;drop(issuer);
    }
    // Literal current finite policy: 128 accepted-or-partial members and an
    // 8MiB accepted envelope. The positive is always a real authenticated HIT.
    for total in [128usize,129] {
        let destination=out.join(format!("record_count_{total}_private"));copy_cache(source,&destination)?;
        let accepted:Vec<_>=fs::read_dir(&destination)?.collect::<std::io::Result<Vec<_>>>()?
            .into_iter().filter(|entry|entry.file_name().to_string_lossy().ends_with(".accepted.json")).map(|entry|entry.path()).collect();
        assert!(!accepted.is_empty()&&accepted.len()<128);
        assert!(!fs::read_dir(&destination)?.any(|entry|entry.unwrap().file_name().to_string_lossy().ends_with(".partial")));
        let protected:Vec<_>=accepted.iter().map(|path|state(path).map(|before|(path.clone(),before))).collect::<std::io::Result<_>>()?;
        for index in accepted.len()..total {
            private_file(&destination.join(format!("u11-quota-{index}.partial")),b"uncommitted, never approval")?;
        }
        let before=state(&destination)?;let workers=state(&destination.join("workers"))?;
        let opened=ValidationIssuer::open(&destination,suite.clone(),profile.clone(),pair()?);
        if total==128 {
            let issuer=opened?;let id=issuer.begin(acquire()?)?;let result=issuer.run(id)?;
            save(&out.join(format!("record_count_{total}.json")),&result)?;
            assert_eq!(result.state,JobState::Passed);assert!(result.cache_hit);assert!(issuer.approval(id).is_ok());
            assert!(!fs::read_dir(&destination)?.any(|entry|entry.unwrap().file_name().to_string_lossy().ends_with(".partial")));
            drop(issuer);
        }else{
            let error=opened.err().expect("129th record must refuse during actual criterion restoration");
            save(&out.join(format!("record_count_{total}_open_error.json")),&error)?;
            assert_eq!(error.kind,ModValidationErrorKind::Incomplete);assert_eq!(error.field_path,"cache.records");
            before.ensure_unchanged(&state(&destination)?)?;
        }
        for(path,prior)in protected{prior.ensure_unchanged(&state(&path)?)?;}
        workers.ensure_unchanged(&state(&destination.join("workers"))?)?;
        original.ensure_unchanged(&state(source)?)?;
    }
    for extra in [0usize,1] {
        let destination=out.join(format!("record_bytes_plus_{extra}_private"));copy_cache(source,&destination)?;
        let target=destination.join(accepted_name);let mut bytes=fs::read(&target)?;
        assert!(bytes.len()<8*1024*1024);
        bytes.resize(8*1024*1024+extra,b' ');fs::write(&target,bytes)?;
        let before=state(&destination)?;
        let issuer=ValidationIssuer::open(&destination,suite.clone(),profile.clone(),pair()?)?;
        let id=issuer.begin(acquire()?)?;let result=issuer.run(id)?;
        save(&out.join(format!("record_bytes_plus_{extra}.json")),&result)?;
        if extra==0 {
            assert_eq!(result.state,JobState::Passed);assert!(result.cache_hit);assert!(issuer.approval(id).is_ok());
        }else{
            assert_eq!(result.state,JobState::Incomplete);assert!(!result.cache_hit);assert!(result.outcome.is_none());
            assert!(issuer.approval(id).is_err());let error=result.error.as_ref().unwrap();
            assert_eq!(error.kind,ModValidationErrorKind::Incomplete);assert_eq!(error.field_path,"cache.record");
            assert_eq!(error.reason,"record byte ceiling");
        }
        before.ensure_unchanged(&state(&destination)?)?;drop(issuer);
        original.ensure_unchanged(&state(source)?)?;
    }
    // The independent accepted control remains usable after every saturation.
    let before=state(&control)?;
    let issuer=ValidationIssuer::open(&control,suite.clone(),profile.clone(),pair()?)?;
    let id=issuer.begin(acquire()?)?;let result=issuer.run(id)?;
    save(&out.join("control_after_saturation_hit.json"),&result)?;
    assert_eq!(result.state,JobState::Passed);assert!(result.cache_hit);assert!(issuer.approval(id).is_ok());
    before.ensure_unchanged(&state(&control)?)?;drop(issuer);
    let destination=out.join("criterion_tamper_private");copy_cache(source,&destination)?;
    let target=destination.join(criterion_name);let mut value:Value=serde_json::from_slice(&fs::read(&target)?)?;
    let n=value["tag"][0].as_u64().unwrap();value["tag"][0]=json!(n^1);fs::write(&target,serde_json::to_vec(&value)?)?;
    let before=state(&destination)?;
    let error=ValidationIssuer::open(&destination,suite,profile,pair()?).err().expect("tampered adoption cannot reopen an issuer");
    save(&out.join("criterion_tamper.json"),&error)?;assert_eq!(error.kind,ModValidationErrorKind::ForeignIssuer);
    before.ensure_unchanged(&state(&destination)?)?;
    original.ensure_unchanged(&state(source)?)?;
    let mut artifacts=Vec::new();for entry in fs::read_dir(out)? {let entry=entry?;if entry.file_type()?.is_file(){artifacts.push(entry.path());}}
    save(&out.join("key_absence.json"),&require_secret_absent(source,&artifacts,64*1024*1024)?)?;
    save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"actual accepted record authentication and finite cache boundaries",
        "exact_control_hit":true,"mutations_rejected":5,"finite_cache_boundaries":4,"original_cache_unchanged":true,
        "no_lab_worker_rerun":true,"private_secret_bytes_exported":false}))?;
    println!("{}",json!({"passed":true,"output":out}));Ok(())
}
