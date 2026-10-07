//! Resolves and validates combat encounter rewards for the concrete mission being started.
//!
//! Reward metadata remains in data; this module applies combat-specific reward
//! policy at battle/reward resolution boundaries.

use crate::game::{
    battle::types::{BattleOutcomeId, BattleVerdict},
    behavior::GameError,
    combat_preview::{CombatMissionVariant, CombatNodeType},
    data::{pve_data::PveEncounter, GameDataBase},
    enums::RewardMode,
    reward::RewardEffect,
    reward::RewardOption,
    reward_policy::CombatRewardPolicy,
};

#[derive(Debug, Clone)]
pub struct ResolvedCombatOutcomePolicy {
    pub outcome_id: BattleOutcomeId,
    pub verdict: BattleVerdict,
    pub reward_mode: RewardMode,
    pub rewards: Vec<RewardOption>,
    pub grant_node_bonus: bool,
    pub advance_encounter_progression: bool,
}

pub(crate) fn resolve_combat_outcome_policies(
    game_data: &GameDataBase,
    encounter: &PveEncounter,
    authored_policy: Option<&crate::game::map::AuthoredCombatNodePolicy>,
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    reward_multiplier_percent: u32,
) -> Result<Vec<ResolvedCombatOutcomePolicy>, GameError> {
    let mut resolved = Vec::with_capacity(encounter.terminal_outcomes.len());
    for outcome in &encounter.terminal_outcomes {
        let base_policy = encounter
            .outcome_policies
            .iter()
            .find(|policy| policy.outcome_id == outcome.id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "encounter '{}' has no policy for outcome '{}'",
                    encounter.id, outcome.id
                ))
            })?;
        let authored_outcome = authored_policy.and_then(|policy| {
            policy
                .outcome_policies
                .iter()
                .find(|candidate| candidate.outcome_id == outcome.id)
        });
        if authored_policy.is_some() && authored_outcome.is_none() {
            return Err(GameError::InvalidStaticData(format!(
                "node policy does not cover encounter '{}' outcome '{}'",
                encounter.id, outcome.id
            )));
        }

        let mut rewards = match authored_outcome {
            Some(policy) => policy
                .reward_ids
                .iter()
                .map(|reward_id| {
                    game_data
                        .reward_data
                        .get_by_id(reward_id)
                        .map(RewardOption::from_metadata)
                        .ok_or(GameError::EventNotFound)
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => base_policy
                .reward_uuids
                .iter()
                .map(|reward_uuid| {
                    game_data
                        .reward_data
                        .get_by_uuid(reward_uuid)
                        .map(RewardOption::from_metadata)
                        .ok_or(GameError::EventNotFound)
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        let (grant_node_bonus, advance_encounter_progression) = authored_outcome.map_or(
            (
                base_policy.grant_node_bonus,
                base_policy.advance_encounter_progression,
            ),
            |policy| {
                (
                    policy.grant_node_bonus,
                    policy.advance_encounter_progression,
                )
            },
        );
        CombatRewardPolicy::for_mission(node_type, mission_variant)
            .validate_rewards(&rewards)
            .map_err(GameError::InvalidStaticData)?;
        if grant_node_bonus {
            apply_combat_reward_multiplier(&mut rewards, reward_multiplier_percent)?;
        }
        if outcome.verdict == BattleVerdict::Defeat
            && (!rewards.is_empty() || grant_node_bonus || advance_encounter_progression)
        {
            return Err(GameError::InvalidStaticData(format!(
                "Defeat outcome '{}' cannot grant rewards, node bonus, or progression",
                outcome.id
            )));
        }
        resolved.push(ResolvedCombatOutcomePolicy {
            outcome_id: BattleOutcomeId::new(outcome.id.clone()),
            verdict: outcome.verdict,
            reward_mode: base_policy.reward_mode,
            rewards,
            grant_node_bonus,
            advance_encounter_progression,
        });
    }
    if authored_policy.is_some_and(|policy| policy.outcome_policies.len() != resolved.len()) {
        return Err(GameError::InvalidStaticData(format!(
            "node policy outcome count does not match encounter '{}'",
            encounter.id
        )));
    }
    Ok(resolved)
}

/// Applies the fixed reward budget authored on a manifested combat node.
///
/// Quantity-bearing effects participate in the budget. Singular identity grants
/// (equipment, equipment-pool rolls, and skill fragments) remain one grant so a
/// percentage modifier cannot silently change their identity/duplication semantics.
pub(crate) fn apply_combat_reward_multiplier(
    rewards: &mut [RewardOption],
    multiplier_percent: u32,
) -> Result<(), GameError> {
    if multiplier_percent == 0 {
        return Err(GameError::InvalidStaticData(
            "combat reward multiplier must be greater than zero".to_string(),
        ));
    }

    for reward in rewards {
        for effect in &mut reward.effects {
            match effect {
                RewardEffect::GrantEnkephalin { amount }
                | RewardEffect::GrantFragmentDust { amount }
                | RewardEffect::GrantRelicCorruptionCatalyst { amount }
                | RewardEffect::GrantEquipmentDust { amount } => {
                    *amount = scaled_reward_amount(*amount, multiplier_percent)?;
                }
                RewardEffect::GrantExperience { amount, .. }
                | RewardEffect::GrantEquipmentMaterial { amount, .. } => {
                    *amount = scaled_reward_amount(*amount, multiplier_percent)?;
                }
                RewardEffect::GrantConsumable { quantity, .. } => {
                    *quantity = scaled_reward_amount(*quantity, multiplier_percent)?;
                }
                RewardEffect::GrantEquipment { .. }
                | RewardEffect::GrantEquipmentFromPool { .. }
                | RewardEffect::GrantSkillFragment { .. }
                | RewardEffect::GrantRelic { .. } => {}
            }
        }
    }

    Ok(())
}

fn scaled_reward_amount(amount: u32, multiplier_percent: u32) -> Result<u32, GameError> {
    if amount == 0 {
        return Ok(0);
    }
    let scaled = (u128::from(amount) * u128::from(multiplier_percent) + 50) / 100;
    let scaled = scaled.max(1);
    u32::try_from(scaled).map_err(|_| {
        GameError::InvalidStaticData(format!(
            "combat reward amount {amount} overflows after applying {multiplier_percent}%"
        ))
    })
}

pub(crate) fn resolve_combat_rewards_from_encounter(
    game_data: &GameDataBase,
    encounter: &PveEncounter,
    node_type: Option<CombatNodeType>,
    mission_variant: Option<CombatMissionVariant>,
) -> Result<(RewardMode, Vec<RewardOption>), GameError> {
    let policy = encounter
        .terminal_outcomes
        .iter()
        .filter(|outcome| outcome.verdict == crate::game::battle::types::BattleVerdict::Victory)
        .min_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.id.cmp(&right.id))
        })
        .and_then(|outcome| {
            encounter
                .outcome_policies
                .iter()
                .find(|policy| policy.outcome_id == outcome.id)
        })
        .ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "combat encounter '{}' has no Victory outcome policy",
                encounter.id
            ))
        })?;
    let mut rewards = Vec::with_capacity(policy.reward_uuids.len());
    for reward_uuid in &policy.reward_uuids {
        let reward = game_data
            .reward_data
            .get_by_uuid(reward_uuid)
            .ok_or(GameError::EventNotFound)?;
        let reward = RewardOption::from_metadata(reward);
        rewards.push(reward);
    }

    if let Some(node_type) = node_type {
        let mission_variant = mission_variant
            .unwrap_or_else(|| CombatMissionVariant::default_for_node_type(node_type));
        CombatRewardPolicy::for_mission(node_type, mission_variant)
            .validate_rewards(&rewards)
            .map_err(GameError::InvalidStaticData)?;
    }

    Ok((policy.reward_mode, rewards))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::reward::ExperienceTargetPolicy;
    use uuid::Uuid;

    fn reward(effects: Vec<RewardEffect>) -> RewardOption {
        RewardOption {
            id: "test_reward".to_string(),
            uuid: Uuid::nil(),
            name: "Test".to_string(),
            description: String::new(),
            icon: String::new(),
            effects,
        }
    }

    #[test]
    fn reward_multiplier_scales_quantity_effects_with_fixed_rounding() {
        let mut rewards = vec![reward(vec![
            RewardEffect::GrantEnkephalin { amount: 100 },
            RewardEffect::GrantExperience {
                amount: 3,
                target: ExperienceTargetPolicy::AliveRoster,
            },
            RewardEffect::GrantEquipmentMaterial {
                material_id: "dust".to_string(),
                amount: 3,
            },
            RewardEffect::GrantConsumable {
                consumable_id: "tonic".to_string(),
                quantity: 1,
            },
            RewardEffect::GrantFragmentDust { amount: 2 },
        ])];

        apply_combat_reward_multiplier(&mut rewards, 125).unwrap();

        assert_eq!(
            rewards[0].effects,
            vec![
                RewardEffect::GrantEnkephalin { amount: 125 },
                RewardEffect::GrantExperience {
                    amount: 4,
                    target: ExperienceTargetPolicy::AliveRoster,
                },
                RewardEffect::GrantEquipmentMaterial {
                    material_id: "dust".to_string(),
                    amount: 4,
                },
                RewardEffect::GrantConsumable {
                    consumable_id: "tonic".to_string(),
                    quantity: 1,
                },
                RewardEffect::GrantFragmentDust { amount: 3 },
            ]
        );
    }

    #[test]
    fn reward_multiplier_preserves_singular_identity_grants() {
        let effects = vec![
            RewardEffect::GrantEquipment {
                equipment_id: "weapon".to_string(),
            },
            RewardEffect::GrantEquipmentFromPool {
                pool_id: "pool".to_string(),
            },
            RewardEffect::GrantSkillFragment {
                fragment_id: "fragment".into(),
            },
        ];
        let mut rewards = vec![reward(effects.clone())];

        apply_combat_reward_multiplier(&mut rewards, 250).unwrap();

        assert_eq!(rewards[0].effects, effects);
    }

    #[test]
    fn monolith_composite_reward_scales_small_quantity_once() {
        let mut rewards = vec![reward(vec![RewardEffect::GrantConsumable {
            consumable_id: "tonic".to_string(),
            quantity: 1,
        }])];

        apply_combat_reward_multiplier(&mut rewards, 313).unwrap();

        assert_eq!(
            rewards[0].effects,
            vec![RewardEffect::GrantConsumable {
                consumable_id: "tonic".to_string(),
                quantity: 3,
            }]
        );
    }

    #[test]
    fn standard_tutorial_pools_and_authored_combat_rewards_are_exact() {
        let game_data = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("live data must load");
        for (pool_id, reward_id) in [
            ("tutorial_f1_training_budget", "training_budget_reward"),
            ("tutorial_f3_salvage", "field_equipment_salvage_reward"),
            ("tutorial_f3_budget", "large_enkephalin_reward"),
        ] {
            assert_eq!(
                game_data
                    .reward_data
                    .pool_by_id(pool_id)
                    .expect("tutorial pool")
                    .reward_ids,
                [reward_id.to_string()]
            );
        }

        let encounter = game_data
            .pve_data
            .get_by_id("suppress_nothing_there")
            .unwrap();
        let authored_policy = crate::game::map::AuthoredCombatNodePolicy {
            outcome_policies: vec![
                crate::game::map::AuthoredCombatOutcomePolicy {
                    outcome_id: "victory".to_string(),
                    reward_ids: vec![
                        "enkephalin_reward".to_string(),
                        "field_equipment_salvage_reward".to_string(),
                    ],
                    grant_node_bonus: true,
                    advance_encounter_progression: true,
                },
                crate::game::map::AuthoredCombatOutcomePolicy {
                    outcome_id: "defeat_all_players".to_string(),
                    reward_ids: Vec::new(),
                    grant_node_bonus: false,
                    advance_encounter_progression: false,
                },
            ],
            actor_stat_overrides: Vec::new(),
        };
        let policies = resolve_combat_outcome_policies(
            &game_data,
            encounter,
            Some(&authored_policy),
            CombatNodeType::Boss,
            CombatMissionVariant::Boss,
            100,
        )
        .unwrap();
        let victory = policies
            .iter()
            .find(|policy| policy.verdict == BattleVerdict::Victory)
            .expect("resolved Victory policy");
        assert_eq!(
            victory.reward_mode,
            crate::game::enums::RewardMode::ClaimAll
        );
        assert_eq!(
            victory
                .rewards
                .iter()
                .map(|reward| reward.id.as_str())
                .collect::<Vec<_>>(),
            ["enkephalin_reward", "field_equipment_salvage_reward"]
        );
    }

    #[test]
    fn live_outcome_resolution_is_exhaustive_and_defeat_is_non_positive() {
        let game_data = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("live data must load");
        let encounter = game_data
            .pve_data
            .get_by_id("suppress_nothing_there")
            .expect("live encounter");
        let policies = resolve_combat_outcome_policies(
            &game_data,
            encounter,
            None,
            CombatNodeType::Boss,
            CombatMissionVariant::Boss,
            100,
        )
        .expect("live outcome policies must resolve");

        assert_eq!(policies.len(), encounter.terminal_outcomes.len());
        for outcome in &encounter.terminal_outcomes {
            let policy = policies
                .iter()
                .find(|policy| policy.outcome_id.as_str() == outcome.id)
                .expect("every terminal outcome must resolve exactly once");
            assert_eq!(policy.verdict, outcome.verdict);
            if outcome.verdict == BattleVerdict::Defeat {
                assert!(policy.rewards.is_empty());
                assert!(!policy.grant_node_bonus);
                assert!(!policy.advance_encounter_progression);
            }
        }
    }
}

crate::authoring::clone_shape::struct_shape!(ResolvedCombatOutcomePolicy {
    outcome_id,
    verdict,
    reward_mode,
    rewards,
    grant_node_bonus,
    advance_encounter_progression
});
