use crate::{ecs::resources::GameState, game::behavior::ActionKind};

/// ActionScheduler
///
/// GameState에 따라 허용되는 액션 capability 목록을 반환
/// payload 유효성은 각 액션 validator가 별도로 검사한다.
pub struct ActionScheduler;

impl ActionScheduler {
    /// 게임 상태에 따라 허용된 행동 종류 반환
    pub fn get_allowed_actions(state: &GameState) -> Vec<ActionKind> {
        match state {
            GameState::NotStarted => {
                // 게임 시작 전: StartNewGame만 가능
                vec![ActionKind::StartNewGame]
            }
            GameState::WaitingPhaseRequest => {
                // 게임 시작 후: Phase 데이터 요청 및 편성 조정 가능
                vec![
                    ActionKind::RequestPhaseData,
                    ActionKind::EquipItem,
                    ActionKind::TransferUnit,
                    ActionKind::MoveUnit,
                ]
            }
            GameState::SelectingEvent => {
                // Phase 데이터 받음: 이벤트 선택 또는 진압 시작 가능
                vec![
                    ActionKind::SelectEvent,
                    ActionKind::StartSuppression,
                    ActionKind::EquipItem,
                    ActionKind::TransferUnit,
                    ActionKind::MoveUnit,
                ]
            }
            GameState::InShop { .. } => {
                vec![
                    ActionKind::PurchaseItem,
                    ActionKind::SellItem,
                    ActionKind::RerollShop,
                    ActionKind::ExitShop,
                ]
            }
            GameState::InBonus { .. } => {
                vec![
                    ActionKind::SelectEvent,
                    ActionKind::ClaimBonus,
                    ActionKind::ExitBonus,
                ]
            }
            GameState::InBonusClaimed { .. } => {
                vec![ActionKind::ExitBonus]
            }
            GameState::InSuppression { .. } => {
                // TODO: SelectWorkType, ExitSuppression 추가 후 활성화
                vec![ActionKind::StartSuppression]
            }
            GameState::InSuppressionReplay { .. } => {
                vec![ActionKind::FinishSuppressionReplay]
            }
            GameState::InBattle { .. } => {
                // TODO: UseCard, EndTurn 추가 후 활성화
                vec![]
            }
            GameState::GameOver => {
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_not_started_allows_only_start_game() {
        let state = GameState::NotStarted;
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0], ActionKind::StartNewGame);
    }

    #[test]
    fn test_in_shop_allows_shop_actions() {
        let state = GameState::InShop {
            shop_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

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

        assert_eq!(allowed.len(), 4);
        assert!(allowed.contains(&ActionKind::RequestPhaseData));
        assert!(allowed.contains(&ActionKind::EquipItem));
        assert!(allowed.contains(&ActionKind::TransferUnit));
        assert!(allowed.contains(&ActionKind::MoveUnit));
    }

    #[test]
    fn test_selecting_event_allows_select_event_and_suppression() {
        let state = GameState::SelectingEvent;
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 5);
        assert!(allowed.contains(&ActionKind::SelectEvent));
        assert!(allowed.contains(&ActionKind::StartSuppression));
        assert!(allowed.contains(&ActionKind::TransferUnit));
        assert!(allowed.contains(&ActionKind::EquipItem));
        assert!(allowed.contains(&ActionKind::MoveUnit));
    }

    #[test]
    fn test_in_suppression_allows_start_suppression() {
        let state = GameState::InSuppression {
            abnormality_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 1);
        assert!(allowed.contains(&ActionKind::StartSuppression));
    }

    #[test]
    fn test_in_suppression_replay_allows_only_finish_replay() {
        let state = GameState::InSuppressionReplay {
            abnormality_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 1);
        assert!(allowed.contains(&ActionKind::FinishSuppressionReplay));
    }

    #[test]
    fn test_in_battle_allows_nothing_for_now() {
        let state = GameState::InBattle {
            battle_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert!(allowed.is_empty());
    }

    #[test]
    fn test_state_transition_flow() {
        let state = GameState::NotStarted;
        let allowed = ActionScheduler::get_allowed_actions(&state);
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0], ActionKind::StartNewGame);

        let state = GameState::WaitingPhaseRequest;
        let allowed = ActionScheduler::get_allowed_actions(&state);
        assert_eq!(allowed.len(), 4);
        assert!(allowed.contains(&ActionKind::RequestPhaseData));
        assert!(allowed.contains(&ActionKind::EquipItem));
        assert!(allowed.contains(&ActionKind::TransferUnit));
        assert!(allowed.contains(&ActionKind::MoveUnit));

        let state = GameState::SelectingEvent;
        let allowed = ActionScheduler::get_allowed_actions(&state);
        assert_eq!(allowed.len(), 5);
        assert!(allowed.contains(&ActionKind::SelectEvent));
        assert!(allowed.contains(&ActionKind::StartSuppression));
        assert!(allowed.contains(&ActionKind::TransferUnit));

        let state = GameState::InShop {
            shop_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);
        assert_eq!(allowed.len(), 4);
    }

    #[test]
    fn test_shop_allowed_actions_completeness() {
        let state = GameState::InShop {
            shop_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 4);
        assert!(allowed.contains(&ActionKind::PurchaseItem));
        assert!(allowed.contains(&ActionKind::SellItem));
        assert!(allowed.contains(&ActionKind::RerollShop));
        assert!(allowed.contains(&ActionKind::ExitShop));
    }

    #[test]
    fn test_all_game_states_coverage() {
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
            GameState::InSuppressionReplay {
                abnormality_uuid: Uuid::nil(),
            },
            GameState::InBattle {
                battle_uuid: Uuid::nil(),
            },
            GameState::GameOver,
        ];

        for state in states {
            let _ = ActionScheduler::get_allowed_actions(&state);
        }
    }

    #[test]
    fn test_action_counts_per_state() {
        let test_cases = vec![
            (GameState::NotStarted, 1),
            (GameState::WaitingPhaseRequest, 4),
            (GameState::SelectingEvent, 5),
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
                3,
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
                GameState::InSuppressionReplay {
                    abnormality_uuid: Uuid::nil(),
                },
                1,
            ),
            (
                GameState::InBattle {
                    battle_uuid: Uuid::nil(),
                },
                0,
            ),
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
