use std::collections::HashMap;

use crate::game::battle::timeline::{Timeline, TimelineCause, TimelineEvent};

use super::types::{TimelineViolation, TimelineViolationKind};

pub(super) fn validate_parent_seq_relations(
    timeline: &Timeline,
    violations: &mut Vec<TimelineViolation>,
) {
    let mut seq_to_index: HashMap<u64, usize> = HashMap::new();
    for (index, entry) in timeline.entries.iter().enumerate() {
        seq_to_index.entry(entry.seq).or_insert(index);
    }

    for (index, entry) in timeline.entries.iter().enumerate() {
        let TimelineCause::Parent { seq } = entry.cause else {
            continue;
        };

        let Some(&cause_index) = seq_to_index.get(&seq) else {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::ParentSeqOutOfRange,
                message: format!("parent seq {} does not reference any entry", seq),
                entry_index: Some(index),
            });
            continue;
        };

        if cause_index >= index {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::ParentSeqInFuture,
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
    timeline: &Timeline,
    violations: &mut Vec<TimelineViolation>,
) {
    for (index, entry) in timeline.entries.iter().enumerate() {
        if !matches!(
            entry.event,
            TimelineEvent::HpChanged { .. }
                | TimelineEvent::StatChanged { .. }
                | TimelineEvent::ResonanceChanged { .. }
                | TimelineEvent::UnitDied { .. }
        ) {
            continue;
        }

        if !matches!(entry.cause, TimelineCause::Parent { .. }) {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::OutcomeMissingParent,
                message: format!("outcome event is not parent-caused: {:?}", entry.event),
                entry_index: Some(index),
            });
        }
    }
}
