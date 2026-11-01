pub mod action;
pub mod condition;
pub mod counter_channel;
pub mod death_burst;
pub mod equipment_attack;
pub mod conditional_damage;
pub mod equipment_defense;
pub mod equipment_forms;
pub mod equipment_state;
pub mod equipment_support;
pub mod flow;
pub mod ground;
pub mod guard_charge;
pub mod hostile_cohort;
pub mod input_plan;
pub mod marks;
pub mod mobile_effect;
pub mod objects;
pub mod paired_areas;
pub mod prepared_volley;
pub mod relation_action;
pub mod resonance;
pub mod state_program;
pub mod status_strike;
pub mod sustained;
pub mod tile_burst;
pub mod weapon_marks;
pub mod weapon_stages;
use std::{borrow::Borrow, fmt, ops::Deref};

pub mod authoring;

use serde::{Deserialize, Serialize};

use crate::game::{battle::tile_range::TileRangePattern, stats::TriggerType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SkillId(String);

impl SkillId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn retained_text_capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl From<&str> for SkillId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SkillId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for SkillId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for SkillId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Borrow<str> for SkillId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<str> for SkillId {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for SkillId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillKind {
    #[default]
    Targeted,
    Untargeted,
}

/// Runtime activation contract for a unit's active skill.
///
/// Skill effect data stays independent from this mode. The same skill runtime can
/// be driven by player input, automatic resonance, or explicit trigger hooks
/// depending on who owns the skill in the current battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillActivationMode {
    Manual,
    #[default]
    Auto,
    Triggered,
}

/// 집중(focus) 동안 허용되는 행동.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FocusPermissions {
    #[serde(default)]
    pub allows_move: bool,
    #[serde(default)]
    pub allows_basic_attack: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UnitTargetRule {
    #[default]
    Nearest,
    CurrentTarget,
    LowestHealthEnemy,
    RouteProgress,
}

/// Unit-target acquisition reach for skills.
///
/// This is deliberately separate from basic-attack `TileRangePolicy` and from
/// `DeliveryDef::TileArea.footprint`: global reach selects one valid unit; it
/// never declares affected tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillTargetReach {
    #[default]
    Pattern,
    Global,
}

/// Who selects the cast-level target. This remains independent from
/// `SkillTarget`, which describes effect targeting semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCastSelection {
    #[serde(rename = "self")]
    SelfUnit,
    AutomaticUnit,
    PlayerUnit,
    PlayerTile,
    /// Ordered, atomically admitted tile pair; equal centers are valid.
    PlayerTilePair,
    PlayerDirection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillTarget {
    SelfUnit,
    /// A living ally, including the caster; automatic selection prefers lower HP ratio.
    AllySingle,
    EnemySingle {
        rule: UnitTargetRule,
    },
    /// Reuse the cast-level target/anchor for delayed spatial deliveries.
    ///
    /// This is intentionally not a range expression. Actual multi-hit area
    /// filtering belongs to `DeliveryDef::TileArea.footprint` plus the delivery
    /// hit filter. DefenseRoute official live skills use `DeliveryDef::TileArea`.
    CastTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProjectileHitPolicy {
    #[default]
    TargetLocked,
    DirectionalCollision,
}

fn default_projectile_hit_policy() -> ProjectileHitPolicy {
    ProjectileHitPolicy::TargetLocked
}

fn default_false() -> bool {
    false
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub enum DeliveryDef {
    #[default]
    Instant,
    Projectile {
        speed_units_per_ms: u32,
        #[serde(default = "default_projectile_hit_policy")]
        hit_policy: ProjectileHitPolicy,
        #[serde(default = "default_false")]
        allow_targetless_cast: bool,
        #[serde(default)]
        max_range_tiles: Option<u32>,
        #[serde(default)]
        max_lifetime_ms: Option<u64>,
        #[serde(default)]
        max_kills: Option<u32>,
        #[serde(default)]
        max_pierces: Option<u32>,
        #[serde(default)]
        collision: SkillProjectileCollisionDef,
    },
    TileArea {
        footprint: TileRangePattern,
        area: SkillTileAreaDeliveryDef,
    },
}

/// Continuous collision filter for skill projectile / area delivery.
///
/// This is intentionally kept separate from `SkillTarget`:
/// - `SkillTarget` answers "who is the step trying to affect?"
/// - `SkillHitTargetFilter` answers "who can this spatial delivery collide with?"
///
/// Runtime collision filter for explicitly spatial skill deliveries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillHitTargetFilter {
    Allies,
    #[default]
    Enemies,
    Any,
}

fn default_projectile_collision_radius_units() -> u32 {
    125_000
}

fn deserialize_nonzero_option_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<u32>::deserialize(deserializer)?;
    match value {
        Some(0) => Err(serde::de::Error::custom(
            "expected non-zero tick_interval_ms",
        )),
        other => Ok(other),
    }
}

/// Continuous collision contract for a skill projectile.
///
/// This applies to skill-delivered projectiles. Basic attacks use their own
/// active projectile collision path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillProjectileCollisionDef {
    #[serde(default = "default_projectile_collision_radius_units")]
    pub radius_units: u32,
    #[serde(default)]
    pub hit_targets: SkillHitTargetFilter,
    #[serde(default)]
    pub piercing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub despawn_on_hit: Option<bool>,
    #[serde(default)]
    pub max_hits: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeliveryValidationError {
    pub field_suffix: &'static str,
    pub cause: &'static str,
}

impl DeliveryValidationError {
    fn new(field_suffix: &'static str, cause: &'static str) -> Self {
        Self {
            field_suffix,
            cause,
        }
    }
}

impl DeliveryDef {
    pub(crate) fn try_validate_projectile_scalar_contract(
        &self,
    ) -> Result<(), DeliveryValidationError> {
        let Self::Projectile {
            speed_units_per_ms,
            max_range_tiles,
            max_lifetime_ms,
            max_kills,
            max_pierces,
            collision,
            ..
        } = self
        else {
            return Ok(());
        };
        if *speed_units_per_ms == 0 {
            return Err(DeliveryValidationError::new(
                "Projectile.speed_units_per_ms",
                "projectile speed_units_per_ms must be greater than zero",
            ));
        }
        if *max_range_tiles == Some(0) {
            return Err(DeliveryValidationError::new(
                "Projectile.max_range_tiles",
                "projectile max_range_tiles must be greater than zero",
            ));
        }
        if *max_lifetime_ms == Some(0) {
            return Err(DeliveryValidationError::new(
                "Projectile.max_lifetime_ms",
                "projectile max_lifetime_ms must be greater than zero",
            ));
        }
        if *max_kills == Some(0) {
            return Err(DeliveryValidationError::new(
                "Projectile.max_kills",
                "projectile max_kills must be greater than zero",
            ));
        }
        if *max_pierces == Some(0) {
            return Err(DeliveryValidationError::new(
                "Projectile.max_pierces",
                "projectile max_pierces must be greater than zero",
            ));
        }
        if collision.max_hits == Some(0) {
            return Err(DeliveryValidationError::new(
                "Projectile.collision.max_hits",
                "projectile collision max_hits must be greater than zero",
            ));
        }
        Ok(())
    }

    /// The existing spatial basic-projectile consumer is shared by weapon
    /// profiles and finite overlays. Target-locked legacy consumers keep their
    /// narrower contract below.
    pub(crate) fn try_validate_directional_basic_attack_contract(
        &self,
    ) -> Result<(), DeliveryValidationError> {
        self.try_validate_projectile_scalar_contract()?;
        let Self::Projectile {
            hit_policy: ProjectileHitPolicy::DirectionalCollision,
            allow_targetless_cast,
            max_range_tiles,
            max_lifetime_ms,
            collision,
            ..
        } = self
        else {
            return Err(DeliveryValidationError::new(
                "Projectile.hit_policy",
                "directional basic projectile required",
            ));
        };
        if *allow_targetless_cast {
            return Err(DeliveryValidationError::new(
                "Projectile.allow_targetless_cast",
                "directional basic requires a target aim",
            ));
        }
        if max_range_tiles.is_none() {
            return Err(DeliveryValidationError::new(
                "Projectile.max_range_tiles",
                "directional basic requires finite range",
            ));
        }
        if max_lifetime_ms.is_none() {
            return Err(DeliveryValidationError::new(
                "Projectile.max_lifetime_ms",
                "directional basic requires finite lifetime",
            ));
        }
        if collision.hit_targets != SkillHitTargetFilter::Enemies {
            return Err(DeliveryValidationError::new(
                "Projectile.collision.hit_targets",
                "directional basic requires enemy collision",
            ));
        }
        Ok(())
    }

    pub(crate) fn try_validate_basic_attack_consumer_contract(
        &self,
    ) -> Result<(), DeliveryValidationError> {
        self.try_validate_projectile_scalar_contract()?;
        let Self::Projectile {
            hit_policy,
            allow_targetless_cast,
            max_range_tiles,
            max_lifetime_ms,
            max_kills,
            max_pierces,
            collision,
            ..
        } = self
        else {
            return match self {
                Self::Instant => Ok(()),
                Self::TileArea { .. } => Err(DeliveryValidationError::new(
                    "TileArea",
                    "TileArea delivery is skill-only for a basic-attack consumer",
                )),
                Self::Projectile { .. } => unreachable!(),
            };
        };
        if *hit_policy != ProjectileHitPolicy::TargetLocked {
            return Err(DeliveryValidationError::new(
                "Projectile.hit_policy",
                "DirectionalCollision is skill-only for a basic-attack consumer",
            ));
        }
        if *allow_targetless_cast {
            return Err(DeliveryValidationError::new(
                "Projectile.allow_targetless_cast",
                "allow_targetless_cast is skill-only for a basic-attack consumer",
            ));
        }
        for (present, field_suffix, cause) in [
            (
                max_range_tiles.is_some(),
                "Projectile.max_range_tiles",
                "max_range_tiles is skill-only for a basic-attack consumer",
            ),
            (
                max_lifetime_ms.is_some(),
                "Projectile.max_lifetime_ms",
                "max_lifetime_ms is skill-only for a basic-attack consumer",
            ),
            (
                max_kills.is_some(),
                "Projectile.max_kills",
                "max_kills is skill-only for a basic-attack consumer",
            ),
            (
                max_pierces.is_some(),
                "Projectile.max_pierces",
                "max_pierces is skill-only for a basic-attack consumer",
            ),
        ] {
            if present {
                return Err(DeliveryValidationError::new(field_suffix, cause));
            }
        }
        let default_collision = SkillProjectileCollisionDef::default();
        for (nondefault, field_suffix, cause) in [
            (
                collision.radius_units != default_collision.radius_units,
                "Projectile.collision.radius_units",
                "collision.radius_units is skill-only for a basic-attack consumer",
            ),
            (
                collision.hit_targets != default_collision.hit_targets,
                "Projectile.collision.hit_targets",
                "collision.hit_targets is skill-only for a basic-attack consumer",
            ),
            (
                collision.piercing != default_collision.piercing,
                "Projectile.collision.piercing",
                "collision.piercing is skill-only for a basic-attack consumer",
            ),
            (
                collision.despawn_on_hit.is_some(),
                "Projectile.collision.despawn_on_hit",
                "collision.despawn_on_hit is skill-only for a basic-attack consumer",
            ),
            (
                collision.max_hits.is_some(),
                "Projectile.collision.max_hits",
                "collision.max_hits is skill-only for a basic-attack consumer",
            ),
        ] {
            if nondefault {
                return Err(DeliveryValidationError::new(field_suffix, cause));
            }
        }
        Ok(())
    }
}

impl Default for SkillProjectileCollisionDef {
    fn default() -> Self {
        Self {
            radius_units: default_projectile_collision_radius_units(),
            hit_targets: SkillHitTargetFilter::Enemies,
            piercing: false,
            despawn_on_hit: None,
            max_hits: None,
        }
    }
}

impl SkillProjectileCollisionDef {
    pub fn validate_runtime_contract(&self) {
        self.try_validate_runtime_contract()
            .expect("invalid SkillProjectileCollisionDef");
    }

    pub fn try_validate_runtime_contract(&self) -> Result<(), String> {
        if self.max_hits == Some(0) {
            return Err("max_hits must be non-zero".to_string());
        }
        Ok(())
    }

    pub fn validate_homing_runtime_contract(&self) {
        self.try_validate_homing_runtime_contract()
            .expect("invalid targeted homing projectile collision contract");
    }

    pub fn try_validate_homing_runtime_contract(&self) -> Result<(), String> {
        if self.radius_units != default_projectile_collision_radius_units() {
            return Err(
                "targeted homing projectile does not support custom collision radius".to_string(),
            );
        }
        if self.hit_targets != SkillHitTargetFilter::Enemies {
            return Err(
                "targeted homing projectile requires the default enemy hit filter".to_string(),
            );
        }
        if self.piercing {
            return Err("targeted homing projectile does not support piercing".to_string());
        }
        if self.despawn_on_hit.is_some() && self.despawn_on_hit != Some(true) {
            return Err(
                "targeted homing projectile does not support despawn_on_hit overrides".to_string(),
            );
        }
        if self.max_hits.is_some() {
            return Err("targeted homing projectile does not support max_hits".to_string());
        }
        Ok(())
    }
}

/// Where an explicit area delivery should be anchored.
///
/// This removes runtime guesswork between:
/// - direct cast-targeted blasts
/// - impact-follow-up explosions
/// - self-centered pulses / ground zones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillAreaAnchorSource {
    #[default]
    CastTarget,
    /// The target tile captured at action admission, even if that unit later leaves.
    AcceptedCastTarget,
    ImpactContext,
    Caster,
    // Use the cast target as the presentation origin instead of the caster.
    CastTargetStart,
    // Use the last projectile/area impact point as the presentation origin.
    ImpactContextStart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillAreaTickPolicy {
    #[default]
    EveryTick,
    OncePerArea,
    OnEnter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillAreaTracking {
    #[default]
    GroundFixed,
    FollowCaster,
    FollowTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillTileAreaOrigin {
    #[default]
    Caster,
    Anchor,
}

/// Tile-membership delivery for DefenseRoute skills.
///
/// `DeliveryDef::TileArea.footprint` owns actual hit tiles independently from
/// cast acquisition reach. This policy defines where that footprint is projected,
/// how units inside it are filtered, and whether the area persists across ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillTileAreaMode {
    /// One release-time query. Empty recipients still complete the admitted use.
    ReleaseQuery,
    /// Caster admission position/facing, owned until this concentration ends.
    ChannelGround { pause_basic: bool },
    #[default]
    Legacy,
    /// Fixed selected tile, world-aligned pattern, lifetime from creation.
    /// May be created without recipients; periodic events re-query membership.
    SelectedGround,
}

crate::authoring::clone_shape::copy_shape!(SkillTileAreaMode);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SkillTileAreaDeliveryDef {
    #[serde(default)]
    pub ground_status: Option<ground::GroundStatusProgram>,
    #[serde(default)]
    pub mode: SkillTileAreaMode,
    #[serde(default)]
    pub emission: Option<SkillAreaEmissionDef>,
    /// Time between release and the first independent impact.
    #[serde(default)]
    pub first_tick_delay_ms: u32,
    /// A single delivery can heal its living caster once after effective damage.
    #[serde(default)]
    pub on_hit_caster_heal_percent: Option<u32>,
    #[serde(default)]
    pub anchor: SkillAreaAnchorSource,
    #[serde(default)]
    pub tile_origin: SkillTileAreaOrigin,
    #[serde(default)]
    pub tracking: SkillAreaTracking,
    #[serde(default)]
    pub hit_targets: SkillHitTargetFilter,
    #[serde(default)]
    pub include_caster: bool,
    #[serde(default)]
    pub tick_policy: SkillAreaTickPolicy,
    #[serde(default)]
    pub duration_ms: u32,
    #[serde(default, deserialize_with = "deserialize_nonzero_option_u32")]
    pub tick_interval_ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillAreaEmissionDef {
    pub footprint: TileRangePattern,
    pub delay_ms: u32,
    pub include_air: bool,
}
crate::authoring::clone_shape::struct_shape!(SkillAreaEmissionDef {
    footprint,
    delay_ms,
    include_air
});

impl DeliveryDef {
    pub(crate) fn validate_authored_provenance(&self) -> Result<(), String> {
        if let Self::TileArea { area, .. } = self {
            if let Some(program) = &area.ground_status {
                for (index, status) in program.statuses.iter().enumerate() {
                    if let Some(vulnerability) = &status.vulnerability {
                        vulnerability.validate_authored_provenance().map_err(|reason| {
                            format!("TileArea.area.ground_status.statuses[{index}].vulnerability.{reason}")
                        })?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl SkillTileAreaDeliveryDef {
    pub fn validate_runtime_contract(&self) {
        self.try_validate_runtime_contract()
            .expect("invalid SkillTileAreaDeliveryDef");
    }

    pub fn is_independent(&self) -> bool {
        self.duration_ms > 0 || self.first_tick_delay_ms > 0
    }

    pub fn try_validate_runtime_contract(&self) -> Result<(), String> {
        if self.mode == SkillTileAreaMode::ReleaseQuery
            && (self.anchor != SkillAreaAnchorSource::Caster
                || self.tile_origin != SkillTileAreaOrigin::Caster
                || self.tracking != SkillAreaTracking::GroundFixed
                || self.duration_ms != 0
                || self.tick_interval_ms.is_some()
                || self.first_tick_delay_ms != 0
                || self.ground_status.is_some()
                || self.emission.is_some()
                || self.on_hit_caster_heal_percent.is_some())
        {
            return Err("release query requires an immediate one-time caster footprint".into());
        }
        if matches!(
            self.mode,
            SkillTileAreaMode::SelectedGround | SkillTileAreaMode::ChannelGround { .. }
        ) && (self.anchor != SkillAreaAnchorSource::AcceptedCastTarget
            || self.tile_origin != SkillTileAreaOrigin::Anchor
            || self.tracking != SkillAreaTracking::GroundFixed
            || self.duration_ms == 0
            || (self.tick_interval_ms.is_none() && self.ground_status.is_none())
            || self.tick_policy != SkillAreaTickPolicy::EveryTick
            || self.emission.is_some()
            || self.on_hit_caster_heal_percent.is_some())
        {
            return Err("selected ground requires a finite world-aligned periodic area at the accepted tile".into());
        }
        if let Some(program) = &self.ground_status {
            program.validate()?;
            if !matches!(
                self.mode,
                SkillTileAreaMode::SelectedGround | SkillTileAreaMode::ChannelGround { .. }
            ) || self.tick_interval_ms.is_some()
                || self.first_tick_delay_ms != 0
            {
                return Err("ground status subscriptions require selected ground without periodic damage ticks".into());
            }
        }
        if let Some(emission) = &self.emission {
            emission.footprint.validate()?;
            let interval = self
                .tick_interval_ms
                .filter(|n| *n > 0)
                .ok_or("emission interval must be positive")?;
            let span = self
                .duration_ms
                .checked_sub(1)
                .ok_or("emission duration must be positive")?;
            if span % interval != 0 || span / interval >= 10_000 {
                return Err(
                    "emission requires an exact finite schedule of at most 10000 opportunities"
                        .into(),
                );
            }
            if self.duration_ms == 0
                || self.tick_interval_ms.is_none()
                || self.first_tick_delay_ms != 0
                || self.on_hit_caster_heal_percent.is_some()
                || self.tracking != SkillAreaTracking::GroundFixed
                || self.anchor != SkillAreaAnchorSource::AcceptedCastTarget
                || self.tile_origin != SkillTileAreaOrigin::Anchor
                || self.hit_targets != SkillHitTargetFilter::Enemies
                || self.tick_policy != SkillAreaTickPolicy::EveryTick
            {
                return Err("area emission requires a finite fixed admission anchor and periodic enemy opportunities".into());
            }
        }
        if self
            .on_hit_caster_heal_percent
            .is_some_and(|p| p == 0 || p > 100)
            || (self.on_hit_caster_heal_percent.is_some()
                && (self.duration_ms != 0 || self.tick_interval_ms.is_some()))
        {
            return Err("on-hit caster healing requires one impact and a percent in 1..100".into());
        }
        if self.first_tick_delay_ms > 0 && self.tracking != SkillAreaTracking::GroundFixed {
            return Err("delayed release requires fixed ground geometry".into());
        }
        if self.anchor == SkillAreaAnchorSource::AcceptedCastTarget
            && self.tracking != SkillAreaTracking::GroundFixed
        {
            return Err("accepted target anchors require fixed ground geometry".into());
        }
        if self.tick_interval_ms == Some(0) {
            return Err("tick_interval_ms must be non-zero".to_string());
        }
        if self.duration_ms > 0 && self.tick_interval_ms.is_none() && self.ground_status.is_none() {
            return Err("persistent tile skill area requires tick_interval_ms".to_string());
        }
        if self.duration_ms > 0 {
            match self.tracking {
                SkillAreaTracking::GroundFixed => {}
                SkillAreaTracking::FollowCaster => {
                    if self.anchor != SkillAreaAnchorSource::Caster {
                        return Err(
                            "FollowCaster persistent tile area requires Caster anchor".to_string()
                        );
                    }
                }
                SkillAreaTracking::FollowTarget => {
                    if !matches!(
                        self.anchor,
                        SkillAreaAnchorSource::CastTarget | SkillAreaAnchorSource::CastTargetStart
                    ) {
                        return Err("FollowTarget persistent tile area requires CastTarget or CastTargetStart anchor".to_string());
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StepTargetingMode {
    #[default]
    ReuseCastTarget,
    RetargetOnStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillUnitReference {
    SelfUnit,
    StepTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillStepCondition {
    #[default]
    Always,
    IfPreviousStepDealtDamage,
    IfCasterHasBuff {
        buff_id: String,
        #[serde(default = "default_min_stacks")]
        min_stacks: u8,
    },
}

fn default_min_stacks() -> u8 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillStepRepeat {
    #[default]
    Once,
    Times {
        count: u8,
    },
    ByBuffStacks {
        unit: SkillUnitReference,
        buff_id: String,
        #[serde(default)]
        max: Option<u8>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillPresentationDef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cast_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projectile_vfx_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact_vfx_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_anchor: Option<String>,
}

impl SkillPresentationDef {
    pub fn is_empty(&self) -> bool {
        self.cast_state
            .as_deref()
            .map_or(true, |value| value.trim().is_empty())
            && self
                .projectile_vfx_id
                .as_deref()
                .map_or(true, |value| value.trim().is_empty())
            && self
                .impact_vfx_id
                .as_deref()
                .map_or(true, |value| value.trim().is_empty())
            && self
                .target_anchor
                .as_deref()
                .map_or(true, |value| value.trim().is_empty())
    }
}

fn default_proc_chance_percent() -> u8 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityActivationDef {
    TriggerProc {
        trigger: TriggerType,
        #[serde(default = "default_proc_chance_percent")]
        proc_chance_percent: u8,
        #[serde(default)]
        internal_cooldown_ms: u64,
        #[serde(default)]
        max_triggers_per_battle: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityActivationBinding {
    pub ability_id: SkillId,
    pub activation: AbilityActivationDef,
}

fn default_step_id() -> String {
    "step".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillCastTargetingDef {
    /// Explicit cast-level targeting independent from individual step execution targets.
    Explicit {
        selection: SkillCastSelection,
        target: SkillTarget,
        reach: SkillTargetReach,
        #[serde(default)]
        target_range: Option<TileRangePattern>,
        air_capable: bool,
    },
}

impl SkillCastTargetingDef {
    pub fn explicit(
        target: SkillTarget,
        reach: SkillTargetReach,
        target_range: Option<TileRangePattern>,
        air_capable: bool,
    ) -> Self {
        let selection = match target {
            SkillTarget::SelfUnit => SkillCastSelection::SelfUnit,
            SkillTarget::EnemySingle { .. } | SkillTarget::AllySingle => {
                SkillCastSelection::AutomaticUnit
            }
            SkillTarget::CastTarget => SkillCastSelection::PlayerTile,
        };
        Self::explicit_with_selection(selection, target, reach, target_range, air_capable)
    }

    pub fn explicit_with_selection(
        selection: SkillCastSelection,
        target: SkillTarget,
        reach: SkillTargetReach,
        target_range: Option<TileRangePattern>,
        air_capable: bool,
    ) -> Self {
        Self::Explicit {
            selection,
            target,
            reach,
            target_range,
            air_capable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillStepDef {
    #[serde(default = "default_step_id")]
    pub id: String,
    #[serde(default)]
    pub delay_ms: u32,
    pub target: SkillTarget,
    #[serde(default)]
    pub targeting: StepTargetingMode,
    #[serde(default)]
    pub when: SkillStepCondition,
    #[serde(default)]
    pub repeat: SkillStepRepeat,
    #[serde(default)]
    pub delivery: DeliveryDef,
    #[serde(default)]
    pub effects: Vec<authoring::AuthoredSkillEffect>,
    #[serde(default)]
    pub presentation: SkillPresentationDef,
}

fn default_skill_name() -> String {
    "Unnamed Skill".to_string()
}

/// 데이터 기반 스킬 정의 (RON 로드 대상)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillUseEffectRef {
    pub step: String,
    pub effect: String,
}

/// Which applied finite effects remain part of the caster's use of this skill.
/// Delivery settlement and independently created children are separate owners.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillUseLifetime {
    Action,
    ActionAndEffects { effects: Vec<SkillUseEffectRef> },
}

impl SkillUseLifetime {
    pub fn effects(&self) -> &[SkillUseEffectRef] {
        match self {
            Self::Action => &[],
            Self::ActionAndEffects { effects } => effects,
        }
    }

    pub fn owns_effect(&self, step: &str, effect: &str) -> bool {
        self.effects()
            .iter()
            .any(|owner| owner.step == step && owner.effect == effect)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_policy: Option<std::sync::Arc<action::ActionPolicy>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resonance_admission: Option<resonance::ResonanceAdmission>,
    pub use_lifetime: SkillUseLifetime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<std::sync::Arc<flow::SkillFlowDef>>,
    pub id: SkillId,
    #[serde(default = "default_skill_name")]
    pub name: String,
    #[serde(default)]
    pub kind: SkillKind,
    pub cast_targeting: SkillCastTargetingDef,
    #[serde(default)]
    pub focus_time_ms: u32,
    #[serde(default)]
    pub focus_permissions: FocusPermissions,
    #[serde(default)]
    pub steps: Vec<SkillStepDef>,
}

/// 스킬 효과(초안): 구현 단계에서 커맨드/시스템으로 매핑될 수 있는 데이터 표현
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillEffectDef {
    GuardCounter {
        program: std::sync::Arc<guard_charge::GuardCounterDef>,
    },
    StatusStrike {
        program: std::sync::Arc<status_strike::StatusStrikeDef>,
    },
    FusionPlacement {
        program: std::sync::Arc<mobile_effect::FusionPlacementDef>,
    },
    PairedAreas {
        program: std::sync::Arc<paired_areas::PairedAreasDef>,
    },
    PreparedVolley {
        program: std::sync::Arc<prepared_volley::PreparedVolleyDef>,
    },
    InputPlan {
        program: std::sync::Arc<input_plan::InputPlanDef>,
    },
    RelationAction {
        program: std::sync::Arc<relation_action::RelationActionDef>,
    },
    CounterChannel {
        program: std::sync::Arc<counter_channel::CounterChannelDef>,
        root: SkillId,
        entry: usize,
    },
    TileBurst {
        program: std::sync::Arc<tile_burst::TileBurstDef>,
    },
    PresenceBurst {
        program: std::sync::Arc<marks::PresenceBurstProgram>,
    },
    PersonalShieldGrant {
        definition: crate::game::relic::structural::authoring::PersonalShieldGrantDef,
    },
    CreateObject {
        definition: objects::CreateObjectDef,
    },
    Pull {
        max_distance_tiles: u32,
    },
    HealthCost {
        amount: u32,
        allow_lethal: bool,
    },
    RechargeShield {
        source_skill: SkillId,
        amount: u32,
    },
    ShieldExpr {
        amount: crate::game::value_expr::AmountExpr,
        duration_ms: u32,
        priority: i32,
        shared_radius_units: Option<u32>,
    },
    Sustained {
        definition: sustained::SustainedEffectDef,
    },
    Damage {
        #[serde(deserialize_with = "deserialize_signed_amount")]
        amount: i32,
        damage_type: crate::game::battle::damage::DamageType,
    },
    DamageExpr {
        amount: crate::game::value_expr::AmountExpr,
        damage_type: crate::game::battle::damage::DamageType,
    },
    ModifyDamage {
        modifiers: crate::game::battle::damage::DamageModifiers,
    },
    Heal {
        #[serde(deserialize_with = "deserialize_signed_amount")]
        amount: i32,
    },
    HealExpr {
        amount: crate::game::value_expr::AmountExpr,
    },
    ModifyResonance {
        #[serde(deserialize_with = "deserialize_signed_amount")]
        amount: i32,
    },
    ModifyTacticalResource {
        #[serde(deserialize_with = "deserialize_signed_amount")]
        amount: i32,
    },
    Resources {
        operations: Vec<crate::game::resource::ResourceOperation>,
    },
    ModifyStats {
        modifier: crate::game::stats::StatModifier,
    },
    ApplyBuff {
        buff_id: String,
        duration_ms: u32,
    },
    InterruptCast,
    ExtraAttack {
        count: u8,
    },
}
impl crate::authoring::clone_shape::CloneShape for SkillTarget {
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        // This enum owns only a discriminant and the fixed UnitTargetRule.
        meter.scan_storage(Some(2))
    }
}

impl SkillDef {
    /// Compiled passive declaration belongs to the equipped fragment form.
    pub fn fragment_guard(&self) -> Option<crate::game::data::skill_fragment_data::ParryGuardDef> {
        self.steps
            .iter()
            .flat_map(|step| &step.effects)
            .find_map(|effect| match &effect.op {
                SkillEffectDef::Sustained {
                    definition: sustained::SustainedEffectDef::Parry { fragment_guard, .. },
                } => *fragment_guard,
                _ => None,
            })
    }
    pub fn first_step(&self) -> Option<&SkillStepDef> {
        self.steps.first()
    }

    pub fn cast_target_definition(
        &self,
    ) -> Option<(
        &SkillTarget,
        SkillTargetReach,
        Option<&TileRangePattern>,
        bool,
    )> {
        match &self.cast_targeting {
            SkillCastTargetingDef::Explicit {
                target,
                reach,
                target_range,
                air_capable,
                ..
            } => Some((target, *reach, target_range.as_ref(), *air_capable)),
        }
    }

    pub fn cast_selection(&self) -> SkillCastSelection {
        match &self.cast_targeting {
            SkillCastTargetingDef::Explicit { selection, .. } => *selection,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_permissions_default_is_hard_lock() {
        let perms = FocusPermissions::default();
        assert!(!perms.allows_move);
        assert!(!perms.allows_basic_attack);
    }

    #[test]
    fn skill_def_ron_deserialization_reads_step_based_schema() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s1",
                name:"Test Skill",
                focus_time_ms:200,
                use_lifetime: Action,
                cast_targeting: Explicit(
                    selection:automatic_unit,
                    target:EnemySingle(rule:Nearest),
                    reach:pattern,
                    air_capable:false,
                ),
                steps:[
                    (
                        id:"hit",
                        target:EnemySingle(rule:Nearest),
                        delivery:Instant,
                        effects:[(id: "effect_01", op: Damage(amount:10, damage_type: Magic))],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(def.id.as_str(), "s1");
        assert_eq!(def.name, "Test Skill");
        assert_eq!(def.kind, SkillKind::Targeted);
        assert_eq!(
            def.cast_targeting,
            SkillCastTargetingDef::Explicit {
                selection: SkillCastSelection::AutomaticUnit,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                reach: Default::default(),
                target_range: None,
                air_capable: false,
            }
        );
        assert_eq!(def.focus_time_ms, 200);
        assert_eq!(def.steps.len(), 1);
        assert_eq!(def.steps[0].id, "hit");
        assert_eq!(def.steps[0].targeting, StepTargetingMode::ReuseCastTarget);
        assert_eq!(def.steps[0].when, SkillStepCondition::Always);
        assert_eq!(def.steps[0].repeat, SkillStepRepeat::Once);
        assert!(matches!(
            def.steps[0].target,
            SkillTarget::EnemySingle { .. }
        ));
        assert!(matches!(def.steps[0].delivery, DeliveryDef::Instant));
    }

    #[test]
    fn enemy_single_target_requires_explicit_rule_and_rejects_unknown_authority() {
        assert!(serde_json::from_str::<SkillTarget>(r#"{"EnemySingle":{}}"#).is_err());
        assert!(serde_json::from_str::<SkillTarget>(
            r#"{"EnemySingle":{"rule":"Nearest","targeting_profile":"LowDefenseFirst"}}"#
        )
        .is_err());
    }

    #[test]
    fn skill_step_ron_deserialization_reads_explicit_targeting_mode() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s2",
                use_lifetime: Action,
                cast_targeting: Explicit(
                    selection:automatic_unit,
                    target:EnemySingle(rule:Nearest),
                    reach:pattern,
                    air_capable:false,
                ),
                steps:[
                    (
                        id:"retarget",
                        target:EnemySingle(rule:Nearest),
                        targeting:RetargetOnStep,
                        effects:[(id: "effect_01", op: Damage(amount:10, damage_type: Magic))],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(def.steps[0].targeting, StepTargetingMode::RetargetOnStep);
    }

    #[test]
    fn skill_step_ron_deserialization_reads_damage_modifiers() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s_damage_mod",
                use_lifetime: Action,
                cast_targeting: Explicit(
                    selection:automatic_unit,
                    target:EnemySingle(rule:Nearest),
                    reach:pattern,
                    air_capable:false,
                ),
                steps:[
                    (
                        id:"hit",
                        target:EnemySingle(rule:Nearest),
                        effects:[
                            (id: "effect_01", op: ModifyDamage(modifiers:(magic_resist_penetration_flat:25))),
                            (id: "effect_02", op: Damage(amount:100, damage_type:Magic)),],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert!(matches!(
            def.steps[0].effects[0].op,
            SkillEffectDef::ModifyDamage { modifiers }
                if modifiers.magic_resist_penetration_flat == 25
                    && modifiers.armor_penetration_flat == 0
        ));
    }

    #[test]
    fn skill_step_ron_deserialization_reads_tactical_resource_modifier() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"stabilize",
                use_lifetime: Action,
                cast_targeting: Explicit(
                    selection:self,
                    target:SelfUnit,
                    reach:pattern,
                    air_capable:false,
                ),
                steps:[
                    (
                        id:"restore_stability",
                        target:SelfUnit,
                        effects:[(id: "effect_01", op: ModifyTacticalResource(amount:5))],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert!(matches!(
            def.steps[0].effects[0].op,
            SkillEffectDef::ModifyTacticalResource { amount: 5 }
        ));
    }

    #[test]
    fn skill_step_ron_rejects_legacy_stabilization_modifier() {
        let error = ron::de::from_str::<SkillDef>(
            r#"
            (
                id:"legacy_stabilize",
                use_lifetime: Action,
                cast_targeting: Explicit(
                    selection:self,
                    target:SelfUnit,
                    reach:pattern,
                    air_capable:false,
                ),
                steps:[
                    (
                        id:"legacy_restore",
                        target:SelfUnit,
                        effects:[(id: "legacy", op: ModifyStabilization(amount:5))],
                    ),
                ],
            )
            "#,
        )
        .expect_err("legacy stabilization effect name must not deserialize");

        assert!(error.to_string().contains("ModifyStabilization"));
    }

    #[test]
    fn skill_step_ron_deserialization_reads_condition_and_repeat() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s3",
                use_lifetime: Action,
                cast_targeting: Explicit(
                    selection:self,
                    target:SelfUnit,
                    reach:pattern,
                    air_capable:false,
                ),
                steps:[
                    (
                        id:"followup",
                        target:SelfUnit,
                        when:IfPreviousStepDealtDamage,
                        repeat:ByBuffStacks(
                            unit:SelfUnit,
                            buff_id:"poison",
                            max:Some(3),
                        ),
                        effects:[(id: "effect_01", op: Heal(amount:10))],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(
            def.steps[0].when,
            SkillStepCondition::IfPreviousStepDealtDamage
        );
        assert_eq!(
            def.steps[0].repeat,
            SkillStepRepeat::ByBuffStacks {
                unit: SkillUnitReference::SelfUnit,
                buff_id: "poison".to_string(),
                max: Some(3),
            }
        );
    }

    #[test]
    fn skill_def_ron_deserialization_reads_explicit_cast_targeting() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s4",
                use_lifetime: Action,
                cast_targeting: Explicit(
                    selection:automatic_unit,
                    target:EnemySingle(rule:LowestHealthEnemy),
                    reach:pattern,
                    air_capable:false,
                ),
                steps:[
                    (
                        id:"charge",
                        target:SelfUnit,
                        effects:[(id: "effect_01", op: Heal(amount:10))],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(
            def.cast_targeting,
            SkillCastTargetingDef::Explicit {
                selection: SkillCastSelection::AutomaticUnit,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::LowestHealthEnemy,
                },
                reach: Default::default(),
                target_range: None,
                air_capable: false,
            }
        );
        assert_eq!(
            def.cast_target_definition(),
            Some((
                &SkillTarget::EnemySingle {
                    rule: UnitTargetRule::LowestHealthEnemy,
                },
                Default::default(),
                None,
                false,
            ))
        );
    }

    #[test]
    fn skill_projectile_collision_def_uses_expected_defaults() {
        let collision: SkillProjectileCollisionDef = ron::de::from_str("()").unwrap();
        assert_eq!(
            collision,
            SkillProjectileCollisionDef {
                radius_units: 125_000,
                hit_targets: SkillHitTargetFilter::Enemies,
                piercing: false,
                despawn_on_hit: None,
                max_hits: None,
            }
        );
    }

    #[test]
    fn skill_projectile_collision_def_reads_explicit_values() {
        let collision: SkillProjectileCollisionDef = ron::de::from_str(
            r#"
            (
                radius_units:250000,
                hit_targets:Any,
                piercing:true,
                despawn_on_hit:Some(false),
                max_hits:Some(3),
            )
            "#,
        )
        .unwrap();

        assert_eq!(
            collision,
            SkillProjectileCollisionDef {
                radius_units: 250_000,
                hit_targets: SkillHitTargetFilter::Any,
                piercing: true,
                despawn_on_hit: Some(false),
                max_hits: Some(3),
            }
        );
    }

    #[test]
    fn delivery_def_projectile_ron_defaults_collision_when_omitted() {
        let delivery: DeliveryDef = ron::de::from_str(
            r#"
            Projectile(
                speed_units_per_ms:6000,
            )
            "#,
        )
        .unwrap();

        assert_eq!(
            delivery,
            DeliveryDef::Projectile {
                speed_units_per_ms: 6_000,
                hit_policy: ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: SkillProjectileCollisionDef::default(),
            }
        );
    }

    #[test]
    fn delivery_def_tile_area_ron_reads_anchor_tile_origin() {
        let delivery: DeliveryDef = ron::de::from_str(
            r#"
            TileArea(
                footprint:(include_anchor_tile:true, rows:[".@."]),
                area:(
                    anchor:CastTarget,
                    tile_origin:Anchor,
                    hit_targets:Enemies,
                ),
            )
            "#,
        )
        .unwrap();

        assert_eq!(
            delivery,
            DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec![".@.".to_string()],
                },
                area: SkillTileAreaDeliveryDef {
                    ground_status: None,
                    mode: Default::default(),
                    emission: None,
                    first_tick_delay_ms: 0,
                    on_hit_caster_heal_percent: None,
                    anchor: SkillAreaAnchorSource::CastTarget,
                    tile_origin: SkillTileAreaOrigin::Anchor,
                    tracking: SkillAreaTracking::GroundFixed,
                    hit_targets: SkillHitTargetFilter::Enemies,
                    include_caster: false,
                    tick_policy: SkillAreaTickPolicy::EveryTick,
                    duration_ms: 0,
                    tick_interval_ms: None,
                },
            }
        );
    }

    #[test]
    fn delivery_def_tile_area_ron_defaults_tile_origin_to_caster() {
        let delivery: DeliveryDef = ron::de::from_str(
            r#"
            TileArea(
                footprint:(include_anchor_tile:true, rows:[".@."]),
                area:(
                    anchor:CastTarget,
                    hit_targets:Enemies,
                ),
            )
            "#,
        )
        .unwrap();

        let DeliveryDef::TileArea { area, .. } = delivery else {
            panic!("expected tile area");
        };
        assert_eq!(area.tile_origin, SkillTileAreaOrigin::Caster);
    }

    #[test]
    fn projectile_collision_raw_ron_candidate_defers_zero_max_hits_to_semantic_validation() {
        let collision = ron::de::from_str::<SkillProjectileCollisionDef>(
            r#"(radius_units:125000, max_hits:Some(0))"#,
        )
        .expect("raw RON parsing may form an unpublished semantic candidate");
        let error = collision
            .try_validate_runtime_contract()
            .expect_err("max_hits == 0 must reject before publication");

        assert_eq!(error, "max_hits must be non-zero");
    }
}

// RON 0.5 parses a negative number's magnitude before applying its sign.
// Widen only signed SkillEffectDef amounts, then enforce the existing i32 wire range.
fn deserialize_signed_amount<'de, D: serde::Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
    let value = i64::deserialize(d)?;
    i32::try_from(value).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod signed_amount_tests {
    use super::*;
    #[test]
    fn r06_signed_amount_raw_ron_full_i32_domain_and_checked_bounds() {
        for kind in [
            "Damage",
            "Heal",
            "ModifyResonance",
            "ModifyTacticalResource",
        ] {
            for value in [i32::MIN, i32::MIN + 1, -1, 0, i32::MAX] {
                let suffix = if kind == "Damage" {
                    ", damage_type: True"
                } else {
                    ""
                };
                let raw = format!("{kind}(amount: {value}{suffix})");
                let op: SkillEffectDef = ron::de::from_str(&raw).unwrap();
                let actual = match op {
                    SkillEffectDef::Damage { amount, .. }
                    | SkillEffectDef::Heal { amount }
                    | SkillEffectDef::ModifyResonance { amount }
                    | SkillEffectDef::ModifyTacticalResource { amount } => amount,
                    _ => unreachable!(),
                };
                assert_eq!(actual, value);
                let encoded = ron::ser::to_string(&op).unwrap();
                assert!(encoded.contains(&value.to_string()));
            }
            for value in [i64::from(i32::MIN) - 1, i64::from(i32::MAX) + 1] {
                let suffix = if kind == "Damage" {
                    ", damage_type: True"
                } else {
                    ""
                };
                assert!(ron::de::from_str::<SkillEffectDef>(&format!(
                    "{kind}(amount: {value}{suffix})"
                ))
                .is_err());
            }
        }
        assert!(ron::de::from_str::<SkillEffectDef>("Heal(amount: 0, unknown: 1)").is_err());
    }
}
