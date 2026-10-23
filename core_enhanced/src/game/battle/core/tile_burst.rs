//! Admission-owned stack consumption and independent finite released explosions.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::{
            marks::{ExplosionDedup, MarkChange, MarkRatio},
            tile_burst::{TileBurstDef, TileBurstSource},
            SkillDef, SkillEffectDef,
        },
        battle::{
            damage::{DamageModifiers, DamageSource, DamageSourceSnapshot},
            enums::BattleEvent,
            ids::UnitInstanceId,
            runtime_facts::{ActionHandle, RuntimeFactSource},
            tile_range::FacingDirection,
        },
        behavior::GameError,
        resources::Position,
    },
};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Debug, Clone)]
struct Center {
    tile: Position,
    rounds: u64,
    damage: u32,
}
crate::authoring::clone_shape::struct_shape!(Center {
    tile,
    rounds,
    damage
});
#[derive(Debug, Clone)]
struct Plan {
    definition: Arc<TileBurstDef>,
    centers: Vec<Center>,
    orb_ms: Option<u64>,
    orb_relic_copy: Option<uuid::Uuid>,
    applied: Vec<crate::game::ability::marks::AppliedMarkModifier>,
    interval_ms: u64,
}
crate::authoring::clone_shape::struct_shape!(Plan {
    definition,
    centers,
    orb_ms,
    orb_relic_copy,
    applied,
    interval_ms
});
impl Plan {
    fn retained_extent(&self) -> Option<usize> {
        let d = &self.definition;
        let mut n = std::mem::size_of::<Self>()
            .checked_add(128)?
            .checked_add(
                self.centers
                    .capacity()
                    .checked_mul(std::mem::size_of::<Center>())?,
            )?
            .checked_add(crate::game::relic::structural::support::pattern_extent(
                &d.range,
            )?)?;
        n = n.checked_add(
            d.modifiers
                .capacity()
                .checked_mul(std::mem::size_of::<crate::game::ability::marks::MarkModifier>())?,
        )?;
        for m in &d.modifiers {
            n = n
                .checked_add(m.id.capacity())?
                .checked_add(m.relic_definition.capacity())?;
            if let MarkChange::Range { pattern } = &m.change {
                n = n.checked_add(crate::game::relic::structural::support::pattern_extent(
                    pattern,
                )?)?;
            }
        }
        if let TileBurstSource::WeaponStacks {
            weapon,
            key,
            conversion,
            ..
        } = &d.source
        {
            n = n
                .checked_add(weapon.capacity())?
                .checked_add(key.capacity())?;
            if let Some(c) = conversion {
                n = n.checked_add(c.relic_definition.capacity())?.checked_add(
                    crate::game::relic::structural::support::pattern_extent(&c.range)?,
                )?;
            }
        }
        n = n.checked_add(self.applied.capacity().checked_mul(std::mem::size_of::<
            crate::game::ability::marks::AppliedMarkModifier,
        >())?)?;
        for m in &self.applied {
            n = n
                .checked_add(m.rule_id.capacity())?
                .checked_add(m.relic_definition.capacity())?;
        }
        Some(n)
    }
}
#[derive(Debug, Clone)]
struct Pending {
    action: ActionHandle,
    caster: UnitInstanceId,
    skill: crate::game::ability::SkillId,
    plan: Plan,
}
crate::authoring::clone_shape::struct_shape!(Pending {
    action,
    caster,
    skill,
    plan
});
#[derive(Debug, Clone)]
struct Released {
    plan: Plan,
    snapshot: DamageSourceSnapshot,
    round: u64,
    next_at: u64,
}
crate::authoring::clone_shape::struct_shape!(Released {
    plan,
    snapshot,
    round,
    next_at
});
#[derive(Debug, Clone, Copy)]
struct Freeze {
    id: u64,
    remaining_ms: u64,
}
crate::authoring::clone_shape::copy_shape!(Freeze);
#[derive(Debug, Clone, Default)]
pub(super) struct TileBurstState {
    next: u64,
    pending: Vec<Pending>,
    released: BTreeMap<u64, Released>,
    freeze: Option<Freeze>,
}
crate::authoring::clone_shape::struct_shape!(TileBurstState {
    next,
    pending,
    released,
    freeze
});
impl TileBurstState {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n = self
            .pending
            .capacity()
            .checked_mul(std::mem::size_of::<Pending>())?;
        for p in &self.pending {
            n = n
                .checked_add(p.plan.retained_extent()?)?
                .checked_add(p.skill.as_str().len().checked_add(32)?)?;
        }
        for r in self.released.values() {
            n = n
                .checked_add(std::mem::size_of::<Released>() + 80)?
                .checked_add(r.plan.retained_extent()?)?
                .checked_add(r.snapshot.fact_retained_extent()?)?;
        }
        Some(n)
    }
    pub(super) fn entity_count(&self) -> usize {
        self.pending.len()
            + self.released.len()
            + self
                .pending
                .iter()
                .map(|p| p.plan.centers.len())
                .sum::<usize>()
            + self
                .released
                .values()
                .map(|r| r.plan.centers.len())
                .sum::<usize>()
    }
    pub(super) fn forced_pause_remaining(&self) -> Option<u64> {
        self.freeze.map(|f| f.remaining_ms)
    }
}
fn amount(base: u64, factor: MarkRatio) -> Result<u32, GameError> {
    let value =
        (u128::from(base) * u128::from(factor.numerator)).div_ceil(u128::from(factor.denominator));
    if value > i32::MAX as u128 {
        return Err(GameError::InvalidAction);
    }
    Ok(value as u32)
}
fn bonus(percent: MarkRatio) -> Result<MarkRatio, GameError> {
    super::marks::ratio_sum(
        MarkRatio {
            numerator: 1,
            denominator: 1,
        },
        super::marks::ratio_product(
            percent,
            MarkRatio {
                numerator: 1,
                denominator: 100,
            },
        )?,
    )
}
impl BattleCore {
    fn tile_burst_definition(skill: &SkillDef) -> Option<&Arc<TileBurstDef>> {
        skill.steps.iter().flat_map(|s| &s.effects).find_map(|e| {
            if let SkillEffectDef::TileBurst { program } = &e.op {
                Some(program)
            } else {
                None
            }
        })
    }
    pub(super) fn tile_burst_skill(skill: &SkillDef) -> bool {
        Self::tile_burst_definition(skill).is_some()
    }
    pub(super) fn has_prepared_tile_burst(&self, action: &ActionHandle) -> bool {
        self.sustained
            .tile_bursts
            .pending
            .iter()
            .any(|p| &p.action == action)
    }
    fn plan_tile_burst(
        &self,
        caster: UnitInstanceId,
        definition: &TileBurstDef,
        now: u64,
    ) -> Result<(Plan, Option<(uuid::Uuid, String)>), GameError> {
        control::charge_serialized(definition, "tile_burst_plan")?;
        let mut d = definition.clone();
        let mut applied = Vec::new();
        let mut orb_relic_copy = None;
        for m in &definition.modifiers {
            let Some(relic_copy_uuid) =
                self.equipped_normal_relic_copy(caster, &m.relic_definition)
            else {
                continue;
            };
            applied.push(crate::game::ability::marks::AppliedMarkModifier {
                rule_id: m.id.clone(),
                relic_definition: m.relic_definition.clone(),
                relic_copy_uuid,
            });
            match &m.change {
                MarkChange::Range { pattern } => d.range = pattern.clone(),
                MarkChange::ExplosionPower { percent } => {
                    d.bonus_percent = super::marks::ratio_sum(d.bonus_percent, *percent)?
                }
                MarkChange::ExplosionOverlap => d.dedup = ExplosionDedup::Explosion,
                _ => return Err(GameError::InvalidAction),
            }
        }
        let mut centers = Vec::new();
        let mut store = None;
        let mut orb_ms = None;
        let mut interval_ms = 1;
        match &d.source {
            TileBurstSource::WalkablePath => {
                if self.scenario.battlefield.walkable_paths.is_empty() {
                    return Err(GameError::InvalidAction);
                }
                // Actual path membership is captured at release; admission verifies domain availability.
                amount(u64::from(d.power), bonus(d.bonus_percent)?)?;
            }
            TileBurstSource::WeaponStacks {
                weapon,
                key,
                divisor,
                rank_bonus_percent,
                interval_ms: interval,
                conversion,
            } => {
                interval_ms = u64::from(*interval);
                control::charge(self.items.len() as u64, "tile_burst_weapon_owner")?;
                let mut copies = self
                    .items
                    .values()
                    .filter(|i| {
                        i.owner_unit_instance == caster
                            && self
                                .game_data
                                .equipment_data
                                .get_by_uuid(&i.base_uuid)
                                .is_some_and(|m| m.id == *weapon)
                    })
                    .map(|i| i.persistent_copy_uuid.unwrap_or(i.instance_id));
                let copy = copies.next().ok_or(GameError::InvalidAction)?;
                if copies.next().is_some() {
                    return Err(GameError::InvalidAction);
                }
                let pool_key = (copy, key.clone());
                let pool = self
                    .sustained
                    .weapon_marks
                    .pools
                    .get(&pool_key)
                    .ok_or(GameError::InvalidAction)?;
                control::charge(
                    (pool.units.len() + pool.tiles.len()) as u64,
                    "tile_burst_stack_capture",
                )?;
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    (pool.units.len() + pool.tiles.len()).checked_mul(160),
                    "tile_burst_centers",
                )?;
                let mut totals = pool.tiles.clone();
                for (unit, stacks) in &pool.units {
                    if !self.units.get(unit).is_some_and(|u| u.is_active()) {
                        continue;
                    }
                    let Some(tile) = self.sample_unit_body_at(*unit, now).map(|b| b.projected_tile()) else {
                        continue;
                    };
                    let n = totals.entry((tile.x, tile.y)).or_default();
                    *n = n.checked_add(*stacks).ok_or(GameError::InvalidAction)?;
                }
                let total = totals
                    .values()
                    .try_fold(0u64, |sum, n| sum.checked_add(*n))
                    .ok_or(GameError::InvalidAction)?;
                if total == 0 {
                    return Err(GameError::InvalidAction);
                }
                if let Some(c) = conversion.as_ref().filter(|c| {
                    total >= c.threshold
                        && self
                            .equipped_normal_relic_copy(caster, &c.relic_definition)
                            .is_some()
                }) {
                    let power = super::marks::ratio_sum(
                        MarkRatio {
                            numerator: u64::from(c.base_power),
                            denominator: 1,
                        },
                        super::marks::ratio_product(
                            MarkRatio {
                                numerator: total,
                                denominator: 1,
                            },
                            c.power_per_stack,
                        )?,
                    )?;
                    let damage = amount(
                        1,
                        super::marks::ratio_product(power, bonus(d.bonus_percent)?)?,
                    )?;
                    d.range = c.range.clone();
                    d.dedup = ExplosionDedup::Wave;
                    centers.push(Center {
                        tile: Position {
                            x: i32::from(self.scenario.battlefield.width) / 2,
                            y: i32::from(self.scenario.battlefield.height) / 2,
                        },
                        rounds: 1,
                        damage,
                    });
                    orb_ms = Some(u64::from(c.presentation_ms));
                    orb_relic_copy = self.equipped_normal_relic_copy(caster, &c.relic_definition);
                } else {
                    for ((x, y), n) in totals {
                        if n == 0 {
                            continue;
                        }
                        let rank = n / u64::from(*divisor);
                        let rounds = if rank == 0 { n } else { rank };
                        let power = super::marks::ratio_product(
                            bonus(super::marks::ratio_product(
                                MarkRatio {
                                    numerator: rank,
                                    denominator: 1,
                                },
                                *rank_bonus_percent,
                            )?)?,
                            bonus(d.bonus_percent)?,
                        )?;
                        centers.push(Center {
                            tile: Position { x, y },
                            rounds,
                            damage: amount(u64::from(d.power), power)?,
                        });
                    }
                }
                store = Some(pool_key);
            }
        }
        centers.sort_by_key(|c| (c.tile.y, c.tile.x));
        let rounds = centers.iter().map(|c| c.rounds).max().unwrap_or(1);
        control::charge(rounds, "tile_burst_finite_rounds")?;
        now.checked_add(u64::from(d.delay_ms))
            .and_then(|t| t.checked_add((rounds - 1).checked_mul(interval_ms)?))
            .ok_or(GameError::InvalidAction)?;
        let plan = Plan {
            definition: Arc::new(d),
            centers,
            orb_ms,
            orb_relic_copy,
            applied,
            interval_ms,
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            plan.retained_extent(),
            "tile_burst_plan_storage",
        )?;
        Ok((plan, store))
    }
    pub(super) fn validate_tile_burst(
        &self,
        caster: UnitInstanceId,
        skill: &SkillDef,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(d) = Self::tile_burst_definition(skill) else {
            return Ok(());
        };
        if self
            .sustained
            .tile_bursts
            .pending
            .iter()
            .any(|p| p.caster == caster && p.skill == skill.id)
        {
            return Ok(());
        }
        self.plan_tile_burst(caster, d, now).map(|_| ())
    }
    pub(super) fn prepare_tile_burst(
        &mut self,
        caster: UnitInstanceId,
        skill: &SkillDef,
        action: &ActionHandle,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(d) = Self::tile_burst_definition(skill) else {
            return Ok(());
        };
        if self.has_prepared_tile_burst(action) {
            return Ok(());
        }
        let (plan, store) = self.plan_tile_burst(caster, d, now)?;
        self.sustained.tile_bursts.pending.push(Pending {
            action: action.clone(),
            caster,
            skill: skill.id.clone(),
            plan,
        });
        if let Some(key) = store {
            self.sustained.weapon_marks.pools.remove(&key);
        }
        control::poll("tile_burst_stacks_consumed")
    }
    pub(super) fn release_tile_burst(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
    ) -> Result<super::types::SkillStepResult, GameError> {
        let action = self
            .runtime_fact_source
            .action
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let index = self
            .sustained
            .tile_bursts
            .pending
            .iter()
            .position(|p| p.caster == caster && &p.action == action)
            .ok_or(GameError::InvalidAction)?;
        let mut p = self.sustained.tile_bursts.pending.remove(index);
        if matches!(p.plan.definition.source, TileBurstSource::WalkablePath) {
            let paths = &self.scenario.battlefield.walkable_paths;
            control::charge(paths.len() as u64, "tile_burst_path_release")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                paths.len().checked_mul(std::mem::size_of::<Center>()),
                "tile_burst_path_centers",
            )?;
            let damage = amount(
                u64::from(p.plan.definition.power),
                bonus(p.plan.definition.bonus_percent)?,
            )?;
            p.plan.centers = paths
                .iter()
                .copied()
                .map(|tile| Center {
                    tile,
                    rounds: 1,
                    damage,
                })
                .collect();
            p.plan.centers.sort_by_key(|c| (c.tile.y, c.tile.x));
        }
        let mut snapshot = self
            .damage_source_snapshot_template_for_unit(
                caster,
                DamageSource::Ability,
                p.plan.definition.damage_type,
                1,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        let ticket = self
            .runtime_facts
            .delivery(&p.action)?
            .ok_or(GameError::InvalidAction)?;
        snapshot.fact_source.resource_capabilities = RuntimeFactSource::resource_capabilities(
            snapshot.fact_source.resource_context().cloned(),
            ticket.resource.clone(),
        )?;
        snapshot.fact_source.delivery = Some(ticket);
        let id = self
            .sustained
            .tile_bursts
            .next
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        let next_at = if p.plan.orb_ms.is_some() {
            now
        } else {
            now.checked_add(u64::from(p.plan.definition.delay_ms))
                .ok_or(GameError::InvalidAction)?
        };
        if let Some(remaining_ms) = p.plan.orb_ms {
            if self.forced_pause_remaining_ms().is_some() {
                return Err(GameError::InvalidAction);
            }
            self.sustained.tile_bursts.freeze = Some(Freeze { id, remaining_ms });
        } else {
            self.event_queue.push_with_context(
                BattleEvent::TileBurstWave {
                    time_ms: next_at,
                    id,
                },
                None,
                snapshot.fact_source.clone(),
            );
        }
        self.sustained.tile_bursts.next = id;
        self.sustained.tile_bursts.released.insert(
            id,
            Released {
                plan: p.plan,
                snapshot,
                round: 0,
                next_at,
            },
        );
        control::poll("tile_burst_released")?;
        Ok(super::types::SkillStepResult {
            applied_effect_count: 1,
            ..Default::default()
        })
    }
    pub(super) fn tile_burst_wave(&mut self, id: u64, now: u64) -> Result<(), GameError> {
        let Some(mut r) = self.sustained.tile_bursts.released.remove(&id) else {
            return Ok(());
        };
        if r.next_at != now {
            self.sustained.tile_bursts.released.insert(id, r);
            return Ok(());
        }
        let mut wave: BTreeMap<UnitInstanceId, u32> = BTreeMap::new();
        let mut hits = Vec::new();
        for c in r.plan.centers.iter().filter(|c| c.rounds > r.round) {
            let tiles = r
                .plan
                .definition
                .range
                .affected_tiles(c.tile, FacingDirection::Up)
                .map_err(GameError::InvalidStaticData)?;
            let targets = self.collect_tile_area_targets_at(
                now,
                r.snapshot.source_side,
                r.snapshot.source_id,
                &tiles,
                crate::game::ability::SkillHitTargetFilter::Enemies,
                false,
            );
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                targets.len().checked_mul(128),
                "tile_burst_hit_accumulation",
            )?;
            for target in targets {
                if r.plan.definition.dedup == ExplosionDedup::Explosion {
                    hits.push((target, c.damage));
                } else {
                    wave.entry(target)
                        .and_modify(|n| *n = (*n).max(c.damage))
                        .or_insert(c.damage);
                }
            }
            control::poll("tile_burst_target_query")?;
        }
        if r.plan.definition.dedup == ExplosionDedup::Wave {
            hits.extend(wave);
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            hits.capacity()
                .checked_mul(std::mem::size_of::<(UnitInstanceId, u32)>()),
            "tile_burst_hits",
        )?;
        for (target, damage) in hits {
            if !self.units.get(&target).is_some_and(|u| u.is_active()) {
                continue;
            }
            let mut snapshot = r.snapshot.clone();
            snapshot.base_damage = damage;
            snapshot.committed_at_ms = now;
            let snapshot = self.materialize_damage_source_snapshot_for_target_hit(
                snapshot,
                target,
                u32::try_from(r.round).map_err(|_| GameError::InvalidAction)?,
            );
            self.execute_committed_damage(super::damage_runtime::CommittedDamageInput {
                target_id: target,
                source_snapshot: snapshot,
                committed_at_ms: now,
                policy: super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
            })
            .ok_or(GameError::InvalidAction)?;
            control::poll("tile_burst_damage")?;
        }
        r.round += 1;
        if r.plan.centers.iter().any(|c| c.rounds > r.round) {
            r.next_at = now
                .checked_add(r.plan.interval_ms)
                .ok_or(GameError::InvalidAction)?;
            self.event_queue.push_with_context(
                BattleEvent::TileBurstWave {
                    time_ms: r.next_at,
                    id,
                },
                None,
                r.snapshot.fact_source.clone(),
            );
            self.sustained.tile_bursts.released.insert(id, r);
        } else if let Some(ticket) = r.snapshot.fact_source.delivery {
            self.runtime_facts.close_delivery(&ticket, now)?;
        }
        control::poll("tile_burst_wave_complete")
    }
    pub(super) fn advance_tile_burst_host(&mut self, elapsed: u64) -> Result<(), GameError> {
        let Some(f) = self.sustained.tile_bursts.freeze.as_mut() else {
            return Ok(());
        };
        f.remaining_ms = f.remaining_ms.saturating_sub(elapsed);
        if f.remaining_ms == 0 {
            let id = f.id;
            self.sustained.tile_bursts.freeze = None;
            let now = self
                .sustained
                .tile_bursts
                .released
                .get(&id)
                .ok_or(GameError::InvalidAction)?
                .next_at;
            self.tile_burst_wave(id, now)?;
        }
        Ok(())
    }
    pub(super) fn reconcile_tile_bursts(&mut self) -> Result<(), GameError> {
        control::charge(
            self.sustained.tile_bursts.pending.len() as u64,
            "tile_burst_pending_reconcile",
        )?;
        self.sustained.tile_bursts.pending.retain(|p| {
            self.runtime_facts
                .action_lifetime(&p.action)
                .is_some_and(|(_, terminal, _)| !terminal)
                && self.units.get(&p.caster).is_some_and(|u| u.is_active())
        });
        Ok(())
    }
    pub(super) fn clear_tile_bursts(&mut self, now: u64) -> Result<(), GameError> {
        let state = std::mem::take(&mut self.sustained.tile_bursts);
        for r in state.released.into_values() {
            if let Some(ticket) = r.snapshot.fact_source.delivery {
                self.runtime_facts.close_delivery(&ticket, now)?;
            }
        }
        self.event_queue.discard_tile_bursts()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;

impl BattleCore {
    pub fn tile_bursts_snapshot(
        &self,
    ) -> Result<crate::game::ability::tile_burst::TileBurstsSnapshot, GameError> {
        use crate::game::ability::tile_burst::{
            ReleasedTileBurstSnapshot, TileBurstCenterSnapshot, TileBurstsSnapshot,
        };
        control::charge(
            self.sustained.tile_bursts.entity_count() as u64,
            "tile_bursts_snapshot",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.tile_bursts.retained_extent(),
            "tile_bursts_snapshot_storage",
        )?;
        let state = &self.sustained.tile_bursts;
        let released = state
            .released
            .iter()
            .map(|(id, r)| {
                Ok(ReleasedTileBurstSnapshot {
                    id: *id,
                    caster: r.snapshot.source_id,
                    action: r
                        .snapshot
                        .fact_source
                        .action
                        .as_ref()
                        .ok_or(GameError::InvalidAction)?
                        .ordinal()
                        .0,
                    round: r.round,
                    next_battle_time_ms: r.next_at,
                    presentation_remaining_ms: state
                        .freeze
                        .filter(|f| f.id == *id)
                        .map(|f| f.remaining_ms),
                    orb_relic_copy: r.plan.orb_relic_copy,
                    centers: r
                        .plan
                        .centers
                        .iter()
                        .map(|c| TileBurstCenterSnapshot {
                            tile: c.tile,
                            remaining_rounds: c.rounds.saturating_sub(r.round),
                            damage: c.damage,
                        })
                        .collect(),
                    applied_modifiers: r.plan.applied.clone(),
                })
            })
            .collect::<Result<Vec<_>, GameError>>()?;
        Ok(TileBurstsSnapshot {
            pending_casts: state.pending.len(),
            released,
        })
    }
}
