use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{
    ecs::resources::Field,
    game::{behavior::GameError, enums::Side},
};

use super::{BattleCore, RuntimeArtifact, RuntimeItem, RuntimeUnit};

impl BattleCore {
    pub(super) fn build_runtime_units_from_decks(&mut self, side: Side) -> Result<(), GameError> {
        let deck = match side {
            Side::Player => &self.player_info,
            Side::Opponent => &self.opponent_info,
        };

        let mut seen_artifacts: HashSet<Uuid> = HashSet::new();
        let artifact_base_uuids: Vec<Uuid> = deck.artifacts.iter().map(|a| a.base_uuid).collect();
        for (index, artifact) in deck.artifacts.iter().enumerate() {
            if !seen_artifacts.insert(artifact.base_uuid) {
                return Err(GameError::InvalidAction);
            }
            let instance_id =
                Self::make_artifact_instance_id(artifact.base_uuid, side, index as u32);
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

        let mut seen_units: HashSet<Uuid> = HashSet::new();
        for (_unit_salt, unit) in deck.units.iter().enumerate() {
            if !seen_units.insert(unit.base_uuid) {
                return Err(GameError::InvalidAction);
            }
            let stats = unit.effective_stats(&self.game_data, artifact_base_uuids.as_slice())?;

            let position = deck
                .positions
                .get(&unit.base_uuid)
                .copied()
                .ok_or(GameError::UnitNotFound)?;

            let unit_instance_id = Self::make_instance_id(unit.base_uuid, side, 0);
            let (resonance_start, resonance_max, resonance_lock_ms) = self
                .game_data
                .abnormality_data
                .get_by_uuid(&unit.base_uuid)
                .map(|meta| {
                    (
                        meta.resonance_start,
                        meta.resonance_max.max(1),
                        meta.resonance_lock_ms,
                    )
                })
                .unwrap_or((0, 100, 1000));
            let resonance_current = resonance_start.min(resonance_max);

            if self
                .units
                .insert(
                    unit_instance_id,
                    RuntimeUnit {
                        instance_id: unit_instance_id,
                        owner: side,
                        base_uuid: unit.base_uuid,
                        stats,
                        position,
                        current_target: None,
                        resonance_current,
                        resonance_max,
                        resonance_lock_ms,
                        resonance_gain_locked_until_ms: 0,
                        casting_until_ms: 0,
                        pending_cast: false,
                    },
                )
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }

            if unit.equipped_items.len() > 1 {
                let mut counts: HashMap<Uuid, usize> = HashMap::new();
                for uuid in &unit.equipped_items {
                    *counts.entry(*uuid).or_default() += 1;
                }
                for (uuid, count) in counts {
                    if count <= 1 {
                        continue;
                    }
                    let allow_duplicate = self
                        .game_data
                        .equipment_data
                        .get_by_uuid(&uuid)
                        .map(|meta| meta.allow_duplicate_equip)
                        .unwrap_or(false);
                    if !allow_duplicate {
                        return Err(GameError::InvalidAction);
                    }
                }
            }

            for (index, equipment_uuid) in unit.equipped_items.iter().enumerate() {
                let item_instance_id = Self::make_item_instance_id(
                    *equipment_uuid,
                    side,
                    unit_instance_id,
                    index as u32,
                );
                self.items.insert(
                    item_instance_id,
                    RuntimeItem {
                        instance_id: item_instance_id,
                        owner: side,
                        owner_unit_instance: unit_instance_id,
                        base_uuid: *equipment_uuid,
                    },
                );
            }
        }

        Ok(())
    }

    pub(super) fn build_runtime_field(&mut self) -> Result<(), GameError> {
        self.runtime_field = Field::new(self.runtime_field.width, self.runtime_field.height);

        for unit in self.units.values() {
            self.runtime_field
                .place(unit.instance_id, unit.owner, unit.position)?;
        }

        Ok(())
    }
}
