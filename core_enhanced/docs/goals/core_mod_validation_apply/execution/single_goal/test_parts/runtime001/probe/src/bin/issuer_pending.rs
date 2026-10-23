//! Actual issuer admission/ledger negatives using real worker build identities.
//! This intentionally never runs a Lab and cannot produce an approval.
use game_core::{authoring::CancellationToken,
    game::data::content_pack::{ContentSelection,InputLimits},
    mod_validation::{Candidate,CancelDisposition,JobState,LabProfile,LabSuite,ModValidationErrorKind,
        TrustedBase,ValidationIssuer,WorkerPair},
    runtime_safety::{RuntimeSafetySettings,EMBEDDED_RUNTIME_SAFETY_RON}};
use serde::Serialize;
use serde_json::json;
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path};
use u11_independent_public_probe::file_state::FileState;
use u11_independent_public_probe::secret_oracle::require_secret_absent;

fn save(path:&Path,value:&impl Serialize)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn suite()->LabSuite {
    let scenario=serde_json::to_vec(&json!({"initial":{
        "player_id":"c0de1100-0000-0000-0000-000000000501","run_seed":11,"profile":"default_memory"},
        "preparation_steps":[],"steps":[{"kind":"snapshot","snapshot":"inventory"}]})).unwrap();
    serde_json::from_value(json!({"revision":"u11-ledger-only-v1","cases":[{
        "id":"unexecuted_inventory","class":"normal","scenario":scenario}]})).unwrap()
}
fn profile()->LabProfile {
    LabProfile{revision:"u11-unapproved-ledger-v1".into(),repeats:1,timeout_ms:30_000,
        max_output_bytes:1024*1024,runtime_safety_ron:EMBEDDED_RUNTIME_SAFETY_RON.to_vec(),criterion:None}
}
fn state(root:&Path)->std::io::Result<FileState>{FileState::capture(root,1024,32*1024*1024)}

fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=4{return Err("usage: issuer_pending FROZEN_FIXTURE_ROOT PLAIN_WORKER INSTRUMENTED_WORKER NEW_OUTPUT_DIRECTORY".into());}
    let root=Path::new(&args[0]);let plain=Path::new(&args[1]);let instrumented=Path::new(&args[2]);
    let out=Path::new(&args[3]);fs::create_dir(out)?;
    let pair_sequence=std::cell::Cell::new(0u32);
    let pair=||->Result<WorkerPair,Box<dyn Error>> {
        let pair=WorkerPair::open(plain.to_owned(),instrumented.to_owned(),"u11-ledger-only-no-measurement/v1".into())?;
        let index=pair_sequence.get();pair_sequence.set(index+1);
        u11_independent_public_probe::worker_evidence::preserve(&pair,&out.join(format!("worker_identity_{index}")))?;
        Ok(pair)
    };
    let base=TrustedBase::acquire(root,ContentSelection{packs:vec!["official".into()],winners:vec![]},
        &InputLimits::default(),RuntimeSafetySettings::embedded_shared(),&CancellationToken::default())?;
    let acquire=|name:&str|Candidate::acquire(root,&fs::read(root.join(format!("registries/{name}.json"))).unwrap(),
        &base,&InputLimits::default(),&CancellationToken::default());
    let source=acquire("ab")?;
    let cache=out.join("issuer_private");
    let issuer=ValidationIssuer::open(&cache,suite(),profile(),pair()?)?;
    let before=state(&cache)?;save(&out.join("private_before_hashes.json"),&before.report())?;
    let a=issuer.begin(source.individual("a",&base,&CancellationToken::default())?)?;
    let pending_a=issuer.query(a)?;save(&out.join("pending_a.json"),&pending_a)?;
    assert_eq!(pending_a.state,JobState::Pending);assert!(pending_a.current&&!pending_a.cache_hit);
    let error=issuer.approval(a).err().expect("a pending job has no capability");
    save(&out.join("pending_approval_denied.json"),&error)?;
    assert_eq!(error.kind,ModValidationErrorKind::Stale);
    let b=issuer.begin(source.individual("b",&base,&CancellationToken::default())?)?;
    let stale_a=issuer.query(a)?;let pending_b=issuer.query(b)?;
    save(&out.join("new_selection_stales_a.json"),&json!({"a":stale_a,"b":pending_b}))?;
    assert_eq!(stale_a.state,JobState::Stale);assert!(!stale_a.current);
    assert!(pending_b.current&&pending_b.selection_revision>pending_a.selection_revision);
    assert_eq!(issuer.cancel(b)?,CancelDisposition::Cancelled);
    let cancelled_b=issuer.query(b)?;save(&out.join("cancelled_b.json"),&cancelled_b)?;
    assert_eq!(cancelled_b.state,JobState::Cancelled);
    assert_eq!(issuer.cancel(b)?,CancelDisposition::Cancelled,"duplicate preaccept cancellation is idempotent");
    let denied=issuer.run(b).err().expect("cancelled job cannot start a worker");
    save(&out.join("cancelled_run_denied.json"),&denied)?;
    assert_eq!(issuer.query(b)?.state,JobState::Cancelled);
    assert!(issuer.approval(b).is_err());

    // The current finite ledger policy is 64 admitted jobs. Admission failure
    // must not increment the selection revision or discard the latest job.
    let mut newest=b;
    for _ in 2..64 {
        newest=issuer.begin(source.individual("a",&base,&CancellationToken::default())?)?;
    }
    let last_before=issuer.query(newest)?;save(&out.join("ledger_exact64.json"),&last_before)?;
    assert_eq!(last_before.state,JobState::Pending);assert!(last_before.current);
    let over=issuer.begin(source.individual("a",&base,&CancellationToken::default())?).err()
        .expect("65th admitted job must fail before changing current selection");
    save(&out.join("ledger_one_over.json"),&over)?;
    assert_eq!(over.kind,ModValidationErrorKind::Incomplete);
    let last_after=issuer.query(newest)?;
    assert_eq!(serde_json::to_value(&last_after)?,serde_json::to_value(&last_before)?);
    assert!(issuer.approval(newest).is_err());
    before.ensure_unchanged(&state(&cache)?)?;

    let foreign=ValidationIssuer::open(&out.join("foreign_private"),suite(),profile(),pair()?)?;
    let foreign_id=foreign.begin(acquire("b")?)?;
    let error=issuer.query(foreign_id).err().expect("foreign job ID grants no local ledger authority");
    save(&out.join("foreign_job_denied.json"),&error)?;
    assert_eq!(error.kind,ModValidationErrorKind::InvalidInput);
    let second_open=ValidationIssuer::open(&cache,suite(),profile(),pair()?).err()
        .expect("one private issuer root cannot have two live writer owners");
    save(&out.join("same_root_second_owner_denied.json"),&second_open)?;
    assert_eq!(second_open.kind,ModValidationErrorKind::Incomplete);
    before.ensure_unchanged(&state(&cache)?)?;
    drop(foreign);drop(issuer);
    let after=state(&cache)?;save(&out.join("private_after_hashes.json"),&after.report())?;
    before.ensure_unchanged(&after)?;
    let mut public_artifacts=Vec::new();
    for entry in fs::read_dir(out)? {let entry=entry?;
        if entry.file_type()?.is_file()&&entry.path().extension().is_some_and(|s|s=="json") {
            public_artifacts.push(entry.path());
        }
    }
    public_artifacts.sort();
    let own_absence=require_secret_absent(&cache,&public_artifacts,32*1024*1024)?;
    let foreign_absence=require_secret_absent(&out.join("foreign_private"),&public_artifacts,32*1024*1024)?;
    save(&out.join("key_absence.json"),&json!({"issuer":own_absence,"foreign":foreign_absence}))?;
    save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"actual issuer pending ledger and finite admission",
        "admitted_jobs":64,"approval_claimed":false,"lab_run_count":0,
        "worker_build_identity_processes_occurred":true,"private_bytes_exported":false}))?;
    println!("{}",json!({"passed":true,"approval_claimed":false,"output":out}));Ok(())
}
