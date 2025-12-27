use uuid::Uuid;

use crate::{
    ecs::resources::{GameState, Position},
    game::behavior::PlayerBehavior,
};

/// ActionScheduler
///
/// GameState에 따라 허용되는 행동 목록을 반환
/// 모든 allowed_actions 로직이 이곳에 집중됨
pub struct ActionScheduler;

impl ActionScheduler {
    /// 게임 상태에 따라 허용된 행동 목록 반환
    ///
    /// # Arguments
    /// * `state` - 현재 게임 상태
    ///
    /// # Returns
    /// 허용된 PlayerBehavior 목록
    pub fn get_allowed_actions(state: &GameState) -> Vec<PlayerBehavior> {
        match state {
            GameState::NotStarted => {
                // 게임 시작 전: StartNewGame만 가능
                vec![PlayerBehavior::StartNewGame]
            }

            GameState::WaitingPhaseRequest => {
                // 게임 시작 후: Phase 데이터 요청만 가능
                vec![
                    PlayerBehavior::RequestPhaseData,
                    PlayerBehavior::EquipItem {
                        item_uuid: Uuid::nil(),
                        target_unit: Uuid::nil(),
                    },
                    PlayerBehavior::MoveUnit {
                        target_unit_uuid: Uuid::nil(),
                        dest_pos: Position::new(0, 0),
                    },
                ]
            }

            GameState::SelectingEvent => {
                // Phase 데이터 받음: 이벤트 선택 또는 진압 시작 가능
                vec![
                    PlayerBehavior::SelectEvent {
                        event_id: Uuid::nil(),
                    },
                    PlayerBehavior::StartSuppression {
                        abnormality_id: String::new(),
                    },
                    PlayerBehavior::EquipItem {
                        item_uuid: Uuid::nil(),
                        target_unit: Uuid::nil(),
                    },
                    PlayerBehavior::MoveUnit {
                        target_unit_uuid: Uuid::nil(),
                        dest_pos: Position::new(0, 0),
                    },
                ]
            }

            GameState::InShop { .. } => {
                // 상점 안: 아이템 구매, 판매, 리롤, 나가기 가능
                vec![
                    PlayerBehavior::PurchaseItem {
                        item_uuid: Uuid::nil(),
                    },
                    PlayerBehavior::SellItem {
                        item_uuid: Uuid::nil(),
                    },
                    PlayerBehavior::RerollShop,
                    PlayerBehavior::ExitShop,
                ]
            }

            GameState::InBonus { .. } => {
                // 보너스 안: 보너스 수령 또는 나가기 가능
                vec![PlayerBehavior::ClaimBonus, PlayerBehavior::ExitBonus]
            }

            GameState::InBonusClaimed { .. } => {
                // 보너스 수령 완료: 나가기만 가능
                vec![PlayerBehavior::ExitBonus]
            }

            GameState::InSuppression { .. } => {
                // 진압 작업 중: 작업 타입 선택, 나가기 가능
                // TODO: SelectWorkType, ExitSuppression 추가 후 활성화
                vec![PlayerBehavior::StartSuppression {
                    abnormality_id: String::new(),
                }]
            }

            GameState::InBattle { .. } => {
                // 전투 중: 턴 종료 등
                // TODO: UseCard, EndTurn 추가 후 활성화
                vec![]
            }

            GameState::GameOver => {
                // 게임 종료: 아무 행동도 불가능
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_started_allows_only_start_game() {
        let state = GameState::NotStarted;
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 1);
        assert!(matches!(allowed[0], PlayerBehavior::StartNewGame));
    }

    #[test]
    fn test_in_shop_allows_shop_actions() {
        let state = GameState::InShop {
            shop_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        // Then: PurchaseItem, SellItem, RerollShop, ExitShop 허용
        assert_eq!(allowed.len(), 4);
    }

    #[test]
    fn test_game_over_allows_nothing() {
        let state = GameState::GameOver;
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert!(allowed.is_empty());
    }

    #[test]
    fn test_waiting_phase_request_allows_only_request() {
        let state = GameState::WaitingPhaseRequest;
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 3);
        assert!(allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::RequestPhaseData)));
        assert!(allowed.iter().any(|a| matches!(a, PlayerBehavior::EquipItem { .. })));
        assert!(allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::MoveUnit { .. })));
    }

    #[test]
    fn test_selecting_event_allows_select_event_and_suppression() {
        let state = GameState::SelectingEvent;
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 4);
        assert!(allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::SelectEvent { .. })));
        assert!(allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::StartSuppression { .. })));
        assert!(allowed.iter().any(|a| matches!(a, PlayerBehavior::EquipItem { .. })));
        assert!(allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::MoveUnit { .. })));
    }

    #[test]
    fn test_in_suppression_allows_start_suppression() {
        let state = GameState::InSuppression {
            abnormality_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 1);
        assert!(allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::StartSuppression { .. })));
    }

    #[test]
    fn test_in_battle_allows_nothing_for_now() {
        let state = GameState::InBattle {
            battle_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        // TODO: TODO가 구현되기 전까지는 비어있어야 함
        assert!(allowed.is_empty());
    }

    #[test]
    fn test_state_transition_flow() {
        // Given: 게임 시작 흐름 검증
        let state = GameState::NotStarted;
        let allowed = ActionScheduler::get_allowed_actions(&state);
        assert_eq!(allowed.len(), 1);
        assert!(matches!(allowed[0], PlayerBehavior::StartNewGame));

        // When: Phase 요청
        let state = GameState::WaitingPhaseRequest;
        let allowed = ActionScheduler::get_allowed_actions(&state);
        assert_eq!(allowed.len(), 3);
        assert!(allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::RequestPhaseData)));
        assert!(allowed.iter().any(|a| matches!(a, PlayerBehavior::EquipItem { .. })));
        assert!(allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::MoveUnit { .. })));

        // When: 이벤트 선택
        let state = GameState::SelectingEvent;
        let allowed = ActionScheduler::get_allowed_actions(&state);
        assert_eq!(allowed.len(), 4);

        // When: 상점 진입
        let state = GameState::InShop {
            shop_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);
        // Then: Purchase/Sell/Reroll/Exit
        assert_eq!(allowed.len(), 4);
    }

    #[test]
    fn test_shop_allowed_actions_completeness() {
        let state = GameState::InShop {
            shop_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        // Then: 정확히 4개의 행동 허용
        assert_eq!(allowed.len(), 4);

        // Then: 각 행동이 존재해야 함
        let has_purchase = allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::PurchaseItem { .. }));
        let has_sell = allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::SellItem { .. }));
        let has_reroll = allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::RerollShop));
        let has_exit = allowed
            .iter()
            .any(|a| matches!(a, PlayerBehavior::ExitShop));

        assert!(has_purchase);
        assert!(has_sell);
        assert!(has_reroll);
        assert!(has_exit);
    }

    #[test]
    fn test_all_game_states_coverage() {
        // Then: 모든 GameState에 대해 allowed_actions가 정의되어 있어야 함
        let states = vec![
            GameState::NotStarted,
            GameState::WaitingPhaseRequest,
            GameState::SelectingEvent,
            GameState::InShop {
                shop_uuid: Uuid::nil(),
            },
            GameState::InBonus {
                bonus_uuid: Uuid::nil(),
            },
            GameState::InBonusClaimed {
                bonus_uuid: Uuid::nil(),
            },
            GameState::InSuppression {
                abnormality_uuid: Uuid::nil(),
            },
            GameState::InBattle {
                battle_uuid: Uuid::nil(),
            },
            GameState::GameOver,
        ];

        for state in states {
            // Then: panic하지 않고 정상적으로 반환되어야 함
            let _ = ActionScheduler::get_allowed_actions(&state);
        }
    }

    #[test]
    fn test_action_counts_per_state() {
        // Then: 각 상태별 허용 행동 개수 검증
        let test_cases = vec![
            (GameState::NotStarted, 1),
            (GameState::WaitingPhaseRequest, 3),
            (GameState::SelectingEvent, 4),
            (
                GameState::InShop {
                    shop_uuid: Uuid::nil(),
                },
                4,
            ),
            (
                GameState::InBonus {
                    bonus_uuid: Uuid::nil(),
                },
                2,
            ),
            (
                GameState::InBonusClaimed {
                    bonus_uuid: Uuid::nil(),
                },
                1,
            ),
            (
                GameState::InSuppression {
                    abnormality_uuid: Uuid::nil(),
                },
                1,
            ),
            (
                GameState::InBattle {
                    battle_uuid: Uuid::nil(),
                },
                0,
            ), // TODO
            (GameState::GameOver, 0),
        ];

        for (state, expected_count) in test_cases {
            let allowed = ActionScheduler::get_allowed_actions(&state);
            assert_eq!(
                allowed.len(),
                expected_count,
                "State {:?} should have {} allowed actions, but got {}",
                state,
                expected_count,
                allowed.len()
            );
        }
    }
}
