//! Non-authoritative developer combat trace output.
//!
//! These files are never gameplay, save, checkpoint, or profile state. Callers
//! must treat every export error as diagnostic-only and continue the gameplay
//! transition unchanged.

use std::{fs::File, io::BufWriter, path::PathBuf};

use serde::Serialize;
use uuid::Uuid;

use crate::game::{
    battle::{
        event_log::BattleEventLog,
        result_stats::BattleResultStatsDto,
        types::{BattleOutcomeId, BattleVerdict, BattleWinner, ParticipantBattleResult},
    },
    behavior::GameError,
    combat_preview::{CombatMissionVariant, CombatNodeType},
    resources::CombatBattleState,
};

#[derive(Debug, Serialize)]
struct CombatTraceDebugExport<'a> {
    version: u32,
    run_seed: u64,
    battle_uuid: Uuid,
    combat_node_uuid: Uuid,
    primary_abnormality_id: Option<&'a str>,
    encounter_id: &'a str,
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    outcome_id: &'a BattleOutcomeId,
    verdict: BattleVerdict,
    winner: BattleWinner,
    participant_results: &'a [ParticipantBattleResult],
    result_stats: &'a BattleResultStatsDto,
    event_log: &'a BattleEventLog,
}

pub(super) fn debug_combat_trace_path(run_seed: u64, battle_uuid: Uuid) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("combat_traces")
        .join(format!("run_{run_seed:016x}"))
        .join(format!("{battle_uuid}.json"))
}

pub(super) fn export_debug_combat_trace(
    run_seed: u64,
    battle_uuid: Uuid,
    battle: &CombatBattleState,
) -> Result<PathBuf, GameError> {
    let out_path = debug_combat_trace_path(run_seed, battle_uuid);
    let out_dir = out_path
        .parent()
        .ok_or_else(|| GameError::InvalidStaticData("combat trace path has no parent".into()))?;
    std::fs::create_dir_all(out_dir).map_err(|error| {
        GameError::InvalidStaticData(format!(
            "failed to create debug combat trace directory '{}': {error}",
            out_dir.display()
        ))
    })?;
    let file = File::create(&out_path).map_err(|error| {
        GameError::InvalidStaticData(format!(
            "failed to create debug combat trace file '{}': {error}",
            out_path.display()
        ))
    })?;
    let export = CombatTraceDebugExport {
        version: 2,
        run_seed,
        battle_uuid,
        combat_node_uuid: battle.abnormality_uuid,
        primary_abnormality_id: battle.primary_abnormality_id.as_deref(),
        encounter_id: &battle.encounter_id,
        node_type: battle.node_type,
        mission_variant: battle.mission_variant,
        outcome_id: &battle.outcome_id,
        verdict: battle.verdict,
        winner: battle.winner,
        participant_results: &battle.participant_results,
        result_stats: &battle.result_stats,
        event_log: &battle.event_log,
    };
    serde_json::to_writer_pretty(BufWriter::new(file), &export).map_err(|error| {
        GameError::InvalidStaticData(format!(
            "failed to write debug combat trace file '{}': {error}",
            out_path.display()
        ))
    })?;
    Ok(out_path)
}
