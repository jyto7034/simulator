//! Legal empty Skill and unconsumed numeric Relic cannot vanish from obligations.
//! An empty Relic is separately retained as an invalid publication negative.
//! Distinct from the actual legal-noop producer tests in bulk_coverage.
use game_core::{authoring::CancellationToken,game::data::content_pack::{ContentSelection,InputLimits},
    mod_validation::{Candidate,TrustedBase,ModValidationErrorKind,CoverageStage}};
use serde_json::json;
use std::{error::Error,fs,path::Path};
use u11_independent_public_probe::normal_preparation as normal;
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=2&&args.len()!=3{return Err("usage: zero_site_coverage FROZEN_LAB_ROOT NEW_OUTPUT [RUNTIME_SAFETY_RON]".into());}
    let root=Path::new(&args[0]);let out=Path::new(&args[1]);fs::create_dir(out)?;
    let (settings,_profile_bytes)=normal::runtime_profile(args.get(2).map(|p|Path::new(p)),&out.join("runtime_context.json"))?;
    let base=TrustedBase::acquire(root,ContentSelection{packs:vec!["official".into()],winners:vec![]},
        &InputLimits::default(),settings.clone(),&CancellationToken::default())?;
    let control=Candidate::acquire(root,&fs::read(root.join("registries/base.json"))?,&base,&InputLimits::default(),&CancellationToken::default())?;
    let scenario=normal::prefix(&control,&out.join("normal_prefix"))?;
    let mut recipe=normal::recipe("host_load",1)?;
    normal::place_one(&control,&scenario,&mut recipe,&out.join("normal_placement.json"))?;
    let normal_control=normal::prepare(&control,&scenario,&recipe)?;
    assert_eq!(normal_control.lab_preparation().unwrap().deployed_count,1);
    normal::save(&out.join("real_normal_control.json"),&normal_control.lab_coverage()?)?;
    let empty=Candidate::acquire(root,&fs::read(root.join("registries/zero_site_relic.json"))?,&base,
        &InputLimits::default(),&CancellationToken::default()).err().ok_or("invalid empty relic published")?;
    normal::save(&out.join("invalid_empty_relic.json"),&empty)?;
    assert_eq!(empty.kind,ModValidationErrorKind::ContentFailed);assert_eq!(empty.field_path,"candidate");
    assert!(empty.reason.contains("effects/rules")&&empty.reason.contains("at least one numeric effect or structural rule is required"));
    for(label,kind,id)in [("zero_site_skill","Skill","u11_empty_step"),
        ("unconsumed_numeric_relic","Relic","u11_unconsumed_numeric_relic")] {
        let candidate=Candidate::acquire(root,&fs::read(root.join(format!("registries/{label}.json")))?,&base,&InputLimits::default(),&CancellationToken::default())?;
        let changed=serde_json::to_value(&candidate.view().changed_definitions)?;
        assert_eq!(changed,json!([{"kind":kind,"id":id}]),"literal single changed definition fixture");
        match normal::prepare(&candidate,&scenario,&recipe) {
            Ok(trial)=>{
                let report=trial.lab_coverage()?;normal::save(&out.join(format!("{label}.json")),&json!({"coverage":report,"candidate":candidate.view()}))?;
                assert!(!report.is_complete(),"unexercised changed zero-site definition must not vanish into empty complete report");
                assert!(!report.rows.is_empty()||!report.unsupported.is_empty(),"actual obligation or explicit unsupported reason required");
            }
            Err(error)=>{
                let text=error.to_string();normal::save(&out.join(format!("{label}.json")),&json!({"candidate":candidate.view(),"preparation_error":text}))?;
                assert!(text.to_ascii_lowercase().contains("coverage")||text.to_ascii_lowercase().contains("unsupported"),
                    "the valid normal control excludes arbitrary setup failures as this regression's success");
            }
        }
    }
    let right=Candidate::acquire_individual(root,&fs::read(root.join("registries/update_missing_combination.json"))?,
        "right",&base,&InputLimits::default(),&CancellationToken::default())?;
    normal::save(&out.join("actual_right_publication.json"),right.view())?;
    let right_trial=normal::prepare(&right,&scenario,&recipe)?;
    let right_report=right_trial.lab_coverage()?;
    normal::save(&out.join("actual_right_unconsumed_coverage.json"),&right_report)?;
    assert!(!right_report.is_complete());
    let missing:Vec<_>=right_report.rows.iter().filter(|row|row.requirement.key.definition_kind=="Relic"
        &&row.requirement.key.definition_id=="u11_unconsumed_numeric_relic").collect();
    assert_eq!(missing.len(),1);assert_eq!(missing[0].requirement.key.action_id,"positive_flat");
    assert_eq!(missing[0].requirement.required_stage,CoverageStage::ConsumedOrEffect);
    assert_eq!(missing[0].counts[CoverageStage::ConsumedOrEffect as usize],0);
    normal::save(&out.join("RESULT.json"),&json!({"passed":true,"zero_site_not_vacuous_complete":true,
        "invalid_empty_relic_rejected":true,"valid_right_publication_unconsumed_relic":true,"approval_claimed":false}))?;Ok(())
}
