use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::game::{
    combat_preview::CombatNodeType,
    data::{build_string_index, once_lock_with},
    enums::{RiskLevel, Tier},
};

fn default_role_weight() -> u32 {
    1
}

fn default_role_cost() -> u32 {
    1
}

fn default_role_min_count() -> u32 {
    0
}

fn default_tier() -> Tier {
    Tier::I
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorrodedWaveCountRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorrodedWaveRoleWeight {
    pub profile_id: String,
    #[serde(default = "default_role_weight")]
    pub weight: u32,
    #[serde(default = "default_role_cost")]
    pub cost: u32,
    #[serde(default = "default_role_min_count")]
    pub min_count: u32,
    #[serde(default)]
    pub max_count: Option<u32>,
    #[serde(default = "default_tier")]
    pub tier: Tier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorrodedWavePreset {
    pub id: String,
    pub difficulty: u8,
    pub pressure: String,
    #[serde(default)]
    pub preferred_node_types: Vec<CombatNodeType>,
    #[serde(default)]
    pub preferred_risk_levels: Vec<RiskLevel>,
    pub budget: u32,
    pub count_range: CorrodedWaveCountRange,
    pub role_mix: Vec<CorrodedWaveRoleWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrodedWavePresetDatabase {
    pub presets: Vec<CorrodedWavePreset>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl CorrodedWavePresetDatabase {
    pub fn new(presets: Vec<CorrodedWavePreset>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &presets,
            "corroded wave preset id",
            |preset| &preset.id,
        ));
        Self { presets, by_id }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.presets, "corroded wave preset id", |preset| {
                &preset.id
            })
        })
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        for preset in &self.presets {
            assert!(
                !preset.id.trim().is_empty(),
                "corroded wave preset id must not be empty"
            );
            assert!(
                preset.budget > 0,
                "corroded wave preset '{}' budget must be greater than zero",
                preset.id
            );
            assert!(
                preset.count_range.min > 0 && preset.count_range.min <= preset.count_range.max,
                "corroded wave preset '{}' count range must be positive and ordered",
                preset.id
            );
            assert!(
                !preset.role_mix.is_empty(),
                "corroded wave preset '{}' must define at least one role",
                preset.id
            );
            for role in &preset.role_mix {
                assert!(
                    !role.profile_id.trim().is_empty(),
                    "corroded wave preset '{}' has empty profile id",
                    preset.id
                );
                assert!(
                    role.weight > 0,
                    "corroded wave preset '{}' role '{}' weight must be greater than zero",
                    preset.id,
                    role.profile_id
                );
                assert!(
                    role.cost > 0,
                    "corroded wave preset '{}' role '{}' cost must be greater than zero",
                    preset.id,
                    role.profile_id
                );
                if let Some(max_count) = role.max_count {
                    assert!(
                        role.min_count <= max_count,
                        "corroded wave preset '{}' role '{}' min_count exceeds max_count",
                        preset.id,
                        role.profile_id
                    );
                }
            }
        }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&CorrodedWavePreset> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.presets.get(index))
    }
}

impl CorrodedWaveRoleWeight {
    pub fn to_enemy_data(&self, count: u32) -> crate::game::data::pve_data::PveWaveEnemyData {
        crate::game::data::pve_data::PveWaveEnemyData::CorrodedEmployee {
            profile_id: self.profile_id.clone(),
            tier: self.tier,
            count,
        }
    }
}
