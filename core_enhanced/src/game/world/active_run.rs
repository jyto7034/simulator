use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::state::GameCoreState;
use super::{state::RunCheckpointPayload, GameCore};
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::world::state::RUN_CHECKPOINT_MAX_LOADS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveRunCheckpointExport {
    pub core_payload_json: Vec<u8>,
    pub loads_used: u8,
    pub run_peak_certified_power: u64,
}

pub struct ActiveRunTransactionRollback {
    state: GameCoreState,
    run_seed: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActiveRunCorePayloadV2 {
    version: u32,
    checkpoint: RunCheckpointPayload,
}

const ACTIVE_RUN_CORE_PAYLOAD_VERSION: u32 = 2;

impl GameCore {
    pub fn capture_active_run_transaction_rollback(&self) -> Option<ActiveRunTransactionRollback> {
        self.state
            .clone_non_battle_transaction_state()
            .map(|state| ActiveRunTransactionRollback {
                state,
                run_seed: self.run_seed,
            })
    }

    pub fn restore_active_run_transaction_rollback(
        &mut self,
        mut rollback: ActiveRunTransactionRollback,
    ) {
        let encounter_compendium = self.state.encounter_compendium.clone();
        rollback
            .state
            .run_power_authority
            .merge_run_peak_monotonic(self.state.run_power_authority);
        self.state = rollback.state;
        // Local-install profile knowledge is a separate monotonic aggregate. An
        // active-run transaction rollback must not rewind discoveries or defeats
        // committed while that transaction was in flight.
        self.state.encounter_compendium.merge(encounter_compendium);
        self.run_seed = rollback.run_seed;
    }

    /// Export only the last completed SavePoint payload. The checkpoint owns its
    /// power-model/anchor generation; only loads and the run peak remain host
    /// monotonic metadata.
    pub fn export_active_run_checkpoint(
        &self,
    ) -> Result<Option<ActiveRunCheckpointExport>, GameError> {
        let Some(checkpoint) = self.state.run_checkpoint.payload.clone() else {
            return Ok(None);
        };
        self.validate_run_checkpoint_payload(&checkpoint)?;
        let core_payload_json = serde_json::to_vec(&ActiveRunCorePayloadV2 {
            version: ACTIVE_RUN_CORE_PAYLOAD_VERSION,
            checkpoint,
        })
        .map_err(|error| {
            GameError::InvalidStaticData(format!(
                "active-run checkpoint serialization failed: {error}"
            ))
        })?;
        Ok(Some(ActiveRunCheckpointExport {
            core_payload_json,
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
        if self.state.run.is_some() || loads_used > RUN_CHECKPOINT_MAX_LOADS {
            return Err(GameError::InvalidAction);
        }
        let document: ActiveRunCorePayloadV2 =
            serde_json::from_slice(core_payload_json).map_err(|error| {
                GameError::InvalidStaticData(format!(
                    "active-run checkpoint deserialization failed: {error}"
                ))
            })?;
        if document.version != ACTIVE_RUN_CORE_PAYLOAD_VERSION {
            return Err(GameError::InvalidStaticData(format!(
                "unsupported active-run core payload version {}",
                document.version
            )));
        }
        self.validate_run_checkpoint_payload(&document.checkpoint)?;

        let checkpoint = document.checkpoint;
        let run_seed = checkpoint.run_progression.run_seed;
        self.state.run_checkpoint.payload = Some(checkpoint.clone());
        self.state.run_checkpoint.loads_used = loads_used;
        self.state.run_power_authority = super::state::RunPowerAuthorityState {
            power_model_version: checkpoint.power_model_version,
            run_peak_certified_power,
            current_floor_power_anchor: checkpoint.current_floor_power_anchor,
        };
        self.state.initialize_player(player_id);
        self.run_seed = run_seed;
        self.state.uuid_manager = UuidManager::new(run_seed);
        self.restore_run_checkpoint_payload(checkpoint)?;
        self.handle_request_map_data()
    }
}
