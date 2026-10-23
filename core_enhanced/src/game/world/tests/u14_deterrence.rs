//! U14 normal authored route + public command oracles. Fixture mutations of trauma
//! and balance establish boundary preconditions; all entry/settlement uses commands.
use super::*;
use crate::game::employee::BreakdownCounter;
use crate::game::data::ValidatedGameData;
use crate::game::map::*;
use crate::game::expedition_lifecycle::ExpeditionContinuationChoice;

fn cost_data(cost:u32)->ValidatedGameData {
    super::u13_facility::development_data_with(|db| {
        let layout=&mut db.layouts[0];
        layout.activities.iter_mut().find(|a|a.id=="save").unwrap().first_entry_cost=Some(cost);
    })
}
fn start(data:ValidatedGameData)->(GameCore,Uuid) {super::u13_facility::start(data,14001)}
fn target(core:&GameCore)->MapNodeId {core.state.run.as_ref().unwrap().map.nodes.iter().find(|n|n.slot_id.as_str()=="2").unwrap().id}
fn restore(core:&GameCore)->GameCore {
    let saved=core.export_active_run_checkpoint().unwrap().unwrap();
    let mut fresh=GameCore::new(core.game_data.clone(),9999);
    fresh.install_authoritative_player_profile(core.state.player_profile.clone()).unwrap();
    fresh.continue_from_active_run_checkpoint(core.state.player.as_ref().unwrap().id,&saved.core_payload_json,saved.loads_used,saved.run_peak_certified_power).unwrap();
    fresh
}
#[test]
fn u14_authored_entry_preview_cost_short_balance_zero_and_continue() {
    for (cost,balance,grace) in [(2,1,4),(2,0,4),(0,0,4),(2,0,2)] {
        let (mut core,player)=start(cost_data(cost));
        let employee=core.state.roster.iter().next().unwrap().uuid;
        let e=core.state.roster.get_mut(&employee).unwrap();
        e.condition.trauma=100;e.breakdown=BreakdownCounter::Active{remaining_cost:grace,entered_node:None};
        core.state.run.as_mut().unwrap().run_progression.deterrence=balance;
        let id=target(&core);
        let before=serde_json::to_value(&core.state.roster).unwrap();
        let preview=core.current_map_view().unwrap().nodes.into_iter().find(|n|n.id==id).unwrap().entry_preview.unwrap();
        assert_eq!((preview.regulated_cost,preview.charged_cost,preview.deterrence_after),(cost,cost,balance.saturating_sub(cost)));
        assert_eq!(preview.breakdown.len(),1);
        assert_eq!(preview.breakdown[0].remaining_cost_after,grace.saturating_sub(cost));
        assert_eq!(preview.breakdown[0].dies_on_entry,cost>0 && cost>=grace);
        assert_eq!(serde_json::to_value(&core.state.roster).unwrap(),before,"preview is read-only");
        core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();
        core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
        assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence,balance.saturating_sub(cost));
        assert!(core.state.run.as_ref().unwrap().run_progression.entry_receipts.contains(&id));
        assert_eq!(core.state.roster.get(&employee).unwrap().condition.is_dead(),cost>0 && cost>=grace);
        if cost<grace {assert_eq!(core.state.roster.get(&employee).unwrap().breakdown.remaining_cost(),Some(grace-cost));}
        let pending=core.state.run.as_ref().unwrap().run_progression.pending_continuation.is_some();
        assert_eq!(pending,balance>0 && balance<=cost);
        // Fresh core Continue restores before entry, including the lost employee and receipt.
        let fresh=restore(&core);
        assert_eq!(fresh.state.run.as_ref().unwrap().run_progression.deterrence,balance);
        assert!(!fresh.state.run.as_ref().unwrap().run_progression.entry_receipts.contains(&id));
        assert_eq!(serde_json::to_value(&fresh.state.roster).unwrap(),before);
        if pending {core.execute(player,PlayerBehavior::ChooseExpeditionContinuation{choice:ExpeditionContinuationChoice::Continue}).unwrap();}
        core.execute(player,PlayerBehavior::CompleteNode).unwrap();
        let e=core.state.roster.get(&employee).unwrap();
        assert_eq!(e.condition.is_dead(),cost>0 && cost>=grace,"SavePoint cannot revive entry deaths");
        if e.is_alive() {assert!(e.condition.trauma<100);assert_eq!(e.breakdown,BreakdownCounter::Inactive);}
        let completed=restore(&core);
        assert_eq!(completed.state.run.as_ref().unwrap().run_progression.deterrence,balance.saturating_sub(cost));
        assert!(completed.state.run.as_ref().unwrap().run_progression.entry_receipts.contains(&id));
    }
}
#[test]
fn u14_official_bough_plan_discloses_only_markers_and_is_stable_across_continue() {
    let data=ValidatedGameData::load_live_embedded().unwrap();
    for seed in [14,15,16,17] {
        let (core,_)=super::u13_facility::start(data.clone(),seed);
        let view=core.facility_view().unwrap().unwrap();
        assert_eq!(view.golden_boughs.len(),3);
        for marker in &view.golden_boughs {
            assert!(!marker.collected);assert!(marker.reachable);
            let room=view.rooms.iter().find(|r|r.id==marker.room_id).unwrap();
            assert!(room.name.is_none() && room.concept.is_none() && room.known_reservations.is_empty());
        }
        let before=core.state.run.as_ref().unwrap().facility_run_state.rooms.as_ref().unwrap().golden_boughs.clone();
        let restored=restore(&core);
        assert_eq!(restored.state.run.as_ref().unwrap().facility_run_state.rooms.as_ref().unwrap().golden_boughs,before);
    }
}
#[test]
fn u14_snapshot_uses_after_cost_and_cached_preview_refreshes_without_compounding() {
    let mut core=GameCore::new(game_data_with_pve_encounters(),14003);let player=Uuid::from_u128(14003);
    start_new_game_with_default_starters(&mut core,player);
    let id=force_first_available_node(&mut core,MapNodeCategory::Combat,"combat_defense",MapNodePayload::Encounter{encounter_id:Some("defense_encounter".into())});
    core.state.run.as_mut().unwrap().run_progression.deterrence=2;
    core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();
    let restrained=core.combat_preview_for_node(id).unwrap().unwrap();
    assert!(!restrained.deterrence.unrestrained);
    core.state.run.as_mut().unwrap().run_progression.deterrence=1;
    let unrestrained=core.combat_preview_for_node(id).unwrap().unwrap();
    assert!(unrestrained.deterrence.unrestrained);
    assert_eq!(unrestrained.deterrence.enemy_health_percent,150);
    for _ in 0..2 {assert_eq!(core.combat_preview_for_node(id).unwrap().unwrap().deterrence,unrestrained.deterrence);}
    core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
    assert!(core.state.active_battle.is_none());
    core.execute(player,PlayerBehavior::ChooseExpeditionContinuation{choice:ExpeditionContinuationChoice::Continue}).unwrap();
    let active=core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.combat_preview.deterrence,unrestrained.deterrence);
    assert_eq!(active.combat_preview.enemy_stat_scale.encounter_health_percent,150);
}

fn sealed_data()->ValidatedGameData { sealed_data_with_defeat(false) }
fn sealed_data_with_defeat(timed_defeat:bool)->ValidatedGameData {
    use crate::game::data::{live_ron::GameplayRonSourceSet,pve_data::*};
    let mut sources=GameplayRonSourceSet::embedded();
    let mut catalog=FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
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
    activity.golden_bough_candidate=true;activity.support=FacilityContentSupport::Supported;
    activity.payload=MapNodePayload::Encounter{encounter_id:Some("suppress_nothing_there".into())};
    for size in &mut layout.allocation.sizes {for weight in size.elite_percent_by_depth.values_mut(){*weight=0;}}
    let location=|slot:&str|GoldenBoughLocation{room_id:"B1-01".into(),slot_id:MapSlotId::new(slot)};
    layout.golden_bough_plans=vec![GoldenBoughPlan{id:"front".into(),weight:1,locations:vec![location("2")],deterrence_refill:8},
        GoldenBoughPlan{id:"front_and_side".into(),weight:1,locations:vec![location("2"),location("10")],deterrence_refill:8}];
    let catalog_text=ron::ser::to_string(&catalog).unwrap();sources.facility_templates=&catalog_text;
    let mut pve:PveEncounterDatabase=ron::de::from_str(sources.pve_encounters).unwrap();
    let encounter=pve.encounters.iter_mut().find(|e|e.id=="suppress_nothing_there").unwrap();
    encounter.deterrence_tuning.seal_health_percent=180;encounter.deterrence_tuning.seal_attack_percent=170;
    encounter.waves.retain(|wave|wave.id=="primary_abnormality");
    encounter.waves[0].activation=PveWaveActivationData::at_time("primary_abnormality",0);
    encounter.waves[0].required_for_victory=true;
    if timed_defeat {
        encounter.terminal_outcomes.iter_mut().find(|outcome|outcome.verdict==crate::game::battle::types::BattleVerdict::Defeat).unwrap().condition=PveTerminalConditionData::TimeReached{time_ms:500};
    }

    encounter.bonus_objectives=vec![
        PveBonusObjective{id:"within_limit".into(),condition:PveBonusCondition::ClearWithin{time_ms:60_000},effects:vec![RewardEffect::GrantExperience{amount:7,target:ExperienceTargetPolicy::AliveRoster},RewardEffect::GrantEquipmentDust{amount:3}]},
        PveBonusObjective{id:"too_late".into(),condition:PveBonusCondition::ClearWithin{time_ms:1},effects:vec![RewardEffect::GrantExperience{amount:999,target:ExperienceTargetPolicy::AliveRoster}]},
        PveBonusObjective{id:"decisive".into(),condition:PveBonusCondition::DecisiveDamage{max_health_percent:25},effects:vec![RewardEffect::GrantEquipmentDust{amount:5}]},
    ];
    let pve_text=ron::ser::to_string(&pve).unwrap();sources.pve_encounters=&pve_text;
    let mut abnormalities:crate::game::data::abnormality_data::AbnormalityDatabase=ron::de::from_str(sources.abnormalities).unwrap();
    let boss=abnormalities.items.iter_mut().find(|a|a.id=="o-06-20_nothing_there").unwrap();
    boss.max_health=10;boss.attack=0;boss.defense=0;boss.magic_resist=0;
    let abno_text=ron::ser::to_string(&abnormalities).unwrap();sources.abnormalities=&abno_text;
    // This authored contact fixture exposes walkable arena floor as tactical
    // slots; official arena deployment restrictions remain unchanged.
    let templates:Vec<crate::game::combat_preview::BattlefieldTemplateDefinition>=ron::de::from_str(sources.battlefield_templates).unwrap();
    let mut templates=serde_json::to_value(templates).unwrap();
    for template in templates.as_array_mut().unwrap() {
        if template["archetype"]=="BossArena" {
            for row in template["rows"].as_array_mut().unwrap() {*row=serde_json::json!(row.as_str().unwrap().replace('.',"P"));}
        }
    }
    let templates:Vec<crate::game::combat_preview::BattlefieldTemplateDefinition>=serde_json::from_value(templates).unwrap();
    let template_text=ron::ser::to_string(&templates).unwrap();sources.battlefield_templates=&template_text;
    GameDataBase::load_live_sources(sources).expect("U14 authored normal-boss fixture")
}
fn sealed_start(balance:u32)->(GameCore,Uuid,MapNodeId) {
    sealed_start_with_data(sealed_data(),balance)
}
fn sealed_start_with_data(data:ValidatedGameData,balance:u32)->(GameCore,Uuid,MapNodeId) {
    let (mut core,player)=start(data);
    let save=core.state.run.as_ref().unwrap().map.nodes.iter().find(|n|n.slot_id.as_str()=="1").unwrap().id;
    core.execute(player,PlayerBehavior::SelectMapNode{node_id:save}).unwrap();
    core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    core.state.run.as_mut().unwrap().run_progression.deterrence=balance;
    let id=target(&core);
    assert!(core.sealed_golden_bough(id).is_some());
    core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();
    core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
    if core.state.run.as_ref().unwrap().run_progression.pending_continuation.is_some() {
        core.execute(player,PlayerBehavior::ChooseExpeditionContinuation{choice:ExpeditionContinuationChoice::Continue}).unwrap();
    }
    (core,player,id)
}
#[test]
fn u14_seal_retreat_continue_then_actual_victory_refill_and_bonus_once() {
    for balance in [1,5,12] {
        let (mut core,player,id)=sealed_start(balance);
        let expected_unrestrained=balance==1;
        let snapshot=core.state.active_battle.as_ref().unwrap().combat_preview.deterrence;
        assert_eq!(snapshot.unrestrained,expected_unrestrained);assert!(snapshot.seal_released);
        assert_eq!(snapshot.enemy_health_percent,if expected_unrestrained{150}else{180});
        assert_eq!(snapshot.enemy_attack_percent,if expected_unrestrained{150}else{170});
        assert!(core.facility_view().unwrap().unwrap().golden_boughs.iter().all(|g|!g.collected));
        let preentry=restore(&core);
        assert_eq!(preentry.state.run.as_ref().unwrap().run_progression.deterrence,balance);
        assert!(!preentry.state.run.as_ref().unwrap().run_progression.entry_receipts.contains(&id));
        let before_retreat=serde_json::to_value((&core.state.roster,&core.state.crafting_resources)).unwrap();
        execute_current_battle_mutation(&mut core,player,PlayerBehavior::RetreatBattle).unwrap();
        assert_eq!(serde_json::to_value((&core.state.roster,&core.state.crafting_resources)).unwrap(),before_retreat,"retreat gives no XP or bonus");
        assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence,balance-1);
        assert!(core.sealed_golden_bough(id).is_some());
        let attempts=core.state.run.as_ref().unwrap().abnormality_attempt_state(id).remaining_attempts();
        assert_eq!(restore(&core).state.run.as_ref().unwrap().abnormality_attempt_state(id).remaining_attempts(),attempts);
        core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();
        core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
        assert_eq!(core.state.active_battle.as_ref().unwrap().combat_preview.deterrence,snapshot);
        let ids=core.state.roster.combat_eligible_employee_ids();
        let employee=*ids.iter().find(|id|core.state.roster.get(id).unwrap().role!=crate::game::employee::EmployeeRole::Healer).unwrap();
        let position=core.state.active_battle.as_ref().unwrap().combat_preview.deployment_zones.iter()
            .find(|z|z.kind==crate::game::combat_preview::DeploymentZoneKind::Ground).unwrap().cells[0];
        execute_current_battle_mutation(&mut core,player,PlayerBehavior::DeployUnit{employee_uuid:employee,position,facing:FacingDirection::Right}).unwrap();
        // Employees hold their tile; use the supported tactical placement preview
        // to put this melee attacker into contact, rather than wait for pursuit.
        core.advance_active_battle_by(5_000).unwrap();
        let BehaviorResult::BattleState{battle_update,..}=core.execute(player,PlayerBehavior::RequestBattleState{since_seq:None}).unwrap() else {panic!("battle state");};
        let enemy=battle_update.checkpoint.units.iter().find(|u|u.owner==crate::game::enums::Side::Opponent).unwrap().position;
        let mut relocated=false;
        for (dx,dy,facing) in [(0,1,FacingDirection::Up),(-1,0,FacingDirection::Right),(0,-1,FacingDirection::Down),(1,0,FacingDirection::Left)] {
            let destination=Position{x:enemy.x+dx,y:enemy.y+dy};
            let preview=core.execute(player,PlayerBehavior::PreviewTacticalCommand{command:crate::game::behavior::TacticalCommandPreviewInput::Relocate{employee_uuid:employee,destination,facing}});
            if let Ok(BehaviorResult::TacticalCommandPreview{result})=preview {
                if result.eligibility.enabled && result.units[0].range_previews.basic_attack.cells.contains(&enemy) {
                    execute_current_battle_mutation(&mut core,player,PlayerBehavior::TacticalRelocateUnit{employee_uuid:employee,destination,facing}).unwrap();
                    relocated=true;break;
                }
            }
        }
        assert!(relocated,"public tactical preview must prove attack contact");
        let dust=core.state.crafting_resources.equipment_dust;
        let before=core.state.roster.iter().map(|e|(e.uuid,e.experience)).collect::<std::collections::BTreeMap<_,_>>();
        for _ in 0..120 {
            core.advance_active_battle_by(500).unwrap();
            if matches!(core.get_state(),GameState::CombatResult{..}) {break;}
        }
        if !matches!(core.get_state(),GameState::CombatResult{..}) {
            eprintln!("U14 unresolved battle: {:?}",core.execute(player,PlayerBehavior::RequestBattleState{since_seq:None}).unwrap());
        }
        assert!(matches!(core.get_state(),GameState::CombatResult{..}),"ordinary attack must defeat the authored weak boss within 60 seconds");
        let multiplier=if expected_unrestrained{2}else{1};
        for e in core.state.roster.iter() {
            assert_eq!(e.experience-before[&e.uuid],(if e.uuid==employee{17}else{7})*multiplier);
        }
        assert_eq!(core.state.crafting_resources.equipment_dust-dust,(3+4+3+5)*multiplier,"base dust3 + encounter salvage4 + time bonus3 + decisive bonus5 all share the canonical owner and grant once");
        assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence,balance-1+8);
        assert!(core.sealed_golden_bough(id).is_none());
        let completed_preview=core.current_map_view().unwrap().nodes.into_iter()
            .find(|node|node.id==id).unwrap().entry_preview.unwrap();
        assert!(!completed_preview.sealed_golden_bough);
        assert!(!completed_preview.combat_deterrence.unwrap().seal_released);
        let collected=core.facility_view().unwrap().unwrap().golden_boughs.iter().filter(|g|g.collected).count();assert_eq!(collected,1);
        let settled=serde_json::to_value((&core.state.roster,&core.state.crafting_resources)).unwrap();
        let fresh=restore(&core);assert_eq!(serde_json::to_value((&fresh.state.roster,&fresh.state.crafting_resources)).unwrap(),settled);
        assert_eq!(fresh.state.run.as_ref().unwrap().run_progression.deterrence,balance-1+8);
        core.execute(player,PlayerBehavior::CompleteCombatResult).unwrap();
        assert_eq!(serde_json::to_value((&core.state.roster,&core.state.crafting_resources)).unwrap(),settled);
        assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence,balance-1+8);
    }
}

#[test]
fn u14_actual_time_boundary_and_defeat_survival_xp() {
    use crate::game::data::pve_data::*;
    for (limit,defeat) in [(499,false),(500,false),(501,false),(1000,true)] {
        let mut data=game_data_with_pve_encounters();
        let db=Arc::get_mut(&mut data).unwrap();
        let pve=Arc::make_mut(&mut db.pve_data);
        let encounter=pve.encounters.iter_mut().find(|e|e.id=="defense_encounter").unwrap();
        encounter.mission_variant=Some(crate::game::combat_preview::CombatMissionVariant::Defense);
        encounter.terminal_outcomes=vec![PveTerminalOutcomeData{id:"u14_timed".into(),verdict:if defeat{crate::game::battle::types::BattleVerdict::Defeat}else{crate::game::battle::types::BattleVerdict::Victory},condition:PveTerminalConditionData::TimeReached{time_ms:500},priority:0}];
        encounter.outcome_policies=vec![PveOutcomePolicyData{outcome_id:"u14_timed".into(),reward_mode:RewardMode::ClaimAll,reward_uuids:vec![],grant_node_bonus:false,advance_encounter_progression:false}];
        if defeat {
            encounter.terminal_outcomes.push(PveTerminalOutcomeData{id:"later_victory".into(),verdict:crate::game::battle::types::BattleVerdict::Victory,condition:PveTerminalConditionData::TimeReached{time_ms:1000},priority:0});
            encounter.outcome_policies.push(PveOutcomePolicyData{outcome_id:"later_victory".into(),reward_mode:RewardMode::ClaimAll,reward_uuids:vec![],grant_node_bonus:false,advance_encounter_progression:false});
        }
        encounter.waves[0].required_for_victory=false;
        encounter.waves[0].activation=PveWaveActivationData::at_time("wave_0",0);
        encounter.bonus_objectives=vec![PveBonusObjective{id:"time".into(),condition:PveBonusCondition::ClearWithin{time_ms:limit},effects:vec![RewardEffect::GrantEquipmentDust{amount:3}]}];
        let mut core=GameCore::new(data,14010);let player=Uuid::from_u128(14010);start_new_game_with_default_starters(&mut core,player);
        let id=force_first_available_node(&mut core,MapNodeCategory::Combat,"combat_defense",MapNodePayload::Encounter{encounter_id:Some("defense_encounter".into())});
        core.state.run.as_mut().unwrap().run_progression.deterrence=1;
        core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
        core.execute(player,PlayerBehavior::ChooseExpeditionContinuation{choice:ExpeditionContinuationChoice::Continue}).unwrap();
        let employee=core.state.roster.combat_eligible_employee_ids()[0];
        let position=core.state.active_battle.as_ref().unwrap().combat_preview.deployment_zones.iter().find(|z|z.kind==crate::game::combat_preview::DeploymentZoneKind::Ground).unwrap().cells[0];
        execute_current_battle_mutation(&mut core,player,PlayerBehavior::DeployUnit{employee_uuid:employee,position,facing:FacingDirection::Right}).unwrap();
        let dust=core.state.crafting_resources.equipment_dust;
        let before=core.state.roster.get(&employee).unwrap().experience;
        core.advance_active_battle_by(499).unwrap();assert!(matches!(core.get_state(),GameState::InBattle{..}));
        core.advance_active_battle_by(1).unwrap();assert!(matches!(core.get_state(),GameState::CombatResult{..}));
        assert_eq!(core.state.crafting_resources.equipment_dust-dust,if limit>=500 && !defeat{6}else{0});
        assert_eq!(core.state.roster.get(&employee).unwrap().experience-before,if defeat{10}else{20});
    }
}

#[test]
fn u14_authored_invalid_tuning_objective_and_bough_plans_reject() {
    use crate::game::data::{live_ron::GameplayRonSourceSet,pve_data::*};
    for case in 0..7 {
        let mut sources=GameplayRonSourceSet::embedded();
        let mut pve:PveEncounterDatabase=ron::de::from_str(sources.pve_encounters).unwrap();
        let encounter=pve.encounters.iter_mut().find(|e|e.id=="suppress_nameless_fetus").unwrap();
        let mut catalog=FacilityTemplateDatabase::from_ron_str(sources.facility_templates).unwrap();
        match case {
            0=>encounter.deterrence_tuning.unrestrained_health_percent=149,
            1=>encounter.deterrence_tuning.unrestrained_attack_percent=201,
            2=>encounter.deterrence_tuning.seal_attack_percent=100,
            3=>encounter.bonus_objectives[0].condition=PveBonusCondition::DecisiveDamage{max_health_percent:101},
            4=>catalog.layouts[0].golden_bough_plans.clear(),
            5=>catalog.layouts[0].golden_bough_plans[0].locations[0].slot_id=MapSlotId::new("missing"),
            6=>catalog.layouts[0].connections.retain(|c|c.to!="B4-02" && c.from!="B4-02"),
            _=>unreachable!(),
        }
        let pve_text=ron::ser::to_string(&pve).unwrap();sources.pve_encounters=&pve_text;
        let catalog_text=ron::ser::to_string(&catalog).unwrap();sources.facility_templates=&catalog_text;
        assert!(GameDataBase::load_live_sources(sources).is_err(),"invalid authored case {case}");
    }
}

#[test]
fn u14_actual_late_spawns_scale_only_hp_attack_and_ignore_roster_power() {
    for (encounter,category) in [("normal_corroded_encounter",MapNodeCategory::Combat),("elite_risk_encounter",MapNodeCategory::Combat),("boss_risk_encounter",MapNodeCategory::Boss)] {
        let mut observed=Vec::new();
        for (balance,stronger) in [(5,false),(5,true),(1,false)] {
            let mut core=GameCore::new(game_data_with_pve_encounters(),14020);let player=Uuid::from_u128(14020);start_new_game_with_default_starters(&mut core,player);
            if stronger {for employee in core.state.roster.iter_mut(){employee.combat_profile.battle_profile.stats.attack+=900;}core.state.run_power_authority.run_peak_certified_power=999_999;}
            core.state.run.as_mut().unwrap().run_progression.deterrence=balance;
            let id=force_first_available_node(&mut core,category,"u14_spawn",MapNodePayload::Encounter{encounter_id:Some(encounter.into())});
            core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
            if balance==1 {core.execute(player,PlayerBehavior::ChooseExpeditionContinuation{choice:ExpeditionContinuationChoice::Continue}).unwrap();}
            core.advance_active_battle_by(6000).unwrap();
            let BehaviorResult::BattleState{battle_update,..}=core.execute(player,PlayerBehavior::RequestBattleState{since_seq:None}).unwrap() else{panic!("live spawn checkpoint");};
            let enemies:Vec<_>=battle_update.checkpoint.units.iter().filter(|u|u.owner==crate::game::enums::Side::Opponent).map(|u|serde_json::json!({"source":u.unit_source,"stats":u.stats,"mobility":u.mobility_kind})).collect();
            assert!(!enemies.is_empty(),"real delayed enemy spawn required");observed.push(enemies);
        }
        assert_eq!(observed[0],observed[1],"player stats and certified peak do not rescale enemies");
        assert_eq!(observed[0].len(),observed[2].len());
        for (base,boosted) in observed[0].iter().zip(&observed[2]) {
            let mut expected=base.clone();
            for key in ["max_health","current_health","attack"] {let n=base["stats"][key].as_u64().unwrap();expected["stats"][key]=serde_json::json!((n*150+99)/100);}
            assert_eq!(&expected,boosted,"HP/ATK only; same composition and non-target stats");
        }
    }
}

#[test]
fn u14_sealed_actual_defeat_never_collects_bough_or_bonus_and_xp_stays_ten() {
    let (mut core,player,id)=sealed_start_with_data(sealed_data_with_defeat(true),1);
    let employee=core.state.roster.combat_eligible_employee_ids()[0];
    let position=core.state.active_battle.as_ref().unwrap().combat_preview.deployment_zones.iter().find(|z|z.kind==crate::game::combat_preview::DeploymentZoneKind::Ground).unwrap().cells[0];
    execute_current_battle_mutation(&mut core,player,PlayerBehavior::DeployUnit{employee_uuid:employee,position,facing:FacingDirection::Right}).unwrap();
    let xp=core.state.roster.get(&employee).unwrap().experience;
    let dust=core.state.crafting_resources.equipment_dust;
    core.advance_active_battle_by(500).unwrap();
    assert!(matches!(core.get_state(),GameState::CombatResult{..}));
    assert_eq!(core.state.roster.get(&employee).unwrap().experience-xp,10);
    assert_eq!(core.state.crafting_resources.equipment_dust,dust);
    assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence,0);
    assert!(core.sealed_golden_bough(id).is_some());
    assert!(core.facility_view().unwrap().unwrap().golden_boughs.iter().all(|g|!g.collected));
}

#[test]
fn u14_public_breakdown_order_and_abandoned_branch_reachability() {
    let (mut core,_)=start(cost_data(2));
    for employee in core.state.roster.iter_mut() {employee.condition.trauma=100;employee.reconcile_breakdown(4,None);}
    let id=target(&core);
    let view=core.current_map_view().unwrap();
    let preview=view.nodes.iter().find(|n|n.id==id).unwrap().entry_preview.as_ref().unwrap();
    assert_eq!(preview.breakdown.len(),3);
    assert!(preview.breakdown.windows(2).all(|pair|pair[0].employee_uuid<pair[1].employee_uuid));
    assert!(preview.breakdown.iter().all(|e|e.remaining_cost_after==2 && !e.dies_on_entry));
    let data=ValidatedGameData::load_live_embedded().unwrap();
    let (core,_)=super::u13_facility::start(data,14030);
    let layout=core.facility_layout().unwrap();
    let reached=layout.reachable_bough_rooms("B8-01");
    assert!(!reached.contains("A5-02") && !reached.contains("B4-02") && !reached.contains("C4-02") && !reached.contains("B7-02"),"forward descent never makes abandoned earlier boughs reachable");
}

#[test]
fn u14_visited_room_preview_preserves_seal_scale_identity_and_continue() {
    let data=super::u13_facility::development_data_with(|db| {
        let layout=&mut db.layouts[0];
        let boss=layout.activities.iter_mut().find(|a|a.id=="reserved_B4-02").unwrap();
        boss.golden_bough_candidate=true;
        boss.support=FacilityContentSupport::Supported;
        boss.payload=MapNodePayload::Encounter{encounter_id:Some("suppress_nothing_there".into())};
        for room in layout.rooms.iter_mut().filter(|r|r.depth==1) {
            for slot in ["1","3"] {
                room.reservations.push(FacilityReservation{slot_id:MapSlotId::new(slot),activity_id:"reserved_B4-02".into()});
            }
        }
        let location=|slot:&str|GoldenBoughLocation{room_id:"B1-01".into(),slot_id:MapSlotId::new(slot)};
        layout.golden_bough_plans=vec![
            GoldenBoughPlan{id:"one".into(),weight:1,locations:vec![location("1")],deterrence_refill:8},
            GoldenBoughPlan{id:"two".into(),weight:1,locations:vec![location("1"),location("3")],deterrence_refill:8},
        ];
    });
    fn preview(core:&GameCore,room_id:&str)->(MapNodeId,NodeEntryPreviewDto) {
        let view=core.facility_view().unwrap().unwrap();
        let room=view.rooms.iter().find(|r|r.id==room_id).unwrap();
        let node=room.current_expedition_map.as_ref().unwrap().nodes.iter().find(|n|n.slot_id.as_str()=="1").unwrap();
        let entry=node.entry_preview.clone().unwrap();
        assert_eq!(entry,room.activities.iter().find(|a|a.node_id==node.id).unwrap().entry_preview);
        (node.id,entry)
    }
    fn gate(core:&mut GameCore,player:Uuid) {
        for (slot,finish) in [("2",Some(PlayerBehavior::CompleteNode)),("5",Some(PlayerBehavior::CompleteNode)),("8",Some(PlayerBehavior::ExitShop)),("G",None)] {
            let id=core.state.run.as_ref().unwrap().map.nodes.iter().find(|n|n.slot_id.as_str()==slot).unwrap().id;
            core.execute(player,PlayerBehavior::SelectMapNode{node_id:id}).unwrap();
            core.execute(player,PlayerBehavior::ConfirmEnterNode).unwrap();
            if core.state.run.as_ref().unwrap().run_progression.pending_continuation.is_some() {
                core.execute(player,PlayerBehavior::ChooseExpeditionContinuation{choice:ExpeditionContinuationChoice::Continue}).unwrap();
            }
            if let Some(finish)=finish {core.execute(player,finish).unwrap();}
        }
        assert!(core.facility_gate_reached());
    }
    for balance in [12,3] {
        let (mut core,player)=start(data.clone());
        // Boundary precondition only; movement, costs and Continue use public commands.
        core.state.run.as_mut().unwrap().run_progression.deterrence=balance;
        let (id,initial)=preview(&core,"B1-01");
        assert!(initial.sealed_golden_bough);
        let hidden=core.current_map_view().unwrap().nodes.into_iter().find(|n|n.id==id).unwrap();
        assert_eq!(hidden.payload,MapNodePayload::Encounter{encounter_id:None});
        gate(&mut core,player);
        core.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B1-02".into()}).unwrap();
        let (away_id,away)=preview(&core,"B1-01");
        assert_eq!(away_id,id);
        assert!(away.sealed_golden_bough);
        let scale=away.combat_deterrence.unwrap();
        assert!(scale.seal_released);
        assert_eq!((scale.enemy_health_percent,scale.enemy_attack_percent),(150,150));
        assert_eq!(scale.unrestrained,balance==3);
        assert_eq!(scale.victory_quantity_multiplier,if balance==3{2}else{1});
        assert!(!preview(&core,"B1-02").1.sealed_golden_bough,"same slot in another room is not the selected bough location");
        assert_eq!(preview(&restore(&core),"B1-01"),(id,away));
        let before=serde_json::to_value(core.get_run_snapshot_dto().unwrap()).unwrap();
        core.execute(player,PlayerBehavior::RequestMapData).unwrap();
        assert_eq!(serde_json::to_value(core.get_run_snapshot_dto().unwrap()).unwrap(),before);
        gate(&mut core,player);
        core.execute(player,PlayerBehavior::MoveToFacilityRoom{room_id:"B1-01".into()}).unwrap();
        let (returned_id,returned)=preview(&core,"B1-01");
        assert_eq!(returned_id,id);
        assert_eq!(returned.combat_deterrence,Some(scale));
        assert!(returned.sealed_golden_bough);
    }
}
