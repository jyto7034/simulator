//! Routed owner checks; external loader/paired-store evidence is collected separately.
use super::*;
use crate::game::behavior::ActionDisabledReasonCode;
use crate::game::data::{relic_data::RelicDatabase, shop_data::ShopRefreshPolicy};

fn setup(seed: u64) -> (GameCore, Uuid) {
    let base = GameCore::new(live_game_data_from_ron(), seed);
    let relics: RelicDatabase = ron::de::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/u08/relics.ron"))).unwrap();
    let entry = |merchandise, price, repeatable| ShopMerchandiseEntry {
        merchandise, quantity: 1, base_price: Some(price), guaranteed: true,
        weight: 1, repeatable, buy_percent: None, tags: vec![],
    };
    let shop = ShopMetadata {
        id: "u16_shop".into(), name: "U16 support fixture".into(), uuid: Uuid::from_u128(1616),
        context: ShopContext::UnknownExploration, offer_count: 3, default_buy_percent: 100,
        preset_refs: vec![], local_entries: vec![
            entry(ShopMerchandise::Relic { relic_id: "u08_probe".into() }, 60, true),
            entry(ShopMerchandise::RevivalCatalyst { material_id: "revival_catalyst".into() }, 150, true),
            entry(ShopMerchandise::RelicCorruptionCatalyst, 30, true),
        ], refresh_policy: Some(ShopRefreshPolicy { base_cost: 7, max_uses: 5 }),
        sell_policy: Some(ShopSellPolicy { price_percent: 50, accepted_tags: None }),
    };
    let data = GameDataBuilder::from_validated(&base.game_data).with_relics(relics).with_shops(ShopDatabase::new(vec![shop])).build_arc();
    let mut core = GameCore::new(data, seed);
    let player = Uuid::from_u128(16);
    start_new_game_with_default_starters(&mut core, player);
    // Setup grant is before entry so both continuation alternatives have the same funds.
    core.apply_grant_effects(&[RewardEffect::GrantEnkephalin { amount: 20000 }]).unwrap();
    let node = force_first_available_node(&mut core, MapNodeCategory::Shop, "shop_general", MapNodePayload::Shop { shop_id: Some("u16_shop".into()), shop_pool_id: None });
    select_and_confirm_map_node(&mut core, player, node);
    (core, player)
}
fn offers(core: &GameCore) -> Vec<crate::game::resources::ShopOfferState> {
    core.state.active_node_content.as_ref().unwrap().as_shop().unwrap().offers.clone()
}
fn roots(core: &GameCore) -> Value {
    json!({"world": non_battle_state_without_profile_fingerprint(core), "relics":core.state.relics,
        "crafting":core.state.crafting_resources,"uuid":core.state.uuid_manager,
        "continue":core.export_active_run_checkpoint().unwrap().map(|v|v.core_payload_json)})
}
fn buy(core: &mut GameCore, player: Uuid, offer: Uuid) -> BehaviorResult {
    core.execute(player, PlayerBehavior::PurchaseItem { offer_id: offer }).unwrap()
}

#[test]
fn u16_new_goods_query_and_atomic_purchase_publish_exact_distinct_assets() {
    let (mut core, player) = setup(16);
    let before = roots(&core);
    let display = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(display["offers"].as_array().unwrap().len(), 3);
    assert_eq!(roots(&core), before, "query does not allocate copy identities");
    let money = core.state.enkephalin.amount;
    for offer in offers(&core) {
        let result = buy(&mut core, player, offer.offer_id);
        let wire = serde_json::to_value(&result).unwrap();
        let decoded: BehaviorResult = serde_json::from_value(wire).unwrap();
        let BehaviorResult::PurchaseItem { relic_diffs, crafting_resources, inventory_diff, .. } = decoded else { panic!("purchase result"); };
        match offer.merchandise {
            ShopMerchandise::Relic { .. } => assert_eq!(relic_diffs.len(), 1),
            ShopMerchandise::RevivalCatalyst { .. } => assert_eq!(inventory_diff.material_stacks.len(), 1),
            ShopMerchandise::RelicCorruptionCatalyst => assert_eq!(crafting_resources.unwrap().relic_corruption_catalysts, 1),
            _ => panic!("fixture goods"),
        }
        let after = roots(&core);
        assert!(core.execute(player, PlayerBehavior::PurchaseItem { offer_id: offer.offer_id }).is_err());
        assert_eq!(roots(&core), after, "a receipt cannot grant twice");
    }
    assert_eq!(core.state.enkephalin.amount, money - 240);
    assert_eq!(core.state.relics.copies().count(), 1);
    assert_eq!(core.state.inventory.equipment_materials.amount("revival_catalyst"), 1);
    assert_eq!(core.state.crafting_resources.relic_corruption_catalysts, 1);
    core.execute(player, PlayerBehavior::RefreshShop).unwrap();
    let relic = offers(&core).into_iter().find(|o| matches!(o.merchandise, ShopMerchandise::Relic {..})).unwrap();
    buy(&mut core, player, relic.offer_id);
    assert_eq!(core.state.relics.copies().count(), 2, "equip duplicates policy does not cap inventory");
}

#[test]
fn u16_shortage_overflow_and_unavailable_sale_preserve_all_roots() {
    let (mut core, player) = setup(17);
    core.state.enkephalin.amount = 0;
    for offer in offers(&core) {
        assert_eq!(core.shop_purchase_eligibility(&offer).disabled_reason_code, Some(ActionDisabledReasonCode::InsufficientResources));
        let before = roots(&core);
        assert!(core.execute(player, PlayerBehavior::PurchaseItem { offer_id: offer.offer_id }).is_err());
        assert_eq!(roots(&core), before);
    }
    core.state.enkephalin.amount = 1000;
    core.state.crafting_resources.relic_corruption_catalysts = u32::MAX;
    let offer = offers(&core).into_iter().find(|o| matches!(o.merchandise, ShopMerchandise::RelicCorruptionCatalyst)).unwrap();
    let before = roots(&core);
    assert_eq!(core.shop_purchase_eligibility(&offer).disabled_reason_code, Some(ActionDisabledReasonCode::InvalidQuantity));
    assert!(core.execute(player, PlayerBehavior::PurchaseItem { offer_id: offer.offer_id }).is_err());
    assert_eq!(roots(&core), before);
    let relic = offers(&core).into_iter().find(|o| matches!(o.merchandise, ShopMerchandise::Relic {..})).unwrap();
    buy(&mut core, player, relic.offer_id);
    let id = core.state.relics.copies().next().unwrap().copy_uuid;
    let before = roots(&core);
    assert!(core.execute(player, PlayerBehavior::SellItem { item_uuid: id, quantity: 1 }).is_err());
    assert_eq!(roots(&core), before);
}

#[test]
fn u16_completed_shop_continue_keeps_goods_unfinished_continue_rewinds_all_goods() {
    for complete in [false, true] {
        let (mut core, player) = setup(18);
        let money = core.state.enkephalin.amount;
        for offer in offers(&core) { buy(&mut core, player, offer.offer_id); }
        if complete { core.execute(player, PlayerBehavior::ExitShop).unwrap(); }
        let saved = core.export_active_run_checkpoint().unwrap().unwrap();
        let mut restarted = core_with_profile_for_continue(&core, 999);
        restarted.continue_from_active_run_checkpoint(player, &saved.core_payload_json, saved.loads_used, saved.run_peak_certified_power).unwrap();
        let count = u32::from(complete);
        assert_eq!(restarted.state.relics.copies().count(), count as usize);
        assert_eq!(restarted.state.inventory.equipment_materials.amount("revival_catalyst"), count);
        assert_eq!(restarted.state.crafting_resources.relic_corruption_catalysts, count);
        assert_eq!(restarted.state.enkephalin.amount, money - count * 240);
    }
}

#[test]
fn u16_basic_revival_offer_is_guaranteed_and_nonrepeatable_after_purchase() {
    let data = live_game_data_from_ron();
    for seed in 0..24 {
        let mut core = GameCore::new(data.clone(), seed);
        let player = Uuid::from_u128(16);
        start_new_game_with_default_starters(&mut core, player);
        core.apply_grant_effects(&[RewardEffect::GrantEnkephalin { amount: 2000 }]).unwrap();
        let shop = data.shop_data.shops.iter().find(|s| s.id == "unknown_exploration_shop").unwrap();
        let node = force_first_available_node(&mut core, MapNodeCategory::Shop, "shop_general", MapNodePayload::Shop { shop_id: Some(shop.id.clone()), shop_pool_id: None });
        select_and_confirm_map_node(&mut core, player, node);
        let offer = offers(&core).into_iter().find(|o| matches!(o.merchandise, ShopMerchandise::RevivalCatalyst {..})).expect("every initial page has a revival catalyst");
        assert_eq!((offer.quantity, offer.total_price, offer.repeatable), (1, 150, false));
        buy(&mut core, player, offer.offer_id);
        core.execute(player, PlayerBehavior::RefreshShop).unwrap();
        assert!(offers(&core).iter().all(|o| !matches!(o.merchandise, ShopMerchandise::RevivalCatalyst {..})));
    }
}

#[test]
fn u16_immediate_treatment_replaces_modifier_and_consumes_exactly_one() {
    let tonic = consumable_meta(0x1610, "u16_tonic", ConsumableTier::Common, ConsumableEffect::TraumaMitigation { percent: 10 });
    let surgery = consumable_meta(0x1611, "u16_surgery", ConsumableTier::Common, ConsumableEffect::ReducePhysicalDamage { amount: 2 });
    let mut core = GameCore::new(game_data_with_consumables(vec![tonic.clone(), surgery.clone()]), 16);
    let player = Uuid::from_u128(16);
    start_new_game_with_default_starters(&mut core, player);
    let owner = core.state.roster.combat_eligible_employee_ids()[0];
    core.state.roster.get_mut(&owner).unwrap().condition.physical_damage = 5;
    core.state.inventory.add_consumable(Arc::new(tonic.clone()), 1).unwrap();
    core.state.inventory.add_consumable(Arc::new(surgery.clone()), 2).unwrap();
    core.execute(player, PlayerBehavior::UseConsumableItem { item_uuid: tonic.uuid, target_employee_uuid: owner }).unwrap();
    let result = core.execute(player, PlayerBehavior::UseConsumableItem { item_uuid: surgery.uuid, target_employee_uuid: owner }).unwrap();
    let BehaviorResult::ConsumableItemUsed { replaced_modifier, active_modifier, condition, .. } = result else { panic!("treatment"); };
    assert_eq!(replaced_modifier.unwrap().definition_id, "u16_tonic");
    assert!(active_modifier.is_none());
    assert_eq!(condition.physical_damage, 3);
    assert_eq!(core.state.inventory.consumables.get_item(&surgery.uuid).unwrap().quantity, 1);
}

#[test]
fn u16_inventory_display_size_does_not_limit_relic_purchase() {
    let (mut core, player) = setup(19);
    for _ in 0..64 {
        core.apply_grant_effects(&[RewardEffect::GrantRelic { relic_id: "u08_probe".into() }]).unwrap();
    }
    let offer = offers(&core).into_iter().find(|o| matches!(o.merchandise, ShopMerchandise::Relic {..})).unwrap();
    buy(&mut core, player, offer.offer_id);
    assert_eq!(core.state.relics.copies().count(), 65);
}

#[test]
fn u16_expanded_carried_equipment_and_shop_goods_continue_with_default_budget() {
    let (mut core, player) = setup(20);
    for _ in 0..64 {
        core.apply_grant_effects(&[RewardEffect::GrantEquipment {
            equipment_id: "standard_training_sword".into(),
        }]).unwrap();
    }
    for offer in offers(&core) { buy(&mut core, player, offer.offer_id); }
    core.execute(player, PlayerBehavior::ExitShop).unwrap();
    let expected = serde_json::to_value(&core.state.inventory).unwrap();
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restarted = core_with_profile_for_continue(&core, 999);
    crate::game::battle::runtime_control::run(Default::default(), || {
        restarted.continue_from_active_run_checkpoint(player, &saved.core_payload_json,
            saved.loads_used, saved.run_peak_certified_power)
    }).unwrap();
    assert_eq!(serde_json::to_value(&restarted.state.inventory).unwrap(), expected);
    assert_eq!(restarted.state.relics.copies().count(), 1);
    assert_eq!(restarted.state.crafting_resources.relic_corruption_catalysts, 1);
}

#[test]
fn u16_hq_terms_match_price_loadout_and_remaining_hires() {
    let mut core = GameCore::new(game_data_with_map_content(), 12401);
    let player = Uuid::from_u128(16);
    start_new_game_with_default_starters(&mut core, player);
    let node = force_first_available_node(&mut core, MapNodeCategory::HeadquartersContact, "headquarters_contact", MapNodePayload::HeadquartersContact { shop_pool_id: None, candidate_count: 6 });
    select_and_confirm_map_node(&mut core, player, node);
    for remaining in [2, 1] {
        let before = roots(&core);
        let query = core.get_selected_event_snapshot_json().unwrap().unwrap();
        assert_eq!(query["recruitment_terms"]["recruit_price"], 100);
        assert_eq!(query["recruitment_terms"]["includes_basic_weapon_and_fragment"], true);
        assert_eq!(query["recruitment_terms"]["remaining_recruits"], remaining);
        assert_eq!(query["recruitment_terms"]["can_recruit"], true);
        assert_eq!(roots(&core), before);
        let id = query["recruitment_candidates"][0]["employee_definition_id"].as_str().unwrap().to_string();
        core.execute(player, PlayerBehavior::RecruitEmployee { employee_definition_id: id }).unwrap();
    }
    let query = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(query["recruitment_terms"]["remaining_recruits"], 0);
    assert_eq!(query["recruitment_terms"]["can_recruit"], false);
}
