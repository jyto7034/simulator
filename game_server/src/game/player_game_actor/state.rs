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
    let (result_type, payload) = behavior_result_payload(result)?;

    Ok(PlayerGameServerMessage::CommandResult {
        request_id,
        ok: true,
        result_type: result_type.to_string(),
        payload,
    })
}

fn behavior_result_payload(
    result: BehaviorResult,
) -> Result<(&'static str, Value), PlayerGameActorError> {
    let mapped = match result {
        BehaviorResult::StartNewGame {
            candidates,
            required_count,
        } => (
            "StartNewGame",
            json!({
                "candidates": candidates,
                "required_count": required_count,
            }),
        ),
        BehaviorResult::StarterEmployeesSelected {
            selected_candidate_ids,
            employee_uuids,
            map,
        } => (
            "StarterEmployeesSelected",
            json!({
                "selected_candidate_ids": selected_candidate_ids,
                "employee_uuids": employee_uuids,
                "map": map,
            }),
        ),
        BehaviorResult::MapState { map } => ("MapState", json!({ "map": map })),
        BehaviorResult::NodePreview {
            node_id,
            kind_id,
            category,
            payload,
            session,
            map,
            combat_preview,
            combat_deployment,
            recon_charge,
        } => (
            "NodePreview",
            json!({
                "node_id": node_id,
                "kind_id": kind_id,
                "category": category,
                "payload": payload,
                "session": session,
                "map": map,
                "combat_preview": combat_preview,
                "combat_deployment": combat_deployment,
                "recon_charge": recon_charge,
            }),
        ),
        BehaviorResult::ReconScanUsed {
            node_id,
            remaining_recon_charge,
            combat_preview,
        } => (
            "ReconScanUsed",
            json!({
                "node_id": node_id,
                "remaining_recon_charge": remaining_recon_charge,
                "combat_preview": combat_preview,
            }),
        ),
        BehaviorResult::NodeEntered {
            node_id,
            kind_id,
            category,
            payload,
            session,
            research_deliveries,
        } => (
            "NodeEntered",
            json!({
                "node_id": node_id,
                "kind_id": kind_id,
                "category": category,
                "payload": payload,
                "session": session,
                "research_deliveries": research_deliveries,
            }),
        ),
        BehaviorResult::NodeCompleted { map, outcome } => (
            "NodeCompleted",
            json!({
                "map": map,
                "outcome": outcome,
            }),
        ),
        BehaviorResult::SupportState {
            node_id,
            support_mode,
            support_type,
            choices,
            selected_support_type,
            target_candidates,
            selected_employee_uuid,
            selected_medical_treatment,
            research_deliveries,
        } => (
            "SupportState",
            json!({
                "node_id": node_id,
                "support_mode": support_mode,
                "support_type": support_type,
                "choices": choices,
                "selected_support_type": selected_support_type,
                "target_candidates": target_candidates,
                "selected_employee_uuid": selected_employee_uuid,
                "selected_medical_treatment": selected_medical_treatment,
                "research_deliveries": research_deliveries,
            }),
        ),
        BehaviorResult::ActComplete { act_index, map } => (
            "ActComplete",
            json!({
                "act_index": act_index,
                "map": map,
            }),
        ),
        BehaviorResult::RunComplete { map } => (
            "RunComplete",
            serde_json::to_value(map).map_err(serialize_error)?,
        ),
        BehaviorResult::RunFailed { reason, outcome } => (
            "RunFailed",
            json!({
                "reason": reason,
                "outcome": outcome,
            }),
        ),
        BehaviorResult::UnEquipItem => ("UnEquipItem", Value::Null),
        BehaviorResult::EquipItem { result } => (
            "EquipItem",
            serde_json::to_value(result).map_err(serialize_error)?,
        ),
        BehaviorResult::SkillFragmentLoadoutUpdated {
            employee_uuid,
            equipped_fragment_ids,
        } => (
            "SkillFragmentLoadoutUpdated",
            json!({
                "employee_uuid": employee_uuid,
                "equipped_fragment_ids": equipped_fragment_ids,
            }),
        ),
        BehaviorResult::SkillFragmentUpgraded {
            target_fragment_id,
            material_fragment_id,
            material_remaining_count,
            progress,
        } => (
            "SkillFragmentUpgraded",
            json!({
                "target_fragment_id": target_fragment_id,
                "material_fragment_id": material_fragment_id,
                "material_remaining_count": material_remaining_count,
                "progress": progress,
            }),
        ),
        BehaviorResult::SkillFragmentAwakened {
            target_fragment_id,
            progress,
        } => (
            "SkillFragmentAwakened",
            json!({
                "target_fragment_id": target_fragment_id,
                "progress": progress,
            }),
        ),
        BehaviorResult::SkillFragmentDismantled {
            fragment_id,
            remaining_count,
            dust_gained,
            total_dust,
        } => (
            "SkillFragmentDismantled",
            json!({
                "fragment_id": fragment_id,
                "remaining_count": remaining_count,
                "dust_gained": dust_gained,
                "total_dust": total_dust,
            }),
        ),
        BehaviorResult::EquipmentRestored {
            recipe_id,
            result_equipment_id,
            inventory_diff,
        } => (
            "EquipmentRestored",
            json!({
                "recipe_id": recipe_id,
                "result_equipment_id": result_equipment_id,
                "inventory_diff": inventory_diff,
            }),
        ),
        BehaviorResult::EquipmentDismantled {
            item_uuid,
            equipment_id,
            inventory_diff,
        } => (
            "EquipmentDismantled",
            json!({
                "item_uuid": item_uuid,
                "equipment_id": equipment_id,
                "inventory_diff": inventory_diff,
            }),
        ),
        BehaviorResult::EquipmentEnhanced {
            item_uuid,
            equipment_id,
            enhancement_level,
            inventory_diff,
        } => (
            "EquipmentEnhanced",
            json!({
                "item_uuid": item_uuid,
                "equipment_id": equipment_id,
                "enhancement_level": enhancement_level,
                "inventory_diff": inventory_diff,
            }),
        ),
        BehaviorResult::MoveUnit => ("MoveUnit", Value::Null),
        BehaviorResult::MoveBenchUnit => ("MoveBenchUnit", Value::Null),
        BehaviorResult::ShopState {
            shop,
            research_deliveries,
        } => (
            "ShopState",
            json!({
                "shop": shop,
                "research_deliveries": research_deliveries,
            }),
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
        BehaviorResult::RewardGranted {
            enkephalin,
            inventory_diff,
        } => (
            "RewardGranted",
            json!({
                "enkephalin": enkephalin,
                "inventory_diff": inventory_diff,
            }),
        ),
        BehaviorResult::CombatRewardsGranted {
            enkephalin,
            inventory_diff,
            outcome,
            completion,
        } => {
            let (completion_type, completion_payload) = behavior_result_payload(*completion)?;
            (
                "CombatRewardsGranted",
                json!({
                    "enkephalin": enkephalin,
                    "inventory_diff": inventory_diff,
                    "outcome": outcome,
                    "completion": {
                        "result_type": completion_type,
                        "payload": completion_payload,
                    },
                }),
            )
        }
        BehaviorResult::CombatResolved { winner, timeline } => (
            "CombatResolved",
            serde_json::to_value(compress_timeline_payload(winner, &timeline)?)
                .map_err(serialize_error)?,
        ),
        BehaviorResult::RewardState {
            mode,
            rewards,
            selected_reward_uuid,
            research_deliveries,
        } => (
            "RewardState",
            json!({
                "mode": mode,
                "rewards": rewards,
                "selected_reward_uuid": selected_reward_uuid,
                "research_deliveries": research_deliveries,
            }),
        ),
        BehaviorResult::Ok => ("Ok", Value::Null),
    };

    Ok(mapped)
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
