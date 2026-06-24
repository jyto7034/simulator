use crate::game::battle::{event_log::BattleEventLog, event_log::BattleLogEvent};

use super::{
    spawns::ExtractedSpawns,
    types::{EventLogViolation, EventLogViolationKind},
};

pub(super) fn validate_attacks(
    event_log: &BattleEventLog,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<EventLogViolation>,
) {
    for (index, entry) in event_log.entries.iter().enumerate() {
        let (attacker_instance_id, target_instance_id) = match &entry.event {
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
            } => (*attacker_instance_id, *target_instance_id),
            _ => continue,
        };

        if attacker_instance_id == target_instance_id {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::AttackTargetsSameUnit,
                message: "attack targets the same unit".to_string(),
                entry_index: Some(index),
            });
            continue;
        }

        let Some(&attacker_owner) = extracted.unit_owner_by_instance.get(&attacker_instance_id)
        else {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::UnknownUnitReference,
                message: format!(
                    "attack references unknown attacker {}",
                    attacker_instance_id
                ),
                entry_index: Some(index),
            });
            continue;
        };

        let Some(&target_owner) = extracted.unit_owner_by_instance.get(&target_instance_id) else {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::UnknownUnitReference,
                message: format!("attack references unknown target {}", target_instance_id),
                entry_index: Some(index),
            });
            continue;
        };

        if attacker_owner == target_owner {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::AttackTargetsAlly,
                message: format!(
                    "attack targets ally: attacker_owner={:?} target_owner={:?}",
                    attacker_owner, target_owner
                ),
                entry_index: Some(index),
            });
        }
    }
}
