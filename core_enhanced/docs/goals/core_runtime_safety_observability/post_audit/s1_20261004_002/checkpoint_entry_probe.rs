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
  let before=app.export_active_run_checkpoint().unwrap().unwrap();
  let token=app.capture_persistence_rollback().unwrap().unwrap();
  command(&mut app,json!({"type":"confirm_enter_node"}));
  let after=app.export_active_run_checkpoint().unwrap().unwrap();
  println!("{}",json!({"active_battle":app.has_active_battle(),"before_peak":before.run_peak_certified_power(),"after_peak":after.run_peak_certified_power(),"checkpoint_equal":before==after,"restore":format!("{:?}",app.restore_persistence_rollback(token))}));
  return;
 }
 panic!("No available combat node");
}
