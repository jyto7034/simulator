use crate::game::battle::{timeline::Timeline, timeline::TimelineEvent};

use super::{
    spawns::ExtractedSpawns,
    types::{TimelineViolation, TimelineViolationKind},
};

pub(super) fn validate_attacks(
    timeline: &Timeline,
    extracted: &ExtractedSpawns,
    violations: &mut Vec<TimelineViolation>,
) {
    for (index, entry) in timeline.entries.iter().enumerate() {
        let (attacker_instance_id, target_instance_id) = match &entry.event {
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
            } => (*attacker_instance_id, *target_instance_id),
            _ => continue,
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
    }
}
