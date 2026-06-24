use std::collections::HashMap;

use crate::game::battle::{
    buffs::{BuffDatabase, BuffId, BuffKind, BuffReapplyPolicy},
    event_log::{BattleEventLog, BattleLogEvent, BuffExpireReason},
    ids::UnitInstanceId,
};

use super::types::{EventLogViolation, EventLogViolationKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BuffInstanceKey {
    caster_instance_id: UnitInstanceId,
    target_instance_id: UnitInstanceId,
    buff_id: BuffId,
}

#[derive(Debug, Clone)]
struct ActiveBuff {
    stacks: u8,
    expires_at_ms: u64,
    next_tick_ms: Option<u64>,
}

fn is_exclusive_hard_cc(kind: BuffKind) -> bool {
    matches!(kind, BuffKind::Stun | BuffKind::Freeze)
}

pub(super) fn validate_buffs(
    event_log: &BattleEventLog,
    buff_data: &BuffDatabase,
    violations: &mut Vec<EventLogViolation>,
) {
    let mut active_buffs: HashMap<BuffInstanceKey, ActiveBuff> = HashMap::new();

    for (index, entry) in event_log.entries.iter().enumerate() {
        match entry.event {
            BattleLogEvent::BuffApplied {
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
            } => {
                let Some(def) = buff_data.get(buff_id) else {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::UnknownBuffId,
                        message: format!("unknown buff_id {} on BuffApplied", buff_id.as_u64()),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if duration_ms == 0 {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffAppliedDurationZero,
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

                if is_exclusive_hard_cc(def.kind)
                    && active_buffs.keys().any(|active_key| {
                        active_key.target_instance_id == target_instance_id
                            && *active_key != key
                            && buff_data
                                .get(active_key.buff_id)
                                .is_some_and(|active_def| is_exclusive_hard_cc(active_def.kind))
                    })
                {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffExpiredInvalid,
                        message: "hard CC BuffApplied replaced an active hard CC without an explicit BuffExpired(reason=Replaced)".to_string(),
                        entry_index: Some(index),
                    });
                    continue;
                }

                let active = active_buffs.entry(key).or_insert(ActiveBuff {
                    stacks: 0,
                    expires_at_ms,
                    next_tick_ms: None,
                });
                match def.reapply_policy {
                    BuffReapplyPolicy::StackRefreshDurationKeepCadence => {
                        active.expires_at_ms = active.expires_at_ms.max(expires_at_ms);
                        active.stacks = active.stacks.saturating_add(1).min(max_stacks);
                    }
                    BuffReapplyPolicy::RefreshDuration => {
                        active.expires_at_ms = expires_at_ms;
                        active.stacks = max_stacks.min(1);
                    }
                }

                if def.tick_interval_ms > 0 && active.next_tick_ms.is_none() {
                    let next_tick = entry.time_ms.saturating_add(def.tick_interval_ms);
                    if next_tick < active.expires_at_ms {
                        active.next_tick_ms = Some(next_tick);
                    }
                }
            }
            BattleLogEvent::BuffTick {
                caster_instance_id,
                target_instance_id,
                buff_id,
            } => {
                let Some(def) = buff_data.get(buff_id) else {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::UnknownBuffId,
                        message: format!("unknown buff_id {} on BuffTick", buff_id.as_u64()),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if def.tick_interval_ms == 0 {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffTickInvalid,
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
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffTickInvalid,
                        message: format!(
                            "BuffTick recorded without an active BuffApplied (buff_id={})",
                            buff_id.as_u64()
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if entry.time_ms >= active.expires_at_ms {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffTickInvalid,
                        message: format!(
                            "BuffTick at {}ms is at/after expires_at_ms {}",
                            entry.time_ms, active.expires_at_ms
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                }

                if active.next_tick_ms != Some(entry.time_ms) {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffTickInvalid,
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
            BattleLogEvent::BuffExpired {
                caster_instance_id,
                target_instance_id,
                buff_id,
                reason,
            } => {
                let Some(_def) = buff_data.get(buff_id) else {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::UnknownBuffId,
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
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffExpiredInvalid,
                        message: format!(
                            "BuffExpired recorded without an active BuffApplied (buff_id={})",
                            buff_id.as_u64()
                        ),
                        entry_index: Some(index),
                    });
                    continue;
                };

                if reason == BuffExpireReason::Natural && entry.time_ms != active.expires_at_ms {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffExpiredInvalid,
                        message: format!(
                            "BuffExpired at {}ms does not match expected expires_at_ms {}",
                            entry.time_ms, active.expires_at_ms
                        ),
                        entry_index: Some(index),
                    });
                } else if reason != BuffExpireReason::Natural
                    && entry.time_ms > active.expires_at_ms
                {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::BuffExpiredInvalid,
                        message: format!(
                            "BuffExpired(reason={:?}) at {}ms is after expires_at_ms {}",
                            reason, entry.time_ms, active.expires_at_ms
                        ),
                        entry_index: Some(index),
                    });
                }
            }
            _ => {}
        }
    }
}
