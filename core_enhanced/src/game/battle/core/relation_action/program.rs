use super::super::marks::{ratio_product, ratio_sum};
use super::*;
use crate::game::relic::structural::support::pattern_extent;
pub(super) fn scale(value: u64, factor: MarkRatio) -> Result<u32, GameError> {
    if factor.denominator == 0 {
        return Err(GameError::InvalidAction);
    }
    u32::try_from(
        (u128::from(value) * u128::from(factor.numerator)).div_ceil(u128::from(factor.denominator)),
    )
    .map_err(|_| GameError::InvalidAction)
}
pub(super) fn percentage(value: u64, percent: MarkRatio) -> Result<u32, GameError> {
    scale(
        value,
        ratio_product(
            percent,
            MarkRatio {
                numerator: 1,
                denominator: 100,
            },
        )?,
    )
}
pub(super) fn extent(p: &RelationActionDef) -> Option<usize> {
    fn count(c: &ActionCount) -> Option<usize> {
        match c {
            ActionCount::Quotient { count: c, .. } => std::mem::size_of::<ActionCount>()
                .checked_add(64)?
                .checked_add(count(c)?),
            ActionCount::Sum { first, second } => 2usize
                .checked_mul(std::mem::size_of::<ActionCount>() + 64)?
                .checked_add(count(first)?)?
                .checked_add(count(second)?),
            _ => Some(0),
        }
    }
    fn range(r: &PlanRange) -> Option<usize> {
        r.definition
            .capacity()
            .checked_add(pattern_extent(&r.pattern)?)?
            .checked_add(64)
    }
    fn status(s: &PlanStatus) -> Option<usize> {
        s.definition
            .capacity()
            .checked_add(count(&s.count)?)?
            .checked_add(64)
    }
    let mut n = std::mem::size_of::<RelationActionDef>().checked_add(128)?;
    for s in [
        &p.definition,
        &p.stock_definition,
        &p.relation_definition,
        &p.mode_definition,
        &p.menu_entry,
    ] {
        n = n.checked_add(s.capacity() + 64)?;
    }
    if let Some(r) = &p.selection_range {
        n = n.checked_add(pattern_extent(r)?)?;
    }
    for e in [&p.normal, &p.active] {
        n = n.checked_add(e.definition.capacity() + 64)?;
        if let Some(r) = &e.selector {
            n = n.checked_add(range(r)?)?;
        }
    }
    n = n
        .checked_add(
            p.plans
                .capacity()
                .checked_mul(std::mem::size_of::<NamedActionPlan>())?,
        )?
        .checked_add(
            p.modifiers
                .capacity()
                .checked_mul(std::mem::size_of::<RelationActionModifier>())?,
        )?;
    for plan in &p.plans {
        n = n.checked_add(plan.definition.capacity() + 64)?;
        match &plan.op {
            ActionPlan::Strike {
                range: r, after, ..
            } => {
                if let Some(r) = r {
                    n = n.checked_add(range(r)?)?;
                }
                n = n.checked_add(
                    after
                        .capacity()
                        .checked_mul(std::mem::size_of::<PlanStatus>())?,
                )?;
                for s in after {
                    n = n.checked_add(status(s)?)?;
                }
            }
            ActionPlan::Repeat { count: c, .. } | ActionPlan::Timed { count: c, .. } => {
                n = n.checked_add(count(c)?)?
            }
            ActionPlan::Spend { amount, after, .. } => {
                if let Some(c) = amount {
                    n = n.checked_add(count(c)?)?;
                }
                if let Some(s) = after {
                    n = n.checked_add(status(s)?)?;
                }
            }
            _ => (),
        }
    }
    for m in &p.modifiers {
        n = n.checked_add(m.id.capacity() + m.relic_definition.capacity() + 128)?;
        match &m.change {
            RelationActionChange::Range {
                definition,
                pattern,
            } => {
                n = n
                    .checked_add(definition.capacity() + 64)?
                    .checked_add(pattern_extent(pattern)?)?
            }
            RelationActionChange::Period { definition, .. }
            | RelationActionChange::StatusCount { definition, .. }
            | RelationActionChange::PrimaryArea { definition }
            | RelationActionChange::ControlVulnerability { definition, .. } => {
                n = n.checked_add(definition.capacity() + 64)?
            }
            RelationActionChange::Multihit { entry, .. }
            | RelationActionChange::Retarget { entry, .. } => {
                n = n.checked_add(entry.capacity() + 64)?
            }
            _ => (),
        }
    }
    Some(n)
}
impl BattleCore {
    pub(super) fn relation_program(
        &self,
        caster: UnitInstanceId,
        source: &RelationActionDef,
    ) -> Result<(RelationActionDef, Vec<AppliedMarkModifier>), GameError> {
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            extent(source),
            "relation_program",
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
        let mut gain = MarkRatio {
            numerator: 1,
            denominator: 1,
        };
        let mut shield = ZERO;
        for m in &p.modifiers {
            match &m.change {
                RelationActionChange::Period {
                    negative, percent, ..
                } => periods.push((*negative, *percent)),
                RelationActionChange::UnlimitedStock => p.stock_capacity = None,
                RelationActionChange::Gain { factor } => gain = ratio_product(gain, *factor)?,
                RelationActionChange::Shield { percent } => shield = ratio_sum(shield, *percent)?,
                RelationActionChange::Protection => {
                    p.normal.protected = true;
                    p.active.protected = true;
                }
                RelationActionChange::Range {
                    definition,
                    pattern,
                } => {
                    for plan in &mut p.plans {
                        if let ActionPlan::Strike { range: Some(r), .. } = &mut plan.op {
                            if &r.definition == definition {
                                r.pattern = controlled_clone(pattern)?;
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        p.period_ms = super::super::marks::modified_period(p.period_ms, &periods)?;
        p.periodic_gain = scale(p.periodic_gain.into(), gain)?;
        p.shield_percent = ratio_product(
            p.shield_percent,
            ratio_sum(
                MarkRatio {
                    numerator: 1,
                    denominator: 1,
                },
                ratio_product(
                    shield,
                    MarkRatio {
                        numerator: 1,
                        denominator: 100,
                    },
                )?,
            )?,
        )?;
        p.validate().map_err(GameError::InvalidStaticData)?;
        let mut applied_extent = applied
            .capacity()
            .checked_mul(std::mem::size_of::<AppliedMarkModifier>());
        for m in &applied {
            applied_extent = applied_extent
                .and_then(|n| n.checked_add(m.rule_id.capacity() + m.relic_definition.capacity()));
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            applied_extent,
            "relation_applied_modifiers",
        )?;
        control::poll("relation_program_modifiers")?;
        Ok((p, applied))
    }
}
