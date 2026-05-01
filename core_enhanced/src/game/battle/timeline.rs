use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::{
            SkillAreaShapeDef, SkillAreaTickPolicy, SkillHitTargetFilter, SkillId,
            SkillPresentationDef,
        },
        battle::cooldown::CooldownSource,
        battle::core::movement::{types::TimelineVec2, MovementSegmentEndKind},
        battle::damage::{DamageBreakdown, DamageSource, DamageType},
        battle::ids::UnitInstanceId,
        battle::{buffs::BuffId, types::BattleWinner},
        enums::Side,
        stats::{StatModifier, UnitStats},
    },
};

pub const TIMELINE_VERSION: u32 = 20;

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
pub enum TimelineRootCause {
    /// Timeline root entries created during battle initialization (spawns, start markers, etc).
    Init,
    /// Timeline root entries emitted by scheduled periodic systems (e.g., auto-attacks).
    Period,
    /// Timeline root entries emitted by the engine/system when there is no meaningful parent seq.
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "cause_type", rename_all = "snake_case")]
pub enum TimelineCause {
    Parent { seq: u64 },
    Root { kind: TimelineRootCause },
}

impl TimelineCause {
    pub fn parent_seq(&self) -> Option<u64> {
        match self {
            Self::Parent { seq } => Some(*seq),
            Self::Root { .. } => None,
        }
    }

    pub fn root_kind(&self) -> Option<TimelineRootCause> {
        match self {
            Self::Parent { .. } => None,
            Self::Root { kind } => Some(*kind),
        }
    }
}

impl Default for TimelineCause {
    fn default() -> Self {
        Self::Root {
            kind: TimelineRootCause::System,
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
pub enum TimelineProjectileGuidance {
    Homing,
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub version: u32,
    pub entries: Vec<TimelineEntry>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            version: TIMELINE_VERSION,
            entries: Vec::new(),
        }
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_pretty_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
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
        let timeline = serde_json::from_reader(reader)?;
        Ok(timeline)
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub time_ms: u64,
    pub seq: u64,
    #[serde(default)]
    pub cause: TimelineCause,
    pub event: TimelineEvent,
}

impl TimelineEntry {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimelineSkillAreaShape {
    Circle {
        radius_units: u32,
    },
    Line {
        length_units: u32,
    },
    Box {
        width_units: u32,
        height_units: u32,
    },
    Rectangle {
        width_units: u32,
        length_units: u32,
    },
    /// Triangle wedge semantics, not a circular sector.
    /// `length_units` is the centerline reach used for visual aiming.
    Cone {
        angle_degrees: u16,
        length_units: u32,
    },
}

impl From<SkillAreaShapeDef> for TimelineSkillAreaShape {
    fn from(shape: SkillAreaShapeDef) -> Self {
        match shape {
            SkillAreaShapeDef::Circle { radius_units } => Self::Circle { radius_units },
            SkillAreaShapeDef::Line { length_units } => Self::Line { length_units },
            SkillAreaShapeDef::Box {
                width_units,
                height_units,
            } => Self::Box {
                width_units,
                height_units,
            },
            SkillAreaShapeDef::Rectangle {
                width_units,
                length_units,
            } => Self::Rectangle {
                width_units,
                length_units,
            },
            SkillAreaShapeDef::Cone {
                angle_degrees,
                length_units,
            } => Self::Cone {
                angle_degrees,
                length_units,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TimelineEvent {
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
        base_uuid: Uuid,
        world_position: TimelineVec2,
        stats: UnitStats,
    },
    MovementSegmentStarted {
        unit_instance_id: UnitInstanceId,
        start: TimelineVec2,
        target: TimelineVec2,
        started_at_ms: u64,
        ends_at_ms: u64,
        end_kind: MovementSegmentEndKind,
    },
    MovementStopped {
        unit_instance_id: UnitInstanceId,
        reason: MovementStopReason,
        world_position: TimelineVec2,
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
    ProjectileMiss {
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
    },
    BasicAttackProjectileLaunched {
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        start: TimelineVec2,
        aim: TimelineVec2,
        fired_at_ms: u64,
        expected_impact_time_ms: u64,
    },
    BasicAttackProjectileImpacted {
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        impact_position: TimelineVec2,
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
        shape: TimelineSkillAreaShape,
        /// Directional shapes use `origin` as the sweep start.
        /// Centered shapes use `center` as the visual center.
        origin: TimelineVec2,
        center: TimelineVec2,
        /// Absolute aim point. Unity should derive the direction vector from
        /// `direction_hint - origin` for line/rectangle/cone visuals.
        direction_hint: TimelineVec2,
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
        guidance: TimelineProjectileGuidance,
        start: TimelineVec2,
        aim: TimelineVec2,
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
        impact_position: TimelineVec2,
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
    },
    BattleEnd {
        winner: BattleWinner,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HpChangeReason {
    BasicAttack,
    Command,
}
