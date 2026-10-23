//! Add reviewed trusted geometry, encounter and published tactical funding.
//! Existing lab root files, including explicit target stats, remain exact bytes.
use game_core::game::data::content_pack::ContentSelection;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use game_core::game::data::run_policy_data::RunPolicyData;
use std::{error::Error,fs,path::Path};
use u11_independent_public_probe::{file_state::FileState,normal_preparation::save};
type Result<T>=std::result::Result<T,Box<dyn Error>>;
fn copy(source:&Path,target:&Path)->Result<()> {
    fs::create_dir(target)?;
    for entry in fs::read_dir(source)?{let entry=entry?;let kind=entry.file_type()?;
        if kind.is_dir(){copy(&entry.path(),&target.join(entry.file_name()))?;}
        else if kind.is_file(){fs::copy(entry.path(),target.join(entry.file_name()))?;}
        else{return Err("regular fixture files/directories required".into());}}
    Ok(())
}
fn content_inventory(state:&FileState)->Value {
    json!(state.report().entries.into_iter().map(|entry|json!({"path":entry.relative,"kind":entry.kind,
        "bytes":entry.bytes,"sha256":entry.sha256,"symlink_target":entry.symlink_target})).collect::<Vec<_>>())
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=2{return Err("usage: prepare_workload_fixture FROZEN_LAB_ROOT NEW_WORKLOAD_ROOT".into());}
    let source=Path::new(&args[0]);let out=Path::new(&args[1]);
    let before=FileState::capture(source,65_536,256*1024*1024)?;
    let lab:Value=serde_json::from_slice(&fs::read(source.join("LAB_BASE_ORACLE.json"))?)?;
    if lab["authored_target"]["max_health"]!=10000||lab["authored_target"]["attack"]!=0{return Err("exact existing trusted lab target baseline required".into());}
    copy(source,out)?;
    let copied=FileState::capture(out,65_536,256*1024*1024)?;
    if content_inventory(&before)!=content_inventory(&copied){return Err("fixture copy changed original file inventory".into());}
    // Preserve the source receipt as history, then publish the standard flat
    // receipt only after every successor file has its final bytes.
    let prior_receipt=out.join("PRIOR_LAB_FIXTURE_RECEIPT.json");
    if prior_receipt.exists(){return Err("exclusive prior lab receipt destination required".into());}
    fs::rename(out.join("FIXTURE_RECEIPT.json"),prior_receipt)?;
    let pack=out.join("u11_workload_context");fs::create_dir(&pack)?;
    for (name,bytes) in [
        ("pack.ron",include_bytes!("../../../fixtures/workload_context/pack.ron").as_slice()),
        ("battlefield_templates.ron",include_bytes!("../../../fixtures/workload_context/battlefield_templates.ron").as_slice()),
        ("encounters.ron",include_bytes!("../../../fixtures/workload_context/encounters.ron").as_slice()),
    ]{fs::write(pack.join(name),bytes)?;}
    // This is published fixture currency. Every deployment still pays cost10.
    let source_policy=source.join("official/data/run/policy.ron");
    let original_policy=fs::read_to_string(&source_policy)?;
    let old="tactical_resource: (\n        initial: 20,\n        maximum: 99,";
    let new="tactical_resource: (\n        initial: 160,\n        maximum: 160,";
    if original_policy.matches(old).count()!=1{return Err("one exact official tactical policy block required".into());}
    let funded_policy=original_policy.replacen(old,new,1);
    if funded_policy.replacen(new,old,1)!=original_policy{return Err("unrelated policy bytes changed".into());}
    let original:RunPolicyData=ron::de::from_str(&original_policy)?;
    let funded:RunPolicyData=ron::de::from_str(&funded_policy)?;
    let mut expected=original.clone();expected.tactical_resource.initial=160;expected.tactical_resource.maximum=160;
    if funded!=expected||funded.tactical_resource.base_deploy_cost!=10{return Err("exact funded policy or unchanged deployment price mismatch".into());}
    fs::write(pack.join("run_policy.ron"),&funded_policy)?;
    let funding=json!({"kind":"published trusted fixture RunPolicy replacement","source_path":source_policy,
        "source_sha256":format!("{:x}",Sha256::digest(original_policy.as_bytes())),
        "funded_sha256":format!("{:x}",Sha256::digest(funded_policy.as_bytes())),
        "original_tactical_resource":original.tactical_resource,"funded_tactical_resource":funded.tactical_resource,
        "all_other_policy_bytes_unchanged":true,"all_three_classes_use_funded_context":true,
        "existing_official_normal_evidence_remains_separate":true,"private_currency_injection":false});
    save(&out.join("TACTICAL_FUNDING.json"),&funding)?;
    save(&out.join("workload_base_selection.json"),&ContentSelection{packs:vec!["official".into(),"u11_workload_context".into()],winners:vec![]})?;
    save(&out.join("WORKLOAD_CONTEXT.json"),&json!({"source_root":source,"trusted_target_baseline":lab,
        "new_trusted_pack":"u11_workload_context","tactical_funding":funding,"normal_encounter":"normal_corroded_patrol",
        "expanded_pathological_encounter":"u11_parallel_patrol","roster_counts":[16,500,1000],"deployed_counts":[1,8,16],
        "elapsed_input_ms":60000,"original_files_exact":true,"runtime_executed":false,"criterion_selected":false}))?;
    let after=FileState::capture(source,65_536,256*1024*1024)?;
    before.ensure_unchanged(&after)?;
    let files=FileState::capture(out,65_536,256*1024*1024)?;
    save(&out.join("WORKLOAD_FIXTURE_RECEIPT.json"),&json!({"schema":"u11.workload-fixture/v1","runtime_executed":false,
        "source_before":before.report(),"source_after":after.report(),"output_files":files.report()}))?;
    let final_files:Vec<_>=FileState::capture(out,65_536,256*1024*1024)?.report().entries.into_iter()
        .filter(|entry|entry.kind=="file").map(|entry|json!({"path":entry.relative,"bytes":entry.bytes,
            "sha256":entry.sha256.expect("actual regular fixture file digest")})).collect();
    save(&out.join("FIXTURE_RECEIPT.json"),&json!({"schema":"u11.fixture-preparation/v1",
        "preparation_only":true,"runtime_executed":false,"criterion_selected":false,
        "source_root":source,"files":final_files,"prior_receipt":"PRIOR_LAB_FIXTURE_RECEIPT.json"}))?;
    Ok(())
}
