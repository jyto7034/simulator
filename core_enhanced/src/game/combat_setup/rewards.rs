//! Resolves and validates combat encounter rewards for the concrete mission being started.
//!
//! Reward metadata remains in data; this module applies combat-specific reward
//! policy at battle/reward resolution boundaries.

use crate::game::{
    behavior::GameError,
    combat_preview::{CombatMissionVariant, CombatNodeType},
    data::{pve_data::PveEncounter, GameDataBase},
    enums::RewardMode,
    reward::RewardOption,
    reward_policy::CombatRewardPolicy,
};

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
