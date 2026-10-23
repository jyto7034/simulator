//! Actual calibrated issuer ownership, independent trial failure and last pins.
//! Included below game::data solely to observe its existing private token Arc.
use crate::{application::{ApplicationCommand,ApplicationSnapshotKind,CoreApplication},
    authoring::{CancellationToken,ScenarioLimits,TrialLimits,TrialScenario,WorldLabPreparation},
    game::data::content_pack::{ContentSelection,InputLimits},
    mod_validation::{ApprovedHost,Candidate,CancelDisposition,JobState,LabProfile,LabSuite,TrustedBase,ValidationIssuer,WorkerPair},
    runtime_safety::RuntimeSafetySettings};
use serde::Deserialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs,path::{Path,PathBuf},sync::Arc};
#[path="../probe/src/file_state.rs"] mod file_state;
#[path="../probe/src/cache_fixture.rs"] mod cache_fixture;
type Result<T> = std::result::Result<T,Box<dyn Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {root:PathBuf,registry_a:PathBuf,registry_b:PathBuf,suite:PathBuf,profile:PathBuf,base_selection:PathBuf,
    plain_worker:PathBuf,instrumented_worker:PathBuf,environment_path:PathBuf,accepted_cache:PathBuf}
fn state(path:&Path)->Result<file_state::FileState>{Ok(file_state::FileState::capture(path,65_536,256*1024*1024)?)}
fn save(path:&Path,value:&impl serde::Serialize)->Result<()> {
    use std::io::Write;
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    fs::OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn run_snapshot(app:&CoreApplication)->Result<Value>{
    Ok(app.snapshot(ApplicationSnapshotKind::Run).map_err(|e|format!("{e:?}"))?.read_model().as_value().clone())
}
fn start(app:&mut CoreApplication,player:uuid::Uuid)->Result<()> {
    let command=ApplicationCommand::from_document(json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false}).into())
        .map_err(|e|format!("{e:?}"))?;
    app.execute(player,command,Default::default()).map_err(|e|format!("{e:?}"))?;Ok(())
}
#[test]
#[ignore="requires actual reviewed matching Core accepted cache and exclusive evidence"]
fn u11_actual_issuer_approval_host_trial_last_pins_release_independently(){
    let run=||->Result<()>{
        let input:Inputs=serde_json::from_slice(&fs::read(std::env::var_os("U11_CACHE_BOUNDARY_INPUTS").ok_or("actual cache inputs required")?)?)?;
        let parent=PathBuf::from(std::env::var_os("U11_CACHE_BOUNDARY_OUTPUT").ok_or("exclusive evidence parent required")?);
        let out=parent.join("issuer_generation_lifetime");cache_fixture::private_directory(&out)?;
        let original=state(&input.accepted_cache)?;
        let suite:LabSuite=serde_json::from_slice(&fs::read(&input.suite)?)?;
        let profile:LabProfile=serde_json::from_slice(&fs::read(&input.profile)?)?;assert!(profile.criterion.is_none());
        let pair=WorkerPair::open(input.plain_worker,input.instrumented_worker,
            format!("u11-reviewed-environment-sha256:{:x}",Sha256::digest(fs::read(&input.environment_path)?)))?;
        for(index,path)in pair.evidence_paths().iter().enumerate(){
            for(kind,source)in [("stdout",path.clone()),("stderr",path.with_extension("stderr")),("process",path.with_extension("process.json"))]{
                use std::io::Write;
                let bytes=fs::read(source)?;assert!(bytes.len()<=65_536);
                fs::OpenOptions::new().write(true).create_new(true).open(out.join(format!("worker_{index}_{kind}.raw")))?.write_all(&bytes)?;
            }
        }
        let selection:ContentSelection=serde_json::from_slice(&fs::read(&input.base_selection)?)?;
        let settings=Arc::new(RuntimeSafetySettings::from_ron("actual lifetime profile",&profile.runtime_safety_ron)?);
        let base=TrustedBase::acquire(&input.root,selection,&InputLimits::default(),settings,&CancellationToken::default())?;
        let acquire=|path:&Path|->Result<Candidate>{Ok(Candidate::acquire(&input.root,&fs::read(path)?,&base,&InputLimits::default(),&CancellationToken::default())?)};
        let a=acquire(&input.registry_a)?;let weak_a=Arc::downgrade(&a.generation().0);
        let b=acquire(&input.registry_b)?;let weak_b=Arc::downgrade(&b.generation().0);
        assert_ne!(a.view().identity,b.view().identity);
        let cache=out.join("private_issuer");cache_fixture::copy(&input.accepted_cache,&cache)?;
        let cache_before=state(&cache)?;
        let issuer=ValidationIssuer::open(&cache,suite.clone(),profile.clone(),pair.clone())?;
        let a_id=issuer.begin(a)?;let accepted=issuer.run(a_id)?;
        assert_eq!(accepted.state,JobState::Passed);assert!(accepted.cache_hit);assert!(accepted.outcome.as_ref().unwrap().is_passed());
        save(&out.join("actual_a_hit.json"),&accepted)?;
        let approval=issuer.approval(a_id)?;let approval_clone=approval.clone();
        let host=ApprovedHost::new(Arc::clone(&issuer),approval.clone())?;
        let (mut active_a,required)=host.new_application(77)?;
        let case=suite.cases.first().ok_or("actual normal case required")?;
        let document:Value=serde_json::from_slice(&case.scenario)?;
        let player:uuid::Uuid=serde_json::from_value(document["initial"]["player_id"].clone())?;
        start(&mut active_a,player)?;let before_run=run_snapshot(&active_a)?;
        let mut fork=active_a.fork_for_authoring().map_err(|e|format!("{e:?}"))?;
        let vn=before_run.get("visual_novel").filter(|v|v.is_object()).ok_or("actual pre-run briefing required")?;
        let choice=vn["choices"].as_array().ok_or("actual choice inventory required")?.iter()
            .find(|choice|choice["selectable"]==true);
        let mut next=json!({"type":if choice.is_some(){"select_visual_novel_choice"}else{"advance_visual_novel"},
            "session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"]});
        if let Some(choice)=choice{next["choice_id"]=choice["choice_id"].clone();}
        let command=ApplicationCommand::from_document(next.clone().into()).map_err(|e|format!("{e:?}"))?;
        let result=fork.execute(player,command,Default::default()).map_err(|e|format!("{e:?}"))?;
        save(&out.join("fork_independent_command.json"),&json!({"command":next,"result":result}))?;
        assert_ne!(run_snapshot(&fork)?,before_run);assert_eq!(run_snapshot(&active_a)?,before_run);
        let host_before=serde_json::to_value(host.query()?)?;let job_before=serde_json::to_value(issuer.query(a_id)?)?;
        let recipe=case.preparation.clone().ok_or("actual public World preparation required")?;
        let scenario=||TrialScenario::from_json_slice(&case.scenario,&ScenarioLimits::default());
        let mut limits=TrialLimits::with_runtime_safety(b.generation().runtime_safety());limits.coverage=Some(b.coverage_plan()?);
        let mut trial=WorldLabPreparation::prepare(b.authoring_content(),scenario()?,recipe.clone(),limits,&CancellationToken::default())?;
        trial.advance(&CancellationToken::default())?;
        let before_trial=json!({"work":trial.accepted_work(),"index":trial.next_step_index(),"coverage":trial.lab_coverage()?,
            "receipts":trial.receipt_documents().collect::<Vec<_>>()});
        let cancelled=CancellationToken::default();cancelled.cancel();
        let error=trial.advance(&cancelled).err().ok_or("cancelled actual trial step must refuse")?;
        save(&out.join("cancelled_trial_step.json"),&error)?;
        assert!(matches!(error.error(),crate::authoring::AuthoringError::Execution{diagnostic} if diagnostic.code=="authoring_cancelled"));
        drop(error);
        assert_eq!(json!({"work":trial.accepted_work(),"index":trial.next_step_index(),"coverage":trial.lab_coverage()?,
            "receipts":trial.receipt_documents().collect::<Vec<_>>()}),before_trial);
        let failed=acquire(&input.registry_b)?;let weak_failed=Arc::downgrade(&failed.generation().0);
        let mut fail_limits=TrialLimits::with_runtime_safety(failed.generation().runtime_safety());
        fail_limits.execution.max_work=1;fail_limits.coverage=Some(failed.coverage_plan()?);
        let failure=WorldLabPreparation::prepare(failed.authoring_content(),scenario()?,recipe,fail_limits,&CancellationToken::default())
            .err().ok_or("actual finite preparation work failure required")?;
        save(&out.join("failed_trial_preparation.json"),&failure)?;
        assert!(matches!(failure.error(),crate::authoring::AuthoringError::Execution{diagnostic} if diagnostic.code=="authoring_work_limit"));
        drop(failure);drop(failed);
        assert!(weak_failed.upgrade().is_none(),"failed preparation must retain no hidden generation pin");
        let mut other_profile=profile.clone();other_profile.revision.push_str("-independent-lifetime");
        let other=ValidationIssuer::open(&out.join("independent_issuer"),suite.clone(),other_profile,pair.clone())?;
        let b_id=other.begin(b)?;assert_eq!(other.cancel(b_id)?,CancelDisposition::Cancelled);
        assert_eq!(other.query(b_id)?.state,JobState::Cancelled);assert!(other.approval(b_id).is_err());
        assert_eq!(serde_json::to_value(issuer.query(a_id)?)?,job_before);assert!(issuer.approval(a_id).is_ok());
        assert_eq!(serde_json::to_value(host.query()?)?,host_before);assert_eq!(run_snapshot(&active_a)?,before_run);
        assert_eq!(json!({"work":trial.accepted_work(),"index":trial.next_step_index(),"coverage":trial.lab_coverage()?,
            "receipts":trial.receipt_documents().collect::<Vec<_>>()}),before_trial);
        let prior_index=trial.next_step_index();trial.advance(&CancellationToken::default())?;
        assert_eq!(trial.next_step_index(),prior_index+1,"independent trial resumes after refusal and other-owner cancellation");
        drop(other);assert!(weak_b.upgrade().is_some(),"the live trial is a legitimate final B consumer");
        drop(trial);assert!(weak_b.upgrade().is_none(),"cancelled issuer and trial release B after their last pins");
        assert_eq!(active_a.content_identity(),&required.content_identity);
        drop(host);drop(issuer);assert!(weak_a.upgrade().is_some());
        drop(approval);assert!(weak_a.upgrade().is_some());
        drop(approval_clone);assert!(weak_a.upgrade().is_some());
        drop(active_a);assert!(weak_a.upgrade().is_some(),"independent fork remains a legitimate A consumer");
        drop(fork);assert!(weak_a.upgrade().is_none(),"cache and result DTOs must not pin the old generation");
        // A retained successful JobView is metadata, not a generation owner.
        assert_eq!(accepted.state,JobState::Passed);cache_before.ensure_unchanged(&state(&cache)?)?;
        let fresh=acquire(&input.registry_a)?;let weak_fresh=Arc::downgrade(&fresh.generation().0);
        assert!(weak_a.upgrade().is_none());
        let reopened=ValidationIssuer::open(&cache,suite,profile,pair)?;
        let id=reopened.begin(fresh)?;let result=reopened.run(id)?;
        assert_eq!(result.state,JobState::Passed);assert!(result.cache_hit);assert!(reopened.approval(id).is_ok());
        save(&out.join("reopened_new_generation_hit.json"),&result)?;
        assert!(weak_fresh.upgrade().is_some());assert!(weak_a.upgrade().is_none());
        drop(reopened);assert!(weak_fresh.upgrade().is_none());
        cache_before.ensure_unchanged(&state(&cache)?)?;original.ensure_unchanged(&state(&input.accepted_cache)?)?;
        save(&out.join("RESULT.json"),&json!({"passed":true,"actual_authenticated_hits":2,
            "independent_issuer_cancel_and_trial_failures":true,"mutable_fork_independent":true,
            "old_generations_reclaimed_after_last_consumers":true,"disk_cache_reopens_with_new_caller_generation":true,
            "original_cache_unchanged":true,"per_owner_heap_allocation_claimed":false}))?;
        Ok(())
    };run().unwrap();
}
