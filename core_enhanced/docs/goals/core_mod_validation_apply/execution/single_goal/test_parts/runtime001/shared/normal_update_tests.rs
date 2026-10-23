//! Independent normal host orchestration over actual calibrated inputs.
use super::support::*;
use crate::mod_admission::AdmissionReport;
use game_core::mod_validation::JobState;
use serde::Deserialize;
use serde_json::json;
use std::{collections::BTreeMap,fs,path::PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs{initial_a:PathBuf,parse_held:PathBuf,runtime_held:PathBuf,combined_failure:PathBuf,missing_combination:PathBuf,
    unsupported_add:PathBuf,unsupported_replace:PathBuf,unsupported_remove:PathBuf}

fn inventory(report:&AdmissionReport,expected:&[(&str,JobState)]) {
    let actual: BTreeMap<_,_>=report.individuals.iter().map(|r|(r.mod_id.as_str(),r.state)).collect();
    assert_eq!(actual.len(),report.individuals.len(),"no duplicate or hidden individual result");
    assert_eq!(actual,expected.iter().copied().collect(),"all independent selected closures must be checked");
    for row in &report.individuals {
        if row.state==JobState::Passed {
            let job=row.job.as_ref().expect("PASS requires an actual individual job");
            assert!(job.outcome.as_ref().expect("actual individual outcome").is_passed());
        }
        if row.state==JobState::DependencyHeld {assert!(row.job.is_none(),"held dependency cannot invent a Lab run");}
    }
}

#[test]
#[ignore="requires actual reviewed same-executable orchestration calibration inputs and exclusive evidence; run explicit U11 selector"]
fn u11_normal_update_checks_all_individuals_reduces_selection_and_requires_combination(){
    let run=||->Result<()>{
        let inputs:Inputs=serde_json::from_slice(&fs::read(std::env::var_os("U11_NORMAL_UPDATE_CASES")
            .ok_or("actual normal update manifest required")?)?)?;
        let parent=PathBuf::from(std::env::var_os("U11_NORMAL_SAVE_OUTPUT").ok_or("exclusive output parent required")?);
        let out=parent.join("normal_update");fs::create_dir(&out)?;
        let mut configs=[config(&inputs.initial_a)?,config(&inputs.parse_held)?,
            config(&inputs.runtime_held)?,config(&inputs.combined_failure)?,config(&inputs.missing_combination)?,
            config(&inputs.unsupported_add)?,config(&inputs.unsupported_replace)?,config(&inputs.unsupported_remove)?];
        let source=configs[0].issuer_root.clone();
        assert!(configs.iter().all(|c|c.issuer_root==source),"actual accepted context inventory must share one issuer");
        assert_eq!(configs[0].individual_suite_paths,configs[2].individual_suite_paths,
            "frozen-candidate update uses individual contexts captured when its session opened");
        assert_eq!(fs::read(&configs[0].suite_path)?,fs::read(&configs[2].suite_path)?,
            "frozen-candidate update uses the session's unchanged final suite");
        let original=state(&source)?;let private=out.join("private_issuer");cache_fixture::copy(&source,&private)?;
        for c in &mut configs{c.issuer_root=private.clone();}
        let mut admission=open(&configs[0])?;
        let (old_app,old_required)=admission.host.new_application(77)?;
        let old_snapshot=run_snapshot(&old_app)?;
        let initial=admission.query()?;save(&out.join("initial.json"),&initial)?;

        // The malformed pack cannot form a combined Candidate. This exercises
        // configuration orchestration before combination acquisition.
        let parse=admission.validate_configuration_and_apply(&configs[1])?;
        save(&out.join("parse_held.json"),&parse)?;
        inventory(&parse,&[("base",JobState::Passed),("bad_parse",JobState::ContentFailed),
            ("dependent",JobState::DependencyHeld),("b",JobState::Passed)]);
        assert_eq!(parse.held_mods.iter().cloned().collect::<std::collections::BTreeSet<_>>(),
            ["bad_parse".to_string(),"dependent".to_string()].into_iter().collect());
        assert_eq!(parse.selected_mods,vec!["base","b"]);
        let final_job=parse.combination.as_ref().expect("reduced selection needs its own actual final job");
        assert_eq!(final_job.state,JobState::Passed);
        assert_eq!(admission.query()?.candidate_identity,final_job.candidate_identity);
        assert_ne!(admission.query()?.required,initial.required);

        // Frozen candidate path: a legal but uncovered no-op mod is held while
        // independent B is still run, then only the reduced candidate is used.
        let runtime=admission.validate_and_apply_report(acquire(&configs[2])?)?;
        save(&out.join("runtime_held.json"),&runtime)?;
        inventory(&runtime,&[("base",JobState::Passed),("noop",JobState::Incomplete),
            ("dependent",JobState::DependencyHeld),("b",JobState::Passed)]);
        assert_eq!(runtime.selected_mods,vec!["base","b"]);
        let final_job=runtime.combination.as_ref().expect("runtime-held selection must rerun final validation");
        assert_eq!(final_job.state,JobState::Passed);
        assert_eq!(admission.query()?.candidate_identity,final_job.candidate_identity);

        // The last independently passed B is not a substitute for a final job.
        // Right is a real winning writer with a valid unconsumed numeric relic;
        // final reduction must preserve its stale authored winner instruction.
        let before_missing=serde_json::to_value(admission.query()?)?;
        let wrapper=admission.validate_and_apply(acquire(&configs[4])?);
        let error=wrapper.err().ok_or("no final combination was disguised as a passed individual JobView")?;
        save(&out.join("missing_combination_wrapper.json"),&json!({"error":error,"report":admission.report}))?;
        inventory(&admission.report,&[("base",JobState::Passed),("a",JobState::Passed),
            ("left",JobState::Passed),("right",JobState::Incomplete),("b",JobState::Passed)]);
        let right=admission.report.individuals.iter().find(|row|row.mod_id=="right").unwrap();
        let right_job=right.job.as_ref().expect("valid right publication must reach an actual Lab job");
        let right_outcome=right_job.outcome.as_ref().expect("runtime coverage hold requires actual worker outcome");
        assert!(right_outcome.metrics.is_complete());assert!(!right_outcome.coverage.is_complete());
        let missing:Vec<_>=right_outcome.coverage.rows.iter().filter(|row|row.requirement.key.definition_kind=="Relic"
            &&row.requirement.key.definition_id=="u11_unconsumed_numeric_relic").collect();
        assert_eq!(missing.len(),1);assert_eq!(missing[0].requirement.key.action_id,"positive_flat");
        assert_eq!(missing[0].requirement.required_stage,game_core::mod_validation::CoverageStage::ConsumedOrEffect);
        assert_eq!(missing[0].counts[game_core::mod_validation::CoverageStage::ConsumedOrEffect as usize],0);
        assert_eq!(admission.report.held_mods,vec!["right"]);
        assert_eq!(admission.report.selected_mods,vec!["base","a","left","b"]);
        assert!(admission.report.combination.is_none());assert!(admission.report.combination_error.is_some());
        assert_eq!(serde_json::to_value(admission.query()?)?,before_missing);

        for(index,label)in [(5,"add"),(6,"replace"),(7,"remove")] {
            let prior=state(&private)?;
            let report=admission.validate_configuration_and_apply(&configs[index])?;
            save(&out.join(format!("unsupported_resource_{label}.json")),&report)?;
            inventory(&report,&[("base",JobState::Passed),("change",JobState::Incomplete)]);
            assert_eq!(report.held_mods,vec!["change"]);assert_eq!(report.selected_mods,vec!["base"]);
            let held=report.individuals.iter().find(|r|r.mod_id=="change").unwrap().job.as_ref()
                .expect("valid unsupported catalog must reach actual supervised validation");
            let outcome=held.outcome.as_ref().expect("actual unsupported coverage result");
            assert!(!outcome.is_passed());assert!(outcome.coverage.unsupported.iter()
                .any(|s|s.contains("Resource:u11_unsupported_resource")));
            assert!(admission.issuer.approval(held.id).is_err());
            assert_eq!(report.combination.as_ref().expect("remaining base independently revalidated").state,JobState::Passed);
            prior.ensure_original_files_unchanged(&state(&private)?)?;
            assert_eq!(run_snapshot(&old_app)?,old_snapshot);
        }

        // Root-reviewed actual criterion must distinguish individually accepted
        // load closures from an over-budget combination. Generic incompleteness
        // cannot satisfy this performance-failure oracle.
        let before=serde_json::to_value(admission.query()?)?;
        let combined=admission.validate_configuration_and_apply(&configs[3])?;
        save(&out.join("combined_failure.json"),&combined)?;
        inventory(&combined,&[("base",JobState::Passed),("load_a",JobState::Passed),("load_b",JobState::Passed)]);
        assert!(combined.held_mods.is_empty());
        let failed=combined.combination.as_ref().expect("both individual PASS results cannot skip the final job");
        assert_eq!(failed.state,JobState::ContentFailed);
        let outcome=failed.outcome.as_ref().expect("actual complete combined failure outcome");
        assert!(!outcome.is_passed());assert!(outcome.metrics.is_complete());assert!(outcome.coverage.is_complete());
        assert!(admission.issuer.approval(failed.id).is_err());
        assert_eq!(serde_json::to_value(admission.query()?)?,before,"failed combination cannot replace installed host");
        assert_eq!(old_app.content_identity(),&old_required.content_identity);
        assert_eq!(run_snapshot(&old_app)?,old_snapshot,"all update paths preserve an existing A application");
        original.ensure_unchanged(&state(&source)?)?;
        save(&out.join("RESULT.json"),&json!({"passed":true,"actual_normal_configuration_update":true,
            "actual_frozen_candidate_update":true,"independent_and_final_jobs_required":true}))?;
        Ok(())
    };run().unwrap();
}
