use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{
    ability::{SkillDef, SkillId, SkillKind, SkillProjectileCollisionDef, SkillStepDef},
    battle::{
        core::{
            commands::projectile_flight_ms_for_delivery,
            movement::ContinuousPosition,
            spatial::{
                circle_contains_point, euclidean_distance_units_ceil,
                DEFAULT_UNIT_HITBOX_RADIUS_UNITS,
            },
            types::{ActiveProjectileRuntime, ProjectileGuidance, SkillImpactContext},
            BattleCore,
        },
        enums::BattleEvent,
        ids::UnitInstanceId,
        timeline::{SkillCastTarget, TimelineCause},
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
    pub(in crate::game::battle::core) start: ContinuousPosition,
    pub(in crate::game::battle::core) aim: ContinuousPosition,
    pub(in crate::game::battle::core) speed_units_per_ms: u32,
    pub(in crate::game::battle::core) target_unit_id: Option<UnitInstanceId>,
    pub(in crate::game::battle::core) travel_time_ms: Option<u64>,
    pub(in crate::game::battle::core) collision: SkillProjectileCollisionDef,
    pub(in crate::game::battle::core) cause: TimelineCause,
}

fn sample_projectile_position_at(
    start: ContinuousPosition,
    aim: ContinuousPosition,
    elapsed_ms: u64,
    total_ms: u64,
) -> ContinuousPosition {
    if total_ms == 0 || elapsed_ms >= total_ms {
        return aim;
    }

    let elapsed = i128::from(elapsed_ms);
    let total = i128::from(total_ms.max(1));
    let dx = i128::from(aim.x_units) - i128::from(start.x_units);
    let dy = i128::from(aim.y_units) - i128::from(start.y_units);

    ContinuousPosition::new(
        (i128::from(start.x_units) + dx.saturating_mul(elapsed) / total)
            .clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64,
        (i128::from(start.y_units) + dy.saturating_mul(elapsed) / total)
            .clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64,
    )
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

        if runtime.guidance != ProjectileGuidance::Fixed {
            self.active_projectiles.insert(delivery_id, runtime);
            return;
        }

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
    ) -> Vec<(u64, UnitInstanceId, ContinuousPosition)> {
        if from_elapsed_ms > to_elapsed_ms {
            return Vec::new();
        }

        let reach_units =
            i64::from(runtime.collision.radius_units) + DEFAULT_UNIT_HITBOX_RADIUS_UNITS;
        let piercing = Self::skill_projectile_pierces(runtime.collision);
        let max_hits = Self::skill_projectile_hit_limit(runtime.collision);
        let mut seen_hits: HashSet<UnitInstanceId> = runtime.hit_unit_ids.iter().copied().collect();
        let mut impacts = Vec::new();

        for dt_ms in from_elapsed_ms..=to_elapsed_ms {
            let projectile_pos = sample_projectile_position_at(
                runtime.start,
                runtime.aim,
                dt_ms,
                runtime.max_travel_ms,
            );

            let mut hits: Vec<(u128, UnitInstanceId)> = self
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
                    let target_pos = self.sample_unit_position_at(
                        unit.instance_id,
                        runtime.spawned_at_ms.saturating_add(dt_ms),
                    )?;
                    if circle_contains_point(projectile_pos, reach_units, target_pos) {
                        Some((
                            super::super::spatial::distance_sq_units(projectile_pos, target_pos),
                            unit.instance_id,
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            hits.sort_by(|a, b| {
                a.0.cmp(&b.0)
                    .then_with(|| a.1.as_bytes().cmp(b.1.as_bytes()))
            });
            if !hits.is_empty() {
                for (_, hit_unit_id) in hits {
                    if seen_hits.insert(hit_unit_id) {
                        impacts.push((
                            runtime.spawned_at_ms.saturating_add(dt_ms),
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
            }
        }

        impacts
    }

    fn compute_homing_skill_projectile_impacts(
        &self,
        launch: &SkillProjectileImpactLaunch,
    ) -> Vec<(u64, Option<UnitInstanceId>, ContinuousPosition)> {
        let travel_ms = launch.travel_time_ms.unwrap_or_else(|| {
            projectile_flight_ms_for_delivery(
                euclidean_distance_units_ceil(launch.start, launch.aim),
                launch.speed_units_per_ms,
            )
        });
        let impact_time_ms = launch.fired_at_ms.saturating_add(travel_ms);

        let first_hit_unit_id = launch.target_unit_id.filter(|unit_id| {
            self.skill_delivery_accepts_unit(
                launch.caster_owner,
                launch.caster_instance_id,
                *unit_id,
                launch.collision.hit_targets,
                false,
            )
        });

        let impact_position = first_hit_unit_id
            .and_then(|unit_id| {
                self.sample_unit_position_at(unit_id, impact_time_ms)
                    .or_else(|| self.unit_continuous_position_or_tile_center(unit_id))
            })
            .or_else(|| {
                launch
                    .target_unit_id
                    .and_then(|unit_id| self.unit_continuous_position_or_tile_center(unit_id))
            })
            .unwrap_or(launch.aim);

        vec![(impact_time_ms, first_hit_unit_id, impact_position)]
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
        impact_position: ContinuousPosition,
        first_hit_unit_id: Option<UnitInstanceId>,
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
            terminal,
            cause,
        });
    }

    fn schedule_homing_skill_projectile_impact(&mut self, launch: SkillProjectileImpactLaunch) {
        let delivery_id =
            self.next_skill_delivery_id(launch.fired_at_ms, launch.caster_instance_id);
        let impacts = self.compute_homing_skill_projectile_impacts(&launch);

        for (impact_time_ms, first_hit_unit_id, impact_position) in impacts {
            self.enqueue_skill_projectile_impact(
                impact_time_ms,
                delivery_id,
                launch.cast_seq,
                launch.step_index,
                launch.skill_id.clone(),
                launch.step_id.clone(),
                launch.caster_instance_id,
                impact_position,
                first_hit_unit_id,
                true,
                launch.cause.clone(),
            );
        }
    }

    fn spawn_fixed_skill_projectile(&mut self, launch: SkillProjectileImpactLaunch) {
        launch.collision.validate_runtime_contract();
        let delivery_id =
            self.next_skill_delivery_id(launch.fired_at_ms, launch.caster_instance_id);
        let max_travel_ms = projectile_flight_ms_for_delivery(
            euclidean_distance_units_ceil(launch.start, launch.aim),
            launch.speed_units_per_ms,
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
                .unit_continuous_position_or_tile_center(caster_instance_id)
                .unwrap_or_else(|| ContinuousPosition::tile_center(caster_pos));
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
                    .unit_continuous_position_or_tile_center(unit_instance_id)
                    .unwrap_or_else(|| ContinuousPosition::tile_center(target_pos)),
                _ => ContinuousPosition::tile_center(target_pos),
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
                ProjectileGuidance::Homing => {
                    let dist_tiles = caster_pos.chebyshev(&target_pos).max(0) as u64;
                    Some(projectile_flight_ms_for_delivery(
                        dist_tiles.saturating_mul(super::super::movement::TILE_UNITS_PER_TILE),
                        speed_units_per_ms,
                    ))
                }
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
                speed_units_per_ms,
                target_unit_id: match iteration_target {
                    Some(SkillCastTarget::Unit { unit_instance_id }) => Some(unit_instance_id),
                    _ => None,
                },
                travel_time_ms,
                collision,
                cause: self.recording_cause().unwrap_or_default(),
            };
            match guidance {
                ProjectileGuidance::Fixed => self.spawn_fixed_skill_projectile(launch),
                ProjectileGuidance::Homing => {
                    launch.collision.validate_homing_runtime_contract();
                    self.schedule_homing_skill_projectile_impact(launch);
                }
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
        impact_position: ContinuousPosition,
        first_hit_unit_id: Option<UnitInstanceId>,
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
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        ecs::resources::Position,
        game::{
            battle::{
                core::{
                    movement::{ActionState, MovementState},
                    types::RuntimeUnit,
                },
                timeline::TimelineCause,
                types::PlayerDeckInfo,
            },
            data::{
                abnormality_data::AbnormalityDatabase, artifact_data::ArtifactDatabase,
                bonus_data::BonusDatabase, equipment_data::EquipmentDatabase,
                pve_data::PveEncounterDatabase, random_event_data::RandomEventDatabase,
                shop_data::ShopDatabase, skill_data::SkillDatabase, GameDataBase,
            },
            stats::UnitStats,
        },
    };

    fn empty_deck() -> PlayerDeckInfo {
        PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        }
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: crate::game::data::event_pools::EventPoolConfig {
                dawn: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                noon: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                dusk: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                midnight: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
                white: crate::game::data::event_pools::EventPhasePool {
                    shops: vec![],
                    bonuses: vec![],
                    random_events: vec![],
                },
            },
        }))
    }

    fn runtime_unit(
        unit_id: UnitInstanceId,
        owner: Side,
        position: ContinuousPosition,
        move_speed_units_per_ms: u32,
    ) -> RuntimeUnit {
        let mut stats = UnitStats::with_values(10, 100, 1, 0, 1);
        stats.move_speed_units_per_ms = move_speed_units_per_ms.max(1);
        RuntimeUnit {
            instance_id: unit_id,
            owner,
            base_uuid: Uuid::nil(),
            stats,
            pos_x_units: position.x_units,
            pos_y_units: position.y_units,
            move_epoch: 0,
            action_state: ActionState::Idle,
            action_locks: Default::default(),
            current_target: None,
            next_basic_attack_ms: 0,
            pending_basic_attack: false,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            next_action_time: 0,
            pending_cast: false,
            pending_cast_cause: None,
            pending_skill_cast: None,
        }
    }

    fn set_linear_motion(
        unit: &mut RuntimeUnit,
        from: ContinuousPosition,
        to: ContinuousPosition,
        started_at_ms: u64,
    ) {
        let mut movement = MovementState::new_at(Position::new(0, 0), started_at_ms);
        movement.last_update_ms = started_at_ms;
        movement.step_start_x_units = from.x_units;
        movement.step_start_y_units = from.y_units;
        movement.target_x_units = to.x_units;
        movement.target_y_units = to.y_units;
        movement.step_started_at_ms = started_at_ms;
        movement.step_ends_at_ms = started_at_ms;
        unit.pos_x_units = from.x_units;
        unit.pos_y_units = from.y_units;
        unit.action_state = ActionState::Moving(movement);
    }

    fn active_fixed_projectile(
        delivery_id: Uuid,
        caster_instance_id: UnitInstanceId,
        caster_owner: Side,
        start: ContinuousPosition,
        aim: ContinuousPosition,
        speed_units_per_ms: u32,
    ) -> ActiveProjectileRuntime {
        ActiveProjectileRuntime {
            delivery_id,
            cast_seq: 1,
            step_index: 0,
            skill_id: "skillshot".to_string(),
            step_id: "step".to_string(),
            caster_instance_id,
            caster_owner,
            spawned_at_ms: 0,
            start,
            current_position: start,
            aim,
            speed_units_per_ms,
            guidance: ProjectileGuidance::Fixed,
            target_unit_id: None,
            collision: SkillProjectileCollisionDef::default(),
            hit_unit_ids: Vec::new(),
            last_reevaluation_ms: None,
            next_reevaluation_ms: 0,
            max_travel_ms: projectile_flight_ms_for_delivery(
                euclidean_distance_units_ceil(start, aim),
                speed_units_per_ms,
            ),
            cause: TimelineCause::default(),
        }
    }

    fn drain_first_projectile_impact(
        core: &mut BattleCore,
    ) -> Option<(u64, Option<UnitInstanceId>, ContinuousPosition)> {
        while let Some(event) = core.event_queue.pop() {
            if let BattleEvent::SkillProjectileImpact {
                time_ms,
                first_hit_unit_id,
                impact_position,
                ..
            } = event
            {
                return Some((time_ms, first_hit_unit_id, impact_position));
            }
        }
        None
    }

    #[test]
    fn active_fixed_projectile_hits_blocker_that_enters_path_after_launch() {
        let deck = empty_deck();
        let mut core = BattleCore::new(&deck, &deck, empty_game_data(), (4, 4), 7);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xA1).into();
        let blocker_id: UnitInstanceId = Uuid::from_u128(0xA2).into();
        let backline_id: UnitInstanceId = Uuid::from_u128(0xA3).into();
        let delivery_id = Uuid::from_u128(0xA4);

        core.units.insert(
            caster_id,
            runtime_unit(
                caster_id,
                Side::Player,
                ContinuousPosition::new(0, 0),
                1_000_000,
            ),
        );
        let mut blocker = runtime_unit(
            blocker_id,
            Side::Opponent,
            ContinuousPosition::new(1_000_000, 1_000_000),
            500_000,
        );
        set_linear_motion(
            &mut blocker,
            ContinuousPosition::new(1_000_000, 1_000_000),
            ContinuousPosition::new(0, 1_000_000),
            0,
        );
        core.units.insert(blocker_id, blocker);
        core.units.insert(
            backline_id,
            runtime_unit(
                backline_id,
                Side::Opponent,
                ContinuousPosition::new(0, 2_000_000),
                1_000_000,
            ),
        );
        core.active_projectiles.insert(
            delivery_id,
            active_fixed_projectile(
                delivery_id,
                caster_id,
                Side::Player,
                ContinuousPosition::new(0, 0),
                ContinuousPosition::new(0, 2_000_000),
                500_000,
            ),
        );

        core.advance_skill_projectile(0, delivery_id);
        core.event_queue.clear();
        core.advance_skill_projectile(1, delivery_id);
        core.event_queue.clear();
        core.advance_skill_projectile(2, delivery_id);

        let mut hit_target = None;
        while let Some(event) = core.event_queue.pop() {
            if let BattleEvent::SkillProjectileImpact {
                first_hit_unit_id, ..
            } = event
            {
                hit_target = first_hit_unit_id;
                break;
            }
        }

        assert_eq!(hit_target, Some(blocker_id));
    }

    #[test]
    fn active_fixed_projectile_expires_as_miss_when_target_dies_after_launch() {
        let deck = empty_deck();
        let mut core = BattleCore::new(&deck, &deck, empty_game_data(), (4, 4), 11);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xB1).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xB2).into();
        let delivery_id = Uuid::from_u128(0xB3);

        core.units.insert(
            caster_id,
            runtime_unit(
                caster_id,
                Side::Player,
                ContinuousPosition::new(0, 0),
                1_000_000,
            ),
        );
        core.units.insert(
            target_id,
            runtime_unit(
                target_id,
                Side::Opponent,
                ContinuousPosition::new(0, 2_000_000),
                1_000_000,
            ),
        );
        core.active_projectiles.insert(
            delivery_id,
            active_fixed_projectile(
                delivery_id,
                caster_id,
                Side::Player,
                ContinuousPosition::new(0, 0),
                ContinuousPosition::new(0, 2_000_000),
                500_000,
            ),
        );

        core.advance_skill_projectile(0, delivery_id);
        core.event_queue.clear();
        core.units.get_mut(&target_id).unwrap().stats.current_health = 0;
        core.advance_skill_projectile(4, delivery_id);

        let impact = drain_first_projectile_impact(&mut core)
            .map(|(time_ms, first_hit_unit_id, _)| (time_ms, first_hit_unit_id));

        assert_eq!(impact, Some((4, None)));
    }

    #[test]
    fn active_fixed_projectile_still_hits_later_unit_after_original_target_dies() {
        let deck = empty_deck();
        let mut core = BattleCore::new(&deck, &deck, empty_game_data(), (4, 4), 13);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xC1).into();
        let original_target_id: UnitInstanceId = Uuid::from_u128(0xC2).into();
        let later_unit_id: UnitInstanceId = Uuid::from_u128(0xC3).into();
        let delivery_id = Uuid::from_u128(0xC4);

        core.units.insert(
            caster_id,
            runtime_unit(
                caster_id,
                Side::Player,
                ContinuousPosition::new(0, 0),
                1_000_000,
            ),
        );
        core.units.insert(
            original_target_id,
            runtime_unit(
                original_target_id,
                Side::Opponent,
                ContinuousPosition::new(0, 2_000_000),
                1_000_000,
            ),
        );
        core.units.insert(
            later_unit_id,
            runtime_unit(
                later_unit_id,
                Side::Opponent,
                ContinuousPosition::new(0, 2_400_000),
                1_000_000,
            ),
        );

        let mut projectile = active_fixed_projectile(
            delivery_id,
            caster_id,
            Side::Player,
            ContinuousPosition::new(0, 0),
            ContinuousPosition::new(0, 2_000_000),
            500_000,
        );
        projectile.target_unit_id = Some(original_target_id);
        projectile.collision.radius_units = 500_000;
        core.active_projectiles.insert(delivery_id, projectile);

        for time_ms in 0..=3 {
            core.advance_skill_projectile(time_ms, delivery_id);
            core.event_queue.clear();
            if time_ms == 1 {
                core.units
                    .get_mut(&original_target_id)
                    .unwrap()
                    .stats
                    .current_health = 0;
            }
        }

        core.advance_skill_projectile(4, delivery_id);
        let impact = drain_first_projectile_impact(&mut core)
            .map(|(time_ms, first_hit_unit_id, _)| (time_ms, first_hit_unit_id));

        assert_eq!(impact, Some((4, Some(later_unit_id))));
    }

    #[test]
    fn active_fixed_projectile_hits_target_sooner_when_cc_stops_its_motion() {
        let deck = empty_deck();
        let mut core = BattleCore::new(&deck, &deck, empty_game_data(), (4, 4), 17);
        let caster_id: UnitInstanceId = Uuid::from_u128(0xD1).into();
        let target_id: UnitInstanceId = Uuid::from_u128(0xD2).into();
        let delivery_id = Uuid::from_u128(0xD3);

        core.units.insert(
            caster_id,
            runtime_unit(
                caster_id,
                Side::Player,
                ContinuousPosition::new(0, 0),
                1_000_000,
            ),
        );
        let mut moving_target = runtime_unit(
            target_id,
            Side::Opponent,
            ContinuousPosition::new(250_000, 2_000_000),
            250_000,
        );
        set_linear_motion(
            &mut moving_target,
            ContinuousPosition::new(250_000, 2_000_000),
            ContinuousPosition::new(2_500_000, 2_000_000),
            0,
        );
        core.units.insert(target_id, moving_target);

        let mut projectile = active_fixed_projectile(
            delivery_id,
            caster_id,
            Side::Player,
            ContinuousPosition::new(0, 0),
            ContinuousPosition::new(0, 4_000_000),
            1_000_000,
        );
        projectile.collision.radius_units = 250_000;
        core.active_projectiles.insert(delivery_id, projectile);

        core.advance_skill_projectile(0, delivery_id);
        core.event_queue.clear();
        core.advance_skill_projectile(1, delivery_id);
        core.event_queue.clear();

        let stopped_position = core.sample_unit_position_at(target_id, 1).unwrap();
        let target = core.units.get_mut(&target_id).unwrap();
        target.pos_x_units = stopped_position.x_units;
        target.pos_y_units = stopped_position.y_units;
        target.action_state = ActionState::Idle;

        core.advance_skill_projectile(2, delivery_id);
        let impact = drain_first_projectile_impact(&mut core)
            .map(|(time_ms, first_hit_unit_id, _)| (time_ms, first_hit_unit_id));

        assert_eq!(impact, Some((2, Some(target_id))));
    }
}
