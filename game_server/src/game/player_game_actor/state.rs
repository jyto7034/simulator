use base64::{engine::general_purpose::STANDARD, Engine as _};
use flate2::{write::GzEncoder, Compression};
use game_core::game::{
    battle::{timeline::Timeline, types::BattleWinner},
    behavior::{BehaviorResult, GameError},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;

use super::messages::PlayerGameServerMessage;
use crate::shared::protocol::{ErrorCode, ServerMessage};

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
            GameError::NotInShopState => {
                ("not_in_shop_state", "Player is not in shop state".into())
            }
            GameError::NotInBonusState => {
                ("not_in_bonus_state", "Player is not in bonus state".into())
            }
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
            GameError::InsufficientResources => {
                ("insufficient_resources", "Insufficient resources".into())
            }
            GameError::PhaseNotReady => ("phase_not_ready", "Phase is not ready".into()),
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
            GameError::UnitAlreadyPlaced => (
                "unit_already_placed",
                "Unit is already placed on the field".into(),
            ),
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
        };

        Self { code, message }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedTimelinePayload {
    pub winner: BattleWinner,
    pub timeline_encoding: String,
    pub timeline_gzip_base64: String,
    pub raw_json_bytes: usize,
    pub gzip_bytes: usize,
}

pub fn compress_timeline_payload(
    winner: BattleWinner,
    timeline: &Timeline,
) -> Result<CompressedTimelinePayload, PlayerGameActorError> {
    let timeline_json = timeline
        .to_json_string()
        .map_err(|error| PlayerGameActorError::new("serialization_failed", error.to_string()))?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(timeline_json.as_bytes())
        .map_err(|error| PlayerGameActorError::new("compression_failed", error.to_string()))?;
    let compressed = encoder
        .finish()
        .map_err(|error| PlayerGameActorError::new("compression_failed", error.to_string()))?;

    Ok(CompressedTimelinePayload {
        winner,
        timeline_encoding: "gzip+base64".to_string(),
        timeline_gzip_base64: STANDARD.encode(&compressed),
        raw_json_bytes: timeline_json.len(),
        gzip_bytes: compressed.len(),
    })
}

pub fn behavior_result_to_command_result(
    request_id: String,
    result: BehaviorResult,
) -> Result<PlayerGameServerMessage, PlayerGameActorError> {
    let (result_type, payload) = match result {
        BehaviorResult::StartNewGame => ("StartNewGame", Value::Null),
        BehaviorResult::RequestPhaseData(event) => (
            "RequestPhaseData",
            serde_json::to_value(event).map_err(serialize_error)?,
        ),
        BehaviorResult::EventSelected => ("EventSelected", Value::Null),
        BehaviorResult::UnEquipItem => ("UnEquipItem", Value::Null),
        BehaviorResult::EquipItem => ("EquipItem", Value::Null),
        BehaviorResult::MoveUnit => ("MoveUnit", Value::Null),
        BehaviorResult::TransferUnit => ("TransferUnit", Value::Null),
        BehaviorResult::ShopState { shop } => (
            "ShopState",
            serde_json::to_value(shop).map_err(serialize_error)?,
        ),
        BehaviorResult::RerollShop { new_items } => {
            ("RerollShop", json!({ "new_items": new_items }))
        }
        BehaviorResult::SellItem {
            enkephalin,
            inventory_diff,
        } => (
            "SellItem",
            json!({
                "enkephalin": enkephalin,
                "inventory_diff": inventory_diff,
            }),
        ),
        BehaviorResult::PurchaseItem {
            enkephalin,
            inventory_diff,
        } => (
            "PurchaseItem",
            json!({
                "enkephalin": enkephalin,
                "inventory_diff": inventory_diff,
            }),
        ),
        BehaviorResult::RandomEventState { event } => (
            "RandomEventState",
            serde_json::to_value(event).map_err(serialize_error)?,
        ),
        BehaviorResult::BonusReward {
            enkephalin,
            inventory_diff,
        } => (
            "BonusReward",
            json!({
                "enkephalin": enkephalin,
                "inventory_diff": inventory_diff,
            }),
        ),
        BehaviorResult::SuppressAbnormality { winner, timeline } => (
            "SuppressAbnormality",
            serde_json::to_value(compress_timeline_payload(winner, &timeline)?)
                .map_err(serialize_error)?,
        ),
        BehaviorResult::RewardState {
            mode,
            rewards,
            selected_reward_uuid,
        } => (
            "RewardState",
            json!({
                "mode": mode,
                "rewards": rewards,
                "selected_reward_uuid": selected_reward_uuid,
            }),
        ),
        BehaviorResult::Ordeal { battle_result } => {
            ("Ordeal", json!({ "battle_result": battle_result }))
        }
        BehaviorResult::AdvancePhase { next_phase_event } => (
            "AdvancePhase",
            json!({ "next_phase_event": next_phase_event }),
        ),
        BehaviorResult::Ok => ("Ok", Value::Null),
    };

    Ok(PlayerGameServerMessage::CommandResult {
        request_id,
        ok: true,
        result_type: result_type.to_string(),
        payload,
    })
}

fn serialize_error(error: serde_json::Error) -> PlayerGameActorError {
    PlayerGameActorError::new("serialization_failed", error.to_string())
}

pub fn legacy_server_message_to_unity(message: ServerMessage) -> PlayerGameServerMessage {
    match message {
        ServerMessage::EnQueued { pod_id } => PlayerGameServerMessage::Notification {
            notification_type: "enqueued".to_string(),
            payload: json!({ "pod_id": pod_id }),
        },
        ServerMessage::DeQueued => PlayerGameServerMessage::Notification {
            notification_type: "dequeued".to_string(),
            payload: Value::Null,
        },
        ServerMessage::MatchFound {
            winner_id,
            opponent_id,
            battle_data,
        } => PlayerGameServerMessage::Notification {
            notification_type: "match_found".to_string(),
            payload: json!({
                "winner_id": winner_id,
                "opponent_id": opponent_id,
                "battle_data": battle_data,
            }),
        },
        ServerMessage::Error { code, message } => PlayerGameServerMessage::Error {
            request_id: None,
            code: legacy_error_code_to_string(code),
            message,
        },
    }
}

fn legacy_error_code_to_string(code: ErrorCode) -> String {
    serde_json::to_value(code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "internal_error".to_string())
}
