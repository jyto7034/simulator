use std::collections::{HashMap, HashSet};

use crate::game::battle::{
    ids::UnitInstanceId,
    timeline::{AttackKind, Timeline, TimelineEvent},
};

use super::{
    spawns::ExtractedSpawns,
    types::{TimelineValidatorConfig, TimelineViolation, TimelineViolationKind},
};

pub(super) fn validate_auto_attack_cadence(
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

    let mut unit_death_time_ms: HashMap<UnitInstanceId, u64> = HashMap::new();
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

    let mut current_interval_ms: HashMap<UnitInstanceId, u64> = extracted
        .unit_spawn_stats
        .iter()
        .map(|(unit_id, stats)| (*unit_id, stats.attack_interval_ms.max(1)))
        .collect();

    let mut expected_next_auto_attack_time_ms: HashMap<UnitInstanceId, u64> = HashMap::new();
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

    let mut missing_reported: HashSet<UnitInstanceId> = HashSet::new();
    let mut casting_until_ms: HashMap<UnitInstanceId, u64> = HashMap::new();

    let mut index = 0usize;
    while index < timeline.entries.len() {
        let time_ms = timeline.entries[index].time_ms;

        if config.validate_auto_attack_presence {
            let expected_entries: Vec<(UnitInstanceId, u64)> = expected_next_auto_attack_time_ms
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
                    if cast_until > effective_expected_time {
                        effective_expected_time = cast_until;
                    }
                }

                if !should_expect_auto_attack(
                    extracted,
                    &unit_death_time_ms,
                    unit_id,
                    effective_expected_time,
                    battle_end_time_ms,
                ) {
                    continue;
                }

                let tolerance = config.auto_attack_timing_tolerance_ms;
                if time_ms > effective_expected_time.saturating_add(tolerance) {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::MissingExpectedAutoAttack,
                        message: format!(
                            "missing expected auto attack: unit={} expected_time_ms={} tolerance_ms={}",
                            unit_id, effective_expected_time, tolerance
                        ),
                        entry_index: None,
                    });
                    missing_reported.insert(unit_id);
                }
            }
        }

        let mut auto_attackers_this_tick: HashSet<UnitInstanceId> = HashSet::new();
        let mut tick_end = index;
        while tick_end < timeline.entries.len() && timeline.entries[tick_end].time_ms == time_ms {
            let entry = &timeline.entries[tick_end];
            match &entry.event {
                TimelineEvent::AttackStart {
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

        for attacker_id in auto_attackers_this_tick {
            let interval = current_interval_ms.get(&attacker_id).copied().unwrap_or(1);
            expected_next_auto_attack_time_ms.insert(attacker_id, time_ms.saturating_add(interval));
        }

        index = tick_end;
    }
}

fn should_expect_auto_attack(
    extracted: &ExtractedSpawns,
    unit_death_time_ms: &HashMap<UnitInstanceId, u64>,
    unit_id: UnitInstanceId,
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
