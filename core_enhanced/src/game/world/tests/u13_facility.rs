//! U13 room movement oracles. The small authored route uses real SavePoint,
//! maintenance and shop actions; its unused combat branches remain intact.
use super::*;
use crate::game::map::*;
use crate::game::data::{ValidatedGameData,live_ron::GameplayRonSourceSet};

fn development_data() -> ValidatedGameData { development_data_with(|_|{}) }
pub(super) fn development_data_with(edit:impl FnOnce(&mut FacilityTemplateDatabase)) -> ValidatedGameData {
    let mut sources=GameplayRonSourceSet::embedded();
    let mut catalog=FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let layout=&mut catalog.layouts[0];
    layout.id="u13_authored_movement".into();
    // This movement fixture removes all fixed boss reservations.
    layout.golden_bough_plans.clear();
    for activity in &mut layout.activities { activity.golden_bough_candidate=false; }
    let initial=layout.room("B1-01").unwrap().clone();
    let mut rooms=vec![layout.room("ENTRY").unwrap().clone()];
    for (id,depth) in [("B1-01",1),("B1-02",1),("B2-01",2),("B3-01",3)] {
        let mut r=initial.clone();r.id=id.into();r.depth=depth;r.name=id.into();r.concept="작성된 지원 활동 경로".into();
        r.reservations=vec![FacilityReservation{slot_id:MapSlotId::new("2"),activity_id:"save".into()},FacilityReservation{slot_id:MapSlotId::new("5"),activity_id:"maintenance".into()},FacilityReservation{slot_id:MapSlotId::new("8"),activity_id:"shop".into()}];
        r.omen_slot=(depth==3).then(||MapSlotId::new("3"));r.elite_slots=vec![];r.safe_access_slots=vec![MapSlotId::new("2"),MapSlotId::new("5"),MapSlotId::new("8")];rooms.push(r);
    }
    layout.rooms=rooms;
    layout.connections=vec![
        FacilityRoomConnection{from:"ENTRY".into(),to:"B1-01".into(),direction:MapEdgeDirection::ForwardOnly,condition:None},
        FacilityRoomConnection{from:"B1-01".into(),to:"B1-02".into(),direction:MapEdgeDirection::Bidirectional,condition:None},
        FacilityRoomConnection{from:"B1-01".into(),to:"B2-01".into(),direction:MapEdgeDirection::ForwardOnly,condition:None},
        FacilityRoomConnection{from:"B2-01".into(),to:"B3-01".into(),direction:MapEdgeDirection::ForwardOnly,condition:None},
    ];
    // No elite candidates in this small movement fixture. General battles still
    // occupy every unreserved activity not selected as event/reward/support.
    for size in &mut layout.allocation.sizes {for weight in size.elite_percent_by_depth.values_mut(){*weight=0;}}
    edit(&mut catalog);
    let text=ron::ser::to_string(&catalog).unwrap();sources.facility_templates=&text;
    GameDataBase::load_live_sources(sources).expect("supported authored route loads through normal validation")
}
pub(super) fn start(data:ValidatedGameData,seed:u64)->(GameCore,Uuid) {
    let player=Uuid::from_u128(13000);let mut core=GameCore::new(data,seed);
    core.execute(player,PlayerBehavior::StartNewGame{game_mode:GameMode::Expedition,replace_active_run:false}).unwrap();
    for _ in 0..16 {
        let run=core.get_run_snapshot_dto().unwrap();
        let Some(vn)=run.visual_novel else {break;};
        core.execute(player,PlayerBehavior::AdvanceVisualNovel{session_id:vn.session_id,current_scene_id:VisualNovelSceneId::new(vn.current_scene_id)}).unwrap();
    }
    let ids=core.state.starter_candidates.iter().take(3).map(|c|c.employee_definition_id.clone()).collect();
    core.execute(player,PlayerBehavior::SelectStarterEmployees{employee_definition_ids:ids,loan_copy_ids:vec![]}).unwrap();(core,player)
}
fn slot(core:&GameCore,id:&str)->MapNodeId {core.state.run.as_ref().unwrap().map.nodes.iter().find(|n|n.slot_id.as_str()==id).unwrap().id}
fn enter(core:&mut GameCore,player:Uuid,id:&str) {
    let id=slot(core,id);core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();
    if core.state.run.as_ref().unwrap().map.node(id).unwrap().category == MapNodeCategory::Gate {
        let snapshot=serde_json::to_value(core.get_run_snapshot_dto().unwrap()).unwrap();
        assert!(snapshot["game_state_context"]["trauma_recovery_preview"].is_null(),"Gate confirmation alone never promises room-descent healing");
    }
    core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
    if core.state.run.as_ref().unwrap().run_progression.pending_continuation.is_some() {
        core.execute(player,PlayerBehavior::ChooseExpeditionContinuation{choice:crate::game::expedition_lifecycle::ExpeditionContinuationChoice::Continue}).unwrap();
    }
}
fn finish_route(core:&mut GameCore,player:Uuid) {
    enter(core,player,"2");core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    enter(core,player,"5");core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    enter(core,player,"8");core.execute(player,PlayerBehavior::ExitShop).unwrap();
    enter(core,player,"G");
    if core.state.run.as_ref().unwrap().run_progression.pending_continuation.is_some() {
        core.execute(player,PlayerBehavior::ChooseExpeditionContinuation{choice:crate::game::expedition_lifecycle::ExpeditionContinuationChoice::Continue}).unwrap();
    }
    assert!(core.facility_gate_reached());
}
fn move_room(core:&mut GameCore,player:Uuid,id:&str) {core.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:id.into()}).unwrap();}
fn roots(core:&GameCore)->Value {json!({"run":core.get_run_snapshot_dto().unwrap(),"profile":core.state.player_profile,"save":core.export_active_run_checkpoint().unwrap().map(|s|s.core_payload_json)})}

#[test]
fn u13_official_public_disclosure_and_unsupported_approach_are_read_only() {
    let (mut core,player)=start(ValidatedGameData::load_live_embedded().unwrap(),13001);
    let view=core.facility_view().unwrap().unwrap();
    assert_eq!(view.rooms.len(),85,"four hidden final rooms are not advertised");
    assert_eq!(view.current_room_id,"B1-01");
    assert!(view.rooms.iter().find(|r|r.id=="B1-02").unwrap().name.is_some());
    assert!(view.rooms.iter().find(|r|r.id=="A7-01").unwrap().name.is_none());
    assert!(view.rooms.iter().filter(|r|r.id!="B1-01").all(|r|r.current_expedition_map.is_none() && r.interior.is_none()));
    let before=roots(&core);
    core.execute(player,PlayerBehavior::RequestMapData).unwrap();assert_eq!(roots(&core),before);
    for id in ["B1-02","A6-H","C8-F","missing"] {assert!(core.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:id.into()}).is_err());assert_eq!(roots(&core),before);}
    let reward=slot(&core,"2");
    let activity=view.rooms.iter().find(|r|r.id=="B1-01").unwrap().activities.iter().find(|a|a.node_id==reward).unwrap();
    assert_eq!(activity.reward_activity,Some(RewardActivityKind::SimpleChoice));
    assert!(activity.support.is_supported(), "U16-2 now executes the reserved simple choice");
    core.execute(player,PlayerBehavior::SelectMapNode{node_id:reward}).unwrap();
    core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
    let choice=core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert!(choice["rewards"].as_array().unwrap().is_empty(), "empty official relic catalog must not grant generic salvage");
    assert_eq!(choice["relic_activity"]["no_candidates_reason"], "no_candidates");
    core.execute(player,PlayerBehavior::ExitReward).unwrap();
    assert!(core.state.relics.is_empty());
    assert_eq!(core.state.run.as_ref().unwrap().map.node(reward).unwrap().state,MapNodeState::Completed);
}

#[test]
fn u13_real_gate_roundtrip_preserves_assignments_receipts_and_continue() {
    let data=development_data();let (mut core,player)=start(data.clone(),13011);
    let before=roots(&core);let save=slot(&core,"2");
    core.execute(player,PlayerBehavior::SelectMapNode{node_id:save}).unwrap();core.execute(player,PlayerBehavior::CancelSelectedNode).unwrap();
    assert_eq!(roots(&core),before,"supported preview cancellation is not entry");
    let initial=core.state.run.as_ref().unwrap().map.clone();
    finish_route(&mut core,player);
    assert!(core.state.player_profile.facility_knowledge.is_empty(),"Gate arrival alone does not complete room");
    let resolved=core.state.run.as_ref().unwrap().map.clone();
    let receipts=core.state.run.as_ref().unwrap().run_progression.entry_receipts.clone();
    let cost=core.state.run.as_ref().unwrap().run_progression.deterrence;
    move_room(&mut core,player,"B1-02");
    assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence,cost);
    assert!(core.state.player_profile.facility_knowledge["u13_authored_movement"].completed_rooms.contains("B1-01"));
    assert!(initial.nodes.iter().all(|n|core.state.run.as_ref().unwrap().map.node(n.id).is_none()),"room node IDs must be disjoint");
    finish_route(&mut core,player);let before_cost=core.state.run.as_ref().unwrap().run_progression.deterrence;
    move_room(&mut core,player,"B1-01");
    let run=core.state.run.as_ref().unwrap();
    for node in &resolved.nodes {let restored=run.map.node(node.id).unwrap();assert_eq!(restored.kind_id,node.kind_id);assert_eq!(restored.payload,node.payload);if node.state==MapNodeState::Completed {assert_eq!(restored.state,node.state);}}
    assert!(receipts.is_subset(&run.run_progression.entry_receipts));assert_eq!(run.run_progression.deterrence,before_cost);
    assert!(core.facility_gate_reached(),"cleared route reaches the same Gate without another entry charge");
    let saved=core.export_active_run_checkpoint().unwrap().unwrap();
    let mut fresh=GameCore::new(data,999);fresh.install_authoritative_player_profile(core.state.player_profile.clone()).unwrap();
    fresh.continue_from_active_run_checkpoint(player,&saved.core_payload_json,saved.loads_used,saved.run_peak_certified_power).unwrap();
    assert_eq!(fresh.facility_view().unwrap().unwrap().current_room_id,"B1-01");
    assert_eq!(serde_json::to_value(&fresh.state.run.as_ref().unwrap().facility_run_state).unwrap(),serde_json::to_value(&core.state.run.as_ref().unwrap().facility_run_state).unwrap());
    assert_eq!(fresh.state.run.as_ref().unwrap().run_progression.entry_receipts,core.state.run.as_ref().unwrap().run_progression.entry_receipts);
    move_room(&mut fresh,player,"B1-02");assert_eq!(fresh.state.run.as_ref().unwrap().run_progression.deterrence,before_cost);
    fresh.execute(player,PlayerBehavior::AbandonRun).unwrap();
    fresh.execute(player,PlayerBehavior::StartNewGame{game_mode:GameMode::Expedition,replace_active_run:false}).unwrap();
    for _ in 0..16 {
        let Some(vn)=fresh.get_run_snapshot_dto().unwrap().visual_novel else {break;};
        fresh.execute(player,PlayerBehavior::AdvanceVisualNovel{session_id:vn.session_id,current_scene_id:VisualNovelSceneId::new(vn.current_scene_id)}).unwrap();
    }
    let ids=fresh.state.starter_candidates.iter().take(3).map(|c|c.employee_definition_id.clone()).collect();
    fresh.execute(player,PlayerBehavior::SelectStarterEmployees{employee_definition_ids:ids,loan_copy_ids:vec![]}).unwrap();
    let view=fresh.facility_view().unwrap().unwrap();
    let previous=view.rooms.iter().find(|r|r.id=="B1-02").unwrap();
    assert!(previous.explored && !previous.visited_this_expedition);
    assert!(previous.interior.is_some() && previous.current_expedition_map.is_none());
    assert_eq!(previous.known_reservations.len(),3);
    assert!(fresh.state.run.as_ref().unwrap().run_progression.entry_receipts.is_empty());
    assert!(fresh.state.run.as_ref().unwrap().facility_run_state.rooms.as_ref().unwrap().visited_rooms.is_empty());

}

#[test]
fn u13_descent_only_recovery_savepoint_keeps_permanent_exploration_and_gate_return() {
    let (mut core,player)=start(development_data(),13021);
    finish_route(&mut core,player);
    for e in core.state.roster.iter_mut(){e.condition.trauma=37;}
    move_room(&mut core,player,"B1-02");assert!(core.state.roster.iter().all(|e|e.condition.trauma==37));
    finish_route(&mut core,player);move_room(&mut core,player,"B1-01");
    for e in core.state.roster.iter_mut(){e.condition.trauma=37;}
    move_room(&mut core,player,"B2-01");assert!(core.state.roster.iter().all(|e|e.condition.trauma==33),"ceil(37*10%) recovery");
    let before=roots(&core);assert!(core.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B1-01".into()}).is_err());assert_eq!(roots(&core),before);
    finish_route(&mut core,player);move_room(&mut core,player,"B3-01");
    let knowledge=core.state.player_profile.facility_knowledge.clone();
    core.execute(player,PlayerBehavior::LoadRunCheckpoint).unwrap();
    assert_eq!(core.state.player_profile.facility_knowledge,knowledge,"SavePoint never rewinds permanent exploration");
    assert_eq!(core.facility_view().unwrap().unwrap().current_room_id,"B2-01");
    // SavePoint was on slot2; complete the remaining real support route again.
    enter(&mut core,player,"5");core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    enter(&mut core,player,"8");core.execute(player,PlayerBehavior::ExitShop).unwrap();enter(&mut core,player,"G");
    move_room(&mut core,player,"B3-01");
    finish_route(&mut core,player);
    core.execute(player,PlayerBehavior::ChooseExpeditionContinuation{choice:crate::game::expedition_lifecycle::ExpeditionContinuationChoice::Return}).unwrap();
    assert!(core.state.player_profile.facility_knowledge["u13_authored_movement"].completed_rooms.contains("B3-01"));
    assert!(core.export_active_run_checkpoint().unwrap().is_none());
}

#[test]
fn u13_reachable_unsupported_room_rejects_retries_and_continue_without_cost() {
    let data=development_data_with(|db| {
        db.layouts[0].rooms.iter_mut().find(|r|r.id=="B2-01").unwrap().support=FacilityContentSupport::Unsupported{reason:"authored executor not supplied".into()};
    });
    let (mut core,player)=start(data.clone(),13041);finish_route(&mut core,player);
    let view=core.facility_view().unwrap().unwrap();
    assert!(matches!(view.rooms.iter().find(|r|r.id=="B2-01").unwrap().support,Some(FacilityContentSupport::Unsupported{..})));
    for _ in 0..2 {
        let before=roots(&core);
        assert!(core.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B2-01".into()}).is_err());
        assert_eq!(roots(&core),before);
    }
    let saved=core.export_active_run_checkpoint().unwrap().unwrap();
    let mut fresh=GameCore::new(data,11);fresh.install_authoritative_player_profile(core.state.player_profile.clone()).unwrap();
    fresh.continue_from_active_run_checkpoint(player,&saved.core_payload_json,saved.loads_used,saved.run_peak_certified_power).unwrap();
    let before=roots(&fresh);
    assert!(fresh.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B2-01".into()}).is_err());assert_eq!(roots(&fresh),before);
    move_room(&mut fresh,player,"B1-02");
}

#[test]
fn u13_authored_category_and_missing_reference_fail_before_runtime() {
    for mistake in 0..3 {
        let mut sources=GameplayRonSourceSet::embedded();
        let mut db=FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
        match mistake {
            0=>{db.templates.iter_mut().find(|t|t.id.as_str()=="facility_9C").unwrap().slots.iter_mut().find(|s|s.id.as_str()=="4").unwrap().allowed_categories.retain(|c|*c!=MapNodeCategory::Reward);},
            1=>{db.layouts[0].rooms.iter_mut().find(|r|r.id=="B1-01").unwrap().reservations[0].activity_id="missing".into();},
            _=>{db.layouts[0].allocation.support_counts=std::collections::BTreeMap::from([(3,100)]);},
        }
        let text=ron::ser::to_string(&db).unwrap();sources.facility_templates=&text;
        assert!(GameDataBase::load_live_sources(sources).is_err(),"normal authoring error {mistake}");
    }
}

#[test]
fn u13_recovery_gate_refreshes_destination_witness_and_descent_clears_restriction() {
    let data=development_data();let (mut core,player)=start(data.clone(),13051);
    enter(&mut core,player,"2");core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    enter(&mut core,player,"5");core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    enter(&mut core,player,"8");core.execute(player,PlayerBehavior::ExitShop).unwrap();
    // Reachable post-combat condition fixture, separate from the actual route.
    for employee in core.state.roster.iter_mut(){employee.condition.trauma=100;}
    core.reconcile_employee_breakdown();
    assert_eq!(core.reassess_combat_recovery().unwrap(),None);
    assert!(core.has_combat_recovery_restriction());
    enter(&mut core,player,"G");
    let saved=core.export_active_run_checkpoint().unwrap().unwrap();
    let mut other=GameCore::new(data,3);other.install_authoritative_player_profile(core.state.player_profile.clone()).unwrap();
    other.continue_from_active_run_checkpoint(player,&saved.core_payload_json,saved.loads_used,saved.run_peak_certified_power).unwrap();
    move_room(&mut core,player,"B1-02");
    assert!(core.state.roster.iter().all(|e|e.condition.trauma==100));
    let run=core.state.run.as_ref().unwrap();
    assert!(run.combat_recovery_restriction.as_ref().unwrap().allowed_node_ids.iter().all(|id|run.map.node(*id).is_some()));
    enter(&mut core,player,"2");core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    assert!(!core.has_combat_recovery_restriction());
    move_room(&mut other,player,"B2-01");
    assert!(other.state.roster.iter().all(|e|e.condition.trauma==90));
    assert!(!other.has_combat_recovery_restriction());
}

#[test]
fn u13_room_move_budget_refusal_cancel_and_retry_preserve_all_roots() {
    use crate::authoring::control::{self,CancellationToken,ExecutionPhase,NativeStorageProvenance};
    let data=development_data();let limits=data.runtime_safety().authoring_execution();let token=CancellationToken::default();
    let ((mut core,player),_,provenance)=control::run_with_provenance(&limits,&token,ExecutionPhase::TrialPreparation,NativeStorageProvenance::fresh(),|| {
        let (mut core,player)=start(data,13061);finish_route(&mut core,player);(core,player)
    }).unwrap();
    let before=roots(&core);let mut successful=core.clone();
    let (result,usage,_)=control::run_with_provenance(&limits,&token,ExecutionPhase::TrialPreparation,provenance,|| {
        successful.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B1-02".into()})
    }).unwrap();result.unwrap();assert!(usage.work>1 && usage.clone_slots_peak>1);
    println!("U13_MOVE_USAGE {}",serde_json::to_string(&usage).unwrap());
    for work in [true,false] {
        let mut exact=limits.clone();if work {exact.max_work=usage.work;}else{exact.max_clone_slots=usage.clone_slots_peak;}
        let mut candidate=core.clone();
        let (result,_,_)=control::run_with_provenance(&exact,&token,ExecutionPhase::TrialPreparation,provenance,||candidate.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B1-02".into()})).unwrap();
        result.unwrap();assert_eq!(roots(&candidate),roots(&successful));
        if work {exact.max_work-=1;}else{exact.max_clone_slots-=1;}
        let mut candidate=core.clone();
        for _ in 0..2 {
            assert!(control::run_with_provenance(&exact,&token,ExecutionPhase::TrialPreparation,provenance,||candidate.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B1-02".into()})).is_err());
            assert_eq!(roots(&candidate),before);
        }
        candidate.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B1-02".into()}).unwrap();assert_eq!(roots(&candidate),roots(&successful));
    }
    let cancelled=CancellationToken::default();cancelled.cancel();
    assert!(control::run_with_provenance(&limits,&cancelled,ExecutionPhase::TrialPreparation,provenance,||core.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B1-02".into()})).is_err());
    assert_eq!(roots(&core),before);
}

#[test]
fn u13_allocation_roles_reject_semantic_mismatches_and_allow_activity_aliases() {
    for (role, wrong) in [("normal","maintenance"),("normal","elite"),("elite","normal"),("event","normal"),("reward","maintenance"),("gate","maintenance"),("support","save"),("support","early_reward")] {
        let mut sources=GameplayRonSourceSet::embedded();
        let mut db=FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
        let a=&mut db.layouts[0].allocation;
        let wrong=match wrong {"normal"=>a.normal_activity_id.clone(),"elite"=>a.elite_activity_id.clone(),other=>other.into()};
        match role {"normal"=>a.normal_activity_id=wrong,"elite"=>a.elite_activity_id=wrong,"event"=>a.event_activity_id=wrong,"reward"=>a.reward_activity_id=wrong,"gate"=>a.gate_activity_id=wrong,_=>{a.support_activities.insert(wrong,1);}}
        let text=ron::ser::to_string(&db).unwrap();sources.facility_templates=&text;
        let err=GameDataBase::load_live_sources(sources).err().expect("wrong role must fail").to_string();
        assert!(err.contains(&format!("allocation role '{role}'")),"wrong rejection: {err}");
    }
    let mut sources=GameplayRonSourceSet::embedded();
    let mut db=FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let layout=&mut db.layouts[0];
    for id in [&mut layout.allocation.normal_activity_id,&mut layout.allocation.elite_activity_id,&mut layout.allocation.event_activity_id,&mut layout.allocation.reward_activity_id,&mut layout.allocation.gate_activity_id] {
        let mut a=layout.activities.iter().find(|a|a.id==*id).unwrap().clone();
        a.id=format!("alias_{}",a.id);*id=a.id.clone();layout.activities.push(a);
    }
    let text=ron::ser::to_string(&db).unwrap();sources.facility_templates=&text;
    GameDataBase::load_live_sources(sources).expect("activity names do not define semantic roles");
}

fn fixed_encounter_data() -> ValidatedGameData {
    development_data_with(|db| {
        let layout=&mut db.layouts[0];
        let mut encounter=layout.activity(&layout.allocation.normal_activity_id).unwrap().clone();
        encounter.id="fixed_encounter".into();
        encounter.payload=MapNodePayload::Encounter{encounter_id:Some("normal_corroded_patrol".into())};
        layout.activities.push(encounter);
        let room=layout.rooms.iter_mut().find(|r|r.id=="B1-01").unwrap();
        room.template_id=Some(MapTemplateId::new("facility_9B"));
        room.reservations=[("1","save"),("4","maintenance"),("7","shop"),("5","fixed_encounter")].into_iter().map(|(slot,activity)|FacilityReservation{slot_id:MapSlotId::new(slot),activity_id:activity.into()}).collect();
        room.safe_access_slots=["1","4","7"].into_iter().map(MapSlotId::new).collect();
    })
}
fn assert_fixed_disclosure(core:&mut GameCore,player:Uuid,visible:bool) {
    let id=slot(core,"5");
    let expected=MapNodePayload::Encounter{encounter_id:visible.then(||"normal_corroded_patrol".into())};
    let before=roots(core);
    let snapshot=core.get_run_snapshot_dto().unwrap();
    assert_eq!(snapshot.map.unwrap().nodes.iter().find(|n|n.id==id).unwrap().payload,expected);
    let nested=snapshot.facility.unwrap().rooms.into_iter().find(|r|r.id=="B1-01").unwrap().current_expedition_map.unwrap();
    assert_eq!(nested.nodes.iter().find(|n|n.id==id).unwrap().payload,expected);
    let BehaviorResult::MapState{map}=core.execute(player,PlayerBehavior::RequestMapData).unwrap() else {panic!("map response")};
    assert_eq!(map.nodes.iter().find(|n|n.id==id).unwrap().payload,expected);
    assert_eq!(roots(core),before,"queries cannot change run/profile/save");
    assert_eq!(core.state.run.as_ref().unwrap().map.node(id).unwrap().payload,MapNodePayload::Encounter{encounter_id:Some("normal_corroded_patrol".into())},"projection cannot erase execution authority");
}
#[test]
fn u13_fixed_encounter_disclosure_tracks_real_room_completion_and_continue() {
    let data=fixed_encounter_data();let (mut core,player)=start(data.clone(),13071);
    assert_fixed_disclosure(&mut core,player,false);
    enter(&mut core,player,"1");
    let BehaviorResult::NodeCompleted{map,..}=core.execute(player,PlayerBehavior::CompleteNode).unwrap() else {panic!("node completion required")};
    assert_eq!(map.nodes.iter().find(|n|n.slot_id.as_str()=="5").unwrap().payload,MapNodePayload::Encounter{encounter_id:None},"completion map must conceal fixed encounter");
    enter(&mut core,player,"4");core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    let id=slot(&core,"5");let before=roots(&core);
    let BehaviorResult::NodePreview{payload,session,combat_preview,..}=core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap() else {panic!("preview")};
    assert_eq!(payload,MapNodePayload::Encounter{encounter_id:None});assert_eq!(session.payload,payload);assert!(combat_preview.is_none());
    let snapshot=core.get_run_snapshot_dto().unwrap();assert_eq!(snapshot.current_node_session.unwrap().payload,payload);
    let crate::game::behavior::GameStateContextDto::NodeConfirm{combat_preview,..}=snapshot.game_state_context else {panic!("confirmation context required")};
    assert!(combat_preview.is_none());
    core.execute(player,PlayerBehavior::CancelSelectedNode).unwrap();assert_eq!(roots(&core),before);
    let saved=core.export_active_run_checkpoint().unwrap().unwrap();
    let mut fresh=GameCore::new(data.clone(),1);fresh.install_authoritative_player_profile(core.state.player_profile.clone()).unwrap();
    fresh.continue_from_active_run_checkpoint(player,&saved.core_payload_json,saved.loads_used,saved.run_peak_certified_power).unwrap();
    assert_fixed_disclosure(&mut fresh,player,false);
    // A real entry still resolves the original concrete encounter.
    core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
    assert!(serde_json::to_value(core.get_run_snapshot_dto().unwrap().game_state_context).unwrap().to_string().contains("normal_corroded_patrol"));
    enter(&mut fresh,player,"7");fresh.execute(player,PlayerBehavior::ExitShop).unwrap();enter(&mut fresh,player,"G");
    assert_fixed_disclosure(&mut fresh,player,false);
    move_room(&mut fresh,player,"B1-02");finish_route(&mut fresh,player);move_room(&mut fresh,player,"B1-01");
    assert_fixed_disclosure(&mut fresh,player,true);
    let saved=fresh.export_active_run_checkpoint().unwrap().unwrap();
    let mut resumed=GameCore::new(data,2);resumed.install_authoritative_player_profile(fresh.state.player_profile.clone()).unwrap();
    resumed.continue_from_active_run_checkpoint(player,&saved.core_payload_json,saved.loads_used,saved.run_peak_certified_power).unwrap();
    assert_fixed_disclosure(&mut resumed,player,true);
}
