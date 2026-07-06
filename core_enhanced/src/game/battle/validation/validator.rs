use std::sync::Arc;

use crate::game::battle::{
    buffs::BuffDatabase,
    event_log::{BattleEventLog, BattleEventLogEntry, BattleLogEvent, BATTLE_EVENT_LOG_VERSION},
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
    parent::{
        validate_outcome_parent_relations, validate_parent_seq_relations,
        validate_skill_cast_interrupt_relations,
    },
    spawns::{extract_spawns, validate_reference_spawn_order, validate_spawn_counts},
    state::validate_stateful_unit_invariants,
    types::{
        EventLogExpectedCounts, EventLogValidatorConfig, EventLogViolation, EventLogViolationKind,
    },
    unit_stats::validate_spawn_stats,
};

pub struct EventLogValidator {
    config: EventLogValidatorConfig,
    buff_data: Arc<BuffDatabase>,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use crate::game::{
        ability::SkillId,
        battle::{
            buffs::{BuffDatabase, BuffId, BuffKind, BuffMetadata, BuffReapplyPolicy},
            event_log::{
                BattleEventCause, BattleEventLog, BattleEventLogEntry, BattleLogEvent,
                BuffExpireReason, SkillCastCancelReason,
            },
            ids::UnitInstanceId,
        },
        enums::Side,
        stats::UnitStats,
    };

    use super::{EventLogValidator, EventLogValidatorConfig, EventLogViolationKind};

    fn buff_only_config() -> EventLogValidatorConfig {
        EventLogValidatorConfig {
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

    fn unit_spawn(seq: u64, unit_instance_id: UnitInstanceId, owner: Side) -> BattleEventLogEntry {
        BattleEventLogEntry {
            time_ms: 0,
            seq,
            cause: BattleEventCause::default(),
            source_command_id: None,
            event: BattleLogEvent::UnitSpawned {
                unit_instance_id,
                owner,
                role: Default::default(),
                threat_class: Default::default(),
                mobility_kind: Default::default(),
                base_uuid: Uuid::nil(),
                unit_source: crate::game::battle::types::BattleUnitSourceIdentity::TestFixture {
                    base_uuid: Uuid::nil(),
                },
                world_position: Default::default(),
                stats: UnitStats::with_values(100, 100, 1, 0, 1),
            },
        }
    }

    #[test]
    fn normal_validation_rejects_legacy_event_log_versions() {
        for legacy_version in [6, 7] {
            let mut event_log = BattleEventLog::new();
            event_log.version = legacy_version;
            event_log.entries.push(unit_spawn(
                0,
                UnitInstanceId::from(Uuid::from_u128(u128::from(legacy_version))),
                Side::Opponent,
            ));

            let violations = EventLogValidator::new(buff_only_config())
                .validate(&event_log, None, None)
                .expect_err("legacy event_log versions must not be accepted");

            assert!(violations.iter().any(|violation| {
                violation.kind == EventLogViolationKind::EventLogVersionMismatch
            }));
        }
    }

    #[test]
    fn event_log_validator_uses_injected_buff_database() {
        let mut event_log = BattleEventLog::new();
        event_log.entries.push(BattleEventLogEntry {
            time_ms: 0,
            seq: 0,
            cause: BattleEventCause::default(),
            source_command_id: None,
            event: BattleLogEvent::BuffApplied {
                caster_instance_id: UnitInstanceId::from(Uuid::from_u128(1)),
                target_instance_id: UnitInstanceId::from(Uuid::from_u128(2)),
                buff_id: BuffId::from_name("poison"),
                duration_ms: 100,
            },
        });

        let validator = EventLogValidator::with_buff_data(
            buff_only_config(),
            Arc::new(BuffDatabase::new(vec![])),
        );
        let violations = validator
            .validate(&event_log, None, None)
            .expect_err("empty injected buff database must reject poison");

        assert!(violations
            .iter()
            .any(|violation| violation.kind == EventLogViolationKind::UnknownBuffId));
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
            max_stacks: 1,
            reapply_policy: BuffReapplyPolicy::RefreshDuration,
        }]));
        let mut event_log = BattleEventLog::new();
        event_log.entries.extend([
            unit_spawn(0, caster, Side::Player),
            unit_spawn(1, target, Side::Opponent),
            BattleEventLogEntry {
                time_ms: 0,
                seq: 2,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::BuffApplied {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id,
                    duration_ms: 100,
                },
            },
            BattleEventLogEntry {
                time_ms: 50,
                seq: 3,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::BuffApplied {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id,
                    duration_ms: 10,
                },
            },
            BattleEventLogEntry {
                time_ms: 60,
                seq: 4,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::BuffExpired {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id,
                    reason: BuffExpireReason::Natural,
                },
            },
        ]);

        let validator = EventLogValidator::with_buff_data(buff_only_config(), buff_data.clone());
        validator
            .validate(&event_log, None, None)
            .expect("RefreshDuration should replace expires_at_ms instead of taking max");

        event_log.entries[4].time_ms = 100;
        let violations = EventLogValidator::with_buff_data(buff_only_config(), buff_data)
            .validate(&event_log, None, None)
            .expect_err("stale original expiration must not validate");
        assert!(violations
            .iter()
            .any(|violation| violation.kind == EventLogViolationKind::BuffExpiredInvalid));
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
        let mut event_log = BattleEventLog::new();
        event_log.entries.extend([
            unit_spawn(0, caster, Side::Player),
            unit_spawn(1, target, Side::Opponent),
            BattleEventLogEntry {
                time_ms: 0,
                seq: 2,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::BuffApplied {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id: stun_id,
                    duration_ms: 100,
                },
            },
            BattleEventLogEntry {
                time_ms: 10,
                seq: 3,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::BuffExpired {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id: stun_id,
                    reason: BuffExpireReason::Replaced,
                },
            },
            BattleEventLogEntry {
                time_ms: 10,
                seq: 4,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::BuffApplied {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id: freeze_id,
                    duration_ms: 20,
                },
            },
            BattleEventLogEntry {
                time_ms: 30,
                seq: 5,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::BuffExpired {
                    caster_instance_id: caster,
                    target_instance_id: target,
                    buff_id: freeze_id,
                    reason: BuffExpireReason::Natural,
                },
            },
        ]);

        EventLogValidator::with_buff_data(buff_only_config(), buff_data.clone())
            .validate(&event_log, None, None)
            .expect("freeze should replace stun and expire at its own duration");

        event_log.entries.push(BattleEventLogEntry {
            time_ms: 100,
            seq: 5,
            cause: BattleEventCause::default(),
            source_command_id: None,
            event: BattleLogEvent::BuffExpired {
                caster_instance_id: caster,
                target_instance_id: target,
                buff_id: stun_id,
                reason: BuffExpireReason::Natural,
            },
        });
        let violations = EventLogValidator::with_buff_data(buff_only_config(), buff_data)
            .validate(&event_log, None, None)
            .expect_err("replaced hard CC should not remain active");
        assert!(violations
            .iter()
            .any(|violation| violation.kind == EventLogViolationKind::BuffExpiredInvalid));
    }

    #[test]
    fn validator_requires_interrupt_to_reference_cast_start() {
        let interrupter = UnitInstanceId::from(Uuid::from_u128(1));
        let caster = UnitInstanceId::from(Uuid::from_u128(2));
        let mut event_log = BattleEventLog::new();
        event_log.entries.extend([
            unit_spawn(0, interrupter, Side::Player),
            unit_spawn(1, caster, Side::Opponent),
            BattleEventLogEntry {
                time_ms: 10,
                seq: 2,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::ManualCastStart {
                    caster_instance_id: caster,
                    skill_id: SkillId::from("casting"),
                    target: Some(crate::game::battle::event_log::SkillCastTarget::Unit {
                        unit_instance_id: caster,
                    }),
                },
            },
            BattleEventLogEntry {
                time_ms: 11,
                seq: 3,
                cause: BattleEventCause::Parent { seq: 2 },
                source_command_id: None,
                event: BattleLogEvent::SkillCastInterrupted {
                    interrupter_instance_id: interrupter,
                    caster_instance_id: caster,
                    interrupted_skill_id: SkillId::from("casting"),
                    interrupted_cast_seq: 2,
                },
            },
        ]);

        EventLogValidator::new(buff_only_config())
            .validate(&event_log, None, None)
            .expect("interrupt should reference a valid cast start");

        if let BattleLogEvent::SkillCastInterrupted {
            interrupted_cast_seq,
            ..
        } = &mut event_log.entries[3].event
        {
            *interrupted_cast_seq = 99;
        }

        let violations = EventLogValidator::new(buff_only_config())
            .validate(&event_log, None, None)
            .expect_err("interrupt with missing cast start must fail");
        assert!(violations
            .iter()
            .any(|violation| violation.kind == EventLogViolationKind::AutoCastPairInvalid));
    }

    #[test]
    fn validator_accepts_cancelled_active_skill_cast_referencing_ability_cast() {
        let caster = UnitInstanceId::from(Uuid::from_u128(1));
        let skill_id = SkillId::from("active_cancel");
        let mut event_log = BattleEventLog::new();
        event_log.entries.extend([
            unit_spawn(0, caster, Side::Player),
            BattleEventLogEntry {
                time_ms: 10,
                seq: 2,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::AbilityCast {
                    skill_id: skill_id.clone(),
                    caster_instance_id: caster,
                    target_instance_id: None,
                },
            },
            BattleEventLogEntry {
                time_ms: 11,
                seq: 3,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::SkillCastCancelled {
                    caster_instance_id: caster,
                    interrupted_skill_id: skill_id,
                    interrupted_cast_seq: 2,
                    reason: SkillCastCancelReason::Withdrawn,
                },
            },
        ]);

        EventLogValidator::new(buff_only_config())
            .validate(&event_log, None, None)
            .expect("cancelled active skill cast should reference AbilityCast seq");
    }
}

impl EventLogValidator {
    pub fn new(config: EventLogValidatorConfig) -> Self {
        Self {
            config,
            buff_data: Arc::new(BuffDatabase::new(vec![])),
        }
    }

    pub fn with_buff_data(config: EventLogValidatorConfig, buff_data: Arc<BuffDatabase>) -> Self {
        Self { config, buff_data }
    }

    pub fn validate(
        &self,
        event_log: &BattleEventLog,
        expected_counts: Option<EventLogExpectedCounts>,
        focus_time_provider: Option<&dyn SkillFocusTimeProvider>,
    ) -> Result<(), Vec<EventLogViolation>> {
        let mut violations = Vec::new();

        if event_log.version != BATTLE_EVENT_LOG_VERSION {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::EventLogVersionMismatch,
                message: format!(
                    "event_log version mismatch: supported={}, got={}",
                    BATTLE_EVENT_LOG_VERSION, event_log.version
                ),
                entry_index: None,
            });
            return Err(violations);
        }

        if event_log.entries.is_empty() {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::MissingEntries,
                message: "event_log has no entries".to_string(),
                entry_index: None,
            });
            return Err(violations);
        }

        if self.config.require_battle_start_end {
            if !matches!(
                event_log.entries.first().map(|e| &e.event),
                Some(BattleLogEvent::BattleStart { .. })
            ) {
                violations.push(EventLogViolation {
                    kind: EventLogViolationKind::MissingBattleStart,
                    message: "event_log does not start with BattleStart".to_string(),
                    entry_index: Some(0),
                });
            }

            if !matches!(
                event_log.entries.last().map(|e| &e.event),
                Some(BattleLogEvent::BattleEnd { .. })
            ) {
                violations.push(EventLogViolation {
                    kind: EventLogViolationKind::MissingBattleEnd,
                    message: "event_log does not end with BattleEnd".to_string(),
                    entry_index: Some(event_log.entries.len().saturating_sub(1)),
                });
            }
        }

        let battlefield = extract_battlefield_size(event_log);

        for (index, entry) in event_log.entries.iter().enumerate() {
            self.validate_entry_index_invariants(
                event_log,
                index,
                entry,
                &battlefield,
                &mut violations,
            );
        }

        if self.config.validate_parent_seq {
            validate_parent_seq_relations(event_log, &mut violations);
        }

        if self.config.require_outcome_parent {
            validate_outcome_parent_relations(event_log, &mut violations);
        }
        validate_skill_cast_interrupt_relations(event_log, &mut violations);

        let extracted = extract_spawns(event_log, &battlefield, &mut violations, &self.config);
        validate_spawn_counts(extracted.counts, expected_counts, &mut violations);
        validate_reference_spawn_order(event_log, &extracted, &mut violations);
        validate_stateful_unit_invariants(event_log, &mut violations, &self.config);
        if self.config.require_autocast_pairs {
            validate_autocast_pairs(event_log, focus_time_provider, &mut violations);
        }
        validate_attacks(event_log, &extracted, &mut violations);
        validate_buffs(event_log, &self.buff_data, &mut violations);
        validate_deaths(event_log, &extracted, &mut violations, &self.config);
        validate_auto_attack_cadence(event_log, &extracted, &mut violations, &self.config);

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    fn validate_entry_index_invariants(
        &self,
        event_log: &BattleEventLog,
        index: usize,
        entry: &BattleEventLogEntry,
        battlefield: &Option<BattlefieldSize>,
        violations: &mut Vec<EventLogViolation>,
    ) {
        if self.config.require_contiguous_seq && entry.seq != index as u64 {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::NonContiguousSeq,
                message: format!("event_log seq {} does not match index {}", entry.seq, index),
                entry_index: Some(index),
            });
        }

        if self.config.require_attack_kind
            && matches!(
                &entry.event,
                BattleLogEvent::AttackStart { kind: None, .. }
                    | BattleLogEvent::AttackResolve { kind: None, .. }
                    | BattleLogEvent::AttackMiss { kind: None, .. }
            )
        {
            violations.push(EventLogViolation {
                kind: EventLogViolationKind::AttackKindMissing,
                message: "Attack event is missing kind (expected Auto/Triggered)".to_string(),
                entry_index: Some(index),
            });
        }

        if self.config.require_non_decreasing_time && index > 0 {
            let prev = &event_log.entries[index - 1];
            if entry.time_ms < prev.time_ms {
                violations.push(EventLogViolation {
                    kind: EventLogViolationKind::TimeWentBackwards,
                    message: format!(
                        "time_ms {} is less than previous time_ms {}",
                        entry.time_ms, prev.time_ms
                    ),
                    entry_index: Some(index),
                });
            }
        }

        if self.config.require_spawn_stats_valid {
            if let BattleLogEvent::UnitSpawned { stats, .. } = &entry.event {
                if let Some(message) = validate_spawn_stats(stats) {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::SpawnStatsInvalid,
                        message,
                        entry_index: Some(index),
                    });
                }
            }
        }

        if self.config.require_hp_delta_consistent {
            if let BattleLogEvent::HpChanged {
                delta,
                hp_before,
                hp_after,
                ..
            } = &entry.event
            {
                let computed = i64::from(*hp_after) - i64::from(*hp_before);
                if computed != i64::from(*delta) {
                    violations.push(EventLogViolation {
                        kind: EventLogViolationKind::HpDeltaMismatch,
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
                        violations.push(EventLogViolation {
                            kind: EventLogViolationKind::PositionOutOfBounds,
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
