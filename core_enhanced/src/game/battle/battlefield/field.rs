use std::collections::HashMap;

use crate::game::battle::battlefield::{
    Battlefield, Reservation, Tile, RESERVATION_HARD_DISTANCE_TILES,
};
use crate::game::battle::ids::UnitInstanceId;
use crate::{ecs::resources::Position, game::behavior::GameError};

impl Battlefield {
    pub fn is_empty_tile(&self, idx: usize) -> bool {
        self.tiles
            .get(idx)
            .is_some_and(|t| t.occupant.is_none() && t.reservation.is_none())
            && !self.static_obstacles.contains(&Position::new(
                idx as i32 % self.width as i32,
                idx as i32 / self.width as i32,
            ))
    }

    pub fn get_surrounding_tiles(&self, unit: UnitInstanceId) -> Option<Vec<Position>> {
        let current = self.position_of(unit)?;
        let mut out: Vec<Position> = Vec::new();

        for next in Self::neighbors_8(current) {
            if !self.in_bounds(next) {
                continue;
            }
            let Ok(next_idx) = self.idx(next) else {
                continue;
            };

            if !self.is_empty_tile(next_idx) {
                continue;
            }

            out.push(next);
        }

        Some(out)
    }
}

impl Battlefield {
    pub fn new(width: u8, height: u8) -> Self {
        let len = (width as usize) * (height as usize);
        Self {
            width,
            height,
            tiles: vec![Tile::default(); len],
            unit_pos: HashMap::new(),
            reserved_by_unit: HashMap::new(),
            static_obstacles: Default::default(),
        }
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn neighbors_8(pos: Position) -> [Position; 8] {
        // Cardinal first, then diagonals (fixed order for deterministic tie-breaking).
        // This makes shortest-path ties prefer "straighter" movement (less diagonal drift).
        [
            Position::new(pos.x, pos.y - 1),
            Position::new(pos.x, pos.y + 1),
            Position::new(pos.x - 1, pos.y),
            Position::new(pos.x + 1, pos.y),
            Position::new(pos.x - 1, pos.y - 1),
            Position::new(pos.x + 1, pos.y - 1),
            Position::new(pos.x - 1, pos.y + 1),
            Position::new(pos.x + 1, pos.y + 1),
        ]
    }

    pub fn clear(&mut self) {
        for tile in &mut self.tiles {
            tile.occupant = None;
            tile.reservation = None;
        }
        self.unit_pos.clear();
        self.reserved_by_unit.clear();
        self.static_obstacles.clear();
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.width as i32 && pos.y < self.height as i32
    }

    pub fn idx(&self, pos: Position) -> Result<usize, GameError> {
        if !self.in_bounds(pos) {
            return Err(GameError::OutOfBounds);
        }
        Ok((pos.y as usize) * (self.width as usize) + (pos.x as usize))
    }

    pub fn position_of(&self, unit: UnitInstanceId) -> Option<Position> {
        self.unit_pos.get(&unit).copied()
    }

    pub fn occupant(&self, pos: Position) -> Result<Option<UnitInstanceId>, GameError> {
        let idx = self.idx(pos)?;
        Ok(self.tiles[idx].occupant)
    }

    pub fn add_static_obstacle(&mut self, pos: Position) -> Result<(), GameError> {
        let idx = self.idx(pos)?;
        if self.tiles[idx].occupant.is_some() {
            return Err(GameError::PositionOccupied);
        }
        self.static_obstacles.insert(pos);
        Ok(())
    }

    pub fn remove_static_obstacle(&mut self, pos: Position) -> bool {
        self.static_obstacles.remove(&pos)
    }

    pub fn static_obstacles(&self) -> Vec<Position> {
        let mut obstacles: Vec<Position> = self.static_obstacles.iter().copied().collect();
        obstacles.sort_by_key(|pos| (pos.y, pos.x));
        obstacles
    }

    pub fn place(&mut self, unit: UnitInstanceId, pos: Position) -> Result<(), GameError> {
        if self.unit_pos.contains_key(&unit) {
            return Err(GameError::UnitAlreadyPlaced);
        }
        let idx = self.idx(pos)?;
        if self.tiles[idx].occupant.is_some() || self.static_obstacles.contains(&pos) {
            return Err(GameError::PositionOccupied);
        }
        self.tiles[idx].set_occupant(unit);
        self.unit_pos.insert(unit, pos);
        Ok(())
    }

    pub fn remove(&mut self, unit: UnitInstanceId) -> Option<Position> {
        self.cancel_reservation(unit);
        let pos = self.unit_pos.remove(&unit)?;
        let idx = self.idx(pos).ok()?;
        let _ = self.tiles[idx].clear_occupant(unit);
        Some(pos)
    }

    pub fn reserved_destination_of(&self, unit: UnitInstanceId) -> Option<Position> {
        self.reserved_by_unit.get(&unit).copied()
    }

    pub fn reservation_at(&self, pos: Position) -> Option<Reservation> {
        let idx = self.idx(pos).ok()?;
        self.tiles.get(idx)?.reservation
    }

    pub fn reservation_blocks_for(&self, mover: UnitInstanceId, pos: Position) -> bool {
        let Some(reservation) = self.reservation_at(pos) else {
            return false;
        };
        if reservation.unit == mover {
            return false;
        }

        let Some(reserver_pos) = self.position_of(reservation.unit) else {
            return true;
        };
        reserver_pos.chebyshev(&pos) <= RESERVATION_HARD_DISTANCE_TILES
    }

    pub fn reserved_unit_at(&self, pos: Position) -> Option<UnitInstanceId> {
        let idx = self.idx(pos).ok()?;
        self.tiles.get(idx)?.reservation.map(|r| r.unit)
    }

    pub fn reserve(
        &mut self,
        unit: UnitInstanceId,
        dest: Position,
        hard_from_ms: u64,
    ) -> Result<(), GameError> {
        self.cancel_reservation(unit);

        let idx = self.idx(dest)?;
        let tile = self.tiles.get_mut(idx).ok_or(GameError::OutOfBounds)?;

        if tile.occupant.is_some() || self.static_obstacles.contains(&dest) {
            return Err(GameError::PositionOccupied);
        }

        if let Some(existing) = tile.reservation {
            if existing.unit != unit {
                return Err(GameError::InvalidAction);
            }
        }

        tile.reservation = Some(Reservation { unit, hard_from_ms });
        self.reserved_by_unit.insert(unit, dest);
        Ok(())
    }

    pub fn cancel_reservation(&mut self, unit_id: UnitInstanceId) {
        let Some(pos) = self.reserved_by_unit.remove(&unit_id) else {
            return;
        };

        let mut cleared = false;
        if let Ok(idx) = self.idx(pos) {
            if let Some(tile) = self.tiles.get_mut(idx) {
                if tile.reservation.map(|r| r.unit) == Some(unit_id) {
                    tile.reservation = None;
                    cleared = true;
                }
            }
        }

        if cleared {
            return;
        }

        for tile in &mut self.tiles {
            if tile.reservation.map(|r| r.unit) == Some(unit_id) {
                tile.reservation = None;
            }
        }
    }

    pub fn move_unit(&mut self, unit: UnitInstanceId, to: Position) -> Result<(), GameError> {
        let from = self
            .unit_pos
            .get(&unit)
            .copied()
            .ok_or(GameError::UnitNotFound)?;
        if from == to {
            return Ok(());
        }

        let from_idx = self.idx(from)?;
        let to_idx = self.idx(to)?;

        if self.tiles[to_idx].occupant.is_some() || self.static_obstacles.contains(&to) {
            return Err(GameError::PositionOccupied);
        }

        if !self.tiles[from_idx].clear_occupant(unit) {
            return Err(GameError::UnitNotFound);
        }

        self.tiles[to_idx].set_occupant(unit);
        self.unit_pos.insert(unit, to);
        Ok(())
    }

    pub fn swap_units(&mut self, a: UnitInstanceId, b: UnitInstanceId) -> Result<(), GameError> {
        if a == b {
            return Ok(());
        }
        let a_pos = self
            .unit_pos
            .get(&a)
            .copied()
            .ok_or(GameError::UnitNotFound)?;
        let b_pos = self
            .unit_pos
            .get(&b)
            .copied()
            .ok_or(GameError::UnitNotFound)?;
        if a_pos == b_pos {
            return Ok(());
        }

        let a_idx = self.idx(a_pos)?;
        let b_idx = self.idx(b_pos)?;

        if self.tiles[a_idx].occupant != Some(a) || self.tiles[b_idx].occupant != Some(b) {
            return Err(GameError::UnitNotFound);
        }

        self.tiles[a_idx].occupant = Some(b);
        self.tiles[b_idx].occupant = Some(a);
        self.unit_pos.insert(a, b_pos);
        self.unit_pos.insert(b, a_pos);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn bfs_8_respects_blocked_tiles_and_reconstructs_path() {
        let mut field = Battlefield::new(3, 3);
        let blocker = UnitInstanceId::from(Uuid::from_u128(1));
        field.place(blocker, Position::new(1, 0)).unwrap(); // block north of start

        let bfs = field
            .bfs_map_8(Position::new(1, 1), |_pos, tile| tile.occupant().is_none())
            .unwrap();

        assert_eq!(bfs.distance_to(Position::new(1, 1)), Some(0));
        assert_eq!(bfs.distance_to(Position::new(1, 0)), None);
        assert_eq!(bfs.distance_to(Position::new(0, 0)), Some(1));

        let path = bfs.reconstruct_path_to(Position::new(0, 0)).unwrap();
        assert_eq!(path.first().copied(), Some(Position::new(1, 1)));
        assert_eq!(path.last().copied(), Some(Position::new(0, 0)));
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn surrounding_tiles_returns_only_empty_in_bounds_neighbors() {
        let mut field = Battlefield::new(3, 3);
        let target = UnitInstanceId::from(Uuid::from_u128(10));
        field.place(target, Position::new(1, 1)).unwrap();

        let blocker = UnitInstanceId::from(Uuid::from_u128(11));
        field.place(blocker, Position::new(0, 0)).unwrap(); // NW is occupied
        field.add_static_obstacle(Position::new(2, 2)).unwrap(); // SE is terrain

        let tiles = field.get_surrounding_tiles(target).unwrap();
        assert_eq!(
            tiles,
            vec![
                Position::new(1, 0),
                Position::new(1, 2),
                Position::new(0, 1),
                Position::new(2, 1),
                Position::new(2, 0),
                Position::new(0, 2),
            ]
        );
    }

    #[test]
    fn static_obstacles_block_placement_movement_and_reservation() {
        let mut field = Battlefield::new(3, 3);
        let unit = UnitInstanceId::from(Uuid::from_u128(1));
        let obstacle = Position::new(1, 1);
        field.add_static_obstacle(obstacle).unwrap();

        assert_eq!(field.static_obstacles(), vec![obstacle]);
        assert!(matches!(
            field.place(unit, obstacle).unwrap_err(),
            GameError::PositionOccupied
        ));

        field.place(unit, Position::new(0, 0)).unwrap();
        assert!(matches!(
            field.move_unit(unit, obstacle).unwrap_err(),
            GameError::PositionOccupied
        ));
        assert!(matches!(
            field.reserve(unit, obstacle, 0).unwrap_err(),
            GameError::PositionOccupied
        ));

        assert!(field.remove_static_obstacle(obstacle));
        field.move_unit(unit, obstacle).unwrap();
    }

    #[test]
    fn reservation_blocks_only_when_reserver_is_close_enough() {
        let mut field = Battlefield::new(10, 10);
        let mover = UnitInstanceId::from(Uuid::from_u128(1));
        let reserver = UnitInstanceId::from(Uuid::from_u128(2));
        field.place(mover, Position::new(0, 0)).unwrap();
        field.place(reserver, Position::new(9, 9)).unwrap();

        // Reserver is far away; reservation should be considered soft for others.
        let dest = Position::new(5, 5);
        field.reserve(reserver, dest, 0).unwrap();
        assert!(!field.reservation_blocks_for(mover, dest));

        // Move reserver close enough to make reservation hard-blocking.
        field.move_unit(reserver, Position::new(6, 6)).unwrap();
        assert!(field.reservation_blocks_for(mover, dest));

        // Reservation never blocks its own mover.
        assert!(!field.reservation_blocks_for(reserver, dest));
    }

    #[test]
    fn reserve_and_cancel_reservation_clear_tile_and_index() {
        let mut field = Battlefield::new(3, 3);
        let unit = UnitInstanceId::from(Uuid::from_u128(1));
        field.place(unit, Position::new(0, 0)).unwrap();

        let dest = Position::new(1, 1);
        field.reserve(unit, dest, 123).unwrap();
        assert_eq!(field.reserved_destination_of(unit), Some(dest));
        assert!(field.reservation_at(dest).is_some());

        field.cancel_reservation(unit);
        assert_eq!(field.reserved_destination_of(unit), None);
        assert!(field.reservation_at(dest).is_none());
    }

    #[test]
    fn reserve_rejects_occupied_destination() {
        let mut field = Battlefield::new(3, 3);
        let a = UnitInstanceId::from(Uuid::from_u128(1));
        let b = UnitInstanceId::from(Uuid::from_u128(2));
        field.place(a, Position::new(0, 0)).unwrap();
        field.place(b, Position::new(1, 1)).unwrap();

        let err = field.reserve(a, Position::new(1, 1), 0).unwrap_err();
        assert!(matches!(err, GameError::PositionOccupied));
    }

    #[test]
    fn cancel_reservation_falls_back_to_scanning_tiles_when_index_is_stale() {
        let mut field = Battlefield::new(3, 3);
        let unit = UnitInstanceId::from(Uuid::from_u128(1));
        field.place(unit, Position::new(0, 0)).unwrap();

        let dest = Position::new(1, 1);
        field.reserve(unit, dest, 0).unwrap();
        assert!(field.reservation_at(dest).is_some());
        assert_eq!(field.reserved_destination_of(unit), Some(dest));

        // Corrupt the reservation tile (simulate desync) while keeping the index.
        let idx = field.idx(dest).unwrap();
        field.tiles[idx].reservation = Some(Reservation {
            unit: UnitInstanceId::from(Uuid::from_u128(999)),
            hard_from_ms: 0,
        });

        // Create a "stray" reservation owned by `unit` at a different tile.
        let stray = Position::new(2, 2);
        let stray_idx = field.idx(stray).unwrap();
        field.tiles[stray_idx].reservation = Some(Reservation {
            unit,
            hard_from_ms: 0,
        });

        // cancel_reservation should still clear any reservation entries owned by `unit`.
        field.cancel_reservation(unit);
        assert_eq!(field.reserved_destination_of(unit), None);
        // The corrupted reservation remains (owned by 999), but we must not keep any for `unit`.
        assert_eq!(
            field
                .tiles
                .iter()
                .filter(|t| t.reservation.map(|r| r.unit) == Some(unit))
                .count(),
            0
        );
    }
}
