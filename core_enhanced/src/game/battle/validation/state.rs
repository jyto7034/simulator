use std::collections::HashMap;

use crate::game::{
    battle::event_log::{BattleEventLog, BattleLogEvent},
    battle::ids::UnitInstanceId,
    stats::UnitStats,
};

use super::{
    types::{EventLogValidatorConfig, EventLogViolation, EventLogViolationKind},
    unit_stats::validate_unit_stats,
};

pub(super) fn validate_stateful_unit_invariants(
    event_log: &BattleEventLog,
    violations: &mut Vec<EventLogViolation>,
    _config: &EventLogValidatorConfig,
) {
    let mut stats_by_unit: HashMap<UnitInstanceId, UnitStats> = HashMap::new();

    for (index, entry) in event_log.entries.iter().enumerate() {
        match &entry.event {
            BattleLogEvent::UnitSpawned {
                unit_instance_id,
                stats,
                ..
            } => {
                stats_by_unit.insert(*unit_instance_id, *stats);
            }
            BattleLogEvent::HpChanged {
                target_instance_id,
                hp_before,
                hp_after,
                ..
            } => {
                let Some(stats) = stats_by_unit.get_mut(target_instance_id) else {
                    continue;
                };
                if stats.current_health != *hp_before {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::HpBeforeMismatch,
                        message: format!("HpChanged hp_before mismatch for {}", target_instance_id),
                        entry_index: Some(index),
                    });
                }
                stats.current_health = *hp_after;
            }
            BattleLogEvent::StatChanged {
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
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::StatsBeforeMismatch,
                        message: format!(
                            "StatChanged stats_before mismatch for {}",
                            target_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }
                if let Some(message) = validate_unit_stats(stats_after, "StatChanged stats_after") {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::StatsAfterInvalid,
                        message,
                        entry_index: Some(index),
                    });
                }
                *stats = *stats_after;
            }
            BattleLogEvent::BuffApplied {
                caster_instance_id, ..
            } => {
                let Some(stats) = stats_by_unit.get(caster_instance_id) else {
                    continue;
                };
                if stats.current_health == 0 {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffAppliedByDeadCaster,
                        message: format!(
                            "BuffApplied caster {} has current_health=0",
                            caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }
            }
            BattleLogEvent::UnitDied {
                unit_instance_id, ..
            } => {
                let Some(stats) = stats_by_unit.get(unit_instance_id) else {
                    continue;
                };
                if stats.current_health != 0 {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::UnitDiedWhileAlive,
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
