//! Entry relations and status contributions are owned by the area, not the caster.
//! Movement/event reconciliation observes authoritative positions; dwell uses
//! one exact next boundary per area instead of periodic membership polling.
use super::*;
use crate::game::{
    ability::ground::*,
    behavior::GameError,
    stats::{StatId, StatModifier, StatModifierKind},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
struct Relation {
    generation: u64,
    entered_at: u64,
    fired: Vec<bool>,
}
crate::authoring::clone_shape::struct_shape!(Relation {
    generation,
    entered_at,
    fired
});
#[derive(Debug, Clone, PartialEq)]
pub(in crate::game::battle::core) struct GroundAreaState {
    program: GroundStatusProgram,
    declaration_seq: u64,
    sampled_at: u64,
    next_generation: u64,
    members: BTreeMap<UnitInstanceId, Relation>,
    contributions: BTreeMap<(UnitInstanceId, usize), u64>,
}
crate::authoring::clone_shape::struct_shape!(GroundAreaState {
    program,
    declaration_seq,
    sampled_at,
    next_generation,
    members,
    contributions
});
impl GroundAreaState {
    fn slow_percent(&self, unit: UnitInstanceId, status: usize) -> u32 {
        match self.program.statuses[status].kind {
            GroundStatusKind::Slow { percent } => percent,
            GroundStatusKind::RampingSlow { duration_ms, max_percent, rate_numerator, rate_denominator } => {
                self.members.get(&unit).map_or(0, |relation| {
                    let progress = u128::from(self.sampled_at.saturating_sub(relation.entered_at)) * u128::from(rate_numerator);
                    let clock = u128::from(duration_ms) * u128::from(rate_denominator);
                    if progress >= clock { max_percent } else { (progress * u128::from(max_percent) / clock) as u32 }
                })
            }
            _ => 0,
        }
    }


    pub(in crate::game::battle::core) fn membership(&self) -> crate::game::battle::core::area_observation::AreaMembership {
        crate::game::battle::core::area_observation::AreaMembership { members: self.members.len(), contributions: self.contributions.len() }
    }
    pub(super) fn new(program: GroundStatusProgram, declaration_seq: u64) -> Self {
        Self {
            program,
            declaration_seq,
            sampled_at: 0,
            next_generation: 0,
            members: BTreeMap::new(),
            contributions: BTreeMap::new(),
        }
    }
    pub(in crate::game::battle::core) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = self
            .program
            .retained_extent()?
            .checked_add(std::mem::size_of::<Self>())?
            .checked_add(self.contributions.len().checked_mul(112)?)?;
        for relation in self.members.values() {
            bytes = bytes.checked_add(128 + relation.fired.capacity())?;
        }
        Some(bytes)
    }
    pub(in crate::game::battle::core) fn entity_count(&self) -> usize {
        self.members
            .len()
            .saturating_add(self.contributions.len())
            .saturating_add(self.members.values().map(|r| r.fired.len()).sum::<usize>())
    }
    pub(in crate::game::battle::core) fn has_stun_target(&self, target: UnitInstanceId) -> Result<bool, GameError> {
        control::charge(self.contributions.len() as u64, "ground_stun_owner_lookup")?;
        Ok(self.contributions.keys().any(|(unit, status)| *unit == target
            && self.program.statuses[*status].kind == GroundStatusKind::Stun))
    }
    pub(in crate::game::battle::core) fn stun_targets(&self) -> Result<Vec<UnitInstanceId>, GameError> {
        control::charge(self.contributions.len() as u64, "ground_stun_end_targets")?;
        crate::game::battle::runtime_control::admit_retained_growth(0,
            self.contributions.len().checked_mul(std::mem::size_of::<UnitInstanceId>()),
            "ground_stun_end_targets")?;
        let mut targets = Vec::with_capacity(self.contributions.len());
        for (target, status) in self.contributions.keys() {
            if self.program.statuses[*status].kind == GroundStatusKind::Stun {
                targets.push(*target);
            }
        }
        // BTreeMap keys are already ordered by target then status.
        targets.dedup();
        Ok(targets)
    }
    pub(super) fn affected_targets(&self) -> BTreeSet<UnitInstanceId> {
        self.contributions
            .keys()
            .map(|(target, _)| *target)
            .collect()
    }
    fn next_boundary(&self, end: u64) -> Option<u64> {
        let dwell = self
            .members
            .values()
            .flat_map(|relation| {
                self.program
                    .rules
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, rule)| {
                        if relation.fired[index] {
                            return None;
                        }
                        match rule.trigger {
                            GroundTrigger::Dwell { duration_ms } => relation
                                .entered_at
                                .checked_add(u64::from(duration_ms))
                                .filter(|at| *at < end),
                            _ => None,
                        }
                    })
            })
            .min();
        let ramp = self
            .contributions
            .keys()
            .filter_map(|(target, status)| {
                let GroundStatusKind::RampingSlow {
                    duration_ms,
                    max_percent,
                    rate_numerator,
                    rate_denominator,
                } = self.program.statuses[*status].kind
                else {
                    return None;
                };
                let relation = self.members.get(target)?;
                if max_percent == 0 {
                    return None;
                }
                let elapsed = u128::from(self.sampled_at.saturating_sub(relation.entered_at));
                let clock = u128::from(duration_ms) * u128::from(rate_denominator);
                let rate = u128::from(max_percent) * u128::from(rate_numerator);
                let progress = elapsed * u128::from(rate_numerator);
                let current = if progress >= clock {
                    u128::from(max_percent)
                } else {
                    progress * u128::from(max_percent) / clock
                };
                if current >= u128::from(max_percent) {
                    return None;
                }
                let offset = u64::try_from(((current + 1) * clock).div_ceil(rate)).ok()?;
                relation
                    .entered_at
                    .checked_add(offset)
                    .filter(|at| *at < end)
            })
            .min();
        dwell.into_iter().chain(ramp).min()
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn reconcile_ground_status(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        control::charge(self.active_areas.len() as u64, "ground_area_scan")?;
        let mut ids: Vec<_> = self
            .active_areas
            .values()
            .filter(|a| a.ground_status.is_some())
            .map(|a| a.area_id)
            .collect();
        ids.sort();
        for id in ids {
            self.reconcile_one_ground_status(id, now)?;
        }
        Ok(())
    }
    fn reconcile_one_ground_status(&mut self, id: Uuid, now: u64) -> Result<(), GameError> {
        let Some(area) = self.active_areas.get(&id) else {
            return Ok(());
        };
        if now >= area.expires_at_ms {
            return self.finish_skill_area_at(id, now);
        }
        let Some(ground) = area.ground_status.as_ref() else {
            return Ok(());
        };
        let Some(tiles) =
            self.valid_tile_area_tiles(&area.tile_range, area.tile_anchor, area.facing)
        else {
            return Err(GameError::InvalidAction);
        };
        let recipients: BTreeSet<_> = self
            .collect_tile_area_targets_at(
                now,
                area.caster_owner,
                area.caster_instance_id,
                &tiles,
                area.hit_targets,
                area.include_caster,
            )
            .into_iter()
            .collect();
        control::charge(
            (ground.members.len() + ground.contributions.len() + recipients.len()) as u64,
            "ground_membership_compare",
        )?;
        if recipients.len() == ground.members.len()
            && recipients
                .iter()
                .all(|target| ground.members.contains_key(target))
            && ground
                .next_boundary(area.expires_at_ms)
                .is_none_or(|at| at > now)
            && ground
                .contributions
                .keys()
                .all(|(target, _)| self.units.get(target).is_some_and(|u| u.is_active()))
        {
            return Ok(());
        }
        let mut ended_stuns = if self.game_data.equipment_state_catalog().has_owner_stun_end {
            ground.stun_targets()?
        } else { Vec::new() };
        let before_membership = ground.membership();
        let previous_entities = ground.entity_count();
        let mut next = crate::authoring::clone_shape::controlled_clone(ground.as_ref())?;
        let caster = area.caster_instance_id;
        let expires = area.expires_at_ms;
        let mut changed = BTreeSet::new();
        let mut stuns = BTreeSet::new();
        let source = area.source_snapshot.fact_source.clone();
        let producer_bytes = area.skill_id.as_str().len().checked_add(area.step_id.len());
        crate::game::battle::runtime_control::admit_retained_growth(0, producer_bytes, "ground_effect_origin")?;
        control::charge(producer_bytes.unwrap_or(usize::MAX) as u64, "ground_effect_origin")?;
        let producer = (area.source_generation.clone(), area.skill_id.clone(), area.step_id.clone(), area.cast_seq,
            area.source_snapshot.skill_cast_source.clone());
        let mut vulnerabilities = Vec::new();
        // Only integer-percentage boundaries are scheduled; no per-frame polling.
        // Reconciliation can also jump across boundaries after a restored checkpoint.
        for (target, status) in next.contributions.keys() {
            if matches!(
                next.program.statuses[*status].kind,
                GroundStatusKind::RampingSlow { .. }
            ) {
                changed.insert(*target);
            }
        }
        next.sampled_at = now;
        control::charge(
            (next.members.len() + recipients.len() + next.contributions.len()) as u64
                * (next.program.rules.len() as u64 + 1),
            "ground_relations",
        )?;
        let exited: Vec<_> = next
            .members
            .keys()
            .filter(|target| !recipients.contains(target))
            .copied()
            .collect();
        for target in exited {
            let relation = next.members.remove(&target).unwrap();
            for rule in &next.program.rules {
                if rule.trigger == GroundTrigger::Exit
                    && next.contributions.get(&(target, rule.status)) == Some(&relation.generation)
                {
                    next.contributions.remove(&(target, rule.status));
                    changed.insert(target);
                }
            }
            next.contributions.retain(|(unit, status), _| {
                let remove = *unit == target
                    && next.program.statuses[*status].lifetime == GroundStatusLifetime::UntilExit;
                if remove {
                    changed.insert(target);
                }
                !remove
            });
        }
        // A target's disappearance ends even contributions that outlive exit.
        next.contributions.retain(|(target, _), _| {
            let live = self.units.get(target).is_some_and(|u| u.is_active());
            if !live {
                changed.insert(*target);
            }
            live
        });
        for target in recipients {
            let entered = !next.members.contains_key(&target);
            if entered {
                next.next_generation = next
                    .next_generation
                    .checked_add(1)
                    .ok_or(GameError::InvalidAction)?;
                self.admit_authoring_runtime_replacement(
                    previous_entities,
                    next.entity_count()
                        .saturating_add(1 + next.program.rules.len()),
                    "ground_relation",
                )?;
                next.members.insert(
                    target,
                    Relation {
                        generation: next.next_generation,
                        entered_at: now,
                        fired: vec![false; next.program.rules.len()],
                    },
                );
            }
            let relation_entities = next.entity_count().saturating_sub(next.contributions.len());
            let relation = next.members.get_mut(&target).unwrap();
            for (index, rule) in next.program.rules.iter().enumerate() {
                let fire = match rule.trigger {
                    GroundTrigger::Enter => entered,
                    GroundTrigger::Dwell { duration_ms } => {
                        !relation.fired[index]
                            && now >= relation.entered_at.saturating_add(u64::from(duration_ms))
                    }
                    GroundTrigger::Exit => false,
                };
                if !fire {
                    continue;
                }
                relation.fired[index] = true;
                let key = (target, rule.status);
                if rule.action == GroundStatusAction::Clear {
                    if next.contributions.get(&key) == Some(&relation.generation) {
                        next.contributions.remove(&key);
                        changed.insert(target);
                    }
                    continue;
                }
                let status = &next.program.statuses[rule.status];
                let traits = &self.units[&target].target_traits;
                let trait_immune = match status.kind {
                    GroundStatusKind::Slow { .. } | GroundStatusKind::RampingSlow { .. } => {
                        traits.contains(&crate::game::battle::types::UnitTargetTrait::SlowImmune)
                    }
                    GroundStatusKind::Stun => {
                        traits.contains(&crate::game::battle::types::UnitTargetTrait::StunImmune)
                    }
                    GroundStatusKind::DamageTaken { .. } => false,
                };
                if trait_immune
                    || !self.stress_allied_effect_allowed(Some(caster), target)
                    || (matches!(
                        status.kind,
                        GroundStatusKind::Slow { .. }
                            | GroundStatusKind::RampingSlow { .. }
                            | GroundStatusKind::Stun
                    ) && self.sustained_invulnerable(target, now)?)
                {
                    continue;
                }
                if !next.contributions.contains_key(&key) {
                    self.admit_authoring_runtime_replacement(
                        previous_entities,
                        relation_entities
                            .saturating_add(next.contributions.len())
                            .saturating_add(1),
                        "ground_contribution",
                    )?;
                    if status.kind == GroundStatusKind::Stun {
                        stuns.insert(target);
                        if let Some(v) = &status.vulnerability {
                            control::charge_serialized(v, "ground_vulnerability_copy")?;
                            vulnerabilities.push((
                                target,
                                rule.status,
                                crate::authoring::clone_shape::controlled_clone(v)?,
                            ));
                        }
                    }
                }
                next.contributions.insert(key, relation.generation);
                changed.insert(target);
            }
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            next.retained_extent(),
            "ground_relation_commit",
        )?;
        let boundary = next.next_boundary(expires);
        // Preserve the unmodified base before publishing the first speed contribution.
        for target in &changed {
            self.ensure_passive_stat_base(*target)?;
        }
        if !ended_stuns.is_empty() {
            let remaining = next.stun_targets()?;
            control::charge(ended_stuns.len() as u64 * 64, "ground_stun_end_compare")?;
            ended_stuns.retain(|target| remaining.binary_search(target).is_err());
        }
        let area = self.active_areas.get_mut(&id).unwrap();
        area.ground_status = Some(Box::new(next));
        let boundary_changed = area.next_tick_ms != boundary;
        area.next_tick_ms = boundary;
        if boundary_changed {
            let event = boundary.map(|time_ms| BattleEvent::SkillAreaTick {
                time_ms,
                area_id: id,
                cast_seq: area.cast_seq,
                step_index: area.step_index,
                skill_id: area.skill_id.clone(),
                step_id: area.step_id.clone(),
                center: area.center,
                cause: crate::game::battle::event_log::BattleEventCause::Parent {
                    seq: area.ground_status.as_ref().unwrap().declaration_seq,
                },
            });
            self.event_queue.discard_ground_dwell_ticks(id)?;
            if let Some(event) = event {
                self.event_queue.push(event);
            }
        }
        let changed_any = !changed.is_empty();
        for target in changed {
            self.refresh_sustained_target(target)?;
        }
        // State is already installed: cancellation may reconcile other owned effects.
        for target in stuns {
            self.cancel_unreleased_actions_for_hard_cc(now, caster, target);
        }
        for (target, status_index, v) in vulnerabilities {
            let bytes = std::mem::size_of::<super::super::super::area_observation::AreaEffectSource>()
                .checked_add(producer.1.as_str().len()).and_then(|n| n.checked_add(producer.2.len()))
                .and_then(|n| n.checked_add(v.definition.len()))
                .and_then(|n| producer.4.as_ref().map_or(Some(n), |cast|
                    n.checked_add(std::mem::size_of_val(cast.as_ref()))
                        .and_then(|n| n.checked_add(cast.skill_id.as_str().len()))))
                .and_then(|n| n.checked_add(v.modifier_source.as_ref().map_or(0, |m| m.definition.len() + m.rule.len())));
            crate::game::battle::runtime_control::admit_retained_growth(0, bytes, "ground_effect_origin")?;
            control::charge(bytes.unwrap_or(usize::MAX) as u64, "ground_effect_origin")?;
            let origin = std::sync::Arc::new(super::super::super::area_observation::AreaEffectSource {
                generation: producer.0.clone(), captured_cast: producer.4.clone(),
                area_id: id, skill_id: producer.1.clone(), step_id: producer.2.clone(),
                cast_sequence: producer.3, status_index, definition: v.definition.clone(), modifier_source: v.modifier_source.clone(),
            });
            let prior = std::mem::replace(&mut self.runtime_fact_source, source.clone());
            let result = self.begin_sustained_from_area(
                caster,
                target,
                crate::game::ability::sustained::SustainedEffectDef::HitStatus {
                    duration_ms: v.duration_ms,
                    profile: crate::game::ability::sustained::HitStatusProfile {
                        definition: v.definition,
                        kind: crate::game::ability::sustained::HitStatusKind::DamageTaken {
                            percent: v.percent,
                        },
                        vulnerability: None,
                        modifier_source: v.modifier_source,
                    },
                },
                now,
                origin,
            );
            self.runtime_fact_source = prior;
            result?;
        }
        if changed_any {
            self.refresh_block_state();
            self.schedule_continuous_movement_tick(now);
        }
        if let Some(area) = self.active_areas.get(&id) {
            crate::game::battle::core::area_observation::record(&mut self.authoring_observation, area, now, crate::game::battle::core::area_observation::AreaChange::Reconciled, Some(before_membership))?;
        }
        for target in ended_stuns {
            self.dispatch_equipment_stun_end(caster, target, now)?;
        }
        control::poll("ground_reconciled")
    }
    pub(in crate::game::battle::core) fn ground_status_totals(
        &self,
        target: UnitInstanceId,
    ) -> Result<(i32, i32), GameError> {
        control::charge(self.ground_composition_work(), "ground_composition")?;
        let mut groups = BTreeMap::new();
        for area in self.active_areas.values() {
            let Some(ground) = &area.ground_status else {
                continue;
            };
            for ((unit, status), _) in &ground.contributions {
                if *unit != target {
                    continue;
                }
                let def = &ground.program.statuses[*status];
                let (slow, damage) = match def.kind {
                    GroundStatusKind::Slow { .. } | GroundStatusKind::RampingSlow { .. } => (ground.slow_percent(*unit, *status), 0),
                    GroundStatusKind::DamageTaken { percent } => (0, percent),
                    GroundStatusKind::Stun => continue,
                };
                let value = groups
                    .entry((&area.skill_id, &def.definition))
                    .or_insert((0u32, 0u32));
                match def.overlap {
                    GroundOverlap::Sum => {
                        value.0 = value.0.saturating_add(slow);
                        value.1 = value.1.saturating_add(damage);
                    }
                    GroundOverlap::Strongest => {
                        value.0 = value.0.max(slow);
                        value.1 = value.1.max(damage);
                    }
                }
            }
        }
        Ok(groups.values().fold((0i32, 0i32), |sum, value| {
            (
                sum.0.saturating_add(value.0.min(i32::MAX as u32) as i32),
                sum.1.saturating_add(value.1.min(i32::MAX as u32) as i32),
            )
        }))
    }
    pub(in crate::game::battle::core) fn ground_stat_modifier(
        &self,
        target: UnitInstanceId,
    ) -> Result<StatModifier, GameError> {
        Ok(StatModifier {
            stat: StatId::MoveSpeedUnitsPerMs,
            kind: StatModifierKind::Percent,
            value: -self.ground_status_totals(target)?.0,
        })
    }
    fn ground_composition_work(&self) -> u64 {
        self.active_areas
            .values()
            .fold(self.active_areas.len() as u64, |sum, a| {
                sum.saturating_add(a.ground_status.as_ref().map_or(0, |g| g.entity_count()) as u64)
            })
    }
    pub(in crate::game::battle::core) fn has_ground_contribution(
        &self,
        target: UnitInstanceId,
    ) -> bool {
        self.active_areas.values().any(|a| {
            a.ground_status
                .as_ref()
                .is_some_and(|g| g.contributions.keys().any(|(unit, _)| *unit == target))
        })
    }
    pub(in crate::game::battle::core) fn ground_stun_until(
        &self,
        target: UnitInstanceId,
        now: Option<u64>,
    ) -> Option<u64> {
        if control::charge(self.ground_composition_work(), "ground_stun_query").is_err() {
            return None;
        }
        self.active_areas
            .values()
            .filter(|a| now.is_none_or(|now| now < a.expires_at_ms))
            .filter(|a| {
                a.ground_status.as_ref().is_some_and(|g| {
                    g.contributions.keys().any(|(unit, status)| {
                        *unit == target
                            && g.program.statuses[*status].kind == GroundStatusKind::Stun
                    })
                })
            })
            .map(|a| a.expires_at_ms.saturating_sub(1))
            .max()
    }
}

impl BattleCore {
    pub(in crate::game::battle::core) fn ground_target_movement_impaired(&self, target: UnitInstanceId, now: u64) -> Result<bool, GameError> {
        control::charge(self.ground_composition_work(), "ground_movement_impairment_query")?;
        Ok(self.active_areas.values().filter(|area| now < area.expires_at_ms).any(|area| {
            area.ground_status.as_ref().is_some_and(|ground| ground.contributions.keys().any(|(unit, status)| {
                *unit == target && (ground.program.statuses[*status].kind == GroundStatusKind::Stun || ground.slow_percent(*unit, *status) > 0)
            }))
        }))
    }
}
