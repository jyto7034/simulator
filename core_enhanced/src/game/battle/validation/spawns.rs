use std::collections::HashMap;

use uuid::Uuid;

use crate::game::{
    battle::{
        ids::UnitInstanceId,
        timeline::{SkillCastTarget, Timeline, TimelineEvent},
    },
    enums::Side,
    stats::UnitStats,
};

use super::{
    battlefield::BattlefieldSize,
    types::{
        TimelineExpectedCounts, TimelineValidatorConfig, TimelineViolation, TimelineViolationKind,
    },
};

pub(super) struct ExtractedSpawns {
    pub(super) counts: TimelineExpectedCounts,
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
    timeline: &Timeline,
    battlefield: &Option<BattlefieldSize>,
    violations: &mut Vec<TimelineViolation>,
    config: &TimelineValidatorConfig,
) -> ExtractedSpawns {
    let mut extracted = ExtractedSpawns {
        counts: TimelineExpectedCounts::default(),
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

    for (index, entry) in timeline.entries.iter().enumerate() {
        match entry.event {
            TimelineEvent::UnitSpawned {
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
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::DuplicateUnitSpawn,
                        message: format!("unit {} spawned multiple times", unit_instance_id),
                        entry_index: Some(index),
                    });
                }
                extracted
                    .unit_base_by_instance
                    .insert(unit_instance_id, base_uuid);

                let _ = (battlefield, config);
            }
            TimelineEvent::ItemSpawned {
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
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::DuplicateItemSpawn,
                        message: format!("item {} spawned multiple times", item_instance_id),
                        entry_index: Some(index),
                    });
                }

                let Some(&unit_side) = extracted
                    .unit_owner_by_instance
                    .get(&owner_unit_instance_id)
                else {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::UnknownUnitReference,
                        message: format!(
                            "item {} spawned for unknown unit {}",
                            item_instance_id, owner_unit_instance_id
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if unit_side != owner {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::UnknownUnitReference,
                        message: format!(
                            "item {} owner side {:?} mismatches owner unit side {:?} (unit={})",
                            item_instance_id, owner, unit_side, owner_unit_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }
            }
            TimelineEvent::ArtifactSpawned {
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
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::DuplicateArtifactSpawn,
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
    actual: TimelineExpectedCounts,
    expected: Option<TimelineExpectedCounts>,
    violations: &mut Vec<TimelineViolation>,
) {
    let Some(expected) = expected else {
        return;
    };

    if actual.units != expected.units {
        violations.push(TimelineViolation {
            kind: TimelineViolationKind::UnitSpawnCountMismatch,
            message: format!(
                "unit spawn count mismatch: expected={}, got={}",
                expected.units, actual.units
            ),
            entry_index: None,
        });
    }
    if actual.items != expected.items {
        violations.push(TimelineViolation {
            kind: TimelineViolationKind::ItemSpawnCountMismatch,
            message: format!(
                "item spawn count mismatch: expected={}, got={}",
                expected.items, actual.items
            ),
            entry_index: None,
        });
    }
    if actual.artifacts != expected.artifacts {
        violations.push(TimelineViolation {
            kind: TimelineViolationKind::ArtifactSpawnCountMismatch,
            message: format!(
                "artifact spawn count mismatch: expected={}, got={}",
                expected.artifacts, actual.artifacts
            ),
            entry_index: None,
        });
    }
}

pub(super) fn validate_reference_spawn_order(
    timeline: &Timeline,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<TimelineViolation>,
) {
    for (index, entry) in timeline.entries.iter().enumerate() {
        let time_ms = entry.time_ms;
        let entry_desc = match &entry.event {
            TimelineEvent::BattleStart { .. } => "BattleStart".to_string(),
            TimelineEvent::BattleEnd { .. } => "BattleEnd".to_string(),
            other => format!("{:?}", other),
        };

        match &entry.event {
            TimelineEvent::UnitSpawned {
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
            TimelineEvent::MovementSegmentStarted {
                unit_instance_id, ..
            }
            | TimelineEvent::MovementStopped {
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
            TimelineEvent::AttackStart {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::AttackMiss {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::ProjectileMiss {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::BasicAttackProjectileLaunched {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::BasicAttackProjectileImpacted {
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
            TimelineEvent::AutoCastStart {
                caster_instance_id,
                target,
                ..
            }
            | TimelineEvent::ManualCastStart {
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
            TimelineEvent::AutoCastEnd { caster_instance_id }
            | TimelineEvent::ManualCastEnd { caster_instance_id } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
            }
            TimelineEvent::TriggeredAbilityProc {
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
            TimelineEvent::AbilityCast {
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
            TimelineEvent::AbilityStepTriggered {
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
            TimelineEvent::SkillAreaDeclared {
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
            TimelineEvent::SkillProjectileLaunched {
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
            TimelineEvent::SkillProjectileImpacted {
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
            TimelineEvent::BuffApplied {
                caster_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::BuffTick {
                caster_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::BuffExpired {
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
            TimelineEvent::HpChanged {
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
            TimelineEvent::StatChanged {
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
            TimelineEvent::ResonanceChanged {
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
            TimelineEvent::UnitDied {
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
            TimelineEvent::ItemSpawned {
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
            TimelineEvent::ArtifactSpawned { .. }
            | TimelineEvent::BattleStart { .. }
            | TimelineEvent::BattleEnd { .. } => {}
        }
    }
}

fn validate_unit_reference_spawned(
    extracted: &ExtractedSpawns,
    unit_id: UnitInstanceId,
    reference_index: usize,
    reference_time_ms: u64,
    entry_desc: &str,
    violations: &mut Vec<TimelineViolation>,
) {
    let Some(&spawn_index) = extracted.unit_spawn_index.get(&unit_id) else {
        violations.push(TimelineViolation {
            kind: TimelineViolationKind::UnknownUnitReference,
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
        violations.push(TimelineViolation {
            kind: TimelineViolationKind::UnitReferencedBeforeSpawn,
            message: format!(
                "unit {} referenced before spawn (ref_index={} ref_time_ms={} spawn_index={} spawn_time_ms={})",
                unit_id, reference_index, reference_time_ms, spawn_index, spawn_time_ms
            ),
            entry_index: Some(reference_index),
        });
    }
}
