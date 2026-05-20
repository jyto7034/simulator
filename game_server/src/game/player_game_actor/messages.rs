use actix::{Message, Recipient};
use game_core::game::{
    behavior::PlayerBehavior,
    data::skill_fragment_data::SkillFragmentId,
    map::{MapNodeId, MedicalTreatmentKind, SupportNodeType},
    resources::Position,
};
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlayerBehaviorRequest {
    StartNewGame,
    SelectStarterEmployees {
        candidate_ids: Vec<String>,
    },
    RequestMapData,
    SelectMapNode {
        node_id: MapNodeId,
    },
    UseReconScan,
    ConfirmEnterNode,
    CancelSelectedNode,
    CompleteNode,
    ChooseSupport {
        support_type: SupportNodeType,
    },
    SelectSupportTarget {
        employee_uuid: Uuid,
    },
    SelectMedicalTreatment {
        treatment: MedicalTreatmentKind,
    },
    SelectReward {
        reward_id: Uuid,
    },
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
        dest_pos: Position,
        #[serde(default)]
        swap_with_unit_uuid: Option<Uuid>,
    },
    MoveBenchUnit {
        target_unit_uuid: Uuid,
        dest_slot: usize,
        #[serde(default)]
        swap_with_unit_uuid: Option<Uuid>,
    },
    EquipSkillFragment {
        employee_uuid: Uuid,
        fragment_id: SkillFragmentId,
    },
    UnequipSkillFragment {
        employee_uuid: Uuid,
        fragment_id: SkillFragmentId,
    },
    UpgradeSkillFragment {
        target_fragment_id: SkillFragmentId,
        material_fragment_id: SkillFragmentId,
    },
    AwakenSkillFragment {
        target_fragment_id: SkillFragmentId,
        material_fragment_ids: Vec<SkillFragmentId>,
    },
    DismantleSkillFragment {
        fragment_id: SkillFragmentId,
    },
    RestoreEquipment {
        recipe_id: String,
    },
    DismantleEquipment {
        item_uuid: Uuid,
    },
    EnhanceEquipment {
        item_uuid: Uuid,
    },
    PurchaseItem {
        item_uuid: Uuid,
    },
    SellItem {
        item_uuid: Uuid,
    },
    RerollShop,
    ExitShop,
    ClaimReward,
    ExitReward,
    FinishCombatReplay,
}

impl From<PlayerBehaviorRequest> for PlayerBehavior {
    fn from(value: PlayerBehaviorRequest) -> Self {
        match value {
            PlayerBehaviorRequest::StartNewGame => Self::StartNewGame,
            PlayerBehaviorRequest::SelectStarterEmployees { candidate_ids } => {
                Self::SelectStarterEmployees { candidate_ids }
            }
            PlayerBehaviorRequest::RequestMapData => Self::RequestMapData,
            PlayerBehaviorRequest::SelectMapNode { node_id } => Self::SelectMapNode { node_id },
            PlayerBehaviorRequest::UseReconScan => Self::UseReconScan,
            PlayerBehaviorRequest::ConfirmEnterNode => Self::ConfirmEnterNode,
            PlayerBehaviorRequest::CancelSelectedNode => Self::CancelSelectedNode,
            PlayerBehaviorRequest::CompleteNode => Self::CompleteNode,
            PlayerBehaviorRequest::ChooseSupport { support_type } => {
                Self::ChooseSupport { support_type }
            }
            PlayerBehaviorRequest::SelectSupportTarget { employee_uuid } => {
                Self::SelectSupportTarget { employee_uuid }
            }
            PlayerBehaviorRequest::SelectMedicalTreatment { treatment } => {
                Self::SelectMedicalTreatment { treatment }
            }
            PlayerBehaviorRequest::SelectReward { reward_id } => Self::SelectReward { reward_id },
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
                swap_with_unit_uuid,
            } => Self::MoveUnit {
                target_unit_uuid,
                dest_pos,
                swap_with_unit_uuid,
            },
            PlayerBehaviorRequest::MoveBenchUnit {
                target_unit_uuid,
                dest_slot,
                swap_with_unit_uuid,
            } => Self::MoveBenchUnit {
                target_unit_uuid,
                dest_slot,
                swap_with_unit_uuid,
            },
            PlayerBehaviorRequest::EquipSkillFragment {
                employee_uuid,
                fragment_id,
            } => Self::EquipSkillFragment {
                employee_uuid,
                fragment_id,
            },
            PlayerBehaviorRequest::UnequipSkillFragment {
                employee_uuid,
                fragment_id,
            } => Self::UnequipSkillFragment {
                employee_uuid,
                fragment_id,
            },
            PlayerBehaviorRequest::UpgradeSkillFragment {
                target_fragment_id,
                material_fragment_id,
            } => Self::UpgradeSkillFragment {
                target_fragment_id,
                material_fragment_id,
            },
            PlayerBehaviorRequest::AwakenSkillFragment {
                target_fragment_id,
                material_fragment_ids,
            } => Self::AwakenSkillFragment {
                target_fragment_id,
                material_fragment_ids,
            },
            PlayerBehaviorRequest::DismantleSkillFragment { fragment_id } => {
                Self::DismantleSkillFragment { fragment_id }
            }
            PlayerBehaviorRequest::RestoreEquipment { recipe_id } => {
                Self::RestoreEquipment { recipe_id }
            }
            PlayerBehaviorRequest::DismantleEquipment { item_uuid } => {
                Self::DismantleEquipment { item_uuid }
            }
            PlayerBehaviorRequest::EnhanceEquipment { item_uuid } => {
                Self::EnhanceEquipment { item_uuid }
            }
            PlayerBehaviorRequest::PurchaseItem { item_uuid } => Self::PurchaseItem { item_uuid },
            PlayerBehaviorRequest::SellItem { item_uuid } => Self::SellItem { item_uuid },
            PlayerBehaviorRequest::RerollShop => Self::RerollShop,
            PlayerBehaviorRequest::ExitShop => Self::ExitShop,
            PlayerBehaviorRequest::ClaimReward => Self::ClaimReward,
            PlayerBehaviorRequest::ExitReward => Self::ExitReward,
            PlayerBehaviorRequest::FinishCombatReplay => Self::FinishCombatReplay,
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

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::game::{behavior::PlayerBehavior, map::SupportNodeType};

    #[test]
    fn deserializes_snake_case_node_flow_requests() {
        let request: PlayerBehaviorRequest = serde_json::from_str(
            r#"{"type":"select_starter_employees","candidate_ids":["a","b","c"]}"#,
        )
        .expect("select starter request should deserialize");
        match PlayerBehavior::from(request) {
            PlayerBehavior::SelectStarterEmployees { candidate_ids } => {
                assert_eq!(candidate_ids, ["a", "b", "c"]);
            }
            other => panic!("unexpected behavior: {other:?}"),
        }

        let request: PlayerBehaviorRequest =
            serde_json::from_str(r#"{"type":"choose_support","support_type":"Maintenance"}"#)
                .expect("support request should deserialize");
        match PlayerBehavior::from(request) {
            PlayerBehavior::ChooseSupport { support_type } => {
                assert_eq!(support_type, SupportNodeType::Maintenance);
            }
            other => panic!("unexpected behavior: {other:?}"),
        }

        let request: PlayerBehaviorRequest =
            serde_json::from_str(r#"{"type":"finish_combat_replay"}"#)
                .expect("finish combat replay request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::FinishCombatReplay
        ));

        let request: PlayerBehaviorRequest = serde_json::from_str(r#"{"type":"claim_reward"}"#)
            .expect("claim reward request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::ClaimReward
        ));
    }

    #[test]
    fn rejects_removed_phase_and_suppression_requests() {
        for legacy_type in [
            "request_phase_data",
            "select_event",
            "start_suppression",
            "finish_suppression_replay",
            "claim_combat_reward",
            "exit_combat_reward",
            "claim_bonus",
            "exit_bonus",
        ] {
            let json = format!(r#"{{"type":"{legacy_type}"}}"#);
            assert!(
                serde_json::from_str::<PlayerBehaviorRequest>(&json).is_err(),
                "legacy request `{legacy_type}` should not deserialize"
            );
        }
    }
}
