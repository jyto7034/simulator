use base64::{engine::general_purpose::STANDARD, Engine as _};
use flate2::{write::GzEncoder, Compression};
use game_core::game::{
    battle::{event_log::BattleEventLog, types::BattleWinner},
    behavior::{
        BehaviorCommandResultContract, BehaviorResult, CombatResultEventLogAttachmentDto,
        GameError, LiveBattleUpdateDto, RunSnapshotDto, SelectedEventSnapshotDto,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;

use super::messages::{BattleSideMessageKind, PlayerGameServerMessage};

#[derive(Debug, Clone)]
pub struct PlayerGameActorError {
    pub code: &'static str,
    pub message: String,
}

impl PlayerGameActorError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn to_server_message(&self, request_id: Option<String>) -> PlayerGameServerMessage {
        PlayerGameServerMessage::Error {
            request_id,
            code: self.code.to_string(),
            message: self.message.clone(),
        }
    }
}

impl From<GameError> for PlayerGameActorError {
    fn from(value: GameError) -> Self {
        let (code, message) = match value {
            GameError::EventNotFound => ("event_not_found", "Selected event was not found".into()),
            GameError::EventTypeMismatch => (
                "event_type_mismatch",
                "Selected event type does not match current state".into(),
            ),
            GameError::InvalidAction => (
                "invalid_action",
                "Action is not allowed in the current state".into(),
            ),
            GameError::InvalidBattleResyncSeq { requested, latest } => (
                "invalid_battle_resync_seq",
                format!(
                    "Battle resync since_seq {} is ahead of latest seq {}",
                    requested, latest
                ),
            ),
            GameError::NotInShopState => {
                ("not_in_shop_state", "Player is not in shop state".into())
            }
            GameError::NotInRewardState => (
                "not_in_reward_state",
                "Player is not in reward state".into(),
            ),
            GameError::ShopRerollNotAllowed => (
                "shop_reroll_not_allowed",
                "Current shop does not support reroll".into(),
            ),
            GameError::ShopItemNotFound => {
                ("shop_item_not_found", "Shop item was not found".into())
            }
            GameError::InventoryFull => ("inventory_full", "Inventory is full".into()),
            GameError::InventoryItemNotFound => (
                "inventory_item_not_found",
                "Inventory item was not found".into(),
            ),
            GameError::InventoryItemNotRemovable => (
                "inventory_item_not_removable",
                "Inventory item cannot be removed through this action".into(),
            ),
            GameError::InsufficientResources => {
                ("insufficient_resources", "Insufficient resources".into())
            }
            GameError::MissingResource(resource) => (
                "missing_resource",
                format!("Missing core resource: {resource}"),
            ),
            GameError::InvalidUnitStats(message) => (
                "invalid_unit_stats",
                format!("Invalid unit stats: {message}"),
            ),
            GameError::OutOfBounds => (
                "out_of_bounds",
                "Requested position is out of bounds".into(),
            ),
            GameError::PositionOccupied => (
                "position_occupied",
                "Requested position is already occupied".into(),
            ),
            GameError::StaticObstacleBlocked => (
                "static_obstacle_blocked",
                "Requested tile is blocked by a static obstacle".into(),
            ),
            GameError::UnitAlreadyPlaced => {
                ("unit_already_placed", "Unit is already deployed".into())
            }
            GameError::UnitNotFound => ("unit_not_found", "Unit was not found".into()),
            GameError::AlreadyOwnedArtifact => (
                "already_owned_artifact",
                "Artifact is already owned and cannot be acquired again".into(),
            ),
            GameError::InvalidStaticData(message) => (
                "invalid_static_data",
                format!("Invalid static data: {message}"),
            ),
            GameError::NotImplemented(feature) => (
                "not_implemented",
                format!("Feature is not implemented: {feature}"),
            ),
            GameError::SkillFragmentIncompatible {
                fragment_id,
                failure_codes,
            } => (
                "skill_fragment_incompatible",
                format!(
                    "Skill fragment '{fragment_id}' is incompatible with the selected employee: {:?}",
                    failure_codes
                ),
            ),
        };

        Self { code, message }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedBattleEventLogPayload {
    pub winner: BattleWinner,
    /// Current event-log transport field. The payload is a
    /// compressed battle event log, not a precomputed offline replay.
    pub event_log_encoding: String,
    /// Current event-log transport field.
    pub event_log_gzip_base64: String,
    pub raw_json_bytes: usize,
    pub gzip_bytes: usize,
}

pub fn compress_battle_event_log_payload(
    winner: BattleWinner,
    event_log: &BattleEventLog,
) -> Result<CompressedBattleEventLogPayload, PlayerGameActorError> {
    let event_log_json = event_log
        .to_json_string()
        .map_err(|error| PlayerGameActorError::new("serialization_failed", error.to_string()))?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(event_log_json.as_bytes())
        .map_err(|error| PlayerGameActorError::new("compression_failed", error.to_string()))?;
    let compressed = encoder
        .finish()
        .map_err(|error| PlayerGameActorError::new("compression_failed", error.to_string()))?;

    Ok(CompressedBattleEventLogPayload {
        winner,
        event_log_encoding: "gzip+base64".to_string(),
        event_log_gzip_base64: STANDARD.encode(&compressed),
        raw_json_bytes: event_log_json.len(),
        gzip_bytes: compressed.len(),
    })
}

pub fn compress_combat_result_event_log_attachment(
    attachment: &CombatResultEventLogAttachmentDto,
) -> Result<CompressedBattleEventLogPayload, PlayerGameActorError> {
    compress_battle_event_log_payload(attachment.winner, &attachment.event_log)
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerSelectedEventSnapshotDto {
    #[serde(flatten)]
    pub event: SelectedEventSnapshotDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed_event_log: Option<CompressedBattleEventLogPayload>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerStateSnapshotDto {
    #[serde(flatten)]
    core: RunSnapshotDto<PlayerSelectedEventSnapshotDto>,
}

impl PlayerStateSnapshotDto {
    pub fn new(
        core: RunSnapshotDto,
        compressed_event_log: Option<CompressedBattleEventLogPayload>,
    ) -> Self {
        let core = core.map_selected_event(|selected_event| {
            selected_event.map(|event| {
                let compressed_event_log = match event {
                    SelectedEventSnapshotDto::CombatBattle { .. } => compressed_event_log,
                    _ => None,
                };

                PlayerSelectedEventSnapshotDto {
                    event,
                    compressed_event_log,
                }
            })
        });

        Self { core }
    }
}

#[derive(Debug)]
pub struct CommandResultMapping {
    pub response: PlayerGameServerMessage,
    pub side_messages: Vec<PlayerGameServerMessage>,
    pub send_state_snapshot: bool,
}

pub fn behavior_result_to_command_result(
    request_id: String,
    result: BehaviorResult,
    battle_side_message: BattleSideMessageKind,
) -> Result<CommandResultMapping, PlayerGameActorError> {
    match result.into_command_result_contract() {
        BehaviorCommandResultContract::BattleUpdate(battle_result) => {
            let accepted_at_battle_time_ms = battle_result.battle_update.server_battle_time_ms;
            let mut side_messages = Vec::new();
            if battle_side_message == BattleSideMessageKind::BattleUpdate {
                if let Some(setup) = battle_result.battle_setup_snapshot {
                    side_messages.push(PlayerGameServerMessage::battle_setup_snapshot(setup));
                }
            }
            let side_message = match battle_side_message {
                BattleSideMessageKind::BattleUpdate => {
                    PlayerGameServerMessage::battle_update(battle_result.battle_update)
                }
                BattleSideMessageKind::BattleResync => {
                    PlayerGameServerMessage::battle_resync(battle_result.battle_update)
                }
            };
            side_messages.push(side_message);
            Ok(CommandResultMapping {
                response: PlayerGameServerMessage::CommandResult {
                    request_id: request_id.clone(),
                    ok: true,
                    result_type: "CommandAccepted".to_string(),
                    payload: json!({
                        "command_id": request_id,
                        "accepted_at_battle_time_ms": accepted_at_battle_time_ms,
                    }),
                },
                side_messages,
                send_state_snapshot: false,
            })
        }
        BehaviorCommandResultContract::CommandPayload(payload_contract) => {
            let (result_type, payload) = command_payload_contract_to_result(payload_contract)?;
            Ok(CommandResultMapping {
                response: PlayerGameServerMessage::CommandResult {
                    request_id,
                    ok: true,
                    result_type,
                    payload,
                },
                side_messages: Vec::new(),
                send_state_snapshot: true,
            })
        }
        BehaviorCommandResultContract::ReadOnlyPreview(payload_contract) => {
            let (result_type, payload) = command_payload_contract_to_result(payload_contract)?;
            Ok(CommandResultMapping {
                response: PlayerGameServerMessage::CommandResult {
                    request_id,
                    ok: true,
                    result_type,
                    payload,
                },
                side_messages: Vec::new(),
                send_state_snapshot: false,
            })
        }
    }
}

fn command_payload_contract_to_result(
    payload_contract: game_core::game::behavior::CommandResultPayloadContract,
) -> Result<(String, Value), PlayerGameActorError> {
    let result_type = payload_contract.result_type().to_string();
    let payload = payload_contract.payload_value().map_err(serialize_error)?;
    Ok((result_type, payload))
}

pub(crate) fn battle_update_from_behavior_result(
    result: &BehaviorResult,
) -> Option<LiveBattleUpdateDto> {
    match result {
        BehaviorResult::BattleAdvanced { battle_update, .. }
        | BehaviorResult::BattleState { battle_update, .. }
        | BehaviorResult::BattlePlaybackChanged { battle_update, .. }
        | BehaviorResult::BattleUnitDeployed { battle_update, .. }
        | BehaviorResult::BattleUnitWithdrawn { battle_update, .. }
        | BehaviorResult::BattleSkillActivated { battle_update, .. } => Some(battle_update.clone()),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn behavior_result_payload(
    result: BehaviorResult,
) -> Result<(&'static str, Value), PlayerGameActorError> {
    let contract = result.into_command_result_contract();
    let payload = match contract {
        BehaviorCommandResultContract::CommandPayload(payload)
        | BehaviorCommandResultContract::ReadOnlyPreview(payload) => payload,
        BehaviorCommandResultContract::BattleUpdate(_) => {
            return Err(PlayerGameActorError::new(
                "battle_update_transport_required",
                "Battle behavior results must be sent through battle_update transport",
            ));
        }
    };
    let result_type = payload.result_type();
    let payload = payload.payload_value().map_err(serialize_error)?;
    Ok((result_type, payload))
}

fn serialize_error(error: serde_json::Error) -> PlayerGameActorError {
    PlayerGameActorError::new("serialization_failed", error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player_game_actor::messages::BattleSideMessageKind;
    use game_core::game::{
        behavior::{
            BattlePlaybackState, DeploymentRangePreviewFacingDto, DeploymentRangePreviewFacingsDto,
            DeploymentRangePreviewResultDto, LiveBattleActiveSkillRangePreviewDto,
            LiveBattleBasicAttackRangePreviewDto, LiveBattleDeploymentDto, LiveBattleEventDeltaDto,
            LiveBattleRangePreviewReason, LiveBattleRangePreviewSource, LiveBattleRangePreviewsDto,
            LiveBattleStateCheckpointDto, LiveBattleUpdateDto, LiveBattleUpdateMessageType,
        },
        combat_preview::{CombatMissionVariant, CombatNodeType},
        resources::Position,
    };
    use uuid::Uuid;

    #[test]
    fn static_obstacle_blocked_uses_static_obstacle_error_code() {
        let error = PlayerGameActorError::from(GameError::StaticObstacleBlocked);

        assert_eq!(error.code, "static_obstacle_blocked");
        assert!(error.message.contains("static obstacle"));
    }

    fn dummy_battle_update(
        battle_uuid: Uuid,
        battle_time_ms: u64,
        to_seq: u64,
        deployment: Option<LiveBattleDeploymentDto>,
    ) -> LiveBattleUpdateDto {
        LiveBattleUpdateDto {
            message_type: LiveBattleUpdateMessageType::BattleUpdate,
            battle_uuid,
            server_battle_time_ms: battle_time_ms,
            events_delta: LiveBattleEventDeltaDto {
                after_seq: to_seq,
                to_seq,
                events: Vec::new(),
            },
            checkpoint: LiveBattleStateCheckpointDto {
                at_seq: to_seq,
                battle_time_ms,
                playback: BattlePlaybackState::default(),
                units: Vec::new(),
                deployment,
            },
        }
    }

    fn dummy_battle_advanced_result() -> BehaviorResult {
        let battle_uuid = Uuid::from_u128(0xB4771E);
        BehaviorResult::BattleAdvanced {
            battle_uuid,
            encounter_id: "defend_black_box_relay".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            battle_setup_snapshot: None,
            battle_update: dummy_battle_update(battle_uuid, 3_000, 12, None),
            finished: false,
        }
    }

    fn dummy_range_previews() -> LiveBattleRangePreviewsDto {
        LiveBattleRangePreviewsDto {
            basic_attack: LiveBattleBasicAttackRangePreviewDto {
                available: true,
                reason: None,
                cells: vec![Position::new(2, 2), Position::new(3, 2)],
                source: LiveBattleRangePreviewSource::FallbackBasicAttack,
                source_id: None,
            },
            active_skill: LiveBattleActiveSkillRangePreviewDto {
                available: false,
                reason: Some(LiveBattleRangePreviewReason::NoSkill),
                skill_id: None,
                targeting_kind: None,
                requires_manual_target: false,
                cast_cells: Vec::new(),
                effect_preview_cells: Vec::new(),
                source: LiveBattleRangePreviewSource::Unavailable,
                source_id: None,
            },
        }
    }

    #[test]
    fn behavior_result_payload_rejects_battle_legacy_transport() {
        let err = behavior_result_payload(dummy_battle_advanced_result())
            .expect_err("battle results must not serialize through legacy command payloads");

        assert_eq!(err.code, "battle_update_transport_required");
    }

    #[test]
    fn behavior_result_to_command_result_uses_battle_update_side_message() {
        let mapped = behavior_result_to_command_result(
            "advance-1".to_string(),
            dummy_battle_advanced_result(),
            BattleSideMessageKind::BattleUpdate,
        )
        .expect("battle result should map to command ack plus side update");

        match mapped.response {
            PlayerGameServerMessage::CommandResult {
                result_type,
                payload,
                ..
            } => {
                assert_eq!(result_type, "CommandAccepted");
                assert_eq!(payload["command_id"], "advance-1");
            }
            other => panic!("expected command result, got {other:?}"),
        }
        assert!(matches!(
            mapped.side_messages.as_slice(),
            [PlayerGameServerMessage::BattleUpdate {
                server_battle_time_ms: 3_000,
                events_delta,
                checkpoint,
                ..
            }] if events_delta.to_seq == 12 && checkpoint.at_seq == 12
        ));
        assert!(
            !mapped.send_state_snapshot,
            "battle side-message command results must not request a trailing state_snapshot"
        );
    }

    #[test]
    fn deployment_range_preview_result_does_not_request_state_snapshot() {
        let range_previews = dummy_range_previews();
        let result = BehaviorResult::DeploymentRangePreview {
            result: DeploymentRangePreviewResultDto {
                employee_uuid: Uuid::from_u128(7),
                position: Position::new(2, 2),
                facings: DeploymentRangePreviewFacingsDto {
                    up: DeploymentRangePreviewFacingDto {
                        range_previews: range_previews.clone(),
                    },
                    right: DeploymentRangePreviewFacingDto {
                        range_previews: range_previews.clone(),
                    },
                    down: DeploymentRangePreviewFacingDto {
                        range_previews: range_previews.clone(),
                    },
                    left: DeploymentRangePreviewFacingDto { range_previews },
                },
            },
        };

        let mapped = behavior_result_to_command_result(
            "preview-1".to_string(),
            result,
            BattleSideMessageKind::BattleUpdate,
        )
        .expect("preview result should serialize");

        match mapped.response {
            PlayerGameServerMessage::CommandResult {
                result_type,
                payload,
                ..
            } => {
                assert_eq!(result_type, "DeploymentRangePreview");
                assert_eq!(payload["position"], serde_json::json!({ "x": 2, "y": 2 }));
                assert_eq!(
                    payload["facings"]["right"]["range_previews"]["basic_attack"]["cells"],
                    serde_json::json!([{ "x": 2, "y": 2 }, { "x": 3, "y": 2 }])
                );
            }
            other => panic!("expected command result, got {other:?}"),
        }
        assert!(mapped.side_messages.is_empty());
        assert!(
            !mapped.send_state_snapshot,
            "preview read request must not be followed by full state_snapshot"
        );
    }
}
