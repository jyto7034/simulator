//! Deterministic publication-boundary tests over a real calibrated issuer.
//! Run by the sole executor with its reviewed, same-context accepted cache.
use super::*;
use crate::{authoring::CancellationToken,game::data::content_pack::{ContentSelection,InputLimits},
    runtime_safety::RuntimeSafetySettings};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::{Path,PathBuf},sync::Arc};
#[path="../probe/src/file_state.rs"] mod file_state;
#[path="../probe/src/cache_fixture.rs"] mod cache_fixture;
type Result<T>=std::result::Result<T,Box<dyn Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {root:PathBuf,registry_a:PathBuf,registry_b:PathBuf,suite:PathBuf,profile:PathBuf,base_selection:PathBuf,
    plain_worker:PathBuf,instrumented_worker:PathBuf,environment_path:PathBuf,accepted_cache:PathBuf}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewedLimits {review_reference:String,reviewed_calibration_file_sha256:String,
    revision:String,limits:CriterionLimits}
fn save(path:&Path,value:&impl Serialize)->Result<()> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn state(path:&Path)->Result<file_state::FileState>{Ok(file_state::FileState::capture(path,65_536,256*1024*1024)?)}
fn inputs()->Result<(Inputs,PathBuf)>{
    let file=std::env::var_os("U11_CACHE_BOUNDARY_INPUTS").ok_or("reviewed actual cache inputs required")?;
    let out=std::env::var_os("U11_CACHE_BOUNDARY_OUTPUT").ok_or("exclusive cache boundary output parent required")?;
    Ok((serde_json::from_slice(&fs::read(file)?)?,PathBuf::from(out)))
}
fn record_paths(root:&Path,identity:&str)->Result<Vec<PathBuf>> {
    let mut paths=Vec::new();
    for entry in fs::read_dir(root)? {let entry=entry?;
        if !entry.file_name().to_string_lossy().ends_with(".accepted.json"){continue;}
        let envelope:Value=serde_json::from_slice(&fs::read(entry.path())?)?;
        let bytes:Vec<u8>=serde_json::from_value(envelope["payload"].clone())?;
        let record:Value=serde_json::from_slice(&bytes)?;
        if record["result"]["candidate_identity"]==identity{paths.push(entry.path());}
    }Ok(paths)
}
#[test]
#[ignore="explicit management entry; actual calibration/adoption is not a functional pass"]
fn u11_actual_cache_calibration_administration(){
    let run=||->Result<()>{
        use sha2::{Digest,Sha256};
        let(input,out)=inputs()?;fs::create_dir(&out)?;
        let action=std::env::var("U11_CACHE_ADMIN_ACTION")?;
        assert!(action=="calibrate"||action=="adopt_and_accept");
        let suite:LabSuite=serde_json::from_slice(&fs::read(&input.suite)?)?;
        let mut profile:LabProfile=serde_json::from_slice(&fs::read(&input.profile)?)?;
        assert!(profile.criterion.is_none());
        let environment=format!("u11-reviewed-environment-sha256:{:x}",Sha256::digest(fs::read(&input.environment_path)?));
        let pair=WorkerPair::open(input.plain_worker,input.instrumented_worker,environment)?;
        for(index,path)in pair.evidence_paths().iter().enumerate(){
            for(kind,source)in [("stdout",path.clone()),("stderr",path.with_extension("stderr")),("process",path.with_extension("process.json"))]{
                let bytes=fs::read(source)?;assert!(bytes.len()<=65_536);
                OpenOptions::new().write(true).create_new(true).open(out.join(format!("worker_{index}_{kind}.raw")))?.write_all(&bytes)?;
            }
        }
        let exe=std::env::current_exe()?;
        save(&out.join("context.json"),&json!({"actual_executable":exe,
            "executable_sha256":format!("{:x}",Sha256::digest(fs::read(&exe)?)),"pair":pair.identity(),
            "suite":suite,"profile":profile,"management_only":true}))?;
        let settings=Arc::new(RuntimeSafetySettings::from_ron("actual cache management",&profile.runtime_safety_ron)?);
        let base=TrustedBase::acquire(&input.root,serde_json::from_slice::<ContentSelection>(&fs::read(&input.base_selection)?)?,
            &InputLimits::default(),settings,&CancellationToken::default())?;
        let acquire=|path:&Path|->Result<Candidate>{Ok(Candidate::acquire(&input.root,&fs::read(path)?,&base,
            &InputLimits::default(),&CancellationToken::default())?)};
        let a=acquire(&input.registry_a)?;
        let evidence=out.join("workers");
        #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;fs::DirBuilder::new().mode(0o700).create(&evidence)?;}
        #[cfg(not(unix))] {return Err("actual private Unix worker environment required".into());}
        let fresh=LabRun::calibrate(&a,&suite,&profile,&pair,&evidence,&CancellationToken::default());
        if let Err(error)=&fresh{save(&out.join("calibration_error.json"),error)?;}
        let calibration=fresh?;
        save(&out.join("calibration.json"),&json!({"outcome":calibration.outcome(),"raw_evidence_directory":calibration.evidence_directory()}))?;
        if action=="calibrate"{return Ok(());}
        let reviewed:ReviewedLimits=serde_json::from_slice(&fs::read(std::env::var_os("U11_CACHE_REVIEWED_LIMITS").ok_or("reviewed limits required")?)?)?;
        let prior_bytes=fs::read(std::env::var_os("U11_CACHE_REVIEWED_CALIBRATION").ok_or("reviewed calibration required")?)?;
        assert!(!reviewed.review_reference.trim().is_empty());
        assert_eq!(reviewed.reviewed_calibration_file_sha256,format!("{:x}",Sha256::digest(&prior_bytes)));
        let prior:Value=serde_json::from_slice(&prior_bytes)?;let current=serde_json::to_value(calibration.outcome())?;
        for field in ["candidate_identity","suite_sha256","profile_sha256"]{
            assert!(!prior["outcome"][field].is_null());assert_eq!(prior["outcome"][field],current[field]);
        }
        assert_eq!(prior["outcome"]["metrics"]["pair"],current["metrics"]["pair"]);
        let criterion=calibration.adopt(reviewed.revision,reviewed.limits)?;
        save(&out.join("adopted_metadata.json"),&criterion)?;profile.criterion=Some(criterion);
        let issuer=ValidationIssuer::open(&input.accepted_cache,suite,profile,pair)?;
        for(label,candidate)in [("a",a),("b",acquire(&input.registry_b)?) ]{
            let id=issuer.begin(candidate)?;let result=issuer.run(id)?;
            save(&out.join(format!("accepted_{label}.json")),&result)?;
            assert_eq!(result.state,JobState::Passed);assert!(result.outcome.as_ref().unwrap().is_passed());
            let host=ApprovedHost::new(Arc::clone(&issuer),issuer.approval(id)?)?;save(&out.join(format!("host_{label}.json")),&host.query()?)?;
        }
        Ok(())
    };run().unwrap();
}
fn exercise(mode:&str)->Result<()> {
    use sha2::{Digest,Sha256};
    let(input,parent)=inputs()?;let out=parent.join(mode);fs::create_dir(&out)?;
    let original=state(&input.accepted_cache)?;
    let suite:LabSuite=serde_json::from_slice(&fs::read(&input.suite)?)?;
    let profile:LabProfile=serde_json::from_slice(&fs::read(&input.profile)?)?;
    assert!(profile.criterion.is_none(),"only actual issuer-adopted criterion may authorize");
    let environment=format!("u11-reviewed-environment-sha256:{:x}",Sha256::digest(fs::read(&input.environment_path)?));
    let pair=WorkerPair::open(input.plain_worker,input.instrumented_worker,environment)?;
    let settings=Arc::new(RuntimeSafetySettings::from_ron("actual cache boundary profile",&profile.runtime_safety_ron)?);
    let base=TrustedBase::acquire(&input.root,serde_json::from_slice::<ContentSelection>(&fs::read(&input.base_selection)?)?,
        &InputLimits::default(),settings,&CancellationToken::default())?;
    let a=Candidate::acquire(&input.root,&fs::read(&input.registry_a)?,&base,&InputLimits::default(),&CancellationToken::default())?;
    let b_bytes=fs::read(&input.registry_b)?;
    let b=Candidate::acquire(&input.root,&b_bytes,&base,&InputLimits::default(),&CancellationToken::default())?;
    let b_restart=Candidate::acquire(&input.root,&b_bytes,&base,&InputLimits::default(),&CancellationToken::default())?;
    let b_identity=b.view().identity.clone();let b_required=b.required_configuration().clone();
    assert_ne!(a.view().identity,b_identity);
    let cache=out.join("private_issuer");cache_fixture::copy(&input.accepted_cache,&cache)?;
    let copied_b=record_paths(&cache,&b_identity)?;assert!(!copied_b.is_empty(),"actual accepted B source required");
    for path in copied_b {fs::remove_file(path)?;}
    let issuer=ValidationIssuer::open(&cache,suite.clone(),profile.clone(),pair.clone())?;
    let a_id=issuer.begin(a)?;let a_result=issuer.run(a_id)?;
    assert_eq!(a_result.state,JobState::Passed);assert!(a_result.cache_hit);
    let host=ApprovedHost::new(Arc::clone(&issuer),issuer.approval(a_id)?)?;
    let before_host=serde_json::to_value(host.query()?)?;
    let mut prior=Vec::new();
    for entry in fs::read_dir(&cache)? {let entry=entry?;if entry.file_type()?.is_file(){prior.push((entry.path(),state(&entry.path())?));}}
    let b_id=issuer.begin(b)?;
    if mode=="cancel_before_publication"||mode=="stale_before_publication" {
        let owner=Arc::clone(&issuer);let evidence=out.join("actual_complete_lab_before_boundary.json");
        let cancel=mode=="cancel_before_publication";
        let _reset=super::u11_cache_hooks::install_after_lab(move|id,result|{
            assert_eq!(id,b_id);let(outcome,hit)=result.expect("actual complete Lab result required");
            assert!(outcome.is_passed());assert!(!hit,"removed only disposable B record must force actual Lab");
            save(&evidence,outcome).unwrap();
            if cancel{assert_eq!(owner.cancel(id).unwrap(),CancelDisposition::Cancelled);}
            else{owner.invalidate_selection().unwrap();}
        });
        let result=issuer.run(b_id)?;save(&out.join("boundary_result.json"),&result)?;
        assert_eq!(result.state,if cancel{JobState::Cancelled}else{JobState::Stale});
        assert!(issuer.approval(b_id).is_err());assert!(record_paths(&cache,&b_identity)?.is_empty());
        assert_eq!(serde_json::to_value(host.query()?)?,before_host);
    }else{
        let _reset=super::u11_cache_hooks::install_post_rename_fault("independent one-shot directory sync fault after actual accepted rename");
        let result=issuer.run(b_id)?;save(&out.join("accepted_pending.json"),&result)?;
        assert_eq!(result.state,JobState::RecoveryPending);assert!(!result.cache_hit);
        assert!(result.outcome.as_ref().expect("real supervised outcome").is_passed());
        assert!(issuer.approval(b_id).is_err());assert_eq!(serde_json::to_value(host.query()?)?,before_host);
        let accepted=record_paths(&cache,&b_identity)?;assert_eq!(accepted.len(),1);
        let accepted_before=state(&accepted[0])?;
        assert_eq!(issuer.cancel(b_id)?,CancelDisposition::TooLatePublicationAccepted);
        assert_eq!(issuer.query(b_id)?.state,JobState::RecoveryPending);
        if mode=="accepted_stale_recovery" {
            issuer.invalidate_selection()?;
            assert!(issuer.recover(b_id).is_err());assert!(issuer.approval(b_id).is_err());
            assert_eq!(serde_json::to_value(host.query()?)?,before_host);
            accepted_before.ensure_unchanged(&state(&accepted[0])?)?;
            // A restart must release both the installed host's issuer Arc and
            // the direct issuer owner before reopening its exclusive cache.
            drop(host);drop(issuer);
            let restarted=ValidationIssuer::open(&cache,suite,profile,pair)?;
            let new_id=restarted.begin(b_restart)?;let new_result=restarted.run(new_id)?;
            assert!(new_result.cache_hit);assert_eq!(new_result.state,JobState::Passed);
            let restarted_host=ApprovedHost::new(Arc::clone(&restarted),restarted.approval(new_id)?)?;
            assert_eq!(restarted_host.query()?.required,b_required);
            assert_ne!(serde_json::to_value(restarted_host.query()?.required)?,before_host["required"]);
            save(&out.join("fresh_restart_reconfirmed.json"),&new_result)?;
        }else{
            assert_eq!(mode,"accepted_current_recovery");
            let recovered=issuer.recover(b_id)?;assert_eq!(recovered.state,JobState::Passed);
            host.apply(issuer.approval(b_id)?)?;
            assert_ne!(serde_json::to_value(host.query()?)?,before_host);
            save(&out.join("actual_durable_recovery.json"),&recovered)?;
        }
        accepted_before.ensure_unchanged(&state(&accepted[0])?)?;
    }
    for(path,before)in prior{before.ensure_unchanged(&state(&path)?)?;}
    original.ensure_unchanged(&state(&input.accepted_cache)?)?;
    save(&out.join("RESULT.json"),&json!({"passed":true,"mode":mode,"actual_calibrated_lab":true,
        "source_cache_unchanged":true,"prior_accepted_files_unchanged":true}))?;Ok(())
}
#[test]
#[ignore="requires real calibrated issuer cache and exclusive evidence; run explicit U11 selector"]
fn u11_actual_lab_cancel_before_publication(){exercise("cancel_before_publication").unwrap();}
#[test]
#[ignore="requires real calibrated issuer cache and exclusive evidence; run explicit U11 selector"]
fn u11_actual_lab_stale_before_publication(){exercise("stale_before_publication").unwrap();}
#[test]
#[ignore="requires real calibrated issuer cache and exclusive evidence; run explicit U11 selector"]
fn u11_actual_accepted_pending_then_current_recovery(){exercise("accepted_current_recovery").unwrap();}
#[test]
#[ignore="requires real calibrated issuer cache and exclusive evidence; run explicit U11 selector"]
fn u11_actual_accepted_pending_stale_and_restart(){exercise("accepted_stale_recovery").unwrap();}
