use std::collections::HashMap;

use crate::game::{
    ability::SkillId,
    battle::event_log::{BattleEventCause, BattleEventLog, BattleLogEvent},
    battle::ids::UnitInstanceId,
};

use super::{
    focus::SkillFocusTimeProvider,
    types::{EventLogViolation, EventLogViolationKind},
};

pub(super) fn validate_autocast_pairs(
    event_log: &BattleEventLog,
    focus_time_provider: Option<&dyn SkillFocusTimeProvider>,
    violations: &mut Vec<EventLogViolation>,
) {
    let mut starts_by_seq: HashMap<u64, (UnitInstanceId, u64, Option<SkillId>, usize)> =
        HashMap::new();
    let mut pending_start_by_caster: HashMap<UnitInstanceId, (u64, u64, usize)> = HashMap::new();

    for (index, entry) in event_log.entries.iter().enumerate() {
        match &entry.event {
            BattleLogEvent::AutoCastStart {
                caster_instance_id,
                skill_id,
                ..
            } => {
                if pending_start_by_caster.contains_key(caster_instance_id) {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "caster {} has overlapping AutoCastStart (seq={})",
                            caster_instance_id, entry.seq
                        ),
                        entry_index: Some(index),
                    });
                }
                pending_start_by_caster
                    .insert(*caster_instance_id, (entry.seq, entry.time_ms, index));
                starts_by_seq.insert(
                    entry.seq,
                    (*caster_instance_id, entry.time_ms, skill_id.clone(), index),
                );
            }
            BattleLogEvent::AutoCastEnd { caster_instance_id } => {
                let BattleEventCause::Parent { seq: start_seq } = entry.cause else {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "AutoCastEnd is not parent-caused (caster={})",
                            caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                let Some((start_caster, start_time_ms, start_skill_id, _start_index)) =
                    starts_by_seq.get(&start_seq).cloned()
                else {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "AutoCastEnd parent seq {} does not reference an AutoCastStart (caster={})",
                            start_seq, caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if start_caster != *caster_instance_id {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "AutoCastEnd caster mismatch: expected {}, got {}",
                            start_caster, caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                }

                let focus_ms = start_skill_id
                    .as_ref()
                    .and_then(|id| focus_time_provider.and_then(|p| p.focus_time_ms(id)))
                    .map(|v| v as u64);

                if let Some(focus_ms) = focus_ms {
                    let expected_end_time = start_time_ms.saturating_add(focus_ms.max(1));
                    if entry.time_ms < expected_end_time {
                        violations.push(EventLogViolation {
                            kind: EventLogViolationKind::AutoCastPairInvalid,
                            message: format!(
                                "AutoCastEnd happens too early for {}: earliest {}ms, got {}ms (start_seq={})",
                                caster_instance_id, expected_end_time, entry.time_ms, start_seq
                            ),
                            entry_index: Some(index),
                        });
                    }
                } else if entry.time_ms < start_time_ms.saturating_add(1) {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "AutoCastEnd happens too early for {}: start_time_ms={} end_time_ms={} (start_seq={})",
                            caster_instance_id, start_time_ms, entry.time_ms, start_seq
                        ),
                        entry_index: Some(index),
                    });
                }

                pending_start_by_caster.remove(caster_instance_id);
            }
            BattleLogEvent::SkillCastInterrupted {
                caster_instance_id,
                interrupted_cast_seq,
                ..
            }
            | BattleLogEvent::SkillCastCancelled {
                caster_instance_id,
                interrupted_cast_seq,
                ..
            } => {
                let Some((start_caster, _start_time_ms, _start_skill_id, _start_index)) =
                    starts_by_seq.get(interrupted_cast_seq).cloned()
                else {
                    continue;
                };

                if start_caster != *caster_instance_id {
                    let event_name = match &entry.event {
                        BattleLogEvent::SkillCastInterrupted { .. } => "SkillCastInterrupted",
                        BattleLogEvent::SkillCastCancelled { .. } => "SkillCastCancelled",
                        _ => unreachable!(),
                    };
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::AutoCastPairInvalid,
                        message: format!(
                            "{} caster mismatch: expected {}, got {}",
                            event_name, start_caster, caster_instance_id
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                }

                pending_start_by_caster.remove(caster_instance_id);
            }
            _ => {}
        }
    }

    for (caster, (start_seq, start_time_ms, start_index)) in pending_start_by_caster {
        violations.push(EventLogViolation {
            kind: EventLogViolationKind::AutoCastPairInvalid,
            message: format!(
                "missing AutoCastEnd, SkillCastInterrupted, or SkillCastCancelled for {} (start_seq={} start_time_ms={})",
                caster, start_seq, start_time_ms
            ),
            entry_index: Some(start_index),
        });
    }
}
