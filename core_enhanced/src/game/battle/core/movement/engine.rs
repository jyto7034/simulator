use std::collections::HashMap;

use crate::{
    ecs::resources::Position,
    game::{
        battle::{
            core::{ActiveMovementSegment, BattleCore},
            ids::UnitInstanceId,
            timeline::TimelineEvent,
        },
        enums::Side,
    },
};

use super::{
    rapier_backend::RapierMovementWorld,
    steering::{self, SteeringParams},
    types::{MovementGoal, TimelineVec2, UnitBody, WorldVec2},
    MovementSegmentEndKind,
};

const SEPARATION_SOLVER_ITERATIONS: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub struct MovementUnitInput {
    pub unit_id: UnitInstanceId,
    pub owner: Side,
    pub body: UnitBody,
    pub current_target: Option<UnitInstanceId>,
    pub attack_range_units: f32,
    pub can_move: bool,
    pub is_dead: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementStaticObstacle {
    pub obstacle_id: u64,
    pub center: WorldVec2,
    pub half_extents: WorldVec2,
}

impl MovementStaticObstacle {
    pub fn tile(tile: Position) -> Self {
        let center = WorldVec2::from_tile_center(tile);
        Self {
            obstacle_id: ((tile.x as u64) << 32) ^ (tile.y as u32 as u64),
            center,
            half_extents: WorldVec2::new(0.5, 0.5),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovementTickInput {
    pub now_ms: u64,
    pub dt_ms: u64,
    pub board_width_units: f32,
    pub board_height_units: f32,
    pub units: Vec<MovementUnitInput>,
    pub static_obstacles: Vec<MovementStaticObstacle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementStopReasonContinuous {
    TargetReached,
    MovementLocked,
    Dead,
    NoGoal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementSegmentReason {
    GoalStarted,
    GoalChanged,
    SteeringChanged,
    CollisionAdjusted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementOutput {
    BodyMoved {
        unit_id: UnitInstanceId,
        from: WorldVec2,
        to: WorldVec2,
        velocity: WorldVec2,
    },
    GoalAssigned {
        unit_id: UnitInstanceId,
        goal: MovementGoal,
    },
    SegmentStarted {
        unit_id: UnitInstanceId,
        start: WorldVec2,
        target: WorldVec2,
        velocity: WorldVec2,
        reason: MovementSegmentReason,
    },
    MovementStopped {
        unit_id: UnitInstanceId,
        position: WorldVec2,
        reason: MovementStopReasonContinuous,
    },
    TargetReached {
        unit_id: UnitInstanceId,
        target_id: UnitInstanceId,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MovementTickResult {
    pub outputs: Vec<MovementOutput>,
}

pub trait MovementEngine {
    fn tick(&mut self, input: MovementTickInput) -> MovementTickResult;
}

pub enum ContinuousMovementBackend {
    Direct(DirectContinuousMovement),
    Rapier(RapierMovementWorld),
}

impl Default for ContinuousMovementBackend {
    fn default() -> Self {
        Self::Rapier(RapierMovementWorld::default())
    }
}

impl MovementEngine for ContinuousMovementBackend {
    fn tick(&mut self, input: MovementTickInput) -> MovementTickResult {
        match self {
            Self::Direct(engine) => engine.tick(input),
            Self::Rapier(engine) => engine.tick(input),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectContinuousMovement {
    pub separation_radius_multiplier: f32,
}

impl Default for DirectContinuousMovement {
    fn default() -> Self {
        Self {
            separation_radius_multiplier: 1.0,
        }
    }
}

impl DirectContinuousMovement {
    pub(super) fn clamp_to_board(
        position: WorldVec2,
        radius: f32,
        board_width: f32,
        board_height: f32,
    ) -> WorldVec2 {
        let min_x = radius.max(0.0);
        let min_y = radius.max(0.0);
        let max_x = (board_width - radius).max(min_x);
        let max_y = (board_height - radius).max(min_y);

        WorldVec2::new(
            position.x.clamp(min_x, max_x),
            position.y.clamp(min_y, max_y),
        )
    }

    fn fallback_separation_normal(a: UnitInstanceId, b: UnitInstanceId) -> WorldVec2 {
        if a.as_bytes() <= b.as_bytes() {
            WorldVec2::new(-1.0, 0.0)
        } else {
            WorldVec2::new(1.0, 0.0)
        }
    }

    fn separation_normal(
        a: UnitInstanceId,
        a_pos: WorldVec2,
        b: UnitInstanceId,
        b_pos: WorldVec2,
    ) -> WorldVec2 {
        let delta = a_pos - b_pos;
        if delta.length_squared() <= f32::EPSILON {
            Self::fallback_separation_normal(a, b)
        } else {
            delta.normalized_or_zero()
        }
    }

    fn apply_separation(
        candidates: &mut [MovementCandidate],
        units: &[MovementUnitInput],
        radius_multiplier: f32,
        board_width: f32,
        board_height: f32,
    ) {
        let radius_multiplier = radius_multiplier.max(0.0);
        if radius_multiplier <= f32::EPSILON {
            return;
        }

        let moving_unit_ids: Vec<UnitInstanceId> = candidates
            .iter()
            .map(|candidate| candidate.unit_id)
            .collect();

        for _ in 0..SEPARATION_SOLVER_ITERATIONS {
            for i in 0..candidates.len() {
                for j in (i + 1)..candidates.len() {
                    let min_distance =
                        (candidates[i].radius + candidates[j].radius) * radius_multiplier;
                    let delta = candidates[i].to - candidates[j].to;
                    let distance = delta.length();
                    if distance >= min_distance {
                        continue;
                    }

                    let normal = if distance <= f32::EPSILON {
                        Self::fallback_separation_normal(
                            candidates[i].unit_id,
                            candidates[j].unit_id,
                        )
                    } else {
                        delta.normalized_or_zero()
                    };
                    let correction = (min_distance - distance) * 0.5;
                    candidates[i].to += normal * correction;
                    candidates[j].to += normal * -correction;
                    candidates[i].to = Self::clamp_to_board(
                        candidates[i].to,
                        candidates[i].radius,
                        board_width,
                        board_height,
                    );
                    candidates[j].to = Self::clamp_to_board(
                        candidates[j].to,
                        candidates[j].radius,
                        board_width,
                        board_height,
                    );
                }
            }

            for candidate in candidates.iter_mut() {
                for unit in units {
                    if unit.unit_id == candidate.unit_id {
                        continue;
                    }
                    if moving_unit_ids.contains(&unit.unit_id) {
                        continue;
                    }

                    let min_distance = (candidate.radius + unit.body.radius) * radius_multiplier;
                    let distance = candidate.to.distance(unit.body.position);
                    if distance >= min_distance {
                        continue;
                    }

                    let normal = Self::separation_normal(
                        candidate.unit_id,
                        candidate.to,
                        unit.unit_id,
                        unit.body.position,
                    );
                    candidate.to += normal * (min_distance - distance);
                    candidate.to = Self::clamp_to_board(
                        candidate.to,
                        candidate.radius,
                        board_width,
                        board_height,
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MovementCandidate {
    unit_id: UnitInstanceId,
    from: WorldVec2,
    to: WorldVec2,
    radius: f32,
}

impl MovementEngine for DirectContinuousMovement {
    fn tick(&mut self, input: MovementTickInput) -> MovementTickResult {
        let mut outputs = Vec::new();
        let mut candidates = Vec::new();
        let dt_seconds = input.dt_ms as f32 / 1_000.0;

        for unit in &input.units {
            if unit.is_dead {
                outputs.push(MovementOutput::MovementStopped {
                    unit_id: unit.unit_id,
                    position: unit.body.position,
                    reason: MovementStopReasonContinuous::Dead,
                });
                continue;
            }

            if !unit.can_move {
                outputs.push(MovementOutput::MovementStopped {
                    unit_id: unit.unit_id,
                    position: unit.body.position,
                    reason: MovementStopReasonContinuous::MovementLocked,
                });
                continue;
            }

            if let Some(reached) = steering::reached_goal(unit, &input.units) {
                outputs.push(reached);
                continue;
            }

            let Some(target) = steering::goal_target_position(unit, &input.units) else {
                outputs.push(MovementOutput::MovementStopped {
                    unit_id: unit.unit_id,
                    position: unit.body.position,
                    reason: MovementStopReasonContinuous::NoGoal,
                });
                continue;
            };

            let displacement = steering::steered_displacement(
                unit,
                &input.units,
                target,
                dt_seconds,
                SteeringParams::default(),
            );
            let to = Self::clamp_to_board(
                unit.body.position + displacement,
                unit.body.radius,
                input.board_width_units,
                input.board_height_units,
            );

            candidates.push(MovementCandidate {
                unit_id: unit.unit_id,
                from: unit.body.position,
                to,
                radius: unit.body.radius,
            });
        }

        Self::apply_separation(
            &mut candidates,
            &input.units,
            self.separation_radius_multiplier,
            input.board_width_units,
            input.board_height_units,
        );

        for candidate in candidates {
            let velocity = steering::movement_velocity(candidate.from, candidate.to, dt_seconds);
            if (candidate.to - candidate.from).length_squared() <= f32::EPSILON {
                continue;
            }
            outputs.push(MovementOutput::BodyMoved {
                unit_id: candidate.unit_id,
                from: candidate.from,
                to: candidate.to,
                velocity,
            });
        }

        MovementTickResult { outputs }
    }
}

impl BattleCore {
    pub fn use_direct_continuous_movement_backend(&mut self) {
        self.movement_backend =
            ContinuousMovementBackend::Direct(DirectContinuousMovement::default());
    }

    pub fn use_rapier_continuous_movement_backend(&mut self) {
        self.movement_backend = ContinuousMovementBackend::Rapier(RapierMovementWorld::default());
    }

    fn record_or_extend_continuous_movement_segment(
        &mut self,
        time_ms: u64,
        dt_ms: u64,
        unit_id: UnitInstanceId,
        from: WorldVec2,
        to: WorldVec2,
        velocity: TimelineVec2,
    ) {
        let ends_at_ms = time_ms.saturating_add(dt_ms);
        let target = to.quantized_milli();

        if let Some(active) = self.active_movement_segments.get(&unit_id).copied() {
            if active.velocity == velocity {
                if let Some(entry) = self.timeline.entries.get_mut(active.timeline_index) {
                    if let TimelineEvent::MovementSegmentStarted {
                        unit_instance_id,
                        target: segment_target,
                        ends_at_ms: segment_ends_at_ms,
                        ..
                    } = &mut entry.event
                    {
                        if *unit_instance_id == unit_id && *segment_ends_at_ms == time_ms {
                            *segment_target = target;
                            *segment_ends_at_ms = ends_at_ms;
                            return;
                        }
                    }
                }
            }
        }

        let timeline_index = self.timeline.entries.len();
        self.record_timeline(
            time_ms,
            TimelineEvent::MovementSegmentStarted {
                unit_instance_id: unit_id,
                start: from.quantized_milli(),
                target,
                started_at_ms: time_ms,
                ends_at_ms,
                end_kind: MovementSegmentEndKind::Boundary,
            },
        );
        self.active_movement_segments.insert(
            unit_id,
            ActiveMovementSegment {
                timeline_index,
                velocity,
            },
        );
    }

    fn stop_continuous_movement_segment(&mut self, unit_id: UnitInstanceId) {
        self.active_movement_segments.remove(&unit_id);
    }

    pub fn run_continuous_attack_movement_tick(
        &mut self,
        now_ms: u64,
        dt_ms: u64,
    ) -> MovementTickResult {
        let goals = self.build_continuous_attack_goals();
        self.run_continuous_movement_tick(now_ms, dt_ms, &goals)
    }

    pub fn build_continuous_movement_input(&self, now_ms: u64, dt_ms: u64) -> MovementTickInput {
        self.build_continuous_movement_input_with_goals(now_ms, dt_ms, &HashMap::new())
    }

    pub fn build_continuous_movement_input_with_goals(
        &self,
        now_ms: u64,
        dt_ms: u64,
        goals: &HashMap<UnitInstanceId, MovementGoal>,
    ) -> MovementTickInput {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let units = unit_ids
            .into_iter()
            .filter_map(|unit_id| {
                let unit = self.units.get(&unit_id)?;
                let mut body = unit.movement_body_view();
                body.goal = goals.get(&unit_id).copied();
                Some(MovementUnitInput {
                    unit_id,
                    owner: unit.owner,
                    body,
                    current_target: unit.current_target,
                    attack_range_units: self.basic_attack_range_units(unit.base_uuid),
                    can_move: unit.action_locks.can_move(now_ms)
                        && unit.stats.move_speed_units_per_ms > 0,
                    is_dead: unit.is_dead(),
                })
            })
            .collect();

        MovementTickInput {
            now_ms,
            dt_ms,
            board_width_units: f32::from(self.battlefield.width()),
            board_height_units: f32::from(self.battlefield.height()),
            units,
            static_obstacles: self.continuous_static_obstacles(),
        }
    }

    fn continuous_static_obstacles(&self) -> Vec<MovementStaticObstacle> {
        self.battlefield
            .static_obstacles()
            .into_iter()
            .map(MovementStaticObstacle::tile)
            .collect()
    }

    pub fn apply_continuous_movement_outputs(
        &mut self,
        time_ms: u64,
        dt_ms: u64,
        outputs: &[MovementOutput],
    ) {
        for output in outputs {
            match *output {
                MovementOutput::BodyMoved {
                    unit_id,
                    from,
                    to,
                    velocity,
                } => {
                    if let Some(unit) = self.units.get(&unit_id) {
                        let mut body = unit.movement_body_view();
                        body.previous_position = from;
                        body.position = to;
                        body.velocity = velocity;
                        self.apply_unit_body_position(unit_id, &body);
                        self.record_or_extend_continuous_movement_segment(
                            time_ms,
                            dt_ms,
                            unit_id,
                            from,
                            to,
                            velocity.quantized_milli(),
                        );
                    }
                }
                MovementOutput::TargetReached { unit_id, target_id } => {
                    self.stop_continuous_movement_segment(unit_id);
                    if let Some(unit) = self.units.get_mut(&unit_id) {
                        unit.current_target = Some(target_id);
                    }
                }
                MovementOutput::MovementStopped { unit_id, .. } => {
                    self.stop_continuous_movement_segment(unit_id);
                }
                MovementOutput::GoalAssigned { .. } | MovementOutput::SegmentStarted { .. } => {}
            }
        }
    }

    pub fn run_continuous_movement_tick(
        &mut self,
        now_ms: u64,
        dt_ms: u64,
        goals: &HashMap<UnitInstanceId, MovementGoal>,
    ) -> MovementTickResult {
        let input = self.build_continuous_movement_input_with_goals(now_ms, dt_ms, goals);
        let result = self.movement_backend.tick(input);
        self.apply_continuous_movement_outputs(now_ms, dt_ms, &result.outputs);
        result
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use crate::game::{
        battle::{
            core::{movement::ActionState, types::RuntimeUnit, BattleCore},
            types::PlayerDeckInfo,
        },
        data::{
            abnormality_data::AbnormalityDatabase,
            artifact_data::ArtifactDatabase,
            bonus_data::BonusDatabase,
            equipment_data::EquipmentDatabase,
            event_pools::{EventPhasePool, EventPoolConfig},
            pve_data::PveEncounterDatabase,
            random_event_data::RandomEventDatabase,
            shop_data::ShopDatabase,
            skill_data::SkillDatabase,
            GameDataBase,
        },
        enums::Side,
        stats::UnitStats,
    };

    use super::*;
    use crate::game::battle::core::movement::types::{MovementMode, UnitBody, DEFAULT_UNIT_RADIUS};

    fn unit(unit_id: u128, position: WorldVec2) -> MovementUnitInput {
        MovementUnitInput {
            unit_id: Uuid::from_u128(unit_id).into(),
            owner: Side::Player,
            body: UnitBody {
                position,
                previous_position: position,
                velocity: WorldVec2::ZERO,
                radius: 0.35,
                move_speed: 1.0,
                mode: MovementMode::Moving,
                goal: None,
                physics_handle: None,
            },
            current_target: None,
            attack_range_units: 1.0,
            can_move: true,
            is_dead: false,
        }
    }

    fn empty_deck() -> PlayerDeckInfo {
        PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        }
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        let pool = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        let event_pools = EventPoolConfig {
            dawn: pool.clone(),
            noon: pool.clone(),
            dusk: pool.clone(),
            midnight: pool.clone(),
            white: pool,
        };

        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools,
        }))
    }

    fn new_core() -> BattleCore {
        let deck = empty_deck();
        BattleCore::new(&deck, &deck, empty_game_data(), (4, 4), 123)
    }

    fn runtime_unit(unit_id: UnitInstanceId, owner: Side, position: WorldVec2) -> RuntimeUnit {
        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 1_000_000;

        RuntimeUnit {
            instance_id: unit_id,
            owner,
            base_uuid: Uuid::nil(),
            stats,
            body: UnitBody::new_at(position, DEFAULT_UNIT_RADIUS, 1.0),
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

    fn moved_to(result: &MovementTickResult, unit_id: UnitInstanceId) -> WorldVec2 {
        result
            .outputs
            .iter()
            .find_map(|output| match output {
                MovementOutput::BodyMoved {
                    unit_id: moved, to, ..
                } if *moved == unit_id => Some(*to),
                _ => None,
            })
            .expect("missing BodyMoved output")
    }

    fn movement_input(dt_ms: u64, units: Vec<MovementUnitInput>) -> MovementTickInput {
        MovementTickInput {
            now_ms: 0,
            dt_ms,
            board_width_units: 10.0,
            board_height_units: 10.0,
            units,
            static_obstacles: Vec::new(),
        }
    }

    #[test]
    fn direct_engine_moves_toward_point_goal() {
        let mut mover = unit(1, WorldVec2::new(1.0, 1.0));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });

        let mut engine = DirectContinuousMovement::default();
        let result = engine.tick(movement_input(500, vec![mover]));

        assert_eq!(result.outputs.len(), 1);
        match result.outputs[0] {
            MovementOutput::BodyMoved { to, velocity, .. } => {
                assert_eq!(to, WorldVec2::new(1.5, 1.0));
                assert_eq!(velocity, WorldVec2::new(1.0, 0.0));
            }
            _ => panic!("expected BodyMoved"),
        }
    }

    #[test]
    fn direct_engine_separates_moving_candidates_that_would_overlap() {
        let first_id: UnitInstanceId = Uuid::from_u128(1001).into();
        let second_id: UnitInstanceId = Uuid::from_u128(1002).into();
        let mut first = unit(1001, WorldVec2::new(1.0, 1.0));
        let mut second = unit(1002, WorldVec2::new(1.0, 1.0));
        first.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });
        second.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });

        let mut engine = DirectContinuousMovement::default();
        let result = engine.tick(movement_input(500, vec![first, second]));

        let first_to = moved_to(&result, first_id);
        let second_to = moved_to(&result, second_id);
        assert!(first_to.distance(second_to) >= 0.7);
    }

    #[test]
    fn direct_engine_separates_mover_from_static_body() {
        let mover_id: UnitInstanceId = Uuid::from_u128(1011).into();
        let blocker_id: UnitInstanceId = Uuid::from_u128(1012).into();
        let mut mover = unit(1011, WorldVec2::new(1.0, 1.0));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });

        let mut blocker = unit(1012, WorldVec2::new(1.6, 1.0));
        blocker.unit_id = blocker_id;
        blocker.can_move = false;

        let mut engine = DirectContinuousMovement::default();
        let result = engine.tick(movement_input(500, vec![mover, blocker]));

        let mover_to = moved_to(&result, mover_id);
        assert!(mover_to.distance(WorldVec2::new(1.6, 1.0)) >= 0.7);
    }

    #[test]
    fn direct_engine_clamps_movement_to_board_bounds() {
        let mover_id: UnitInstanceId = Uuid::from_u128(1021).into();
        let mut mover = unit(1021, WorldVec2::new(1.5, 1.5));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(-10.0, -10.0),
            stop_radius: 0.1,
        });

        let mut engine = DirectContinuousMovement::default();
        let result = engine.tick(MovementTickInput {
            now_ms: 0,
            dt_ms: 2_000,
            board_width_units: 2.0,
            board_height_units: 2.0,
            units: vec![mover],
            static_obstacles: Vec::new(),
        });

        let mover_to = moved_to(&result, mover_id);
        assert_eq!(mover_to, WorldVec2::new(0.35, 0.35));
    }

    #[test]
    fn direct_engine_keeps_separation_correction_inside_board_bounds() {
        let mut first = unit(1031, WorldVec2::new(0.35, 0.35));
        let mut second = unit(1032, WorldVec2::new(0.35, 0.35));
        first.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(-10.0, 0.35),
            stop_radius: 0.1,
        });
        second.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(-10.0, 0.35),
            stop_radius: 0.1,
        });

        let mut engine = DirectContinuousMovement::default();
        let result = engine.tick(MovementTickInput {
            now_ms: 0,
            dt_ms: 500,
            board_width_units: 2.0,
            board_height_units: 2.0,
            units: vec![first, second],
            static_obstacles: Vec::new(),
        });

        let mut moved_count = 0;
        for output in &result.outputs {
            if let MovementOutput::BodyMoved { to, .. } = output {
                moved_count += 1;
                assert!(to.x >= 0.35 && to.x <= 1.65);
                assert!(to.y >= 0.35 && to.y <= 1.65);
            }
        }
        assert!(moved_count > 0);
    }

    #[test]
    fn direct_engine_reports_attack_target_reached() {
        let target_id = Uuid::from_u128(2).into();
        let mut mover = unit(1, WorldVec2::new(0.0, 0.0));
        mover.body.goal = Some(MovementGoal::AttackUnit {
            target_id,
            desired_range: 1.0,
            approach_point: None,
        });

        let mut target = unit(2, WorldVec2::new(1.6, 0.0));
        target.unit_id = target_id;
        target.owner = Side::Opponent;

        let mut engine = DirectContinuousMovement::default();
        let result = engine.tick(movement_input(50, vec![mover, target]));

        assert!(result.outputs.iter().any(|output| matches!(
            output,
            MovementOutput::TargetReached { unit_id, target_id: reached }
                if *unit_id == Uuid::from_u128(1).into() && *reached == target_id
        )));
    }

    #[test]
    fn battle_core_can_run_one_direct_continuous_tick_with_explicit_goal() {
        let mut core = new_core();
        core.use_direct_continuous_movement_backend();
        let unit_id: UnitInstanceId = Uuid::from_u128(10).into();
        core.units.insert(
            unit_id,
            runtime_unit(unit_id, Side::Player, WorldVec2::new(1.0, 1.0)),
        );

        let mut goals = HashMap::new();
        goals.insert(
            unit_id,
            MovementGoal::MoveToPoint {
                point: WorldVec2::new(2.0, 1.0),
                stop_radius: 0.1,
            },
        );

        let result = core.run_continuous_movement_tick(0, 500, &goals);

        assert!(matches!(
            result.outputs.as_slice(),
            [MovementOutput::BodyMoved { unit_id: moved, .. }] if *moved == unit_id
        ));
        assert_eq!(
            core.unit_world_position(unit_id),
            Some(WorldVec2::new(1.5, 1.0))
        );
    }

    #[test]
    fn battle_core_can_run_one_rapier_continuous_tick_with_explicit_goal() {
        let mut core = new_core();
        core.use_rapier_continuous_movement_backend();
        let unit_id: UnitInstanceId = Uuid::from_u128(11).into();
        core.units.insert(
            unit_id,
            runtime_unit(unit_id, Side::Player, WorldVec2::new(1.0, 1.0)),
        );

        let mut goals = HashMap::new();
        goals.insert(
            unit_id,
            MovementGoal::MoveToPoint {
                point: WorldVec2::new(2.0, 1.0),
                stop_radius: 0.1,
            },
        );

        let result = core.run_continuous_movement_tick(0, 500, &goals);

        assert!(matches!(
            result.outputs.as_slice(),
            [MovementOutput::BodyMoved { unit_id: moved, .. }] if *moved == unit_id
        ));
        assert_eq!(
            core.unit_world_position(unit_id),
            Some(WorldVec2::new(1.5, 1.0))
        );
    }

    #[test]
    fn battle_core_movement_input_projects_battlefield_static_obstacles() {
        let mut core = new_core();
        let obstacle = Position::new(2, 2);
        core.battlefield.add_static_obstacle(obstacle).unwrap();

        let input = core.build_continuous_movement_input(0, 50);

        assert_eq!(
            input.static_obstacles,
            vec![MovementStaticObstacle::tile(obstacle)]
        );
    }

    #[test]
    fn battle_core_builds_nearest_enemy_attack_goal_from_continuous_distance() {
        let mut core = new_core();
        let mover_id: UnitInstanceId = Uuid::from_u128(20).into();
        let far_id: UnitInstanceId = Uuid::from_u128(21).into();
        let near_id: UnitInstanceId = Uuid::from_u128(22).into();

        core.units.insert(
            mover_id,
            runtime_unit(mover_id, Side::Player, WorldVec2::new(0.0, 0.0)),
        );
        core.units.insert(
            far_id,
            runtime_unit(far_id, Side::Opponent, WorldVec2::new(5.0, 0.0)),
        );
        core.units.insert(
            near_id,
            runtime_unit(near_id, Side::Opponent, WorldVec2::new(2.0, 0.0)),
        );

        let goals = core.build_continuous_attack_goals();

        assert_eq!(
            goals.get(&mover_id),
            Some(&MovementGoal::AttackUnit {
                target_id: near_id,
                desired_range: 1.0,
                approach_point: goals.get(&mover_id).and_then(|goal| match goal {
                    MovementGoal::AttackUnit { approach_point, .. } => *approach_point,
                    MovementGoal::MoveToPoint { .. } => None,
                })
            })
        );
    }

    #[test]
    fn battle_core_assigns_distinct_melee_approach_slots_for_same_target() {
        let mut core = new_core();
        let target_id: UnitInstanceId = Uuid::from_u128(50).into();
        let first_id: UnitInstanceId = Uuid::from_u128(51).into();
        let second_id: UnitInstanceId = Uuid::from_u128(52).into();
        let third_id: UnitInstanceId = Uuid::from_u128(53).into();

        core.units.insert(
            target_id,
            runtime_unit(target_id, Side::Opponent, WorldVec2::new(2.0, 2.0)),
        );
        core.units.insert(
            first_id,
            runtime_unit(first_id, Side::Player, WorldVec2::new(0.5, 0.5)),
        );
        core.units.insert(
            second_id,
            runtime_unit(second_id, Side::Player, WorldVec2::new(2.0, 0.1)),
        );
        core.units.insert(
            third_id,
            runtime_unit(third_id, Side::Player, WorldVec2::new(3.5, 0.5)),
        );

        let goals = core.build_continuous_attack_goals();
        let mut approach_points = vec![first_id, second_id, third_id]
            .into_iter()
            .map(|unit_id| match goals.get(&unit_id) {
                Some(MovementGoal::AttackUnit {
                    target_id: goal_target_id,
                    approach_point: Some(point),
                    ..
                }) if *goal_target_id == target_id => *point,
                other => panic!("expected melee approach slot for {unit_id}: {other:?}"),
            })
            .collect::<Vec<_>>();

        approach_points.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));
        approach_points.dedup();

        assert_eq!(approach_points.len(), 3);
    }

    #[test]
    fn battle_core_avoids_world_space_melee_slot_conflicts_across_targets() {
        let mut core = new_core();
        let first_target_id: UnitInstanceId = Uuid::from_u128(60).into();
        let second_target_id: UnitInstanceId = Uuid::from_u128(61).into();
        let first_id: UnitInstanceId = Uuid::from_u128(62).into();
        let second_id: UnitInstanceId = Uuid::from_u128(63).into();

        core.units.insert(
            first_target_id,
            runtime_unit(first_target_id, Side::Opponent, WorldVec2::new(2.0, 2.0)),
        );
        core.units.insert(
            second_target_id,
            runtime_unit(second_target_id, Side::Player, WorldVec2::new(2.0, 2.0)),
        );

        let mut first = runtime_unit(first_id, Side::Player, WorldVec2::new(2.0, 0.1));
        first.current_target = Some(first_target_id);
        core.units.insert(first_id, first);

        let mut second = runtime_unit(second_id, Side::Opponent, WorldVec2::new(2.0, 0.1));
        second.current_target = Some(second_target_id);
        core.units.insert(second_id, second);

        let goals = core.build_continuous_attack_goals();
        let first_slot = match goals.get(&first_id) {
            Some(MovementGoal::AttackUnit {
                approach_point: Some(point),
                ..
            }) => *point,
            other => panic!("expected first melee approach slot: {other:?}"),
        };
        let second_slot = match goals.get(&second_id) {
            Some(MovementGoal::AttackUnit {
                approach_point: Some(point),
                ..
            }) => *point,
            other => panic!("expected second melee approach slot: {other:?}"),
        };

        assert!(
            first_slot.distance(second_slot) >= DEFAULT_UNIT_RADIUS * 2.0,
            "slots for different targets should not overlap: first={first_slot:?} second={second_slot:?}"
        );
    }

    #[test]
    fn battle_core_does_not_assign_melee_engagement_point_when_already_in_range() {
        let mut core = new_core();
        let target_id: UnitInstanceId = Uuid::from_u128(70).into();
        let mover_id: UnitInstanceId = Uuid::from_u128(71).into();

        core.units.insert(
            target_id,
            runtime_unit(target_id, Side::Opponent, WorldVec2::new(2.0, 2.0)),
        );
        core.units.insert(
            mover_id,
            runtime_unit(mover_id, Side::Player, WorldVec2::new(1.0, 2.0)),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&mover_id),
            Some(MovementGoal::AttackUnit {
                target_id: goal_target_id,
                approach_point: None,
                ..
            }) if *goal_target_id == target_id
        ));
    }

    #[test]
    fn battle_core_keeps_previous_melee_engagement_point_across_small_target_drift() {
        let mut core = new_core();
        let target_id: UnitInstanceId = Uuid::from_u128(72).into();
        let mover_id: UnitInstanceId = Uuid::from_u128(73).into();

        core.units.insert(
            target_id,
            runtime_unit(target_id, Side::Opponent, WorldVec2::new(2.5, 2.5)),
        );
        core.units.insert(
            mover_id,
            runtime_unit(mover_id, Side::Player, WorldVec2::new(0.5, 0.5)),
        );

        let first_goals = core.build_continuous_attack_goals();
        let first_point = match first_goals.get(&mover_id) {
            Some(MovementGoal::AttackUnit {
                approach_point: Some(point),
                ..
            }) => *point,
            other => panic!("expected initial melee engagement point: {other:?}"),
        };

        let target = core.units.get_mut(&target_id).expect("target exists");
        target.body.position += WorldVec2::new(0.05, 0.0);

        let second_goals = core.build_continuous_attack_goals();
        let second_point = match second_goals.get(&mover_id) {
            Some(MovementGoal::AttackUnit {
                approach_point: Some(point),
                ..
            }) => *point,
            other => panic!("expected retained melee engagement point: {other:?}"),
        };

        assert_eq!(second_point, first_point);
    }

    #[test]
    fn battle_core_continuous_attack_tick_moves_toward_nearest_enemy() {
        let mut core = new_core();
        let mover_id: UnitInstanceId = Uuid::from_u128(30).into();
        let target_id: UnitInstanceId = Uuid::from_u128(31).into();

        core.units.insert(
            mover_id,
            runtime_unit(mover_id, Side::Player, WorldVec2::new(1.0, 1.0)),
        );
        core.units.insert(
            target_id,
            runtime_unit(target_id, Side::Opponent, WorldVec2::new(3.0, 1.0)),
        );

        let result = core.run_continuous_attack_movement_tick(0, 500);

        assert!(result.outputs.iter().any(|output| matches!(
            output,
            MovementOutput::BodyMoved { unit_id, .. } if *unit_id == mover_id
        )));
        let mover_position = core
            .unit_world_position(mover_id)
            .expect("missing mover position");
        let target_position = core
            .unit_world_position(target_id)
            .expect("missing target position");
        assert!(
            mover_position.x > 1.0,
            "expected mover to advance toward enemy, got {mover_position:?}"
        );
        assert!(
            mover_position.distance(target_position) < WorldVec2::new(1.0, 1.0).distance(target_position),
            "expected mover to reduce distance to target, mover={mover_position:?} target={target_position:?}"
        );
    }

    #[test]
    fn battle_core_continuous_attack_goal_preserves_alive_sticky_target() {
        let mut core = new_core();
        let mover_id: UnitInstanceId = Uuid::from_u128(40).into();
        let sticky_id: UnitInstanceId = Uuid::from_u128(41).into();
        let nearer_id: UnitInstanceId = Uuid::from_u128(42).into();

        let mut mover = runtime_unit(mover_id, Side::Player, WorldVec2::new(0.0, 0.0));
        mover.current_target = Some(sticky_id);
        core.units.insert(mover_id, mover);
        core.units.insert(
            sticky_id,
            runtime_unit(sticky_id, Side::Opponent, WorldVec2::new(5.0, 0.0)),
        );
        core.units.insert(
            nearer_id,
            runtime_unit(nearer_id, Side::Opponent, WorldVec2::new(2.0, 0.0)),
        );

        let goals = core.build_continuous_attack_goals();

        assert_eq!(
            goals.get(&mover_id),
            Some(&MovementGoal::AttackUnit {
                target_id: sticky_id,
                desired_range: 1.0,
                approach_point: goals.get(&mover_id).and_then(|goal| match goal {
                    MovementGoal::AttackUnit { approach_point, .. } => *approach_point,
                    MovementGoal::MoveToPoint { .. } => None,
                })
            })
        );
    }
}
