use std::collections::HashMap;

use crate::{
    ecs::resources::Position,
    game::{
        battle::ids::UnitInstanceId,
        battle::timeline::{Timeline, TimelineEvent},
        stats::UnitStats,
    },
};

use super::{
    types::{TimelineValidatorConfig, TimelineViolation, TimelineViolationKind},
    unit_stats::validate_unit_stats,
};

pub(super) fn validate_stateful_unit_invariants(
    timeline: &Timeline,
    violations: &mut Vec<TimelineViolation>,
    config: &TimelineValidatorConfig,
) {
    let mut stats_by_unit: HashMap<UnitInstanceId, UnitStats> = HashMap::new();
    let mut position_by_unit: HashMap<UnitInstanceId, Position> = HashMap::new();

    for (index, entry) in timeline.entries.iter().enumerate() {
        match &entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                stats,
                position,
                ..
            } => {
                stats_by_unit.insert(*unit_instance_id, *stats);
                position_by_unit.insert(*unit_instance_id, *position);
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
                        message: format!("HpChanged hp_before mismatch for {}", target_instance_id),
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
                    || stats.magic_resist != stats_before.magic_resist
                    || stats.attack_interval_ms != stats_before.attack_interval_ms
                    || stats.move_speed_units_per_ms != stats_before.move_speed_units_per_ms
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
            TimelineEvent::UnitMoved {
                unit_instance_id,
                from,
                to,
            } => {
                if from == to {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::UnitMovedInvalid,
                        message: format!("UnitMoved has from == to for {}", unit_instance_id),
                        entry_index: Some(index),
                    });
                }
                if config.require_movement_path_consistent {
                    if let Some(current) = position_by_unit.get(unit_instance_id).copied() {
                        if current != *from {
                            violations.push(TimelineViolation {
                                kind: TimelineViolationKind::MovementPositionMismatch,
                                message: format!(
                                    "UnitMoved.from mismatch for {} (expected {:?}, got {:?})",
                                    unit_instance_id, current, from
                                ),
                                entry_index: Some(index),
                            });
                        }
                    }
                }
                position_by_unit.insert(*unit_instance_id, *to);
            }
            TimelineEvent::MovementStopped {
                unit_instance_id,
                position,
                ..
            } => {
                if config.require_movement_path_consistent {
                    if let Some(current) = position_by_unit.get(unit_instance_id).copied() {
                        if current != *position {
                            violations.push(TimelineViolation {
                                kind: TimelineViolationKind::MovementPositionMismatch,
                                message: format!(
                                    "MovementStopped.position mismatch for {} (expected {:?}, got {:?})",
                                    unit_instance_id, current, position
                                ),
                                entry_index: Some(index),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
