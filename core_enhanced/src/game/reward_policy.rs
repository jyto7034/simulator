use crate::game::{
    combat_mission_policy::CombatMissionPolicy,
    combat_preview::{CombatMissionVariant, CombatNodeType},
    data::reward_data::RewardGrantKind,
    reward::RewardOption,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatRewardPolicy {
    node_type: CombatNodeType,
    mission_variant: CombatMissionVariant,
    featured_kinds: &'static [RewardGrantKind],
    allowed_kinds: &'static [RewardGrantKind],
}

impl CombatRewardPolicy {
    pub fn for_mission(node_type: CombatNodeType, mission_variant: CombatMissionVariant) -> Self {
        Self::new(
            node_type,
            mission_variant,
            CombatMissionPolicy::featured_reward_kinds_for_mission(node_type, mission_variant),
        )
    }

    const fn new(
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        featured_kinds: &'static [RewardGrantKind],
    ) -> Self {
        Self {
            node_type,
            mission_variant,
            featured_kinds,
            allowed_kinds: CombatMissionPolicy::ALLOWED_COMBAT_REWARD_KINDS,
        }
    }

    pub fn node_type(&self) -> CombatNodeType {
        self.node_type
    }

    pub fn mission_variant(&self) -> CombatMissionVariant {
        self.mission_variant
    }

    pub fn featured_kinds(&self) -> &'static [RewardGrantKind] {
        self.featured_kinds
    }

    pub fn allowed_kinds(&self) -> &'static [RewardGrantKind] {
        self.allowed_kinds
    }

    pub fn validate_rewards(&self, rewards: &[RewardOption]) -> Result<(), String> {
        for reward in rewards {
            let grant_kinds = reward.grant_kinds();
            if grant_kinds.is_empty() {
                return Err(format!(
                    "reward '{}' has no semantic grant kinds",
                    reward.id
                ));
            }
            if let Some(forbidden_kind) = grant_kinds
                .iter()
                .find(|kind| !self.allowed_kinds.contains(kind))
            {
                return Err(format!(
                    "reward '{}' grant kind {:?} is not allowed for {:?}",
                    reward.id, forbidden_kind, self.node_type
                ));
            }
        }

        if rewards.is_empty() {
            return Ok(());
        }

        let has_featured_reward = rewards.iter().any(|reward| {
            reward
                .grant_kinds()
                .iter()
                .any(|kind| self.featured_kinds.contains(kind))
        });
        if !has_featured_reward {
            return Err(format!(
                "{:?} reward pool must include at least one featured grant kind {:?}",
                self.mission_variant, self.featured_kinds
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

    fn reward(id: &str, effects: Vec<RewardEffect>) -> RewardOption {
        RewardOption {
            id: id.to_string(),
            uuid: Uuid::from_u128(1),
            name: id.to_string(),
            description: String::new(),
            icon: String::new(),
            effects,
        }
    }

    #[test]
    fn boss_policy_accepts_skill_fragment_focused_reward_pool() {
        let policy =
            CombatRewardPolicy::for_mission(CombatNodeType::Boss, CombatMissionVariant::Boss);

        policy
            .validate_rewards(&[
                reward(
                    "currency",
                    vec![RewardEffect::GrantEnkephalin { amount: 1 }],
                ),
                reward(
                    "fragment",
                    vec![RewardEffect::GrantSkillFragment {
                        fragment_id: crate::game::data::skill_fragment_data::SkillFragmentId::from(
                            "fragment_test",
                        ),
                    }],
                ),
            ])
            .expect("boss reward pool should accept skill fragment candidates");
    }

    #[test]
    fn policy_rejects_disallowed_grant_kinds() {
        let policy =
            CombatRewardPolicy::for_mission(CombatNodeType::Defense, CombatMissionVariant::Defense);
        let err = policy
            .validate_rewards(&[reward(
                "consumable",
                vec![RewardEffect::GrantConsumable {
                    consumable_id: "test".to_string(),
                }],
            )])
            .expect_err("disallowed grant kinds must not be valid combat rewards");

        assert!(err.contains("not allowed"));
    }

    #[test]
    fn policy_requires_at_least_one_featured_grant_kind_when_rewards_exist() {
        let policy =
            CombatRewardPolicy::for_mission(CombatNodeType::Defense, CombatMissionVariant::Defense);
        let err = policy
            .validate_rewards(&[reward(
                "currency",
                vec![RewardEffect::GrantEnkephalin { amount: 1 }],
            )])
            .expect_err("currency-only pools should not define combat reward identity");

        assert!(err.contains("featured grant kind"));
    }

    #[test]
    fn defense_policy_requires_defense_identity_reward() {
        let policy =
            CombatRewardPolicy::for_mission(CombatNodeType::Defense, CombatMissionVariant::Defense);

        assert!(policy.validate_rewards(&[]).is_ok());
        let err = policy
            .validate_rewards(&[reward(
                "currency",
                vec![RewardEffect::GrantEnkephalin { amount: 1 }],
            )])
            .expect_err("defense reward pool should contain a defense featured grant kind");

        assert!(err.contains("featured grant kind"));
    }
}
