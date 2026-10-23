//! External public consumer and finite diagnostic. This graph issues no approval.
use game_core::{application::{ApplicationCheckpoint,ApplicationCommand,ApplicationExecutionContext,ApplicationSnapshotKind,CoreApplication},game::{data::{ValidatedGameData,content_pack},map::*}};
use serde_json::{json,Value};
use std::{error::Error,fs,path::Path,time::Instant};
use uuid::Uuid;
type Result<T> = std::result::Result<T,Box<dyn Error>>;
const PLAYER:Uuid=Uuid::from_u128(13031);
#[global_allocator]
static ALLOCATOR:game_server::runtime_measurement::CountingAllocator<'static,std::alloc::System>=game_server::runtime_measurement::process_allocator();

fn data(case:&str)->Result<ValidatedGameData> {
    if case=="legacy" {
        let root=std::env::var_os("U13_LEGACY_CONTENT").ok_or("U13_LEGACY_CONTENT required")?;
        let selection=content_pack::ContentSelection{packs:vec!["official".into()],winners:vec![]};
        return Ok(content_pack::load(Path::new(&root),&selection,&content_pack::InputLimits::default())?);
    }
    let root=std::env::var_os("U13_PROBE_INPUTS").ok_or("U13_PROBE_INPUTS external root required")?;
    let selection=content_pack::ContentSelection{packs:vec!["official".into()],winners:vec![]};
    Ok(content_pack::load(&Path::new(&root).join(case).join("content"),&selection,&content_pack::InputLimits::default())?)
}
fn edit_catalog(case:&str,db:&mut FacilityTemplateDatabase)->Result<()> {
    let layout=&mut db.layouts[0];
    if case=="expanded" {
        let rooms=layout.rooms.clone();let edges=layout.connections.clone();
        for mut room in rooms.into_iter().filter(|r|r.depth>0) {
            room.id=format!("X{}",room.id);
            room.exclusive_approach_room_id=room.exclusive_approach_room_id.map(|id|format!("X{id}"));
            layout.rooms.push(room);
        }
        for mut edge in edges {
            if edge.from!="ENTRY" {edge.from=format!("X{}",edge.from);}
            edge.to=format!("X{}",edge.to);layout.connections.push(edge);
        }
    } else if case=="movement" {
        layout.id="u13_public_movement".into();
        let base=layout.room("B1-01").unwrap().clone();
        let mut rooms=vec![layout.room("ENTRY").unwrap().clone()];
        for (id,depth) in [("B1-01",1),("B1-02",1),("B2-01",2),("B3-01",3)] {
            let mut room=base.clone();room.id=id.into();room.name=id.into();room.depth=depth;
            room.reservations=vec![FacilityReservation{slot_id:MapSlotId::new("2"),activity_id:"save".into()},FacilityReservation{slot_id:MapSlotId::new("5"),activity_id:"maintenance".into()},FacilityReservation{slot_id:MapSlotId::new("8"),activity_id:"shop".into()}];
            room.omen_slot=(depth==3).then(||MapSlotId::new("3"));
            room.elite_slots.clear();room.safe_access_slots=vec![MapSlotId::new("2"),MapSlotId::new("5"),MapSlotId::new("8")];rooms.push(room);
        }
        layout.rooms=rooms;
        layout.connections=[("ENTRY","B1-01",MapEdgeDirection::ForwardOnly),("B1-01","B1-02",MapEdgeDirection::Bidirectional),("B1-01","B2-01",MapEdgeDirection::ForwardOnly),("B2-01","B3-01",MapEdgeDirection::ForwardOnly)].into_iter().map(|(a,b,d)|FacilityRoomConnection{from:a.into(),to:b.into(),direction:d,condition:None}).collect();
        for size in &mut layout.allocation.sizes {for weight in size.elite_percent_by_depth.values_mut(){*weight=0;}}
    } else {return Err("unknown bounded input case".into());}
    Ok(())
}
fn copy_directory(source:&Path,destination:&Path)->Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry=entry?;let target=destination.join(entry.file_name());
        if entry.file_type()?.is_dir(){copy_directory(&entry.path(),&target)?;}
        else if entry.file_type()?.is_file(){fs::copy(entry.path(),target)?;}
        else {return Err("only regular source content accepted".into());}
    }Ok(())
}
fn prepare(source:&Path,out:&Path)->Result<()> {
    fs::create_dir(out)?;
    for case in ["official","expanded","movement"] {
        let base=out.join(case);fs::create_dir(&base)?;
        let copied=base.join("source");copy_directory(source,&copied)?;
        if case!="official" {
            let path=copied.join("data/map/facility_templates.ron");
            let mut db=FacilityTemplateDatabase::from_ron_str(&fs::read_to_string(&path)?)?;
            edit_catalog(case,&mut db)?;fs::write(path,ron::ser::to_string(&db)?)?;
        }
        fs::create_dir(base.join("content"))?;
        let receipt=content_pack::package_official(&copied,&base.join("content/official"),&content_pack::InputLimits::default())?;
        write(&base.join("PACKAGE.json"),&serde_json::to_value(receipt)?)?;
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
fn enter(app:&mut CoreApplication,slot:&str)->Result<()> {
    let view=run(app)?;
    let id=view["map"]["nodes"].as_array().ok_or("map")?.iter().find(|n|n["slot_id"]==slot).ok_or("slot")?["id"].clone();
    execute(app,json!({"type":"select_map_node","node_id":id}))?;
    execute(app,json!({"type":"confirm_enter_node"}))?;
    if !run(app)?["run_progression"]["pending_continuation"].is_null(){execute(app,json!({"type":"choose_expedition_continuation","choice":"Continue"}))?;}
    Ok(())
}
fn gate(app:&mut CoreApplication)->Result<()> {
    for (slot,finish) in [("2","complete_node"),("5","complete_node"),("8","exit_shop")] {enter(app,slot)?;execute(app,json!({"type":finish}))?;}
    enter(app,"G")?;assert_eq!(run(app)?["facility"]["gate_reached"],true);Ok(())
}
fn move_to(app:&mut CoreApplication,id:&str)->Result<()> {execute(app,json!({"type":"move_to_facility_room","room_id":id}))?;Ok(())}
fn write(path:&Path,value:&Value)->Result<()> {use std::io::Write;let mut f=fs::OpenOptions::new().write(true).create_new(true).open(path)?;f.write_all(&serde_json::to_vec_pretty(value)?)?;Ok(())}
fn restore(saved:&Value)->Result<CoreApplication> {
    let mut app=CoreApplication::new(data("movement")?,13031);
    app.install_authoritative_player_profile(saved["profile"].clone().into())?;
    let c=&saved["checkpoint"];
    let checkpoint=ApplicationCheckpoint::new(serde_json::from_value(c["bytes"].clone())?,c["loads"].as_u64().ok_or("loads")? as u8,c["peak"].as_u64().ok_or("peak")?);
    app.continue_from_active_run_checkpoint(PLAYER,&checkpoint)?;Ok(app)
}
fn route(out:&Path)->Result<()> {
    fs::create_dir(out)?;let mut app=CoreApplication::new(data("movement")?,13031);start(&mut app)?;
    let initial=roots(&app)?;
    assert!(execute(&mut app,json!({"type":"move_to_facility_room","room_id":"B1-02"})).is_err());assert_eq!(roots(&app)?,initial);
    gate(&mut app)?;
    let before=roots(&app)?;
    let prepared=app.prepare_command(PLAYER,ApplicationCommand::from_document(json!({"type":"move_to_facility_room","room_id":"B1-02"}).into())?,Default::default())?;drop(prepared);
    assert_eq!(roots(&app)?,before,"discarded room movement must preserve every public root");
    move_to(&mut app,"B1-02")?;gate(&mut app)?;move_to(&mut app,"B1-01")?;
    let saved=json!({"profile":snapshot(&app,ApplicationSnapshotKind::PlayerProfile)?,"checkpoint":checkpoint(&app)?,"roots":roots(&app)?});
    write(&out.join("saved.json"),&saved)?;
    let status=std::process::Command::new(std::env::current_exe()?).arg("resume").arg(out.join("saved.json")).arg(out.join("resumed.json")).status()?;
    assert!(status.success());
    let resumed:Value=serde_json::from_slice(&fs::read(out.join("resumed.json"))?)?;assert_eq!(resumed["roots"],saved["roots"]);
    move_to(&mut app,"B2-01")?;gate(&mut app)?;move_to(&mut app,"B3-01")?;
    let knowledge=snapshot(&app,ApplicationSnapshotKind::PlayerProfile)?["facility_knowledge"].clone();
    execute(&mut app,json!({"type":"load_run_checkpoint"}))?;
    assert_eq!(snapshot(&app,ApplicationSnapshotKind::PlayerProfile)?["facility_knowledge"],knowledge);
    assert_eq!(run(&app)?["facility"]["current_room_id"],"B2-01");
    enter(&mut app,"5")?;execute(&mut app,json!({"type":"complete_node"}))?;enter(&mut app,"8")?;execute(&mut app,json!({"type":"exit_shop"}))?;enter(&mut app,"G")?;
    move_to(&mut app,"B3-01")?;gate(&mut app)?;execute(&mut app,json!({"type":"choose_expedition_continuation","choice":"Return"}))?;
    assert!(checkpoint(&app)?.is_null());
    write(&out.join("RESULT.json"),&json!({"passed":true,"approval_claimed":false,"fresh_process":true,"prepared_discard_unchanged":true,"final_roots":roots(&app)?}))
}
#[derive(serde::Serialize)]
struct Stage {
    name: &'static str,
    seconds: f64,
    heap: game_server::runtime_measurement::HeapWindow,
    native: game_core::runtime_observation::OperationSummary,
    ok: bool,
}
fn measured<T>(name:&'static str,stages:&mut Vec<Stage>,f:impl FnOnce()->Result<T>)->Result<T> {
    let observation=game_core::runtime_observation::begin_lab_window().ok_or("observation already active")?;
    let heap=game_server::runtime_measurement::begin_process_window()?;let start=Instant::now();let result=f();let seconds=start.elapsed().as_secs_f64();
    let heap=heap.finish();let native=observation.finish().ok_or("missing observation")?;
    stages.push(Stage{name,seconds,heap,native,ok:result.is_ok()});result
}
fn measure(case:&str,out:&Path)->Result<()> {
    // Warm global loader/observer state before the paired owner lifetime window.
    let mut warm=CoreApplication::new(data("official")?,1);start(&mut warm)?;drop(warm);let mut stages=Vec::with_capacity(8);
    let data=measured("load",&mut stages,||data(case))?;
    let baseline=game_server::runtime_measurement::process_heap();
    let mut app=measured("new",&mut stages,||Ok(CoreApplication::new(data.clone(),13031)))?;
    measured("start",&mut stages,||start(&mut app))?;
    let view=measured("whole_facility_query",&mut stages,||run(&app))?;
    let count=view["facility"]["rooms"].as_array().ok_or("facility rooms")?.len();drop(view);
    let saved=measured("checkpoint_export",&mut stages,||checkpoint(&app))?;drop(saved);
    measured("drop_application",&mut stages,||{drop(app);Ok(())})?;
    let after=game_server::runtime_measurement::process_heap();
    write(out,&json!({"passed":true,"case":case,"approval_claimed":false,"scope":"normal default limits; initial room manifested, full static graph loaded and disclosed according to knowledge","visible_rooms":count,"baseline":baseline,"after":after,"exact_requested_bytes_recovered":baseline.live_requested_bytes==after.live_requested_bytes,"stages":stages,"rss":game_server::runtime_measurement::process_rss()}))
}
struct BattleHost {app:CoreApplication,data:ValidatedGameData}
impl u12_independent_lifecycle_probe::Host for BattleHost {
    fn app(&self)->&CoreApplication{&self.app}
    fn command(&mut self,document:Value,context:ApplicationExecutionContext)->Result<Value>{Ok(self.app.execute(PLAYER,ApplicationCommand::from_document(document.into())?,context)?.payload().as_value().clone())}
    fn tick(&mut self,delta_ms:u64,after_seq:u64)->Result<Option<Value>>{Ok(self.app.prepare_active_battle_for_host_tick(delta_ms,after_seq)?.commit().map(|r|r.payload().as_value().clone()))}
    fn restart(&mut self)->Result<()>{
        let checkpoint=self.app.export_active_run_checkpoint()?.ok_or("checkpoint")?;
        let profile=snapshot(&self.app,ApplicationSnapshotKind::PlayerProfile)?;
        let mut next=CoreApplication::new(self.data.clone(),13031);next.install_authoritative_player_profile(profile.into())?;
        next.continue_from_active_run_checkpoint(PLAYER,&checkpoint)?;self.app=next;Ok(())
    }
}
fn battle(out:&Path)->Result<()> {
    fs::create_dir(out)?;let data=data("movement")?;
    let mut host=BattleHost{app:CoreApplication::new(data.clone(),13031),data};start(&mut host.app)?;
    // The development profile uses an actual supported SavePoint at departure.
    // Official relic activity execution belongs to U16 and is not substituted.
    enter(&mut host.app,"2")?;execute(&mut host.app,json!({"type":"complete_node"}))?;
    let before=run(&host.app)?;
    let node=before["map"]["nodes"].as_array().ok_or("map")?.iter().find(|n|n["state"]=="Available" && n["category"]=="Combat").ok_or("bounded official departure needs an available ordinary combat")?;
    execute(&mut host.app,json!({"type":"select_map_node","node_id":node["id"]}))?;
    write(&out.join("BRIEFING.json"),&run(&host.app)?)?;
    execute(&mut host.app,json!({"type":"confirm_enter_node"}))?;
    let result=u12_independent_lifecycle_probe::walk(&mut host,"Expedition","battle",&out.join("trace.jsonl"));
    write(&out.join("RESULT.json"),&json!({"passed":result.is_ok(),"approval_claimed":false,"result":result.as_ref().ok(),"error":result.as_ref().err().map(ToString::to_string),"final_roots":roots(&host.app)?}))?;result?;Ok(())
}
fn read_profile(root:&Path)->Result<game_core::game::encounter_compendium::PlayerProfileState> {
    use game_server::persistence::player_profile_store::{PlayerProfileStore,PlayerProfileLoad};
    match PlayerProfileStore::open(root)?.load()? {PlayerProfileLoad::Loaded{profile,..}=>Ok(profile),_=>Err("profile missing".into())}
}
fn legacy(input:&Path,knowledge:&Path,out:&Path)->Result<()> {
    use game_server::persistence::player_profile_store::PlayerProfileStore;
    fs::create_dir(out)?;let saved:Value=serde_json::from_slice(&fs::read(input)?)?;
    let profile_root=out.join("profile");fs::create_dir(&profile_root)?;
    let original=fs::read(saved["profile_path"].as_str().ok_or("original profile path")?)?;
    fs::write(profile_root.join("player_profile.json"),&original)?;
    let old=read_profile(&profile_root)?;assert!(old.facility_knowledge.is_empty());
    assert_eq!(fs::read(profile_root.join("player_profile.json"))?,original);
    let checkpoint=ApplicationCheckpoint::new(serde_json::from_value(saved["bytes"].clone())?,saved["loads"].as_u64().ok_or("loads")? as u8,saved["peak"].as_u64().ok_or("peak")?);
    let player:Uuid=serde_json::from_value(saved["player"].clone())?;
    let mut app=CoreApplication::new(data("legacy")?,12077);app.install_authoritative_player_profile(serde_json::to_value(&old)?.into())?;
    app.continue_from_active_run_checkpoint(player,&checkpoint)?;
    assert!(run(&app)?["facility"].is_null());
    let mut changed=old.clone();let actual:Value=serde_json::from_slice(&fs::read(knowledge)?)?;
    changed.facility_knowledge=serde_json::from_value(actual["profile"]["facility_knowledge"].clone())?;
    assert!(!changed.facility_knowledge.is_empty());
    let store=PlayerProfileStore::open(&profile_root)?;
    // Real create/write failure: a directory occupies the owned temporary-file path.
    fs::create_dir(profile_root.join("player_profile.tmp"))?;
    assert!(store.publish(&changed).is_err());assert_eq!(fs::read(profile_root.join("player_profile.json"))?,original);
    fs::remove_dir(profile_root.join("player_profile.tmp"))?;
    store.publish(&changed)?;drop(store);
    let written:Value=serde_json::from_slice(&fs::read(profile_root.join("player_profile.json"))?)?;assert_eq!(written["version"],5);
    let status=std::process::Command::new(std::env::current_exe()?).arg("profile-read").arg(&profile_root).arg(out.join("profile-resumed.json")).status()?;assert!(status.success());
    let fresh:Value=serde_json::from_slice(&fs::read(out.join("profile-resumed.json"))?)?;assert_eq!(fresh,serde_json::to_value(&changed)?);
    write(&out.join("RESULT.json"),&json!({"passed":true,"approval_claimed":false,"genuine_old_writer_version":12,"genuine_profile_version":4,"profile_current_version":5,"actual_io_failure_preserved_original":true,"retry_and_new_process":true,"run":run(&app)?}))
}
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
        if let Err(err)=packaged {summary.push(json!({"case":case,"loader_rejected":true,"error":err.to_string()}));continue;}
        let selected=content_pack::ContentSelection{packs:vec!["official".into()],winners:vec![]};
        let loaded=content_pack::load(&content,&selected,&Default::default());
        let loaded=match loaded {Ok(data)=>data,Err(err)=>{summary.push(json!({"case":case,"loader_rejected":true,"error":err.to_string()}));continue;}};
        let mut app=CoreApplication::new(loaded,13031);start(&mut app)?;
        let snapshot=run(&app)?;
        write(&case_dir.join("run.json"),&snapshot)?;
        let queried=execute(&mut app,json!({"type":"request_map_data"}))?;
        write(&case_dir.join("map_query.json"),&queried)?;
        assert_eq!(snapshot,run(&app)?,"query must not modify state");
        let room=snapshot["facility"]["rooms"].as_array().unwrap().iter().find(|r|r["id"]=="B1-01").unwrap();
        if case.starts_with("hidden") {
            let node=snapshot["map"]["nodes"].as_array().unwrap().iter().find(|n|n["slot_id"]=="5").unwrap();
            let activity=room["activities"].as_array().unwrap().iter().find(|a|a["node_id"]==node["id"]).unwrap();
            let facility_node=room["current_expedition_map"]["nodes"].as_array().unwrap().iter().find(|n|n["id"]==node["id"]).unwrap();
            summary.push(json!({"case":case,"loader_rejected":false,"explored":room["explored"],"public_map_node":node,"facility_map_node":facility_node,"activity":activity}));
        } else {
            let nodes=snapshot["map"]["nodes"].as_array().unwrap();
            let counts=nodes.iter().fold(std::collections::BTreeMap::<String,usize>::new(),|mut counts,n|{*counts.entry(n["category"].as_str().unwrap_or("unknown").to_owned()).or_default()+=1;counts});
            summary.push(json!({"case":case,"loader_rejected":false,"categories":counts}));
        }
    }
    write(&out.join("RESULT.json"),&json!({"target":"u13_current_001","observations":summary}))?;
    println!("{}",serde_json::to_string_pretty(&summary)?);Ok(())
}
