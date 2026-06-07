use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    data::{build_string_index, build_uuid_index, once_lock_with},
    reward::RewardEffect,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RewardTag {
    Currency,
    Experience,
    Equipment,
    Artifact,
    Consumable,
    SkillFragment,
    ResearchProgress,
    Forbidden,
    Narrative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(default)]
    pub tags: Vec<RewardTag>,
    pub effects: Vec<RewardEffect>,
}

impl RewardMetadata {
    pub fn resolved_tags(&self) -> Vec<RewardTag> {
        if !self.tags.is_empty() {
            return normalized_tags(self.tags.clone());
        }

        RewardTag::from_effects(&self.effects)
    }
}

impl RewardTag {
    pub fn from_effects(effects: &[RewardEffect]) -> Vec<Self> {
        let mut tags = Vec::new();
        for effect in effects {
            match effect {
                RewardEffect::GrantEnkephalin { .. } => tags.push(Self::Currency),
                RewardEffect::GrantExperience { .. } => tags.push(Self::Experience),
                RewardEffect::GrantEquipment { .. }
                | RewardEffect::GrantEquipmentMaterial { .. } => tags.push(Self::Equipment),
                RewardEffect::GrantArtifact { .. } => tags.push(Self::Artifact),
                RewardEffect::GrantConsumable { .. } => tags.push(Self::Consumable),
                RewardEffect::GrantSkillFragment { .. } => tags.push(Self::SkillFragment),
                RewardEffect::GrantSkillFragmentResearch { .. } => {
                    tags.push(Self::ResearchProgress)
                }
                RewardEffect::ForbiddenAbnormalityGrant => tags.push(Self::Forbidden),
            }
        }
        normalized_tags(tags)
    }
}

fn normalized_tags(mut tags: Vec<RewardTag>) -> Vec<RewardTag> {
    tags.sort_by_key(|tag| *tag as u8);
    tags.dedup();
    tags
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDatabase {
    pub rewards: Vec<RewardMetadata>,
    #[serde(default)]
    pub pools: Vec<RewardPoolMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardPoolMetadata {
    pub id: String,
    pub reward_ids: Vec<String>,
}

impl RewardDatabase {
    pub fn new(rewards: Vec<RewardMetadata>) -> Self {
        Self::new_with_pools(rewards, vec![])
    }

    pub fn new_with_pools(rewards: Vec<RewardMetadata>, pools: Vec<RewardPoolMetadata>) -> Self {
        let by_id = once_lock_with(build_string_index(&rewards, "reward id", |item| &item.id));
        let by_uuid = once_lock_with(build_uuid_index(&rewards, "reward uuid", |item| item.uuid));

        Self {
            rewards,
            pools,
            by_id,
            by_uuid,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.rewards, "reward id", |item| &item.id))
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid
            .get_or_init(|| build_uuid_index(&self.rewards, "reward uuid", |item| item.uuid))
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_uuid();
        for reward in &self.rewards {
            assert!(
                !reward.resolved_tags().is_empty(),
                "reward '{}' must resolve at least one reward tag",
                reward.id
            );
        }
        for pool in &self.pools {
            assert!(!pool.id.is_empty(), "reward pool id must not be empty");
            for reward_id in &pool.reward_ids {
                assert!(
                    self.get_by_id(reward_id).is_some(),
                    "reward pool '{}' references missing reward '{}'",
                    pool.id,
                    reward_id
                );
            }
        }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&RewardMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.rewards.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&RewardMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.rewards.get(index))
    }

    pub fn pool_by_id(&self, id: &str) -> Option<&RewardPoolMetadata> {
        self.pools.iter().find(|pool| pool.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::skill_fragment_data::SkillFragmentId;

    #[test]
    fn reward_tags_are_inferred_from_effects_when_omitted() {
        let reward = RewardMetadata {
            id: "mixed_reward".to_string(),
            uuid: Uuid::from_u128(1),
            name: "Mixed".to_string(),
            description: "Mixed reward".to_string(),
            icon: "icons/mixed.png".to_string(),
            tags: Vec::new(),
            effects: vec![
                RewardEffect::GrantEnkephalin { amount: 10 },
                RewardEffect::GrantSkillFragment {
                    fragment_id: SkillFragmentId::from("fragment_test"),
                },
            ],
        };

        assert_eq!(
            reward.resolved_tags(),
            vec![RewardTag::Currency, RewardTag::SkillFragment]
        );
    }

    #[test]
    fn explicit_reward_tags_are_normalized_and_preferred() {
        let reward = RewardMetadata {
            id: "research_reward".to_string(),
            uuid: Uuid::from_u128(2),
            name: "Research".to_string(),
            description: "Research reward".to_string(),
            icon: "icons/research.png".to_string(),
            tags: vec![RewardTag::ResearchProgress, RewardTag::ResearchProgress],
            effects: vec![RewardEffect::GrantEnkephalin { amount: 1 }],
        };

        assert_eq!(reward.resolved_tags(), vec![RewardTag::ResearchProgress]);
    }

    #[test]
    fn skill_fragment_research_effect_infers_research_progress_tag() {
        let reward = RewardMetadata {
            id: "fragment_research_reward".to_string(),
            uuid: Uuid::from_u128(3),
            name: "Fragment Research".to_string(),
            description: "Fragment research reward".to_string(),
            icon: "icons/research.png".to_string(),
            tags: Vec::new(),
            effects: vec![RewardEffect::GrantSkillFragmentResearch {
                fragment_id: SkillFragmentId::from("fragment_test"),
                amount: 3,
            }],
        };

        assert_eq!(reward.resolved_tags(), vec![RewardTag::ResearchProgress]);
    }

    #[test]
    fn equipment_material_effect_infers_equipment_tag() {
        let reward = RewardMetadata {
            id: "equipment_material_reward".to_string(),
            uuid: Uuid::from_u128(4),
            name: "Equipment Material".to_string(),
            description: "Equipment material reward".to_string(),
            icon: "icons/equipment_material.png".to_string(),
            tags: Vec::new(),
            effects: vec![RewardEffect::GrantEquipmentMaterial {
                material_id: "damaged_weapon_fragment".to_string(),
                amount: 2,
            }],
        };

        assert_eq!(reward.resolved_tags(), vec![RewardTag::Equipment]);
    }
}
