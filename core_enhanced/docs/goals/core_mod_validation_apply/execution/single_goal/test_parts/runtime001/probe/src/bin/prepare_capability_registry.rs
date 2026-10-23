//! Explicit trusted resource base and supported-kind consumer candidates.
//! Copies the nonlethal lab base into a fresh root; never approves Resource edits.
use game_core::game::{ability::{AbilityActivationBinding,AbilityActivationDef,SkillId},
    data::{content_pack::{CatalogFile,CatalogKind,ContentContract,PackManifest,PackNamespace},equipment_data::EquipmentDatabase},
    stats::TriggerType};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs,path::Path};
use u11_independent_public_probe::{file_state::FileState,normal_preparation::save};
use uuid::Uuid;
type Result<T>=std::result::Result<T,Box<dyn Error>>;
fn copy(source:&Path,target:&Path)->Result<()> {
    fs::create_dir(target)?;
    for entry in fs::read_dir(source)?{let entry=entry?;
        if entry.file_type()?.is_dir(){copy(&entry.path(),&target.join(entry.file_name()))?;}
        else if entry.file_type()?.is_file(){fs::copy(entry.path(),target.join(entry.file_name()))?;}
        else{return Err("explicit laboratory base requires only regular members".into());}}
    Ok(())
}
fn pack(root:&Path,id:&str,kind:CatalogKind,raw:&str,dependencies:Vec<String>)->Result<()> {
    let dir=root.join(id);fs::create_dir(&dir)?;fs::write(dir.join("catalog.ron"),raw)?;
    let manifest=PackManifest{pack_id:id.into(),pack_version:"1".into(),namespace:PackNamespace::Global,
        contract:ContentContract::current(),required_capabilities:vec![],dependencies,
        files:vec![CatalogFile{kind,path:"catalog.ron".into()}],operations:vec![]};
    fs::write(dir.join("pack.ron"),ron::ser::to_string(&manifest)?)?;Ok(())
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=2{return Err("usage: prepare_capability_registry NONLETHAL_LAB_ROOT NEW_CAPABILITY_ROOT".into());}
    let source=Path::new(&args[0]);let root=Path::new(&args[1]);
    let before=FileState::capture(source,65_536,256*1024*1024)?;
    let lab:Value=serde_json::from_slice(&fs::read(source.join("LAB_BASE_ORACLE.json"))?)?;
    assert_eq!(lab["authored_target"]["max_health"],10_000);
    fs::create_dir(root)?;copy(&source.join("official"),&root.join("official"))?;
    fs::copy(source.join("LAB_BASE_ORACLE.json"),root.join("LAB_BASE_ORACLE.json"))?;
    let resource_path=root.join("official/data/resources/base.ron");
    let original=fs::read(&resource_path)?;
    let baseline:game_core::game::resource::ResourceCatalog=ron::de::from_bytes(&original)?;
    assert!(baseline.resources.is_empty(),"explicit baseline currently has no official typed resources");
    let seeded="(resources:[(id:\"u11_meter\",owner:Employee,lifetime:Deployment,initial:9,cap:20)])";
    fs::write(&resource_path,seeded)?;
    save(&root.join("TRUSTED_RESOURCE_BASE.json"),&json!({"purpose":"fixed laboratory definitions before TrustedBase acquisition; supported Skills and Relics consume them",
        "source_root":source,"original_resource_sha256":format!("{:x}",Sha256::digest(&original)),
        "seeded_resource_sha256":format!("{:x}",Sha256::digest(seeded.as_bytes())),
        "resource":{"id":"u11_meter","owner":"Employee","lifetime":"Deployment","initial":9,"cap":20},
        "resource_catalog_edit_approval":false,"runtime_executed":false}))?;
    let equipment:EquipmentDatabase=ron::de::from_bytes(&fs::read(root.join("official/data/equipments/base.ron"))?)?;
    fs::create_dir(root.join("registries"))?;
    for(label,skill_id,serial,skills,relics)in [
        ("resources","u11_resource_strike",6001,include_str!("../../../fixtures/skills_resources.ron"),include_str!("../../../fixtures/relics_resources.ron")),
        ("recipe","u11_recipe_strike",6002,include_str!("../../../fixtures/skills_recipe.ron"),include_str!("../../../fixtures/relics_recipe.ron")),
    ]{
        let skills_pack=format!("{label}_skills");let relic_pack=format!("{label}_relics");let item_pack=format!("{label}_item");
        pack(root,&skills_pack,CatalogKind::Skills,skills,vec!["official".into()])?;
        pack(root,&relic_pack,CatalogKind::Relics,relics,vec!["official".into(),skills_pack.clone()])?;
        let mut item=equipment.items.iter().find(|e|e.id=="standard_training_sword").ok_or("canonical weapon absent")?.clone();
        item.id=format!("u11_{label}_weapon");item.name=item.id.clone();
        item.uuid=Uuid::from_u128(0xc0de1100000000000000000000000000|serial);
        item.triggered_effects.clear();item.ability_activations=vec![AbilityActivationBinding{ability_id:SkillId::from(skill_id),
            activation:AbilityActivationDef::TriggerProc{trigger:TriggerType::OnAttack,proc_chance_percent:100,
                internal_cooldown_ms:0,max_triggers_per_battle:Some(1)}}];
        pack(root,&item_pack,CatalogKind::Equipment,&format!("EquipmentDatabase(items:[{}])",ron::ser::to_string(&item)?),
            vec!["official".into(),skills_pack.clone()])?;
        save(&root.join(format!("registries/{label}.json")),&json!({"schema_version":1,
            "mods":[{"mod_id":"base","mod_version":"1","packs":["official"],"dependencies":[]},
                {"mod_id":label,"mod_version":"1","packs":[skills_pack,relic_pack,item_pack],"dependencies":["base"]}],
            "selection":{"mods":["base",label],"winners":[]}}))?;
    }
    // Distinct genuine selected provider; retain the original equipment proc
    // registry and authored bytes as the missing-player-selection negative.
    pack(root,"recipe_manual_skills",CatalogKind::Skills,include_str!("../../../fixtures/skills_recipe_manual.ron"),vec!["official".into()])?;
    pack(root,"recipe_manual_relics",CatalogKind::Relics,include_str!("../../../fixtures/relics_recipe.ron"),vec!["official".into(),"recipe_manual_skills".into()])?;
    pack(root,"recipe_manual_fragment",CatalogKind::SkillFragments,include_str!("../../../fixtures/fragment_recipe_manual.ron"),vec!["official".into(),"recipe_manual_skills".into()])?;
    save(&root.join("registries/recipe_manual.json"),&json!({"schema_version":1,
        "mods":[{"mod_id":"base","mod_version":"1","packs":["official"],"dependencies":[]},
            {"mod_id":"recipe_manual","mod_version":"1","packs":["recipe_manual_skills","recipe_manual_relics","recipe_manual_fragment"],"dependencies":["base"]}],
        "selection":{"mods":["base","recipe_manual"],"winners":[]}}))?;
    before.ensure_unchanged(&FileState::capture(source,65_536,256*1024*1024)?)?;
    save(&root.join("SOURCE_UNCHANGED.json"),&before.report())?;
    let files:Vec<_>=FileState::capture(root,65_536,256*1024*1024)?.report().entries.into_iter()
        .filter(|e|e.kind=="file").map(|e|json!({"path":e.relative,"bytes":e.bytes,"sha256":e.sha256.expect("actual regular file hash")})).collect();
    save(&root.join("FIXTURE_RECEIPT.json"),&json!({"schema":"u11.fixture-preparation/v1","preparation_only":true,"runtime_executed":false,
        "criterion_selected":false,"resource_catalog_approval":false,"source_root":source,
        "files":files,
        "resource_oracle":{"initial":9,"direct_consume":5,"direct_grant":3,"live_read":7,"damage_multiplier":2,"damage":14,"causal_consume":2,"final":5},
        "recipe_oracle":{"original_initial":41,"original_add":7,"original_capacity":48,"original_duration_ms":250,
            "generated_initial":53,"generated_add":11,"generated_capacity":64,"generated_duration_ms":350,"generated_delay_ms":10}}))?;
    Ok(())
}
