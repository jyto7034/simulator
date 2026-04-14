use crate::{
    ecs::resources::Position,
    game::battle::{enums::BattleEvent, ids::UnitInstanceId},
};
use serde::{Deserialize, Serialize};

use super::BattleCore;
pub const TILE_UNITS_PER_TILE: u64 = 1_000_000;
pub const HALF_TILE_UNITS: u64 = TILE_UNITS_PER_TILE / 2;
pub const REPATH_BASE_DELAY_MS: u64 = 100;
pub const YIELD_RETRY_DELAY_MS: u64 = 10;
pub const BLOCKED_RETRY_DELAY_MS: u64 = 30;
pub const HOLD_RETRY_DELAY_MS: u64 = 20;

mod execute;
mod orchestrator;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContinuousPosition {
    pub x_units: i64,
    pub y_units: i64,
}

impl ContinuousPosition {
    pub const fn new(x_units: i64, y_units: i64) -> Self {
        Self { x_units, y_units }
    }

    pub fn tile_center(tile: Position) -> Self {
        let (x_units, y_units) = tile_center_units(tile);
        Self { x_units, y_units }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementSegmentEndKind {
    Boundary,
    RangeEnter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovementSegment {
    pub from_tile: Position,
    pub to_tile: Position,
    pub start: ContinuousPosition,
    pub target: ContinuousPosition,
    pub started_at_ms: u64,
    pub ends_at_ms: u64,
    pub end_kind: MovementSegmentEndKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlannedContinuation {
    pub step: Position,
    pub owner_priority: u8,
    pub claim_priority: Option<u8>,
    pub retry_budget: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementState {
    pub path: Vec<Position>,
    pub reserved_destination: Option<Position>,
    pub planned_continuation: Option<PlannedContinuation>,
    pub path_cursor: u32,
    pub repath_counter: u32,
    pub orchestrator_priority: u8,
    pub last_update_ms: u64,

    pub step_from: Position,
    pub step_to: Position,
    pub step_start_x_units: i64,
    pub step_start_y_units: i64,
    pub target_x_units: i64,
    pub target_y_units: i64,
    pub step_started_at_ms: u64,
    pub step_ends_at_ms: u64,
    pub step_end_kind: MovementSegmentEndKind,
}

impl MovementState {
    pub fn new_at(pos: Position, now_ms: u64) -> Self {
        Self {
            path: vec![pos],
            reserved_destination: None,
            planned_continuation: None,
            path_cursor: 0,
            repath_counter: 0,
            orchestrator_priority: u8::MAX,
            last_update_ms: now_ms,
            step_from: pos,
            step_to: pos,
            step_start_x_units: tile_center_units(pos).0,
            step_start_y_units: tile_center_units(pos).1,
            target_x_units: tile_center_units(pos).0,
            target_y_units: tile_center_units(pos).1,
            step_started_at_ms: now_ms,
            step_ends_at_ms: now_ms,
            step_end_kind: MovementSegmentEndKind::Boundary,
        }
    }

    pub fn current_segment(&self) -> MovementSegment {
        MovementSegment {
            from_tile: self.step_from,
            to_tile: self.step_to,
            start: ContinuousPosition::new(self.step_start_x_units, self.step_start_y_units),
            target: ContinuousPosition::new(self.target_x_units, self.target_y_units),
            started_at_ms: self.step_started_at_ms,
            ends_at_ms: self.step_ends_at_ms,
            end_kind: self.step_end_kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionState {
    Idle,
    Moving(MovementState),
    Holding {
        until_ms: u64,
        repath_counter: u32,
    },
    Yielding {
        until_ms: u64,
        repath_counter: u32,
    },
    Blocked {
        until_ms: u64,
        repath_counter: u32,
    },
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
    pub enemy_pos: Position,
    pub chase_dist: u32,
    pub best_dest: Position,
    pub dest_candidates: Vec<Position>,
}

impl BattleCore {
    pub(in crate::game::battle::core) fn next_wait_repath_until_ms(
        &self,
        now_ms: u64,
        unit_instance_id: UnitInstanceId,
        repath_counter: u32,
    ) -> u64 {
        now_ms.saturating_add(REPATH_BASE_DELAY_MS).saturating_add(
            crate::game::determinism::repath_jitter_ms(self.seed, unit_instance_id, repath_counter),
        )
    }

    pub(in crate::game::battle::core) fn enter_wait_repath(
        &mut self,
        unit_instance_id: UnitInstanceId,
        until_ms: u64,
        repath_counter: u32,
    ) {
        self.battlefield.cancel_reservation(unit_instance_id);
        if let Some(unit) = self.units.get_mut(&unit_instance_id) {
            unit.current_target = None;
            unit.move_epoch = unit.move_epoch.wrapping_add(1);
            unit.action_state = ActionState::WaitRepath {
                until_ms,
                repath_counter,
            };
        }
        self.schedule_movement_intent(until_ms);
    }

    pub(in crate::game::battle::core) fn enter_yield(
        &mut self,
        now_ms: u64,
        unit_instance_id: UnitInstanceId,
        retry_delay_ms: u64,
        repath_counter: u32,
    ) {
        if matches!(
            self.units.get(&unit_instance_id).map(|u| &u.action_state),
            Some(ActionState::Moving(_))
        ) {
            self.update_move_position_to(unit_instance_id, now_ms);
        }

        self.battlefield.cancel_reservation(unit_instance_id);
        if let Some(unit) = self.units.get_mut(&unit_instance_id) {
            unit.move_epoch = unit.move_epoch.wrapping_add(1);
            unit.action_state = ActionState::Yielding {
                until_ms: now_ms.saturating_add(retry_delay_ms),
                repath_counter,
            };
        }
        self.schedule_movement_intent(now_ms.saturating_add(retry_delay_ms));
    }

    pub(in crate::game::battle::core) fn enter_blocked(
        &mut self,
        now_ms: u64,
        unit_instance_id: UnitInstanceId,
        retry_delay_ms: u64,
        repath_counter: u32,
    ) {
        self.battlefield.cancel_reservation(unit_instance_id);
        if let Some(unit) = self.units.get_mut(&unit_instance_id) {
            unit.move_epoch = unit.move_epoch.wrapping_add(1);
            unit.action_state = ActionState::Blocked {
                until_ms: now_ms.saturating_add(retry_delay_ms),
                repath_counter,
            };
        }
        self.schedule_movement_intent(now_ms.saturating_add(retry_delay_ms));
    }

    pub(in crate::game::battle::core) fn enter_hold(
        &mut self,
        now_ms: u64,
        unit_instance_id: UnitInstanceId,
        retry_delay_ms: u64,
        repath_counter: u32,
    ) {
        self.battlefield.cancel_reservation(unit_instance_id);
        if let Some(unit) = self.units.get_mut(&unit_instance_id) {
            unit.move_epoch = unit.move_epoch.wrapping_add(1);
            unit.action_state = ActionState::Holding {
                until_ms: now_ms.saturating_add(retry_delay_ms),
                repath_counter,
            };
        }
        self.schedule_movement_intent(now_ms.saturating_add(retry_delay_ms));
    }

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
        assert_eq!(state.reserved_destination, None);
        assert_eq!(state.planned_continuation, None);
        assert_eq!(state.path_cursor, 0);
        assert_eq!(state.last_update_ms, 123);
        assert_eq!(state.step_from, pos);
        assert_eq!(state.step_to, pos);
        assert_eq!(state.step_started_at_ms, 123);
        assert_eq!(state.step_ends_at_ms, 123);
        assert_eq!(state.step_end_kind, MovementSegmentEndKind::Boundary);

        let (cx, cy) = tile_center_units(pos);
        assert_eq!(state.step_start_x_units, cx);
        assert_eq!(state.step_start_y_units, cy);
        assert_eq!(state.target_x_units, cx);
        assert_eq!(state.target_y_units, cy);
    }

    #[test]
    fn movement_state_exposes_current_segment() {
        let pos = Position::new(1, 1);
        let mut state = MovementState::new_at(pos, 123);
        state.step_to = Position::new(1, 0);
        state.step_start_x_units = 1_000_000;
        state.step_start_y_units = 1_000_000;
        state.target_x_units = 1_000_000;
        state.target_y_units = 500_000;
        state.step_ends_at_ms = 173;
        state.step_end_kind = MovementSegmentEndKind::RangeEnter;

        let segment = state.current_segment();
        assert_eq!(segment.from_tile, pos);
        assert_eq!(segment.to_tile, Position::new(1, 0));
        assert_eq!(segment.start, ContinuousPosition::new(1_000_000, 1_000_000));
        assert_eq!(segment.target, ContinuousPosition::new(1_000_000, 500_000));
        assert_eq!(segment.started_at_ms, 123);
        assert_eq!(segment.ends_at_ms, 173);
        assert_eq!(segment.end_kind, MovementSegmentEndKind::RangeEnter);
    }
}
