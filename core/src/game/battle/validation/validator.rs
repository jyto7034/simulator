use crate::game::battle::timeline::{Timeline, TimelineEntry, TimelineEvent, TIMELINE_VERSION};

use super::{
    attacks::validate_attacks,
    auto_attack::validate_auto_attack_cadence,
    autocast::validate_autocast_pairs,
    battlefield::{
        extract_battlefield_size, position_in_bounds, positions_from_event, BattlefieldSize,
    },
    buffs::validate_buffs,
    deaths::validate_deaths,
    focus::SkillFocusTimeProvider,
    parent::{validate_outcome_parent_relations, validate_parent_seq_relations},
    spawns::{extract_spawns, validate_reference_spawn_order, validate_spawn_counts},
    state::validate_stateful_unit_invariants,
    types::{
        TimelineExpectedCounts, TimelineValidatorConfig, TimelineViolation, TimelineViolationKind,
    },
    unit_stats::validate_spawn_stats,
};

pub struct TimelineValidator {
    config: TimelineValidatorConfig,
}

impl TimelineValidator {
    pub fn new(config: TimelineValidatorConfig) -> Self {
        Self { config }
    }

    pub fn validate(
        &self,
        timeline: &Timeline,
        expected_counts: Option<TimelineExpectedCounts>,
        focus_time_provider: Option<&dyn SkillFocusTimeProvider>,
    ) -> Result<(), Vec<TimelineViolation>> {
        let mut violations = Vec::new();

        if timeline.version != TIMELINE_VERSION && timeline.version != 6 && timeline.version != 7 {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::TimelineVersionMismatch,
                message: format!(
                    "timeline version mismatch: supported={{6, 7, {}}}, got={}",
                    TIMELINE_VERSION, timeline.version
                ),
                entry_index: None,
            });
            return Err(violations);
        }

        if timeline.entries.is_empty() {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::MissingEntries,
                message: "timeline has no entries".to_string(),
                entry_index: None,
            });
            return Err(violations);
        }

        if self.config.require_battle_start_end {
            if !matches!(
                timeline.entries.first().map(|e| &e.event),
                Some(TimelineEvent::BattleStart { .. })
            ) {
                violations.push(TimelineViolation {
                    kind: TimelineViolationKind::MissingBattleStart,
                    message: "timeline does not start with BattleStart".to_string(),
                    entry_index: Some(0),
                });
            }

            if !matches!(
                timeline.entries.last().map(|e| &e.event),
                Some(TimelineEvent::BattleEnd { .. })
            ) {
                violations.push(TimelineViolation {
                    kind: TimelineViolationKind::MissingBattleEnd,
                    message: "timeline does not end with BattleEnd".to_string(),
                    entry_index: Some(timeline.entries.len().saturating_sub(1)),
                });
            }
        }

        let battlefield = extract_battlefield_size(timeline);

        for (index, entry) in timeline.entries.iter().enumerate() {
            self.validate_entry_index_invariants(
                timeline,
                index,
                entry,
                &battlefield,
                &mut violations,
            );
        }

        if self.config.validate_parent_seq {
            validate_parent_seq_relations(timeline, &mut violations);
        }

        if self.config.require_outcome_parent {
            validate_outcome_parent_relations(timeline, &mut violations);
        }

        let extracted = extract_spawns(timeline, &battlefield, &mut violations, &self.config);
        validate_spawn_counts(extracted.counts, expected_counts, &mut violations);
        validate_reference_spawn_order(timeline, &extracted, &mut violations);
        validate_stateful_unit_invariants(timeline, &mut violations, &self.config);
        if self.config.require_autocast_pairs {
            validate_autocast_pairs(timeline, focus_time_provider, &mut violations);
        }
        validate_attacks(timeline, &extracted, &mut violations);
        validate_buffs(timeline, &mut violations);
        validate_deaths(timeline, &extracted, &mut violations, &self.config);
        validate_auto_attack_cadence(timeline, &extracted, &mut violations, &self.config);

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    fn validate_entry_index_invariants(
        &self,
        timeline: &Timeline,
        index: usize,
        entry: &TimelineEntry,
        battlefield: &Option<BattlefieldSize>,
        violations: &mut Vec<TimelineViolation>,
    ) {
        if self.config.require_contiguous_seq && entry.seq != index as u64 {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::NonContiguousSeq,
                message: format!("timeline seq {} does not match index {}", entry.seq, index),
                entry_index: Some(index),
            });
        }

        if self.config.require_attack_kind
            && matches!(
                &entry.event,
                TimelineEvent::Attack { kind: None, .. }
                    | TimelineEvent::AttackStart { kind: None, .. }
                    | TimelineEvent::AttackResolve { kind: None, .. }
                    | TimelineEvent::AttackMiss { kind: None, .. }
            )
        {
            violations.push(TimelineViolation {
                kind: TimelineViolationKind::AttackKindMissing,
                message: "Attack event is missing kind (expected Auto/Triggered)".to_string(),
                entry_index: Some(index),
            });
        }

        if self.config.require_non_decreasing_time && index > 0 {
            let prev = &timeline.entries[index - 1];
            if entry.time_ms < prev.time_ms {
                violations.push(TimelineViolation {
                    kind: TimelineViolationKind::TimeWentBackwards,
                    message: format!(
                        "time_ms {} is less than previous time_ms {}",
                        entry.time_ms, prev.time_ms
                    ),
                    entry_index: Some(index),
                });
            }
        }

        if self.config.require_spawn_stats_valid {
            if let TimelineEvent::UnitSpawned { stats, .. } = &entry.event {
                if let Some(message) = validate_spawn_stats(stats) {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::SpawnStatsInvalid,
                        message,
                        entry_index: Some(index),
                    });
                }
            }
        }

        if self.config.require_hp_delta_consistent {
            if let TimelineEvent::HpChanged {
                delta,
                hp_before,
                hp_after,
                ..
            } = &entry.event
            {
                let computed = i64::from(*hp_after) - i64::from(*hp_before);
                if computed != i64::from(*delta) {
                    violations.push(TimelineViolation {
                        kind: TimelineViolationKind::HpDeltaMismatch,
                        message: format!(
                            "hp delta mismatch: delta={}, before={}, after={}, computed={}",
                            delta, hp_before, hp_after, computed
                        ),
                        entry_index: Some(index),
                    });
                }
            }
        }

        if self.config.validate_positions_within_bounds {
            if let Some(size) = battlefield.as_ref() {
                for (pos, ctx) in positions_from_event(&entry.event) {
                    if !position_in_bounds(pos, size.width, size.height) {
                        violations.push(TimelineViolation {
                            kind: TimelineViolationKind::PositionOutOfBounds,
                            message: format!(
                                "position out of bounds: {} (width={} height={})",
                                ctx, size.width, size.height
                            ),
                            entry_index: Some(index),
                        });
                    }
                }
            }
        }
    }
}
