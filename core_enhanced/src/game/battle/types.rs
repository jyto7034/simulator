use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    game::resources::Position,
    game::{
        ability::{SkillActivationMode, SkillId},
        battle::core::movement::types::WorldVec2,
        battle::damage::DamageModifiers,
        battle::ids::UnitInstanceId,
        battle::tile_range::TileRangePattern,
        battle::timeline::Timeline,
        behavior::GameError,
        data::{
            abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
            equipment_data::{EquipmentType, WeaponCombatProfile},
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
    pub mobility_kind: MobilityKind,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentAffinity {
    #[default]
    GroundOnly,
    PlatformOnly,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MobilityKind {
    #[default]
    Ground,
    Airborne,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitTargetTrait {
    Airborne,
}

impl MobilityKind {
    pub fn is_airborne(self) -> bool {
        matches!(self, Self::Airborne)
    }

    pub fn can_be_blocked(self) -> bool {
        matches!(self, Self::Ground)
    }
}

impl DeploymentAffinity {
    pub fn allows_ground(self) -> bool {
        matches!(
            self,
            DeploymentAffinity::GroundOnly | DeploymentAffinity::Any
        )
    }

    pub fn allows_platform(self) -> bool {
        matches!(
            self,
            DeploymentAffinity::PlatformOnly | DeploymentAffinity::Any
        )
    }
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
    pub weapon_profile: Option<WeaponCombatProfile>,
    pub movement: MovementDef,
    pub resonance: ResonanceDef,
    pub skill_id: Option<SkillId>,
    pub skill_activation_mode: SkillActivationMode,
    pub deployment_affinity: DeploymentAffinity,
    pub block_capacity: u32,
    pub block_radius_units: f32,
    pub blockable: bool,
    pub mobility_kind: MobilityKind,
    pub target_traits: Vec<UnitTargetTrait>,
    pub incoming_damage_modifiers: DamageModifiers,
}

impl UnitCombatProfile {
    pub fn employee_default() -> Self {
        let basic_attack = BasicAttackDef {
            range_units: 1.0,
            defense_tile_range: Some(TileRangePattern {
                include_anchor_tile: false,
                rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
            }),
            interval_ms: 1500,
            windup_ms: 200,
            delivery: Default::default(),
            ..BasicAttackDef::default()
        };
        let movement = MovementDef::default();
        let mut stats = UnitStats::with_values(100, 100, 10, 0, basic_attack.interval_ms);
        stats.magic_resist = 0;
        stats.move_speed_units_per_ms = movement.speed_units_per_ms;

        Self {
            stats,
            basic_attack,
            weapon_profile: None,
            movement,
            resonance: ResonanceDef::default(),
            skill_id: None,
            skill_activation_mode: SkillActivationMode::Auto,
            deployment_affinity: DeploymentAffinity::GroundOnly,
            block_capacity: 1,
            block_radius_units: 0.75,
            blockable: true,
            mobility_kind: MobilityKind::Ground,
            target_traits: Vec::new(),
            incoming_damage_modifiers: DamageModifiers::default(),
        }
    }

    pub fn apply_weapon_profile(&mut self, weapon_profile: &WeaponCombatProfile) {
        weapon_profile
            .validate_runtime_contract("equipped weapon")
            .expect("validated equipment weapon profile");
        self.basic_attack.range_units = weapon_profile.range_units;
        self.basic_attack.defense_tile_range = Some(weapon_profile.defense_tile_range.clone());
        self.basic_attack.damage_type = weapon_profile.damage_type;
        self.basic_attack.targeting_profile = weapon_profile.targeting_profile;
        self.basic_attack.air_capable = weapon_profile.air_capable;
        self.basic_attack.range_role = weapon_profile.range_role;
        self.basic_attack.interval_ms = weapon_profile.interval_ms;
        self.basic_attack.windup_ms = weapon_profile.windup_ms;
        self.basic_attack.delivery = weapon_profile.delivery.clone();
        self.stats.attack_interval_ms = weapon_profile.interval_ms;
        self.weapon_profile = Some(weapon_profile.clone());
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
            weapon_profile: None,
            movement: meta.movement.clone(),
            resonance: meta.resonance.clone(),
            skill_id: meta.skill_id.clone(),
            skill_activation_mode: SkillActivationMode::Auto,
            deployment_affinity: DeploymentAffinity::GroundOnly,
            block_capacity: 0,
            block_radius_units: 0.0,
            blockable: meta.mobility_kind.can_be_blocked(),
            mobility_kind: meta.mobility_kind,
            target_traits: effective_target_traits(meta.mobility_kind, &meta.target_traits),
            incoming_damage_modifiers: DamageModifiers::default(),
        })
    }

    pub fn combat_profile(&self, game_data: &GameDataBase) -> Result<UnitCombatProfile, GameError> {
        let mut profile = match &self.source {
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
        }?;

        if matches!(&self.source, BattleUnitSource::Employee(_)) {
            let mut weapon_profile: Option<&WeaponCombatProfile> = None;
            for item_uuid in &self.equipped_items {
                let item = game_data
                    .equipment_data
                    .get_by_uuid(item_uuid)
                    .ok_or(GameError::MissingResource(""))?;
                if item.equipment_type != EquipmentType::Weapon {
                    continue;
                }
                let next_profile = item.weapon_profile.as_ref().ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "weapon equipment '{}' is missing weapon_profile",
                        item.id
                    ))
                })?;
                if weapon_profile.is_some() {
                    return Err(GameError::InvalidStaticData(
                        "battle unit draft contains multiple equipped weapons".to_string(),
                    ));
                }
                weapon_profile = Some(next_profile);
            }
            if let Some(weapon_profile) = weapon_profile {
                profile.apply_weapon_profile(weapon_profile);
            }
        }

        Ok(profile)
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

fn effective_target_traits(
    mobility_kind: MobilityKind,
    authored_traits: &[UnitTargetTrait],
) -> Vec<UnitTargetTrait> {
    let mut traits = authored_traits.to_vec();
    if mobility_kind.is_airborne() && !traits.contains(&UnitTargetTrait::Airborne) {
        traits.push(UnitTargetTrait::Airborne);
    }
    traits
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
    fn employee_default_profile_has_ground_blocking_contract() {
        let profile = UnitCombatProfile::employee_default();

        assert_eq!(profile.deployment_affinity, DeploymentAffinity::GroundOnly);
        assert_eq!(profile.block_capacity, 1);
        assert!((profile.block_radius_units - 0.75).abs() <= f32::EPSILON);
        assert!(profile.blockable);
    }

    #[test]
    fn abnormality_profile_defaults_to_blockable_enemy_without_deployment_capacity() {
        let abno_uuid = Uuid::from_u128(0xAB);
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
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .build();

        let profile =
            BattleUnitDraft::combat_profile_from_abnormality(abno_uuid, &game_data).unwrap();

        assert_eq!(profile.deployment_affinity, DeploymentAffinity::GroundOnly);
        assert_eq!(profile.block_capacity, 0);
        assert_eq!(profile.block_radius_units, 0.0);
        assert_eq!(profile.mobility_kind, MobilityKind::Ground);
        assert!(profile.blockable);
    }

    #[test]
    fn abnormality_airborne_profile_derives_unblockable_display_trait() {
        let abno_uuid = Uuid::from_u128(0xAC);
        let abno = AbnormalityMetadata {
            id: "drone".to_string(),
            uuid: abno_uuid,
            name: "Drone".to_string(),
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
            mobility_kind: MobilityKind::Airborne,
            target_traits: Vec::new(),
        };
        let game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno.clone()])
            .build();

        let profile =
            BattleUnitDraft::combat_profile_from_abnormality(abno_uuid, &game_data).unwrap();

        assert_eq!(profile.mobility_kind, MobilityKind::Airborne);
        assert!(!profile.blockable);
        assert!(profile.target_traits.contains(&UnitTargetTrait::Airborne));
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
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
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
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: item_triggers,
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
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
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };
        let item = EquipmentMetadata {
            id: "enhanced_item".to_string(),
            uuid: item_uuid,
            name: "Enhanced Item".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
            weapon_profile: Some(Default::default()),
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
    #[should_panic(expected = "basic_attack interval_ms must be greater than zero")]
    fn game_data_validation_rejects_attack_interval_zero() {
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
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
        };
        abno.basic_attack.interval_ms = 0;

        let _game_data = GameDataBuilder::empty()
            .with_abnormalities(vec![abno])
            .build();
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
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
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
