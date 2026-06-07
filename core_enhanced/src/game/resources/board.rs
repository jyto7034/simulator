use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::behavior::GameError;

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

#[derive(Debug, Clone)]
pub struct RosterOrder {
    pub max_slots: usize,
    pub slots: Vec<Option<Uuid>>,
    unit_slots: HashMap<Uuid, usize>,
}

impl Default for RosterOrder {
    fn default() -> Self {
        Self::new(8)
    }
}

impl RosterOrder {
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

    pub fn sync_owned_units(&mut self, owned_units: &[Uuid]) {
        let owned_set = owned_units
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        let assigned_units = self.unit_slots.keys().copied().collect::<Vec<_>>();

        for unit_uuid in assigned_units {
            if !owned_set.contains(&unit_uuid) {
                self.remove(unit_uuid);
            }
        }

        for unit_uuid in owned_units {
            if self.unit_slots.contains_key(unit_uuid) {
                continue;
            }

            let _ = self.place_first_available(*unit_uuid);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roster_order_places_and_swaps_units() {
        let left = Uuid::from_u128(1);
        let right = Uuid::from_u128(2);
        let mut roster_order = RosterOrder::new(2);

        roster_order.place_at(left, 0).unwrap();
        roster_order.place_at(right, 1).unwrap();
        roster_order.move_unit(left, 1, Some(right)).unwrap();

        assert_eq!(roster_order.slot_of(left), Some(1));
        assert_eq!(roster_order.slot_of(right), Some(0));
    }
}
