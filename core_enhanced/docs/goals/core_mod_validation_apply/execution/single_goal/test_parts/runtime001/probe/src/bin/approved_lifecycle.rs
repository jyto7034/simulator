//! Actual calibrated issuer/cache/host lifecycle. Sol is the only executor.
//! Numeric input is a separately reviewed artifact, never inferred by this probe.
use game_core::{authoring::CancellationToken,
    game::data::content_pack::InputLimits,
    mod_validation::{ApprovedHost,Candidate,CancelDisposition,CriterionLimits,JobId,JobState,
        LabProfile,LabRun,LabSuite,ModValidationErrorKind,TrustedBase,ValidationIssuer,WorkerPair},
    runtime_safety::RuntimeSafetySettings};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path,sync::Arc};
use u11_independent_public_probe::{criterion_oracle::verify_boundaries,file_state::FileState,
    secret_oracle::require_secret_absent};

#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
struct ReviewedLimits {
    review_reference:String,
    reviewed_calibration_file_sha256:String,
    revision:String,
    limits:CriterionLimits,
}
fn save(path:&Path,value:&impl Serialize)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn private_dir(path:&Path)->Result<(),Box<dyn Error>> {
    #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;
        let mut builder=fs::DirBuilder::new();builder.mode(0o700);builder.create(path)?;Ok(())}
    #[cfg(not(unix))] {let _=path;Err("actual private Unix worker environment required".into())}
}
fn state(path:&Path)->std::io::Result<FileState>{FileState::capture(path,65_536,256*1024*1024)}
fn passed(issuer:&ValidationIssuer,id:JobId,out:&Path,label:&str,expected_hit:bool)
    ->Result<(),Box<dyn Error>> {
    let result=issuer.run(id);
    save(&out.join(format!("{label}.json")),&result)?;
    let view=result?;
    assert_eq!(view.state,JobState::Passed,"actual accepted outcome required for {label}");
    assert!(view.current);assert_eq!(view.cache_hit,expected_hit);
    let outcome=view.outcome.as_ref().expect("actual result retained");
    assert!(outcome.is_passed()&&outcome.metrics.is_complete()&&outcome.coverage.is_complete());
    Ok(())
}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=11&&args.len()!=12 {return Err("usage: approved_lifecycle FROZEN_ROOT A_REGISTRY B_REGISTRY SUITE DIAGNOSTIC_PROFILE ACTUAL_ENVIRONMENT PLAIN_WORKER INSTRUMENTED_WORKER REVIEWED_LIMITS REVIEWED_CALIBRATION_FILE NEW_OUTPUT [TRUSTED_SELECTION_JSON]".into());}
    let root=Path::new(&args[0]);let out=Path::new(&args[10]);private_dir(out)?;
    let a_bytes=fs::read(&args[1])?;let b_bytes=fs::read(&args[2])?;
    let suite_bytes=fs::read(&args[3])?;let profile_bytes=fs::read(&args[4])?;
    let environment_bytes=fs::read(&args[5])?;let reviewed_bytes=fs::read(&args[8])?;
    let prior_bytes=fs::read(&args[9])?;
    let reviewed:ReviewedLimits=serde_json::from_slice(&reviewed_bytes)?;
    assert!(!reviewed.review_reference.trim().is_empty(),"review reference required");
    assert_eq!(reviewed.reviewed_calibration_file_sha256,format!("{:x}",Sha256::digest(&prior_bytes)),
        "Root's limits must name the exact actual calibration file reviewed");
    let prior:Value=serde_json::from_slice(&prior_bytes)?;
    let suite:LabSuite=serde_json::from_slice(&suite_bytes)?;
    let diagnostic:LabProfile=serde_json::from_slice(&profile_bytes)?;
    assert!(diagnostic.criterion.is_none());
    let environment_sha=format!("{:x}",Sha256::digest(&environment_bytes));
    let pair_sequence=std::cell::Cell::new(0u32);
    let pair=||->Result<WorkerPair,Box<dyn Error>> {
        let pair=WorkerPair::open(Path::new(&args[6]).to_owned(),Path::new(&args[7]).to_owned(),
            format!("u11-reviewed-environment-sha256:{environment_sha}"))?;
        let index=pair_sequence.get();pair_sequence.set(index+1);
        u11_independent_public_probe::worker_evidence::preserve(&pair,&out.join(format!("worker_identity_{index}")))?;
        Ok(pair)
    };
    let workers=pair()?;
    let settings=Arc::new(RuntimeSafetySettings::from_ron("independent approved lifecycle",&diagnostic.runtime_safety_ron)?);
    let selection=u11_independent_public_probe::normal_preparation::trusted_selection(
        args.get(11).map(|path|Path::new(path)),&out.join("trusted_base_context.json"))?;
    let base=TrustedBase::acquire(root,selection,
        &InputLimits::default(),settings,&CancellationToken::default())?;
    let acquire=|bytes:&[u8]|Candidate::acquire(root,bytes,&base,&InputLimits::default(),&CancellationToken::default());
    let a=acquire(&a_bytes)?;let b=acquire(&b_bytes)?;
    assert_ne!(a.view().identity,b.view().identity,"meaningful content identity change required");
    assert_ne!(a.required_configuration(),b.required_configuration());
    save(&out.join("inputs.json"),&json!({"suite":suite,"diagnostic_profile":diagnostic,
        "pair":workers.identity(),"environment_sha256":environment_sha,"reviewed_limits":reviewed,
        "reviewed_limits_file_sha256":format!("{:x}",Sha256::digest(&reviewed_bytes)),
        "a":a.view(),"b":b.view(),"a_registry_sha256":format!("{:x}",Sha256::digest(&a_bytes)),
        "b_registry_sha256":format!("{:x}",Sha256::digest(&b_bytes))}))?;
    let calibration_root=out.join("fresh_calibration");private_dir(&calibration_root)?;
    let fresh=LabRun::calibrate(&a,&suite,&diagnostic,&workers,&calibration_root,&CancellationToken::default());
    if let Err(error)=&fresh {save(&out.join("failed_fresh_calibration.json"),error)?;}
    let calibration=fresh?;
    save(&out.join("fresh_calibration.json"),&json!({"outcome":calibration.outcome(),
        "raw_evidence_directory":calibration.evidence_directory()}))?;
    assert!(calibration.outcome().coverage.is_complete());
    let current=serde_json::to_value(calibration.outcome())?;
    for field in ["candidate_identity","suite_sha256","profile_sha256"] {
        assert!(!prior["outcome"][field].is_null(),"reviewed calibration must contain {field}");
        assert_eq!(prior["outcome"][field],current[field],"reviewed calibration {field} must match fresh calibration");
    }
    assert_eq!(prior["outcome"]["metrics"]["pair"],current["metrics"]["pair"],
        "reviewed calibration must use the exact worker pair and environment");
    let criterion=calibration.adopt(reviewed.revision.clone(),reviewed.limits.clone())?;
    save(&out.join("adopted_criterion_metadata.json"),&criterion)?;
    let mut profile=diagnostic.clone();profile.criterion=Some(criterion.clone());
    let cache=out.join("issuer_private");
    let issuer=ValidationIssuer::open(&cache,suite.clone(),profile.clone(),workers)?;
    let job_a=issuer.begin(a)?;
    passed(&issuer,job_a,out,"initial_a_actual_miss",false)?;
    let a_outcome=issuer.query(job_a)?.outcome.unwrap();
    verify_boundaries(&criterion,&a_outcome.metrics,&out.join("comparator"))?;
    let approval_a=issuer.approval(job_a)?;
    let host=ApprovedHost::new(Arc::clone(&issuer),approval_a.clone())?;
    let (mut active_a,required_a)=host.new_application(11)?;
    let required_a=u11_independent_public_probe::core_host_oracle::start(&host,&mut active_a,required_a,&out.join("normal_core_start_a"))?;
    assert_eq!(active_a.content_identity(),&required_a.content_identity);
    let active_a_before=u11_independent_public_probe::core_host_oracle::snapshot(&active_a)?;
    let (mut dormant_a,dormant_required_a)=host.new_application(13)?;
    assert_eq!(dormant_required_a,required_a);assert!(dormant_a.is_not_started());
    let dormant_a_before=u11_independent_public_probe::core_host_oracle::snapshot(&dormant_a)?;
    let before_duplicate=state(&cache)?;
    assert_eq!(issuer.cancel(job_a)?,CancelDisposition::TooLatePublicationAccepted);
    let duplicate=issuer.run(job_a).err().expect("completed job cannot rerun");
    save(&out.join("duplicate_and_late_cancel.json"),&duplicate)?;
    assert_eq!(issuer.query(job_a)?.state,JobState::Passed);
    before_duplicate.ensure_unchanged(&state(&cache)?)?;

    // Another issuer cannot consume an actual local capability even when it
    // adopts the same calibration and accepts the same suite/pair identity.
    let foreign=ValidationIssuer::open(&out.join("foreign_private"),suite.clone(),profile.clone(),pair()?)?;
    let denied=ApprovedHost::new(Arc::clone(&foreign),approval_a.clone()).err().expect("foreign issuer denied");
    save(&out.join("foreign_capability_denied.json"),&denied)?;
    assert_eq!(denied.kind,ModValidationErrorKind::ForeignIssuer);
    drop(foreign);

    // Approval B changes only the future host choice. An existing application
    // keeps A's exact generation. This is a Core facade oracle; normal server
    // persistence and actual battle behavior have separate required probes.
    let job_b=issuer.begin(b)?;
    let host_before=serde_json::to_value(host.query()?)?;
    let denied=ApprovedHost::new(Arc::clone(&issuer),approval_a.clone()).err()
        .expect("a never-installed host cannot consume a stale capability");
    save(&out.join("stale_constructor_capability_denied.json"),&denied)?;
    assert_eq!(denied.kind,ModValidationErrorKind::Stale);
    assert_eq!(serde_json::to_value(host.query()?)?,host_before,
        "previously installed A remains the selected future while B validates");
    let denied=host.apply(approval_a.clone()).err().expect("old revision denied");
    save(&out.join("stale_capability_denied.json"),&denied)?;
    assert_eq!(denied.kind,ModValidationErrorKind::Stale);
    assert_eq!(serde_json::to_value(host.query()?)?,host_before);
    passed(&issuer,job_b,out,"changed_b_actual_miss",false)?;
    let approval_b=issuer.approval(job_b)?;
    let selection_b=host.apply(approval_b.clone())?;
    let (mut future_b,required_b)=host.new_application(12)?;
    let required_b=u11_independent_public_probe::core_host_oracle::start(&host,&mut future_b,required_b,&out.join("normal_core_start_b"))?;
    assert_eq!(active_a.content_identity(),&required_a.content_identity);
    assert_eq!(future_b.content_identity(),&required_b.content_identity);
    assert_ne!(active_a.content_identity(),future_b.content_identity());
    save(&out.join("active_a_future_b.json"),&json!({"active_a":required_a,"future_b":required_b,"host":selection_b}))?;
    assert_eq!(u11_independent_public_probe::core_host_oracle::snapshot(&active_a)?,active_a_before);
    assert_eq!(u11_independent_public_probe::core_host_oracle::snapshot(&dormant_a)?,dormant_a_before);
    assert_eq!(dormant_a.content_identity(),&required_a.content_identity);
    u11_independent_public_probe::core_host_oracle::verify_generation_switch(&host,&mut active_a,&mut dormant_a,&required_a,&required_b,&out.join("normal_core_generation_switch"))?;
    drop(active_a);drop(dormant_a);drop(future_b);drop(host);drop(issuer);

    // No criterion is supplied on restart: only authenticated exact-context
    // persisted adoption may restore it. Worker-directory byte/membership and
    // mtime equality makes cache-hit reporting independently checkable.
    let worker_before=state(&cache.join("workers"))?;
    let cache_before=state(&cache)?;
    save(&out.join("restart_before_hashes.json"),&cache_before.report())?;
    let restarted=ValidationIssuer::open(&cache,suite.clone(),diagnostic.clone(),pair()?)?;
    let restored_b=restarted.begin(acquire(&b_bytes)?)?;
    passed(&restarted,restored_b,out,"restart_b_exact_hit",true)?;
    worker_before.ensure_unchanged(&state(&cache.join("workers"))?)?;
    cache_before.ensure_unchanged(&state(&cache)?)?;
    let restored_approval=restarted.approval(restored_b)?;
    let restored_host=ApprovedHost::new(Arc::clone(&restarted),restored_approval.clone())?;
    let denied=restored_host.apply(approval_b).err().expect("old process capability owner rejected after restart");
    save(&out.join("previous_owner_capability_denied.json"),&denied)?;
    assert_eq!(denied.kind,ModValidationErrorKind::ForeignIssuer);
    let current_host=serde_json::to_value(restored_host.query()?)?;
    restarted.invalidate_selection()?;
    let denied=restored_host.apply(restored_approval).err().expect("invalidated approval denied");
    save(&out.join("invalidated_capability_denied.json"),&denied)?;
    assert_eq!(denied.kind,ModValidationErrorKind::Stale);
    assert_eq!(serde_json::to_value(restored_host.query()?)?,current_host);
    drop(restored_host);drop(restarted);
    cache_before.ensure_unchanged(&state(&cache)?)?;
    let mut artifacts=Vec::new();
    for entry in fs::read_dir(out)? {let entry=entry?;
        if entry.file_type()?.is_file()&&entry.path().extension().is_some_and(|s|s=="json") {artifacts.push(entry.path());}}
    artifacts.sort();
    save(&out.join("key_absence.json"),&require_secret_absent(&cache,&artifacts,64*1024*1024)?)?;
    save(&out.join("RESULT.json"),&json!({"passed":true,
        "scope":"actual calibrated Core issuer and host lifecycle; normal server/save gameplay is separate",
        "actual_passes":2,"exact_restart_hit":true,"no_worker_rerun_on_hit":true,
        "foreign_and_stale_capabilities_denied":true,"active_a_future_b_identity_preserved":true}))?;
    println!("{}",json!({"passed":true,"output":out}));Ok(())
}
