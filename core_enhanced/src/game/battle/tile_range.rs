use serde::{Deserialize, Serialize};

use crate::game::resources::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacingDirection {
    Up,
    Right,
    Down,
    Left,
}

impl FacingDirection {
    pub fn rotate_offset(self, dx: i32, dy: i32) -> (i32, i32) {
        match self {
            FacingDirection::Up => (dx, dy),
            FacingDirection::Right => (-dy, dx),
            FacingDirection::Down => (-dx, -dy),
            FacingDirection::Left => (dy, -dx),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileRangePattern {
    #[serde(default)]
    pub include_anchor_tile: bool,
    pub rows: Vec<String>,
}

impl TileRangePattern {
    pub fn validate(&self) -> Result<(), String> {
        let mut anchor_count = 0usize;
        let mut affected_count = 0usize;
        let width = self
            .rows
            .first()
            .map(|row| row.chars().count())
            .ok_or_else(|| "tile range pattern must contain at least one row".to_string())?;

        if width == 0 {
            return Err("tile range pattern rows must not be empty".to_string());
        }

        for (y, row) in self.rows.iter().enumerate() {
            let row_width = row.chars().count();
            if row_width != width {
                return Err(format!(
                    "tile range pattern row {y} has width {row_width}, expected {width}"
                ));
            }
            for ch in row.chars() {
                match ch {
                    '@' => anchor_count += 1,
                    'X' => affected_count += 1,
                    '.' => {}
                    other => {
                        return Err(format!(
                            "tile range pattern contains unsupported character '{other}'"
                        ));
                    }
                }
            }
        }

        if anchor_count != 1 {
            return Err(format!(
                "tile range pattern must contain exactly one '@' anchor, found {anchor_count}"
            ));
        }
        if affected_count == 0 && !self.include_anchor_tile {
            return Err(
                "tile range pattern must contain at least one 'X' or include_anchor_tile=true"
                    .to_string(),
            );
        }

        Ok(())
    }

    pub fn affected_tiles(
        &self,
        anchor_world_tile: Position,
        facing: FacingDirection,
    ) -> Result<Vec<Position>, String> {
        self.validate()?;
        let anchor_pattern_pos = self.anchor_pattern_position()?;
        let mut affected = Vec::new();

        for (row_index, row) in self.rows.iter().enumerate() {
            for (col_index, ch) in row.chars().enumerate() {
                let is_affected = ch == 'X' || (ch == '@' && self.include_anchor_tile);
                if !is_affected {
                    continue;
                }

                let dx = col_index as i32 - anchor_pattern_pos.x;
                let dy = row_index as i32 - anchor_pattern_pos.y;
                let (rotated_dx, rotated_dy) = facing.rotate_offset(dx, dy);
                affected.push(Position::new(
                    anchor_world_tile.x + rotated_dx,
                    anchor_world_tile.y + rotated_dy,
                ));
            }
        }

        affected.sort_by_key(|pos| (pos.y, pos.x));
        affected.dedup();
        Ok(affected)
    }

    pub fn contains_target_tile(
        &self,
        anchor_world_tile: Position,
        facing: FacingDirection,
        target_tile: Position,
    ) -> Result<bool, String> {
        Ok(self
            .affected_tiles(anchor_world_tile, facing)?
            .into_iter()
            .any(|tile| tile == target_tile))
    }

    fn anchor_pattern_position(&self) -> Result<Position, String> {
        for (row_index, row) in self.rows.iter().enumerate() {
            for (col_index, ch) in row.chars().enumerate() {
                if ch == '@' {
                    return Ok(Position::new(col_index as i32, row_index as i32));
                }
            }
        }
        Err("tile range pattern missing '@' anchor".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forward_pattern() -> TileRangePattern {
        TileRangePattern {
            include_anchor_tile: false,
            rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
        }
    }

    #[test]
    fn tile_range_pattern_rotates_from_up_authored_rows() {
        let pattern = forward_pattern();
        let anchor = Position::new(10, 10);

        assert_eq!(
            pattern.affected_tiles(anchor, FacingDirection::Up).unwrap(),
            vec![Position::new(10, 9)]
        );
        assert_eq!(
            pattern
                .affected_tiles(anchor, FacingDirection::Right)
                .unwrap(),
            vec![Position::new(11, 10)]
        );
        assert_eq!(
            pattern
                .affected_tiles(anchor, FacingDirection::Down)
                .unwrap(),
            vec![Position::new(10, 11)]
        );
        assert_eq!(
            pattern
                .affected_tiles(anchor, FacingDirection::Left)
                .unwrap(),
            vec![Position::new(9, 10)]
        );
    }

    #[test]
    fn tile_range_pattern_includes_anchor_only_when_requested() {
        let mut pattern = TileRangePattern {
            include_anchor_tile: false,
            rows: vec!["@".to_string()],
        };
        assert!(pattern.validate().is_err());

        pattern.include_anchor_tile = true;
        assert_eq!(
            pattern
                .affected_tiles(Position::new(3, 4), FacingDirection::Up)
                .unwrap(),
            vec![Position::new(3, 4)]
        );
    }
}
