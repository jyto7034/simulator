//! Installed pair ownership is independent from the completed casting action.
use super::*;
use crate::{
    authoring::clone_shape::controlled_clone,
    game::{
        ability::{paired_areas::*, SkillHitTargetFilter},
        battle::{
            damage::{DamageModifiers, DamageSource, DamageType},
            event_log::{HpChangeReason, SkillCastTarget},
            tile_range::FacingDirection,
            types::{BattleUnitRole, BattleUnitSourceIdentity, BattleUnitThreatClass},
        },
        enums::Side,
        resources::Position,
    },
};
use uuid::Uuid;
#[derive(Debug, Clone)]
struct Pair {
    id: u64,
    copy: Uuid,
    caster: UnitInstanceId,
    side: Side,
    centers: [Position; 2],
    program: Arc<PairedAreasDef>,
    source: RuntimeFactSource,
    expires: u64,
    next: u64,
    burst: bool,
    members: BTreeMap<UnitInstanceId, u64>,
}
impl crate::authoring::clone_shape::CloneShape for Pair {
    fn visit_clone_shape(
        &self,
        m: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), GameError> {
        m.scan_storage(Some(std::mem::size_of::<Self>()))?;
        self.program.visit_clone_shape(m)?;
        self.source.visit_clone_shape(m)?;
        self.members.visit_clone_shape(m)
    }
}
#[derive(Debug, Clone, Default)]
pub(super) struct PairedAreasState {
    next: u64,
    pairs: BTreeMap<u64, Pair>,
}
crate::authoring::clone_shape::struct_shape!(PairedAreasState { next, pairs });
impl PairedAreasState {
    pub(super) fn entity_count(&self) -> usize {
        self.pairs.len() + self.pairs.values().map(|p| p.members.len()).sum::<usize>()
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        self.pairs.values().try_fold(0usize, |n, p| {
            n.checked_add(std::mem::size_of::<Pair>() + 80)?
                .checked_add(p.program.retained_extent()?)?
                .checked_add(p.source.retained_extent()?)?
                .checked_add(p.members.len().checked_mul(96)?)
        })
    }
}
fn boosted(
    base: u32,
    changes: &[crate::game::ability::marks::MarkRatio],
) -> Result<u32, GameError> {
    if base == 0 {
        return Ok(0);
    }
    let value = super::super::marks::modified_period(
        base.max(1),
        &changes.iter().map(|v| (false, *v)).collect::<Vec<_>>(),
    )?;
    Ok(if base == 0 { 0 } else { value })
}
impl BattleCore {
    pub fn paired_areas_snapshot(&self) -> Result<PairedAreasSnapshot, GameError> {
        control::charge(
            self.sustained.pairs.entity_count() as u64,
            "paired_snapshot",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.pairs.retained_extent(),
            "paired_snapshot",
        )?;
        let areas = self
            .sustained
            .pairs
            .pairs
            .values()
            .filter(|p| !p.burst)
            .map(|p| {
                Ok(PairedAreaView {
                    activation: p.id,
                    copy: p.copy,
                    caster: p.caster,
                    first_center: p.centers[0],
                    second_center: p.centers[1],
                    first: controlled_clone(&p.program.first)?,
                    second: controlled_clone(&p.program.second)?,
                    expires_at_ms: p.expires,
                    next_pulse_ms: p.next,
                    intersection_members: p.members.keys().copied().collect(),
                })
            })
            .collect::<Result<Vec<_>, GameError>>()?;
        Ok(PairedAreasSnapshot { areas })
    }

    pub(in crate::game::battle::core) fn paired_area_skill(
        skill: &crate::game::ability::SkillDef,
    ) -> bool {
        skill.steps.iter().flat_map(|s| &s.effects).any(|e| {
            matches!(
                e.op,
                crate::game::ability::SkillEffectDef::PairedAreas { .. }
            )
        })
    }
    pub(in crate::game::battle::core) fn validate_paired_area_start(
        &self,
        caster: UnitInstanceId,
        skill: &crate::game::ability::SkillDef,
        now: u64,
    ) -> Result<(), GameError> {
        for e in skill.steps.iter().flat_map(|s| &s.effects) {
            if let crate::game::ability::SkillEffectDef::PairedAreas { program } = &e.op {
                if !self
                    .units
                    .get(&caster)
                    .is_some_and(|u| u.fragment_source.is_some())
                {
                    return Err(GameError::InvalidAction);
                }
                let p = self.pair_program(caster, program)?;
                now.checked_add(u64::from(p.duration_ms))
                    .ok_or(GameError::InvalidAction)?;
                for m in &p.modifiers {
                    if let PairChange::CoincidentBurst { delay_ms, .. } = m.change {
                        now.checked_add(u64::from(delay_ms))
                            .ok_or(GameError::InvalidAction)?;
                    }
                }
            }
        }
        Ok(())
    }
    fn pair_program(
        &self,
        caster: UnitInstanceId,
        p: &PairedAreasDef,
    ) -> Result<PairedAreasDef, GameError> {
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            p.retained_extent(),
            "paired_program",
        )?;
        let mut p = controlled_clone(p)?;
        p.modifiers.retain(|m| {
            self.equipped_normal_relic_copy(caster, &m.relic_definition)
                .is_some()
        });
        let mut durations = Vec::new();
        let mut healing = Vec::new();
        let mut buffs = Vec::new();
        for m in &p.modifiers {
            match &m.change {
                PairChange::Duration { negative, percent } => durations.push((*negative, *percent)),
                PairChange::Range {
                    first,
                    second,
                    pattern,
                } => {
                    if *first {
                        p.first = controlled_clone(pattern)?;
                    }
                    if *second {
                        p.second = controlled_clone(pattern)?;
                    }
                }
                PairChange::Healing { percent } => healing.push(*percent),
                PairChange::Buff { percent } => buffs.push(*percent),
                _ => (),
            }
        }
        p.duration_ms = super::super::marks::modified_period(p.duration_ms, &durations)?;
        p.healing = boosted(p.healing, &healing)?;
        p.intersection.attack_speed_points = boosted(p.intersection.attack_speed_points, &buffs)?;
        p.validate().map_err(GameError::InvalidStaticData)?;
        Ok(p)
    }

    pub(in crate::game::battle::core) fn install_paired_areas(
        &mut self,
        caster: UnitInstanceId,
        p: &PairedAreasDef,
        now: u64,
    ) -> Result<super::super::types::SkillStepResult, GameError> {
        let cast = self
            .runtime_fact_source
            .cast
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let target = self
            .active_skill_casts
            .get(&cast.cast_seq)
            .and_then(|c| c.flow.as_ref())
            .and_then(|f| f.accepted_target());
        let Some(SkillCastTarget::TilePair { first, second }) = target else {
            return Err(GameError::InvalidAction);
        };
        let unit = self
            .units
            .get(&caster)
            .filter(|u| u.is_active())
            .ok_or(GameError::InvalidAction)?;
        let copy = unit
            .fragment_source
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .copy_uuid;
        let side = unit.owner;
        let p = self.pair_program(caster, p)?;
        let burst_delay = if first == second {
            p.modifiers.iter().find_map(|m| match m.change {
                PairChange::CoincidentBurst { delay_ms, .. } => Some(delay_ms),
                _ => None,
            })
        } else {
            None
        };
        let expires = now
            .checked_add(u64::from(burst_delay.unwrap_or(p.duration_ms)))
            .ok_or(GameError::InvalidAction)?;
        let next = if burst_delay.is_some() {
            expires
        } else {
            now.checked_add(u64::from(p.interval_ms))
                .ok_or(GameError::InvalidAction)?
                .min(expires)
        };
        let source = self.runtime_fact_source.independent_origin()?;
        source.admit_owner("paired_area_source")?;
        self.admit_authoring_runtime_growth(1, "paired_area_install")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            source
                .retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<Pair>() + 80)),
            "paired_area_install",
        )?;
        let id = self
            .sustained
            .pairs
            .next
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        let old: Vec<_> = self
            .sustained
            .pairs
            .pairs
            .values()
            .filter(|p| p.copy == copy)
            .map(|p| p.id)
            .collect();
        for id in old {
            self.end_pair(id, now, SustainedEndReason::Replaced)?;
        }
        self.sustained.pairs.next = id;
        self.sustained.pairs.pairs.insert(
            id,
            Pair {
                id,
                copy,
                caster,
                side,
                centers: [first, second],
                program: Arc::new(p),
                source,
                expires,
                next,
                burst: burst_delay.is_some(),
                members: Default::default(),
            },
        );
        self.refresh_pair_members(id, now)?;
        self.event_queue
            .push(BattleEvent::PairedAreaTick { time_ms: next, id });
        Ok(super::super::types::SkillStepResult {
            applied_effect_count: 1,
            ..Default::default()
        })
    }
    fn pair_memberships(&self, p: &Pair, now: u64) -> Result<[Vec<UnitInstanceId>; 2], GameError> {
        let mut sets = [Vec::new(), Vec::new()];
        for (i, pattern) in [&p.program.first, &p.program.second]
            .into_iter()
            .enumerate()
        {
            let tiles = self
                .valid_tile_area_tiles(pattern, p.centers[i], FacingDirection::Up)
                .ok_or(GameError::InvalidAction)?;
            sets[i] = self.collect_tile_area_targets_at(
                now,
                p.side,
                p.caster,
                &tiles,
                SkillHitTargetFilter::Allies,
                true,
            );
        }
        let count = sets[0].len().saturating_add(sets[1].len());
        control::charge(
            (count as u64)
                .saturating_mul(u64::from(usize::BITS - count.leading_zeros()) + 1)
                .saturating_mul(4),
            "paired_membership_settlement",
        )?;
        control::poll("paired_membership_query")?;
        Ok(sets)
    }
    fn refresh_pair_members(&mut self, id: u64, now: u64) -> Result<(), GameError> {
        let Some(p) = self.sustained.pairs.pairs.get(&id).filter(|p| !p.burst) else {
            return Ok(());
        };
        let p = controlled_clone(p)?;
        let sets = self.pair_memberships(&p, now)?;
        self.refresh_pair_members_from(&p, &sets, now)
    }
    fn refresh_pair_members_from(
        &mut self,
        p: &Pair,
        sets: &[Vec<UnitInstanceId>; 2],
        now: u64,
    ) -> Result<(), GameError> {
        let id = p.id;
        let members: Vec<_> = sets[0]
            .iter()
            .copied()
            .filter(|t| sets[1].binary_search(t).is_ok())
            .collect();
        for (&target, &activation) in &p.members {
            if members.binary_search(&target).is_err() {
                self.end_sustained(activation, now, SustainedEndReason::TargetUnavailable)?;
                if let Some(p) = self.sustained.pairs.pairs.get_mut(&id) {
                    p.members.remove(&target);
                }
            }
        }
        for target in members {
            if p.members.contains_key(&target) {
                continue;
            }
            let prior =
                std::mem::replace(&mut self.runtime_fact_source, controlled_clone(&p.source)?);
            let before = self.sustained.next;
            let result = self.begin_sustained(
                p.caster,
                target,
                crate::game::ability::sustained::SustainedEffectDef::TargetState {
                    duration_ms: u32::try_from(p.expires.saturating_sub(now))
                        .map_err(|_| GameError::InvalidAction)?,
                    profile: controlled_clone(&p.program.intersection)?,
                },
                now,
            );
            self.runtime_fact_source = prior;
            result?;
            if self.sustained.next != before {
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    Some(96),
                    "paired_membership",
                )?;
                if let Some(p) = self.sustained.pairs.pairs.get_mut(&id) {
                    p.members.insert(target, self.sustained.next);
                }
            }
        }
        Ok(())
    }
    fn end_pair(&mut self, id: u64, now: u64, reason: SustainedEndReason) -> Result<(), GameError> {
        let Some(p) = self.sustained.pairs.pairs.remove(&id) else {
            return Ok(());
        };
        self.event_queue.discard_pair_tick(Some(id))?;
        for (_, activation) in p.members {
            self.end_sustained(activation, now, reason)?;
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn clear_paired_areas(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        while let Some(id) = self.sustained.pairs.pairs.keys().next().copied() {
            self.end_pair(id, now, SustainedEndReason::Terminal)?;
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn reconcile_paired_areas(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        control::charge(
            self.sustained.pairs.entity_count() as u64,
            "paired_reconcile",
        )?;
        let ids: Vec<_> = self.sustained.pairs.pairs.keys().copied().collect();
        for id in ids {
            let p = &self.sustained.pairs.pairs[&id];
            if !self.units.get(&p.caster).is_some_and(|u| {
                u.is_active()
                    && u.fragment_source
                        .as_ref()
                        .is_some_and(|f| f.copy_uuid == p.copy)
            }) {
                self.end_pair(id, now, SustainedEndReason::SourceUnavailable)?;
            } else if !p.burst && now >= p.expires {
                self.end_pair(id, now, SustainedEndReason::Expired)?;
            } else {
                self.refresh_pair_members(id, now)?;
            }
        }
        Ok(())
    }
    fn pair_heal_eligible(&self, target: UnitInstanceId) -> bool {
        !self.stress_protected_target(target)
            && self
                .units
                .get(&target)
                .is_some_and(|u| u.is_active() && u.role != BattleUnitRole::IndependentWall)
    }
    fn pair_heal(
        &mut self,
        p: &Pair,
        target: UnitInstanceId,
        amount: u32,
        now: u64,
    ) -> Result<u32, GameError> {
        if !self.pair_heal_eligible(target) {
            return Ok(0);
        }
        let before = self.units[&target].stats.current_health;
        let cap = self.units[&target].stats.max_health.max(1);
        let mut remaining = amount;
        while remaining > 0 {
            let part = remaining.min(i32::MAX as u32);
            self.apply_hp_delta_and_record(
                Some(p.caster),
                target,
                part as i32,
                now,
                HpChangeReason::Command,
            );
            control::poll("paired_heal_segment")?;
            remaining -= part;
        }
        control::poll("paired_heal")?;
        let actual = self
            .units
            .get(&target)
            .map_or(0, |u| u.stats.current_health.saturating_sub(before));
        if actual > 0 {
            for m in &p.program.modifiers {
                if let PairChange::HealBuff {
                    duration_ms,
                    profile,
                } = &m.change
                {
                    self.begin_sustained(
                        p.caster,
                        target,
                        crate::game::ability::sustained::SustainedEffectDef::TargetState {
                            duration_ms: *duration_ms,
                            profile: controlled_clone(profile)?,
                        },
                        now,
                    )?;
                }
            }
        }
        Ok(amount.saturating_sub(cap.saturating_sub(before)))
    }
    fn settle_pair(
        &mut self,
        p: &Pair,
        sets: &[Vec<UnitInstanceId>; 2],
        now: u64,
    ) -> Result<(), GameError> {
        let mut assigned = [Vec::new(), Vec::new()];
        let mut union = sets[0].clone();
        union.extend(sets[1].iter().copied());
        union.sort();
        union.dedup();
        for target in union {
            let side = if sets[0].binary_search(&target).is_err() {
                1
            } else if sets[1].binary_search(&target).is_err() {
                0
            } else {
                let pos = self
                    .sample_unit_body_at(target, now)
                    .ok_or(GameError::InvalidAction)?
                    .position
                    .project_to_tile();
                usize::from(pos.manhattan(&p.centers[1]) < pos.manhattan(&p.centers[0]))
            };
            assigned[side].push(target);
        }
        let mut overflow = [0u128; 2];
        for side in 0..2 {
            for &target in &assigned[side] {
                overflow[side] = overflow[side]
                    .checked_add(u128::from(self.pair_heal(
                        p,
                        target,
                        p.program.healing,
                        now,
                    )?))
                    .ok_or(GameError::InvalidAction)?;
            }
        }
        // Freeze deficits for both directions only after every base heal settles.
        let deficits = assigned.map(|targets| {
            targets
                .into_iter()
                .filter(|t| self.pair_heal_eligible(*t))
                .map(|t| {
                    (
                        t,
                        self.units[&t]
                            .stats
                            .max_health
                            .saturating_sub(self.units[&t].stats.current_health),
                    )
                })
                .filter(|(_, d)| *d > 0)
                .collect::<Vec<_>>()
        });
        for source in 0..2 {
            let recipients = &deficits[1 - source];
            let total = recipients.iter().map(|(_, d)| u128::from(*d)).sum::<u128>();
            if total == 0 {
                continue;
            }
            let ratio = p.program.transfer_percent;
            let budget = (overflow[source]
                .checked_mul(u128::from(ratio.numerator))
                .ok_or(GameError::InvalidAction)?
                / (100 * u128::from(ratio.denominator)))
            .min(total);
            let mut shares: Vec<_> = recipients
                .iter()
                .map(|(t, d)| (*t, *d, (budget * u128::from(*d) / total) as u32))
                .collect();
            let mut remainder =
                budget - shares.iter().map(|(_, _, n)| u128::from(*n)).sum::<u128>();
            for (_, deficit, share) in &mut shares {
                if remainder > 0 && *share < *deficit {
                    *share += 1;
                    remainder -= 1;
                }
            }
            for (target, _, share) in shares {
                if share > 0 {
                    self.pair_heal(p, target, share, now)?;
                }
            }
        }
        Ok(())
    }
    fn pair_burst(&mut self, p: &Pair, now: u64) -> Result<(), GameError> {
        let (pattern, percent) = p
            .program
            .modifiers
            .iter()
            .find_map(|m| match &m.change {
                PairChange::CoincidentBurst {
                    pattern,
                    max_hp_percent,
                    ..
                } => Some((pattern, *max_hp_percent)),
                _ => None,
            })
            .ok_or(GameError::InvalidAction)?;
        let tiles = self
            .valid_tile_area_tiles(pattern, p.centers[0], FacingDirection::Up)
            .ok_or(GameError::InvalidAction)?;
        let targets = self.collect_tile_area_targets_at(
            now,
            p.side,
            p.caster,
            &tiles,
            SkillHitTargetFilter::Any,
            true,
        );
        control::poll("pair_burst_targets")?;
        let template = self
            .damage_source_snapshot_template_for_unit(
                p.caster,
                DamageSource::Ability,
                DamageType::True,
                0,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        // This is one committed area occurrence: killing its caster does not skip later members.
        for target in targets {
            let Some(u) = self.units.get(&target).filter(|u| u.is_active()) else {
                continue;
            };
            if u.threat_class == BattleUnitThreatClass::Boss
                || matches!(
                    u.source_identity,
                    BattleUnitSourceIdentity::Abnormality { .. }
                )
            {
                let damage = (u128::from(u.stats.max_health) * u128::from(percent))
                    .div_ceil(100)
                    .min(u128::from(u32::MAX)) as u32;
                let mut snapshot = controlled_clone(&template)?;
                snapshot.base_damage = damage;
                snapshot.fact_source = controlled_clone(&p.source)?;
                self.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
                    target_id: target,
                    source_snapshot: snapshot,
                    committed_at_ms: now,
                    policy: super::super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
                });
            } else if u.owner == p.side
                || matches!(
                    u.source_identity,
                    BattleUnitSourceIdentity::CorrodedEmployee { .. }
                )
            {
                let side = u.owner;
                self.finalize_unit_death(Some(p.caster), target, side, now);
            }
            control::poll("pair_burst_effect")?;
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn paired_area_tick(
        &mut self,
        id: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(p) = self
            .sustained
            .pairs
            .pairs
            .get(&id)
            .filter(|p| p.next == now)
        else {
            return Ok(());
        };
        if !self.units.get(&p.caster).is_some_and(|u| {
            u.is_active()
                && u.fragment_source
                    .as_ref()
                    .is_some_and(|f| f.copy_uuid == p.copy)
        }) {
            return self.end_pair(id, now, SustainedEndReason::SourceUnavailable);
        }
        if !p.burst && now >= p.expires {
            return self.end_pair(id, now, SustainedEndReason::Expired);
        }
        let p = controlled_clone(p)?;
        let prior = std::mem::replace(&mut self.runtime_fact_source, controlled_clone(&p.source)?);
        let result = if p.burst {
            self.pair_burst(&p, now)
        } else {
            (|| {
                let sets = self.pair_memberships(&p, now)?;
                self.refresh_pair_members_from(&p, &sets, now)?;
                self.settle_pair(&p, &sets, now)
            })()
        };
        self.runtime_fact_source = prior;
        result?;
        if p.burst {
            return self.end_pair(id, now, SustainedEndReason::Expired);
        }
        if let Some(active) = self.sustained.pairs.pairs.get_mut(&id) {
            active.next = now
                .checked_add(u64::from(p.program.interval_ms))
                .ok_or(GameError::InvalidAction)?
                .min(p.expires);
            self.event_queue.push(BattleEvent::PairedAreaTick {
                time_ms: active.next,
                id,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
