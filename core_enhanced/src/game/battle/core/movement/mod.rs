use serde::{Deserialize, Serialize};

pub const TILE_UNITS_PER_TILE: u64 = 1_000_000;
pub const HALF_TILE_UNITS: u64 = TILE_UNITS_PER_TILE / 2;

pub mod engine;
mod lifecycle;
pub mod planner;
pub mod rapier_backend;
pub mod steering;
pub mod types;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementSegmentEndKind {
    Boundary,
    RangeEnter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionState {
    Idle,
    Dead,
}
