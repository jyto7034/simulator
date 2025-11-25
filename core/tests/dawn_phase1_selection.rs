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
    use game_core::ecs::resources::GameState;

    use super::*;

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

        // Then: 허용된 행동 확인 (구매, 리롤, 나가기)
        let allowed_actions = game.get_allowed_actions();
        assert_eq!(allowed_actions.len(), 3);

        // Then: CurrentPhaseEvents에서 선택된 이벤트가 제거됨
        assert_eq!(game.get_phase_events_count(), 2);
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

        let first_items: Vec<Uuid> =
            shop_metadata.visible_items.iter().map(|item| item.uuid()).collect();

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

        // Then: 허용된 행동 확인 (구매, 리롤, 나가기)
        let allowed_actions = game.get_allowed_actions();
        assert_eq!(allowed_actions.len(), 3);

        // Then: CurrentPhaseEvents에서 선택된 이벤트가 제거됨
        assert_eq!(game.get_phase_events_count(), 2);

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
            let rerolled: Vec<Uuid> = new_items.iter().map(|item| item.uuid()).collect();
            assert_ne!(rerolled, first_items, "Items must be different");
        }
    }

    // TODO: 각 상점의 visiable_items 을 순회하면서 아이템을 구매하는 helper 함수
    fn purchase_item(player_id: Uuid, game: &mut GameCore, item_uuid: Uuid) -> BehaviorResult {
        // When: 아이템 구매 시도
        let result = game.execute(player_id, PlayerBehavior::PurchaseItem { item_uuid });

        // Then: 행동 결과 확인
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap(),
            BehaviorResult::PurchaseItem {
                enkephalin,
                inventory_metadata,
                shop_metadata
            }
        ));

        todo!()
    }

    #[test]
    fn test_purchade_item() {
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

        // Then: 허용된 행동 확인 (구매, 리롤, 나가기)
        let allowed_actions = game.get_allowed_actions();
        assert_eq!(allowed_actions.len(), 3);

        // Then: CurrentPhaseEvents에서 선택된 이벤트가 제거됨
        assert_eq!(game.get_phase_events_count(), 2);

        let items = shop_metadata.visible_items.clone();
        for item in items {
            let uuid = item.uuid();
            purchase_item(player_id, &mut game, uuid);
        }
    }
}

// ============================================================
// Bonus Tests
// ============================================================
#[cfg(test)]
mod bonus {
    use super::*;

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

        // Then: 보너스 선택 성공
        assert!(result.is_ok());

        assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));

        // Then: Enkephalin이 증가했는지 확인 (50~100 범위)
        let final_enkephalin = game.get_enkephalin();
        assert!(
            final_enkephalin >= 50 && final_enkephalin <= 100,
            "Enkephalin should be between 50 and 100, but got {}",
            final_enkephalin
        );

        // Then: CurrentPhaseEvents에서 선택된 이벤트가 제거됨
        assert_eq!(game.get_phase_events_count(), 2);
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

        let result = game.execute(
            player_id,
            PlayerBehavior::SelectEvent {
                event_id: event_uuid,
            },
        );

        // Then: 랜덤 이벤트 진입 성공
        assert!(result.is_ok());

        assert!(matches!(result.unwrap(), BehaviorResult::EventSelected));

        // Then: 게임 상태가 InRandomEvent로 변경됨
        use game_core::ecs::resources::GameState;
        match game.get_state() {
            GameState::InRandomEvent { event_uuid: uuid } => {
                assert_eq!(uuid, event_uuid);
            }
            _ => panic!("Expected InRandomEvent state"),
        }

        // Then: 허용된 행동 확인 (선택지 선택, 나가기)
        let allowed_actions = game.get_allowed_actions();
        assert_eq!(allowed_actions.len(), 2);

        // Then: CurrentPhaseEvents에서 선택된 이벤트가 제거됨
        assert_eq!(game.get_phase_events_count(), 2);
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
        let (ordeal, phase) = game.get_progression();
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
    }

    /// 통합 테스트: 잘못된 이벤트 ID 선택 시 에러
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

        // Then: 패닉이 발생해야 함 (현재 GameCore 구현 상 panic!())
        // TODO: 에러 핸들링 개선 후 assert!(result.is_err()) 로 변경
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            game.execute(
                player_id,
                PlayerBehavior::SelectEvent {
                    event_id: invalid_uuid,
                },
            )
        }));

        assert!(result.is_err(), "Should panic on invalid event ID");
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

        // Then: SelectEvent만 허용됨
        let allowed = game.get_allowed_actions();
        assert_eq!(allowed.len(), 1);
        assert!(game.is_action_allowed(&PlayerBehavior::SelectEvent {
            event_id: Uuid::new_v4()
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
        let (ordeal, phase) = game.get_progression();
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
