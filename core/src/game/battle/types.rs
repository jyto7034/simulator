use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        battle::ids::UnitInstanceId,
        battle::timeline::Timeline,
        behavior::GameError,
        data::GameDataBase,
        enums::{Side, Tier},
        growth::{GrowthId, GrowthStack},
        stats::UnitStats,
    },
};

pub struct BattleResult {
    pub winner: BattleWinner,
    pub timeline: Timeline,
}

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
    pub id: UnitInstanceId,
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
    pub owned_uuid: Uuid,
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
        stats.magic_resist = origin.magic_resist;

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

            stats
                .apply_permanent_effects(&origin_item.triggered_effects)
                .map_err(|err| match err {
                    GameError::InvalidStaticData(message) => GameError::InvalidStaticData(format!(
                        "equipment '{}' invalid Permanent effect: {}",
                        origin_item.id, message
                    )),
                    other => other,
                })?;
        }

        // 아티팩트 스탯 적용
        for artifact_uuid in artifacts {
            let origin_artifact = game_data
                .artifact_data
                .get_by_uuid(artifact_uuid)
                .ok_or(GameError::MissingResource(""))?;

            stats
                .apply_permanent_effects(&origin_artifact.triggered_effects)
                .map_err(|err| match err {
                    GameError::InvalidStaticData(message) => GameError::InvalidStaticData(format!(
                        "artifact '{}' invalid Permanent effect: {}",
                        origin_artifact.id, message
                    )),
                    other => other,
                })?;
        }

        // Growth 스택 / 장비 / 아티팩트는 "영구 스탯"으로 간주하므로,
        // 최종 max_health 기준으로 전투 시작 HP는 풀피로 맞춘다.
        stats.current_health = stats.max_health;

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::{
        abnormality_data::{AbnormalityDatabase, AbnormalityMetadata},
        artifact_data::{ArtifactDatabase, ArtifactMetadata},
        bonus_data::BonusDatabase,
        equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType},
        event_pools::{EventPhasePool, EventPoolConfig},
        pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        GameDataBase,
    };
    use crate::game::enums::RiskLevel;
    use crate::game::stats::{
        Effect, StatId, StatModifier, StatModifierKind, TriggerEffectTarget, TriggerType,
        TriggeredEffect,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    fn empty_event_pools() -> EventPoolConfig {
        let pool = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        EventPoolConfig {
            dawn: pool.clone(),
            noon: pool.clone(),
            dusk: pool.clone(),
            midnight: pool.clone(),
            white: pool,
        }
    }

    #[test]
    fn effective_stats_applies_growth_and_permanent_item_and_artifact_effects() {
        let abno_uuid = Uuid::from_u128(1);
        let item_uuid = Uuid::from_u128(2);
        let artifact_uuid = Uuid::from_u128(3);

        let abno = AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: abno_uuid,
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 5,
            magic_resist: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };

        let mut item_triggers: HashMap<TriggerType, Vec<TriggeredEffect>> = HashMap::new();
        item_triggers.insert(
            TriggerType::Permanent,
            vec![TriggeredEffect::legacy(Effect::Modifier(StatModifier {
                stat: StatId::Attack,
                kind: StatModifierKind::Flat,
                value: 7,
            }))],
        );
        let item = EquipmentMetadata {
            id: "item".to_string(),
            uuid: item_uuid,
            name: "Item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            triggered_effects: item_triggers,
            ability_activations: vec![],
        };

        let mut artifact_triggers: HashMap<TriggerType, Vec<TriggeredEffect>> = HashMap::new();
        artifact_triggers.insert(
            TriggerType::Permanent,
            vec![TriggeredEffect::legacy(Effect::Modifier(StatModifier {
                stat: StatId::Defense,
                kind: StatModifierKind::Flat,
                value: 3,
            }))],
        );
        let artifact = ArtifactMetadata {
            id: "artifact".to_string(),
            uuid: artifact_uuid,
            name: "Artifact".to_string(),
            description: "".to_string(),
            rarity: RiskLevel::ZAYIN,
            price: 0,
            triggered_effects: artifact_triggers,
            ability_activations: vec![],
        };

        let game_data = GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![abno])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![artifact])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![item])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: empty_event_pools(),
        });

        let mut growth = GrowthStack::new();
        growth.add(GrowthId::KillStack, 2);

        let unit = OwnedUnit {
            owned_uuid: Uuid::from_u128(10),
            base_uuid: abno_uuid,
            level: Tier::I,
            growth_stacks: growth,
            equipped_items: vec![item_uuid],
        };

        let stats = unit.effective_stats(&game_data, &[artifact_uuid]).unwrap();
        assert_eq!(stats.max_health, 100);
        assert_eq!(stats.current_health, 100);
        assert_eq!(stats.attack, 10 + 2 + 7);
        assert_eq!(stats.defense, 5 + 3);
        assert!(stats.attack_interval_ms > 0);
    }

    #[test]
    fn effective_stats_errors_when_abnormality_missing() {
        let game_data = GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: empty_event_pools(),
        });

        let unit = OwnedUnit {
            owned_uuid: Uuid::from_u128(10),
            base_uuid: Uuid::from_u128(1),
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        };

        let err = unit.effective_stats(&game_data, &[]).unwrap_err();
        assert!(matches!(
            err,
            GameError::MissingResource("AbnormalityMetadata")
        ));
    }

    #[test]
    fn effective_stats_errors_when_attack_interval_is_zero() {
        let abno_uuid = Uuid::from_u128(1);
        let mut abno = AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: abno_uuid,
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 5,
            magic_resist: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };
        abno.basic_attack.interval_ms = 0;

        let game_data = GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![abno])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: empty_event_pools(),
        });

        let unit = OwnedUnit {
            owned_uuid: Uuid::from_u128(10),
            base_uuid: abno_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        };

        let err = unit.effective_stats(&game_data, &[]).unwrap_err();
        assert!(matches!(err, GameError::InvalidUnitStats(_)));
    }

    #[test]
    fn effective_stats_rejects_invalid_permanent_targeting() {
        let abno_uuid = Uuid::from_u128(1);
        let artifact_uuid = Uuid::from_u128(2);

        let abno = AbnormalityMetadata {
            id: "abno".to_string(),
            uuid: abno_uuid,
            name: "Abno".to_string(),
            risk_level: RiskLevel::ZAYIN,
            price: 0,
            max_health: 100,
            attack: 10,
            defense: 5,
            magic_resist: 0,
            movement: Default::default(),
            basic_attack: Default::default(),
            resonance: Default::default(),
            skill_id: None,
        };

        let mut artifact_triggers: HashMap<TriggerType, Vec<TriggeredEffect>> = HashMap::new();
        artifact_triggers.insert(
            TriggerType::Permanent,
            vec![TriggeredEffect::targeted(
                TriggerEffectTarget::CounterpartUnit,
                Effect::Modifier(StatModifier {
                    stat: StatId::Defense,
                    kind: StatModifierKind::Flat,
                    value: 3,
                }),
            )],
        );
        let artifact = ArtifactMetadata {
            id: "artifact".to_string(),
            uuid: artifact_uuid,
            name: "Artifact".to_string(),
            description: "".to_string(),
            rarity: RiskLevel::ZAYIN,
            price: 0,
            triggered_effects: artifact_triggers,
            ability_activations: vec![],
        };

        let game_data = GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![abno])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![artifact])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: empty_event_pools(),
        });

        let unit = OwnedUnit {
            owned_uuid: Uuid::from_u128(10),
            base_uuid: abno_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        };

        assert!(matches!(
            unit.effective_stats(&game_data, &[artifact_uuid]),
            Err(GameError::InvalidStaticData(_))
        ));
    }
}
