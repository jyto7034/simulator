use actix::prelude::*;
struct Sink;
impl Actor for Sink {type Context=Context<Self>;}
impl Handler<game_server::game::player_game_actor::messages::PlayerGameServerMessage> for Sink {type Result=();fn handle(&mut self,_:game_server::game::player_game_actor::messages::PlayerGameServerMessage,_:&mut Context<Self>) {}}
impl Handler<game_server::game::player_game_actor::messages::ForceDisconnect> for Sink {type Result=();fn handle(&mut self,_:game_server::game::player_game_actor::messages::ForceDisconnect,_:&mut Context<Self>) {}}
use game_core::application::*;
use game_core::game::data::content_pack::{self,ContentSelection,InputLimits};
use serde_json::{Value,json};
use std::path::Path;
fn main(){
 let root=Path::new("/mnt/f/work/simulator/core_enhanced/docs/goals/content_authoring_r08_r12_runtime/execution/single_goal/evidence/cli_predecessor_current_001/r06");
 let selection:ContentSelection=serde_json::from_slice(&std::fs::read(root.join("selection.json")).unwrap()).unwrap();
 let data=content_pack::load(&root.join("content"),&selection,&InputLimits::default()).unwrap();
 let scenario:Value=serde_json::from_slice(&std::fs::read(root.join("scenario.json")).unwrap()).unwrap();
 let player=uuid::Uuid::parse_str(scenario["initial"]["player_id"].as_str().unwrap()).unwrap();
 let mut app=CoreApplication::new(data,scenario["initial"]["run_seed"].as_u64().unwrap());
 let mut records=Vec::<Value>::new();
 let mut token=None;
 let mut duplicate=None;
 let mut before=None;
 let mut durable_before=None;
 for (index,step) in scenario["preparation_steps"].as_array().unwrap().iter().take(4).enumerate(){
  let mut step=step.clone();
  if let Some(bindings)=step["bindings"].as_array().cloned(){
   for binding in bindings{
    let value=records[binding["from"]["step"].as_u64().unwrap() as usize].pointer(binding["from"]["pointer"].as_str().unwrap()).unwrap().clone();
    *step.pointer_mut(binding["to"].as_str().unwrap()).unwrap()=value;
   }
  }
  if index==5{
   assert!(!app.has_active_battle());
   before=Some(app.snapshot(ApplicationSnapshotKind::Run).unwrap());
   durable_before=Some(app.export_active_run_checkpoint().unwrap());
   token=app.capture_persistence_rollback().unwrap();assert!(token.is_some());
   duplicate=app.capture_persistence_rollback().unwrap();
  }
  let out=if step["kind"]=="snapshot"{
   serde_json::to_value(app.snapshot(ApplicationSnapshotKind::Run).unwrap()).unwrap()
  }else{
   let cmd=ApplicationCommand::from_document(step["command"].clone().into()).unwrap();
   serde_json::to_value(app.execute(player,cmd,ApplicationExecutionContext::default()).unwrap()).unwrap()
  };
  println!("{}",json!({"step":index,"kind":out["kind"],"active_battle":app.has_active_battle()}));
  records.push(out);
 }

 let admin=|app:&mut CoreApplication,doc:Value| app.execute_admin(ApplicationAdminCommand::from_document(doc.into()).unwrap()).unwrap();
 let command=|app:&mut CoreApplication,doc:Value| app.execute(player,ApplicationCommand::from_document(doc.into()).unwrap(),ApplicationExecutionContext::default()).unwrap();
 admin(&mut app,json!({"type":"admin_enter_save_point"}));
 command(&mut app,json!({"type":"complete_node"}));
 let saved=app.export_active_run_checkpoint().unwrap().unwrap();
 println!("saved peak {}",saved.run_peak_certified_power());
 let grant=admin(&mut app,json!({"type":"admin_grant_equipment","definition_id":"crimson_goggles","count":1}));
 println!("grant {}",grant.payload().as_value());
 let run=app.snapshot(ApplicationSnapshotKind::Run).unwrap();
 println!("roster {}",run.read_model().as_value()["roster"]);
 let employee=run.read_model().as_value()["roster"]["employees"][0]["uuid"].clone();
 let item=grant.payload().as_value()["item_uuids"][0].clone();
 command(&mut app,json!({"type":"equip_item","item_uuid":item,"target_unit":employee}));
 let map=command(&mut app,json!({"type":"request_map_data"}));
 println!("map {}",map.payload().as_value());
 for node in map.payload().as_value()["map"]["nodes"].as_array().unwrap(){
  if node["state"]!="Available" || node["category"]!="Combat" {continue;}
  command(&mut app,json!({"type":"select_map_node","node_id":node["id"]}));

  actix::System::new().block_on(async move {
   use std::sync::{Arc,Mutex};
   use game_server::game::{load_balance_actor::LoadBalanceActor,player_game_actor::{PlayerGameActor,messages::*}};
   use game_server::persistence::active_run_store::{ActiveRunStore,StoredCheckpoint,ActiveRunLoad};
   let before=app.export_active_run_checkpoint().unwrap().unwrap();
   let temp=std::env::temp_dir().join(format!("s1f01-audit-host-{}",std::process::id()));std::fs::create_dir(&temp).unwrap();
   let store=Arc::new(Mutex::new(ActiveRunStore::open(&temp).unwrap()));
   store.lock().unwrap().publish(&StoredCheckpoint::from(&before)).unwrap();
   std::fs::create_dir(temp.join("active_run.tmp")).unwrap();
   let player_actor=PlayerGameActor::new(player,app,LoadBalanceActor::new().start()).with_active_run_store(store.clone()).start();
   let sink=Sink.start();let session=uuid::Uuid::from_u128(988);
   player_actor.send(AttachSession{session_id:session,socket:sink.clone().recipient(),control:sink.clone().recipient()}).await.unwrap().unwrap();
   let result=player_actor.send(ExecutePlayerBehavior{session_id:session,request_id:"entry-with-new-peak".into(),behavior:ApplicationCommand::from_document(json!({"type":"confirm_enter_node"}).into()).unwrap(),battle_authority:None,battle_side_message:BattleSideMessageKind::BattleUpdate}).await.unwrap();
   println!("host_command_result {:?}",result);
   let attached=player_actor.send(AttachSession{session_id:session,socket:sink.clone().recipient(),control:sink.recipient()}).await.unwrap().unwrap();
   println!("post_failure_snapshot {}",attached.state_snapshot);
   println!("post_failure_battle_side_messages {}",attached.side_messages.len());
   let persisted=store.lock().unwrap().load().unwrap();
   if let ActiveRunLoad::Loaded{checkpoint,..}=persisted{println!("persisted_peak {}",checkpoint.run_peak_certified_power);}
   assert_eq!(result.unwrap_err().code,"active_battle_persistence_rollback");
   assert!(attached.side_messages.len()>=2,"committed battle remains despite error");
  });
  return;
 }
 panic!("No available combat node");
}
