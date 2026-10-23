#[path="/mnt/f/work/simulator/core_enhanced/docs/goals/core_expedition_deterrence_rewards/execution/single_goal/test_parts/u14_contact_v3.rs"]
mod contact;
fn prepare_sealed(source:&Path)->Result<()> {
    use game_core::game::data::pve_data::*;
    use game_core::game::reward::{RewardEffect,ExperienceTargetPolicy};
    let mut catalog=FacilityTemplateDatabase::from_ron_str(&fs::read_to_string(source.join("data/map/facility_templates.ron"))?).unwrap();
    let layout=&mut catalog.layouts[0];
    let entrance=layout.room("ENTRY").unwrap().clone();
    let mut room=layout.room("B4-02").unwrap().clone();
    room.id="B1-01".into();room.depth=1;room.omen_slot=None;room.elite_slots.clear();
    room.reservations=vec![FacilityReservation{slot_id:MapSlotId::new("1"),activity_id:"save".into()},
        FacilityReservation{slot_id:MapSlotId::new("2"),activity_id:"reserved_B4-02".into()},
        FacilityReservation{slot_id:MapSlotId::new("10"),activity_id:"reserved_B4-02".into()}];
    room.safe_access_slots=vec![MapSlotId::new("1")];
    layout.rooms=vec![entrance,room];
    layout.connections=vec![FacilityRoomConnection{from:"ENTRY".into(),to:"B1-01".into(),direction:MapEdgeDirection::ForwardOnly,condition:None}];
    for activity in &mut layout.activities {activity.golden_bough_candidate=false;}
    let activity=layout.activities.iter_mut().find(|a|a.id=="reserved_B4-02").unwrap();
    activity.first_entry_cost=Some(11);activity.golden_bough_candidate=true;activity.support=FacilityContentSupport::Supported;
    activity.payload=MapNodePayload::Encounter{encounter_id:Some("suppress_nothing_there".into())};
    for size in &mut layout.allocation.sizes {for weight in size.elite_percent_by_depth.values_mut(){*weight=0;}}
    let location=|slot:&str|GoldenBoughLocation{room_id:"B1-01".into(),slot_id:MapSlotId::new(slot)};
    layout.golden_bough_plans=vec![GoldenBoughPlan{id:"front".into(),weight:1,locations:vec![location("2")],deterrence_refill:8},
        GoldenBoughPlan{id:"front_and_side".into(),weight:1,locations:vec![location("2"),location("10")],deterrence_refill:8}];
    let catalog_text=ron::ser::to_string(&catalog).unwrap();fs::write(source.join("data/map/facility_templates.ron"),&catalog_text)?;
    let mut pve:PveEncounterDatabase=ron::de::from_str(&fs::read_to_string(source.join("data/pve/encounters.ron"))?).unwrap();
    let encounter=pve.encounters.iter_mut().find(|e|e.id=="suppress_nothing_there").unwrap();
    encounter.deterrence_tuning.seal_health_percent=180;encounter.deterrence_tuning.seal_attack_percent=170;
    encounter.waves.retain(|wave|wave.id=="primary_abnormality");
    encounter.waves[0].activation=PveWaveActivationData::at_time("primary_abnormality",0);
    encounter.waves[0].required_for_victory=true;
    if false {
        encounter.terminal_outcomes.iter_mut().find(|outcome|outcome.verdict==game_core::game::battle::types::BattleVerdict::Defeat).unwrap().condition=PveTerminalConditionData::TimeReached{time_ms:500};
    }

    encounter.bonus_objectives=vec![
        PveBonusObjective{id:"within_limit".into(),condition:PveBonusCondition::ClearWithin{time_ms:60_000},effects:vec![RewardEffect::GrantExperience{amount:7,target:ExperienceTargetPolicy::AliveRoster},RewardEffect::GrantEquipmentDust{amount:3}]},
        PveBonusObjective{id:"too_late".into(),condition:PveBonusCondition::ClearWithin{time_ms:1},effects:vec![RewardEffect::GrantExperience{amount:999,target:ExperienceTargetPolicy::AliveRoster}]},
        PveBonusObjective{id:"decisive".into(),condition:PveBonusCondition::DecisiveDamage{max_health_percent:25},effects:vec![RewardEffect::GrantEquipmentDust{amount:5}]},
    ];
    let pve_text=ron::ser::to_string(&pve).unwrap();fs::write(source.join("data/pve/encounters.ron"),&pve_text)?;
    let mut abnormalities:game_core::game::data::abnormality_data::AbnormalityDatabase=ron::de::from_str(&fs::read_to_string(source.join("data/abnormalities/base.ron"))?).unwrap();
    let boss=abnormalities.items.iter_mut().find(|a|a.id=="o-06-20_nothing_there").unwrap();
    boss.max_health=10;boss.attack=0;boss.defense=0;boss.magic_resist=0;
    let abno_text=ron::ser::to_string(&abnormalities).unwrap();fs::write(source.join("data/abnormalities/base.ron"),&abno_text)?;
    // This authored contact fixture exposes walkable arena floor as tactical
    // slots; official arena deployment restrictions remain unchanged.
    let path=source.join("data/map/battlefield_templates.ron");
    let text=fs::read_to_string(&path)?;
    let mut boss=false;let mut rows=false;let mut changed=String::new();
    for line in text.lines() {
        if line.contains("archetype:") {boss=line.contains("BossArena");}
        if line.contains("rows:") {rows=true;}
        if rows && boss && line.trim_start().starts_with('"') {changed.push_str(&line.replace('.',"P"));}else{changed.push_str(line);}
        changed.push('\n');if rows && line.trim()=="]," {rows=false;}
    }
    fs::write(path,changed)?;
    Ok(())
}
// External public consumer and finite diagnostic. This graph issues no approval.
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
        let root=std::env::var_os("U14_LEGACY_CONTENT").ok_or("U14_LEGACY_CONTENT required")?;
        let selection=content_pack::ContentSelection{packs:vec!["official".into()],winners:vec![]};
        return Ok(content_pack::load(Path::new(&root),&selection,&content_pack::InputLimits::default())?);
    }
    let root=std::env::var_os("U14_PROBE_INPUTS").ok_or("U14_PROBE_INPUTS external root required")?;
    let selection=content_pack::ContentSelection{packs:vec!["official".into()],winners:vec![]};
    Ok(content_pack::load(&Path::new(&root).join(case).join("content"),&selection,&content_pack::InputLimits::default())?)
}
fn edit_catalog(case:&str,db:&mut FacilityTemplateDatabase)->Result<()> {
    let layout=&mut db.layouts[0];
    if case=="expanded" {
        let rooms=layout.rooms.clone();let edges=layout.connections.clone();
        for mut room in rooms.into_iter().filter(|r|r.depth>0) {
            room.reservations.retain(|r| !layout.activities.iter().any(|a|a.id==r.activity_id && a.golden_bough_candidate));
            room.id=format!("X{}",room.id);
            room.exclusive_approach_room_id=room.exclusive_approach_room_id.map(|id|format!("X{id}"));
            layout.rooms.push(room);
        }
        for mut edge in edges {
            if edge.from!="ENTRY" {edge.from=format!("X{}",edge.from);}
            edge.to=format!("X{}",edge.to);layout.connections.push(edge);
        }
    } else if case=="growth" {
        prepare_growth_layout(layout)?;
    } else if case=="movement" {
        layout.id="u14_public_movement".into();layout.golden_bough_plans.clear();for a in &mut layout.activities {a.golden_bough_candidate=false;}
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
fn prepare(source:&Path,out:&Path,cases:&[&str])->Result<()> {
    fs::create_dir(out)?;
    for &case in cases {
        let base=out.join(case);fs::create_dir(&base)?;
        let copied=base.join("source");copy_directory(source,&copied)?;
        if case=="sealed" {prepare_sealed(&copied)?;} else if case!="official" {
            let path=copied.join("data/map/facility_templates.ron");
            let mut db=FacilityTemplateDatabase::from_ron_str(&fs::read_to_string(&path)?)?;
            edit_catalog(case,&mut db)?;fs::write(path,ron::ser::to_string(&db)?)?;
        }
        if case=="growth" {prepare_growth_arena(&copied)?;}
        fs::create_dir(base.join("content"))?;
        let receipt=content_pack::package_official(&copied,&base.join("content/official"),&content_pack::InputLimits::default())?;
        write(&base.join("PACKAGE.json"),&serde_json::to_value(receipt)?)?;
        let data=content_pack::load(&base.join("content"),&content_pack::ContentSelection{packs:vec!["official".into()],winners:vec![]},&content_pack::InputLimits::default())?;
        write(&base.join("IDENTITY.json"),&json!({"content_identity":data.identity(),"source_receipt":data.source_receipt()}))?;
    }Ok(())
}
fn snapshot(app:&CoreApplication,kind:ApplicationSnapshotKind)->Result<Value>{Ok(app.snapshot(kind)?.read_model().as_value().clone())}
fn run(app:&CoreApplication)->Result<Value>{snapshot(app,ApplicationSnapshotKind::Run)}
fn execute(app:&mut CoreApplication,input:Value)->Result<Value>{
    let trace=std::env::var_os("U14_COMMAND_TRACE");
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
    let mut app=CoreApplication::new(data(saved["case"].as_str().unwrap_or("movement"))?,13031);
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

fn prepare_growth_arena(source:&Path)->Result<()> {
    use game_core::game::{data::pve_data::*,combat_preview::{BattlefieldArchetype,BattlefieldSizeClass}};
    let path=source.join("data/pve/encounters.ron");
    let mut pve:PveEncounterDatabase=ron::de::from_str(&fs::read_to_string(&path)?)?;
    pve.encounters.iter_mut().find(|e|e.id=="normal_corroded_patrol").ok_or("growth patrol")?.battlefield=Some(PveBattlefieldOverrideData{archetype:Some(BattlefieldArchetype::OpenHall),size_class:Some(BattlefieldSizeClass::Medium)});
    fs::write(path,ron::ser::to_string(&pve)?)?;Ok(())
}

fn prepare_growth_layout(layout:&mut FacilityLayout)->Result<()> {
    layout.id="u14_authored_growth".into();layout.golden_bough_plans.clear();
    for a in &mut layout.activities {a.golden_bough_candidate=false;}
    let ordinary=layout.activities.iter_mut().find(|a|a.id=="ordinary").ok_or("ordinary")?;
    ordinary.payload=MapNodePayload::Encounter{encounter_id:Some("normal_corroded_patrol".into())};
    let base=layout.room("B1-01").ok_or("base")?.clone();
    let mut rooms=vec![layout.room("ENTRY").ok_or("entry")?.clone()];let mut edges=Vec::new();let mut previous="ENTRY".to_string();
    for depth in 1..=4 {for index in 1..=3 {
        let mut room=base.clone();room.id=format!("B{depth}-{index:02}");room.name=room.id.clone();room.depth=depth;
        room.omen_slot=(depth>=3).then(||MapSlotId::new("3"));room.elite_slots.clear();room.safe_access_slots=vec![MapSlotId::new("2"),MapSlotId::new("8")];
        room.reservations=vec![FacilityReservation{slot_id:MapSlotId::new("2"),activity_id:"save".into()},FacilityReservation{slot_id:MapSlotId::new("5"),activity_id:"ordinary".into()},FacilityReservation{slot_id:MapSlotId::new("8"),activity_id:"shop".into()}];
        edges.push(FacilityRoomConnection{from:previous,to:room.id.clone(),direction:if index==1 {MapEdgeDirection::ForwardOnly}else{MapEdgeDirection::Bidirectional},condition:None});previous=room.id.clone();rooms.push(room);
    }}
    layout.rooms=rooms;layout.connections=edges;
    for size in &mut layout.allocation.sizes {for weight in size.elite_percent_by_depth.values_mut(){*weight=0;}}
    Ok(())
}
fn growth(out:&Path)->Result<()> {
    fs::create_dir(out)?;let data=data("growth")?;let mut host=BattleHost{app:CoreApplication::new(data.clone(),13031),data};start(&mut host.app)?;
    let mut observations=Vec::new();let mut count=0;
    for depth in 1..=4 {for index in 1..=3 {
        let room=format!("B{depth}-{index:02}");if count>0 {move_to(&mut host.app,&room)?;}
        enter(&mut host.app,"2")?;execute(&mut host.app,json!({"type":"complete_node"}))?;
        enter(&mut host.app,"5")?;
        let before=run(&host.app)?;
        let outcome=u12_independent_lifecycle_probe::walk(&mut host,"Expedition","battle",&out.join(format!("battle_{count:02}.jsonl")));
        let after=run(&host.app)?;
        write(&out.join(format!("battle_{count:02}.json")),&json!({"room":room,"before":before,"after":after,"error":outcome.as_ref().err().map(ToString::to_string)}))?;outcome?;
        assert_eq!(after["selected_event"]["verdict"],"victory");
        count+=1;let expected=if count<=4 {count*10}else{40+(count-4)*20};
        for employee in after["roster"]["employees"].as_array().ok_or("roster")? {
            assert_eq!(employee["level"],1+expected/100);assert_eq!(employee["experience"],expected%100);
        }
        observations.push(json!({"room":room,"battle":count,"deterrence":after["run_progression"]["deterrence"],"roster":after["roster"],"resources":after["resources"]}));
        execute(&mut host.app,json!({"type":"complete_combat_result"}))?;
        enter(&mut host.app,"8")?;execute(&mut host.app,json!({"type":"exit_shop"}))?;enter(&mut host.app,"G")?;
    }}
    assert_eq!(count,12);
    write(&out.join("RESULT.json"),&json!({"passed":true,"approval_claimed":false,"official_enemy_reward_growth_tuning_unchanged":true,"authored_battlefield_override":"OpenHall/Medium; controlled three-participant formation","battles":count,"observations":observations,"final_roots":roots(&host.app)?}))
}
fn sealed(out:&Path)->Result<()> {
    fs::create_dir(out)?;let data=data("sealed")?;let mut host=BattleHost{app:CoreApplication::new(data.clone(),13031),data};start(&mut host.app)?;
    enter(&mut host.app,"1")?;execute(&mut host.app,json!({"type":"complete_node"}))?;
    let before=roots(&host.app)?;assert_eq!(run(&host.app)?["run_progression"]["deterrence"],11);
    enter(&mut host.app,"2")?;assert_eq!(run(&host.app)?["run_progression"]["deterrence"],0);
    let saved=json!({"case":"sealed","profile":snapshot(&host.app,ApplicationSnapshotKind::PlayerProfile)?,"checkpoint":checkpoint(&host.app)?});
    write(&out.join("unfinished.json"),&saved)?;
    let status=std::process::Command::new(std::env::current_exe()?).arg("resume").arg(out.join("unfinished.json")).arg(out.join("unfinished_resumed.json")).status()?;assert!(status.success());
    let fresh:Value=serde_json::from_slice(&fs::read(out.join("unfinished_resumed.json"))?)?;
    assert_eq!(fresh["roots"]["snapshots"][0]["run_progression"]["deterrence"],11);
    assert_eq!(fresh["roots"]["snapshots"][0]["roster"],before["snapshots"][0]["roster"]);
    let result=contact::battle(&mut host,&out.join("battle.jsonl"))?;
    let after=run(&host.app)?;assert_eq!(after["run_progression"]["deterrence"],8);
    assert_eq!(after["facility"]["golden_boughs"].as_array().ok_or("markers")?.iter().filter(|g|g["collected"]==true).count(),1);
    let settled=roots(&host.app)?;
    let saved=json!({"case":"sealed","profile":snapshot(&host.app,ApplicationSnapshotKind::PlayerProfile)?,"checkpoint":checkpoint(&host.app)?});write(&out.join("settled.json"),&saved)?;
    let status=std::process::Command::new(std::env::current_exe()?).arg("resume").arg(out.join("settled.json")).arg(out.join("settled_resumed.json")).status()?;assert!(status.success());
    let fresh:Value=serde_json::from_slice(&fs::read(out.join("settled_resumed.json"))?)?;assert_eq!(fresh["roots"],settled);
    execute(&mut host.app,json!({"type":"complete_combat_result"}))?;
    assert_eq!(run(&host.app)?["run_progression"]["deterrence"],8);
    write(&out.join("RESULT.json"),&json!({"passed":true,"approval_claimed":false,"fresh_process_before_entry_and_confirmed_result":true,"battle":result,"final_roots":roots(&host.app)?}))
}

fn main()->Result<()> {
    let args:Vec<_>=std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("prepare-growth") if args.len()==3=>prepare(Path::new(&args[1]),Path::new(&args[2]),&["growth"]),
        Some("correction-visited") if args.len()==3=>correction_visited(Path::new(&args[1]),Path::new(&args[2])),
        Some("growth") if args.len()==2=>growth(Path::new(&args[1])),
        Some("sealed") if args.len()==2=>sealed(Path::new(&args[1])),
        Some("battle") if args.len()==2=>battle(Path::new(&args[1])),
        Some("prepare") if args.len()==3=>prepare(Path::new(&args[1]),Path::new(&args[2]),&["official","expanded","movement","sealed","growth"]),
        Some("route") if args.len()==2=>route(Path::new(&args[1])),
        Some("resume") if args.len()==3=>{let saved:Value=serde_json::from_slice(&fs::read(&args[1])?)?;let app=restore(&saved)?;write(Path::new(&args[2]),&json!({"roots":roots(&app)?}))},
        Some("measure") if args.len()==3=>measure(&args[1],Path::new(&args[2])),
        _=>Err("usage: route OUT_DIR | resume INPUT OUTPUT | measure official|expanded|movement OUTPUT".into()),
    }
}


// F-U14-001 regression: same authored boss slots in two rooms, one plan owner.
fn correction_visited(source:&Path,out:&Path)->Result<()> {
    fs::create_dir(out)?;
    let mut results=Vec::new();
    for (case,balance,collect) in [("restrained",12,false),("unrestrained",3,false),("collected",12,true)] {
        let base=out.join(case);fs::create_dir(&base)?;
        let copied=base.join("source");copy_directory(source,&copied)?;
        prepare_sealed(&copied)?;
        let path=copied.join("data/map/facility_templates.ron");
        // Begin from the official layout so the movement author has its four source rooms.
        let mut db=FacilityTemplateDatabase::from_ron_str(&fs::read_to_string(source.join("data/map/facility_templates.ron"))?)?;
        edit_catalog("movement",&mut db)?;
        let layout=&mut db.layouts[0];
        let boss=layout.activities.iter_mut().find(|a|a.id=="reserved_B4-02").unwrap();
        boss.golden_bough_candidate=true;boss.support=FacilityContentSupport::Supported;
        boss.payload=MapNodePayload::Encounter{encounter_id:Some("suppress_nothing_there".into())};boss.first_entry_cost=Some(1);
        for room in layout.rooms.iter_mut().filter(|r|r.depth==1) {for slot in ["1","3"] {room.reservations.push(FacilityReservation{slot_id:MapSlotId::new(slot),activity_id:"reserved_B4-02".into()});}}
        let loc=|slot:&str|GoldenBoughLocation{room_id:"B1-01".into(),slot_id:MapSlotId::new(slot)};
        layout.golden_bough_plans=vec![GoldenBoughPlan{id:"one".into(),weight:1,locations:vec![loc("1")],deterrence_refill:8},GoldenBoughPlan{id:"two".into(),weight:1,locations:vec![loc("1"),loc("3")],deterrence_refill:8}];
        fs::write(path,ron::ser::to_string(&db)?)?;
        let path=copied.join("data/run/policy.ron");
        let mut policy:game_core::game::data::run_policy_data::RunPolicyData=ron::de::from_str(&fs::read_to_string(&path)?)?;
        policy.expedition_deterrence_initial=balance;fs::write(path,ron::ser::to_string(&policy)?)?;
        fs::create_dir(base.join("content"))?;
        let receipt=content_pack::package_official(&copied,&base.join("content/official"),&content_pack::InputLimits::default())?;
        write(&base.join("PACKAGE.json"),&serde_json::to_value(receipt)?)?;
        let data=content_pack::load(&base.join("content"),&content_pack::ContentSelection{packs:vec!["official".into()],winners:vec![]},&content_pack::InputLimits::default())?;
        let mut host=BattleHost{app:CoreApplication::new(data.clone(),13031),data};start(&mut host.app)?;
        fn observed(app:&CoreApplication,room_id:&str)->Result<Value> {
            let view=run(app)?;let room=view["facility"]["rooms"].as_array().unwrap().iter().find(|r|r["id"]==room_id).unwrap();
            let node=room["current_expedition_map"]["nodes"].as_array().unwrap().iter().find(|n|n["slot_id"]=="1").unwrap();
            let activity=room["activities"].as_array().unwrap().iter().find(|a|a["node_id"]==node["id"]).unwrap();
            assert_eq!(node["entry_preview"],activity["entry_preview"]);
            Ok(json!({"current_room":view["facility"]["current_room_id"],"balance":view["run_progression"]["deterrence"],"markers":view["facility"]["golden_boughs"],"node":node,"activity":activity}))
        }
        let initial=observed(&host.app,"B1-01")?;
        assert_eq!(initial["node"]["entry_preview"]["sealed_golden_bough"],true);
        assert!(initial["node"]["payload"]["Encounter"]["encounter_id"].is_null());
        assert!(initial["activity"]["label"].is_null());
        if collect {
            enter(&mut host.app,"2")?;execute(&mut host.app,json!({"type":"complete_node"}))?;
            enter(&mut host.app,"1")?;
            let actual=contact::battle(&mut host,&base.join("battle.jsonl"))?;write(&base.join("battle_result.json"),&actual)?;
            assert_eq!(run(&host.app)?["run_progression"]["deterrence"],balance-2+8);
            execute(&mut host.app,json!({"type":"complete_combat_result"}))?;
        }
        if collect {
            for (slot,finish) in [("5","complete_node"),("8","exit_shop")] {enter(&mut host.app,slot)?;execute(&mut host.app,json!({"type":finish}))?;}
            enter(&mut host.app,"G")?;assert_eq!(run(&host.app)?["facility"]["gate_reached"],true);
        } else {gate(&mut host.app)?;}
        move_to(&mut host.app,"B1-02")?;
        let away=observed(&host.app,"B1-01")?;
        assert_eq!(away["node"]["id"],initial["node"]["id"]);
        assert_eq!(away["node"]["entry_preview"]["sealed_golden_bough"],!collect);
        let scale=&away["node"]["entry_preview"]["combat_deterrence"];
        assert_eq!(scale["seal_released"],!collect);
        assert_eq!(scale["unrestrained"],balance==3);
        assert_eq!(scale["enemy_health_percent"],if balance==3{150}else if collect{100}else{180});
        assert_eq!(scale["enemy_attack_percent"],if balance==3{150}else if collect{100}else{170});
        assert_eq!(scale["victory_quantity_multiplier"],if balance==3{2}else{1});
        assert_eq!(observed(&host.app,"B1-02")?["node"]["entry_preview"]["sealed_golden_bough"],false);
        let before=roots(&host.app)?;execute(&mut host.app,json!({"type":"request_map_data"}))?;assert_eq!(roots(&host.app)?,before);
        // Child process uses the same normally packaged authored input and save codec.
        let saved=json!({"case":case,"profile":snapshot(&host.app,ApplicationSnapshotKind::PlayerProfile)?,"checkpoint":checkpoint(&host.app)?});
        write(&base.join("saved.json"),&saved)?;
        let status=std::process::Command::new(std::env::current_exe()?).env("U14_PROBE_INPUTS",out).arg("resume").arg(base.join("saved.json")).arg(base.join("resumed.json")).status()?;
        assert!(status.success());let resumed:Value=serde_json::from_slice(&fs::read(base.join("resumed.json"))?)?;assert_eq!(resumed["roots"],before);
        gate(&mut host.app)?;move_to(&mut host.app,"B1-01")?;let returned=observed(&host.app,"B1-01")?;
        assert_eq!(returned["node"]["id"],initial["node"]["id"]);
        assert_eq!(returned["node"]["entry_preview"]["combat_deterrence"],*scale);
        assert_eq!(returned["markers"],away["markers"]);
        let result=json!({"case":case,"passed":true,"initial":initial,"away":away,"returned":returned,"fresh_process_continue_roots_equal":true,"actual_boss_victory":collect});
        write(&base.join("RESULT.json"),&result)?;results.push(json!({"case":case,"passed":true}));
    }
    write(&out.join("RESULT.json"),&json!({"passed":true,"cases":results}))?;println!("3/3 correction public scenarios passed");Ok(())
}
