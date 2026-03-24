pub mod types;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use uuid::Uuid;

use crate::game::{
    battle::{
        buffs,
        ids::UnitInstanceId,
        replay::types::{
            TimelineReplayViolation, TimelineReplayViolationKind, TimelineReplayerConfig,
        },
        timeline::{Timeline, TimelineCause, TimelineEvent, TIMELINE_VERSION},
    },
    data::GameDataBase,
};

pub struct TimelineReplayer {
    game_data: Arc<GameDataBase>,
    pub config: TimelineReplayerConfig,
}

impl TimelineReplayer {
    pub fn new(game_data: Arc<GameDataBase>, config: TimelineReplayerConfig) -> Self {
        Self { game_data, config }
    }

    pub fn replay(&self, timeline: &Timeline) -> Result<(), Vec<TimelineReplayViolation>> {
        let mut violations: Vec<TimelineReplayViolation> = Vec::new();

        if timeline.version != TIMELINE_VERSION && timeline.version != 6 && timeline.version != 7 {
            violations.push(TimelineReplayViolation {
                kind: TimelineReplayViolationKind::TimelineVersionMismatch,
                message: format!(
                    "timeline version mismatch: supported={{6, 7, {}}}, got={}",
                    TIMELINE_VERSION, timeline.version
                ),
                entry_index: None,
            });
            return Err(violations);
        }

        let mut seq_to_index: HashMap<u64, usize> = HashMap::new();
        for (index, entry) in timeline.entries.iter().enumerate() {
            seq_to_index.insert(entry.seq, index);
        }

        #[derive(Debug, Clone, Copy)]
        struct UnitState {
            basic_attack_locked_until_ms: u64,
        }

        let mut units: HashMap<UnitInstanceId, UnitState> = HashMap::new();
        let mut seen_items: HashSet<Uuid> = HashSet::new();
        let mut seen_artifacts: HashSet<Uuid> = HashSet::new();

        // For pairing: caster -> (start_seq, due_time_ms)
        let mut expected_autocast_end: HashMap<UnitInstanceId, (u64, u64)> = HashMap::new();

        for (index, entry) in timeline.entries.iter().enumerate() {
            match &entry.event {
                TimelineEvent::BattleStart { .. }
                | TimelineEvent::BattleEnd { .. }
                | TimelineEvent::ResonanceChanged { .. } => {}

                TimelineEvent::UnitSpawned {
                    unit_instance_id,
                    base_uuid,
                    ..
                } => {
                    if self.config.validate_unit_base_uuid
                        && self
                            .game_data
                            .abnormality_data
                            .get_by_uuid(base_uuid)
                            .is_none()
                    {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownUnitBaseReference,
                            message: format!(
                                "UnitSpawned references unknown base_uuid {}",
                                base_uuid
                            ),
                            entry_index: Some(index),
                        });
                    }

                    units.insert(
                        *unit_instance_id,
                        UnitState {
                            basic_attack_locked_until_ms: 0,
                        },
                    );
                }

                TimelineEvent::ItemSpawned {
                    item_instance_id,
                    owner_unit_instance_id,
                    base_uuid,
                    ..
                } => {
                    if !seen_items.insert(*item_instance_id) {
                        // validator handles dupes; keep replay lenient.
                    }

                    if units.get(owner_unit_instance_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownUnitReference,
                            message: format!(
                                "ItemSpawned references unknown owner unit {}",
                                owner_unit_instance_id
                            ),
                            entry_index: Some(index),
                        });
                    }

                    if self
                        .game_data
                        .equipment_data
                        .get_by_uuid(base_uuid)
                        .is_none()
                    {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownItemReference,
                            message: format!(
                                "ItemSpawned references unknown base_uuid {}",
                                base_uuid
                            ),
                            entry_index: Some(index),
                        });
                    }
                }

                TimelineEvent::ArtifactSpawned {
                    artifact_instance_id,
                    base_uuid,
                    ..
                } => {
                    if !seen_artifacts.insert(*artifact_instance_id) {
                        // validator handles dupes; keep replay lenient.
                    }

                    if self
                        .game_data
                        .artifact_data
                        .get_by_uuid(base_uuid)
                        .is_none()
                    {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownArtifactReference,
                            message: format!(
                                "ArtifactSpawned references unknown base_uuid {}",
                                base_uuid
                            ),
                            entry_index: Some(index),
                        });
                    }
                }

                TimelineEvent::UnitMoved { .. } | TimelineEvent::MovementStopped { .. } => {
                    // Movement is validated in `battle::validation`; replay focuses on cause/outcome links.
                }

                TimelineEvent::AttackStart {
                    attacker_instance_id,
                    target_instance_id,
                    kind,
                    ..
                } => {
                    if kind.is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::AttackKindMissing,
                            message: "Attack event is missing kind (expected Auto/Triggered)"
                                .to_string(),
                            entry_index: Some(index),
                        });
                    }

                    if units.get(attacker_instance_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownUnitReference,
                            message: format!(
                                "Attack references unknown attacker {}",
                                attacker_instance_id
                            ),
                            entry_index: Some(index),
                        });
                    }
                    if units.get(target_instance_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownUnitReference,
                            message: format!(
                                "Attack references unknown target {}",
                                target_instance_id
                            ),
                            entry_index: Some(index),
                        });
                    }

                    if let Some(state) = units.get(attacker_instance_id) {
                        if entry.time_ms < state.basic_attack_locked_until_ms {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::AttackDuringCast,
                                message: format!(
                                    "Attack at {}ms occurs before basic_attack_locked_until_ms {} (attacker={})",
                                    entry.time_ms,
                                    state.basic_attack_locked_until_ms,
                                    attacker_instance_id
                                ),
                                entry_index: Some(index),
                            });
                        }
                    }
                }
                TimelineEvent::AttackResolve { .. }
                | TimelineEvent::AttackMiss { .. }
                | TimelineEvent::ProjectileMiss { .. } => {}

                TimelineEvent::AutoCastStart {
                    caster_instance_id,
                    skill_id,
                    ..
                } => {
                    if units.get(caster_instance_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownUnitReference,
                            message: format!(
                                "AutoCastStart references unknown caster {}",
                                caster_instance_id
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    if let TimelineCause::Parent { seq: parent_seq } = entry.cause {
                        let Some(&parent_index) = seq_to_index.get(&parent_seq) else {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                                message: format!(
                                    "AutoCastStart cause seq {} does not reference any entry (caster={})",
                                    parent_seq, caster_instance_id
                                ),
                                entry_index: Some(index),
                            });
                            continue;
                        };

                        if parent_index >= index {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                                message: format!(
                                    "AutoCastStart cause seq {} is not before entry index {} (caster={})",
                                    parent_seq, index, caster_instance_id
                                ),
                                entry_index: Some(index),
                            });
                            continue;
                        }

                        let parent_event = &timeline.entries[parent_index].event;
                        let valid_parent = matches!(
                            parent_event,
                            TimelineEvent::AttackStart { .. }
                                | TimelineEvent::AttackResolve { .. }
                                | TimelineEvent::AttackMiss { .. }
                                | TimelineEvent::AbilityCast { .. }
                                | TimelineEvent::BuffTick { .. }
                                | TimelineEvent::AutoCastStart { .. }
                                | TimelineEvent::AutoCastEnd { .. }
                        );
                        if !valid_parent {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                                message: format!(
                                    "AutoCastStart cause seq {} points to invalid parent event {:?} (caster={})",
                                    parent_seq, parent_event, caster_instance_id
                                ),
                                entry_index: Some(index),
                            });
                        }

                        if let TimelineEvent::AutoCastStart {
                            caster_instance_id: parent_caster,
                            ..
                        } = parent_event
                        {
                            if parent_caster != caster_instance_id {
                                violations.push(TimelineReplayViolation {
                                    kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                                    message: format!(
                                        "AutoCastStart cause seq {} points to AutoCastStart of different caster (expected {}, got {})",
                                        parent_seq, caster_instance_id, parent_caster
                                    ),
                                    entry_index: Some(index),
                                });
                            }
                        }

                        if let TimelineEvent::AutoCastEnd {
                            caster_instance_id: parent_caster,
                        } = parent_event
                        {
                            if parent_caster != caster_instance_id {
                                violations.push(TimelineReplayViolation {
                                    kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                                    message: format!(
                                        "AutoCastStart cause seq {} points to AutoCastEnd of different caster (expected {}, got {})",
                                        parent_seq, caster_instance_id, parent_caster
                                    ),
                                    entry_index: Some(index),
                                });
                            }
                        }
                    }

                    let maybe_skill = skill_id
                        .as_ref()
                        .and_then(|id| self.game_data.skill_data.get_by_id(id));

                    let cast_end_ms = if let Some(skill) = maybe_skill {
                        if skill.focus_time_ms == 0 {
                            entry.time_ms.saturating_add(1)
                        } else {
                            entry.time_ms.saturating_add(skill.focus_time_ms as u64)
                        }
                    } else {
                        entry.time_ms.saturating_add(1)
                    };

                    if self.config.validate_autocast_pairing {
                        if expected_autocast_end
                            .insert(*caster_instance_id, (entry.seq, cast_end_ms))
                            .is_some()
                        {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                                message: format!(
                                    "AutoCastStart overlaps with an existing pending cast (caster={})",
                                    caster_instance_id
                                ),
                                entry_index: Some(index),
                            });
                        }
                    }

                    // If focus disallows basic attacks, enforce an attack lock window.
                    if let Some(skill) = maybe_skill {
                        if !skill.focus_permissions.allows_basic_attack {
                            if let Some(unit) = units.get_mut(caster_instance_id) {
                                unit.basic_attack_locked_until_ms =
                                    unit.basic_attack_locked_until_ms.max(cast_end_ms);
                            }
                        }
                    }
                }

                TimelineEvent::AutoCastEnd { caster_instance_id } => {
                    if !self.config.validate_autocast_pairing {
                        continue;
                    }

                    let TimelineCause::Parent { seq: start_seq } = entry.cause else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                            message: format!(
                                "AutoCastEnd recorded without a Parent cause (caster={})",
                                caster_instance_id
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    match expected_autocast_end.remove(caster_instance_id) {
                        None => {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                                message: format!(
                                    "AutoCastEnd recorded without a prior AutoCastStart (caster={})",
                                    caster_instance_id
                                ),
                                entry_index: Some(index),
                            });
                        }
                        Some((expected_start_seq, due_time_ms)) => {
                            if start_seq != expected_start_seq {
                                violations.push(TimelineReplayViolation {
                                    kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                                    message: format!(
                                        "AutoCastEnd cause seq mismatch for {}: expected {}, got {}",
                                        caster_instance_id, expected_start_seq, start_seq
                                    ),
                                    entry_index: Some(index),
                                });
                            }
                            if entry.time_ms < due_time_ms {
                                violations.push(TimelineReplayViolation {
                                    kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                                    message: format!(
                                        "AutoCastEnd recorded too early for {}: earliest {}ms, got {}ms",
                                        caster_instance_id, due_time_ms, entry.time_ms
                                    ),
                                    entry_index: Some(index),
                                });
                            }
                        }
                    }
                }

                TimelineEvent::AbilityCast {
                    caster_instance_id, ..
                } => {
                    // AbilityCast may be produced by auto-cast (unit) or by proc skills
                    // (items/artifacts) once `CastSkill` is implemented.
                    let TimelineCause::Parent { seq: parent_seq } = entry.cause else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnexpectedDecision,
                            message: format!(
                                "AbilityCast recorded without a Parent cause: {:?}",
                                entry.event
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    let Some(&parent_index) = seq_to_index.get(&parent_seq) else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnexpectedDecision,
                            message: format!(
                                "AbilityCast cause seq {} does not reference any entry",
                                parent_seq
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    if parent_index >= index {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnexpectedDecision,
                            message: format!(
                                "AbilityCast cause seq {} is not before entry index {}",
                                parent_seq, index
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    }

                    if !matches!(
                        timeline.entries[parent_index].event,
                        TimelineEvent::AutoCastStart { .. }
                            | TimelineEvent::AttackStart { .. }
                            | TimelineEvent::AttackResolve { .. }
                            | TimelineEvent::BuffTick { .. }
                    ) {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnexpectedDecision,
                            message: format!(
                                "AbilityCast cause seq {} points to invalid parent event: {:?}",
                                parent_seq, timeline.entries[parent_index].event
                            ),
                            entry_index: Some(index),
                        });
                    }

                    if units.get(caster_instance_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownUnitReference,
                            message: format!(
                                "AbilityCast references unknown caster {}",
                                caster_instance_id
                            ),
                            entry_index: Some(index),
                        });
                    }
                }
                TimelineEvent::AbilityStepTriggered {
                    caster_instance_id,
                    target_instance_id,
                    ..
                } => {
                    if units.get(caster_instance_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownUnitReference,
                            message: format!(
                                "AbilityStepTriggered references unknown caster {}",
                                caster_instance_id
                            ),
                            entry_index: Some(index),
                        });
                    }
                    if let Some(target_instance_id) = target_instance_id {
                        if units.get(target_instance_id).is_none() {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::UnknownUnitReference,
                                message: format!(
                                    "AbilityStepTriggered references unknown target {}",
                                    target_instance_id
                                ),
                                entry_index: Some(index),
                            });
                        }
                    }
                }

                TimelineEvent::BuffApplied {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                    duration_ms,
                } => {
                    if buffs::get(*buff_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownBuffReference,
                            message: format!(
                                "BuffApplied references unknown buff_id {}",
                                buff_id.as_u64()
                            ),
                            entry_index: Some(index),
                        });
                    }

                    if *duration_ms == 0 {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::InvalidBuffEvent,
                            message: "BuffApplied has duration_ms == 0".to_string(),
                            entry_index: Some(index),
                        });
                    }

                    if units.get(caster_instance_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownUnitReference,
                            message: format!(
                                "BuffApplied references unknown caster {}",
                                caster_instance_id
                            ),
                            entry_index: Some(index),
                        });
                    }
                    if units.get(target_instance_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownUnitReference,
                            message: format!(
                                "BuffApplied references unknown target {}",
                                target_instance_id
                            ),
                            entry_index: Some(index),
                        });
                    }
                }

                TimelineEvent::BuffTick {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                } => {
                    // Relationship: BuffTick should be a child of a BuffApplied entry.
                    // TODO: If buff refresh semantics change, decide whether BuffTick should parent
                    // to the initial BuffApplied (current behavior) or the latest BuffApplied.
                    let TimelineCause::Parent { seq: parent_seq } = entry.cause else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::InvalidBuffEvent,
                            message: "BuffTick recorded without a Parent cause".to_string(),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    let Some(&parent_index) = seq_to_index.get(&parent_seq) else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::InvalidBuffEvent,
                            message: format!(
                                "BuffTick cause seq {} does not reference any entry",
                                parent_seq
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    if parent_index >= index {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::InvalidBuffEvent,
                            message: format!(
                                "BuffTick cause seq {} is not before entry index {}",
                                parent_seq, index
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    }

                    match &timeline.entries[parent_index].event {
                        TimelineEvent::BuffApplied {
                            caster_instance_id: p_caster,
                            target_instance_id: p_target,
                            buff_id: p_buff,
                            ..
                        } if p_caster == caster_instance_id
                            && p_target == target_instance_id
                            && p_buff == buff_id => {}
                        other => {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::InvalidBuffEvent,
                                message: format!(
                                    "BuffTick cause seq {} points to mismatched BuffApplied: {:?}",
                                    parent_seq, other
                                ),
                                entry_index: Some(index),
                            });
                        }
                    }

                    if buffs::get(*buff_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownBuffReference,
                            message: format!(
                                "BuffTick references unknown buff_id {}",
                                buff_id.as_u64()
                            ),
                            entry_index: Some(index),
                        });
                    }
                }

                TimelineEvent::BuffExpired {
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                } => {
                    // Relationship: BuffExpired should be a child of a BuffApplied entry.
                    let TimelineCause::Parent { seq: parent_seq } = entry.cause else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::InvalidBuffEvent,
                            message: "BuffExpired recorded without a Parent cause".to_string(),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    let Some(&parent_index) = seq_to_index.get(&parent_seq) else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::InvalidBuffEvent,
                            message: format!(
                                "BuffExpired cause seq {} does not reference any entry",
                                parent_seq
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    if parent_index >= index {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::InvalidBuffEvent,
                            message: format!(
                                "BuffExpired cause seq {} is not before entry index {}",
                                parent_seq, index
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    }

                    match &timeline.entries[parent_index].event {
                        TimelineEvent::BuffApplied {
                            caster_instance_id: p_caster,
                            target_instance_id: p_target,
                            buff_id: p_buff,
                            ..
                        } if p_caster == caster_instance_id
                            && p_target == target_instance_id
                            && p_buff == buff_id => {}
                        other => {
                            violations.push(TimelineReplayViolation {
                                kind: TimelineReplayViolationKind::InvalidBuffEvent,
                                message: format!(
                                    "BuffExpired cause seq {} points to mismatched BuffApplied: {:?}",
                                    parent_seq, other
                                ),
                                entry_index: Some(index),
                            });
                        }
                    }

                    if buffs::get(*buff_id).is_none() {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::UnknownBuffReference,
                            message: format!(
                                "BuffExpired references unknown buff_id {}",
                                buff_id.as_u64()
                            ),
                            entry_index: Some(index),
                        });
                    }
                }

                TimelineEvent::HpChanged { .. } | TimelineEvent::StatChanged { .. } => {
                    // Outcome events must have a Parent cause pointing to a "cause event".
                    let TimelineCause::Parent { seq: cause_seq } = entry.cause else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::OutcomeMismatch,
                            message: format!(
                                "outcome event missing parent cause: {:?}",
                                entry.event
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    let Some(&cause_index) = seq_to_index.get(&cause_seq) else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::OutcomeMismatch,
                            message: format!(
                                "outcome event cause seq {} does not reference any entry: {:?}",
                                cause_seq, entry.event
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    if cause_index >= index {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::OutcomeMismatch,
                            message: format!(
                                "outcome event cause seq {} is not before entry index {}: {:?}",
                                cause_seq, index, entry.event
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    }

                    let valid_cause = matches!(
                        timeline.entries[cause_index].event,
                        TimelineEvent::AttackStart { .. }
                            | TimelineEvent::AttackResolve { .. }
                            | TimelineEvent::AbilityCast { .. }
                            | TimelineEvent::BuffTick { .. }
                    );
                    if !valid_cause {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::OutcomeMismatch,
                            message: format!(
                                "outcome event cause seq {} points to non-cause event {:?}",
                                cause_seq, timeline.entries[cause_index].event
                            ),
                            entry_index: Some(index),
                        });
                    }
                }

                TimelineEvent::UnitDied { .. } => {
                    // Outcome events must have a Parent cause pointing to a "cause event".
                    let TimelineCause::Parent { seq: cause_seq } = entry.cause else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::OutcomeMismatch,
                            message: format!(
                                "outcome event missing parent cause: {:?}",
                                entry.event
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    let Some(&cause_index) = seq_to_index.get(&cause_seq) else {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::OutcomeMismatch,
                            message: format!(
                                "outcome event cause seq {} does not reference any entry: {:?}",
                                cause_seq, entry.event
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    };

                    if cause_index >= index {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::OutcomeMismatch,
                            message: format!(
                                "outcome event cause seq {} is not before entry index {}: {:?}",
                                cause_seq, index, entry.event
                            ),
                            entry_index: Some(index),
                        });
                        continue;
                    }

                    let valid_cause = matches!(
                        timeline.entries[cause_index].event,
                        TimelineEvent::AttackStart { .. }
                            | TimelineEvent::AttackResolve { .. }
                            | TimelineEvent::AbilityCast { .. }
                            | TimelineEvent::BuffTick { .. }
                    );
                    if !valid_cause {
                        violations.push(TimelineReplayViolation {
                            kind: TimelineReplayViolationKind::OutcomeMismatch,
                            message: format!(
                                "outcome event cause seq {} points to non-cause event {:?}",
                                cause_seq, timeline.entries[cause_index].event
                            ),
                            entry_index: Some(index),
                        });
                    }
                }
            }
        }

        if self.config.validate_autocast_pairing {
            for (caster, (start_seq, due_time_ms)) in expected_autocast_end {
                violations.push(TimelineReplayViolation {
                    kind: TimelineReplayViolationKind::InvalidAutoCastEvent,
                    message: format!(
                        "missing AutoCastEnd for {}: expected at {}ms (cause_seq={})",
                        caster, due_time_ms, start_seq
                    ),
                    entry_index: None,
                });
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}
