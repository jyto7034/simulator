use std::collections::{HashMap, HashSet};

use rapier2d::control::{CharacterLength, KinematicCharacterController};
use rapier2d::prelude::{
    BroadPhaseBvh, BroadPhasePairEvent, Collider, ColliderBuilder, ColliderHandle, ColliderSet,
    ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase,
    QueryFilter, RigidBodyBuilder, RigidBodyHandle, RigidBodySet, Vector,
};

use crate::game::battle::ids::UnitInstanceId;

use super::engine::{
    run_continuous_movement_tick, ContinuousMovementResolver, DirectContinuousMovement,
    MovementEngine, MovementStaticObstacle, MovementTickInput, MovementTickResult,
    MovementUnitInput,
};
use super::types::WorldVec2;

/// Rapier-side handles owned by the continuous movement backend.
///
/// `RuntimeUnit` keeps gameplay body state. Rapier handles stay inside this
/// backend so physics internals do not leak into combat rules.
#[derive(Debug, Clone, Copy)]
pub struct RapierUnitHandles {
    pub rigid_body: RigidBodyHandle,
    pub collider: ColliderHandle,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct RapierBoardBounds {
    pub width_units: f32,
    pub height_units: f32,
    pub rigid_body: RigidBodyHandle,
    pub colliders: Vec<ColliderHandle>,
}

#[derive(Debug, Clone, Copy)]
pub struct RapierStaticObstacleHandles {
    pub rigid_body: RigidBodyHandle,
    pub collider: ColliderHandle,
    pub center: WorldVec2,
    pub half_extents: WorldVec2,
}

pub struct RapierMovementWorld {
    islands: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    rigid_bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    controller: KinematicCharacterController,
    unit_handles: HashMap<UnitInstanceId, RapierUnitHandles>,
    board_bounds: Option<RapierBoardBounds>,
    static_obstacles: HashMap<u64, RapierStaticObstacleHandles>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RapierSyncReport {
    pub created: usize,
    pub updated: usize,
    pub removed: usize,
}

impl RapierMovementWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.unit_handles.is_empty()
    }

    pub fn len(&self) -> usize {
        self.unit_handles.len()
    }

    pub fn rigid_bodies(&self) -> &RigidBodySet {
        &self.rigid_bodies
    }

    pub fn colliders(&self) -> &ColliderSet {
        &self.colliders
    }

    #[cfg(test)]
    pub fn board_bounds(&self) -> Option<&RapierBoardBounds> {
        self.board_bounds.as_ref()
    }

    pub fn static_obstacle_count(&self) -> usize {
        self.static_obstacles.len()
    }

    pub fn corrected_ground_static_obstacle_translation_for(
        &mut self,
        unit_id: UnitInstanceId,
        desired_translation: WorldVec2,
        dt_seconds: f32,
    ) -> Option<WorldVec2> {
        self.refresh_broad_phase();

        let handles = self.handles_for(unit_id)?;
        let collider = self.colliders.get(handles.collider)?;
        let body = self.rigid_bodies.get(handles.rigid_body)?;
        let mut ignored_colliders = self
            .unit_handles
            .values()
            .map(|handles| handles.collider)
            .collect::<HashSet<_>>();
        if let Some(bounds) = &self.board_bounds {
            ignored_colliders.extend(bounds.colliders.iter().copied());
        }
        let include_ground_static_obstacle_colliders =
            |collider_handle: ColliderHandle, _collider: &Collider| {
                !ignored_colliders.contains(&collider_handle)
            };
        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_bodies,
            &self.colliders,
            QueryFilter {
                exclude_collider: Some(handles.collider),
                predicate: Some(&include_ground_static_obstacle_colliders),
                ..QueryFilter::default()
            },
        );
        let movement = self.controller.move_shape(
            dt_seconds,
            &query_pipeline,
            collider.shape(),
            body.position(),
            to_rapier_vector(desired_translation),
            |_| {},
        );

        Some(from_rapier_vector(movement.translation))
    }

    pub fn rapier_position_for(&self, unit_id: UnitInstanceId) -> Option<WorldVec2> {
        let handles = self.handles_for(unit_id)?;
        let body = self.rigid_bodies.get(handles.rigid_body)?;
        Some(from_rapier_vector(body.translation()))
    }

    pub fn handles_for(&self, unit_id: UnitInstanceId) -> Option<RapierUnitHandles> {
        self.unit_handles.get(&unit_id).copied()
    }

    pub fn forget_unit(&mut self, unit_id: UnitInstanceId) -> Option<RapierUnitHandles> {
        let handles = self.unit_handles.remove(&unit_id)?;
        self.rigid_bodies.remove(
            handles.rigid_body,
            &mut self.islands,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
        Some(handles)
    }

    pub fn sync_units(&mut self, units: &[MovementUnitInput]) -> RapierSyncReport {
        let mut report = RapierSyncReport::default();
        let mut present_unit_ids = Vec::new();

        for unit in units {
            if unit.is_dead {
                if self.forget_unit(unit.unit_id).is_some() {
                    report.removed += 1;
                }
                continue;
            }

            present_unit_ids.push(unit.unit_id);
            if self.handles_for(unit.unit_id).is_some() {
                self.sync_existing_unit(unit);
                report.updated += 1;
            } else {
                self.insert_unit(unit);
                report.created += 1;
            }
        }

        let stale_unit_ids: Vec<UnitInstanceId> = self
            .unit_handles
            .keys()
            .copied()
            .filter(|unit_id| !present_unit_ids.contains(unit_id))
            .collect();
        for unit_id in stale_unit_ids {
            if self.forget_unit(unit_id).is_some() {
                report.removed += 1;
            }
        }

        report
    }

    pub fn clear(&mut self) {
        self.islands = IslandManager::new();
        self.broad_phase = BroadPhaseBvh::new();
        self.narrow_phase = NarrowPhase::new();
        self.rigid_bodies = RigidBodySet::new();
        self.colliders = ColliderSet::new();
        self.impulse_joints = ImpulseJointSet::new();
        self.multibody_joints = MultibodyJointSet::new();
        self.unit_handles.clear();
        self.board_bounds = None;
        self.static_obstacles.clear();
    }

    pub fn sync_static_obstacles(&mut self, obstacles: &[MovementStaticObstacle]) {
        let mut present = HashSet::new();

        for obstacle in obstacles {
            present.insert(obstacle.obstacle_id);

            if self
                .static_obstacles
                .get(&obstacle.obstacle_id)
                .is_some_and(|handles| static_obstacle_matches(handles, obstacle))
            {
                continue;
            }

            self.remove_static_obstacle(obstacle.obstacle_id);
            self.insert_static_obstacle(*obstacle);
        }

        let stale_ids: Vec<u64> = self
            .static_obstacles
            .keys()
            .copied()
            .filter(|obstacle_id| !present.contains(obstacle_id))
            .collect();
        for obstacle_id in stale_ids {
            self.remove_static_obstacle(obstacle_id);
        }
    }

    #[cfg(test)]
    pub fn sync_board_bounds(&mut self, width_units: f32, height_units: f32) {
        let width_units = width_units.max(0.0);
        let height_units = height_units.max(0.0);

        if self.board_bounds.as_ref().is_some_and(|bounds| {
            (bounds.width_units - width_units).abs() <= f32::EPSILON
                && (bounds.height_units - height_units).abs() <= f32::EPSILON
        }) {
            return;
        }

        self.remove_board_bounds();
        let rigid_body = self.rigid_bodies.insert(RigidBodyBuilder::fixed().build());
        let wall_thickness = 1.0;
        let horizontal_half_width = width_units * 0.5 + wall_thickness;
        let vertical_half_height = height_units * 0.5 + wall_thickness;
        let colliders = vec![
            self.insert_wall_collider(
                rigid_body,
                horizontal_half_width,
                wall_thickness * 0.5,
                WorldVec2::new(width_units * 0.5, -wall_thickness * 0.5),
            ),
            self.insert_wall_collider(
                rigid_body,
                horizontal_half_width,
                wall_thickness * 0.5,
                WorldVec2::new(width_units * 0.5, height_units + wall_thickness * 0.5),
            ),
            self.insert_wall_collider(
                rigid_body,
                wall_thickness * 0.5,
                vertical_half_height,
                WorldVec2::new(-wall_thickness * 0.5, height_units * 0.5),
            ),
            self.insert_wall_collider(
                rigid_body,
                wall_thickness * 0.5,
                vertical_half_height,
                WorldVec2::new(width_units + wall_thickness * 0.5, height_units * 0.5),
            ),
        ];

        self.board_bounds = Some(RapierBoardBounds {
            width_units,
            height_units,
            rigid_body,
            colliders,
        });
    }

    fn insert_unit(&mut self, unit: &MovementUnitInput) {
        let body = RigidBodyBuilder::kinematic_position_based()
            .translation(to_rapier_vector(unit.body.position))
            .build();
        let rigid_body = self.rigid_bodies.insert(body);
        let collider = ColliderBuilder::ball(unit.body.radius.max(0.0)).build();
        let collider =
            self.colliders
                .insert_with_parent(collider, rigid_body, &mut self.rigid_bodies);

        self.unit_handles.insert(
            unit.unit_id,
            RapierUnitHandles {
                rigid_body,
                collider,
                radius: unit.body.radius,
            },
        );
    }

    #[cfg(test)]
    fn insert_wall_collider(
        &mut self,
        rigid_body: RigidBodyHandle,
        half_width: f32,
        half_height: f32,
        position: WorldVec2,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(half_width, half_height)
            .translation(to_rapier_vector(position))
            .build();
        self.colliders
            .insert_with_parent(collider, rigid_body, &mut self.rigid_bodies)
    }

    fn insert_static_obstacle(&mut self, obstacle: MovementStaticObstacle) {
        let rigid_body = self.rigid_bodies.insert(
            RigidBodyBuilder::fixed()
                .translation(to_rapier_vector(obstacle.center))
                .build(),
        );
        let collider = ColliderBuilder::cuboid(
            obstacle.half_extents.x.max(0.0),
            obstacle.half_extents.y.max(0.0),
        )
        .build();
        let collider =
            self.colliders
                .insert_with_parent(collider, rigid_body, &mut self.rigid_bodies);
        self.static_obstacles.insert(
            obstacle.obstacle_id,
            RapierStaticObstacleHandles {
                rigid_body,
                collider,
                center: obstacle.center,
                half_extents: obstacle.half_extents,
            },
        );
    }

    fn remove_static_obstacle(&mut self, obstacle_id: u64) {
        let Some(handles) = self.static_obstacles.remove(&obstacle_id) else {
            return;
        };
        self.rigid_bodies.remove(
            handles.rigid_body,
            &mut self.islands,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
    }

    #[cfg(test)]
    fn remove_board_bounds(&mut self) {
        let Some(bounds) = self.board_bounds.take() else {
            return;
        };
        self.rigid_bodies.remove(
            bounds.rigid_body,
            &mut self.islands,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
    }

    fn sync_existing_unit(&mut self, unit: &MovementUnitInput) {
        let Some(handles) = self.handles_for(unit.unit_id) else {
            return;
        };

        if (handles.radius - unit.body.radius).abs() > f32::EPSILON {
            self.forget_unit(unit.unit_id);
            self.insert_unit(unit);
            return;
        }

        if let Some(body) = self.rigid_bodies.get_mut(handles.rigid_body) {
            body.set_translation(to_rapier_vector(unit.body.position), true);
            body.set_next_kinematic_translation(to_rapier_vector(unit.body.position));
        }
    }

    fn refresh_broad_phase(&mut self) {
        for _ in self.colliders.iter_mut() {}

        let modified = self.colliders.take_modified();
        let removed = self.colliders.take_removed();
        let mut events: Vec<BroadPhasePairEvent> = Vec::new();
        self.broad_phase.update(
            &IntegrationParameters::default(),
            &self.colliders,
            &self.rigid_bodies,
            &modified,
            &removed,
            &mut events,
        );
    }

    fn apply_unit_translation(
        &mut self,
        unit_id: UnitInstanceId,
        position: WorldVec2,
    ) -> Option<()> {
        let handles = self.handles_for(unit_id)?;
        let body = self.rigid_bodies.get_mut(handles.rigid_body)?;
        body.set_translation(to_rapier_vector(position), true);
        body.set_next_kinematic_translation(to_rapier_vector(position));
        Some(())
    }
}

impl Default for RapierMovementWorld {
    fn default() -> Self {
        Self {
            islands: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            controller: KinematicCharacterController {
                offset: CharacterLength::Absolute(0.001),
                slide: false,
                autostep: None,
                snap_to_ground: None,
                ..KinematicCharacterController::default()
            },
            unit_handles: HashMap::new(),
            board_bounds: None,
            static_obstacles: HashMap::new(),
        }
    }
}

impl MovementEngine for RapierMovementWorld {
    fn tick(&mut self, input: MovementTickInput) -> MovementTickResult {
        run_continuous_movement_tick(self, input)
    }
}

impl ContinuousMovementResolver for RapierMovementWorld {
    fn prepare_tick(
        &mut self,
        units: &[MovementUnitInput],
        static_obstacles: &[MovementStaticObstacle],
    ) {
        self.sync_units(units);
        self.sync_static_obstacles(static_obstacles);
    }

    fn corrected_position(
        &mut self,
        unit: &MovementUnitInput,
        desired_to: WorldVec2,
        board_width_units: f32,
        board_height_units: f32,
        _static_obstacles: &[MovementStaticObstacle],
        dt_seconds: f32,
    ) -> WorldVec2 {
        let desired_translation = desired_to - unit.body.position;
        let corrected_translation = if unit.terrain_policy.applies_static_obstacles() {
            self.corrected_ground_static_obstacle_translation_for(
                unit.unit_id,
                desired_translation,
                dt_seconds,
            )
            .unwrap_or(desired_translation)
        } else {
            desired_translation
        };
        DirectContinuousMovement::clamp_to_board(
            unit.body.position + corrected_translation,
            unit.body.radius,
            board_width_units,
            board_height_units,
        )
    }

    fn apply_position(&mut self, unit_id: UnitInstanceId, position: WorldVec2) {
        self.apply_unit_translation(unit_id, position);
    }
}

fn static_obstacle_matches(
    handles: &RapierStaticObstacleHandles,
    obstacle: &MovementStaticObstacle,
) -> bool {
    handles.center.distance(obstacle.center) <= f32::EPSILON
        && handles.half_extents.distance(obstacle.half_extents) <= f32::EPSILON
}

fn to_rapier_vector(position: WorldVec2) -> Vector {
    Vector::new(position.x, position.y)
}

fn from_rapier_vector(position: Vector) -> WorldVec2 {
    WorldVec2::new(position.x, position.y)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::game::{
        battle::core::movement::{
            engine::{
                MovementEngine, MovementOutput, MovementStaticObstacle, MovementTerrainPolicy,
                MovementTickInput, MovementUnitInput,
            },
            types::{MovementGoal, MovementMode, UnitBody, WorldVec2},
        },
        enums::Side,
    };

    use super::*;

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
            can_move: true,
            is_dead: false,
        }
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

    fn bounded_movement_input(
        dt_ms: u64,
        board_width_units: f32,
        board_height_units: f32,
        units: Vec<MovementUnitInput>,
    ) -> MovementTickInput {
        MovementTickInput {
            now_ms: 0,
            dt_ms,
            board_width_units,
            board_height_units,
            units,
            static_obstacles: Vec::new(),
        }
    }

    #[test]
    fn sync_units_creates_kinematic_bodies_and_ball_colliders() {
        let unit_id = Uuid::from_u128(1).into();
        let unit = unit(1, WorldVec2::new(1.25, 2.5));
        let mut world = RapierMovementWorld::new();

        let report = world.sync_units(&[unit]);

        assert_eq!(
            report,
            RapierSyncReport {
                created: 1,
                updated: 0,
                removed: 0,
            }
        );
        assert_eq!(world.len(), 1);
        assert_eq!(
            world.rapier_position_for(unit_id),
            Some(WorldVec2::new(1.25, 2.5))
        );
        let handles = world.handles_for(unit_id).expect("missing handles");
        assert!(world.rigid_bodies().contains(handles.rigid_body));
        assert!(world.colliders().contains(handles.collider));
    }

    #[test]
    fn sync_units_updates_existing_body_position() {
        let unit_id = Uuid::from_u128(2).into();
        let mut unit = unit(2, WorldVec2::new(1.0, 1.0));
        let mut world = RapierMovementWorld::new();
        world.sync_units(&[unit.clone()]);

        unit.body.position = WorldVec2::new(3.0, 4.0);
        let report = world.sync_units(&[unit]);

        assert_eq!(report.updated, 1);
        assert_eq!(
            world.rapier_position_for(unit_id),
            Some(WorldVec2::new(3.0, 4.0))
        );
    }

    #[test]
    fn sync_units_removes_dead_and_absent_units() {
        let mut first = unit(3, WorldVec2::new(1.0, 1.0));
        let second = unit(4, WorldVec2::new(2.0, 2.0));
        let first_id = first.unit_id;
        let second_id = second.unit_id;
        let mut world = RapierMovementWorld::new();
        world.sync_units(&[first.clone(), second.clone()]);

        first.is_dead = true;
        let report = world.sync_units(&[first]);

        assert_eq!(report.removed, 2);
        assert!(world.handles_for(first_id).is_none());
        assert!(world.handles_for(second_id).is_none());
        assert!(world.is_empty());
    }

    #[test]
    fn sync_board_bounds_creates_four_static_wall_colliders() {
        let mut world = RapierMovementWorld::new();

        world.sync_board_bounds(4.0, 3.0);

        let bounds = world.board_bounds().expect("missing board bounds");
        assert_eq!(bounds.width_units, 4.0);
        assert_eq!(bounds.height_units, 3.0);
        assert_eq!(bounds.colliders.len(), 4);
        assert!(world.rigid_bodies().contains(bounds.rigid_body));
        for collider in &bounds.colliders {
            assert!(world.colliders().contains(*collider));
        }
    }

    #[test]
    fn rapier_ground_correction_ignores_board_wall_colliders() {
        let mover_id = Uuid::from_u128(90).into();
        let mut mover = unit(90, WorldVec2::new(1.0, 1.0));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });

        let mut world = RapierMovementWorld::new();
        world.sync_board_bounds(2.0, 2.0);
        let result = world.tick(movement_input(2_000, vec![mover]));

        let moved_to = result
            .outputs
            .iter()
            .find_map(|output| match output {
                MovementOutput::BodyMoved {
                    unit_id: moved, to, ..
                } if *moved == mover_id => Some(*to),
                _ => None,
            })
            .expect("missing BodyMoved output");

        assert_eq!(
            moved_to,
            WorldVec2::new(3.0, 1.0),
            "runtime movement should ignore stale/test board wall colliders and use core clamp"
        );
    }

    #[test]
    fn sync_static_obstacles_creates_updates_and_removes_fixed_colliders() {
        let mut world = RapierMovementWorld::new();
        let mut obstacle = MovementStaticObstacle {
            obstacle_id: 10,
            center: WorldVec2::new(2.0, 2.0),
            half_extents: WorldVec2::new(0.5, 0.5),
        };

        world.sync_static_obstacles(&[obstacle]);

        assert_eq!(world.static_obstacle_count(), 1);

        obstacle.center = WorldVec2::new(3.0, 2.0);
        world.sync_static_obstacles(&[obstacle]);

        assert_eq!(world.static_obstacle_count(), 1);

        world.sync_static_obstacles(&[]);

        assert_eq!(world.static_obstacle_count(), 0);
    }

    #[test]
    fn rapier_engine_moves_toward_point_goal() {
        let unit_id = Uuid::from_u128(5).into();
        let mut mover = unit(5, WorldVec2::new(1.0, 1.0));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });

        let mut world = RapierMovementWorld::new();
        let result = world.tick(movement_input(500, vec![mover]));

        assert!(result.outputs.iter().any(|output| matches!(
            output,
            MovementOutput::BodyMoved { unit_id: moved, to, .. }
                if *moved == unit_id && *to == WorldVec2::new(1.5, 1.0)
        )));
        assert_eq!(
            world.rapier_position_for(unit_id),
            Some(WorldVec2::new(1.5, 1.0))
        );
    }

    #[test]
    fn rapier_engine_ignores_unit_colliders_when_correcting_translation() {
        let mover_id = Uuid::from_u128(6).into();
        let blocker_id = Uuid::from_u128(7).into();
        let mut mover = unit(6, WorldVec2::new(1.0, 1.0));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });

        let mut blocker = unit(7, WorldVec2::new(1.9, 1.0));
        blocker.unit_id = blocker_id;
        blocker.can_move = false;

        let mut world = RapierMovementWorld::new();
        let result = world.tick(movement_input(2_000, vec![mover, blocker]));

        let moved_to = result
            .outputs
            .iter()
            .find_map(|output| match output {
                MovementOutput::BodyMoved {
                    unit_id: moved, to, ..
                } if *moved == mover_id => Some(*to),
                _ => None,
            })
            .expect("missing BodyMoved output");

        assert_eq!(moved_to, WorldVec2::new(3.0, 1.0));
    }

    #[test]
    fn rapier_engine_static_obstacle_blocks_corrected_translation() {
        let mover_id = Uuid::from_u128(70).into();
        let mut mover = unit(70, WorldVec2::new(1.0, 1.0));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });

        let mut input = movement_input(2_000, vec![mover]);
        input.static_obstacles.push(MovementStaticObstacle {
            obstacle_id: 70,
            center: WorldVec2::new(2.0, 1.0),
            half_extents: WorldVec2::new(0.25, 0.5),
        });

        let mut world = RapierMovementWorld::new();
        let result = world.tick(input);

        let moved_to = result
            .outputs
            .iter()
            .find_map(|output| match output {
                MovementOutput::BodyMoved {
                    unit_id: moved, to, ..
                } if *moved == mover_id => Some(*to),
                _ => None,
            })
            .expect("missing BodyMoved output");

        assert!(
            moved_to.x < 1.75,
            "expected static obstacle to block movement before its left edge: moved_to={moved_to:?}"
        );
        assert_eq!(world.static_obstacle_count(), 1);
    }

    #[test]
    fn rapier_ground_static_obstacle_does_not_slide_along_blocking_wall() {
        let mover_id = Uuid::from_u128(73).into();
        let mut mover = unit(73, WorldVec2::new(1.0, 1.0));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(5.0, 3.0),
            stop_radius: 0.1,
        });

        let mut input = movement_input(3_000, vec![mover]);
        input.static_obstacles.push(MovementStaticObstacle {
            obstacle_id: 73,
            center: WorldVec2::new(2.0, 2.0),
            half_extents: WorldVec2::new(0.1, 5.0),
        });

        let mut world = RapierMovementWorld::new();
        let result = world.tick(input);

        let moved_to = result
            .outputs
            .iter()
            .find_map(|output| match output {
                MovementOutput::BodyMoved {
                    unit_id: moved, to, ..
                } if *moved == mover_id => Some(*to),
                _ => None,
            })
            .expect("missing BodyMoved output");

        assert!(
            moved_to.x < 1.55 && moved_to.y < 1.35,
            "ground obstacle response should stop at the blocking wall instead of sliding along it: {moved_to:?}"
        );
    }

    #[test]
    fn rapier_engine_airborne_policy_ignores_static_obstacles() {
        let mover_id = Uuid::from_u128(71).into();
        let mut mover = unit(71, WorldVec2::new(1.0, 1.0));
        mover.terrain_policy = MovementTerrainPolicy::Airborne;
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });

        let mut input = movement_input(2_000, vec![mover]);
        input.static_obstacles.push(MovementStaticObstacle {
            obstacle_id: 71,
            center: WorldVec2::new(2.0, 1.0),
            half_extents: WorldVec2::new(0.25, 0.5),
        });

        let mut world = RapierMovementWorld::new();
        let result = world.tick(input);

        let moved_to = result
            .outputs
            .iter()
            .find_map(|output| match output {
                MovementOutput::BodyMoved {
                    unit_id: moved, to, ..
                } if *moved == mover_id => Some(*to),
                _ => None,
            })
            .expect("missing BodyMoved output");

        assert_eq!(moved_to, WorldVec2::new(3.0, 1.0));
    }

    #[test]
    fn rapier_engine_airborne_policy_still_clamps_to_board_bounds() {
        let mover_id = Uuid::from_u128(72).into();
        let mut mover = unit(72, WorldVec2::new(1.5, 1.5));
        mover.terrain_policy = MovementTerrainPolicy::Airborne;
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(-10.0, -10.0),
            stop_radius: 0.1,
        });

        let mut world = RapierMovementWorld::new();
        let result = world.tick(bounded_movement_input(2_000, 2.0, 2.0, vec![mover]));

        let moved_to = result
            .outputs
            .iter()
            .find_map(|output| match output {
                MovementOutput::BodyMoved {
                    unit_id: moved, to, ..
                } if *moved == mover_id => Some(*to),
                _ => None,
            })
            .expect("missing BodyMoved output");

        assert!(
            moved_to.distance(WorldVec2::new(0.35, 0.35)) <= f32::EPSILON,
            "expected airborne movement to clamp to board bounds, got {moved_to:?}"
        );
    }

    #[test]
    fn rapier_engine_does_not_depenetrate_overlapping_units() {
        let first_id = Uuid::from_u128(60).into();
        let second_id = Uuid::from_u128(61).into();
        let first = unit(60, WorldVec2::new(2.0, 2.0));
        let second = unit(61, WorldVec2::new(2.0, 2.0));

        let mut world = RapierMovementWorld::new();
        let result = world.tick(movement_input(50, vec![first, second]));

        let first_position = world
            .rapier_position_for(first_id)
            .expect("missing first position");
        let second_position = world
            .rapier_position_for(second_id)
            .expect("missing second position");

        assert_eq!(first_position, WorldVec2::new(2.0, 2.0));
        assert_eq!(second_position, WorldVec2::new(2.0, 2.0));
        assert!(!result.outputs.iter().any(|output| matches!(
            output,
            MovementOutput::BodyMoved { unit_id, .. }
                if *unit_id == first_id || *unit_id == second_id
        )));
    }

    #[test]
    fn rapier_engine_does_not_push_movable_unit_out_of_locked_body_overlap() {
        let mover_id = Uuid::from_u128(62).into();
        let blocker_id = Uuid::from_u128(63).into();
        let mover = unit(62, WorldVec2::new(1.0, 1.0));
        let mut blocker = unit(63, WorldVec2::new(1.4, 1.0));
        blocker.can_move = false;

        let mut world = RapierMovementWorld::new();
        world.tick(movement_input(50, vec![mover, blocker]));

        let mover_position = world
            .rapier_position_for(mover_id)
            .expect("missing mover position");
        let blocker_position = world
            .rapier_position_for(blocker_id)
            .expect("missing blocker position");

        assert_eq!(mover_position, WorldVec2::new(1.0, 1.0));
        assert_eq!(blocker_position, WorldVec2::new(1.4, 1.0));
    }

    #[test]
    fn rapier_engine_keeps_movement_inside_board_bounds() {
        let mover_id = Uuid::from_u128(8).into();
        let mut mover = unit(8, WorldVec2::new(1.5, 1.5));
        mover.body.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(-10.0, -10.0),
            stop_radius: 0.1,
        });

        let mut world = RapierMovementWorld::new();
        let result = world.tick(bounded_movement_input(2_000, 2.0, 2.0, vec![mover]));

        let moved_to = result
            .outputs
            .iter()
            .find_map(|output| match output {
                MovementOutput::BodyMoved {
                    unit_id: moved, to, ..
                } if *moved == mover_id => Some(*to),
                _ => None,
            })
            .expect("missing BodyMoved output");

        assert!(moved_to.x >= 0.35 && moved_to.x <= 1.65);
        assert!(moved_to.y >= 0.35 && moved_to.y <= 1.65);
        assert!(
            world.board_bounds().is_none(),
            "runtime movement should use core clamp as the board-bound source of truth"
        );
    }
}
