use std::collections::HashSet;

use crate::game::resources::Position;
use crate::owned_collections::MapCloneHistory;

pub mod field;

/// Static battlefield authoring state.
///
/// Continuous combat movement does not update this tile registry. During battle,
/// `RuntimeUnit.body` is the source of truth for world-space unit position.
#[derive(Clone)]
pub struct BattlefieldLayout {
    width: u8,
    height: u8,
    valid_tiles: Option<HashSet<Position>>,
    static_obstacles: HashSet<Position>,
    static_obstacles_clone_history: MapCloneHistory,
}

impl crate::authoring::clone_shape::CloneShape for BattlefieldLayout {
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), crate::game::behavior::GameError> {
        let Self {
            width: _,
            height: _,
            valid_tiles,
            static_obstacles,
            static_obstacles_clone_history: _,
        } = self;
        // Both sets have Copy Position children. Their lengths and proven native
        // bucket bounds are O(1); the public sorted projection is never allocated.
        let slots = self
            .authoring_clone_table_slots()
            .and_then(|n| n.checked_add(8))
            .and_then(|n| n.checked_add(valid_tiles.as_ref().map_or(0, |s| s.len())))
            .and_then(|n| n.checked_add(static_obstacles.len()));
        meter.scan_storage(slots)
    }
}

impl BattlefieldLayout {
    pub(crate) fn authoring_clone_table_slots(&self) -> Option<usize> {
        let valid_slots = match &self.valid_tiles {
            // Constructed from an insertion-only collect and immutable after
            // construction; public queries cannot mutate this native table.
            Some(tiles) => {
                MapCloneHistory::from_tombstone_free_capacity(tiles.capacity()).bucket_bound()?
            }
            None => 0,
        };
        valid_slots.checked_add(self.static_obstacles_clone_history.bucket_bound()?)
    }
}
