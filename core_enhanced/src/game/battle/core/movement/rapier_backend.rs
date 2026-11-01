use std::collections::{HashMap, HashSet};

use rapier2d::control::{CharacterLength, KinematicCharacterController};
use rapier2d::prelude::{
    BroadPhaseBvh, BroadPhasePairEvent, Collider, ColliderBuilder, ColliderHandle, ColliderSet,
    ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase,
    QueryFilter, RigidBodyBuilder, RigidBodyHandle, RigidBodySet, Vector,
};

use crate::game::battle::ids::UnitInstanceId;
use crate::{authoring::control, game::behavior::GameError};

use super::engine::{
    clamp_to_board, run_continuous_movement_tick, ContinuousMovementResolver, MovementEngine,
    MovementStaticObstacle, MovementTickInput, MovementTickResult, MovementUnitInput,
};
use super::types::WorldVec2;

/// Rapier-side handles owned by the continuous movement backend.
///
/// `RuntimeUnit` keeps gameplay body state. Rapier handles stay inside this
/// backend so physics internals do not leak into combat rules.
#[derive(Debug, Clone, Copy)]
struct RapierUnitHandles {
    rigid_body: RigidBodyHandle,
    collider: ColliderHandle,
    radius: f32,
}

#[derive(Debug, Clone, Copy)]
struct RapierStaticObstacleHandles {
    rigid_body: RigidBodyHandle,
    center: WorldVec2,
    half_extents: WorldVec2,
}

/// Captures retained native history from construction, including ordinary calls
/// before a later authoring fork. Counters never drive physics or gameplay IDs.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct CloneHistory {
    body_peak: usize,
    collider_peak: usize,
    body_insertions: usize,
    collider_insertions: usize,
    collider_removals: usize,
    pair_events: usize,
}

// Both pinned hashbrown table versions can resize to at most twice the insertion
// count, then round for load factor and buckets. Unlike capacity(), this bound
// remains valid after tombstone churn and retained-table clear.
fn insertion_bucket_bound(insertions: usize) -> Option<usize> {
    if insertions == 0 {
        return Some(0);
    }
    Some(
        (insertions.checked_mul(2)?.checked_mul(8)? / 7)
            .checked_next_power_of_two()?
            .max(16),
    )
}

impl CloneHistory {
    fn inserted(&mut self, bodies: usize, colliders: usize) {
        self.body_peak = self.body_peak.max(bodies);
        self.collider_peak = self.collider_peak.max(colliders);
        self.body_insertions = self.body_insertions.saturating_add(1);
        self.collider_insertions = self.collider_insertions.saturating_add(1);
    }
    fn removed_colliders(&mut self, count: usize) {
        self.collider_removals = self.collider_removals.saturating_add(count);
    }
    fn storage_slots(&self) -> Option<usize> {
        // Pinned Rapier Arena initializes four slots, doubles on exhaustion and
        // keeps freed slots. BroadPhase's pinned hashbrown table retains buckets.
        let body_arena = self.body_peak.checked_next_power_of_two()?.max(4);
        let collider_arena = self.collider_peak.checked_next_power_of_two()?.max(4);
        let pair_buckets = insertion_bucket_bound(self.pair_events)?;
        // Nodes+parents <=2H, retained refit_tmp <=2H, rebuild_leaves <=H,
        // and the indexed leaf map <= collider arena's initialized slots.
        let bvh = self
            .collider_peak
            .checked_mul(5)?
            .checked_add(collider_arena)?;
        [
            body_arena,
            collider_arena,
            8,
            self.body_insertions,
            self.collider_insertions,
            self.collider_removals,
            bvh,
            pair_buckets,
        ]
        .into_iter()
        .try_fold(0_usize, usize::checked_add)
    }
}

#[derive(Clone)]
pub(in crate::game::battle::core) struct RapierMovementWorld {
    islands: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    rigid_bodies: RigidBodySet,
    colliders: ColliderSet,
    // This backend never creates joints. Rapier only needs these empty
    // collections when removing a body; defer their reserved graph storage.
    impulse_joints: Option<ImpulseJointSet>,
    multibody_joints: Option<MultibodyJointSet>,
    controller: KinematicCharacterController,
    unit_handles: HashMap<UnitInstanceId, RapierUnitHandles>,
    static_obstacles: HashMap<u64, RapierStaticObstacleHandles>,
    clone_history: CloneHistory,
    // These maps retain their tables when clear() replaces the Rapier owners.
    unit_map_insertions: usize,
    obstacle_map_insertions: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct RapierSyncReport {
    created: usize,
    updated: usize,
    removed: usize,
}

impl crate::authoring::clone_shape::CloneShape for RapierMovementWorld {
    fn visit_clone_shape(&self, meter: &mut control::CloneMeter) -> Result<(), GameError> {
        // The pinned storage model is O(1); no backend traversal or clone is used
        // to measure it. The enclosing whole-owner finish pays its native copy.
        meter.scan_storage(Some(1))?;
        self.visit_authoring_clone_storage(meter);
        meter.admit_storage("owned_clone_storage")
    }
}

impl RapierMovementWorld {
    /// Structural admission, not exact native allocation bytes. This accounts
    /// only for this backend; the enclosing complete clone owns other fields.
    pub(in crate::game::battle::core) fn preflight_authoring_clone(&self) -> Result<(), GameError> {
        if !control::is_active() {
            return Ok(());
        }
        control::poll("rapier_clone_entry")?;
        let mut meter = control::CloneMeter::default();
        self.visit_authoring_clone_storage(&mut meter);
        meter.finish("rapier_clone_storage", "rapier_clone_storage")
    }

    pub(in crate::game::battle::core) fn visit_authoring_clone_storage(
        &self,
        meter: &mut control::CloneMeter,
    ) {
        let Self {
            islands: _,
            broad_phase: _,
            narrow_phase: _,
            rigid_bodies: _,
            colliders: _,
            impulse_joints: _,
            multibody_joints: _,
            controller: _,
            unit_handles: _,
            static_obstacles: _,
            clone_history,
            unit_map_insertions,
            obstacle_map_insertions,
        } = self;
        meter.add_storage(
            clone_history
                .storage_slots()
                .and_then(|n| n.checked_add(insertion_bucket_bound(*unit_map_insertions)?))
                .and_then(|n| n.checked_add(insertion_bucket_bound(*obstacle_map_insertions)?)),
        );
    }
    #[cfg(test)]
    fn new() -> Self {
        Self::default()
    }

    fn corrected_ground_static_obstacle_translation_for(
        &mut self,
        unit_id: UnitInstanceId,
        desired_translation: WorldVec2,
        dt_seconds: f32,
    ) -> Option<WorldVec2> {
        self.refresh_broad_phase();

        let handles = self.handles_for(unit_id)?;
        let collider = self.colliders.get(handles.collider)?;
        let body = self.rigid_bodies.get(handles.rigid_body)?;
        let ignored_colliders = self
            .unit_handles
            .values()
            .map(|handles| handles.collider)
            .collect::<HashSet<_>>();
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

    #[cfg(test)]
    fn rapier_position_for(&self, unit_id: UnitInstanceId) -> Option<WorldVec2> {
        let handles = self.handles_for(unit_id)?;
        let body = self.rigid_bodies.get(handles.rigid_body)?;
        Some(from_rapier_vector(body.translation()))
    }

    fn handles_for(&self, unit_id: UnitInstanceId) -> Option<RapierUnitHandles> {
        self.unit_handles.get(&unit_id).copied()
    }

    fn forget_unit(&mut self, unit_id: UnitInstanceId) -> Option<RapierUnitHandles> {
        let handles = self.unit_handles.remove(&unit_id)?;
        let previous_colliders = self.colliders.len();
        self.rigid_bodies.remove(
            handles.rigid_body,
            &mut self.islands,
            &mut self.colliders,
            self.impulse_joints.get_or_insert_with(ImpulseJointSet::new),
            self.multibody_joints
                .get_or_insert_with(MultibodyJointSet::new),
            true,
        );
        self.clone_history
            .removed_colliders(previous_colliders - self.colliders.len());
        Some(handles)
    }

    fn sync_units(&mut self, units: &[MovementUnitInput]) -> RapierSyncReport {
        let mut report = RapierSyncReport::default();

        for unit in units {
            if unit.is_dead {
                if self.forget_unit(unit.unit_id).is_some() {
                    report.removed += 1;
                }
                continue;
            }

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
            .filter(|unit_id| {
                !units
                    .iter()
                    .any(|unit| !unit.is_dead && unit.unit_id == *unit_id)
            })
            .collect();
        for unit_id in stale_unit_ids {
            if self.forget_unit(unit_id).is_some() {
                report.removed += 1;
            }
        }

        report
    }

    pub(in crate::game::battle::core) fn clear(&mut self) {
        self.islands = IslandManager::new();
        self.broad_phase = BroadPhaseBvh::new();
        self.narrow_phase = NarrowPhase::new();
        self.rigid_bodies = RigidBodySet::new();
        self.colliders = ColliderSet::new();
        self.impulse_joints = None;
        self.multibody_joints = None;
        self.unit_handles.clear();
        self.static_obstacles.clear();
        self.clone_history = CloneHistory::default();
    }

    fn sync_static_obstacles(&mut self, obstacles: &[MovementStaticObstacle]) {
        if obstacles.is_empty() && self.static_obstacles.is_empty() {
            return;
        }
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

    fn insert_unit(&mut self, unit: &MovementUnitInput) {
        let body = RigidBodyBuilder::kinematic_position_based()
            .translation(to_rapier_vector(unit.body.position))
            .build();
        let rigid_body = self.rigid_bodies.insert(body);
        let collider = ColliderBuilder::ball(unit.body.radius.max(0.0)).build();
        let collider =
            self.colliders
                .insert_with_parent(collider, rigid_body, &mut self.rigid_bodies);
        self.clone_history
            .inserted(self.rigid_bodies.len(), self.colliders.len());

        self.unit_handles.insert(
            unit.unit_id,
            RapierUnitHandles {
                rigid_body,
                collider,
                radius: unit.body.radius,
            },
        );
        self.unit_map_insertions = self.unit_map_insertions.saturating_add(1);
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
        self.colliders
            .insert_with_parent(collider, rigid_body, &mut self.rigid_bodies);
        self.clone_history
            .inserted(self.rigid_bodies.len(), self.colliders.len());
        self.static_obstacles.insert(
            obstacle.obstacle_id,
            RapierStaticObstacleHandles {
                rigid_body,
                center: obstacle.center,
                half_extents: obstacle.half_extents,
            },
        );
        self.obstacle_map_insertions = self.obstacle_map_insertions.saturating_add(1);
    }

    fn remove_static_obstacle(&mut self, obstacle_id: u64) {
        let Some(handles) = self.static_obstacles.remove(&obstacle_id) else {
            return;
        };
        let previous_colliders = self.colliders.len();
        self.rigid_bodies.remove(
            handles.rigid_body,
            &mut self.islands,
            &mut self.colliders,
            self.impulse_joints.get_or_insert_with(ImpulseJointSet::new),
            self.multibody_joints
                .get_or_insert_with(MultibodyJointSet::new),
            true,
        );
        self.clone_history
            .removed_colliders(previous_colliders - self.colliders.len());
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
            if body.translation().x == unit.body.position.x
                && body.translation().y == unit.body.position.y
            {
                return;
            }
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
        self.clone_history.pair_events =
            self.clone_history.pair_events.saturating_add(events.len());
    }

    fn apply_unit_translation(
        &mut self,
        unit_id: UnitInstanceId,
        position: WorldVec2,
    ) -> Option<()> {
        let handles = self.handles_for(unit_id)?;
        let body = self.rigid_bodies.get_mut(handles.rigid_body)?;
        if body.translation().x == position.x && body.translation().y == position.y {
            return Some(());
        }
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
            impulse_joints: None,
            multibody_joints: None,
            controller: KinematicCharacterController {
                offset: CharacterLength::Absolute(0.001),
                slide: false,
                autostep: None,
                snap_to_ground: None,
                ..KinematicCharacterController::default()
            },
            unit_handles: HashMap::new(),
            static_obstacles: HashMap::new(),
            clone_history: CloneHistory::default(),
            unit_map_insertions: 0,
            obstacle_map_insertions: 0,
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
        static_obstacles: &[MovementStaticObstacle],
        dt_seconds: f32,
    ) -> WorldVec2 {
        let desired_translation = desired_to - unit.body.position;
        let corrected_translation =
            if unit.terrain_policy.applies_static_obstacles() && !static_obstacles.is_empty() {
                self.corrected_ground_static_obstacle_translation_for(
                    unit.unit_id,
                    desired_translation,
                    dt_seconds,
                )
                .unwrap_or(desired_translation)
            } else {
                desired_translation
            };
        clamp_to_board(
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

    use crate::game::battle::core::movement::{
        engine::{
            MovementEngine, MovementOutput, MovementStaticObstacle, MovementTerrainPolicy,
            MovementTickInput, MovementUnitInput,
        },
        types::{MovementGoal, UnitBody, WorldVec2},
    };

    use super::*;

    fn unit(unit_id: u128, position: WorldVec2) -> MovementUnitInput {
        MovementUnitInput {
            unit_id: Uuid::from_u128(unit_id).into(),
            body: UnitBody::new_at(position, 0.35, 1.0),
            goal: None,
            terrain_policy: MovementTerrainPolicy::Ground,
            can_move: true,
            is_dead: false,
        }
    }

    fn movement_input(dt_ms: u64, units: Vec<MovementUnitInput>) -> MovementTickInput {
        MovementTickInput {
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
            dt_ms,
            board_width_units,
            board_height_units,
            units,
            static_obstacles: Vec::new(),
        }
    }

    #[test]
    fn rapier_engine_canonicalizes_unit_order_before_resolution() {
        let first_id = Uuid::from_u128(1).into();
        let mut first = unit(1, WorldVec2::new(1.0, 1.0));
        let mut second = unit(2, WorldVec2::new(3.0, 1.0));
        first.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(2.0, 1.0),
            stop_radius: 0.1,
        });
        second.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(4.0, 1.0),
            stop_radius: 0.1,
        });

        let mut reversed_world = RapierMovementWorld::new();
        let reversed =
            reversed_world.tick(movement_input(500, vec![second.clone(), first.clone()]));
        let mut canonical_world = RapierMovementWorld::new();
        let canonical = canonical_world.tick(movement_input(500, vec![first, second]));

        assert_eq!(reversed, canonical);
        assert!(matches!(
            reversed.outputs.first(),
            Some(MovementOutput::BodyMoved { unit_id, .. }) if *unit_id == first_id
        ));
    }

    #[test]
    fn native_empty_joint_sets_are_lazy_and_clone_removal_is_independent() {
        let mut world = RapierMovementWorld::new();
        let sample = unit(71, WorldVec2::new(1.0, 1.0));
        world.sync_units(&[sample.clone()]);
        assert!(world.impulse_joints.is_none());
        assert!(world.multibody_joints.is_none());
        let mut branch = world.clone();
        branch.forget_unit(sample.unit_id).unwrap();
        assert!(branch.impulse_joints.as_ref().unwrap().is_empty());
        assert!(branch
            .multibody_joints
            .as_ref()
            .unwrap()
            .iter()
            .next()
            .is_none());
        assert!(world.handles_for(sample.unit_id).is_some());
        assert!(world.impulse_joints.is_none());
        assert!(world.multibody_joints.is_none());
        branch.clear();
        assert!(branch.impulse_joints.is_none());
        assert!(branch.multibody_joints.is_none());
        branch.sync_units(&[sample]);
        assert_eq!(branch.rigid_bodies.len(), 1);
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
        assert_eq!(world.unit_handles.len(), 1);
        assert_eq!(
            world.rapier_position_for(unit_id),
            Some(WorldVec2::new(1.25, 2.5))
        );
        let handles = world.handles_for(unit_id).expect("missing handles");
        assert!(world.rigid_bodies.contains(handles.rigid_body));
        assert!(world.colliders.contains(handles.collider));
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
        assert!(world.unit_handles.is_empty());
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

        assert_eq!(world.static_obstacles.len(), 1);

        obstacle.center = WorldVec2::new(3.0, 2.0);
        world.sync_static_obstacles(&[obstacle]);

        assert_eq!(world.static_obstacles.len(), 1);

        world.sync_static_obstacles(&[]);

        assert_eq!(world.static_obstacles.len(), 0);
    }

    #[test]
    fn rapier_engine_moves_toward_point_goal() {
        let unit_id = Uuid::from_u128(5).into();
        let mut mover = unit(5, WorldVec2::new(1.0, 1.0));
        mover.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });

        let mut world = RapierMovementWorld::new();
        let result = world.tick(movement_input(500, vec![mover]));

        assert!(result.outputs.iter().any(|output| matches!(
            output,
            MovementOutput::BodyMoved {
                unit_id: moved,
                to,
                ..
            } if *moved == unit_id
                && *to == WorldVec2::new(1.5, 1.0)
        )));
        assert_eq!(
            world.rapier_position_for(unit_id),
            Some(WorldVec2::new(1.5, 1.0))
        );
    }

    #[test]
    fn rapier_engine_reports_attack_target_reached() {
        let mover_id = Uuid::from_u128(50).into();
        let target_id = Uuid::from_u128(51).into();
        let mut mover = unit(50, WorldVec2::new(0.0, 0.0));
        mover.goal = Some(MovementGoal::AttackUnit { target_id });
        let target = unit(51, WorldVec2::new(1.6, 0.0));

        let mut world = RapierMovementWorld::new();
        let result = world.tick(movement_input(50, vec![mover, target]));

        assert!(result.outputs.iter().any(|output| matches!(
            output,
            MovementOutput::TargetReached {
                unit_id,
                target_id: reached,
            } if *unit_id == mover_id && *reached == target_id
        )));
    }

    #[test]
    fn rapier_engine_allows_moving_candidates_to_overlap() {
        let first_id = Uuid::from_u128(52).into();
        let second_id = Uuid::from_u128(53).into();
        let mut first = unit(52, WorldVec2::new(1.0, 1.0));
        let mut second = unit(53, WorldVec2::new(1.0, 1.0));
        first.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(10.0, 1.0),
            stop_radius: 0.1,
        });
        second.goal = first.goal;

        let mut world = RapierMovementWorld::new();
        let result = world.tick(movement_input(500, vec![first, second]));
        let moved_to = |unit_id| {
            result.outputs.iter().find_map(|output| match output {
                MovementOutput::BodyMoved {
                    unit_id: moved, to, ..
                } if *moved == unit_id => Some(*to),
                _ => None,
            })
        };

        assert_eq!(moved_to(first_id), Some(WorldVec2::new(1.5, 1.0)));
        assert_eq!(moved_to(second_id), Some(WorldVec2::new(1.5, 1.0)));
    }

    #[test]
    fn rapier_engine_ignores_unit_colliders_when_correcting_translation() {
        let mover_id = Uuid::from_u128(6).into();
        let blocker_id = Uuid::from_u128(7).into();
        let mut mover = unit(6, WorldVec2::new(1.0, 1.0));
        mover.goal = Some(MovementGoal::MoveToPoint {
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
        mover.goal = Some(MovementGoal::MoveToPoint {
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
        assert_eq!(world.static_obstacles.len(), 1);
    }

    #[test]
    fn rapier_ground_static_obstacle_does_not_slide_along_blocking_wall() {
        let mover_id = Uuid::from_u128(73).into();
        let mut mover = unit(73, WorldVec2::new(1.0, 1.0));
        mover.goal = Some(MovementGoal::MoveToPoint {
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
        mover.goal = Some(MovementGoal::MoveToPoint {
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
        mover.goal = Some(MovementGoal::MoveToPoint {
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
    fn rapier_engine_allows_overlapping_units_at_board_bounds() {
        let first_id = Uuid::from_u128(64).into();
        let second_id = Uuid::from_u128(65).into();
        let mut first = unit(64, WorldVec2::new(0.35, 0.35));
        let mut second = unit(65, WorldVec2::new(0.35, 0.35));
        first.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(-10.0, 0.35),
            stop_radius: 0.1,
        });
        second.goal = first.goal;

        let mut world = RapierMovementWorld::new();
        let result = world.tick(bounded_movement_input(500, 2.0, 2.0, vec![first, second]));

        assert!(result.outputs.is_empty());
        assert_eq!(
            world.rapier_position_for(first_id),
            Some(WorldVec2::new(0.35, 0.35))
        );
        assert_eq!(
            world.rapier_position_for(second_id),
            Some(WorldVec2::new(0.35, 0.35))
        );
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
    fn authoring_clone_admits_retained_history_before_native_copy_and_preserves_continuation() {
        use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut world = RapierMovementWorld::new();
        let sample = unit(1, WorldVec2::new(1.0, 1.0));
        for _ in 0..64 {
            world.insert_unit(&sample);
            world.forget_unit(sample.unit_id).unwrap();
        }
        assert_eq!(world.rigid_bodies.len(), 0);
        assert_eq!(world.clone_history.body_peak, 1);
        assert_eq!(world.clone_history.body_insertions, 64);
        assert_eq!(world.clone_history.collider_removals, 64);
        world.insert_unit(&sample);
        let (result, usage) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                world.preflight_authoring_clone()?;
                Ok::<_, GameError>(world.clone())
            },
        )
        .unwrap();
        let mut controlled_clone = result.unwrap();
        assert!(usage.clone_slots_peak > 64 * 3);
        assert_eq!(controlled_clone.clone_history, world.clone_history);
        let exact = ExecutionLimits {
            max_clone_slots: usage.clone_slots_peak,
            ..Default::default()
        };
        control::run(
            &exact,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || world.preflight_authoring_clone(),
        )
        .unwrap()
        .0
        .unwrap();
        let did_copy = std::cell::Cell::new(false);
        let failure = control::run(
            &ExecutionLimits {
                max_clone_slots: usage.clone_slots_peak - 1,
                ..exact
            },
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                world.preflight_authoring_clone()?;
                let copy = world.clone();
                did_copy.set(true);
                Ok::<_, GameError>(copy)
            },
        );
        assert_eq!(failure.err().unwrap().code, "authoring_clone_limit");
        assert!(!did_copy.get());
        let mut mover = sample;
        mover.goal = Some(MovementGoal::MoveToPoint {
            point: WorldVec2::new(3.0, 1.0),
            stop_radius: 0.1,
        });
        assert_eq!(
            world.tick(movement_input(300, vec![mover.clone()])),
            controlled_clone.tick(movement_input(300, vec![mover]))
        );
    }

    #[test]
    fn clone_history_includes_removed_population_and_pair_growth_and_resets_only_with_native_owners(
    ) {
        let mut world = RapierMovementWorld::new();
        let units: Vec<_> = (1..=33)
            .map(|id| unit(id, WorldVec2::new(1.0, 1.0)))
            .collect();
        world.sync_units(&units);
        world.refresh_broad_phase();
        assert_eq!(world.clone_history.collider_peak, 33);
        assert!(world.clone_history.pair_events > 0);
        let pair_events = world.clone_history.pair_events;
        world.sync_units(&[]);
        world.refresh_broad_phase();
        assert_eq!(world.colliders.len(), 0);
        assert_eq!(world.clone_history.collider_peak, 33);
        assert!(world.clone_history.pair_events >= pair_events);
        assert!(world.clone_history.storage_slots().unwrap() > 33);
        let map_capacity = world.unit_handles.capacity();
        assert!(map_capacity >= 33);
        world.clear();
        assert_eq!(world.clone_history, CloneHistory::default());
        assert_eq!(world.unit_handles.capacity(), map_capacity);
        assert_eq!(world.unit_map_insertions, 33);
        let map_slots = insertion_bucket_bound(world.unit_map_insertions).unwrap();
        assert!(map_slots > map_capacity);
        let limits = crate::authoring::ExecutionLimits {
            max_clone_slots: CloneHistory::default().storage_slots().unwrap() + map_capacity,
            ..Default::default()
        };
        let failure = control::run(
            &limits,
            &crate::authoring::CancellationToken::default(),
            crate::authoring::ExecutionPhase::TrialExecution,
            || world.preflight_authoring_clone(),
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
    }

    #[test]
    fn clone_history_overflow_is_rejected_in_control_without_changing_ordinary_gameplay() {
        use crate::authoring::{CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut world = RapierMovementWorld::new();
        world.clone_history.body_insertions = usize::MAX;
        let sample = unit(1, WorldVec2::new(1.0, 1.0));
        world.insert_unit(&sample);
        assert_eq!(world.rigid_bodies.len(), 1);
        assert_eq!(world.clone_history.body_insertions, usize::MAX);
        world.preflight_authoring_clone().unwrap();
        let failure = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || world.preflight_authoring_clone(),
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_clone_limit");
        assert_eq!(
            world.rapier_position_for(sample.unit_id),
            Some(sample.body.position)
        );
    }

    #[test]
    fn rapier_engine_keeps_movement_inside_board_bounds() {
        let mover_id = Uuid::from_u128(8).into();
        let mut mover = unit(8, WorldVec2::new(1.5, 1.5));
        mover.goal = Some(MovementGoal::MoveToPoint {
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
            "expected exact manual board clamp endpoint, got {moved_to:?}"
        );
    }
}
