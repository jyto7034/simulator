use super::*;
use crate::game::battle::{
    core::movement::types::WorldVec2, enums::BattleEvent, runtime_facts::MobileEffectOrigin,
    tile_range::FacingDirection,
};
use uuid::Uuid;
#[derive(Debug, Clone)]
struct Chase {
    id: u64,
    owner: UnitInstanceId,
    copy: Uuid,
    side: Side,
    family: String,
    profile: Arc<MobileProfile>,
    template: DamageSourceSnapshot,
    source: RuntimeFactSource,
    relic_copies: BTreeMap<String, Uuid>,
    death: Option<Arc<MobileDeathOrigin>>,
    position: WorldVec2,
    target: Option<UnitInstanceId>,
    created: u64,
    last: u64,
    expires: u64,
}
crate::authoring::clone_shape::struct_shape!(Chase {
    id,
    owner,
    copy,
    side,
    family,
    profile,
    template,
    source,
    relic_copies,
    death,
    position,
    target,
    created,
    last,
    expires
});
impl Chase {
    fn retained_extent(&self) -> Option<usize> {
        let mut n = std::mem::size_of::<Self>()
            .checked_add(80)?
            .checked_add(self.family.capacity())?
            .checked_add(self.profile.retained_extent()?)?
            .checked_add(self.template.fact_retained_extent()?)?
            .checked_add(self.source.retained_extent()?)?
            .checked_add(
                self.death
                    .as_ref()
                    .map_or(Some(0), |d| d.retained_extent())?,
            )?;
        for k in self.relic_copies.keys() {
            n = n.checked_add(96 + k.capacity())?;
        }
        Some(n)
    }
}
#[derive(Debug, Clone, Default)]
pub(in crate::game::battle::core) struct ChaseState {
    next_tick: Option<u64>,
    effects: BTreeMap<u64, Chase>,
}
crate::authoring::clone_shape::struct_shape!(ChaseState { next_tick, effects });
impl ChaseState {
    pub(in crate::game::battle::core) fn entity_count(&self) -> usize {
        self.effects.len()
    }
    pub(in crate::game::battle::core) fn retained_extent(&self) -> Option<usize> {
        self.effects
            .values()
            .try_fold(0usize, |n, e| n.checked_add(e.retained_extent()?))
    }
}
impl BattleCore {
    fn schedule_status_mobile(&mut self, at: u64) -> Result<(), GameError> {
        if self.sustained.chases.next_tick.is_some_and(|old| old <= at) {
            return Ok(());
        }
        self.event_queue.discard_status_mobile_tick()?;
        self.sustained.chases.next_tick = Some(at);
        self.event_queue
            .push(BattleEvent::StatusMobileTick { time_ms: at });
        control::poll("status_mobile_enqueue")
    }
    pub(super) fn spawn_status_mobile(
        &mut self,
        c: &StatusContext,
        position: WorldVec2,
        death: Option<Arc<MobileDeathOrigin>>,
        now: u64,
    ) -> Result<bool, GameError> {
        control::charge(self.sustained.chases.effects.len() as u64, "chase_limit")?;
        if self
            .sustained
            .chases
            .effects
            .values()
            .filter(|e| e.owner == c.owner && e.family == c.program.definition)
            .count()
            >= c.program.limit as usize
        {
            return Ok(false);
        }
        let mut profile = controlled_clone(&c.program.mobile)?;
        let mut relic_copies = BTreeMap::new();
        profile.modifiers.retain(|m| {
            if let Some(copy) = self.equipped_normal_relic_copy(c.owner, &m.relic_definition) {
                relic_copies.insert(m.id.clone(), copy);
                true
            } else {
                false
            }
        });
        for m in &profile.modifiers {
            if let MobileChange::Range { pattern } = &m.change {
                profile.range = controlled_clone(pattern)?;
            }
        }
        let expires = now
            .checked_add(u64::from(profile.delay_ms))
            .ok_or(GameError::InvalidAction)?;
        let mut template = self
            .damage_source_snapshot_template_for_unit(
                c.owner,
                DamageSource::Ability,
                DamageType::Magic,
                0,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        template.fact_source = controlled_clone(&c.source)?;
        template.reactive_capture = None;
        template.admit_fact_owner()?;
        let e = Chase {
            id: 0,
            owner: c.owner,
            copy: c.fragment.copy_uuid,
            side: c.side,
            family: c.program.definition.clone(),
            profile: Arc::new(profile),
            template,
            source: controlled_clone(&c.source)?,
            relic_copies,
            death,
            position,
            target: None,
            created: now,
            last: now,
            expires,
        };
        self.admit_authoring_runtime_growth(1, "chase_spawn")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            e.retained_extent(),
            "chase_spawn",
        )?;
        let id = self.allocate_mobile_id()?;
        self.sustained.chases.effects.insert(id, Chase { id, ..e });
        self.schedule_status_mobile(now)?;
        Ok(true)
    }
    pub(in crate::game::battle::core) fn status_mobile_tick(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        if self.sustained.chases.next_tick != Some(now) {
            return Ok(());
        }
        self.sustained.chases.next_tick = None;
        control::charge(self.sustained.chases.effects.len() as u64, "chase_tick")?;
        let ids: Vec<_> = self.sustained.chases.effects.keys().copied().collect();
        for id in ids {
            let mut e = self
                .sustained
                .chases
                .effects
                .remove(&id)
                .ok_or(GameError::InvalidAction)?;
            let tiles = self
                .valid_tile_area_tiles(
                    &e.profile.detection,
                    e.position.project_to_tile(),
                    FacingDirection::Up,
                )
                .ok_or(GameError::InvalidAction)?;
            let mut targets = self.collect_tile_area_targets_at(
                now,
                e.side,
                e.owner,
                &tiles,
                SkillHitTargetFilter::Enemies,
                false,
            );
            targets.sort_by(|a, b| {
                let distance = |id| {
                    self.sample_unit_body_at(id, now)
                        .map(|b| e.position.distance_squared(b.position))
                        .unwrap_or(f32::INFINITY)
                };
                distance(*a).total_cmp(&distance(*b)).then(a.cmp(b))
            });
            e.target = targets.first().copied();
            if let Some(target) = e.target {
                let pos = self
                    .sample_unit_body_at(target, now)
                    .ok_or(GameError::InvalidAction)?
                    .position;
                let delta = pos - e.position;
                let movement = (e.profile.speed_units_per_second as f64 * (now - e.last) as f64
                    / 1_000_000.0) as f32;
                e.position = e.position + delta.clamp_length(movement);
            }
            e.last = now;
            control::poll("chase_move")?;
            if e.expires <= now {
                let tiles = self
                    .valid_tile_area_tiles(
                        &e.profile.range,
                        e.position.project_to_tile(),
                        FacingDirection::Up,
                    )
                    .ok_or(GameError::InvalidAction)?;
                let targets = self.collect_tile_area_targets_at(
                    now,
                    e.side,
                    e.owner,
                    &tiles,
                    SkillHitTargetFilter::Enemies,
                    false,
                );
                let area_percent = self.area_damage_percent(e.owner, e.side,
                    &e.source, &targets, now)?;
                for target in targets {
                    self.execute_mobile_hit(super::super::mobile_effect::MobileDamageInput {
                        profile: &e.profile,
                        units: 1,
                        template: &e.template,
                        source: &e.source,
                        owner: e.owner,
                        side: e.side,
                        relic_copies: &e.relic_copies,
                        origin: MobileEffectOrigin {
                            activation: e.id,
                            owner: e.owner,
                            copy: e.copy,
                            definition: e.profile.definition.clone(),
                            motion: MobileMotion::ChaseDelay,
                            seed_origin: None,
                            death_origin: e.death.clone(),
                        },
                        capture: None,
                        area_percent,
                        target,
                        now,
                    })?;
                }
            } else {
                self.sustained.chases.effects.insert(id, e);
            }
        }
        if let Some(expiry) = self
            .sustained
            .chases
            .effects
            .values()
            .map(|e| e.expires)
            .min()
        {
            self.schedule_status_mobile(
                now.checked_add(self.movement_tick_ms())
                    .ok_or(GameError::InvalidAction)?
                    .min(expiry),
            )?;
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn clear_status_mobile(&mut self) -> Result<(), GameError> {
        self.sustained.chases.effects.clear();
        self.sustained.chases.next_tick = None;
        self.event_queue.discard_status_mobile_tick()
    }
    pub(in crate::game::battle::core) fn status_mobile_views(
        &self,
    ) -> Result<Vec<MobileEffectView>, GameError> {
        control::charge(self.sustained.chases.effects.len() as u64, "chase_snapshot")?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            self.sustained.chases.retained_extent(),
            "chase_snapshot",
        )?;
        Ok(self
            .sustained
            .chases
            .effects
            .values()
            .map(|e| {
                let point = e.position.quantized_milli();
                let (x_milli, y_milli) = (i64::from(point.x_milli), i64::from(point.y_milli));
                MobileEffectView {
                    activation: e.id,
                    owner: e.owner,
                    copy: e.copy,
                    family: e.family.clone(),
                    profile: e.profile.definition.clone(),
                    units: 1,
                    tile: e.position.project_to_tile(),
                    created_at_ms: e.created,
                    waiting_expires_at_ms: e.expires,
                    seed_origin: None,
                    death_origin: e.death.clone(),
                    phase: MobilePhaseView::Chasing {
                        x_milli,
                        y_milli,
                        target: e.target,
                        explodes_at_ms: e.expires,
                    },
                }
            })
            .collect())
    }
}
