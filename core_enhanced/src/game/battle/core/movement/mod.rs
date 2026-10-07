use serde::{Deserialize, Serialize};

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
pub const TILE_UNITS_PER_TILE: u64 = 1_000_000;
#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
pub const HALF_TILE_UNITS: u64 = TILE_UNITS_PER_TILE / 2;

mod blocking;
mod engine;
mod lifecycle;
mod path;
pub mod planner;
mod rapier_backend;
mod steering;
pub mod types;

pub(super) use rapier_backend::RapierMovementWorld;

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

crate::authoring::clone_shape::enum_shape!(ActionState {
    units: [Idle, Dead],
    tuples: [],
    records: []
});
