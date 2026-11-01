use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::super::{
    application::ApplicationRoute,
    state::{
        CombatRecoveryRestriction, CombatRewardContinuation, ConfirmedCombatResult,
        RunCheckpointPayload,
    },
    GameCore,
};
use crate::game::{
    behavior::{ActionKind, GameError},
    map::MapNodeId,
    resources::{ActiveNodeContent, GameState, RewardSessionState},
};

/// An immutable confirmed continuation. While a node is unfinished this remains
/// its pre-entry state; only successful retreats are overlaid on that state.
#[derive(Clone, Default)]
pub(in crate::game::world) struct ContinueState {
    pub(in crate::game::world) payload: Option<Arc<ContinuePayload>>,
    pub(in crate::game::world) encoded: Option<Arc<[u8]>>,
    unfinished_node: Option<MapNodeId>,
}

impl ContinueState {
    pub(in crate::game::world) fn has_relic_owner(&self) -> bool {
        self.payload.as_ref().is_some_and(|p| p.has_relic_owner())
    }
    pub(in crate::game::world) fn relic_owned_extent(&self) -> Result<usize, GameError> {
        let Some(payload) = &self.payload else {
            return Ok(0);
        };
        if !payload.has_relic_owner() {
            return Ok(0);
        }
        // Current is an independent checkpoint value; savepoint is one shared
        // Arc payload and is counted once here. World clones only Arc handles.
        let bytes = payload
            .current
            .relic_owned_extent()?
            .checked_add(std::mem::size_of::<ContinuePayload>() + 64)
            .ok_or_else(|| invalid("relic current extent overflow"))?;
        let savepoint = payload
            .savepoint
            .as_ref()
            .map(|p| {
                p.relic_owned_extent()?
                    .checked_add(std::mem::size_of::<RunCheckpointPayload>() + 64)
                    .ok_or_else(|| invalid("relic SavePoint extent overflow"))
            })
            .transpose()?
            .unwrap_or(0);
        let encoded = self
            .encoded
            .as_ref()
            .map(|v| v.len().checked_add(64))
            .unwrap_or(Some(0))
            .ok_or_else(|| invalid("relic encoded extent overflow"))?;
        bytes
            .checked_add(savepoint)
            .and_then(|b| b.checked_add(encoded))
            .ok_or_else(|| {
                GameError::InvalidStaticData("relic continuation extent overflow".into())
            })
    }
}
impl ContinuePayload {
    fn has_relic_owner(&self) -> bool {
        self.current.has_relic_owner()
            || self.savepoint.as_ref().is_some_and(|p| p.has_relic_owner())
    }
}

#[derive(Clone, Serialize)]
pub(in crate::game::world) struct ContinuePayload {
    current: RunCheckpointPayload,
    savepoint: Option<Arc<RunCheckpointPayload>>,
    phase: GameState,
    pending_node_session: Option<crate::game::map::NodeSession>,
    recovery_restriction: Option<CombatRecoveryRestriction>,
    result: Option<Arc<ConfirmedCombatResult>>,
    reward: Option<RewardSessionState>,
    reward_continuation: Option<CombatRewardContinuation>,
    visual_novel: crate::game::visual_novel::VisualNovelRuntimeState,
    persistent_vn_consumed: Vec<crate::game::visual_novel::VisualNovelTriggerConsumedKey>,
}

// This is a codec representation, never a mutable gameplay owner. A SavePoint
// identical to current is encoded once; restoration materializes an independent
// checkpoint before any later current-state mutation can affect it.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum ContinueSavePointWire {
    Current {},
    Snapshot {
        checkpoint: Arc<RunCheckpointPayload>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContinuePayloadWire {
    current: RunCheckpointPayload,
    #[serde(deserialize_with = "required_option")]
    savepoint: Option<ContinueSavePointWire>,
    phase: GameState,
    #[serde(deserialize_with = "required_option")]
    pending_node_session: Option<crate::game::map::NodeSession>,
    #[serde(deserialize_with = "required_option")]
    recovery_restriction: Option<CombatRecoveryRestriction>,
    #[serde(deserialize_with = "required_option")]
    result: Option<Arc<ConfirmedCombatResult>>,
    #[serde(deserialize_with = "required_option")]
    reward: Option<RewardSessionState>,
    #[serde(deserialize_with = "required_option")]
    reward_continuation: Option<CombatRewardContinuation>,
    visual_novel: crate::game::visual_novel::VisualNovelRuntimeState,
    persistent_vn_consumed: Vec<crate::game::visual_novel::VisualNovelTriggerConsumedKey>,
}

impl ContinuePayloadWire {
    fn resolve(self) -> Result<ContinuePayload, GameError> {
        let savepoint = match self.savepoint {
            None => None,
            Some(ContinueSavePointWire::Snapshot { checkpoint }) => Some(checkpoint),
            Some(ContinueSavePointWire::Current {}) => {
                if self.current.run_progression.pending_continuation.is_some() { return Err(invalid("SavePoint alias cannot carry a pending choice")); }
                // This is the fresh, tombstone-free serde owner. Account for
                // its native table before the first independent checkpoint
                // clone, not only after resolving and validating the payload.
                {
                    crate::game::relic::allocation::admit_arc(
                        std::mem::size_of::<RunCheckpointPayload>(),
                        "relic_continue_savepoint_arc",
                    )?;
                }
                Some(Arc::new(self.current.copy_owned()?))
            }
        };
        Ok(ContinuePayload {
            current: self.current,
            savepoint,
            phase: self.phase,
            pending_node_session: self.pending_node_session,
            recovery_restriction: self.recovery_restriction,
            result: self.result,
            reward: self.reward,
            reward_continuation: self.reward_continuation,
            visual_novel: self.visual_novel,
            persistent_vn_consumed: self.persistent_vn_consumed,
        })
    }
}

// Only the wire ownership shape is needed before typed decoding. Relic
// inventory no longer decides whether the rest of the continuation is metered.
#[derive(Deserialize)]
struct ContinueAllocationHint {
    continuation: ContinueSavepointHint,
}
#[derive(Deserialize)]
struct ContinueSavepointHint {
    current: CheckpointAllocationHint,
    savepoint: Option<SavepointAllocationHint>,
}
#[derive(Deserialize)]
struct SavepointAllocationHint {
    snapshot: Option<SnapshotAllocationHint>,
}
#[derive(Deserialize)]
struct SnapshotAllocationHint {
    checkpoint: CheckpointAllocationHint,
}
#[derive(Deserialize)]
struct CheckpointAllocationHint {
    typed_resources: ResourceAllocationHint,
}
#[derive(Deserialize)]
struct ResourceAllocationHint {
    entries: EntryCount,
}
struct EntryCount(usize);
impl<'de> Deserialize<'de> for EntryCount {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = EntryCount;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("run resource entries") }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<EntryCount, A::Error> {
                let mut count = 0usize;
                while seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                    count = count.checked_add(1).ok_or_else(|| serde::de::Error::custom("resource count overflow"))?;
                }
                Ok(EntryCount(count))
            }
        }
        d.deserialize_seq(Visitor)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContinueDocument {
    version: u32,
    continuation: ContinuePayloadWire,
    continuation_sha256: String,
}

const VERSION: u32 = 18;

/// Strict codec/checksum inspection only. No application or restored World is
/// created, and no gameplay or filesystem owner is changed.
pub(crate) fn inspect_requirements(bytes: &[u8], max_bytes: usize) -> Result<crate::game::data::content_pack::ContentIdentity, GameError> {
    if bytes.len() > max_bytes.min(64 * 1024 * 1024) {
        return Err(invalid("read-only inspection byte ceiling"));
    }
    let header: super::ActiveRunCorePayloadHeader = super::parse_active_run_checkpoint(bytes)?;
    if header.version != VERSION {
        return Err(invalid(&format!("unsupported read-only continuation version {}", header.version)));
    }
    let document: ContinueDocument = super::parse_active_run_checkpoint(bytes)?;
    if document.continuation_sha256 != canonical_digest(&canonical_value(&document.continuation, false)?)? {
        return Err(invalid("SHA-256 mismatch"));
    }
    let identity = document.continuation.current.content_identity;
    if let Some(ContinueSavePointWire::Snapshot { checkpoint }) = document.continuation.savepoint {
        if checkpoint.content_identity != identity {
            return Err(invalid("current and SavePoint content identities differ"));
        }
    }
    Ok(identity)
}

crate::authoring::clone_shape::struct_shape!(ContinueState {
    payload,
    encoded,
    unfinished_node
});
crate::authoring::clone_shape::struct_shape!(ContinuePayload {
    current,
    savepoint,
    phase,
    pending_node_session,
    recovery_restriction,
    result,
    reward,
    reward_continuation,
    visual_novel,
    persistent_vn_consumed
});

fn canonical_value(
    value: &impl Serialize,
    meter_storage: bool,
) -> Result<serde_json::Value, GameError> {
    crate::authoring::control::metadata_size(value)?;
    crate::authoring::control::charge_serialized(value, "continue_encode_value")?;
    let mut value = serde_json::to_value(value)
        .map_err(|error| super::active_run_checkpoint_codec_error("serialization", error))?;
    normalize_unordered_checkpoint_fields(&mut value, meter_storage)?;
    Ok(value)
}

fn canonical_bytes(value: &serde_json::Value, meter_storage: bool) -> Result<Vec<u8>, GameError> {
    crate::authoring::control::charge_serialized(value, "continue_encode_bytes")?;
    let mut bytes = if meter_storage {
        Vec::with_capacity(crate::game::relic::allocation::admit_json_buffer(value)?)
    } else {
        Vec::new()
    };
    super::write_canonical_json(value, &mut bytes)
        .map_err(|error| super::active_run_checkpoint_codec_error("serialization", error))?;
    crate::authoring::control::poll("continue_encode_complete")?;
    Ok(bytes)
}

fn canonical_digest(value: &serde_json::Value) -> Result<String, GameError> {
    struct HashWriter(Sha256);
    impl std::io::Write for HashWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
    }
    crate::authoring::control::charge_serialized(value, "continue_checksum_bytes")?;
    let mut writer = HashWriter(Sha256::new());
    super::write_canonical_json(value, &mut writer)
        .map_err(|error| super::active_run_checkpoint_codec_error("checksum", error))?;
    Ok(format!("{:x}", writer.0.finalize()))
}

// Writer and reader consume the same normalized wire value. The reader keeps
// this verified encoding instead of capturing and encoding the World again.
fn seal_document(continuation: serde_json::Value, expected: Option<&str>) -> Result<Vec<u8>, GameError> {
    crate::game::relic::allocation::admit_codec_fields(
        &["version", "continuation", "continuation_sha256"], 64,
    )?;
    let digest = canonical_digest(&continuation)?;
    if expected.is_some_and(|expected| expected != digest) {
        return Err(invalid("SHA-256 mismatch"));
    }
    let document = serde_json::Value::Object(serde_json::Map::from_iter([
        ("version".into(), serde_json::Value::from(VERSION)),
        ("continuation".into(), continuation),
        ("continuation_sha256".into(), serde_json::Value::String(digest)),
    ]));
    let encoded = canonical_bytes(&document, true)?;
    if encoded.len() > 64 * 1024 * 1024 {
        return Err(invalid("continuation exceeds supported byte ceiling"));
    }
    Ok(encoded)
}

// These checkpoint collections are sets. Their iteration order is deliberately
// excluded from the wire identity, including after a fresh-process restore.
fn normalize_unordered_checkpoint_fields(
    value: &mut serde_json::Value,
    meter_storage: bool,
) -> Result<(), GameError> {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                if matches!(
                    key.as_str(),
                    "visual_novel_trigger_checkpoint_consumed" | "purchased_merchandise"
                ) {
                    if let Some(values) = value.as_array_mut() {
                        let comparisons = values
                            .len()
                            .checked_mul(values.len().max(1).ilog2() as usize + 1)
                            .ok_or_else(|| invalid("set ordering work overflow"))?;
                        crate::authoring::control::charge(
                            comparisons as u64,
                            "continue_set_order",
                        )?;
                        if meter_storage {
                            crate::game::relic::allocation::admit_normalization_keys(values)?;
                        } else {
                            for value in values.iter() {
                                crate::authoring::control::charge_serialized(
                                    value,
                                    "continue_set_key",
                                )?;
                            }
                        }
                        values.sort_by_cached_key(|value| {
                            serde_json::to_vec(value).expect("JSON value serialization")
                        });
                    }
                }
                normalize_unordered_checkpoint_fields(value, meter_storage)?;
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                normalize_unordered_checkpoint_fields(value, meter_storage)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

fn invalid(message: &str) -> GameError {
    GameError::InvalidStaticData(format!("active-run continuation {message}"))
}

fn acknowledgement_matches_phase(
    result: &crate::game::behavior::BehaviorResult,
    phase: &GameState,
) -> bool {
    use crate::game::behavior::BehaviorResult as Result;
    match (result, phase) {
        (Result::NodeCompleted { .. } | Result::FloorAdvanced { .. }, GameState::ViewingMap)
        | (Result::RunComplete { .. }, GameState::RunComplete) => true,
        (Result::RunFailed { reason, .. }, GameState::RunFailed { reason: expected }) => {
            reason == expected
        }
        (
            Result::RunFailureDecisionRequired {
                reason,
                contributing_reason,
                ..
            },
            GameState::AwaitingRunFailureDecision {
                reason: expected,
                contributing_reason: expected_contributing,
            },
        ) => reason == expected && contributing_reason == expected_contributing,
        _ => false,
    }
}

fn terminal(phase: &GameState) -> bool {
    matches!(
        phase,
        GameState::RunComplete | GameState::RunFailed { .. } | GameState::GameOver
    )
}

impl GameCore {
    pub(crate) fn durable_run_is_terminal(&self) -> bool {
        self.state.combat_reward_continuation.is_none()
            && terminal(
                self.state
                    .confirmed_combat_result
                    .as_ref()
                    .map_or(self.state.lifecycle.phase(), |result| {
                        &result.continuation_phase
                    }),
            )
    }

    fn ensure_continuation_operation(&self) -> Result<(), GameError> {
        use crate::game::battle::runtime_control as native;
        if native::is_active() { return native::poll(); }
        let settings = self.game_data.runtime_safety();
        native::run_with_profile(settings.native_runtime(), settings, || {
            // Existing relic owners keep their established seed. Storage
            // producers below account independently of inventory contents.
            native::seed_retained_extent(self.state.relic_owned_extent()?)
        })
    }

    fn install_continue_payload(&mut self, payload: ContinuePayload) -> Result<(), GameError> {
        self.ensure_continuation_operation()?;
        {
            crate::game::relic::allocation::admit_json_value(&payload)?;
            crate::game::relic::allocation::admit_codec_fields(
                &["savepoint", "snapshot", "checkpoint", "current"],
                0,
            )?;
        }
        let mut continuation = canonical_value(&payload, true)?;
        let fields = continuation
            .as_object_mut()
            .expect("typed continuation object");
        let savepoint = fields
            .remove("savepoint")
            .expect("required SavePoint field");
        let savepoint = if savepoint.is_null() {
            savepoint
        } else {
            let current = fields.get("current").expect("required current field");
            crate::authoring::control::charge_serialized(current, "continue_savepoint_comparison")?;
            crate::authoring::control::charge_serialized(
                &savepoint,
                "continue_savepoint_comparison",
            )?;
            if &savepoint == current {
                serde_json::json!({ "current": {} })
            } else {
                serde_json::Value::Object(serde_json::Map::from_iter([(
                    "snapshot".into(),
                    serde_json::Value::Object(serde_json::Map::from_iter([(
                        "checkpoint".into(),
                        savepoint,
                    )])),
                )]))
            }
        };
        fields.insert("savepoint".into(), savepoint);
        let encoded = seal_document(continuation, None)?;
        self.cache_continue_payload(payload, encoded)
    }

    fn cache_continue_payload(&mut self, payload: ContinuePayload, encoded: Vec<u8>) -> Result<(), GameError> {
        {
            crate::game::relic::allocation::admit_arc(
                std::mem::size_of::<ContinuePayload>(),
                "relic_continue_payload_arc",
            )?;
            crate::game::relic::allocation::admit_arc(encoded.len(), "relic_continue_bytes_arc")?;
        }
        self.state.continue_state.payload = Some(Arc::new(payload));
        self.state.continue_state.encoded = Some(encoded.into());
        Ok(())
    }

    fn capture_continue_payload(&self) -> Result<ContinuePayload, GameError> {
        self.ensure_continuation_operation()?;
        let run = self.run_state()?;
        let current = RunCheckpointPayload::capture(&self.state, run, self.game_data.identity())?;
        let savepoint = match &self.state.run_checkpoint.payload {
            Some(checkpoint) => {
                {
                    crate::game::relic::allocation::admit_arc(
                        std::mem::size_of::<RunCheckpointPayload>(),
                        "relic_continue_savepoint_arc",
                    )?;
                }
                Some(Arc::new(checkpoint.copy_owned()?))
            }
            None => None,
        };
        let phase = match self.get_state() {
            GameState::NodeConfirm { .. } if run.run_progression.pending_continuation.is_none() => GameState::ViewingMap,
            phase => phase,
        };
        let reward = if self.state.combat_reward_continuation.is_some() {
            Some(crate::authoring::clone_shape::controlled_clone(
                self.state
                    .active_node_content
                    .as_ref()
                    .ok_or(GameError::InvalidAction)?
                    .as_reward()?,
            )?)
        } else {
            None
        };
        let triggers =
            crate::authoring::clone_shape::controlled_clone(&self.state.visual_novel_triggers)?;
        let mut persistent_vn_consumed =
            triggers.persistent_consumed.into_iter().collect::<Vec<_>>();
        let scope_key = |scope: &crate::game::visual_novel::VisualNovelTriggerConsumedScope| {
            use crate::game::visual_novel::VisualNovelTriggerConsumedScope as Scope;
            match scope {
                Scope::Run => (0, 0),
                Scope::Battle { battle_uuid } => (1, battle_uuid.as_u128()),
                Scope::Node { node_id } => (2, node_id.0.as_u128()),
            }
        };
        persistent_vn_consumed.sort_by(|a, b| {
            a.trigger_id
                .as_str()
                .cmp(b.trigger_id.as_str())
                .then_with(|| scope_key(&a.scope).cmp(&scope_key(&b.scope)))
        });
        Ok(ContinuePayload {
            visual_novel: crate::authoring::clone_shape::controlled_clone(
                &self.state.visual_novel,
            )?,
            persistent_vn_consumed,
            current,
            savepoint,
            phase,
            pending_node_session: if run.run_progression.pending_continuation.is_some() {
                crate::authoring::clone_shape::controlled_clone(&self.state.node_session)?
            } else { None },
            recovery_restriction: crate::authoring::clone_shape::controlled_clone(
                &run.combat_recovery_restriction,
            )?,
            result: self.state.confirmed_combat_result.clone(),
            reward,
            reward_continuation: crate::authoring::clone_shape::controlled_clone(
                &self.state.combat_reward_continuation,
            )?,
        })
    }

    pub(in crate::game::world) fn capture_expedition_entry_for_continue(&mut self, node_id: MapNodeId) -> Result<(), GameError> {
        if self.run_state()?.run_progression.game_mode == crate::game::map::GameMode::Expedition
            && self.state.continue_state.unfinished_node != Some(node_id)
        {
            let payload = self.capture_continue_payload()?;
            self.install_continue_payload(payload)?;
            self.state.continue_state.unfinished_node = Some(node_id);
        }
        Ok(())
    }

    pub(in crate::game::world) fn prepare_continue_before_operation(
        &mut self,
        route: ApplicationRoute,
    ) -> Result<(), GameError> {
        if matches!(
            route,
            ApplicationRoute::Behavior(ActionKind::ConfirmEnterNode)
        ) {
            if let GameState::NodeConfirm { node_id, .. } = self.get_state() {
                if self.state.continue_state.unfinished_node.is_none() {
                    let payload = self.capture_continue_payload()?;
                    self.install_continue_payload(payload)?;
                    self.state.continue_state.unfinished_node = Some(node_id);
                }
            }
        }
        Ok(())
    }

    pub(in crate::game::world) fn refresh_continue_after_operation(
        &mut self,
        route: ApplicationRoute,
    ) -> Result<(), GameError> {
        if self.state.run.is_none() {
            self.state.continue_state = Default::default();
            return Ok(());
        }
        let phase = self.get_state();
        let final_phase = self
            .state
            .confirmed_combat_result
            .as_ref()
            .map_or(&phase, |result| &result.continuation_phase);
        // A final battle's mandatory choice remains an entitlement until claimed
        // and exited. The terminal disposition is durable immediately afterward.
        if terminal(final_phase) && self.state.combat_reward_continuation.is_none() {
            self.state.continue_state = Default::default();
            return Ok(());
        }
        // Continue has already validated and installed its canonical confirmed
        // root. Reconciliation below the restore boundary does not invent new
        // progress; recapturing here duplicated decode/clone/JSON costs.
        if matches!(route, ApplicationRoute::ActiveRunContinue) {
            return Ok(());
        }
        // Ordinary retreat returns to confirmation and preserves its pre-entry
        // Continue root. Stress retreat consumes the node and must publish the
        // resulting stable state below, including its retained entry cost.
        if matches!(route, ApplicationRoute::Behavior(ActionKind::RetreatBattle))
            && matches!(phase, GameState::NodeConfirm { .. }) {
            let node_id = self
                .state
                .continue_state
                .unfinished_node
                .ok_or(GameError::InvalidAction)?;
            let before = self
                .state
                .continue_state
                .payload
                .as_ref()
                .ok_or(GameError::InvalidAction)?;
            // Only current is independently copied; SavePoint/result remain shared.
            before.current.preflight_copy()?;
            let mut payload = crate::authoring::clone_shape::controlled_clone(before.as_ref())?;
            payload.current.abnormality_attempts.insert(
                node_id,
                self.run_state()?.abnormality_attempt_state(node_id),
            );
            self.install_continue_payload(payload)?;
            return Ok(());
        }
        if matches!(
            route,
            ApplicationRoute::Behavior(ActionKind::LoadRunCheckpoint)
        ) {
            self.state.continue_state.unfinished_node = None;
        }
        let stable = match phase {
            GameState::ViewingMap
            | GameState::CombatResult { .. }
            | GameState::AwaitingRunFailureDecision { .. } => true,
            GameState::NodeConfirm { .. } => self.state.continue_state.unfinished_node.is_none()
                || self.run_state()?.run_progression.pending_continuation.is_some_and(|pending|
                    pending.cause == crate::game::expedition_lifecycle::ExpeditionContinuationCause::Gate),
            GameState::InReward { .. } | GameState::InRewardClaimed { .. } => {
                self.state.combat_reward_continuation.is_some()
            }
            _ => false,
        };
        if stable {
            // Read-only projections do not recapture or encode the run. Live
            // battle operations likewise retain the immutable pre-entry root.
            if matches!(
                route,
                ApplicationRoute::Behavior(
                    ActionKind::RequestMapData
                        | ActionKind::RequestRelicState
                        | ActionKind::RequestRelicNumericPlan
                        | ActionKind::PreviewRelicCorruption
                        | ActionKind::PreviewRelicDismantle
                        | ActionKind::PreviewSkillFragmentBloom
                        | ActionKind::RequestBattleState
                        | ActionKind::RequestDeploymentRangePreview
                        | ActionKind::PreviewSkillTarget
                ) | ApplicationRoute::MergeEncounterCompendiumProfile
                    | ApplicationRoute::MergePlayerProfile
                    | ApplicationRoute::ResetPlayerProfile
                    | ApplicationRoute::BattleDisconnectPause
            ) {
                return Ok(());
            }
            self.state.continue_state.unfinished_node = None;
            let payload = self.capture_continue_payload()?;
            self.install_continue_payload(payload)?;
        }
        Ok(())
    }

    pub(in crate::game::world) fn restore_continue_document(
        &mut self,
        player_id: Uuid,
        bytes: &[u8],
        loads_used: u8,
        run_peak_certified_power: u64,
    ) -> Result<(), GameError> {
        self.ensure_continuation_operation()?;
        let hint: ContinueAllocationHint = super::parse_active_run_checkpoint(bytes)?;
        let snapshot = hint.continuation.savepoint.as_ref().and_then(|s| s.snapshot.as_ref());
        let resource_entries = hint.continuation.current.typed_resources.entries.0
            .checked_add(snapshot.map_or(0, |s| s.checkpoint.typed_resources.entries.0))
            .ok_or_else(|| invalid("resource decode count overflow"))?;
        crate::game::relic::allocation::admit_json_input(bytes, resource_entries)?;
        if hint.continuation.savepoint.as_ref().is_some_and(|s| s.snapshot.is_some()) {
            crate::game::relic::allocation::admit_arc(
                std::mem::size_of::<RunCheckpointPayload>(),
                "relic_continue_decode_savepoint_arc",
            )?;
        }
        let header: super::ActiveRunCorePayloadHeader = super::parse_active_run_checkpoint(bytes)?;
        if header.version != VERSION {
            return Err(invalid(&format!(
                "unsupported active-run core payload version {}",
                header.version
            )));
        }
        let document: ContinueDocument = super::parse_active_run_checkpoint(bytes)?;
        crate::game::relic::allocation::admit_json_value(&document.continuation)?;
        let encoded = seal_document(canonical_value(&document.continuation, true)?,
            Some(&document.continuation_sha256))?;
        let payload = document.continuation.resolve()?;
        if payload.current.run_progression.expedition_instance.is_some_and(|instance| instance.player_id != player_id) {
            return Err(invalid("continuation belongs to another player"));
        }
        self.validate_continue_payload(&payload)?;
        let next = payload
            .result
            .as_ref()
            .map_or(&payload.phase, |result| &result.continuation_phase);
        if matches!(next, GameState::AwaitingRunFailureDecision { .. })
            && (payload.savepoint.is_none()
                || loads_used >= super::super::state::RUN_CHECKPOINT_MAX_LOADS)
        {
            return Err(invalid("failure decision has no available SavePoint"));
        }
        // Preserve the validated immutable continuation before moving gameplay
        // roots into the draft. Only current is independently copied here;
        // the shared SavePoint uses the same preflight in from_shared below.
        payload.current.preflight_copy()?;
        let retained = crate::authoring::clone_shape::controlled_clone(&payload)?;
        self.run_seed = payload.current.run_progression.run_seed;
        self.state.initialize_player(player_id);
        self.state.uuid_manager =
            crate::game::managers::uuid_manager::UuidManager::for_instance(self.run_seed, payload.current.run_progression.instance_namespace);
        self.state.run_checkpoint.payload = payload.savepoint
            .map(RunCheckpointPayload::from_shared)
            .transpose()?;
        self.state.run_checkpoint.loads_used = loads_used;
        self.state.run_power_authority.run_peak_certified_power = run_peak_certified_power;
        payload.current.restore_into_state(&mut self.state)?;
        self.run_state_mut()?.combat_recovery_restriction = payload.recovery_restriction;
        self.state.confirmed_combat_result = payload.result;
        self.state.combat_reward_continuation = payload.reward_continuation;
        self.state.active_node_content = payload.reward.map(ActiveNodeContent::Reward);
        self.state.visual_novel = payload.visual_novel;
        self.state.node_session = payload.pending_node_session;
        self.state.visual_novel_triggers.persistent_consumed =
            payload.persistent_vn_consumed.into_iter().collect();
        self.state.continue_state = Default::default();
        self.transition_to(payload.phase)?;
        self.cache_continue_payload(retained, encoded)?;
        Ok(())
    }

    fn validate_continue_payload(&self, payload: &ContinuePayload) -> Result<(), GameError> {
        self.validate_run_checkpoint_payload_for_continue(&payload.current)?;
        self.validate_pending_expedition_selection(&payload.current.run_progression, &payload.current.map,
            &payload.current.map_progression, &payload.phase, payload.pending_node_session.as_ref())?;
        if let Some(checkpoint) = &payload.savepoint {
            if checkpoint.run_progression.pending_continuation.is_some() { return Err(invalid("SavePoint cannot carry pending expedition choice")); }
            self.validate_run_checkpoint_payload_for_continue(checkpoint)?;
            if checkpoint.run_progression.run_seed != payload.current.run_progression.run_seed
                || checkpoint.run_progression.instance_namespace != payload.current.run_progression.instance_namespace {
                return Err(invalid("SavePoint belongs to another run"));
            }
        }
        if payload
            .current
            .abnormality_attempts
            .iter()
            .any(|(node, attempt)| {
                payload.current.facility_run_state.node(&payload.current.map,*node).is_none()
                    || attempt.retreats_used > 2
                    || attempt.max_attempts != 3
                    || attempt.attempts_started > 3
                    || attempt.attempts_started < attempt.retreats_used
            })
        {
            return Err(invalid("invalid retreat ledger"));
        }
        if payload
            .recovery_restriction
            .as_ref()
            .is_some_and(|restriction| {
                restriction
                    .allowed_node_ids
                    .iter()
                    .any(|node| payload.current.facility_run_state.node(&payload.current.map,*node).is_none())
            })
        {
            return Err(invalid("invalid recovery restriction"));
        }
        let mut session_ids = std::collections::HashSet::new();
        for session in payload
            .visual_novel
            .active
            .iter()
            .chain(payload.visual_novel.queue.iter())
        {
            let definition = self
                .game_data
                .visual_novel_data
                .get_by_id(session.vn_id.as_str())
                .ok_or_else(|| invalid("unknown visual novel"))?;
            let scene = definition
                .scene_by_id(session.current_scene_id.as_str())
                .ok_or_else(|| invalid("unknown visual novel scene"))?;
            if session.session_id.is_empty() || !session_ids.insert(&session.session_id) {
                return Err(invalid("duplicate or missing visual novel identity"));
            }
            if let Some(choice_id) = &session.committed_choice_id {
                if !matches!(&scene.next, crate::game::data::visual_novel_data::VisualNovelSceneNext::Choices { choices } if choices.iter().any(|choice| &choice.id == choice_id))
                {
                    return Err(invalid("unknown visual novel choice"));
                }
            }
        }
        let mut consumed = std::collections::HashSet::new();
        for key in &payload.persistent_vn_consumed {
            use crate::game::{
                data::visual_novel_trigger_data::{
                    VisualNovelTriggerCheckpointPolicy as Policy,
                    VisualNovelTriggerRepeat as Repeat,
                },
                visual_novel::VisualNovelTriggerConsumedScope as Scope,
            };
            let trigger = self
                .game_data
                .visual_novel_trigger_data
                .get_by_id(key.trigger_id.as_str())
                .ok_or_else(|| invalid("unknown persistent visual novel trigger"))?;
            if !consumed.insert(key)
                || trigger.checkpoint_policy != Policy::PersistAcrossCheckpointLoad
                || !matches!(
                    (trigger.repeat, key.scope),
                    (Repeat::OncePerRun, Scope::Run)
                        | (Repeat::OncePerBattle, Scope::Battle { .. })
                        | (Repeat::OncePerNode, Scope::Node { .. })
                )
            {
                return Err(invalid("invalid persistent visual novel trigger"));
            }
        }
        let valid_return = |phase: &GameState| {
            matches!(
                phase,
                GameState::ViewingMap
                    | GameState::AwaitingRunFailureDecision { .. }
                    | GameState::RunComplete
                    | GameState::RunFailed { .. }
                    | GameState::GameOver
            )
        };
        match (&payload.reward, &payload.reward_continuation) {
            (None, None) => {}
            (Some(reward), Some(context)) => {
                let mut ids = std::collections::HashSet::new();
                if reward.mode != crate::game::enums::RewardMode::ChooseOne
                    || reward.rewards.is_empty()
                    || reward.can_skip
                    || !reward.rewards.iter().all(|reward| ids.insert(reward.uuid))
                    || reward
                        .selected_reward_uuid
                        .is_some_and(|id| !ids.contains(&id))
                    || !valid_return(&context.return_phase)
                    || context.battle_uuid.is_nil()
                    || context
                        .context
                        .combat_participant_employee_ids
                        .iter()
                        .flatten()
                        .any(|id| payload.current.roster.get(id).is_none())
                {
                    return Err(invalid("invalid pending combat reward"));
                }
                let mut participants = std::collections::HashSet::new();
                if context.context.selected_employee_id.is_some()
                    || context.context.combat_participant_employee_ids.is_none()
                    || context
                        .context
                        .combat_participant_employee_ids
                        .iter()
                        .flatten()
                        .any(|id| !participants.insert(id))
                    || reward.stage_uuid.is_nil()
                {
                    return Err(invalid("invalid combat grant context"));
                }
                if let Some(hatched)=payload.current.encounter_progression.revival.nodes.get(&crate::game::map::MapNodeId(reward.stage_uuid)) {
                    let stage = hatched.stage_policy(&self.game_data.pve_data)
                        .map_err(|_| invalid("pending revival reward source is unavailable"))?;
                    let expected = stage.equipment_reward_options(&self.game_data.equipment_data)?;
                    if !hatched.resolved || reward.rewards!=expected {
                        return Err(invalid("pending revival reward differs from retained content"));
                    }
                } else {
                for option in &reward.rewards {
                    let definition = self
                        .game_data
                        .reward_data
                        .get_by_uuid(&option.uuid)
                        .ok_or_else(|| invalid("unknown pending reward"))?;
                    let original = crate::game::reward::RewardOption::from_metadata(definition);
                    let multiplier = payload
                        .current
                        .map_progression
                        .current_node_id
                        .and_then(|node| payload.current.map.node(node))
                        .and_then(|node| node.combat_difficulty.as_ref())
                        .and_then(|difficulty| {
                            self.run_policy()
                                .resolved_tuning_for_manifested_node(difficulty)
                        })
                        .map_or(100, |tuning| tuning.reward_multiplier_percent);
                    if !crate::game::combat_setup::rewards::matches_retained_combat_reward(
                        option,
                        &original,
                        multiplier,
                    ) {
                        return Err(invalid("pending reward differs from retained content"));
                    }
                }
                }
            }
            _ => return Err(invalid("reward continuation is incomplete")),
        }
        match (&payload.phase, &payload.result) {
            (GameState::CombatResult { battle_uuid }, Some(result)) => {
                if *battle_uuid != result.battle_uuid
                    || battle_uuid.is_nil()
                    || !valid_return(&result.continuation_phase)
                    || payload.reward_continuation.as_ref().is_some_and(|context| {
                        context.battle_uuid != *battle_uuid
                            || context.return_phase != result.continuation_phase
                    })
                    || !matches!(
                        &result.summary,
                        crate::game::behavior::SelectedEventSnapshotDto::CombatBattle {
                            has_event_log: false,
                            ..
                        }
                    )
                {
                    return Err(invalid("invalid confirmed result"));
                }
                use crate::game::behavior::{BehaviorResult, SelectedEventSnapshotDto};
                let SelectedEventSnapshotDto::CombatBattle {
                    verdict,
                    winner,
                    reward_mode,
                    rewards,
                    no_reward_candidates,
                    ..
                } = &result.summary
                else {
                    unreachable!()
                };
                if *winner != verdict.winner()
                    || *no_reward_candidates
                        != (*reward_mode == crate::game::enums::RewardMode::ChooseOne
                            && rewards.is_empty())
                {
                    return Err(invalid("incoherent confirmed outcome"));
                }
                let (completion, granted) = match result.acknowledgement.as_ref() {
                    BehaviorResult::CombatRewardsGranted { completion, .. } => {
                        (completion.as_ref(), true)
                    }
                    completion => (completion, false),
                };
                if !acknowledgement_matches_phase(completion, &result.continuation_phase)
                    || granted && *verdict != crate::game::battle::types::BattleVerdict::Victory
                {
                    return Err(invalid("acknowledgement differs from confirmed phase"));
                }
                let requires_choice = granted
                    && *reward_mode == crate::game::enums::RewardMode::ChooseOne
                    && !rewards.is_empty();
                if requires_choice != payload.reward.is_some() {
                    return Err(invalid("confirmed choice entitlement is incomplete"));
                }
                if let Some(reward) = &payload.reward {
                    if reward.selected_reward_uuid.is_some()
                        || rewards.len() != reward.rewards.len()
                        || rewards
                            .iter()
                            .zip(&reward.rewards)
                            .any(|(display, option)| {
                                display.uuid != option.uuid
                                    || display.id != option.id
                                    || display.name != option.name
                                    || display.effects != option.effects
                                    || display.description != option.description
                                    || display.icon != option.icon
                                    || display.kind != "reward"
                                    || display.rarity.is_some()
                                    || display.price != 0
                                    || display.grant_kinds != option.grant_kinds()
                            })
                    {
                        return Err(invalid("confirmed candidates differ from entitlement"));
                    }
                }
            }
            (
                GameState::InReward { reward_uuid } | GameState::InRewardClaimed { reward_uuid },
                None,
            ) => {
                if payload
                    .reward
                    .as_ref()
                    .is_none_or(|reward| reward.stage_uuid != *reward_uuid)
                    || matches!(payload.phase, GameState::InRewardClaimed { .. })
                        && payload
                            .reward
                            .as_ref()
                            .is_none_or(|reward| reward.selected_reward_uuid.is_none())
                {
                    return Err(invalid("invalid combat reward phase"));
                }
            }
            (GameState::NodeConfirm { .. }, None) if payload.current.run_progression.pending_continuation.is_some() && payload.reward.is_none() => {},
            (GameState::ViewingMap | GameState::AwaitingRunFailureDecision { .. }, None)
                if payload.reward.is_none() => {}
            _ => return Err(invalid("unsupported continuation phase")),
        }
        if payload
            .result
            .as_ref()
            .is_some_and(|result| terminal(&result.continuation_phase))
            && payload.reward.is_none()
        {
            return Err(invalid("terminal run cannot continue"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::control::{
        self, CancellationToken, ExecutionLimits, ExecutionPhase, NativeStorageProvenance,
    };
    use crate::game::{
        data::content_pack::{ContentContract, ContentIdentity},
        map::{test_facility_map, GameMode, MapProgression, RunProgression},
        revival::{HatchedEncounter, RevivalStage},
        world::state::{GameCoreState, RunState},
    };

    #[test]
    fn u17_shared_savepoint_import_charges_revival_tree_before_clone() {
        let map = test_facility_map(41);
        let run = RunState::new(
            map.clone(),
            MapProgression::from_map(&map),
            RunProgression::new(41, GameMode::Tutorial, 1),
        );
        let mut current = RunCheckpointPayload::capture(
            &GameCoreState::new(41),
            &run,
            &ContentIdentity {
                contract: ContentContract::current(),
                semantic_sha256: "test".into(),
            },
        )
        .unwrap();
        for id in 1..=64 {
            current
                .encounter_progression
                .revival
                .nodes
                .insert(
                    MapNodeId(Uuid::from_u128(id)),
                    HatchedEncounter { reward_source_id: "test".into(), stage: RevivalStage::Stage2, resolved: false, payment: Default::default() },
                );
        }
        let wire = ContinuePayloadWire {
            current,
            savepoint: Some(ContinueSavePointWire::Current {}),
            phase: GameState::ViewingMap,
            pending_node_session: None,
            recovery_restriction: None,
            result: None,
            reward: None,
            reward_continuation: None,
            visual_novel: Default::default(),
            persistent_vn_consumed: Vec::new(),
        };
        // Exercise the fresh serde owner, whose actual table capacity is allowed
        // to seed import provenance. Existing arbitrary roots remain uncertified.
        let bytes = serde_json::to_vec(&wire).unwrap();
        let fresh = || serde_json::from_slice::<ContinuePayloadWire>(&bytes).unwrap();
        let baseline = fresh();
        let limits = ExecutionLimits::default();
        let token = CancellationToken::default();
        let phase = ExecutionPhase::TrialPreparation;
        let (expected, usage, _) = control::run_with_provenance(
            &limits,
            &token,
            phase,
            NativeStorageProvenance::fresh(),
            || {
                crate::authoring::clone_shape::controlled_clone(&baseline.current)
            },
        )
        .unwrap();
        expected.unwrap();
        let input = fresh();
        let (resolved, actual, _) = control::run_with_provenance(
            &limits,
            &token,
            phase,
            NativeStorageProvenance::fresh(),
            || input.resolve(),
        )
        .unwrap();
        let resolved = resolved.unwrap();
        assert_eq!(actual.clone_slots_peak, usage.clone_slots_peak);
        assert!(actual.work > usage.work,
            "wire alias also admits its Arc owner, including without relics");
        assert_eq!(
            resolved.current.encounter_progression.revival,
            resolved
                .savepoint
                .as_ref()
                .unwrap()
                .encounter_progression
                .revival
        );
        let mut exact = limits.clone();
        exact.max_clone_slots = usage.clone_slots_peak;
        let input = fresh();
        control::run_with_provenance(
            &exact,
            &token,
            phase,
            NativeStorageProvenance::fresh(),
            || input.resolve(),
        )
        .unwrap()
        .0
        .unwrap();
        let mut short = limits;
        short.max_clone_slots = usage.clone_slots_peak - 1;
        let input = fresh();
        let failure = control::run_with_provenance(
            &short,
            &token,
            phase,
            NativeStorageProvenance::fresh(),
            || input.resolve(),
        )
        .err()
        .unwrap();
        assert_eq!(failure.code, "authoring_clone_limit");
        let input = fresh();
        // A BTreeMap's native clone shape is bounded by length; it does not
        // need the deleted HashMap's tombstone/capacity provenance.
        control::run(&ExecutionLimits::default(), &token, phase, || input.resolve())
            .unwrap().0.unwrap();
    }
}
