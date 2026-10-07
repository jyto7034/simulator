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
 let mut before=None;
 for (index,step) in scenario["preparation_steps"].as_array().unwrap().iter().take(6).enumerate(){
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
   token=app.capture_persistence_rollback().unwrap();assert!(token.is_some());
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
 assert!(app.has_active_battle());
 assert!(app.capture_persistence_rollback().unwrap().is_none());
 let restored=app.restore_persistence_rollback(token.unwrap());
 println!("{}",json!({"restore_result":format!("{:?}",restored),"active_battle_after_restore":app.has_active_battle(),"run_equals_prebattle":app.snapshot(ApplicationSnapshotKind::Run).unwrap()==before.unwrap()}));
 assert!(restored.is_err(),"contract requires rejecting restoration during active battle");
}
