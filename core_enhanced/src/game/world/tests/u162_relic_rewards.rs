//! Policy oracles through normal authored maps and routed Core commands.
//! Private setup grants are explicit; fresh-process normal host evidence is separate.
use super::*;
use crate::game::{map::*, relic_reward::*, data::{relic_data::RelicDatabase,live_ron::GameplayRonSourceSet,ValidatedGameData}};

fn data(kinds: [RewardActivityKind;3], edit: impl FnOnce(&mut BasicRelicRewardPolicy)) -> ValidatedGameData {
    let mut sources = GameplayRonSourceSet::embedded();
    let mut relics: RelicDatabase = ron::de::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/u08/relics.ron"))).unwrap();
    let original = relics.relics.remove(0);
    for (g,rarity) in ["common","rare","special"].iter().enumerate() {
        for i in 0..4 { let mut d = original.clone(); d.id = format!("{rarity}_{i}"); d.uuid=Uuid::from_u128(162000 + g as u128*10+i); d.rarity=rarity.to_string(); relics.relics.push(d); }
    }
    let mut catalog=FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
    let layout=&mut catalog.layouts[0]; layout.id="u162_authored".into();layout.golden_bough_plans.clear();
    for a in &mut layout.activities { a.golden_bough_candidate=false; }
    layout.rooms.retain(|r|r.id=="ENTRY" || r.id=="B1-01");
    layout.connections.retain(|c|c.from=="ENTRY" && c.to=="B1-01");
    let base=layout.activity("early_reward").unwrap().clone();
    for (i,kind) in kinds.into_iter().enumerate() {let mut a=base.clone();a.id=format!("basic_{i}");a.support=FacilityContentSupport::Supported;a.reward_activity=Some(kind);layout.activities.push(a);}
    let room=layout.rooms.iter_mut().find(|r|r.id=="B1-01").unwrap();
    room.reservations=["2","5","8"].into_iter().enumerate().map(|(i,id)|FacilityReservation{slot_id:MapSlotId::new(id),activity_id:format!("basic_{i}")}).collect();
    room.elite_slots.clear();room.omen_slot=None;
    for size in &mut layout.allocation.sizes { for weight in size.elite_percent_by_depth.values_mut(){*weight=0;} }
    layout.relic_rewards.candidates=relics.relics.iter().map(|d|RelicRewardCandidate{relic_id:d.id.clone(),weight:1}).collect();
    edit(&mut layout.relic_rewards);
    let text=ron::ser::to_string(&catalog).unwrap();let relic_text=ron::ser::to_string(&relics).unwrap();
    sources.facility_templates=&text;sources.relics=&relic_text;
    GameDataBase::load_live_sources(sources).expect("supported authored activity inputs")
}
fn setup(kind:RewardActivityKind,edit:impl FnOnce(&mut BasicRelicRewardPolicy))->(GameCore,Uuid) {
    u13_facility::start(data([kind;3],edit),16201)
}
fn enter(core:&mut GameCore,player:Uuid,slot:&str) -> MapNodeId {
    let id=core.state.run.as_ref().unwrap().map.nodes.iter().find(|n|n.slot_id.as_str()==slot).unwrap().id;
    core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();
    core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap(); id
}
fn view(core:&GameCore)->Value {core.get_selected_event_snapshot_json().unwrap().unwrap()}
fn roots(core:&GameCore)->Value {json!({"world":non_battle_state_without_profile_fingerprint(core),"relics":core.state.relics,"crafting":core.state.crafting_resources,"uuid":core.state.uuid_manager,"continue":core.export_active_run_checkpoint().unwrap().map(|v|v.core_payload_json)})}
fn give(core:&mut GameCore,ids:&[&str],dust:u32)->Vec<Uuid> {
    let mut effects=ids.iter().map(|id|RewardEffect::GrantRelic{relic_id:id.to_string()}).collect::<Vec<_>>();
    if dust>0 {effects.push(RewardEffect::GrantEquipmentDust{amount:dust});}
    core.apply_grant_effects(&effects).unwrap().relic_diffs.iter().map(|c|c.copy_uuid).collect()
}
fn choose(core:&mut GameCore,player:Uuid)->crate::game::relic_reward::RelicRewardReceipt {
    let id=core.state.active_node_content.as_ref().unwrap().as_reward().unwrap().rewards[0].uuid;
    core.execute(player,PlayerBehavior::SelectReward{reward_id:id}).unwrap();
    let result=core.execute(player,PlayerBehavior::ClaimReward).unwrap();
    let wire=serde_json::to_value(&result).unwrap();let decoded:BehaviorResult=serde_json::from_value(wire).unwrap();
    let BehaviorResult::RewardGranted{relic_activity:Some(receipt),..}=decoded else {panic!("activity receipt");}; receipt
}
fn restore(core:&GameCore,player:Uuid)->GameCore {
    let saved=core.export_active_run_checkpoint().unwrap().unwrap();let mut fresh=GameCore::new(core.game_data.clone(),999);
    fresh.install_authoritative_player_profile(core.state.player_profile.clone()).unwrap();
    fresh.continue_from_active_run_checkpoint(player,&saved.core_payload_json,saved.loads_used,saved.run_peak_certified_power).unwrap();fresh
}

#[test]
fn u162_choice_distinct_shortage_zero_skip_and_once() {
    for count in 0..=4 {
        let (mut core,p)=setup(RewardActivityKind::SimpleChoice,|pol|pol.candidates.truncate(count));
        let node=enter(&mut core,p,"2");let before=roots(&core);let snapshot=view(&core);
        let candidates=snapshot["rewards"].as_array().unwrap();assert_eq!(candidates.len(),count.min(3));
        let ids=candidates.iter().map(|c|c["id"].as_str().unwrap()).collect::<std::collections::HashSet<_>>();assert_eq!(ids.len(),candidates.len());
        assert_eq!(snapshot["relic_activity"]["candidates"].as_array().unwrap().len(),count.min(3));
        assert_eq!(roots(&core),before,"read-only candidates/effects");
        if count==0 {
            assert_eq!(snapshot["relic_activity"]["no_candidates_reason"],"no_candidates");
            assert!(core.execute(p,PlayerBehavior::ClaimReward).is_err());
            core.execute(p,PlayerBehavior::ExitReward).unwrap();assert_eq!(core.state.relics.copies().count(),0);
        } else {let receipt=choose(&mut core,p);assert_eq!(receipt.granted.len(),1);assert!(receipt.consumed_copies.is_empty());assert_eq!(receipt.paid_dust,0);assert_eq!(receipt.granted[0].variant,crate::game::relic::RelicVariant::Normal);}
        assert_eq!(core.state.run.as_ref().unwrap().map.node(node).unwrap().state,MapNodeState::Completed);
        let after=roots(&core);assert!(core.execute(p,PlayerBehavior::ClaimReward).is_err());assert!(core.execute(p,PlayerBehavior::ExitReward).is_err());assert_eq!(roots(&core),after);
        let fresh=restore(&core,p);assert_eq!(fresh.state.relics,core.state.relics);
    }
}
#[test]
fn u162_investment_threshold_replacement_and_final_single_grant() {
    let (mut core,p)=setup(RewardActivityKind::ResourceInvestment,|_|{});give(&mut core,&[],18);let node=enter(&mut core,p,"2");
    let mut stale=None;
    for (grade,left,paid,next) in [("Common",15,3,Some(6)),("Rare",9,9,Some(9)),("Special",0,18,None)] {
        core.execute(p,PlayerBehavior::InvestRelicReward).unwrap();let v=view(&core);
        assert_eq!(core.state.crafting_resources.equipment_dust,left);assert_eq!(v["relic_activity"]["investment_grade"],grade);assert_eq!(v["relic_activity"]["paid_dust"],paid);
        assert_eq!(v["relic_activity"]["investment"]["additional_dust"],json!(next));assert!(v["relic_activity"]["investment"].get("candidates").is_none());
        assert_eq!(core.state.relics.copies().count(),0);
        if let Some(id)=stale {let before=roots(&core);assert!(core.execute(p,PlayerBehavior::SelectReward{reward_id:id}).is_err());assert_eq!(roots(&core),before);}
        stale=Some(core.state.active_node_content.as_ref().unwrap().as_reward().unwrap().rewards[0].uuid);
        core.execute(p,PlayerBehavior::SelectReward{reward_id:stale.unwrap()}).unwrap();
        assert_eq!(view(&core),view(&core));
    }
    let before=roots(&core);assert!(core.execute(p,PlayerBehavior::InvestRelicReward).is_err());assert_eq!(roots(&core),before);
    let receipt=choose(&mut core,p);assert_eq!(receipt.paid_dust,18);assert_eq!(receipt.granted.len(),1);assert!(receipt.granted[0].definition_id.starts_with("special"));
    assert_eq!(core.state.run.as_ref().unwrap().map.node(node).unwrap().state,MapNodeState::Completed);
    let fresh=restore(&core,p);assert_eq!(fresh.state.relics,core.state.relics);assert_eq!(fresh.state.crafting_resources.equipment_dust,0);
}
#[test]
fn u162_investment_shortage_empty_upgrade_and_unfinished_rewind() {
    let (mut short,p)=setup(RewardActivityKind::ResourceInvestment,|_|{});give(&mut short,&[],2);enter(&mut short,p,"2");let before=roots(&short);
    assert_eq!(view(&short)["relic_activity"]["investment"]["eligibility"]["disabled_reason"],"insufficient_dust");assert!(short.execute(p,PlayerBehavior::InvestRelicReward).is_err());assert_eq!(roots(&short),before);
    let (mut core,p)=setup(RewardActivityKind::ResourceInvestment,|pol|pol.candidates.retain(|c|c.relic_id.starts_with("common")));give(&mut core,&[],18);enter(&mut core,p,"2");
    core.execute(p,PlayerBehavior::InvestRelicReward).unwrap();let first=view(&core);let before=roots(&core);
    assert_eq!(first["relic_activity"]["investment"]["eligibility"]["disabled_reason"],"no_candidates");assert!(core.execute(p,PlayerBehavior::InvestRelicReward).is_err());assert_eq!(roots(&core),before);
    let mut fresh=restore(&core,p);assert_eq!(fresh.state.crafting_resources.equipment_dust,18);enter(&mut fresh,p,"2");fresh.execute(p,PlayerBehavior::InvestRelicReward).unwrap();assert_eq!(view(&fresh),first);
    let receipt=choose(&mut core,p);assert_eq!(receipt.paid_dust,3);assert!(receipt.granted[0].definition_id.starts_with("common"));
}
#[test]
fn u162_fusion_six_probability_rows_input_order_and_atomic_result() {
    let rows=[("common_0","common_1",[80,19,1]),("common_0","rare_0",[40,55,5]),("common_0","special_0",[0,70,30]),("rare_0","rare_1",[0,80,20]),("rare_0","special_0",[0,40,60]),("special_0","special_1",[0,0,100])];
    for (a,b,expected) in rows {
        let (mut core,p)=setup(RewardActivityKind::RelicGacha,|_|{});let copies=give(&mut core,&[a,b,"special_3"],0);let node=enter(&mut core,p,"2");
        core.execute(p,PlayerBehavior::SelectRelicFusionInputs{copies:Some([copies[0],copies[1]])}).unwrap();let first=view(&core);
        assert_eq!(first["relic_activity"]["fusion"]["probabilities"],json!(expected));assert_eq!(first["relic_activity"]["fusion"]["eligibility"]["enabled"],true);
        core.execute(p,PlayerBehavior::SelectRelicFusionInputs{copies:Some([copies[1],copies[0]])}).unwrap();assert_eq!(view(&core),first);
        let mut replay=restore(&core,p);enter(&mut replay,p,"2");
        replay.execute(p,PlayerBehavior::SelectRelicFusionInputs{copies:Some([copies[1],copies[0]])}).unwrap();
        let replay_result=replay.execute(p,PlayerBehavior::ConfirmRelicFusion).unwrap();
        let result=core.execute(p,PlayerBehavior::ConfirmRelicFusion).unwrap();
        assert_eq!(serde_json::to_value(&result).unwrap(),serde_json::to_value(&replay_result).unwrap(),"fresh-process entry rewind and reversed input preserve exact result/copy identity");
        let BehaviorResult::RewardGranted{relic_activity:Some(receipt),..}=result else{panic!("receipt");};
        assert_eq!(receipt.granted.len(),1);assert_eq!(receipt.consumed_copies.len(),2);assert_eq!(core.state.relics.copies().count(),2);assert!(core.state.relics.get(copies[2]).is_some());
        let out=&receipt.granted[0];assert_ne!(out.definition_id,a);assert_ne!(out.definition_id,b);assert_eq!(out.variant,crate::game::relic::RelicVariant::Normal);
        let grade=RelicRewardGrade::from_rarity(&core.game_data.relic_data.get_by_id(&out.definition_id).unwrap().rarity).unwrap();assert!(expected[grade.index()]>0);
        assert_eq!(core.state.run.as_ref().unwrap().map.node(node).unwrap().state,MapNodeState::Completed);let after=roots(&core);
        assert!(core.execute(p,PlayerBehavior::ConfirmRelicFusion).is_err());assert_eq!(roots(&core),after);
        assert_eq!(restore(&core,p).state.relics,core.state.relics);
    }
}
#[test]
fn u162_fusion_missing_positive_grade_blocks_without_redistribution_or_loss() {
    let (mut core,p)=setup(RewardActivityKind::RelicGacha,|pol|pol.candidates.retain(|c|!c.relic_id.starts_with("special")));let ids=give(&mut core,&["common_0","common_1"],0);enter(&mut core,p,"2");
    core.execute(p,PlayerBehavior::SelectRelicFusionInputs{copies:Some([ids[0],ids[1]])}).unwrap();let v=view(&core);assert_eq!(v["relic_activity"]["fusion"]["probabilities"],json!([80,19,1]));assert_eq!(v["relic_activity"]["fusion"]["eligibility"]["disabled_reason"],"missing_outcome_grade");
    let before=roots(&core);assert!(core.execute(p,PlayerBehavior::ConfirmRelicFusion).is_err());assert_eq!(roots(&core),before);
    core.execute(p,PlayerBehavior::SelectRelicFusionInputs{copies:None}).unwrap();assert_eq!(view(&core)["relic_activity"]["fusion"]["probabilities"],Value::Null);
    core.execute(p,PlayerBehavior::ExitReward).unwrap();assert_eq!(core.state.relics.copies().count(),2);
}
#[test]
fn u162_fusion_input_eligibility_corrupt_normal_and_owned_kind_allowed() {
    // Only special_2 remains after excluding the two input kinds. It is already
    // owned, so rejecting duplicates would make this otherwise valid row fail.
    let (mut core,p)=setup(RewardActivityKind::RelicGacha,|pol|pol.candidates.retain(|c|["special_0","special_1","special_2"].contains(&c.relic_id.as_str())));
    let ids=give(&mut core,&["special_0","special_1","special_2"],0);
    core.apply_grant_effects(&[RewardEffect::GrantRelicCorruptionCatalyst{amount:1}]).unwrap();
    core.execute(p,PlayerBehavior::CorruptRelic{copy_uuid:ids[0]}).unwrap();
    let owner=core.state.roster.iter().next().unwrap().uuid;
    core.execute(p,PlayerBehavior::EquipRelic{copy_uuid:ids[1],employee_uuid:owner,slot:0}).unwrap();
    enter(&mut core,p,"2");let before=roots(&core);
    for copies in [[ids[0],ids[0]],[ids[0],Uuid::nil()],[ids[0],ids[1]]] {assert!(core.execute(p,PlayerBehavior::SelectRelicFusionInputs{copies:Some(copies)}).is_err());assert_eq!(roots(&core),before);}
    // Equipped input stays unavailable during the node; use a separate legal
    // pre-entry loadout for the success counterpart.
    let (mut other,p)=setup(RewardActivityKind::RelicGacha,|pol|pol.candidates.retain(|c|["special_0","special_1","special_2"].contains(&c.relic_id.as_str())));
    let ids=give(&mut other,&["special_0","special_1","special_2"],0);other.apply_grant_effects(&[RewardEffect::GrantRelicCorruptionCatalyst{amount:1}]).unwrap();other.execute(p,PlayerBehavior::CorruptRelic{copy_uuid:ids[0]}).unwrap();enter(&mut other,p,"2");
    other.execute(p,PlayerBehavior::SelectRelicFusionInputs{copies:Some([ids[0],ids[1]])}).unwrap();let result=other.execute(p,PlayerBehavior::ConfirmRelicFusion).unwrap();
    let BehaviorResult::RewardGranted{relic_activity:Some(receipt),..}=result else{panic!()};assert_eq!(receipt.granted[0].definition_id,"special_2");assert_eq!(receipt.granted[0].variant,crate::game::relic::RelicVariant::Normal);assert_eq!(other.state.relics.copies().filter(|c|c.definition_id=="special_2").count(),2);
}

#[test]
fn u162_exit_before_use_and_after_investment_preserves_exact_paid_cost() {
    for kind in [RewardActivityKind::SimpleChoice,RewardActivityKind::ResourceInvestment,RewardActivityKind::RelicGacha] {
        let (mut core,p)=setup(kind,|_|{});give(&mut core,&["common_0","common_0"],18);enter(&mut core,p,"2");
        if kind==RewardActivityKind::ResourceInvestment {core.execute(p,PlayerBehavior::InvestRelicReward).unwrap();}
        core.execute(p,PlayerBehavior::ExitReward).unwrap();
        let expected=if kind==RewardActivityKind::ResourceInvestment {15} else {18};
        assert_eq!(core.state.crafting_resources.equipment_dust,expected);assert_eq!(core.state.relics.copies().count(),2);
        let fresh=restore(&core,p);assert_eq!(fresh.state.relics,core.state.relics);assert_eq!(fresh.state.crafting_resources.equipment_dust,expected);
    }
}
#[test]
fn u162_two_distinct_copies_of_same_kind_are_valid_inputs() {
    let (mut core,p)=setup(RewardActivityKind::RelicGacha,|_|{});let copies=give(&mut core,&["common_0","common_0"],0);assert_ne!(copies[0],copies[1]);enter(&mut core,p,"2");
    core.execute(p,PlayerBehavior::SelectRelicFusionInputs{copies:Some([copies[0],copies[1]])}).unwrap();
    assert_eq!(view(&core)["relic_activity"]["fusion"]["probabilities"],json!([80,19,1]));
    core.execute(p,PlayerBehavior::ConfirmRelicFusion).unwrap();assert_eq!(core.state.relics.copies().count(),1);assert_ne!(core.state.relics.copies().next().unwrap().definition_id,"common_0");
}
