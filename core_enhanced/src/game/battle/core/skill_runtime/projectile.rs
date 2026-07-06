use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{
    ability::{
        DeliveryDef, ProjectileHitPolicy, SkillDef, SkillId, SkillProjectileCollisionDef,
        SkillStepDef,
    },
    battle::{
        core::{
            movement::types::{WorldVec2, DATA_UNITS_PER_WORLD, WORLD_UNITS_PER_TILE},
            projectile_math::projectile_flight_ms,
            spatial::{data_units_to_world, moving_circle_sweep_hit_fraction},
            types::{ActiveProjectileRuntime, ProjectileGuidance, SkillImpactContext},
            BattleCore,
        },
        damage::{DamageModifiers, DamageSource, DamageSourceSnapshot, DamageType},
        enums::BattleEvent,
        event_log::{BattleEventCause, BattleLogEvent, BattleProjectileGuidance, SkillCastTarget},
        ids::UnitInstanceId,
        tile_range::FacingDirection,
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
    pub(in crate::game::battle::core) source_snapshot: DamageSourceSnapshot,
    pub(in crate::game::battle::core) start: WorldVec2,
    pub(in crate::game::battle::core) aim: WorldVec2,
    pub(in crate::game::battle::core) target: Option<SkillCastTarget>,
    pub(in crate::game::battle::core) speed_units_per_ms: u32,
    pub(in crate::game::battle::core) target_unit_id: Option<UnitInstanceId>,
    pub(in crate::game::battle::core) travel_time_ms: Option<u64>,
    pub(in crate::game::battle::core) collision: SkillProjectileCollisionDef,
    pub(in crate::game::battle::core) max_kills: Option<u32>,
    pub(in crate::game::battle::core) projectile_vfx_id: Option<String>,
    pub(in crate::game::battle::core) impact_vfx_id: Option<String>,
    pub(in crate::game::battle::core) cause: BattleEventCause,
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
    let distance_units = (start.distance(aim) * DATA_UNITS_PER_WORLD).ceil() as u64;
    projectile_flight_ms(distance_units, speed_units_per_ms)
}

fn projectile_impact_position_at_hit_fraction(
    window_start: WorldVec2,
    window_end: WorldVec2,
    hit_fraction: f32,
) -> WorldVec2 {
    window_start + (window_end - window_start) * hit_fraction.clamp(0.0, 1.0)
}

fn event_log_projectile_guidance(guidance: ProjectileGuidance) -> BattleProjectileGuidance {
    match guidance {
        ProjectileGuidance::Homing => BattleProjectileGuidance::Homing,
        ProjectileGuidance::Fixed => BattleProjectileGuidance::Fixed,
    }
}

fn facing_direction_vector(facing: FacingDirection) -> WorldVec2 {
    match facing {
        FacingDirection::Up => WorldVec2::new(0.0, -1.0),
        FacingDirection::Right => WorldVec2::new(1.0, 0.0),
        FacingDirection::Down => WorldVec2::new(0.0, 1.0),
        FacingDirection::Left => WorldVec2::new(-1.0, 0.0),
    }
}

fn directional_projectile_max_distance_world(
    speed_units_per_ms: u32,
    max_range_tiles: Option<u32>,
    max_lifetime_ms: Option<u64>,
) -> Option<f32> {
    let range_distance = max_range_tiles.map(|tiles| tiles as f32 * WORLD_UNITS_PER_TILE);
    let lifetime_distance =
        max_lifetime_ms.map(|ms| (speed_units_per_ms as f32 / DATA_UNITS_PER_WORLD) * ms as f32);

    match (range_distance, lifetime_distance) {
        (Some(range), Some(lifetime)) => Some(range.min(lifetime)),
        (Some(range), None) => Some(range),
        (None, Some(lifetime)) => Some(lifetime),
        (None, None) => None,
    }
}

fn directional_projectile_travel_ms(
    start: WorldVec2,
    aim: WorldVec2,
    speed_units_per_ms: u32,
    max_lifetime_ms: Option<u64>,
) -> u64 {
    let range_travel_ms = projectile_travel_ms(start, aim, speed_units_per_ms);
    max_lifetime_ms.map_or(range_travel_ms, |lifetime| lifetime.min(range_travel_ms))
}

fn apply_projectile_stop_policy_to_collision(
    mut collision: SkillProjectileCollisionDef,
    max_pierces: Option<u32>,
) -> SkillProjectileCollisionDef {
    let Some(max_pierces) = max_pierces else {
        return collision;
    };
    let max_hits_from_pierces = max_pierces.saturating_add(1).min(u8::MAX as u32) as u8;
    collision.max_hits = Some(match collision.max_hits {
        Some(existing) => existing.min(max_hits_from_pierces),
        None => max_hits_from_pierces,
    });
    collision
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
        let mut impacts = self.collect_fixed_skill_projectile_hits_in_window(
            &runtime,
            from_elapsed_ms,
            to_elapsed_ms,
        );
        if runtime.max_kills.is_some() && impacts.len() > 1 {
            impacts.truncate(1);
        }

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
                runtime.source_snapshot.clone(),
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
                runtime.source_snapshot.clone(),
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
                runtime.source_snapshot.clone(),
                runtime.current_position,
                None,
                runtime.impact_vfx_id,
                true,
                runtime.cause,
            );
            return;
        };

        let target_alive = self.units.get(&target_unit_id).is_some_and(|unit| {
            unit.is_active()
                && self.skill_delivery_accepts_unit(
                    runtime.caster_owner,
                    runtime.caster_instance_id,
                    target_unit_id,
                    runtime.collision.hit_targets,
                    false,
                )
        });
        let from_time_ms = runtime
            .last_reevaluation_ms
            .unwrap_or(runtime.spawned_at_ms);
        let Some(target_body_start) = target_alive
            .then(|| self.sample_unit_body_at(target_unit_id, from_time_ms))
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
                runtime.source_snapshot.clone(),
                runtime.current_position,
                None,
                runtime.impact_vfx_id,
                true,
                runtime.cause,
            );
            return;
        };
        let Some(target_body_end) = target_alive
            .then(|| self.sample_unit_body_at(target_unit_id, reevaluation_time_ms))
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
                runtime.source_snapshot.clone(),
                runtime.current_position,
                None,
                runtime.impact_vfx_id,
                true,
                runtime.cause,
            );
            return;
        };

        let window_duration_ms = reevaluation_time_ms.saturating_sub(from_time_ms);
        let aim = target_body_end.position;
        let direction = aim - runtime.current_position;
        let distance = direction.length();
        let speed_world_per_ms = runtime.speed_units_per_ms as f32 / DATA_UNITS_PER_WORLD;
        let max_step = speed_world_per_ms * window_duration_ms as f32;
        let next_position = if distance <= f32::EPSILON
            || runtime.speed_units_per_ms == 0
            || max_step >= distance
        {
            aim
        } else {
            runtime.current_position + direction * (max_step / distance)
        };
        let reach = data_units_to_world(i64::from(runtime.collision.radius_units))
            + target_body_start
                .radius
                .max(target_body_end.radius)
                .max(0.0);

        if let Some(hit_fraction) = moving_circle_sweep_hit_fraction(
            runtime.current_position,
            next_position,
            reach,
            target_body_start.position,
            target_body_end.position,
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
                runtime.source_snapshot.clone(),
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
                runtime.source_snapshot.clone(),
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

        let projectile_radius = data_units_to_world(i64::from(runtime.collision.radius_units));
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
                let target_body_start = self.sample_unit_body_at(
                    unit.instance_id,
                    runtime.spawned_at_ms.saturating_add(from_elapsed_ms),
                )?;
                let target_body_end = self.sample_unit_body_at(
                    unit.instance_id,
                    runtime.spawned_at_ms.saturating_add(to_elapsed_ms),
                )?;
                let reach = projectile_radius
                    + target_body_start
                        .radius
                        .max(target_body_end.radius)
                        .max(0.0);
                let hit_fraction = moving_circle_sweep_hit_fraction(
                    window_start,
                    window_end,
                    reach,
                    target_body_start.position,
                    target_body_end.position,
                )?;
                let elapsed_delta = ((window_duration_ms as f32) * hit_fraction).ceil() as u64;
                let hit_elapsed_ms =
                    from_elapsed_ms.saturating_add(elapsed_delta.min(window_duration_ms));
                let projectile_pos = projectile_impact_position_at_hit_fraction(
                    window_start,
                    window_end,
                    hit_fraction,
                );
                let target_pos = projectile_impact_position_at_hit_fraction(
                    target_body_start.position,
                    target_body_end.position,
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
        source_snapshot: DamageSourceSnapshot,
        impact_position: WorldVec2,
        first_hit_unit_id: Option<UnitInstanceId>,
        impact_vfx_id: Option<String>,
        terminal: bool,
        cause: BattleEventCause,
    ) {
        self.event_queue.push(BattleEvent::SkillProjectileImpact {
            time_ms,
            delivery_id,
            cast_seq,
            step_index,
            skill_id,
            step_id,
            caster_instance_id,
            source_snapshot,
            impact_position,
            first_hit_unit_id,
            impact_vfx_id,
            terminal,
            cause,
        });
    }

    fn apply_skill_projectile_kill_stop_policy(
        &mut self,
        delivery_id: Uuid,
        killed_target_count: usize,
    ) {
        if killed_target_count == 0 {
            return;
        }
        let Some(runtime) = self.active_projectiles.get_mut(&delivery_id) else {
            return;
        };
        let Some(max_kills) = runtime.max_kills else {
            return;
        };
        runtime.killed_unit_count = runtime
            .killed_unit_count
            .saturating_add(killed_target_count as u32);
        if runtime.killed_unit_count >= max_kills {
            self.active_projectiles.remove(&delivery_id);
        }
    }

    fn record_skill_projectile_launched(
        &mut self,
        launch: &SkillProjectileImpactLaunch,
        delivery_id: Uuid,
        guidance: ProjectileGuidance,
        expected_end_time_ms: u64,
    ) {
        self.with_recording_context(launch.cause, |core| {
            core.record_event_log(
                launch.fired_at_ms,
                BattleLogEvent::SkillProjectileLaunched {
                    delivery_id,
                    skill_id: launch.skill_id.clone(),
                    step_id: launch.step_id.clone(),
                    caster_instance_id: launch.caster_instance_id,
                    target: launch.target,
                    guidance: event_log_projectile_guidance(guidance),
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
            source_snapshot: launch.source_snapshot.clone(),
            spawned_at_ms: launch.fired_at_ms,
            start: launch.start,
            current_position: launch.start,
            aim: launch.aim,
            speed_units_per_ms: launch.speed_units_per_ms,
            guidance: ProjectileGuidance::Homing,
            target_unit_id: launch.target_unit_id,
            collision: launch.collision,
            hit_unit_ids: Vec::new(),
            killed_unit_count: 0,
            max_kills: launch.max_kills,
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
            source_snapshot: launch.source_snapshot.clone(),
            spawned_at_ms: launch.fired_at_ms,
            start: launch.start,
            current_position: launch.start,
            aim: launch.aim,
            speed_units_per_ms: launch.speed_units_per_ms,
            guidance: ProjectileGuidance::Fixed,
            target_unit_id: None,
            collision: launch.collision,
            hit_unit_ids: Vec::new(),
            killed_unit_count: 0,
            max_kills: launch.max_kills,
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
        delivery: &DeliveryDef,
        repeat_count: usize,
    ) -> usize {
        let DeliveryDef::Projectile {
            speed_units_per_ms,
            hit_policy,
            allow_targetless_cast,
            max_range_tiles,
            max_lifetime_ms,
            max_kills,
            max_pierces,
            collision,
        } = delivery
        else {
            return 0;
        };

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
            let Some(source_snapshot) = self.damage_source_snapshot_template_for_unit(
                caster_instance_id,
                DamageSource::Ability,
                DamageType::default(),
                0,
                DamageModifiers::default(),
                0,
                time_ms,
                false,
            ) else {
                continue;
            };

            let mut projectile_collision =
                apply_projectile_stop_policy_to_collision(*collision, *max_pierces);
            let (target_aim, guidance, target_unit_id, travel_time_ms) = match hit_policy {
                ProjectileHitPolicy::TargetLocked => {
                    let Some(SkillCastTarget::Unit { unit_instance_id }) = iteration_target else {
                        continue;
                    };
                    let Some(target_pos) = self.live_unit_projected_tile(unit_instance_id) else {
                        continue;
                    };
                    let target_aim = self
                        .unit_world_position_or_tile_center(unit_instance_id)
                        .unwrap_or_else(|| WorldVec2::from_tile_center(target_pos));
                    let travel_time_ms = projectile_flight_ms(
                        (caster_origin.distance(target_aim) * DATA_UNITS_PER_WORLD).ceil() as u64,
                        *speed_units_per_ms,
                    );
                    (
                        target_aim,
                        ProjectileGuidance::Homing,
                        Some(unit_instance_id),
                        Some(travel_time_ms),
                    )
                }
                ProjectileHitPolicy::DirectionalCollision => {
                    if iteration_target.is_none() && !*allow_targetless_cast {
                        continue;
                    }
                    let Some(facing) = self
                        .units
                        .get(&caster_instance_id)
                        .and_then(|unit| unit.facing_direction)
                    else {
                        continue;
                    };
                    let Some(distance_world) = directional_projectile_max_distance_world(
                        *speed_units_per_ms,
                        *max_range_tiles,
                        *max_lifetime_ms,
                    ) else {
                        continue;
                    };
                    let direction = facing_direction_vector(facing);
                    let target_aim = caster_origin + direction * distance_world;
                    let travel_time_ms = directional_projectile_travel_ms(
                        caster_origin,
                        target_aim,
                        *speed_units_per_ms,
                        *max_lifetime_ms,
                    );
                    (
                        target_aim,
                        ProjectileGuidance::Fixed,
                        None,
                        Some(travel_time_ms),
                    )
                }
            };
            if !Self::skill_projectile_pierces(projectile_collision) && max_pierces.is_some() {
                projectile_collision.piercing = true;
            }
            let launch = SkillProjectileImpactLaunch {
                fired_at_ms: time_ms,
                cast_seq,
                step_index,
                skill_id: skill.id.clone(),
                step_id: step.id.clone(),
                caster_instance_id,
                caster_owner,
                source_snapshot,
                start: caster_origin,
                aim: target_aim,
                target: iteration_target,
                speed_units_per_ms: *speed_units_per_ms,
                target_unit_id,
                travel_time_ms,
                collision: projectile_collision,
                max_kills: *max_kills,
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
        source_snapshot: DamageSourceSnapshot,
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
            .filter(|unit_id| self.units.get(unit_id).is_some_and(|unit| unit.is_active()))
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

        self.record_event_log(
            time_ms,
            BattleLogEvent::SkillProjectileImpacted {
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

        let (commands, result) = self.build_skill_step_commands(
            caster_instance_id,
            step,
            &targets,
            source_snapshot.committed_at_ms,
            Some(&source_snapshot),
        );
        let signal_result =
            self.record_skill_step_live_signals(caster_instance_id, &skill_id, step, &targets);
        let mut resolved_result = result;
        resolved_result.merge(&signal_result);
        if !commands.is_empty() {
            let summary = self.process_commands(commands, time_ms);
            resolved_result.actual_damage_target_count = resolved_result
                .actual_damage_target_count
                .saturating_add(summary.actual_damage_target_count);
            self.apply_skill_projectile_kill_stop_policy(delivery_id, summary.killed_target_count);
            self.resolve_skill_step_delivery(
                cast_seq,
                step_index,
                resolved_result,
                terminal,
                time_ms,
            );
        } else {
            self.resolve_skill_step_delivery(
                cast_seq,
                step_index,
                resolved_result,
                terminal,
                time_ms,
            );
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
