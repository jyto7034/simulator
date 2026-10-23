//! Independent non-blocking effects, composed into source-owned fusion families.
use super::{marks::MarkRatio, resonance::ResonanceAdmission};
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MobilePower {
    Weapon {
        coefficient: MarkRatio,
    },
    Add {
        base: Box<MobilePower>,
        amount: u32,
    },
    Fusion {
        base: Box<MobilePower>,
        baseline: u32,
        percent: MarkRatio,
    },
}
impl MobilePower {
    pub fn validate(&self) -> Result<(), String> { self.validate_depth(0) }
    fn validate_depth(&self, depth: usize) -> Result<(), String> {
        if depth > 32 {
            return Err("Mobile power expression is too deep".into());
        }
        match self {
            Self::Weapon { coefficient } => coefficient.validate(false),
            Self::Add { base, amount } => {
                if *amount == 0 {
                    return Err("Additional power must be positive".into());
                }
                base.validate_depth(depth + 1)
            }
            Self::Fusion {
                base,
                baseline,
                percent,
            } => {
                if *baseline == 0 {
                    return Err("Fusion baseline must be positive".into());
                }
                percent.validate(true)?;
                base.validate_depth(depth + 1)
            }
        }
    }
    pub(crate) fn resolve(&self, attack: u32, units: u32) -> Result<u32, String> {
        let value = match self {
            Self::Weapon { coefficient: r } => {
                (u128::from(attack) * u128::from(r.numerator)).div_ceil(u128::from(r.denominator))
            }
            Self::Add { base, amount } => {
                u128::from(base.resolve(attack, units)?) + u128::from(*amount)
            }
            Self::Fusion {
                base,
                baseline,
                percent: r,
            } => {
                let denominator = 100 * u128::from(r.denominator);
                let factor = denominator
                    .checked_add(
                        u128::from(units.saturating_sub(*baseline)) * u128::from(r.numerator),
                    )
                    .ok_or("Fusion power overflow")?;
                u128::from(base.resolve(attack, units)?)
                    .checked_mul(factor)
                    .ok_or("Fusion power overflow")?
                    .div_ceil(denominator)
            }
        };
        u32::try_from(value).map_err(|_| "Mobile power exceeds u32".into())
    }
    pub(crate) fn heap_extent(&self) -> Option<usize> {
        match self {
            Self::Weapon { .. } => Some(0),
            Self::Add { base, .. } | Self::Fusion { base, .. } => {
                std::mem::size_of::<Self>().checked_add(base.heap_extent()?)
            }
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobileMotion {
    WaitLeap,
    ChaseDelay,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MobileChange {
    Power {
        percent: MarkRatio,
    },
    Range {
        pattern: TileRangePattern,
    },
    Vulnerability {
        percent: MarkRatio,
        duration_ms: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MobileModifier {
    pub id: String,
    pub relic_definition: String,
    pub change: MobileChange,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MobileProfile {
    pub definition: String,
    pub motion: MobileMotion,
    pub detection: TileRangePattern,
    pub range: TileRangePattern,
    pub power: MobilePower,
    pub damage_type: DamageType,
    /// Millitiles per second; authoring must be exactly representable.
    pub speed_units_per_second: u32,
    pub delay_ms: u32,
    pub lifetime_ms: Option<u32>,
    pub modifiers: Vec<MobileModifier>,
}
fn identity(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 256
}
impl MobileProfile {
    pub fn validate(&self) -> Result<(), String> {
        if !identity(&self.definition)
            || self.speed_units_per_second == 0
            || self.delay_ms == 0
            || self.lifetime_ms == Some(0)
            || self.modifiers.len() > 64
        {
            return Err("Invalid mobile identity, speed, lifetime or modifier count".into());
        }
        if self.motion == MobileMotion::WaitLeap && self.lifetime_ms.is_none() {
            return Err("Waiting mobile effect needs a finite lifetime".into());
        }
        if !matches!(self.damage_type, DamageType::Physical | DamageType::Magic) {
            return Err("Unsupported mobile damage type".into());
        }
        self.detection.validate()?;
        self.range.validate()?;
        self.power.validate_depth(0)?;
        let mut ids = std::collections::BTreeSet::new();
        for m in &self.modifiers {
            if !identity(&m.id) || !identity(&m.relic_definition) || !ids.insert(&m.id) {
                return Err("Invalid mobile modifier identity".into());
            }
            match &m.change {
                MobileChange::Power { percent } => percent.validate(true)?,
                MobileChange::Range { pattern } => pattern.validate()?,
                MobileChange::Vulnerability {
                    percent,
                    duration_ms,
                } => {
                    percent.validate(true)?;
                    if *duration_ms == 0 {
                        return Err("Vulnerability duration must be positive".into());
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        use crate::game::relic::structural::support::pattern_extent;
        let mut n = std::mem::size_of::<Self>()
            .checked_add(self.definition.capacity())?
            .checked_add(self.power.heap_extent()?)?
            .checked_add(pattern_extent(&self.range)?)?
            .checked_add(pattern_extent(&self.detection)?)?
            .checked_add(
                self.modifiers
                    .capacity()
                    .checked_mul(std::mem::size_of::<MobileModifier>())?,
            )?;
        for m in &self.modifiers {
            n = n
                .checked_add(m.id.capacity())?
                .checked_add(m.relic_definition.capacity())?;
            if let MobileChange::Range { pattern } = &m.change {
                n = n.checked_add(pattern_extent(pattern)?)?;
            }
        }
        Some(n)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FusionTier {
    pub minimum: u32,
    pub profile: MobileProfile,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedDurationModifier {
    pub id: String,
    pub relic_definition: String,
    pub negative: bool,
    pub percent: MarkRatio,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FusionSeed {
    pub definition: String,
    pub relic_definition: String,
    pub duration_ms: u32,
    pub modifiers: Vec<SeedDurationModifier>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FusionPlacementDef {
    pub definition: String,
    pub admission: ResonanceAdmission,
    pub selection_range: TileRangePattern,
    pub primary: u32,
    pub adjacent: u32,
    pub max_units: u32,
    pub max_tiles: u32,
    pub tiers: Vec<FusionTier>,
    pub seed: Option<FusionSeed>,
}
impl FusionPlacementDef {
    pub fn validate(&self) -> Result<(), String> {
        self.admission.validate().map_err(str::to_owned)?;
        self.selection_range.validate()?;
        if !identity(&self.definition)
            || self.primary == 0
            || self.adjacent == 0
            || self.primary.checked_add(self.adjacent).is_none()
            || self.max_units < self.primary.max(self.adjacent)
            || self.max_tiles == 0
            || self.max_tiles > 4096
            || self.tiers.is_empty()
            || self.tiers.len() > 64
        {
            return Err("Invalid fusion placement limits".into());
        }
        let mut last = 0;
        for (i, t) in self.tiers.iter().enumerate() {
            if t.minimum <= last || t.minimum > self.max_units || (i == 0 && t.minimum != 1) {
                return Err(
                    "Fusion tiers must start at one and be strictly ordered within the cap".into(),
                );
            }
            last = t.minimum;
            t.profile.validate()?;
            if t.profile.motion != MobileMotion::WaitLeap {
                return Err("Fusion only merges waiting effects".into());
            }
        }
        if let Some(s) = &self.seed {
            if !identity(&s.definition)
                || !identity(&s.relic_definition)
                || s.duration_ms == 0
                || s.modifiers.len() > 64
            {
                return Err("Invalid fusion death seed".into());
            }
            let mut ids = std::collections::BTreeSet::new();
            for m in &s.modifiers {
                if !identity(&m.id) || !identity(&m.relic_definition) || !ids.insert(&m.id) {
                    return Err("Invalid seed modifier".into());
                }
                m.percent.validate(true)?;
                if m.negative
                    && u128::from(m.percent.numerator) >= 100 * u128::from(m.percent.denominator)
                {
                    return Err("Seed duration must remain positive".into());
                }
            }
        }
        Ok(())
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>()
            .checked_add(self.definition.capacity())?
            .checked_add(crate::game::relic::structural::support::pattern_extent(
                &self.selection_range,
            )?)?
            .checked_add(
                self.tiers
                    .capacity()
                    .checked_mul(std::mem::size_of::<FusionTier>())?,
            )?;
        for t in &self.tiers {
            n = n.checked_add(t.profile.retained_extent()?)?;
        }
        if let Some(s) = &self.seed {
            n = n
                .checked_add(s.definition.capacity())?
                .checked_add(s.relic_definition.capacity())?
                .checked_add(
                    s.modifiers
                        .capacity()
                        .checked_mul(std::mem::size_of::<SeedDurationModifier>())?,
                )?;
            for m in &s.modifiers {
                n = n
                    .checked_add(m.id.capacity())?
                    .checked_add(m.relic_definition.capacity())?;
            }
        }
        Some(n)
    }
}
crate::authoring::clone_shape::struct_shape!(MobileModifier {
    id,
    relic_definition,
    change
});
crate::authoring::clone_shape::struct_shape!(MobileProfile {
    definition,
    motion,
    detection,
    range,
    power,
    damage_type,
    speed_units_per_second,
    delay_ms,
    lifetime_ms,
    modifiers
});
crate::authoring::clone_shape::struct_shape!(FusionTier { minimum, profile });
crate::authoring::clone_shape::struct_shape!(SeedDurationModifier {
    id,
    relic_definition,
    negative,
    percent
});
crate::authoring::clone_shape::struct_shape!(FusionSeed {
    definition,
    relic_definition,
    duration_ms,
    modifiers
});
crate::authoring::clone_shape::struct_shape!(FusionPlacementDef {
    definition,
    admission,
    selection_range,
    primary,
    adjacent,
    max_units,
    max_tiles,
    tiers,
    seed
});
impl crate::authoring::clone_shape::CloneShape for MobileMotion {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(std::mem::size_of::<Self>()))
    }
}
impl crate::authoring::clone_shape::CloneShape for MobilePower {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(std::mem::size_of::<Self>()))?;
        match self {
            Self::Weapon { .. } => Ok(()),
            Self::Add { base, .. } | Self::Fusion { base, .. } => base.visit_clone_shape(m),
        }
    }
}
impl crate::authoring::clone_shape::CloneShape for MobileChange {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(std::mem::size_of::<Self>()))?;
        match self {
            Self::Range { pattern } => pattern.visit_clone_shape(m),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileDeathOrigin {pub target:crate::game::battle::ids::UnitInstanceId,pub relic:super::sustained::HitModifierSource,pub states:Vec<crate::game::battle::runtime_facts::StatusPredecessor>}
crate::authoring::clone_shape::struct_shape!(MobileDeathOrigin{target,relic,states});
impl MobileDeathOrigin {pub(crate) fn retained_extent(&self)->Option<usize>{let mut n=std::mem::size_of::<Self>().checked_add(64)?.checked_add(self.relic.definition.capacity())?.checked_add(self.relic.rule.capacity())?.checked_add(self.states.capacity().checked_mul(std::mem::size_of::<crate::game::battle::runtime_facts::StatusPredecessor>())?)?;for s in &self.states{n=n.checked_add(s.retained_extent()?)?;}Some(n)}}
/// Immediate parent effect and the equipped relic copy responsible for a death seed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileSeedOrigin {
    pub parent_activation: u64,
    pub relic: super::sustained::HitModifierSource,
}
crate::authoring::clone_shape::struct_shape!(MobileSeedOrigin {
    parent_activation,
    relic
});
impl MobileSeedOrigin {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(64)?
            .checked_add(self.relic.definition.capacity())?
            .checked_add(self.relic.rule.capacity())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobilePhaseView {
    Chasing { x_milli:i64, y_milli:i64, target:Option<crate::game::battle::ids::UnitInstanceId>, explodes_at_ms:u64 },
    Waiting,
    Leaping {
        destination: crate::game::resources::Position,
        started_at_ms: u64,
        arrives_at_ms: u64,
        explodes_at_ms: u64,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileEffectView {
    #[serde(default)]
    pub death_origin:Option<std::sync::Arc<MobileDeathOrigin>>,
    #[serde(default)]
    pub seed_origin: Option<std::sync::Arc<MobileSeedOrigin>>,
    pub activation: u64,
    pub owner: crate::game::battle::ids::UnitInstanceId,
    pub copy: uuid::Uuid,
    pub family: String,
    pub profile: String,
    pub units: u32,
    pub tile: crate::game::resources::Position,
    pub created_at_ms: u64,
    pub waiting_expires_at_ms: u64,
    pub phase: MobilePhaseView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileSeedView {
    pub origin: std::sync::Arc<MobileSeedOrigin>,
    pub owner: crate::game::battle::ids::UnitInstanceId,
    pub copy: uuid::Uuid,
    pub target: crate::game::battle::ids::UnitInstanceId,
    pub definition: String,
    pub expires_at_ms: u64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileEffectsSnapshot {
    pub effects: Vec<MobileEffectView>,
    pub seeds: Vec<MobileSeedView>,
}
