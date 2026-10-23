//! Meaningful context changes against an actual authenticated accepted cache.
//! Every miss executes the unchanged full workload; source caches stay untouched.
use game_core::{authoring::CancellationToken,
    game::data::content_pack::{ContentSelection,InputLimits},
    mod_validation::{Candidate,JobState,LabProfile,LabSuite,TrustedBase,ValidationIssuer,WorkerPair},
    runtime_safety::RuntimeSafetySettings};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs,path::Path,sync::Arc};
use u11_independent_public_probe::{cache_fixture, file_state::FileState,
    normal_preparation::save, secret_oracle::require_secret_absent, worker_evidence};
type Result<T> = std::result::Result<T,Box<dyn Error>>;
fn state(path:&Path)->Result<FileState>{Ok(FileState::capture(path,65_536,256*1024*1024)?)}
fn copy_tree(source:&Path,destination:&Path)->Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {let entry=entry?;let kind=entry.file_type()?;
        if kind.is_dir(){copy_tree(&entry.path(),&destination.join(entry.file_name()))?;}
        else if kind.is_file(){fs::copy(entry.path(),destination.join(entry.file_name()))?;}
        else{return Err("only frozen regular fixture members may be copied".into());}
    }Ok(())
}
fn acquire(root:&Path,registry:&[u8],selection:&ContentSelection,profile:&LabProfile)->Result<Candidate>{
    let settings=Arc::new(RuntimeSafetySettings::from_ron("actual cache context",&profile.runtime_safety_ron)?);
    let base=TrustedBase::acquire(root,selection.clone(),&InputLimits::default(),settings,&CancellationToken::default())?;
    Ok(Candidate::acquire(root,registry,&base,&InputLimits::default(),&CancellationToken::default())?)
}
fn exact_hit(cache:&Path,candidate:Candidate,suite:&LabSuite,profile:&LabProfile,pair:&WorkerPair,out:&Path)->Result<()> {
    let before=state(cache)?;
    let issuer=ValidationIssuer::open(cache,suite.clone(),profile.clone(),pair.clone())?;
    let id=issuer.begin(candidate)?;let view=issuer.run(id)?;save(out,&view)?;
    assert_eq!(view.state,JobState::Passed);assert!(view.cache_hit);
    assert!(view.outcome.as_ref().unwrap().is_passed());assert!(issuer.approval(id).is_ok());
    drop(issuer);before.ensure_unchanged(&state(cache)?)?;Ok(())
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=10{return Err("usage: cache_context FROZEN_ROOT A_REGISTRY SUITE DIAGNOSTIC_PROFILE ACTUAL_ENVIRONMENT PLAIN_WORKER INSTRUMENTED_WORKER ORIGINAL_ACCEPTED_CACHE NEW_PRIVATE_OUTPUT TRUSTED_SELECTION_JSON".into());}
    let root=Path::new(&args[0]);let source=Path::new(&args[7]);let out=Path::new(&args[8]);
    cache_fixture::private_directory(out)?;
    let original=state(source)?;let original_fixture=state(root)?;
    let registry=fs::read(&args[1])?;let suite:LabSuite=serde_json::from_slice(&fs::read(&args[2])?)?;
    let profile:LabProfile=serde_json::from_slice(&fs::read(&args[3])?)?;assert!(profile.criterion.is_none());
    let selection:ContentSelection=serde_json::from_slice(&fs::read(&args[9])?)?;
    let environment=format!("u11-reviewed-environment-sha256:{:x}",Sha256::digest(fs::read(&args[4])?));
    let pair=WorkerPair::open(Path::new(&args[5]).to_owned(),Path::new(&args[6]).to_owned(),environment)?;
    worker_evidence::preserve(&pair,&out.join("worker_identity"))?;
    let baseline=acquire(root,&registry,&selection,&profile)?;
    let baseline_view=serde_json::to_value(baseline.view())?;
    let baseline_generation=baseline.generation().identity().clone();
    save(&out.join("actual_inputs.json"),&json!({"candidate":baseline.view(),"suite":suite,"profile":profile,"pair":pair.identity(),"selection":selection}))?;
    let control=out.join("control_private");cache_fixture::copy(source,&control)?;
    exact_hit(&control,baseline,&suite,&profile,&pair,&out.join("unchanged_control_hit.json"))?;
    for label in ["source_comment","profile_revision","runtime_bytes_only","scenario_seed"] {
        let directory=out.join(label);cache_fixture::private_directory(&directory)?;
        let cache=directory.join("private_issuer");cache_fixture::copy(source,&cache)?;
        let mut actual_suite=suite.clone();let mut actual_profile=profile.clone();
        let scratch=directory.join("fixture");
        let actual_root=if label=="source_comment" {
            copy_tree(root,&scratch)?;
            let catalog=scratch.join("a_skills/catalog.ron");
            let mut bytes=fs::read(&catalog)?;bytes.extend_from_slice(b"\n// U11 source-only cache boundary\n");
            fs::write(catalog,bytes)?;&scratch
        }else{root};
        match label {
            "profile_revision"=>{actual_profile.revision.push_str("-cache-context");assert_ne!(actual_profile.revision,profile.revision);},
            "runtime_bytes_only"=>{
                actual_profile.runtime_safety_ron.extend_from_slice(b"\n// U11 exact byte identity only\n");
                let old=RuntimeSafetySettings::from_ron("unchanged finite settings",&profile.runtime_safety_ron)?;
                let new=RuntimeSafetySettings::from_ron("comment-only finite settings",&actual_profile.runtime_safety_ron)?;
                assert_eq!(old.effective_sha256(),new.effective_sha256());assert_ne!(old.raw_sha256(),new.raw_sha256());
                save(&directory.join("runtime_identity.json"),&json!({"before_effective":old.effective_sha256(),"after_effective":new.effective_sha256(),
                    "before_raw":old.raw_sha256(),"after_raw":new.raw_sha256(),"changed_numeric_limit":false}))?;
            },
            "scenario_seed"=>{
                let mut document:Value=serde_json::from_slice(&actual_suite.cases[0].scenario)?;
                let seed=document["initial"]["run_seed"].as_u64().ok_or("literal actual run seed required")?;
                document["initial"]["run_seed"]=json!(seed.checked_add(1).ok_or("seed overflow")?);
                actual_suite.cases[0].scenario=serde_json::to_vec(&document)?;
                assert_ne!(actual_suite.cases[0].scenario,suite.cases[0].scenario);
            },_=>{},
        }
        let candidate=acquire(actual_root,&registry,&selection,&actual_profile)?;
        if label=="source_comment" {
            assert_eq!(candidate.generation().identity(),&baseline_generation);
            assert_ne!(candidate.view().identity,baseline_view["identity"].as_str().unwrap());
            assert_ne!(serde_json::to_value(&candidate.view().source_receipt)?,baseline_view["source_receipt"]);
        }else{assert_eq!(serde_json::to_value(candidate.view())?,baseline_view);}
        save(&directory.join("actual_changed_inputs.json"),&json!({"candidate":candidate.view(),"suite":actual_suite,"profile":actual_profile}))?;
        let protected:Vec<_>=fs::read_dir(&cache)?.map(|entry|entry.map(|e|e.path())).collect::<std::io::Result<Vec<_>>>()?
            .into_iter().filter(|path|path.is_file()).map(|path|state(&path).map(|before|(path,before))).collect::<Result<_>>()?;
        assert_eq!(fs::read_dir(cache.join("workers"))?.count(),0);
        let issuer=ValidationIssuer::open(&cache,actual_suite,actual_profile,pair.clone())?;
        let id=issuer.begin(candidate)?;let result=issuer.run(id)?;save(&directory.join("actual_miss.json"),&result)?;
        assert!(!result.cache_hit);assert!(result.error.is_none());
        let outcome=result.outcome.as_ref().ok_or("actual supervised outcome required for context miss")?;
        assert!(outcome.metrics.is_complete()&&outcome.coverage.is_complete());
        assert_eq!(outcome.metrics.repeat_count,profile.repeats);
        assert_eq!(fs::read_dir(cache.join("workers"))?.count(),1,"miss must launch a new supervised run");
        if label=="source_comment" {
            assert_eq!(result.state,JobState::Passed);assert!(outcome.is_passed());assert!(issuer.approval(id).is_ok());
        }else{
            assert_eq!(result.state,JobState::Incomplete);assert!(!outcome.is_passed());
            assert_eq!(outcome.reasons,vec!["criterion not calibrated/adopted"]);
            assert!(issuer.approval(id).is_err());
        }
        drop(issuer);
        for(path,before)in protected{before.ensure_unchanged(&state(&path)?)?;}
        exact_hit(&cache,acquire(root,&registry,&selection,&profile)?,&suite,&profile,&pair,&directory.join("restored_exact_hit.json"))?;
        exact_hit(&control,acquire(root,&registry,&selection,&profile)?,&suite,&profile,&pair,&directory.join("independent_control_hit.json"))?;
        original.ensure_unchanged(&state(source)?)?;original_fixture.ensure_unchanged(&state(root)?)?;
    }
    let public:Vec<_>=fs::read_dir(out)?.filter_map(|entry|entry.ok()).filter(|entry|entry.file_type().is_ok_and(|t|t.is_file())).map(|entry|entry.path()).collect();
    save(&out.join("key_absence.json"),&require_secret_absent(source,&public,64*1024*1024)?)?;
    save(&out.join("RESULT.json"),&json!({"passed":true,"meaningful_context_misses":4,"source_only_semantics_unchanged":true,
        "numeric_runtime_limits_changed":false,"old_calibration_not_transferred":true,"restored_exact_hits":4,
        "independent_control_hits":5,"original_cache_and_fixture_unchanged":true,"actual_full_workloads_executed":true}))?;
    Ok(())
}
