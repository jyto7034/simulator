//! Separate trusted laboratory base: nonlethal observation targets. This never
//! alters the original packaged fixture, chooses criteria, or grants approval.
use game_core::game::data::corroded_employee_data::CorrodedEmployeeProfileDatabase;
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{error::Error,fs,path::Path};
use u11_independent_public_probe::normal_preparation::save;

fn copy(source:&Path,target:&Path)->Result<(),Box<dyn Error>> {
    fs::create_dir(target)?;
    for entry in fs::read_dir(source)? {let entry=entry?;
        if entry.file_type()?.is_dir(){copy(&entry.path(),&target.join(entry.file_name()))?;}
        else if entry.file_type()?.is_file(){fs::copy(entry.path(),target.join(entry.file_name()))?;}
        else{return Err("fixture source contains non-regular member".into());}}
    Ok(())
}
fn inventory(root:&Path,current:&Path,rows:&mut Vec<Value>)->Result<(),Box<dyn Error>> {
    let mut entries=fs::read_dir(current)?.collect::<Result<Vec<_>,_>>()?;entries.sort_by_key(|e|e.file_name());
    for entry in entries {let path=entry.path();if entry.file_type()?.is_dir(){inventory(root,&path,rows)?;}
        else if entry.file_type()?.is_file(){let bytes=fs::read(&path)?;
            rows.push(json!({"path":path.strip_prefix(root)?.to_string_lossy(),"bytes":bytes.len(),"sha256":format!("{:x}",Sha256::digest(&bytes))}));}
        else{return Err("fixture output contains non-regular member".into());}}
    Ok(())
}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()!=2{return Err("usage: prepare_lab_registry ORIGINAL_FIXTURE_ROOT NEW_LAB_FIXTURE_ROOT".into());}
    let source=Path::new(&args[0]);let root=Path::new(&args[1]);copy(source,root)?;
    fs::rename(root.join("FIXTURE_RECEIPT.json"),root.join("ORIGINAL_FIXTURE_RECEIPT.json"))?;
    let path=root.join("official/data/enemies/corroded_employees.ron");
    let original=fs::read(&path)?;
    let mut database:CorrodedEmployeeProfileDatabase=ron::de::from_bytes(&original)?;
    let mut profiles=Vec::new();
    for profile in &mut database.profiles {
        profiles.push(profile.id.clone());
        profile.max_health=10_000;profile.attack=0;profile.defense=0;profile.magic_resist=0;
        profile.basic_attack.interval_ms=60_000;profile.basic_attack.windup_ms=60_000;
        profile.skill_id=None;
    }
    if profiles.is_empty(){return Err("independent target catalog is empty".into());}
    let expected_resolved=serde_json::to_value(&database)?;
    let mut preset_authorities=Vec::new();
    for profile in &mut database.profiles {
        if let Some(preset)=&profile.basic_attack_range_preset {
            preset_authorities.push(json!({"profile":profile.id,"preset":preset,
                "resolved_range":profile.basic_attack.defense_tile_range}));
            // Deserialization resolves the preset into this runtime field. The
            // authored successor must retain only its original preset authority.
            profile.basic_attack.defense_tile_range=None;
        }
    }
    let replacement=ron::ser::to_string(&database)?;
    let round_trip:CorrodedEmployeeProfileDatabase=ron::de::from_str(&replacement)?;
    assert_eq!(serde_json::to_value(&round_trip)?,expected_resolved,
        "authored serialization must recover the exact intended nonlethal profiles and original resolved ranges");
    fs::write(&path,replacement.as_bytes())?;
    save(&root.join("LAB_BASE_ORACLE.json"),&json!({"schema":"u11.lab-base-fixture/v1",
        "original_file_sha256":format!("{:x}",Sha256::digest(&original)),
        "replacement_file_sha256":format!("{:x}",Sha256::digest(replacement.as_bytes())),"profiles":profiles,
        "range_authority":"Original preset retained; resolved runtime range omitted only in serialized authoring input",
        "preset_authorities":preset_authorities,"resolved_round_trip_exact":true,
        "authored_target":{"max_health":10_000,"attack":0,"defense":0,"magic_resist":0,
            "interval_ms":60_000,"windup_ms":60_000,"skill_id":null},
        "purpose":"nonlethal exact damage, full-health legal heal no-op and finite cleanup observations",
        "constraint":"independent coverage fixtures only; these base changes are explicit exact trusted input",
        "runtime_executed":false,"criterion_selected":false}))?;
    let mut files=Vec::new();inventory(root,root,&mut files)?;
    save(&root.join("FIXTURE_RECEIPT.json"),&json!({"schema":"u11.fixture-preparation/v1",
        "source_root":source,"fixture_root":root,"files":files,"runtime_executed":false}))?;
    println!("{}",json!({"fixture_prepared":true,"runtime_executed":false,"root":root}));Ok(())
}
