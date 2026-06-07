use std::sync::Arc;

use crate::game::battle::{
    buffs::BuffDatabase,
    timeline::{Timeline, TimelineEntry, TimelineEvent, TIMELINE_VERSION},
};

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
    buff_data: Arc<BuffDatabase>,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use crate::game::battle::{
        buffs::{BuffDatabase, BuffId, BuffKind, BuffMetadata, BuffReapplyPolicy},
        ids::UnitInstanceId,
        timeline::{Timeline, TimelineCause, TimelineEntry, TimelineEvent},
    };
    use crate::game::{enums::Side, stats::UnitStats};

    use super::{TimelineValidator, TimelineValidatorConfig, TimelineViolationKind};

    fn buff_only_config() -> TimelineValidatorConfig {
        TimelineValidatorConfig {
            require_battle_start_end: false,
            require_contiguous_seq: false,
            require_non_decreasing_time: false,
            require_attack_kind: false,
            validate_parent_seq: false,
            require_outcome_parent: false,
            require_autocast_pairs: false,
            require_spawn_stats_valid: false,
            require_hp_delta_consistent: false,
            forbid_dead_units_as_attackers: false,
            forbid_dead_units_as_targets: false,
            validate_positions_within_bounds: false,
            require_movement_path_consistent: false,
            validate_auto_attack_min_interval: false,
            validate_auto_attack_presence: false,
            auto_attack_timing_tolerance_ms: 0,
        }
    }

    fn unit_spawn(seq: u64, unit_instance_id: UnitInstanceId, owner: Side) -> TimelineEntry {
        TimelineEntry {
            time_ms: 0,
            seq,
            cause: TimelineCause::default(),
            event: TimelineEvent::UnitSpawned {
                unit_instance_id,
                owner,
                role: Default::default(),
                mobility_kind: Default::default(),
                base_uuid: Uuid::nil(),
                world_position: Default::default(),
                stats: UnitStats::with_values(100, 100, 1, 0, 1),
            },
        }
    }

    #[test]
    fn timeline_validator_uses_injected_buff_database() {
        let mut timeline = Timeline::new();
        timeline.entries.push(TimelineEntry {
            time_ms: 0,
            seq: 0,
            cause: TimelineCause::default(),
            event: TimelineEvent::BuffApplied {
                caster_instance_id: UnitInstanceId::from(Uuid::from_u128(1)),
                target_instance_id: UnitInstanceId::from(Uuid::from_u128(2)),
                buff_id: BuffId::from_name("poison"),
                duration_ms: 100,
            },
        });

        let validator = TimelineValidator::with_buff_data(
            buff_only_config(),
            Arc::new(BuffDatabase::new(vec![])),
        );
        let violations = validator
            .validate(&timeline, None, None)
            .expect_err("empty injected buff database must reject poison");

        assert!(violations
            .iter()
            .any(|violation| violation.kind == TimelineViolationKind::UnknownBuffId));
    }

    #[test]
    fn buff_validator_respects_refresh_duration_reapply_policy() {
        let caster = UnitInstanceId::from(Uuid::from_u128(1));
        let target = UnitInstanceId::from(Uuid::from_u128(2));
        let buff_id = BuffId::from_name("refreshing_silence");
        let buff_data = Arc::new(BuffDatabase::new(vec![BuffMetadata {
            name: "refreshing_silence".to_string(),
            kind: BuffKind::Silence,
            tick_interval_ms: 0,
            max_stacks: 3,
            reapply_policy: BuffReapplyPolicy::RefreshDuration,
        }]));
        let mut timeline = Timeline::new();
        timeline.entries.extend([
            unit_spawn(0, caster, Side::Player),
            unit_spawn(1, target, Side::Opponent),
            TimelineEntry {
                time_ms: 0,
                seq: 2,
                cause: TimelineCause::default(),
                event: TimelineEvent::BuffApplied {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id,
                    duration_ms: 100,
                },
            },
            TimelineEntry {
                time_ms: 50,
                seq: 3,
                cause: TimelineCause::default(),
                event: TimelineEvent::BuffApplied {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id,
                    duration_ms: 10,
                },
            },
            TimelineEntry {
                time_ms: 60,
                seq: 4,
                cause: TimelineCause::default(),
                event: TimelineEvent::BuffExpired {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id,
                },
            },
        ]);

        let validator = TimelineValidator::with_buff_data(buff_only_config(), buff_data.clone());
        validator
            .validate(&timeline, None, None)
            .expect("RefreshDuration should replace expires_at_ms instead of taking max");

        timeline.entries[4].time_ms = 100;
        let violations = TimelineValidator::with_buff_data(buff_only_config(), buff_data)
            .validate(&timeline, None, None)
            .expect_err("stale original expiration must not validate");
        assert!(violations
            .iter()
            .any(|violation| violation.kind == TimelineViolationKind::BuffExpiredInvalid));
    }

    #[test]
    fn buff_validator_models_exclusive_hard_cc_replacement() {
        let caster = UnitInstanceId::from(Uuid::from_u128(1));
        let target = UnitInstanceId::from(Uuid::from_u128(2));
        let stun_id = BuffId::from_name("stun");
        let freeze_id = BuffId::from_name("freeze");
        let buff_data = Arc::new(BuffDatabase::new(vec![
            BuffMetadata {
                name: "stun".to_string(),
                kind: BuffKind::Stun,
                tick_interval_ms: 0,
                max_stacks: 1,
                reapply_policy: BuffReapplyPolicy::RefreshDuration,
            },
            BuffMetadata {
                name: "freeze".to_string(),
                kind: BuffKind::Freeze,
                tick_interval_ms: 0,
                max_stacks: 1,
                reapply_policy: BuffReapplyPolicy::RefreshDuration,
            },
        ]));
        let mut timeline = Timeline::new();
        timeline.entries.extend([
            unit_spawn(0, caster, Side::Player),
            unit_spawn(1, target, Side::Opponent),
            TimelineEntry {
                time_ms: 0,
                seq: 2,
                cause: TimelineCause::default(),
                event: TimelineEvent::BuffApplied {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id: stun_id,
                    duration_ms: 100,
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 3,
                cause: TimelineCause::default(),
                event: TimelineEvent::BuffApplied {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id: freeze_id,
                    duration_ms: 20,
                },
            },
            TimelineEntry {
                time_ms: 30,
                seq: 4,
                cause: TimelineCause::default(),
                event: TimelineEvent::BuffExpired {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id: freeze_id,
                },
            },
        ]);

        TimelineValidator::with_buff_data(buff_only_config(), buff_data.clone())
            .validate(&timeline, None, None)
            .expect("freeze should replace stun and expire at its own duration");

        timeline.entries.push(TimelineEntry {
            time_ms: 100,
            seq: 5,
            cause: TimelineCause::default(),
            event: TimelineEvent::BuffExpired {
                caster_instance_id: caster,
                target_instance_id: target,
                buff_id: stun_id,
            },
        });
        let violations = TimelineValidator::with_buff_data(buff_only_config(), buff_data)
            .validate(&timeline, None, None)
            .expect_err("replaced hard CC should not remain active");
        assert!(violations
            .iter()
            .any(|violation| violation.kind == TimelineViolationKind::BuffExpiredInvalid));
    }
}

impl TimelineValidator {
    pub fn new(config: TimelineValidatorConfig) -> Self {
        Self {
            config,
            buff_data: Arc::new(BuffDatabase::new(vec![])),
        }
    }

    pub fn with_live_buff_data(config: TimelineValidatorConfig) -> Self {
        Self {
            config,
            buff_data: Arc::new(BuffDatabase::live_default()),
        }
    }

    pub fn with_buff_data(config: TimelineValidatorConfig, buff_data: Arc<BuffDatabase>) -> Self {
        Self { config, buff_data }
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
        validate_buffs(timeline, &self.buff_data, &mut violations);
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
                TimelineEvent::AttackStart { kind: None, .. }
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
