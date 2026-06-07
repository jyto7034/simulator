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
pub enum ConsumableEffect {
    DeathPrevent,
    TraumaMitigation { percent: u32 },
    RunHpLossMitigation { percent: u32 },
    BattleHpSetup { bonus_percent: u32 },
    DeployCostReduction { percent: u32 },
    DefenseMitigation { percent: u32 },
    InitialSkillCharge { percent: u32 },
    OffenseBoost { attack_bonus_percent: u32 },
}

impl ConsumableEffect {
    fn bounded_percent_fields(&self) -> Vec<(&'static str, &'static str, u32)> {
        match self {
            Self::DeathPrevent => vec![],
            Self::TraumaMitigation { percent } => {
                vec![("TraumaMitigation", "percent", *percent)]
            }
            Self::RunHpLossMitigation { percent } => {
                vec![("RunHpLossMitigation", "percent", *percent)]
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
pub struct ConsumableMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub tier: ConsumableTier,
    #[serde(default = "default_consumable_rarity")]
    pub rarity: RiskLevel,
    pub price: u32,
    pub target_policy: ConsumableTargetPolicy,
    pub duration_policy: ConsumableDurationPolicy,
    pub effect: ConsumableEffect,
    #[serde(default)]
    pub live_pool: bool,
}

fn default_consumable_rarity() -> RiskLevel {
    RiskLevel::ZAYIN
}

pub type ConsumableItem = ConsumableMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_uuid();
        for item in &self.items {
            if matches!(item.effect, ConsumableEffect::OffenseBoost { .. })
                && !item.tier.allows_offense_boost()
            {
                panic!(
                    "consumable '{}' uses OffenseBoost below Rare tier ({:?})",
                    item.id, item.tier
                );
            }
            if item.tier == ConsumableTier::Forbidden && item.live_pool {
                panic!(
                    "forbidden consumable '{}' must not be included in live pool before side-effect policy is implemented",
                    item.id
                );
            }
            for (effect_name, field_name, percent) in item.effect.bounded_percent_fields() {
                if percent > 100 {
                    panic!(
                        "consumable '{}' uses {effect_name}.{field_name} above 100% ({percent})",
                        item.id
                    );
                }
            }
            if let ConsumableEffect::InitialSkillCharge { percent } = item.effect {
                if percent > item.tier.max_initial_skill_charge_percent() {
                    panic!(
                        "consumable '{}' uses InitialSkillCharge {}% above {:?} tier cap {}%",
                        item.id,
                        percent,
                        item.tier,
                        item.tier.max_initial_skill_charge_percent()
                    );
                }
            }
            if matches!(
                item.duration_policy,
                ConsumableDurationPolicy::CombatNodes(0)
            ) {
                panic!("consumable '{}' uses CombatNodes(0)", item.id);
            }
        }
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
    #[should_panic(expected = "uses CombatNodes(0)")]
    fn validation_rejects_zero_combat_node_duration() {
        ConsumableDatabase::new(vec![consumable(
            "zero_duration",
            ConsumableTier::Common,
            ConsumableDurationPolicy::CombatNodes(0),
            ConsumableEffect::TraumaMitigation { percent: 10 },
        )])
        .validate_indexes();
    }

    #[test]
    #[should_panic(expected = "uses OffenseBoost below Rare tier")]
    fn validation_rejects_low_tier_offense_boost() {
        ConsumableDatabase::new(vec![consumable(
            "cheap_attack_boost",
            ConsumableTier::Common,
            ConsumableDurationPolicy::NextCombatNode,
            ConsumableEffect::OffenseBoost {
                attack_bonus_percent: 10,
            },
        )])
        .validate_indexes();
    }

    #[test]
    #[should_panic(expected = "InitialSkillCharge 100% above Common tier cap 20%")]
    fn validation_rejects_initial_skill_charge_above_tier_cap() {
        ConsumableDatabase::new(vec![consumable(
            "cheap_full_charge",
            ConsumableTier::Common,
            ConsumableDurationPolicy::NextCombatNode,
            ConsumableEffect::InitialSkillCharge { percent: 100 },
        )])
        .validate_indexes();
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
                "invalid_run_hp",
                ConsumableEffect::RunHpLossMitigation { percent: 101 },
                "RunHpLossMitigation.percent above 100%",
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
            let result = std::panic::catch_unwind(|| {
                ConsumableDatabase::new(vec![consumable(
                    id,
                    ConsumableTier::Critical,
                    ConsumableDurationPolicy::NextCombatNode,
                    effect,
                )])
                .validate_indexes();
            });
            let error = result.expect_err("expected percent validation panic");
            let message = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied())
                .unwrap_or("<non-string panic>");
            assert!(
                message.contains(expected),
                "panic '{message}' did not contain '{expected}'"
            );
        }
    }
}
