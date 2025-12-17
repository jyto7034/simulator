mod common;

use common::create_test_game_data;
use game_core::game::behavior::{BehaviorResult, PlayerBehavior};
use game_core::game::enums::{GameOption, PhaseEvent};
use game_core::game::world::GameCore;
use uuid::Uuid;

// ============================================================
// Shop Tests
// ============================================================
#[cfg(test)]
mod shop {
    use super::*;
    use game_core::ecs::resources::GameState;

    /// 통합 테스트: 상점 선택 시나리오
    #[test]
    fn select_shop_option() {
        // Given: 게임 시작 및 Phase 데이터 받음
        let game_data = create_test_game_data();
        let mut game = GameCore::new(game_data.clone(), 12345);
        let player_id = Uuid::new_v4();

        game.execute(player_id, PlayerBehavior::StartNewGame)
            .unwrap();
        let result = game
            .execute(player_id, PlayerBehavior::RequestPhaseData)
            .unwrap();

        let phase_event = match result {
            BehaviorResult::RequestPhaseData(event) => event,
            _ => panic!("Expected RequestPhaseData result"),
        };

        // When: 상점 선택지 선택
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

        // Then: 상점 진입 성공
        assert!(result.is_ok());

        assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));

        // Then: 게임 상태가 InShop으로 변경됨
        use game_core::ecs::resources::GameState;
        match game.get_state() {
            GameState::InShop { shop_uuid: uuid } => {
                assert_eq!(uuid, shop_uuid);
            }
            _ => panic!("Expected InShop state"),
        }

        // Then: 허용된 행동 확인 (구매, 판매, 리롤, 나가기)
        let allowed_actions = game.get_allowed_actions();
        assert_eq!(allowed_actions.len(), 4);

        // Then: 한 Phase에서 이벤트는 1개만 선택되므로 남은 선택지는 폐기됨
        assert_eq!(game.get_phase_events_count(), 0);
    }

    /// 통합 테스트: 상점 Reroll
    #[test]
    fn reroll_shop() {
        // Given: 게임 시작 및 Phase 데이터 받음
        let game_data = create_test_game_data();
        let mut game = GameCore::new(game_data.clone(), 12345);
        let player_id = Uuid::new_v4();

        // When: 게임 시작
        let result = game.execute(player_id, PlayerBehavior::StartNewGame);

        // Then: 게임 시작 성공
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BehaviorResult::StartNewGame));

        // Then: 게임 상태가 WaitingPhaseRequest로 변경됨
        assert_eq!(game.get_state(), GameState::WaitingPhaseRequest);

        // When: Phase 데이터 요청
        let result = game.execute(player_id, PlayerBehavior::RequestPhaseData);

        // Then: Phase 데이터 수신 성공
        assert!(result.is_ok());

        let phase_event = match result.unwrap() {
            BehaviorResult::RequestPhaseData(event) => event,
            _ => panic!("Expected RequestPhaseData result"),
        };

        let shop_metadata = match &phase_event {
            PhaseEvent::EventSelection { shop, .. } => shop,
            _ => unreachable!("EventSelection phase expected"),
        };

        let shop_uuid = shop_metadata.uuid;

        let first_items: Vec<Uuid> = shop_metadata.visible_items.iter().cloned().collect();

        // When: 상점 이벤트 선택
        let result = game.execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: shop_uuid,
            },
        );

        // Then: 행동이 잘 수행되었는지 확인
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));

        // Then: 현재 game state 가 InShop 인지 확인
        match game.get_state() {
            GameState::InShop { shop_uuid: uuid } => {
                assert_eq!(uuid, shop_uuid);
            }
            _ => panic!("Expected InShop state"),
        }

        // Then: 허용된 행동 확인 (구매, 판매, 리롤, 나가기)
        let allowed_actions = game.get_allowed_actions();
        assert_eq!(allowed_actions.len(), 4);

        // Then: 한 Phase에서 이벤트는 1개만 선택되므로 남은 선택지는 폐기됨
        assert_eq!(game.get_phase_events_count(), 0);

        // When: Reroll 행동 수행
        let result = game.execute(player_id, PlayerBehavior::RerollShop);

        // Then: 행동 결과 검증
        assert!(result.is_ok());
        assert!(matches!(
            result.as_ref().unwrap(),
            BehaviorResult::RerollShop { .. }
        ));

        // Then: 처음 받은 아이템과 다른지 검증 (UUID 기준)
        if let Ok(BehaviorResult::RerollShop { new_items }) = result {
            let rerolled: Vec<Uuid> = new_items.clone();
            assert_ne!(rerolled, first_items, "Items must be different");
        }
    }

    #[test]
    fn test_purchase_item() {
        let game_data = create_test_game_data();
        let mut game = GameCore::new(game_data.clone(), 12345);
        let player_id = Uuid::new_v4();

        // When: 게임 시작
        let result = game.execute(player_id, PlayerBehavior::StartNewGame);

        // Then: 게임 시작 성공
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BehaviorResult::StartNewGame));

        // Then: 게임 상태가 WaitingPhaseRequest로 변경됨
        assert_eq!(game.get_state(), GameState::WaitingPhaseRequest);

        // When: Phase 데이터 요청
        let result = game.execute(player_id, PlayerBehavior::RequestPhaseData);

        // Then: Phase 데이터 수신 성공
        assert!(result.is_ok());

        let phase_event = match result.unwrap() {
            BehaviorResult::RequestPhaseData(event) => event,
            _ => panic!("Expected RequestPhaseData result"),
        };

        let shop_metadata = match &phase_event {
            PhaseEvent::EventSelection { shop, .. } => shop,
            _ => unreachable!("EventSelection phase expected"),
        };

        let shop_uuid = shop_metadata.uuid;

        // When: 상점 이벤트 선택
        let result = game.execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: shop_uuid,
            },
        );

        // Then: 행동이 잘 수행되었는지 확인
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));

        // Then: 현재 game state 가 InShop 인지 확인
        match game.get_state() {
            GameState::InShop { shop_uuid: uuid } => {
                assert_eq!(uuid, shop_uuid);
            }
            _ => panic!("Expected InShop state"),
        }

        // Then: 허용된 행동 확인 (구매, 판매, 리롤, 나가기)
        let allowed_actions = game.get_allowed_actions();
        assert_eq!(allowed_actions.len(), 4);

        // Then: 한 Phase에서 이벤트는 1개만 선택되므로 남은 선택지는 폐기됨
        assert_eq!(game.get_phase_events_count(), 0);

        // 테스트용으로 충분한 Enkephalin 지급
        game.set_enkephalin(10_000);

        let items_with_prices: Vec<(Uuid, u32)> = shop_metadata
            .visible_items
            .iter()
            .filter_map(|uuid| game_data.item(uuid).map(|item| (*uuid, item.price())))
            .collect();

        let mut expected_enkephalin = 10_000u32;

        for (item_uuid, item_price) in items_with_prices {
            // Given: 구매 전 엔케팔린 확인
            let enkephalin_before = game.get_enkephalin();
            assert_eq!(enkephalin_before, expected_enkephalin);

            // When: 아이템 구매
            let result = game.execute(player_id, PlayerBehavior::PurchaseItem { item_uuid });

            // Then: 구매 성공
            assert!(result.is_ok());
            assert!(matches!(
                result.as_ref().unwrap(),
                BehaviorResult::PurchaseItem { .. }
            ));

            // Then: 반환값 검증
            let res = result.unwrap();
            let (returned_enkephalin, inventory_diff) = res.as_purchase_item().unwrap();

            // 1. 반환된 엔케팔린이 예상값과 일치하는지
            expected_enkephalin -= item_price;
            assert_eq!(
                returned_enkephalin, expected_enkephalin,
                "구매 후 엔케팔린이 예상값과 다릅니다: expected={}, actual={}",
                expected_enkephalin, returned_enkephalin
            );

            // 2. 실제 게임 상태의 엔케팔린과 일치하는지
            let actual_enkephalin = game.get_enkephalin();
            assert_eq!(
                actual_enkephalin, returned_enkephalin,
                "반환된 엔케팔린과 게임 상태의 엔케팔린이 다릅니다"
            );

            // 3. inventory_diff.added에 구매한 아이템이 포함되어 있는지
            assert_eq!(
                inventory_diff.added.len(),
                1,
                "구매한 아이템이 1개여야 합니다"
            );
            assert_eq!(
                inventory_diff.added[0].uuid(),
                item_uuid,
                "구매한 아이템의 UUID가 일치해야 합니다"
            );

            // 4. inventory_diff.updated와 removed는 비어있어야 함
            assert!(
                inventory_diff.updated.is_empty(),
                "updated는 비어있어야 합니다"
            );
            assert!(
                inventory_diff.removed.is_empty(),
                "removed는 비어있어야 합니다"
            );
        }
    }
}

// ============================================================
// Bonus Tests
// ============================================================
#[cfg(test)]
mod bonus {
    use super::*;
    use game_core::ecs::resources::GameState;

    /// 통합 테스트: 보너스 선택 시나리오
    #[test]
    fn select_bonus_option() {
        // Given: 게임 시작 및 Phase 데이터 받음
        let game_data = create_test_game_data();
        let mut game = GameCore::new(game_data.clone(), 12345);
        let player_id = Uuid::new_v4();

        game.execute(player_id, PlayerBehavior::StartNewGame)
            .unwrap();

        // Given: 초기 Enkephalin 확인 (0이어야 함)
        let initial_enkephalin = game.get_enkephalin();
        assert_eq!(initial_enkephalin, 0);

        let result = game
            .execute(player_id, PlayerBehavior::RequestPhaseData)
            .unwrap();

        let phase_event = match result {
            BehaviorResult::RequestPhaseData(event) => event,
            _ => panic!("Expected RequestPhaseData result"),
        };

        // When: 보너스 선택지 선택
        let bonus_uuid = phase_event
            .options()
            .iter()
            .find_map(|opt| match opt {
                GameOption::Bonus { bonus } => Some(bonus.uuid),
                _ => None,
            })
            .expect("Bonus option should exist");

        let result = game.execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: bonus_uuid,
            },
        );

        // Then: 보너스 선택 성공 및 InBonus 상태 진입
        assert!(result.is_ok());

        assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));

        // Then: 게임 상태가 InBonus로 변경됨
        match game.get_state() {
            GameState::InBonus { bonus_uuid: uuid } => {
                assert_eq!(uuid, bonus_uuid);
            }
            _ => panic!("Expected InBonus state"),
        }

        // Then: 허용된 행동 확인 (ClaimBonus, ExitBonus)
        let allowed_actions = game.get_allowed_actions();
        assert_eq!(allowed_actions.len(), 2);
        assert!(game.is_action_allowed(&PlayerBehavior::ClaimBonus));
        assert!(game.is_action_allowed(&PlayerBehavior::ExitBonus));

        // 보너스 진입 시점에서는 아직 자원이 지급되지 않음
        let mid_enkephalin = game.get_enkephalin();
        assert_eq!(mid_enkephalin, initial_enkephalin);

        // When: 보너스 수령
        let result = game.execute(player_id, PlayerBehavior::ClaimBonus);

        // Then: 보너스 수령 성공 및 보상 확인
        assert!(result.is_ok());

        let (returned_enkephalin, inventory_diff) = match result.unwrap() {
            BehaviorResult::BonusReward {
                enkephalin,
                inventory_diff,
            } => (enkephalin, inventory_diff),
            other => panic!("Expected BonusReward result, got {:?}", other),
        };

        // Then: Enkephalin이 증가했는지 확인 (고정값 100)
        let final_enkephalin = game.get_enkephalin();
        assert_eq!(
            final_enkephalin, 100,
            "Enkephalin should be 100, but got {}",
            final_enkephalin
        );
        assert_eq!(final_enkephalin, returned_enkephalin);

        // 현재는 인벤토리 변화 없음 (향후 BonusType::Item/Abnormality 구현 시 확장)
        assert!(inventory_diff.added.is_empty());
        assert!(inventory_diff.updated.is_empty());
        assert!(inventory_diff.removed.is_empty());

        // Then: 보너스 수령 완료 상태로 전환됨 (Exit에서만 다음 Phase로 진행)
        assert_eq!(
            game.get_state(),
            GameState::InBonusClaimed { bonus_uuid }
        );

        // Then: Claim은 더 이상 허용되지 않고 Exit만 가능
        assert!(!game.is_action_allowed(&PlayerBehavior::ClaimBonus));
        assert!(game.is_action_allowed(&PlayerBehavior::ExitBonus));

        // Then: 한 Phase에서 이벤트는 1개만 선택되므로 남은 선택지는 폐기됨
        assert_eq!(game.get_phase_events_count(), 0);
    }
}

// ============================================================
// Random Event Tests
// ============================================================
#[cfg(test)]
mod random_event {
    use super::*;

    /// 통합 테스트: 랜덤 이벤트 선택 시나리오
    #[test]
    fn select_random_event_option() {
        // Given: 게임 시작 및 Phase 데이터 받음
        let game_data = create_test_game_data();
        let mut game = GameCore::new(game_data.clone(), 12345);
        let player_id = Uuid::new_v4();

        game.execute(player_id, PlayerBehavior::StartNewGame)
            .unwrap();
        let result = game
            .execute(player_id, PlayerBehavior::RequestPhaseData)
            .unwrap();

        let phase_event = match result {
            BehaviorResult::RequestPhaseData(event) => event,
            _ => panic!("Expected RequestPhaseData result"),
        };

        // When: 랜덤 이벤트 선택지 선택
        let event_uuid = phase_event
            .options()
            .iter()
            .find_map(|opt| match opt {
                GameOption::Random { event } => Some(event.uuid),
                _ => None,
            })
            .expect("Random event option should exist");

        // RandomEvent 가 라우팅할 대상 Bonus 의 uuid
        let bonus_uuid = phase_event
            .options()
            .iter()
            .find_map(|opt| match opt {
                GameOption::Bonus { bonus } => Some(bonus.uuid),
                _ => None,
            })
            .expect("Bonus option should exist");

        let result = game.execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: event_uuid,
            },
        );

        // Then: 랜덤 이벤트 선택 성공
        assert!(result.is_ok());

        assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));

        // Then: RandomEvent 가 Bonus 로 라우팅되어 InBonus 상태로 변경됨
        use game_core::ecs::resources::GameState;
        match game.get_state() {
            GameState::InBonus { bonus_uuid: uuid } => {
                assert_eq!(uuid, bonus_uuid);
            }
            _ => panic!("Expected InBonus state after random event selection"),
        }

        // Then: 허용된 행동 확인 (선택지 선택, 나가기)
        let allowed_actions = game.get_allowed_actions();
        assert_eq!(allowed_actions.len(), 2);

        // Then: 한 Phase에서 이벤트는 1개만 선택되므로 남은 선택지는 폐기됨
        assert_eq!(game.get_phase_events_count(), 0);
    }
}

// ============================================================
// Integration Tests
// ============================================================
#[cfg(test)]
mod integration {
    use super::*;

    /// 통합 테스트: Dawn Phase 1 - 이벤트 선택 전체 흐름
    ///
    /// 시나리오:
    /// 1. 게임 시작 (StartNewGame)
    /// 2. Phase 데이터 요청 (RequestPhaseData)
    /// 3. EventSelection 타입의 이벤트 3개 받음 (Shop, Bonus, Random)
    /// 4. 각 선택지를 선택했을 때 올바른 상태 전환이 일어나는지 확인
    #[test]
    fn complete_event_selection_flow() {
        // Given: Dawn Phase 1 게임 설정
        let game_data = create_test_game_data();
        let mut game = GameCore::new(game_data.clone(), 12345);
        let player_id = Uuid::new_v4();

        // When: 게임 시작
        let result = game.execute(player_id, PlayerBehavior::StartNewGame);

        // Then: 게임 시작 성공
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BehaviorResult::StartNewGame));

        // Then: 게임 상태가 WaitingPhaseRequest로 변경됨
        use game_core::ecs::resources::GameState;
        assert_eq!(game.get_state(), GameState::WaitingPhaseRequest);

        // When: Phase 데이터 요청
        let result = game.execute(player_id, PlayerBehavior::RequestPhaseData);

        // Then: Phase 데이터 수신 성공
        assert!(result.is_ok());

        let phase_event = match result.unwrap() {
            BehaviorResult::RequestPhaseData(event) => event,
            _ => panic!("Expected RequestPhaseData result"),
        };

        // Then: 게임 상태가 SelectingEvent로 변경됨
        assert_eq!(game.get_state(), GameState::SelectingEvent);

        // Then: CurrentPhaseEvents에 3개의 이벤트가 저장됨
        assert_eq!(game.get_phase_events_count(), 3);

        // Then: EventSelection 타입이어야 함
        assert!(phase_event.is_event_selection());

        // Then: 정확히 3개의 선택지가 있어야 함
        let options = phase_event.options();
        assert_eq!(options.len(), 3);

        // Then: Dawn Phase I 진행 상황 확인
        use game_core::game::enums::{OrdealType, PhaseType};
        let (ordeal, phase) = game.get_progression().unwrap();
        assert_eq!(ordeal, OrdealType::Dawn);
        assert_eq!(phase, PhaseType::I);

        // Then: Shop, Bonus, Random 이벤트가 각각 1개씩 있어야 함
        let has_shop = options
            .iter()
            .any(|opt| matches!(opt, GameOption::Shop { .. }));
        let has_bonus = options
            .iter()
            .any(|opt| matches!(opt, GameOption::Bonus { .. }));
        let has_random = options
            .iter()
            .any(|opt| matches!(opt, GameOption::Random { .. }));

        assert!(has_shop, "Should have a Shop option");
        assert!(has_bonus, "Should have a Bonus option");
        assert!(has_random, "Should have a Random event option");

        // Given: Shop 아이템 구매를 위한 엔케팔린
        game.set_enkephalin(1000);

        // Then: 1,000 개가 충전되었는지 확인
        assert_eq!(game.get_enkephalin(), 1000);

        let shop = phase_event.as_event_selection().unwrap().0;

        // When: 상점 선택 행동 수행
        let result = game.execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: shop.uuid,
            },
        );

        // Then: 행동 검증
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));

        // When: 아이템 구매
        let result = game.execute(
            player_id,
            PlayerBehavior::PurchaseItem {
                item_uuid: *shop.visible_items.get(0).unwrap(),
            },
        );

        // Then: 구매 검증
        assert!(matches!(
            result.unwrap(),
            BehaviorResult::PurchaseItem { .. }
        ));

        // ============================================================
        // Phase I → Phase II 전환
        // ============================================================

        // When: 상점 나가기 (Phase 진행)
        let result = game.execute(player_id, PlayerBehavior::ExitShop);

        // Then: Phase 진행 성공
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            BehaviorResult::AdvancePhase { .. }
        ));

        // Then: 게임 상태가 WaitingPhaseRequest로 변경됨
        assert_eq!(game.get_state(), GameState::WaitingPhaseRequest);

        // Then: Phase II로 진행됨
        let (ordeal, phase) = game.get_progression().unwrap();
        assert_eq!(ordeal, OrdealType::Dawn);
        assert_eq!(phase, PhaseType::II);

        // ============================================================
        // Phase II: EventSelection
        // ============================================================

        // When: Phase 데이터 요청
        let result = game.execute(player_id, PlayerBehavior::RequestPhaseData);
        assert!(result.is_ok());

        let phase_event = match result.unwrap() {
            BehaviorResult::RequestPhaseData(event) => event,
            _ => panic!("Expected RequestPhaseData result"),
        };

        // Then: EventSelection 타입
        assert!(phase_event.is_event_selection());

        // Given: 보너스 선택
        let bonus = phase_event.as_event_selection().unwrap().1;

        // When: 보너스 선택
        let result = game.execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: bonus.uuid,
            },
        );
        assert!(result.is_ok());

        // When: 보너스 나가기 (Phase 진행)
        let result = game.execute(player_id, PlayerBehavior::ExitBonus);
        assert!(result.is_ok());

        // Then: Phase III로 진행됨
        let (ordeal, phase) = game.get_progression().unwrap();
        assert_eq!(ordeal, OrdealType::Dawn);
        assert_eq!(phase, PhaseType::III);

        // ============================================================
        // Phase III: Suppression (PvE 전투)
        // ============================================================

        // When: Phase 데이터 요청
        let result = game.execute(player_id, PlayerBehavior::RequestPhaseData);
        assert!(result.is_ok());

        let phase_event = match result.unwrap() {
            BehaviorResult::RequestPhaseData(event) => event,
            _ => panic!("Expected RequestPhaseData result"),
        };

        // Then: Suppression 타입
        assert!(phase_event.is_suppression());

        // Then: 3개의 진압 선택지
        let candidates = phase_event.as_suppression().unwrap();
        assert_eq!(candidates.len(), 3);

        // Given: 첫 번째 진압 대상 선택
        let suppression_target = &candidates[0];

        // When: 진압 전투 시작
        let result = game.execute(
            player_id,
            PlayerBehavior::StartSuppression {
                abnormality_id: suppression_target.abnormality_id.clone(),
            },
        );

        // Then: 전투 완료 및 Phase 진행
        assert!(
            result.is_ok(),
            "StartSuppression failed: {:?}",
            result.err()
        );
        assert!(matches!(
            result.unwrap(),
            BehaviorResult::AdvancePhase { .. }
        ));

        // Then: Phase IV로 진행됨
        let (ordeal, phase) = game.get_progression().unwrap();
        assert_eq!(ordeal, OrdealType::Dawn);
        assert_eq!(phase, PhaseType::IV);

        // Then: 게임 상태가 WaitingPhaseRequest로 변경됨
        assert_eq!(game.get_state(), GameState::WaitingPhaseRequest);
    }

    /// 통합 테스트: 잘못된 이벤트 ID 선택 시 에러 반환
    #[test]
    fn select_invalid_event_id() {
        // Given: 게임 시작 및 Phase 데이터 받음
        let game_data = create_test_game_data();
        let mut game = GameCore::new(game_data.clone(), 12345);
        let player_id = Uuid::new_v4();

        game.execute(player_id, PlayerBehavior::StartNewGame)
            .unwrap();
        game.execute(player_id, PlayerBehavior::RequestPhaseData)
            .unwrap();

        // When: 존재하지 않는 이벤트 ID로 선택 시도
        let invalid_uuid = Uuid::new_v4();

        let result = game.execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: invalid_uuid,
            },
        );

        // Then: panic 대신 GameError::EventNotFound 를 반환해야 함
        assert!(result.is_err(), "Expected error on invalid event ID");
        match result.unwrap_err() {
            game_core::game::behavior::GameError::EventNotFound => {}
            other => panic!("Expected EventNotFound, got {:?}", other),
        }
    }

    /// 통합 테스트: 허용된 행동 검증
    ///
    /// 각 상태에서 올바른 행동만 허용되는지 확인
    #[test]
    fn allowed_actions_validation() {
        use game_core::ecs::resources::GameState;

        // Given: 게임 시작
        let game_data = create_test_game_data();
        let mut game = GameCore::new(game_data.clone(), 12345);
        let player_id = Uuid::new_v4();

        // When: NotStarted 상태
        // Then: StartNewGame만 허용됨
        assert_eq!(game.get_state(), GameState::NotStarted);
        let allowed = game.get_allowed_actions();
        assert_eq!(allowed.len(), 1);
        assert!(game.is_action_allowed(&PlayerBehavior::StartNewGame));
        assert!(!game.is_action_allowed(&PlayerBehavior::RequestPhaseData));

        // When: 게임 시작 후 WaitingPhaseRequest 상태
        game.execute(player_id, PlayerBehavior::StartNewGame)
            .unwrap();
        assert_eq!(game.get_state(), GameState::WaitingPhaseRequest);

        // Then: RequestPhaseData만 허용됨
        let allowed = game.get_allowed_actions();
        assert_eq!(allowed.len(), 1);
        assert!(game.is_action_allowed(&PlayerBehavior::RequestPhaseData));
        assert!(!game.is_action_allowed(&PlayerBehavior::StartNewGame));

        // When: Phase 데이터 요청 후 SelectingEvent 상태
        game.execute(player_id, PlayerBehavior::RequestPhaseData)
            .unwrap();
        assert_eq!(game.get_state(), GameState::SelectingEvent);

        // Then: SelectEvent, StartSuppression 허용됨
        let allowed = game.get_allowed_actions();
        assert_eq!(allowed.len(), 2);
        assert!(game.is_action_allowed(&PlayerBehavior::SelectEvent {
            event_id: Uuid::new_v4()
        }));
        assert!(game.is_action_allowed(&PlayerBehavior::StartSuppression {
            abnormality_id: String::new()
        }));
        assert!(!game.is_action_allowed(&PlayerBehavior::RequestPhaseData));
    }
}

// ============================================================
// Initial State Tests
// ============================================================
#[cfg(test)]
mod initial_state {
    use super::*;

    /// 통합 테스트: Level과 WinCount 초기값 확인
    #[test]
    fn initial_game_resources() {
        // Given: 새 게임
        let game_data = create_test_game_data();
        let game = GameCore::new(game_data.clone(), 12345);

        // Then: 초기값 확인
        assert_eq!(game.get_enkephalin(), 0);
        assert_eq!(game.get_level(), 1);
        assert_eq!(game.get_win_count(), 0);
        assert_eq!(game.get_phase_events_count(), 0);

        // Then: 초기 진행 상황 확인
        use game_core::game::enums::{OrdealType, PhaseType};
        let (ordeal, phase) = game.get_progression().unwrap();
        assert_eq!(ordeal, OrdealType::Dawn);
        assert_eq!(phase, PhaseType::I);
    }

    /// 통합 테스트: EventManager와 OrdealScheduler 통합
    ///
    /// EventManager가 OrdealScheduler를 올바르게 참조하여
    /// Dawn Phase I에 EventSelection 타입 이벤트를 생성하는지 확인
    #[test]
    fn event_manager_ordeal_scheduler_integration() {
        // Given: Dawn Phase I 설정
        let game_data = create_test_game_data();
        let mut game = GameCore::new(game_data.clone(), 12345);
        let player_id = Uuid::new_v4();

        // When: 게임 시작 및 Phase 데이터 요청
        game.execute(player_id, PlayerBehavior::StartNewGame)
            .unwrap();
        let result = game
            .execute(player_id, PlayerBehavior::RequestPhaseData)
            .unwrap();

        // Then: EventManager가 OrdealScheduler를 참조하여 올바른 타입 반환
        let phase_event = match result {
            BehaviorResult::RequestPhaseData(event) => event,
            _ => panic!("Expected RequestPhaseData result"),
        };

        // Dawn Phase I는 EventSelection이어야 함
        assert!(
            phase_event.is_event_selection(),
            "Dawn Phase I should be EventSelection"
        );
        assert!(
            !phase_event.is_suppression(),
            "Dawn Phase I should not be Suppression"
        );
        assert!(
            !phase_event.is_ordeal(),
            "Dawn Phase I should not be Ordeal"
        );
    }
}
