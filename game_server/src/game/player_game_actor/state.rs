use base64::{engine::general_purpose::STANDARD, Engine as _};
use flate2::{write::GzEncoder, Compression};
use game_core::application::{
    ApplicationError, ApplicationResult, ApplicationResultDisposition, ApplicationResultKind,
};
use game_core::game::{
    battle::{event_log::BattleEventLog, types::BattleWinner},
    behavior::{
        ActionCapabilityDto, ActionKind, ActionRoute, HostRecoveryAction, LiveBattleUpdateDto,
        RunSnapshotDto, SelectedEventSnapshotDto,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;

use super::messages::{BattleSideMessageKind, PlayerGameServerMessage};

#[derive(Debug, Clone)]
pub struct PlayerGameActorError {
    pub code: String,
    pub message: String,
}

impl PlayerGameActorError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
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

impl From<ApplicationError> for PlayerGameActorError {
    fn from(error: ApplicationError) -> Self {
        use game_core::application::ApplicationErrorKind as K;
        let details = error.details().as_value();
        let field = |key: &str| {
            details[key]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| details[key].to_string())
        };
        let authority = |key: &str| {
            format!(
                "{}@{}",
                details[key]["battle_uuid"].as_str().unwrap_or("?"),
                details[key]["battle_state_revision"]
            )
        };
        let message = match error.kind() {
            K::EventNotFound => "Selected event was not found".into(),
            K::EventTypeMismatch => "Selected event type does not match current state".into(),
            K::InvalidAction => "Action is not allowed in the current state".into(),
            K::TutorialCompletionRequired => format!(
                "Standard tutorial completion is required before starting {}",
                field("game_mode")
            ),
            K::ActionUnavailable => {
                let reason = serde_json::from_value::<
                    game_core::game::behavior::ActionDisabledReasonCode,
                >(details["reason"].clone())
                .map(|reason| format!("{reason:?}"))
                .unwrap_or_else(|_| field("reason"));
                format!("Action {} is unavailable: {reason}", field("action"))
            }
            K::WeaponRequired => "An equipped weapon is required for this battle action".into(),
            K::InvalidBattleResyncSeq => format!(
                "Battle resync since_seq {} is ahead of latest seq {}",
                field("requested"),
                field("latest")
            ),
            K::BattleStateRevisionOverflow => "Battle state revision can no longer advance".into(),
            K::MissingBattleStateAuthority => format!(
                "Battle mutation requires current authority {}",
                authority("current")
            ),
            K::StaleBattleStateAuthority => format!(
                "Stale battle authority {}; current authority is {}",
                authority("expected"),
                authority("current")
            ),
            K::NotInShopState => "Player is not in shop state".into(),
            K::NotInRewardState => "Player is not in reward state".into(),
            K::ShopItemNotFound => "Shop item was not found".into(),
            K::InventoryFull => "Inventory is full".into(),
            K::InventoryItemNotFound => "Inventory item was not found".into(),
            K::InventoryItemNotRemovable => {
                "Inventory item cannot be removed through this action".into()
            }
            K::InsufficientResources => "Insufficient resources".into(),
            K::MissingResource => format!("Missing core resource: {}", field("resource")),
            K::InvalidUnitStats => format!("Invalid unit stats: {}", field("reason")),
            K::OutOfBounds => "Requested position is out of bounds".into(),
            K::PositionOccupied => "Requested position is already occupied".into(),
            K::StaticObstacleBlocked => "Requested tile is blocked by a static obstacle".into(),
            K::UnitAlreadyPlaced => "Unit is already deployed".into(),
            K::UnitNotFound => "Unit was not found".into(),
            K::InvalidStaticData if details.get("reason").is_some() => {
                format!("Invalid static data: {}", field("reason"))
            }
            K::NotImplemented => format!("Feature is not implemented: {}", field("feature")),
            K::SkillFragmentIncompatible => {
                let failures = serde_json::from_value::<Vec<game_core::game::data::skill_fragment_data::SkillFragmentCompatibilityFailureCode>>(details["failure_codes"].clone())
                    .map(|codes| format!("{codes:?}")).unwrap_or_else(|_| field("failure_codes"));
                format!(
                    "Skill fragment '{}' is incompatible with the selected employee: {failures}",
                    field("fragment_id")
                )
            }
            _ => details
                .get("message")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| format!("{error}: {details}")),
        };
        Self::new(error.code(), message)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedBattleEventLogPayload {
    pub outcome_id: game_core::game::battle::types::BattleOutcomeId,
    pub verdict: game_core::game::battle::types::BattleVerdict,
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
    outcome_id: game_core::game::battle::types::BattleOutcomeId,
    verdict: game_core::game::battle::types::BattleVerdict,
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
        outcome_id,
        verdict,
        winner,
        event_log_encoding: "gzip+base64".to_string(),
        event_log_gzip_base64: STANDARD.encode(&compressed),
        raw_json_bytes: event_log_json.len(),
        gzip_bytes: compressed.len(),
    })
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
    #[serde(skip_serializing_if = "Option::is_none")]
    active_run_continue: Option<ActiveRunContinueSnapshotDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    player_profile_persistence: Option<PlayerProfilePersistenceSnapshotDto>,
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

        Self {
            core,
            active_run_continue: None,
            player_profile_persistence: None,
        }
    }

    pub(crate) fn with_host_overlay(mut self, overlay: HostSnapshotOverlay) -> Self {
        overlay.compose_capabilities(&mut self.core.action_capabilities);
        self.active_run_continue = overlay.active_run_continue;
        self.player_profile_persistence = overlay.player_profile_persistence;
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ActiveRunContinueSnapshotDto {
    available: bool,
    recovered_from_backup: bool,
    reset_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_issue: Option<String>,
}

impl ActiveRunContinueSnapshotDto {
    pub(crate) fn missing() -> Self {
        Self {
            available: false,
            recovered_from_backup: false,
            reset_required: false,
            system_issue: None,
        }
    }

    pub(crate) fn loaded(recovered_from_backup: bool) -> Self {
        Self {
            available: true,
            recovered_from_backup,
            reset_required: false,
            system_issue: None,
        }
    }

    pub(crate) fn reset_required(system_issue: impl Into<String>) -> Self {
        Self {
            available: false,
            recovered_from_backup: false,
            reset_required: true,
            system_issue: Some(system_issue.into()),
        }
    }

    pub(crate) fn protected(system_issue: impl Into<String>) -> Self {
        Self {
            available: false,
            recovered_from_backup: false,
            reset_required: false,
            system_issue: Some(system_issue.into()),
        }
    }

    fn recovery_action(&self) -> Option<(ActionKind, HostRecoveryAction)> {
        if self.available {
            Some((ActionKind::ContinueRun, HostRecoveryAction::ContinueRun))
        } else if self.reset_required {
            Some((
                ActionKind::ResetActiveRun,
                HostRecoveryAction::ResetActiveRun,
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PlayerProfilePersistenceSnapshotDto {
    state: &'static str,
    dirty: bool,
    recovered_from_backup: bool,
    reset_required: bool,
    system_issue: Option<String>,
}

impl PlayerProfilePersistenceSnapshotDto {
    pub(crate) fn clean() -> Self {
        Self {
            state: "clean",
            dirty: false,
            recovered_from_backup: false,
            reset_required: false,
            system_issue: None,
        }
    }

    pub(crate) fn recovered_from_backup() -> Self {
        Self {
            state: "recovered",
            dirty: true,
            recovered_from_backup: true,
            reset_required: false,
            system_issue: None,
        }
    }

    pub(crate) fn dirty(system_issue: impl Into<String>) -> Self {
        Self {
            state: "dirty",
            dirty: true,
            recovered_from_backup: false,
            reset_required: false,
            system_issue: Some(system_issue.into()),
        }
    }

    pub(crate) fn fatal(system_issue: impl Into<String>) -> Self {
        Self {
            state: "error",
            dirty: false,
            recovered_from_backup: false,
            reset_required: true,
            system_issue: Some(system_issue.into()),
        }
    }

    pub(crate) fn protected(system_issue: impl Into<String>) -> Self {
        Self {
            state: "protected",
            dirty: false,
            recovered_from_backup: false,
            reset_required: false,
            system_issue: Some(system_issue.into()),
        }
    }

    pub(crate) fn application_recovery_pending(system_issue: impl Into<String>) -> Self {
        Self {
            state: "application_recovery_pending",
            dirty: true,
            recovered_from_backup: false,
            reset_required: false,
            system_issue: Some(system_issue.into()),
        }
    }

    fn application_recovery_pending_state(&self) -> bool {
        self.state == "application_recovery_pending"
    }

    fn retry_available(&self) -> bool {
        self.dirty || self.reset_required || self.state == "protected"
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct HostSnapshotOverlay {
    active_run_continue: Option<ActiveRunContinueSnapshotDto>,
    player_profile_persistence: Option<PlayerProfilePersistenceSnapshotDto>,
    title_screen: bool,
}

impl HostSnapshotOverlay {
    pub(crate) fn new(
        title_screen: bool,
        active_run_continue: Option<ActiveRunContinueSnapshotDto>,
        player_profile_persistence: Option<PlayerProfilePersistenceSnapshotDto>,
    ) -> Self {
        Self {
            active_run_continue,
            player_profile_persistence,
            title_screen,
        }
    }

    fn compose_capabilities(&self, capabilities: &mut Vec<ActionCapabilityDto>) {
        if self
            .player_profile_persistence
            .as_ref()
            .is_some_and(PlayerProfilePersistenceSnapshotDto::application_recovery_pending_state)
        {
            capabilities.clear();
            push_host_recovery_capability(
                capabilities,
                ActionKind::RetryPlayerProfile,
                HostRecoveryAction::RetryPlayerProfile,
            );
            return;
        }

        if self
            .player_profile_persistence
            .as_ref()
            .is_some_and(|status| status.state == "protected")
        {
            capabilities.clear();
            push_host_recovery_capability(
                capabilities,
                ActionKind::RetryPlayerProfile,
                HostRecoveryAction::RetryPlayerProfile,
            );
            return;
        }

        if !self.title_screen {
            return;
        }
        if self.active_run_continue.as_ref().is_some_and(|status| {
            !status.available && !status.reset_required && status.system_issue.is_some()
        }) {
            capabilities.clear();
            return;
        }

        if self
            .player_profile_persistence
            .as_ref()
            .is_some_and(|status| status.reset_required)
        {
            capabilities.clear();
        } else if let Some((action, operation)) = self
            .active_run_continue
            .as_ref()
            .and_then(ActiveRunContinueSnapshotDto::recovery_action)
        {
            push_host_recovery_capability(capabilities, action, operation);
        }

        if let Some(status) = self.player_profile_persistence.as_ref() {
            if status.retry_available() {
                push_host_recovery_capability(
                    capabilities,
                    ActionKind::RetryPlayerProfile,
                    HostRecoveryAction::RetryPlayerProfile,
                );
            }
            push_host_recovery_capability(
                capabilities,
                ActionKind::ResetPlayerProfile,
                HostRecoveryAction::ResetPlayerProfile,
            );
        }
    }
}

fn push_host_recovery_capability(
    capabilities: &mut Vec<ActionCapabilityDto>,
    action: ActionKind,
    operation: HostRecoveryAction,
) {
    debug_assert_eq!(action.route(), ActionRoute::HostRecovery(operation));
    debug_assert!(!capabilities
        .iter()
        .any(|capability| capability.action == action));
    capabilities.push(ActionCapabilityDto::enabled(action, action.scope()));
}

#[derive(Debug)]
pub struct CommandResultMapping {
    pub response: PlayerGameServerMessage,
    pub side_messages: Vec<PlayerGameServerMessage>,
    pub send_state_snapshot: bool,
}

#[derive(Deserialize)]
pub(crate) struct BattleApplicationPayload {
    #[serde(default)]
    pub battle_setup_snapshot: Option<game_core::game::behavior::LiveBattleSetupSnapshotDto>,
    pub battle_update: LiveBattleUpdateDto,
}

pub(crate) fn battle_payload(
    result: &ApplicationResult,
) -> Result<Option<BattleApplicationPayload>, PlayerGameActorError> {
    use ApplicationResultKind::{
        BattleAdvanced, BattleEngagementStanceChanged, BattlePlaybackChanged, BattleSkillActivated,
        BattleRelationSelected,
        BattleSkillTargetingFocusChanged, BattleState, BattleUnitDeployed,
        BattleUnitTacticallyRelocated, BattleUnitWithdrawn, BattleUnitsTacticallySwapped,
    };
    if !matches!(
        result.kind(),
        BattleAdvanced
            | BattleState
            | BattlePlaybackChanged
            | BattleUnitDeployed
            | BattleUnitWithdrawn
            | BattleEngagementStanceChanged
            | BattleUnitTacticallyRelocated
            | BattleUnitsTacticallySwapped
            | BattleSkillTargetingFocusChanged
            | BattleSkillActivated
            | BattleRelationSelected
    ) {
        return Ok(None);
    }
    if result.disposition() == ApplicationResultDisposition::AcceptedRecoveryPending {
        return Ok(None);
    }
    serde_json::from_value(result.payload().as_value().clone())
        .map(Some)
        .map_err(serialize_error)
}

pub fn behavior_result_to_command_result(
    request_id: String,
    result: ApplicationResult,
    battle_side_message: BattleSideMessageKind,
) -> Result<CommandResultMapping, PlayerGameActorError> {
    if let Some(battle_result) = battle_payload(&result)? {
        let accepted_at_battle_time_ms = battle_result.battle_update.server_battle_time_ms;
        let battle_state_revision = battle_result.battle_update.battle_state_revision;
        let mut side_messages = Vec::new();
        if battle_side_message == BattleSideMessageKind::BattleUpdate {
            if let Some(setup) = battle_result.battle_setup_snapshot {
                side_messages.push(PlayerGameServerMessage::battle_setup_snapshot(setup));
            }
        }
        side_messages.push(match battle_side_message {
            BattleSideMessageKind::BattleUpdate => {
                PlayerGameServerMessage::battle_update(battle_result.battle_update)
            }
            BattleSideMessageKind::BattleResync => {
                PlayerGameServerMessage::battle_resync(battle_result.battle_update)
            }
        });
        return Ok(CommandResultMapping {
            response: PlayerGameServerMessage::CommandResult {
                request_id: request_id.clone(),
                ok: true,
                result_type: "CommandAccepted".into(),
                payload: json!({"command_id": request_id, "accepted_at_battle_time_ms": accepted_at_battle_time_ms, "battle_state_revision": battle_state_revision}),
            },
            side_messages,
            send_state_snapshot: false,
        });
    }
    Ok(CommandResultMapping {
        response: PlayerGameServerMessage::CommandResult {
            request_id,
            ok: true,
            result_type: result.meaning().to_owned(),
            payload: result.payload().as_value().clone(),
        },
        side_messages: Vec::new(),
        send_state_snapshot: result.disposition() != ApplicationResultDisposition::ReadOnly,
    })
}

pub(crate) fn battle_update_from_behavior_result(
    result: &ApplicationResult,
) -> Option<LiveBattleUpdateDto> {
    battle_payload(result)
        .ok()
        .flatten()
        .map(|payload| payload.battle_update)
}

#[cfg(test)]
pub(crate) fn behavior_result_payload(
    result: ApplicationResult,
) -> Result<(String, serde_json::Value), PlayerGameActorError> {
    if battle_payload(&result)?.is_some() {
        return Err(PlayerGameActorError::new(
            "battle_update_transport_required",
            "Battle behavior results must be sent through battle_update transport",
        ));
    }
    Ok((
        result.meaning().to_owned(),
        result.payload().as_value().clone(),
    ))
}

fn serialize_error(error: serde_json::Error) -> PlayerGameActorError {
    PlayerGameActorError::new("serialization_failed", error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player_game_actor::messages::BattleSideMessageKind;
    use crate::test_support::{error, result};
    use flate2::read::GzDecoder;
    use game_core::game::{
        ability::SkillId,
        battle::{event_log::SkillCastTarget, ids::UnitInstanceId},
        behavior::{
            ActionDisabledReasonCode, ActionKind, BattlePlaybackState, BattleStateAuthorityDto,
            DeploymentRangePreviewFacingDto, DeploymentRangePreviewFacingsDto,
            DeploymentRangePreviewResultDto, LiveBattleActiveSkillRangePreviewDto,
            LiveBattleBasicAttackRangePreviewDto, LiveBattleDeploymentDto, LiveBattleEventDeltaDto,
            LiveBattleRangePreviewReason, LiveBattleRangePreviewSource, LiveBattleRangePreviewsDto,
            LiveBattleStateCheckpointDto, LiveBattleTacticalResourceDto, LiveBattleUpdateDto,
            LiveBattleUpdateMessageType, MasteredEliteSweepResultDto,
            MasteredEliteSweepRewardBranchDto, SkillTargetPreviewResultDto,
        },
        combat_preview::{CombatMissionVariant, CombatNodeType},
        data::skill_fragment_data::{SkillFragmentCompatibilityFailureCode, SkillFragmentId},
        map::MapNodeId,
        resources::{InventoryDiffDto, Position},
    };
    use uuid::Uuid;

    #[test]
    fn mastered_elite_sweep_result_uses_generic_typed_command_payload_transport() {
        let sweep_uuid = Uuid::from_u128(0x5EE0);
        let node_id = MapNodeId::new(Uuid::from_u128(0x5EE1));
        let (result_type, payload) =
            behavior_result_payload(result!(MasteredEliteSweepResultStaged {
                result: MasteredEliteSweepResultDto {
                    sweep_uuid,
                    node_id,
                    encounter_id: "elite".to_string(),
                    reward_source_id: "source_elite".to_string(),
                    fragment_id: SkillFragmentId::from("fragment_elite"),
                    reward_branch: MasteredEliteSweepRewardBranchDto::ReacquireFragment,
                    base_rewards: vec![],
                    exclusive_effects: vec![],
                    enkephalin_after: 7,
                    inventory_diff: InventoryDiffDto::default(),
                    skill_fragment_diffs: vec![],
                    employee_experience_diffs: vec![],
                },
            }))
            .unwrap();

        assert_eq!(result_type, "MasteredEliteSweepResultStaged");
        assert_eq!(payload["result"]["sweep_uuid"], sweep_uuid.to_string());
        assert_eq!(payload["result"]["reward_branch"], "reacquire_fragment");
        assert!(payload.get("battle_update").is_none());
        assert!(payload["result"].get("result_stats").is_none());
        assert!(payload["result"].get("event_log").is_none());
    }

    #[test]
    fn compressed_battle_event_log_payload_round_trips_exact_compact_json() {
        use std::io::Read;

        let event_log = BattleEventLog::default();
        let expected_json = event_log.to_json_string().expect("compact event log json");
        let payload = compress_battle_event_log_payload(
            game_core::game::battle::types::BattleOutcomeId::new("victory"),
            game_core::game::battle::types::BattleVerdict::Victory,
            BattleWinner::Player,
            &event_log,
        )
        .expect("compress event log");

        assert_eq!(payload.outcome_id.as_str(), "victory");
        assert_eq!(
            payload.verdict,
            game_core::game::battle::types::BattleVerdict::Victory
        );
        assert_eq!(payload.winner, BattleWinner::Player);
        assert_eq!(payload.event_log_encoding, "gzip+base64");
        assert_eq!(payload.raw_json_bytes, expected_json.len());

        let compressed = STANDARD
            .decode(&payload.event_log_gzip_base64)
            .expect("decode base64 event log");
        assert_eq!(payload.gzip_bytes, compressed.len());

        let mut decoded_json = String::new();
        GzDecoder::new(compressed.as_slice())
            .read_to_string(&mut decoded_json)
            .expect("decompress event log");
        assert_eq!(decoded_json, expected_json);

        let round_tripped: BattleEventLog =
            serde_json::from_str(&decoded_json).expect("deserialize compressed event log");
        assert_eq!(
            serde_json::to_value(round_tripped).expect("round-tripped event log value"),
            serde_json::to_value(event_log).expect("original event log value")
        );
    }

    const ALL_DISABLED_REASONS: [ActionDisabledReasonCode; 42] = [
        ActionDisabledReasonCode::ContextIrrelevant,
        ActionDisabledReasonCode::NoEligiblePayload,
        ActionDisabledReasonCode::MissingEventChoiceKey,
        ActionDisabledReasonCode::CheckpointUnavailable,
        ActionDisabledReasonCode::ProgressionUnavailable,
        ActionDisabledReasonCode::ContentUnavailable,
        ActionDisabledReasonCode::CompletionUnavailable,
        ActionDisabledReasonCode::AlreadyActive,
        ActionDisabledReasonCode::InsufficientResources,
        ActionDisabledReasonCode::RedeployCooldown,
        ActionDisabledReasonCode::StanceCooldown,
        ActionDisabledReasonCode::RelocationCooldown,
        ActionDisabledReasonCode::UnitUnavailable,
        ActionDisabledReasonCode::WeaponRequired,
        ActionDisabledReasonCode::EmployeeDead,
        ActionDisabledReasonCode::CriticalPhysicalDamage,
        ActionDisabledReasonCode::Breakdown,
        ActionDisabledReasonCode::NoTrauma,
        ActionDisabledReasonCode::SkillUnavailable,
        ActionDisabledReasonCode::ActivationModeNotManual,
        ActionDisabledReasonCode::HardCrowdControl,
        ActionDisabledReasonCode::Silenced,
        ActionDisabledReasonCode::DefenseStance,
        ActionDisabledReasonCode::DefenseRequired,
        ActionDisabledReasonCode::ActionInProgress,
        ActionDisabledReasonCode::ActionLocked,
        ActionDisabledReasonCode::FieldWorking,
        ActionDisabledReasonCode::ResonanceNotFull,
        ActionDisabledReasonCode::SkillTargetRequired,
        ActionDisabledReasonCode::SkillTargetTypeMismatch,
        ActionDisabledReasonCode::NoValidTarget,
        ActionDisabledReasonCode::OfferUnavailable,
        ActionDisabledReasonCode::RefreshUnavailable,
        ActionDisabledReasonCode::RefreshUsesExhausted,
        ActionDisabledReasonCode::PoolExhausted,
        ActionDisabledReasonCode::SellingUnavailable,
        ActionDisabledReasonCode::InvalidQuantity,
        ActionDisabledReasonCode::InvalidQuote,
        ActionDisabledReasonCode::ItemBound,
        ActionDisabledReasonCode::ItemEquipped,
        ActionDisabledReasonCode::RelicNotSellable,
        ActionDisabledReasonCode::TagNotAccepted,
    ];

    fn authority(battle_uuid: u128, battle_state_revision: u64) -> BattleStateAuthorityDto {
        BattleStateAuthorityDto {
            battle_uuid: Uuid::from_u128(battle_uuid),
            battle_state_revision,
        }
    }

    fn characterized_fixed_game_errors() -> Vec<(ApplicationError, &'static str, String)> {
        let current = authority(0xCAFE, 17);
        let expected = authority(0xBEEF, 11);

        vec![
            (
                error!(EventNotFound),
                "event_not_found",
                "Selected event was not found".into(),
            ),
            (
                error!(EventTypeMismatch),
                "event_type_mismatch",
                "Selected event type does not match current state".into(),
            ),
            (
                error!(InvalidAction),
                "invalid_action",
                "Action is not allowed in the current state".into(),
            ),
            (
                error!(WeaponRequired),
                "weapon_required",
                "An equipped weapon is required for this battle action".into(),
            ),
            (
                error!(InvalidBattleResyncSeq {
                    requested: 29,
                    latest: 23,
                }),
                "invalid_battle_resync_seq",
                "Battle resync since_seq 29 is ahead of latest seq 23".into(),
            ),
            (
                error!(BattleStateRevisionOverflow),
                "battle_state_revision_overflow",
                "Battle state revision can no longer advance".into(),
            ),
            (
                error!(MissingBattleStateAuthority { current }),
                "missing_battle_state_authority",
                format!(
                    "Battle mutation requires current authority {}@{}",
                    current.battle_uuid, current.battle_state_revision
                ),
            ),
            (
                error!(StaleBattleStateAuthority { expected, current }),
                "stale_battle_state_authority",
                format!(
                    "Stale battle authority {}@{}; current authority is {}@{}",
                    expected.battle_uuid,
                    expected.battle_state_revision,
                    current.battle_uuid,
                    current.battle_state_revision
                ),
            ),
            (
                error!(NotInShopState),
                "not_in_shop_state",
                "Player is not in shop state".into(),
            ),
            (
                error!(NotInRewardState),
                "not_in_reward_state",
                "Player is not in reward state".into(),
            ),
            (
                error!(ShopItemNotFound),
                "shop_item_not_found",
                "Shop item was not found".into(),
            ),
            (
                error!(InventoryFull),
                "inventory_full",
                "Inventory is full".into(),
            ),
            (
                error!(InventoryItemNotFound),
                "inventory_item_not_found",
                "Inventory item was not found".into(),
            ),
            (
                error!(InventoryItemNotRemovable),
                "inventory_item_not_removable",
                "Inventory item cannot be removed through this action".into(),
            ),
            (
                error!(InsufficientResources),
                "insufficient_resources",
                "Insufficient resources".into(),
            ),
            (
                error!(MissingResource { resource: "RunState" }),
                "missing_resource",
                "Missing core resource: RunState".into(),
            ),
            (
                error!(InvalidUnitStats { reason: "negative defense" }),
                "invalid_unit_stats",
                "Invalid unit stats: negative defense".into(),
            ),
            (
                error!(InvalidStaticData { reason: "broken fixture" }),
                "invalid_static_data",
                "Invalid static data: broken fixture".into(),
            ),
            (
                error!(NotImplemented { feature: "future loop" }),
                "not_implemented",
                "Feature is not implemented: future loop".into(),
            ),
            (
                error!(SkillFragmentIncompatible {
                    fragment_id: SkillFragmentId::from("fragment_contract"),
                    failure_codes: vec![
                        SkillFragmentCompatibilityFailureCode::WeaponRequired,
                        SkillFragmentCompatibilityFailureCode::RangeRoleMismatch,
                    ],
                }),
                "skill_fragment_incompatible",
                "Skill fragment 'fragment_contract' is incompatible with the selected employee: [WeaponRequired, RangeRoleMismatch]".into(),
            ),
            (
                error!(OutOfBounds),
                "out_of_bounds",
                "Requested position is out of bounds".into(),
            ),
            (
                error!(PositionOccupied),
                "position_occupied",
                "Requested position is already occupied".into(),
            ),
            (
                error!(StaticObstacleBlocked),
                "static_obstacle_blocked",
                "Requested tile is blocked by a static obstacle".into(),
            ),
            (
                error!(UnitAlreadyPlaced),
                "unit_already_placed",
                "Unit is already deployed".into(),
            ),
            (
                error!(UnitNotFound),
                "unit_not_found",
                "Unit was not found".into(),
            ),
        ]
    }

    #[test]
    fn server_error_projection_characterizes_every_game_error_code_and_message() {
        let fixed_cases = characterized_fixed_game_errors();
        assert_eq!(fixed_cases.len(), 25);
        assert_eq!(ALL_DISABLED_REASONS.len(), 42);

        for (error, expected_code, expected_message) in fixed_cases {
            let actual = PlayerGameActorError::from(error);
            assert_eq!(actual.code, expected_code);
            assert_eq!(actual.message, expected_message);
            assert_eq!(
                serde_json::to_value(actual.to_server_message(Some("request-32".into()))).unwrap(),
                serde_json::json!({
                    "type": "error",
                    "request_id": "request-32",
                    "code": expected_code,
                    "message": expected_message,
                })
            );
        }

        for reason in ALL_DISABLED_REASONS {
            let error = error!(ActionUnavailable {
                action: ActionKind::DeployUnit,
                reason,
            });
            let expected_message = format!("Action DeployUnit is unavailable: {reason:?}");
            let actual = PlayerGameActorError::from(error);
            assert_eq!(actual.code, reason.as_str(), "{reason:?}");
            assert_eq!(actual.message, expected_message, "{reason:?}");
            assert_eq!(
                serde_json::to_value(actual.to_server_message(Some("request-32".into()))).unwrap(),
                serde_json::json!({
                    "type": "error",
                    "request_id": "request-32",
                    "code": reason.as_str(),
                    "message": expected_message,
                }),
                "{reason:?}"
            );
        }

        assert_eq!(25 + ALL_DISABLED_REASONS.len(), 67);
    }

    #[test]
    fn static_obstacle_blocked_uses_static_obstacle_error_code() {
        let error = PlayerGameActorError::from(error!(StaticObstacleBlocked));

        assert_eq!(error.code, "static_obstacle_blocked");
        assert!(error.message.contains("static obstacle"));
    }

    #[test]
    fn action_unavailable_preserves_typed_disabled_reason_code() {
        let error = PlayerGameActorError::from(error!(ActionUnavailable {
            action: ActionKind::LoadRunCheckpoint,
            reason: ActionDisabledReasonCode::CheckpointUnavailable,
        }));

        assert_eq!(error.code, "checkpoint_unavailable");
        assert!(error.message.contains("LoadRunCheckpoint"));

        let weapon_error = PlayerGameActorError::from(error!(ActionUnavailable {
            action: ActionKind::DeployUnit,
            reason: ActionDisabledReasonCode::WeaponRequired,
        }));
        assert_eq!(weapon_error.code, "weapon_required");
        assert!(weapon_error.message.contains("DeployUnit"));

        let relic_error = PlayerGameActorError::from(error!(ActionUnavailable {
            action: ActionKind::SellItem,
            reason: ActionDisabledReasonCode::RelicNotSellable,
        }));
        assert_eq!(relic_error.code, "relic_not_sellable");
        assert_eq!(
            relic_error.message,
            "Action SellItem is unavailable: RelicNotSellable"
        );
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
            battle_state_revision: 7,
            action_capabilities: Vec::new(),
            server_battle_time_ms: battle_time_ms,
            events_delta: LiveBattleEventDeltaDto {
                after_seq: to_seq,
                to_seq,
                events: Vec::new(),
            },
            checkpoint: LiveBattleStateCheckpointDto {
                presence_marks: Default::default(),
                weapon_marks: Default::default(),
                tile_bursts: Default::default(),
                counter_channels: Default::default(),
                relation_actions: Default::default(),
                input_plans: Default::default(),
                shared_cohorts: Default::default(),
                prepared_volleys: Default::default(),
                mobile_effects: Default::default(),
                paired_areas: Default::default(),

                sustained_effects: Default::default(),
                independent_objects: Default::default(),
                at_seq: to_seq,
                battle_time_ms,
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
                deployment,
            },
            visual_novel: None,
        }
    }

    fn dummy_battle_advanced_result() -> ApplicationResult {
        let battle_uuid = Uuid::from_u128(0xB4771E);
        result!(BattleAdvanced {
            battle_uuid,
            encounter_id: "defend_black_box_relay".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            battle_setup_snapshot:
                Option::<game_core::game::behavior::LiveBattleSetupSnapshotDto>::None,
            battle_update: dummy_battle_update(battle_uuid, 3_000, 12, None),
            finished: false,
        })
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
            [PlayerGameServerMessage::BattleUpdate(update)]
                if update.server_battle_time_ms == 3_000
                    && update.events_delta.to_seq == 12
                    && update.checkpoint.at_seq == 12
                    && update.checkpoint.tactical_resource.current == 20
                    && update.checkpoint.tactical_resource.maximum == 99
        ));
        assert!(
            !mapped.send_state_snapshot,
            "battle side-message command results must not request a trailing state_snapshot"
        );
    }

    #[test]
    fn relation_selection_result_uses_ack_and_preserves_battle_checkpoint() {
        let battle_uuid = Uuid::from_u128(0xB4771E);
        let employee_uuid = Uuid::from_u128(7);
        let unit_instance_id = game_core::game::battle::ids::UnitInstanceId::from(employee_uuid);
        let mut update = dummy_battle_update(battle_uuid, 3_000, 12, None);
        update.checkpoint.relation_actions.owners.push(
            game_core::game::ability::relation_action::RelationOwnerSnapshot {
                source_copy: Uuid::from_u128(8),
                caster: unit_instance_id,
                stock: 2,
                capacity: Some(4),
                active_mode: false,
                mode_reserved: false,
                shield_used: false,
                target: Some(unit_instance_id),
                generation: Some(3),
                next_gain_ms: Some(4_000),
                menu_entry: "blessing".into(),
                menu_available: true,
                eligible_targets: vec![unit_instance_id],
                pending: false,
                active: None,
                applied_modifiers: Vec::new(),
            },
        );
        let expected = serde_json::to_value(&update).unwrap();
        let result = result!(BattleRelationSelected {
            battle_uuid,
            encounter_id: "relation_trial".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            battle_update: update,
            employee_uuid,
            unit_instance_id,
            entry: "blessing".to_string(),
        });
        assert_eq!(
            serde_json::to_value(battle_update_from_behavior_result(&result).unwrap()).unwrap(),
            expected
        );
        assert_eq!(
            behavior_result_payload(result.clone()).unwrap_err().code,
            "battle_update_transport_required"
        );
        let mapped = behavior_result_to_command_result(
            "relation-1".into(),
            result,
            BattleSideMessageKind::BattleUpdate,
        )
        .unwrap();
        let PlayerGameServerMessage::CommandResult {
            request_id,
            ok,
            result_type,
            payload,
        } = mapped.response
        else {
            panic!("command acknowledgement missing")
        };
        assert_eq!(request_id, "relation-1");
        assert!(ok);
        assert_eq!(result_type, "CommandAccepted");
        assert_eq!(payload["accepted_at_battle_time_ms"], 3_000);
        assert_eq!(
            payload["battle_state_revision"],
            expected["battle_state_revision"]
        );
        assert!(!mapped.send_state_snapshot);
        assert_eq!(mapped.side_messages.len(), 1);
        let PlayerGameServerMessage::BattleUpdate(update) = &mapped.side_messages[0] else {
            panic!("battle update missing")
        };
        assert_eq!(serde_json::to_value(update).unwrap(), expected);
    }

    #[test]
    fn deployment_range_preview_result_does_not_request_state_snapshot() {
        let range_previews = dummy_range_previews();
        let result = result!(DeploymentRangePreview {
            result: DeploymentRangePreviewResultDto {
                employee_uuid: Uuid::from_u128(7),
                position: Position::new(2, 2),
                deploy_eligibility: game_core::game::behavior::BattlePayloadEligibilityDto::enabled(
                ),
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
        });

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
                assert_eq!(
                    payload["facings"]["right"]["range_previews"]["active_skill"]["cast_cells"],
                    serde_json::json!([])
                );
                assert_eq!(
                    payload["facings"]["right"]["range_previews"]["active_skill"]
                        ["effect_preview_cells"],
                    serde_json::json!([])
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

    #[test]
    fn skill_target_preview_result_is_read_only_and_preserves_revision_and_reason() {
        let employee_uuid = Uuid::from_u128(0x701);
        let unit_instance_id = UnitInstanceId::from(Uuid::from_u128(0x702));
        let target_id = UnitInstanceId::from(Uuid::from_u128(0x703));
        let result = result!(SkillTargetPreview {
            result: SkillTargetPreviewResultDto {
                counter_channel: None,
                object_placements: Vec::new(),
                employee_uuid,
                unit_instance_id,
                skill_id: SkillId::from("fragment_freischutz_black_round"),
                target: SkillCastTarget::Unit {
                    unit_instance_id: target_id,
                },
                battle_state_revision: 17,
                eligibility: game_core::game::behavior::BattlePayloadEligibilityDto::disabled(
                    ActionDisabledReasonCode::NoValidTarget,
                ),
            },
        });

        let mapped = behavior_result_to_command_result(
            "skill-preview-1".to_string(),
            result,
            BattleSideMessageKind::BattleUpdate,
        )
        .expect("skill target preview should serialize");

        let PlayerGameServerMessage::CommandResult {
            result_type,
            payload,
            ..
        } = mapped.response
        else {
            panic!("expected command result");
        };
        assert_eq!(result_type, "SkillTargetPreview");
        assert_eq!(payload["employee_uuid"], employee_uuid.to_string());
        assert_eq!(payload["unit_instance_id"], unit_instance_id.to_string());
        assert_eq!(payload["target"]["type"], "Unit");
        assert_eq!(payload["target"]["unit_instance_id"], target_id.to_string());
        assert_eq!(payload["battle_state_revision"], 17);
        assert_eq!(
            payload["eligibility"]["disabled_reason_code"],
            "no_valid_target"
        );
        assert!(mapped.side_messages.is_empty());
        assert!(!mapped.send_state_snapshot);
    }

    #[test]
    fn planning_return_uses_current_exact_payload_before_trailing_snapshot() {
        let result = result!(NodeCompleted {
            map: game_core::game::map::MapViewDto {
                map_template_id: game_core::game::map::MapTemplateId::new("delivery_server"),
                facility_template_version: game_core::game::map::FACILITY_TEMPLATE_VERSION,
                nodes: vec![],
            },
            outcome: Option::<serde_json::Value>::None,
        });

        let mapped = behavior_result_to_command_result(
            "complete-node-1".to_string(),
            result,
            BattleSideMessageKind::BattleUpdate,
        )
        .expect("planning return should serialize");

        match mapped.response {
            PlayerGameServerMessage::CommandResult {
                result_type,
                payload,
                ..
            } => {
                assert_eq!(result_type, "NodeCompleted");
                assert!(payload.get("research_deliveries").is_none());
            }
            other => panic!("expected command result, got {other:?}"),
        }
        assert!(mapped.side_messages.is_empty());
        assert!(
            mapped.send_state_snapshot,
            "authoritative completion acknowledgement must precede the latest root snapshot"
        );
    }

    #[test]
    fn run_complete_preserves_direct_map_payload_and_trailing_snapshot() {
        let map = game_core::game::map::MapViewDto {
            map_template_id: game_core::game::map::MapTemplateId::new("completed_run"),
            facility_template_version: game_core::game::map::FACILITY_TEMPLATE_VERSION,
            nodes: vec![],
        };
        let expected_payload = serde_json::to_value(&map).unwrap();
        let mapped = behavior_result_to_command_result(
            "complete-run-1".to_string(),
            result!(RunComplete { map }),
            BattleSideMessageKind::BattleUpdate,
        )
        .expect("RunComplete should serialize");

        let PlayerGameServerMessage::CommandResult {
            result_type,
            payload,
            ..
        } = mapped.response
        else {
            panic!("expected command result");
        };
        assert_eq!(result_type, "RunComplete");
        assert_eq!(payload, expected_payload);
        assert!(payload.get("map").is_none());
        assert!(mapped.side_messages.is_empty());
        assert!(mapped.send_state_snapshot);
    }

    #[test]
    fn event_state_mapping_preserves_choice_evaluation_and_requests_trailing_snapshot() {
        let event = serde_json::from_value::<game_core::game::behavior::EventSceneSnapshotDto>(
            serde_json::json!({
                "event_id": "server_event",
                "current_scene_id": "check",
                "presentation": {
                    "panels": [{"image_id": "image", "text_ids": ["text"]}]
                },
                "interaction": {
                    "type": "employee_check_choices",
                    "choices": [{
                        "choice_id": "inspect",
                        "label_id": "inspect_label",
                        "aptitude": "Analysis",
                        "dc": 16,
                        "required_choice_key": "sealed_archive_access",
                        "eligibility": {"enabled": true},
                        "employee_candidates": [{
                            "employee_uuid": "00000000-0000-0000-0000-000000000002",
                            "name": "Employee",
                            "eligibility": {"enabled": true},
                            "employee_modifier": 0,
                            "equipment_modifier": 2,
                            "modifier_total": 2,
                            "modifier_components": [{
                                "type": "equipment",
                                "equipment_instance_uuid": "00000000-0000-0000-0000-000000000003",
                                "equipment_definition_id": "archive_scanner",
                                "aptitude": "Analysis",
                                "amount": 2
                            }],
                            "choice_key_evidence": {
                                "key_id": "sealed_archive_access",
                                "equipment_instance_uuid": "00000000-0000-0000-0000-000000000003",
                                "equipment_definition_id": "archive_scanner"
                            },
                            "success_chance_percent": 35
                        }]
                    }]
                }
            }),
        )
        .expect("final Event DTO should deserialize");
        let mapped = behavior_result_to_command_result(
            "resolve-event-1".to_string(),
            result!(EventState {
                node_id: game_core::game::map::MapNodeId::new(uuid::Uuid::from_u128(1)),
                event,
            }),
            BattleSideMessageKind::BattleUpdate,
        )
        .expect("EventState should map without repair");

        let PlayerGameServerMessage::CommandResult {
            result_type,
            payload,
            ..
        } = mapped.response
        else {
            panic!("expected command result")
        };
        assert_eq!(result_type, "EventState");
        assert_eq!(payload["event"]["interaction"]["choices"][0]["dc"], 16);
        assert_eq!(
            payload["event"]["interaction"]["choices"][0]["employee_candidates"][0]
                ["success_chance_percent"],
            35
        );
        assert!(payload["event"]["interaction"]
            .get("required_roll")
            .is_none());
        assert!(mapped.side_messages.is_empty());
        assert!(mapped.send_state_snapshot);
    }
}
