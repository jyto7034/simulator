use super::*;

fn recruitment_session(seed: u64) -> (GameCore, Uuid, Vec<EmployeeDefinition>) {
    let player = Uuid::from_u128(u128::from(seed));
    let mut core = GameCore::new(game_data_with_map_content(), seed);
    start_new_game_with_default_starters(&mut core, player);
    let node = force_first_available_node(&mut core, MapNodeCategory::HeadquartersContact,
        "headquarters_contact", MapNodePayload::HeadquartersContact { shop_pool_id: None, candidate_count: 6 });
    let entered = select_and_confirm_map_node(&mut core, player, node);
    let BehaviorResult::HeadquartersContactState { recruitment_candidates, .. } = entered else { panic!("HQ"); };
    assert!(!recruitment_candidates.is_empty());
    (core, player, recruitment_candidates)
}

#[test]
fn u12_lifecycle_paid_recruitment_debits_once_grants_normal_loadout_and_caps_session_at_two() {
    let (mut core, player, candidates) = recruitment_session(12401);
    let start_balance = core.state.enkephalin.amount;
    let initial_people = core.state.roster.len();
    for (index, candidate) in candidates.iter().take(2).enumerate() {
        let before_ids: BTreeSet<_> = core.state.roster.iter().map(|employee| employee.uuid).collect();
        core_command(&mut core, player, json!({"type":"recruit_employee",
            "employee_definition_id":candidate.employee_definition_id})).unwrap();
        assert_eq!(core.state.enkephalin.amount, start_balance - 100 * (index as u32 + 1));
        assert_eq!(core.state.roster.len(), initial_people + index + 1);
        let employee = core.state.roster.iter().find(|employee| !before_ids.contains(&employee.uuid)).unwrap();
        assert_eq!(employee.employee_definition_id, candidate.employee_definition_id);
        assert_eq!(employee.role, candidate.role);
        assert!(!employee.loadout.item_slot.is_empty(), "a recruit receives an actual normal equipment set");
        let owned: BTreeSet<_> = employee.loadout.item_slot.iter().map(|item| item.instance_uuid).collect();
        assert!(!owned.is_empty());
        for copy in owned {
            assert_eq!(core.state.inventory.equipments.get_item(&copy).unwrap().equipped_to, Some(employee.uuid));
        }
        let after = core_roots(&core);
        assert!(core_command(&mut core, player, json!({"type":"recruit_employee",
            "employee_definition_id":candidate.employee_definition_id})).is_err());
        assert_eq!(core_roots(&core), after, "repeat candidate cannot duplicate charge or assets");
    }
    let before = core_roots(&core);
    assert!(core_command(&mut core, player, json!({"type":"recruit_employee",
        "employee_definition_id":candidates[2].employee_definition_id})).is_err());
    assert_eq!(core_roots(&core), before, "a third hire is rejected atomically");
}

#[test]
fn u12_lifecycle_insufficient_money_and_unknown_recruit_preserve_queue_and_identity_stream() {
    let (mut core, player, candidates) = recruitment_session(12411);
    core.state.enkephalin.amount = 99;
    for id in [candidates[0].employee_definition_id.as_str(), "not_in_this_queue"] {
        let before = core_roots(&core);
        assert!(core_command(&mut core, player, json!({"type":"recruit_employee","employee_definition_id":id})).is_err());
        assert_eq!(core_roots(&core), before);
    }
    core.state.enkephalin.amount = 100;
    core_command(&mut core, player, json!({"type":"recruit_employee",
        "employee_definition_id":candidates[0].employee_definition_id})).unwrap();
    assert_eq!(core.state.enkephalin.amount, 0);
}

#[test]
fn u12_lifecycle_hq_query_and_command_share_price_depth_and_roster_boundaries() {
    for (depth, roster_size, wallet, allowed) in [
        (4, 9, 100, true), (4, 9, 101, true), (4, 9, 99, false),
        (5, 9, 100, false), (4, 10, 100, false),
    ] {
        let (mut core, player, candidates) = recruitment_session(12421);
        core.state.run.as_mut().unwrap().run_progression.mode_state =
            crate::game::map::RunProgressionModeState::Expedition { floor_index: depth - 1 };
        let reserved: BTreeSet<_> = candidates.iter().take(2)
            .map(|candidate| candidate.employee_definition_id.as_str()).collect();
        while core.state.roster.len() < roster_size {
            let index = core.state.roster.len();
            let definition = core.game_data.employee_definition_data.definitions.iter().find(|definition|
                !reserved.contains(definition.employee_definition_id.as_str())
                    && !core.state.roster.iter().any(|employee| employee.employee_definition_id == definition.employee_definition_id))
                .expect("fixture has enough real definitions to reach the roster boundary").clone();
            core.state.roster.add(crate::game::employee::Employee::from_definition(
                Uuid::from_u128(12430 + index as u128), &definition));
        }
        let owned: Vec<_> = core.state.roster.iter().map(|employee| employee.employee_definition_id.clone()).collect();
        let mut recruitment = crate::game::recruitment::RecruitmentState::initialize(
            12421, &core.game_data.employee_definition_data.definitions, &owned).unwrap();
        recruitment.begin_session(core.state.node_session.as_ref().unwrap().node_id, 6).unwrap();
        recruitment.validate_exact_ownership(&core.game_data.employee_definition_data.definitions, &core.state.roster).unwrap();
        core.state.run.as_mut().unwrap().recruitment = recruitment;
        core.state.enkephalin.amount = wallet;
        let projected = core.headquarters_contact_state_result().unwrap();
        let BehaviorResult::HeadquartersContactState { recruitment_candidates, .. } = projected else { panic!("HQ projection"); };
        assert_eq!(!recruitment_candidates.is_empty(), allowed, "depth={depth} roster={roster_size} wallet={wallet}");
        let query = core.get_run_snapshot_json().unwrap();
        let visible = query["selected_event"]["recruitment_candidates"].as_array().unwrap();
        assert_eq!(!visible.is_empty(), allowed, "actual query depth={depth} roster={roster_size} wallet={wallet}");
        assert_eq!(serde_json::to_value(&recruitment_candidates).unwrap(), json!(visible));
        let before = core_roots(&core);
        let result = core_command(&mut core, player, json!({"type":"recruit_employee",
            "employee_definition_id":candidates[0].employee_definition_id}));
        if allowed {
            result.unwrap();
            assert_eq!(core.state.roster.len(), 10);
            assert_eq!(core.state.enkephalin.amount, wallet - 100);
            let after = core_roots(&core);
            assert!(core_command(&mut core, player, json!({"type":"recruit_employee",
                "employee_definition_id":candidates[1].employee_definition_id})).is_err());
            assert_eq!(core_roots(&core), after);
        } else {
            assert!(result.is_err());
            assert_eq!(core_roots(&core), before);
        }
    }
}

#[test]
fn u12_lifecycle_hq_mixed_queue_exposes_only_deployable_candidates() {
    let mut definitions = test_employee_definition_database().definitions;
    for candidate in definitions.iter_mut().filter(|candidate| !candidate.starter_eligible) {
        if candidate.employee_definition_id != "recruit_0" {
            candidate.initial_loadout.equipment_ids.clear();
        }
    }
    let data = test_game_data_builder().with_employee_definitions(EmployeeDefinitionDatabase::new(definitions)).build_arc();
    let player = Uuid::from_u128(12461);
    let mut core = GameCore::new(data, 12461);
    start_new_game_with_default_starters(&mut core, player);
    let node = force_first_available_node(&mut core, MapNodeCategory::HeadquartersContact,
        "headquarters_contact", MapNodePayload::HeadquartersContact { shop_pool_id: None, candidate_count: 32 });
    let entered = select_and_confirm_map_node(&mut core, player, node);
    let BehaviorResult::HeadquartersContactState { recruitment_candidates, .. } = entered else { panic!("HQ"); };
    assert!(recruitment_candidates.iter().any(|candidate| candidate.employee_definition_id == "recruit_0"));
    assert!(recruitment_candidates.iter().all(|candidate| candidate.starter_eligible || candidate.employee_definition_id == "recruit_0"));
    assert_eq!(recruitment_candidates.len(), 4, "three unselected starters plus the one deployable recruit");
    let query = core.get_run_snapshot_json().unwrap();
    assert_eq!(query["selected_event"]["recruitment_candidates"], serde_json::to_value(&recruitment_candidates).unwrap());
    let before = core_roots(&core);
    assert!(core_command(&mut core, player, json!({"type":"recruit_employee","employee_definition_id":"recruit_1"})).is_err());
    assert_eq!(core_roots(&core), before);
    core_command(&mut core, player, json!({"type":"recruit_employee","employee_definition_id":"recruit_0"})).unwrap();
    assert!(core.get_run_snapshot_json().unwrap()["selected_event"]["recruitment_candidates"].as_array().unwrap()
        .iter().all(|candidate| candidate["employee_definition_id"] != "recruit_0"));
}

#[test]
fn u12_lifecycle_paid_grant_collision_with_dormant_vault_rejects_every_root_then_retries() {
    for cross_kind in [false, true] {
        let (mut core, player, candidates) = recruitment_session(12471);
        let mut future = core.state.uuid_manager.clone();
        future.next_owned_equipment();
        let collision = future.next_owned_equipment();
        if cross_kind {
            core.state.player_profile.lifecycle.vault.fragments.insert(collision,
                fragment(collision.as_u128(), "dormant_collision_fragment"));
        } else {
            core.state.player_profile.lifecycle.vault.equipment.insert(collision,
                equipment(collision.as_u128(), "dormant_collision_weapon"));
        }
        core.state.player_profile.validate_persisted().unwrap();
        core.state.player_profile.lifecycle.vault.validate_active_ownership(
            &core.state.inventory, &core.state.skill_fragments).unwrap();
        let before = core_roots(&core);
        let command = json!({"type":"recruit_employee","employee_definition_id":candidates[0].employee_definition_id});
        assert!(core_command(&mut core, player, command.clone()).is_err());
        assert_eq!(core_roots(&core), before, "payment, queue, UUID, profile and continuation all stay unchanged");
        core.state.player_profile.lifecycle.vault.equipment.remove(&collision);
        core.state.player_profile.lifecycle.vault.fragments.remove(&collision);
        let wallet = core.state.enkephalin.amount;
        core_command(&mut core, player, command).unwrap();
        assert_eq!(core.state.enkephalin.amount, wallet - 100);
        assert!(core.state.inventory.equipments.get_item(&collision).is_some(), "the same reachable paid grant now succeeds with its unconsumed UUID");
    }
}

#[test]
fn u12_lifecycle_depth_five_hq_disables_recruitment_but_supplies_and_shop_still_complete() {
    for shop in [false, true] {
        let player = Uuid::from_u128(12481);
        // Depth five can select the policy-owned wandering merchant while
        // resolving a pooled shop. Supply the referenced shop as real data so
        // this oracle reaches the HQ session transition rather than failing
        // because the shallow map fixture omits a late-floor dependency.
        let mut wandering = test_shop_metadata("hq_supply_shop",
            Uuid::from_u128(12_481), ShopContext::HeadquartersSupport);
        // Shop pricing requires positive base prices. Free departure weapons
        // are not merchandise: use separate priced fixture definitions.
        // Reuse the existing priced, unbound, validated armor fixture in full;
        // change only definition/copy identities for three distinct choices.
        let merchandise=[("u12_hq_merchant_armor_a",124_811),
            ("u12_hq_merchant_armor_b",124_812),("u12_hq_merchant_armor_c",124_813)]
            .map(|(id,uuid)| {let mut item=starter_equipment_meta();item.id=id.into();item.uuid=Uuid::from_u128(uuid);item});
        assert_eq!(merchandise.iter().map(|item|item.uuid).collect::<BTreeSet<_>>().len(),3);
        wandering.offer_count = 3;
        wandering.local_entries = ["u12_hq_merchant_armor_a", "u12_hq_merchant_armor_b", "u12_hq_merchant_armor_c"]
            .into_iter().map(|equipment_id| ShopMerchandiseEntry {
                merchandise: ShopMerchandise::Equipment { equipment_id: equipment_id.into() },
                quantity: 1, weight: 1, repeatable: false, buy_percent: None,
            base_price: None,
            guaranteed: false,
                tags: vec!["equipment".into()],
            }).collect();
        let data = test_game_data_builder().with_equipment(equipment_with_starter_assets(merchandise.to_vec()))
            .with_shops(ShopDatabase::new_with_pools(
            vec![
                test_shop_metadata("map_shop", Uuid::from_u128(10_001), ShopContext::UnknownExploration),
                wandering,
            ],
            vec![
                ShopPoolMetadata { id: "default_shops".into(), shop_ids: vec!["map_shop".into()] },
                ShopPoolMetadata { id: "headquarters_basic_supplies".into(), shop_ids: vec!["hq_supply_shop".into()] },
            ],
        )).build_arc();
        for equipment_id in ["u12_hq_merchant_armor_a", "u12_hq_merchant_armor_b", "u12_hq_merchant_armor_c"] {
            assert!(data.equipment_data.get_by_id(equipment_id).unwrap().price>0, "priced merchant reference {equipment_id}");
        }
        let mut core = GameCore::new(data, 12481);
        start_new_game_with_default_starters(&mut core, player);
        core.state.run.as_mut().unwrap().run_progression.mode_state =
            crate::game::map::RunProgressionModeState::Expedition { floor_index: 4 };
        let queue_before: Vec<_> = core.state.run.as_ref().unwrap().recruitment.queue_definition_ids().map(str::to_owned).collect();
        let node = force_first_available_node(&mut core, MapNodeCategory::HeadquartersContact,
            "headquarters_contact", MapNodePayload::HeadquartersContact {
                shop_pool_id: Some("headquarters_basic_supplies".into()), candidate_count: 6 });
        let entered = select_and_confirm_map_node(&mut core, player, node);
        assert!(matches!(entered, BehaviorResult::HeadquartersContactState { recruitment_candidates, .. } if recruitment_candidates.is_empty()));
        assert!(core.get_run_snapshot_json().unwrap()["selected_event"]["recruitment_candidates"].as_array().unwrap().is_empty());
        assert_eq!(core.state.run.as_ref().unwrap().recruitment.queue_definition_ids().map(str::to_owned).collect::<Vec<_>>(), queue_before);
        let before = core_roots(&core);
        assert!(core_command(&mut core, player, json!({"type":"recruit_employee","employee_definition_id":queue_before[0]})).is_err());
        assert_eq!(core_roots(&core), before);
        let wallet = core.state.enkephalin.amount;
        if shop {
            let opened = core_command(&mut core, player, json!({"type":"open_headquarters_shop"})).unwrap();
            assert!(matches!(opened, BehaviorResult::ShopState { .. }));
            let offered = core.get_run_snapshot_json().unwrap()["selected_event"]["offers"].as_array().unwrap().clone();
            assert_eq!(offered.len(), 3, "the policy-owned first page is fully realized");
            assert_eq!(offered.iter().map(|offer| offer["merchandise"]["equipment_id"].as_str().unwrap()).collect::<BTreeSet<_>>().len(), 3);
            let mut prices:Vec<_>=offered.iter().map(|offer|offer["total_price"].as_u64().unwrap()).collect();
            prices.sort_unstable();assert_eq!(prices,vec![80,80,80]);
            let copies_before: BTreeSet<_>=core.state.inventory.equipments.iter().map(|copy|copy.instance_uuid).collect();
            core_command(&mut core,player,json!({"type":"purchase_item","offer_id":offered[0]["offer_id"]})).unwrap();
            assert_eq!(core.state.enkephalin.amount,wallet-80);
            let purchased:Vec<_>=core.state.inventory.equipments.iter().filter(|copy|!copies_before.contains(&copy.instance_uuid)).collect();
            assert_eq!(purchased.len(),1);
            assert_eq!(purchased[0].meta.id,offered[0]["merchandise"]["equipment_id"].as_str().unwrap());
            core_command(&mut core, player, json!({"type":"exit_shop"})).unwrap();
            assert_eq!(core.state.enkephalin.amount, wallet-80);
        } else {
            let supplied = core_command(&mut core, player, json!({"type":"request_emergency_supplies"})).unwrap();
            assert!(matches!(supplied, BehaviorResult::EmergencySuppliesGranted { .. }));
            assert_eq!(core.state.enkephalin.amount, wallet + 120);
        }
        assert!(matches!(core.state.lifecycle.phase(), GameState::ViewingMap));
        let recruitment = &core.state.run.as_ref().unwrap().recruitment;
        assert!(recruitment.active_session().is_none());
        assert_eq!(recruitment.queue_definition_ids().map(str::to_owned).collect::<Vec<_>>(), queue_before);
        recruitment.validate_exact_ownership(&core.game_data.employee_definition_data.definitions, &core.state.roster).unwrap();
    }
}
