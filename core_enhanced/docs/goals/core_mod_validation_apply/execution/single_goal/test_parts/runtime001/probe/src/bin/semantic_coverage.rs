//! Literal support inventory and actual nonbattle/removal consumer diagnostics.
use game_core::{application::ApplicationSnapshotKind,authoring::{CancellationToken,LabRelicLoadout,
    TrialLimits,TrialPosition,WorldLabPreparation},game::data::content_pack::{ContentSelection,DefinitionKey,DefinitionKind,InputLimits},
    mod_validation::{coverage_support,run_driver,Candidate,CoverageStage,LabSuite,ModValidationErrorKind,TrustedBase}};
use serde_json::{json,Value};
use std::{collections::BTreeSet,error::Error,fs,path::Path};
use u11_independent_public_probe::{file_state::FileState,normal_preparation as normal};
type Result<T>=std::result::Result<T,Box<dyn Error>>;
const KINDS:[&str;45]=["ShopPreset","Shop","ShopPool","Reward","RewardPool","EquipmentRewardPool","Event",
    "VisualNovel","VisualNovelTrigger","BossOmenChain","BossPattern","BossActorBehavior","BossEncounter","BossSignal",
    "Resource","SummonedUnit","IndependentObject","Abnormality","CorrodedRangePreset","CorrodedProfile","CorrodedWavePreset",
    "Employee","Equipment","EquipmentMaterial","DismantleRecipe","EnhancementRecipe","Consumable","Buff","SkillRangePreset",
    "Skill","SkillFragment","Relic","PveEncounter","PveRewardSource","RunPolicy","MapNode","FacilityTags","FacilityTemplate",
    "FacilityPool","NodeAssignmentProfile","StandardSchedule","EndlessSchedule","BattlefieldSize","BattlefieldArchetype","BattlefieldTemplate"];
const ADD_REPLACE:[&str;9]=["Employee","Equipment","SkillFragment","Relic","PveEncounter","RunPolicy","Skill","Buff","IndependentObject"];
const REMOVE:[&str;5]=["Employee","Equipment","SkillFragment","Relic","PveEncounter"];
fn acquire(root:&Path,name:&str,base:&TrustedBase)->Result<Candidate>{Ok(Candidate::acquire(root,&fs::read(root.join(format!("registries/{name}.json")))?,base,&InputLimits::default(),&CancellationToken::default())?)}
fn suite(label:&str,document:&Value,recipe:&game_core::authoring::LabPreparation,refs:Value)->Result<LabSuite>{
    Ok(serde_json::from_value(json!({"revision":"u11-independent-semantic-v1","cases":[{"id":label,"class":"normal",
        "scenario":serde_json::to_vec(document)?,"preparation":recipe,"removed_references":refs}]}))?)
}
fn incomplete(result:std::result::Result<game_core::mod_validation::DriverReport,game_core::mod_validation::ModValidationError>,out:&Path)->Result<()> {
    match result{Ok(report)=>{normal::save(out,&report)?;assert!(!report.coverage.is_complete(),"insufficient removal proof cannot become complete");},
        Err(error)=>{normal::save(out,&error)?;assert_eq!(error.kind,ModValidationErrorKind::Incomplete);}}Ok(())
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();if args.len()!=2&&args.len()!=3{return Err("usage: semantic_coverage PREPARED_SEMANTIC_ROOT NEW_OUTPUT [RUNTIME_SAFETY_RON]".into());}
    let root=Path::new(&args[0]);let out=Path::new(&args[1]);fs::create_dir(out)?;
    let (settings,_profile_bytes)=normal::runtime_profile(args.get(2).map(|p|Path::new(p)),&out.join("runtime_context.json"))?;
    let original=FileState::capture(root,65_536,256*1024*1024)?;
    let support=coverage_support();assert_eq!(support.revision,"cmva.coverage-support/v1");
    let expected:BTreeSet<_>=KINDS.into_iter().collect();assert_eq!(expected.len(),45);
    let actual:BTreeSet<_>=support.definitions.iter().map(|r|format!("{:?}",r.kind)).collect();
    assert_eq!(actual,expected.iter().map(|s|s.to_string()).collect());assert_eq!(support.definitions.len(),45);
    for row in &support.definitions{let name=format!("{:?}",row.kind);
        assert_eq!(row.add_or_replace,ADD_REPLACE.contains(&name.as_str()));assert_eq!(row.remove,REMOVE.contains(&name.as_str()));}
    let inventory:Vec<_>=KINDS.into_iter().map(|kind|json!({"kind":kind,
        "Add":if ADD_REPLACE.contains(&kind){"actual consumer required"}else{"unsupported: hold Incomplete"},
        "Replace":if ADD_REPLACE.contains(&kind){"actual consumer required"}else{"unsupported: hold Incomplete"},
        "Remove":if REMOVE.contains(&kind){"actual base-positive, exact-refusal, surviving-consumer required"}else{"unsupported: hold Incomplete"}})).collect();
    normal::save(&out.join("independent_kind_operation_inventory.json"),&json!({"kinds":inventory,"actual_public_support":support,
        "not_executed_claim":"This inventory does not claim 135 runtime cases; representative unsupported cases follow."}))?;
    let cancel=CancellationToken::default();
    let selection:ContentSelection=serde_json::from_slice(&fs::read(root.join("removal_base_selection.json"))?)?;
    let base=TrustedBase::acquire(root,selection,&InputLimits::default(),settings.clone(),&cancel)?;
    let base_candidate=acquire(root,"removal_base",&base)?;
    let document=normal::prefix(&base_candidate,&out.join("normal_prefix"))?;
    let mut surviving=normal::recipe("host_a",1)?;
    surviving.employees[0].active_fragment_id=Some("u11_surviving_fragment".into());
    surviving.employees[0].relics=vec![LabRelicLoadout{definition_id:"u11_surviving_relic".into(),corrupt:false}];
    normal::place_one(&base_candidate,&document,&mut surviving,&out.join("placement.json"))?;
    let accepted=normal::prepare(&base_candidate,&document,&surviving)?;
    let accepted_before=normal::reconnect(&accepted)?;let accepted_work=accepted.accepted_work();
    let base_run=accepted.snapshot(ApplicationSnapshotKind::Run,&settings.authoring_execution(),&cancel)?;
    let base_run:Value=serde_json::from_str(base_run.document())?;
    let baseline_employee_id=accepted.lab_preparation().unwrap().employee_ids[0].to_string();
    let baseline_employee=base_run.pointer("/read_model/roster/employees").and_then(Value::as_array)
        .ok_or("actual baseline roster required")?.iter().find(|e|e["uuid"]==baseline_employee_id)
        .ok_or("prepared baseline employee absent")?.clone();
    let baseline_identity=base_candidate.generation().identity().clone();
    for(kind,id,label)in [(DefinitionKind::Employee,"u11_removed_employee","employee"),
        (DefinitionKind::Equipment,"u11_removed_equipment","equipment"),(DefinitionKind::SkillFragment,"u11_removed_fragment","fragment"),
        (DefinitionKind::Relic,"u11_removed_relic","relic"),(DefinitionKind::PveEncounter,"u11_removed_encounter","encounter")] {
        let folder=out.join(label);fs::create_dir(&folder)?;let key=DefinitionKey{kind,id:id.into()};
        let candidate=acquire(root,&format!("remove_{label}"),&base)?;
        assert_eq!(candidate.view().removed_definitions,vec![key.clone()]);assert!(candidate.view().changed_definitions.is_empty());
        let mut positive=surviving.clone();match kind{
            DefinitionKind::Employee=>positive.employees[0].definition_id=id.into(),
            DefinitionKind::Equipment=>positive.employees[0].equipment_ids=Some(vec![id.into(),"standard_armor".into()]),
            DefinitionKind::SkillFragment=>positive.employees[0].active_fragment_id=Some(id.into()),
            DefinitionKind::Relic=>positive.employees[0].relics[0].definition_id=id.into(),
            DefinitionKind::PveEncounter=>positive.encounter_id=id.into(),_=>unreachable!(),}
        let positive_trial=normal::prepare(&base_candidate,&document,&positive)?;
        assert_eq!(positive_trial.lab_preparation().expect("actual base receipt").deployed_count,1);
        normal::save(&folder.join("independent_actual_base_positive.json"),&json!({"key":key,"recipe":positive,
            "receipt":positive_trial.lab_preparation(),"reconnect":normal::reconnect(&positive_trial)?}))?;
        let failure=WorldLabPreparation::prepare(candidate.authoring_content(),normal::checked(&document)?,positive.clone(),
            TrialLimits::with_runtime_safety(candidate.generation().runtime_safety()),&cancel).err().ok_or("removed reference unexpectedly prepared")?;
        assert_eq!(failure.position(),TrialPosition::PreparationReady);
        normal::save(&folder.join("independent_actual_removed_refusal.json"),&json!({"error":format!("{failure:?}")}))?;
        let references=json!([{"key":key,"scenario":serde_json::to_vec(&document)?,"preparation":positive}]);
        let valid=suite(label,&document,&surviving,references.clone())?;normal::save(&folder.join("suite.json"),&valid)?;
        let report=run_driver(&candidate,&valid,&cancel)?;normal::save(&folder.join("actual_driver.json"),&report)?;
        assert!(report.coverage.is_complete());assert_eq!(report.coverage.rows.len(),3);
        let expected_rows:BTreeSet<_>=[("removed_absence",CoverageStage::CatalogValidated as usize),
            ("removed_reference",CoverageStage::Ineligible as usize),("removed_surviving",CoverageStage::ConsumedOrEffect as usize)].into_iter().collect();
        let rows:BTreeSet<_>=report.coverage.rows.iter().map(|row|{
            let k=&row.requirement.key;assert_eq!(k.definition_id,id);assert_eq!(k.definition_kind,format!("{kind:?}"));
            assert_eq!(k.generation,candidate.generation().identity().semantic_sha256);assert!(k.action_id.is_empty()&&k.effect_id.is_empty());
            let stage=row.requirement.required_stage as usize;assert_eq!(row.counts[stage],1);(k.producer_family.as_str(),stage)}).collect();
        assert_eq!(rows,expected_rows);assert_eq!(report.cases.len(),1);
        let proof=&report.cases[0].removed_references[0];assert_eq!(report.cases[0].removed_references.len(),1);
        assert_eq!(proof.key,key);assert_eq!(proof.base_generation,baseline_identity);assert_eq!(proof.base_preparation.deployed_count,1);assert!(proof.setup_work>0);
        assert_eq!(report.cases[0].preparation.as_ref().expect("surviving actual receipt").deployed_count,1);
        incomplete(run_driver(&candidate,&suite(label,&document,&surviving,json!([]))?,&cancel),&folder.join("missing_proof.json"))?;
        let mut wrong=references.clone();wrong[0]["key"]["id"]=json!("u11_never_existed");
        incomplete(run_driver(&candidate,&suite(label,&document,&surviving,wrong)?,&cancel),&folder.join("wrong_key.json"))?;
        let mut bad_base=references.clone();bad_base[0]["preparation"]["encounter_id"]=json!("u11_never_existed");
        incomplete(run_driver(&candidate,&suite(label,&document,&surviving,bad_base)?,&cancel),&folder.join("unrelated_base_failure.json"))?;
        let mut no_deployment=surviving.clone();no_deployment.placements.clear();
        incomplete(run_driver(&candidate,&suite(label,&document,&no_deployment,references.clone())?,&cancel),&folder.join("missing_surviving_deployment.json"))?;
        let duplicate=json!([references[0],references[0]]);
        incomplete(run_driver(&candidate,&suite(label,&document,&surviving,duplicate)?,&cancel),&folder.join("duplicate_proof.json"))?;
        assert_eq!(normal::reconnect(&accepted)?,accepted_before);assert_eq!(accepted.accepted_work(),accepted_work);
    }
    let official=TrustedBase::acquire(root,ContentSelection{packs:vec!["official".into()],winners:vec![]},&InputLimits::default(),settings.clone(),&cancel)?;
    // Resource catalog changes remain explicit unsupported holds. Their mechanics
    // are exercised separately by supported Skill/Relic runtime fixtures.
    for(label,kind_base)in [("add_unsupported_resource",&official),("replace_unsupported_resource",&base),("remove_unsupported_resource",&base)] {
        let candidate=acquire(root,label,kind_base)?;let mut recipe=normal::recipe("host_a",1)?;recipe.placements=surviving.placements.clone();
        let run=run_driver(&candidate,&suite(label,&document,&recipe,json!([]))?,&cancel)?;
        normal::save(&out.join(format!("{label}.json")),&run)?;assert!(!run.coverage.is_complete());
        assert!(run.coverage.unsupported.iter().any(|s|s.contains("Resource:u11_unsupported_resource")));
        assert_eq!(normal::reconnect(&accepted)?,accepted_before);assert_eq!(accepted.accepted_work(),accepted_work);
    }
    for(label,requirements)in [("add_employee",vec![("Employee","u11_profile_employee","employee_profile")]),
        ("add_encounter",vec![("PveEncounter","u11_profile_encounter","encounter_plan")]),
        ("add_fragment",vec![("Employee","u11_fragment_employee","employee_profile"),("SkillFragment","u11_profile_fragment","fragment_profile")]),
        ("replace_run_policy",vec![("RunPolicy","run_policy","run_start_policy")])] {
        let candidate=acquire(root,label,&official)?;let folder=out.join(label);fs::create_dir(&folder)?;
        let document=normal::prefix(&candidate,&folder.join("normal_prefix"))?;
        let mut recipe=normal::recipe("host_a",1)?;recipe.placements=surviving.placements.clone();
        if label=="add_employee"{recipe.employees[0].definition_id="u11_profile_employee".into();}
        if label=="add_fragment"{recipe.employees[0].definition_id="u11_fragment_employee".into();}
        if label=="add_encounter"{recipe.encounter_id="u11_profile_encounter".into();}
        let trial=normal::prepare(&candidate,&document,&recipe)?;
        let projection=trial.snapshot(ApplicationSnapshotKind::Run,&settings.authoring_execution(),&cancel)?;
        let projection:Value=serde_json::from_str(projection.document())?;
        normal::save(&folder.join("actual_run.json"),&projection)?;
        if label=="replace_run_policy"{assert_eq!(projection.pointer("/read_model/resources/enkephalin").ok_or("run resource projection absent")?,777);}
        if label=="add_employee"||label=="add_fragment" {
            let id=trial.lab_preparation().unwrap().employee_ids[0].to_string();
            let employee=projection.pointer("/read_model/roster/employees").and_then(Value::as_array)
                .ok_or("actual changed roster required")?.iter().find(|e|e["uuid"]==id).ok_or("prepared changed employee absent")?;
            assert_eq!(employee["employee_definition_id"],recipe.employees[0].definition_id);
            assert_eq!(employee.pointer("/profile/base_combat_stats/max_hp").ok_or("profile HP absent")?,777);
            if label=="add_fragment" {
                assert_eq!(employee["skill_fragments"]["baseline_ids"],json!(["u11_profile_fragment"]));
                let attack=employee.pointer("/combat_profile/effective_stats/attack").and_then(Value::as_u64).ok_or("actual effective attack absent")?;
                let base_attack=baseline_employee.pointer("/combat_profile/effective_stats/attack").and_then(Value::as_u64).ok_or("baseline effective attack absent")?;
                normal::save(&folder.join("independent_baseline_employee.json"),&baseline_employee)?;
                assert_eq!(baseline_employee["skill_fragments"]["baseline_ids"],json!([]),
                    "records_clerk has no authored baseline fragment before this added profile");
                assert_eq!(baseline_employee.pointer("/combat_profile/base_stats/attack"),Some(&json!(10)));
                assert_eq!(employee.pointer("/combat_profile/base_stats/attack"),Some(&json!(10)));
                assert_eq!(base_attack,10);assert_eq!(attack,51);
                assert_eq!(attack.checked_sub(base_attack),Some(41),
                    "adding the authored bonus41 to an empty baseline changes effective attack10 to51");
            }
        }
        let report=run_driver(&candidate,&suite(label,&document,&recipe,json!([]))?,&cancel)?;
        normal::save(&folder.join("actual_driver.json"),&report)?;assert!(report.coverage.is_complete());
        let actual:BTreeSet<_>=report.coverage.rows.iter().map(|row|{
            assert_eq!(row.requirement.required_stage,CoverageStage::ConsumedOrEffect);assert!(row.counts[4]>0);
            let key=&row.requirement.key;(key.definition_kind.as_str(),key.definition_id.as_str(),key.producer_family.as_str())}).collect();
        assert_eq!(actual,requirements.into_iter().collect());
        assert_eq!(normal::reconnect(&accepted)?,accepted_before);assert_eq!(accepted.accepted_work(),accepted_work);
    }
    original.ensure_unchanged(&FileState::capture(root,65_536,256*1024*1024)?)?;
    normal::save(&out.join("RESULT.json"),&json!({"passed":true,"scope":"actual semantic diagnostics; no approval",
        "removed_positive_kinds":5,"unsupported_operations":3,"nonbattle_positive_cases":4,"literal_kind_inventory":45,
        "original_content_and_other_trial_unchanged":true,"approval_claimed":false}))?;Ok(())
}
