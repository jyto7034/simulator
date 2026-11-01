//! U15 owner oracles. Normal external pack/host evidence is recorded separately.
use super::*;
use crate::game::data::equipment_data::{EquipmentMaterialCost, EquipmentMaterialType};
use crate::game::data::skill_fragment_data::{SkillFragmentForm, SkillFragmentRecipe};
use crate::game::material_economy::{MaterialPayment, MaterialRecovery, MaterialSources};
use crate::game::reward::RewardEffect;
use std::collections::BTreeMap;

fn fixture() -> (GameCore, Uuid, Uuid, Uuid) { fixture_with_costs(7,5) }
fn fixture_with_costs(enkephalin:u32, part_a:u32) -> (GameCore, Uuid, Uuid, Uuid) { fixture_with_options(enkephalin, part_a, false) }
fn fixture_with_options(enkephalin:u32, part_a:u32, evolution: bool) -> (GameCore, Uuid, Uuid, Uuid) {
    fixture_with_weapon_stages(enkephalin, part_a, evolution, false)
}
fn fixture_with_weapon_stages(enkephalin:u32, part_a:u32, evolution: bool, staged_shots: bool) -> (GameCore, Uuid, Uuid, Uuid) {
    let mut fragment = active_skill_fragment("u15_fragment", 1501, "u15_skill");
    let skills = game_data_with_skill_fragments(vec![fragment.clone()]).skill_data.clone();
    if let SkillFragmentEffectDef::ActiveSkill { forms, dismantle_recipe, .. } = &mut fragment.effect {
        forms.insert("bloom".into(), SkillFragmentForm {
            resonance_max: 100, skill_id: "u15_skill".into(), compatibility: None,
            bloom_recipe: Some(SkillFragmentRecipe { enkephalin, materials: vec![cost("part_a", 2), cost("part_a", part_a-2), cost("part_b", 3)] }),
        });
        *dismantle_recipe = Some(SkillFragmentRecipe { enkephalin: 0, materials: vec![cost("equipment_dust", 1)] });
    }
    let mut weapon = weapon_equipment(1502, "u15_weapon", WeaponArchetype::Sword);
    if staged_shots {
        weapon.weapon_profile = Some(ron::de::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/skill_composition/butterfly.weapon.ron"))).unwrap());
    }
    if evolution {
        use crate::game::ability::{marks::{MarkRatio, MarkFilter}, weapon_marks::*};
        let ratio = |numerator| MarkRatio { numerator, denominator: 1 };
        weapon.weapon_profile.as_mut().unwrap().mark_weapon = Some(MarkWeaponDef {
            evolution: WeaponEvolutionDef { fragment: "u15_fragment".into(), form: "bloom".into(), base: "base-gun".into(), upgraded: "upgraded-gun".into() },
            key: "marks".into(), filter: MarkFilter { air: false, boss: true, abnormality: true, summon: false, temporary: false },
            base: WeaponMarkHit {chance_percent: ratio(100), amount: 2, multiplier: ratio(1)},
            upgraded: WeaponMarkHit {chance_percent: ratio(100), amount: 1, multiplier: ratio(1)}, modifiers: vec![],
        });
    }
    let materials = [("equipment_dust", EquipmentMaterialType::EquipmentDust), ("part_a", EquipmentMaterialType::AbnormalityPart), ("part_b", EquipmentMaterialType::AbnormalityPart), ("unused_part", EquipmentMaterialType::AbnormalityPart), ("revival_catalyst", EquipmentMaterialType::RevivalCatalyst)]
        .into_iter().enumerate().map(|(i, (id, material_type))| EquipmentMaterialMetadata {
            id: id.into(), uuid: Uuid::from_u128(1510 + i as u128), name: id.into(), description: id.into(), material_type,
            rarity: crate::game::enums::RiskLevel::ZAYIN, equipment_type: None,
        }).collect();
    let equipment = EquipmentDatabase::with_all(equipment_with_starter_assets(vec![weapon]), materials,
        vec![EquipmentDismantleRecipeMetadata { equipment_id: "u15_weapon".into(), yields: vec![cost("equipment_dust", 1)] }],
        vec![EquipmentEnhancementRecipeMetadata { equipment_id: "u15_weapon".into(), max_level: 5, costs_per_level: vec![cost("equipment_dust", 1), cost("equipment_dust", 2)], modifiers_per_level: vec![StatModifier { stat: StatId::Attack, kind: StatModifierKind::Flat, value: 2 }] }]);
    let mut builder = test_game_data_builder().with_skill_data(skills)
        .with_skill_fragments(SkillFragmentDatabase::with_builtin_starter(vec![fragment]))
        .with_equipment_data(Arc::new(equipment));
    if staged_shots {
        let source = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/skill_composition/weapon_stages.skill.yaml")).replace("r11_weapon", "u15_weapon");
        builder = builder.with_equipment_attacks(crate::authoring::composition::compile_yaml(&source).unwrap().equipment_attacks().to_vec());
    }
    let data = builder.build_arc();
    let mut core = GameCore::new(data, 1515);
    core.state.player_profile.lifecycle.vault.materials = BTreeMap::from([("part_a".into(), 4), ("part_b".into(), 2), ("unused_part".into(), 9)]);
    let player = Uuid::from_u128(1515);
    start_new_game_with_default_starters(&mut core, player);
    let granted = core.apply_grant_effects(&[
        RewardEffect::GrantSkillFragment { fragment_id: "u15_fragment".into() },
        RewardEffect::GrantEquipment { equipment_id: "u15_weapon".into() },
        RewardEffect::GrantEquipmentMaterial { material_id: "part_a".into(), amount: 3 },
        RewardEffect::GrantEquipmentMaterial { material_id: "part_b".into(), amount: 3 },
        RewardEffect::GrantEquipmentDust { amount: 20 },
        RewardEffect::GrantEquipmentMaterial { material_id: "revival_catalyst".into(), amount: 2 },
    ]).unwrap();
    let copy = granted.skill_fragment_diffs[0].copy_uuid;
    let weapon = core.state.inventory.equipments.iter().find(|c| c.meta.id == "u15_weapon").unwrap().instance_uuid;
    (core, player, copy, weapon)
}
fn cost(id: &str, amount: u32) -> EquipmentMaterialCost { EquipmentMaterialCost { material_id: id.into(), amount } }
fn enter(core: &mut GameCore, player: Uuid, category: MapNodeCategory, id: &str, payload: MapNodePayload) {
    let node = force_first_available_node(core, category, id, payload);
    select_and_confirm_map_node(core, player, node);
}
fn maintenance(core: &mut GameCore, player: Uuid) { enter(core, player, MapNodeCategory::Maintenance, "maintenance", MapNodePayload::Maintenance); }
fn payment() -> Vec<MaterialPayment> { vec![MaterialPayment { material_id: "part_a".into(), carried: 3, warehouse: 2 }, MaterialPayment { material_id: "part_b".into(), carried: 1, warehouse: 2 }] }
fn bloom(core: &mut GameCore, player: Uuid, copy: Uuid) {
    core.execute(player, PlayerBehavior::AwakenSkillFragmentWithMaterials { copy_uuid: copy, form_id: "bloom".into(), materials: payment() }).unwrap();
}
fn assets(core: &GameCore) -> Value { json!({"profile":core.state.player_profile,"inventory":core.state.inventory,"copies":core.state.skill_fragments,"roster":core.state.roster,"money":core.state.enkephalin,"crafting":core.state.crafting_resources,"uuids":core.state.uuid_manager,"run":core.state.run.as_ref().unwrap().run_progression}) }
fn count(core: &GameCore, id: &str) -> u32 { crate::game::material_economy::carried_amount(&core.state.inventory, &core.state.crafting_resources, id) }

#[test]
fn u15_mixed_payment_refund_and_repeated_savepoint_restore_move_all_roots_together() {
    let (mut core, player, copy, _) = fixture();
    enter(&mut core, player, MapNodeCategory::SavePoint, "save_point", MapNodePayload::SavePoint);
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let saved_inventory = json!(core.state.inventory);
    let saved_warehouse = core.state.player_profile.lifecycle.vault.materials.clone();
    maintenance(&mut core, player);
    let entry_cost = core.run_state().unwrap().run_progression.deterrence;
    let money = core.state.enkephalin.amount;
    bloom(&mut core, player, copy);
    assert_eq!(core.state.enkephalin.amount, money - 7);
    assert_eq!((count(&core,"part_a"),count(&core,"part_b")),(0,2));
    assert_eq!(core.state.player_profile.lifecycle.vault.materials.get("part_a"),Some(&2));
    assert!(!core.state.player_profile.lifecycle.vault.materials.contains_key("part_b"));
    let ledger = &core.state.skill_fragments.get(&copy).unwrap().investment;
    assert_eq!(ledger.materials["part_a"], MaterialSources {carried:3,warehouse:2});
    assert_eq!(ledger.refunds().unwrap(),BTreeMap::from([("part_a".into(),2),("part_b".into(),1)]));
    let owner = core.state.roster.combat_eligible_employee_ids()[0];
    core.execute(player, PlayerBehavior::EquipSkillFragment { employee_uuid: owner, copy_uuid: copy }).unwrap();
    let before = assets(&core);
    assert!(core.execute(player,PlayerBehavior::DismantleSkillFragment {copy_uuid:copy,confirmed:false}).is_err());
    assert_eq!(assets(&core),before);
    core.execute(player,PlayerBehavior::SelectSkillFragmentForm {copy_uuid:copy,form_id:"bloom".into()}).unwrap();
    core.execute(player,PlayerBehavior::AwakenSkillFragment {copy_uuid:copy,form_id:"bloom".into()}).unwrap();
    let warehouse = core.state.player_profile.lifecycle.vault.materials.clone();
    core.execute(player,PlayerBehavior::DismantleSkillFragment {copy_uuid:copy,confirmed:true}).unwrap();
    assert_eq!((count(&core,"part_a"),count(&core,"part_b"),count(&core,"equipment_dust")),(2,3,21));
    assert_eq!(core.state.enkephalin.amount,money-7,"investment currency has no refund");
    assert_eq!(core.state.player_profile.lifecycle.vault.materials,warehouse,"refunds go to carried inventory");
    assert_eq!(core.run_state().unwrap().run_progression.deterrence,entry_cost);
    core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    for used in 1..=3 {
        core.execute(player,PlayerBehavior::LoadRunCheckpoint).unwrap();
        assert_eq!(core.state.run_checkpoint.loads_used,used);
        assert_eq!(json!(core.state.inventory),saved_inventory);
        assert_eq!(core.state.player_profile.lifecycle.vault.materials,saved_warehouse);
        assert!(core.state.skill_fragments.get(&copy).unwrap().investment.materials.is_empty());
        assert!(!core.state.skill_fragments.get(&copy).unwrap().unlocked_forms.contains("bloom"));
    }
}

#[test]
fn u15_explicit_allocation_rejection_is_atomic_and_never_falls_back_to_warehouse() {
    let (mut core,player,copy,_) = fixture();maintenance(&mut core,player);
    let mut duplicate=payment();duplicate.push(duplicate[0].clone());
    let mut insufficient=payment();insufficient[0].warehouse=5;insufficient[0].carried=0;
    let mut mismatch=payment();mismatch[1].carried=0;
    for materials in [duplicate,insufficient,mismatch,vec![]] {
        let before=assets(&core);
        assert!(core.execute(player,PlayerBehavior::AwakenSkillFragmentWithMaterials {copy_uuid:copy,form_id:"bloom".into(),materials}).is_err());
        assert_eq!(assets(&core),before);
    }
    let before=assets(&core);
    assert!(core.execute(player,PlayerBehavior::AwakenSkillFragment {copy_uuid:copy,form_id:"bloom".into()}).is_err());
    assert_eq!(assets(&core),before,"legacy carried-only request cannot consume warehouse");
    bloom(&mut core,player,copy);
}

#[test]
fn u15_weapon_paid_totals_confirmation_and_free_growth_are_distinct() {
    let (mut core,player,_,weapon)=fixture();maintenance(&mut core,player);
    let owner=core.state.roster.combat_eligible_employee_ids()[0];
    core.execute(player,PlayerBehavior::EquipItem {item_uuid:weapon,target_unit:owner}).unwrap();
    for _ in 0..3 { core.execute(player,PlayerBehavior::EnhanceEquipment {item_uuid:weapon}).unwrap(); }
    assert_eq!(count(&core,"equipment_dust"),11);
    let options=core.maintenance_options();let item=options.items.iter().find(|i|i.target_id==weapon.to_string()).unwrap();
    assert_eq!(item.investment_refund[0].amount,4,"sum paid9 then floor, not floor3 per operation");
    assert_eq!(item.operations.dismantle.gains[0].amount,5,"base1 + refund4");
    assert!(item.operations.dismantle.requires_confirm);
    let before=assets(&core);
    assert!(core.execute(player,PlayerBehavior::DismantleEquipment {item_uuid:weapon}).is_err());assert_eq!(assets(&core),before);
    core.execute(player,PlayerBehavior::DismantleEquipmentConfirmed {item_uuid:weapon,confirmed:true}).unwrap();
    assert_eq!(count(&core,"equipment_dust"),16);
    let before=assets(&core);
    assert!(core.execute(player,PlayerBehavior::DismantleEquipmentConfirmed {item_uuid:weapon,confirmed:true}).is_err());assert_eq!(assets(&core),before);
    // A free producer's level does not synthesize paid investment.
    let mut free=OwnedEquipment::new(Uuid::from_u128(1599),Arc::new(core.game_data.equipment_data.get_by_id("u15_weapon").unwrap().clone()));
    free.enhancement_level=3;core.state.inventory.equipments.add_item(free).unwrap();
    core.execute(player,PlayerBehavior::DismantleEquipment {item_uuid:Uuid::from_u128(1599)}).unwrap();assert_eq!(count(&core,"equipment_dust"),17);
}

#[test]
fn u15_unfinished_continue_restores_sources_but_completed_continue_and_failure_keep_spend() {
    for complete in [false,true] {
        let (mut core,player,copy,_)=fixture();maintenance(&mut core,player);bloom(&mut core,player,copy);
        if complete {core.execute(player,PlayerBehavior::CompleteNode).unwrap();}
        let saved=core.export_active_run_checkpoint().unwrap().unwrap();
        let mut restarted=core_with_profile_for_continue(&core,991);
        restarted.continue_from_active_run_checkpoint(player,&saved.core_payload_json,saved.loads_used,saved.run_peak_certified_power).unwrap();
        assert_eq!(restarted.state.skill_fragments.get(&copy).unwrap().unlocked_forms.contains("bloom"),complete);
        assert_eq!(restarted.state.player_profile.lifecycle.vault.materials["part_a"],if complete {2}else{4});
        assert_eq!(count(&restarted,"part_a"),if complete {0}else{3});
        if complete {
            let warehouse=restarted.state.player_profile.lifecycle.vault.materials.clone();
            restarted.execute(player,PlayerBehavior::AbandonRun).unwrap();
            assert_eq!(restarted.state.player_profile.lifecycle.vault.materials,warehouse);
            assert!(restarted.state.player_profile.lifecycle.warehouse_debits.is_empty());
        }
    }
}

#[test]
fn u15_selected_return_caps_and_fragment_investment_survive_reloan_weapon_investment_does_not() {
    let (mut core,player,copy,weapon)=fixture();maintenance(&mut core,player);bloom(&mut core,player,copy);
    core.execute(player,PlayerBehavior::EnhanceEquipment {item_uuid:weapon}).unwrap();
    core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    core.state.run.as_mut().unwrap().run_progression.mode_state=crate::game::map::RunProgressionModeState::Expedition {floor_index:2};
    enter(&mut core,player,MapNodeCategory::Gate,"gate_stairs",MapNodePayload::None);
    for materials in [vec![MaterialRecovery {material_id:"part_b".into(),amount:3}],vec![MaterialRecovery {material_id:"revival_catalyst".into(),amount:2}],vec![MaterialRecovery {material_id:"equipment_dust".into(),amount:1}]] {
        let before=assets(&core);assert!(core.execute(player,PlayerBehavior::ReturnFromExpedition {materials}).is_err());assert_eq!(assets(&core),before);
    }
    core.execute(player,PlayerBehavior::ReturnFromExpedition {materials:vec![MaterialRecovery {material_id:"part_b".into(),amount:2},MaterialRecovery {material_id:"revival_catalyst".into(),amount:1}]}).unwrap();
    let profile=&core.state.player_profile.lifecycle;
    assert_eq!(profile.vault.materials,BTreeMap::from([("part_a".into(),2),("part_b".into(),2),("unused_part".into(),9),("revival_catalyst".into(),1)]));
    assert_eq!(profile.vault.equipment[&weapon].enhancement_level,0);
    assert_eq!(profile.vault.fragments[&copy].investment.refunds().unwrap()["part_a"],2);
    core.execute(player,PlayerBehavior::AcknowledgeFirstReturnStory).unwrap();
    // Supported next application generation has a different authored bloom price.
    let revised=fixture_with_costs(99,99).0;
    let mut next=GameCore::new(revised.game_data.clone(),1515);
    next.install_authoritative_player_profile(core.state.player_profile.clone()).unwrap();
    let (mut core,ids)=super::u12_lifecycle::selecting_core(next,player);
    core.execute(player,PlayerBehavior::SelectStarterEmployees {employee_definition_ids:ids,loan_copy_ids:vec![copy,weapon]}).unwrap();
    assert!(core.state.inventory.equipments.get_item(&weapon).unwrap().investment.materials.is_empty());
    maintenance(&mut core,player);
    core.execute(player,PlayerBehavior::DismantleSkillFragment {copy_uuid:copy,confirmed:true}).unwrap();
    assert_eq!((count(&core,"part_a"),count(&core,"part_b")),(2,1));
    assert_eq!(core.state.player_profile.lifecycle.vault.materials["part_a"],2);
}

#[test]
fn u15_unaccepted_transaction_rollback_restores_warehouse_and_investment_before_retry() {
    let (mut core,player,copy,_)=fixture();maintenance(&mut core,player);
    let before=assets(&core);
    let rollback=core.capture_active_run_transaction_rollback().unwrap().unwrap();
    bloom(&mut core,player,copy);
    core.restore_active_run_transaction_rollback(rollback);
    let after=assets(&core);
    for (key,value) in before.as_object().unwrap() {assert_eq!(&after[key],value,"rollback root {key}");}
    bloom(&mut core,player,copy);
    assert_eq!(core.state.player_profile.lifecycle.vault.materials["part_a"],2);
    assert_eq!(core.state.skill_fragments.get(&copy).unwrap().investment.refunds().unwrap()["part_a"],2);
}

#[test]
fn u15_selected_payment_query_is_read_only_and_matches_actual_paid_sources() {
    let (mut core,player,copy,_)=fixture();maintenance(&mut core,player);
    let before=assets(&core);let saved=core.export_active_run_checkpoint().unwrap().unwrap();
    for (selected,expected) in [(payment(),true),(vec![],false)] {
        let result=core.execute(player,PlayerBehavior::PreviewSkillFragmentBloom{weapon_copy_uuid:None,copy_uuid:copy,form_id:"bloom".into(),materials:selected}).unwrap();
        let BehaviorResult::MaintenanceState{maintenance_options,..}=result else {panic!("quote");};
        let quote=maintenance_options.selected_bloom_payment.unwrap();
        assert_eq!(quote.can_execute,expected);assert_eq!(quote.enkephalin,7);
        assert_eq!(quote.payment.requirements,BTreeMap::from([("part_a".into(),5),("part_b".into(),3)]));
        if expected {assert_eq!(quote.payment.actual.unwrap().materials["part_a"],MaterialSources{carried:3,warehouse:2});}
        assert_eq!(assets(&core),before);
        assert_eq!(core.export_active_run_checkpoint().unwrap().unwrap().core_payload_json,saved.core_payload_json);
    }
    bloom(&mut core,player,copy);
    let paid=core.state.skill_fragments.get(&copy).unwrap().investment.clone();
    let owners=core.state.roster.combat_eligible_employee_ids();
    core.execute(player,PlayerBehavior::EquipSkillFragment{employee_uuid:owners[0],copy_uuid:copy}).unwrap();
    core.execute(player,PlayerBehavior::TransferSkillFragment{copy_uuid:copy,target_employee_uuid:owners[1]}).unwrap();
    core.execute(player,PlayerBehavior::SelectSkillFragmentForm{copy_uuid:copy,form_id:"bloom".into()}).unwrap();
    core.execute(player,PlayerBehavior::AwakenSkillFragment{copy_uuid:copy,form_id:"bloom".into()}).unwrap();
    assert_eq!(core.state.skill_fragments.get(&copy).unwrap().investment,paid);
}

#[test]
fn u15_restore_keeps_precheckpoint_warehouse_spend_and_first_copy_investment() {
    let (mut core,player,first,_)=fixture();maintenance(&mut core,player);bloom(&mut core,player,first);
    core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    let granted=core.apply_grant_effects(&[
        RewardEffect::GrantSkillFragment{fragment_id:"u15_fragment".into()},
        RewardEffect::GrantEquipmentMaterial{material_id:"part_a".into(),amount:3},
        RewardEffect::GrantEquipmentMaterial{material_id:"part_b".into(),amount:1},
    ]).unwrap();let second=granted.skill_fragment_diffs[0].copy_uuid;
    enter(&mut core,player,MapNodeCategory::SavePoint,"save_point",MapNodePayload::SavePoint);
    core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    let saved=(core.state.inventory.clone(),core.state.skill_fragments.clone(),core.state.player_profile.lifecycle.clone());
    maintenance(&mut core,player);
    core.execute(player,PlayerBehavior::AwakenSkillFragmentWithMaterials{copy_uuid:second,form_id:"bloom".into(),materials:vec![
        MaterialPayment{material_id:"part_a".into(),carried:3,warehouse:2},MaterialPayment{material_id:"part_b".into(),carried:3,warehouse:0}]}).unwrap();
    core.execute(player,PlayerBehavior::DismantleSkillFragment{copy_uuid:second,confirmed:true}).unwrap();
    core.execute(player,PlayerBehavior::CompleteNode).unwrap();
    for _ in 0..3 {
        core.execute(player,PlayerBehavior::LoadRunCheckpoint).unwrap();
        assert_eq!(json!(core.state.inventory),json!(saved.0));assert_eq!(json!(core.state.skill_fragments),json!(saved.1));
        assert_eq!(core.state.player_profile.lifecycle,saved.2);
        assert_eq!(core.state.player_profile.lifecycle.warehouse_debits,BTreeMap::from([("part_a".into(),2),("part_b".into(),2)]));
    }
}

#[test]
fn u15_refund_overflow_and_weapon_maximum_refuse_without_partial_destruction() {
    let (mut core,player,_,weapon)=fixture();maintenance(&mut core,player);
    for _ in 0..5 {core.execute(player,PlayerBehavior::EnhanceEquipment{item_uuid:weapon}).unwrap();}
    let before=assets(&core);
    assert!(!core.maintenance_options().items.iter().find(|i|i.target_id==weapon.to_string()).unwrap().operations.enhance.can_execute);
    assert!(core.execute(player,PlayerBehavior::EnhanceEquipment{item_uuid:weapon}).is_err());assert_eq!(assets(&core),before);
    core.apply_grant_effects(&[RewardEffect::GrantEquipmentMaterial{material_id:"equipment_dust".into(),amount:u32::MAX-count(&core,"equipment_dust")}]).unwrap();
    let before=assets(&core);
    assert!(!core.maintenance_options().items.iter().find(|i|i.target_id==weapon.to_string()).unwrap().operations.dismantle.can_execute);
    assert!(core.execute(player,PlayerBehavior::DismantleEquipment{item_uuid:weapon}).is_err());assert_eq!(assets(&core),before);
}

#[test]
fn u15_catalyst_common_payment_quote_commit_and_rewind_share_sources_without_incubation() {
    use crate::game::material_economy::{pay_materials,preview_material_payment,reconcile_warehouse};
    let (core,_,_,_)=fixture();let mut inventory=core.state.inventory.clone();let mut resources=core.state.crafting_resources;
    let mut warehouse=BTreeMap::from([("revival_catalyst".into(),1),("unused_part".into(),7)]);let mut debits=BTreeMap::new();
    let requirements=BTreeMap::from([("revival_catalyst".into(),2)]);
    let selected=vec![MaterialPayment{material_id:"revival_catalyst".into(),carried:1,warehouse:1}];
    let before=(json!(inventory),warehouse.clone(),debits.clone());
    let quote=preview_material_payment(&inventory,&resources,&warehouse,&debits,&requirements,&selected,true,&core.game_data.equipment_data);
    assert_eq!((json!(inventory),warehouse.clone(),debits.clone()),before);
    let paid=pay_materials(&mut inventory,&mut resources,&mut warehouse,&mut debits,&requirements,Some(&selected),true,&core.game_data.equipment_data).unwrap();
    assert_eq!(quote.actual,Some(paid));assert_eq!(inventory.equipment_materials.amount("revival_catalyst"),1);
    assert!(!warehouse.contains_key("revival_catalyst"));assert_eq!(warehouse["unused_part"],7);
    assert_eq!(reconcile_warehouse(&warehouse,&debits,&BTreeMap::new()).unwrap(),before.1);
    let before=(json!(inventory),warehouse.clone(),debits.clone());
    assert!(pay_materials(&mut inventory,&mut resources,&mut warehouse,&mut debits,&requirements,Some(&selected),true,&core.game_data.equipment_data).is_err());
    assert_eq!((json!(inventory),warehouse,debits),before);
}

#[test]
fn u15_currency_shortage_refuses_before_any_material_or_copy_changes() {
    let (mut core,player,copy,_)=fixture_with_costs(u32::MAX,5);maintenance(&mut core,player);
    let before=assets(&core);
    assert!(core.execute(player,PlayerBehavior::AwakenSkillFragmentWithMaterials{copy_uuid:copy,form_id:"bloom".into(),materials:payment()}).is_err());
    assert_eq!(assets(&core),before);
}


#[test]
fn weapon_evolution_selected_copy_cost_unlock_save_and_replay_are_atomic() {
    use crate::game::ability::weapon_marks::WeaponStage;
    let (mut core, player, fragment, weapon) = fixture_with_options(7, 5, true);
    core.apply_grant_effects(&[RewardEffect::GrantEquipment {equipment_id: "u15_weapon".into()}]).unwrap();
    let second = core.state.inventory.equipments.iter().find(|c| c.meta.id == "u15_weapon" && c.instance_uuid != weapon).unwrap().instance_uuid;
    maintenance(&mut core, player);
    let before = assets(&core);
    for selection in [None, Some(Uuid::from_u128(0xdead))] {
        assert!(core.handle_awaken_skill_fragment_selected(fragment, "bloom", Some(&payment()), selection).is_err());
        assert_eq!(assets(&core), before);
    }
    let result = core.execute(player, PlayerBehavior::AwakenSkillFragmentWithWeapon {
        copy_uuid: fragment, form_id: "bloom".into(), weapon_copy_uuid: second, materials: Some(payment()),
    }).unwrap();
    let _ = result;
    assert_eq!(core.state.enkephalin.amount, before["money"]["amount"].as_u64().unwrap() as u32 - 7);
    assert!(core.state.skill_fragments.get(&fragment).unwrap().unlocked_forms.contains("bloom"));
    assert_eq!(core.state.inventory.equipments.get_item(&weapon).unwrap().weapon_stage, WeaponStage::Base);
    let upgraded = core.state.inventory.equipments.get_item(&second).unwrap();
    assert_eq!(upgraded.weapon_stage, WeaponStage::Upgraded);
    let json = serde_json::to_string(upgraded).unwrap();
    let loaded: OwnedEquipment = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.weapon_stage, WeaponStage::Upgraded);
    let returned = crate::game::expedition_lifecycle::VaultEquipment::from_returned(upgraded);
    assert_eq!(returned.weapon_stage, WeaponStage::Upgraded);
    let after = assets(&core);
    assert!(core.execute(player, PlayerBehavior::AwakenSkillFragmentWithWeapon {
        copy_uuid: fragment, form_id: "bloom".into(), weapon_copy_uuid: weapon, materials: None,
    }).is_err());
    assert_eq!(assets(&core), after);
    core.execute(player, PlayerBehavior::AwakenSkillFragment {copy_uuid:fragment, form_id:"bloom".into()}).unwrap();
    assert_eq!(assets(&core), after);
    let owner = core.state.roster.combat_eligible_employee_ids()[0];
    core.execute(player, PlayerBehavior::EquipItem {item_uuid:second,target_unit:owner}).unwrap();
    core.execute(player, PlayerBehavior::EquipSkillFragment {employee_uuid:owner,copy_uuid:fragment}).unwrap();
    for form in ["bloom", "base"] {
        core.execute(player, PlayerBehavior::SelectSkillFragmentForm {copy_uuid:fragment,form_id:form.into()}).unwrap();
        let draft = crate::game::combat_setup::player_spawns::battle_unit_draft_for_employee(
            &core.state.roster, &core.state.inventory, &core.state.skill_fragments, &core.game_data, owner).unwrap();
        assert_eq!(draft.equipped_items.iter().find(|b| b.persistent_copy_uuid == Some(second)).unwrap().weapon_stage, WeaponStage::Upgraded);
    }
    core.execute(player, PlayerBehavior::UnequipSkillFragment {employee_uuid:owner,copy_uuid:fragment}).unwrap();
    assert_eq!(core.state.inventory.equipments.get_item(&second).unwrap().weapon_stage, WeaponStage::Upgraded);
    core.apply_grant_effects(&[RewardEffect::GrantEquipment {equipment_id: "u15_weapon".into()}]).unwrap();
    assert!(core.state.inventory.equipments.iter().filter(|c| c.meta.id == "u15_weapon" && c.instance_uuid != second).all(|c| c.weapon_stage == WeaponStage::Base));
}

#[test]
fn weapon_evolution_failed_payment_and_post_evolution_cancel_restore_all_assets() {
    use crate::authoring::control;
    let (mut core, player, fragment, weapon) = fixture_with_options(7, 5, true);
    maintenance(&mut core, player);
    let before = assets(&core);
    assert!(core.handle_awaken_skill_fragment_selected(fragment, "bloom", Some(&[]), Some(weapon)).is_err());
    assert_eq!(assets(&core), before);
    let result = control::with_test_cancellation("fragment_weapon_evolved", 1, || {
        control::run(&control::ExecutionLimits::default(), &control::CancellationToken::default(), control::ExecutionPhase::TrialExecution, || {
            core.handle_awaken_skill_fragment_selected(fragment, "bloom", Some(&payment()), Some(weapon))
        })
    });
    assert!(result.is_err());
    assert_eq!(assets(&core), before);
    core.execute(player, PlayerBehavior::AwakenSkillFragmentWithWeapon {
        copy_uuid: fragment, form_id: "bloom".into(), weapon_copy_uuid: weapon, materials: Some(payment()),
    }).unwrap();
}

include!("u15_weapon_stages.rs");
