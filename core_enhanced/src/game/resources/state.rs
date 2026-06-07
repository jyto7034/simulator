use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::map::{MapNodeCategory, MapNodeId, MapNodeKindId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunFailureReason {
    NoLivingEmployees,
    NoDeployableEmployees,
    BossDefeated,
}

/// 게임의 명시적인 상태.
///
/// 기본 allowed_actions는 이 상태로 결정하고, GameCore가 현재 노드 세션 내용을 더해 최종 보정한다.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum GameState {
    #[default]
    NotStarted,
    SelectingStarterEmployees,
    ViewingMap,
    NodeConfirm {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
    },
    InNode {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
    },
    InShop {
        shop_uuid: Uuid,
    },
    InReward {
        reward_uuid: Uuid,
    },
    InRewardClaimed {
        reward_uuid: Uuid,
    },
    CombatResult {
        battle_uuid: Uuid,
    },
    InBattle {
        battle_uuid: Uuid,
    },
    GameOver,
    RunComplete,
    RunFailed {
        reason: RunFailureReason,
    },
}
