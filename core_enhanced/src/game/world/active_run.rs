use crate::game::managers::uuid_manager::UuidManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::state::GameCoreState;
use super::{state::RunCheckpointPayload, GameCore};
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::world::state::RUN_CHECKPOINT_MAX_LOADS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActiveRunCheckpointExport {
    pub core_payload_json: std::sync::Arc<[u8]>,
    pub loads_used: u8,
    pub run_peak_certified_power: u64,
}

pub(crate) struct ActiveRunTransactionRollback {
    state: GameCoreState,
    run_seed: u64,
    game_data: crate::game::data::ValidatedGameData,
    authoring_lifecycle: Option<super::authoring_lifecycle::TrialLifecycle>,
    authoring_observation: Option<super::authoring_observation::TrialObservation>,
}

#[derive(Serialize)]
struct ActiveRunCorePayloadV12Ref<'a> {
    version: u32,
    checkpoint: &'a RunCheckpointPayload,
    checkpoint_sha256: String,
}

#[derive(Deserialize)]
struct ActiveRunCorePayloadHeader {
    version: u32,
}

const ACTIVE_RUN_CORE_PAYLOAD_VERSION: u32 = 12;
mod continuation;
pub(super) use continuation::ContinueState;
mod v7;
mod v8;

// Called only after the explicit legacy converter verifies the original checksum
// and content identity. Custom legacy resource policies cannot be mapped to the
// new employee-owned policy without losing meaning, so reject them.
fn convert_legacy_employee_resonance(
    checkpoint: &mut serde_json::Map<String, Value>,
) -> Result<(), GameError> {
    let bad =
        |message: &str| active_run_checkpoint_codec_error("employee resonance conversion", message);
    let employees = checkpoint
        .get_mut("roster")
        .and_then(|value| value.get_mut("employees"))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| bad("missing employee roster"))?;
    let supported = serde_json::json!({"start": 0, "max": 100, "gain_lock_ms": 1000});
    for employee in employees.values_mut() {
        let profile = employee
            .get_mut("combat_profile")
            .and_then(|value| value.get_mut("battle_profile"))
            .and_then(Value::as_object_mut)
            .ok_or_else(|| bad("missing employee battle profile"))?;
        if profile.get("resonance") != Some(&supported) {
            return Err(bad("unsupported legacy employee resonance policy"));
        }
        profile.remove("resonance");
    }
    Ok(())
}

fn active_run_checkpoint_codec_error(operation: &str, error: impl std::fmt::Display) -> GameError {
    GameError::InvalidStaticData(format!("active-run checkpoint {operation} failed: {error}"))
}

fn write_canonical_json(value: &Value, output: &mut Vec<u8>) -> Result<(), serde_json::Error> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_writer(output, value)?;
        }
        Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        Value::Object(values) => {
            output.push(b'{');
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                serde_json::to_writer(&mut *output, key)?;
                output.push(b':');
                write_canonical_json(&values[key], output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

pub(super) fn active_run_checkpoint_sha256(
    checkpoint: &RunCheckpointPayload,
) -> Result<String, GameError> {
    let value = serde_json::to_value(checkpoint)
        .map_err(|error| active_run_checkpoint_codec_error("serialization", error))?;
    let mut canonical_json = Vec::new();
    write_canonical_json(&value, &mut canonical_json)
        .map_err(|error| active_run_checkpoint_codec_error("serialization", error))?;
    let digest = Sha256::digest(canonical_json);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to a String is infallible");
    }
    Ok(encoded)
}

pub(super) fn encode_active_run_checkpoint(
    checkpoint: &RunCheckpointPayload,
) -> Result<Vec<u8>, GameError> {
    let checkpoint_sha256 = active_run_checkpoint_sha256(checkpoint)?;
    let value = serde_json::to_value(&ActiveRunCorePayloadV12Ref {
        version: ACTIVE_RUN_CORE_PAYLOAD_VERSION,
        checkpoint,
        checkpoint_sha256,
    })
    .map_err(|error| active_run_checkpoint_codec_error("serialization", error))?;
    let mut bytes = Vec::new();
    write_canonical_json(&value, &mut bytes)
        .map_err(|error| active_run_checkpoint_codec_error("serialization", error))?;
    Ok(bytes)
}

fn parse_active_run_checkpoint<T>(core_payload_json: &[u8]) -> Result<T, GameError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_slice(core_payload_json)
        .map_err(|error| active_run_checkpoint_codec_error("deserialization", error))
}

impl GameCore {
    pub fn capture_active_run_transaction_rollback(
        &self,
    ) -> Result<Option<ActiveRunTransactionRollback>, GameError> {
        let authoring_observation =
            crate::authoring::clone_shape::controlled_clone(&self.authoring_observation)?;
        Ok(self
            .state
            .clone_non_battle_transaction_state()?
            .map(|state| ActiveRunTransactionRollback {
                state,
                run_seed: self.run_seed,
                game_data: self.game_data.clone(),
                authoring_lifecycle: self.authoring_lifecycle,
                authoring_observation,
            }))
    }

    #[cfg(test)]
    pub fn restore_active_run_transaction_rollback(
        &mut self,
        rollback: ActiveRunTransactionRollback,
    ) {
        self.try_restore_active_run_transaction_rollback(rollback)
            .expect("captured rollback must contain a coherent private lifecycle context");
    }

    pub(crate) fn try_restore_active_run_transaction_rollback(
        &mut self,
        rollback: ActiveRunTransactionRollback,
    ) -> Result<(), GameError> {
        self.execute_application(
            super::application::ApplicationRoute::ActiveRunRollbackRestore,
            move |draft| {
                draft.restore_active_run_transaction_rollback_draft(rollback);
                Ok(())
            },
        )
    }

    fn restore_active_run_transaction_rollback_draft(
        &mut self,
        mut rollback: ActiveRunTransactionRollback,
    ) {
        let player_profile = self.state.player_profile.clone();
        rollback
            .state
            .run_power_authority
            .merge_run_peak_monotonic(self.state.run_power_authority);
        self.state = rollback.state;
        // Local-install profile knowledge is a separate monotonic aggregate. An
        // active-run transaction rollback must not rewind registrations committed
        // while that transaction was in flight.
        self.state.player_profile.merge(player_profile);
        self.run_seed = rollback.run_seed;
        self.game_data = rollback.game_data;
        self.authoring_lifecycle = rollback.authoring_lifecycle;
        self.authoring_observation = rollback.authoring_observation;
    }

    /// Export the cached confirmed continuation, independently of SavePoint.
    pub fn export_active_run_checkpoint(
        &self,
    ) -> Result<Option<ActiveRunCheckpointExport>, GameError> {
        Ok(self
            .state
            .continue_state
            .encoded
            .as_ref()
            .map(|encoded| ActiveRunCheckpointExport {
                core_payload_json: encoded.clone(),
                loads_used: self.state.run_checkpoint.loads_used,
                run_peak_certified_power: self.state.run_power_authority.run_peak_certified_power,
            }))
    }

    /// Restore a durable active run as a continue operation. This is not an
    /// explicit checkpoint load and therefore does not increment `loads_used`.
    /// The transport player identity is injected at bootstrap and is not part of
    /// the local-install active-run payload.
    pub fn continue_from_active_run_checkpoint(
        &mut self,
        player_id: Uuid,
        core_payload_json: &[u8],
        loads_used: u8,
        run_peak_certified_power: u64,
    ) -> Result<BehaviorResult, GameError> {
        self.execute_application(
            super::application::ApplicationRoute::ActiveRunContinue,
            move |draft| {
                draft.continue_from_active_run_checkpoint_draft(
                    player_id,
                    core_payload_json,
                    loads_used,
                    run_peak_certified_power,
                )
            },
        )
    }

    fn continue_from_active_run_checkpoint_draft(
        &mut self,
        player_id: Uuid,
        core_payload_json: &[u8],
        loads_used: u8,
        run_peak_certified_power: u64,
    ) -> Result<BehaviorResult, GameError> {
        if self.state.run.is_some() || loads_used > RUN_CHECKPOINT_MAX_LOADS {
            return Err(GameError::InvalidAction);
        }
        self.restore_continue_document(
            player_id,
            core_payload_json,
            loads_used,
            run_peak_certified_power,
        )?;
        self.handle_request_map_data()
    }
}
