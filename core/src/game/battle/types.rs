use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{behavior::GameError, data::GameDataBase, enums::Tier, stats::UnitStats},
};

use super::Timeline;

#[derive(Clone)]
pub struct PlayerDeckInfo {
    pub units: Vec<OwnedUnit>,
    pub artifacts: Vec<OwnedArtifact>,
    pub positions: HashMap<Uuid, Position>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleWinner {
    Player,
    Opponent,
    Draw,
}

pub struct BattleResult {
    pub winner: BattleWinner,
    pub timeline: Timeline,
}

pub struct Event {}

#[derive(Debug, Clone)]
pub struct OwnedArtifact {
    pub base_uuid: Uuid,
}

#[derive(Debug, Clone)]
pub struct OwnedItem {
    pub base_uuid: Uuid,
}

#[derive(Debug, Clone)]
pub struct OwnedUnit {
    pub base_uuid: Uuid,
    pub level: Tier,
    pub growth_stacks: GrowthStack,
    pub equipped_items: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrowthId {
    KillStack,
    PveWinStack,
    QuestRewardStack,
}

#[derive(Debug, Clone, Default)]
pub struct GrowthStack {
    pub stacks: HashMap<GrowthId, i32>,
}

impl GrowthStack {
    pub fn new() -> Self {
        Self {
            stacks: HashMap::new(),
        }
    }
}

impl OwnedUnit {
    pub fn effective_stats(
        &self,
        game_data: &GameDataBase,
        artifacts: &[Uuid],
    ) -> Result<UnitStats, GameError> {
        let origin = game_data
            .abnormality_data
            .get_by_uuid(&self.base_uuid)
            .ok_or(GameError::MissingResource("AbnormalityMetadata"))?;

        if origin.attack_interval_ms == 0 {
            return Err(GameError::InvalidUnitStats(
                "attack_interval_ms must be > 0",
            ));
        }

        let mut stats = UnitStats::with_values(
            origin.max_health,
            origin.max_health,
            origin.attack,
            origin.defense,
            origin.attack_interval_ms,
        );

        for (stat_id, value) in &self.growth_stacks.stacks {
            match stat_id {
                GrowthId::KillStack => {
                    stats.add_attack(*value);
                }
                GrowthId::PveWinStack => {}
                GrowthId::QuestRewardStack => {}
            }
        }

        for item_uuid in &self.equipped_items {
            let origin_item = game_data
                .equipment_data
                .get_by_uuid(item_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats.apply_permanent_effects(&origin_item.triggered_effects);
        }

        for artifact_uuid in artifacts {
            let origin_artifact = game_data
                .artifact_data
                .get_by_uuid(artifact_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats.apply_permanent_effects(&origin_artifact.triggered_effects);
        }

        Ok(stats)
    }
}
