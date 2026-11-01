use crate::authoring::control;
use uuid::Uuid;

use crate::game::{
    ability::{
        DeliveryDef, SkillAreaAnchorSource, SkillAreaTickPolicy, SkillAreaTracking, SkillHitTargetFilter,
        SkillId, SkillStepDef, SkillTileAreaDeliveryDef, SkillTileAreaOrigin,
    },
    battle::{
        core::{
            movement::types::{EventLogVec2, WorldVec2},
            types::{AreaRuntime, SkillImpactContext, SkillStepResult},
            BattleCore,
        },
        damage::{
            CombatRollIdentity, CombatRollKind, DamageModifiers, DamageSource,
            DamageSourceSnapshot, DamageType,
        },
        enums::BattleEvent,
        event_log::{BattleLogEvent, BattleSkillAreaShape, SkillCastTarget},
        ids::UnitInstanceId,
        tile_range::{FacingDirection, TileRangePattern, TileRangePolicy, TileReachQuery},
    },
    determinism,
    enums::Side,
    resources::Position,
};

const INSTANT_AREA_DISPLAY_DURATION_MS: u32 = 250;

fn event_log_point(position: WorldVec2) -> EventLogVec2 {
    position.quantized_milli()
}

impl BattleCore {
    pub(in crate::game::battle::core) fn tile_area_facing(
        &self,
        caster: UnitInstanceId,
        footprint: &TileRangePattern,
        area: &SkillTileAreaDeliveryDef,
    ) -> Option<FacingDirection> {
        if area.mode == crate::game::ability::SkillTileAreaMode::SelectedGround {
            Some(FacingDirection::Up)
        } else {
            self.tile_area_projection_facing(caster, footprint)
        }
    }
    pub(in crate::game::battle::core) fn charge_authoring_tile_pattern(
        pattern: &TileRangePattern,
        projections: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        if !control::is_active() {
            return Ok(());
        }
        let mut cells = 0usize;
        for row in &pattern.rows {
            control::admit(
                control::Resource::GeometryCells,
                cells,
                row.len(),
                "tile_pattern_cells",
            )?;
            cells += row.len();
            control::charge(
                (row.len() as u64 + 1)
                    .saturating_mul(projections)
                    .saturating_mul(3),
                "tile_pattern_scan",
            )?;
        }
        // Projection validates, locates the anchor, collects cells and sorts.
        let comparisons = u64::from(usize::BITS - cells.leading_zeros());
        control::charge(
            (cells as u64)
                .saturating_mul(comparisons)
                .saturating_mul(projections),
            "tile_pattern_sort",
        )
    }

    fn area_tick_source_snapshot(
        &self,
        runtime: &AreaRuntime,
        time_ms: u64,
    ) -> Result<DamageSourceSnapshot, crate::game::behavior::GameError> {
        const AREA_TICK_DAMAGE_NS: u64 = 0x4152_4541_544B_444Du64; // "AREATKDM"

        let mut high = [0u8; 8];
        let mut low = [0u8; 8];
        high.copy_from_slice(&runtime.area_id.as_bytes()[..8]);
        low.copy_from_slice(&runtime.area_id.as_bytes()[8..]);
        let area_tag = u64::from_be_bytes(high) ^ u64::from_be_bytes(low).rotate_left(29);
        let source_instance_id = determinism::uuid_v4_from_seed(
            self.seed ^ area_tag ^ time_ms.rotate_left(17),
            AREA_TICK_DAMAGE_NS,
            time_ms,
        );

        let mut snapshot = runtime.source_snapshot.try_clone_fact_source()?;
        snapshot.committed_at_ms = time_ms;
        snapshot.crit_roll_percent = None;
        snapshot.crit_roll_identity = CombatRollIdentity::new(
            CombatRollKind::SkillCrit,
            Some(runtime.caster_instance_id),
            source_instance_id,
        );
        Ok(snapshot)
    }

    pub(in crate::game::battle::core) fn tile_area_projection_facing(
        &self,
        caster_instance_id: UnitInstanceId,
        footprint: &TileRangePattern,
    ) -> Option<FacingDirection> {
        self.units
            .get(&caster_instance_id)
            .and_then(|unit| unit.facing_direction)
            .or_else(|| {
                Self::charge_authoring_tile_pattern(footprint, 4).ok()?;
                footprint
                    .is_rotation_invariant()
                    .then_some(FacingDirection::Up)
            })
    }

    pub(in crate::game::battle::core) fn valid_tile_area_tiles(
        &self,
        tile_range: &TileRangePattern,
        anchor_tile: Position,
        facing: FacingDirection,
    ) -> Option<Vec<Position>> {
        Self::charge_authoring_tile_pattern(tile_range, 1).ok()?;
        let query = TileReachQuery::try_new(
            &self.battlefield,
            TileRangePolicy::Pattern,
            Some(tile_range),
            Some(anchor_tile),
            Some(facing),
        )
        .ok()?;
        let valid_tiles = query.materialize_valid_cells();
        (!valid_tiles.is_empty()).then_some(valid_tiles)
    }

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
            self.seed
                ^ cast_seq.rotate_left(17)
                ^ time_ms
                ^ u64::from_be_bytes(caster_instance_id.as_bytes()[..8].try_into().unwrap()),
            SKILL_AREA_DELIVERY_NS,
            delivery_seq,
        )
    }

    pub(in crate::game::battle::core) fn resolve_area_anchor_position_for(
        &self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        anchor: SkillAreaAnchorSource,
    ) -> Option<WorldVec2> {
        let (_, caster_tile_pos) =
            self.resolve_cast_origin_context(Some(cast_seq), caster_instance_id, false)?;
        let caster_position = self
            .sample_unit_world_position_at(caster_instance_id, time_ms)
            .unwrap_or_else(|| WorldVec2::from_tile_center(caster_tile_pos));
        let cast_target_anchor_position = self
            .stored_skill_cast_anchor_world_position(cast_seq)
            .or_else(|| {
                self.stored_skill_cast_anchor_position(cast_seq)
                    .map(WorldVec2::from_tile_center)
            });

        match anchor {
            SkillAreaAnchorSource::AcceptedCastTarget => self.stored_skill_cast_anchor_position(cast_seq).map(WorldVec2::from_tile_center),
            SkillAreaAnchorSource::CastTarget | SkillAreaAnchorSource::CastTargetStart => {
                match step_target {
                    Some(SkillCastTarget::TilePair { .. } | SkillCastTarget::Direction { .. }) => None,
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
            Some(SkillCastTarget::TilePair { .. } | SkillCastTarget::Direction { .. }) => None,
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

    pub(in crate::game::battle::core) fn resolve_tile_area_geometry(
        &self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        area: &SkillTileAreaDeliveryDef,
    ) -> Option<(Side, WorldVec2, WorldVec2, WorldVec2, Position)> {
        let (caster_owner, caster_tile_pos) =
            self.resolve_cast_origin_context(Some(cast_seq), caster_instance_id, false)?;
        let caster_position = self
            .sample_unit_world_position_at(caster_instance_id, time_ms)
            .unwrap_or_else(|| WorldVec2::from_tile_center(caster_tile_pos));
        let anchor_position = self.resolve_area_anchor_position_for(
            time_ms,
            cast_seq,
            step_index,
            caster_instance_id,
            step_target,
            area.anchor,
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
            SkillAreaAnchorSource::AcceptedCastTarget
            | SkillAreaAnchorSource::CastTarget
            | SkillAreaAnchorSource::ImpactContext
            | SkillAreaAnchorSource::Caster => caster_position,
        };
        let anchor_tile = match area.tile_origin {
            SkillTileAreaOrigin::Caster => caster_tile_pos,
            SkillTileAreaOrigin::Anchor => anchor_position.project_to_tile(),
        };
        Some((
            caster_owner,
            origin,
            anchor_position,
            direction_hint,
            anchor_tile,
        ))
    }

    pub(in crate::game::battle::core) fn collect_tile_area_targets_at(
        &self,
        time_ms: u64,
        caster_owner: Side,
        caster_instance_id: UnitInstanceId,
        affected_tiles: &[Position],
        hit_targets: SkillHitTargetFilter,
        include_caster: bool,
    ) -> Vec<UnitInstanceId> {
        if self
            .admit_authoring_runtime_growth(0, "area_target_state")
            .is_err()
            || control::charge(
                (self.units.len() as u64).saturating_mul(affected_tiles.len() as u64 + 1),
                "area_target_scan",
            )
            .is_err()
            || control::charge(
                (self.units.len() as u64)
                    .saturating_mul(u64::from(usize::BITS - self.units.len().leading_zeros())),
                "area_target_sort",
            )
            .is_err()
        {
            return Vec::new();
        }
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
                let body = self.sample_unit_body_at(unit.instance_id, time_ms)?;
                affected_tiles
                    .contains(&body.position.project_to_tile())
                    .then_some(unit.instance_id)
            })
            .collect();

        targets.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        targets
    }

    pub(in crate::game::battle::core) fn resolve_instant_tile_area_targets(
        &self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        step_target: Option<SkillCastTarget>,
        tile_range: &TileRangePattern,
        area: &SkillTileAreaDeliveryDef,
    ) -> Option<(
        WorldVec2,
        WorldVec2,
        WorldVec2,
        Vec<Position>,
        Vec<UnitInstanceId>,
    )> {
        let (caster_owner, origin, anchor_position, direction_hint, anchor_tile) = self
            .resolve_tile_area_geometry(
                time_ms,
                cast_seq,
                step_index,
                caster_instance_id,
                step_target,
                area,
            )?;
        let facing = self.tile_area_facing(caster_instance_id, tile_range, area)?;
        let affected_tiles = self.valid_tile_area_tiles(tile_range, anchor_tile, facing)?;
        let targets = self.collect_tile_area_targets_at(
            time_ms,
            caster_owner,
            caster_instance_id,
            &affected_tiles,
            area.hit_targets,
            area.include_caster,
        );
        Some((
            origin,
            anchor_position,
            direction_hint,
            affected_tiles,
            targets,
        ))
    }

    pub(in crate::game::battle::core) fn persistent_tile_area_geometry_at(
        &self,
        time_ms: u64,
        runtime: &AreaRuntime,
    ) -> Option<(WorldVec2, WorldVec2, WorldVec2, Position)> {
        match runtime.tracking {
            SkillAreaTracking::GroundFixed => Some((
                runtime.origin,
                runtime.center,
                runtime.direction_hint,
                runtime.tile_anchor,
            )),
            SkillAreaTracking::FollowCaster | SkillAreaTracking::FollowTarget => {
                let area = SkillTileAreaDeliveryDef {
                    ground_status: None,
                    mode: Default::default(),
                    emission: None, first_tick_delay_ms: 0,
                    on_hit_caster_heal_percent: None,
                    anchor: runtime.anchor,
                    tile_origin: runtime.tile_origin,
                    tracking: runtime.tracking,
                    hit_targets: runtime.hit_targets,
                    include_caster: runtime.include_caster,
                    tick_policy: runtime.tick_policy,
                    duration_ms: runtime.duration_ms,
                    tick_interval_ms: runtime.tick_interval_ms,
                };
                self.resolve_tile_area_geometry(
                    time_ms,
                    runtime.cast_seq,
                    runtime.step_index,
                    runtime.caster_instance_id,
                    runtime.step_target,
                    &area,
                )
                .map(|(_, origin, center, direction_hint, anchor_tile)| {
                    (origin, center, direction_hint, anchor_tile)
                })
            }
        }
    }

    pub(in crate::game::battle::core) fn record_skill_tile_area_declared(
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
        affected_tiles: Vec<Position>,
        area: SkillTileAreaDeliveryDef,
    ) -> u64 {
        let display_duration_ms = if area.mode == crate::game::ability::SkillTileAreaMode::SelectedGround { area.duration_ms } else { area.first_tick_delay_ms.saturating_add(if area.duration_ms == 0 {
            INSTANT_AREA_DISPLAY_DURATION_MS
        } else {
            area.duration_ms
        }) };

        self.record_event_log(
            time_ms,
            BattleLogEvent::SkillAreaDeclared {
                area_id,
                skill_id,
                step_id,
                caster_instance_id,
                target: step_target,
                shape: BattleSkillAreaShape::TilePattern { affected_tiles },
                origin: event_log_point(origin),
                center: event_log_point(center),
                direction_hint: event_log_point(direction_hint),
                start_time_ms: time_ms,
                duration_ms: area.duration_ms,
                display_duration_ms,
                warning_ms: 0,
                tick_interval_ms: area.tick_interval_ms,
                tick_policy: area.tick_policy,
                tracking: area.tracking,
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
            self.seed
                ^ cast_seq.rotate_left(17)
                ^ time_ms
                ^ u64::from_be_bytes(caster_instance_id.as_bytes()[..8].try_into().unwrap()),
            SKILL_AREA_NS,
            seq,
        )
    }

    pub(in crate::game::battle::core) fn register_persistent_tile_area(
        &mut self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        skill_id: SkillId,
        step_id: String,
        step: &SkillStepDef,
        step_target: Option<SkillCastTarget>,
        tile_range: TileRangePattern,
        area: SkillTileAreaDeliveryDef,
    ) -> Result<bool, crate::game::behavior::GameError> {
        area.validate_runtime_contract();
        if let Some(emission) = &area.emission {
            Self::charge_authoring_tile_pattern(&emission.footprint, 2)?;
            Self::admit_emission_copy(emission)?;

        }
        let Some((_caster_owner, origin, center, direction_hint, anchor_tile)) = self
            .resolve_tile_area_geometry(
                time_ms,
                cast_seq,
                step_index,
                caster_instance_id,
                step_target,
                &area,
            )
        else {
            return Ok(false);
        };
        let facing = if matches!(area.mode, crate::game::ability::SkillTileAreaMode::ChannelGround { .. }) {
            self.active_skill_casts.get(&cast_seq).and_then(|c| c.accepted_facing).or_else(|| self.tile_area_facing(caster_instance_id, &tile_range, &area))
        } else { self.tile_area_facing(caster_instance_id, &tile_range, &area) };
        let Some(facing) = facing else {
            return Ok(false);
        };
        let Some(mut source_snapshot) = self.damage_source_snapshot_template_for_unit(
            caster_instance_id,
            DamageSource::Ability,
            DamageType::default(),
            0,
            DamageModifiers::default(),
            0,
            time_ms,
            false,
        ) else {
            return Ok(false);
        };
        source_snapshot.skill_cast_source = self
            .active_skill_casts
            .get(&cast_seq)
            .and_then(|cast| cast.execution_source.clone());
        source_snapshot.fact_source.cast = source_snapshot.skill_cast_source.clone();

        let Some(affected_tiles) = self.valid_tile_area_tiles(&tile_range, anchor_tile, facing)
        else {
            return Ok(false);
        };
        let targets = self.collect_tile_area_targets_at(
            time_ms,
            _caster_owner,
            caster_instance_id,
            &affected_tiles,
            area.hit_targets,
            area.include_caster,
        );
        if !matches!(area.mode, crate::game::ability::SkillTileAreaMode::SelectedGround | crate::game::ability::SkillTileAreaMode::ChannelGround { .. }) && area.first_tick_delay_ms == 0 && !matches!(&step.delivery, DeliveryDef::TileArea { area, .. } if area.emission.is_some()) && self.skill_step_needs_hostile_usefulness_gate(
            caster_instance_id,
            step,
            area.hit_targets,
            &targets,
        ) && !self
            .effective_footprint_usefulness(
                time_ms,
                step_index,
                caster_instance_id,
                step,
                step_target,
            )?
            .unwrap_or_else(|| {
                self.skill_step_has_useful_hostile_target(caster_instance_id, step, &targets)
            })
        {
            return Ok(false);
        }

        self.admit_authoring_runtime_growth(2, "persistent_area_spawn")?;
        let area_id = self.allocate_area_instance_id(cast_seq, caster_instance_id, time_ms);
        let area_declared_seq = self.record_skill_tile_area_declared(
            time_ms,
            area_id,
            skill_id.clone(),
            step_id.clone(),
            caster_instance_id,
            step_target,
            origin,
            center,
            direction_hint,
            affected_tiles,
            area.clone(),
        );
        let first_tick_ms = time_ms.checked_add(u64::from(area.first_tick_delay_ms)).ok_or(crate::game::behavior::GameError::InvalidAction)?;
        let lifetime_start = if area.mode == crate::game::ability::SkillTileAreaMode::SelectedGround { time_ms } else { first_tick_ms };
        let expires_at_ms = lifetime_start.checked_add(u64::from(area.duration_ms)).ok_or(crate::game::behavior::GameError::InvalidAction)?;
        let next_tick_ms = area
            .tick_interval_ms
            .map(|interval| first_tick_ms.saturating_add(u64::from(interval)));
        source_snapshot.fact_source = self.reserve_finite_source(source_snapshot.fact_source)?;
        source_snapshot.value_expr_capture = self.build_value_expr_delivery_capture(
            step,
            &source_snapshot.fact_source,
            None,
            time_ms,
            Some(expires_at_ms),
        )?;
        let tick_source = source_snapshot.fact_source.clone();
        source_snapshot.admit_fact_owner()?;
        if area.emission.is_some() {
            self.replace_area_emitter(caster_instance_id, &skill_id, &step_id, time_ms)?;
        }
        let channel_started_at = if matches!(area.mode, crate::game::ability::SkillTileAreaMode::ChannelGround { .. }) {
            Some(self.channel_started_at(caster_instance_id, time_ms)?.unwrap_or(time_ms))
        } else { None };
        self.active_areas.insert(
            area_id,
            AreaRuntime {
                channel_started_at,
                source_generation: self.content_identity.clone(),
                mode: area.mode,
                ground_status: area.ground_status.map(|program| Box::new(ground_status::GroundAreaState::new(program, area_declared_seq))),
                emission: area.emission,
                area_id,
                cast_seq,
                step_index,
                skill_id: skill_id.clone(),
                step_id: step_id.clone(),
                caster_instance_id,
                caster_owner: _caster_owner,
                source_snapshot,
                anchor: area.anchor,
                tile_origin: area.tile_origin,
                tracking: area.tracking,
                origin,
                center,
                direction_hint,
                facing,
                tile_range,
                tile_anchor: anchor_tile,
                hit_targets: area.hit_targets,
                include_caster: area.include_caster,
                tick_policy: area.tick_policy,
                duration_ms: area.duration_ms,
                spawned_at_ms: time_ms,
                expires_at_ms,
                tick_interval_ms: area.tick_interval_ms,
                next_tick_ms,
                step_target,
                hit_unit_ids: Vec::new(),
                previous_tick_unit_ids: Vec::new(),
                effective_history: None,
            },
        );
        if let Some(cast) = self.active_skill_casts.get_mut(&cast_seq) {
            cast.active_area_ids.push(area_id);
        }
        if matches!(area.mode, crate::game::ability::SkillTileAreaMode::ChannelGround { pause_basic: true }) {
            self.pause_channel_basic_attack(caster_instance_id)?;
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
        crate::game::battle::core::area_observation::record(&mut self.authoring_observation, &self.active_areas[&area_id], time_ms, crate::game::battle::core::area_observation::AreaChange::Started, None)?;
        control::poll("registered_area_context")?;

        let cause = crate::game::battle::event_log::BattleEventCause::Parent {
            seq: area_declared_seq,
        };
        if self.active_areas[&area_id].ground_status.is_some() {
            self.reconcile_ground_status(time_ms)?;
        } else if area.mode != crate::game::ability::SkillTileAreaMode::SelectedGround || first_tick_ms < expires_at_ms {
        self.enqueue_carrier_completion(
            BattleEvent::SkillAreaTick {
                time_ms: first_tick_ms,
                area_id,
                cast_seq,
                step_index,
                skill_id: skill_id.clone(),
                step_id: step_id.clone(),
                center,
                cause,
            },
            None,
            tick_source,
        )?;
        }
        self.event_queue.push(BattleEvent::SkillAreaExpire {
            time_ms: expires_at_ms,
            area_id,
            cast_seq,
            step_index,
            skill_id,
            step_id,
            cause,
        });
        control::poll("area_registration_complete")?;
        Ok(true)
    }

    pub(in crate::game::battle::core) fn apply_skill_area_tick(
        &mut self,
        time_ms: u64,
        area_id: Uuid,
    ) -> Result<(), crate::game::behavior::GameError> {
        let Some(runtime) = self.active_areas.get(&area_id) else {
            return Ok(());
        };
        if runtime.ground_status.is_some() {
            return self.reconcile_ground_status(time_ms);
        }
        Self::charge_authoring_tile_pattern(&runtime.tile_range, 1)?;
        control::charge(
            runtime.hit_unit_ids.len() as u64 + runtime.previous_tick_unit_ids.len() as u64 + 1,
            "area_runtime_copy",
        )?;
        self.admit_authoring_runtime_growth(0, "area_tick_state")?;
        runtime.source_snapshot.admit_fact_owner()?;
        runtime.admit_effective_history_copy()?;
        if let Some(emission) = &runtime.emission {
            Self::charge_authoring_tile_pattern(&emission.footprint, 1)?;
            Self::admit_emission_copy(emission)?;
        }
        let runtime = runtime.clone();
        if runtime.emission.is_some() {
            return self.apply_area_emission_tick(&runtime, time_ms);
        }

        let Some((origin, center, direction_hint, anchor_tile)) =
            self.persistent_tile_area_geometry_at(time_ms, &runtime)
        else {
            return Ok(());
        };
        let facing = match runtime.tracking {
            SkillAreaTracking::GroundFixed => runtime.facing,
            SkillAreaTracking::FollowCaster | SkillAreaTracking::FollowTarget => {
                let Some(facing) = self
                    .tile_area_projection_facing(runtime.caster_instance_id, &runtime.tile_range)
                else {
                    return Ok(());
                };
                facing
            }
        };
        let Some(affected_tiles) =
            self.valid_tile_area_tiles(&runtime.tile_range, anchor_tile, facing)
        else {
            return Ok(());
        };
        let raw_targets = self.collect_tile_area_targets_at(
            time_ms,
            runtime.caster_owner,
            runtime.caster_instance_id,
            &affected_tiles,
            runtime.hit_targets,
            runtime.include_caster,
        );
        let history = match runtime.tick_policy {
            SkillAreaTickPolicy::EveryTick => 0,
            SkillAreaTickPolicy::OncePerArea => runtime.hit_unit_ids.len(),
            SkillAreaTickPolicy::OnEnter => runtime.previous_tick_unit_ids.len(),
        };
        control::charge(
            (raw_targets.len() as u64).saturating_mul(history as u64 + 1),
            "area_tick_policy_scan",
        )?;
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

        let Some(skill) =
            self.cloned_skill_for_source(&runtime.skill_id, &runtime.source_snapshot.fact_source)?
        else {
            return Ok(());
        };
        let Some(step) = Self::resolve_skill_step(&skill, runtime.step_index, &runtime.step_id)
        else {
            return Ok(());
        };
        let mut source_snapshot = self.area_tick_source_snapshot(&runtime, time_ms)?;
        let inherited_delivery = self.runtime_fact_source.action == source_snapshot.fact_source.action
            && self.runtime_fact_source.delivery.is_some()
            && self.runtime_fact_source.delivery != source_snapshot.fact_source.delivery;
        if inherited_delivery {
            source_snapshot.fact_source = self.runtime_fact_source.clone();
        }
        let independent_tick = matches!(&step.delivery, DeliveryDef::TileArea { area, .. }
            if area.mode == crate::game::ability::SkillTileAreaMode::SelectedGround);
        if independent_tick {
            source_snapshot.fact_source = self.independent_periodic_source(&source_snapshot.fact_source)?;
        }
        if inherited_delivery || independent_tick {
            if let Some(capture) = &source_snapshot.value_expr_capture {
                source_snapshot.value_expr_capture =
                    Some(capture.rebind_consumption_source(&source_snapshot.fact_source)?);
            }
        }
        // Commands (including healing) inherit the active execution source.
        // Install the tick root for the whole fan-out, then restore it even on error.
        let prior_source = independent_tick.then(|| std::mem::replace(
            &mut self.runtime_fact_source, source_snapshot.fact_source.clone(),
        ));
        let resolved_result = self.execute_ordered_skill_effects(
            runtime.caster_instance_id,
            &skill.id,
            step,
            &targets,
            time_ms,
            source_snapshot.committed_at_ms,
            Some(&source_snapshot),
            source_snapshot.skill_cast_source.clone(),
        );
        if let Some(prior) = prior_source {
            self.runtime_fact_source = prior;
        }
        let resolved_result = resolved_result?;
        if let crate::game::ability::DeliveryDef::TileArea { area, .. } = &step.delivery {
            self.apply_area_hit_reaction(runtime.caster_instance_id, area.on_hit_caster_heal_percent,
                &resolved_result, Some(&source_snapshot), time_ms)?;
        }
        self.update_skill_cast_impact_context(
            runtime.cast_seq,
            runtime.step_index,
            SkillImpactContext {
                delivery_id: runtime.area_id,
                impact_time_ms: time_ms,
                impact_position: center,
                direction_hint: Some(direction_hint),
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

        control::poll("area_step_resolved")?;
        if let Some(active) = self.active_areas.get(&area_id) {
            let change = match active.tick_policy {
                SkillAreaTickPolicy::EveryTick => (0, 0),
                SkillAreaTickPolicy::OncePerArea => {
                    control::charge(
                        (targets.len() as u64).saturating_mul(active.hit_unit_ids.len() as u64 + 1),
                        "area_hit_history_scan",
                    )?;
                    (
                        0,
                        targets
                            .iter()
                            .filter(|target| !active.hit_unit_ids.contains(target))
                            .count(),
                    )
                }
                SkillAreaTickPolicy::OnEnter => {
                    (active.previous_tick_unit_ids.len(), raw_targets.len())
                }
            };
            let sort_size = active
                .hit_unit_ids
                .len()
                .saturating_add(targets.len())
                .max(raw_targets.len());
            control::charge(
                (sort_size as u64)
                    .saturating_mul(u64::from(usize::BITS - sort_size.leading_zeros())),
                "area_history_sort",
            )?;
            self.admit_authoring_runtime_replacement(change.0, change.1, "area_hit_history")?;
        }

        if let Some(active) = self.active_areas.get_mut(&area_id) {
            active.origin = origin;
            active.center = center;
            active.direction_hint = direction_hint;
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
                    let event = BattleEvent::SkillAreaTick {
                        time_ms: next_tick_ms,
                        area_id,
                        cast_seq: active.cast_seq,
                        step_index: active.step_index,
                        skill_id: active.skill_id.clone(),
                        step_id: active.step_id.clone(),
                        center: active.center,
                        cause: self.recording_cause().unwrap_or_default(),
                    };
                    self.enqueue_carrier_completion(
                        event,
                        None,
                        runtime.source_snapshot.fact_source.clone(),
                    )?;
                }
            }
        }
        self.schedule_pending_autocasts(time_ms);
        Ok(())
    }

    pub(in crate::game::battle::core) fn apply_area_hit_reaction(
        &mut self,
        caster: UnitInstanceId,
        percent: Option<u32>,
        result: &SkillStepResult,
        source: Option<&DamageSourceSnapshot>,
        now: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        let Some(percent) = percent else { return Ok(()); };
        if result.effective_hit_count == 0 || !self.units.get(&caster).is_some_and(|u| u.is_active()) {
            return Ok(());
        }
        crate::game::battle::runtime_control::admit_retained_growth(0,
            Some(std::mem::size_of::<crate::game::battle::damage::BattleCommand>()), "area_hit_reaction")?;
        let prior = if let Some(snapshot) = source {
            snapshot.admit_fact_owner()?;
            Some(std::mem::replace(&mut self.runtime_fact_source, snapshot.fact_source.observation()))
        } else { None };
        self.process_commands(vec![crate::game::battle::damage::BattleCommand::ApplyHeal {
            target_id: caster, flat: 0, percent: percent as i32, source_id: Some(caster),
        }], now);
        if let Some(prior) = prior { self.runtime_fact_source = prior; }
        control::poll("area_hit_reaction_applied")?;
        crate::game::battle::runtime_control::poll()
    }

    pub(in crate::game::battle::core) fn expire_skill_area(
        &mut self,
        area_id: Uuid,
    ) -> Result<(), crate::game::behavior::GameError> {
        let Some(now) = self.active_areas.get(&area_id).map(|r| r.expires_at_ms) else { return Ok(()); };
        self.finish_skill_area_at(area_id, now)
    }

    pub(in crate::game::battle::core) fn finish_skill_area_at(
        &mut self, area_id: Uuid, now: u64,
    ) -> Result<(), crate::game::behavior::GameError> {
        self.finish_skill_area_at_with_reason(area_id, now, true, crate::game::ability::sustained::SustainedEndReason::Expired)
    }
    pub(super) fn finish_skill_area_at_with_reason(&mut self, area_id:Uuid, now:u64, resume_basic:bool, reason:crate::game::ability::sustained::SustainedEndReason) -> Result<(), crate::game::behavior::GameError> {
        let Some(runtime) = self.active_areas.remove(&area_id) else {
            return Ok(());
        };
        if let Some(ground) = &runtime.ground_status {
            for target in ground.affected_targets() { self.refresh_sustained_target(target)?; }
            self.refresh_block_state();
            self.schedule_continuous_movement_tick(now);
        }
        if let Some(cast) = self.active_skill_casts.get_mut(&runtime.cast_seq) {
            cast.active_area_ids.retain(|id| *id != area_id);
        }
        self.resolve_skill_step_delivery(
            runtime.cast_seq,
            runtime.step_index,
            SkillStepResult::default(),
            true,
            now,
        );
        if resume_basic && matches!(runtime.mode, crate::game::ability::SkillTileAreaMode::ChannelGround { .. }) && !self.channel_active(runtime.caster_instance_id)
            && self.units.get(&runtime.caster_instance_id).is_some_and(|u| u.is_active()) {
            self.event_queue.push(BattleEvent::AttackStart { time_ms: now, attacker_instance_id: runtime.caster_instance_id, target_instance_id: None, schedule_next: true, cause: Default::default() });
        }
        self.close_source_resource_delivery(runtime.source_snapshot.fact_source.resource_delivery().map(std::sync::Arc::as_ref))?;
        if let Some(ticket) = &runtime.source_snapshot.fact_source.delivery {
            self.runtime_facts
                .close_delivery(&ticket, now)?;
        }
        crate::game::battle::core::area_observation::record(&mut self.authoring_observation, &runtime, now, crate::game::battle::core::area_observation::AreaChange::Ended { reason }, runtime.ground_status.as_ref().map(|g| g.membership()))?;
        if self.game_data.equipment_state_catalog().has_owner_stun_end
            && !matches!(reason, crate::game::ability::sustained::SustainedEndReason::Terminal
                | crate::game::ability::sustained::SustainedEndReason::SourceUnavailable
                | crate::game::ability::sustained::SustainedEndReason::TargetUnavailable) {
            if let Some(ground) = &runtime.ground_status {
                for target in ground.stun_targets()? {
                    self.dispatch_equipment_stun_end(runtime.caster_instance_id, target, now)?;
                }
            }
        }
        control::poll("area_expired")?;
        Ok(())
    }
}

mod emission;
#[cfg(test)]
mod delayed_tests;

#[cfg(test)]
mod emission_tests;

#[cfg(test)]
mod ground_tests;

pub(in crate::game::battle::core) mod ground_status;
