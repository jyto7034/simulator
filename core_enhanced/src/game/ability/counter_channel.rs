//! Fragment-copy counters, menu admissions and ordered finite channel phases.
//! IDs identify authored declarations; display names never select runtime behavior.
use super::{
    marks::MarkRatio, resonance::ResonanceAdmission, sustained::TargetStateProfile,
    SkillHitTargetFilter,
};
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterSpend {
    None,
    AdmissionAll,
    EndAll,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseOrigin {
    SelectedTile,
    SelectedDirection,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterGainTarget {
    Periodic,
    Completion,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterRange {
    pub definition: String,
    pub pattern: TileRangePattern,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PhaseRange {
    Fixed(CounterRange),
    Consumed(Vec<(u32, CounterRange)>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterAmount {
    pub definition: String,
    pub base: u32,
    pub per_count_percent: MarkRatio,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PhaseEffect {
    Pull {
        distance_units: u32,
    },
    Damage {
        amount: CounterAmount,
        damage_type: DamageType,
    },
    Heal {
        amount: CounterAmount,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseQuery {
    pub definition: String,
    pub targets: SkillHitTargetFilter,
    pub include_caster: bool,
    pub corrupted_only: bool,
    pub effect: PhaseEffect,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterPhase {
    pub definition: String,
    pub origin: PhaseOrigin,
    pub range: PhaseRange,
    pub duration_ms: u32,
    pub interval_ms: u32,
    /// Each query observes current positions after preceding effects.
    pub queries: Vec<PhaseQuery>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionAward {
    pub base: u32,
    pub damage_and_healing_bonus: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterEntry {
    pub key: String,
    pub automatic: bool,
    pub admission: ResonanceAdmission,
    pub selection_range: Option<TileRangePattern>,
    pub minimum: u32,
    pub maximum: u32,
    pub spend: CounterSpend,
    pub remember_aim: bool,
    pub pause_basic: bool,
    pub protected: bool,
    pub phases: Vec<CounterPhase>,
    pub completion_award: Option<CompletionAward>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CounterChange {
    Period {
        decrease: bool,
        percent: MarkRatio,
    },
    Gain {
        target: CounterGainTarget,
        factor: MarkRatio,
    },
    Range {
        definition: String,
        pattern: TileRangePattern,
    },
    Healing {
        definition: String,
        percent: MarkRatio,
    },
    HealBuff {
        definition: String,
        duration_ms: u32,
        profile: TargetStateProfile,
    },
    Protection {
        entry: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterModifier {
    pub id: String,
    pub relic_definition: String,
    pub change: CounterChange,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterChannelDef {
    pub definition: String,
    pub capacity: u32,
    pub interval_ms: u32,
    pub periodic_gain: u32,
    pub default_entry: usize,
    pub entries: Vec<CounterEntry>,
    pub modifiers: Vec<CounterModifier>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterChannelsSnapshot {
    pub owners: Vec<CounterOwnerSnapshot>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterOwnerSnapshot {
    pub source_copy: uuid::Uuid,
    pub caster: crate::game::battle::ids::UnitInstanceId,
    pub count: u32,
    pub capacity: u32,
    pub next_gain_ms: Option<u64>,
    pub remembered_tile: Option<crate::game::resources::Position>,
    pub automatic_tile: crate::game::resources::Position,
    pub automatic_reserved: bool,
    pub pending_skill: Option<super::SkillId>,
    pub active: Option<CounterActiveSnapshot>,
    pub menu: Vec<CounterMenuSnapshot>,
    pub automatic_areas: Vec<CounterPhasePreview>,
    pub applied_modifiers: Vec<super::marks::AppliedMarkModifier>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterActiveSnapshot {
    pub skill_id: super::SkillId,
    pub phase: String,
    pub phase_end_ms: u64,
    pub next_tick_ms: u64,
    pub consumed: u32,
    pub automatic: bool,
    pub cancellable: bool,
    pub damaged_enemy_hp: bool,
    pub healed_ally_hp: bool,
    pub area: CounterPhasePreview,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterMenuSnapshot {
    pub skill_id: super::SkillId,
    pub key: String,
    pub minimum: u32,
    pub maximum: u32,
    pub counter_eligible: bool,
    pub resonance_eligible: bool,
    pub selection_range: crate::game::battle::tile_range::TileRangePattern,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterPhasePreview {
    pub phase: String,
    pub origin: crate::game::resources::Position,
    pub tiles: Vec<crate::game::resources::Position>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterChannelPreview {
    pub phases: Vec<CounterPhasePreview>,
    /// A warning based on the possible completion bonus, not an awarded result.
    pub possible_automatic_followup: Vec<CounterPhasePreview>,
}
fn identity(s: &str) -> Result<(), String> {
    if s.is_empty() || s.trim() != s || s.len() > 256 {
        return Err("Invalid counter channel declaration identity".into());
    }
    Ok(())
}
impl CounterChannelDef {
    pub fn entry_skill_id(&self, root: &super::SkillId, entry: usize) -> super::SkillId {
        if entry == self.default_entry {
            root.clone()
        } else {
            format!("{root}::entry:{entry}").into()
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        identity(&self.definition)?;
        if self.capacity == 0
            || self.interval_ms == 0
            || self.periodic_gain == 0
            || self.entries.is_empty()
            || self.entries.len() > 32
            || self.modifiers.len() > 128
        {
            return Err(
                "Counter channel requires a finite positive counter schedule and bounded menu"
                    .into(),
            );
        }
        if self
            .entries
            .get(self.default_entry)
            .is_none_or(|e| e.automatic)
            || self.entries.iter().filter(|e| e.automatic).count() != 1
        {
            return Err(
                "Counter channel requires one automatic cap entry and a manual default".into(),
            );
        }
        let mut keys = std::collections::BTreeSet::new();
        let mut ranges = std::collections::BTreeMap::new();
        let mut heals = std::collections::BTreeSet::new();
        for e in &self.entries {
            identity(&e.key)?;
            e.admission.validate().map_err(str::to_owned)?;
            if !keys.insert(&e.key)
                || e.minimum > e.maximum
                || e.maximum > self.capacity
                || e.phases.is_empty()
                || e.phases.len() > 64
            {
                return Err("Invalid counter entry bounds or phase list".into());
            }
            if e.automatic {
                if e.minimum != self.capacity
                    || e.maximum != self.capacity
                    || e.spend != CounterSpend::EndAll
                    || e.selection_range.is_some()
                    || e.remember_aim
                    || e.completion_award.is_some()
                    || e.admission.requirement != super::resonance::ResonanceRequirement::None
                    || e.admission.cost != super::resonance::ResonanceCost::Zero
                {
                    return Err("Automatic cap entry must consume on end without an extra resonance charge or reward".into());
                }
            } else if e.maximum >= self.capacity
                || e.selection_range.is_none()
                || e.spend == CounterSpend::EndAll
            {
                return Err(
                    "Manual menu entries require a selected tile below the reserved cap".into(),
                );
            }
            if let Some(r) = &e.selection_range {
                r.validate()?;
            }
            if e.completion_award.is_some() && e.spend != CounterSpend::None {
                return Err("Completion rewards cannot regenerate the consumed counter".into());
            }
            let mut phase_ids = std::collections::BTreeSet::new();
            let mut total = 0u64;
            for p in &e.phases {
                identity(&p.definition)?;
                if !phase_ids.insert(&p.definition)
                    || p.duration_ms == 0
                    || p.interval_ms == 0
                    || p.queries.is_empty()
                    || p.queries.len() > 64
                {
                    return Err(
                        "Channel phase requires a finite half-open cadence and ordered queries"
                            .into(),
                    );
                }
                total = total
                    .checked_add(u64::from(p.duration_ms))
                    .ok_or("Channel duration overflow")?;
                let mut add_range = |r: &CounterRange| -> Result<(), String> {
                    identity(&r.definition)?;
                    r.pattern.validate()?;
                    if let Some(previous) = ranges.insert(r.definition.clone(), r.pattern.clone()) {
                        if previous != r.pattern {
                            return Err("One range definition has conflicting patterns".into());
                        }
                    }
                    Ok(())
                };
                match &p.range {
                    PhaseRange::Fixed(r) => add_range(r)?,
                    PhaseRange::Consumed(tiers) => {
                        if e.spend != CounterSpend::AdmissionAll
                            || tiers.len()
                                != usize::try_from(u64::from(e.maximum) - u64::from(e.minimum) + 1)
                                    .map_err(|_| "Count tiers overflow")?
                            || tiers.len() > 128
                        {
                            return Err("Consumed ranges must cover each admissible count".into());
                        }
                        let mut previous = std::collections::BTreeSet::new();
                        for (i, (count, r)) in tiers.iter().enumerate() {
                            if u64::from(*count) != u64::from(e.minimum) + i as u64 {
                                return Err(
                                    "Count range tiers must be unique and contiguous".into()
                                );
                            }
                            add_range(r)?;
                            let cells = r.pattern.affected_tiles(
                                crate::game::resources::Position::new(0, 0),
                                crate::game::battle::tile_range::FacingDirection::Up,
                            )?;
                            let cells: std::collections::BTreeSet<_> =
                                cells.into_iter().map(|p| (p.x, p.y)).collect();
                            if !previous.is_subset(&cells) {
                                return Err(
                                    "Larger consumed-count ranges must include smaller tiers"
                                        .into(),
                                );
                            }
                            previous = cells;
                        }
                    }
                }
                for q in &p.queries {
                    identity(&q.definition)?;
                    match &q.effect {
                        PhaseEffect::Pull { distance_units } => {
                            if *distance_units == 0
                                || !q.corrupted_only
                                || q.targets != SkillHitTargetFilter::Enemies
                                || q.include_caster
                            {
                                return Err(
                                    "Pull requires positive distance and corrupted enemies".into(),
                                );
                            }
                        }
                        PhaseEffect::Damage { amount, .. } | PhaseEffect::Heal { amount } => {
                            identity(&amount.definition)?;
                            amount.per_count_percent.validate(true)?;
                            if amount.base > i32::MAX as u32
                                || (amount.per_count_percent.numerator > 0
                                    && e.spend != CounterSpend::AdmissionAll)
                            {
                                return Err("Counter amount exceeds native range or lacks a consumed-count receipt".into());
                            }
                            if matches!(q.effect, PhaseEffect::Heal { .. }) {
                                heals.insert(amount.definition.clone());
                            }
                        }
                    }
                }
            }
        }
        let mut ids = std::collections::BTreeSet::new();
        for m in &self.modifiers {
            identity(&m.id)?;
            identity(&m.relic_definition)?;
            if !ids.insert(&m.id) {
                return Err("Duplicate counter modifier identity".into());
            }
            match &m.change {
                CounterChange::Period { decrease, percent } => {
                    percent.validate(true)?;
                    if *decrease
                        && u128::from(percent.numerator) >= 100 * u128::from(percent.denominator)
                    {
                        return Err("Counter period reduction must retain positive time".into());
                    }
                }
                CounterChange::Gain { factor, .. } => factor.validate(true)?,
                CounterChange::Range {
                    definition,
                    pattern,
                } => {
                    pattern.validate()?;
                    if !ranges.contains_key(definition) {
                        return Err("Unknown counter phase range".into());
                    }
                }
                CounterChange::Healing {
                    definition,
                    percent,
                } => {
                    percent.validate(true)?;
                    if !heals.contains(definition) {
                        return Err("Healing modifier has no healing declaration".into());
                    }
                }
                CounterChange::HealBuff {
                    definition,
                    duration_ms,
                    profile,
                } => {
                    profile.validate()?;
                    if *duration_ms == 0 || !heals.contains(definition) {
                        return Err(
                            "Heal buff requires positive duration and a healing declaration".into(),
                        );
                    }
                }
                CounterChange::Protection { entry } => {
                    if !keys.contains(entry) {
                        return Err("Protection targets an unknown channel entry".into());
                    }
                }
            }
        }
        Ok(())
    }
}
crate::authoring::clone_shape::copy_shape!(CounterSpend, PhaseOrigin, CounterGainTarget);
crate::authoring::clone_shape::struct_shape!(CounterRange {
    definition,
    pattern
});
crate::authoring::clone_shape::struct_shape!(CounterAmount {
    definition,
    base,
    per_count_percent
});
crate::authoring::clone_shape::struct_shape!(PhaseQuery {
    definition,
    targets,
    include_caster,
    corrupted_only,
    effect
});
crate::authoring::clone_shape::struct_shape!(CounterPhase {
    definition,
    origin,
    range,
    duration_ms,
    interval_ms,
    queries
});
crate::authoring::clone_shape::struct_shape!(CompletionAward {
    base,
    damage_and_healing_bonus
});
crate::authoring::clone_shape::struct_shape!(CounterEntry {
    key,
    automatic,
    admission,
    selection_range,
    minimum,
    maximum,
    spend,
    remember_aim,
    pause_basic,
    protected,
    phases,
    completion_award
});
crate::authoring::clone_shape::struct_shape!(CounterModifier {
    id,
    relic_definition,
    change
});
crate::authoring::clone_shape::struct_shape!(CounterChannelDef {
    definition,
    capacity,
    interval_ms,
    periodic_gain,
    default_entry,
    entries,
    modifiers
});

impl crate::authoring::clone_shape::CloneShape for PhaseRange {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(1))?;
        match self {
            Self::Fixed(r) => r.visit_clone_shape(m),
            Self::Consumed(v) => v.visit_clone_shape(m),
        }
    }
}
impl crate::authoring::clone_shape::CloneShape for PhaseEffect {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(1))?;
        match self {
            Self::Pull { .. } => Ok(()),
            Self::Damage { amount, .. } | Self::Heal { amount } => amount.visit_clone_shape(m),
        }
    }
}
impl crate::authoring::clone_shape::CloneShape for CounterChange {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        m.scan_storage(Some(1))?;
        match self {
            Self::Period { percent, .. } => percent.visit_clone_shape(m),
            Self::Gain { factor, .. } => factor.visit_clone_shape(m),
            Self::Range {
                definition,
                pattern,
            } => {
                definition.visit_clone_shape(m)?;
                pattern.visit_clone_shape(m)
            }
            Self::Healing {
                definition,
                percent,
            } => {
                definition.visit_clone_shape(m)?;
                percent.visit_clone_shape(m)
            }
            Self::HealBuff {
                definition,
                profile,
                ..
            } => {
                definition.visit_clone_shape(m)?;
                profile.visit_clone_shape(m)
            }
            Self::Protection { entry } => entry.visit_clone_shape(m),
        }
    }
}
