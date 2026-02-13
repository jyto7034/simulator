use crate::{
    ecs::resources::Position,
    game::battle::{enums::BattleEvent, ids::UnitInstanceId},
};

use super::BattleCore;
pub const TILE_UNITS_PER_TILE: u64 = 1_000_000;
pub const HALF_TILE_UNITS: u64 = TILE_UNITS_PER_TILE / 2;

mod execute;
mod plan;

pub(super) fn tile_center_units(tile: Position) -> (i64, i64) {
    (
        (tile.x as i64).saturating_mul(TILE_UNITS_PER_TILE as i64),
        (tile.y as i64).saturating_mul(TILE_UNITS_PER_TILE as i64),
    )
}

pub(super) fn boundary_target_units(from: Position, to: Position) -> (i64, i64) {
    let (cx, cy) = tile_center_units(from);
    let dx = (to.x - from.x).signum() as i64;
    let dy = (to.y - from.y).signum() as i64;
    (
        cx.saturating_add(dx.saturating_mul(HALF_TILE_UNITS as i64)),
        cy.saturating_add(dy.saturating_mul(HALF_TILE_UNITS as i64)),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementState {
    pub path: Vec<Position>,
    pub reserved_destination: Option<Position>,
    pub path_cursor: u32,
    pub repath_counter: u32,
    pub last_update_ms: u64,

    pub step_from: Position,
    pub step_to: Position,
    pub target_x_units: i64,
    pub target_y_units: i64,
    pub step_started_at_ms: u64,
    pub step_ends_at_ms: u64,
}

impl MovementState {
    pub fn new_at(pos: Position, now_ms: u64) -> Self {
        Self {
            path: vec![pos],
            reserved_destination: None,
            path_cursor: 0,
            repath_counter: 0,
            last_update_ms: now_ms,
            step_from: pos,
            step_to: pos,
            target_x_units: tile_center_units(pos).0,
            target_y_units: tile_center_units(pos).1,
            step_started_at_ms: now_ms,
            step_ends_at_ms: now_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionState {
    Idle,
    Moving(MovementState),
    WaitRepath {
        until_ms: u64,
        repath_counter: u32,
    },
    HardCCLocked {
        until_ms: u64,
        /// Preserved movement state when CC interrupts movement mid-step.
        movement: Option<MovementState>,
    },
    Dead,
}

#[derive(Debug, Clone)]
pub struct EnemyChasePlan {
    pub enemy_id: UnitInstanceId,
    pub chase_dist: u32,
    pub dest_candidates: Vec<Position>,
}

impl BattleCore {
    pub(in crate::game::battle) fn schedule_move_step_at(
        &mut self,
        unit_instance_id: UnitInstanceId,
        time_ms: u64,
    ) {
        let expected_move_epoch = self
            .units
            .get(&unit_instance_id)
            .map(|u| u.move_epoch)
            .unwrap_or(0);
        self.event_queue.push(BattleEvent::MoveStep {
            time_ms,
            unit_instance_id,
            expected_move_epoch,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_center_units_maps_tiles_to_unit_coordinates() {
        let p = Position::new(2, 3);
        let (x, y) = tile_center_units(p);
        assert_eq!(x, 2_i64 * TILE_UNITS_PER_TILE as i64);
        assert_eq!(y, 3_i64 * TILE_UNITS_PER_TILE as i64);
    }

    #[test]
    fn boundary_target_units_points_to_tile_boundary_in_move_direction() {
        let from = Position::new(5, 5);

        let (tx, ty) = boundary_target_units(from, Position::new(6, 5));
        assert_eq!(
            tx,
            5_i64 * TILE_UNITS_PER_TILE as i64 + HALF_TILE_UNITS as i64
        );
        assert_eq!(ty, 5_i64 * TILE_UNITS_PER_TILE as i64);

        let (tx, ty) = boundary_target_units(from, Position::new(5, 4));
        assert_eq!(tx, 5_i64 * TILE_UNITS_PER_TILE as i64);
        assert_eq!(
            ty,
            5_i64 * TILE_UNITS_PER_TILE as i64 - HALF_TILE_UNITS as i64
        );

        let (tx, ty) = boundary_target_units(from, Position::new(4, 4));
        assert_eq!(
            tx,
            5_i64 * TILE_UNITS_PER_TILE as i64 - HALF_TILE_UNITS as i64
        );
        assert_eq!(
            ty,
            5_i64 * TILE_UNITS_PER_TILE as i64 - HALF_TILE_UNITS as i64
        );
    }

    #[test]
    fn movement_state_new_at_sets_consistent_defaults() {
        let pos = Position::new(1, 1);
        let state = MovementState::new_at(pos, 123);
        assert_eq!(state.path, vec![pos]);
        assert_eq!(state.path_cursor, 0);
        assert_eq!(state.last_update_ms, 123);
        assert_eq!(state.step_from, pos);
        assert_eq!(state.step_to, pos);
        assert_eq!(state.step_started_at_ms, 123);
        assert_eq!(state.step_ends_at_ms, 123);

        let (cx, cy) = tile_center_units(pos);
        assert_eq!(state.target_x_units, cx);
        assert_eq!(state.target_y_units, cy);
    }
}
