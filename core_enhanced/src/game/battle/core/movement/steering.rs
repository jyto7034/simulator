use crate::game::battle::ids::UnitInstanceId;

use super::{
    engine::{MovementOutput, MovementStopReasonContinuous, MovementUnitInput},
    types::{MovementGoal, WorldVec2},
};

const DEFAULT_SEPARATION_MULTIPLIER: f32 = 0.0;
const DEFAULT_SEPARATION_STRENGTH: f32 = 0.0;
const DEFAULT_SIDE_BIAS_STRENGTH: f32 = 0.0;
const DEFAULT_CONGESTION_LOOKAHEAD_MULTIPLIER: f32 = 0.0;
const DEFAULT_MIN_CONGESTION_SPEED_SCALE: f32 = 1.0;
const DEFAULT_SIDE_BIAS_ACTIVATION_MULTIPLIER: f32 = 0.9;
const DEFAULT_EARLY_STRAIGHTEN_DISTANCE_MULTIPLIER: f32 = 3.0;
const DEFAULT_EARLY_STRAIGHTEN_PRESSURE: f32 = 0.45;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SteeringParams {
    pub separation_distance_multiplier: f32,
    pub separation_strength: f32,
    pub side_bias_strength: f32,
    pub congestion_lookahead_multiplier: f32,
    pub min_congestion_speed_scale: f32,
    pub side_bias_activation_multiplier: f32,
    pub early_straighten_distance_multiplier: f32,
    pub early_straighten_pressure: f32,
}

impl Default for SteeringParams {
    fn default() -> Self {
        Self {
            separation_distance_multiplier: DEFAULT_SEPARATION_MULTIPLIER,
            separation_strength: DEFAULT_SEPARATION_STRENGTH,
            side_bias_strength: DEFAULT_SIDE_BIAS_STRENGTH,
            congestion_lookahead_multiplier: DEFAULT_CONGESTION_LOOKAHEAD_MULTIPLIER,
            min_congestion_speed_scale: DEFAULT_MIN_CONGESTION_SPEED_SCALE,
            side_bias_activation_multiplier: DEFAULT_SIDE_BIAS_ACTIVATION_MULTIPLIER,
            early_straighten_distance_multiplier: DEFAULT_EARLY_STRAIGHTEN_DISTANCE_MULTIPLIER,
            early_straighten_pressure: DEFAULT_EARLY_STRAIGHTEN_PRESSURE,
        }
    }
}

pub(super) fn movement_velocity(from: WorldVec2, to: WorldVec2, dt_seconds: f32) -> WorldVec2 {
    if dt_seconds <= f32::EPSILON {
        WorldVec2::ZERO
    } else {
        WorldVec2::new((to.x - from.x) / dt_seconds, (to.y - from.y) / dt_seconds)
    }
}

pub(super) fn desired_velocity(unit: &MovementUnitInput, target: WorldVec2) -> WorldVec2 {
    let desired = target - unit.body.position;
    desired.normalized_or_zero() * unit.body.move_speed
}

pub(super) fn steered_displacement(
    unit: &MovementUnitInput,
    units: &[MovementUnitInput],
    target: WorldVec2,
    dt_seconds: f32,
    params: SteeringParams,
) -> WorldVec2 {
    let max_distance = unit.body.move_speed * dt_seconds;
    if max_distance <= f32::EPSILON {
        return WorldVec2::ZERO;
    }

    let target_delta = target - unit.body.position;
    let target_distance = target_delta.length();
    let target_direction = target_delta.normalized_or_zero();
    let mut velocity = desired_velocity(unit, target);

    if target_direction.length_squared() > f32::EPSILON {
        velocity += avoidance_velocity(unit, units, target_direction, target_distance, params);
    }

    let speed_scale = congestion_speed_scale(unit, units, target_direction, params);
    (velocity * dt_seconds).clamp_length(max_distance * speed_scale)
}

pub(super) fn goal_target_position(
    unit: &MovementUnitInput,
    units: &[MovementUnitInput],
) -> Option<WorldVec2> {
    match unit.body.goal {
        Some(MovementGoal::MoveToPoint { point, .. }) => Some(point),
        Some(MovementGoal::AttackUnit {
            target_id,
            desired_range,
            approach_point,
        }) => {
            if let Some(point) = approach_point {
                return Some(point);
            }

            let target = units
                .iter()
                .find(|candidate| candidate.unit_id == target_id)?;
            let away = (unit.body.position - target.body.position).normalized_or_zero();
            Some(target.body.position + away * desired_range)
        }
        None => None,
    }
}

pub(super) fn reached_goal(
    unit: &MovementUnitInput,
    units: &[MovementUnitInput],
) -> Option<MovementOutput> {
    match unit.body.goal {
        Some(MovementGoal::MoveToPoint { point, stop_radius }) => {
            if unit.body.position.distance(point) <= stop_radius {
                Some(MovementOutput::MovementStopped {
                    unit_id: unit.unit_id,
                    position: unit.body.position,
                    reason: MovementStopReasonContinuous::TargetReached,
                })
            } else {
                None
            }
        }
        Some(MovementGoal::AttackUnit {
            target_id,
            desired_range,
            approach_point: _,
        }) => {
            let target = units
                .iter()
                .find(|candidate| candidate.unit_id == target_id)?;
            if unit.body.can_reach(&target.body, desired_range) {
                Some(MovementOutput::TargetReached {
                    unit_id: unit.unit_id,
                    target_id,
                })
            } else {
                None
            }
        }
        None => Some(MovementOutput::MovementStopped {
            unit_id: unit.unit_id,
            position: unit.body.position,
            reason: MovementStopReasonContinuous::NoGoal,
        }),
    }
}

fn avoidance_velocity(
    unit: &MovementUnitInput,
    units: &[MovementUnitInput],
    target_direction: WorldVec2,
    target_distance: f32,
    params: SteeringParams,
) -> WorldVec2 {
    let mut avoidance = WorldVec2::ZERO;
    let side_direction = stable_side_direction(unit.unit_id, target_direction);

    for other in units {
        if other.unit_id == unit.unit_id || other.is_dead {
            continue;
        }

        let delta_from_other = unit.body.position - other.body.position;
        let distance = delta_from_other.length();
        let separation_distance =
            (unit.body.radius + other.body.radius) * params.separation_distance_multiplier.max(0.0);
        if separation_distance <= f32::EPSILON {
            continue;
        }

        if distance < separation_distance {
            let normal = if distance <= f32::EPSILON {
                stable_fallback_normal(unit.unit_id, other.unit_id)
            } else {
                delta_from_other * (1.0 / distance)
            };
            let pressure = ((separation_distance - distance) / separation_distance).clamp(0.0, 1.0);
            avoidance += normal * unit.body.move_speed * params.separation_strength * pressure;
        }

        let toward_other = other.body.position - unit.body.position;
        let forward = dot(toward_other, target_direction);
        if forward <= 0.0 || forward > separation_distance * 2.0 {
            continue;
        }

        let lateral_pressure = forward_lane_pressure(
            toward_other,
            target_direction,
            separation_distance,
            params.side_bias_activation_multiplier,
        );
        if lateral_pressure > 0.0 {
            let pressure = lateral_pressure
                * side_bias_distance_scale(target_distance, separation_distance, params);
            avoidance +=
                side_direction * unit.body.move_speed * params.side_bias_strength * pressure;
        }
    }

    avoidance
}

fn forward_lane_pressure(
    toward_other: WorldVec2,
    target_direction: WorldVec2,
    separation_distance: f32,
    activation_multiplier: f32,
) -> f32 {
    let forward = dot(toward_other, target_direction);
    if forward <= 0.0 || forward > separation_distance * activation_multiplier.max(0.0) {
        return 0.0;
    }

    let lateral = (toward_other - target_direction * forward).length();
    if lateral >= separation_distance {
        return 0.0;
    }

    let forward_pressure =
        1.0 - (forward / (separation_distance * activation_multiplier.max(0.001))).clamp(0.0, 1.0);
    let lateral_pressure = 1.0 - (lateral / separation_distance).clamp(0.0, 1.0);
    forward_pressure * lateral_pressure
}

fn side_bias_distance_scale(
    target_distance: f32,
    separation_distance: f32,
    params: SteeringParams,
) -> f32 {
    let early_distance = separation_distance * params.early_straighten_distance_multiplier.max(0.0);
    if early_distance <= f32::EPSILON || target_distance <= early_distance {
        return 1.0;
    }

    params.early_straighten_pressure.clamp(0.0, 1.0)
}

fn congestion_speed_scale(
    unit: &MovementUnitInput,
    units: &[MovementUnitInput],
    target_direction: WorldVec2,
    params: SteeringParams,
) -> f32 {
    if target_direction.length_squared() <= f32::EPSILON {
        return 1.0;
    }

    let mut strongest_pressure = 0.0_f32;
    for other in units {
        if other.unit_id == unit.unit_id || other.is_dead {
            continue;
        }

        let separation_distance =
            (unit.body.radius + other.body.radius) * params.separation_distance_multiplier.max(0.0);
        let lookahead = separation_distance * params.congestion_lookahead_multiplier.max(0.0);
        if lookahead <= f32::EPSILON {
            continue;
        }

        let toward_other = other.body.position - unit.body.position;
        let forward = dot(toward_other, target_direction);
        if forward <= 0.0 || forward > lookahead {
            continue;
        }

        let lateral = (toward_other - target_direction * forward).length();
        if lateral >= separation_distance {
            continue;
        }

        let forward_pressure = 1.0 - (forward / lookahead).clamp(0.0, 1.0);
        let lateral_pressure = 1.0 - (lateral / separation_distance).clamp(0.0, 1.0);
        strongest_pressure = strongest_pressure.max(forward_pressure * lateral_pressure);
    }

    let min_scale = params.min_congestion_speed_scale.clamp(0.0, 1.0);
    1.0 - (1.0 - min_scale) * strongest_pressure.clamp(0.0, 1.0)
}

fn stable_side_direction(unit_id: UnitInstanceId, target_direction: WorldVec2) -> WorldVec2 {
    let side = if unit_id.as_bytes()[15] % 2 == 0 {
        1.0
    } else {
        -1.0
    };
    WorldVec2::new(-target_direction.y, target_direction.x) * side
}

fn stable_fallback_normal(a: UnitInstanceId, b: UnitInstanceId) -> WorldVec2 {
    if a.as_bytes() <= b.as_bytes() {
        WorldVec2::new(-1.0, 0.0)
    } else {
        WorldVec2::new(1.0, 0.0)
    }
}

fn dot(a: WorldVec2, b: WorldVec2) -> f32 {
    a.x * b.x + a.y * b.y
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::game::{battle::ids::UnitInstanceId, enums::Side};

    use super::*;
    use crate::game::battle::core::movement::{engine::MovementTerrainPolicy, types::UnitBody};

    fn movement_unit(unit_id: u128, position: WorldVec2, goal: MovementGoal) -> MovementUnitInput {
        MovementUnitInput {
            unit_id: UnitInstanceId::from(Uuid::from_u128(unit_id)),
            owner: Side::Player,
            body: UnitBody {
                position,
                previous_position: position,
                velocity: WorldVec2::ZERO,
                radius: 0.35,
                move_speed: 1.0,
                mode: Default::default(),
                goal: Some(goal),
                physics_handle: None,
            },
            terrain_policy: MovementTerrainPolicy::Ground,
            current_target: None,
            attack_range_units: 0.5,
            can_move: true,
            is_dead: false,
        }
    }

    #[test]
    fn steering_adds_lateral_bias_when_forward_lane_is_crowded() {
        let target = WorldVec2::new(4.0, 0.0);
        let mover = movement_unit(
            2,
            WorldVec2::new(0.0, 0.0),
            MovementGoal::MoveToPoint {
                point: target,
                stop_radius: 0.1,
            },
        );
        let blocker = movement_unit(
            3,
            WorldVec2::new(0.25, 0.0),
            MovementGoal::MoveToPoint {
                point: target,
                stop_radius: 0.1,
            },
        );
        let units = vec![mover.clone(), blocker];

        let displacement = steered_displacement(
            &mover,
            &units,
            target,
            0.05,
            SteeringParams {
                separation_distance_multiplier: 1.15,
                separation_strength: 0.0,
                side_bias_strength: 1.0,
                congestion_lookahead_multiplier: 2.25,
                min_congestion_speed_scale: 0.35,
                side_bias_activation_multiplier: 0.9,
                early_straighten_distance_multiplier: 3.0,
                early_straighten_pressure: 0.45,
            },
        );

        assert!(
            displacement.x > 0.0,
            "steering should still make forward progress: {displacement:?}"
        );
        assert!(
            displacement.y.abs() > f32::EPSILON,
            "steering should add lateral movement instead of pushing straight into the blocker: {displacement:?}"
        );
    }

    #[test]
    fn steering_keeps_early_far_movement_straighter_when_lane_pressure_is_weak() {
        let target = WorldVec2::new(6.0, 0.0);
        let mover = movement_unit(
            8,
            WorldVec2::new(0.0, 0.0),
            MovementGoal::MoveToPoint {
                point: target,
                stop_radius: 0.1,
            },
        );
        let weak_forward_neighbor = movement_unit(
            9,
            WorldVec2::new(1.0, 0.0),
            MovementGoal::MoveToPoint {
                point: target,
                stop_radius: 0.1,
            },
        );
        let units = vec![mover.clone(), weak_forward_neighbor];

        let displacement = steered_displacement(
            &mover,
            &units,
            target,
            0.05,
            SteeringParams {
                separation_distance_multiplier: 1.15,
                separation_strength: 0.0,
                side_bias_strength: 1.0,
                congestion_lookahead_multiplier: 2.25,
                min_congestion_speed_scale: 0.35,
                side_bias_activation_multiplier: 0.9,
                early_straighten_distance_multiplier: 3.0,
                early_straighten_pressure: 0.45,
            },
        );

        assert!(
            displacement.y.abs() <= f32::EPSILON,
            "weak early forward pressure should not create visible side-to-side wobble: {displacement:?}"
        );
    }

    #[test]
    fn steering_keeps_full_speed_near_goal() {
        let target = WorldVec2::new(0.5, 0.0);
        let mover = movement_unit(
            4,
            WorldVec2::new(0.0, 0.0),
            MovementGoal::MoveToPoint {
                point: target,
                stop_radius: 0.1,
            },
        );
        let units = vec![mover.clone()];

        let displacement = steered_displacement(
            &mover,
            &units,
            target,
            0.5,
            SteeringParams {
                separation_distance_multiplier: 1.15,
                separation_strength: 0.0,
                side_bias_strength: 0.0,
                congestion_lookahead_multiplier: 2.25,
                min_congestion_speed_scale: 0.35,
                side_bias_activation_multiplier: 0.9,
                early_straighten_distance_multiplier: 3.0,
                early_straighten_pressure: 0.45,
            },
        );

        assert_eq!(mover.body.move_speed, 1.0);
        assert!(
            (displacement.length() - mover.body.move_speed * 0.5).abs() <= f32::EPSILON,
            "arrival should not slow combat movement near the goal: {displacement:?}"
        );
    }

    #[test]
    fn steering_slows_down_when_forward_lane_is_congested() {
        let target = WorldVec2::new(4.0, 0.0);
        let mover = movement_unit(
            6,
            WorldVec2::new(0.0, 0.0),
            MovementGoal::MoveToPoint {
                point: target,
                stop_radius: 0.1,
            },
        );
        let blocker = movement_unit(
            7,
            WorldVec2::new(0.35, 0.0),
            MovementGoal::MoveToPoint {
                point: target,
                stop_radius: 0.1,
            },
        );
        let units = vec![mover.clone(), blocker];

        let displacement = steered_displacement(
            &mover,
            &units,
            target,
            0.5,
            SteeringParams {
                separation_distance_multiplier: 1.15,
                separation_strength: 0.0,
                side_bias_strength: 0.0,
                congestion_lookahead_multiplier: 2.25,
                min_congestion_speed_scale: 0.35,
                side_bias_activation_multiplier: 0.9,
                early_straighten_distance_multiplier: 3.0,
                early_straighten_pressure: 0.45,
            },
        );

        assert!(
            displacement.length() < mover.body.move_speed * 0.5,
            "congestion should reduce this tick's displacement instead of pushing at full speed: {displacement:?}"
        );
    }
}
