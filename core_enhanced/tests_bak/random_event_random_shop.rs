use std::sync::Arc;

use game_core::game::data::random_event_data::RandomEventTarget;
use game_core::game::data::GameDataBase;
use game_core::game::events::event_selection::random::RandomEventType;

mod common;

use common::load_game_data_from_ron;

/// 랜덤 이벤트가 전용 상점 RON 을 통해 정의된 상점을
/// 정상적으로 참조하는지 검증하는 테스트
#[test]
fn random_event_shop_inner_metadata_resolves_to_random_shop() {
    // Given: 실제 RON 기반 GameDataBase 로드
    let game_data: Arc<GameDataBase> = load_game_data_from_ron();

    // When: wandering_merchant 랜덤 이벤트 메타데이터 조회
    let event = game_data
        .random_event_data
        .get_by_id("wandering_merchant")
        .expect("wandering_merchant random event must exist");

    // Then: wandering_merchant는 Shop 타입이어야 함
    assert!(
        matches!(event.event_type, RandomEventType::Shop),
        "wandering_merchant should be a Shop-type random event"
    );

    // When: inner_metadata 로 실제 상점 메타데이터를 해석
    let target = event
        .inner_metadata
        .resolve(&game_data)
        .expect("inner_metadata resolution must succeed");

    // Then: RandomEventTarget::Shop 으로 해석되어야 함
    // Then: 해당 상점은 random_shops.ron 에 정의된 상점이어야 함
    match target {
        RandomEventTarget::Shop(shop) => {
            assert_eq!(shop.id, "random_event_shop_1");
            assert_eq!(
                shop.name, "수상한 떠돌이 상인",
                "Random shop name should match random_shops.ron"
            );
        }
        other => panic!("Expected RandomEventTarget::Shop, got {:?}", other),
    }
}

/// 랜덤 이벤트가 전용 보너스 RON 을 통해 정의된 보너스를
/// 정상적으로 참조하는지 검증하는 테스트
#[test]
fn random_event_bonus_inner_metadata_resolves_to_random_bonus() {
    // Given: 실제 RON 기반 GameDataBase 로드
    let game_data: Arc<GameDataBase> = load_game_data_from_ron();

    // When: suspicious_box 랜덤 이벤트 메타데이터 조회
    let event = game_data
        .random_event_data
        .get_by_id("suspicious_box")
        .expect("suspicious_box random event must exist");

    // Then: suspicious_box는 Bonus 타입이어야 함
    assert!(
        matches!(event.event_type, RandomEventType::Bonus),
        "suspicious_box should be a Bonus-type random event"
    );

    // When: inner_metadata 로 실제 보너스 메타데이터를 해석
    let target = event
        .inner_metadata
        .resolve(&game_data)
        .expect("inner_metadata resolution must succeed");

    // Then: RandomEventTarget::Bonus 으로 해석되어야 함
    match target {
        RandomEventTarget::Bonus(bonus) => {
            assert_eq!(bonus.id, "random_event_bonus_1");
        }
        other => panic!("Expected RandomEventTarget::Bonus, got {:?}", other),
    }
}

/// 랜덤 이벤트가 전용 Suppress RON 을 통해 정의된 기물을
/// 정상적으로 참조하는지 검증하는 테스트
#[test]
fn random_event_suppress_inner_metadata_resolves_to_random_abnormality() {
    // Given: 실제 RON 기반 GameDataBase 로드
    let game_data: Arc<GameDataBase> = load_game_data_from_ron();

    // When: cursed_fountain 랜덤 이벤트 메타데이터 조회
    let event = game_data
        .random_event_data
        .get_by_id("cursed_fountain")
        .expect("cursed_fountain random event must exist");

    // Then: cursed_fountain은 Suppress 타입이어야 함
    assert!(
        matches!(event.event_type, RandomEventType::Suppress),
        "cursed_fountain should be a Suppress-type random event"
    );

    // When: inner_metadata 로 실제 기물 메타데이터를 해석
    let target = event
        .inner_metadata
        .resolve(&game_data)
        .expect("inner_metadata resolution must succeed");

    // Then: RandomEventTarget::Suppress 으로 해석되어야 함
    match target {
        RandomEventTarget::Suppress(abno) => {
            assert_eq!(abno.id, "random_event_abnormality_1");
        }
        other => panic!("Expected RandomEventTarget::Suppress, got {:?}", other),
    }
}
