use game_core::{
    game::resources::Position,
    game::{
        battle::{
            buffs::BuffId,
            core::movement::types::WorldVec2,
            ids::UnitInstanceId,
            timeline::{AttackKind, Timeline, TimelineEntry, TimelineEvent},
        },
        enums::Side,
        stats::{StatId, StatModifierKind},
    },
};

pub fn find_unit_spawn_at(
    timeline: &Timeline,
    position: Position,
    owner: Option<Side>,
) -> Option<&TimelineEntry> {
    timeline.entries.iter().find(|entry| {
        matches!(
            &entry.event,
            TimelineEvent::UnitSpawned {
                world_position,
                owner: entry_owner,
                ..
            } if spawn_matches_tile_center(*world_position, position)
                && owner.is_none_or(|expected| expected == *entry_owner)
        )
    })
}

fn spawn_matches_tile_center(
    world_position: game_core::game::battle::core::movement::types::TimelineVec2,
    position: Position,
) -> bool {
    world_position
        .to_world()
        .distance(WorldVec2::from_tile_center(position))
        <= 0.001
}

pub fn find_unit_instance_at(
    timeline: &Timeline,
    position: Position,
    owner: Option<Side>,
) -> Option<UnitInstanceId> {
    find_unit_spawn_at(timeline, position, owner).and_then(|entry| match entry.event {
        TimelineEvent::UnitSpawned {
            unit_instance_id, ..
        } => Some(unit_instance_id),
        _ => None,
    })
}

pub fn find_first_ability_cast<'a>(
    timeline: &'a Timeline,
    caster_instance_id: UnitInstanceId,
    skill_id: &str,
) -> Option<&'a TimelineEntry> {
    timeline.entries.iter().find(|entry| {
        matches!(
            &entry.event,
            TimelineEvent::AbilityCast {
                skill_id: actual_skill_id,
                caster_instance_id: actual_caster_id,
                ..
            } if actual_skill_id == skill_id && *actual_caster_id == caster_instance_id
        )
    })
}

pub fn descendants_of(timeline: &Timeline, parent_seq: u64) -> Vec<&TimelineEntry> {
    timeline
        .entries
        .iter()
        .filter(|entry| entry_caused_by_seq(timeline, entry, parent_seq))
        .collect()
}

pub fn entry_caused_by_seq(timeline: &Timeline, entry: &TimelineEntry, expected_seq: u64) -> bool {
    let Some(parent_seq) = entry.cause.parent_seq() else {
        return false;
    };
    if parent_seq == expected_seq {
        return true;
    }

    timeline
        .entries
        .iter()
        .find(|entry| entry.seq == parent_seq)
        .is_some_and(|parent| {
            matches!(parent.event, TimelineEvent::SkillAreaDeclared { .. })
                && parent.cause.parent_seq() == Some(expected_seq)
        })
}

pub fn step_entries_for_cast(timeline: &Timeline, cast_seq: u64) -> Vec<&TimelineEntry> {
    timeline
        .entries
        .iter()
        .filter(|entry| {
            matches!(entry.event, TimelineEvent::AbilityStepTriggered { .. })
                && entry.cause.parent_seq() == Some(cast_seq)
        })
        .collect()
}

pub fn focused_timeline_for_cast(timeline: &Timeline, cast_seq: u64) -> Timeline {
    let mut relevant_seqs = vec![cast_seq];

    for entry in &timeline.entries {
        if entry
            .cause
            .parent_seq()
            .is_some_and(|parent_seq| relevant_seqs.contains(&parent_seq))
        {
            relevant_seqs.push(entry.seq);
        }
    }

    let max_relevant_time_ms = timeline
        .entries
        .iter()
        .filter(|entry| relevant_seqs.contains(&entry.seq))
        .map(|entry| entry.time_ms)
        .max()
        .unwrap_or_default();

    Timeline {
        version: timeline.version,
        entries: timeline
            .entries
            .iter()
            .filter(|entry| entry.time_ms <= max_relevant_time_ms)
            .cloned()
            .collect(),
    }
}

pub fn hp_changes_for_unit(
    timeline: &Timeline,
    unit_instance_id: UnitInstanceId,
) -> Vec<&TimelineEntry> {
    timeline
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.event,
                TimelineEvent::HpChanged {
                    target_instance_id: changed_unit,
                    ..
                } if changed_unit == unit_instance_id
            )
        })
        .collect()
}

pub fn hp_changes_caused_by(timeline: &Timeline, parent_seq: u64) -> Vec<&TimelineEntry> {
    timeline
        .entries
        .iter()
        .filter(|entry| {
            entry_caused_by_seq(timeline, entry, parent_seq)
                && matches!(entry.event, TimelineEvent::HpChanged { .. })
        })
        .collect()
}

pub fn damage_hp_changes_caused_by(timeline: &Timeline, parent_seq: u64) -> Vec<&TimelineEntry> {
    hp_changes_caused_by(timeline, parent_seq)
        .into_iter()
        .filter(|entry| {
            matches!(
                entry.event,
                TimelineEvent::HpChanged { delta, .. } if delta < 0
            )
        })
        .collect()
}

pub fn healing_hp_changes_caused_by(timeline: &Timeline, parent_seq: u64) -> Vec<&TimelineEntry> {
    hp_changes_caused_by(timeline, parent_seq)
        .into_iter()
        .filter(|entry| {
            matches!(
                entry.event,
                TimelineEvent::HpChanged { delta, .. } if delta > 0
            )
        })
        .collect()
}

pub fn stat_changes_caused_by(timeline: &Timeline, parent_seq: u64) -> Vec<&TimelineEntry> {
    timeline
        .entries
        .iter()
        .filter(|entry| {
            entry_caused_by_seq(timeline, entry, parent_seq)
                && matches!(entry.event, TimelineEvent::StatChanged { .. })
        })
        .collect()
}

pub fn buffs_applied_by(timeline: &Timeline, parent_seq: u64) -> Vec<&TimelineEntry> {
    timeline
        .entries
        .iter()
        .filter(|entry| {
            entry_caused_by_seq(timeline, entry, parent_seq)
                && matches!(entry.event, TimelineEvent::BuffApplied { .. })
        })
        .collect()
}

pub fn buff_ids(entries: &[&TimelineEntry]) -> Vec<BuffId> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            TimelineEvent::BuffApplied { buff_id, .. } => Some(buff_id),
            _ => None,
        })
        .collect()
}

pub fn attack_starts_caused_by(timeline: &Timeline, parent_seq: u64) -> Vec<&TimelineEntry> {
    timeline
        .entries
        .iter()
        .filter(|entry| {
            entry_caused_by_seq(timeline, entry, parent_seq)
                && matches!(entry.event, TimelineEvent::AttackStart { .. })
        })
        .collect()
}

pub fn attack_kinds(entries: &[&TimelineEntry]) -> Vec<Option<AttackKind>> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            TimelineEvent::AttackStart { kind, .. }
            | TimelineEvent::AttackResolve { kind, .. }
            | TimelineEvent::AttackMiss { kind, .. } => Some(kind),
            _ => None,
        })
        .collect()
}

pub fn stat_modifier_summaries(entries: &[&TimelineEntry]) -> Vec<(StatId, StatModifierKind, i32)> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            TimelineEvent::StatChanged { modifier, .. } => {
                Some((modifier.stat, modifier.kind, modifier.value))
            }
            _ => None,
        })
        .collect()
}

pub fn step_ids<'a>(entries: &'a [&'a TimelineEntry]) -> Vec<&'a str> {
    entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::AbilityStepTriggered { step_id, .. } => Some(step_id.as_str()),
            _ => None,
        })
        .collect()
}

pub fn target_unit_ids(entries: &[&TimelineEntry]) -> Vec<UnitInstanceId> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            TimelineEvent::HpChanged {
                target_instance_id, ..
            }
            | TimelineEvent::StatChanged {
                target_instance_id, ..
            }
            | TimelineEvent::BuffApplied {
                target_instance_id, ..
            }
            | TimelineEvent::AttackStart {
                target_instance_id, ..
            }
            | TimelineEvent::AttackResolve {
                target_instance_id, ..
            }
            | TimelineEvent::AttackMiss {
                target_instance_id, ..
            } => Some(target_instance_id),
            _ => None,
        })
        .collect()
}

pub fn hp_deltas(entries: &[&TimelineEntry]) -> Vec<i32> {
    entries
        .iter()
        .filter_map(|entry| match entry.event {
            TimelineEvent::HpChanged { delta, .. } => Some(delta),
            _ => None,
        })
        .collect()
}
