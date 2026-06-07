use std::collections::{HashMap, HashSet};

use crate::{game::battle::ids::UnitInstanceId, game::resources::Position};

pub mod field;

#[derive(Debug, Clone, Default)]
pub struct Tile {
    occupant: Option<UnitInstanceId>,
}

impl Tile {
    pub fn occupant(&self) -> Option<UnitInstanceId> {
        self.occupant
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

/// Static battlefield authoring state.
///
/// Continuous combat movement does not update this tile registry. During battle,
/// `RuntimeUnit.body` is the source of truth for world-space unit position.
pub struct Battlefield {
    width: u8,
    height: u8,
    tiles: Vec<Tile>,
    valid_tiles: Option<HashSet<Position>>,
    unit_pos: HashMap<UnitInstanceId, Position>,
    static_obstacles: HashSet<Position>,
}
