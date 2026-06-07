use crate::{game::behavior::ActionKind, game::resources::GameState};

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
            GameState::SelectingStarterEmployees => {
                vec![ActionKind::SelectStarterEmployees]
            }
            GameState::ViewingMap => {
                vec![
                    ActionKind::RequestMapData,
                    ActionKind::SelectMapNode,
                    ActionKind::EquipItem,
                    ActionKind::UnEquipItem,
                    ActionKind::UseConsumableItem,
                    ActionKind::EquipSkillFragment,
                    ActionKind::UnequipSkillFragment,
                    ActionKind::MoveRosterUnit,
                ]
            }
            GameState::NodeConfirm { .. } => {
                vec![
                    ActionKind::RequestMapData,
                    ActionKind::SelectMapNode,
                    ActionKind::ConfirmEnterNode,
                    ActionKind::CancelSelectedNode,
                    ActionKind::EquipItem,
                    ActionKind::UnEquipItem,
                    ActionKind::UseConsumableItem,
                    ActionKind::EquipSkillFragment,
                    ActionKind::UnequipSkillFragment,
                ]
            }
            GameState::InNode { .. } => {
                vec![
                    ActionKind::CompleteNode,
                    ActionKind::ChooseSupport,
                    ActionKind::SelectSupportTarget,
                    ActionKind::SelectMedicalTreatment,
                    ActionKind::RecruitEmployee,
                    ActionKind::RequestEmergencySupplies,
                    ActionKind::OpenHeadquartersShop,
                    ActionKind::RequestMapData,
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
            GameState::InReward { .. } => {
                vec![
                    ActionKind::SelectReward,
                    ActionKind::ClaimReward,
                    ActionKind::ExitReward,
                ]
            }
            GameState::InRewardClaimed { .. } => {
                vec![ActionKind::ExitReward]
            }
            GameState::CombatResult { .. } => vec![ActionKind::CompleteCombatResult],
            GameState::InBattle { .. } => {
                vec![
                    ActionKind::RequestBattleState,
                    ActionKind::DeployUnit,
                    ActionKind::WithdrawUnit,
                    ActionKind::ActivateSkill,
                    ActionKind::RetreatBattle,
                    ActionKind::PauseBattle,
                    ActionKind::ResumeBattle,
                    ActionKind::SetBattleSpeed,
                ]
            }
            GameState::GameOver | GameState::RunComplete | GameState::RunFailed { .. } => {
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::{MapNodeCategory, MapNodeId, MapNodeKindId};
    use crate::game::resources::RunFailureReason;
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
    fn test_combat_result_allows_only_complete_result() {
        let state = GameState::CombatResult {
            battle_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 1);
        assert!(allowed.contains(&ActionKind::CompleteCombatResult));
    }

    #[test]
    fn test_in_battle_allows_live_battle_actions() {
        let state = GameState::InBattle {
            battle_uuid: Uuid::nil(),
        };
        let allowed = ActionScheduler::get_allowed_actions(&state);

        assert_eq!(allowed.len(), 8);
        assert!(allowed.contains(&ActionKind::RequestBattleState));
        assert!(allowed.contains(&ActionKind::DeployUnit));
        assert!(allowed.contains(&ActionKind::WithdrawUnit));
        assert!(allowed.contains(&ActionKind::ActivateSkill));
        assert!(allowed.contains(&ActionKind::RetreatBattle));
        assert!(allowed.contains(&ActionKind::PauseBattle));
        assert!(allowed.contains(&ActionKind::ResumeBattle));
        assert!(allowed.contains(&ActionKind::SetBattleSpeed));
    }

    #[test]
    fn test_state_transition_flow() {
        let state = GameState::NotStarted;
        let allowed = ActionScheduler::get_allowed_actions(&state);
        assert_eq!(allowed.len(), 1);
        assert_eq!(allowed[0], ActionKind::StartNewGame);

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
            GameState::ViewingMap,
            GameState::NodeConfirm {
                node_id: MapNodeId::new(Uuid::nil()),
                kind_id: MapNodeKindId::new("combat_monster"),
                category: MapNodeCategory::Combat,
            },
            GameState::InNode {
                node_id: MapNodeId::new(Uuid::nil()),
                kind_id: MapNodeKindId::new("combat_monster"),
                category: MapNodeCategory::Combat,
            },
            GameState::InShop {
                shop_uuid: Uuid::nil(),
            },
            GameState::InReward {
                reward_uuid: Uuid::nil(),
            },
            GameState::InRewardClaimed {
                reward_uuid: Uuid::nil(),
            },
            GameState::CombatResult {
                battle_uuid: Uuid::nil(),
            },
            GameState::InBattle {
                battle_uuid: Uuid::nil(),
            },
            GameState::GameOver,
            GameState::RunComplete,
            GameState::RunFailed {
                reason: RunFailureReason::NoLivingEmployees,
            },
        ];

        for state in states {
            let _ = ActionScheduler::get_allowed_actions(&state);
        }
    }
}
