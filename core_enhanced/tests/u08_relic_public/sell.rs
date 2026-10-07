use super::support::*;
use serde_json::json;
use uuid::Uuid;

#[test]
fn u08_public_shop_rejects_all_relic_variants_and_loadouts_with_a_specific_reason() {
    let (mut app, employees) = app();
    let copies: Vec<_> = (0..4).map(|_| claim_relic_reward(&mut app)).collect();
    equip_relic(&mut app, copies[0], employees[0], 0);
    equip_relic(&mut app, copies[2], employees[1], 2);
    admin(
        &mut app,
        json!({"type":"admin_grant_relic_corruption_catalyst", "amount":2}),
    );
    for copy in &copies[2..] {
        command(&mut app, json!({"type":"corrupt_relic", "copy_uuid":copy}));
    }
    admin(
        &mut app,
        json!({"type":"admin_enter_shop", "shop_id":"unknown_exploration_shop"}),
    );
    assert_eq!(selected_event(&app)["selling_enabled"], true);
    let state = command(&mut app, json!({"type":"request_relic_state"}));
    let mut sorted = copies.clone();
    sorted.sort();
    assert_eq!(state["not_sellable_copy_uuids"], json!(sorted));
    let zero_before = public_roots(&app);
    let zero = execute(
        &mut app,
        json!({"type":"sell_item", "item_uuid":copies[0], "quantity":0}),
    )
    .unwrap_err();
    assert_eq!(zero.code(), "invalid_quantity");
    assert_eq!(public_roots(&app), zero_before);
    for copy in copies {
        let before = public_roots(&app);
        let error = execute(
            &mut app,
            json!({"type":"sell_item", "item_uuid":copy, "quantity":1}),
        )
        .unwrap_err();
        assert_eq!(error.code(), "relic_not_sellable");
        assert_eq!(
            public_roots(&app),
            before,
            "a relic sale must preserve copies, placement, currency and persistence"
        );
    }
    let before = public_roots(&app);
    let unknown = execute(
        &mut app,
        json!({"type":"sell_item", "item_uuid":Uuid::nil(), "quantity":1}),
    )
    .unwrap_err();
    assert_eq!(unknown.code(), "offer_unavailable");
    assert_eq!(public_roots(&app), before);
}

#[test]
fn u08_public_relic_does_not_bypass_the_existing_disabled_shop_sale_policy() {
    let (mut app, _) = app();
    let copy = claim_relic_reward(&mut app);
    let outside = public_roots(&app);
    let error = execute(
        &mut app,
        json!({"type":"sell_item", "item_uuid":copy, "quantity":1}),
    )
    .unwrap_err();
    assert_eq!(error.code(), "not_in_shop_state");
    assert_eq!(public_roots(&app), outside);
    admin(
        &mut app,
        json!({"type":"admin_enter_shop", "shop_id":"u08_public_no_sales"}),
    );
    assert_eq!(selected_event(&app)["selling_enabled"], false);
    let before = public_roots(&app);
    let error = execute(
        &mut app,
        json!({"type":"sell_item", "item_uuid":copy, "quantity":1}),
    )
    .unwrap_err();
    assert_eq!(error.code(), "selling_unavailable");
    assert_eq!(public_roots(&app), before);
}

#[test]
fn u08_public_empty_relic_state_omits_the_derived_unsellable_list() {
    let (mut app, _) = app();
    let state = command(&mut app, json!({"type":"request_relic_state"}));
    assert!(state.get("not_sellable_copy_uuids").is_none());
}
