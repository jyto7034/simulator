use actix::{Message, Recipient};
use game_core::application::{
    ApplicationAdminCommand, ApplicationCommand, ApplicationCommandKind, ApplicationDocument,
};
use game_core::game::behavior::{
    BattleStateAuthorityDto, LiveBattleSetupSnapshotDto, LiveBattleUpdateDto,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::state::PlayerGameActorError;
use game_core::game::encounter_compendium::EncounterCompendiumProfileState;

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
        #[serde(deserialize_with = "deserialize_command")]
        behavior: ApplicationCommand,
        #[serde(default)]
        battle_authority: Option<BattleStateAuthorityDto>,
        #[serde(default)]
        battle_response: Option<BattleSideMessageKind>,
    },
    AdminCommand {
        request_id: String,
        #[serde(deserialize_with = "deserialize_admin")]
        admin: ApplicationAdminCommand,
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
    pub behavior: ApplicationCommand,
    pub battle_authority: Option<BattleStateAuthorityDto>,
    pub battle_side_message: BattleSideMessageKind,
}

impl PlayerBehaviorCommand {
    pub fn from_transport(
        behavior: ApplicationCommand,
        battle_authority: Option<BattleStateAuthorityDto>,
        battle_response: Option<BattleSideMessageKind>,
    ) -> Result<Self, PlayerGameActorError> {
        let battle_side_message = battle_response.unwrap_or(BattleSideMessageKind::BattleUpdate);
        if matches!(battle_side_message, BattleSideMessageKind::BattleResync)
            && behavior.kind() != ApplicationCommandKind::RequestBattleState
        {
            return Err(PlayerGameActorError::new(
                "invalid_battle_response",
                "battle_resync response envelope is only valid for request_battle_state",
            ));
        }

        Ok(Self {
            behavior,
            battle_authority,
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
    BattleResync {
        battle_uuid: Uuid,
        update: LiveBattleUpdateDto,
    },
    Pong,
    #[serde(untagged)]
    BattleUpdate(LiveBattleUpdateDto),
    #[serde(untagged)]
    BattleSetupSnapshot(LiveBattleSetupSnapshotDto),
}

impl PlayerGameServerMessage {
    pub fn battle_update(update: game_core::game::behavior::LiveBattleUpdateDto) -> Self {
        Self::BattleUpdate(update)
    }

    pub fn battle_setup_snapshot(setup: LiveBattleSetupSnapshotDto) -> Self {
        Self::BattleSetupSnapshot(setup)
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

#[derive(Debug, Clone)]
pub struct AttachSessionResult {
    pub state_snapshot: Value,
    pub side_messages: Vec<PlayerGameServerMessage>,
}

#[derive(Message)]
#[rtype(result = "Result<AttachSessionResult, PlayerGameActorError>")]
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
    pub behavior: ApplicationCommand,
    pub battle_authority: Option<BattleStateAuthorityDto>,
    pub battle_side_message: BattleSideMessageKind,
}

#[derive(Message)]
#[rtype(result = "Result<CommandExecutionResult, PlayerGameActorError>")]
pub struct ExecuteAdminCommand {
    pub session_id: Uuid,
    pub request_id: String,
    pub command: ApplicationAdminCommand,
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

#[derive(Message)]
#[rtype(result = "Result<EncounterCompendiumProfileState, PlayerGameActorError>")]
pub struct GetEncounterCompendiumProfile;

fn deserialize_command<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<ApplicationCommand, D::Error> {
    ApplicationCommand::from_document(ApplicationDocument::deserialize(d)?)
        .map_err(serde::de::Error::custom)
}
fn deserialize_admin<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<ApplicationAdminCommand, D::Error> {
    ApplicationAdminCommand::from_document(ApplicationDocument::deserialize(d)?)
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::game::{
        battle::types::TacticalTileKind,
        behavior::{
            ActionCapabilityDto, ActionDisabledReasonCode, BattlePlaybackSpeed,
            BattlePlaybackState, LiveBattleEventDeltaDto, LiveBattleFieldWorkCellStateDto,
            LiveBattleFieldWorkCompletionEffectDto, LiveBattleFieldWorkObjectStateDto,
            LiveBattleFieldWorkObjectStatusDto, LiveBattleFieldWorkPresentationKindDto,
            LiveBattlePresentationEventDto, LiveBattlePresentationEventKindDto,
            LiveBattleSetupFieldWorkCellDto, LiveBattleSetupFieldWorkObjectDto,
            LiveBattleStateCheckpointDto, LiveBattleTacticalFocusDto,
            LiveBattleTacticalFocusPurposeDto, LiveBattleTacticalResourceDto,
            LiveBattleUnitControlStateDto,
        },
        resources::Position,
    };

    #[test]
    fn serializes_control_status_checkpoint_contract_with_typed_reason() {
        let control = LiveBattleUnitControlStateDto {
            hard_cc_active: true,
            hard_cc_release_time_ms: Some(5_401),
            silenced: true,
            skill_start_disabled_reason: Some(ActionDisabledReasonCode::HardCrowdControl),
        };

        let json = serde_json::to_value(control).expect("control state should serialize");
        assert_eq!(json["hard_cc_active"], true);
        assert_eq!(json["hard_cc_release_time_ms"], 5_401);
        assert_eq!(json["silenced"], true);
        assert_eq!(json["skill_start_disabled_reason"], "hard_crowd_control");
        assert_eq!(
            serde_json::from_value::<LiveBattleUnitControlStateDto>(json).unwrap(),
            control
        );
    }

    #[test]
    fn serializes_battle_update_as_top_level_transport_message() {
        let battle_uuid = Uuid::from_u128(0xB4771E);
        let update = LiveBattleUpdateDto {
            message_type: game_core::game::behavior::LiveBattleUpdateMessageType::BattleUpdate,
            battle_uuid,
            battle_state_revision: 7,
            action_capabilities: vec![ActionCapabilityDto::enabled(
                game_core::game::behavior::ActionKind::PauseBattle,
                game_core::game::behavior::ActionScope::Gameplay,
            )],
            server_battle_time_ms: 5_200,
            events_delta: LiveBattleEventDeltaDto {
                after_seq: 120,
                to_seq: 128,
                events: vec![LiveBattlePresentationEventDto {
                    time_ms: 5_000,
                    seq: 128,
                    cause: Default::default(),
                    source_command_id: Some("deploy-worker".to_string()),
                    event: LiveBattlePresentationEventKindDto::FieldWorkAssignmentStarted {
                        object_id: "black_box_emergency_repair".to_string(),
                        work_cell_id: "repair_cell".to_string(),
                        worker_unit_instance_id: Uuid::from_u128(0xF13D).into(),
                    },
                }],
            },
            checkpoint: LiveBattleStateCheckpointDto {
                sustained_effects: Default::default(),
                independent_objects: Default::default(),
                at_seq: 128,
                battle_time_ms: 5_200,
                playback: BattlePlaybackState::default(),
                playback_eligibility:
                    game_core::game::behavior::LiveBattlePlaybackEligibilityDto::for_state(
                        BattlePlaybackState::default(),
                    ),
                interaction: Default::default(),
                tactical_focus: Some(LiveBattleTacticalFocusDto {
                    purpose: LiveBattleTacticalFocusPurposeDto::PlayerUnitSkill,
                    caster_unit_instance_id: Uuid::from_u128(0xCA57).into(),
                    skill_id: game_core::game::ability::SkillId::from(
                        "fragment_freischutz_black_round",
                    ),
                    restore_playback: BattlePlaybackState {
                        paused: false,
                        speed: BattlePlaybackSpeed::X2,
                    },
                }),
                field_work_objects: vec![LiveBattleFieldWorkObjectStateDto {
                    id: "black_box_emergency_repair".to_string(),
                    status: LiveBattleFieldWorkObjectStatusDto::Active,
                    accumulated_work_ms: 5_000,
                    required_work_ms: 10_000,
                    work_cells: vec![LiveBattleFieldWorkCellStateDto {
                        id: "repair_cell".to_string(),
                        position: Position::new(2, 6),
                        tile_kind: TacticalTileKind::Ground,
                        worker_unit_instance_id: Some(Uuid::from_u128(0xF13D).into()),
                    }],
                }],
                units: Vec::new(),
                tactical_resource: LiveBattleTacticalResourceDto {
                    current: 20,
                    maximum: 99,
                    recovery_per_second: 1,
                },
                deployment: None,
            },
            visual_novel: None,
        };
        let expected_json =
            serde_json::to_value(&update).expect("Core battle update should serialize");
        let expected_text =
            serde_json::to_string(&update).expect("Core battle update should serialize to text");
        let message = PlayerGameServerMessage::battle_update(update);

        assert_eq!(serde_json::to_string(&message).unwrap(), expected_text);
        let json = serde_json::to_value(message).expect("battle update should serialize");

        assert_eq!(json, expected_json);
        assert!(matches!(
            serde_json::from_value::<PlayerGameServerMessage>(json.clone()).unwrap(),
            PlayerGameServerMessage::BattleUpdate(update)
                if update.battle_state_revision == 7
        ));
        assert_eq!(json["type"], "battle_update");
        assert_eq!(json["battle_uuid"], battle_uuid.to_string());
        assert_eq!(json["battle_state_revision"], 7);
        assert_eq!(json["action_capabilities"][0]["action"], "PauseBattle");
        assert_eq!(json["server_battle_time_ms"], 5_200);
        assert_eq!(json["events_delta"]["after_seq"], 120);
        assert_eq!(json["events_delta"]["to_seq"], 128);
        assert_eq!(
            json["events_delta"]["events"][0]["event"]["type"],
            "FieldWorkAssignmentStarted"
        );
        assert_eq!(
            json["events_delta"]["events"][0]["event"]["object_id"],
            "black_box_emergency_repair"
        );
        assert_eq!(json["checkpoint"]["at_seq"], 128);
        assert_eq!(
            json["checkpoint"]["field_work_objects"][0]["status"],
            "active"
        );
        assert_eq!(
            json["checkpoint"]["field_work_objects"][0]["accumulated_work_ms"],
            5_000
        );
        assert_eq!(
            json["checkpoint"]["field_work_objects"][0]["work_cells"][0]["worker_unit_instance_id"],
            Uuid::from_u128(0xF13D).to_string()
        );
        assert_eq!(
            json["checkpoint"]["tactical_focus"]["purpose"],
            "player_unit_skill"
        );
        assert_eq!(
            json["checkpoint"]["tactical_focus"]["caster_unit_instance_id"],
            Uuid::from_u128(0xCA57).to_string()
        );
        assert_eq!(
            json["checkpoint"]["tactical_focus"]["skill_id"],
            "fragment_freischutz_black_round"
        );
        assert_eq!(
            json["checkpoint"]["tactical_focus"]["restore_playback"]["speed"],
            "X2"
        );
        assert_eq!(json["checkpoint"]["tactical_resource"]["current"], 20);
        assert_eq!(json["checkpoint"]["tactical_resource"]["maximum"], 99);
        assert_eq!(
            json["checkpoint"]["tactical_resource"]["recovery_per_second"],
            1
        );
        assert!(json["visual_novel"].is_null());
        assert!(json.get("message_type").is_none());
    }

    #[test]
    fn serializes_battle_setup_snapshot_as_top_level_transport_message() {
        let battle_uuid = Uuid::from_u128(0x5E7A);
        let setup = game_core::game::behavior::LiveBattleSetupSnapshotDto {
            message_type:
                game_core::game::behavior::LiveBattleSetupSnapshotMessageType::BattleSetupSnapshot,
            setup_version: 2,
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
            field_work_objects: vec![LiveBattleSetupFieldWorkObjectDto {
                id: "black_box_emergency_repair".to_string(),
                presentation_kind: LiveBattleFieldWorkPresentationKindDto::EmergencyRepairTerminal,
                position: Position::new(2, 6),
                work_cells: vec![LiveBattleSetupFieldWorkCellDto {
                    id: "repair_cell".to_string(),
                    position: Position::new(2, 6),
                    tile_kind: TacticalTileKind::Ground,
                }],
                required_work_ms: 10_000,
                completion_effect: LiveBattleFieldWorkCompletionEffectDto::RepairProtectedUnit {
                    unit_ref: "black_box_device".to_string(),
                    flat_hp: 100,
                },
            }],
            initial_units: Vec::new(),
            catalog_refs: game_core::game::behavior::LiveBattleSetupCatalogRefsDto {
                battlefield_template_id: "choke_point_medium_01".to_string(),
                abnormality_ids: vec!["o-02-56_punishing_bird".to_string()],
            },
        };
        let expected_json =
            serde_json::to_value(&setup).expect("Core battle setup should serialize");
        let expected_text =
            serde_json::to_string(&setup).expect("Core battle setup should serialize to text");
        let message = PlayerGameServerMessage::battle_setup_snapshot(setup);

        assert_eq!(serde_json::to_string(&message).unwrap(), expected_text);
        let json = serde_json::to_value(message).expect("setup snapshot should serialize");

        assert_eq!(json, expected_json);
        assert!(matches!(
            serde_json::from_value::<PlayerGameServerMessage>(json.clone()).unwrap(),
            PlayerGameServerMessage::BattleSetupSnapshot(setup)
                if setup.setup_version == 2
        ));
        assert_eq!(json["type"], "battle_setup_snapshot");
        assert_eq!(json["setup_version"], 2);
        assert_eq!(json["battle_uuid"], battle_uuid.to_string());
        assert_eq!(json["encounter_id"], "defend_black_box_relay");
        assert_eq!(json["survive_timer_ms"], 45_000);
        assert_eq!(json["battlefield"]["width"], 9);
        assert_eq!(
            json["field_work_objects"][0]["presentation_kind"],
            "emergency_repair_terminal"
        );
        assert_eq!(json["field_work_objects"][0]["position"]["x"], 2);
        assert_eq!(json["field_work_objects"][0]["position"]["y"], 6);
        assert_eq!(
            json["field_work_objects"][0]["completion_effect"]["type"],
            "repair_protected_unit"
        );
        assert_eq!(
            json["field_work_objects"][0]["completion_effect"]["flat_hp"],
            100
        );
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
            battle_state_revision: 9,
            action_capabilities: vec![ActionCapabilityDto::enabled(
                game_core::game::behavior::ActionKind::RequestBattleState,
                game_core::game::behavior::ActionScope::Query,
            )],
            server_battle_time_ms: 6_200,
            events_delta: LiveBattleEventDeltaDto {
                after_seq: 120,
                to_seq: 150,
                events: Vec::new(),
            },
            checkpoint: LiveBattleStateCheckpointDto {
                sustained_effects: Default::default(),
                independent_objects: Default::default(),
                at_seq: 150,
                battle_time_ms: 6_200,
                playback: BattlePlaybackState::default(),
                playback_eligibility:
                    game_core::game::behavior::LiveBattlePlaybackEligibilityDto::for_state(
                        BattlePlaybackState::default(),
                    ),
                tactical_focus: None,
                interaction: Default::default(),
                field_work_objects: Vec::new(),
                units: Vec::new(),
                tactical_resource: LiveBattleTacticalResourceDto {
                    current: 20,
                    maximum: 99,
                    recovery_per_second: 1,
                },
                deployment: None,
            },
            visual_novel: None,
        };
        let expected_update =
            serde_json::to_value(&update).expect("Core battle update should serialize");
        let expected_json = serde_json::json!({
            "type": "battle_resync",
            "battle_uuid": battle_uuid,
            "update": expected_update,
        });
        let message = PlayerGameServerMessage::battle_resync(update);

        let json = serde_json::to_value(message).expect("battle resync should serialize");

        assert_eq!(json, expected_json);
        assert_eq!(json["type"], "battle_resync");
        assert_eq!(json["battle_uuid"], battle_uuid.to_string());
        assert!(json.get("setup").is_none());
        assert_eq!(json["update"]["type"], "battle_update");
        assert_eq!(json["update"]["battle_state_revision"], 9);
        assert_eq!(json["update"]["events_delta"]["after_seq"], 120);
        assert_eq!(json["update"]["events_delta"]["to_seq"], 150);
        assert_eq!(json["update"]["checkpoint"]["at_seq"], 150);
    }

    fn assert_command(input: Value, kind: ApplicationCommandKind) -> PlayerBehaviorCommand {
        let message: PlayerGameClientMessage = serde_json::from_value(serde_json::json!({
            "type": "command", "request_id": "contract", "behavior": input.clone()
        }))
        .expect("public command transport must deserialize");
        let PlayerGameClientMessage::Command {
            request_id,
            behavior,
            battle_authority,
            battle_response,
        } = message
        else {
            panic!("command envelope")
        };
        assert_eq!(request_id, "contract");
        assert_eq!(behavior.kind(), kind);
        assert_eq!(behavior.document().as_value(), &input);
        PlayerBehaviorCommand::from_transport(behavior, battle_authority, battle_response).unwrap()
    }
    fn reject_command(input: Value) {
        assert!(
            serde_json::from_value::<PlayerGameClientMessage>(serde_json::json!({
                "type": "command", "request_id": "invalid", "behavior": input
            }))
            .is_err()
        );
    }

    #[test]
    fn deserializes_snake_case_node_flow_requests() {
        use ApplicationCommandKind as K;
        let node = Uuid::from_u128(0x5100);
        let copy = Uuid::from_u128(0xF001);
        let employee = Uuid::from_u128(0xE001);
        let item = Uuid::from_u128(0x1001);
        for (kind, input) in [
            (
                K::SelectStarterEmployees,
                serde_json::json!({"type":"select_starter_employees","employee_definition_ids":["a","b","c"]}),
            ),
            (
                K::ConvertMapNodeWithMonolith,
                serde_json::json!({"type":"convert_map_node_with_monolith","node_id":node,"abnormality_id":"o-01-15_nameless_fetus"}),
            ),
            (
                K::AwakenSkillFragment,
                serde_json::json!({"type":"awaken_skill_fragment","copy_uuid":copy,"form_id":"awakened"}),
            ),
            (K::ClaimReward, serde_json::json!({"type":"claim_reward"})),
            (
                K::CompleteCombatResult,
                serde_json::json!({"type":"complete_combat_result"}),
            ),
            (
                K::SweepMasteredElite,
                serde_json::json!({"type":"sweep_mastered_elite"}),
            ),
            (
                K::CompleteMasteredEliteSweep,
                serde_json::json!({"type":"complete_mastered_elite_sweep"}),
            ),
            (
                K::RetryPlayerProfile,
                serde_json::json!({"type":"retry_player_profile"}),
            ),
            (
                K::ResetPlayerProfile,
                serde_json::json!({"type":"reset_player_profile"}),
            ),
            (
                K::RecruitEmployee,
                serde_json::json!({"type":"recruit_employee","employee_definition_id":"candidate_a"}),
            ),
            (
                K::RequestEmergencySupplies,
                serde_json::json!({"type":"request_emergency_supplies"}),
            ),
            (
                K::OpenHeadquartersShop,
                serde_json::json!({"type":"open_headquarters_shop"}),
            ),
            (
                K::UseConsumableItem,
                serde_json::json!({"type":"use_consumable_item","item_uuid":item,"target_employee_uuid":employee}),
            ),
        ] {
            assert_command(input, kind);
        }
        for input in [
            serde_json::json!({"type":"choose_support","support_type":"Rest"}),
            serde_json::json!({"type":"upgrade_skill_fragment","target_fragment_id":"fragment_a"}),
            serde_json::json!({"type":"awaken_skill_fragment","target_fragment_id":"fragment_a"}),
            serde_json::json!({"type":"sweep_mastered_elite","ignored":true}),
        ] {
            reject_command(input);
        }
    }

    #[test]
    fn deserializes_pending_deployment_range_preview_request() {
        assert_command(
            serde_json::json!({"type":"request_deployment_range_preview","employee_uuid":Uuid::from_u128(0xABC),"position":{"x":6,"y":6}}),
            ApplicationCommandKind::RequestDeploymentRangePreview,
        );
    }

    #[test]
    fn deserializes_admin_command_as_separate_top_level_message() {
        for (id, token, admin) in [
            (
                "admin-001",
                Some("dev"),
                serde_json::json!({"type":"admin_enter_maintenance"}),
            ),
            (
                "admin-002",
                None,
                serde_json::json!({"type":"admin_dump_grant_catalog"}),
            ),
            (
                "admin-condition",
                None,
                serde_json::json!({"type":"admin_set_employee_condition","employee_uuid":Uuid::from_u128(0x00C0_11D1_710A),"physical_damage":5,"trauma":100}),
            ),
        ] {
            let message: PlayerGameClientMessage = serde_json::from_value(serde_json::json!({"type":"admin_command","request_id":id,"token":token,"admin":admin.clone()})).unwrap();
            let PlayerGameClientMessage::AdminCommand {
                request_id,
                admin: command,
                token: actual_token,
            } = message
            else {
                panic!("separate admin envelope")
            };
            assert_eq!(request_id, id);
            assert_eq!(actual_token.as_deref(), token);
            assert_eq!(command.document().as_value(), &admin);
        }
        for legacy_type in ["admin_set_employee_hp", "admin_set_employee_trauma"] {
            assert!(serde_json::from_value::<PlayerGameClientMessage>(serde_json::json!({"type":"admin_command","request_id":"legacy","admin":{"type":legacy_type,"employee_uuid":Uuid::nil(),"current_hp":1,"trauma":1}})).is_err());
        }
    }

    #[test]
    fn admin_command_is_not_a_player_behavior_request() {
        reject_command(serde_json::json!({"type":"admin_enter_maintenance"}));
    }

    #[test]
    fn deserializes_live_battle_requests() {
        use ApplicationCommandKind as K;
        let employee = Uuid::from_u128(0xE111);
        let second = Uuid::from_u128(0x2222);
        let unit = Uuid::from_u128(0x5151);
        for (kind, input) in [
            (
                K::DeployUnit,
                serde_json::json!({"type":"deploy_unit","employee_uuid":employee,"position":{"x":3,"y":4},"facing":"right"}),
            ),
            (
                K::WithdrawUnit,
                serde_json::json!({"type":"withdraw_unit","employee_uuid":employee}),
            ),
            (
                K::SetEngagementStance,
                serde_json::json!({"type":"set_engagement_stance","employee_uuid":employee,"stance":"defense"}),
            ),
            (
                K::TacticalRelocateUnit,
                serde_json::json!({"type":"tactical_relocate_unit","employee_uuid":employee,"destination":{"x":5,"y":6},"facing":"left"}),
            ),
            (
                K::TacticalSwapUnits,
                serde_json::json!({"type":"tactical_swap_units","first_employee_uuid":employee,"second_employee_uuid":second,"first_facing":"left","second_facing":"right"}),
            ),
            (
                K::FocusBattleUnit,
                serde_json::json!({"type":"focus_battle_unit","unit_instance_id":unit}),
            ),
            (
                K::ClearBattleFocus,
                serde_json::json!({"type":"clear_battle_focus"}),
            ),
            (
                K::BeginDeploymentSelection,
                serde_json::json!({"type":"begin_deployment_selection","employee_uuid":employee}),
            ),
            (
                K::CancelDeploymentSelection,
                serde_json::json!({"type":"cancel_deployment_selection"}),
            ),
            (
                K::OpenBattleSettings,
                serde_json::json!({"type":"open_battle_settings"}),
            ),
            (
                K::CloseBattleSettings,
                serde_json::json!({"type":"close_battle_settings"}),
            ),
            (
                K::BeginSkillTargeting,
                serde_json::json!({"type":"begin_skill_targeting","employee_uuid":employee,"skill_id":"fragment_freischutz_black_round"}),
            ),
            (
                K::CancelSkillTargeting,
                serde_json::json!({"type":"cancel_skill_targeting"}),
            ),
            (
                K::PreviewTacticalCommand,
                serde_json::json!({"type":"preview_tactical_command","command":{"type":"relocate","employee_uuid":employee,"destination":{"x":3,"y":4},"facing":"left"}}),
            ),
            (
                K::PreviewTacticalCommand,
                serde_json::json!({"type":"preview_tactical_command","command":{"type":"swap","first_employee_uuid":employee,"second_employee_uuid":second,"first_facing":"left","second_facing":"right"}}),
            ),
            (
                K::PreviewSkillTarget,
                serde_json::json!({"type":"preview_skill_target","employee_uuid":employee,"skill_id":"fragment_freischutz_black_round","target":{"type":"Unit","unit_instance_id":unit}}),
            ),
            (
                K::RequestBattleState,
                serde_json::json!({"type":"request_battle_state","since_seq":7}),
            ),
            (
                K::RequestBattleState,
                serde_json::json!({"type":"request_battle_state"}),
            ),
            (
                K::RecoverBattleSetupLoss,
                serde_json::json!({"type":"recover_battle_setup_loss"}),
            ),
            (
                K::ActivateSkill,
                serde_json::json!({"type":"activate_skill","employee_uuid":employee,"skill_id":"fragment_one_sin_penitence","target":null}),
            ),
            (
                K::RetreatBattle,
                serde_json::json!({"type":"retreat_battle"}),
            ),
            (K::PauseBattle, serde_json::json!({"type":"pause_battle"})),
            (K::ResumeBattle, serde_json::json!({"type":"resume_battle"})),
            (
                K::SetBattleSpeed,
                serde_json::json!({"type":"set_battle_speed","speed":"X0_5"}),
            ),
        ] {
            let command = assert_command(input, kind);
            assert_eq!(
                command.battle_side_message,
                BattleSideMessageKind::BattleUpdate
            );
        }
        for input in [
            serde_json::json!({"type":"set_engagement_stance","employee_uuid":employee,"stance":"defense_focus"}),
            serde_json::json!({"type":"tactical_relocate_unit","employee_uuid":employee,"destination":{"x":5,"y":6},"facing":"left","path":[{"x":4,"y":6}]}),
            serde_json::json!({"type":"tactical_swap_units","first_employee_uuid":employee,"second_employee_uuid":second,"first_facing":"left","second_facing":"right","destination":{"x":5,"y":6}}),
            serde_json::json!({"type":"preview_skill_target","employee_uuid":employee,"skill_id":"fragment_freischutz_black_round"}),
            serde_json::json!({"type":"begin_skill_targeting","employee_uuid":employee,"skill_id":"fragment_freischutz_black_round","target":null}),
        ] {
            reject_command(input);
        }
        let request: PlayerGameClientMessage = serde_json::from_value(serde_json::json!({
            "type":"command","request_id":"battle-001","battle_response":"battle_resync","behavior":{"type":"request_battle_state","since_seq":120}
        })).unwrap();
        let PlayerGameClientMessage::Command {
            request_id,
            behavior,
            battle_authority,
            battle_response,
        } = request
        else {
            panic!("command")
        };
        assert_eq!(request_id, "battle-001");
        let command =
            PlayerBehaviorCommand::from_transport(behavior, battle_authority, battle_response)
                .unwrap();
        assert_eq!(command.behavior.kind(), K::RequestBattleState);
        assert_eq!(command.behavior.document().as_value()["since_seq"], 120);
        assert_eq!(
            command.battle_side_message,
            BattleSideMessageKind::BattleResync
        );
        let battle_uuid = Uuid::from_u128(0xB4771E);
        let request: PlayerGameClientMessage = serde_json::from_value(serde_json::json!({
            "type":"command","request_id":"battle-authority-001","battle_authority":{"battle_uuid":battle_uuid,"battle_state_revision":7},"behavior":{"type":"pause_battle"}
        })).unwrap();
        let PlayerGameClientMessage::Command {
            behavior,
            battle_authority,
            battle_response,
            ..
        } = request
        else {
            panic!("command")
        };
        let command =
            PlayerBehaviorCommand::from_transport(behavior, battle_authority, battle_response)
                .unwrap();
        assert_eq!(
            command.battle_authority,
            Some(BattleStateAuthorityDto {
                battle_uuid,
                battle_state_revision: 7
            })
        );
        assert!(PlayerBehaviorCommand::from_transport(
            command.behavior,
            command.battle_authority,
            Some(BattleSideMessageKind::BattleResync)
        )
        .is_err());
    }
}
