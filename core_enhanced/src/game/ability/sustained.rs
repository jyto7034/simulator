//! Authored finite effects. Permanent ModifyStats retains its original contract.
use crate::game::stats::{StatId, StatModifier, StatModifierKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SustainedEffectDef {
    Parry {
        duration_ms: u32,
        window_ms: u32,
        invulnerable_ms: u32,
        miss_recovery_ms: u32,
        reduction_percent: u32,
        counter_skill: super::SkillId,
    },
    Control {
        duration_ms: u32,
        kind: ControlKind,
    },
    Survival {
        duration_ms: u32,
        policy: SurvivalPolicy,
    },
    SegmentedShield {
        duration_ms: u32,
        pool: SegmentedShieldDef,
    },
    Shield {
        duration_ms: u32,
        amount: u32,
        priority: i32,
        shared_radius_units: Option<u32>,
    },
    BasicAttackOverlay {
        duration_ms: u32,
        next_attack_only: bool,
        #[serde(default)]
        goodbye_skill: Option<super::SkillId>,
        profile: BasicAttackOverlayProfile,
    },
    ChannelAura {
        duration_ms: u32,
        aura: ChannelAuraDef,
    },
    Modifiers {
        duration_ms: u32,
        modifiers: Vec<StatModifier>,
    },
    CombatModifiers {
        duration_ms: u32,
        modifiers: CombatModifierProfile,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatModifierProfile {
    pub damage_reduction_percent: u32,
    pub basic_lifesteal_percent: u32,
    pub basic_lifesteal_max_heal: u32,
}
crate::authoring::clone_shape::copy_shape!(CombatModifierProfile);

/// Immutable pool policy; the live remaining HP is owned by ShieldPoolView.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SegmentedShieldDef {
    pub pool_id: String,
    pub segment_health: u32,
    pub max_segments: u32,
    pub initial_health: u32,
    pub block_per_segment: u32,
    pub priority: i32,
}
crate::authoring::clone_shape::struct_shape!(SegmentedShieldDef {
    pool_id,
    segment_health,
    max_segments,
    initial_health,
    block_per_segment,
    priority
});
impl SegmentedShieldDef {
    pub fn validate(&self) -> Result<(), String> {
        let capacity = self.segment_health.checked_mul(self.max_segments);
        if self.pool_id.trim().is_empty()
            || self.segment_health == 0
            || self.max_segments == 0
            || self.initial_health == 0
            || capacity.is_none_or(|n| n > i32::MAX as u32 || self.initial_health > n)
            || self
                .max_segments
                .checked_mul(self.block_per_segment)
                .is_none()
        {
            return Err("SegmentedShield requires an explicit pool, positive signed-range capacity/initial health and bounded block bonus".into());
        }
        Ok(())
    }
    pub(crate) fn stacks(&self, remaining: u32) -> u32 {
        remaining.div_ceil(self.segment_health)
    }
    pub(crate) fn top_health(&self, remaining: u32) -> u32 {
        if remaining == 0 {
            0
        } else {
            remaining - (self.stacks(remaining) - 1) * self.segment_health
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentedShieldView {
    pub pool_id: String,
    pub segment_health: u32,
    pub max_segments: u32,
    pub remaining_health: u32,
    pub stacks: u32,
    pub top_health: u32,
    pub block_bonus: u32,
}
crate::authoring::clone_shape::struct_shape!(SegmentedShieldView {
    pool_id,
    segment_health,
    max_segments,
    remaining_health,
    stacks,
    top_health,
    block_bonus
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentDetachReceipt {
    pub activation: u64,
    pub owner: crate::game::battle::ids::UnitInstanceId,
    pub pool_id: String,
    pub time_ms: u64,
    pub before: u32,
    pub detached: u32,
    pub after: u32,
    pub fragment_copy_uuid: Option<uuid::Uuid>,
    pub equipment_runtime_uuid: Option<uuid::Uuid>,
}
crate::authoring::clone_shape::struct_shape!(SegmentDetachReceipt {
    activation,
    owner,
    pool_id,
    time_ms,
    before,
    detached,
    after,
    fragment_copy_uuid,
    equipment_runtime_uuid
});

/// Replaces only acquisition range and delivery. Weapon identity, damage type,
/// windup, interval, air capability and all equipment traits remain unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasicAttackOverlayProfile {
    pub range_policy: crate::game::battle::tile_range::TileRangePolicy,
    pub defense_tile_range: Option<crate::game::battle::tile_range::TileRangePattern>,
    pub delivery: super::DeliveryDef,
}
impl BasicAttackOverlayProfile {
    pub(crate) fn supports_action(
        &self,
        action: &crate::game::data::abnormality_data::BasicAttackAction,
    ) -> bool {
        !action.is_heal() || matches!(self.delivery, super::DeliveryDef::Instant)
    }

    pub fn validate(&self) -> Result<(), String> {
        use super::*;
        use crate::game::battle::tile_range::TileRangePolicy;
        match (self.range_policy, &self.defense_tile_range) {
            (TileRangePolicy::Pattern, Some(pattern)) => pattern.validate()?,
            (TileRangePolicy::WholeFieldValidTiles, None) => {}
            _ => return Err(
                "overlay requires an explicit pattern or WholeFieldValidTiles without a pattern"
                    .into(),
            ),
        }
        self.delivery
            .try_validate_projectile_scalar_contract()
            .map_err(|e| e.cause.to_owned())?;
        match &self.delivery {
            DeliveryDef::Instant => Ok(()),
            DeliveryDef::Projectile {
                hit_policy: ProjectileHitPolicy::TargetLocked,
                ..
            } => self
                .delivery
                .try_validate_basic_attack_consumer_contract()
                .map_err(|e| e.cause.to_owned()),
            DeliveryDef::Projectile {
                allow_targetless_cast,
                max_range_tiles,
                max_lifetime_ms,
                collision,
                ..
            } => {
                if *allow_targetless_cast
                    || max_range_tiles.is_none()
                    || max_lifetime_ms.is_none()
                    || collision.hit_targets != SkillHitTargetFilter::Enemies
                {
                    return Err("directional basic overlay requires a locked aim, finite range/lifetime and enemy collision".into());
                }
                Ok(())
            }
            DeliveryDef::TileArea { footprint, area } => {
                footprint.validate()?;
                area.try_validate_runtime_contract()?;
                if area.duration_ms != 0
                    || area.tick_interval_ms.is_some()
                    || area.tracking != SkillAreaTracking::GroundFixed
                    || area.tick_policy != SkillAreaTickPolicy::EveryTick
                    || area.hit_targets != SkillHitTargetFilter::Enemies
                    || area.include_caster
                    || !matches!(
                        area.anchor,
                        SkillAreaAnchorSource::Caster
                            | SkillAreaAnchorSource::CastTarget
                            | SkillAreaAnchorSource::CastTargetStart
                    )
                {
                    return Err("basic area overlay requires an instantaneous enemy footprint at caster or current target".into());
                }
                Ok(())
            }
        }
    }
    pub(crate) fn apply(&self, basic: &mut crate::game::data::abnormality_data::BasicAttackDef) {
        basic.range_policy = self.range_policy;
        basic.defense_tile_range = self.defense_tile_range.clone();
        basic.delivery = self.delivery.clone();
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelAuraDef {
    pub family: String,
    pub priority: i32,
    pub radius_units: u32,
    pub basic_damage_percent: u32,
    pub basic_speed_points: u32,
    #[serde(default, skip_serializing_if = "SupportCoefficientScope::is_legacy")]
    pub basic_scope: SupportCoefficientScope,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportCoefficientScope {
    #[default]
    LegacyBasic,
    RegularOnly,
}
impl SupportCoefficientScope {
    pub fn is_legacy(&self) -> bool {
        *self == Self::LegacyBasic
    }
}
crate::authoring::clone_shape::copy_shape!(SupportCoefficientScope);
crate::authoring::clone_shape::struct_shape!(ChannelAuraDef {
    family,
    priority,
    radius_units,
    basic_damage_percent,
    basic_speed_points,
    basic_scope
});
impl SustainedEffectDef {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::SegmentedShield { duration_ms, pool } => {
                if *duration_ms == 0 {
                    return Err("SegmentedShield requires positive duration".into());
                }
                pool.validate()
            }
            Self::CombatModifiers {
                duration_ms,
                modifiers,
            } => {
                if *duration_ms == 0
                    || modifiers.damage_reduction_percent > 100
                    || modifiers.basic_lifesteal_percent > 100
                    || (modifiers.damage_reduction_percent == 0
                        && modifiers.basic_lifesteal_percent == 0)
                    || (modifiers.basic_lifesteal_percent == 0)
                        != (modifiers.basic_lifesteal_max_heal == 0)
                    || modifiers.basic_lifesteal_max_heal > i32::MAX as u32
                {
                    return Err("CombatModifiers requires positive duration, nonzero effect, percentages in 0..100 and lifesteal cap in 1..i32::MAX only when lifesteal is enabled".into());
                }
                Ok(())
            }
            Self::Parry {
                duration_ms,
                window_ms,
                invulnerable_ms,
                miss_recovery_ms,
                reduction_percent,
                counter_skill,
            } => {
                if *window_ms == 0
                    || *invulnerable_ms == 0
                    || *miss_recovery_ms == 0
                    || *reduction_percent > 100
                    || counter_skill.as_str().is_empty()
                    || u64::from(*duration_ms)
                        < u64::from(*window_ms)
                            + u64::from((*invulnerable_ms).max(*miss_recovery_ms))
                {
                    return Err("Parry requires positive bounded windows, duration covering success/miss recovery, reduction in 0..100 and a counter skill".into());
                }
                Ok(())
            }
            Self::Control { duration_ms, kind } => {
                if *duration_ms == 0
                    || matches!(kind, ControlKind::Slow { percent } if *percent == 0 || *percent > 99)
                {
                    return Err(
                        "Control requires positive duration and Slow percent in 1..99".into(),
                    );
                }
                Ok(())
            }
            Self::Survival { duration_ms, .. } => {
                if *duration_ms == 0 {
                    return Err("Survival requires positive duration".into());
                }
                Ok(())
            }
            Self::Shield {
                duration_ms,
                amount,
                shared_radius_units,
                ..
            } => {
                if *duration_ms == 0 || *amount == 0 || *shared_radius_units == Some(0) {
                    return Err(
                        "Shield requires positive duration, capacity and optional shared radius"
                            .into(),
                    );
                }
                Ok(())
            }
            Self::BasicAttackOverlay {
                duration_ms,
                profile,
                ..
            } => {
                if *duration_ms == 0 {
                    return Err("overlay requires positive duration".into());
                }
                profile.validate()
            }
            Self::ChannelAura { duration_ms, aura } => {
                if *duration_ms == 0
                    || aura.family.trim().is_empty()
                    || aura.family.len() > 128
                    || aura.radius_units == 0
                    || aura.basic_damage_percent > 10000
                    || aura.basic_speed_points > 10000
                {
                    return Err("ChannelAura requires positive duration/radius, a bounded family, and damage/speed in 0..10000".into());
                }
                Ok(())
            }
            Self::Modifiers {
                duration_ms,
                modifiers,
            } => {
                if *duration_ms == 0 || modifiers.is_empty() || modifiers.len() > 32 {
                    return Err(
                        "Sustained Modifiers requires positive duration and 1..32 modifiers".into(),
                    );
                }
                if modifiers.iter().any(|m| m.stat == StatId::MaxHealth) {
                    return Err(
                        "Sustained MaxHealth is unsupported; HP is not a reversible stat".into(),
                    );
                }
                Ok(())
            }
        }
    }
    pub(crate) fn is_hostile(&self) -> bool {
        match self {
            Self::Control { .. } => true,
            Self::Parry { .. }
            | Self::Survival { .. }
            | Self::Shield { .. }
            | Self::SegmentedShield { .. }
            | Self::ChannelAura { .. }
            | Self::CombatModifiers { .. }
            | Self::BasicAttackOverlay { .. } => false,
            Self::Modifiers { modifiers, .. } => modifiers.iter().any(|m| {
                if m.stat == StatId::AttackIntervalMs {
                    m.value > 0
                } else {
                    m.value < 0
                }
            }),
        }
    }
}

/// Sum flats and percentage points before narrowing; cancellation never uses
/// inverse arithmetic. All supported properties share this one composition rule.
pub(crate) fn compose(
    base: crate::game::stats::UnitStats,
    modifiers: impl Iterator<Item = StatModifier>,
) -> crate::game::stats::UnitStats {
    let mut totals = [[0i128; 2]; 6];
    for m in modifiers {
        let index = match m.stat {
            StatId::Attack => 0,
            StatId::SpellPower => 1,
            StatId::Defense => 2,
            StatId::MagicResist => 3,
            StatId::AttackIntervalMs => 4,
            StatId::MoveSpeedUnitsPerMs => 5,
            StatId::MaxHealth => continue,
        };
        totals[index][usize::from(m.kind == StatModifierKind::Percent)] += i128::from(m.value);
    }
    let apply = |index: usize, value: i128, min: i128, max: i128| {
        let flat = value + totals[index][0];
        (flat + flat * totals[index][1] / 100).clamp(min, max)
    };
    let mut result = base;
    result.attack = apply(0, base.attack.into(), 0, u32::MAX.into()) as u32;
    result.spell_power = apply(1, base.spell_power.into(), 0, u32::MAX.into()) as u32;
    result.defense = apply(2, base.defense.into(), i32::MIN.into(), i32::MAX.into()) as i32;
    result.magic_resist = apply(
        3,
        base.magic_resist.into(),
        i32::MIN.into(),
        i32::MAX.into(),
    ) as i32;
    result.attack_interval_ms = apply(4, base.attack_interval_ms.into(), 1, u64::MAX.into()) as u64;
    result.move_speed_units_per_ms =
        apply(5, base.move_speed_units_per_ms.into(), 0, u32::MAX.into()) as u32;
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SustainedEndReason {
    Expired,
    Stopped,
    Interrupted,
    HardCrowdControl,
    SourceUnavailable,
    TargetUnavailable,
    Terminal,
    AttackReleased,
    Depleted,
    Triggered,
    Reinput,
    Replaced,
}
crate::authoring::clone_shape::copy_shape!(SustainedEndReason);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SustainedEndReceipt {
    pub activation: u64,
    pub caster: crate::game::battle::ids::UnitInstanceId,
    pub target: crate::game::battle::ids::UnitInstanceId,
    pub time_ms: u64,
    pub reason: SustainedEndReason,
}
crate::authoring::clone_shape::copy_shape!(SustainedEndReceipt);
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersonalShieldGrantView {
    pub effect: crate::game::relic::structural::EffectiveEffectKey,
    pub fragment_copy_uuid: Option<uuid::Uuid>,
    pub equipment_copy_uuid: Option<uuid::Uuid>,
    pub legacy_recharge_supported: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixedSupportConnectionState {
    InRange,
    OutOfRange,
    TerminalInvalid,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixedSupportConnectionView {
    pub recipient: crate::game::battle::ids::UnitInstanceId,
    pub authored_reach_units: u32,
    pub state: FixedSupportConnectionState,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SustainedEffectView {
    pub activation: u64,
    pub caster: crate::game::battle::ids::UnitInstanceId,
    pub target: crate::game::battle::ids::UnitInstanceId,
    pub expires_at_ms: u64,
    pub is_channel: bool,
    pub shield: Option<ShieldPoolView>,
    #[serde(default, skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub personal_grant: Option<PersonalShieldGrantView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_connection: Option<FixedSupportConnectionView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segmented: Option<SegmentedShieldView>,
    pub survival: Option<SurvivalPolicy>,
    pub control: Option<ControlKind>,
    pub parry: Option<ParryView>,
    pub overlay: Option<BasicAttackOverlayProfile>,
    pub next_attack_only: bool,
    pub goodbye_skill: Option<super::SkillId>,
    pub modifiers: Vec<StatModifier>,
    pub combat_modifiers: Option<CombatModifierProfile>,
    pub members: Vec<crate::game::battle::ids::UnitInstanceId>,
    pub authored_site: Option<super::authoring::EffectSite>,
    pub skill_id: Option<super::SkillId>,
    pub cast_sequence: Option<u64>,
    pub flow_occurrence: Option<(usize, usize)>,
    pub fragment_copy_uuid: Option<uuid::Uuid>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SustainedEffectsSnapshot {
    pub started: u64,
    pub ended: u64,
    pub latest_end: Option<SustainedEndReceipt>,
    pub latest_recharge: Option<ShieldRechargeReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_detachment: Option<SegmentDetachReceipt>,
    pub active: Vec<SustainedEffectView>,
    #[serde(default)]
    pub passive_guards: Vec<PassiveGuardView>,
}

/// One activation owns this pool; members never own a copy of its balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShieldPoolView {
    pub capacity: u32,
    pub remaining: u32,
    pub priority: i32,
    pub shared_radius_units: Option<u32>,
}
crate::authoring::clone_shape::copy_shape!(ShieldPoolView);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurvivalPolicy {
    FloorOne,
    PreventDeathOnce,
}
crate::authoring::clone_shape::copy_shape!(SurvivalPolicy);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShieldRechargeReceipt {
    pub activation: u64,
    pub time_ms: u64,
    pub before: u32,
    pub after: u32,
    pub caster: Option<crate::game::battle::ids::UnitInstanceId>,
    pub skill_id: Option<super::SkillId>,
    pub cast_sequence: Option<u64>,
    pub authored_site: Option<super::authoring::EffectSite>,
    pub fragment_copy_uuid: Option<uuid::Uuid>,
    pub equipment_copy_uuid: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlKind {
    Slow { percent: u32 },
    Root,
    Taunt { priority: i32 },
}
crate::authoring::clone_shape::copy_shape!(ControlKind);

/// Absolute boundaries are half-open. A miss disables only this activation's
/// reduction during [window_until, recovery_at); success does not rearm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParryView {
    pub window_until_ms: u64,
    pub miss_recovery_at_ms: u64,
    pub invulnerable_ms: u32,
    pub invulnerable_until_ms: Option<u64>,
    pub reduction_percent: u32,
    pub counter_skill: super::SkillId,
}
crate::authoring::clone_shape::struct_shape!(ParryView {
    window_until_ms,
    miss_recovery_at_ms,
    invulnerable_ms,
    invulnerable_until_ms,
    reduction_percent,
    counter_skill
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassiveGuardView {
    pub unit: crate::game::battle::ids::UnitInstanceId,
    pub fragment: crate::game::skill_fragment::FragmentSourceSnapshot,
    pub policy: crate::game::data::skill_fragment_data::ParryGuardDef,
    /// Half-open battle-clock interval. A future window predicts a miss;
    /// success removes it. Queries do not advance the battle clock.
    pub reduction_suppressed_intervals_ms: Vec<(u64, u64)>,
}
