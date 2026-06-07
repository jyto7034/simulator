use std::{borrow::Borrow, fmt, ops::Deref};

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTarget {
    SelfUnit,
    EnemySingle {
        #[serde(default)]
        rule: UnitTargetRule,
    },
    /// Reuse the cast-level target/anchor for delayed spatial deliveries.
    ///
    /// This is intentionally not a range expression. Actual multi-hit area
    /// filtering belongs to `SkillStepDef.defense_tile_range` plus the delivery
    /// hit filter. DefenseRoute official live skills use `DeliveryDef::TileArea`.
    CastTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeliveryDef {
    #[default]
    Instant,
    Projectile {
        speed_units_per_ms: u32,
        #[serde(default)]
        collision: SkillProjectileCollisionDef,
    },
    TileArea {
        area: SkillTileAreaDeliveryDef,
    },
}

/// Continuous collision filter for skill projectile / area delivery.
///
/// This is intentionally kept separate from `SkillTarget`:
/// - `SkillTarget` answers "who is the step trying to affect?"
/// - `SkillHitTargetFilter` answers "who can this spatial delivery collide with?"
///
/// Runtime wiring is introduced in a later phase; for now this type fixes the
/// data contract for projectile/AoE continuous delivery.
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

fn deserialize_nonzero_option_u8<'de, D>(deserializer: D) -> Result<Option<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<u8>::deserialize(deserializer)?;
    match value {
        Some(0) => Err(serde::de::Error::custom("expected non-zero max_hits")),
        other => Ok(other),
    }
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
    #[serde(default, deserialize_with = "deserialize_nonzero_option_u8")]
    pub max_hits: Option<u8>,
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
        assert_ne!(
            self.max_hits,
            Some(0),
            "SkillProjectileCollisionDef.max_hits must be non-zero"
        );
    }

    pub fn validate_homing_runtime_contract(&self) {
        assert_eq!(
            self.radius_units,
            default_projectile_collision_radius_units(),
            "targeted homing projectile does not support custom collision radius"
        );
        assert_eq!(
            self.hit_targets,
            SkillHitTargetFilter::Enemies,
            "targeted homing projectile requires the default enemy hit filter"
        );
        assert!(
            !self.piercing,
            "targeted homing projectile does not support piercing"
        );
        assert!(
            self.despawn_on_hit.is_none() || self.despawn_on_hit == Some(true),
            "targeted homing projectile does not support despawn_on_hit overrides"
        );
        assert_eq!(
            self.max_hits, None,
            "targeted homing projectile does not support max_hits"
        );
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
/// In DefenseRoute, `SkillStepDef.defense_tile_range` is the single source of
/// truth for presentation, cast range, target candidates, and actual hit tiles.
/// This delivery defines where the tile pattern is projected, how targets inside
/// that tile range are filtered, and whether the area persists across ticks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SkillTileAreaDeliveryDef {
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

impl SkillTileAreaDeliveryDef {
    pub fn validate_runtime_contract(&self) {
        assert_ne!(
            self.tick_interval_ms,
            Some(0),
            "SkillTileAreaDeliveryDef.tick_interval_ms must be non-zero"
        );
        assert!(
            self.duration_ms == 0 || self.tick_interval_ms.is_some(),
            "persistent tile skill area requires tick_interval_ms"
        );
        if self.duration_ms > 0 {
            match self.tracking {
                SkillAreaTracking::GroundFixed => {}
                SkillAreaTracking::FollowCaster => assert_eq!(
                    self.anchor,
                    SkillAreaAnchorSource::Caster,
                    "FollowCaster persistent tile area requires Caster anchor"
                ),
                SkillAreaTracking::FollowTarget => assert!(
                    matches!(
                        self.anchor,
                        SkillAreaAnchorSource::CastTarget | SkillAreaAnchorSource::CastTargetStart
                    ),
                    "FollowTarget persistent tile area requires CastTarget or CastTargetStart anchor"
                ),
            }
        }
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

fn default_step_range_units() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum SkillCastTargetingDef {
    /// Backward-compatible default: infer the cast-level target from the first step.
    #[default]
    FirstStepTarget,
    /// Explicit cast-level targeting independent from individual step execution targets.
    Explicit {
        #[serde(default = "default_step_range_units")]
        range_units: f32,
        target: SkillTarget,
        #[serde(default)]
        defense_tile_range: Option<TileRangePattern>,
        #[serde(default)]
        air_capable: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStepDef {
    #[serde(default = "default_step_id")]
    pub id: String,
    #[serde(default)]
    pub delay_ms: u32,
    #[serde(default = "default_step_range_units")]
    pub range_units: f32,
    #[serde(default)]
    pub defense_tile_range: Option<TileRangePattern>,
    #[serde(default)]
    pub air_capable: bool,
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
    pub effects: Vec<SkillEffectDef>,
    #[serde(default)]
    pub presentation: SkillPresentationDef,
}

fn default_skill_name() -> String {
    "Unnamed Skill".to_string()
}

/// 데이터 기반 스킬 정의 (RON 로드 대상)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub id: SkillId,
    #[serde(default = "default_skill_name")]
    pub name: String,
    #[serde(default)]
    pub kind: SkillKind,
    #[serde(default)]
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
pub enum SkillEffectDef {
    Damage {
        amount: i32,
        damage_type: crate::game::battle::damage::DamageType,
    },
    ModifyDamage {
        modifiers: crate::game::battle::damage::DamageModifiers,
    },
    Heal {
        amount: i32,
    },
    ModifyResonance {
        amount: i32,
    },
    ModifyStabilization {
        amount: i32,
    },
    ModifyStats {
        modifier: crate::game::stats::StatModifier,
    },
    ApplyBuff {
        buff_id: String,
        duration_ms: u32,
    },
    ExtraAttack {
        count: u8,
    },
}

impl SkillDef {
    pub fn first_step(&self) -> Option<&SkillStepDef> {
        self.steps.first()
    }

    pub fn cast_target_definition(
        &self,
    ) -> Option<(f32, &SkillTarget, Option<&TileRangePattern>, bool)> {
        match &self.cast_targeting {
            SkillCastTargetingDef::FirstStepTarget => self.first_step().map(|step| {
                (
                    step.range_units,
                    &step.target,
                    step.defense_tile_range.as_ref(),
                    step.air_capable,
                )
            }),
            SkillCastTargetingDef::Explicit {
                range_units,
                target,
                defense_tile_range,
                air_capable,
            } => Some((
                *range_units,
                target,
                defense_tile_range.as_ref(),
                *air_capable,
            )),
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
                steps:[
                    (
                        id:"hit",
                        range_units:3,
                        target:EnemySingle(rule:Nearest),
                        delivery:Instant,
                        effects:[Damage(amount:10, damage_type: Magic)],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(def.id.as_str(), "s1");
        assert_eq!(def.name, "Test Skill");
        assert_eq!(def.kind, SkillKind::Targeted);
        assert_eq!(def.cast_targeting, SkillCastTargetingDef::FirstStepTarget);
        assert_eq!(def.focus_time_ms, 200);
        assert_eq!(def.steps.len(), 1);
        assert_eq!(def.steps[0].id, "hit");
        assert_eq!(def.steps[0].range_units, 3.0);
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
    fn enemy_single_target_defaults_to_nearest_rule_when_field_missing() {
        let target: SkillTarget = serde_json::from_str(r#"{"EnemySingle":{}}"#).unwrap();
        assert_eq!(
            target,
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest
            }
        );
    }

    #[test]
    fn skill_step_ron_deserialization_reads_explicit_targeting_mode() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s2",
                steps:[
                    (
                        id:"retarget",
                        range_units:2,
                        target:EnemySingle(rule:Nearest),
                        targeting:RetargetOnStep,
                        effects:[Damage(amount:10, damage_type: Magic)],
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
                steps:[
                    (
                        id:"hit",
                        range_units:2,
                        target:EnemySingle(rule:Nearest),
                        effects:[
                            ModifyDamage(modifiers:(magic_resist_penetration_flat:25)),
                            Damage(amount:100, damage_type:Magic),
                        ],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert!(matches!(
            def.steps[0].effects[0],
            SkillEffectDef::ModifyDamage { modifiers }
                if modifiers.magic_resist_penetration_flat == 25
                    && modifiers.armor_penetration_flat == 0
        ));
    }

    #[test]
    fn skill_step_ron_deserialization_reads_stabilization_modifier() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"stabilize",
                steps:[
                    (
                        id:"restore_stability",
                        range_units:0,
                        target:SelfUnit,
                        effects:[ModifyStabilization(amount:5)],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert!(matches!(
            def.steps[0].effects[0],
            SkillEffectDef::ModifyStabilization { amount: 5 }
        ));
    }

    #[test]
    fn skill_step_ron_deserialization_reads_condition_and_repeat() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s3",
                steps:[
                    (
                        id:"followup",
                        range_units:1,
                        target:SelfUnit,
                        when:IfPreviousStepDealtDamage,
                        repeat:ByBuffStacks(
                            unit:SelfUnit,
                            buff_id:"poison",
                            max:Some(3),
                        ),
                        effects:[Heal(amount:10)],
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
                cast_targeting:Explicit(
                    range_units:4,
                    target:EnemySingle(rule:LowestHealthEnemy),
                ),
                steps:[
                    (
                        id:"charge",
                        target:SelfUnit,
                        effects:[Heal(amount:10)],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(
            def.cast_targeting,
            SkillCastTargetingDef::Explicit {
                range_units: 4.0,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::LowestHealthEnemy,
                },
                defense_tile_range: None,
                air_capable: false,
            }
        );
        assert_eq!(
            def.cast_target_definition(),
            Some((
                4.0,
                &SkillTarget::EnemySingle {
                    rule: UnitTargetRule::LowestHealthEnemy,
                },
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
                collision: SkillProjectileCollisionDef::default(),
            }
        );
    }

    #[test]
    fn delivery_def_tile_area_ron_reads_anchor_tile_origin() {
        let delivery: DeliveryDef = ron::de::from_str(
            r#"
            TileArea(
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
                area: SkillTileAreaDeliveryDef {
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
                area:(
                    anchor:CastTarget,
                    hit_targets:Enemies,
                ),
            )
            "#,
        )
        .unwrap();

        let DeliveryDef::TileArea { area } = delivery else {
            panic!("expected tile area");
        };
        assert_eq!(area.tile_origin, SkillTileAreaOrigin::Caster);
    }

    #[test]
    fn projectile_collision_ron_rejects_zero_max_hits() {
        let err = ron::de::from_str::<SkillProjectileCollisionDef>(
            r#"(radius_units:125000, max_hits:Some(0))"#,
        )
        .expect_err("max_hits == 0 must be rejected");

        assert!(err.to_string().contains("non-zero max_hits"));
    }
}
