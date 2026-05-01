use uuid::Uuid;

use crate::game::{
    ability::{
        SkillAreaAnchorSource, SkillAreaDeliveryDef, SkillAreaShapeDef, SkillAreaTickPolicy,
        SkillHitTargetFilter, SkillId,
    },
    battle::{
        core::{
            movement::types::{TimelineVec2, WorldVec2},
            spatial::{legacy_units_to_world, AreaQueryShape, DEFAULT_UNIT_HITBOX_RADIUS_UNITS},
            types::{AreaRuntime, SkillImpactContext, SkillStepResult},
            BattleCore,
        },
        enums::BattleEvent,
        ids::UnitInstanceId,
        timeline::{SkillCastTarget, TimelineEvent, TimelineSkillAreaShape},
    },
    determinism,
    enums::Side,
};

const INSTANT_AREA_DISPLAY_DURATION_MS: u32 = 250;

fn timeline_point(position: WorldVec2) -> TimelineVec2 {
    position.quantized_milli()
}

impl BattleCore {
    pub(in crate::game::battle::core) fn allocate_skill_area_delivery_id(
        &mut self,
        cast_seq: u64,
        caster_instance_id: UnitInstanceId,
        time_ms: u64,
    ) -> Uuid {
        const SKILL_AREA_DELIVERY_NS: u64 = 0x534B_4152_4541_4456u64; // "SKAREADV"

        let delivery_seq = self.area_seq;
        self.area_seq = self.area_seq.wrapping_add(1);
        determinism::uuid_v4_from_seed(
            cast_seq
                ^ time_ms
                ^ u64::from_be_bytes(caster_instance_id.as_bytes()[..8].try_into().unwrap()),
            SKILL_AREA_DELIVERY_NS,
            delivery_seq,
        )
    }

    pub(in crate::game::battle::core) fn resolve_area_anchor_position(
        &self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        area: &SkillAreaDeliveryDef,
    ) -> Option<WorldVec2> {
        let (_, caster_tile_pos) =
            self.resolve_cast_origin_context(Some(cast_seq), caster_instance_id, false)?;
        let caster_position = self
            .sample_unit_world_position_at(caster_instance_id, time_ms)
            .unwrap_or_else(|| WorldVec2::from_tile_center(caster_tile_pos));
        let cast_target_anchor_position = self
            .stored_skill_cast_anchor_position(cast_seq)
            .map(WorldVec2::from_tile_center);

        match area.anchor {
            SkillAreaAnchorSource::CastTarget | SkillAreaAnchorSource::CastTargetStart => {
                match step_target {
                    Some(SkillCastTarget::Tile { position }) => {
                        Some(WorldVec2::from_tile_center(position))
                    }
                    Some(SkillCastTarget::Unit { unit_instance_id }) => self
                        .sample_unit_world_position_at(unit_instance_id, time_ms)
                        .or_else(|| self.unit_world_position_or_tile_center(unit_instance_id))
                        .or(cast_target_anchor_position),
                    None => cast_target_anchor_position,
                }
            }
            SkillAreaAnchorSource::ImpactContext | SkillAreaAnchorSource::ImpactContextStart => {
                self.impact_context_before_step(cast_seq, step_index)
                    .map(|context| context.impact_position)
            }
            SkillAreaAnchorSource::Caster => Some(caster_position),
        }
    }

    pub(in crate::game::battle::core) fn resolve_area_direction_hint(
        &self,
        time_ms: u64,
        area_anchor: SkillAreaAnchorSource,
        caster_owner: Side,
        caster_position: WorldVec2,
        anchor_position: WorldVec2,
        step_target: Option<SkillCastTarget>,
        impact_context: Option<&SkillImpactContext>,
    ) -> WorldVec2 {
        if matches!(
            area_anchor,
            SkillAreaAnchorSource::ImpactContext | SkillAreaAnchorSource::ImpactContextStart
        ) {
            if let Some(direction_hint) = impact_context.and_then(|context| context.direction_hint)
            {
                return direction_hint;
            }

            if caster_position != anchor_position {
                return anchor_position + (anchor_position - caster_position);
            }

            let fallback = match caster_owner {
                Side::Player => WorldVec2::new(0.0, -1.0),
                Side::Opponent => WorldVec2::new(0.0, 1.0),
            };
            return anchor_position + fallback;
        }

        let hint = match step_target {
            Some(SkillCastTarget::Tile { position }) => Some(WorldVec2::from_tile_center(position)),
            Some(SkillCastTarget::Unit { unit_instance_id }) => self
                .sample_unit_world_position_at(unit_instance_id, time_ms)
                .or_else(|| self.unit_world_position_or_tile_center(unit_instance_id)),
            None => impact_context.map(|context| context.impact_position),
        };

        if let Some(hint) = hint.filter(|pos| *pos != anchor_position) {
            return hint;
        }

        if caster_position != anchor_position {
            return anchor_position + (anchor_position - caster_position);
        }

        let fallback = match caster_owner {
            Side::Player => WorldVec2::new(0.0, -1.0),
            Side::Opponent => WorldVec2::new(0.0, 1.0),
        };
        anchor_position + fallback
    }

    pub(in crate::game::battle::core) fn resolve_instant_area_targets(
        &self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        area: &SkillAreaDeliveryDef,
    ) -> Option<(WorldVec2, WorldVec2, WorldVec2, Vec<UnitInstanceId>)> {
        let (caster_owner, origin, anchor_position, direction_hint) = self.resolve_area_geometry(
            time_ms,
            cast_seq,
            step_index,
            caster_instance_id,
            step_target,
            area,
        )?;
        Some((
            origin,
            anchor_position,
            direction_hint,
            self.collect_area_targets_at(
                time_ms,
                caster_owner,
                caster_instance_id,
                origin,
                anchor_position,
                direction_hint,
                area.shape,
                area.hit_targets,
                area.include_caster,
            ),
        ))
    }

    pub(in crate::game::battle::core) fn resolve_area_geometry(
        &self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        area: &SkillAreaDeliveryDef,
    ) -> Option<(Side, WorldVec2, WorldVec2, WorldVec2)> {
        let (caster_owner, caster_tile_pos) =
            self.resolve_cast_origin_context(Some(cast_seq), caster_instance_id, false)?;
        let caster_position = self
            .sample_unit_world_position_at(caster_instance_id, time_ms)
            .unwrap_or_else(|| WorldVec2::from_tile_center(caster_tile_pos));
        let anchor_position = self.resolve_area_anchor_position(
            time_ms,
            cast_seq,
            step_index,
            caster_instance_id,
            step_target,
            area,
        )?;
        let impact_context = self.impact_context_before_step(cast_seq, step_index);
        let direction_hint = self.resolve_area_direction_hint(
            time_ms,
            area.anchor,
            caster_owner,
            caster_position,
            anchor_position,
            step_target,
            impact_context.as_ref(),
        );
        let origin = match area.anchor {
            SkillAreaAnchorSource::CastTargetStart | SkillAreaAnchorSource::ImpactContextStart => {
                anchor_position
            }
            SkillAreaAnchorSource::CastTarget
            | SkillAreaAnchorSource::ImpactContext
            | SkillAreaAnchorSource::Caster => caster_position,
        };
        Some((caster_owner, origin, anchor_position, direction_hint))
    }

    pub(in crate::game::battle::core) fn collect_area_targets_at(
        &self,
        time_ms: u64,
        caster_owner: Side,
        caster_instance_id: UnitInstanceId,
        origin: WorldVec2,
        center: WorldVec2,
        direction_hint: WorldVec2,
        shape: SkillAreaShapeDef,
        hit_targets: SkillHitTargetFilter,
        include_caster: bool,
    ) -> Vec<UnitInstanceId> {
        let hitbox_expansion = legacy_units_to_world(DEFAULT_UNIT_HITBOX_RADIUS_UNITS);
        let query = AreaQueryShape {
            origin,
            center,
            direction_hint,
            shape,
            hitbox_expansion,
        };

        let mut targets: Vec<UnitInstanceId> = self
            .units
            .values()
            .filter(|unit| {
                self.skill_delivery_accepts_unit(
                    caster_owner,
                    caster_instance_id,
                    unit.instance_id,
                    hit_targets,
                    include_caster,
                )
            })
            .filter_map(|unit| {
                let position = self.sample_unit_world_position_at(unit.instance_id, time_ms)?;
                self.spatial_query_backend
                    .contains_area_point(&query, position)
                    .then_some(unit.instance_id)
            })
            .collect();

        targets.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        targets
    }

    pub(in crate::game::battle::core) fn record_skill_area_declared(
        &mut self,
        time_ms: u64,
        area_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        origin: WorldVec2,
        center: WorldVec2,
        direction_hint: WorldVec2,
        area: SkillAreaDeliveryDef,
    ) -> u64 {
        let display_duration_ms = if area.duration_ms == 0 {
            INSTANT_AREA_DISPLAY_DURATION_MS
        } else {
            area.duration_ms
        };

        self.record_timeline(
            time_ms,
            TimelineEvent::SkillAreaDeclared {
                area_id,
                skill_id,
                step_id,
                caster_instance_id,
                target: step_target,
                shape: TimelineSkillAreaShape::from(area.shape),
                origin: timeline_point(origin),
                center: timeline_point(center),
                direction_hint: timeline_point(direction_hint),
                start_time_ms: time_ms,
                duration_ms: area.duration_ms,
                display_duration_ms,
                warning_ms: 0,
                tick_interval_ms: area.tick_interval_ms,
                tick_policy: area.tick_policy,
                hit_targets: area.hit_targets,
                include_caster: area.include_caster,
            },
        )
    }

    pub(in crate::game::battle::core) fn allocate_area_instance_id(
        &mut self,
        cast_seq: u64,
        caster_instance_id: UnitInstanceId,
        time_ms: u64,
    ) -> Uuid {
        const SKILL_AREA_NS: u64 = 0x534B_4152_4541_5A4Eu64; // "SKAREAZN"

        let seq = self.area_seq;
        self.area_seq = self.area_seq.wrapping_add(1);
        determinism::uuid_v4_from_seed(
            cast_seq
                ^ time_ms
                ^ u64::from_be_bytes(caster_instance_id.as_bytes()[..8].try_into().unwrap()),
            SKILL_AREA_NS,
            seq,
        )
    }

    pub(in crate::game::battle::core) fn register_persistent_area(
        &mut self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        skill_id: SkillId,
        step_id: String,
        step_target: Option<SkillCastTarget>,
        area: SkillAreaDeliveryDef,
    ) -> bool {
        area.validate_runtime_contract();
        let Some((_caster_owner, origin, center, direction_hint)) = self.resolve_area_geometry(
            time_ms,
            cast_seq,
            step_index,
            caster_instance_id,
            step_target,
            &area,
        ) else {
            return false;
        };

        let area_id = self.allocate_area_instance_id(cast_seq, caster_instance_id, time_ms);
        let area_declared_seq = self.record_skill_area_declared(
            time_ms,
            area_id,
            skill_id.clone(),
            step_id.clone(),
            caster_instance_id,
            step_target,
            origin,
            center,
            direction_hint,
            area,
        );
        let expires_at_ms = time_ms.saturating_add(u64::from(area.duration_ms));
        let next_tick_ms = area
            .tick_interval_ms
            .map(|interval| time_ms.saturating_add(u64::from(interval)));
        self.active_areas.insert(
            area_id,
            AreaRuntime {
                area_id,
                cast_seq,
                step_index,
                skill_id: skill_id.clone(),
                step_id: step_id.clone(),
                caster_instance_id,
                caster_owner: _caster_owner,
                origin,
                center,
                direction_hint,
                shape: area.shape,
                hit_targets: area.hit_targets,
                include_caster: area.include_caster,
                tick_policy: area.tick_policy,
                spawned_at_ms: time_ms,
                expires_at_ms,
                tick_interval_ms: area.tick_interval_ms,
                next_tick_ms,
                hit_unit_ids: Vec::new(),
                previous_tick_unit_ids: Vec::new(),
            },
        );
        if let Some(cast) = self.active_skill_casts.get_mut(&cast_seq) {
            cast.active_area_ids.push(area_id);
        }

        self.update_skill_cast_impact_context(
            cast_seq,
            step_index,
            SkillImpactContext {
                delivery_id: area_id,
                impact_time_ms: time_ms,
                impact_position: center,
                direction_hint: Some(direction_hint),
                first_hit_unit_id: None,
                hit_unit_ids: Vec::new(),
                spawned_area_id: Some(area_id),
            },
        );

        let cause = crate::game::battle::timeline::TimelineCause::Parent {
            seq: area_declared_seq,
        };
        self.event_queue.push(BattleEvent::SkillAreaTick {
            time_ms,
            area_id,
            cast_seq,
            step_index,
            skill_id: skill_id.clone(),
            step_id: step_id.clone(),
            center,
            cause,
        });
        self.event_queue.push(BattleEvent::SkillAreaExpire {
            time_ms: expires_at_ms,
            area_id,
            cast_seq,
            step_index,
            skill_id,
            step_id,
            cause,
        });
        true
    }

    pub(in crate::game::battle::core) fn apply_skill_area_tick(
        &mut self,
        time_ms: u64,
        area_id: Uuid,
    ) {
        let Some(runtime) = self.active_areas.get(&area_id).cloned() else {
            return;
        };

        let raw_targets = self.collect_area_targets_at(
            time_ms,
            runtime.caster_owner,
            runtime.caster_instance_id,
            runtime.origin,
            runtime.center,
            runtime.direction_hint,
            runtime.shape,
            runtime.hit_targets,
            runtime.include_caster,
        );
        let targets: Vec<UnitInstanceId> = match runtime.tick_policy {
            SkillAreaTickPolicy::EveryTick => raw_targets.clone(),
            SkillAreaTickPolicy::OncePerArea => raw_targets
                .iter()
                .copied()
                .filter(|target_id| !runtime.hit_unit_ids.contains(target_id))
                .collect(),
            SkillAreaTickPolicy::OnEnter => raw_targets
                .iter()
                .copied()
                .filter(|target_id| !runtime.previous_tick_unit_ids.contains(target_id))
                .collect(),
        };

        let Some(skill) = self
            .game_data
            .skill_data
            .get_by_id(&runtime.skill_id)
            .cloned()
        else {
            return;
        };
        let Some(step) = Self::resolve_skill_step(&skill, runtime.step_index, &runtime.step_id)
        else {
            return;
        };
        let (commands, result) =
            Self::build_skill_step_commands(runtime.caster_instance_id, step, &targets);
        let mut resolved_result = result;
        if !commands.is_empty() {
            let summary = self.process_commands(commands, time_ms);
            resolved_result.actual_damage_target_count = resolved_result
                .actual_damage_target_count
                .saturating_add(summary.actual_damage_target_count);
        }
        self.update_skill_cast_impact_context(
            runtime.cast_seq,
            runtime.step_index,
            SkillImpactContext {
                delivery_id: runtime.area_id,
                impact_time_ms: time_ms,
                impact_position: runtime.center,
                direction_hint: Some(runtime.direction_hint),
                first_hit_unit_id: targets.first().copied(),
                hit_unit_ids: targets.clone(),
                spawned_area_id: Some(runtime.area_id),
            },
        );
        self.resolve_skill_step_delivery(
            runtime.cast_seq,
            runtime.step_index,
            resolved_result,
            false,
            time_ms,
        );

        if let Some(active) = self.active_areas.get_mut(&area_id) {
            match active.tick_policy {
                SkillAreaTickPolicy::EveryTick => {}
                SkillAreaTickPolicy::OncePerArea => {
                    for target_id in &targets {
                        if !active.hit_unit_ids.contains(target_id) {
                            active.hit_unit_ids.push(*target_id);
                        }
                    }
                    active
                        .hit_unit_ids
                        .sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
                    active.hit_unit_ids.dedup();
                }
                SkillAreaTickPolicy::OnEnter => {
                    active.previous_tick_unit_ids = raw_targets;
                    active
                        .previous_tick_unit_ids
                        .sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
                    active.previous_tick_unit_ids.dedup();
                }
            }
        }

        if let Some(interval) = runtime.tick_interval_ms {
            if let Some(active) = self.active_areas.get_mut(&area_id) {
                let next_tick_ms = time_ms.saturating_add(u64::from(interval));
                active.next_tick_ms = Some(next_tick_ms);
                if next_tick_ms < active.expires_at_ms {
                    self.event_queue.push(BattleEvent::SkillAreaTick {
                        time_ms: next_tick_ms,
                        area_id,
                        cast_seq: active.cast_seq,
                        step_index: active.step_index,
                        skill_id: active.skill_id.clone(),
                        step_id: active.step_id.clone(),
                        center: active.center,
                        cause: self.recording_cause().unwrap_or_default(),
                    });
                }
            }
        }
        self.schedule_pending_autocasts(time_ms);
    }

    pub(in crate::game::battle::core) fn expire_skill_area(&mut self, area_id: Uuid) {
        let Some(runtime) = self.active_areas.remove(&area_id) else {
            return;
        };
        if let Some(cast) = self.active_skill_casts.get_mut(&runtime.cast_seq) {
            cast.active_area_ids.retain(|id| *id != area_id);
        }
        self.resolve_skill_step_delivery(
            runtime.cast_seq,
            runtime.step_index,
            SkillStepResult::default(),
            true,
            runtime.expires_at_ms,
        );
    }
}
