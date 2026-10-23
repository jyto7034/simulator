//! Finite periodic opportunities own independent, fixed-tile impacts. Ending an
//! emitter retires only its own continuations; released child areas retain their
//! original action and delivery ownership.
use super::*;
use crate::game::behavior::GameError;

impl AreaRuntime {
    pub(in crate::game::battle::core) fn emission_extent(&self) -> Option<usize> {
        self.emission
            .as_ref()
            .map_or(Some(0), BattleCore::emission_extent)
    }
}
impl BattleCore {
    fn emission_extent(emission: &crate::game::ability::SkillAreaEmissionDef) -> Option<usize> {
        control::charge(emission.footprint.rows.len() as u64, "emission_extent_scan").ok()?;
        emission.footprint.rows.iter().try_fold(
            emission
                .footprint
                .rows
                .capacity()
                .checked_mul(std::mem::size_of::<String>())?,
            |sum, row| sum.checked_add(row.capacity()),
        )
    }
    pub(super) fn admit_emission_copy(
        emission: &crate::game::ability::SkillAreaEmissionDef,
    ) -> Result<(), GameError> {
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Self::emission_extent(emission),
            "area_emission_copy",
        )
    }

    pub(in crate::game::battle::core) fn clear_skill_areas(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        loop {
            control::charge(self.active_areas.len() as u64, "terminal_area_scan")?;
            let Some(id) = self.active_areas.keys().min().copied() else {
                return Ok(());
            };
            self.stop_skill_area_with_resume(id, now, false)?;
        }
    }

    pub(in crate::game::battle::core) fn reconcile_area_emitters(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        loop {
            control::charge(self.active_areas.len() as u64, "emitter_source_scan")?;
            let ended = self
                .active_areas
                .values()
                .filter(|r| {
                    (r.emission.is_some() || matches!(r.mode, crate::game::ability::SkillTileAreaMode::ChannelGround { .. }))
                        && !self
                            .units
                            .get(&r.caster_instance_id)
                            .is_some_and(|u| u.is_active())
                })
                .map(|r| r.area_id)
                .min();
            let Some(id) = ended else {
                return Ok(());
            };
            self.stop_skill_area(id, now)?;
        }
    }

    pub(super) fn replace_area_emitter(
        &mut self,
        caster: UnitInstanceId,
        skill: &SkillId,
        step: &str,
        now: u64,
    ) -> Result<(), GameError> {
        loop {
            control::charge(self.active_areas.len() as u64, "emitter_replacement_scan")?;
            let source = &self.runtime_fact_source;
            let previous = self
                .active_areas
                .values()
                .filter(|r| {
                    r.emission.is_some()
                        && r.caster_instance_id == caster
                        && &r.skill_id == skill
                        && r.step_id == step
                        && r.source_snapshot
                            .fact_source
                            .fragment
                            .as_ref()
                            .map(|f| f.copy_uuid)
                            == source.fragment.as_ref().map(|f| f.copy_uuid)
                        && r.source_snapshot.fact_source.equipment == source.equipment
                })
                .map(|r| r.area_id)
                .min();
            let Some(id) = previous else {
                return Ok(());
            };
            self.stop_skill_area(id, now)?;
        }
    }

    pub(in crate::game::battle::core) fn stop_skill_area(&mut self, id: Uuid, now: u64) -> Result<(), GameError> {
        self.stop_skill_area_with_resume(id, now, true)
    }
    fn stop_skill_area_with_resume(&mut self,id:Uuid,now:u64,resume_basic:bool)->Result<(),GameError> {
        while let Some(queued) = self.event_queue.take_area_event(id)? {
            if queued.close_delivery_on_dispatch {
                let source = queued.fact_source.into_owned()?;
                self.close_source_resource_delivery(
                    source.resource_delivery().map(std::sync::Arc::as_ref),
                )?;
                if let Some(ticket) = source.delivery {
                    self.runtime_facts.close_delivery(&ticket, now)?;
                }
            }
        }
        self.finish_skill_area_at_with_resume(id, now, resume_basic)?;
        control::poll("area_emitter_stopped")
    }

    pub(super) fn apply_area_emission_tick(
        &mut self,
        runtime: &AreaRuntime,
        now: u64,
    ) -> Result<(), GameError> {
        let emission = runtime.emission.as_ref().ok_or(GameError::InvalidAction)?;
        if !self
            .units
            .get(&runtime.caster_instance_id)
            .is_some_and(|u| u.is_active())
        {
            return self.stop_skill_area(runtime.area_id, now);
        }
        let tiles = self
            .valid_tile_area_tiles(&runtime.tile_range, runtime.tile_anchor, runtime.facing)
            .ok_or(GameError::InvalidAction)?;
        let candidates = self.collect_tile_area_targets_at(
            now,
            runtime.caster_owner,
            runtime.caster_instance_id,
            &tiles,
            SkillHitTargetFilter::Enemies,
            false,
        );
        let mut best: Option<(f32, u64, UnitInstanceId)> = None;
        for id in candidates {
            control::poll("area_emitter_target")?;
            if !self.single_target_can_target_unit(id, emission.include_air) {
                continue;
            }
            let unit = &self.units[&id];
            let plan = unit
                .enemy_movement_plan
                .as_ref()
                .unwrap_or(&self.scenario.tactical_plan.enemy_plan);
            let crate::game::battle::scenario::EnemyMovementPlan::PathAlongCells { cells } = plan;
            control::charge(cells.len() as u64 * 2 + 1, "emitter_route_projection")?;
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                cells.len().checked_mul(std::mem::size_of::<WorldVec2>()),
                "emitter_route_projection",
            )?;
            let progress = self.enemy_route_progress(id);
            if best.is_none_or(|(p, spawn, old)| {
                progress
                    .total_cmp(&p)
                    .reverse()
                    .then_with(|| unit.spawn_order.cmp(&spawn))
                    .then_with(|| id.cmp(&old))
                    .is_lt()
            }) {
                best = Some((progress, unit.spawn_order, id));
            }
        }
        if let Some((_, _, target)) = best {
            let tile = self
                .sample_unit_world_position_at(target, now)
                .or_else(|| self.unit_world_position_or_tile_center(target))
                .ok_or(GameError::InvalidAction)?
                .project_to_tile();
            let skill = self
                .cloned_skill_for_source(&runtime.skill_id, &runtime.source_snapshot.fact_source)?
                .ok_or(GameError::InvalidAction)?;
            let step = Self::resolve_skill_step(&skill, runtime.step_index, &runtime.step_id)
                .ok_or(GameError::InvalidAction)?;
            let area = SkillTileAreaDeliveryDef {
                anchor: SkillAreaAnchorSource::CastTarget,
                tile_origin: SkillTileAreaOrigin::Anchor,
                hit_targets: SkillHitTargetFilter::Enemies,
                first_tick_delay_ms: emission.delay_ms,
                ..Default::default()
            };
            Self::charge_authoring_tile_pattern(&emission.footprint, 1)?;
            Self::admit_emission_copy(emission)?;
            if self.register_persistent_tile_area(
                now,
                runtime.cast_seq,
                runtime.step_index,
                runtime.caster_instance_id,
                runtime.skill_id.clone(),
                runtime.step_id.clone(),
                step,
                Some(SkillCastTarget::Tile { position: tile }),
                emission.footprint.clone(),
                area,
            )? {
                self.begin_skill_step_delivery(runtime.cast_seq, runtime.step_index, 1);
            }
        }
        // Empty opportunities are consumed exactly like successful releases.
        if let Some(interval) = runtime.tick_interval_ms {
            let next = now
                .checked_add(u64::from(interval))
                .ok_or(GameError::InvalidAction)?;
            if next < runtime.expires_at_ms {
                if let Some(active) = self.active_areas.get_mut(&runtime.area_id) {
                    active.next_tick_ms = Some(next);
                }
                self.enqueue_carrier_completion(
                    BattleEvent::SkillAreaTick {
                        time_ms: next,
                        area_id: runtime.area_id,
                        cast_seq: runtime.cast_seq,
                        step_index: runtime.step_index,
                        skill_id: runtime.skill_id.clone(),
                        step_id: runtime.step_id.clone(),
                        center: runtime.center,
                        cause: self.recording_cause().unwrap_or_default(),
                    },
                    None,
                    runtime.source_snapshot.fact_source.clone(),
                )?;
            }
        }
        control::poll("area_emission_complete")?;
        self.schedule_pending_autocasts(now);
        Ok(())
    }
}
