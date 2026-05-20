use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    game::resources::Position,
    game::{
        ability::SkillId,
        battle::core::movement::types::WorldVec2,
        battle::ids::UnitInstanceId,
        battle::timeline::Timeline,
        behavior::GameError,
        data::{
            abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
            GameDataBase,
        },
        enums::{Side, Tier},
        growth::{GrowthId, GrowthStack},
        stats::UnitStats,
    },
};

pub struct BattleResult {
    pub winner: BattleWinner,
    pub timeline: Timeline,
    pub participant_results: Vec<ParticipantBattleResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantBattleResult {
    pub unit_instance_id: UnitInstanceId,
    pub owned_uuid: Uuid,
    pub side: Side,
    pub survived: bool,
    pub final_hp: u32,
    pub max_hp: u32,
    pub became_incapacitated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleWinner {
    Player,
    Opponent,
    Draw,
}

/// 어빌리티 실행에 필요한 유닛 정보
#[derive(Debug, Clone)]
pub struct UnitSnapshot {
    pub id: UnitInstanceId,
    pub owner: Side,
    pub role: BattleUnitRole,
    pub position: Position,
    pub world_position: WorldVec2,
    pub stats: UnitStats,
}

#[derive(Debug, Clone)]
pub struct BattleUnitDraft {
    pub owned_uuid: Uuid,
    pub source: BattleUnitSource,
    pub level: Tier,
    pub growth_stacks: GrowthStack,
    pub equipped_items: Vec<Uuid>,
    pub equipped_item_enhancements: Vec<BattleEquipmentEnhancement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleEquipmentEnhancement {
    pub base_uuid: Uuid,
    pub enhancement_level: u8,
}

#[derive(Debug, Clone)]
pub enum BattleUnitSource {
    Employee(UnitCombatProfile),
    Abnormality {
        base_uuid: Uuid,
    },
    CorrodedEmployee {
        profile_id: String,
        base_uuid: Uuid,
    },
    TestFixture {
        base_uuid: Uuid,
        profile: UnitCombatProfile,
    },
    DefenseObject {
        base_uuid: Uuid,
        profile: UnitCombatProfile,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BattleUnitRole {
    #[default]
    Combatant,
    DefenseObject,
}

impl BattleUnitSource {
    pub fn base_uuid(&self, owned_uuid: Uuid) -> Uuid {
        match self {
            BattleUnitSource::Employee(_) => owned_uuid,
            BattleUnitSource::Abnormality { base_uuid } => *base_uuid,
            BattleUnitSource::CorrodedEmployee { base_uuid, .. } => *base_uuid,
            BattleUnitSource::TestFixture { base_uuid, .. } => *base_uuid,
            BattleUnitSource::DefenseObject { base_uuid, .. } => *base_uuid,
        }
    }

    pub fn role(&self) -> BattleUnitRole {
        match self {
            BattleUnitSource::DefenseObject { .. } => BattleUnitRole::DefenseObject,
            _ => BattleUnitRole::Combatant,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnitCombatProfile {
    pub stats: UnitStats,
    pub basic_attack: BasicAttackDef,
    pub movement: MovementDef,
    pub resonance: ResonanceDef,
    pub skill_id: Option<SkillId>,
}

impl UnitCombatProfile {
    pub fn employee_default() -> Self {
        let basic_attack = BasicAttackDef {
            range_units: 1.0,
            interval_ms: 1500,
            windup_ms: 200,
            delivery: Default::default(),
        };
        let movement = MovementDef::default();
        let mut stats = UnitStats::with_values(100, 100, 10, 0, basic_attack.interval_ms);
        stats.magic_resist = 0;
        stats.move_speed_units_per_ms = movement.speed_units_per_ms;

        Self {
            stats,
            basic_attack,
            movement,
            resonance: ResonanceDef::default(),
            skill_id: None,
        }
    }
}

impl BattleUnitDraft {
    pub fn base_uuid(&self) -> Uuid {
        self.source.base_uuid(self.owned_uuid)
    }

    pub fn combat_profile_from_abnormality(
        base_uuid: Uuid,
        game_data: &GameDataBase,
    ) -> Result<UnitCombatProfile, GameError> {
        let meta = game_data
            .abnormality_data
            .get_by_uuid(&base_uuid)
            .ok_or(GameError::MissingResource("AbnormalityMetadata"))?;

        if meta.basic_attack.interval_ms == 0 {
            return Err(GameError::InvalidUnitStats(
                "attack_interval_ms must be > 0",
            ));
        }

        let mut stats = UnitStats::with_values(
            meta.max_health,
            meta.max_health,
            meta.attack,
            meta.defense,
            meta.basic_attack.interval_ms,
        );
        stats.magic_resist = meta.magic_resist;
        stats.move_speed_units_per_ms = meta.movement.speed_units_per_ms;

        Ok(UnitCombatProfile {
            stats,
            basic_attack: meta.basic_attack.clone(),
            movement: meta.movement.clone(),
            resonance: meta.resonance.clone(),
            skill_id: meta.skill_id.clone(),
        })
    }

    pub fn combat_profile(&self, game_data: &GameDataBase) -> Result<UnitCombatProfile, GameError> {
        match &self.source {
            BattleUnitSource::Employee(profile) => Ok(profile.clone()),
            BattleUnitSource::Abnormality { base_uuid } => {
                Self::combat_profile_from_abnormality(*base_uuid, game_data)
            }
            BattleUnitSource::CorrodedEmployee { profile_id, .. } => game_data
                .corroded_employee_data
                .get_by_id(profile_id)
                .map(|profile| profile.to_combat_profile())
                .ok_or(GameError::MissingResource("CorrodedEmployeeProfile")),
            BattleUnitSource::TestFixture { profile, .. } => Ok(profile.clone()),
            BattleUnitSource::DefenseObject { profile, .. } => Ok(profile.clone()),
        }
    }

    pub fn effective_stats(
        &self,
        game_data: &GameDataBase,
        artifacts: &[Uuid],
    ) -> Result<UnitStats, GameError> {
        let base_profile = self.combat_profile(game_data)?;
        let base_current_health = base_profile.stats.current_health;
        let base_max_health = base_profile.stats.max_health.max(1);
        let mut stats = base_profile.stats;

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

            let enhancement_level: u32 = self
                .equipped_item_enhancements
                .iter()
                .filter(|enhancement| enhancement.base_uuid == *item_uuid)
                .map(|enhancement| u32::from(enhancement.enhancement_level))
                .sum();
            if enhancement_level > 0 {
                if let Some(recipe) = game_data
                    .equipment_data
                    .get_enhancement_recipe_by_equipment_id(&origin_item.id)
                {
                    for modifier in &recipe.modifiers_per_level {
                        for _ in 0..enhancement_level {
                            stats.apply_modifier(*modifier);
                        }
                    }
                }
            }
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

        // Growth/gear/artifacts can change max HP. Preserve the authored run-HP ratio instead
        // of blindly starting every combat at full health.
        stats.current_health = if base_current_health == 0 {
            0
        } else {
            let scaled = base_current_health.saturating_mul(stats.max_health) / base_max_health;
            scaled.max(1).min(stats.max_health)
        };

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::{
        abnormality_data::AbnormalityMetadata,
        artifact_data::ArtifactMetadata,
        equipment_data::{
            EquipmentEnhancementRecipeMetadata, EquipmentMaterialCost, EquipmentMaterialMetadata,
            EquipmentMaterialType, EquipmentMetadata, EquipmentType,
        },
        GameDataBuilder,
    };
    use crate::game::enums::RiskLevel;
    use crate::game::stats::{
        Effect, StatId, StatModifier, StatModifierKind, TriggerEffectTarget, TriggerType,
        TriggeredEffect,
    };
    use std::collections::HashMap;

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

        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .with_artifacts(vec![artifact])
            .with_equipment(vec![item])
            .build();

        let mut growth = GrowthStack::new();
        growth.add(GrowthId::KillStack, 2);

        let unit = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(10),
            source: BattleUnitSource::Abnormality {
                base_uuid: abno_uuid,
            },
            level: Tier::I,
            growth_stacks: growth,
            equipped_items: vec![item_uuid],
            equipped_item_enhancements: vec![],
        };

        let stats = unit.effective_stats(&game_data, &[artifact_uuid]).unwrap();
        assert_eq!(stats.max_health, 100);
        assert_eq!(stats.current_health, 100);
        assert_eq!(stats.attack, 10 + 2 + 7);
        assert_eq!(stats.defense, 5 + 3);
        assert!(stats.attack_interval_ms > 0);
    }

    #[test]
    fn effective_stats_applies_owned_equipment_enhancement_modifiers() {
        let abno_uuid = Uuid::from_u128(1);
        let item_uuid = Uuid::from_u128(2);

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
        let item = EquipmentMetadata {
            id: "enhanced_item".to_string(),
            uuid: item_uuid,
            name: "Enhanced Item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
        };
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .with_equipment_data(std::sync::Arc::new(
                crate::game::data::equipment_data::EquipmentDatabase::with_all(
                    vec![item],
                    vec![EquipmentMaterialMetadata {
                        id: "test_material".to_string(),
                        uuid: Uuid::from_u128(3),
                        name: "Test Material".to_string(),
                        description: String::new(),
                        material_type: EquipmentMaterialType::Fragment,
                        rarity: RiskLevel::ZAYIN,
                        equipment_type: Some(EquipmentType::Weapon),
                    }],
                    vec![],
                    vec![],
                    vec![],
                    vec![EquipmentEnhancementRecipeMetadata {
                        equipment_id: "enhanced_item".to_string(),
                        max_level: 3,
                        costs_per_level: vec![EquipmentMaterialCost {
                            material_id: "test_material".to_string(),
                            amount: 1,
                        }],
                        modifiers_per_level: vec![StatModifier {
                            stat: StatId::Attack,
                            kind: StatModifierKind::Flat,
                            value: 2,
                        }],
                    }],
                ),
            ))
            .build();

        let unit = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(10),
            source: BattleUnitSource::Abnormality {
                base_uuid: abno_uuid,
            },
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![item_uuid],
            equipped_item_enhancements: vec![BattleEquipmentEnhancement {
                base_uuid: item_uuid,
                enhancement_level: 2,
            }],
        };

        let stats = unit.effective_stats(&game_data, &[]).unwrap();
        assert_eq!(stats.attack, 14);
    }

    #[test]
    fn effective_stats_errors_when_abnormality_missing() {
        let game_data = GameDataBuilder::empty().build();

        let unit = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(10),
            source: BattleUnitSource::Abnormality {
                base_uuid: Uuid::from_u128(1),
            },
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
            equipped_item_enhancements: vec![],
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

        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .build();

        let unit = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(10),
            source: BattleUnitSource::Abnormality {
                base_uuid: abno_uuid,
            },
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
            equipped_item_enhancements: vec![],
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

        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .with_artifacts(vec![artifact])
            .build();

        let unit = BattleUnitDraft {
            owned_uuid: Uuid::from_u128(10),
            source: BattleUnitSource::Abnormality {
                base_uuid: abno_uuid,
            },
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
            equipped_item_enhancements: vec![],
        };

        assert!(matches!(
            unit.effective_stats(&game_data, &[artifact_uuid]),
            Err(GameError::InvalidStaticData(_))
        ));
    }
}
