use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::game::{
    battle::{
        buffs,
        timeline::{AttackKind, HpChangeReason, Timeline, TimelineEntry, TimelineEvent},
        PlayerDeckInfo,
    },
    enums::Side,
    stats::UnitStats,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BasicAttackHpChangeKey {
    cause_seq: u64,
    time_ms: u64,
    attacker_instance_id: Uuid,
    target_instance_id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineViolationKind {
    TimelineVersionMismatch,
    MissingEntries,
    MissingBattleStart,
    MissingBattleEnd,
    NonContiguousSeq,
    TimeWentBackwards,
    AttackKindMissing,
    AutoCastPairInvalid,
    OutcomeMissingCauseSeq,
    OutcomeCauseSeqOutOfRange,
    OutcomeCauseSeqInFuture,
    OutcomeCauseSeqInvalidType,
    SpawnStatsInvalid,
    StatsAfterInvalid,
    HpDeltaMismatch,
    HpBeforeMismatch,
    StatsBeforeMismatch,
    UnitDiedWhileAlive,
    BuffAppliedByDeadCaster,
    UnitSpawnCountMismatch,
    ItemSpawnCountMismatch,
    ArtifactSpawnCountMismatch,
    DuplicateUnitSpawn,
    DuplicateItemSpawn,
    DuplicateArtifactSpawn,
    UnknownUnitReference,
    UnitReferencedBeforeSpawn,
    AttackTargetsSameUnit,
    AttackTargetsAlly,
    AttackMissingBasicHpChanged,
    UnitDiedDuplicate,
    DeadUnitActsAfterDeath,
    AutoAttackTooEarly,
    MissingExpectedAutoAttack,
    UnknownBuffId,
    BuffAppliedDurationZero,
    BuffTickInvalid,
    BuffExpiredInvalid,
}

#[derive(Debug, Clone)]
pub struct TimelineViolation {
    pub kind: TimelineViolationKind,
    pub message: String,
    pub entry_index: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct TimelineExpectedCounts {
    pub units: usize,
    pub items: usize,
    pub artifacts: usize,
}

impl Default for TimelineExpectedCounts {
    fn default() -> Self {
        Self {
            units: 0,
            items: 0,
            artifacts: 0,
        }
    }
}

impl TimelineExpectedCounts {
    pub fn from_decks(player: &PlayerDeckInfo, opponent: &PlayerDeckInfo) -> Self {
        let units = player.units.len() + opponent.units.len();
        let items = player
            .units
            .iter()
            .chain(opponent.units.iter())
            .map(|unit| unit.equipped_items.len())
            .sum();
        let artifacts = player.artifacts.len() + opponent.artifacts.len();
        Self {
            units,
            items,
            artifacts,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimelineValidatorConfig {
    pub require_battle_start_end: bool,
    pub require_contiguous_seq: bool,
    pub require_non_decreasing_time: bool,
    pub validate_outcome_cause_seq: bool,
    pub require_autocast_pairs: bool,
    pub require_spawn_stats_valid: bool,
    pub require_hp_delta_consistent: bool,
    pub require_attack_has_basic_hp_change: bool,
    pub forbid_dead_units_as_attackers: bool,
    pub forbid_dead_units_as_targets: bool,
    pub validate_auto_attack_min_interval: bool,
    pub validate_auto_attack_presence: bool,
    pub auto_attack_timing_tolerance_ms: u64,
}

impl Default for TimelineValidatorConfig {
    fn default() -> Self {
        Self {
            require_battle_start_end: true,
            require_contiguous_seq: true,
            require_non_decreasing_time: true,
            validate_outcome_cause_seq: true,
            require_autocast_pairs: true,
            require_spawn_stats_valid: true,
            require_hp_delta_consistent: true,
            require_attack_has_basic_hp_change: true,
            forbid_dead_units_as_attackers: true,
            forbid_dead_units_as_targets: true,
            validate_auto_attack_min_interval: true,
            validate_auto_attack_presence: false,
            auto_attack_timing_tolerance_ms: 2,
        }
    }
}

pub struct TimelineValidator {
    config: TimelineValidatorConfig,
}

impl TimelineValidator {
    pub fn new(config: TimelineValidatorConfig) -> Self {
        Self { config }
    }

    pub fn validate(
        &self,
        timeline: &Timeline,
        expected_counts: Option<TimelineExpectedCounts>,
    ) -> Result<(), Vec<TimelineViolation>> {
        let mut violations = Vec::new();

        if timeline.version != crate::game::battle::timeline::TIMELINE_VERSION {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::TimelineVersionMismatch,
                message: format!(
                    "timeline version mismatch: expected={}, got={}",
                    crate::game::battle::timeline::TIMELINE_VERSION,
                    timeline.version
                ),
                entry_index: None,
            });
            return Err(violations);
        }

        if timeline.entries.is_empty() {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::MissingEntries,
                message: "timeline has no entries".to_string(),
                entry_index: None,
            });
            return Err(violations);
        }

        if self.config.require_battle_start_end {
            if !matches!(
                timeline.entries.first().map(|e| &e.event),
                Some(TimelineEvent::BattleStart { .. })
            ) {
                violations.push(TimelineViolation {
                    kind: TimelineViolationKind::MissingBattleStart,
                    message: "timeline does not start with BattleStart".to_string(),
                    entry_index: Some(0),
                });
            }

            if !matches!(
                timeline.entries.last().map(|e| &e.event),
                Some(TimelineEvent::BattleEnd { .. })
            ) {
                violations.push(TimelineViolation {
                    kind: TimelineViolationKind::MissingBattleEnd,
                    message: "timeline does not end with BattleEnd".to_string(),
                    entry_index: Some(timeline.entries.len().saturating_sub(1)),
                });
            }
        }

        // Index-level invariants.
        for (index, entry) in timeline.entries.iter().enumerate() {
            self.validate_entry_index_invariants(timeline, index, entry, &mut violations);
        }

        if self.config.validate_outcome_cause_seq {
            validate_outcome_cause_relations(timeline, &mut violations);
        }

        // Spawn integrity and reference checks.
        let extracted = extract_spawns(timeline, &mut violations);
        validate_spawn_counts(extracted.counts, expected_counts, &mut violations);
        validate_reference_spawn_order(timeline, &extracted, &mut violations);
        validate_stateful_unit_invariants(timeline, &mut violations);
        if self.config.require_autocast_pairs {
            validate_autocast_pairs(timeline, &mut violations);
        }
        validate_attacks(timeline, &extracted, &mut violations, &self.config);
        validate_buffs(timeline, &mut violations);
        validate_deaths(timeline, &extracted, &mut violations, &self.config);
        validate_auto_attack_cadence(timeline, &extracted, &mut violations, &self.config);

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    fn validate_entry_index_invariants(
        &self,
        timeline: &Timeline,
        index: usize,
        entry: &TimelineEntry,
        violations: &mut Vec<TimelineViolation>,
    ) {
        if self.config.require_contiguous_seq && entry.seq != index as u64 {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::NonContiguousSeq,
                message: format!("timeline seq {} does not match index {}", entry.seq, index),
                entry_index: Some(index),
            });
        }

        if matches!(&entry.event, TimelineEvent::Attack { kind: None, .. }) {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::AttackKindMissing,
                message: "Attack event is missing kind (expected Auto/Triggered)".to_string(),
                entry_index: Some(index),
            });
        }

        if self.config.require_non_decreasing_time && index > 0 {
            let prev = &timeline.entries[index - 1];
            if entry.time_ms < prev.time_ms {
                violations.push(TimelineViolation {
                    kind: TimelineViolationKind::TimeWentBackwards,
                    message: format!(
                        "time_ms {} is less than previous time_ms {}",
                        entry.time_ms, prev.time_ms
                    ),
                    entry_index: Some(index),
                });
            }
        }

        if self.config.require_spawn_stats_valid {
            if let TimelineEvent::UnitSpawned { stats, .. } = &entry.event {
                if let Some(message) = validate_spawn_stats(stats) {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::SpawnStatsInvalid,
                        message,
                        entry_index: Some(index),
                    });
                }
            }
        }

        if self.config.require_hp_delta_consistent {
            if let TimelineEvent::HpChanged {
                delta,
                hp_before,
                hp_after,
                ..
            } = &entry.event
            {
                let computed = i64::from(*hp_after) - i64::from(*hp_before);
                if computed != i64::from(*delta) {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::HpDeltaMismatch,
                        message: format!(
                            "hp delta mismatch: delta={}, before={}, after={}, computed={}",
                            delta, hp_before, hp_after, computed
                        ),
                        entry_index: Some(index),
                    });
                }
            }
        }
    }
}

fn validate_outcome_cause_relations(timeline: &Timeline, violations: &mut Vec<TimelineViolation>) {
    let mut seq_to_index: HashMap<u64, usize> = HashMap::new();
    for (index, entry) in timeline.entries.iter().enumerate() {
        seq_to_index.entry(entry.seq).or_insert(index);
    }

    for (index, entry) in timeline.entries.iter().enumerate() {
        if !matches!(
            entry.event,
            TimelineEvent::HpChanged { .. }
                | TimelineEvent::StatChanged { .. }
                | TimelineEvent::UnitDied { .. }
        ) {
            continue;
        }

        let Some(cause_seq) = entry.cause_seq else {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::OutcomeMissingCauseSeq,
                message: format!("outcome event missing cause_seq: {:?}", entry.event),
                entry_index: Some(index),
            });
            continue;
        };

        let Some(&cause_index) = seq_to_index.get(&cause_seq) else {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::OutcomeCauseSeqOutOfRange,
                message: format!(
                    "outcome event cause_seq {} does not reference any entry: {:?}",
                    cause_seq, entry.event
                ),
                entry_index: Some(index),
            });
            continue;
        };

        if cause_index >= index {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::OutcomeCauseSeqInFuture,
                message: format!(
                    "outcome event cause_seq {} is not before entry index {}: {:?}",
                    cause_seq, index, entry.event
                ),
                entry_index: Some(index),
            });
            continue;
        }

        let cause_entry = &timeline.entries[cause_index];
        let valid_cause = matches!(
            cause_entry.event,
            TimelineEvent::Attack { .. }
                | TimelineEvent::AbilityCast { .. }
                | TimelineEvent::BuffTick { .. }
        );
        if !valid_cause {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::OutcomeCauseSeqInvalidType,
                message: format!(
                    "outcome event cause_seq {} points to non-cause event {:?}",
                    cause_seq, cause_entry.event
                ),
                entry_index: Some(index),
            });
        }
    }
}

fn validate_autocast_pairs(timeline: &Timeline, violations: &mut Vec<TimelineViolation>) {
    let mut pending_start_by_caster: HashMap<Uuid, (u64, u64, usize)> = HashMap::new();
    let mut starts_by_seq: HashMap<u64, (Uuid, u64)> = HashMap::new();

    for (index, entry) in timeline.entries.iter().enumerate() {
        match &entry.event {
            TimelineEvent::AutoCastStart {
                caster_instance_id, ..
            } => {
                if pending_start_by_caster
                    .insert(*caster_instance_id, (entry.seq, entry.time_ms, index))
                    .is_some()
                {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "AutoCastStart overlaps an existing pending cast (caster={})",
                            caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }
                starts_by_seq.insert(entry.seq, (*caster_instance_id, entry.time_ms));
            }
            TimelineEvent::AutoCastEnd { caster_instance_id } => {
                let Some(cause_seq) = entry.cause_seq else {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "AutoCastEnd is missing cause_seq (caster={})",
                            caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                let Some((start_caster, start_time_ms)) = starts_by_seq.get(&cause_seq).copied()
                else {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "AutoCastEnd cause_seq {} does not reference an AutoCastStart (caster={})",
                            cause_seq, caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if start_caster != *caster_instance_id {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "AutoCastEnd caster mismatch: expected {}, got {}",
                            start_caster, caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }

                let expected_end_time = start_time_ms.saturating_add(1);
                if entry.time_ms != expected_end_time {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "AutoCastEnd timing mismatch for {}: expected {}ms, got {}ms",
                            caster_instance_id, expected_end_time, entry.time_ms
                        ),
                        entry_index: Some(index),
                    });
                }

                pending_start_by_caster.remove(caster_instance_id);
            }
            _ => {}
        }
    }

    for (caster, (start_seq, start_time_ms, start_index)) in pending_start_by_caster {
        violations.push(TimelineViolation {
            kind: TimelineViolationKind::AutoCastPairInvalid,
            message: format!(
                "missing AutoCastEnd for {} (start_seq={} start_time_ms={})",
                caster, start_seq, start_time_ms
            ),
            entry_index: Some(start_index),
        });
    }
}

fn validate_spawn_stats(stats: &UnitStats) -> Option<String> {
    validate_unit_stats(stats, "spawned unit")
}

fn validate_unit_stats(stats: &UnitStats, context: &str) -> Option<String> {
    if stats.current_health > stats.max_health {
        return Some(format!(
            "{context} has current_health {} > max_health {}",
            stats.current_health,
            stats.max_health
        ));
    }
    if stats.attack_interval_ms == 0 {
        return Some(format!("{context} has attack_interval_ms == 0"));
    }
    None
}

struct ExtractedSpawns {
    counts: TimelineExpectedCounts,
    unit_spawn_index: HashMap<Uuid, usize>,
    unit_spawn_time_ms: HashMap<Uuid, u64>,
    unit_spawn_stats: HashMap<Uuid, UnitStats>,
    unit_owner_by_instance: HashMap<Uuid, Side>,
    unit_base_by_instance: HashMap<Uuid, Uuid>,
    item_spawn_index: HashMap<Uuid, usize>,
    item_owner_by_instance: HashMap<Uuid, Side>,
    artifact_spawn_index: HashMap<Uuid, usize>,
    artifact_owner_by_instance: HashMap<Uuid, Side>,
}

fn extract_spawns(timeline: &Timeline, violations: &mut Vec<TimelineViolation>) -> ExtractedSpawns {
    let mut extracted = ExtractedSpawns {
        counts: TimelineExpectedCounts {
            units: 0,
            items: 0,
            artifacts: 0,
        },
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

                // Referenced owner unit must exist and have consistent side.
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

fn validate_reference_spawn_order(
    timeline: &Timeline,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<TimelineViolation>,
) {
    for (index, entry) in timeline.entries.iter().enumerate() {
        let time_ms = entry.time_ms;
        let entry_desc = format!("{:?}", entry.event);
        match &entry.event {
            TimelineEvent::BattleStart { .. }
            | TimelineEvent::UnitSpawned { .. }
            | TimelineEvent::ArtifactSpawned { .. }
            | TimelineEvent::BattleEnd { .. } => {}
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
            TimelineEvent::Attack {
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
            TimelineEvent::AutoCastEnd { caster_instance_id } => {
                validate_unit_reference_spawned(
                    extracted,
                    *caster_instance_id,
                    index,
                    time_ms,
                    &entry_desc,
                    violations,
                );
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
        }
    }
}

fn validate_unit_reference_spawned(
    extracted: &ExtractedSpawns,
    unit_id: Uuid,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BuffInstanceKey {
    caster_instance_id: Uuid,
    target_instance_id: Uuid,
    buff_id: crate::game::battle::buffs::BuffId,
}

#[derive(Debug, Clone)]
struct ActiveBuff {
    stacks: u8,
    expires_at_ms: u64,
    next_tick_ms: Option<u64>,
}

fn validate_buffs(timeline: &Timeline, violations: &mut Vec<TimelineViolation>) {
    let mut active_buffs: HashMap<BuffInstanceKey, ActiveBuff> = HashMap::new();

    for (index, entry) in timeline.entries.iter().enumerate() {
        match entry.event {
            TimelineEvent::BuffApplied {
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
            } => {
                let Some(def) = buffs::get(buff_id) else {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::UnknownBuffId,
                        message: format!("unknown buff_id {} on BuffApplied", buff_id.as_u64()),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if duration_ms == 0 {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::BuffAppliedDurationZero,
                        message: "BuffApplied has duration_ms == 0".to_string(),
                        entry_index: Some(index),
                    });
                    continue;
                }

                let key = BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };
                let expires_at_ms = entry.time_ms.saturating_add(duration_ms);
                let max_stacks = def.max_stacks.max(1);

                let active = active_buffs.entry(key).or_insert(ActiveBuff {
                    stacks: 0,
                    expires_at_ms,
                    next_tick_ms: None,
                });
                active.expires_at_ms = active.expires_at_ms.max(expires_at_ms);
                active.stacks = active.stacks.saturating_add(1).min(max_stacks);

                if def.tick_interval_ms > 0 && active.next_tick_ms.is_none() {
                    let next_tick = entry.time_ms.saturating_add(def.tick_interval_ms);
                    if next_tick < active.expires_at_ms {
                        active.next_tick_ms = Some(next_tick);
                    }
                }
            }
            TimelineEvent::BuffTick {
                caster_instance_id,
                target_instance_id,
                buff_id,
            } => {
                let Some(def) = buffs::get(buff_id) else {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::UnknownBuffId,
                        message: format!("unknown buff_id {} on BuffTick", buff_id.as_u64()),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if def.tick_interval_ms == 0 {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::BuffTickInvalid,
                        message: "BuffTick recorded for buff with tick_interval_ms == 0"
                            .to_string(),
                        entry_index: Some(index),
                    });
                    continue;
                }

                let key = BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let Some(active) = active_buffs.get_mut(&key) else {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::BuffTickInvalid,
                        message: format!(
                            "BuffTick recorded without an active BuffApplied (buff_id={})",
                            buff_id.as_u64()
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if entry.time_ms >= active.expires_at_ms {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::BuffTickInvalid,
                        message: format!(
                            "BuffTick at {}ms is at/after expires_at_ms {}",
                            entry.time_ms, active.expires_at_ms
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                }

                if active.next_tick_ms != Some(entry.time_ms) {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::BuffTickInvalid,
                        message: format!(
                            "BuffTick at {}ms does not match expected next_tick_ms {:?}",
                            entry.time_ms, active.next_tick_ms
                        ),
                        entry_index: Some(index),
                    });
                }

                let next_tick = entry.time_ms.saturating_add(def.tick_interval_ms);
                if next_tick < active.expires_at_ms {
                    active.next_tick_ms = Some(next_tick);
                } else {
                    active.next_tick_ms = None;
                }
            }
            TimelineEvent::BuffExpired {
                caster_instance_id,
                target_instance_id,
                buff_id,
            } => {
                if buffs::get(buff_id).is_none() {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::UnknownBuffId,
                        message: format!("unknown buff_id {} on BuffExpired", buff_id.as_u64()),
                        entry_index: Some(index),
                    });
                    continue;
                }

                let key = BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };
                let Some(active) = active_buffs.get(&key) else {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::BuffExpiredInvalid,
                        message: format!(
                            "BuffExpired recorded without an active BuffApplied (buff_id={})",
                            buff_id.as_u64()
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if entry.time_ms < active.expires_at_ms {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::BuffExpiredInvalid,
                        message: format!(
                            "BuffExpired at {}ms is before expires_at_ms {}",
                            entry.time_ms, active.expires_at_ms
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                }

                active_buffs.remove(&key);
            }
            TimelineEvent::UnitDied {
                unit_instance_id, ..
            } => {
                active_buffs.retain(|key, _| key.target_instance_id != unit_instance_id);
            }
            _ => {}
        }
    }
}

fn validate_spawn_counts(
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
                "UnitSpawned count mismatch: expected={}, actual={}",
                expected.units, actual.units
            ),
            entry_index: None,
        });
    }
    if actual.items != expected.items {
        violations.push(TimelineViolation {
            kind: TimelineViolationKind::ItemSpawnCountMismatch,
            message: format!(
                "ItemSpawned count mismatch: expected={}, actual={}",
                expected.items, actual.items
            ),
            entry_index: None,
        });
    }
    if actual.artifacts != expected.artifacts {
        violations.push(TimelineViolation {
            kind: TimelineViolationKind::ArtifactSpawnCountMismatch,
            message: format!(
                "ArtifactSpawned count mismatch: expected={}, actual={}",
                expected.artifacts, actual.artifacts
            ),
            entry_index: None,
        });
    }
}

fn validate_stateful_unit_invariants(timeline: &Timeline, violations: &mut Vec<TimelineViolation>) {
    let mut stats_by_unit: HashMap<Uuid, UnitStats> = HashMap::new();

    for (index, entry) in timeline.entries.iter().enumerate() {
        match &entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                stats,
                ..
            } => {
                stats_by_unit.insert(*unit_instance_id, *stats);
            }
            TimelineEvent::HpChanged {
                target_instance_id,
                hp_before,
                hp_after,
                ..
            } => {
                let Some(stats) = stats_by_unit.get_mut(target_instance_id) else {
                    continue;
                };
                if stats.current_health != *hp_before {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::HpBeforeMismatch,
                        message: format!(
                            "HpChanged hp_before mismatch for {}: state={}, event={}",
                            target_instance_id, stats.current_health, hp_before
                        ),
                        entry_index: Some(index),
                    });
                }
                stats.current_health = *hp_after;
            }
            TimelineEvent::StatChanged {
                target_instance_id,
                stats_before,
                stats_after,
                ..
            } => {
                let Some(stats) = stats_by_unit.get_mut(target_instance_id) else {
                    continue;
                };
                if stats.current_health != stats_before.current_health
                    || stats.max_health != stats_before.max_health
                    || stats.attack != stats_before.attack
                    || stats.defense != stats_before.defense
                    || stats.attack_interval_ms != stats_before.attack_interval_ms
                {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::StatsBeforeMismatch,
                        message: format!(
                            "StatChanged stats_before mismatch for {}",
                            target_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }
                if let Some(message) = validate_unit_stats(stats_after, "StatChanged stats_after") {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::StatsAfterInvalid,
                        message,
                        entry_index: Some(index),
                    });
                }
                *stats = *stats_after;
            }
            TimelineEvent::BuffApplied {
                caster_instance_id, ..
            } => {
                let Some(stats) = stats_by_unit.get(caster_instance_id) else {
                    continue;
                };
                if stats.current_health == 0 {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::BuffAppliedByDeadCaster,
                        message: format!(
                            "BuffApplied caster {} has current_health=0",
                            caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }
            }
            TimelineEvent::UnitDied {
                unit_instance_id, ..
            } => {
                let Some(stats) = stats_by_unit.get(unit_instance_id) else {
                    continue;
                };
                if stats.current_health != 0 {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::UnitDiedWhileAlive,
                        message: format!(
                            "UnitDied recorded for {} with current_health={}",
                            unit_instance_id, stats.current_health
                        ),
                        entry_index: Some(index),
                    });
                }
            }
            _ => {}
        }
    }
}

fn validate_attacks(
    timeline: &Timeline,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<TimelineViolation>,
    config: &TimelineValidatorConfig,
) {
    let mut basic_attack_hp_changes: HashSet<BasicAttackHpChangeKey> = HashSet::new();
    if config.require_attack_has_basic_hp_change {
        for entry in &timeline.entries {
            if let TimelineEvent::HpChanged {
                source_instance_id: Some(source_id),
                target_instance_id,
                reason: HpChangeReason::BasicAttack,
                ..
            } = entry.event
            {
                if let Some(cause_seq) = entry.cause_seq {
                    basic_attack_hp_changes.insert(BasicAttackHpChangeKey {
                        cause_seq,
                        time_ms: entry.time_ms,
                        attacker_instance_id: source_id,
                        target_instance_id,
                    });
                }
            }
        }
    }

    for (index, entry) in timeline.entries.iter().enumerate() {
        let TimelineEvent::Attack {
            attacker_instance_id,
            target_instance_id,
            ..
        } = entry.event
        else {
            continue;
        };

        if attacker_instance_id == target_instance_id {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::AttackTargetsSameUnit,
                message: "attack targets the same unit".to_string(),
                entry_index: Some(index),
            });
            continue;
        }

        let Some(&attacker_owner) = extracted.unit_owner_by_instance.get(&attacker_instance_id)
        else {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::UnknownUnitReference,
                message: format!(
                    "attack references unknown attacker {}",
                    attacker_instance_id
                ),
                entry_index: Some(index),
            });
            continue;
        };

        let Some(&target_owner) = extracted.unit_owner_by_instance.get(&target_instance_id) else {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::UnknownUnitReference,
                message: format!("attack references unknown target {}", target_instance_id),
                entry_index: Some(index),
            });
            continue;
        };

        if attacker_owner == target_owner {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::AttackTargetsAlly,
                message: format!(
                    "attack targets ally: attacker_owner={:?} target_owner={:?}",
                    attacker_owner, target_owner
                ),
                entry_index: Some(index),
            });
        }

        if config.require_attack_has_basic_hp_change {
            let saw_basic_hp_change = basic_attack_hp_changes.contains(&BasicAttackHpChangeKey {
                cause_seq: entry.seq,
                time_ms: entry.time_ms,
                attacker_instance_id,
                target_instance_id,
            });

            if !saw_basic_hp_change {
                violations.push(TimelineViolation {
                    kind: TimelineViolationKind::AttackMissingBasicHpChanged,
                    message: "attack missing BasicAttack HpChanged at same time_ms".to_string(),
                    entry_index: Some(index),
                });
            }
        }
    }
}

fn validate_deaths(
    timeline: &Timeline,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<TimelineViolation>,
    config: &TimelineValidatorConfig,
) {
    let mut died_units: HashSet<Uuid> = HashSet::new();
    let mut death_index_by_unit: HashMap<Uuid, usize> = HashMap::new();

    for (index, entry) in timeline.entries.iter().enumerate() {
        let TimelineEvent::UnitDied {
            unit_instance_id, ..
        } = entry.event
        else {
            continue;
        };

        death_index_by_unit
            .entry(unit_instance_id)
            .and_modify(|existing| *existing = (*existing).min(index))
            .or_insert(index);

        if !died_units.insert(unit_instance_id) {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::UnitDiedDuplicate,
                message: format!("unit {} has multiple UnitDied entries", unit_instance_id),
                entry_index: Some(index),
            });
        }
    }

    // Validate post-death references.
    for (index, entry) in timeline.entries.iter().enumerate() {
        if death_index_by_unit.is_empty() {
            break;
        }
        for unit_id in referenced_unit_ids(&entry.event) {
            if let Some(&death_index) = death_index_by_unit.get(&unit_id) {
                if death_index < index && is_dead_unit_operated_on(&entry.event, unit_id, config) {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::DeadUnitActsAfterDeath,
                        message: format!(
                            "unit {} is referenced after death at time_ms {}",
                            unit_id, entry.time_ms
                        ),
                        entry_index: Some(index),
                    });
                }
            }
        }
    }

    // Sanity: all UnitDied should reference a known unit instance (spawned earlier).
    for (index, entry) in timeline.entries.iter().enumerate() {
        let TimelineEvent::UnitDied {
            unit_instance_id, ..
        } = entry.event
        else {
            continue;
        };
        if !extracted
            .unit_owner_by_instance
            .contains_key(&unit_instance_id)
        {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::UnknownUnitReference,
                message: format!("UnitDied references unknown unit {}", unit_instance_id),
                entry_index: Some(index),
            });
        }
    }
}

fn referenced_unit_ids(event: &TimelineEvent) -> Vec<Uuid> {
    match event {
        TimelineEvent::Attack {
            attacker_instance_id,
            target_instance_id,
            ..
        } => vec![*attacker_instance_id, *target_instance_id],
        TimelineEvent::AutoCastStart {
            caster_instance_id,
            target_instance_id,
            ..
        } => target_instance_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
        TimelineEvent::AutoCastEnd { caster_instance_id } => vec![*caster_instance_id],
        TimelineEvent::AbilityCast {
            caster_instance_id,
            target_instance_id,
            ..
        } => target_instance_id
            .map(|target| vec![*caster_instance_id, target])
            .unwrap_or_else(|| vec![*caster_instance_id]),
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
        } => vec![*caster_instance_id, *target_instance_id],
        TimelineEvent::HpChanged {
            source_instance_id,
            target_instance_id,
            ..
        } => source_instance_id
            .map(|source| vec![source, *target_instance_id])
            .unwrap_or_else(|| vec![*target_instance_id]),
        TimelineEvent::StatChanged {
            source_instance_id,
            target_instance_id,
            ..
        } => source_instance_id
            .map(|source| vec![source, *target_instance_id])
            .unwrap_or_else(|| vec![*target_instance_id]),
        TimelineEvent::UnitDied {
            unit_instance_id,
            killer_instance_id,
            ..
        } => killer_instance_id
            .map(|killer| vec![*unit_instance_id, killer])
            .unwrap_or_else(|| vec![*unit_instance_id]),
        TimelineEvent::BattleStart { .. }
        | TimelineEvent::ArtifactSpawned { .. }
        | TimelineEvent::BattleEnd { .. } => Vec::new(),
        TimelineEvent::UnitSpawned {
            unit_instance_id, ..
        } => vec![*unit_instance_id],
        TimelineEvent::ItemSpawned {
            owner_unit_instance_id,
            ..
        } => vec![*owner_unit_instance_id],
    }
}

fn validate_auto_attack_cadence(
    timeline: &Timeline,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<TimelineViolation>,
    config: &TimelineValidatorConfig,
) {
    if !config.validate_auto_attack_min_interval && !config.validate_auto_attack_presence {
        return;
    }

    let battle_end_time_ms = timeline
        .entries
        .last()
        .map(|e| e.time_ms)
        .unwrap_or_default();

    let mut unit_death_time_ms: HashMap<Uuid, u64> = HashMap::new();
    for entry in &timeline.entries {
        if let TimelineEvent::UnitDied {
            unit_instance_id, ..
        } = entry.event
        {
            unit_death_time_ms
                .entry(unit_instance_id)
                .or_insert(entry.time_ms);
        }
    }

    // Interval state is reconstructed from spawn stats and StatChanged events.
    let mut current_interval_ms: HashMap<Uuid, u64> = extracted
        .unit_spawn_stats
        .iter()
        .map(|(unit_id, stats)| (*unit_id, stats.attack_interval_ms.max(1)))
        .collect();

    // Expected next auto attack time (best-effort, assumes attacks are scheduled at
    // `last_auto_attack_time + interval`, potentially delayed by casting).
    let mut expected_next_auto_attack_time_ms: HashMap<Uuid, u64> = HashMap::new();
    for (unit_id, stats) in &extracted.unit_spawn_stats {
        let spawn_time = extracted
            .unit_spawn_time_ms
            .get(unit_id)
            .copied()
            .unwrap_or(0);
        expected_next_auto_attack_time_ms.insert(
            *unit_id,
            spawn_time.saturating_add(stats.attack_interval_ms.max(1)),
        );
    }

    let mut missing_reported: HashSet<Uuid> = HashSet::new();
    let mut casting_until_ms: HashMap<Uuid, u64> = HashMap::new();

    // Iterate grouped by time_ms so we can apply StatChanged in the same tick before
    // computing the next expected attack time.
    let mut index = 0usize;
    while index < timeline.entries.len() {
        let time_ms = timeline.entries[index].time_ms;

        // Before processing this tick, check if we have missed an expected auto attack.
        if config.validate_auto_attack_presence {
            let expected_entries: Vec<(Uuid, u64)> = expected_next_auto_attack_time_ms
                .iter()
                .map(|(unit_id, time)| (*unit_id, *time))
                .collect();
            for (unit_id, expected_time) in expected_entries {
                if missing_reported.contains(&unit_id) {
                    continue;
                }

                if expected_time > battle_end_time_ms {
                    continue;
                }

                let mut effective_expected_time = expected_time;
                if let Some(&cast_until) = casting_until_ms.get(&unit_id) {
                    if effective_expected_time < cast_until {
                        effective_expected_time = cast_until;
                        expected_next_auto_attack_time_ms.insert(unit_id, effective_expected_time);
                    }
                }

                if effective_expected_time.saturating_add(config.auto_attack_timing_tolerance_ms)
                    < time_ms
                    && should_expect_auto_attack(
                        extracted,
                        &unit_death_time_ms,
                        unit_id,
                        effective_expected_time,
                        battle_end_time_ms,
                    )
                {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::MissingExpectedAutoAttack,
                        message: format!(
                            "missing expected auto attack: unit={} expected_time_ms={} tolerance_ms={}",
                            unit_id, effective_expected_time, config.auto_attack_timing_tolerance_ms
                        ),
                        entry_index: None,
                    });
                    missing_reported.insert(unit_id);
                }
            }
        }

        let mut auto_attackers_this_tick: HashSet<Uuid> = HashSet::new();
        let mut tick_end = index;
        while tick_end < timeline.entries.len() && timeline.entries[tick_end].time_ms == time_ms {
            let entry = &timeline.entries[tick_end];
            match &entry.event {
                TimelineEvent::Attack {
                    attacker_instance_id,
                    kind: Some(AttackKind::Auto),
                    ..
                } => {
                    auto_attackers_this_tick.insert(*attacker_instance_id);
                }
                TimelineEvent::AutoCastStart {
                    caster_instance_id, ..
                } => {
                    casting_until_ms.insert(*caster_instance_id, time_ms.saturating_add(1));
                }
                TimelineEvent::AutoCastEnd { caster_instance_id } => {
                    casting_until_ms.remove(caster_instance_id);
                }
                TimelineEvent::StatChanged {
                    target_instance_id,
                    stats_after,
                    ..
                } => {
                    current_interval_ms
                        .insert(*target_instance_id, stats_after.attack_interval_ms.max(1));
                }
                _ => {}
            }
            tick_end += 1;
        }

        if config.validate_auto_attack_min_interval {
            for &attacker_id in &auto_attackers_this_tick {
                let Some(expected_next) =
                    expected_next_auto_attack_time_ms.get(&attacker_id).copied()
                else {
                    continue;
                };

                if time_ms < expected_next {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::AutoAttackTooEarly,
                        message: format!(
                            "auto attack too early: unit={} time_ms={} expected_min_time_ms={}",
                            attacker_id, time_ms, expected_next
                        ),
                        entry_index: None,
                    });
                }
            }
        }

        // After processing all entries at this time, compute next expected auto-attack time for attackers.
        for attacker_id in auto_attackers_this_tick {
            let interval = current_interval_ms.get(&attacker_id).copied().unwrap_or(1);
            expected_next_auto_attack_time_ms.insert(attacker_id, time_ms.saturating_add(interval));
        }

        index = tick_end;
    }
}

fn should_expect_auto_attack(
    extracted: &ExtractedSpawns,
    unit_death_time_ms: &HashMap<Uuid, u64>,
    unit_id: Uuid,
    at_time_ms: u64,
    battle_end_time_ms: u64,
) -> bool {
    if at_time_ms > battle_end_time_ms {
        return false;
    }

    let Some(&unit_side) = extracted.unit_owner_by_instance.get(&unit_id) else {
        return false;
    };

    let spawn_time = extracted
        .unit_spawn_time_ms
        .get(&unit_id)
        .copied()
        .unwrap_or(0);
    if at_time_ms < spawn_time {
        return false;
    }

    if let Some(&death_time) = unit_death_time_ms.get(&unit_id) {
        if death_time <= at_time_ms {
            return false;
        }
    }

    // Require at least one enemy to be alive at that time.
    extracted
        .unit_owner_by_instance
        .iter()
        .any(|(enemy_id, enemy_side)| {
            if *enemy_side == unit_side {
                return false;
            }
            let enemy_spawn = extracted
                .unit_spawn_time_ms
                .get(enemy_id)
                .copied()
                .unwrap_or(0);
            if enemy_spawn > at_time_ms {
                return false;
            }
            match unit_death_time_ms.get(enemy_id) {
                Some(&enemy_death) => enemy_death > at_time_ms,
                None => true,
            }
        })
}

fn is_dead_unit_operated_on(
    event: &TimelineEvent,
    dead_unit_id: Uuid,
    config: &TimelineValidatorConfig,
) -> bool {
    match event {
        TimelineEvent::Attack {
            attacker_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *attacker_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id)
        }
        TimelineEvent::AutoCastStart {
            caster_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && target_instance_id.is_some_and(|t| t == dead_unit_id))
        }
        TimelineEvent::AutoCastEnd {
            caster_instance_id, ..
        } => config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id,
        TimelineEvent::AbilityCast {
            caster_instance_id,
            target_instance_id,
            ..
        } => {
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets
                    && target_instance_id.is_some_and(|t| t == dead_unit_id))
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
            let _ = caster_instance_id;
            config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id
        }
        TimelineEvent::HpChanged {
            target_instance_id, ..
        } => config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id,
        TimelineEvent::StatChanged {
            target_instance_id, ..
        } => config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id,
        TimelineEvent::ItemSpawned {
            owner_unit_instance_id,
            ..
        } => *owner_unit_instance_id == dead_unit_id,
        TimelineEvent::UnitSpawned {
            unit_instance_id, ..
        } => *unit_instance_id == dead_unit_id,
        TimelineEvent::UnitDied {
            unit_instance_id, ..
        } => *unit_instance_id == dead_unit_id,
        TimelineEvent::BattleStart { .. }
        | TimelineEvent::ArtifactSpawned { .. }
        | TimelineEvent::BattleEnd { .. } => false,
    }
}
