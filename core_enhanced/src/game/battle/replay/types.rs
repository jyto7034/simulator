use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::SkillId,
        battle::{
            buffs::BuffId,
            cooldown::CooldownSource,
            core::movement::{types::WorldVec2, MovementSegmentEndKind},
            ids::UnitInstanceId,
            timeline::{AttackKind, HpChangeReason, TimelineEvent},
        },
        data::GameDataBase,
        enums::Side,
        stats::{StatModifier, UnitStats},
    },
};

#[derive(Debug, Clone)]
pub struct AbilityExecutor;

#[derive(Debug, Clone, Copy)]
pub struct PendingScheduledAttack {
    pub attacker_instance_id: UnitInstanceId,
    pub earliest_time_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifiedCauseKind {
    Attack,
    Ability,
    BuffTick,
}

#[derive(Debug, Clone)]
pub enum ExpectedDecision {
    AbilityCast {
        time_ms: u64,
        skill_id: SkillId,
        caster_instance_id: UnitInstanceId,
        target_instance_id: Option<UnitInstanceId>,
        cooldown_source: CooldownSource,
    },
    Attack {
        time_ms: u64,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        kind: AttackKind,
    },
    BuffApplied {
        time_ms: u64,
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        duration_ms: u64,
    },
}

impl ExpectedDecision {
    pub fn matches(&self, time_ms: u64, event: &TimelineEvent) -> bool {
        match (self, event) {
            (
                ExpectedDecision::AbilityCast {
                    time_ms: expected_time,
                    skill_id,
                    caster_instance_id,
                    target_instance_id,
                    ..
                },
                TimelineEvent::AbilityCast {
                    skill_id: actual_skill,
                    caster_instance_id: actual_caster,
                    target_instance_id: actual_target,
                },
            ) => {
                *expected_time == time_ms
                    && *skill_id == *actual_skill
                    && *caster_instance_id == *actual_caster
                    && *target_instance_id == *actual_target
            }
            (
                ExpectedDecision::Attack {
                    time_ms: expected_time,
                    attacker_instance_id,
                    target_instance_id,
                    kind,
                },
                TimelineEvent::AttackStart {
                    attacker_instance_id: actual_attacker,
                    target_instance_id: actual_target,
                    kind: actual_kind,
                    ..
                },
            ) => {
                *expected_time == time_ms
                    && *attacker_instance_id == *actual_attacker
                    && *target_instance_id == *actual_target
                    && Some(*kind) == *actual_kind
            }
            (
                ExpectedDecision::BuffApplied {
                    time_ms: expected_time,
                    caster_instance_id,
                    target_instance_id,
                    buff_id,
                    duration_ms,
                },
                TimelineEvent::BuffApplied {
                    caster_instance_id: actual_caster,
                    target_instance_id: actual_target,
                    buff_id: actual_buff,
                    duration_ms: actual_duration,
                },
            ) => {
                *expected_time == time_ms
                    && *caster_instance_id == *actual_caster
                    && *target_instance_id == *actual_target
                    && *buff_id == *actual_buff
                    && *duration_ms == *actual_duration
            }
            _ => false,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            ExpectedDecision::AbilityCast {
                time_ms,
                skill_id,
                caster_instance_id,
                target_instance_id,
                cooldown_source,
            } => format!(
                "AbilityCast(time_ms={}, skill_id={:?}, caster={}, target={:?}, cooldown_source={:?})",
                time_ms, skill_id, caster_instance_id, target_instance_id, cooldown_source
            ),
            ExpectedDecision::Attack {
                time_ms,
                attacker_instance_id,
                target_instance_id,
                kind,
            } => format!(
                "Attack(time_ms={}, attacker={}, target={}, kind={:?})",
                time_ms, attacker_instance_id, target_instance_id, kind
            ),
            ExpectedDecision::BuffApplied {
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
            } => format!(
                "BuffApplied(time_ms={}, caster={}, target={}, buff_id={}, duration_ms={})",
                time_ms,
                caster_instance_id,
                target_instance_id,
                buff_id.as_u64(),
                duration_ms
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpectedOutcome {
    HpChanged {
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        delta: i32,
        hp_before: u32,
        hp_after: u32,
        reason: HpChangeReason,
    },
    StatChanged {
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        modifier: StatModifier,
        stats_before: UnitStats,
        stats_after: UnitStats,
    },
    UnitDied {
        unit_instance_id: UnitInstanceId,
        owner: Side,
        killer_instance_id: Option<UnitInstanceId>,
    },
}

impl ExpectedOutcome {
    pub fn matches(&self, event: &TimelineEvent) -> bool {
        match (self, event) {
            (
                ExpectedOutcome::HpChanged {
                    source_instance_id,
                    target_instance_id,
                    delta,
                    hp_before,
                    hp_after,
                    reason,
                },
                TimelineEvent::HpChanged {
                    source_instance_id: actual_source,
                    target_instance_id: actual_target,
                    delta: actual_delta,
                    hp_before: actual_before,
                    hp_after: actual_after,
                    reason: actual_reason,
                    ..
                },
            ) => {
                *source_instance_id == *actual_source
                    && *target_instance_id == *actual_target
                    && *delta == *actual_delta
                    && *hp_before == *actual_before
                    && *hp_after == *actual_after
                    && *reason == *actual_reason
            }
            (
                ExpectedOutcome::StatChanged {
                    source_instance_id,
                    target_instance_id,
                    modifier,
                    stats_before,
                    stats_after,
                },
                TimelineEvent::StatChanged {
                    source_instance_id: actual_source,
                    target_instance_id: actual_target,
                    modifier: actual_modifier,
                    stats_before: actual_before,
                    stats_after: actual_after,
                },
            ) => {
                *source_instance_id == *actual_source
                    && *target_instance_id == *actual_target
                    && modifier.stat == actual_modifier.stat
                    && modifier.kind == actual_modifier.kind
                    && modifier.value == actual_modifier.value
                    && stats_before.max_health == actual_before.max_health
                    && stats_before.current_health == actual_before.current_health
                    && stats_before.attack == actual_before.attack
                    && stats_before.defense == actual_before.defense
                    && stats_before.magic_resist == actual_before.magic_resist
                    && stats_before.attack_interval_ms == actual_before.attack_interval_ms
                    && stats_after.max_health == actual_after.max_health
                    && stats_after.current_health == actual_after.current_health
                    && stats_after.attack == actual_after.attack
                    && stats_after.defense == actual_after.defense
                    && stats_after.magic_resist == actual_after.magic_resist
                    && stats_after.attack_interval_ms == actual_after.attack_interval_ms
            }
            (
                ExpectedOutcome::UnitDied {
                    unit_instance_id,
                    owner,
                    killer_instance_id,
                },
                TimelineEvent::UnitDied {
                    unit_instance_id: actual_unit,
                    owner: actual_owner,
                    killer_instance_id: actual_killer,
                },
            ) => {
                *unit_instance_id == *actual_unit
                    && *owner == *actual_owner
                    && *killer_instance_id == *actual_killer
            }
            _ => false,
        }
    }

    pub fn describe(&self) -> String {
        match self {
            ExpectedOutcome::HpChanged {
                source_instance_id,
                target_instance_id,
                delta,
                hp_before,
                hp_after,
                reason,
            } => format!(
                "HpChanged(source={:?}, target={}, delta={}, before={}, after={}, reason={:?})",
                source_instance_id, target_instance_id, delta, hp_before, hp_after, reason
            ),
            ExpectedOutcome::StatChanged {
                target_instance_id,
                modifier,
                ..
            } => format!(
                "StatChanged(target={}, modifier={:?}:{:?} {})",
                target_instance_id, modifier.stat, modifier.kind, modifier.value
            ),
            ExpectedOutcome::UnitDied {
                unit_instance_id,
                owner,
                killer_instance_id,
            } => format!(
                "UnitDied(unit={}, owner={:?}, killer={:?})",
                unit_instance_id, owner, killer_instance_id
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeUnit {
    pub instance_id: UnitInstanceId,
    pub owner: Side,
    pub base_uuid: Uuid,
    pub stats: UnitStats,
    pub position: Position,
    pub current_target: Option<UnitInstanceId>,
    pub resonance_current: u32,
    pub resonance_max: u32,
    pub resonance_lock_ms: u64,
    pub resonance_gain_locked_until_ms: u64,
    pub casting_until_ms: u64,
    pub pending_cast: bool,
}

#[derive(Debug, Clone)]
pub struct RuntimeArtifact {
    pub instance_id: Uuid,
    pub owner: Side,
    pub base_uuid: Uuid,
}

#[derive(Debug, Clone)]
pub struct RuntimeItem {
    pub instance_id: Uuid,
    pub owner_unit_instance: UnitInstanceId,
    pub base_uuid: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BuffInstanceKey {
    pub caster_instance_id: UnitInstanceId,
    pub target_instance_id: UnitInstanceId,
    pub buff_id: BuffId,
}

#[derive(Debug, Clone)]
pub struct ActiveBuff {
    pub stacks: u8,
    pub expires_at_ms: u64,
    pub next_tick_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum TriggerSource {
    Artifact { side: Side },
    Item { unit_instance_id: UnitInstanceId },
}

pub struct ReplayState {
    pub game_data: Arc<GameDataBase>,
    pub units: HashMap<UnitInstanceId, RuntimeUnit>,
    pub known_units: HashSet<UnitInstanceId>,
    pub artifacts: HashMap<Uuid, RuntimeArtifact>,
    pub items: HashMap<Uuid, RuntimeItem>,
    pub buffs: HashMap<BuffInstanceKey, ActiveBuff>,
}

impl ReplayState {
    pub fn new(game_data: Arc<GameDataBase>) -> Self {
        Self {
            game_data,
            units: HashMap::new(),
            known_units: HashSet::new(),
            artifacts: HashMap::new(),
            items: HashMap::new(),
            buffs: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineReplayViolationKind {
    TimelineVersionMismatch,
    DuplicateSeq,
    AttackKindMissing,
    UnknownUnitReference,
    UnknownUnitBaseReference,
    UnknownItemReference,
    UnknownArtifactReference,
    UnknownBuffReference,
    AttackDuringCast,
    AutoCastWithoutFullResonance,
    ExpectedOutcomeMissing,
    ExpectedDecisionMissing,
    UnexpectedDecision,
    UnexpectedOutcome,
    OutcomeMismatch,
    InvalidBuffEvent,
    InvalidAutoCastEvent,
}

#[derive(Debug, Clone)]
pub struct TimelineReplayViolation {
    pub kind: TimelineReplayViolationKind,
    pub message: String,
    pub entry_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TimelineReplayerConfig {
    pub validate_ability_outcomes: bool,
    pub validate_buff_tick_outcomes: bool,
    pub validate_autocast_gating: bool,
    pub validate_autocast_pairing: bool,
    pub validate_expected_decisions: bool,
    pub validate_unit_base_uuid: bool,
    pub forbid_unexpected_outcomes_for_verified_causes: bool,
}

impl Default for TimelineReplayerConfig {
    fn default() -> Self {
        Self {
            validate_ability_outcomes: true,
            validate_buff_tick_outcomes: true,
            validate_autocast_gating: true,
            validate_autocast_pairing: true,
            validate_expected_decisions: true,
            validate_unit_base_uuid: false,
            forbid_unexpected_outcomes_for_verified_causes: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReplayMovementSegment {
    pub unit_instance_id: UnitInstanceId,
    pub start: WorldVec2,
    pub target: WorldVec2,
    pub started_at_ms: u64,
    pub ends_at_ms: u64,
    pub end_kind: MovementSegmentEndKind,
}

impl ReplayMovementSegment {
    pub fn from_timeline_event(event: &TimelineEvent) -> Option<Self> {
        let TimelineEvent::MovementSegmentStarted {
            unit_instance_id,
            start,
            target,
            started_at_ms,
            ends_at_ms,
            end_kind,
        } = event
        else {
            return None;
        };

        Some(Self {
            unit_instance_id: *unit_instance_id,
            start: start.to_world(),
            target: target.to_world(),
            started_at_ms: *started_at_ms,
            ends_at_ms: *ends_at_ms,
            end_kind: *end_kind,
        })
    }

    pub fn sample_position_at(&self, time_ms: u64) -> WorldVec2 {
        if self.ends_at_ms <= self.started_at_ms || time_ms <= self.started_at_ms {
            return self.start;
        }
        if time_ms >= self.ends_at_ms {
            return self.target;
        }

        let elapsed = (time_ms - self.started_at_ms) as f32;
        let duration = (self.ends_at_ms - self.started_at_ms) as f32;
        let t = elapsed / duration;

        self.start + (self.target - self.start) * t
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayUnitSpatialState {
    pub settled_position: WorldVec2,
    pub active_segment: Option<ReplayMovementSegment>,
}

impl ReplayUnitSpatialState {
    pub fn new_spawned(position: WorldVec2) -> Self {
        Self {
            settled_position: position,
            active_segment: None,
        }
    }

    pub fn apply_event(&mut self, event: &TimelineEvent) {
        match event {
            TimelineEvent::MovementSegmentStarted { .. } => {
                if let Some(segment) = ReplayMovementSegment::from_timeline_event(event) {
                    self.active_segment = Some(segment);
                }
            }
            TimelineEvent::MovementStopped { world_position, .. } => {
                self.settled_position = world_position.to_world();
                self.active_segment = None;
            }
            _ => {}
        }
    }

    pub fn sample_position_at(&self, time_ms: u64) -> WorldVec2 {
        self.active_segment
            .map(|segment| segment.sample_position_at(time_ms))
            .unwrap_or(self.settled_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::core::movement::types::TimelineVec2;

    #[test]
    fn replay_movement_segment_samples_linearly() {
        let segment = ReplayMovementSegment {
            unit_instance_id: Uuid::from_u128(1).into(),
            start: WorldVec2::new(2.0, 3.0),
            target: WorldVec2::new(2.0, 2.5),
            started_at_ms: 100,
            ends_at_ms: 150,
            end_kind: MovementSegmentEndKind::Boundary,
        };

        assert_eq!(segment.sample_position_at(100), WorldVec2::new(2.0, 3.0));
        assert_eq!(segment.sample_position_at(125), WorldVec2::new(2.0, 2.75));
        assert_eq!(segment.sample_position_at(150), WorldVec2::new(2.0, 2.5));
    }

    #[test]
    fn replay_unit_spatial_state_uses_segment_then_stop_snapshot() {
        let mut state = ReplayUnitSpatialState::new_spawned(WorldVec2::new(2.0, 3.0));
        state.apply_event(&TimelineEvent::MovementSegmentStarted {
            unit_instance_id: Uuid::from_u128(2).into(),
            start: TimelineVec2 {
                x_milli: 2_000,
                y_milli: 3_000,
            },
            target: TimelineVec2 {
                x_milli: 2_000,
                y_milli: 2_500,
            },
            started_at_ms: 100,
            ends_at_ms: 150,
            end_kind: MovementSegmentEndKind::Boundary,
        });

        assert_eq!(state.sample_position_at(125), WorldVec2::new(2.0, 2.75));

        state.apply_event(&TimelineEvent::MovementStopped {
            unit_instance_id: Uuid::from_u128(2).into(),
            reason: crate::game::battle::timeline::MovementStopReason::Arrived,
            world_position: TimelineVec2 {
                x_milli: 2_000,
                y_milli: 2_500,
            },
            until_ms: None,
        });

        assert_eq!(state.sample_position_at(151), WorldVec2::new(2.0, 2.5));
        assert!(state.active_segment.is_none());
    }

    #[test]
    fn replay_unit_spatial_state_uses_spawn_world_position_without_segment() {
        let state = ReplayUnitSpatialState::new_spawned(WorldVec2::new(1.25, 1.75));
        assert_eq!(state.sample_position_at(0), WorldVec2::new(1.25, 1.75));
    }
}
