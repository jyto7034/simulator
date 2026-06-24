use crate::game::battle::battlefield::BattlefieldLayout;
use crate::{game::behavior::GameError, game::resources::Position};

impl BattlefieldLayout {
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            width,
            height,
            valid_tiles: None,
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
        self.static_obstacles.clear();
    }

    pub fn position_in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.width as i32 && pos.y < self.height as i32
    }

    pub fn is_valid_tile(&self, pos: Position) -> bool {
        self.position_in_bounds(pos)
            && self
                .valid_tiles
                .as_ref()
                .is_none_or(|valid_tiles| valid_tiles.contains(&pos))
    }

    pub fn valid_positions(&self) -> Vec<Position> {
        let mut positions = match &self.valid_tiles {
            Some(valid_tiles) => valid_tiles.iter().copied().collect::<Vec<_>>(),
            None => (0..self.height as i32)
                .flat_map(|y| (0..self.width as i32).map(move |x| Position::new(x, y)))
                .collect::<Vec<_>>(),
        };
        positions.sort_by_key(|pos| (pos.y, pos.x));
        positions
    }

    pub fn ensure_valid_tile(&self, pos: Position) -> Result<(), GameError> {
        if !self.is_valid_tile(pos) {
            return Err(GameError::OutOfBounds);
        }
        Ok(())
    }

    pub fn ensure_walkable_tile(&self, pos: Position) -> Result<(), GameError> {
        self.ensure_valid_tile(pos)?;
        if self.is_static_obstacle(pos) {
            return Err(GameError::StaticObstacleBlocked);
        }
        Ok(())
    }

    pub fn add_static_obstacle(&mut self, pos: Position) -> Result<(), GameError> {
        self.ensure_valid_tile(pos)?;
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
        self.is_valid_tile(pos) && !self.is_static_obstacle(pos)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_obstacles_block_placement() {
        let mut field = BattlefieldLayout::new(3, 3);
        let obstacle = Position::new(1, 1);
        field.add_static_obstacle(obstacle).unwrap();

        assert_eq!(field.static_obstacles(), vec![obstacle]);
        assert!(matches!(
            field.ensure_walkable_tile(obstacle).unwrap_err(),
            GameError::StaticObstacleBlocked
        ));

        assert!(field.remove_static_obstacle(obstacle));
        field.ensure_walkable_tile(obstacle).unwrap();
    }

    #[test]
    fn non_rectangular_valid_tiles_block_void_placement() {
        let field = BattlefieldLayout::new_with_valid_tiles(
            3,
            3,
            vec![
                Position::new(1, 0),
                Position::new(1, 1),
                Position::new(1, 2),
            ],
        );

        assert!(!field.is_valid_tile(Position::new(0, 0)));
        assert!(matches!(
            field.ensure_walkable_tile(Position::new(0, 0)).unwrap_err(),
            GameError::OutOfBounds
        ));
        field.ensure_walkable_tile(Position::new(1, 1)).unwrap();
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
