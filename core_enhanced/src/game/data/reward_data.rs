use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    data::{
        build_string_index, build_uuid_index,
        equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType},
        once_lock_with,
    },
    enums::RiskLevel,
    reward::RewardEffect,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RewardGrantKind {
    Currency,
    Experience,
    Equipment,
    Artifact,
    Consumable,
    SkillFragment,
    FragmentDust,
    ResearchProgress,
    Narrative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub effects: Vec<RewardEffect>,
}

impl RewardMetadata {
    pub fn grant_kinds(&self) -> Vec<RewardGrantKind> {
        RewardGrantKind::from_effects(&self.effects)
    }
}

impl RewardGrantKind {
    pub fn from_effects(effects: &[RewardEffect]) -> Vec<Self> {
        let mut kinds = Vec::new();
        for effect in effects {
            match effect {
                RewardEffect::GrantEnkephalin { .. } => kinds.push(Self::Currency),
                RewardEffect::GrantExperience { .. } => kinds.push(Self::Experience),
                RewardEffect::GrantEquipment { .. }
                | RewardEffect::GrantEquipmentFromPool { .. }
                | RewardEffect::GrantEquipmentMaterial { .. } => kinds.push(Self::Equipment),
                RewardEffect::GrantArtifact { .. } => kinds.push(Self::Artifact),
                RewardEffect::GrantConsumable { .. } => kinds.push(Self::Consumable),
                RewardEffect::GrantSkillFragment { .. } => kinds.push(Self::SkillFragment),
                RewardEffect::GrantFragmentDust { .. } => kinds.push(Self::FragmentDust),
                RewardEffect::GrantSkillFragmentResearch { .. } => {
                    kinds.push(Self::ResearchProgress)
                }
            }
        }
        normalized_grant_kinds(kinds)
    }
}

fn normalized_grant_kinds(mut kinds: Vec<RewardGrantKind>) -> Vec<RewardGrantKind> {
    kinds.sort_by_key(|kind| *kind as u8);
    kinds.dedup();
    kinds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDatabase {
    pub rewards: Vec<RewardMetadata>,
    #[serde(default)]
    pub pools: Vec<RewardPoolMetadata>,
    #[serde(default)]
    pub equipment_pools: Vec<EquipmentRewardPoolMetadata>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentRewardPoolMetadata {
    pub id: String,
    #[serde(default)]
    pub filter: EquipmentRewardPoolFilter,
    #[serde(default)]
    pub entries: Vec<EquipmentRewardPoolEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentRewardPoolFilter {
    pub rarity_min: Option<RiskLevel>,
    pub rarity_max: Option<RiskLevel>,
    #[serde(default)]
    pub equipment_type_in: Vec<EquipmentType>,
    #[serde(default)]
    pub exclude_bound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentRewardPoolEntry {
    pub equipment_id: String,
    pub weight: u32,
}

impl EquipmentRewardPoolMetadata {
    pub fn candidates<'a>(
        &'a self,
        equipment_data: &'a EquipmentDatabase,
    ) -> Vec<(&'a EquipmentMetadata, u32)> {
        if self.entries.is_empty() {
            return equipment_data
                .items
                .iter()
                .filter(|equipment| self.filter.matches(equipment))
                .map(|equipment| (equipment, 1))
                .collect();
        }

        self.entries
            .iter()
            .filter(|entry| entry.weight > 0)
            .filter_map(|entry| {
                equipment_data
                    .get_by_id(&entry.equipment_id)
                    .filter(|equipment| self.filter.matches(equipment))
                    .map(|equipment| (equipment, entry.weight))
            })
            .collect()
    }
}

impl EquipmentRewardPoolFilter {
    pub fn matches(&self, equipment: &EquipmentMetadata) -> bool {
        if self.exclude_bound && equipment.bound {
            return false;
        }
        if let Some(min) = self.rarity_min {
            if risk_rank(equipment.rarity) < risk_rank(min) {
                return false;
            }
        }
        if let Some(max) = self.rarity_max {
            if risk_rank(equipment.rarity) > risk_rank(max) {
                return false;
            }
        }
        if !self.equipment_type_in.is_empty()
            && !self.equipment_type_in.contains(&equipment.equipment_type)
        {
            return false;
        }
        true
    }
}

fn risk_rank(risk: RiskLevel) -> u8 {
    match risk {
        RiskLevel::ZAYIN => 0,
        RiskLevel::TETH => 1,
        RiskLevel::HE => 2,
        RiskLevel::WAW => 3,
        RiskLevel::ALEPH => 4,
    }
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
            equipment_pools: vec![],
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
                !reward.grant_kinds().is_empty(),
                "reward '{}' must resolve at least one grant kind",
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

    pub fn equipment_pool_by_id(&self, id: &str) -> Option<&EquipmentRewardPoolMetadata> {
        self.equipment_pools.iter().find(|pool| pool.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::skill_fragment_data::SkillFragmentId;

    #[test]
    fn reward_grant_kinds_are_inferred_from_effects() {
        let reward = RewardMetadata {
            id: "mixed_reward".to_string(),
            uuid: Uuid::from_u128(1),
            name: "Mixed".to_string(),
            description: "Mixed reward".to_string(),
            icon: "icons/mixed.png".to_string(),
            effects: vec![
                RewardEffect::GrantEnkephalin { amount: 10 },
                RewardEffect::GrantSkillFragment {
                    fragment_id: SkillFragmentId::from("fragment_test"),
                },
            ],
        };

        assert_eq!(
            reward.grant_kinds(),
            vec![RewardGrantKind::Currency, RewardGrantKind::SkillFragment]
        );
    }

    #[test]
    fn skill_fragment_research_effect_infers_research_progress_tag() {
        let reward = RewardMetadata {
            id: "fragment_research_reward".to_string(),
            uuid: Uuid::from_u128(3),
            name: "Fragment Research".to_string(),
            description: "Fragment research reward".to_string(),
            icon: "icons/research.png".to_string(),
            effects: vec![RewardEffect::GrantSkillFragmentResearch {
                fragment_id: SkillFragmentId::from("fragment_test"),
                amount: 3,
            }],
        };

        assert_eq!(
            reward.grant_kinds(),
            vec![RewardGrantKind::ResearchProgress]
        );
    }

    #[test]
    fn equipment_material_effect_infers_equipment_tag() {
        let reward = RewardMetadata {
            id: "equipment_material_reward".to_string(),
            uuid: Uuid::from_u128(4),
            name: "Equipment Material".to_string(),
            description: "Equipment material reward".to_string(),
            icon: "icons/equipment_material.png".to_string(),
            effects: vec![RewardEffect::GrantEquipmentMaterial {
                material_id: "equipment_dust".to_string(),
                amount: 2,
            }],
        };

        assert_eq!(reward.grant_kinds(), vec![RewardGrantKind::Equipment]);
    }
}
