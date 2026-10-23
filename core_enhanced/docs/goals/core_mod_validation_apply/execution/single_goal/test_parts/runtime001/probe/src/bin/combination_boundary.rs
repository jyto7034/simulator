//! Real individual passes and combined calibrated performance failure.
//! No test threshold is chosen here: exact Root-reviewed limits are required.
use game_core::{authoring::CancellationToken,
    game::data::content_pack::InputLimits,
    mod_validation::{ApprovedHost,Candidate,JobState,LabProfile,LabSuite,TrustedBase,ValidationIssuer,WorkerPair},
    runtime_safety::RuntimeSafetySettings};
use serde::Serialize;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs::{self,OpenOptions},io::Write,path::Path,sync::Arc};
use u11_independent_public_probe::{cache_fixture,file_state::FileState,secret_oracle::require_secret_absent};
fn save(path:&Path,value:&impl Serialize)->Result<(),Box<dyn Error>> {
    let mut bytes=serde_json::to_vec_pretty(value)?;bytes.push(b'\n');
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(&bytes)?;Ok(())
}
fn state(path:&Path)->std::io::Result<FileState>{FileState::capture(path,65_536,256*1024*1024)}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=12&&args.len()!=13 {return Err("usage: combination_boundary FROZEN_ROOT A_REGISTRY B_REGISTRY COMBINED_REGISTRY SUITE DIAGNOSTIC_PROFILE ACTUAL_ENVIRONMENT PLAIN_WORKER INSTRUMENTED_WORKER REVIEWED_LIMITS ORIGINAL_ACCEPTED_CACHE NEW_OUTPUT [TRUSTED_SELECTION_JSON]".into());}
    let root=Path::new(&args[0]);let source=Path::new(&args[10]);let out=Path::new(&args[11]);cache_fixture::private_directory(out)?;
    let original=state(source)?;
    let suite:LabSuite=serde_json::from_slice(&fs::read(&args[4])?)?;
    let profile:LabProfile=serde_json::from_slice(&fs::read(&args[5])?)?;assert!(profile.criterion.is_none());
    let reviewed:Value=serde_json::from_slice(&fs::read(&args[9])?)?;
    assert!(reviewed["review_reference"].as_str().is_some_and(|s|!s.trim().is_empty()));
    let mut calibration_files=Vec::new();
    for entry in fs::read_dir(source)? {let entry=entry?;
        if entry.file_name().to_string_lossy().ends_with(".criterion.json") {calibration_files.push(entry.path());}}
    assert_eq!(calibration_files.len(),1);
    let envelope:Value=serde_json::from_slice(&fs::read(&calibration_files[0])?)?;
    let payload:Vec<u8>=serde_json::from_value(envelope["payload"].clone())?;
    let adopted:Value=serde_json::from_slice(&payload)?;
    assert_eq!(adopted["criterion"]["limits"],reviewed["limits"],"independent numeric oracle must use the exact adopted limits");
    assert_eq!(adopted["criterion"]["revision"],reviewed["revision"]);
    let environment=format!("u11-reviewed-environment-sha256:{:x}",Sha256::digest(fs::read(&args[6])?));
    let pair=WorkerPair::open(Path::new(&args[7]).to_owned(),Path::new(&args[8]).to_owned(),environment)?;
    u11_independent_public_probe::worker_evidence::preserve(&pair,&out.join("worker_identity"))?;
    let settings=Arc::new(RuntimeSafetySettings::from_ron("combined actual profile",&profile.runtime_safety_ron)?);
    let selection=u11_independent_public_probe::normal_preparation::trusted_selection(
        args.get(12).map(|path|Path::new(path)),&out.join("trusted_base_context.json"))?;
    let base=TrustedBase::acquire(root,selection,
        &InputLimits::default(),settings,&CancellationToken::default())?;
    let acquire=|index:usize|Candidate::acquire(root,&fs::read(&args[index]).unwrap(),&base,&InputLimits::default(),&CancellationToken::default());
    let a=acquire(1)?;let b=acquire(2)?;let combined=acquire(3)?;
    assert_ne!(a.view().identity,b.view().identity);assert_ne!(combined.view().identity,a.view().identity);assert_ne!(combined.view().identity,b.view().identity);
    let mods=|candidate:&Candidate|candidate.required_configuration().mods.iter()
        .map(|m|m.mod_id.clone()).collect::<std::collections::BTreeSet<_>>();
    let a_mods=mods(&a);let b_mods=mods(&b);let combined_mods=mods(&combined);
    assert_eq!(combined_mods,a_mods.union(&b_mods).cloned().collect(),"combined selection must be the exact individual union");
    let a_only:Vec<_>=a_mods.difference(&b_mods).collect();let b_only:Vec<_>=b_mods.difference(&a_mods).collect();
    assert_eq!(a_only.len(),1);assert_eq!(b_only.len(),1);
    assert_eq!(combined.individual(a_only[0],&base,&CancellationToken::default())?.required_configuration(),a.required_configuration());
    assert_eq!(combined.individual(b_only[0],&base,&CancellationToken::default())?.required_configuration(),b.required_configuration());
    let definitions=|candidate:&Candidate|candidate.view().changed_definitions.iter()
        .map(|k|format!("{:?}:{}",k.kind,k.id)).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(definitions(&combined),definitions(&a).union(&definitions(&b)).cloned().collect(),
        "combined actual changed definitions must be the exact individual union");
    save(&out.join("candidate_views.json"),&json!({"a":a.view(),"b":b.view(),"combined":combined.view(),"reviewed_limits":reviewed}))?;
    let cache=out.join("disposable_private");cache_fixture::copy(source,&cache)?;
    let issuer=ValidationIssuer::open(&cache,suite,profile,pair)?;
    let mut last=None;
    for (label,candidate) in [("individual_a",a),("individual_b",b)] {
        let id=issuer.begin(candidate)?;let before=state(&cache)?;
        let result=issuer.run(id)?;save(&out.join(format!("{label}.json")),&result)?;
        assert_eq!(result.state,JobState::Passed);assert!(result.cache_hit);
        assert!(result.outcome.as_ref().expect("actual individual result").is_passed());
        before.ensure_unchanged(&state(&cache)?)?;last=Some(issuer.approval(id)?);
    }
    let host=ApprovedHost::new(Arc::clone(&issuer),last.unwrap())?;
    let selected=serde_json::to_value(host.query()?)?;
    let mut protected=Vec::new();for entry in fs::read_dir(&cache)? {let entry=entry?;
        if entry.file_type()?.is_file(){protected.push((entry.path(),state(&entry.path())?));}}
    let id=issuer.begin(combined)?;let result=issuer.run(id)?;
    save(&out.join("combined_actual_result.json"),&result)?;
    assert_eq!(result.state,JobState::ContentFailed);assert!(!result.cache_hit);
    let outcome=result.outcome.as_ref().expect("actual complete combined result");
    assert!(outcome.metrics.is_complete()&&outcome.coverage.is_complete(),
        "missing witnesses or incomplete measurement cannot stand in for actual combined performance failure");
    let limits=&reviewed["limits"];
    let mut exceeded=Vec::new();
    for (instrumented,samples) in [(false,&outcome.metrics.plain),(true,&outcome.metrics.instrumented)] {
        for sample in samples {
            let mut metrics=vec![("elapsed_ns",sample.elapsed_ns,"max_elapsed_ns"),
                ("native_work",sample.native_work,"max_native_work")];
            if instrumented {metrics.push(("requested_heap_live_bytes",sample.requested_heap_live_bytes.unwrap(),"max_requested_heap_bytes"));
                metrics.push(("requested_heap_peak_bytes",sample.requested_heap_peak_bytes.unwrap(),"max_requested_heap_bytes"));
                metrics.push(("rss_current_bytes",sample.rss_bytes.unwrap(),"max_rss_bytes"));
                metrics.push(("rss_process_lifetime_high_water_bytes",sample.rss_lifetime_peak_bytes.unwrap(),"max_rss_bytes"));}
            for (metric,actual,limit_name) in metrics {let limit=limits[limit_name].as_u64().expect("reviewed finite limit");
                assert!(limit>0&&limit<u64::MAX);
                if actual>limit {exceeded.push(json!({"instrumented":instrumented,"repeat":sample.repeat_index,
                    "phase":sample.phase,"metric":metric,"actual":actual,"limit":limit}));}}
        }
    }
    save(&out.join("independent_actual_exceedances.json"),&exceeded)?;
    assert!(!exceeded.is_empty(),"real measured limit exceedance required; no artificial failure label");
    assert!(issuer.approval(id).is_err());assert_eq!(serde_json::to_value(host.query()?)?,selected);
    for (path,before) in &protected {before.ensure_unchanged(&state(path)?)?;}
    save(&out.join("private_after_hashes.json"),&state(&cache)?.report())?;
    drop(host);drop(issuer);original.ensure_unchanged(&state(source)?)?;
    let mut artifacts=Vec::new();for entry in fs::read_dir(out)? {let entry=entry?;if entry.file_type()?.is_file(){artifacts.push(entry.path());}}
    save(&out.join("key_absence.json"),&require_secret_absent(source,&artifacts,64*1024*1024)?)?;
    save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"actual individual PASS and combined calibrated limit FAIL",
        "individual_actual_cached_passes":2,"combined_actual_fresh_failure":true,
        "combined_measurement_and_coverage_complete":true,"new_approval_issued":false,
        "prior_host_unchanged":true,"original_source_cache_unchanged":true}))?;
    println!("{}",json!({"passed":true,"output":out}));Ok(())
}
