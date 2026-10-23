//! Independent waiting/leaping effects. All same-time deaths seed before detection.
mod damage;
use super::{types::SkillStepResult, BattleCore};
use crate::{
    authoring::{clone_shape::controlled_clone, control},
    game::{
        ability::{
            mobile_effect::*,
            sustained::{HitModifierSource, HitStatusKind, HitStatusProfile},
            SkillDef, SkillEffectDef, SkillHitTargetFilter,
        },
        battle::{
            damage::{DamageModifiers, DamageSource, DamageSourceSnapshot, DamageType},
            enums::BattleEvent,
            event_log::SkillCastTarget,
            ids::UnitInstanceId,
            runtime_facts::RuntimeFactSource,
            tile_range::FacingDirection,
        },
        behavior::GameError,
        enums::Side,
        resources::Position,
    },
};
pub(super) use damage::MobileDamageInput;
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq, Eq)]
struct FusionContext {
    owner: UnitInstanceId,
    copy: Uuid,
    side: Side,
    program: Arc<FusionPlacementDef>,
    template: DamageSourceSnapshot,
    source: RuntimeFactSource,
    seed_ms: Option<u32>,
    relic_copies: BTreeMap<String, Uuid>,
}
crate::authoring::clone_shape::struct_shape!(FusionContext {
    owner,
    copy,
    side,
    program,
    template,
    source,
    seed_ms,
    relic_copies
});
impl FusionContext {
    fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>()
            .checked_add(self.program.retained_extent()?)?
            .checked_add(self.template.fact_retained_extent()?)?
            .checked_add(self.source.retained_extent()?)?;
        for s in self.relic_copies.keys() {
            n = n.checked_add(96 + s.capacity())?;
        }
        Some(n)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MobileCapture {
    context: Arc<FusionContext>,
    activation: u64,
}
crate::authoring::clone_shape::struct_shape!(MobileCapture {
    context,
    activation
});
impl MobileCapture {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>().checked_add(self.context.retained_extent()?)
    }
}
#[derive(Debug, Clone)]
struct Mobile {
    seed_origin: Option<Arc<MobileSeedOrigin>>,
    id: u64,
    context: Arc<FusionContext>,
    tile: Position,
    units: u32,
    created: u64,
    expires: u64,
    /// None while waiting; landing and explosion are locked at launch.
    flight: Option<Flight>,
}
#[derive(Debug, Clone)]
struct Flight {
    destination: Position,
    started: u64,
    arrives: u64,
    explodes: u64,
    tier: usize,
}
crate::authoring::clone_shape::struct_shape!(Flight {
    destination,
    started,
    arrives,
    explodes,
    tier
});
crate::authoring::clone_shape::struct_shape!(Mobile {
    seed_origin,
    id,
    context,
    tile,
    units,
    created,
    expires,
    flight
});
#[derive(Debug, Clone)]
struct Seed {
    origin: Arc<MobileSeedOrigin>,
    context: Arc<FusionContext>,
    target: UnitInstanceId,
    expires: u64,
}
crate::authoring::clone_shape::struct_shape!(Seed {
    origin,
    context,
    target,
    expires
});
#[derive(Debug, Clone)]
struct PendingSeed {
    origin: Arc<MobileSeedOrigin>,
    context: Arc<FusionContext>,
    tile: Position,
}
crate::authoring::clone_shape::struct_shape!(PendingSeed {
    origin,
    context,
    tile
});
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SeedKey {
    owner: UnitInstanceId,
    copy: Uuid,
    definition: String,
    target: UnitInstanceId,
}
crate::authoring::clone_shape::struct_shape!(SeedKey {
    owner,
    copy,
    definition,
    target
});
#[derive(Debug, Clone, Default)]
pub(super) struct MobileState {
    next: u64,
    next_tick: Option<u64>,
    effects: BTreeMap<u64, Mobile>,
    seeds: BTreeMap<SeedKey, Seed>,
    pending: Vec<PendingSeed>,
}
crate::authoring::clone_shape::struct_shape!(MobileState {
    next,
    next_tick,
    effects,
    seeds,
    pending
});
impl MobileState {
    pub(super) fn entity_count(&self) -> usize {
        self.effects.len() + self.seeds.len() + self.pending.len()
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n = 0usize;
        for p in self.effects.values() {
            n = n
                .checked_add(std::mem::size_of::<Mobile>() + 80)?
                .checked_add(p.context.retained_extent()?)?
                .checked_add(
                    p.seed_origin
                        .as_ref()
                        .map_or(Some(0), |s| s.retained_extent())?,
                )?;
        }
        for (k, s) in &self.seeds {
            n = n
                .checked_add(std::mem::size_of::<Seed>() + 128 + k.definition.capacity())?
                .checked_add(s.context.retained_extent()?)?
                .checked_add(s.origin.retained_extent()?)?;
        }
        n = n.checked_add(
            self.pending
                .capacity()
                .checked_mul(std::mem::size_of::<PendingSeed>())?,
        )?;
        for p in &self.pending {
            n = n
                .checked_add(p.context.retained_extent()?)?
                .checked_add(p.origin.retained_extent()?)?;
        }
        Some(n)
    }
}
fn tier(p: &FusionPlacementDef, units: u32) -> usize {
    p.tiers
        .iter()
        .rposition(|t| t.minimum <= units)
        .unwrap_or(0)
}
impl BattleCore {
    pub(super) fn allocate_mobile_id(&mut self) -> Result<u64, GameError> {
        let n = self
            .sustained
            .mobile
            .next
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        self.sustained.mobile.next = n;
        Ok(n)
    }
    pub(super) fn fusion_placement_skill(skill: &SkillDef) -> bool {
        skill
            .steps
            .iter()
            .flat_map(|s| &s.effects)
            .any(|e| matches!(e.op, SkillEffectDef::FusionPlacement { .. }))
    }
    fn fusion_tile_valid(&self, tile: Position, now: u64) -> Result<bool, GameError> {
        let domain = self
            .scenario
            .battlefield
            .object_placement
            .as_ref()
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "Fusion placement requires an authored ground placement domain".into(),
                )
            })?;
        control::charge(
            (domain.ground.len() + domain.reserved.len() + self.units.len()) as u64,
            "fusion_placement_query",
        )?;
        if !domain.ground.contains(&tile)
            || domain.reserved.contains(&tile)
            || !self.battlefield.is_walkable_tile(tile)
        {
            return Ok(false);
        }
        Ok(!self.units.values().any(|u| {
            u.is_active()
                && self
                    .sample_unit_body_at(u.instance_id, now)
                    .is_some_and(|b| b.position.project_to_tile() == tile)
        }))
    }
    fn fusion_layout(
        &self,
        owner: UnitInstanceId,
        copy: Uuid,
        p: &FusionPlacementDef,
        tile: Position,
        now: u64,
    ) -> Result<Vec<(Position, u32)>, GameError> {
        if !self.fusion_tile_valid(tile, now)? {
            return Err(GameError::InvalidAction);
        }
        let mut adjacent = None;
        for (dx, dy) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let pos = Position::new(tile.x + dx, tile.y + dy);
            if self.fusion_tile_valid(pos, now)? {
                adjacent = Some(pos);
                break;
            }
        }
        let layout = if let Some(pos) = adjacent {
            vec![(tile, p.primary), (pos, p.adjacent)]
        } else {
            vec![(
                tile,
                p.primary
                    .checked_add(p.adjacent)
                    .ok_or(GameError::InvalidAction)?,
            )]
        };
        let existing: Vec<_> = self
            .sustained
            .mobile
            .effects
            .values()
            .filter(|e| {
                e.flight.is_none()
                    && e.expires > now
                    && e.context.owner == owner
                    && e.context.copy == copy
                    && e.context.program.definition == p.definition
            })
            .map(|e| e.tile)
            .collect();
        let added = layout
            .iter()
            .filter(|(pos, _)| !existing.contains(pos))
            .count();
        if existing
            .len()
            .checked_add(added)
            .is_none_or(|n| n > p.max_tiles as usize)
        {
            return Err(GameError::InvalidAction);
        }
        Ok(layout)
    }
    pub(super) fn validate_fusion_start(
        &self,
        owner: UnitInstanceId,
        skill: &SkillDef,
        target: Option<SkillCastTarget>,
        now: u64,
    ) -> Result<(), GameError> {
        for effect in skill.steps.iter().flat_map(|s| &s.effects) {
            if let SkillEffectDef::FusionPlacement { program } = &effect.op {
                let Some(SkillCastTarget::Tile { position: tile }) = target else {
                    return Err(GameError::InvalidAction);
                };
                let copy = self
                    .units
                    .get(&owner)
                    .and_then(|u| u.fragment_source.as_ref())
                    .ok_or(GameError::InvalidAction)?
                    .copy_uuid;
                self.fusion_layout(owner, copy, program, tile, now)?;
            }
        }
        Ok(())
    }
    fn fusion_context(
        &mut self,
        owner: UnitInstanceId,
        p: &FusionPlacementDef,
        now: u64,
    ) -> Result<Arc<FusionContext>, GameError> {
        let u = self
            .units
            .get(&owner)
            .filter(|u| u.is_active())
            .ok_or(GameError::InvalidAction)?;
        let side = u.owner;
        let copy = u
            .fragment_source
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .copy_uuid;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            p.retained_extent(),
            "fusion_context",
        )?;
        let mut p = controlled_clone(p)?;
        let mut relic_copies = BTreeMap::new();
        for t in &mut p.tiers {
            t.profile.modifiers.retain(|m| {
                if let Some(copy) = self.equipped_normal_relic_copy(owner, &m.relic_definition) {
                    relic_copies.insert(m.id.clone(), copy);
                    true
                } else {
                    false
                }
            });
            for m in &t.profile.modifiers {
                if let MobileChange::Range { pattern } = &m.change {
                    t.profile.range = controlled_clone(pattern)?;
                }
            }
        }
        let seed_ms = if let Some(s) = &p.seed {
            if let Some(copy) = self.equipped_normal_relic_copy(owner, &s.relic_definition) {
                relic_copies.insert(s.definition.clone(), copy);
                let changes = s
                    .modifiers
                    .iter()
                    .filter(|m| {
                        self.equipped_normal_relic_copy(owner, &m.relic_definition)
                            .is_some()
                    })
                    .map(|m| (m.negative, m.percent))
                    .collect::<Vec<_>>();
                Some(super::marks::modified_period(s.duration_ms, &changes)?)
            } else {
                None
            }
        } else {
            None
        };
        let source = self.runtime_fact_source.independent_origin()?;
        source.admit_owner("fusion_source")?;
        let mut template = self
            .damage_source_snapshot_template_for_unit(
                owner,
                DamageSource::Ability,
                DamageType::Magic,
                0,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        template.fact_source = controlled_clone(&source)?;
        template.admit_fact_owner()?;
        Ok(Arc::new(FusionContext {
            owner,
            copy,
            side,
            program: Arc::new(p),
            template,
            source,
            seed_ms,
            relic_copies,
        }))
    }
    fn schedule_mobile_tick(&mut self, at: u64) -> Result<(), GameError> {
        if self.sustained.mobile.next_tick.is_some_and(|old| old <= at) {
            return Ok(());
        }
        self.event_queue.discard_mobile_tick()?;
        self.sustained.mobile.next_tick = Some(at);
        self.event_queue
            .push(BattleEvent::MobileEffectsTick { time_ms: at });
        control::poll("mobile_tick_enqueue")
    }
    pub(super) fn install_fusion(
        &mut self,
        owner: UnitInstanceId,
        p: &FusionPlacementDef,
        now: u64,
    ) -> Result<SkillStepResult, GameError> {
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
        let Some(SkillCastTarget::Tile { position: tile }) = target else {
            return Err(GameError::InvalidAction);
        };
        let context = self.fusion_context(owner, p, now)?;
        let layout = self.fusion_layout(owner, context.copy, p, tile, now)?;
        for (pos, units) in layout {
            if !self.seed_fusion(context.clone(), None, pos, units, now)? {
                return Err(GameError::InvalidAction);
            }
        }
        self.schedule_mobile_tick(now)?;
        Ok(SkillStepResult {
            applied_effect_count: 1,
            ..Default::default()
        })
    }
    fn seed_fusion(
        &mut self,
        context: Arc<FusionContext>,
        seed_origin: Option<Arc<MobileSeedOrigin>>,
        tile: Position,
        units: u32,
        now: u64,
    ) -> Result<bool, GameError> {
        control::charge(
            self.sustained.mobile.effects.len() as u64,
            "fusion_merge_scan",
        )?;
        let matches = |e: &&Mobile| {
            e.context.owner == context.owner
                && e.context.copy == context.copy
                && e.context.program.definition == context.program.definition
                && e.flight.is_none()
                && e.expires > now
        };
        let old = self
            .sustained
            .mobile
            .effects
            .values()
            .filter(matches)
            .find(|e| e.tile == tile)
            .map(|e| e.id);
        if let Some(id) = old {
            let e = self.sustained.mobile.effects.get_mut(&id).unwrap();
            e.units = e
                .units
                .saturating_add(units)
                .min(e.context.program.max_units);
            return Ok(true);
        }
        if self
            .sustained
            .mobile
            .effects
            .values()
            .filter(matches)
            .count()
            >= context.program.max_tiles as usize
        {
            return Ok(false);
        }
        // Death seeds may land on the dead victim's tile; other active units still block it.
        if !self.fusion_tile_valid(tile, now)? {
            return Ok(false);
        }
        let units = units.min(context.program.max_units);
        let t = tier(&context.program, units);
        let lifetime = context.program.tiers[t]
            .profile
            .lifetime_ms
            .ok_or(GameError::InvalidAction)?;
        let expires = now
            .checked_add(u64::from(lifetime))
            .ok_or(GameError::InvalidAction)?;
        self.admit_authoring_runtime_growth(1, "fusion_effect")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            context
                .retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<Mobile>() + 80))
                .and_then(|n| {
                    n.checked_add(
                        seed_origin
                            .as_ref()
                            .map_or(Some(0), |s| s.retained_extent())?,
                    )
                }),
            "fusion_effect",
        )?;
        let id = self
            .sustained
            .mobile
            .next
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        self.sustained.mobile.next = id;
        self.sustained.mobile.effects.insert(
            id,
            Mobile {
                seed_origin,
                id,
                context,
                tile,
                units,
                created: now,
                expires,
                flight: None,
            },
        );
        Ok(true)
    }
    pub(super) fn mobile_accepted_hit(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(capture) = &snapshot.mobile_capture else {
            return Ok(());
        };
        let context = &capture.context;
        if snapshot.fact_source.reaction || snapshot.source_id != context.owner {
            return Ok(());
        }
        let Some(duration) = context.seed_ms else {
            return Ok(());
        };
        let seed = context
            .program
            .seed
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let origin = Arc::new(MobileSeedOrigin {
            parent_activation: capture.activation,
            relic: HitModifierSource {
                copy_uuid: *context
                    .relic_copies
                    .get(&seed.definition)
                    .ok_or(GameError::InvalidAction)?,
                definition: seed.relic_definition.clone(),
                rule: seed.definition.clone(),
            },
        });
        let key = SeedKey {
            owner: context.owner,
            copy: context.copy,
            definition: seed.definition.clone(),
            target,
        };
        let expires = now
            .checked_add(u64::from(duration))
            .ok_or(GameError::InvalidAction)?;
        if !self.sustained.mobile.seeds.contains_key(&key) {
            self.admit_authoring_runtime_growth(1, "fusion_seed")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                context.retained_extent().and_then(|n| {
                    n.checked_add(std::mem::size_of::<Seed>() + 128 + key.definition.capacity())?
                        .checked_add(origin.retained_extent()?)
                }),
                "fusion_seed",
            )?;
        }
        self.sustained.mobile.seeds.insert(
            key,
            Seed {
                origin,
                context: context.clone(),
                target,
                expires,
            },
        );
        self.schedule_mobile_tick(expires)?;
        Ok(())
    }
    pub(super) fn mobile_target_died(
        &mut self,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        control::charge(
            self.sustained.mobile.seeds.len() as u64,
            "fusion_death_seeds",
        )?;
        let keys: Vec<_> = self
            .sustained
            .mobile
            .seeds
            .iter()
            .filter(|(_, s)| s.target == target)
            .map(|(k, _)| k.clone())
            .collect();
        let tile = self
            .sample_unit_body_at(target, now)
            .map(|b| b.position.project_to_tile())
            .or_else(|| self.units.get(&target).map(|u| u.body.projected_tile()));
        for key in keys {
            let s = self.sustained.mobile.seeds.remove(&key).unwrap();
            if s.expires <= now {
                continue;
            }
            let Some(tile) = tile else {
                continue;
            };
            self.admit_authoring_runtime_growth(1, "fusion_pending_seed")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                s.context
                    .retained_extent()
                    .and_then(|n| n.checked_add(std::mem::size_of::<PendingSeed>()))
                    .and_then(|n| n.checked_add(s.origin.retained_extent()?)),
                "fusion_pending_seed",
            )?;
            self.sustained.mobile.pending.push(PendingSeed {
                origin: s.origin,
                context: s.context,
                tile,
            });
        }
        if !self.sustained.mobile.pending.is_empty() {
            self.schedule_mobile_tick(now)?;
        }
        Ok(())
    }
    fn explode_mobile(&mut self, e: Mobile, now: u64) -> Result<(), GameError> {
        let f = e.flight.as_ref().ok_or(GameError::InvalidAction)?;
        let p = &e.context.program.tiers[f.tier].profile;
        let tiles = self
            .valid_tile_area_tiles(&p.range, f.destination, FacingDirection::Up)
            .ok_or(GameError::InvalidAction)?;
        let targets = self.collect_tile_area_targets_at(
            now,
            e.context.side,
            e.context.owner,
            &tiles,
            SkillHitTargetFilter::Enemies,
            false,
        );
        control::poll("mobile_explosion_targets")?;
        let capture = Arc::new(MobileCapture {
            context: e.context.clone(),
            activation: e.id,
        });
        for target in targets {
            self.execute_mobile_hit(MobileDamageInput {
                profile: p,
                units: e.units,
                template: &e.context.template,
                source: &e.context.source,
                owner: e.context.owner,
                side: e.context.side,
                relic_copies: &e.context.relic_copies,
                origin: crate::game::battle::runtime_facts::MobileEffectOrigin {
                    motion: MobileMotion::WaitLeap,
                    death_origin: None,
                    seed_origin: e.seed_origin.clone(),
                    activation: e.id,
                    owner: e.context.owner,
                    copy: e.context.copy,
                    definition: p.definition.clone(),
                },
                capture: Some(capture.clone()),
                target,
                now,
            })?;
        }
        Ok(())
    }
    pub(super) fn mobile_effects_tick(&mut self, now: u64) -> Result<(), GameError> {
        if self.sustained.mobile.next_tick != Some(now) {
            return Ok(());
        }
        self.sustained.mobile.next_tick = None;
        control::charge(self.sustained.mobile.entity_count() as u64, "mobile_tick")?;
        self.sustained.mobile.seeds.retain(|_, s| s.expires > now);
        self.sustained
            .mobile
            .effects
            .retain(|_, e| e.flight.is_some() || e.expires > now);
        let due: Vec<_> = self
            .sustained
            .mobile
            .effects
            .iter()
            .filter(|(_, e)| e.flight.as_ref().is_some_and(|f| f.explodes <= now))
            .map(|(id, _)| *id)
            .collect();
        for id in due {
            let e = self.sustained.mobile.effects.remove(&id).unwrap();
            self.explode_mobile(e, now)?;
        }
        // Every death in the due batch has completed before any new effect searches.
        let pending = std::mem::take(&mut self.sustained.mobile.pending);
        for s in pending {
            self.seed_fusion(s.context, Some(s.origin), s.tile, 1, now)?;
        }
        let waiting: Vec<_> = self
            .sustained
            .mobile
            .effects
            .iter()
            .filter(|(_, e)| e.flight.is_none())
            .map(|(id, _)| *id)
            .collect();
        for id in waiting {
            let e = controlled_clone(&self.sustained.mobile.effects[&id])?;
            let t = tier(&e.context.program, e.units);
            let p = &e.context.program.tiers[t].profile;
            let tiles = self
                .valid_tile_area_tiles(&p.detection, e.tile, FacingDirection::Up)
                .ok_or(GameError::InvalidAction)?;
            let mut targets = self.collect_tile_area_targets_at(
                now,
                e.context.side,
                e.context.owner,
                &tiles,
                SkillHitTargetFilter::Enemies,
                false,
            );
            targets.sort_by_key(|target| {
                let tile = self
                    .sample_unit_body_at(*target, now)
                    .map(|b| b.position.project_to_tile())
                    .unwrap_or(e.tile);
                (e.tile.manhattan(&tile), *target)
            });
            control::poll("mobile_detect")?;
            if let Some(target) = targets.first() {
                let destination = self
                    .sample_unit_body_at(*target, now)
                    .ok_or(GameError::InvalidAction)?
                    .position
                    .project_to_tile();
                let dx = f64::from(destination.x - e.tile.x);
                let dy = f64::from(destination.y - e.tile.y);
                let travel = ((dx * dx + dy * dy).sqrt() * 1_000_000.0
                    / f64::from(p.speed_units_per_second))
                .ceil();
                if !travel.is_finite() || travel > u64::MAX as f64 {
                    return Err(GameError::InvalidAction);
                }
                let arrives = now
                    .checked_add(travel as u64)
                    .ok_or(GameError::InvalidAction)?;
                let explodes = arrives
                    .checked_add(u64::from(p.delay_ms))
                    .ok_or(GameError::InvalidAction)?;
                self.sustained.mobile.effects.get_mut(&id).unwrap().flight = Some(Flight {
                    destination,
                    started: now,
                    arrives,
                    explodes,
                    tier: t,
                });
            }
        }
        let mut next = self
            .sustained
            .mobile
            .seeds
            .values()
            .map(|s| s.expires)
            .min();
        for e in self.sustained.mobile.effects.values() {
            let at = if let Some(f) = &e.flight {
                f.explodes
            } else {
                now.checked_add(self.movement_tick_ms())
                    .ok_or(GameError::InvalidAction)?
                    .min(e.expires)
            };
            next = Some(next.map_or(at, |n| n.min(at)));
        }
        if let Some(next) = next {
            self.schedule_mobile_tick(next)?;
        }
        Ok(())
    }
    pub(super) fn clear_mobile_effects(&mut self) -> Result<(), GameError> {
        self.sustained.mobile.effects.clear();
        self.sustained.mobile.seeds.clear();
        self.sustained.mobile.pending = Vec::new();
        self.sustained.mobile.next_tick = None;
        self.event_queue.discard_mobile_tick()
    }
}

#[cfg(test)]
mod tests;

impl BattleCore {
    pub fn mobile_effects_snapshot(&self) -> Result<MobileEffectsSnapshot, GameError> {
        control::charge(
            self.sustained.mobile.entity_count() as u64,
            "mobile_snapshot",
        )?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.mobile.retained_extent(),
            "mobile_snapshot",
        )?;
        let mut effects: Vec<_> = self
            .sustained
            .mobile
            .effects
            .values()
            .map(|e| {
                let t = e
                    .flight
                    .as_ref()
                    .map_or_else(|| tier(&e.context.program, e.units), |f| f.tier);
                MobileEffectView {
                    death_origin: None,
                    seed_origin: e.seed_origin.clone(),
                    activation: e.id,
                    owner: e.context.owner,
                    copy: e.context.copy,
                    family: e.context.program.definition.clone(),
                    profile: e.context.program.tiers[t].profile.definition.clone(),
                    units: e.units,
                    tile: e.tile,
                    created_at_ms: e.created,
                    waiting_expires_at_ms: e.expires,
                    phase: e.flight.as_ref().map_or(MobilePhaseView::Waiting, |f| {
                        MobilePhaseView::Leaping {
                            destination: f.destination,
                            started_at_ms: f.started,
                            arrives_at_ms: f.arrives,
                            explodes_at_ms: f.explodes,
                        }
                    }),
                }
            })
            .collect();
        let seeds = self
            .sustained
            .mobile
            .seeds
            .iter()
            .map(|(k, s)| MobileSeedView {
                origin: s.origin.clone(),
                owner: k.owner,
                copy: k.copy,
                target: k.target,
                definition: k.definition.clone(),
                expires_at_ms: s.expires,
            })
            .collect();
        effects.extend(self.status_mobile_views()?);
        Ok(MobileEffectsSnapshot { effects, seeds })
    }
}
