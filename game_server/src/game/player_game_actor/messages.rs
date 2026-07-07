use actix::{Message, Recipient};
use game_core::game::{
    behavior::{
        LiveBattleEventDeltaDto, LiveBattleSetupBattlefieldDto, LiveBattleSetupCatalogRefsDto,
        LiveBattleSetupInitialUnitDto, LiveBattleSetupSnapshotDto, LiveBattleSetupStaticObjectDto,
        LiveBattleSetupTacticalPointDto, LiveBattleStateCheckpointDto, LiveBattleUpdateDto,
        PlayerBehavior,
    },
    combat_preview::{
        BattlefieldRoute, CombatMissionVariant, CombatNodeType, DeploymentZone, SpawnZone,
    },
    world::AdminCommand,
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
        behavior: PlayerBehavior,
        #[serde(default)]
        battle_response: Option<BattleSideMessageKind>,
    },
    AdminCommand {
        request_id: String,
        admin: AdminCommand,
        #[serde(default)]
        token: Option<String>,
    },
    Ping,
    /// 플레이어 의도적 종료: Actor 즉시 중지 + 다음 접속 시 새로운 게임으로 시작.
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleSideMessageKind {
    BattleUpdate,
    BattleResync,
}

#[derive(Debug, Clone)]
pub struct PlayerBehaviorCommand {
    pub behavior: PlayerBehavior,
    pub battle_side_message: BattleSideMessageKind,
}

impl PlayerBehaviorCommand {
    pub fn from_transport(
        behavior: PlayerBehavior,
        battle_response: Option<BattleSideMessageKind>,
    ) -> Result<Self, PlayerGameActorError> {
        let battle_side_message = battle_response.unwrap_or(BattleSideMessageKind::BattleUpdate);
        if matches!(battle_side_message, BattleSideMessageKind::BattleResync)
            && !matches!(behavior, PlayerBehavior::RequestBattleState { .. })
        {
            return Err(PlayerGameActorError::new(
                "invalid_battle_response",
                "battle_resync response envelope is only valid for request_battle_state",
            ));
        }

        Ok(Self {
            behavior,
            battle_side_message,
        })
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
    AdminResult {
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
    BattleUpdate {
        battle_uuid: Uuid,
        server_battle_time_ms: u64,
        events_delta: LiveBattleEventDeltaDto,
        checkpoint: LiveBattleStateCheckpointDto,
    },
    BattleSetupSnapshot {
        setup_version: u32,
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        survive_timer_ms: Option<u64>,
        battlefield: LiveBattleSetupBattlefieldDto,
        routes: Vec<BattlefieldRoute>,
        deployment_zones: Vec<DeploymentZone>,
        spawn_zones: Vec<SpawnZone>,
        tactical_points: Vec<LiveBattleSetupTacticalPointDto>,
        static_objects: Vec<LiveBattleSetupStaticObjectDto>,
        initial_units: Vec<LiveBattleSetupInitialUnitDto>,
        catalog_refs: LiveBattleSetupCatalogRefsDto,
    },
    BattleResync {
        battle_uuid: Uuid,
        update: LiveBattleUpdateDto,
    },
    Pong,
}

impl PlayerGameServerMessage {
    pub fn battle_update(update: game_core::game::behavior::LiveBattleUpdateDto) -> Self {
        Self::BattleUpdate {
            battle_uuid: update.battle_uuid,
            server_battle_time_ms: update.server_battle_time_ms,
            events_delta: update.events_delta,
            checkpoint: update.checkpoint,
        }
    }

    pub fn battle_setup_snapshot(setup: LiveBattleSetupSnapshotDto) -> Self {
        Self::BattleSetupSnapshot {
            setup_version: setup.setup_version,
            battle_uuid: setup.battle_uuid,
            encounter_id: setup.encounter_id,
            node_type: setup.node_type,
            mission_variant: setup.mission_variant,
            survive_timer_ms: setup.survive_timer_ms,
            battlefield: setup.battlefield,
            routes: setup.routes,
            deployment_zones: setup.deployment_zones,
            spawn_zones: setup.spawn_zones,
            tactical_points: setup.tactical_points,
            static_objects: setup.static_objects,
            initial_units: setup.initial_units,
            catalog_refs: setup.catalog_refs,
        }
    }

    pub fn battle_resync(update: LiveBattleUpdateDto) -> Self {
        Self::BattleResync {
            battle_uuid: update.battle_uuid,
            update,
        }
    }
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
    pub side_messages: Vec<PlayerGameServerMessage>,
    pub state_snapshot: Option<Value>,
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
    pub battle_side_message: BattleSideMessageKind,
}

#[derive(Message)]
#[rtype(result = "Result<CommandExecutionResult, PlayerGameActorError>")]
pub struct ExecuteAdminCommand {
    pub session_id: Uuid,
    pub request_id: String,
    pub command: AdminCommand,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PushServerMessage {
    pub message: PlayerGameServerMessage,
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct EnsureLiveBattleTick;

/// 클라이언트 의도적 Quit 로 인한 Actor 종료 요청.
/// 재접속 TTL 건너뛰고 즉시 중지 + LoadBalance 등록에서 제거.
#[derive(Message)]
#[rtype(result = "()")]
pub struct QuitPlayerActor;

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::game::{
        battle::tile_range::FacingDirection,
        behavior::{
            BattlePlaybackSpeed, BattlePlaybackState, LiveBattleEventDeltaDto,
            LiveBattleStateCheckpointDto, PlayerBehavior,
        },
        map::SupportNodeType,
        resources::Position,
    };

    #[test]
    fn serializes_battle_update_as_top_level_transport_message() {
        let battle_uuid = Uuid::from_u128(0xB4771E);
        let message = PlayerGameServerMessage::BattleUpdate {
            battle_uuid,
            server_battle_time_ms: 5_200,
            events_delta: LiveBattleEventDeltaDto {
                after_seq: 120,
                to_seq: 128,
                events: Vec::new(),
            },
            checkpoint: LiveBattleStateCheckpointDto {
                at_seq: 128,
                battle_time_ms: 5_200,
                playback: BattlePlaybackState::default(),
                units: Vec::new(),
                deployment: None,
            },
        };

        let json = serde_json::to_value(message).expect("battle update should serialize");

        assert_eq!(json["type"], "battle_update");
        assert_eq!(json["battle_uuid"], battle_uuid.to_string());
        assert_eq!(json["server_battle_time_ms"], 5_200);
        assert_eq!(json["events_delta"]["after_seq"], 120);
        assert_eq!(json["events_delta"]["to_seq"], 128);
        assert_eq!(json["checkpoint"]["at_seq"], 128);
        assert!(json.get("message_type").is_none());
    }

    #[test]
    fn serializes_battle_setup_snapshot_as_top_level_transport_message() {
        let battle_uuid = Uuid::from_u128(0x5E7A);
        let setup = game_core::game::behavior::LiveBattleSetupSnapshotDto {
            message_type:
                game_core::game::behavior::LiveBattleSetupSnapshotMessageType::BattleSetupSnapshot,
            setup_version: 1,
            battle_uuid,
            encounter_id: "defend_black_box_relay".to_string(),
            node_type: game_core::game::combat_preview::CombatNodeType::Defense,
            mission_variant: game_core::game::combat_preview::CombatMissionVariant::Defense,
            survive_timer_ms: Some(45_000),
            battlefield: game_core::game::behavior::LiveBattleSetupBattlefieldDto {
                width: 9,
                height: 9,
                tiles: Vec::new(),
                valid_tiles: Vec::new(),
                blocked_tiles: Vec::new(),
                static_obstacles: Vec::new(),
            },
            routes: Vec::new(),
            deployment_zones: Vec::new(),
            spawn_zones: Vec::new(),
            tactical_points: Vec::new(),
            static_objects: Vec::new(),
            initial_units: Vec::new(),
            catalog_refs: game_core::game::behavior::LiveBattleSetupCatalogRefsDto {
                battlefield_template_id: "choke_point_medium_01".to_string(),
                abnormality_ids: vec!["o-02-56_punishing_bird".to_string()],
            },
        };
        let message = PlayerGameServerMessage::battle_setup_snapshot(setup);

        let json = serde_json::to_value(message).expect("setup snapshot should serialize");

        assert_eq!(json["type"], "battle_setup_snapshot");
        assert_eq!(json["setup_version"], 1);
        assert_eq!(json["battle_uuid"], battle_uuid.to_string());
        assert_eq!(json["encounter_id"], "defend_black_box_relay");
        assert_eq!(json["survive_timer_ms"], 45_000);
        assert_eq!(json["battlefield"]["width"], 9);
        assert_eq!(
            json["catalog_refs"]["battlefield_template_id"],
            "choke_point_medium_01"
        );
        assert!(json.get("setup").is_none());
        assert!(json.get("message_type").is_none());
    }

    #[test]
    fn serializes_battle_resync_with_nested_battle_update() {
        let battle_uuid = Uuid::from_u128(0xB4771E);
        let update = LiveBattleUpdateDto {
            message_type: game_core::game::behavior::LiveBattleUpdateMessageType::BattleUpdate,
            battle_uuid,
            server_battle_time_ms: 6_200,
            events_delta: LiveBattleEventDeltaDto {
                after_seq: 120,
                to_seq: 150,
                events: Vec::new(),
            },
            checkpoint: LiveBattleStateCheckpointDto {
                at_seq: 150,
                battle_time_ms: 6_200,
                playback: BattlePlaybackState::default(),
                units: Vec::new(),
                deployment: None,
            },
        };
        let message = PlayerGameServerMessage::battle_resync(update);

        let json = serde_json::to_value(message).expect("battle resync should serialize");

        assert_eq!(json["type"], "battle_resync");
        assert_eq!(json["battle_uuid"], battle_uuid.to_string());
        assert!(json.get("setup").is_none());
        assert_eq!(json["update"]["type"], "battle_update");
        assert_eq!(json["update"]["events_delta"]["after_seq"], 120);
        assert_eq!(json["update"]["events_delta"]["to_seq"], 150);
        assert_eq!(json["update"]["checkpoint"]["at_seq"], 150);
    }

    #[test]
    fn deserializes_snake_case_node_flow_requests() {
        let request: PlayerBehavior = serde_json::from_str(
            r#"{"type":"select_starter_employees","candidate_ids":["a","b","c"]}"#,
        )
        .expect("select starter request should deserialize");
        match request {
            PlayerBehavior::SelectStarterEmployees { candidate_ids } => {
                assert_eq!(candidate_ids, ["a", "b", "c"]);
            }
            other => panic!("unexpected behavior: {other:?}"),
        }

        let request: PlayerBehavior =
            serde_json::from_str(r#"{"type":"choose_support","support_type":"Rest"}"#)
                .expect("support request should deserialize");
        match request {
            PlayerBehavior::ChooseSupport { support_type } => {
                assert_eq!(support_type, SupportNodeType::Rest);
            }
            other => panic!("unexpected behavior: {other:?}"),
        }

        let request: PlayerBehavior = serde_json::from_str(
            r#"{"type":"upgrade_skill_fragment","target_fragment_id":"fragment_a"}"#,
        )
        .expect("skill fragment upgrade request should deserialize");
        match request {
            PlayerBehavior::UpgradeSkillFragment { target_fragment_id } => {
                assert_eq!(target_fragment_id.as_str(), "fragment_a");
            }
            other => panic!("unexpected behavior: {other:?}"),
        }

        let request: PlayerBehavior = serde_json::from_str(
            r#"{"type":"awaken_skill_fragment","target_fragment_id":"fragment_a"}"#,
        )
        .expect("skill fragment awaken request should deserialize");
        match request {
            PlayerBehavior::AwakenSkillFragment { target_fragment_id } => {
                assert_eq!(target_fragment_id.as_str(), "fragment_a");
            }
            other => panic!("unexpected behavior: {other:?}"),
        }

        let request: PlayerBehavior = serde_json::from_str(r#"{"type":"claim_reward"}"#)
            .expect("claim reward request should deserialize");
        assert!(matches!(request, PlayerBehavior::ClaimReward));

        let request: PlayerBehavior = serde_json::from_str(r#"{"type":"complete_combat_result"}"#)
            .expect("complete combat result request should deserialize");
        assert!(matches!(request, PlayerBehavior::CompleteCombatResult));

        let request: PlayerBehavior =
            serde_json::from_str(r#"{"type":"recruit_employee","candidate_id":"candidate_a"}"#)
                .expect("recruit employee request should deserialize");
        match request {
            PlayerBehavior::RecruitEmployee { candidate_id } => {
                assert_eq!(candidate_id, "candidate_a");
            }
            other => panic!("unexpected behavior: {other:?}"),
        }

        let request: PlayerBehavior =
            serde_json::from_str(r#"{"type":"request_emergency_supplies"}"#)
                .expect("emergency supplies request should deserialize");
        assert!(matches!(request, PlayerBehavior::RequestEmergencySupplies));

        let request: PlayerBehavior = serde_json::from_str(r#"{"type":"open_headquarters_shop"}"#)
            .expect("open headquarters shop request should deserialize");
        assert!(matches!(request, PlayerBehavior::OpenHeadquartersShop));

        let employee_uuid = Uuid::new_v4();
        let request: PlayerBehavior = serde_json::from_str(&format!(
            r#"{{"type":"move_roster_unit","target_unit_uuid":"{employee_uuid}","dest_slot":2}}"#
        ))
        .expect("roster order move request should deserialize");
        assert!(matches!(
            request,
            PlayerBehavior::MoveRosterUnit {
                target_unit_uuid,
                dest_slot: 2,
                swap_with_unit_uuid: None,
            } if target_unit_uuid == employee_uuid
        ));

        let item_uuid = Uuid::new_v4();
        let request: PlayerBehavior = serde_json::from_str(&format!(
            r#"{{"type":"use_consumable_item","item_uuid":"{item_uuid}","target_employee_uuid":"{employee_uuid}"}}"#
        ))
        .expect("use consumable item request should deserialize");
        assert!(matches!(
            request,
            PlayerBehavior::UseConsumableItem {
                item_uuid: actual_item_uuid,
                target_employee_uuid,
            } if actual_item_uuid == item_uuid && target_employee_uuid == employee_uuid
        ));
    }

    #[test]
    fn deserializes_pending_deployment_range_preview_request() {
        let employee_uuid = Uuid::from_u128(0xABC);
        let request: PlayerBehavior = serde_json::from_str(&format!(
            r#"{{
                "type":"request_deployment_range_preview",
                "employee_uuid":"{employee_uuid}",
                "position":{{"x":6,"y":6}}
            }}"#
        ))
        .expect("pending deployment preview request should deserialize");

        match request {
            PlayerBehavior::RequestDeploymentRangePreview {
                employee_uuid: parsed_employee_uuid,
                position,
            } => {
                assert_eq!(parsed_employee_uuid, employee_uuid);
                assert_eq!(position, Position::new(6, 6));
            }
            other => panic!("unexpected behavior: {other:?}"),
        }
    }

    #[test]
    fn deserializes_admin_command_as_separate_top_level_message() {
        let request: PlayerGameClientMessage = serde_json::from_str(
            r#"{"type":"admin_command","request_id":"admin-001","token":"dev","admin":{"type":"admin_enter_maintenance"}}"#,
        )
        .expect("admin command should deserialize as top-level message");

        match request {
            PlayerGameClientMessage::AdminCommand {
                request_id,
                admin: AdminCommand::AdminEnterMaintenance,
                token,
            } => {
                assert_eq!(request_id, "admin-001");
                assert_eq!(token.as_deref(), Some("dev"));
            }
            other => panic!("unexpected message: {other:?}"),
        }

        let request: PlayerGameClientMessage = serde_json::from_str(
            r#"{"type":"admin_command","request_id":"admin-002","admin":{"type":"admin_dump_grant_catalog"}}"#,
        )
        .expect("admin grant catalog command should deserialize as top-level message");

        match request {
            PlayerGameClientMessage::AdminCommand {
                request_id,
                admin: AdminCommand::AdminDumpGrantCatalog,
                token,
            } => {
                assert_eq!(request_id, "admin-002");
                assert!(token.is_none());
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn admin_command_is_not_a_player_behavior_request() {
        let err = serde_json::from_str::<PlayerBehavior>(r#"{"type":"admin_enter_maintenance"}"#)
            .expect_err("admin command must not deserialize as gameplay behavior");

        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn deserializes_live_battle_requests() {
        let employee_uuid = Uuid::new_v4();
        let request: PlayerBehavior = serde_json::from_str(&format!(
            r#"{{"type":"deploy_unit","employee_uuid":"{employee_uuid}","position":{{"x":3,"y":4}},"facing":"right"}}"#
        ))
        .expect("deploy unit request should deserialize");
        match request {
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

        let request: PlayerBehavior = serde_json::from_str(&format!(
            r#"{{"type":"withdraw_unit","employee_uuid":"{employee_uuid}"}}"#
        ))
        .expect("withdraw unit request should deserialize");
        assert!(matches!(
            request,
            PlayerBehavior::WithdrawUnit {
                employee_uuid: actual_uuid
            } if actual_uuid == employee_uuid
        ));

        let request: PlayerBehavior =
            serde_json::from_str(r#"{"type":"request_battle_state","since_seq":7}"#)
                .expect("battle state request should deserialize");
        assert!(matches!(
            request,
            PlayerBehavior::RequestBattleState { since_seq: Some(7) }
        ));

        let request: PlayerBehavior = serde_json::from_str(r#"{"type":"request_battle_state"}"#)
            .expect("battle state request without since_seq should deserialize");
        assert!(matches!(
            request,
            PlayerBehavior::RequestBattleState { since_seq: None }
        ));

        let request: PlayerGameClientMessage = serde_json::from_str(
            r#"{"type":"command","request_id":"battle-001","battle_response":"battle_resync","behavior":{"type":"request_battle_state","since_seq":120}}"#,
        )
        .expect("battle resync request should deserialize");
        let PlayerGameClientMessage::Command {
            request_id,
            behavior,
            battle_response,
        } = request
        else {
            panic!("expected command message");
        };
        assert_eq!(request_id, "battle-001");
        let command = PlayerBehaviorCommand::from_transport(behavior, battle_response)
            .expect("catch-up resync should map to battle state request");
        assert!(matches!(
            command.behavior,
            PlayerBehavior::RequestBattleState {
                since_seq: Some(120)
            }
        ));
        assert_eq!(
            command.battle_side_message,
            BattleSideMessageKind::BattleResync
        );

        let request: PlayerBehavior =
            serde_json::from_str(r#"{"type":"recover_battle_setup_loss"}"#)
                .expect("setup-loss recovery behavior should deserialize");
        let command = PlayerBehaviorCommand::from_transport(request, None)
            .expect("setup-loss recovery should map to default update envelope");
        assert!(matches!(
            command.behavior,
            PlayerBehavior::RecoverBattleSetupLoss
        ));
        assert_eq!(
            command.battle_side_message,
            BattleSideMessageKind::BattleUpdate
        );

        let request: PlayerBehavior = serde_json::from_str(&format!(
            r#"{{"type":"activate_skill","employee_uuid":"{employee_uuid}","skill_id":"fragment_one_sin_penitence","target":null}}"#
        ))
        .expect("activate skill request should deserialize");
        assert!(matches!(
            request,
            PlayerBehavior::ActivateSkill {
                employee_uuid: actual_uuid,
                skill_id,
                target: None,
            } if actual_uuid == employee_uuid && skill_id.as_str() == "fragment_one_sin_penitence"
        ));

        let request: PlayerBehavior = serde_json::from_str(r#"{"type":"retreat_battle"}"#)
            .expect("retreat battle request should deserialize");
        assert!(matches!(request, PlayerBehavior::RetreatBattle));

        let request: PlayerBehavior = serde_json::from_str(r#"{"type":"pause_battle"}"#)
            .expect("pause battle request should deserialize");
        assert!(matches!(request, PlayerBehavior::PauseBattle));

        let request: PlayerBehavior = serde_json::from_str(r#"{"type":"resume_battle"}"#)
            .expect("resume battle request should deserialize");
        assert!(matches!(request, PlayerBehavior::ResumeBattle));

        let request: PlayerBehavior =
            serde_json::from_str(r#"{"type":"set_battle_speed","speed":"X0_5"}"#)
                .expect("battle speed request should deserialize");
        assert!(matches!(
            request,
            PlayerBehavior::SetBattleSpeed {
                speed: BattlePlaybackSpeed::X0_5
            }
        ));
    }
}
