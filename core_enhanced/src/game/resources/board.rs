use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{behavior::GameError, enums::Side};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn manhattan(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn chebyshev(&self, other: &Position) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitPlacement {
    pub uuid: Uuid,
    pub side: Side,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub width: u8,
    pub height: u8,
    pub placements: HashMap<Position, UnitPlacement>,
    pub unit_positions: HashMap<Uuid, Position>,
}

impl Default for Field {
    fn default() -> Self {
        Self::new(3, 3)
    }
}

impl Field {
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            width,
            height,
            placements: HashMap::new(),
            unit_positions: HashMap::new(),
        }
    }

    pub fn place(&mut self, uuid: Uuid, side: Side, pos: Position) -> Result<(), GameError> {
        if pos.x < 0 || pos.x >= self.width as i32 || pos.y < 0 || pos.y >= self.height as i32 {
            return Err(GameError::OutOfBounds);
        }
        if self.placements.contains_key(&pos) {
            return Err(GameError::PositionOccupied);
        }
        if self.unit_positions.contains_key(&uuid) {
            return Err(GameError::UnitAlreadyPlaced);
        }

        self.placements.insert(pos, UnitPlacement { uuid, side });
        self.unit_positions.insert(uuid, pos);
        Ok(())
    }

    pub fn has_unit_on_side(&self, side: Side) -> bool {
        self.placements
            .values()
            .any(|placement| placement.side == side)
    }

    pub fn remove(&mut self, unit_uuid: Uuid) -> Option<Position> {
        let pos = self.unit_positions.remove(&unit_uuid)?;
        self.placements.remove(&pos);
        Some(pos)
    }

    pub fn move_unit(&mut self, unit_uuid: Uuid, new_pos: Position) -> Result<(), GameError> {
        if pos_out_of_bounds(new_pos, self.width, self.height) {
            return Err(GameError::OutOfBounds);
        }

        let old_pos = self
            .unit_positions
            .get(&unit_uuid)
            .ok_or(GameError::UnitNotFound)?;
        if *old_pos == new_pos {
            return Ok(());
        }
        if self.placements.contains_key(&new_pos) {
            return Err(GameError::PositionOccupied);
        }

        let placement = self
            .placements
            .remove(old_pos)
            .ok_or(GameError::UnitNotFound)?;
        self.placements.insert(new_pos, placement);
        self.unit_positions.insert(unit_uuid, new_pos);
        Ok(())
    }

    pub fn swap_units(&mut self, left_uuid: Uuid, right_uuid: Uuid) -> Result<(), GameError> {
        if left_uuid == right_uuid {
            return Ok(());
        }

        let left_pos = *self
            .unit_positions
            .get(&left_uuid)
            .ok_or(GameError::UnitNotFound)?;
        let right_pos = *self
            .unit_positions
            .get(&right_uuid)
            .ok_or(GameError::UnitNotFound)?;
        let left_placement = self
            .placements
            .remove(&left_pos)
            .ok_or(GameError::UnitNotFound)?;
        let right_placement = self
            .placements
            .remove(&right_pos)
            .ok_or(GameError::UnitNotFound)?;

        self.placements.insert(left_pos, right_placement);
        self.placements.insert(right_pos, left_placement);
        self.unit_positions.insert(left_uuid, right_pos);
        self.unit_positions.insert(right_uuid, left_pos);
        Ok(())
    }

    pub fn get_position(&self, unit_uuid: Uuid) -> Option<Position> {
        self.unit_positions.get(&unit_uuid).copied()
    }

    pub fn get_unit_at(&self, pos: Position) -> Option<Uuid> {
        self.placements.get(&pos).map(|p| p.uuid)
    }

    pub fn get_placement_at(&self, pos: Position) -> Option<&UnitPlacement> {
        self.placements.get(&pos)
    }

    pub fn find_nearest_enemy(&self, from_uuid: Uuid, from_side: Side) -> Option<Uuid> {
        let from_pos = self.unit_positions.get(&from_uuid)?;
        self.find_nearest(from_pos, from_side, true)
    }

    pub fn find_nearest_ally(&self, from_uuid: Uuid, from_side: Side) -> Option<Uuid> {
        let from_pos = self.unit_positions.get(&from_uuid)?;
        self.find_nearest(from_pos, from_side, false)
    }

    fn find_nearest(&self, from_pos: &Position, from_side: Side, find_enemy: bool) -> Option<Uuid> {
        let mut nearest: Option<(Uuid, i32)> = None;

        for (pos, placement) in &self.placements {
            let is_enemy = placement.side != from_side;
            if is_enemy != find_enemy {
                continue;
            }

            let distance = from_pos.chebyshev(pos);
            match nearest {
                None => nearest = Some((placement.uuid, distance)),
                Some((_best_uuid, best_dist)) if distance < best_dist => {
                    nearest = Some((placement.uuid, distance));
                }
                Some((best_uuid, best_dist))
                    if distance == best_dist
                        && placement.uuid.as_bytes() < best_uuid.as_bytes() =>
                {
                    nearest = Some((placement.uuid, distance));
                }
                _ => {}
            }
        }

        nearest.map(|(uuid, _)| uuid)
    }

    pub fn get_units_by_side(&self, side: Side) -> Vec<Uuid> {
        self.placements
            .values()
            .filter(|p| p.side == side)
            .map(|p| p.uuid)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    pub fn count_by_side(&self, side: Side) -> usize {
        self.placements.values().filter(|p| p.side == side).count()
    }

    pub fn get_positions_by_side(&self, side: Side) -> HashMap<Uuid, Position> {
        self.placements
            .iter()
            .filter(|(_, p)| p.side == side)
            .map(|(pos, p)| (p.uuid, *pos))
            .collect()
    }

    pub fn clear(&mut self) {
        self.placements.clear();
        self.unit_positions.clear();
    }

    pub fn clear_side(&mut self, side: Side) {
        let uuids_to_remove = self
            .placements
            .iter()
            .filter(|(_, p)| p.side == side)
            .map(|(_, p)| p.uuid)
            .collect::<Vec<_>>();

        for uuid in uuids_to_remove {
            self.remove(uuid);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bench {
    pub max_slots: usize,
    pub slots: Vec<Option<Uuid>>,
    unit_slots: HashMap<Uuid, usize>,
}

impl Default for Bench {
    fn default() -> Self {
        Self::new(8)
    }
}

impl Bench {
    pub fn new(max_slots: usize) -> Self {
        Self {
            max_slots,
            slots: vec![None; max_slots],
            unit_slots: HashMap::new(),
        }
    }

    pub fn slot_of(&self, unit_uuid: Uuid) -> Option<usize> {
        self.unit_slots.get(&unit_uuid).copied()
    }

    pub fn occupant(&self, slot: usize) -> Option<Uuid> {
        self.slots.get(slot).copied().flatten()
    }

    pub fn remove(&mut self, unit_uuid: Uuid) -> Option<usize> {
        let slot = self.unit_slots.remove(&unit_uuid)?;
        if self.slots.get(slot).copied().flatten() == Some(unit_uuid) {
            self.slots[slot] = None;
        }
        Some(slot)
    }

    pub fn place_first_available(&mut self, unit_uuid: Uuid) -> Result<usize, GameError> {
        if let Some(slot) = self.slot_of(unit_uuid) {
            return Ok(slot);
        }

        let slot = self
            .slots
            .iter()
            .position(Option::is_none)
            .ok_or(GameError::InventoryFull)?;
        self.place_at(unit_uuid, slot)
    }

    pub fn place_at(&mut self, unit_uuid: Uuid, slot: usize) -> Result<usize, GameError> {
        if slot >= self.max_slots {
            return Err(GameError::OutOfBounds);
        }
        if self.slots[slot].is_some() {
            return Err(GameError::PositionOccupied);
        }

        self.remove(unit_uuid);
        self.slots[slot] = Some(unit_uuid);
        self.unit_slots.insert(unit_uuid, slot);
        Ok(slot)
    }

    pub fn replace_unit(
        &mut self,
        removed_unit: Uuid,
        placed_unit: Uuid,
    ) -> Result<usize, GameError> {
        let slot = self.remove(removed_unit).ok_or(GameError::UnitNotFound)?;
        self.place_at(placed_unit, slot)
    }

    pub fn move_unit(
        &mut self,
        unit_uuid: Uuid,
        dest_slot: usize,
        swap_with_unit_uuid: Option<Uuid>,
    ) -> Result<(), GameError> {
        if dest_slot >= self.max_slots {
            return Err(GameError::OutOfBounds);
        }

        let source_slot = self.slot_of(unit_uuid).ok_or(GameError::UnitNotFound)?;
        if source_slot == dest_slot {
            return Ok(());
        }

        if let Some(occupant_uuid) = self.occupant(dest_slot) {
            if swap_with_unit_uuid != Some(occupant_uuid) {
                return Err(GameError::PositionOccupied);
            }

            self.slots[source_slot] = Some(occupant_uuid);
            self.slots[dest_slot] = Some(unit_uuid);
            self.unit_slots.insert(unit_uuid, dest_slot);
            self.unit_slots.insert(occupant_uuid, source_slot);
            return Ok(());
        }

        self.slots[source_slot] = None;
        self.slots[dest_slot] = Some(unit_uuid);
        self.unit_slots.insert(unit_uuid, dest_slot);
        Ok(())
    }

    pub fn sync_owned_units(&mut self, owned_units: &[Uuid], field_units: &HashSet<Uuid>) {
        let owned_set = owned_units.iter().copied().collect::<HashSet<_>>();
        let assigned_units = self.unit_slots.keys().copied().collect::<Vec<_>>();

        for unit_uuid in assigned_units {
            if !owned_set.contains(&unit_uuid) || field_units.contains(&unit_uuid) {
                self.remove(unit_uuid);
            }
        }

        for unit_uuid in owned_units {
            if field_units.contains(unit_uuid) || self.unit_slots.contains_key(unit_uuid) {
                continue;
            }

            let _ = self.place_first_available(*unit_uuid);
        }
    }
}

fn pos_out_of_bounds(pos: Position, width: u8, height: u8) -> bool {
    pos.x < 0 || pos.x >= width as i32 || pos.y < 0 || pos.y >= height as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_places_moves_and_removes_units() {
        let unit = Uuid::from_u128(1);
        let mut field = Field::new(3, 3);

        field
            .place(unit, Side::Player, Position::new(0, 0))
            .unwrap();
        assert_eq!(field.get_position(unit), Some(Position::new(0, 0)));

        field.move_unit(unit, Position::new(1, 1)).unwrap();
        assert_eq!(field.get_unit_at(Position::new(1, 1)), Some(unit));
        assert_eq!(field.remove(unit), Some(Position::new(1, 1)));
        assert!(field.is_empty());
    }

    #[test]
    fn bench_places_and_swaps_units() {
        let left = Uuid::from_u128(1);
        let right = Uuid::from_u128(2);
        let mut bench = Bench::new(2);

        bench.place_at(left, 0).unwrap();
        bench.place_at(right, 1).unwrap();
        bench.move_unit(left, 1, Some(right)).unwrap();

        assert_eq!(bench.slot_of(left), Some(1));
        assert_eq!(bench.slot_of(right), Some(0));
    }
}
