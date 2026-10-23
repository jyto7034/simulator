//! External public consumer and finite diagnostic. This graph issues no approval.
use game_core::{application::{ApplicationCommand,ApplicationExecutionContext,ApplicationSnapshotKind,CoreApplication},game::{data::{content_pack},map::*}};
use serde_json::{json,Value};
use std::{error::Error,fs,path::Path};
use uuid::Uuid;
type Result<T> = std::result::Result<T,Box<dyn Error>>;
const PLAYER:Uuid=Uuid::from_u128(13031);
#[global_allocator]
static ALLOCATOR:game_server::runtime_measurement::CountingAllocator<'static,std::alloc::System>=game_server::runtime_measurement::process_allocator();

fn copy_directory(source:&Path,destination:&Path)->Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry=entry?;let target=destination.join(entry.file_name());
        if entry.file_type()?.is_dir(){copy_directory(&entry.path(),&target)?;}
        else if entry.file_type()?.is_file(){fs::copy(entry.path(),target)?;}
        else {return Err("only regular source content accepted".into());}
    }Ok(())
}

fn snapshot(app:&CoreApplication,kind:ApplicationSnapshotKind)->Result<Value>{Ok(app.snapshot(kind)?.read_model().as_value().clone())}
fn run(app:&CoreApplication)->Result<Value>{snapshot(app,ApplicationSnapshotKind::Run)}
fn execute(app:&mut CoreApplication,input:Value)->Result<Value>{
    let trace=std::env::var_os("U13_COMMAND_TRACE");
    let before=if trace.is_some(){Some(run(app)?)}else{None};
    let result=app.execute(PLAYER,ApplicationCommand::from_document(input.clone().into())?,ApplicationExecutionContext::default())?.payload().as_value().clone();
    if let Some(path)=trace {use std::io::Write;let mut file=fs::OpenOptions::new().create(true).append(true).open(path)?;writeln!(file,"{}",json!({"input":input,"run_before":before,"result":result,"run_after":run(app)?}))?;}
    Ok(result)
}
fn checkpoint(app:&CoreApplication)->Result<Value>{Ok(app.export_active_run_checkpoint()?.map(|c|json!({"bytes":c.core_payload_json(),"loads":c.loads_used(),"peak":c.run_peak_certified_power()})).unwrap_or(Value::Null))}
fn roots(app:&CoreApplication)->Result<Value>{Ok(json!({"snapshots":ApplicationSnapshotKind::ALL.iter().map(|k|snapshot(app,*k)).collect::<Result<Vec<_>>>()?,"continue":checkpoint(app)?}))}
fn start(app:&mut CoreApplication)->Result<()> {
    execute(app,json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}))?;
    for _ in 0..32 {
        let view=run(app)?;let vn=&view["visual_novel"];
        if !vn.is_null(){execute(app,json!({"type":"advance_visual_novel","session_id":vn["session_id"],"current_scene_id":vn["current_scene_id"]}))?;continue;}
        let candidates=view["game_state_context"]["candidates"].as_array().ok_or("starter selection required")?;
        let ids:Vec<_>=["Dealer","Tank","Healer"].iter().map(|role|candidates.iter().find(|r|r["role"]==*role).expect("required starter role")["employee_definition_id"].clone()).collect();
        execute(app,json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":[]}))?;return Ok(());
    }Err("bounded briefing did not finish".into())
}

fn write(path:&Path,value:&Value)->Result<()> {use std::io::Write;let mut f=fs::OpenOptions::new().write(true).create_new(true).open(path)?;f.write_all(&serde_json::to_vec_pretty(value)?)?;Ok(())}
fn main()->Result<()> {
    let out=std::env::args().nth(1).ok_or("output directory required")?;
    let out=Path::new(&out);fs::create_dir_all(out)?;
    let source=Path::new("/mnt/f/work/simulator/game_resources");
    let mut summary=vec![];
    for case in ["hidden_official", "hidden_concrete", "wrong_normal_role"] {
        let case_dir=out.join(case);fs::create_dir(&case_dir)?;
        let authored=case_dir.join("source");copy_directory(source,&authored)?;
        let path=authored.join("data/map/facility_templates.ron");
        let mut db=FacilityTemplateDatabase::from_ron_str(&fs::read_to_string(&path)?)?;
        let layout=&mut db.layouts[0];
        if case.starts_with("hidden") {
            let mut room=layout.room("A2-02").unwrap().clone();
            room.id="B1-01".into();room.depth=1;room.wing=Some("B".into());room.elite_slots.clear();room.safe_access_slots.clear();
            *layout.rooms.iter_mut().find(|r|r.id=="B1-01").unwrap()=room;
            if case=="hidden_concrete" {
                let a=layout.activities.iter_mut().find(|a|a.id=="reserved_A2-02").unwrap();
                a.payload=MapNodePayload::Encounter{encounter_id:Some("suppress_little_red".into())};
            }
        } else {layout.allocation.normal_activity_id="maintenance".into();}
        fs::write(path,ron::ser::to_string(&db)?)?;
        let content=case_dir.join("content");fs::create_dir(&content)?;
        let packaged=content_pack::package_official(&authored,&content.join("official"),&Default::default());
        if let Err(err)=packaged {assert_eq!(case,"wrong_normal_role");assert!(err.to_string().contains("allocation role 'normal'"));summary.push(json!({"case":case,"loader_rejected":true,"error":err.to_string()}));continue;}
        let selected=content_pack::ContentSelection{packs:vec!["official".into()],winners:vec![]};
        let loaded=content_pack::load(&content,&selected,&Default::default());
        let loaded=match loaded {Ok(data)=>data,Err(err)=>{assert_eq!(case,"wrong_normal_role");assert!(err.to_string().contains("allocation role 'normal'"));summary.push(json!({"case":case,"loader_rejected":true,"error":err.to_string()}));continue;}};
        assert_ne!(case,"wrong_normal_role","wrong role must be rejected");
        let mut app=CoreApplication::new(loaded,13031);start(&mut app)?;
        let before=roots(&app)?;
        let snapshot=run(&app)?;
        write(&case_dir.join("run.json"),&snapshot)?;
        let queried=execute(&mut app,json!({"type":"request_map_data"}))?;
        write(&case_dir.join("map_query.json"),&queried)?;
        assert_eq!(roots(&app)?,before,"query must not modify any public or persisted root");
        let room=snapshot["facility"]["rooms"].as_array().unwrap().iter().find(|r|r["id"]=="B1-01").unwrap();
        if case.starts_with("hidden") {
            let node=snapshot["map"]["nodes"].as_array().unwrap().iter().find(|n|n["slot_id"]=="5").unwrap();
            let activity=room["activities"].as_array().unwrap().iter().find(|a|a["node_id"]==node["id"]).unwrap();
            let facility_node=room["current_expedition_map"]["nodes"].as_array().unwrap().iter().find(|n|n["id"]==node["id"]).unwrap();
            assert_eq!(room["explored"],false);
            assert_eq!(node["payload"],json!({"Encounter":{"encounter_id":null}}));
            assert_eq!(node["payload"],facility_node["payload"]);
            let queried_node=queried["map"]["nodes"].as_array().unwrap().iter().find(|n|n["id"]==node["id"]).unwrap();
            assert_eq!(node["payload"],queried_node["payload"]);
            summary.push(json!({"case":case,"loader_rejected":false,"explored":room["explored"],"public_map_node":node,"facility_map_node":facility_node,"activity":activity}));
        } else {
            let nodes=snapshot["map"]["nodes"].as_array().unwrap();
            let counts=nodes.iter().fold(std::collections::BTreeMap::<String,usize>::new(),|mut counts,n|{*counts.entry(n["category"].as_str().unwrap_or("unknown").to_owned()).or_default()+=1;counts});
            summary.push(json!({"case":case,"loader_rejected":false,"categories":counts}));
        }
    }
    write(&out.join("RESULT.json"),&json!({"correction":"u13_correction_001","passed":true,"observations":summary}))?;
    println!("{}",serde_json::to_string_pretty(&summary)?);Ok(())
}
