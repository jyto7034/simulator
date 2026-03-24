use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{behavior::GameError, enums::Side};

use super::{
    movement::{ActionState, TILE_UNITS_PER_TILE},
    BattleCore, RuntimeArtifact, RuntimeItem, RuntimeUnit,
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
            let mut stats = unit.effective_stats(&self.game_data, &artifact_base_uuids)?;

            let position = deck
                .positions
                .get(&unit.owned_uuid)
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
            stats.move_speed_units_per_ms = move_speed_units_per_ms.max(1);

            if self
                .units
                .insert(
                    instance_id,
                    RuntimeUnit {
                        instance_id: instance_id,
                        owner: side,
                        base_uuid: unit.base_uuid,
                        stats,
                        pos_x_units: (position.x as i64).saturating_mul(TILE_UNITS_PER_TILE as i64),
                        pos_y_units: (position.y as i64).saturating_mul(TILE_UNITS_PER_TILE as i64),
                        move_epoch: 0,
                        action_state: ActionState::Idle,
                        action_locks: Default::default(),
                        current_target: None,
                        next_basic_attack_ms: 0,
                        pending_basic_attack: false,
                        resonance_current,
                        resonance_max,
                        resonance_lock_ms,
                        next_action_time: 0,
                        pending_cast: false,
                        pending_cast_cause: None,
                        pending_skill_cast: None,
                    },
                )
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }

            self.battlefield.place(instance_id, position)?;

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
        for unit_id in self.units.keys().copied().collect::<Vec<_>>() {
            if self.battlefield.position_of(unit_id).is_none() {
                return Err(GameError::UnitNotFound);
            }
        }
        Ok(())
    }
}
