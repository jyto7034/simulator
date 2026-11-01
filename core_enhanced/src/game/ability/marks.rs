//! Source-owned deployment marks and their independent continuous power clock.
//! Numeric authoring remains exact through the native integer-damage boundary.
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkRatio {
    pub numerator: u64,
    pub denominator: u64,
}
impl MarkRatio {
    pub fn validate(self, zero: bool) -> Result<(), String> {
        if self.denominator == 0 || (!zero && self.numerator == 0) {
            return Err("Invalid mark numeric ratio".into());
        }
        Ok(())
    }
    pub fn at_most(self, value: u32) -> bool {
        u128::from(self.numerator) <= u128::from(value) * u128::from(self.denominator)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkFilter {
    pub air: bool,
    pub boss: bool,
    pub abnormality: bool,
    pub summon: bool,
    pub temporary: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationPowerDef {
    pub growth_per_second: MarkRatio,
    pub cap: MarkRatio,
    pub decay_per_second: MarkRatio,
    pub grace_ms: u32,
    pub prevent_decay: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresenceMarksDef {
    pub definition: String,
    pub filter: MarkFilter,
    pub chance_percent: MarkRatio,
    pub spread_radius_units: u32,
    pub spread_limit: u32,
    pub power: PopulationPowerDef,
    pub periodic_interval_ms: u32,
    pub periodic_coefficient: MarkRatio,
    pub periodic_damage_type: DamageType,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplosionDedup {
    Wave,
    Explosion,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkExplosionDef {
    pub range: TileRangePattern,
    pub coefficient: MarkRatio,
    pub host_multiplier: MarkRatio,
    pub damage_type: DamageType,
    pub bonus_percent: MarkRatio,
    pub dedup: ExplosionDedup,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MarkChange {
    Period { decrease: bool, percent: MarkRatio },
    Growth { percent: MarkRatio },
    PreservePower,
    Range { pattern: TileRangePattern },
    ExplosionPower { percent: MarkRatio },
    ExplosionOverlap,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkModifier {
    pub id: String,
    pub relic_definition: String,
    pub change: MarkChange,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresenceBurstProgram {
    pub marks: PresenceMarksDef,
    pub explosion: MarkExplosionDef,
    pub modifiers: Vec<MarkModifier>,
}
fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
impl PopulationPowerDef {
    /// All authored rates share an exact integer unit. Including 1000 makes
    /// every millisecond integral without rounding at event boundaries.
    pub fn scale(&self) -> Result<u64, String> {
        let mut scale = 1u64;
        for r in [self.growth_per_second, self.cap, self.decay_per_second] {
            r.validate(true)?;
            scale = (scale / gcd(scale, r.denominator))
                .checked_mul(r.denominator)
                .ok_or("Mark power precision exceeds native integer scale")?;
        }
        scale
            .checked_mul(1000)
            .ok_or_else(|| "Mark power millisecond precision exceeds native integer scale".into())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.growth_per_second.validate(true)?;
        self.cap.validate(false)?;
        self.decay_per_second.validate(true)?;
        if !self.cap.at_most(i32::MAX as u32) {
            return Err("Mark power cap exceeds native damage amount".into());
        }
        self.scale()?;
        Ok(())
    }
}
impl PresenceBurstProgram {
    pub fn validate(&self) -> Result<(), String> {
        let m = &self.marks;
        if m.definition.is_empty()
            || m.definition.len() > 256
            || m.spread_limit == 0
            || m.periodic_interval_ms == 0
        {
            return Err(
                "Presence marks require identity, spread count and positive damage cadence".into(),
            );
        }
        m.chance_percent.validate(true)?;
        if !m.chance_percent.at_most(100) {
            return Err("Mark chance must be between zero and one hundred".into());
        }
        m.power.validate()?;
        m.periodic_coefficient.validate(false)?;
        self.explosion.range.validate()?;
        self.explosion.coefficient.validate(false)?;
        self.explosion.bonus_percent.validate(true)?;
        self.explosion.host_multiplier.validate(false)?;
        if !self.explosion.host_multiplier.at_most(4)
            || u128::from(self.explosion.host_multiplier.numerator)
                < 2 * u128::from(self.explosion.host_multiplier.denominator)
        {
            return Err("Own-host burst multiplier must be between two and four".into());
        }
        if self.modifiers.len() > 64 {
            return Err("Too many presence modifiers".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for m in &self.modifiers {
            if m.id.is_empty()
                || m.id.len() > 128
                || m.relic_definition.is_empty()
                || m.relic_definition.len() > 128
                || !ids.insert(&m.id)
            {
                return Err("Invalid presence modifier identity".into());
            }
            match &m.change {
                MarkChange::Period { decrease, percent } => {
                    percent.validate(true)?;
                    if *decrease && u128::from(percent.numerator) >= 100 * u128::from(percent.denominator) {
                        return Err("Period reduction must remain below 100 percent".into());
                    }
                }
                MarkChange::Growth { percent } | MarkChange::ExplosionPower { percent } => {
                    percent.validate(true)?
                }
                MarkChange::Range { pattern } => pattern.validate()?,
                _ => (),
            }
        }
        Ok(())
    }
}
crate::authoring::clone_shape::struct_shape!(MarkRatio {
    numerator,
    denominator
});
crate::authoring::clone_shape::struct_shape!(MarkFilter {
    air,
    boss,
    abnormality,
    summon,
    temporary
});
crate::authoring::clone_shape::struct_shape!(PopulationPowerDef {
    growth_per_second,
    cap,
    decay_per_second,
    grace_ms,
    prevent_decay
});
crate::authoring::clone_shape::struct_shape!(PresenceMarksDef {
    definition,
    filter,
    chance_percent,
    spread_radius_units,
    spread_limit,
    power,
    periodic_interval_ms,
    periodic_coefficient,
    periodic_damage_type
});
crate::authoring::clone_shape::struct_shape!(MarkExplosionDef {
    range,
    coefficient,
    host_multiplier,
    damage_type,
    bonus_percent,
    dedup
});
crate::authoring::clone_shape::struct_shape!(MarkModifier {
    id,
    relic_definition,
    change
});
crate::authoring::clone_shape::struct_shape!(PresenceBurstProgram {
    marks,
    explosion,
    modifiers
});
impl crate::authoring::clone_shape::CloneShape for ExplosionDedup {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(1))
    }
}
impl crate::authoring::clone_shape::CloneShape for MarkChange {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(1))?;
        match self {
            Self::Period { percent, .. } | Self::Growth { percent } => percent.visit_clone_shape(m),
            Self::Range { pattern } => pattern.visit_clone_shape(m),
            Self::ExplosionPower { percent } => percent.visit_clone_shape(m),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedMarkModifier {
    pub rule_id: String,
    pub relic_definition: String,
    pub relic_copy_uuid: uuid::Uuid,
}
crate::authoring::clone_shape::struct_shape!(AppliedMarkModifier {
    rule_id,
    relic_definition,
    relic_copy_uuid
});
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresenceHostView {
    pub unit: crate::game::battle::ids::UnitInstanceId,
    pub next_damage_at_ms: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresenceNetworkView {
    pub caster: crate::game::battle::ids::UnitInstanceId,
    pub fragment: crate::game::skill_fragment::FragmentSourceSnapshot,
    pub definition: String,
    /// Decimal integer text preserves exact values across JSON consumers.
    pub power_units: String,
    pub power_scale: u64,
    pub empty_since_ms: Option<u64>,
    pub hosts: Vec<PresenceHostView>,
    pub applied_modifiers: Vec<AppliedMarkModifier>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingPresenceBurstView {
    pub caster: crate::game::battle::ids::UnitInstanceId,
    pub fragment_copy_uuid: uuid::Uuid,
    pub action_ordinal: u64,
    pub centers: Vec<(
        crate::game::battle::ids::UnitInstanceId,
        crate::game::resources::Position,
    )>,
    pub ordinary_damage: u32,
    pub host_damage: u32,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresenceMarksSnapshot {
    pub networks: Vec<PresenceNetworkView>,
    pub pending_bursts: Vec<PendingPresenceBurstView>,
}
