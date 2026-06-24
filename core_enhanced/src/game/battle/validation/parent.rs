use std::collections::HashMap;

use crate::game::battle::{
    event_log::{BattleEventCause, BattleEventLog, BattleLogEvent},
    ids::UnitInstanceId,
};

use super::types::{EventLogViolation, EventLogViolationKind};

pub(super) fn validate_parent_seq_relations(
    event_log: &BattleEventLog,
    violations: &mut Vec<EventLogViolation>,
) {
    let mut seq_to_index: HashMap<u64, usize> = HashMap::new();
    for (index, entry) in event_log.entries.iter().enumerate() {
        seq_to_index.entry(entry.seq).or_insert(index);
    }

    for (index, entry) in event_log.entries.iter().enumerate() {
        let BattleEventCause::Parent { seq } = entry.cause else {
            continue;
        };

        let Some(&cause_index) = seq_to_index.get(&seq) else {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::ParentSeqOutOfRange,
                message: format!("parent seq {} does not reference any entry", seq),
                entry_index: Some(index),
            });
            continue;
        };

        if cause_index >= index {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::ParentSeqInFuture,
                message: format!(
                    "parent seq {} resolves to index {} which is not before current index {}",
                    seq, cause_index, index
                ),
                entry_index: Some(index),
            });
        }
    }
}

pub(super) fn validate_outcome_parent_relations(
    event_log: &BattleEventLog,
    violations: &mut Vec<EventLogViolation>,
) {
    for (index, entry) in event_log.entries.iter().enumerate() {
        if !matches!(
            entry.event,
            BattleLogEvent::HpChanged { .. }
                | BattleLogEvent::StatChanged { .. }
                | BattleLogEvent::ResonanceChanged { .. }
                | BattleLogEvent::UnitDied { .. }
        ) {
            continue;
        }

        if !matches!(entry.cause, BattleEventCause::Parent { .. }) {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::OutcomeMissingParent,
                message: format!("outcome event is not parent-caused: {:?}", entry.event),
                entry_index: Some(index),
            });
        }
    }
}

pub(super) fn validate_skill_cast_interrupt_relations(
    event_log: &BattleEventLog,
    violations: &mut Vec<EventLogViolation>,
) {
    let mut cast_start_by_seq: HashMap<u64, UnitInstanceId> = HashMap::new();
    for entry in &event_log.entries {
        match &entry.event {
            BattleLogEvent::AutoCastStart {
                caster_instance_id, ..
            }
            | BattleLogEvent::ManualCastStart {
                caster_instance_id, ..
            }
            | BattleLogEvent::AbilityCast {
                caster_instance_id, ..
            } => {
                cast_start_by_seq.insert(entry.seq, *caster_instance_id);
            }
            _ => {}
        }
    }

    for (index, entry) in event_log.entries.iter().enumerate() {
        let Some((caster_instance_id, interrupted_cast_seq, event_name)) = (match &entry.event {
            BattleLogEvent::SkillCastInterrupted {
                caster_instance_id,
                interrupted_cast_seq,
                ..
            } => Some((
                caster_instance_id,
                interrupted_cast_seq,
                "SkillCastInterrupted",
            )),
            BattleLogEvent::SkillCastCancelled {
                caster_instance_id,
                interrupted_cast_seq,
                ..
            } => Some((
                caster_instance_id,
                interrupted_cast_seq,
                "SkillCastCancelled",
            )),
            _ => None,
        }) else {
            continue;
        };

        let Some(start_caster_id) = cast_start_by_seq.get(interrupted_cast_seq) else {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::AutoCastPairInvalid,
                message: format!(
                    "{} interrupted_cast_seq {} does not reference a cast start",
                    event_name, interrupted_cast_seq
                ),
                entry_index: Some(index),
            });
            continue;
        };

        if start_caster_id != caster_instance_id {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::AutoCastPairInvalid,
                message: format!(
                    "{} caster mismatch: start seq {} belongs to {}, got {}",
                    event_name, interrupted_cast_seq, start_caster_id, caster_instance_id
                ),
                entry_index: Some(index),
            });
        }
    }
}
