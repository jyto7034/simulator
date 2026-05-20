use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::SkillId,
    data::{build_string_index, build_uuid_index, once_lock_with},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SkillFragmentId(String);

impl SkillFragmentId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SkillFragmentId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SkillFragmentId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SkillFragmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillFragmentRarity {
    Common,
    Rare,
    Exceptional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillFragmentAcquisitionSource {
    AbnormalitySuppression { abnormality_id: String },
    AbnormalityContainment { abnormality_id: String },
    RareReward,
    DependentConcept { concept_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillFragmentDependency {
    SourceAbnormality { abnormality_id: String },
    Concept { concept_id: String },
    RequiresFragment { fragment_id: SkillFragmentId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillFragmentOrigin {
    Abnormality { abnormality_id: String },
    Concept { concept_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillFragmentEffectDef {
    BasicAttackModifier {
        attack_bonus: u32,
        attack_interval_ms_reduction: u32,
    },
    ActiveSkill {
        imitation_skill_id: SkillId,
        #[serde(default)]
        upgrade_skill_ids: BTreeMap<u8, SkillId>,
        #[serde(default)]
        awakened_skill_id: Option<SkillId>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFragmentMetadata {
    pub id: SkillFragmentId,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub rarity: SkillFragmentRarity,
    #[serde(default)]
    pub origin: Option<SkillFragmentOrigin>,
    pub sources: Vec<SkillFragmentAcquisitionSource>,
    #[serde(default)]
    pub dependencies: Vec<SkillFragmentDependency>,
    pub effect: SkillFragmentEffectDef,
}

impl SkillFragmentMetadata {
    pub fn starter_basic_attack() -> Self {
        Self {
            id: SkillFragmentId::from("starter_basic_attack_enhancement"),
            uuid: Uuid::from_u128(0x5354_4152_5445_525f_4652_4147_0000_0001),
            name: "Starter Basic Attack Enhancement".to_string(),
            description:
                "A baseline fragment that lets employees perform reinforced basic attacks."
                    .to_string(),
            rarity: SkillFragmentRarity::Common,
            origin: Some(SkillFragmentOrigin::Concept {
                concept_id: "employee_baseline_training".to_string(),
            }),
            sources: vec![SkillFragmentAcquisitionSource::DependentConcept {
                concept_id: "employee_baseline_training".to_string(),
            }],
            dependencies: vec![],
            effect: SkillFragmentEffectDef::BasicAttackModifier {
                attack_bonus: 2,
                attack_interval_ms_reduction: 0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFragmentDatabase {
    pub fragments: Vec<SkillFragmentMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl SkillFragmentDatabase {
    pub fn new(fragments: Vec<SkillFragmentMetadata>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &fragments,
            "skill fragment id",
            |item| item.id.as_str(),
        ));
        let by_uuid = once_lock_with(build_uuid_index(
            &fragments,
            "skill fragment uuid",
            |item| item.uuid,
        ));

        Self {
            fragments,
            by_id,
            by_uuid,
        }
    }

    pub fn with_builtin_starter(mut fragments: Vec<SkillFragmentMetadata>) -> Self {
        if !fragments
            .iter()
            .any(|fragment| fragment.id.as_str() == "starter_basic_attack_enhancement")
        {
            fragments.push(SkillFragmentMetadata::starter_basic_attack());
        }
        Self::new(fragments)
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.fragments, "skill fragment id", |item| {
                item.id.as_str()
            })
        })
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid.get_or_init(|| {
            build_uuid_index(&self.fragments, "skill fragment uuid", |item| item.uuid)
        })
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_uuid();
    }

    pub fn get_by_id(&self, id: &SkillFragmentId) -> Option<&SkillFragmentMetadata> {
        self.by_id()
            .get(id.as_str())
            .and_then(|&index| self.fragments.get(index))
    }

    pub fn get_by_id_str(&self, id: &str) -> Option<&SkillFragmentMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.fragments.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&SkillFragmentMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.fragments.get(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_preserves_sources_and_dependencies_for_future_concepts() {
        let metadata = SkillFragmentMetadata {
            id: SkillFragmentId::from("one_sin_fragment"),
            uuid: Uuid::from_u128(10),
            name: "One Sin Fragment".to_string(),
            description: "desc".to_string(),
            rarity: SkillFragmentRarity::Rare,
            origin: Some(SkillFragmentOrigin::Abnormality {
                abnormality_id: "one_sin".to_string(),
            }),
            sources: vec![SkillFragmentAcquisitionSource::AbnormalitySuppression {
                abnormality_id: "one_sin".to_string(),
            }],
            dependencies: vec![SkillFragmentDependency::SourceAbnormality {
                abnormality_id: "one_sin".to_string(),
            }],
            effect: SkillFragmentEffectDef::ActiveSkill {
                imitation_skill_id: SkillId::from("fragment_one_sin_penitence"),
                upgrade_skill_ids: BTreeMap::new(),
                awakened_skill_id: None,
            },
        };

        assert!(matches!(
            metadata.origin,
            Some(SkillFragmentOrigin::Abnormality { .. })
        ));
        assert_eq!(metadata.sources.len(), 1);
        assert_eq!(metadata.dependencies.len(), 1);
        assert!(matches!(
            metadata.effect,
            SkillFragmentEffectDef::ActiveSkill { .. }
        ));
    }
}
