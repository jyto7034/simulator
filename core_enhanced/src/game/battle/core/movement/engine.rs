use std::collections::HashMap;

use crate::{
    game::resources::Position,
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
    steering,
    types::{MovementGoal, TimelineVec2, UnitBody, WorldVec2},
    MovementSegmentEndKind,
};

const STATIC_OBSTACLE_EPSILON: f32 = 0.001;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MovementTerrainPolicy {
    #[default]
    Ground,
    Airborne,
}

impl MovementTerrainPolicy {
    pub fn applies_static_obstacles(self) -> bool {
        matches!(self, Self::Ground)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovementUnitInput {
    pub unit_id: UnitInstanceId,
    pub owner: Side,
    pub body: UnitBody,
    pub terrain_policy: MovementTerrainPolicy,
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

    pub fn void_tile(tile: Position) -> Self {
        let mut obstacle = Self::tile(tile);
        obstacle.obstacle_id ^= 0x8000_0000_0000_0000;
        obstacle
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
    StaticObstacleBlocked,
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

pub(in crate::game::battle::core::movement) fn canonicalize_movement_units(
    units: &mut [MovementUnitInput],
) {
    units.sort_by(|a, b| a.unit_id.as_bytes().cmp(b.unit_id.as_bytes()));
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

impl ContinuousMovementBackend {
    pub(in crate::game::battle::core) fn reset_for_battle(&mut self) {
        if let Self::Rapier(engine) = self {
            engine.clear();
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DirectContinuousMovement;

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

    fn resolve_static_obstacles(
        from: WorldVec2,
        desired: WorldVec2,
        radius: f32,
        obstacles: &[MovementStaticObstacle],
        board_width: f32,
        board_height: f32,
    ) -> WorldVec2 {
        let mut resolved = desired;
        if let Some(hit_t) = obstacles
            .iter()
            .filter_map(|obstacle| swept_point_vs_expanded_aabb(from, desired, radius, obstacle))
            .min_by(|a, b| a.total_cmp(b))
        {
            let motion = desired - from;
            resolved = from + motion * (hit_t - STATIC_OBSTACLE_EPSILON).clamp(0.0, 1.0);
        }

        for obstacle in obstacles {
            resolved = depenetrate_expanded_aabb(resolved, radius, obstacle);
        }

        Self::clamp_to_board(resolved, radius, board_width, board_height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MovementCandidate {
    unit_id: UnitInstanceId,
    from: WorldVec2,
    desired_to: WorldVec2,
    to: WorldVec2,
    radius: f32,
    terrain_policy: MovementTerrainPolicy,
}

impl MovementEngine for DirectContinuousMovement {
    fn tick(&mut self, input: MovementTickInput) -> MovementTickResult {
        let mut outputs = Vec::new();
        let mut candidates = Vec::new();
        let dt_seconds = input.dt_ms as f32 / 1_000.0;
        let mut units = input.units;
        canonicalize_movement_units(&mut units);

        for unit in &units {
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

            if let Some(reached) = steering::reached_goal(unit, &units) {
                outputs.push(reached);
                continue;
            }

            let Some(target) = steering::goal_target_position(unit, &units) else {
                outputs.push(MovementOutput::MovementStopped {
                    unit_id: unit.unit_id,
                    position: unit.body.position,
                    reason: MovementStopReasonContinuous::NoGoal,
                });
                continue;
            };

            let displacement = steering::steered_displacement(unit, target, dt_seconds);
            let desired_to = Self::clamp_to_board(
                unit.body.position + displacement,
                unit.body.radius,
                input.board_width_units,
                input.board_height_units,
            );
            let to = if unit.terrain_policy.applies_static_obstacles() {
                Self::resolve_static_obstacles(
                    unit.body.position,
                    desired_to,
                    unit.body.radius,
                    &input.static_obstacles,
                    input.board_width_units,
                    input.board_height_units,
                )
            } else {
                desired_to
            };

            candidates.push(MovementCandidate {
                unit_id: unit.unit_id,
                from: unit.body.position,
                desired_to,
                to,
                radius: unit.body.radius,
                terrain_policy: unit.terrain_policy,
            });
        }

        for candidate in &mut candidates {
            if candidate.terrain_policy.applies_static_obstacles() {
                candidate.to = Self::resolve_static_obstacles(
                    candidate.from,
                    candidate.to,
                    candidate.radius,
                    &input.static_obstacles,
                    input.board_width_units,
                    input.board_height_units,
                );
            }
        }

        for candidate in candidates {
            let velocity = steering::movement_velocity(candidate.from, candidate.to, dt_seconds);
            if (candidate.to - candidate.from).length_squared() <= f32::EPSILON {
                if candidate.terrain_policy.applies_static_obstacles()
                    && !input.static_obstacles.is_empty()
                    && (candidate.desired_to - candidate.from).length_squared() > f32::EPSILON
                {
                    outputs.push(MovementOutput::MovementStopped {
                        unit_id: candidate.unit_id,
                        position: candidate.from,
                        reason: MovementStopReasonContinuous::StaticObstacleBlocked,
                    });
                }
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

fn expanded_aabb(obstacle: &MovementStaticObstacle, radius: f32) -> (f32, f32, f32, f32) {
    let radius = radius.max(0.0);
    (
        obstacle.center.x - obstacle.half_extents.x - radius,
        obstacle.center.x + obstacle.half_extents.x + radius,
        obstacle.center.y - obstacle.half_extents.y - radius,
        obstacle.center.y + obstacle.half_extents.y + radius,
    )
}

fn point_inside_expanded_aabb(
    point: WorldVec2,
    radius: f32,
    obstacle: &MovementStaticObstacle,
) -> bool {
    let (min_x, max_x, min_y, max_y) = expanded_aabb(obstacle, radius);
    point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
}

fn swept_point_vs_expanded_aabb(
    from: WorldVec2,
    to: WorldVec2,
    radius: f32,
    obstacle: &MovementStaticObstacle,
) -> Option<f32> {
    if point_inside_expanded_aabb(from, radius, obstacle) {
        return Some(0.0);
    }

    let (min_x, max_x, min_y, max_y) = expanded_aabb(obstacle, radius);
    let delta = to - from;
    let (mut enter, mut exit) = (0.0_f32, 1.0_f32);

    for (start, movement, min, max) in [
        (from.x, delta.x, min_x, max_x),
        (from.y, delta.y, min_y, max_y),
    ] {
        if movement.abs() <= f32::EPSILON {
            if start < min || start > max {
                return None;
            }
            continue;
        }

        let inv = 1.0 / movement;
        let mut axis_enter = (min - start) * inv;
        let mut axis_exit = (max - start) * inv;
        if axis_enter > axis_exit {
            std::mem::swap(&mut axis_enter, &mut axis_exit);
        }

        enter = enter.max(axis_enter);
        exit = exit.min(axis_exit);
        if enter > exit {
            return None;
        }
    }

    (0.0..=1.0).contains(&enter).then_some(enter)
}

fn depenetrate_expanded_aabb(
    point: WorldVec2,
    radius: f32,
    obstacle: &MovementStaticObstacle,
) -> WorldVec2 {
    if !point_inside_expanded_aabb(point, radius, obstacle) {
        return point;
    }

    let (min_x, max_x, min_y, max_y) = expanded_aabb(obstacle, radius);
    let exits = [
        (
            point.x - min_x,
            WorldVec2::new(min_x - STATIC_OBSTACLE_EPSILON, point.y),
        ),
        (
            max_x - point.x,
            WorldVec2::new(max_x + STATIC_OBSTACLE_EPSILON, point.y),
        ),
        (
            point.y - min_y,
            WorldVec2::new(point.x, min_y - STATIC_OBSTACLE_EPSILON),
        ),
        (
            max_y - point.y,
            WorldVec2::new(point.x, max_y + STATIC_OBSTACLE_EPSILON),
        ),
    ];

    exits
        .into_iter()
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, point)| point)
        .unwrap_or(point)
}

impl BattleCore {
    #[cfg(test)]
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
                            self.active_movement_segments.insert(
                                unit_id,
                                ActiveMovementSegment {
                                    target: to,
                                    ends_at_ms,
                                    ..active
                                },
                            );
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
                start: from,
                target: to,
                started_at_ms: time_ms,
                ends_at_ms,
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
        self.refresh_block_state();
        let goals = self.build_continuous_attack_goals_from_current_block_state();
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
                let is_blocked = self.blocked_by(unit_id).is_some();
                Some(MovementUnitInput {
                    unit_id,
                    owner: unit.owner,
                    body,
                    terrain_policy: if unit.is_airborne() {
                        MovementTerrainPolicy::Airborne
                    } else {
                        MovementTerrainPolicy::Ground
                    },
                    current_target: unit.current_target,
                    attack_range_units: unit.basic_attack.range_units.max(0.0),
                    can_move: unit.action_locks.can_move(now_ms) && unit.can_move() && !is_blocked,
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
        let mut obstacles = self
            .battlefield
            .static_obstacles()
            .into_iter()
            .map(MovementStaticObstacle::tile)
            .collect::<Vec<_>>();
        obstacles.extend(
            self.battlefield
                .void_tiles()
                .into_iter()
                .map(MovementStaticObstacle::void_tile),
        );
        obstacles
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
            scenario::BattleScenario,
        },
        data::{GameDataBase, GameDataBuilder},
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
            terrain_policy: MovementTerrainPolicy::Ground,
            current_target: None,
            attack_range_units: 1.0,
            can_move: true,
            is_dead: false,
        }
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        GameDataBuilder::empty().build_arc()
    }

    fn new_core() -> BattleCore {
        BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), empty_game_data(), 123)
    }

    fn runtime_unit(unit_id: UnitInstanceId, owner: Side, position: WorldVec2) -> RuntimeUnit {
        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 1_000_000;

        RuntimeUnit {
            instance_id: unit_id,
            spawn_order: u64::from(unit_id.as_bytes()[15]),
            source_owned_uuid: unit_id.as_uuid(),
            owner,
            role: crate::game::battle::types::BattleUnitRole::Combatant,
            base_uuid: Uuid::nil(),
            stats,
            incoming_damage_modifiers: Default::default(),
            basic_attack: Default::default(),
            skill_id: None,
            skill_activation_mode: crate::game::ability::SkillActivationMode::Auto,
            body: UnitBody::new_at(position, DEFAULT_UNIT_RADIUS, 1.0),
            tactical_anchor: Some(position),
            enemy_movement_plan: None,
            block_capacity: 0,
            block_radius_units: 0.0,
            blockable: true,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
            facing_direction: None,
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

    fn run_direct_and_rapier(input: MovementTickInput) -> (MovementTickResult, MovementTickResult) {
        let mut direct = DirectContinuousMovement::default();
        let direct_result = direct.tick(input.clone());
        let mut rapier = RapierMovementWorld::new();
        let rapier_result = rapier.tick(input);
        (direct_result, rapier_result)
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
    fn movement_backends_canonicalize_unit_order_before_resolution() {
        let mut first = unit(1, WorldVec2::new(1.0, 1.0));
        let mut second = unit(2, WorldVec2::new(3.0, 1.0));
        first.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(2.0, 1.0),
            stop_radius: 0.1,
        });
        second.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(4.0, 1.0),
            stop_radius: 0.1,
        });

        let mut direct = DirectContinuousMovement::default();
        let direct_result = direct.tick(movement_input(500, vec![second.clone(), first.clone()]));
        assert!(matches!(
            direct_result.outputs.first(),
            Some(MovementOutput::BodyMoved { unit_id, .. })
                if *unit_id == Uuid::from_u128(1).into()
        ));

        let mut rapier = RapierMovementWorld::new();
        let rapier_result = rapier.tick(movement_input(500, vec![second, first]));
        assert!(matches!(
            rapier_result.outputs.first(),
            Some(MovementOutput::BodyMoved { unit_id, .. })
                if *unit_id == Uuid::from_u128(1).into()
        ));
    }

    #[test]
    fn movement_backends_match_ground_static_obstacle_stop_policy() {
        let mover_id: UnitInstanceId = Uuid::from_u128(2001).into();
        let mut mover = unit(2001, WorldVec2::new(1.0, 1.0));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(5.0, 3.0),
            stop_radius: 0.1,
        });
        let mut input = movement_input(3_000, vec![mover]);
        input.static_obstacles.push(MovementStaticObstacle {
            obstacle_id: 2001,
            center: WorldVec2::new(2.0, 2.0),
            half_extents: WorldVec2::new(0.1, 5.0),
        });

        let (direct_result, rapier_result) = run_direct_and_rapier(input);
        let direct_to = moved_to(&direct_result, mover_id);
        let rapier_to = moved_to(&rapier_result, mover_id);

        assert!(
            direct_to.distance(rapier_to) <= 0.01,
            "ground obstacle stop policy should be backend-equivalent: direct={direct_to:?}, rapier={rapier_to:?}"
        );
        assert!(
            rapier_to.x < 1.55 && rapier_to.y < 1.35,
            "ground obstacle response should expose the blocked route instead of sliding around it: {rapier_to:?}"
        );
    }

    #[test]
    fn movement_backends_match_airborne_static_obstacle_policy() {
        let mover_id: UnitInstanceId = Uuid::from_u128(2002).into();
        let mut mover = unit(2002, WorldVec2::new(0.5, 1.5));
        mover.terrain_policy = MovementTerrainPolicy::Airborne;
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(3.5, 1.5),
            stop_radius: 0.1,
        });
        let mut input = movement_input(2_000, vec![mover]);
        input
            .static_obstacles
            .push(MovementStaticObstacle::tile(Position::new(1, 1)));

        let (direct_result, rapier_result) = run_direct_and_rapier(input);

        assert_eq!(moved_to(&direct_result, mover_id), WorldVec2::new(2.5, 1.5));
        assert_eq!(moved_to(&rapier_result, mover_id), WorldVec2::new(2.5, 1.5));
    }

    #[test]
    fn movement_backends_match_board_clamp_policy() {
        let mover_id: UnitInstanceId = Uuid::from_u128(2003).into();
        let mut mover = unit(2003, WorldVec2::new(1.5, 1.5));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(-10.0, -10.0),
            stop_radius: 0.1,
        });
        let input = MovementTickInput {
            now_ms: 0,
            dt_ms: 2_000,
            board_width_units: 2.0,
            board_height_units: 2.0,
            units: vec![mover],
            static_obstacles: Vec::new(),
        };

        let (direct_result, rapier_result) = run_direct_and_rapier(input);

        let expected = WorldVec2::new(0.35, 0.35);
        assert!(
            moved_to(&direct_result, mover_id).distance(expected) <= f32::EPSILON,
            "direct clamp should resolve to board minimum"
        );
        assert!(
            moved_to(&rapier_result, mover_id).distance(expected) <= 0.0001,
            "rapier clamp should resolve to the same board minimum"
        );
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
    fn direct_engine_allows_moving_candidates_to_overlap() {
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
        assert_eq!(first_to, second_to);
    }

    #[test]
    fn direct_engine_allows_mover_to_overlap_locked_unit_body() {
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
        assert_eq!(mover_to, WorldVec2::new(1.5, 1.0));
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
    fn direct_engine_allows_overlapping_units_at_board_bounds() {
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

        assert!(result.outputs.is_empty());
    }

    #[test]
    fn direct_engine_static_obstacle_blocks_swept_movement() {
        let mover_id: UnitInstanceId = Uuid::from_u128(1041).into();
        let mut mover = unit(1041, WorldVec2::new(0.5, 1.5));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(3.5, 1.5),
            stop_radius: 0.1,
        });

        let mut input = movement_input(2_000, vec![mover]);
        input
            .static_obstacles
            .push(MovementStaticObstacle::tile(Position::new(1, 1)));

        let mut engine = DirectContinuousMovement::default();
        let result = engine.tick(input);

        let mover_to = moved_to(&result, mover_id);
        assert!(
            mover_to.x < 0.65,
            "expanded obstacle should stop movement before tile (1, 1), got {mover_to:?}"
        );
    }

    #[test]
    fn direct_engine_airborne_policy_ignores_static_obstacles() {
        let mover_id: UnitInstanceId = Uuid::from_u128(1042).into();
        let mut mover = unit(1042, WorldVec2::new(0.5, 1.5));
        mover.terrain_policy = MovementTerrainPolicy::Airborne;
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(3.5, 1.5),
            stop_radius: 0.1,
        });

        let mut input = movement_input(2_000, vec![mover]);
        input
            .static_obstacles
            .push(MovementStaticObstacle::tile(Position::new(1, 1)));

        let mut engine = DirectContinuousMovement;
        let result = engine.tick(input);

        let mover_to = moved_to(&result, mover_id);
        assert_eq!(mover_to, WorldVec2::new(2.5, 1.5));
    }

    #[test]
    fn direct_engine_airborne_policy_still_clamps_to_board_bounds() {
        let mover_id: UnitInstanceId = Uuid::from_u128(1043).into();
        let mut mover = unit(1043, WorldVec2::new(1.5, 1.5));
        mover.terrain_policy = MovementTerrainPolicy::Airborne;
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(-10.0, -10.0),
            stop_radius: 0.1,
        });

        let mut engine = DirectContinuousMovement;
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
    fn battle_core_movement_input_projects_non_rectangular_void_tiles_as_static_obstacles() {
        let scenario = BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
                width: 3,
                height: 3,
                valid_tiles: vec![
                    Position::new(1, 0),
                    Position::new(1, 1),
                    Position::new(1, 2),
                ],
                obstacles: vec![Position::new(1, 1)],
            },
            artifacts: Vec::new(),
            groups: Vec::new(),
            events: Vec::new(),
            win_condition:
                crate::game::battle::scenario::WinCondition::AllRequiredEnemyGroupsDefeated,
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        core.battlefield
            .add_static_obstacle(Position::new(1, 1))
            .unwrap();

        let input = core.build_continuous_movement_input(0, 50);

        assert!(input
            .static_obstacles
            .contains(&MovementStaticObstacle::tile(Position::new(1, 1))));
        assert!(input
            .static_obstacles
            .contains(&MovementStaticObstacle::void_tile(Position::new(0, 0))));
        assert!(input
            .static_obstacles
            .contains(&MovementStaticObstacle::void_tile(Position::new(2, 2))));
        assert!(!input
            .static_obstacles
            .contains(&MovementStaticObstacle::void_tile(Position::new(1, 0))));
    }
}
