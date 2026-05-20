use crate::game::{
    behavior::GameError,
    combat_preview::CombatNodeType,
    data::{pve_data::PveEncounter, reward_data::RewardTag, GameDataBase},
    enums::RewardMode,
    reward::RewardOption,
    reward_policy::CombatRewardPolicy,
};

pub(crate) fn resolve_combat_rewards_from_encounter(
    game_data: &GameDataBase,
    encounter: &PveEncounter,
    node_type: Option<CombatNodeType>,
) -> Result<(RewardMode, Vec<RewardOption>), GameError> {
    let mut rewards = Vec::with_capacity(encounter.reward_uuids.len());
    for reward_uuid in &encounter.reward_uuids {
        let reward = game_data
            .reward_data
            .get_by_uuid(reward_uuid)
            .ok_or(GameError::EventNotFound)?;
        let reward = RewardOption::from_metadata(reward);
        if reward.tags.contains(&RewardTag::Forbidden) {
            return Err(GameError::InvalidStaticData(format!(
                "combat encounter '{}' references forbidden reward '{}'",
                encounter.id, reward.id
            )));
        }
        rewards.push(reward);
    }

    if let Some(node_type) = node_type {
        CombatRewardPolicy::for_node_type(node_type)
            .validate_rewards(&rewards)
            .map_err(GameError::InvalidStaticData)?;
    }

    Ok((encounter.reward_mode, rewards))
}
