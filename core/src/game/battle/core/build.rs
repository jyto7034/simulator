use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::ecs::resources::Position;
use crate::game::{
    battle::core::{BattleCore, RuntimeArtifact, RuntimeItem, RuntimeUnit},
    behavior::GameError,
    enums::Side,
};

impl BattleCore {
    pub(super) fn build_runtime_units_from_decks(&mut self, side: Side) -> Result<(), GameError> {
        let deck = match side {
            Side::Opponent => &self.opponent_info,
            Side::Player => &self.player_info,
        };

        let mut seen_artifacts: HashSet<Uuid> = HashSet::new();
        for (idx, artifact) in deck.artifacts.iter().enumerate() {
            if !seen_artifacts.insert(artifact.base_uuid) {
                return Err(GameError::InvalidAction);
            }
            let instance_id = Self::make_artifact_instance_id(artifact.base_uuid, side, idx as u32);
            if self
                .artifacts
                .insert(
                    instance_id,
                    RuntimeArtifact {
                        instance_id,
                        owner: side,
                        base_uuid: artifact.base_uuid,
                    },
                )
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }
        }

        let artifact_base_uuids: Vec<Uuid> = deck.artifacts.iter().map(|a| a.base_uuid).collect();
        for (idx, unit) in deck.units.iter().enumerate() {
            let stats = unit.effective_stats(&self.game_data, &artifact_base_uuids)?;

            let position = deck
                .positions
                .get(&unit.base_uuid)
                .copied()
                .ok_or(GameError::UnitNotFound)?;

            let instance_id = Self::make_instance_id(unit.base_uuid, side, idx as u32);
            let (resonance_start, resonance_max, resonance_lock_ms) = self
                .game_data
                .abnormality_data
                .get_by_uuid(&unit.base_uuid)
                .map(|meta| {
                    (
                        meta.resonance.start,
                        meta.resonance.max.max(1),
                        meta.resonance.gain_lock_ms,
                    )
                })
                .unwrap_or((0, 100, 1000));
            let resonance_current = resonance_start.min(resonance_max);
            let move_speed_units_per_ms = self
                .game_data
                .abnormality_data
                .get_by_uuid(&unit.base_uuid)
                .map(|meta| meta.movement.speed_units_per_ms)
                .unwrap_or(3000);

            if self
                .units
                .insert(
                    instance_id,
                    RuntimeUnit {
                        instance_id: instance_id,
                        owner: side,
                        base_uuid: unit.base_uuid,
                        stats,
                        position,
                        current_target: None,
                        resonance_current,
                        resonance_max,
                        resonance_lock_ms,
                        resonance_gain_locked_until_ms: 0,
                        next_action_time: 0,
                        pending_cast: false,
                        pending_cast_cause: None,
                        move_state: super::MoveState::Acquire,
                        soft_until_ms: None,
                        move_progress_units: 0,
                        move_last_update_ms: 0,
                        move_speed_units_per_ms,
                        move_next_step_ms: None,
                        reserved_destination: None,
                        move_path: Vec::new(),
                        repath_counter: 0,
                    },
                )
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }

            for (idx, equipment_uuid) in unit.equipped_items.iter().enumerate() {
                let item_instance_id =
                    Self::make_item_instance_id(*equipment_uuid, side, instance_id, idx as u32);
                self.items.insert(
                    item_instance_id,
                    RuntimeItem {
                        instance_id: item_instance_id,
                        owner: side,
                        owner_unit_instance: instance_id,
                        base_uuid: *equipment_uuid,
                    },
                );
            }
        }

        Ok(())
    }

    pub(super) fn build_runtime_field(&mut self) -> Result<(), GameError> {
        self.movement_field =
            super::RuntimeField::new(self.movement_field.width, self.movement_field.height);

        // Validate: all units are in-bounds and no two units share the same tile at spawn.
        let mut occupied: HashMap<Position, Uuid> = HashMap::new();
        for unit in self.units.values() {
            if !self.movement_field.in_bounds(unit.position) {
                return Err(GameError::OutOfBounds);
            }
            if occupied.insert(unit.position, unit.instance_id).is_some() {
                return Err(GameError::PositionOccupied);
            }
        }

        Ok(())
    }
}
