use std::collections::HashSet;

use crate::game::resources::Position;

pub mod field;

/// Static battlefield authoring state.
///
/// Continuous combat movement does not update this tile registry. During battle,
/// `RuntimeUnit.body` is the source of truth for world-space unit position.
pub struct BattlefieldLayout {
    width: u8,
    height: u8,
    valid_tiles: Option<HashSet<Position>>,
    static_obstacles: HashSet<Position>,
}
