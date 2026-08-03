use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::game::{
    combat_preview::CombatNodeType,
    data::{build_string_index, deserialize_required_option, once_lock_with},
    enums::{RiskLevel, Tier},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorrodedWaveCountRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorrodedWaveRoleWeight {
    pub profile_id: String,
    pub weight: u32,
    pub cost: u32,
    pub min_count: u32,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub max_count: Option<u32>,
    pub tier: Tier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorrodedWavePreset {
    pub id: String,
    pub pressure: String,
    pub preferred_node_types: Vec<CombatNodeType>,
    pub preferred_risk_levels: Vec<RiskLevel>,
    pub budget: u32,
    pub count_range: CorrodedWaveCountRange,
    pub role_mix: Vec<CorrodedWaveRoleWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (preset_index, preset) in self.presets.iter().enumerate() {
            let preset_path = format!("presets[{preset_index}]");
            if preset.id.trim().is_empty() {
                return Err(crate::game::data::DataValidationError::at(
                    &preset.id,
                    format!("{preset_path}.id"),
                    "preset id must not be empty",
                ));
            }
            if preset.budget == 0 {
                return Err(crate::game::data::DataValidationError::at(
                    &preset.id,
                    format!("{preset_path}.budget"),
                    "budget must be greater than zero",
                ));
            }
            if preset.count_range.min == 0 || preset.count_range.min > preset.count_range.max {
                return Err(crate::game::data::DataValidationError::at(
                    &preset.id,
                    format!("{preset_path}.count_range"),
                    "count range must be positive and ordered",
                ));
            }
            if preset.role_mix.is_empty() {
                return Err(crate::game::data::DataValidationError::at(
                    &preset.id,
                    format!("{preset_path}.role_mix"),
                    "must define at least one role",
                ));
            }
            for (role_index, role) in preset.role_mix.iter().enumerate() {
                let role_path = format!("{preset_path}.role_mix[{role_index}]");
                if role.profile_id.trim().is_empty() {
                    return Err(crate::game::data::DataValidationError::at(
                        &preset.id,
                        format!("{role_path}.profile_id"),
                        "profile id must not be empty",
                    ));
                }
                if role.weight == 0 {
                    return Err(crate::game::data::DataValidationError::at(
                        &preset.id,
                        format!("{role_path}.weight"),
                        "role weight must be greater than zero",
                    ));
                }
                if role.cost == 0 {
                    return Err(crate::game::data::DataValidationError::at(
                        &preset.id,
                        format!("{role_path}.cost"),
                        "role cost must be greater than zero",
                    ));
                }
                if role
                    .max_count
                    .is_some_and(|max_count| role.min_count > max_count)
                {
                    return Err(crate::game::data::DataValidationError::at(
                        &preset.id,
                        format!("{role_path}.max_count"),
                        "min_count exceeds max_count",
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .unwrap_or_else(|error| panic!("invalid corroded wave data: {error:?}"));
        let _ = self.by_id();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_corroded_wave_presets_parse_without_difficulty() {
        let database: CorrodedWavePresetDatabase = ron::de::from_str(include_str!(
            "../../../../game_resources/data/enemies/corroded_wave_presets.ron"
        ))
        .expect("live corroded_wave_presets.ron should parse");

        assert!(!database.presets.is_empty());
    }

    #[test]
    fn stale_difficulty_field_is_rejected() {
        let stale = r#"
            CorrodedWavePresetDatabase(
                presets: [
                    CorrodedWavePreset(
                        id: "stale",
                        difficulty: 1,
                        pressure: "test",
                        budget: 1,
                        count_range: (min: 1, max: 1),
                        role_mix: [
                            (profile_id: "corroded_guard"),
                        ],
                    ),
                ],
            )
        "#;

        let result = ron::de::from_str::<CorrodedWavePresetDatabase>(stale);

        assert!(result.is_err(), "stale difficulty field must not parse");
    }
}
