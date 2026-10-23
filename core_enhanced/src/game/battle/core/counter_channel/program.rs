use super::*;
use crate::game::ability::marks::MarkRatio;
use crate::game::relic::structural::support::pattern_extent;

pub(super) fn extent(p: &CounterChannelDef) -> Option<usize> {
    fn range(r: &CounterRange) -> Option<usize> {
        r.definition
            .capacity()
            .checked_add(pattern_extent(&r.pattern)?)
    }
    let mut n = std::mem::size_of::<CounterChannelDef>()
        .checked_add(128)?
        .checked_add(p.definition.capacity())?
        .checked_add(
            p.entries
                .capacity()
                .checked_mul(std::mem::size_of::<CounterEntry>())?,
        )?;
    for e in &p.entries {
        n = n.checked_add(e.key.capacity())?.checked_add(
            e.phases
                .capacity()
                .checked_mul(std::mem::size_of::<CounterPhase>())?,
        )?;
        if let Some(r) = &e.selection_range {
            n = n.checked_add(pattern_extent(r)?)?;
        }
        for phase in &e.phases {
            n = n.checked_add(phase.definition.capacity())?;
            match &phase.range {
                PhaseRange::Fixed(r) => n = n.checked_add(range(r)?)?,
                PhaseRange::Consumed(v) => {
                    n = n.checked_add(
                        v.capacity()
                            .checked_mul(std::mem::size_of::<(u32, CounterRange)>())?,
                    )?;
                    for (_, r) in v {
                        n = n.checked_add(range(r)?)?;
                    }
                }
            }
            n = n.checked_add(
                phase
                    .queries
                    .capacity()
                    .checked_mul(std::mem::size_of::<PhaseQuery>())?,
            )?;
            for q in &phase.queries {
                n = n.checked_add(q.definition.capacity())?;
                if let PhaseEffect::Damage { amount, .. } | PhaseEffect::Heal { amount } = &q.effect
                {
                    n = n.checked_add(amount.definition.capacity())?;
                }
            }
        }
    }
    n = n.checked_add(
        p.modifiers
            .capacity()
            .checked_mul(std::mem::size_of::<CounterModifier>())?,
    )?;
    for m in &p.modifiers {
        n = n
            .checked_add(m.id.capacity())?
            .checked_add(m.relic_definition.capacity())?;
        match &m.change {
            CounterChange::Range {
                definition,
                pattern,
            } => {
                n = n
                    .checked_add(definition.capacity())?
                    .checked_add(pattern_extent(pattern)?)?
            }
            CounterChange::Healing { definition, .. } => {
                n = n.checked_add(definition.capacity())?
            }
            CounterChange::HealBuff {
                definition,
                profile,
                ..
            } => {
                n = n
                    .checked_add(definition.capacity())?
                    .checked_add(profile.definition.capacity())?
            }
            CounterChange::Protection { entry } => n = n.checked_add(entry.capacity())?,
            _ => (),
        }
    }
    Some(n)
}
fn scale(base: u32, ratio: MarkRatio) -> Result<u32, GameError> {
    u32::try_from(
        (u128::from(base) * u128::from(ratio.numerator)).div_ceil(u128::from(ratio.denominator)),
    )
    .map_err(|_| GameError::InvalidAction)
}
pub(super) fn gain(
    p: &CounterChannelDef,
    target: CounterGainTarget,
    base: u64,
) -> Result<u32, GameError> {
    let mut ratio = MarkRatio {
        numerator: 1,
        denominator: 1,
    };
    for m in &p.modifiers {
        if let CounterChange::Gain { target: t, factor } = &m.change {
            if *t == target {
                ratio = super::super::marks::ratio_product(ratio, *factor)?;
            }
        }
    }
    let value =
        (u128::from(base) * u128::from(ratio.numerator)).div_ceil(u128::from(ratio.denominator));
    Ok(value.min(u128::from(p.capacity)) as u32)
}
pub(super) fn amount(
    p: &CounterChannelDef,
    amount: &CounterAmount,
    consumed: u32,
    heal: bool,
) -> Result<u32, GameError> {
    use super::super::marks::{ratio_product, ratio_sum};
    let one = MarkRatio {
        numerator: 1,
        denominator: 1,
    };
    let hundredth = MarkRatio {
        numerator: 1,
        denominator: 100,
    };
    let count = ratio_product(
        amount.per_count_percent,
        MarkRatio {
            numerator: u64::from(consumed),
            denominator: 1,
        },
    )?;
    let mut ratio = ratio_sum(one, ratio_product(count, hundredth)?)?;
    if heal {
        let mut percent = MarkRatio {
            numerator: 0,
            denominator: 1,
        };
        for m in &p.modifiers {
            if let CounterChange::Healing {
                definition,
                percent: change,
            } = &m.change
            {
                if definition == &amount.definition {
                    percent = ratio_sum(percent, *change)?;
                }
            }
        }
        ratio = ratio_product(ratio, ratio_sum(one, ratio_product(percent, hundredth)?)?)?;
    }
    let value = scale(amount.base, ratio)?;
    if value > i32::MAX as u32 {
        return Err(GameError::InvalidAction);
    }
    Ok(value)
}
impl BattleCore {
    pub(super) fn counter_program(
        &self,
        caster: UnitInstanceId,
        source: &CounterChannelDef,
    ) -> Result<(CounterChannelDef, Vec<AppliedMarkModifier>), GameError> {
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            extent(source),
            "counter_program",
        )?;
        let mut p = controlled_clone(source)?;
        let mut applied = Vec::new();
        p.modifiers.retain(|m| {
            if let Some(copy) = self.equipped_normal_relic_copy(caster, &m.relic_definition) {
                applied.push(AppliedMarkModifier {
                    rule_id: m.id.clone(),
                    relic_definition: m.relic_definition.clone(),
                    relic_copy_uuid: copy,
                });
                true
            } else {
                false
            }
        });
        let mut periods = Vec::new();
        for m in &p.modifiers {
            match &m.change {
                CounterChange::Period { decrease, percent } => periods.push((*decrease, *percent)),
                CounterChange::Protection { entry } => {
                    p.entries
                        .iter_mut()
                        .find(|e| &e.key == entry)
                        .ok_or(GameError::InvalidAction)?
                        .protected = true;
                }
                CounterChange::Range {
                    definition,
                    pattern,
                } => {
                    for phase in p.entries.iter_mut().flat_map(|e| &mut e.phases) {
                        match &mut phase.range {
                            PhaseRange::Fixed(r) => {
                                if &r.definition == definition {
                                    r.pattern = controlled_clone(pattern)?;
                                }
                            }
                            PhaseRange::Consumed(v) => {
                                for (_, r) in v {
                                    if &r.definition == definition {
                                        r.pattern = controlled_clone(pattern)?;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        p.interval_ms = super::super::marks::modified_period(p.interval_ms, &periods)?;
        p.validate().map_err(GameError::InvalidStaticData)?;
        control::poll("counter_program_modifiers")?;
        Ok((p, applied))
    }
}
