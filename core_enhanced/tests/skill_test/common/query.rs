use game_core::{
    game::resources::Position,
    game::{
        battle::{
            buffs::BuffId,
            core::movement::types::WorldVec2,
            event_log::{AttackKind, BattleEventLog, BattleEventLogEntry, BattleLogEvent},
            ids::UnitInstanceId,
        },
        enums::Side,
        stats::{StatId, StatModifierKind},
    },
};

pub fn find_unit_spawn_at(
    event_log: &BattleEventLog,
    position: Position,
    owner: Option<Side>,
) -> Option<&BattleEventLogEntry> {
    event_log.entries.iter().find(|entry| {
        matches!(
            &entry.event,
            BattleLogEvent::UnitSpawned {
                world_position,
                owner: entry_owner,
                ..
            } if spawn_matches_tile_center(*world_position, position)
                && owner.is_none_or(|expected| expected == *entry_owner)
        )
    })
}

fn spawn_matches_tile_center(
    world_position: game_core::game::battle::core::movement::types::EventLogVec2,
    position: Position,
) -> bool {
    world_position
        .to_world()
        .distance(WorldVec2::from_tile_center(position))
        <= 0.001
}

pub fn find_unit_instance_at(
    event_log: &BattleEventLog,
    position: Position,
    owner: Option<Side>,
) -> Option<UnitInstanceId> {
    find_unit_spawn_at(event_log, position, owner).and_then(|entry| match entry.event {
        BattleLogEvent::UnitSpawned {
            unit_instance_id, ..
        } => Some(unit_instance_id),
        _ => None,
    })
}

pub fn find_first_ability_cast<'a>(
    event_log: &'a BattleEventLog,
    caster_instance_id: UnitInstanceId,
    skill_id: &str,
) -> Option<&'a BattleEventLogEntry> {
    event_log.entries.iter().find(|entry| {
        matches!(
            &entry.event,
            BattleLogEvent::AbilityCast {
                skill_id: actual_skill_id,
                caster_instance_id: actual_caster_id,
                ..
            } if actual_skill_id == skill_id && *actual_caster_id == caster_instance_id
        )
    })
}

pub fn descendants_of(event_log: &BattleEventLog, parent_seq: u64) -> Vec<&BattleEventLogEntry> {
    event_log
        .entries
        .iter()
        .filter(|entry| entry_caused_by_seq(event_log, entry, parent_seq))
        .collect()
}

pub fn entry_caused_by_seq(
    event_log: &BattleEventLog,
    entry: &BattleEventLogEntry,
    expected_seq: u64,
) -> bool {
    let Some(parent_seq) = entry.cause.parent_seq() else {
        return false;
    };
    if parent_seq == expected_seq {
        return true;
    }

    event_log
        .entries
        .iter()
        .find(|entry| entry.seq == parent_seq)
        .is_some_and(|parent| {
            matches!(parent.event, BattleLogEvent::SkillAreaDeclared { .. })
                && parent.cause.parent_seq() == Some(expected_seq)
        })
}

pub fn step_entries_for_cast(
    event_log: &BattleEventLog,
    cast_seq: u64,
) -> Vec<&BattleEventLogEntry> {
    event_log
        .entries
        .iter()
        .filter(|entry| {
            matches!(entry.event, BattleLogEvent::AbilityStepTriggered { .. })
                && entry.cause.parent_seq() == Some(cast_seq)
        })
        .collect()
}

pub fn focused_event_log_for_cast(event_log: &BattleEventLog, cast_seq: u64) -> BattleEventLog {
    let mut relevant_seqs = vec![cast_seq];

    for entry in &event_log.entries {
        if entry
            .cause
            .parent_seq()
            .is_some_and(|parent_seq| relevant_seqs.contains(&parent_seq))
        {
            relevant_seqs.push(entry.seq);
        }
    }

    let max_relevant_time_ms = event_log
        .entries
        .iter()
        .filter(|entry| relevant_seqs.contains(&entry.seq))
        .map(|entry| entry.time_ms)
        .max()
        .unwrap_or_default();

    BattleEventLog {
        version: event_log.version,
        entries: event_log
            .entries
            .iter()
            .filter(|entry| entry.time_ms <= max_relevant_time_ms)
            .cloned()
            .collect(),
    }
}

pub fn hp_changes_for_unit(
    event_log: &BattleEventLog,
    unit_instance_id: UnitInstanceId,
) -> Vec<&BattleEventLogEntry> {
    event_log
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.event,
                BattleLogEvent::HpChanged {
                    target_instance_id: changed_unit,
                    ..
                } if changed_unit == unit_instance_id
            )
        })
        .collect()
}

pub fn hp_changes_caused_by(
    event_log: &BattleEventLog,
    parent_seq: u64,
) -> Vec<&BattleEventLogEntry> {
    event_log
        .entries
        .iter()
        .filter(|entry| {
            entry_caused_by_seq(event_log, entry, parent_seq)
                && matches!(entry.event, BattleLogEvent::HpChanged { .. })
        })
        .collect()
}

pub fn damage_hp_changes_caused_by(
    event_log: &BattleEventLog,
    parent_seq: u64,
) -> Vec<&BattleEventLogEntry> {
    hp_changes_caused_by(event_log, parent_seq)
        .into_iter()
        .filter(|entry| {
            matches!(
                entry.event,
                BattleLogEvent::HpChanged { delta, .. } if delta < 0
            )
        })
        .collect()
}

pub fn healing_hp_changes_caused_by(
    event_log: &BattleEventLog,
    parent_seq: u64,
) -> Vec<&BattleEventLogEntry> {
    hp_changes_caused_by(event_log, parent_seq)
        .into_iter()
        .filter(|entry| {
            matches!(
                entry.event,
                BattleLogEvent::HpChanged { delta, .. } if delta > 0
            )
        })
        .collect()
}

pub fn stat_changes_caused_by(
    event_log: &BattleEventLog,
    parent_seq: u64,
) -> Vec<&BattleEventLogEntry> {
    event_log
        .entries
        .iter()
        .filter(|entry| {
            entry_caused_by_seq(event_log, entry, parent_seq)
                && matches!(entry.event, BattleLogEvent::StatChanged { .. })
        })
        .collect()
}

pub fn buffs_applied_by(event_log: &BattleEventLog, parent_seq: u64) -> Vec<&BattleEventLogEntry> {
    event_log
        .entries
        .iter()
        .filter(|entry| {
            entry_caused_by_seq(event_log, entry, parent_seq)
                && matches!(entry.event, BattleLogEvent::BuffApplied { .. })
        })
        .collect()
}

pub fn buff_ids(entries: &[&BattleEventLogEntry]) -> Vec<BuffId> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::BuffApplied { buff_id, .. } => Some(buff_id),
            _ => None,
        })
        .collect()
}

pub fn attack_starts_caused_by(
    event_log: &BattleEventLog,
    parent_seq: u64,
) -> Vec<&BattleEventLogEntry> {
    event_log
        .entries
        .iter()
        .filter(|entry| {
            entry_caused_by_seq(event_log, entry, parent_seq)
                && matches!(entry.event, BattleLogEvent::AttackStart { .. })
        })
        .collect()
}

pub fn attack_kinds(entries: &[&BattleEventLogEntry]) -> Vec<Option<AttackKind>> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::AttackStart { kind, .. }
            | BattleLogEvent::AttackResolve { kind, .. }
            | BattleLogEvent::AttackMiss { kind, .. } => Some(kind),
            _ => None,
        })
        .collect()
}

pub fn stat_modifier_summaries(
    entries: &[&BattleEventLogEntry],
) -> Vec<(StatId, StatModifierKind, i32)> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::StatChanged { modifier, .. } => {
                Some((modifier.stat, modifier.kind, modifier.value))
            }
            _ => None,
        })
        .collect()
}

pub fn step_ids<'a>(entries: &'a [&'a BattleEventLogEntry]) -> Vec<&'a str> {
    entries
        .iter()
        .filter_map(|entry| match &entry.event {
            BattleLogEvent::AbilityStepTriggered { step_id, .. } => Some(step_id.as_str()),
            _ => None,
        })
        .collect()
}

pub fn target_unit_ids(entries: &[&BattleEventLogEntry]) -> Vec<UnitInstanceId> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::HpChanged {
                target_instance_id, ..
            }
            | BattleLogEvent::StatChanged {
                target_instance_id, ..
            }
            | BattleLogEvent::BuffApplied {
                target_instance_id, ..
            }
            | BattleLogEvent::AttackStart {
                target_instance_id, ..
            }
            | BattleLogEvent::AttackResolve {
                target_instance_id, ..
            }
            | BattleLogEvent::AttackMiss {
                target_instance_id, ..
            } => Some(target_instance_id),
            _ => None,
        })
        .collect()
}

pub fn hp_deltas(entries: &[&BattleEventLogEntry]) -> Vec<i32> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            BattleLogEvent::HpChanged { delta, .. } => Some(delta),
            _ => None,
        })
        .collect()
}
