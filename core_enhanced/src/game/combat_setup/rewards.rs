//! Resolves and validates combat encounter rewards for the concrete mission being started.
//!
//! Reward metadata remains in data; this module applies combat-specific reward
//! policy at battle/reward resolution boundaries.

use crate::game::{
    behavior::GameError,
    combat_preview::{CombatMissionVariant, CombatNodeType},
    data::{pve_data::PveEncounter, GameDataBase},
    enums::RewardMode,
    reward::RewardEffect,
    reward::RewardOption,
    reward_policy::CombatRewardPolicy,
};

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
                | RewardEffect::GrantFragmentDust { amount } => {
                    *amount = scaled_reward_amount(*amount, multiplier_percent)?;
                }
                RewardEffect::GrantExperience { amount, .. }
                | RewardEffect::GrantEquipmentMaterial { amount, .. }
                | RewardEffect::GrantSkillFragmentResearch { amount, .. } => {
                    *amount = scaled_reward_amount(*amount, multiplier_percent)?;
                }
                RewardEffect::GrantConsumable { quantity, .. } => {
                    *quantity = scaled_reward_amount(*quantity, multiplier_percent)?;
                }
                RewardEffect::GrantEquipment { .. }
                | RewardEffect::GrantEquipmentFromPool { .. }
                | RewardEffect::GrantSkillFragment { .. } => {}
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
    let mut rewards = Vec::with_capacity(encounter.reward_uuids.len());
    for reward_uuid in &encounter.reward_uuids {
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

    Ok((encounter.reward_mode, rewards))
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
}
