use std::collections::HashMap;

use crate::game::battle::{
    buffs,
    ids::UnitInstanceId,
    timeline::{Timeline, TimelineEvent},
};

use super::types::{TimelineViolation, TimelineViolationKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BuffInstanceKey {
    caster_instance_id: UnitInstanceId,
    target_instance_id: UnitInstanceId,
    buff_id: buffs::BuffId,
}

#[derive(Debug, Clone)]
struct ActiveBuff {
    stacks: u8,
    expires_at_ms: u64,
    next_tick_ms: Option<u64>,
}

pub(super) fn validate_buffs(timeline: &Timeline, violations: &mut Vec<TimelineViolation>) {
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
                    continue;
                }

                let next_tick = entry.time_ms.saturating_add(def.tick_interval_ms);
                active.next_tick_ms = if next_tick < active.expires_at_ms {
                    Some(next_tick)
                } else {
                    None
                };
            }
            TimelineEvent::BuffExpired {
                caster_instance_id,
                target_instance_id,
                buff_id,
            } => {
                let Some(_def) = buffs::get(buff_id) else {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::UnknownBuffId,
                        message: format!("unknown buff_id {} on BuffExpired", buff_id.as_u64()),
                        entry_index: Some(index),
                    });
                    continue;
                };

                let key = BuffInstanceKey {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                };

                let Some(active) = active_buffs.remove(&key) else {
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

                if entry.time_ms != active.expires_at_ms {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::BuffExpiredInvalid,
                        message: format!(
                            "BuffExpired at {}ms does not match expected expires_at_ms {}",
                            entry.time_ms, active.expires_at_ms
                        ),
                        entry_index: Some(index),
                    });
                }
            }
            _ => {}
        }
    }
}
