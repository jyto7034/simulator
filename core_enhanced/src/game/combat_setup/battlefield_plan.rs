//! Converts a generated combat preview into the static battlefield plan used to start battle.
//!
//! This module intentionally owns only field dimensions, valid tiles, and static
//! obstacles. Spawn groups, mission policy, and rewards are assembled by sibling
//! combat setup modules.

use crate::game::{
    battle::scenario::{BattleFieldSpec, BattleScenario},
    behavior::GameError,
    combat_preview::CombatPreview,
    data::GameDataBase,
    resources::Position,
};

#[derive(Debug, Clone)]
pub(crate) struct BattleStartPlan {
    pub(crate) field_size: (u8, u8),
    valid_tiles: Vec<Position>,
    static_obstacles: Vec<Position>,
}

impl BattleStartPlan {
    pub(crate) fn from_preview(
        game_data: &GameDataBase,
        encounter_id: &str,
        combat_preview: &CombatPreview,
    ) -> Result<Self, GameError> {
        game_data
            .pve_data
            .get_by_id(encounter_id)
            .ok_or(GameError::MissingResource("PveEncounter"))?;
        let field_size = (
            checked_preview_dimension(combat_preview.width)?,
            checked_preview_dimension(combat_preview.height)?,
        );

        Ok(Self {
            field_size,
            valid_tiles: combat_preview.valid_tiles.clone(),
            static_obstacles: combat_preview.obstacles.clone(),
        })
    }

    pub(crate) fn into_battlefield_spec(self) -> BattleFieldSpec {
        BattleFieldSpec {
            width: self.field_size.0,
            height: self.field_size.1,
            valid_tiles: self.valid_tiles,
            obstacles: self.static_obstacles,
        }
    }
}

pub(crate) fn validate_static_obstacles_do_not_overlap_scenario(
    scenario: &BattleScenario,
) -> Result<(), GameError> {
    for obstacle in &scenario.battlefield.obstacles {
        if scenario
            .groups
            .iter()
            .flat_map(|group| group.spawns.iter())
            .any(|spawn| spawn.position == *obstacle)
        {
            return Err(GameError::StaticObstacleBlocked);
        }
    }

    Ok(())
}

fn checked_preview_dimension(value: i32) -> Result<u8, GameError> {
    u8::try_from(value).map_err(|_| {
        GameError::InvalidStaticData(format!(
            "battlefield dimension must fit u8 and be non-negative: {}",
            value
        ))
    })
}
