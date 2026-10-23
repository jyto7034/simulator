//! Actual normal LoadBalanceActor path. Read-only probe messages inspect state;
//! all writes go through ordinary session commands and approved persistence.
use super::{support::*,disposable_configs,output};
use crate::{game::{load_balance_actor::{LoadBalanceActor,messages::{FindPlayer,GetOrCreatePlayerActor,GetPlayerCount}},
    player_game_actor::{PlayerGameActor,messages::*}},mod_admission::ApprovedPersistence};
use actix::{Actor,Addr,Context,Handler,Message,MessageResult};
use game_core::{application::{ApplicationCommand,ApplicationSnapshotKind},game::{behavior::{GameStateContextDto,RunSnapshotDto},
    data::{ValidatedGameData,content_pack::ContentIdentity},map::{MapNodeCategory,MapNodeState}},mod_validation::JobState};
use serde_json::{json,Value};
use std::{fs,path::{Path,PathBuf},sync::Arc,time::Duration};
use uuid::Uuid;

struct Sink;
impl Actor for Sink{type Context=Context<Self>;}
impl Handler<PlayerGameServerMessage> for Sink{type Result=();fn handle(&mut self,_:PlayerGameServerMessage,_:&mut Context<Self>) {}}
impl Handler<ForceDisconnect> for Sink{type Result=();fn handle(&mut self,_:ForceDisconnect,_:&mut Context<Self>) {}}
#[derive(Message)]
#[rtype(result="Value")]
struct ReadOnly;
impl Handler<ReadOnly> for PlayerGameActor {
    type Result=MessageResult<ReadOnly>;
    fn handle(&mut self,_:ReadOnly,_:&mut Context<Self>)->Self::Result {
        let snapshot=self.game_core.snapshot(ApplicationSnapshotKind::Run).unwrap();
        let (checkpoint,checkpoint_error)=match self.game_core.export_active_run_checkpoint(){
            Ok(checkpoint)=>(checkpoint,None),Err(error)=>(None,Some(error)),
        };
        MessageResult(json!({"identity":self.game_core.content_identity(),"required":self.active_required,
            "run":snapshot.read_model().as_value(),"fault":self.game_core.battle_fault(),
            "checkpoint_sha256":checkpoint.as_ref().map(|c|digest(c.core_payload_json())),
            "checkpoint_error":checkpoint_error,
            "tick_scheduled":self.live_battle_tick.is_some()}))
    }
}
async fn actor(lb:&Addr<LoadBalanceActor>,id:Uuid,seed:u64,raw:&ValidatedGameData,approved:&ContentIdentity)
    ->Result<(Addr<PlayerGameActor>,Uuid,Addr<Sink>)>{
    // A distinct token acquired through production content loading is deliberately
    // supplied in the actor request. The LB must use its approved selection.
    assert_ne!(raw.identity(),approved,"raw input must differ from the approved selection");
    let addr=lb.send(GetOrCreatePlayerActor{player_id:id,run_seed:seed,game_data:raw.clone()}).await?;
    let selected=addr.send(ReadOnly).await?;
    assert_eq!(selected["identity"],serde_json::to_value(approved)?);
    assert_ne!(selected["identity"],serde_json::to_value(raw.identity())?);
    let session=Uuid::new_v4();let sink=Sink.start();
    addr.send(AttachSession{session_id:session,socket:sink.clone().recipient(),control:sink.clone().recipient()}).await?
        .map_err(|e|format!("normal actor attach: {} {}",e.code,e.message))?;
    Ok((addr,session,sink))
}
async fn command(actor:&Addr<PlayerGameActor>,session:Uuid,value:Value,out:&Path)->Result<CommandExecutionResult>{
    let behavior=ApplicationCommand::from_document(value.clone().into()).map_err(|e|format!("command parse: {e:?}"))?;
    let result=actor.send(ExecutePlayerBehavior{session_id:session,request_id:out.file_stem().unwrap().to_string_lossy().into(),
        behavior,battle_authority:None,battle_side_message:BattleSideMessageKind::BattleUpdate}).await?
        .map_err(|e|format!("normal actor command: {} {}",e.code,e.message))?;
    match &result.response{PlayerGameServerMessage::CommandResult{ok:true,..}=>{},other=>return Err(format!("normal command refused: {other:?}").into())}
    save(out,&json!({"command":value,"response":result.response,"side_messages":result.side_messages,"state":result.state_snapshot}))?;
    Ok(result)
}
async fn starters(actor:&Addr<PlayerGameActor>,session:Uuid,out:&Path)->Result<()> {
    fs::create_dir(out)?;
    for index in 0..128 {
        let probe=actor.send(ReadOnly).await?;let run:RunSnapshotDto=serde_json::from_value(probe["run"].clone())?;
        let value=if let Some(vn)=run.visual_novel {
            if let Some(choice)=vn.choices.iter().find(|c|c.selectable){json!({"type":"select_visual_novel_choice",
                "session_id":vn.session_id,"current_scene_id":vn.current_scene_id,"choice_id":choice.choice_id})}
            else{json!({"type":"advance_visual_novel","session_id":vn.session_id,"current_scene_id":vn.current_scene_id})}
        }else if let GameStateContextDto::SelectingStarterEmployees{required_count,candidates}=run.game_state_context {
            let ids:Vec<_>=candidates.into_iter().take(required_count).map(|c|c.employee_definition_id).collect();
            assert_eq!(ids.len(),required_count);json!({"type":"select_starter_employees","employee_definition_ids":ids})
        }else{assert!(run.roster.combat_eligible_employee_ids.len()>=2);return Ok(());};
        command(actor,session,value,&out.join(format!("{index:03}.json"))).await?;
    }Err("normal actor starter bound exceeded".into())
}

#[actix_web::test]
#[ignore="requires actual reviewed same-executable calibration cases and exclusive evidence; run explicit U11 selector"]
async fn u11_normal_actor_pins_a_new_actor_uses_b_and_transport_has_no_approval(){
    let run=async{
        let(cases,out)=output("normal_actors")?;let cases=disposable_configs(&cases,&out)?;
        let mut admission=open(&config(&cases.a)?)?;let a=admission.query()?.required;
        let raw_a=acquire(&config(&cases.a)?)?.generation().clone();
        let raw_b=acquire(&config(&cases.b)?)?.generation().clone();
        let root=out.join("save");let persistence=ApprovedPersistence::open(&root,Arc::clone(&admission.host))?;
        let lb=LoadBalanceActor::with_approved_persistence(Arc::clone(&admission.host),persistence).start();
        let(first,session,_sink)=actor(&lb,PLAYER,77,&raw_b,&a.content_identity).await?;
        command(&first,session,json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false}),&out.join("start_a.json")).await?;
        starters(&first,session,&out.join("starter_a")).await?;
        let first_before=first.send(ReadOnly).await?;
        assert_eq!(first_before["required"],serde_json::to_value(&a)?);
        let actual_b=admission.validate_and_apply(acquire(&config(&cases.b)?)?)?;
        assert_eq!(actual_b.state,JobState::Passed);save(&out.join("actual_b_acceptance.json"),&actual_b)?;
        let b=admission.query()?.required;assert_ne!(a,b);
        assert_eq!(first.send(ReadOnly).await?,first_before,"already started A must stay pinned");
        let(second,second_session,_sink2)=actor(&lb,Uuid::from_u128(0x1102),99,&raw_a,&b.content_identity).await?;
        let second_before=second.send(ReadOnly).await?;
        assert_eq!(second_before["required"],serde_json::to_value(&b)?);
        assert_eq!(second_before["identity"],serde_json::to_value(&b.content_identity)?);
        assert_eq!(lb.send(GetPlayerCount).await?,2);
        let save_before=state(&root)?;let cache_before=state(&config(&cases.a)?.issuer_root)?;
        for(kind,payload)in [("apply_mods",json!({"mod_path":"/tmp/untrusted"})),
            ("apply_approved_generation",json!({"status":"passed","candidate_identity":"forged"})),
            ("select_mod_configuration",serde_json::to_value(&b)?)] {
            let mut behavior=payload;behavior["type"]=json!(kind);
            let transport=json!({"type":"command","request_id":"untrusted-admission","behavior":behavior});
            let error=serde_json::from_value::<PlayerGameClientMessage>(transport.clone()).err()
                .ok_or("transport unexpectedly accepts a host approval/selection command")?;
            save(&out.join(format!("transport_{kind}.json")),&json!({"input":transport,"error":error.to_string()}))?;
        }
        save_before.ensure_unchanged(&state(&root)?)?;cache_before.ensure_unchanged(&state(&config(&cases.a)?.issuer_root)?)?;
        assert_eq!(first.send(ReadOnly).await?,first_before);assert_eq!(second.send(ReadOnly).await?,second_before);
        command(&second,second_session,json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":true}),&out.join("new_b_actor_start.json")).await?;
        starters(&second,second_session,&out.join("starter_b")).await?;
        assert_eq!(active(&root.join("active_run.json"))?.unsigned.required_configuration.as_ref(),Some(&b));
        assert_eq!(first.send(ReadOnly).await?,first_before);
        save(&out.join("RESULT.json"),&json!({"passed":true,"actual_load_balancer":true,"a_pinned":a,"new_actor_b":b,
            "distinct_production_raw_token_ignored":true,"raw_supplied_for_a":raw_b.identity(),"raw_supplied_for_b":raw_a.identity(),"transport_approval_rejected":true}))?;
        Ok::<(),Box<dyn std::error::Error>>(())
    };run.await.unwrap();
}

// This observer reads only public CoreApplication APIs. It deliberately does not
// claim the private complete-root fingerprint owned by application/fault_tests.rs.
#[derive(Message)]
#[rtype(result="Value")]
struct FaultPublicView;
impl Handler<FaultPublicView> for PlayerGameActor {
    type Result=MessageResult<FaultPublicView>;
    fn handle(&mut self,_:FaultPublicView,_:&mut Context<Self>)->Self::Result {
        let mut snapshots=serde_json::Map::new();
        for kind in ApplicationSnapshotKind::ALL {
            let key=serde_json::to_value(kind).unwrap().as_str().unwrap().to_owned();
            snapshots.insert(key,self.game_core.snapshot(*kind).unwrap().read_model().as_value().clone());
        }
        MessageResult(json!({"snapshots":snapshots,
            "runtime_facts":self.game_core.battle_runtime_facts().unwrap(),
            "run_resources":self.game_core.run_resources(),
            "battle_typed_resources":self.game_core.battle_typed_resources(),
            "reconnect":self.game_core.active_battle_reconnect(0)}))
    }
}
struct ResumeCapture {
    response:CommandExecutionResult,
    public_before_tick:Value,
    durable_before_tick:file_state::FileState,
}
#[derive(Message)]
#[rtype(result="std::result::Result<ResumeCapture,String>")]
struct ResumeAndCapture {
    command:ExecutePlayerBehavior,
    save_root:PathBuf,
}
impl Handler<ResumeAndCapture> for PlayerGameActor {
    type Result=std::result::Result<ResumeCapture,String>;
    fn handle(&mut self,msg:ResumeAndCapture,ctx:&mut Context<Self>)->Self::Result {
        // Execute the same production session/authority/persistence handler.
        // The mailbox cannot run the scheduled tick until this read capture ends.
        let response=<Self as Handler<ExecutePlayerBehavior>>::handle(self,msg.command,ctx)
            .map_err(|e|format!("normal resume: {} {}",e.code,e.message))?;
        let public_before_tick=<Self as Handler<FaultPublicView>>::handle(self,FaultPublicView,ctx).0;
        let durable_before_tick=state(&msg.save_root).map_err(|e|e.to_string())?;
        Ok(ResumeCapture{response,public_before_tick,durable_before_tick})
    }
}
// Test mailbox adapter: observe public readiness and dispatch ordinary PauseBattle
// in one turn, so a host tick cannot stale the authority between those operations.
#[derive(Message)]
#[rtype(result="std::result::Result<(Value,Option<Value>),String>")]
struct PauseReadyEmployee {session:Uuid,employee:Uuid}
impl Handler<PauseReadyEmployee> for PlayerGameActor {
    type Result=std::result::Result<(Value,Option<Value>),String>;
    fn handle(&mut self,msg:PauseReadyEmployee,ctx:&mut Context<Self>)->Self::Result {
        let view=<Self as Handler<FaultPublicView>>::handle(self,FaultPublicView,ctx).0;
        let update=live_update(&view).map_err(|e|e.to_string())?;
        let unit=update["checkpoint"]["deployment"]["deployed_units"].as_array().ok_or("live charging deployment required")?.iter()
            .find(|u|u["employee_uuid"]==json!(msg.employee)).ok_or("same charging employee must stay live")?;
        let readiness=&unit["skill_readiness"];
        if readiness["resonance_current"]!=1 || readiness["eligibility"]["enabled"]!=true
            || readiness["manual_activation_allowed"]!=true {return Ok((view,None));}
        let authority=serde_json::from_value(json!({"battle_uuid":update["battle_uuid"],
            "battle_state_revision":update["battle_state_revision"]})).map_err(|e|e.to_string())?;
        let input=json!({"type":"pause_battle"});
        let behavior=ApplicationCommand::from_document(input.clone().into()).map_err(|e|format!("normal charge pause parse: {e:?}"))?;
        let result=<Self as Handler<ExecutePlayerBehavior>>::handle(self,ExecutePlayerBehavior{
            session_id:msg.session,request_id:"pause-ready-expression-caster".into(),behavior,
            battle_authority:Some(authority),battle_side_message:BattleSideMessageKind::BattleUpdate},ctx)
            .map_err(|e|format!("normal charge pause: {} {}",e.code,e.message))?;
        if !matches!(result.response,PlayerGameServerMessage::CommandResult{ok:true,..}) {
            return Err("ordinary charge pause must succeed".into());
        }
        Ok((view,Some(json!({"command":input,"authority":authority,"response":result.response,
            "side_messages":result.side_messages,"state":result.state_snapshot}))))
    }
}
fn atomic_public_projection(view:&Value)->Value {
    let mut value=view.clone();
    // Reconnect is gated after fault, not a surviving state projection. Exact
    // Run/action-capability exclusions are documented in application.rs snapshot().
    value.as_object_mut().unwrap().remove("reconnect");
    let snapshots=value["snapshots"].as_object_mut().unwrap();
    snapshots.remove("action_capabilities");
    let run=snapshots.get_mut("run").unwrap().as_object_mut().unwrap();
    run.remove("battle_fault");run.remove("action_capabilities");
    let context=run.get_mut("game_state_context").unwrap().as_object_mut().unwrap();
    context.remove("can_retreat");context.remove("deployment");
    value
}
fn live_update(view:&Value)->Result<&Value>{
    view.pointer("/reconnect/Ok/payload/battle_update").ok_or("live public reconnect update required".into())
}
async fn battle_command(actor:&Addr<PlayerGameActor>,session:Uuid,value:Value,out:&Path)->Result<CommandExecutionResult>{
    let view=actor.send(FaultPublicView).await?;
    let update=live_update(&view)?;
    let authority=serde_json::from_value(json!({"battle_uuid":update["battle_uuid"],
        "battle_state_revision":update["battle_state_revision"]}))?;
    let behavior=ApplicationCommand::from_document(value.clone().into()).map_err(|e|format!("battle command parse: {e:?}"))?;
    let result=actor.send(ExecutePlayerBehavior{session_id:session,request_id:out.file_stem().unwrap().to_string_lossy().into(),
        behavior,battle_authority:Some(authority),battle_side_message:BattleSideMessageKind::BattleUpdate}).await?
        .map_err(|e|format!("public battle command: {} {}",e.code,e.message))?;
    save(out,&json!({"command":value,"authority":authority,"response":result.response,
        "side_messages":result.side_messages,"state":result.state_snapshot}))?;
    assert!(matches!(result.response,PlayerGameServerMessage::CommandResult{ok:true,..}));
    Ok(result)
}
async fn deploy_expression_caster(actor:&Addr<PlayerGameActor>,session:Uuid,entered:&CommandExecutionResult,out:&Path)->Result<Uuid>{
    let setup=entered.side_messages.iter().find_map(|message|match message{
        PlayerGameServerMessage::BattleSetupSnapshot(setup)=>Some(serde_json::to_value(setup).unwrap()),_=>None
    }).ok_or("actual entered battle setup required")?;
    // Wait for an actual authored opponent through ordinary host scheduling.
    // No caster is deployed yet, so it cannot start a basic attack while waiting.
    actor.send(EnsureLiveBattleTick).await?;
    let mut spawned=false;
    for _ in 0..400 {
        let view=actor.send(FaultPublicView).await?;
        let cp=&live_update(&view)?["checkpoint"];
        if cp["units"].as_array().ok_or("actual unit checkpoint required")?.iter()
            .any(|u|u["owner"]=="Opponent"&&u["stats"]["current_health"].as_u64().is_some_and(|hp|hp>10)) {
            save(&out.join("actual_opponent_spawn.json"),&view)?;spawned=true;break;
        }
        actix_web::rt::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(spawned,"normal host must publish an actual live opponent before target selection");
    battle_command(actor,session,json!({"type":"pause_battle"}),&out.join("pause_before_deploy.json")).await?;
    let view=actor.send(FaultPublicView).await?;
    let roster=&view["snapshots"]["employee_roster"];
    let employee=roster["employees"].as_array().ok_or("roster employees required")?.iter()
        .find(|e|e["employee_definition_id"]=="records_clerk").ok_or("normal records_clerk starter required")?;
    let id:Uuid=serde_json::from_value(employee["uuid"].clone())?;
    assert!(roster["combat_eligible_employee_ids"].as_array().unwrap().contains(&json!(id)));
    assert!(employee["equipped_items"].as_array().unwrap().iter().any(|e|e["definition_id"]=="standard_training_sword"));
    assert_eq!(employee["combat_profile"]["effective_deployment_affinity"],"ground_only");
    let update=live_update(&view)?;let cp=&update["checkpoint"];
    assert_eq!(cp["playback"]["paused"],true);
    let cost=cp["deployment"]["unit_deploy_costs"].as_array().unwrap().iter()
        .find(|c|c["employee_uuid"]==json!(id)).ok_or("normal deployment cost required")?;
    assert_eq!(cost["eligibility"]["enabled"],true);
    assert!(cost["effective_deploy_cost"].as_u64().unwrap()<=cp["tactical_resource"]["current"].as_u64().unwrap());
    let position=setup["deployment_zones"].as_array().unwrap().iter().filter(|z|z["kind"]=="Ground")
        .flat_map(|z|z["cells"].as_array().unwrap()).find(|cell|!cp["units"].as_array().unwrap().iter().any(|u|u["position"]==**cell))
        .cloned().ok_or("returned unoccupied ground deployment tile required")?;
    battle_command(actor,session,json!({"type":"deploy_unit","employee_uuid":id,"position":position,"facing":"up"}),
        &out.join("deploy_records_clerk.json")).await?;
    // Ordinary employee resonance starts at max/5, hence zero for max1.
    // Let the normal host tick reach its next whole-second +1 pulse.
    let initial=actor.send(FaultPublicView).await?;
    save(&out.join("actual_post_deploy_before_charge.json"),&initial)?;
    let initial_update=live_update(&initial)?;
    let initial_unit=initial_update["checkpoint"]["deployment"]["deployed_units"].as_array().unwrap().iter()
        .find(|u|u["employee_uuid"]==json!(id)).ok_or("actual deployed charging employee required")?;
    assert_eq!(initial_unit["skill_readiness"]["resonance_current"],0);
    assert_eq!(initial_unit["skill_readiness"]["resonance_max"],1);
    let charging_unit=initial_unit["unit_instance_id"].clone();
    let initial_seq=initial_update["checkpoint"]["at_seq"].as_u64().ok_or("actual before-charge event sequence required")?;
    battle_command(actor,session,json!({"type":"resume_battle"}),&out.join("resume_for_resonance_charge.json")).await?;
    let mut charged=false;let mut observations=Vec::new();
    for _ in 0..400 {
        let(view,pause)=actor.send(PauseReadyEmployee{session,employee:id}).await??;
        let cp=&live_update(&view)?["checkpoint"];
        let unit=cp["deployment"]["deployed_units"].as_array().ok_or("actual charging deployment required")?.iter()
            .find(|u|u["employee_uuid"]==json!(id)).ok_or("same deployed employee must survive charging")?;
        let readiness=&unit["skill_readiness"];
        assert_eq!(readiness["skill_id"],"training_berserker");
        assert_eq!(readiness["resonance_max"],1);
        observations.push(json!({"battle_time_ms":cp["deployment"]["battle_time_ms"],"readiness":readiness}));
        if let Some(pause)=pause {
            let gains:Vec<_>=live_update(&view)?["events_delta"]["events"].as_array().ok_or("actual resonance event delta required")?.iter()
                .filter(|e|e["seq"].as_u64().is_some_and(|seq|seq>initial_seq)
                    &&e["event"]["type"]=="ResonanceChanged"&&e["event"]["unit_instance_id"]==charging_unit
                    &&e["event"]["before"]==0&&e["event"]["after"]==1&&e["event"]["max"]==1).cloned().collect();
            assert_eq!(gains.len(),1,"one ordinary employee resonance pulse must charge the same caster");
            save(&out.join("actual_resonance_ready_before_pause.json"),&view)?;
            save(&out.join("pause_after_resonance_charge.json"),&json!({"pause":pause,"actual_charge_events":gains}))?;
            charged=true;break;
        }
        actix_web::rt::time::sleep(Duration::from_millis(25)).await;
    }
    save(&out.join("actual_resonance_charge_observations.json"),&json!({"charged":charged,"observations":observations}))?;
    assert!(charged,"normal host resonance pulse must make the actual employee skill manually ready");
    let deployed=actor.send(FaultPublicView).await?;
    let cp=&live_update(&deployed)?["checkpoint"];
    assert_eq!(cp["playback"]["paused"],true);
    let unit=cp["deployment"]["deployed_units"].as_array().unwrap().iter()
        .find(|u|u["employee_uuid"]==json!(id)).ok_or("actual deployed records_clerk required")?;
    save(&out.join("actual_public_skill_ready.json"),&deployed)?;
    let readiness=&unit["skill_readiness"];
    assert_eq!(readiness["skill_id"],"training_berserker");
    assert_eq!(readiness["resonance_max"],1);
    assert_eq!(readiness["resonance_current"],1);
    assert_eq!(readiness["eligibility"]["enabled"],true);
    assert_eq!(readiness["manual_activation_allowed"],true);
    assert_eq!(readiness["selection"],"player_unit");
    Ok(id)
}

#[actix_web::test]
#[ignore="requires actual approved expression adversarial config and exclusive evidence; run explicit U11 selector"]
async fn u11_normal_approved_actor_real_u03_fault_isolated(){
    let run=async{
        let(_,parent)=super::inputs()?;let out=parent.join("normal_actor_fault");fs::create_dir(&out)?;
        let original_cfg=PathBuf::from(std::env::var_os("U11_NORMAL_ACTOR_FAULT_CONFIG").ok_or("actual calibrated adversarial host config required")?);
        let mut cfg=config(&original_cfg)?;
        let mut cfg_b=config(&PathBuf::from(std::env::var_os("U11_NORMAL_ACTOR_FAULT_B_CONFIG")
            .ok_or("actual calibrated adversarial B host config required")?))?;
        assert_eq!(cfg_b.issuer_root,cfg.issuer_root,"fault A/B need the same actual issuer");
        assert_eq!(fs::read(&cfg_b.runtime_safety_path)?,fs::read(&cfg.runtime_safety_path)?,
            "B approval must use the same actual unchanged parent safety context");
        let actual=fs::read(&cfg.runtime_safety_path)?;
        let parent_profile=PathBuf::from(std::env::var_os("U11_NORMAL_ACTOR_FAULT_PARENT_PROFILE")
            .ok_or("exact approved parent safety profile path required")?);
        assert_eq!(actual,fs::read(&parent_profile)?,"expression successor retains exact reviewed parent safety bytes");
        fs::write(out.join("actual_fault_profile.ron"),&actual)?;
        let original_cache=state(&cfg.issuer_root)?;let cache=out.join("private_issuer");cache_fixture::copy(&cfg.issuer_root,&cache)?;
        original_cache.ensure_unchanged(&state(&cfg.issuer_root)?)?;cfg.issuer_root=cache.clone();cfg_b.issuer_root=cache.clone();
        let mut admission=open(&cfg)?;save(&out.join("actual_complete_approval.json"),&admission.report)?;
        let required_a=admission.query()?.required;
        let raw_a=acquire(&cfg)?.generation().clone();
        let raw_b=acquire(&cfg_b)?.generation().clone();
        let root=out.join("save");let persistence=ApprovedPersistence::open(&root,Arc::clone(&admission.host))?;
        let lb=LoadBalanceActor::with_approved_persistence(Arc::clone(&admission.host),persistence).start();
        let(first,session,_sink)=actor(&lb,Uuid::from_u128(0x5203),23,&raw_b,&required_a.content_identity).await?;
        let(other,_other_session,_other_sink)=actor(&lb,Uuid::from_u128(0x5204),24,&raw_b,&required_a.content_identity).await?;
        command(&first,session,json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false}),&out.join("start.json")).await?;
        starters(&first,session,&out.join("starters")).await?;
        let mut selected=false;
        for index in 0..128 {
            let view=first.send(ReadOnly).await?;assert!(view["fault"].is_null());
            let run:RunSnapshotDto=serde_json::from_value(view["run"].clone())?;let map=run.map.ok_or("normal map required")?;
            if let Some(node)=map.nodes.iter().find(|n|n.state==MapNodeState::Available&&n.category==MapNodeCategory::Combat){
                command(&first,session,json!({"type":"select_map_node","node_id":node.id}),&out.join("select_combat.json")).await?;selected=true;break;
            }
            let node=map.nodes.iter().find(|n|n.state==MapNodeState::Available&&n.category==MapNodeCategory::Start)
                .ok_or("seed23 no adjacent normal combat/start")?;
            for(step,value)in [json!({"type":"select_map_node","node_id":node.id}),json!({"type":"confirm_enter_node"}),json!({"type":"complete_node"})].into_iter().enumerate(){
                command(&first,session,value,&out.join(format!("map_{index}_{step}.json"))).await?;
            }
        }
        assert!(selected,"bounded normal combat selection required");
        let entered=command(&first,session,json!({"type":"confirm_enter_node"}),&out.join("enter_combat.json")).await?;
        let employee=deploy_expression_caster(&first,session,&entered,&out).await?;
        let target_view=first.send(FaultPublicView).await?;
        let target=live_update(&target_view)?["checkpoint"]["units"].as_array().ok_or("normal live target checkpoint required")?.iter()
            .find(|u|u["owner"]=="Opponent"&&u["stats"]["current_health"].as_u64().is_some_and(|hp|hp>10))
            .ok_or("normal paused live hostile target required")?["unit_instance_id"].clone();
        battle_command(&first,session,json!({"type":"begin_skill_targeting","employee_uuid":employee,"skill_id":"training_berserker"}),
            &out.join("begin_actual_hostile_targeting.json")).await?;
        let preview=battle_command(&first,session,json!({"type":"preview_skill_target","employee_uuid":employee,
            "skill_id":"training_berserker","target":{"type":"Unit","unit_instance_id":target}}),&out.join("actual_hostile_target_preview.json")).await?;
        let PlayerGameServerMessage::CommandResult{payload,..}=&preview.response else{return Err("actual target preview response required".into());};
        assert_eq!(payload["target"],json!({"type":"Unit","unit_instance_id":target}));
        assert_eq!(payload["eligibility"]["enabled"],true,"ordinary Sword must pass actual hostile target eligibility before expression evaluation");
        let unrelated_before=other.send(ReadOnly).await?;
        let input=json!({"type":"activate_skill","employee_uuid":employee,"skill_id":"training_berserker",
            "target":{"type":"Unit","unit_instance_id":target}});
        battle_command(&first,session,input.clone(),&out.join("accepted_pending_expression_cast.json")).await?;
        let paused=first.send(FaultPublicView).await?;
        let update=live_update(&paused)?;
        assert_eq!(update["checkpoint"]["playback"]["paused"],true);
        let authority=serde_json::from_value(json!({"battle_uuid":update["battle_uuid"],
            "battle_state_revision":update["battle_state_revision"]}))?;
        let behavior=ApplicationCommand::from_document(json!({"type":"resume_battle"}).into())
            .map_err(|e|format!("normal resume parse: {e:?}"))?;
        let capture=first.send(ResumeAndCapture{command:ExecutePlayerBehavior{session_id:session,
            request_id:"resume-accepted-expression-cast".into(),behavior,battle_authority:Some(authority),
            battle_side_message:BattleSideMessageKind::BattleUpdate},save_root:root.clone()}).await??;
        assert!(matches!(capture.response.response,PlayerGameServerMessage::CommandResult{ok:true,..}));
        let public_before=capture.public_before_tick;
        assert_eq!(live_update(&public_before)?["checkpoint"]["playback"]["paused"],false);
        let durable_before=capture.durable_before_tick;let cache_before=state(&cache)?;
        save(&out.join("actual_resumed_pre_tick_baseline.json"),&json!({"resume_response":capture.response.response,
            "resume_side_messages":capture.response.side_messages,"public_before_tick":public_before,
            "durable_before_tick":durable_before.report()}))?;
        let mut observed_fault=None;
        for _ in 0..400 {
            let current=first.send(ReadOnly).await?;
            if !current["fault"].is_null(){observed_fault=Some(current);break;}
            actix_web::rt::time::sleep(Duration::from_millis(25)).await;
        }
        let first_fault=observed_fault.ok_or("ordinary resumed host tick did not reach expression release fault")?;
        assert_eq!(first_fault["fault"]["operation"],"host_tick");
        assert_eq!(first_fault["fault"]["cause"]["kind"],"expression");
        assert_eq!(first_fault["fault"]["cause"]["code"],"divide_by_zero");
        assert_eq!(first_fault["fault"]["cause"]["phase"],"evaluate");
        let public_after=first.send(FaultPublicView).await?;
        save(&out.join("actual_expression_atomicity.json"),&json!({"input":input,"authority":authority,
            "first_fault":first_fault["fault"],"failed_operation":"normal scheduled HostTick after successful ActivateSkill and ResumeBattle", "before":public_before,"after":public_after,
            "comparison":"all public snapshots except documented fault gate fields, and all public runtime facts and typed resources; complete private-root oracle remains the existing Core unit regression"}))?;
        assert_eq!(atomic_public_projection(&public_after),atomic_public_projection(&public_before),
            "failed expression must preserve all observable state apart from documented fault-gate projections");
        assert_eq!(public_after["snapshots"]["action_capabilities"],json!([]));
        assert_eq!(public_after["reconnect"]["Err"]["kind"],"battle_runtime_fault");
        durable_before.ensure_unchanged(&state(&root)?)?;
        assert_eq!(first_fault["checkpoint_error"]["kind"],"battle_runtime_fault",
            "the actual faulted application must refuse checkpoint export with its typed error");
        assert!(first_fault["checkpoint_sha256"].is_null());
        assert_eq!(first_fault["tick_scheduled"],false);save(&out.join("first_actual_fault.json"),&first_fault)?;
        // A real second approval cannot clear an already faulted actor. New B
        // gameplay uses an explicit separate persistence root so original A's
        // last durable continuation remains the same original throughout.
        let actual_b=admission.validate_configuration_and_apply(&cfg_b)?;
        save(&out.join("actual_b_while_a_faulted.json"),&actual_b)?;
        assert_eq!(actual_b.combination.as_ref().expect("actual B final job").state,JobState::Passed);
        let required_b=admission.query()?.required;assert_ne!(required_a,required_b);
        assert_eq!(first.send(ReadOnly).await?,first_fault,"B Apply cannot clear or replace the first fault");
        durable_before.ensure_unchanged(&state(&root)?)?;
        let b_root=out.join("independent_b_save");
        let b_persistence=ApprovedPersistence::open(&b_root,Arc::clone(&admission.host))?;
        let b_lb=LoadBalanceActor::with_approved_persistence(Arc::clone(&admission.host),b_persistence).start();
        let(b_actor,b_session,_b_sink)=actor(&b_lb,Uuid::from_u128(0x5205),23,&raw_a,&required_b.content_identity).await?;
        command(&b_actor,b_session,json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false}),&out.join("independent_b_start.json")).await?;
        starters(&b_actor,b_session,&out.join("independent_b_starters")).await?;
        let b_accepted=b_actor.send(ReadOnly).await?;
        assert_eq!(b_accepted["required"],serde_json::to_value(&required_b)?);
        assert_eq!(b_accepted.pointer("/run/resources/enkephalin").ok_or("actual B run resources required")?,777,
            "new B actor must consume the changed RunPolicy through normal start");
        assert_eq!(active(&b_root.join("active_run.json"))?.unsigned.required_configuration.as_ref(),Some(&required_b));
        assert_eq!(first.send(ReadOnly).await?,first_fault);
        assert_eq!(other.send(ReadOnly).await?,unrelated_before);
        durable_before.ensure_unchanged(&state(&root)?)?;
        // Cache may legitimately gain B's actual result. Freeze the new accepted
        // inventory for the remaining fault/stop checks, preserving old records.
        let cache_after_b=state(&cache)?;
        cache_before.ensure_original_files_unchanged(&cache_after_b)?;
        for _ in 0..3{first.send(EnsureLiveBattleTick).await?;}
        actix_web::rt::time::sleep(Duration::from_millis(120)).await;
        assert_eq!(first.send(ReadOnly).await?,first_fault,"later ticks cannot change the first-fault state");
        assert_eq!(other.send(ReadOnly).await?,unrelated_before,"unrelated actor remains usable and unchanged");
        durable_before.ensure_unchanged(&state(&root)?)?;cache_after_b.ensure_unchanged(&state(&cache)?)?;
        let behavior=ApplicationCommand::from_document(json!({"type":"load_run_checkpoint"}).into()).map_err(|e|format!("normal load: {e:?}"))?;
        let refusal=first.send(ExecutePlayerBehavior{session_id:session,request_id:"after-real-fault".into(),behavior,
            battle_authority:None,battle_side_message:BattleSideMessageKind::BattleUpdate}).await?.err()
            .ok_or("faulted actor accepted a later mutation")?;
        assert_eq!(refusal.code,"battle_faulted","later mutation must be rejected by the actual first-fault gate");
        save(&out.join("after_fault_refusal.json"),&json!({"code":refusal.code,"message":refusal.message}))?;
        let restored_a=admission.validate_configuration_and_apply(&cfg)?;
        save(&out.join("restore_actual_a_after_fault.json"),&restored_a)?;
        assert_eq!(restored_a.combination.as_ref().expect("actual restored A final job").state,JobState::Passed);
        assert_eq!(admission.query()?.required,required_a);
        assert_eq!(first.send(ReadOnly).await?,first_fault);
        assert_eq!(b_actor.send(ReadOnly).await?,b_accepted,"already started B remains pinned after A restore");
        let cache_before_stop=state(&cache)?;
        cache_after_b.ensure_original_files_unchanged(&cache_before_stop)?;
        first.send(QuitPlayerActor).await?;
        let mut deregistered=false;
        for _ in 0..100 {
            if lb.send(FindPlayer{player_id:Uuid::from_u128(0x5203)}).await?.is_none(){deregistered=true;break;}
            actix_web::rt::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(deregistered,"actual faulted actor must deregister within the bounded wait");
        durable_before.ensure_unchanged(&state(&root)?)?;cache_before_stop.ensure_unchanged(&state(&cache)?)?;
        assert_eq!(other.send(ReadOnly).await?,unrelated_before);
        assert_eq!(lb.send(GetPlayerCount).await?,1,"faulted actor must actually stop before recreation");
        let(restored,restored_session,_restored_sink)=actor(&lb,Uuid::from_u128(0x5203),23,&raw_b,&required_a.content_identity).await?;
        command(&restored,restored_session,json!({"type":"continue_run"}),&out.join("same_original_after_fault_continue.json")).await?;
        let continued=restored.send(ReadOnly).await?;assert!(continued["fault"].is_null());
        let run:RunSnapshotDto=serde_json::from_value(continued["run"].clone())?;
        assert!(run.roster.combat_eligible_employee_ids.len()>=2);
        durable_before.ensure_unchanged(&state(&root)?)?;cache_before_stop.ensure_unchanged(&state(&cache)?)?;
        original_cache.ensure_unchanged(&state(&config(&original_cfg)?.issuer_root)?)?;
        save(&out.join("RESULT.json"),&json!({"passed":true,"actual_approved_path":true,"actual_expression_fault":first_fault["fault"],"public_projection_runtime_fact_and_resource_atomicity":true,
            "unrelated_actor_unchanged":true,"approved_b_cannot_clear_fault":true,"independent_actor_consumed_b":true,
            "original_durable_save_profile_unchanged":true,"cache_acceptance_only_from_real_validation":true,
            "same_original_continued_after_fault":true}))?;
        Ok::<(),Box<dyn std::error::Error>>(())
    };run.await.unwrap();
}
