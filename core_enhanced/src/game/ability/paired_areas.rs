//! Two fixed areas share a lifetime and a synchronous healing settlement.
use super::{marks::MarkRatio, resonance::ResonanceAdmission, sustained::TargetStateProfile};
use crate::game::battle::tile_range::TileRangePattern;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PairChange {
    Duration {
        negative: bool,
        percent: MarkRatio,
    },
    Range {
        first: bool,
        second: bool,
        pattern: TileRangePattern,
    },
    Healing {
        percent: MarkRatio,
    },
    Buff {
        percent: MarkRatio,
    },
    HealBuff {
        duration_ms: u32,
        profile: TargetStateProfile,
    },
    CoincidentBurst {
        delay_ms: u32,
        pattern: TileRangePattern,
        max_hp_percent: u32,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairModifier {
    pub id: String,
    pub relic_definition: String,
    pub change: PairChange,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedAreasDef {
    pub definition: String,
    pub admission: ResonanceAdmission,
    pub selection_range: TileRangePattern,
    pub first: TileRangePattern,
    pub second: TileRangePattern,
    pub duration_ms: u32,
    pub interval_ms: u32,
    pub healing: u32,
    pub transfer_percent: MarkRatio,
    pub intersection: TargetStateProfile,
    pub modifiers: Vec<PairModifier>,
}
impl PairedAreasDef {
    pub fn validate(&self) -> Result<(), String> {
        self.admission.validate().map_err(str::to_owned)?;
        if self.definition.trim().is_empty()
            || self.definition.len() > 256
            || self.duration_ms == 0
            || self.interval_ms == 0
            || self.healing > i32::MAX as u32
        {
            return Err("Invalid paired area identity, lifetime, cadence or healing".into());
        }
        for p in [&self.selection_range, &self.first, &self.second] {
            p.validate()?;
        }
        self.transfer_percent.validate(true)?;
        if !self.transfer_percent.at_most(100) {
            return Err("Transfer must be in 0..100 percent".into());
        }
        self.intersection.validate()?;
        if self.intersection.attack_speed_points == 0
            || self.intersection.attack_percent != 0
            || self.intersection.defense_percent != 0
            || self.intersection.hp_floor_one
            || !self.intersection.end_on_source_exit
        {
            return Err("Intersection is a source-owned basic attack speed contribution".into());
        }
        if self.modifiers.len() > 64 {
            return Err("Too many pair modifiers".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut bursts = 0;
        for m in &self.modifiers {
            if m.id.trim().is_empty()
                || m.id.len() > 128
                || m.relic_definition.len() > 256
                || m.relic_definition.trim().is_empty()
                || !ids.insert(&m.id)
            {
                return Err("Invalid pair modifier identity".into());
            }
            match &m.change {
                PairChange::Duration { negative, percent } => {
                    percent.validate(true)?;
                    if *negative
                        && u128::from(percent.numerator) >= 100 * u128::from(percent.denominator)
                    {
                        return Err("Duration must remain positive".into());
                    }
                }
                PairChange::Range {
                    first,
                    second,
                    pattern,
                } => {
                    if !first && !second {
                        return Err("Range modifier needs an area".into());
                    }
                    pattern.validate()?;
                }
                PairChange::Healing { percent } | PairChange::Buff { percent } => {
                    percent.validate(true)?
                }
                PairChange::HealBuff {
                    duration_ms,
                    profile,
                } => {
                    if *duration_ms == 0 || profile.end_on_source_exit {
                        return Err("Healing buff needs an independent finite lifetime".into());
                    }
                    profile.validate()?;
                }
                PairChange::CoincidentBurst {
                    pattern,
                    max_hp_percent,
                    ..
                } => {
                    bursts += 1;
                    pattern.validate()?;
                    if *max_hp_percent == 0 || *max_hp_percent > 100 {
                        return Err("Invalid coincident burst percentage".into());
                    }
                }
            }
        }
        if bursts > 1 {
            return Err("Conflicting coincident replacements".into());
        }
        Ok(())
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        use crate::game::relic::structural::support::pattern_extent;
        let mut n = std::mem::size_of::<Self>()
            .checked_add(self.definition.capacity())?
            .checked_add(self.intersection.definition.capacity())?
            .checked_add(
                self.modifiers
                    .capacity()
                    .checked_mul(std::mem::size_of::<PairModifier>())?,
            )?;
        for p in [&self.selection_range, &self.first, &self.second] {
            n = n.checked_add(pattern_extent(p)?)?;
        }
        for m in &self.modifiers {
            n = n
                .checked_add(m.id.capacity())?
                .checked_add(m.relic_definition.capacity())?;
            n = n.checked_add(match &m.change {
                PairChange::Range { pattern, .. } | PairChange::CoincidentBurst { pattern, .. } => {
                    pattern_extent(pattern)?
                }
                PairChange::HealBuff { profile, .. } => profile.definition.capacity(),
                _ => 0,
            })?;
        }
        Some(n)
    }
}
crate::authoring::clone_shape::struct_shape!(PairModifier {
    id,
    relic_definition,
    change
});
crate::authoring::clone_shape::struct_shape!(PairedAreasDef {
    definition,
    admission,
    selection_range,
    first,
    second,
    duration_ms,
    interval_ms,
    healing,
    transfer_percent,
    intersection,
    modifiers
});
impl crate::authoring::clone_shape::CloneShape for PairChange {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        match self {
            Self::Range { pattern, .. } | Self::CoincidentBurst { pattern, .. } => {
                pattern.visit_clone_shape(m)
            }
            Self::HealBuff { profile, .. } => profile.visit_clone_shape(m),
            _ => m.scan_storage(Some(1)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairedAreaView {
    pub activation: u64,
    pub copy: uuid::Uuid,
    pub caster: crate::game::battle::ids::UnitInstanceId,
    pub first_center: crate::game::resources::Position,
    pub second_center: crate::game::resources::Position,
    pub first: TileRangePattern,
    pub second: TileRangePattern,
    pub expires_at_ms: u64,
    pub next_pulse_ms: u64,
    pub intersection_members: Vec<crate::game::battle::ids::UnitInstanceId>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairedAreasSnapshot {
    pub areas: Vec<PairedAreaView>,
}
