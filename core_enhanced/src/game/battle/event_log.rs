//! Battle event log contract.
//!
//! `BattleEventLog` is the append-only log of battle events produced by the
//! current DefenseRoute live battle model. It is not an offline replay driver;
//! it powers battle result records, debug exports, and behavior tests. Live
//! `battle_update.events_delta` payloads are converted to explicit wire DTOs at
//! the transport boundary.

use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};
use uuid::Uuid;

use crate::{
    game::resources::Position,
    game::{
        ability::{
            SkillAreaTickPolicy, SkillAreaTracking, SkillHitTargetFilter, SkillId,
            SkillPresentationDef,
        },
        battle::cooldown::CooldownSource,
        battle::core::movement::{types::EventLogVec2, MovementSegmentEndKind},
        battle::damage::{DamageBreakdown, DamageFeedbackTag, DamageSource, DamageType},
        battle::ids::UnitInstanceId,
        battle::{
            buffs::BuffId,
            types::{
                BattleUnitRole, BattleUnitSourceIdentity, BattleUnitThreatClass, BattleWinner,
                MobilityKind,
            },
        },
        enums::Side,
        stats::{StatModifier, UnitStats},
    },
};

pub const BATTLE_EVENT_LOG_VERSION: u32 = 27;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MovementStopReason {
    TargetAcquired,
    AttackStarted,
    CastStarted,
    HardCC,
    WaitRepath,
    /// Logical arrival: the destination tile is now occupied.
    /// The accompanying continuous coordinates remain the actual simulation-space
    /// stop position and are not snapped for presentation.
    Arrived,
    Died,
    InvalidState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BattleEventRootCause {
    /// Root entries created during battle initialization (spawns, start markers, etc).
    Init,
    /// Root entries emitted by scheduled periodic systems (e.g., auto-attacks).
    Period,
    /// Root entries emitted by the engine/system when there is no meaningful parent seq.
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "cause_type", rename_all = "snake_case")]
pub enum BattleEventCause {
    Parent { seq: u64 },
    Root { kind: BattleEventRootCause },
}

impl BattleEventCause {
    pub fn parent_seq(&self) -> Option<u64> {
        match self {
            Self::Parent { seq } => Some(*seq),
            Self::Root { .. } => None,
        }
    }

    pub fn root_kind(&self) -> Option<BattleEventRootCause> {
        match self {
            Self::Parent { .. } => None,
            Self::Root { kind } => Some(*kind),
        }
    }
}

impl Default for BattleEventCause {
    fn default() -> Self {
        Self::Root {
            kind: BattleEventRootCause::System,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttackKind {
    /// Scheduled auto attack (attack interval based).
    Auto,
    /// Triggered/one-off attack (e.g., ability extra attack).
    Triggered,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttackDelivery {
    Instant,
    Projectile,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BattleProjectileGuidance {
    Homing,
    Fixed,
}

/// Append-only battle event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleEventLog {
    pub version: u32,
    pub entries: Vec<BattleEventLogEntry>,
}

impl BattleEventLog {
    pub fn new() -> Self {
        Self {
            version: BATTLE_EVENT_LOG_VERSION,
            entries: Vec::new(),
        }
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_pretty_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn entries_after_seq(&self, last_seen_seq: u64) -> Vec<BattleEventLogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.seq > last_seen_seq)
            .cloned()
            .collect()
    }

    pub fn write_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    pub fn write_pretty_json<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn read_json<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let event_log = serde_json::from_reader(reader)?;
        Ok(event_log)
    }
}

impl Default for BattleEventLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleEventLogEntry {
    pub time_ms: u64,
    pub seq: u64,
    #[serde(default)]
    pub cause: BattleEventCause,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_command_id: Option<String>,
    pub event: BattleLogEvent,
}

impl BattleEventLogEntry {
    pub fn cause_seq(&self) -> Option<u64> {
        self.cause.parent_seq()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum SkillCastTarget {
    Unit { unit_instance_id: UnitInstanceId },
    Tile { position: Position },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BattleSkillAreaShape {
    TilePattern { affected_tiles: Vec<Position> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BattleLogEvent {
    BattleStart {
        width: u8,
        height: u8,
    },
    ArtifactSpawned {
        artifact_instance_id: Uuid,
        owner: Side,
        base_uuid: Uuid,
    },
    ItemSpawned {
        item_instance_id: Uuid,
        owner: Side,
        owner_unit_instance_id: UnitInstanceId,
        base_uuid: Uuid,
    },
    UnitSpawned {
        unit_instance_id: UnitInstanceId,
        owner: Side,
        #[serde(default)]
        role: BattleUnitRole,
        #[serde(default)]
        threat_class: BattleUnitThreatClass,
        #[serde(default)]
        mobility_kind: MobilityKind,
        base_uuid: Uuid,
        unit_source: BattleUnitSourceIdentity,
        world_position: EventLogVec2,
        stats: UnitStats,
    },
    UnitDeployed {
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
        position: Position,
        facing: crate::game::battle::tile_range::FacingDirection,
    },
    UnitWithdrawn {
        unit_instance_id: UnitInstanceId,
        world_position: EventLogVec2,
        position: Position,
    },
    MovementSegmentStarted {
        unit_instance_id: UnitInstanceId,
        start: EventLogVec2,
        target: EventLogVec2,
        started_at_ms: u64,
        ends_at_ms: u64,
        end_kind: MovementSegmentEndKind,
    },
    MovementStopped {
        unit_instance_id: UnitInstanceId,
        reason: MovementStopReason,
        world_position: EventLogVec2,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        until_ms: Option<u64>,
    },
    AttackStart {
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery: Option<AttackDelivery>,
    },
    AttackResolve {
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery: Option<AttackDelivery>,
    },
    AttackMiss {
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery: Option<AttackDelivery>,
    },
    BasicAttackProjectileLaunched {
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        start: EventLogVec2,
        aim: EventLogVec2,
        fired_at_ms: u64,
        expected_impact_time_ms: u64,
    },
    BasicAttackProjectileImpacted {
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        impact_position: EventLogVec2,
        hit: bool,
    },
    AutoCastStart {
        caster_instance_id: UnitInstanceId,
        skill_id: Option<SkillId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
    },
    AutoCastEnd {
        caster_instance_id: UnitInstanceId,
    },
    ManualCastStart {
        caster_instance_id: UnitInstanceId,
        skill_id: SkillId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
    },
    ManualCastEnd {
        caster_instance_id: UnitInstanceId,
    },
    SkillCastInterrupted {
        interrupter_instance_id: UnitInstanceId,
        caster_instance_id: UnitInstanceId,
        interrupted_skill_id: SkillId,
        interrupted_cast_seq: u64,
    },
    SkillCastCancelled {
        caster_instance_id: UnitInstanceId,
        interrupted_skill_id: SkillId,
        interrupted_cast_seq: u64,
        reason: SkillCastCancelReason,
    },
    TriggeredAbilityProc {
        skill_id: SkillId,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_instance_id: Option<UnitInstanceId>,
        activation_source: CooldownSource,
        binding_index: usize,
    },
    AbilityCast {
        skill_id: SkillId,
        caster_instance_id: UnitInstanceId,
        target_instance_id: Option<UnitInstanceId>,
    },
    AbilityStepTriggered {
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        target_instance_id: Option<UnitInstanceId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        presentation: Option<SkillPresentationDef>,
    },
    SkillAreaDeclared {
        area_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
        shape: BattleSkillAreaShape,
        /// Tile area events use `shape.affected_tiles` as their gameplay and
        /// presentation source of truth. `origin`, `center`, and
        /// `direction_hint` are retained for VFX anchoring.
        origin: EventLogVec2,
        center: EventLogVec2,
        direction_hint: EventLogVec2,
        start_time_ms: u64,
        duration_ms: u32,
        /// Presentation lifetime. Instant gameplay areas use a short non-zero
        /// value so debug visuals are visible without changing simulation rules.
        display_duration_ms: u32,
        #[serde(default)]
        warning_ms: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tick_interval_ms: Option<u32>,
        tick_policy: SkillAreaTickPolicy,
        #[serde(default)]
        tracking: SkillAreaTracking,
        hit_targets: SkillHitTargetFilter,
        include_caster: bool,
    },
    SkillProjectileLaunched {
        delivery_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
        guidance: BattleProjectileGuidance,
        start: EventLogVec2,
        aim: EventLogVec2,
        fired_at_ms: u64,
        expected_end_time_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projectile_vfx_id: Option<String>,
    },
    SkillProjectileImpacted {
        delivery_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        first_hit_unit_id: Option<UnitInstanceId>,
        impact_position: EventLogVec2,
        terminal: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        impact_vfx_id: Option<String>,
    },
    BuffApplied {
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        duration_ms: u64,
    },
    BuffTick {
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
    },
    BuffExpired {
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        reason: BuffExpireReason,
    },
    HpChanged {
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        delta: i32,
        hp_before: u32,
        hp_after: u32,
        reason: HpChangeReason,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_source: Option<DamageSource>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_type: Option<DamageType>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        raw_damage: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        final_damage: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_breakdown: Option<Vec<DamageBreakdown>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        critical: Option<bool>,
        #[serde(default)]
        feedback_tags: Vec<DamageFeedbackTag>,
    },
    StatChanged {
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        modifier: StatModifier,
        stats_before: UnitStats,
        stats_after: UnitStats,
    },
    ResonanceChanged {
        unit_instance_id: UnitInstanceId,
        before: u32,
        after: u32,
        max: u32,
    },
    UnitDied {
        unit_instance_id: UnitInstanceId,
        owner: Side,
        killer_instance_id: Option<UnitInstanceId>,
        world_position: EventLogVec2,
        position: Position,
    },
    BattleEnd {
        winner: BattleWinner,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffExpireReason {
    Natural,
    Replaced,
    TargetDied,
    CasterDied,
    TargetWithdrawn,
    CasterWithdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillCastCancelReason {
    Withdrawn,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HpChangeReason {
    BasicAttack,
    Command,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hp_changed_serializes_feedback_tags_as_snake_case_array() {
        let source_instance_id = UnitInstanceId::from(Uuid::from_u128(0xA));
        let target_instance_id = UnitInstanceId::from(Uuid::from_u128(0xB));
        let event = BattleLogEvent::HpChanged {
            source_instance_id: Some(source_instance_id),
            target_instance_id,
            delta: -10,
            hp_before: 100,
            hp_after: 90,
            reason: HpChangeReason::Command,
            damage_source: Some(DamageSource::Ability),
            damage_type: Some(DamageType::Physical),
            raw_damage: Some(20),
            final_damage: Some(10),
            damage_breakdown: None,
            critical: Some(true),
            feedback_tags: vec![DamageFeedbackTag::Critical, DamageFeedbackTag::Mitigated],
        };

        let value = serde_json::to_value(event).expect("serialize HpChanged");

        assert_eq!(value["type"], "HpChanged");
        assert_eq!(
            value["feedback_tags"],
            serde_json::json!(["critical", "mitigated"])
        );
    }
}
