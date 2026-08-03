use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    data::{build_string_index, build_uuid_index, once_lock_with},
    enums::RiskLevel,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsumableTier {
    Common,
    Uncommon,
    Rare,
    Critical,
    Forbidden,
}

impl ConsumableTier {
    pub fn allows_offense_boost(self) -> bool {
        matches!(self, Self::Rare | Self::Critical | Self::Forbidden)
    }

    pub fn max_initial_skill_charge_percent(self) -> u32 {
        match self {
            Self::Common => 20,
            Self::Uncommon => 40,
            Self::Rare => 70,
            Self::Critical | Self::Forbidden => 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsumableTargetPolicy {
    SingleEmployee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsumableDurationPolicy {
    NextCombatNode,
    CombatNodes(u32),
}

impl ConsumableDurationPolicy {
    pub fn initial_remaining_combat_nodes(self) -> u32 {
        match self {
            Self::NextCombatNode => 1,
            Self::CombatNodes(nodes) => nodes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ConsumableEffect {
    DeathPrevent,
    ReducePhysicalDamage { amount: u8 },
    TraumaMitigation { percent: u32 },
    BattleHpSetup { bonus_percent: u32 },
    DeployCostReduction { percent: u32 },
    DefenseMitigation { percent: u32 },
    InitialSkillCharge { percent: u32 },
    OffenseBoost { attack_bonus_percent: u32 },
}

impl ConsumableEffect {
    fn bounded_percent_fields(&self) -> Vec<(&'static str, &'static str, u32)> {
        match self {
            Self::DeathPrevent | Self::ReducePhysicalDamage { .. } => vec![],
            Self::TraumaMitigation { percent } => {
                vec![("TraumaMitigation", "percent", *percent)]
            }
            Self::BattleHpSetup { bonus_percent } => {
                vec![("BattleHpSetup", "bonus_percent", *bonus_percent)]
            }
            Self::DeployCostReduction { percent } => {
                vec![("DeployCostReduction", "percent", *percent)]
            }
            Self::DefenseMitigation { percent } => {
                vec![("DefenseMitigation", "percent", *percent)]
            }
            Self::InitialSkillCharge { percent } => {
                vec![("InitialSkillCharge", "percent", *percent)]
            }
            Self::OffenseBoost {
                attack_bonus_percent,
            } => vec![(
                "OffenseBoost",
                "attack_bonus_percent",
                *attack_bonus_percent,
            )],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumableMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub tier: ConsumableTier,
    pub rarity: RiskLevel,
    pub price: u32,
    pub target_policy: ConsumableTargetPolicy,
    pub duration_policy: ConsumableDurationPolicy,
    pub effect: ConsumableEffect,
    pub live_pool: bool,
}

pub type ConsumableItem = ConsumableMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumableDatabase {
    pub items: Vec<ConsumableMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl ConsumableDatabase {
    pub fn new(items: Vec<ConsumableMetadata>) -> Self {
        let by_id = once_lock_with(build_string_index(&items, "consumable id", |item| &item.id));
        let by_uuid = once_lock_with(build_uuid_index(&items, "consumable uuid", |item| {
            item.uuid
        }));
        Self {
            items,
            by_id,
            by_uuid,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.items, "consumable id", |item| &item.id))
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid
            .get_or_init(|| build_uuid_index(&self.items, "consumable uuid", |item| item.uuid))
    }

    pub fn get_by_id(&self, id: &str) -> Option<&ConsumableMetadata> {
        self.by_id()
            .get(id)
            .and_then(|index| self.items.get(*index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&ConsumableMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|index| self.items.get(*index))
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (index, item) in self.items.iter().enumerate() {
            if matches!(item.effect, ConsumableEffect::OffenseBoost { .. })
                && !item.tier.allows_offense_boost()
            {
                return Err(crate::game::data::DataValidationError::at(
                    &item.id,
                    format!("items[{index}].effect"),
                    format!("uses OffenseBoost below Rare tier ({:?})", item.tier),
                ));
            }
            if item.tier == ConsumableTier::Forbidden && item.live_pool {
                return Err(crate::game::data::DataValidationError::at(
                    &item.id,
                    format!("items[{index}].live_pool"),
                    "Forbidden consumable must not be in the live pool before side-effect policy is implemented",
                ));
            }
            if matches!(
                item.effect,
                ConsumableEffect::ReducePhysicalDamage { amount: 0 }
            ) {
                return Err(crate::game::data::DataValidationError::at(
                    &item.id,
                    format!("items[{index}].effect.ReducePhysicalDamage.amount"),
                    "physical damage treatment amount must be greater than zero",
                ));
            }
            for (effect_name, field_name, percent) in item.effect.bounded_percent_fields() {
                if percent > 100 {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        format!("items[{index}].effect.{effect_name}.{field_name}"),
                        format!("percentage {percent} exceeds 100"),
                    ));
                }
            }
            if let ConsumableEffect::InitialSkillCharge { percent } = item.effect {
                if percent > item.tier.max_initial_skill_charge_percent() {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        format!("items[{index}].effect.InitialSkillCharge.percent"),
                        format!(
                            "value {percent} exceeds {:?} tier cap {}",
                            item.tier,
                            item.tier.max_initial_skill_charge_percent()
                        ),
                    ));
                }
            }
            if matches!(
                item.duration_policy,
                ConsumableDurationPolicy::CombatNodes(0)
            ) {
                return Err(crate::game::data::DataValidationError::at(
                    &item.id,
                    format!("items[{index}].duration_policy"),
                    "CombatNodes count must be greater than zero",
                ));
            }
        }
        Ok(())
    }

    pub fn validate_indexes(&self) {
        self.validate_contract()
            .unwrap_or_else(|error| panic!("invalid consumable data: {error:?}"));
        let _ = self.by_id();
        let _ = self.by_uuid();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consumable(
        id: &str,
        tier: ConsumableTier,
        duration_policy: ConsumableDurationPolicy,
        effect: ConsumableEffect,
    ) -> ConsumableMetadata {
        ConsumableMetadata {
            id: id.to_string(),
            uuid: Uuid::new_v4(),
            name: id.to_string(),
            description: String::new(),
            tier,
            rarity: RiskLevel::ZAYIN,
            price: 1,
            target_policy: ConsumableTargetPolicy::SingleEmployee,
            duration_policy,
            effect,
            live_pool: false,
        }
    }

    #[test]
    fn validation_rejects_zero_combat_node_duration() {
        let database = ConsumableDatabase::new(vec![consumable(
            "zero_duration",
            ConsumableTier::Common,
            ConsumableDurationPolicy::CombatNodes(0),
            ConsumableEffect::TraumaMitigation { percent: 10 },
        )]);
        let error = database
            .validate_contract()
            .expect_err("CombatNodes(0) must reject");
        assert_eq!(error.object_id.as_deref(), Some("zero_duration"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("items[0].duration_policy")
        );
    }

    #[test]
    fn legacy_run_hp_loss_mitigation_effect_is_rejected() {
        let error = ron::de::from_str::<ConsumableEffect>("RunHpLossMitigation(percent: 25)")
            .expect_err("removed Run HP effect must not deserialize");

        assert!(error.to_string().contains("RunHpLossMitigation"));
    }

    #[test]
    fn validation_rejects_zero_physical_treatment_amount() {
        let database = ConsumableDatabase::new(vec![consumable(
            "zero_treatment",
            ConsumableTier::Common,
            ConsumableDurationPolicy::NextCombatNode,
            ConsumableEffect::ReducePhysicalDamage { amount: 0 },
        )]);

        let error = database
            .validate_contract()
            .expect_err("zero physical treatment must reject");
        assert_eq!(
            error.field_path.as_deref(),
            Some("items[0].effect.ReducePhysicalDamage.amount")
        );
    }

    #[test]
    fn validation_rejects_low_tier_offense_boost() {
        let database = ConsumableDatabase::new(vec![consumable(
            "cheap_attack_boost",
            ConsumableTier::Common,
            ConsumableDurationPolicy::NextCombatNode,
            ConsumableEffect::OffenseBoost {
                attack_bonus_percent: 10,
            },
        )]);
        let error = database
            .validate_contract()
            .expect_err("low-tier OffenseBoost must reject");
        assert_eq!(error.object_id.as_deref(), Some("cheap_attack_boost"));
        assert_eq!(error.field_path.as_deref(), Some("items[0].effect"));
    }

    #[test]
    fn validation_rejects_initial_skill_charge_above_tier_cap() {
        let database = ConsumableDatabase::new(vec![consumable(
            "cheap_full_charge",
            ConsumableTier::Common,
            ConsumableDurationPolicy::NextCombatNode,
            ConsumableEffect::InitialSkillCharge { percent: 100 },
        )]);
        let error = database
            .validate_contract()
            .expect_err("tier-cap violation must reject");
        assert_eq!(error.object_id.as_deref(), Some("cheap_full_charge"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("items[0].effect.InitialSkillCharge.percent")
        );
        assert!(error.cause.contains("Common tier cap 20"));
    }

    #[test]
    fn validation_rejects_percent_effects_above_full_scale() {
        let cases = [
            (
                "invalid_trauma",
                ConsumableEffect::TraumaMitigation { percent: 101 },
                "TraumaMitigation.percent above 100%",
            ),
            (
                "invalid_battle_hp",
                ConsumableEffect::BattleHpSetup { bonus_percent: 101 },
                "BattleHpSetup.bonus_percent above 100%",
            ),
            (
                "invalid_deploy_cost",
                ConsumableEffect::DeployCostReduction { percent: 101 },
                "DeployCostReduction.percent above 100%",
            ),
            (
                "invalid_defense",
                ConsumableEffect::DefenseMitigation { percent: 101 },
                "DefenseMitigation.percent above 100%",
            ),
            (
                "invalid_charge",
                ConsumableEffect::InitialSkillCharge { percent: 101 },
                "InitialSkillCharge.percent above 100%",
            ),
            (
                "invalid_offense",
                ConsumableEffect::OffenseBoost {
                    attack_bonus_percent: 101,
                },
                "OffenseBoost.attack_bonus_percent above 100%",
            ),
        ];

        for (id, effect, expected) in cases {
            let database = ConsumableDatabase::new(vec![consumable(
                id,
                ConsumableTier::Critical,
                ConsumableDurationPolicy::NextCombatNode,
                effect,
            )]);
            let error = database
                .validate_contract()
                .expect_err("percent above 100 must reject");
            assert_eq!(error.object_id.as_deref(), Some(id));
            assert!(
                error
                    .field_path
                    .as_deref()
                    .is_some_and(|path| path.contains(expected.split(" above").next().unwrap())),
                "unexpected field path: {:?}",
                error.field_path
            );
            assert!(error.cause.contains("exceeds 100"));
        }
    }
}
