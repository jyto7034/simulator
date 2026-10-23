//! Owned damage subscriptions and pre-hit state transitions with a shared mobile follow-up.
use super::{marks::MarkRatio, mobile_effect::*, resonance::ResonanceAdmission};
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactiveKind {
    Slow { percent: u32 },
    Freeze,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ReactiveChange {
    Duration {
        negative: bool,
        percent: MarkRatio,
    },
    Vulnerability {
        percent: MarkRatio,
        duration_ms: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReactiveModifier {
    pub id: String,
    pub relic_definition: String,
    pub change: ReactiveChange,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReactiveStatusDef {
    pub definition: String,
    pub kind: ReactiveKind,
    pub duration_ms: u32,
    pub refresh: bool,
    pub modifiers: Vec<ReactiveModifier>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusDeathMobileDef {
    pub definition: String,
    pub relic_definition: String,
    pub states: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusStrikeDef {
    pub definition: String,
    pub admission: ResonanceAdmission,
    pub range: TileRangePattern,
    pub range_modifiers: Vec<MobileModifier>,
    pub power: MobilePower,
    pub damage_type: DamageType,
    pub from: ReactiveStatusDef,
    pub to: ReactiveStatusDef,
    pub owned_damage: bool,
    pub mobile: MobileProfile,
    pub limit: u32,
    pub death: Option<StatusDeathMobileDef>,
}
fn identity(s: &str) -> bool {
    !s.is_empty() && s.trim() == s && s.len() <= 256
}
impl ReactiveStatusDef {
    pub fn validate(&self) -> Result<(), String> {
        if !identity(&self.definition)
            || self.duration_ms == 0
            || self.modifiers.len() > 64
            || matches!(self.kind,ReactiveKind::Slow{percent} if percent==0||percent>100)
        {
            return Err("Invalid reactive status".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for m in &self.modifiers {
            if !identity(&m.id) || !identity(&m.relic_definition) || !ids.insert(&m.id) {
                return Err("Invalid reactive modifier identity".into());
            }
            match m.change {
                ReactiveChange::Duration { percent, .. } => percent.validate(true)?,
                ReactiveChange::Vulnerability {
                    percent,
                    duration_ms,
                } => {
                    percent.validate(true)?;
                    if self.kind != ReactiveKind::Freeze || duration_ms == 0 {
                        return Err("Control vulnerability requires a successful freeze and finite duration".into());
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>()
            .checked_add(self.definition.capacity())?
            .checked_add(
                self.modifiers
                    .capacity()
                    .checked_mul(std::mem::size_of::<ReactiveModifier>())?,
            )?;
        for m in &self.modifiers {
            n = n
                .checked_add(m.id.capacity())?
                .checked_add(m.relic_definition.capacity())?;
        }
        Some(n)
    }
}
impl StatusStrikeDef {
    pub fn validate(&self) -> Result<(), String> {
        self.admission.validate()?;
        self.range.validate()?;
        self.from.validate()?;
        self.to.validate()?;
        self.mobile.validate()?;
        // The current authored transition is a refreshable slow followed by a non-refreshing hard control.
        if !identity(&self.definition)
            || !matches!(self.from.kind, ReactiveKind::Slow { .. })
            || !self.from.refresh
            || self.to.kind != ReactiveKind::Freeze
            || self.to.refresh
            || self.from.definition == self.to.definition
            || self.limit == 0
            || self.limit > 4096
            || self.mobile.motion != MobileMotion::ChaseDelay
            || self.mobile.lifetime_ms.is_some()
            || !matches!(self.damage_type, DamageType::Physical | DamageType::Magic)
            || self.range_modifiers.len() > 64
        {
            return Err("Invalid state transition or chase effect contract".into());
        }
        self.power.validate()?;
        let mut ids = std::collections::BTreeSet::new();
        for m in &self.range_modifiers {
            if !identity(&m.id) || !identity(&m.relic_definition) || !ids.insert(&m.id) {
                return Err("Invalid strike range modifier".into());
            }
            if let MobileChange::Range { pattern } = &m.change {
                pattern.validate()?;
            } else {
                return Err("Strike range accepts range modifiers only".into());
            }
        }
        if let Some(d) = &self.death {
            if !identity(&d.definition)
                || !identity(&d.relic_definition)
                || d.states.is_empty()
                || d.states.len() > 2
                || d.states
                    .iter()
                    .any(|s| s != &self.from.definition && s != &self.to.definition)
                || d.states
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != d.states.len()
            {
                return Err("Death generation requires distinct emitted state definitions".into());
            }
        }
        Ok(())
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        use crate::game::relic::structural::support::pattern_extent;
        let mut n = std::mem::size_of::<Self>()
            .checked_add(self.definition.capacity())?
            .checked_add(pattern_extent(&self.range)?)?
            .checked_add(self.power.heap_extent()?)?
            .checked_add(self.from.retained_extent()?)?
            .checked_add(self.to.retained_extent()?)?
            .checked_add(self.mobile.retained_extent()?)?
            .checked_add(
                self.range_modifiers
                    .capacity()
                    .checked_mul(std::mem::size_of::<MobileModifier>())?,
            )?;
        for m in &self.range_modifiers {
            n = n
                .checked_add(m.id.capacity())?
                .checked_add(m.relic_definition.capacity())?;
            if let MobileChange::Range { pattern } = &m.change {
                n = n.checked_add(pattern_extent(pattern)?)?;
            }
        }
        if let Some(d) = &self.death {
            n = n
                .checked_add(std::mem::size_of::<StatusDeathMobileDef>())?
                .checked_add(d.definition.capacity())?
                .checked_add(d.relic_definition.capacity())?
                .checked_add(
                    d.states
                        .capacity()
                        .checked_mul(std::mem::size_of::<String>())?,
                )?;
            for s in &d.states {
                n = n.checked_add(s.capacity())?;
            }
        }
        Some(n)
    }
}
crate::authoring::clone_shape::copy_shape!(ReactiveKind);
crate::authoring::clone_shape::copy_shape!(ReactiveChange);
crate::authoring::clone_shape::struct_shape!(ReactiveModifier {
    id,
    relic_definition,
    change
});
crate::authoring::clone_shape::struct_shape!(ReactiveStatusDef {
    definition,
    kind,
    duration_ms,
    refresh,
    modifiers
});
crate::authoring::clone_shape::struct_shape!(StatusDeathMobileDef {
    definition,
    relic_definition,
    states
});
crate::authoring::clone_shape::struct_shape!(StatusStrikeDef {
    definition,
    admission,
    range,
    range_modifiers,
    power,
    damage_type,
    from,
    to,
    owned_damage,
    mobile,
    limit,
    death
});
