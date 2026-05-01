use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{
    ability::{SkillDef, SkillId, SkillKind, SkillProjectileCollisionDef, SkillStepDef},
    battle::{
        core::{
            commands::projectile_flight_ms_for_delivery,
            movement::types::{WorldVec2, LEGACY_POSITION_UNITS_PER_WORLD},
            spatial::{legacy_units_to_world, DEFAULT_UNIT_HITBOX_RADIUS_UNITS},
            types::{ActiveProjectileRuntime, ProjectileGuidance, SkillImpactContext},
            BattleCore,
        },
        enums::BattleEvent,
        ids::UnitInstanceId,
        timeline::{SkillCastTarget, TimelineCause, TimelineEvent, TimelineProjectileGuidance},
    },
    determinism,
    enums::Side,
};

const SKILL_DELIVERY_NS: u64 = 0x534B_494C_4C44_4C56u64; // "SKILLDLV"
const SKILL_PROJECTILE_REEVALUATION_TICK_MS: u64 = 1;

#[derive(Debug, Clone)]
pub(in crate::game::battle::core) struct SkillProjectileImpactLaunch {
    pub(in crate::game::battle::core) fired_at_ms: u64,
    pub(in crate::game::battle::core) cast_seq: u64,
    pub(in crate::game::battle::core) step_index: usize,
    pub(in crate::game::battle::core) skill_id: SkillId,
    pub(in crate::game::battle::core) step_id: String,
    pub(in crate::game::battle::core) caster_instance_id: UnitInstanceId,
    pub(in crate::game::battle::core) caster_owner: Side,
    pub(in crate::game::battle::core) start: WorldVec2,
    pub(in crate::game::battle::core) aim: WorldVec2,
    pub(in crate::game::battle::core) target: Option<SkillCastTarget>,
    pub(in crate::game::battle::core) speed_units_per_ms: u32,
    pub(in crate::game::battle::core) target_unit_id: Option<UnitInstanceId>,
    pub(in crate::game::battle::core) travel_time_ms: Option<u64>,
    pub(in crate::game::battle::core) collision: SkillProjectileCollisionDef,
    pub(in crate::game::battle::core) projectile_vfx_id: Option<String>,
    pub(in crate::game::battle::core) impact_vfx_id: Option<String>,
    pub(in crate::game::battle::core) cause: TimelineCause,
}

fn sample_projectile_position_at(
    start: WorldVec2,
    aim: WorldVec2,
    elapsed_ms: u64,
    total_ms: u64,
) -> WorldVec2 {
    if total_ms == 0 || elapsed_ms >= total_ms {
        return aim;
    }

    let t = elapsed_ms as f32 / total_ms.max(1) as f32;
    start + (aim - start) * t
}

fn projectile_travel_ms(start: WorldVec2, aim: WorldVec2, speed_units_per_ms: u32) -> u64 {
    let distance_units = (start.distance(aim) * LEGACY_POSITION_UNITS_PER_WORLD).ceil() as u64;
    projectile_flight_ms_for_delivery(distance_units, speed_units_per_ms)
}

fn projectile_impact_position_at_hit_fraction(
    window_start: WorldVec2,
    window_end: WorldVec2,
    hit_fraction: f32,
) -> WorldVec2 {
    window_start + (window_end - window_start) * hit_fraction.clamp(0.0, 1.0)
}

fn timeline_projectile_guidance(guidance: ProjectileGuidance) -> TimelineProjectileGuidance {
    match guidance {
        ProjectileGuidance::Homing => TimelineProjectileGuidance::Homing,
        ProjectileGuidance::Fixed => TimelineProjectileGuidance::Fixed,
    }
}

impl BattleCore {
    pub(in crate::game::battle::core) fn advance_skill_projectile(
        &mut self,
        time_ms: u64,
        delivery_id: Uuid,
    ) {
        let Some(mut runtime) = self.active_projectiles.remove(&delivery_id) else {
            return;
        };

        let reevaluation_time_ms = time_ms
            .min(runtime.spawned_at_ms.saturating_add(runtime.max_travel_ms))
            .max(runtime.spawned_at_ms);

        if runtime
            .last_reevaluation_ms
            .is_some_and(|last| reevaluation_time_ms <= last)
        {
            self.active_projectiles.insert(delivery_id, runtime);
            return;
        }

        if runtime.guidance == ProjectileGuidance::Homing {
            self.advance_homing_skill_projectile_runtime(runtime, reevaluation_time_ms);
            return;
        }

        let from_elapsed_ms = runtime.last_reevaluation_ms.map_or(0, |last| {
            last.saturating_sub(runtime.spawned_at_ms).saturating_add(1)
        });
        let to_elapsed_ms = reevaluation_time_ms.saturating_sub(runtime.spawned_at_ms);
        let impacts = self.collect_fixed_skill_projectile_hits_in_window(
            &runtime,
            from_elapsed_ms,
            to_elapsed_ms,
        );

        runtime.current_position = sample_projectile_position_at(
            runtime.start,
            runtime.aim,
            to_elapsed_ms,
            runtime.max_travel_ms,
        );
        runtime.last_reevaluation_ms = Some(reevaluation_time_ms);

        let piercing = Self::skill_projectile_pierces(runtime.collision);
        let expired = to_elapsed_ms >= runtime.max_travel_ms;
        let hit_limit_reached = Self::skill_projectile_hit_limit(runtime.collision)
            .is_some_and(|limit| runtime.hit_unit_ids.len().saturating_add(impacts.len()) >= limit);
        let should_terminate = expired || hit_limit_reached || (!piercing && !impacts.is_empty());

        for (impact_index, (impact_time_ms, hit_unit_id, impact_position)) in
            impacts.iter().enumerate()
        {
            runtime.hit_unit_ids.push(*hit_unit_id);
            self.enqueue_skill_projectile_impact(
                *impact_time_ms,
                runtime.delivery_id,
                runtime.cast_seq,
                runtime.step_index,
                runtime.skill_id.clone(),
                runtime.step_id.clone(),
                runtime.caster_instance_id,
                *impact_position,
                Some(*hit_unit_id),
                runtime.impact_vfx_id.clone(),
                should_terminate && impact_index + 1 == impacts.len(),
                runtime.cause.clone(),
            );
        }

        let had_new_hit = !impacts.is_empty();

        if expired && runtime.hit_unit_ids.is_empty() {
            self.enqueue_skill_projectile_impact(
                runtime.spawned_at_ms.saturating_add(runtime.max_travel_ms),
                runtime.delivery_id,
                runtime.cast_seq,
                runtime.step_index,
                runtime.skill_id,
                runtime.step_id,
                runtime.caster_instance_id,
                runtime.current_position,
                None,
                runtime.impact_vfx_id,
                true,
                runtime.cause,
            );
            return;
        }

        if hit_limit_reached || (!piercing && had_new_hit) || expired {
            return;
        }

        runtime.next_reevaluation_ms =
            reevaluation_time_ms.saturating_add(SKILL_PROJECTILE_REEVALUATION_TICK_MS);
        self.event_queue.push(BattleEvent::SkillProjectileAdvance {
            time_ms: runtime.next_reevaluation_ms,
            delivery_id: runtime.delivery_id,
            cause: runtime.cause.clone(),
        });
        self.active_projectiles.insert(delivery_id, runtime);
    }

    fn advance_homing_skill_projectile_runtime(
        &mut self,
        mut runtime: ActiveProjectileRuntime,
        reevaluation_time_ms: u64,
    ) {
        let Some(target_unit_id) = runtime.target_unit_id else {
            self.enqueue_skill_projectile_impact(
                reevaluation_time_ms,
                runtime.delivery_id,
                runtime.cast_seq,
                runtime.step_index,
                runtime.skill_id,
                runtime.step_id,
                runtime.caster_instance_id,
                runtime.current_position,
                None,
                runtime.impact_vfx_id,
                true,
                runtime.cause,
            );
            return;
        };

        let target_alive = self.units.get(&target_unit_id).is_some_and(|unit| {
            !unit.is_dead()
                && self.skill_delivery_accepts_unit(
                    runtime.caster_owner,
                    runtime.caster_instance_id,
                    target_unit_id,
                    runtime.collision.hit_targets,
                    false,
                )
        });
        let Some(target_body) = target_alive
            .then(|| self.unit_body_view(target_unit_id))
            .flatten()
        else {
            self.enqueue_skill_projectile_impact(
                reevaluation_time_ms,
                runtime.delivery_id,
                runtime.cast_seq,
                runtime.step_index,
                runtime.skill_id,
                runtime.step_id,
                runtime.caster_instance_id,
                runtime.current_position,
                None,
                runtime.impact_vfx_id,
                true,
                runtime.cause,
            );
            return;
        };

        let from_time_ms = runtime
            .last_reevaluation_ms
            .unwrap_or(runtime.spawned_at_ms);
        let window_duration_ms = reevaluation_time_ms.saturating_sub(from_time_ms);
        let aim = target_body.position;
        let direction = aim - runtime.current_position;
        let distance = direction.length();
        let speed_world_per_ms =
            runtime.speed_units_per_ms as f32 / LEGACY_POSITION_UNITS_PER_WORLD;
        let max_step = speed_world_per_ms * window_duration_ms as f32;
        let next_position = if distance <= f32::EPSILON
            || runtime.speed_units_per_ms == 0
            || max_step >= distance
        {
            aim
        } else {
            runtime.current_position + direction * (max_step / distance)
        };
        let reach =
            legacy_units_to_world(i64::from(runtime.collision.radius_units)) + target_body.radius;

        if let Some(hit_fraction) = self.spatial_query_backend.projectile_sweep_hit_fraction(
            runtime.current_position,
            next_position,
            reach,
            target_body.position,
        ) {
            let elapsed_delta = ((window_duration_ms as f32) * hit_fraction).ceil() as u64;
            let impact_time_ms = from_time_ms.saturating_add(elapsed_delta.min(window_duration_ms));
            let impact_position = projectile_impact_position_at_hit_fraction(
                runtime.current_position,
                next_position,
                hit_fraction,
            );
            self.enqueue_skill_projectile_impact(
                impact_time_ms,
                runtime.delivery_id,
                runtime.cast_seq,
                runtime.step_index,
                runtime.skill_id,
                runtime.step_id,
                runtime.caster_instance_id,
                impact_position,
                Some(target_unit_id),
                runtime.impact_vfx_id,
                true,
                runtime.cause,
            );
            return;
        }

        let expired =
            reevaluation_time_ms >= runtime.spawned_at_ms.saturating_add(runtime.max_travel_ms);
        runtime.current_position = next_position;
        runtime.aim = aim;
        runtime.last_reevaluation_ms = Some(reevaluation_time_ms);

        if expired {
            self.enqueue_skill_projectile_impact(
                reevaluation_time_ms,
                runtime.delivery_id,
                runtime.cast_seq,
                runtime.step_index,
                runtime.skill_id,
                runtime.step_id,
                runtime.caster_instance_id,
                runtime.current_position,
                None,
                runtime.impact_vfx_id,
                true,
                runtime.cause,
            );
            return;
        }

        runtime.next_reevaluation_ms =
            reevaluation_time_ms.saturating_add(SKILL_PROJECTILE_REEVALUATION_TICK_MS);
        self.event_queue.push(BattleEvent::SkillProjectileAdvance {
            time_ms: runtime.next_reevaluation_ms,
            delivery_id: runtime.delivery_id,
            cause: runtime.cause.clone(),
        });
        self.active_projectiles.insert(runtime.delivery_id, runtime);
    }

    fn skill_projectile_pierces(collision: SkillProjectileCollisionDef) -> bool {
        collision.piercing || matches!(collision.despawn_on_hit, Some(false))
    }

    fn skill_projectile_hit_limit(collision: SkillProjectileCollisionDef) -> Option<usize> {
        collision.validate_runtime_contract();
        collision.max_hits.map(|hits| hits as usize)
    }

    fn collect_fixed_skill_projectile_hits_in_window(
        &self,
        runtime: &ActiveProjectileRuntime,
        from_elapsed_ms: u64,
        to_elapsed_ms: u64,
    ) -> Vec<(u64, UnitInstanceId, WorldVec2)> {
        if from_elapsed_ms > to_elapsed_ms {
            return Vec::new();
        }

        let reach = legacy_units_to_world(i64::from(runtime.collision.radius_units))
            + legacy_units_to_world(DEFAULT_UNIT_HITBOX_RADIUS_UNITS);
        let piercing = Self::skill_projectile_pierces(runtime.collision);
        let max_hits = Self::skill_projectile_hit_limit(runtime.collision);
        let mut seen_hits: HashSet<UnitInstanceId> = runtime.hit_unit_ids.iter().copied().collect();
        let mut impacts = Vec::new();

        let window_duration_ms = to_elapsed_ms.saturating_sub(from_elapsed_ms);
        let window_start = sample_projectile_position_at(
            runtime.start,
            runtime.aim,
            from_elapsed_ms,
            runtime.max_travel_ms,
        );
        let window_end = sample_projectile_position_at(
            runtime.start,
            runtime.aim,
            to_elapsed_ms,
            runtime.max_travel_ms,
        );

        let mut hits: Vec<(u64, f32, UnitInstanceId, WorldVec2)> = self
            .units
            .values()
            .filter(|unit| {
                self.skill_delivery_accepts_unit(
                    runtime.caster_owner,
                    runtime.caster_instance_id,
                    unit.instance_id,
                    runtime.collision.hit_targets,
                    false,
                ) && !seen_hits.contains(&unit.instance_id)
            })
            .filter_map(|unit| {
                let target_pos = self.sample_unit_world_position_at(
                    unit.instance_id,
                    runtime.spawned_at_ms.saturating_add(to_elapsed_ms),
                )?;
                let hit_fraction = self.spatial_query_backend.projectile_sweep_hit_fraction(
                    window_start,
                    window_end,
                    reach,
                    target_pos,
                )?;
                let elapsed_delta = ((window_duration_ms as f32) * hit_fraction).ceil() as u64;
                let hit_elapsed_ms =
                    from_elapsed_ms.saturating_add(elapsed_delta.min(window_duration_ms));
                let projectile_pos = projectile_impact_position_at_hit_fraction(
                    window_start,
                    window_end,
                    hit_fraction,
                );
                Some((
                    hit_elapsed_ms,
                    projectile_pos.distance_squared(target_pos),
                    unit.instance_id,
                    projectile_pos,
                ))
            })
            .collect();

        hits.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.total_cmp(&b.1))
                .then_with(|| a.2.as_bytes().cmp(b.2.as_bytes()))
        });

        for (hit_elapsed_ms, _, hit_unit_id, projectile_pos) in hits {
            if seen_hits.insert(hit_unit_id) {
                impacts.push((
                    runtime.spawned_at_ms.saturating_add(hit_elapsed_ms),
                    hit_unit_id,
                    projectile_pos,
                ));
            }
            if max_hits.is_some_and(|limit| seen_hits.len() >= limit) {
                return impacts;
            }
            if !piercing {
                return impacts;
            }
        }

        impacts
    }

    fn next_skill_delivery_id(
        &mut self,
        fired_at_ms: u64,
        caster_instance_id: UnitInstanceId,
    ) -> Uuid {
        let seed = self
            .recording_cause()
            .and_then(|cause| cause.parent_seq())
            .unwrap_or_else(|| {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&caster_instance_id.as_bytes()[..8]);
                fired_at_ms ^ u64::from_be_bytes(bytes)
            });
        let delivery_seq = self.projectile_seq;
        self.projectile_seq = self.projectile_seq.wrapping_add(1);
        determinism::uuid_v4_from_seed(seed, SKILL_DELIVERY_NS, delivery_seq)
    }

    fn enqueue_skill_projectile_impact(
        &mut self,
        time_ms: u64,
        delivery_id: Uuid,
        cast_seq: u64,
        step_index: usize,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        impact_position: WorldVec2,
        first_hit_unit_id: Option<UnitInstanceId>,
        impact_vfx_id: Option<String>,
        terminal: bool,
        cause: TimelineCause,
    ) {
        self.event_queue.push(BattleEvent::SkillProjectileImpact {
            time_ms,
            delivery_id,
            cast_seq,
            step_index,
            skill_id,
            step_id,
            caster_instance_id,
            impact_position,
            first_hit_unit_id,
            impact_vfx_id,
            terminal,
            cause,
        });
    }

    fn record_skill_projectile_launched(
        &mut self,
        launch: &SkillProjectileImpactLaunch,
        delivery_id: Uuid,
        guidance: ProjectileGuidance,
        expected_end_time_ms: u64,
    ) {
        self.with_recording_context(launch.cause, |core| {
            core.record_timeline(
                launch.fired_at_ms,
                TimelineEvent::SkillProjectileLaunched {
                    delivery_id,
                    skill_id: launch.skill_id.clone(),
                    step_id: launch.step_id.clone(),
                    caster_instance_id: launch.caster_instance_id,
                    target: launch.target,
                    guidance: timeline_projectile_guidance(guidance),
                    start: launch.start.quantized_milli(),
                    aim: launch.aim.quantized_milli(),
                    fired_at_ms: launch.fired_at_ms,
                    expected_end_time_ms,
                    projectile_vfx_id: launch.projectile_vfx_id.clone(),
                },
            );
        });
    }

    fn spawn_homing_skill_projectile(&mut self, launch: SkillProjectileImpactLaunch) {
        launch.collision.validate_homing_runtime_contract();
        let delivery_id =
            self.next_skill_delivery_id(launch.fired_at_ms, launch.caster_instance_id);
        let max_travel_ms = launch.travel_time_ms.unwrap_or_else(|| {
            projectile_travel_ms(launch.start, launch.aim, launch.speed_units_per_ms)
        });
        self.record_skill_projectile_launched(
            &launch,
            delivery_id,
            ProjectileGuidance::Homing,
            launch.fired_at_ms.saturating_add(max_travel_ms),
        );
        let runtime = ActiveProjectileRuntime {
            delivery_id,
            cast_seq: launch.cast_seq,
            step_index: launch.step_index,
            skill_id: launch.skill_id,
            step_id: launch.step_id,
            caster_instance_id: launch.caster_instance_id,
            caster_owner: launch.caster_owner,
            spawned_at_ms: launch.fired_at_ms,
            start: launch.start,
            current_position: launch.start,
            aim: launch.aim,
            speed_units_per_ms: launch.speed_units_per_ms,
            guidance: ProjectileGuidance::Homing,
            target_unit_id: launch.target_unit_id,
            collision: launch.collision,
            hit_unit_ids: Vec::new(),
            impact_vfx_id: launch.impact_vfx_id,
            last_reevaluation_ms: None,
            next_reevaluation_ms: launch.fired_at_ms,
            max_travel_ms,
            cause: launch.cause.clone(),
        };
        self.active_projectiles.insert(delivery_id, runtime);
        self.event_queue.push(BattleEvent::SkillProjectileAdvance {
            time_ms: launch.fired_at_ms,
            delivery_id,
            cause: launch.cause,
        });
    }

    fn spawn_fixed_skill_projectile(&mut self, launch: SkillProjectileImpactLaunch) {
        launch.collision.validate_runtime_contract();
        let delivery_id =
            self.next_skill_delivery_id(launch.fired_at_ms, launch.caster_instance_id);
        let max_travel_ms =
            projectile_travel_ms(launch.start, launch.aim, launch.speed_units_per_ms);
        self.record_skill_projectile_launched(
            &launch,
            delivery_id,
            ProjectileGuidance::Fixed,
            launch.fired_at_ms.saturating_add(max_travel_ms),
        );
        let runtime = ActiveProjectileRuntime {
            delivery_id,
            cast_seq: launch.cast_seq,
            step_index: launch.step_index,
            skill_id: launch.skill_id,
            step_id: launch.step_id,
            caster_instance_id: launch.caster_instance_id,
            caster_owner: launch.caster_owner,
            spawned_at_ms: launch.fired_at_ms,
            start: launch.start,
            current_position: launch.start,
            aim: launch.aim,
            speed_units_per_ms: launch.speed_units_per_ms,
            guidance: ProjectileGuidance::Fixed,
            target_unit_id: None,
            collision: launch.collision,
            hit_unit_ids: Vec::new(),
            impact_vfx_id: launch.impact_vfx_id,
            last_reevaluation_ms: None,
            next_reevaluation_ms: launch.fired_at_ms,
            max_travel_ms,
            cause: launch.cause.clone(),
        };
        self.active_projectiles.insert(delivery_id, runtime);
        self.event_queue.push(BattleEvent::SkillProjectileAdvance {
            time_ms: launch.fired_at_ms,
            delivery_id,
            cause: launch.cause,
        });
    }

    pub(in crate::game::battle::core) fn dispatch_skill_projectile_delivery(
        &mut self,
        time_ms: u64,
        cast_seq: u64,
        step_index: usize,
        caster_instance_id: UnitInstanceId,
        skill: &SkillDef,
        step: &SkillStepDef,
        cast_target: Option<SkillCastTarget>,
        speed_units_per_ms: u32,
        collision: SkillProjectileCollisionDef,
        repeat_count: usize,
    ) -> usize {
        let mut launched_count = 0usize;
        for _ in 0..repeat_count {
            let iteration_target =
                self.resolve_skill_step_context(cast_seq, caster_instance_id, step, cast_target);
            let Some((caster_owner, caster_pos)) =
                self.resolve_cast_origin_context(Some(cast_seq), caster_instance_id, false)
            else {
                return launched_count;
            };
            let caster_origin = self
                .unit_world_position_or_tile_center(caster_instance_id)
                .unwrap_or_else(|| WorldVec2::from_tile_center(caster_pos));
            let target_pos = match iteration_target {
                Some(SkillCastTarget::Unit { unit_instance_id }) => {
                    self.battlefield.position_of(unit_instance_id)
                }
                Some(SkillCastTarget::Tile { position }) => Some(position),
                None => None,
            };
            let Some(target_pos) = target_pos else {
                continue;
            };
            let target_aim = match iteration_target {
                Some(SkillCastTarget::Unit { unit_instance_id }) => self
                    .unit_world_position_or_tile_center(unit_instance_id)
                    .unwrap_or_else(|| WorldVec2::from_tile_center(target_pos)),
                _ => WorldVec2::from_tile_center(target_pos),
            };
            let guidance = match skill.kind {
                SkillKind::Targeted
                    if matches!(
                        step.target,
                        crate::game::ability::SkillTarget::EnemySingle { .. }
                    ) && matches!(iteration_target, Some(SkillCastTarget::Unit { .. })) =>
                {
                    ProjectileGuidance::Homing
                }
                _ => ProjectileGuidance::Fixed,
            };
            let travel_time_ms = match guidance {
                ProjectileGuidance::Homing => Some(projectile_flight_ms_for_delivery(
                    (caster_origin.distance(target_aim) * LEGACY_POSITION_UNITS_PER_WORLD).ceil()
                        as u64,
                    speed_units_per_ms,
                )),
                ProjectileGuidance::Fixed => None,
            };
            let launch = SkillProjectileImpactLaunch {
                fired_at_ms: time_ms,
                cast_seq,
                step_index,
                skill_id: skill.id.clone(),
                step_id: step.id.clone(),
                caster_instance_id,
                caster_owner,
                start: caster_origin,
                aim: target_aim,
                target: iteration_target,
                speed_units_per_ms,
                target_unit_id: match iteration_target {
                    Some(SkillCastTarget::Unit { unit_instance_id }) => Some(unit_instance_id),
                    _ => None,
                },
                travel_time_ms,
                collision,
                projectile_vfx_id: step.presentation.projectile_vfx_id.clone(),
                impact_vfx_id: step.presentation.impact_vfx_id.clone(),
                cause: self.recording_cause().unwrap_or_default(),
            };
            match guidance {
                ProjectileGuidance::Fixed => self.spawn_fixed_skill_projectile(launch),
                ProjectileGuidance::Homing => self.spawn_homing_skill_projectile(launch),
            }
            launched_count = launched_count.saturating_add(1);
        }
        launched_count
    }

    pub(in crate::game::battle::core) fn apply_skill_projectile_impact(
        &mut self,
        time_ms: u64,
        delivery_id: Uuid,
        cast_seq: u64,
        step_index: usize,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        impact_position: WorldVec2,
        first_hit_unit_id: Option<UnitInstanceId>,
        impact_vfx_id: Option<String>,
        terminal: bool,
    ) {
        let Some(skill) = self.game_data.skill_data.get_by_id(&skill_id).cloned() else {
            return;
        };
        let Some(step) = Self::resolve_skill_step(&skill, step_index, &step_id) else {
            return;
        };

        let targets: Vec<UnitInstanceId> = first_hit_unit_id
            .filter(|unit_id| self.units.get(unit_id).is_some_and(|unit| !unit.is_dead()))
            .into_iter()
            .collect();

        self.update_skill_cast_impact_context(
            cast_seq,
            step_index,
            SkillImpactContext {
                delivery_id,
                impact_time_ms: time_ms,
                impact_position,
                direction_hint: None,
                first_hit_unit_id: targets.first().copied(),
                hit_unit_ids: targets.clone(),
                spawned_area_id: None,
            },
        );

        self.record_timeline(
            time_ms,
            TimelineEvent::SkillProjectileImpacted {
                delivery_id,
                skill_id: skill_id.clone(),
                step_id: step_id.clone(),
                caster_instance_id,
                first_hit_unit_id: targets.first().copied(),
                impact_position: impact_position.quantized_milli(),
                terminal,
                impact_vfx_id,
            },
        );

        let (commands, result) =
            Self::build_skill_step_commands(caster_instance_id, step, &targets);
        if !commands.is_empty() {
            let summary = self.process_commands(commands, time_ms);
            let mut resolved_result = result;
            resolved_result.actual_damage_target_count = resolved_result
                .actual_damage_target_count
                .saturating_add(summary.actual_damage_target_count);
            self.resolve_skill_step_delivery(
                cast_seq,
                step_index,
                resolved_result,
                terminal,
                time_ms,
            );
        } else {
            self.resolve_skill_step_delivery(cast_seq, step_index, result, terminal, time_ms);
        }
        self.schedule_pending_autocasts(time_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_projectile_impact_position_uses_hit_fraction_not_quantized_event_time() {
        let window_start = WorldVec2::new(0.0, 0.0);
        let window_end = WorldVec2::new(10.0, 0.0);
        let hit_fraction = 0.21;

        let event_elapsed_ms = ((10_u64 as f32) * hit_fraction).ceil() as u64;
        let quantized_event_position =
            sample_projectile_position_at(window_start, window_end, event_elapsed_ms, 10);
        let impact_position =
            projectile_impact_position_at_hit_fraction(window_start, window_end, hit_fraction);

        assert_eq!(event_elapsed_ms, 3);
        assert_eq!(quantized_event_position, WorldVec2::new(3.0, 0.0));
        assert_eq!(impact_position, WorldVec2::new(2.1, 0.0));
    }
}
