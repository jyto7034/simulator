use crate::game::{
    combat_mission_policy::CombatMissionPolicy, combat_preview::CombatNodeType,
    data::reward_data::RewardTag, reward::RewardOption,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatRewardPolicy {
    node_type: CombatNodeType,
    featured_tags: &'static [RewardTag],
    allowed_tags: &'static [RewardTag],
}

impl CombatRewardPolicy {
    pub fn for_node_type(node_type: CombatNodeType) -> Self {
        Self::new(
            node_type,
            CombatMissionPolicy::featured_reward_tags_for_node_type(node_type),
        )
    }

    const fn new(node_type: CombatNodeType, featured_tags: &'static [RewardTag]) -> Self {
        Self {
            node_type,
            featured_tags,
            allowed_tags: CombatMissionPolicy::ALLOWED_COMBAT_REWARD_TAGS,
        }
    }

    pub fn node_type(&self) -> CombatNodeType {
        self.node_type
    }

    pub fn featured_tags(&self) -> &'static [RewardTag] {
        self.featured_tags
    }

    pub fn allowed_tags(&self) -> &'static [RewardTag] {
        self.allowed_tags
    }

    pub fn validate_rewards(&self, rewards: &[RewardOption]) -> Result<(), String> {
        for reward in rewards {
            if reward.tags.is_empty() {
                return Err(format!(
                    "reward '{}' has no semantic reward tags",
                    reward.id
                ));
            }
            if let Some(forbidden_tag) = reward
                .tags
                .iter()
                .find(|tag| !self.allowed_tags.contains(tag))
            {
                return Err(format!(
                    "reward '{}' tag {:?} is not allowed for {:?}",
                    reward.id, forbidden_tag, self.node_type
                ));
            }
        }

        if rewards.is_empty() {
            return Ok(());
        }

        let has_featured_reward = rewards.iter().any(|reward| {
            reward
                .tags
                .iter()
                .any(|tag| self.featured_tags.contains(tag))
        });
        if !has_featured_reward {
            return Err(format!(
                "{:?} reward pool must include at least one featured tag {:?}",
                self.node_type, self.featured_tags
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::reward::RewardEffect;
    use uuid::Uuid;

    fn reward(id: &str, tags: Vec<RewardTag>) -> RewardOption {
        RewardOption {
            id: id.to_string(),
            uuid: Uuid::from_u128(1),
            name: id.to_string(),
            description: String::new(),
            icon: String::new(),
            tags,
            effects: vec![RewardEffect::GrantEnkephalin { amount: 1 }],
        }
    }

    #[test]
    fn suppression_policy_accepts_skill_fragment_focused_reward_pool() {
        let policy = CombatRewardPolicy::for_node_type(CombatNodeType::Suppression);

        policy
            .validate_rewards(&[
                reward("currency", vec![RewardTag::Currency]),
                reward("fragment", vec![RewardTag::SkillFragment]),
            ])
            .expect("suppression reward pool should accept skill fragment candidates");
    }

    #[test]
    fn policy_rejects_forbidden_legacy_reward_tags() {
        let policy = CombatRewardPolicy::for_node_type(CombatNodeType::Suppression);
        let err = policy
            .validate_rewards(&[reward("legacy", vec![RewardTag::Forbidden])])
            .expect_err("forbidden tags must not be valid combat rewards");

        assert!(err.contains("not allowed"));
    }

    #[test]
    fn policy_requires_at_least_one_featured_tag_when_rewards_exist() {
        let policy = CombatRewardPolicy::for_node_type(CombatNodeType::Encirclement);
        let err = policy
            .validate_rewards(&[reward("currency", vec![RewardTag::Currency])])
            .expect_err("currency-only pools should not define combat reward identity");

        assert!(err.contains("featured tag"));
    }

    #[test]
    fn split_operation_policy_rejects_reward_pools_until_mode_is_implemented() {
        let policy = CombatRewardPolicy::for_node_type(CombatNodeType::SplitOperation);

        assert!(policy.validate_rewards(&[]).is_ok());
        let err = policy
            .validate_rewards(&[reward("equipment", vec![RewardTag::Equipment])])
            .expect_err("deferred split operation should not define live rewards yet");

        assert!(err.contains("featured tag"));
    }
}
