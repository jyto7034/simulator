//! Independent nonbattle/removal inputs. Preparation never issues approval.
use game_core::game::{data::{content_pack::{self,ContentSelection,CatalogFile,CatalogKind,ContentContract,
    DefinitionKey,DefinitionKind,DefinitionOperation,InputLimits,OperationKind,PackManifest,PackNamespace},
    employee_data::EmployeeDefinitionDatabase,equipment_data::EquipmentDatabase,pve_data::PveEncounterDatabase,
    skill_fragment_data::{SkillFragmentDatabase,SkillFragmentEffectDef}}};
use serde_json::json;
use std::{error::Error,fs,path::Path};
use u11_independent_public_probe::{normal_preparation::save,file_state::FileState};
use uuid::Uuid;
type Result<T>=std::result::Result<T,Box<dyn Error>>;
fn pack(root:&Path,id:&str,files:Vec<(CatalogKind,String)>,dependencies:Vec<String>,operations:Vec<DefinitionOperation>)->Result<()> {
    let dir=root.join(id);fs::create_dir(&dir)?;let mut catalogs=Vec::new();
    for(index,(kind,raw))in files.into_iter().enumerate(){let path=format!("{index}.ron");fs::write(dir.join(&path),raw)?;catalogs.push(CatalogFile{kind,path:path.into()});}
    let manifest=PackManifest{pack_id:id.into(),pack_version:"1".into(),namespace:PackNamespace::Global,
        contract:ContentContract::current(),required_capabilities:vec![],dependencies,files:catalogs,operations};
    fs::write(dir.join("pack.ron"),ron::ser::to_string(&manifest)?)?;Ok(())
}
fn main()->Result<()> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();if args.len()!=2{return Err("usage: prepare_semantic_registry OFFICIAL_SOURCE_ROOT NEW_ROOT".into());}
    let root=Path::new(&args[1]);fs::create_dir(root)?;
    let packaged=content_pack::package_official(Path::new(&args[0]),&root.join("official"),&InputLimits::default())?;
    save(&root.join("official_packaging.json"),&packaged)?;
    let official=root.join("official/data");
    let employees:EmployeeDefinitionDatabase=ron::de::from_str(&fs::read_to_string(official.join("employees/definitions.ron"))?)?;
    let equipment:EquipmentDatabase=ron::de::from_str(&fs::read_to_string(official.join("equipments/base.ron"))?)?;
    let fragments:SkillFragmentDatabase=ron::de::from_str(&fs::read_to_string(official.join("skill_fragments/base.ron"))?)?;
    let encounters:PveEncounterDatabase=ron::de::from_str(&fs::read_to_string(official.join("pve/encounters.ron"))?)?;
    let mut employee=employees.get_by_id("records_clerk").ok_or("official employee absent")?.clone();
    employee.employee_definition_id="u11_removed_employee".into();employee.name="U11 independent removable employee".into();employee.starter_eligible=false;
    let mut item=equipment.items.iter().find(|i|i.id=="standard_training_sword").ok_or("official weapon absent")?.clone();
    item.id="u11_removed_equipment".into();item.uuid=Uuid::from_u128(0xc0de1100000000000000000000006101);item.name=item.id.clone();
    let mut encounter=encounters.encounters.iter().find(|e|e.id=="normal_corroded_patrol").ok_or("official encounter absent")?.clone();
    encounter.id="u11_removed_encounter".into();
    let source=fragments.fragments.iter().find(|f|f.id.as_str()=="fragment_freischutz_black_round").ok_or("official active fragment absent")?;
    let mut active_fragments=Vec::new();
    for(index,id)in ["u11_removed_fragment","u11_surviving_fragment"].into_iter().enumerate(){
        let mut fragment=source.clone();fragment.id=id.into();fragment.uuid=Uuid::from_u128(0xc0de1100000000000000000000006201+index as u128);
        fragment.name=id.into();fragment.compatibility=Default::default();active_fragments.push(fragment);
    }
    let relics=format!("(relics:[{}])",["u11_removed_relic","u11_surviving_relic"].into_iter().enumerate().map(|(i,id)|
        format!("(id:\"{id}\",uuid:\"{}\",name:\"{id}\",description:\"Independent unchanged trusted-base removal control\",rarity:\"probe\",allow_duplicates:false,effects:[(id:\"positive_control\",flat:1,percent:0)],rules:[],dismantle_equipment_dust:1)",
            Uuid::from_u128(0xc0de1100000000000000000000006301+i as u128))).collect::<Vec<_>>().join(","));
    let resource="(resources:[(id:\"u11_unsupported_resource\",owner:Employee,lifetime:Run,initial:3,cap:9)])";
    pack(root,"semantic_base",vec![
        (CatalogKind::Employees,format!("(definitions:[{}])",ron::ser::to_string(&employee)?)),
        (CatalogKind::Equipment,format!("(items:[{}])",ron::ser::to_string(&item)?)),
        (CatalogKind::SkillFragments,ron::ser::to_string(&SkillFragmentDatabase::new(active_fragments))?),
        (CatalogKind::Relics,relics),(CatalogKind::PveEncounters,ron::ser::to_string(&PveEncounterDatabase::new(vec![encounter.clone()]))?),
        (CatalogKind::Resources,resource.into())],vec!["official".into()],vec![])?;
    fs::create_dir(root.join("registries"))?;
    let base_selection=ContentSelection{packs:vec!["official".into(),"semantic_base".into()],winners:vec![]};
    save(&root.join("removal_base_selection.json"),&base_selection)?;
    save(&root.join("official_base_selection.json"),&ContentSelection{packs:vec!["official".into()],winners:vec![]})?;
    let base=json!({"mod_id":"base","mod_version":"1","packs":["official","semantic_base"],"dependencies":[]});
    save(&root.join("registries/removal_base.json"),&json!({"schema_version":1,"mods":[base],"selection":{"mods":["base"],"winners":[]}}))?;
    let mut inventory=Vec::new();
    for(kind,id,label)in [(DefinitionKind::Employee,"u11_removed_employee","employee"),
        (DefinitionKind::Equipment,"u11_removed_equipment","equipment"),(DefinitionKind::SkillFragment,"u11_removed_fragment","fragment"),
        (DefinitionKind::Relic,"u11_removed_relic","relic"),(DefinitionKind::PveEncounter,"u11_removed_encounter","encounter"),
        (DefinitionKind::Resource,"u11_unsupported_resource","unsupported_resource")] {
        let pack_id=format!("remove_{label}");let key=DefinitionKey{kind,id:id.into()};
        pack(root,&pack_id,vec![(CatalogKind::Skills,"SkillDatabase(skills:[])".into())],vec!["semantic_base".into()],
            vec![DefinitionOperation{target:key.clone(),operation:OperationKind::Remove,after:vec!["semantic_base".into()]}])?;
        save(&root.join(format!("registries/{pack_id}.json")),&json!({"schema_version":1,
            "mods":[base,{"mod_id":"change","mod_version":"1","packs":[pack_id],"dependencies":["base"]}],
            "selection":{"mods":["base","change"],"winners":[]}}))?;
        inventory.push(json!({"case":pack_id,"key":key,"operation":"Remove","supported":label!="unsupported_resource"}));
    }
    pack(root,"replace_unsupported_resource",vec![(CatalogKind::Resources,resource.replace("initial:3","initial:4"))],vec!["semantic_base".into()],
        vec![DefinitionOperation{target:DefinitionKey{kind:DefinitionKind::Resource,id:"u11_unsupported_resource".into()},operation:OperationKind::Replace,after:vec!["semantic_base".into()]}])?;
    save(&root.join("registries/replace_unsupported_resource.json"),&json!({"schema_version":1,"mods":[base,
        {"mod_id":"change","mod_version":"1","packs":["replace_unsupported_resource"],"dependencies":["base"]}],
        "selection":{"mods":["base","change"],"winners":[]}}))?;
    employee.employee_definition_id="u11_profile_employee".into();employee.profile.base_combat_stats.max_hp=777;
    pack(root,"add_employee",vec![(CatalogKind::Employees,format!("(definitions:[{}])",ron::ser::to_string(&employee)?))],vec!["official".into()],vec![])?;
    encounter.id="u11_profile_encounter".into();
    pack(root,"add_encounter",vec![(CatalogKind::PveEncounters,ron::ser::to_string(&PveEncounterDatabase::new(vec![encounter]))?)],vec!["official".into()],vec![])?;
    let mut basic=fragments.fragments.iter().find(|f|f.id.as_str()=="starter_basic_attack_enhancement").ok_or("official baseline fragment absent")?.clone();
    basic.id="u11_profile_fragment".into();basic.uuid=Uuid::from_u128(0xc0de1100000000000000000000006401);
    basic.effect=SkillFragmentEffectDef::BasicAttackModifier{attack_bonus:41,attack_interval_ms_reduction:0};
    employee.employee_definition_id="u11_fragment_employee".into();employee.initial_loadout.baseline_skill_fragment_ids=vec![basic.id.clone()];
    pack(root,"add_fragment",vec![(CatalogKind::Employees,format!("(definitions:[{}])",ron::ser::to_string(&employee)?)),
        (CatalogKind::SkillFragments,ron::ser::to_string(&SkillFragmentDatabase::new(vec![basic]))?)],vec!["official".into()],vec![])?;
    let policy=fs::read_to_string(official.join("run/policy.ron"))?;
    let changed=policy.replacen("starter_enkephalin: 500","starter_enkephalin: 777",1);assert_ne!(changed,policy);
    pack(root,"replace_run_policy",vec![(CatalogKind::RunPolicy,changed)],vec!["official".into()],
        vec![DefinitionOperation{target:DefinitionKey{kind:DefinitionKind::RunPolicy,id:"run_policy".into()},operation:OperationKind::Replace,after:vec!["official".into()]}])?;
    pack(root,"add_unsupported_resource",vec![(CatalogKind::Resources,resource.into())],vec!["official".into()],vec![])?;
    for id in ["add_employee","add_encounter","add_fragment","replace_run_policy","add_unsupported_resource"] {
        save(&root.join(format!("registries/{id}.json")),&json!({"schema_version":1,"mods":[
            {"mod_id":"base","mod_version":"1","packs":["official"],"dependencies":[]},
            {"mod_id":"change","mod_version":"1","packs":[id],"dependencies":["base"]}],
            "selection":{"mods":["base","change"],"winners":[]}}))?;
    }
    save(&root.join("SEMANTIC_ORACLES.json"),&json!({"prepared_only":true,"removals":inventory,
        "profile_employee_max_hp":777,"baseline_fragment_attack_bonus":41,"run_start_enkephalin":777,
        "remove_proof_required":["actual frozen base positive deployed preparation","exact typed missing reference","actual different surviving preparation"]}))?;
    let files:Vec<_>=FileState::capture(root,65_536,256*1024*1024)?.report().entries.into_iter()
        .filter(|e|e.kind=="file").map(|e|json!({"path":e.relative,"bytes":e.bytes,"sha256":e.sha256.expect("actual regular file hash")})).collect();
    save(&root.join("FIXTURE_RECEIPT.json"),&json!({"schema":"u11.fixture-preparation/v1","runtime_executed":false,
        "source_root":Path::new(&args[0]),"fixture_root":root,"files":files,"criterion_selected":false}))?;
    Ok(())
}
