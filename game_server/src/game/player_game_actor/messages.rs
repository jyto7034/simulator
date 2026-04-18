use actix::{Message, Recipient};
use game_core::game::behavior::PlayerBehavior;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::state::PlayerGameActorError;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlayerGameClientMessage {
    Auth {
        player_id: Uuid,
        #[serde(default)]
        token: Option<String>,
    },
    Command {
        request_id: String,
        behavior: PlayerBehaviorRequest,
    },
    Ping,
    /// 플레이어 의도적 종료: Actor 즉시 중지 + 다음 접속 시 새로운 게임으로 시작.
    Quit,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum PlayerBehaviorRequest {
    StartNewGame,
    UnEquipItem {
        item_uuid: Uuid,
        target_unit: Uuid,
    },
    EquipItem {
        item_uuid: Uuid,
        target_unit: Uuid,
    },
    MoveUnit {
        target_unit_uuid: Uuid,
        dest_pos: game_core::ecs::resources::Position,
    },
    TransferUnit {
        target_unit_uuid: Uuid,
        dest_zone: game_core::game::enums::ZoneType,
    },
    RequestPhaseData,
    SelectEvent {
        event_id: Uuid,
    },
    PurchaseItem {
        item_uuid: Uuid,
    },
    SellItem {
        item_uuid: Uuid,
    },
    RerollShop,
    ExitShop,
    ClaimBonus,
    ExitBonus,
    StartSuppression {
        abnormality_id: String,
    },
    FinishSuppressionReplay,
}

impl From<PlayerBehaviorRequest> for PlayerBehavior {
    fn from(value: PlayerBehaviorRequest) -> Self {
        match value {
            PlayerBehaviorRequest::StartNewGame => Self::StartNewGame,
            PlayerBehaviorRequest::UnEquipItem {
                item_uuid,
                target_unit,
            } => Self::UnEquipItem {
                item_uuid,
                target_unit,
            },
            PlayerBehaviorRequest::EquipItem {
                item_uuid,
                target_unit,
            } => Self::EquipItem {
                item_uuid,
                target_unit,
            },
            PlayerBehaviorRequest::MoveUnit {
                target_unit_uuid,
                dest_pos,
            } => Self::MoveUnit {
                target_unit_uuid,
                dest_pos,
            },
            PlayerBehaviorRequest::TransferUnit {
                target_unit_uuid,
                dest_zone,
            } => Self::TransferUnit {
                target_unit_uuid,
                dest_zone,
            },
            PlayerBehaviorRequest::RequestPhaseData => Self::RequestPhaseData,
            PlayerBehaviorRequest::SelectEvent { event_id } => Self::SelectEvent { event_id },
            PlayerBehaviorRequest::PurchaseItem { item_uuid } => Self::PurchaseItem { item_uuid },
            PlayerBehaviorRequest::SellItem { item_uuid } => Self::SellItem { item_uuid },
            PlayerBehaviorRequest::RerollShop => Self::RerollShop,
            PlayerBehaviorRequest::ExitShop => Self::ExitShop,
            PlayerBehaviorRequest::ClaimBonus => Self::ClaimBonus,
            PlayerBehaviorRequest::ExitBonus => Self::ExitBonus,
            PlayerBehaviorRequest::StartSuppression { abnormality_id } => {
                Self::StartSuppression { abnormality_id }
            }
            PlayerBehaviorRequest::FinishSuppressionReplay => Self::FinishSuppressionReplay,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlayerGameServerMessage {
    Authed {
        player_id: Uuid,
    },
    StateSnapshot {
        state: Value,
    },
    CommandResult {
        request_id: String,
        ok: bool,
        result_type: String,
        payload: Value,
    },
    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
        code: String,
        message: String,
    },
    Notification {
        notification_type: String,
        payload: Value,
    },
    Pong,
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct ForceDisconnect {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CommandExecutionResult {
    pub response: PlayerGameServerMessage,
    pub state_snapshot: Value,
}

#[derive(Message)]
#[rtype(result = "Result<Value, PlayerGameActorError>")]
pub struct AttachSession {
    pub session_id: Uuid,
    pub socket: Recipient<PlayerGameServerMessage>,
    pub control: Recipient<ForceDisconnect>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DetachSession {
    pub session_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Result<CommandExecutionResult, PlayerGameActorError>")]
pub struct ExecutePlayerBehavior {
    pub session_id: Uuid,
    pub request_id: String,
    pub behavior: PlayerBehavior,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PushServerMessage {
    pub message: PlayerGameServerMessage,
}

/// 클라이언트 의도적 Quit 로 인한 Actor 종료 요청.
/// 재접속 TTL 건너뛰고 즉시 중지 + LoadBalance 등록에서 제거.
#[derive(Message)]
#[rtype(result = "()")]
pub struct QuitPlayerActor;
