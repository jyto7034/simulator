use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        behavior::GameError,
        data::GameDataBase,
        enums::{Side, Tier},
        growth::{GrowthId, GrowthStack},
        stats::UnitStats,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleWinner {
    Player,
    Opponent,
    Draw,
}

#[derive(Clone)]
pub struct PlayerDeckInfo {
    pub units: Vec<OwnedUnit>,
    pub artifacts: Vec<OwnedArtifact>,
    pub positions: HashMap<Uuid, Position>,
}

/// 어빌리티 실행에 필요한 유닛 정보
#[derive(Debug, Clone)]
pub struct UnitSnapshot {
    pub id: Uuid,
    pub owner: Side,
    pub position: Position,
    pub stats: UnitStats,
}

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

        if origin.basic_attack.interval_ms == 0 {
            return Err(GameError::InvalidUnitStats(
                "attack_interval_ms must be > 0",
            ));
        }

        let mut stats = UnitStats::with_values(
            origin.max_health,
            origin.max_health,
            origin.attack,
            origin.defense,
            origin.basic_attack.interval_ms,
        );

        // 성장형 스택 적용
        for (stat_id, value) in &self.growth_stacks.stacks {
            match stat_id {
                GrowthId::KillStack => {
                    stats.add_attack(*value);
                }
                GrowthId::PveWinStack => {}
                GrowthId::QuestRewardStack => {}
            }
        }

        // 아이템 스탯 적용
        for item_uuid in &self.equipped_items {
            let origin_item = game_data
                .equipment_data
                .get_by_uuid(item_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats.apply_permanent_effects(&origin_item.triggered_effects);
        }

        // 아티팩트 스탯 적용
        for artifact_uuid in artifacts {
            let origin_artifact = game_data
                .artifact_data
                .get_by_uuid(artifact_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats.apply_permanent_effects(&origin_artifact.triggered_effects);
        }

        // Growth 스택 / 장비 / 아티팩트는 "영구 스탯"으로 간주하므로,
        // 최종 max_health 기준으로 전투 시작 HP는 풀피로 맞춘다.
        stats.current_health = stats.max_health;

        Ok(stats)
    }
}
