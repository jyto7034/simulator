use actix::{Message, Recipient};
use game_core::game::{
    ability::SkillId,
    battle::tile_range::FacingDirection,
    battle::timeline::SkillCastTarget,
    behavior::{BattlePlaybackSpeed, PlayerBehavior},
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
    RecruitEmployee {
        candidate_id: String,
    },
    RequestEmergencySupplies,
    OpenHeadquartersShop,
    UnEquipItem {
        item_uuid: Uuid,
        target_unit: Uuid,
    },
    EquipItem {
        item_uuid: Uuid,
        target_unit: Uuid,
    },
    UseConsumableItem {
        item_uuid: Uuid,
        target_employee_uuid: Uuid,
    },
    MoveRosterUnit {
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
    CompleteCombatResult,
    RequestBattleState {
        #[serde(default)]
        since_seq: Option<u64>,
    },
    DeployUnit {
        employee_uuid: Uuid,
        position: Position,
        facing: FacingDirection,
    },
    WithdrawUnit {
        employee_uuid: Uuid,
    },
    ActivateSkill {
        employee_uuid: Uuid,
        skill_id: SkillId,
        #[serde(default)]
        target: Option<SkillCastTarget>,
    },
    RetreatBattle,
    PauseBattle,
    ResumeBattle,
    SetBattleSpeed {
        speed: BattlePlaybackSpeed,
    },
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
            PlayerBehaviorRequest::RecruitEmployee { candidate_id } => {
                Self::RecruitEmployee { candidate_id }
            }
            PlayerBehaviorRequest::RequestEmergencySupplies => Self::RequestEmergencySupplies,
            PlayerBehaviorRequest::OpenHeadquartersShop => Self::OpenHeadquartersShop,
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
            PlayerBehaviorRequest::UseConsumableItem {
                item_uuid,
                target_employee_uuid,
            } => Self::UseConsumableItem {
                item_uuid,
                target_employee_uuid,
            },
            PlayerBehaviorRequest::MoveRosterUnit {
                target_unit_uuid,
                dest_slot,
                swap_with_unit_uuid,
            } => Self::MoveRosterUnit {
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
            PlayerBehaviorRequest::CompleteCombatResult => Self::CompleteCombatResult,
            PlayerBehaviorRequest::RequestBattleState { since_seq } => {
                Self::RequestBattleState { since_seq }
            }
            PlayerBehaviorRequest::DeployUnit {
                employee_uuid,
                position,
                facing,
            } => Self::DeployUnit {
                employee_uuid,
                position,
                facing,
            },
            PlayerBehaviorRequest::WithdrawUnit { employee_uuid } => {
                Self::WithdrawUnit { employee_uuid }
            }
            PlayerBehaviorRequest::ActivateSkill {
                employee_uuid,
                skill_id,
                target,
            } => Self::ActivateSkill {
                employee_uuid,
                skill_id,
                target,
            },
            PlayerBehaviorRequest::RetreatBattle => Self::RetreatBattle,
            PlayerBehaviorRequest::PauseBattle => Self::PauseBattle,
            PlayerBehaviorRequest::ResumeBattle => Self::ResumeBattle,
            PlayerBehaviorRequest::SetBattleSpeed { speed } => Self::SetBattleSpeed { speed },
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

        let request: PlayerBehaviorRequest = serde_json::from_str(r#"{"type":"claim_reward"}"#)
            .expect("claim reward request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::ClaimReward
        ));

        let request: PlayerBehaviorRequest =
            serde_json::from_str(r#"{"type":"complete_combat_result"}"#)
                .expect("complete combat result request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::CompleteCombatResult
        ));

        let request: PlayerBehaviorRequest =
            serde_json::from_str(r#"{"type":"recruit_employee","candidate_id":"candidate_a"}"#)
                .expect("recruit employee request should deserialize");
        match PlayerBehavior::from(request) {
            PlayerBehavior::RecruitEmployee { candidate_id } => {
                assert_eq!(candidate_id, "candidate_a");
            }
            other => panic!("unexpected behavior: {other:?}"),
        }

        let request: PlayerBehaviorRequest =
            serde_json::from_str(r#"{"type":"request_emergency_supplies"}"#)
                .expect("emergency supplies request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::RequestEmergencySupplies
        ));

        let request: PlayerBehaviorRequest =
            serde_json::from_str(r#"{"type":"open_headquarters_shop"}"#)
                .expect("open headquarters shop request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::OpenHeadquartersShop
        ));

        let employee_uuid = Uuid::new_v4();
        let request: PlayerBehaviorRequest = serde_json::from_str(&format!(
            r#"{{"type":"move_roster_unit","target_unit_uuid":"{employee_uuid}","dest_slot":2}}"#
        ))
        .expect("roster order move request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::MoveRosterUnit {
                target_unit_uuid,
                dest_slot: 2,
                swap_with_unit_uuid: None,
            } if target_unit_uuid == employee_uuid
        ));

        let item_uuid = Uuid::new_v4();
        let request: PlayerBehaviorRequest = serde_json::from_str(&format!(
            r#"{{"type":"use_consumable_item","item_uuid":"{item_uuid}","target_employee_uuid":"{employee_uuid}"}}"#
        ))
        .expect("use consumable item request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::UseConsumableItem {
                item_uuid: actual_item_uuid,
                target_employee_uuid,
            } if actual_item_uuid == item_uuid && target_employee_uuid == employee_uuid
        ));
    }

    #[test]
    fn deserializes_live_battle_requests() {
        let employee_uuid = Uuid::new_v4();
        let request: PlayerBehaviorRequest = serde_json::from_str(&format!(
            r#"{{"type":"deploy_unit","employee_uuid":"{employee_uuid}","position":{{"x":3,"y":4}},"facing":"right"}}"#
        ))
        .expect("deploy unit request should deserialize");
        match PlayerBehavior::from(request) {
            PlayerBehavior::DeployUnit {
                employee_uuid: actual_uuid,
                position,
                facing,
            } => {
                assert_eq!(actual_uuid, employee_uuid);
                assert_eq!(position.x, 3);
                assert_eq!(position.y, 4);
                assert_eq!(facing, FacingDirection::Right);
            }
            other => panic!("unexpected behavior: {other:?}"),
        }

        let request: PlayerBehaviorRequest = serde_json::from_str(&format!(
            r#"{{"type":"withdraw_unit","employee_uuid":"{employee_uuid}"}}"#
        ))
        .expect("withdraw unit request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::WithdrawUnit {
                employee_uuid: actual_uuid
            } if actual_uuid == employee_uuid
        ));

        let request: PlayerBehaviorRequest =
            serde_json::from_str(r#"{"type":"request_battle_state","since_seq":7}"#)
                .expect("battle state request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::RequestBattleState { since_seq: Some(7) }
        ));

        let request: PlayerBehaviorRequest =
            serde_json::from_str(r#"{"type":"request_battle_state"}"#)
                .expect("battle state request without since_seq should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::RequestBattleState { since_seq: None }
        ));

        let request: PlayerBehaviorRequest = serde_json::from_str(&format!(
            r#"{{"type":"activate_skill","employee_uuid":"{employee_uuid}","skill_id":"fragment_one_sin_penitence","target":null}}"#
        ))
        .expect("activate skill request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::ActivateSkill {
                employee_uuid: actual_uuid,
                skill_id,
                target: None,
            } if actual_uuid == employee_uuid && skill_id.as_str() == "fragment_one_sin_penitence"
        ));

        let request: PlayerBehaviorRequest = serde_json::from_str(r#"{"type":"retreat_battle"}"#)
            .expect("retreat battle request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::RetreatBattle
        ));

        let request: PlayerBehaviorRequest = serde_json::from_str(r#"{"type":"pause_battle"}"#)
            .expect("pause battle request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::PauseBattle
        ));

        let request: PlayerBehaviorRequest = serde_json::from_str(r#"{"type":"resume_battle"}"#)
            .expect("resume battle request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::ResumeBattle
        ));

        let request: PlayerBehaviorRequest =
            serde_json::from_str(r#"{"type":"set_battle_speed","speed":"X0_5"}"#)
                .expect("battle speed request should deserialize");
        assert!(matches!(
            PlayerBehavior::from(request),
            PlayerBehavior::SetBattleSpeed {
                speed: BattlePlaybackSpeed::X0_5
            }
        ));
    }
}
