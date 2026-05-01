use std::collections::{HashMap, HashSet};

use crate::{ecs::resources::Position, game::battle::ids::UnitInstanceId};

pub mod bfs;
pub mod field;

/// If the reserver is within this distance (Chebyshev, tiles) to its reserved tile,
/// treat the reservation as hard (blocks other units). Otherwise it's considered soft
/// and other units may enter/step through, potentially canceling the reservation.
pub const RESERVATION_HARD_DISTANCE_TILES: i32 = 3;

#[derive(Debug, Clone, Default)]
pub struct Tile {
    occupant: Option<UnitInstanceId>,
    reservation: Option<Reservation>,
}

impl Tile {
    pub fn occupant(&self) -> Option<UnitInstanceId> {
        self.occupant
    }

    pub fn is_empty(&self) -> bool {
        self.occupant.is_none()
    }

    pub fn reservation(&self) -> Option<Reservation> {
        self.reservation
    }

    fn set_occupant(&mut self, unit: UnitInstanceId) {
        self.occupant = Some(unit);
    }

    fn clear_occupant(&mut self, unit: UnitInstanceId) -> bool {
        if self.occupant == Some(unit) {
            self.occupant = None;
            return true;
        }
        false
    }
}

pub struct Battlefield {
    width: u8,
    height: u8,
    tiles: Vec<Tile>,
    unit_pos: HashMap<UnitInstanceId, Position>,
    reserved_by_unit: HashMap<UnitInstanceId, Position>,
    static_obstacles: HashSet<Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservationKind {
    Soft,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reservation {
    pub unit: UnitInstanceId,
    pub hard_from_ms: u64,
}
