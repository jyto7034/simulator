use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::game::{
    battle::{
        timeline::{AttackKind, HpChangeReason, Timeline, TimelineEntry, TimelineEvent},
        PlayerDeckInfo,
    },
    enums::Side,
    stats::UnitStats,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineViolationKind {
    MissingEntries,
    MissingBattleStart,
    MissingBattleEnd,
    NonContiguousSeq,
    TimeWentBackwards,
    SpawnStatsInvalid,
    HpDeltaMismatch,
    UnitSpawnCountMismatch,
    ItemSpawnCountMismatch,
    ArtifactSpawnCountMismatch,
    DuplicateUnitSpawn,
    DuplicateItemSpawn,
    DuplicateArtifactSpawn,
    UnknownUnitReference,
    AttackTargetsSameUnit,
    AttackTargetsAlly,
    AttackMissingBasicHpChanged,
    UnitDiedDuplicate,
    DeadUnitActsAfterDeath,
    AutoAttackTooEarly,
    MissingExpectedAutoAttack,
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

        // Spawn integrity and reference checks.
        let extracted = extract_spawns(timeline, &mut violations);
        validate_spawn_counts(extracted.counts, expected_counts, &mut violations);
        validate_attacks(timeline, &extracted, &mut violations, &self.config);
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
        if self.config.require_contiguous_seq && entry.seq as usize != index {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::NonContiguousSeq,
                message: format!("timeline seq {} does not match index {}", entry.seq, index),
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
                let computed = *hp_after as i32 - *hp_before as i32;
                if computed != *delta {
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

fn validate_spawn_stats(stats: &UnitStats) -> Option<String> {
    if stats.current_health > stats.max_health {
        return Some(format!(
            "spawned unit has current_health {} > max_health {}",
            stats.current_health, stats.max_health
        ));
    }
    if stats.attack_interval_ms == 0 {
        return Some("spawned unit has attack_interval_ms == 0".to_string());
    }
    None
}

struct ExtractedSpawns {
    counts: TimelineExpectedCounts,
    unit_spawn_time_ms: HashMap<Uuid, u64>,
    unit_spawn_stats: HashMap<Uuid, UnitStats>,
    unit_owner_by_instance: HashMap<Uuid, Side>,
    unit_base_by_instance: HashMap<Uuid, Uuid>,
    item_owner_by_instance: HashMap<Uuid, Side>,
    artifact_owner_by_instance: HashMap<Uuid, Side>,
}

fn extract_spawns(timeline: &Timeline, violations: &mut Vec<TimelineViolation>) -> ExtractedSpawns {
    let mut extracted = ExtractedSpawns {
        counts: TimelineExpectedCounts {
            units: 0,
            items: 0,
            artifacts: 0,
        },
        unit_spawn_time_ms: HashMap::new(),
        unit_spawn_stats: HashMap::new(),
        unit_owner_by_instance: HashMap::new(),
        unit_base_by_instance: HashMap::new(),
        item_owner_by_instance: HashMap::new(),
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
                    .unit_spawn_time_ms
                    .insert(unit_instance_id, entry.time_ms);
                extracted.unit_spawn_stats.insert(unit_instance_id, stats);
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

fn validate_attacks(
    timeline: &Timeline,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<TimelineViolation>,
    config: &TimelineValidatorConfig,
) {
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
            let saw_basic_hp_change =
                timeline
                    .entries
                    .iter()
                    .any(|hp_entry| match hp_entry.event {
                        TimelineEvent::HpChanged {
                            source_instance_id: Some(source_id),
                            target_instance_id: target_id,
                            reason: HpChangeReason::BasicAttack,
                            ..
                        } => {
                            hp_entry.time_ms == entry.time_ms
                                && source_id == attacker_instance_id
                                && target_id == target_instance_id
                        }
                        _ => false,
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

    for (index, entry) in timeline.entries.iter().enumerate() {
        let TimelineEvent::UnitDied {
            unit_instance_id, ..
        } = entry.event
        else {
            continue;
        };

        if !died_units.insert(unit_instance_id) {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::UnitDiedDuplicate,
                message: format!("unit {} has multiple UnitDied entries", unit_instance_id),
                entry_index: Some(index),
            });
        }

        // Validate post-death references.
        for later in &timeline.entries[index + 1..] {
            if is_dead_unit_operated_on(&later.event, unit_instance_id, config) {
                violations.push(TimelineViolation {
                    kind: TimelineViolationKind::DeadUnitActsAfterDeath,
                    message: format!(
                        "unit {} is referenced after death at time_ms {}",
                        unit_instance_id, later.time_ms
                    ),
                    entry_index: None,
                });
                break;
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

    // Iterate grouped by time_ms so we can apply StatChanged in the same tick before
    // computing the next expected attack time.
    let mut index = 0usize;
    while index < timeline.entries.len() {
        let time_ms = timeline.entries[index].time_ms;

        // Before processing this tick, check if we have missed an expected auto attack.
        if config.validate_auto_attack_presence {
            for (&unit_id, &expected_time) in expected_next_auto_attack_time_ms.iter() {
                if missing_reported.contains(&unit_id) {
                    continue;
                }

                if expected_time > battle_end_time_ms {
                    continue;
                }

                if expected_time.saturating_add(config.auto_attack_timing_tolerance_ms) < time_ms
                    && should_expect_auto_attack(
                        extracted,
                        &unit_death_time_ms,
                        unit_id,
                        expected_time,
                        battle_end_time_ms,
                    )
                {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::MissingExpectedAutoAttack,
                        message: format!(
                            "missing expected auto attack: unit={} expected_time_ms={} tolerance_ms={}",
                            unit_id, expected_time, config.auto_attack_timing_tolerance_ms
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
            (config.forbid_dead_units_as_attackers && *caster_instance_id == dead_unit_id)
                || (config.forbid_dead_units_as_targets && *target_instance_id == dead_unit_id)
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
