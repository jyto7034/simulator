use std::collections::HashMap;

use uuid::Uuid;

use crate::game::{
    battle::{
        event_log::{BattleEventLog, BattleLogEvent, SkillCastTarget},
        ids::UnitInstanceId,
    },
    enums::Side,
    stats::UnitStats,
};

use super::{
    battlefield::BattlefieldSize,
    types::{
        EventLogExpectedCounts, EventLogValidatorConfig, EventLogViolation, EventLogViolationKind,
    },
};

pub(super) struct ExtractedSpawns {
    pub(super) counts: EventLogExpectedCounts,
    pub(super) unit_spawn_index: HashMap<UnitInstanceId, usize>,
    pub(super) unit_spawn_time_ms: HashMap<UnitInstanceId, u64>,
    pub(super) unit_spawn_stats: HashMap<UnitInstanceId, UnitStats>,
    pub(super) unit_owner_by_instance: HashMap<UnitInstanceId, Side>,
    pub(super) unit_base_by_instance: HashMap<UnitInstanceId, Uuid>,
    pub(super) item_spawn_index: HashMap<Uuid, usize>,
    pub(super) item_owner_by_instance: HashMap<Uuid, Side>,
    pub(super) artifact_spawn_index: HashMap<Uuid, usize>,
    pub(super) artifact_owner_by_instance: HashMap<Uuid, Side>,
}

pub(super) fn extract_spawns(
    event_log: &BattleEventLog,
    battlefield: &Option<BattlefieldSize>,
    violations: &mut Vec<EventLogViolation>,
    config: &EventLogValidatorConfig,
) -> ExtractedSpawns {
    let mut extracted = ExtractedSpawns {
        counts: EventLogExpectedCounts::default(),
        unit_spawn_index: HashMap::new(),
        unit_spawn_time_ms: HashMap::new(),
        unit_spawn_stats: HashMap::new(),
        unit_owner_by_instance: HashMap::new(),
        unit_base_by_instance: HashMap::new(),
        item_spawn_index: HashMap::new(),
        item_owner_by_instance: HashMap::new(),
        artifact_spawn_index: HashMap::new(),
        artifact_owner_by_instance: HashMap::new(),
    };

    for (index, entry) in event_log.entries.iter().enumerate() {
        match entry.event {
            BattleLogEvent::UnitSpawned {
                unit_instance_id,
                owner,
                base_uuid,
                stats,
                ..
            } => {
                extracted.counts.units += 1;
                extracted
                    .unit_spawn_index
                    .entry(unit_instance_id)
                    .and_modify(|existing| *existing = (*existing).min(index))
                    .or_insert(index);
                extracted
                    .unit_spawn_time_ms
                    .entry(unit_instance_id)
                    .and_modify(|existing| *existing = (*existing).min(entry.time_ms))
                    .or_insert(entry.time_ms);
                extracted
                    .unit_spawn_stats
                    .entry(unit_instance_id)
                    .or_insert(stats);
                if extracted
                    .unit_owner_by_instance
                    .insert(unit_instance_id, owner)
                    .is_some()
                {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::DuplicateUnitSpawn,
                        message: format!("unit {} spawned multiple times", unit_instance_id),
                        entry_index: Some(index),
                    });
                }
                extracted
                    .unit_base_by_instance
                    .insert(unit_instance_id, base_uuid);

                let _ = (battlefield, config);
            }
            BattleLogEvent::ItemSpawned {
                item_instance_id,
                owner,
                owner_unit_instance_id,
                ..
            } => {
                extracted.counts.items += 1;
                extracted
                    .item_spawn_index
                    .entry(item_instance_id)
                    .and_modify(|existing| *existing = (*existing).min(index))
                    .or_insert(index);
                if extracted
                    .item_owner_by_instance
                    .insert(item_instance_id, owner)
                    .is_some()
                {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::DuplicateItemSpawn,
                        message: format!("item {} spawned multiple times", item_instance_id),
                        entry_index: Some(index),
                    });
                }

                let Some(&unit_side) = extracted
                    .unit_owner_by_instance
                    .get(&owner_unit_instance_id)
                else {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::UnknownUnitReference,
                        message: format!(
                            "item {} spawned for unknown unit {}",
                            item_instance_id, owner_unit_instance_id
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if unit_side != owner {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::UnknownUnitReference,
                        message: format!(
                            "item {} owner side {:?} mismatches owner unit side {:?} (unit={})",
                            item_instance_id, owner, unit_side, owner_unit_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }
            }
            BattleLogEvent::ArtifactSpawned {
                artifact_instance_id,
                owner,
                ..
            } => {
                extracted.counts.artifacts += 1;
                extracted
                    .artifact_spawn_index
                    .entry(artifact_instance_id)
                    .and_modify(|existing| *existing = (*existing).min(index))
                    .or_insert(index);
                if extracted
                    .artifact_owner_by_instance
                    .insert(artifact_instance_id, owner)
                    .is_some()
                {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::DuplicateArtifactSpawn,
                        message: format!(
                            "artifact {} spawned multiple times",
                            artifact_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }
            }
            _ => {}
        }
    }

    extracted
}

pub(super) fn validate_spawn_counts(
    actual: EventLogExpectedCounts,
    expected: Option<EventLogExpectedCounts>,
    violations: &mut Vec<EventLogViolation>,
) {
    let Some(expected) = expected else {
        return;
    };

    if actual.units != expected.units {
        violations.push(EventLogViolation {
            kind: EventLogViolationKind::UnitSpawnCountMismatch,
            message: format!(
                "unit spawn count mismatch: expected={}, got={}",
                expected.units, actual.units
            ),
            entry_index: None,
        });
    }
    if actual.items != expected.items {
        violations.push(EventLogViolation {
            kind: EventLogViolationKind::ItemSpawnCountMismatch,
            message: format!(
                "item spawn count mismatch: expected={}, got={}",
                expected.items, actual.items
            ),
            entry_index: None,
        });
    }
    if actual.artifacts != expected.artifacts {
        violations.push(EventLogViolation {
            kind: EventLogViolationKind::ArtifactSpawnCountMismatch,
            message: format!(
                "artifact spawn count mismatch: expected={}, got={}",
                expected.artifacts, actual.artifacts
            ),
            entry_index: None,
        });
    }
}

pub(super) fn validate_reference_spawn_order(
    event_log: &BattleEventLog,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<EventLogViolation>,
) {
    for (index, entry) in event_log.entries.iter().enumerate() {
        let time_ms = entry.time_ms;
        let entry_desc = match &entry.event {
            BattleLogEvent::BattleStart { .. } => "BattleStart".to_string(),
            BattleLogEvent::BattleEnd { .. } => "BattleEnd".to_string(),
            other => format!("{:?}", other),
        };

        match &entry.event {
            BattleLogEvent::UnitSpawned {
                unit_instance_id, ..
            }
            | BattleLogEvent::UnitDeployed {
                unit_instance_id, ..
            }
            | BattleLogEvent::UnitWithdrawn {
                unit_instance_id, ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *unit_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::MovementSegmentStarted {
                unit_instance_id, ..
            }
            | BattleLogEvent::MovementStopped {
                unit_instance_id, ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *unit_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::AttackStart {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | BattleLogEvent::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | BattleLogEvent::AttackMiss {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | BattleLogEvent::BasicAttackProjectileLaunched {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | BattleLogEvent::BasicAttackProjectileImpacted {
                attacker_instance_id,
                target_instance_id,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *attacker_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                validate_unit_reference_spawned(
                    extracted,
                    *target_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::AutoCastStart {
                caster_instance_id,
                target,
                ..
            }
            | BattleLogEvent::ManualCastStart {
                caster_instance_id,
                target,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                if let Some(SkillCastTarget::Unit { unit_instance_id }) = target {
                    validate_unit_reference_spawned(
                        extracted,
                        *unit_instance_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
            }
            BattleLogEvent::AutoCastEnd { caster_instance_id }
            | BattleLogEvent::ManualCastEnd { caster_instance_id } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::SkillCastInterrupted {
                interrupter_instance_id,
                caster_instance_id,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *interrupter_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::SkillCastCancelled {
                caster_instance_id, ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::TriggeredAbilityProc {
                caster_instance_id,
                target_instance_id,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                if let Some(target_id) = target_instance_id {
                    validate_unit_reference_spawned(
                        extracted,
                        *target_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
            }
            BattleLogEvent::AbilityCast {
                caster_instance_id,
                target_instance_id,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                if let Some(target_id) = target_instance_id {
                    validate_unit_reference_spawned(
                        extracted,
                        *target_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
            }
            BattleLogEvent::AbilityStepTriggered {
                caster_instance_id,
                target_instance_id,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                if let Some(target_id) = target_instance_id {
                    validate_unit_reference_spawned(
                        extracted,
                        *target_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
            }
            BattleLogEvent::SkillAreaDeclared {
                caster_instance_id,
                target,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                if let Some(SkillCastTarget::Unit { unit_instance_id }) = target {
                    validate_unit_reference_spawned(
                        extracted,
                        *unit_instance_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
            }
            BattleLogEvent::SkillProjectileLaunched {
                caster_instance_id,
                target,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                if let Some(SkillCastTarget::Unit { unit_instance_id }) = target {
                    validate_unit_reference_spawned(
                        extracted,
                        *unit_instance_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
            }
            BattleLogEvent::SkillProjectileImpacted {
                caster_instance_id,
                first_hit_unit_id,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                if let Some(unit_instance_id) = first_hit_unit_id {
                    validate_unit_reference_spawned(
                        extracted,
                        *unit_instance_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
            }
            BattleLogEvent::BuffApplied {
                caster_instance_id,
                target_instance_id,
                ..
            }
            | BattleLogEvent::BuffTick {
                caster_instance_id,
                target_instance_id,
                ..
            }
            | BattleLogEvent::BuffExpired {
                caster_instance_id,
                target_instance_id,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                validate_unit_reference_spawned(
                    extracted,
                    *target_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                ..
            } => {
                if let Some(source_id) = source_instance_id {
                    validate_unit_reference_spawned(
                        extracted,
                        *source_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
                validate_unit_reference_spawned(
                    extracted,
                    *target_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::StatChanged {
                source_instance_id,
                target_instance_id,
                ..
            } => {
                if let Some(source_id) = source_instance_id {
                    validate_unit_reference_spawned(
                        extracted,
                        *source_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
                validate_unit_reference_spawned(
                    extracted,
                    *target_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::ResonanceChanged {
                unit_instance_id, ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *unit_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::UnitDied {
                unit_instance_id,
                killer_instance_id,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *unit_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
                if let Some(killer_id) = killer_instance_id {
                    validate_unit_reference_spawned(
                        extracted,
                        *killer_id,
                        index,
                        time_ms,
                        &entry_desc,
                        violations,
                    );
                }
            }
            BattleLogEvent::ItemSpawned {
                owner_unit_instance_id,
                ..
            } => {
                validate_unit_reference_spawned(
                    extracted,
                    *owner_unit_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            BattleLogEvent::ArtifactSpawned { .. }
            | BattleLogEvent::BattleStart { .. }
            | BattleLogEvent::BattleEnd { .. } => {}
        }
    }
}

fn validate_unit_reference_spawned(
    extracted: &ExtractedSpawns,
    unit_id: UnitInstanceId,
    reference_index: usize,
    reference_time_ms: u64,
    entry_desc: &str,
    violations: &mut Vec<EventLogViolation>,
) {
    let Some(&spawn_index) = extracted.unit_spawn_index.get(&unit_id) else {
        violations.push(EventLogViolation {
            kind: EventLogViolationKind::UnknownUnitReference,
            message: format!("{entry_desc} references unknown unit {}", unit_id),
            entry_index: Some(reference_index),
        });
        return;
    };
    let spawn_time_ms = extracted
        .unit_spawn_time_ms
        .get(&unit_id)
        .copied()
        .unwrap_or_default();

    if spawn_index > reference_index || spawn_time_ms > reference_time_ms {
        violations.push(EventLogViolation {
            kind: EventLogViolationKind::UnitReferencedBeforeSpawn,
            message: format!(
                "unit {} referenced before spawn (ref_index={} ref_time_ms={} spawn_index={} spawn_time_ms={})",
                unit_id, reference_index, reference_time_ms, spawn_index, spawn_time_ms
            ),
            entry_index: Some(reference_index),
        });
    }
}
