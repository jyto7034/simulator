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
/// Capability projection is derived on demand from this state and active domain content.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GameState {
    #[default]
    NotStarted,
    PreRunBriefing {
        game_mode: crate::game::map::GameMode,
    },
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
    MasteredEliteSweepResult {
        sweep_uuid: Uuid,
    },
    InBattle {
        battle_uuid: Uuid,
    },
    AwaitingRunFailureDecision {
        reason: RunFailureReason,
        contributing_reason: Option<RunFailureReason>,
    },
    GameOver,
    RunComplete,
    RunFailed {
        reason: RunFailureReason,
    },
}
