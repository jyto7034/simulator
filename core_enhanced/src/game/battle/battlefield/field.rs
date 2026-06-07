use std::collections::HashMap;

use crate::game::battle::battlefield::{Battlefield, Tile};
use crate::game::battle::ids::UnitInstanceId;
use crate::{game::behavior::GameError, game::resources::Position};

impl Battlefield {
    pub fn new(width: u8, height: u8) -> Self {
        let len = (width as usize) * (height as usize);
        Self {
            width,
            height,
            tiles: vec![Tile::default(); len],
            valid_tiles: None,
            unit_pos: HashMap::new(),
            static_obstacles: Default::default(),
        }
    }

    pub fn new_with_valid_tiles(width: u8, height: u8, valid_tiles: Vec<Position>) -> Self {
        let mut field = Self::new(width, height);
        if !valid_tiles.is_empty() {
            field.valid_tiles = Some(valid_tiles.into_iter().collect());
        }
        field
    }

    pub fn width(&self) -> u8 {
        self.width
    }

    pub fn height(&self) -> u8 {
        self.height
    }

    pub fn clear(&mut self) {
        for tile in &mut self.tiles {
            tile.occupant = None;
        }
        self.unit_pos.clear();
        self.static_obstacles.clear();
    }

    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && pos.x < self.width as i32
            && pos.y < self.height as i32
            && self
                .valid_tiles
                .as_ref()
                .is_none_or(|valid_tiles| valid_tiles.contains(&pos))
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

    pub fn is_static_obstacle(&self, pos: Position) -> bool {
        self.static_obstacles.contains(&pos)
    }

    pub fn is_walkable_tile(&self, pos: Position) -> bool {
        self.in_bounds(pos) && !self.is_static_obstacle(pos)
    }

    pub fn void_tiles(&self) -> Vec<Position> {
        let Some(valid_tiles) = &self.valid_tiles else {
            return Vec::new();
        };

        let mut void_tiles = Vec::new();
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let position = Position::new(x, y);
                if !valid_tiles.contains(&position) {
                    void_tiles.push(position);
                }
            }
        }
        void_tiles
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
        let pos = self.unit_pos.remove(&unit)?;
        let idx = self.idx(pos).ok()?;
        let _ = self.tiles[idx].clear_occupant(unit);
        Some(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn static_obstacles_block_placement() {
        let mut field = Battlefield::new(3, 3);
        let unit = UnitInstanceId::from(Uuid::from_u128(1));
        let obstacle = Position::new(1, 1);
        field.add_static_obstacle(obstacle).unwrap();

        assert_eq!(field.static_obstacles(), vec![obstacle]);
        assert!(matches!(
            field.place(unit, obstacle).unwrap_err(),
            GameError::PositionOccupied
        ));

        assert!(field.remove_static_obstacle(obstacle));
        field.place(unit, obstacle).unwrap();
    }

    #[test]
    fn non_rectangular_valid_tiles_block_void_placement() {
        let unit = UnitInstanceId::from(Uuid::from_u128(1));
        let mut field = Battlefield::new_with_valid_tiles(
            3,
            3,
            vec![
                Position::new(1, 0),
                Position::new(1, 1),
                Position::new(1, 2),
            ],
        );

        assert!(!field.in_bounds(Position::new(0, 0)));
        assert!(matches!(
            field.place(unit, Position::new(0, 0)).unwrap_err(),
            GameError::OutOfBounds
        ));
        field.place(unit, Position::new(1, 1)).unwrap();
        assert_eq!(
            field.void_tiles(),
            vec![
                Position::new(0, 0),
                Position::new(2, 0),
                Position::new(0, 1),
                Position::new(2, 1),
                Position::new(0, 2),
                Position::new(2, 2),
            ]
        );
    }
}
