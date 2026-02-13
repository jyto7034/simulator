use game_core::ecs::resources::GameState;
use game_core::ecs::resources::InventoryItemDto;
use game_core::game::behavior::{BehaviorResult, GameError, PlayerBehavior};
use game_core::game::enums::{GameOption, PhaseEvent};
use game_core::game::world::GameCore;
use std::collections::HashSet;
use uuid::Uuid;

use crate::common::create_test_game_data;

fn contains_behavior_variant(haystack: &[PlayerBehavior], needle: &PlayerBehavior) -> bool {
    // NOTE: enum의 "variant"만 비교(필드 값 무시)하며, ActionValidator 정책과 맞춘다.
    let needle = std::mem::discriminant(needle);
    haystack.iter().any(|b| std::mem::discriminant(b) == needle)
}

fn start_game_and_request_phase(game: &mut GameCore, player_id: Uuid) -> PhaseEvent {
    game.execute(player_id, PlayerBehavior::StartNewGame)
        .unwrap();
    match game
        .execute(player_id, PlayerBehavior::RequestPhaseData)
        .unwrap()
    {
        BehaviorResult::RequestPhaseData(event) => event,
        other => panic!("expected RequestPhaseData, got {other:?}"),
    }
}

fn request_phase(game: &mut GameCore, player_id: Uuid) -> PhaseEvent {
    match game
        .execute(player_id, PlayerBehavior::RequestPhaseData)
        .unwrap()
    {
        BehaviorResult::RequestPhaseData(event) => event,
        other => panic!("expected RequestPhaseData, got {other:?}"),
    }
}

fn select_shop_from_phase_event(
    game: &mut GameCore,
    player_id: Uuid,
    phase_event: &PhaseEvent,
) -> Uuid {
    let shop_uuid = phase_event
        .options()
        .iter()
        .find_map(|opt| match opt {
            GameOption::Shop { shop } => Some(shop.uuid),
            _ => None,
        })
        .expect("Shop option should exist");

    let result = game.execute(
        player_id,
        PlayerBehavior::SelectEvent {
            event_id: shop_uuid,
        },
    );
    assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));
    shop_uuid
}

fn find_shop_visible_item_uuid<F>(phase_event: &PhaseEvent, predicate: F) -> Uuid
where
    F: Fn(Uuid) -> bool,
{
    let (shop, _bonus, _random) = phase_event
        .as_event_selection()
        .expect("EventSelection phase expected");

    shop.visible_items
        .iter()
        .copied()
        .find(|uuid| predicate(*uuid))
        .expect("expected a matching visible item")
}

#[test]
fn select_shop_option_transitions_and_allows_shop_actions() {
    // Given: 결정적인 테스트 데이터로 새 게임을 생성한다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    // When: 게임을 시작하고 Phase 데이터를 요청한다.
    let phase_event = start_game_and_request_phase(&mut game, player_id);
    // When: Phase 선택지에서 상점을 선택한다.
    let shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    // Then: 게임 상태는 선택한 상점 UUID로 InShop이 된다.
    match game.get_state() {
        GameState::InShop { shop_uuid: uuid } => assert_eq!(uuid, shop_uuid),
        other => panic!("expected InShop state, got {other:?}"),
    }

    // Then: 허용 행동 목록에 상점 행동들이 포함된다(variant 기준).
    let allowed_actions = game.get_allowed_actions();
    assert!(contains_behavior_variant(
        &allowed_actions,
        &PlayerBehavior::PurchaseItem {
            item_uuid: Uuid::nil()
        }
    ));
    assert!(contains_behavior_variant(
        &allowed_actions,
        &PlayerBehavior::SellItem {
            item_uuid: Uuid::nil()
        }
    ));
    assert!(contains_behavior_variant(
        &allowed_actions,
        &PlayerBehavior::RerollShop
    ));
    assert!(contains_behavior_variant(
        &allowed_actions,
        &PlayerBehavior::ExitShop
    ));
    assert!(!contains_behavior_variant(
        &allowed_actions,
        &PlayerBehavior::SelectEvent {
            event_id: Uuid::nil()
        }
    ));

    // Then: 이벤트 1개를 선택했으므로 Phase 선택지는 비워진다.
    assert_eq!(game.get_phase_events_count(), 0);
}

#[test]
fn select_event_before_phase_data_is_rejected() {
    // Given: 게임은 시작했지만 Phase 데이터는 아직 요청하지 않았다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    // When: 게임을 시작한다.
    game.execute(player_id, PlayerBehavior::StartNewGame)
        .unwrap();
    // When: RequestPhaseData 이전에 SelectEvent를 시도한다.
    let err = game
        .execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: Uuid::from_u128(1),
            },
        )
        .unwrap_err();
    // Then: ActionValidator / GameState 게이팅에 의해 거부된다.
    assert!(matches!(err, GameError::InvalidAction));
}

#[test]
fn select_unknown_event_id_is_rejected() {
    // Given: Phase 데이터를 요청했고, 선택지가 존재한다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    let _ = start_game_and_request_phase(&mut game, player_id);
    // When: CurrentPhaseEvents에 없는 event_id를 선택한다.
    let err = game
        .execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: Uuid::from_u128(999_999),
            },
        )
        .unwrap_err();
    // Then: 존재하지 않는 이벤트 ID는 거부된다.
    assert!(matches!(err, GameError::EventNotFound));
}

#[test]
fn select_event_twice_is_rejected_after_entering_shop() {
    // Given: 상점 이벤트를 한 번 선택해 이미 상점에 진입했다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    // When: 상점 상태에서 다시 SelectEvent를 시도한다.
    let err = game
        .execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: Uuid::from_u128(2),
            },
        )
        .unwrap_err();
    // Then: 현재 GameState 게이팅에 의해 거부된다.
    assert!(matches!(err, GameError::InvalidAction));
}

#[test]
fn reroll_shop_swaps_visible_to_hidden_and_is_one_shot() {
    // Given: visible/hidden 아이템이 모두 있는 결정적 상점을 준비한다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data.clone(), 12345);
    let player_id = Uuid::new_v4();

    // When: 게임을 시작하고 Phase 이벤트 스냅샷을 받는다.
    let phase_event = start_game_and_request_phase(&mut game, player_id);

    let shop_metadata = match &phase_event {
        PhaseEvent::EventSelection { shop, .. } => shop,
        _ => unreachable!("EventSelection phase expected"),
    };

    // Given: Phase 스냅샷에서 초기 visible/hidden 목록을 캡처한다.
    let shop_uuid = shop_metadata.uuid;
    let first_visible: Vec<Uuid> = shop_metadata.visible_items.clone();
    let first_hidden: Vec<Uuid> = shop_metadata.hidden_items.clone();
    assert!(!first_hidden.is_empty());

    // When: 상점에 진입한다.
    let _ = game.execute(
        player_id,
        PlayerBehavior::SelectEvent {
            event_id: shop_uuid,
        },
    );

    // When: 리롤을 1회 수행한다.
    let reroll = game.execute(player_id, PlayerBehavior::RerollShop).unwrap();
    let BehaviorResult::RerollShop { new_items } = reroll else {
        panic!("expected RerollShop");
    };

    // Then: 첫 리롤 결과 visible은 (순서 무관하게) 기존 hidden과 동일해야 한다.
    let new_set: HashSet<Uuid> = new_items.iter().copied().collect();
    let expected_set: HashSet<Uuid> = first_hidden.iter().copied().collect();
    assert_eq!(new_set, expected_set);
    assert_ne!(
        new_set,
        first_visible.iter().copied().collect::<HashSet<_>>()
    );

    // When: 리롤을 한 번 더 수행한다(2회차).
    let err = game
        .execute(player_id, PlayerBehavior::RerollShop)
        .unwrap_err();
    // Then: 리롤은 1회 제한이므로 거부된다.
    assert!(matches!(err, GameError::ShopRerollNotAllowed));
}

#[test]
fn purchase_item_rejects_hidden_item_uuid() {
    // Given: 상점에 진입했고, Phase 스냅샷에서 hidden 아이템 UUID를 알고 있다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data.clone(), 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    let shop_metadata = match &phase_event {
        PhaseEvent::EventSelection { shop, .. } => shop,
        _ => unreachable!("EventSelection phase expected"),
    };
    assert_eq!(shop_metadata.uuid, shop_uuid);

    // When: 현재 visible이 아닌(hidden) 아이템 구매를 시도한다.
    let hidden_item_uuid = *shop_metadata.hidden_items.first().expect("has hidden");
    let err = game
        .execute(
            player_id,
            PlayerBehavior::PurchaseItem {
                item_uuid: hidden_item_uuid,
            },
        )
        .unwrap_err();
    // Then: visible_items에 없으므로 구매가 거부된다.
    assert!(matches!(err, GameError::ShopItemNotFound));
}

#[test]
fn purchase_item_rejects_insufficient_resources() {
    // Given: 상점에 진입했고, visible 아이템 UUID를 알고 있다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data.clone(), 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    let shop_metadata = match &phase_event {
        PhaseEvent::EventSelection { shop, .. } => shop,
        _ => unreachable!("EventSelection phase expected"),
    };
    let visible_item_uuid = *shop_metadata.visible_items.first().expect("has visible");

    // When: Enkephalin을 가격 미만으로 만들고 구매를 시도한다.
    game.set_enkephalin(0);
    let err = game
        .execute(
            player_id,
            PlayerBehavior::PurchaseItem {
                item_uuid: visible_item_uuid,
            },
        )
        .unwrap_err();
    // Then: 자원 부족으로 구매가 거부된다.
    assert!(matches!(err, GameError::InsufficientResources));
}

#[test]
fn purchase_item_removes_from_shop_and_returns_inventory_diff() {
    // Given: 상점에 진입했고, 가격을 알 수 있는 visible 아이템을 고른다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data.clone(), 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    let shop_metadata = match &phase_event {
        PhaseEvent::EventSelection { shop, .. } => shop,
        _ => unreachable!("EventSelection phase expected"),
    };
    let (item_uuid, item_price) = shop_metadata
        .visible_items
        .iter()
        .find_map(|uuid| game_data.item(uuid).map(|item| (*uuid, item.price())))
        .expect("visible item exists in registry");

    // When: Enkephalin을 정확히 가격만큼 맞추고 구매한다.
    game.set_enkephalin(item_price);

    let result = game
        .execute(player_id, PlayerBehavior::PurchaseItem { item_uuid })
        .unwrap();
    // Then: Enkephalin이 정확히 차감되고, inventory_diff.added가 1개여야 한다.
    let (remaining_enkephalin, inventory_diff) = result.as_purchase_item().unwrap();
    assert_eq!(remaining_enkephalin, 0);
    assert_eq!(inventory_diff.added.len(), 1);

    // When: 같은 아이템을 다시 구매 시도한다.
    // Purchasing the same shop item again must fail (no longer visible).
    let err = game
        .execute(player_id, PlayerBehavior::PurchaseItem { item_uuid })
        .unwrap_err();
    // Then: 상점 visible_items에서 제거되었으므로 구매가 거부된다.
    assert!(matches!(err, GameError::ShopItemNotFound));
}

#[test]
fn purchase_after_reroll_can_buy_abnormality_and_uses_distinct_owned_uuid() {
    // Given: 상점 진입 후 리롤해서 환상체가 visible에 나오게 만든다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data.clone(), 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    let reroll = game.execute(player_id, PlayerBehavior::RerollShop).unwrap();
    let BehaviorResult::RerollShop { new_items } = reroll else {
        panic!("expected RerollShop");
    };

    // When: 리롤된 visible 목록에서 환상체를 찾는다.
    let abno_uuid = new_items
        .iter()
        .copied()
        .find(|uuid| {
            matches!(
                game_data.item(uuid),
                Some(game_core::game::data::Item::Abnormality(_))
            )
        })
        .expect("rerolled shop should include an abnormality");

    let price = game_data.item(&abno_uuid).unwrap().price();
    // When: Enkephalin을 정확히 맞추고 구매한다.
    game.set_enkephalin(price);

    let result = game
        .execute(
            player_id,
            PlayerBehavior::PurchaseItem {
                item_uuid: abno_uuid,
            },
        )
        .unwrap();
    let (_remaining, inventory_diff) = result.as_purchase_item().unwrap();
    let owned_uuid = inventory_diff.added[0].uuid();
    // Then: 환상체는 base_uuid가 아니라 별도 owned_uuid로 소유되어야 한다.
    assert_ne!(
        owned_uuid, abno_uuid,
        "abnormality should be owned by a distinct instance uuid"
    );
}

#[test]
fn exit_shop_advances_phase_and_disallows_shop_actions() {
    // Given: 상점에 진입한 상태
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    // When: 상점에서 나가기(ExitShop)를 수행한다.
    let result = game.execute(player_id, PlayerBehavior::ExitShop).unwrap();

    // Then: Phase가 진행되며 AdvancePhase 결과를 반환한다.
    assert!(matches!(result, BehaviorResult::AdvancePhase { .. }));

    // Then: 상점 컨텍스트는 종료되고, 상태는 WaitingPhaseRequest로 돌아간다.
    assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));

    // Then: 선택지/선택 이벤트는 폐기된다(현재 Phase 이벤트는 0).
    assert_eq!(game.get_phase_events_count(), 0);

    // Then: 상점 행동은 더 이상 허용되지 않는다(게이팅).
    let err = game
        .execute(player_id, PlayerBehavior::RerollShop)
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidAction));
}

#[test]
fn purchase_equipment_then_sell_updates_enkephalin_and_prevents_double_sell() {
    // Given: 상점에 진입했고, visible에서 장비 아이템을 선택한다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data.clone(), 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    let equipment_base_uuid = find_shop_visible_item_uuid(&phase_event, |uuid| {
        matches!(
            game_data.item(&uuid),
            Some(game_core::game::data::Item::Equipment(_))
        )
    });

    let price = game_data
        .item(&equipment_base_uuid)
        .expect("equipment exists")
        .price();

    // When: Enkephalin을 정확히 가격만큼 맞추고 구매한다.
    game.set_enkephalin(price);
    let purchase = game
        .execute(
            player_id,
            PlayerBehavior::PurchaseItem {
                item_uuid: equipment_base_uuid,
            },
        )
        .unwrap();

    // Then: 구매 결과로 owned_uuid(인벤토리 인스턴스)가 반환된다.
    let (remaining, diff) = purchase.as_purchase_item().unwrap();
    assert_eq!(remaining, 0);
    assert_eq!(diff.added.len(), 1);

    let owned_uuid = diff.added[0].uuid();
    assert_ne!(owned_uuid, Uuid::nil());
    assert!(matches!(diff.added[0], InventoryItemDto::Equipment(_)));

    // When: 방금 산 아이템을 즉시 판매한다.
    let sell = game
        .execute(
            player_id,
            PlayerBehavior::SellItem {
                item_uuid: owned_uuid,
            },
        )
        .unwrap();

    // Then: 판매 가격은 원가의 50%이며, removed에 해당 owned_uuid가 포함된다.
    let (remaining, diff) = sell.as_sell_item().unwrap();
    assert_eq!(remaining, price / 2);
    assert_eq!(diff.removed, vec![owned_uuid]);

    // When: 같은 아이템을 다시 판매하려고 시도한다.
    let err = game
        .execute(
            player_id,
            PlayerBehavior::SellItem {
                item_uuid: owned_uuid,
            },
        )
        .unwrap_err();

    // Then: 인벤토리에 없으므로 거부된다.
    assert!(matches!(err, GameError::InventoryItemNotFound));
}

// TODO: 검토 필요함
#[test]
fn equipped_item_cannot_be_sold_when_reentering_shop() {
    // Given: 상점에서 환상체+장비를 구매하고, 상점 밖(WaitingPhaseRequest)에서 장비를 장착한다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data.clone(), 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    // When: 리롤하여 환상체가 visible에 나오게 한다.
    let reroll = game.execute(player_id, PlayerBehavior::RerollShop).unwrap();
    let BehaviorResult::RerollShop { new_items } = reroll else {
        panic!("expected RerollShop");
    };

    let abno_base_uuid = new_items
        .iter()
        .copied()
        .find(|uuid| {
            matches!(
                game_data.item(uuid),
                Some(game_core::game::data::Item::Abnormality(_))
            )
        })
        .expect("rerolled shop should include an abnormality");
    let equip_base_uuid = new_items
        .iter()
        .copied()
        .find(|uuid| {
            matches!(
                game_data.item(uuid),
                Some(game_core::game::data::Item::Equipment(_))
            )
        })
        .expect("rerolled shop should include an equipment");

    let abno_price = game_data.item(&abno_base_uuid).unwrap().price();
    let equip_price = game_data.item(&equip_base_uuid).unwrap().price();

    // When: 두 아이템을 모두 구매할 수 있게 Enkephalin을 설정하고 구매한다.
    game.set_enkephalin(abno_price + equip_price);

    let purchase_abno = game
        .execute(
            player_id,
            PlayerBehavior::PurchaseItem {
                item_uuid: abno_base_uuid,
            },
        )
        .unwrap();
    let (_remaining, diff) = purchase_abno.as_purchase_item().unwrap();
    let owned_abno_uuid = diff.added[0].uuid();

    let purchase_equip = game
        .execute(
            player_id,
            PlayerBehavior::PurchaseItem {
                item_uuid: equip_base_uuid,
            },
        )
        .unwrap();
    let (_remaining, diff) = purchase_equip.as_purchase_item().unwrap();
    let owned_equip_uuid = diff.added[0].uuid();

    // When: 상점에서 나가서(Phase 진행) 장착을 수행한다.
    let _ = game.execute(player_id, PlayerBehavior::ExitShop).unwrap();
    assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));

    game.execute(
        player_id,
        PlayerBehavior::EquipItem {
            item_uuid: owned_equip_uuid,
            target_unit: owned_abno_uuid,
        },
    )
    .unwrap();

    // When: 다음 Phase에서 다시 상점에 진입한다.
    let phase_event = request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    // When: 장착된 아이템을 판매하려고 시도한다.
    let before = game.get_enkephalin();
    let err = game
        .execute(
            player_id,
            PlayerBehavior::SellItem {
                item_uuid: owned_equip_uuid,
            },
        )
        .unwrap_err();

    // Then: 장착 아이템은 판매 불가로 거부되며(InvalidAction), Enkephalin은 변하지 않는다.
    assert!(matches!(err, GameError::InvalidAction));
    assert_eq!(game.get_enkephalin(), before);
}

#[test]
fn shop_actions_are_rejected_outside_shop_state() {
    // Given: 게임을 시작했지만 상점에는 진입하지 않았다(WaitingPhaseRequest 상태).
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    game.execute(player_id, PlayerBehavior::StartNewGame)
        .unwrap();
    assert!(matches!(game.get_state(), GameState::WaitingPhaseRequest));

    // When: 상점 전용 행동들을 상점 밖에서 시도한다.
    let purchase_err = game
        .execute(
            player_id,
            PlayerBehavior::PurchaseItem {
                item_uuid: Uuid::from_u128(1),
            },
        )
        .unwrap_err();
    let sell_err = game
        .execute(
            player_id,
            PlayerBehavior::SellItem {
                item_uuid: Uuid::from_u128(2),
            },
        )
        .unwrap_err();
    let reroll_err = game
        .execute(player_id, PlayerBehavior::RerollShop)
        .unwrap_err();
    let exit_err = game
        .execute(player_id, PlayerBehavior::ExitShop)
        .unwrap_err();

    // Then: ActionValidator / GameState 게이팅에 의해 전부 거부된다.
    assert!(matches!(purchase_err, GameError::InvalidAction));
    assert!(matches!(sell_err, GameError::InvalidAction));
    assert!(matches!(reroll_err, GameError::InvalidAction));
    assert!(matches!(exit_err, GameError::InvalidAction));
}

#[test]
fn sell_item_not_in_inventory_is_rejected_in_shop() {
    // Given: 상점에 진입했지만, 특정 owned_uuid는 인벤토리에 존재하지 않는다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    // When: 인벤토리에 없는 UUID를 판매하려고 시도한다.
    let err = game
        .execute(
            player_id,
            PlayerBehavior::SellItem {
                item_uuid: Uuid::from_u128(999_999),
            },
        )
        .unwrap_err();

    // Then: 인벤토리에서 찾을 수 없으므로 거부된다.
    assert!(matches!(err, GameError::InventoryItemNotFound));
}

#[test]
fn reroll_makes_previous_visible_items_unpurchasable() {
    // Given: 상점에 진입했고, 초기 visible 목록을 알고 있다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data.clone(), 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let (shop, _bonus, _random) = phase_event
        .as_event_selection()
        .expect("EventSelection phase expected");
    let first_visible_uuid = *shop.visible_items.first().expect("has visible");

    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    // When: 리롤을 수행한다(1회차).
    let _ = game.execute(player_id, PlayerBehavior::RerollShop).unwrap();

    // When: 이제 hidden으로 내려간(이전 visible) 아이템을 구매하려고 시도한다.
    game.set_enkephalin(u32::MAX / 4);
    let err = game
        .execute(
            player_id,
            PlayerBehavior::PurchaseItem {
                item_uuid: first_visible_uuid,
            },
        )
        .unwrap_err();

    // Then: 현재 visible_items에 없으므로 구매가 거부된다.
    assert!(matches!(err, GameError::ShopItemNotFound));
}

#[test]
fn reroll_does_not_change_enkephalin() {
    // Given: 상점에 진입했고, 임의의 Enkephalin 값을 세팅한다.
    let game_data = create_test_game_data();
    let mut game = GameCore::new(game_data, 12345);
    let player_id = Uuid::new_v4();

    let phase_event = start_game_and_request_phase(&mut game, player_id);
    let _shop_uuid = select_shop_from_phase_event(&mut game, player_id, &phase_event);

    game.set_enkephalin(123);
    let before = game.get_enkephalin();

    // When: 리롤을 수행한다.
    let _ = game.execute(player_id, PlayerBehavior::RerollShop).unwrap();

    // Then: 리롤은 자원에 영향을 주지 않는다.
    assert_eq!(game.get_enkephalin(), before);
}
