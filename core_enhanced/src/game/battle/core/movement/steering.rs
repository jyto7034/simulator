use super::{
    engine::{MovementOutput, MovementStopReasonContinuous, MovementUnitInput},
    types::{MovementGoal, WorldVec2},
};

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
    target: WorldVec2,
    dt_seconds: f32,
) -> WorldVec2 {
    let max_distance = unit.body.move_speed * dt_seconds;
    if max_distance <= f32::EPSILON {
        return WorldVec2::ZERO;
    }

    (desired_velocity(unit, target) * dt_seconds).clamp_length(max_distance)
}

pub(super) fn goal_target_position(
    unit: &MovementUnitInput,
    _units: &[MovementUnitInput],
) -> Option<WorldVec2> {
    match unit.body.goal {
        Some(MovementGoal::MoveToPoint { point, .. }) => Some(point),
        Some(MovementGoal::AttackUnit { .. }) => None,
        None => None,
    }
}

pub(super) fn reached_goal(
    unit: &MovementUnitInput,
    _units: &[MovementUnitInput],
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
        Some(MovementGoal::AttackUnit { target_id }) => Some(MovementOutput::TargetReached {
            unit_id: unit.unit_id,
            target_id,
        }),
        None => Some(MovementOutput::MovementStopped {
            unit_id: unit.unit_id,
            position: unit.body.position,
            reason: MovementStopReasonContinuous::NoGoal,
        }),
    }
}
