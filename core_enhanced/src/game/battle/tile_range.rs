use serde::{Deserialize, Serialize};

use crate::game::{battle::battlefield::BattlefieldLayout, resources::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TileRangePolicy {
    #[default]
    Pattern,
    WholeFieldValidTiles,
}

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

    pub fn from_delta(dx: f32, dy: f32) -> Option<Self> {
        if dx.abs() <= f32::EPSILON && dy.abs() <= f32::EPSILON {
            return None;
        }
        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                Some(FacingDirection::Right)
            } else {
                Some(FacingDirection::Left)
            }
        } else if dy.abs() > dx.abs() {
            if dy > 0.0 {
                Some(FacingDirection::Down)
            } else {
                Some(FacingDirection::Up)
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

    /// Whether the authored footprint produces the same tiles for every facing.
    /// Such a footprint can be projected for stationary/facingless casters.
    pub fn is_rotation_invariant(&self) -> bool {
        let anchor = Position::new(0, 0);
        let Ok(up) = self.affected_tiles(anchor, FacingDirection::Up) else {
            return false;
        };
        [
            FacingDirection::Right,
            FacingDirection::Down,
            FacingDirection::Left,
        ]
        .into_iter()
        .all(|facing| {
            self.affected_tiles(anchor, facing)
                .is_ok_and(|tiles| tiles == up)
        })
    }

    pub fn contains_target_tile(
        &self,
        anchor_world_tile: Position,
        facing: FacingDirection,
        target_tile: Position,
    ) -> Result<bool, String> {
        self.validate()?;
        let anchor_pattern_pos = self.anchor_pattern_position()?;
        let mut contains = false;
        // Membership does not need the sorted/deduplicated materialization.
        // Keep the same validation, row order and coordinate arithmetic while
        // avoiding a temporary Vec for every basic-attack target comparison.
        for (row_index, row) in self.rows.iter().enumerate() {
            for (col_index, ch) in row.chars().enumerate() {
                if ch != 'X' && !(ch == '@' && self.include_anchor_tile) {
                    continue;
                }
                let dx = col_index as i32 - anchor_pattern_pos.x;
                let dy = row_index as i32 - anchor_pattern_pos.y;
                let (rotated_dx, rotated_dy) = facing.rotate_offset(dx, dy);
                let tile = Position::new(
                    anchor_world_tile.x + rotated_dx,
                    anchor_world_tile.y + rotated_dy,
                );
                contains |= tile == target_tile;
            }
        }
        Ok(contains)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TileReachQueryError {
    MissingPattern,
    MissingAnchor,
    MissingFacing,
    InvalidPattern,
}

#[derive(Clone, Copy)]
enum TileReachQueryKind<'a> {
    Pattern {
        pattern: &'a TileRangePattern,
        anchor: Position,
        facing: FacingDirection,
    },
    WholeFieldValidTiles,
}

/// A validated tile-reach projection over one battlefield snapshot.
///
/// Membership and materialization deliberately share valid-tile clipping while
/// leaving obstacle and occupancy policy to their callers.
pub(crate) struct TileReachQuery<'a> {
    battlefield: &'a BattlefieldLayout,
    kind: TileReachQueryKind<'a>,
}

impl<'a> TileReachQuery<'a> {
    pub(crate) fn try_new(
        battlefield: &'a BattlefieldLayout,
        policy: TileRangePolicy,
        pattern: Option<&'a TileRangePattern>,
        anchor: Option<Position>,
        facing: Option<FacingDirection>,
    ) -> Result<Self, TileReachQueryError> {
        let kind = match policy {
            TileRangePolicy::Pattern => {
                let pattern = pattern.ok_or(TileReachQueryError::MissingPattern)?;
                let anchor = anchor.ok_or(TileReachQueryError::MissingAnchor)?;
                let facing = facing.ok_or(TileReachQueryError::MissingFacing)?;
                pattern
                    .validate()
                    .map_err(|_| TileReachQueryError::InvalidPattern)?;
                TileReachQueryKind::Pattern {
                    pattern,
                    anchor,
                    facing,
                }
            }
            TileRangePolicy::WholeFieldValidTiles => TileReachQueryKind::WholeFieldValidTiles,
        };
        Ok(Self { battlefield, kind })
    }

    pub(crate) fn contains(&self, target: Position) -> bool {
        if !self.battlefield.is_valid_tile(target) {
            return false;
        }
        match self.kind {
            TileReachQueryKind::Pattern {
                pattern,
                anchor,
                facing,
            } => pattern
                .contains_target_tile(anchor, facing, target)
                .expect("TileReachQuery stores a validated pattern"),
            TileReachQueryKind::WholeFieldValidTiles => true,
        }
    }

    pub(crate) fn materialize_valid_cells(&self) -> Vec<Position> {
        match self.kind {
            TileReachQueryKind::Pattern {
                pattern,
                anchor,
                facing,
            } => pattern
                .affected_tiles(anchor, facing)
                .expect("TileReachQuery stores a validated pattern")
                .into_iter()
                .filter(|tile| self.battlefield.is_valid_tile(*tile))
                .collect(),
            TileReachQueryKind::WholeFieldValidTiles => self.battlefield.valid_positions(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::battlefield::BattlefieldLayout;

    #[test]
    fn membership_matches_materialization_for_all_facings_anchors_and_pattern_cells() {
        for rows in [
            vec![".X.", "X@X", "..."],
            vec!["@XXX", ".X.."],
            vec!["XX", "X@", ".X"],
        ] {
            for include_anchor_tile in [false, true] {
                let pattern = TileRangePattern {
                    include_anchor_tile,
                    rows: rows.iter().map(|row| (*row).to_owned()).collect(),
                };
                for anchor in [
                    Position::new(0, 0),
                    Position::new(-4, 7),
                    Position::new(9, -6),
                ] {
                    for facing in [
                        FacingDirection::Up,
                        FacingDirection::Right,
                        FacingDirection::Down,
                        FacingDirection::Left,
                    ] {
                        let tiles = pattern.affected_tiles(anchor, facing).unwrap();
                        for x in anchor.x - 5..=anchor.x + 5 {
                            for y in anchor.y - 5..=anchor.y + 5 {
                                let target = Position::new(x, y);
                                assert_eq!(
                                    pattern
                                        .contains_target_tile(anchor, facing, target)
                                        .unwrap(),
                                    tiles.contains(&target)
                                );
                            }
                        }
                    }
                }
            }
        }
        let invalid = TileRangePattern {
            include_anchor_tile: false,
            rows: vec!["XX".into()],
        };
        assert_eq!(
            invalid
                .contains_target_tile(
                    Position::new(0, 0),
                    FacingDirection::Up,
                    Position::new(0, 0)
                )
                .unwrap_err(),
            invalid
                .affected_tiles(Position::new(0, 0), FacingDirection::Up)
                .unwrap_err()
        );
    }

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

    #[test]
    fn typed_tile_reach_query_unifies_membership_and_stable_materialization() {
        let field = BattlefieldLayout::new_with_valid_tiles(
            5,
            5,
            vec![
                Position::new(3, 3),
                Position::new(2, 1),
                Position::new(1, 1),
                Position::new(2, 2),
                Position::new(3, 2),
                Position::new(1, 3),
                Position::new(3, 1),
                Position::new(1, 2),
            ],
        );
        let pattern = TileRangePattern {
            include_anchor_tile: true,
            rows: vec![".XX".to_string(), ".@.".to_string(), "X..".to_string()],
        };
        let anchor = Position::new(2, 2);

        for facing in [
            FacingDirection::Up,
            FacingDirection::Right,
            FacingDirection::Down,
            FacingDirection::Left,
        ] {
            let query = TileReachQuery::try_new(
                &field,
                TileRangePolicy::Pattern,
                Some(&pattern),
                Some(anchor),
                Some(facing),
            )
            .unwrap();
            let cells = query.materialize_valid_cells();
            let mut expected = pattern.affected_tiles(anchor, facing).unwrap();
            expected.retain(|tile| field.is_valid_tile(*tile));
            assert_eq!(cells, expected, "facing {facing:?}");

            for y in -1..=5 {
                for x in -1..=5 {
                    let tile = Position::new(x, y);
                    assert_eq!(
                        query.contains(tile),
                        cells.contains(&tile),
                        "facing {facing:?}, tile {tile:?}"
                    );
                }
            }
        }

        let whole_field = TileReachQuery::try_new(
            &field,
            TileRangePolicy::WholeFieldValidTiles,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            whole_field.materialize_valid_cells(),
            field.valid_positions()
        );
        for y in -1..=5 {
            for x in -1..=5 {
                let tile = Position::new(x, y);
                assert_eq!(whole_field.contains(tile), field.is_valid_tile(tile));
            }
        }
    }
}
