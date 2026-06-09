use base64::{engine::general_purpose::STANDARD, Engine as _};
use flate2::{write::GzEncoder, Compression};
use game_core::game::{
    ability::SkillId,
    battle::{
        ids::UnitInstanceId,
        timeline::{Timeline, TimelineEntry},
        types::BattleWinner,
    },
    behavior::{BattlePlaybackState, BehaviorResult, GameError, LiveBattleDeploymentDto},
    combat_preview::{CombatMissionVariant, CombatNodeType, CombatPreview},
    data::skill_fragment_data::SkillFragmentId,
    employee::ActiveConsumableModifier,
    resources::InventoryDiffDto,
    skill_fragment::SkillFragmentProgress,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;

use super::messages::PlayerGameServerMessage;

#[derive(Debug, Serialize)]
struct ConsumableItemUsedPayload {
    item_uuid: uuid::Uuid,
    target_employee_uuid: uuid::Uuid,
    replaced_modifier: Option<ActiveConsumableModifier>,
    applied_modifier: ActiveConsumableModifier,
    inventory_diff: InventoryDiffDto,
}

#[derive(Debug, Serialize)]
struct SkillFragmentUpgradedPayload {
    target_fragment_id: SkillFragmentId,
    dust_spent: u32,
    remaining_dust: u32,
    progress: SkillFragmentProgress,
}

#[derive(Debug, Serialize)]
struct SkillFragmentAwakenedPayload {
    target_fragment_id: SkillFragmentId,
    dust_spent: u32,
    remaining_dust: u32,
    progress: SkillFragmentProgress,
}

#[derive(Debug, Serialize)]
struct BattleAdvancedPayload {
    battle_uuid: uuid::Uuid,
    encounter_id: String,
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    playback: BattlePlaybackState,
    battle_time_ms: u64,
    timeline_delta: Vec<TimelineEntry>,
    last_timeline_seq: u64,
    finished: bool,
    deployment: Option<LiveBattleDeploymentDto>,
}

#[derive(Debug, Serialize)]
struct BattleStatePayload {
    battle_uuid: uuid::Uuid,
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    encounter_id: String,
    combat_preview: CombatPreview,
    playback: BattlePlaybackState,
    battle_time_ms: u64,
    timeline_delta: Vec<TimelineEntry>,
    last_timeline_seq: u64,
    finished: bool,
    deployment: Option<LiveBattleDeploymentDto>,
}

#[derive(Debug, Serialize)]
struct BattlePlaybackChangedPayload {
    battle_uuid: uuid::Uuid,
    encounter_id: String,
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    playback: BattlePlaybackState,
    battle_time_ms: u64,
    last_timeline_seq: u64,
    deployment: Option<LiveBattleDeploymentDto>,
}

#[derive(Debug, Serialize)]
struct BattleUnitDeployedPayload {
    battle_uuid: uuid::Uuid,
    encounter_id: String,
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    playback: BattlePlaybackState,
    employee_uuid: uuid::Uuid,
    unit_instance_id: UnitInstanceId,
    timeline_delta: Vec<TimelineEntry>,
    last_timeline_seq: u64,
    deployment: LiveBattleDeploymentDto,
}

#[derive(Debug, Serialize)]
struct BattleUnitWithdrawnPayload {
    battle_uuid: uuid::Uuid,
    encounter_id: String,
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    playback: BattlePlaybackState,
    employee_uuid: uuid::Uuid,
    unit_instance_id: UnitInstanceId,
    timeline_delta: Vec<TimelineEntry>,
    last_timeline_seq: u64,
    deployment: LiveBattleDeploymentDto,
}

#[derive(Debug, Serialize)]
struct BattleSkillActivatedPayload {
    battle_uuid: uuid::Uuid,
    encounter_id: String,
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    playback: BattlePlaybackState,
    employee_uuid: uuid::Uuid,
    unit_instance_id: UnitInstanceId,
    skill_id: SkillId,
    timeline_delta: Vec<TimelineEntry>,
    last_timeline_seq: u64,
    deployment: LiveBattleDeploymentDto,
}

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
    /// Legacy field name kept for the Unity JSON contract. The payload is a
    /// compressed battle event log, not a precomputed offline replay.
    pub timeline_encoding: String,
    /// Legacy field name kept for the Unity JSON contract.
    pub timeline_gzip_base64: String,
    pub raw_json_bytes: usize,
    pub gzip_bytes: usize,
}

pub fn compress_battle_event_log_payload(
    winner: BattleWinner,
    event_log: &Timeline,
) -> Result<CompressedBattleEventLogPayload, PlayerGameActorError> {
    let timeline_json = event_log
        .to_json_string()
        .map_err(|error| PlayerGameActorError::new("serialization_failed", error.to_string()))?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(timeline_json.as_bytes())
        .map_err(|error| PlayerGameActorError::new("compression_failed", error.to_string()))?;
    let compressed = encoder
        .finish()
        .map_err(|error| PlayerGameActorError::new("compression_failed", error.to_string()))?;

    Ok(CompressedBattleEventLogPayload {
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

pub(crate) fn behavior_result_payload(
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
        BehaviorResult::MaintenanceState {
            node_id,
            maintenance_options,
            research_deliveries,
        } => (
            "MaintenanceState",
            json!({
                "node_id": node_id,
                "maintenance_options": maintenance_options,
                "research_deliveries": research_deliveries,
            }),
        ),
        BehaviorResult::HeadquartersContactState {
            node_id,
            options,
            recruitment_candidates,
            shop_pool_id,
            research_deliveries,
        } => (
            "HeadquartersContactState",
            json!({
                "node_id": node_id,
                "options": options,
                "recruitment_candidates": recruitment_candidates,
                "shop_pool_id": shop_pool_id,
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
        BehaviorResult::UnEquipItem { result } => (
            "UnEquipItem",
            serde_json::to_value(result).map_err(serialize_error)?,
        ),
        BehaviorResult::EquipItem { result } => (
            "EquipItem",
            serde_json::to_value(result).map_err(serialize_error)?,
        ),
        BehaviorResult::ConsumableItemUsed {
            item_uuid,
            target_employee_uuid,
            replaced_modifier,
            applied_modifier,
            inventory_diff,
        } => {
            let payload = ConsumableItemUsedPayload {
                item_uuid,
                target_employee_uuid,
                replaced_modifier,
                applied_modifier,
                inventory_diff,
            };
            (
                "ConsumableItemUsed",
                serde_json::to_value(payload).map_err(|error| {
                    PlayerGameActorError::new("serialization_failed", error.to_string())
                })?,
            )
        }
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
            dust_spent,
            remaining_dust,
            progress,
        } => {
            let payload = SkillFragmentUpgradedPayload {
                target_fragment_id,
                dust_spent,
                remaining_dust,
                progress,
            };
            (
                "SkillFragmentUpgraded",
                serde_json::to_value(payload).map_err(serialize_error)?,
            )
        }
        BehaviorResult::SkillFragmentAwakened {
            target_fragment_id,
            dust_spent,
            remaining_dust,
            progress,
        } => {
            let payload = SkillFragmentAwakenedPayload {
                target_fragment_id,
                dust_spent,
                remaining_dust,
                progress,
            };
            (
                "SkillFragmentAwakened",
                serde_json::to_value(payload).map_err(serialize_error)?,
            )
        }
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
        BehaviorResult::MoveRosterUnit { roster_slots } => (
            "MoveRosterUnit",
            json!({
                "roster_slots": roster_slots,
            }),
        ),
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
        BehaviorResult::EmployeeRecruited {
            candidate_id,
            employee_uuid,
            completion,
        } => {
            let (completion_type, completion_payload) = behavior_result_payload(*completion)?;
            (
                "EmployeeRecruited",
                json!({
                    "candidate_id": candidate_id,
                    "employee_uuid": employee_uuid,
                    "completion": {
                        "result_type": completion_type,
                        "payload": completion_payload,
                    },
                }),
            )
        }
        BehaviorResult::EmergencySuppliesGranted {
            enkephalin,
            inventory_diff,
            completion,
        } => {
            let (completion_type, completion_payload) = behavior_result_payload(*completion)?;
            (
                "EmergencySuppliesGranted",
                json!({
                    "enkephalin": enkephalin,
                    "inventory_diff": inventory_diff,
                    "completion": {
                        "result_type": completion_type,
                        "payload": completion_payload,
                    },
                }),
            )
        }
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
        BehaviorResult::BattleAdvanced {
            battle_uuid,
            encounter_id,
            node_type,
            mission_variant,
            playback,
            battle_time_ms,
            timeline_delta,
            last_timeline_seq,
            finished,
            deployment,
        } => {
            let payload = BattleAdvancedPayload {
                battle_uuid,
                encounter_id,
                node_type,
                mission_variant,
                playback,
                battle_time_ms,
                timeline_delta,
                last_timeline_seq,
                finished,
                deployment,
            };
            (
                "BattleAdvanced",
                serde_json::to_value(payload).map_err(serialize_error)?,
            )
        }
        BehaviorResult::BattleState {
            battle_uuid,
            node_type,
            mission_variant,
            encounter_id,
            combat_preview,
            playback,
            battle_time_ms,
            timeline_delta,
            last_timeline_seq,
            finished,
            deployment,
        } => {
            let payload = BattleStatePayload {
                battle_uuid,
                node_type,
                mission_variant,
                encounter_id,
                combat_preview,
                playback,
                battle_time_ms,
                timeline_delta,
                last_timeline_seq,
                finished,
                deployment,
            };
            (
                "BattleState",
                serde_json::to_value(payload).map_err(serialize_error)?,
            )
        }
        BehaviorResult::BattlePlaybackChanged {
            battle_uuid,
            encounter_id,
            node_type,
            mission_variant,
            playback,
            battle_time_ms,
            last_timeline_seq,
            deployment,
        } => {
            let payload = BattlePlaybackChangedPayload {
                battle_uuid,
                encounter_id,
                node_type,
                mission_variant,
                playback,
                battle_time_ms,
                last_timeline_seq,
                deployment,
            };
            (
                "BattlePlaybackChanged",
                serde_json::to_value(payload).map_err(serialize_error)?,
            )
        }
        BehaviorResult::BattleUnitDeployed {
            battle_uuid,
            encounter_id,
            node_type,
            mission_variant,
            playback,
            employee_uuid,
            unit_instance_id,
            timeline_delta,
            last_timeline_seq,
            deployment,
        } => {
            let payload = BattleUnitDeployedPayload {
                battle_uuid,
                encounter_id,
                node_type,
                mission_variant,
                playback,
                employee_uuid,
                unit_instance_id,
                timeline_delta,
                last_timeline_seq,
                deployment,
            };
            (
                "BattleUnitDeployed",
                serde_json::to_value(payload).map_err(serialize_error)?,
            )
        }
        BehaviorResult::BattleUnitWithdrawn {
            battle_uuid,
            encounter_id,
            node_type,
            mission_variant,
            playback,
            employee_uuid,
            unit_instance_id,
            timeline_delta,
            last_timeline_seq,
            deployment,
        } => {
            let payload = BattleUnitWithdrawnPayload {
                battle_uuid,
                encounter_id,
                node_type,
                mission_variant,
                playback,
                employee_uuid,
                unit_instance_id,
                timeline_delta,
                last_timeline_seq,
                deployment,
            };
            (
                "BattleUnitWithdrawn",
                serde_json::to_value(payload).map_err(serialize_error)?,
            )
        }
        BehaviorResult::BattleSkillActivated {
            battle_uuid,
            encounter_id,
            node_type,
            mission_variant,
            playback,
            employee_uuid,
            unit_instance_id,
            skill_id,
            timeline_delta,
            last_timeline_seq,
            deployment,
        } => {
            let payload = BattleSkillActivatedPayload {
                battle_uuid,
                encounter_id,
                node_type,
                mission_variant,
                playback,
                employee_uuid,
                unit_instance_id,
                skill_id,
                timeline_delta,
                last_timeline_seq,
                deployment,
            };
            (
                "BattleSkillActivated",
                serde_json::to_value(payload).map_err(serialize_error)?,
            )
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::game::{
        battle::{
            core::movement::types::TimelineVec2,
            damage::{DamageFeedbackTag, DamageSource, DamageType},
            ids::UnitInstanceId,
            tile_range::FacingDirection,
            timeline::{HpChangeReason, TimelineCause, TimelineEntry, TimelineEvent},
            types::{BattleUnitRole, MobilityKind},
        },
        behavior::{
            BattlePlaybackState, LiveBattleDeployedUnitDto, LiveBattleDeploymentDto,
            LiveBattleUnitDeployCostDto,
        },
        combat_preview::{CombatMissionVariant, CombatNodeType},
        enums::Side,
        stats::UnitStats,
    };
    use uuid::Uuid;

    #[test]
    fn battle_advanced_payload_preserves_mission_identity() {
        let (result_type, payload) = behavior_result_payload(BehaviorResult::BattleAdvanced {
            battle_uuid: Uuid::from_u128(0xB4771E),
            encounter_id: "defend_black_box_relay".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Encirclement,
            playback: BattlePlaybackState::default(),
            battle_time_ms: 3_000,
            timeline_delta: Vec::new(),
            last_timeline_seq: 12,
            finished: false,
            deployment: None,
        })
        .expect("battle advanced result should serialize");

        assert_eq!(result_type, "BattleAdvanced");
        assert_eq!(payload["encounter_id"], "defend_black_box_relay");
        assert_eq!(payload["node_type"], "Defense");
        assert_eq!(payload["mission_variant"], "Encirclement");
        assert_eq!(payload["playback"]["paused"], false);
        assert_eq!(payload["playback"]["speed"], "X1");
        assert_eq!(payload["battle_time_ms"], 3_000);
        assert_eq!(payload["last_timeline_seq"], 12);
    }

    #[test]
    fn battle_advanced_payload_preserves_timeline_contract_fields() {
        let source = UnitInstanceId(Uuid::from_u128(0xA11CE));
        let target = UnitInstanceId(Uuid::from_u128(0xB0B));
        let (result_type, payload) = behavior_result_payload(BehaviorResult::BattleAdvanced {
            battle_uuid: Uuid::from_u128(0xB4771E),
            encounter_id: "airborne_damage_contract".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            playback: BattlePlaybackState::default(),
            battle_time_ms: 4_000,
            timeline_delta: vec![
                TimelineEntry {
                    time_ms: 100,
                    seq: 1,
                    cause: TimelineCause::default(),
                    event: TimelineEvent::UnitSpawned {
                        unit_instance_id: target,
                        owner: Side::Opponent,
                        role: BattleUnitRole::Combatant,
                        mobility_kind: MobilityKind::Airborne,
                        base_uuid: Uuid::from_u128(0xABCD),
                        world_position: TimelineVec2::default(),
                        stats: UnitStats::default(),
                    },
                },
                TimelineEntry {
                    time_ms: 200,
                    seq: 2,
                    cause: TimelineCause::default(),
                    event: TimelineEvent::HpChanged {
                        source_instance_id: Some(source),
                        target_instance_id: target,
                        delta: -10,
                        hp_before: 100,
                        hp_after: 90,
                        reason: HpChangeReason::Command,
                        damage_source: Some(DamageSource::Ability),
                        damage_type: Some(DamageType::Physical),
                        raw_damage: Some(20),
                        final_damage: Some(10),
                        damage_breakdown: None,
                        critical: Some(true),
                        feedback_tags: vec![
                            DamageFeedbackTag::Critical,
                            DamageFeedbackTag::Mitigated,
                        ],
                    },
                },
            ],
            last_timeline_seq: 2,
            finished: false,
            deployment: None,
        })
        .expect("battle advanced result should serialize timeline contract fields");

        assert_eq!(result_type, "BattleAdvanced");
        assert_eq!(payload["timeline_delta"][0]["event"]["type"], "UnitSpawned");
        assert_eq!(
            payload["timeline_delta"][0]["event"]["mobility_kind"],
            "airborne"
        );
        assert_eq!(payload["timeline_delta"][1]["event"]["type"], "HpChanged");
        assert_eq!(
            payload["timeline_delta"][1]["event"]["damage_type"],
            "Physical"
        );
        assert_eq!(
            payload["timeline_delta"][1]["event"]["feedback_tags"],
            serde_json::json!(["critical", "mitigated"])
        );
    }

    #[test]
    fn live_deployment_command_payloads_preserve_mission_identity() {
        let deployment = LiveBattleDeploymentDto {
            battle_time_ms: 1_000,
            current_cost: 10,
            max_cost: 30,
            base_deploy_cost: 10,
            cost_per_second: 1,
            unit_deploy_costs: vec![LiveBattleUnitDeployCostDto {
                employee_uuid: Uuid::from_u128(0xEFFE_C7),
                base_deploy_cost: 10,
                effective_deploy_cost: 7,
            }],
            deployed_units: vec![LiveBattleDeployedUnitDto {
                employee_uuid: Uuid::from_u128(0xEFFE_C7),
                unit_instance_id: UnitInstanceId(Uuid::from_u128(0xD3F3_0001)),
                facing: FacingDirection::Right,
                skill_readiness: None,
            }],
            redeploying_units: Vec::new(),
        };
        let (result_type, payload) = behavior_result_payload(BehaviorResult::BattleUnitDeployed {
            battle_uuid: Uuid::from_u128(0xB4771E),
            encounter_id: "defense_encounter".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            playback: BattlePlaybackState::default(),
            employee_uuid: Uuid::from_u128(0xEFFE_C7),
            unit_instance_id: UnitInstanceId(Uuid::from_u128(0xD3F3_0001)),
            timeline_delta: Vec::new(),
            last_timeline_seq: 21,
            deployment,
        })
        .expect("battle unit deployed result should serialize");

        assert_eq!(result_type, "BattleUnitDeployed");
        assert_eq!(payload["encounter_id"], "defense_encounter");
        assert_eq!(payload["node_type"], "Defense");
        assert_eq!(payload["mission_variant"], "Defense");
        assert_eq!(payload["last_timeline_seq"], 21);
        assert_eq!(payload["deployment"]["battle_time_ms"], 1_000);
        assert_eq!(
            payload["deployment"]["deployed_units"][0]["facing"],
            "right"
        );
        assert_eq!(
            payload["deployment"]["unit_deploy_costs"][0]["employee_uuid"],
            "00000000-0000-0000-0000-000000effec7"
        );
        assert_eq!(
            payload["deployment"]["unit_deploy_costs"][0]["base_deploy_cost"],
            10
        );
        assert_eq!(
            payload["deployment"]["unit_deploy_costs"][0]["effective_deploy_cost"],
            7
        );

        let deployment = LiveBattleDeploymentDto {
            battle_time_ms: 2_000,
            current_cost: 10,
            max_cost: 30,
            base_deploy_cost: 10,
            cost_per_second: 1,
            unit_deploy_costs: vec![LiveBattleUnitDeployCostDto {
                employee_uuid: Uuid::from_u128(0xEFFE_C7),
                base_deploy_cost: 10,
                effective_deploy_cost: 10,
            }],
            deployed_units: Vec::new(),
            redeploying_units: Vec::new(),
        };
        let (result_type, payload) = behavior_result_payload(BehaviorResult::BattleUnitWithdrawn {
            battle_uuid: Uuid::from_u128(0xB4771E),
            encounter_id: "blue_star_encirclement".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Encirclement,
            playback: BattlePlaybackState::default(),
            employee_uuid: Uuid::from_u128(0xEFFE_C7),
            unit_instance_id: UnitInstanceId(Uuid::from_u128(0xD3F3_0001)),
            timeline_delta: Vec::new(),
            last_timeline_seq: 22,
            deployment,
        })
        .expect("battle unit withdrawn result should serialize");

        assert_eq!(result_type, "BattleUnitWithdrawn");
        assert_eq!(payload["encounter_id"], "blue_star_encirclement");
        assert_eq!(payload["node_type"], "Defense");
        assert_eq!(payload["mission_variant"], "Encirclement");
        assert_eq!(payload["last_timeline_seq"], 22);
        assert_eq!(payload["deployment"]["battle_time_ms"], 2_000);
        assert_eq!(
            payload["deployment"]["unit_deploy_costs"][0]["effective_deploy_cost"],
            10
        );
    }
}
