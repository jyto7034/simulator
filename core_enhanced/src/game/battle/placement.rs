use serde::{Deserialize, Serialize};

use crate::{
    ecs::resources::Position,
    game::{battle::core::movement::types::WorldVec2, enums::Side},
};

/// Stable identifier for a pre-combat deployment slot.
///
/// This intentionally reuses the old integer grid coordinate as an id-shaped
/// input, but runtime combat must consume `world_center` rather than treating
/// the id as a square tile center.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlacementSlotId {
    pub col: i32,
    pub row: i32,
}

impl From<Position> for PlacementSlotId {
    fn from(position: Position) -> Self {
        Self {
            col: position.x,
            row: position.y,
        }
    }
}

impl From<PlacementSlotId> for Position {
    fn from(slot_id: PlacementSlotId) -> Self {
        Position::new(slot_id.col, slot_id.row)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlacementSlot {
    pub id: PlacementSlotId,
    pub side: Side,
    pub world_center: WorldVec2,
}

/// TFT-like pre-combat placement board.
///
/// Slots use flat-top hex geometry with odd-r row offset. Combat starts at the
/// exact center of each hex cell.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlacementBoard {
    width_slots: u8,
    height_slots: u8,
}

impl PlacementBoard {
    pub const HEX_RADIUS: f32 = 0.45;
    pub const SQRT_3: f32 = 1.732_050_8;
    pub const HEX_WIDTH: f32 = Self::SQRT_3 * Self::HEX_RADIUS;
    pub const HEX_VERTICAL_SPACING: f32 = 1.5 * Self::HEX_RADIUS;
    pub const ODD_ROW_X_OFFSET: f32 = Self::HEX_WIDTH / 2.0;
    pub const ORIGIN_X: f32 = 0.5;
    pub const ORIGIN_Y: f32 = 0.5;

    pub fn new(width_slots: u8, height_slots: u8) -> Self {
        Self {
            width_slots,
            height_slots,
        }
    }

    pub fn slot(&self, side: Side, id: PlacementSlotId) -> Option<PlacementSlot> {
        self.contains(id).then(|| PlacementSlot {
            id,
            side,
            world_center: self.world_center(id),
        })
    }

    pub fn world_center(&self, id: PlacementSlotId) -> WorldVec2 {
        let row_offset = if id.row.rem_euclid(2) == 0 {
            0.0
        } else {
            Self::ODD_ROW_X_OFFSET
        };

        WorldVec2::new(
            Self::ORIGIN_X + id.col as f32 * Self::HEX_WIDTH + row_offset,
            Self::ORIGIN_Y + id.row as f32 * Self::HEX_VERTICAL_SPACING,
        )
    }

    pub fn contains(&self, id: PlacementSlotId) -> bool {
        id.col >= 0
            && id.row >= 0
            && id.col < i32::from(self.width_slots)
            && id.row < i32::from(self.height_slots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_board_maps_slots_to_flat_top_hex_centers() {
        let board = PlacementBoard::new(7, 8);

        assert_world_near(
            board.world_center(Position::new(0, 0).into()),
            WorldVec2::new(0.5, 0.5),
        );
        assert_world_near(
            board.world_center(Position::new(0, 1).into()),
            WorldVec2::new(
                PlacementBoard::ORIGIN_X + PlacementBoard::ODD_ROW_X_OFFSET,
                PlacementBoard::ORIGIN_Y + PlacementBoard::HEX_VERTICAL_SPACING,
            ),
        );
        assert_world_near(
            board.world_center(Position::new(6, 7).into()),
            WorldVec2::new(
                PlacementBoard::ORIGIN_X
                    + 6.0 * PlacementBoard::HEX_WIDTH
                    + PlacementBoard::ODD_ROW_X_OFFSET,
                PlacementBoard::ORIGIN_Y + 7.0 * PlacementBoard::HEX_VERTICAL_SPACING,
            ),
        );
    }

    #[test]
    fn adjacent_flat_top_hex_centers_are_geometrically_consistent() {
        let board = PlacementBoard::new(7, 8);
        let origin = board.world_center(Position::new(0, 0).into());
        let east = board.world_center(Position::new(1, 0).into());
        let south_east = board.world_center(Position::new(0, 1).into());

        assert_near(origin.distance(east), PlacementBoard::HEX_WIDTH);
        assert_near(origin.distance(south_east), PlacementBoard::HEX_WIDTH);
    }

    fn assert_world_near(actual: WorldVec2, expected: WorldVec2) {
        assert!(
            actual.distance(expected) <= 0.0001,
            "actual={actual:?} expected={expected:?}"
        );
    }

    fn assert_near(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "actual={actual:?} expected={expected:?}"
        );
    }
}
