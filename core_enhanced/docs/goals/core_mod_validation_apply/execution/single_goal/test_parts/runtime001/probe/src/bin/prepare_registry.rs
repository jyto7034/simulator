//! Independent external inputs. This is fixture preparation, never a Lab PASS.
//! Uses existing public packaging/types; no private construction/test feature.
use game_core::game::{
    ability::{AbilityActivationBinding, AbilityActivationDef, SkillId},
    data::{
        content_pack::{self, CatalogFile, CatalogKind, ContentContract, DefinitionKey,
            DefinitionKind, DefinitionOperation, InputLimits, OperationKind, PackManifest,
            PackNamespace},
        equipment_data::EquipmentDatabase,
    },
    stats::TriggerType,
};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{error::Error, fs::{self, OpenOptions}, io::Write, path::Path};
use uuid::Uuid;

const A: &str = include_str!("../../../fixtures/skills_a.ron");
const B: &str = include_str!("../../../fixtures/skills_b.ron");
const NOOP: &str = include_str!("../../../fixtures/skills_noop.ron");
const STRUCTURAL_SKILLS: &str = include_str!("../../../fixtures/skills_structural.ron");
const STRUCTURAL_RELICS: &str = include_str!("../../../fixtures/relics_structural.ron");
const CAUSAL_SKILLS: &str = include_str!("../../../fixtures/skills_causal.ron");
const CAUSAL_RELICS: &str = include_str!("../../../fixtures/relics_causal.ron");
const OBJECT_SKILLS: &str = include_str!("../../../fixtures/skills_object.ron");
const OBJECTS: &str = include_str!("../../../fixtures/objects.ron");

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    OpenOptions::new().write(true).create_new(true).open(path)?.write_all(bytes)?;
    Ok(())
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<(), Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    write_new(path, &bytes)
}
fn pack(root: &Path, id: &str, kind: CatalogKind, raw: &str,
        dependencies: &[&str], operations: Vec<DefinitionOperation>) -> Result<(), Box<dyn Error>> {
    let directory = root.join(id);
    fs::create_dir(&directory)?;
    write_new(&directory.join("catalog.ron"), raw.as_bytes())?;
    let manifest = PackManifest {
        pack_id: id.into(), pack_version: "1".into(), namespace: PackNamespace::Global,
        contract: ContentContract::current(), required_capabilities: vec![],
        dependencies: dependencies.iter().map(|s| (*s).into()).collect(),
        files: vec![CatalogFile { kind, path: "catalog.ron".into() }], operations,
    };
    write_new(&directory.join("pack.ron"), ron::ser::to_string(&manifest)?.as_bytes())
}
fn equipment(original: &EquipmentDatabase, source: &str, id: &str, serial: u128,
             skill: &str) -> Result<String, Box<dyn Error>> {
    let mut item = original.items.iter().find(|item| item.id == source)
        .ok_or("fixture's original equipment is missing")?.clone();
    item.id = id.into();
    item.uuid = Uuid::from_u128(0xc0de1100_0000_0000_0000_000000000000 | serial);
    item.name = id.into();
    item.triggered_effects.clear();
    item.ability_activations = vec![AbilityActivationBinding {
        ability_id: SkillId::from(skill),
        activation: AbilityActivationDef::TriggerProc { trigger: TriggerType::OnAttack,
            proc_chance_percent: 100, internal_cooldown_ms: 0, max_triggers_per_battle: Some(1) },
    }];
    // The database owns private indexes; deserialize an authored single-item
    // catalog rather than manufacturing its private runtime index fields.
    Ok(format!("EquipmentDatabase(items:[{}])", ron::ser::to_string(&item)?))
}
fn mod_entry(id: &str, packs: &[&str], dependencies: &[&str]) -> Value {
    json!({"mod_id":id,"mod_version":"1","packs":packs,"dependencies":dependencies})
}
fn registry(mods: Vec<Value>, selected: &[&str], winners: Value) -> Value {
    json!({"schema_version":1,"mods":mods,"selection":{"mods":selected,"winners":winners}})
}
fn load_skill(id:&str,count:usize)->String {
    assert!(count>=2);
    let effects=(0..count).map(|index|format!(
        "(id:\"effect_{index}\",op:Damage(amount:{},damage_type:True))",u8::from(index==0)))
        .collect::<Vec<_>>().join(",");
    format!("SkillDatabase(skills:[SkillDef(id:\"{id}\",name:\"Independent finite effect batch\",kind:Targeted,focus_time_ms:0,use_lifetime:Action,cast_targeting:Explicit(selection:automatic_unit,target:EnemySingle(rule:Nearest),reach:global,air_capable:true),steps:[(id:\"load\",delay_ms:0,target:EnemySingle(rule:Nearest),delivery:Instant,effects:[{effects}])])])")
}
fn inventory(root: &Path) -> Result<Vec<Value>, Box<dyn Error>> {
    fn visit(root: &Path, current: &Path, rows: &mut Vec<Value>) -> Result<(), Box<dyn Error>> {
        let mut entries = fs::read_dir(current)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let kind = entry.file_type()?;
            if kind.is_dir() { visit(root, &path, rows)?; }
            else if kind.is_file() {
                let bytes = fs::read(&path)?;
                rows.push(json!({"path":path.strip_prefix(root)?.to_string_lossy(),
                    "bytes":bytes.len(),"sha256":format!("{:x}",Sha256::digest(&bytes))}));
            } else { return Err("fixture inventory contains non-regular entry".into()); }
        }
        Ok(())
    }
    let mut rows = vec![];
    visit(root, root, &mut rows)?;
    Ok(rows)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 { return Err("usage: prepare_registry OFFICIAL_SOURCE_ROOT NEW_FIXTURE_ROOT".into()); }
    let source = Path::new(&args[0]);
    let root = Path::new(&args[1]);
    fs::create_dir(root)?;
    let receipt = content_pack::package_official(source, &root.join("official"), &InputLimits::default())?;
    write_json(&root.join("official_packaging.json"), &receipt)?;
    let equipment_path = root.join("official/data/equipments/base.ron");
    let original: EquipmentDatabase = ron::de::from_str(&fs::read_to_string(equipment_path)?)?;
    pack(root, "a_skills", CatalogKind::Skills, A, &["official"], vec![])?;
    pack(root, "b_skills", CatalogKind::Skills, B, &["official"], vec![])?;
    pack(root, "noop_skills", CatalogKind::Skills, NOOP, &["official"], vec![])?;
    pack(root,"terminal_skills",CatalogKind::Skills,include_str!("../../../fixtures/skills_terminal.ron"),&["official"],vec![])?;
    pack(root,"terminal_item",CatalogKind::Equipment,
        &equipment(&original,"standard_training_sword","u11_terminal_weapon",9,"u11_terminal_chain")?,
        &["official","terminal_skills"],vec![])?;
    pack(root,"zero_site_skill",CatalogKind::Skills,
        "SkillDatabase(skills:[SkillDef(id:\"u11_empty_step\",name:\"Legal empty step\",kind:Targeted,focus_time_ms:0,use_lifetime:Action,cast_targeting:Explicit(selection:automatic_unit,target:EnemySingle(rule:Nearest),reach:global,air_capable:true),steps:[(id:\"empty\",delay_ms:0,target:EnemySingle(rule:Nearest),delivery:Instant,effects:[])])])",
        &["official"],vec![])?;
    pack(root,"zero_site_relic",CatalogKind::Relics,
        "(relics:[(id:\"u11_empty_relic\",uuid:\"c0de1100-0000-0000-0000-000000000299\",name:\"Invalid empty relic\",description:\"Explicit contract rejection: no effects or rules\",rarity:\"probe\",allow_duplicates:false,effects:[],rules:[],dismantle_equipment_dust:1)])",
        &["official"],vec![])?;
    pack(root,"unconsumed_numeric_relic",CatalogKind::Relics,
        "(relics:[(id:\"u11_unconsumed_numeric_relic\",uuid:\"c0de1100-0000-0000-0000-000000000298\",name:\"Valid unconsumed numeric relic\",description:\"Publication succeeds; normal suite never equips this relic\",rarity:\"probe\",allow_duplicates:false,effects:[(id:\"positive_flat\",flat:1,percent:0)],rules:[],dismantle_equipment_dust:1)])",
        &["official"],vec![])?;
    pack(root,"noop_heal_item",CatalogKind::Equipment,
        &equipment(&original,"standard_training_sword","u11_heal_weapon",7,"u11_full_health_heal")?,
        &["official","noop_skills"],vec![])?;
    pack(root,"noop_zero_item",CatalogKind::Equipment,
        &equipment(&original,"standard_armor","u11_zero_armor",8,"u11_zero_damage")?,
        &["official","noop_skills"],vec![])?;
    for (prefix, skills, relics, skill_id, serial) in [
        ("structural",STRUCTURAL_SKILLS,STRUCTURAL_RELICS,"u11_structural_strike",4),
        ("causal",CAUSAL_SKILLS,CAUSAL_RELICS,"u11_causal_strike",5),
    ] {
        let skills_pack=format!("{prefix}_skills");
        pack(root,&skills_pack,CatalogKind::Skills,skills,&["official"],vec![])?;
        pack(root,&format!("{prefix}_relics"),CatalogKind::Relics,relics,&["official",&skills_pack],vec![])?;
        pack(root,&format!("{prefix}_item"),CatalogKind::Equipment,
            &equipment(&original,"standard_training_sword",&format!("u11_{prefix}_weapon"),serial,skill_id)?,
            &["official",&skills_pack],vec![])?;
    }
    pack(root,"object_definitions",CatalogKind::SummonedUnits,OBJECTS,&["official"],vec![])?;
    pack(root,"object_skills",CatalogKind::Skills,OBJECT_SKILLS,&["official","object_definitions"],vec![])?;
    pack(root,"object_item",CatalogKind::Equipment,
        &equipment(&original,"standard_training_sword","u11_object_weapon",6,"u11_object_create")?,
        &["official","object_skills"],vec![])?;
    pack(root, "a_item", CatalogKind::Equipment,
        &equipment(&original, "standard_training_sword", "u11_weapon_a", 1, "u11_damage_a")?,
        &["official", "a_skills"], vec![])?;
    pack(root, "b_item", CatalogKind::Equipment,
        &equipment(&original, "standard_armor", "u11_armor_b", 2, "u11_damage_b")?,
        &["official", "b_skills"], vec![])?;
    for (side,source_item,skills_pack,skill_id,serial) in [
        ("a","standard_training_sword","a_skills","u11_damage_a",1001),
        ("b","standard_armor","b_skills","u11_damage_b",1002),
    ] {
        pack(root,&format!("host_{side}_item"),CatalogKind::Equipment,
            &equipment(&original,source_item,source_item,serial,skill_id)?,&["official",skills_pack],
            vec![DefinitionOperation{target:DefinitionKey{kind:DefinitionKind::Equipment,id:source_item.into()},
                operation:OperationKind::Replace,after:vec!["official".into()]}])?;
    }
    pack(root,"remove_a",CatalogKind::Skills,"SkillDatabase(skills:[])",
        &["official","a_skills","a_item"],vec![
            DefinitionOperation{target:DefinitionKey{kind:DefinitionKind::Skill,id:"u11_damage_a".into()},
                operation:OperationKind::Remove,after:vec!["a_skills".into()]},
            DefinitionOperation{target:DefinitionKey{kind:DefinitionKind::Equipment,id:"u11_weapon_a".into()},
                operation:OperationKind::Remove,after:vec!["a_item".into()]},
        ])?;
    pack(root, "broken_item", CatalogKind::Equipment,
        &equipment(&original, "standard_training_sword", "u11_broken_weapon", 3, "u11_missing_skill")?,
        &["official", "a_skills"], vec![])?;
    fs::create_dir(root.join("broken_manifest"))?;
    write_new(&root.join("broken_manifest/pack.ron"), b"(pack_id:\"broken_manifest\", deliberately_invalid:")?;
    for (id, amount) in [("replace_left", 23), ("replace_right", 31)] {
        let text = A.replace("amount: 13", &format!("amount: {amount}"));
        assert_ne!(text, A);
        pack(root, id, CatalogKind::Skills, &text, &["official", "a_skills"],
            vec![DefinitionOperation { target: DefinitionKey { kind: DefinitionKind::Skill, id:"u11_damage_a".into() },
                operation: OperationKind::Replace, after: vec!["a_skills".into()] }])?;
    }
    let base = mod_entry("base", &["official"], &[]);
    let a = mod_entry("a", &["a_skills", "a_item"], &["base"]);
    let b = mod_entry("b", &["b_skills", "b_item"], &["base"]);
    let n = mod_entry("noop", &["noop_skills","noop_heal_item","noop_zero_item"], &["base"]);
    let mut normal = vec![base.clone(), a.clone(), b.clone(), n.clone(),
        mod_entry("terminal",&["terminal_skills","terminal_item"],&["base"]),
        mod_entry("structural",&["structural_skills","structural_item","structural_relics"],&["base"]),
        mod_entry("causal",&["causal_skills","causal_item","causal_relics"],&["base"]),
        mod_entry("objects",&["object_definitions","object_skills","object_item"],&["base"])];
    // Input tiers for real calibration, not performance criteria or predeclared
    // PASS/FAIL. Each batch includes one positive damage site and individually
    // addressed legal zero sites; both mods can fire in one actual actor bucket.
    let workload_tiers=[("small",64usize),("expanded",256),("pathological",1024)];
    let mut workload_oracles=vec![];
    for (tier_index,(tier,count)) in workload_tiers.iter().enumerate() {
        for (side_index,(side,source_item,slot)) in [("a","standard_training_sword","weapon"),
            ("b","standard_armor","armor")].iter().enumerate() {
            let mod_id=format!("load_{tier}_{side}");let skill_id=format!("u11_{mod_id}");
            let skills_pack=format!("{mod_id}_skills");let item_pack=format!("{mod_id}_item");
            let equipment_id=format!("u11_{mod_id}_{slot}");
            pack(root,&skills_pack,CatalogKind::Skills,&load_skill(&skill_id,*count),&["official"],vec![])?;
            pack(root,&item_pack,CatalogKind::Equipment,
                &equipment(&original,source_item,&equipment_id,100+tier_index as u128*2+side_index as u128,&skill_id)?,
                &["official",&skills_pack],vec![])?;
            normal.push(mod_entry(&mod_id,&[&skills_pack,&item_pack],&["base"]));
            pack(root,&format!("host_{mod_id}_item"),CatalogKind::Equipment,
                &equipment(&original,source_item,source_item,2000+tier_index as u128*2+side_index as u128,&skill_id)?,
                &["official",&skills_pack],vec![DefinitionOperation{
                    target:DefinitionKey{kind:DefinitionKind::Equipment,id:(*source_item).into()},
                    operation:OperationKind::Replace,after:vec!["official".into()]}])?;
            workload_oracles.push(json!({"mod":mod_id,"skill":skill_id,"equipment":equipment_id,
                "effects":count,"positive_site":{"action":"load","effect":"effect_0","requested_damage":1},
                "legal_zero_sites":{"action":"load","first_effect":"effect_1","last_effect":format!("effect_{}",count-1)},
                "max_triggers_per_battle":1,"runtime_consumption_proven":false}));
        }
    }
    let mut cases: Vec<(&str, Value, &str)> = vec![
        ("base", registry(normal.clone(), &["base"], json!([])), "valid trusted-base candidate; not approval"),
        ("a", registry(normal.clone(), &["base","a"], json!([])), "valid two-pack mod A"),
        ("b", registry(normal.clone(), &["base","b"], json!([])), "valid two-pack mod B"),
        ("ab", registry(normal.clone(), &["base","a","b"], json!([])), "valid combined inputs; actual combination Lab required"),
        ("noop", registry(normal.clone(), &["base","noop"], json!([])), "legal no-op sites require actual branch witness and positive control"),
        ("terminal",registry(normal.clone(),&["base","terminal"],json!([])),"one ordered actual self chain: cost7/heal7/fullhealnoop/shield10/absorbed5/zeroDamageExpr; terminal counts exact and deliberate coverage hold"),
        ("structural",registry(normal.clone(),&["base","structural"],json!([])),"wear structural weapon and own relic; original30/generated75 must retain separate producer sites"),
        ("causal",registry(normal.clone(),&["base","causal"],json!([])),"wear causal weapon and own relic; wedge/subscriber/two Join branches/continuation/periodic require actual witnesses and cleanup"),
        ("objects",registry(normal.clone(),&["base","objects"],json!([])),"free source+(1,0) tile; one triggered finite actor must spawn, expire at250ms and reclaim"),
    ];
    for(id,pack_id,oracle)in [
        ("zero_site_skill","zero_site_skill","legal changed empty Skill retains an obligation or explicit Incomplete"),
        ("zero_site_relic","zero_site_relic","invalid empty Relic must fail publication with its actual effects/rules contract diagnostic"),
        ("unconsumed_numeric_relic","unconsumed_numeric_relic","valid nonzero numeric Relic publishes but missing actual consumer remains Incomplete")] {
        cases.push((id,registry(vec![base.clone(),mod_entry(id,&[pack_id],&["base"])],&["base",id],json!([])),
            oracle));
    }
    let host_mods=vec![base.clone(),mod_entry("host_a",&["a_skills","host_a_item"],&["base"]),
        mod_entry("host_b",&["b_skills","host_b_item"],&["base"])];
    cases.push(("host_a",registry(host_mods.clone(),&["base","host_a"],json!([])),
        "same canonical loadout/suite as B: sword OnAttack13; active A must retain its actual source after future B Apply"));
    cases.push(("host_b",registry(host_mods.clone(),&["base","host_b"],json!([])),
        "same canonical loadout/suite as A: armor OnAttack17; future B removes A's skill and restores base sword"));
    cases.push(("host_ab",registry(host_mods,&["base","host_a","host_b"],json!([])),
        "same canonical loadout/suite: independent sword13 and armor17 sites in one bucket"));
    pack(root,"update_dependent",CatalogKind::Skills,"SkillDatabase(skills:[])",&["official"],vec![])?;
    let mut update_mods=vec![base.clone(),mod_entry("a",&["a_skills","host_a_item"],&["base"]),
        mod_entry("b",&["b_skills","host_b_item"],&["base"]),n.clone(),
        mod_entry("bad_parse",&["broken_manifest"],&["base"]),
        mod_entry("dependent",&["update_dependent"],&["bad_parse"])];
    cases.push(("update_initial_a",registry(update_mods.clone(),&["base","a"],json!([])),
        "normal update initial A; registered individual contexts remain fixed across frozen update"));
    cases.push(("update_parse_held",registry(update_mods.clone(),&["base","bad_parse","dependent","b"],json!([])),
        "malformed mod held, dependent held, independent B validated; exact reduced [base,b] requires its actual final job"));
    update_mods.last_mut().unwrap()["dependencies"]=json!(["noop"]);
    cases.push(("update_runtime_held",registry(update_mods,&["base","noop","dependent","b"],json!([])),
        "normal host loadout does not consume legal noop equipment; actual Incomplete held, dependent held, independent B and reduced final still checked"));
    let winner_mods=vec![base.clone(),mod_entry("a",&["a_skills","host_a_item"],&["base"]),
        mod_entry("b",&["b_skills","host_b_item"],&["base"]),
        mod_entry("left",&["replace_left"],&["a"]),
        mod_entry("right",&["replace_right","unconsumed_numeric_relic"],&["a"])];
    cases.push(("update_missing_combination",registry(winner_mods,&["base","a","left","right","b"],
        json!([{"target":{"kind":"Skill","id":"u11_damage_a"},"pack_id":"replace_right"}])),
        "individual applicable winners allow base/A/left/B; right publishes but its valid unconsumed numeric relic holds it Incomplete; final original right winner is stale, so no final job or wrapper PASS"));
    for (tier,_) in workload_tiers {
        let a_id=format!("load_{tier}_a");let b_id=format!("load_{tier}_b");
        for (suffix,selected) in [("a",vec!["base",a_id.as_str()]),("b",vec!["base",b_id.as_str()]),
            ("ab",vec!["base",a_id.as_str(),b_id.as_str()])] {
            // Save these generated names separately because the fixed cases
            // below deliberately retain simple literal names for diagnostics.
            let id=format!("load_{tier}_{suffix}");
            write_json(&root.join(format!("registries/{id}.json")),&registry(normal.clone(),&selected,json!([])))?;
        }
        let a_skills=format!("{a_id}_skills");let b_skills=format!("{b_id}_skills");
        let a_item=format!("host_{a_id}_item");let b_item=format!("host_{b_id}_item");
        let host_mods=vec![base.clone(),mod_entry("load_a",&[&a_skills,&a_item],&["base"]),
            mod_entry("load_b",&[&b_skills,&b_item],&["base"])];
        for (suffix,selected) in [("a",vec!["base","load_a"]),("b",vec!["base","load_b"]),
            ("ab",vec!["base","load_a","load_b"])] {
            write_json(&root.join(format!("registries/host_load_{tier}_{suffix}.json")),
                &registry(host_mods.clone(),&selected,json!([])))?;
        }
    }
    let mut removal=normal.clone();removal.push(mod_entry("remove_a",&["remove_a"],&["a"]));
    cases.push(("remove_a",registry(removal,&["base","a","remove_a"],json!([])),
        "against host-pinned official+A base, removed inventory must be exact Skill+Equipment; previous A stays immutable"));
    let mut broken = normal.clone();
    broken[1] = mod_entry("a", &["a_skills","broken_item"], &["base"]);
    cases.push(("broken_member", registry(broken, &["base","a","b"], json!([])), "mod A held; independent B remains independently checkable"));
    let mut broken = normal.clone();
    broken.push(mod_entry("bad_parse", &["broken_manifest"], &["base"]));
    cases.push(("bad_parse_independent", registry(broken, &["base","b","bad_parse"], json!([])), "acquisition/parse failure belongs to bad_parse; B not poisoned"));
    let mut duplicate = normal.clone(); duplicate.push(a.clone());
    cases.push(("duplicate_mod", registry(duplicate, &["base","a"], json!([])), "reject duplicate mod identity"));
    let mut duplicate = normal.clone(); duplicate.push(mod_entry("copy", &["a_skills"], &["base"]));
    cases.push(("duplicate_owner", registry(duplicate, &["base","a","copy"], json!([])), "reject pack with two mod owners"));
    let mut missing = normal.clone(); missing[1]["dependencies"] = json!(["absent"]);
    cases.push(("missing_dependency", registry(missing, &["base","a"], json!([])), "dependency-held or typed missing dependency; no approval"));
    let mut cycle = normal.clone(); cycle[1]["dependencies"] = json!(["b"]); cycle[2]["dependencies"] = json!(["a"]);
    cases.push(("cycle", registry(cycle, &["base","a","b"], json!([])), "reject cycle; not performance failure"));
    let mut forged_base = normal.clone(); forged_base[0]["packs"] = json!(["b_skills"]); forged_base[2]["packs"] = json!(["official","b_item"]);
    cases.push(("forged_base", registry(forged_base, &["base","b"], json!([])), "name base grants no trusted authority"));
    let mut spoof = registry(normal.clone(), &["base","a"], json!([])); spoof["passed"] = json!(true);
    cases.push(("provided_pass", spoof, "strict registry rejects mod-provided approval field"));
    let mut writers = normal.clone();
    writers.push(mod_entry("left", &["replace_left"], &["a"]));
    writers.push(mod_entry("right", &["replace_right"], &["a"]));
    let winner = json!([{"target":{"kind":"Skill","id":"u11_damage_a"},"pack_id":"replace_right"}]);
    cases.push(("writer_right", registry(writers.clone(), &["base","a","left","right"], winner.clone()), "right writer selected; requested amount31"));
    cases.push(("writer_ambiguous", registry(writers.clone(), &["base","a","left","right"], json!([])), "reject ambiguous writers"));
    let duplicate_winners = json!([
        {"target":{"kind":"Skill","id":"u11_damage_a"},"pack_id":"replace_left"},
        {"target":{"kind":"Skill","id":"u11_damage_a"},"pack_id":"replace_right"}
    ]);
    cases.push(("writer_duplicate_target", registry(writers.clone(), &["base","a","left","right"], duplicate_winners), "reject contradictory explicit writer target before normalization"));
    cases.push(("writer_stale", registry(writers, &["base","a","left"], winner), "removed right selection cannot leave its writer/effect behind"));
    let mut descriptions = vec![];
    for (id, value, oracle) in cases {
        write_json(&root.join(format!("registries/{id}.json")), &value)?;
        descriptions.push(json!({"case":id,"registry":format!("registries/{id}.json"),"oracle":oracle}));
    }
    write_json(&root.join("INDEPENDENT_ORACLES.json"), &json!({
        "schema":"u11.fixture-oracles/v1","prepared_only":true,"cases":descriptions,
        "same_bucket":{"driver":"one actor wearing u11_weapon_a and u11_armor_b; actual OnAttack producer",
            "sites":[{"definition":"u11_damage_a","action":"u11_a_strike","effect":"u11_a_damage","requested_damage":13},
                     {"definition":"u11_damage_b","action":"u11_b_strike","effect":"u11_b_damage","requested_damage":17}],
            "max_triggers_per_battle_each":1,"required":"both independent actual source/site witnesses, not latest-only or global sum"},
        "noop_sites":[{"definition":"u11_full_health_heal","effect":"u11_heal_seven","requested_heal":7},
                      {"definition":"u11_zero_damage","effect":"u11_zero_effect","requested_damage":0}],
        "structural":{"skill":"u11_structural_strike","equipment":"u11_structural_weapon","relic":"u11_structural_relic",
            "original_requested":30,"generated_requested":75,"sum_if_both_consumed":105,
            "required":"nonlethal target; distinct original/generated effect-site and actual HP-loss witnesses"},
        "causal":{"skill":"u11_causal_strike","equipment":"u11_causal_weapon","relic":"u11_causal_relic",
            "authored_direct":[10,10,20],"echo_per_eligible_direct":7,"join_branch_amounts":[2,3],"join_continuation":5,
            "periodic_interval_ms":1000,"periodic_health_cost":3,
            "required":"source-tag exclusion; actual wedge amount progression; both branches once before each continuation; release on battle end"},
        "objects":{"skill":"u11_object_create","equipment":"u11_object_weapon","definition":"u11_finite_actor",
            "placement_offset":{"x":1,"y":0},"ttl_ms":250,"required":"eligible free placement; producer spawn and terminal/reclaim witness, not final active snapshot alone"},
        "workload_batches":workload_oracles,
        "workload_registry_pattern":"registries/load_{small|expanded|pathological}_{a|b|ab}.json",
        "host_registry_cases":{"normal":["host_a","host_b","host_ab"],
            "load_pattern":"host_load_{small|expanded|pathological}_{a|b|ab}",
            "shared_equipment":["standard_training_sword","standard_armor"],
            "purpose":"identical trusted suite/loadout IDs across A, B and combination; selected mods replace distinct base equipment and own their skill pack once"},
        "workload_policy":"measure all actual sites, positive effect and legal zero branches with eligible nonlethal target; tiers are authored batch sizes only, no pass/fail threshold or calibrated class is asserted",
        "not_claimed":["successful preparation of a battle","runtime same-bucket coverage","mod approval","measured criteria","combination failure"]
    }))?;
    let files = inventory(root)?;
    write_json(&root.join("FIXTURE_RECEIPT.json"), &json!({"schema":"u11.fixture-preparation/v1",
        "source_root":source,"fixture_root":root,"files":files,"runtime_executed":false}))?;
    println!("{}", json!({"fixture_prepared":true,"runtime_executed":false,"root":root}));
    Ok(())
}
