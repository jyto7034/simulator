use std::{collections::HashMap, sync::OnceLock};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::{SkillActivationMode, SkillId},
    battle::{
        tile_range::{TileRangePattern, TileRangePolicy},
        types::DeploymentAffinity,
    },
    data::{
        abnormality_data::{AuthoredBasicAttackDef, BasicAttackDef, MovementDef, ResonanceDef},
        build_string_index, build_uuid_index,
    },
    stats::UnitStats,
};

fn default_magic_resist() -> i32 {
    0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CorrodedEmployeeProfileRole {
    #[default]
    Normal,
    Special,
    LegacyEcho,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorrodedEmployeeBasicAttackRangePresetDef {
    pub id: String,
    pub range: TileRangePattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorrodedEmployeeProfileMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    #[serde(default)]
    pub profile_role: CorrodedEmployeeProfileRole,
    #[serde(default)]
    pub basic_attack_range_preset: Option<String>,
    pub max_health: u32,
    pub attack: u32,
    pub defense: i32,
    #[serde(default = "default_magic_resist")]
    pub magic_resist: i32,
    #[serde(default)]
    pub movement: MovementDef,
    #[serde(default)]
    pub basic_attack: BasicAttackDef,
    #[serde(default)]
    pub resonance: ResonanceDef,
    #[serde(default)]
    pub skill_id: Option<SkillId>,
}

impl CorrodedEmployeeProfileMetadata {
    pub fn to_combat_profile(&self) -> crate::game::battle::types::UnitCombatProfile {
        let mut stats = UnitStats::with_values(
            self.max_health,
            self.max_health,
            self.attack,
            self.defense,
            self.basic_attack.interval_ms,
        );
        stats.magic_resist = self.magic_resist;
        stats.move_speed_units_per_ms = self.movement.speed_units_per_ms;

        crate::game::battle::types::UnitCombatProfile {
            stats,
            basic_attack: self.basic_attack.clone(),
            basic_attack_speed_percent: 100,
            weapon_profile: None,
            movement: self.movement.clone(),
            resonance: self.resonance.clone(),
            skill_id: self.skill_id.clone(),
            skill_activation_mode: SkillActivationMode::Auto,
            deployment_affinity: DeploymentAffinity::GroundOnly,
            block_capacity: 0,
            block_radius_units: 0.0,
            blockable: true,
            mobility_kind: crate::game::battle::types::MobilityKind::Ground,
            target_traits: Vec::new(),
            incoming_damage_modifiers: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrodedEmployeeProfileDatabase {
    #[serde(default)]
    pub range_presets: Vec<CorrodedEmployeeBasicAttackRangePresetDef>,
    pub profiles: Vec<CorrodedEmployeeProfileMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl CorrodedEmployeeProfileDatabase {
    pub fn new(profiles: Vec<CorrodedEmployeeProfileMetadata>) -> Self {
        Self::with_range_presets(Vec::new(), profiles)
    }

    pub fn with_range_presets(
        range_presets: Vec<CorrodedEmployeeBasicAttackRangePresetDef>,
        profiles: Vec<CorrodedEmployeeProfileMetadata>,
    ) -> Self {
        Self {
            range_presets,
            profiles,
            by_id: OnceLock::new(),
            by_uuid: OnceLock::new(),
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.profiles, "corroded employee profile id", |profile| {
                &profile.id
            })
        })
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid.get_or_init(|| {
            build_uuid_index(
                &self.profiles,
                "corroded employee profile uuid",
                |profile| profile.uuid,
            )
        })
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .expect("invalid corroded employee authoring contract");
        let _ = self.by_id();
        let _ = self.by_uuid();
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        let mut preset_ids = std::collections::HashSet::new();
        for (preset_index, preset) in self.range_presets.iter().enumerate() {
            if preset.id.trim().is_empty() {
                return Err(crate::game::data::DataValidationError::at(
                    &preset.id,
                    format!("range_presets[{preset_index}].id"),
                    "range preset id must not be empty",
                ));
            }
            if !preset_ids.insert(preset.id.as_str()) {
                return Err(crate::game::data::DataValidationError::at(
                    &preset.id,
                    format!("range_presets[{preset_index}].id"),
                    "duplicate range preset id",
                ));
            }
            preset.range.validate().map_err(|error| {
                crate::game::data::DataValidationError::at(
                    &preset.id,
                    format!("range_presets[{preset_index}].range"),
                    format!("invalid range preset: {error}"),
                )
            })?;
        }
        for (profile_index, profile) in self.profiles.iter().enumerate() {
            let profile_path = format!("profiles[{profile_index}]");
            if profile.max_health == 0 {
                return Err(crate::game::data::DataValidationError::at(
                    &profile.id,
                    format!("{profile_path}.max_health"),
                    "max_health must be greater than zero",
                ));
            }
            profile
                .basic_attack
                .try_validate_runtime_contract()
                .map_err(|cause| {
                    crate::game::data::DataValidationError::at(
                        &profile.id,
                        format!("{profile_path}.basic_attack"),
                        cause,
                    )
                })?;
            if profile.profile_role == CorrodedEmployeeProfileRole::Normal {
                if profile.basic_attack_range_preset.is_none() {
                    return Err(crate::game::data::DataValidationError::at(
                        &profile.id,
                        format!("{profile_path}.basic_attack_range_preset"),
                        "normal profile must declare basic_attack_range_preset",
                    ));
                }
                if profile.basic_attack.range_policy == TileRangePolicy::WholeFieldValidTiles {
                    return Err(crate::game::data::DataValidationError::at(
                        &profile.id,
                        format!("{profile_path}.basic_attack.range_policy"),
                        "normal profile must not use WholeFieldValidTiles",
                    ));
                }
            }
            if let Some(preset_id) = &profile.basic_attack_range_preset {
                if !preset_ids.contains(preset_id.as_str()) {
                    return Err(crate::game::data::DataValidationError::at(
                        &profile.id,
                        format!("{profile_path}.basic_attack_range_preset"),
                        format!("references unknown range preset '{preset_id}'"),
                    ));
                }
                if profile.basic_attack.defense_tile_range.is_none() {
                    return Err(crate::game::data::DataValidationError::at(
                        &profile.id,
                        format!("{profile_path}.basic_attack.defense_tile_range"),
                        format!("preset '{preset_id}' was not resolved"),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Option<&CorrodedEmployeeProfileMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.profiles.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&CorrodedEmployeeProfileMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.profiles.get(index))
    }
}

impl<'de> Deserialize<'de> for CorrodedEmployeeProfileDatabase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawCorrodedEmployeeProfileDatabase::deserialize(deserializer)?;
        raw.try_into_database().map_err(D::Error::custom)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "CorrodedEmployeeProfileDatabase")]
#[serde(deny_unknown_fields)]
struct RawCorrodedEmployeeProfileDatabase {
    #[serde(default)]
    range_presets: Vec<CorrodedEmployeeBasicAttackRangePresetDef>,
    profiles: Vec<RawCorrodedEmployeeProfileMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCorrodedEmployeeProfileMetadata {
    id: String,
    uuid: Uuid,
    name: String,
    profile_role: CorrodedEmployeeProfileRole,
    #[serde(default)]
    basic_attack_range_preset: Option<String>,
    max_health: u32,
    attack: u32,
    defense: i32,
    magic_resist: i32,
    movement: MovementDef,
    basic_attack: AuthoredBasicAttackDef,
    resonance: ResonanceDef,
    #[serde(default)]
    skill_id: Option<SkillId>,
}

impl RawCorrodedEmployeeProfileDatabase {
    fn try_into_database(self) -> Result<CorrodedEmployeeProfileDatabase, String> {
        let mut preset_index = HashMap::new();
        for (index, preset) in self.range_presets.iter().enumerate() {
            if preset_index.insert(preset.id.clone(), index).is_some() {
                return Err(format!(
                    "duplicate basic attack range preset id '{}'",
                    preset.id
                ));
            }
        }
        for preset in &self.range_presets {
            preset.range.validate().map_err(|error| {
                format!(
                    "basic attack range preset '{}' is invalid: {error}",
                    preset.id
                )
            })?;
        }

        let profiles = self
            .profiles
            .into_iter()
            .map(|profile| profile.try_into_profile(&self.range_presets, &preset_index))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CorrodedEmployeeProfileDatabase::with_range_presets(
            self.range_presets,
            profiles,
        ))
    }
}

impl RawCorrodedEmployeeProfileMetadata {
    fn try_into_profile(
        self,
        presets: &[CorrodedEmployeeBasicAttackRangePresetDef],
        preset_index: &HashMap<String, usize>,
    ) -> Result<CorrodedEmployeeProfileMetadata, String> {
        let mut basic_attack: BasicAttackDef = self.basic_attack.into();
        if let Some(preset_id) = &self.basic_attack_range_preset {
            if basic_attack.defense_tile_range.is_some() {
                return Err(format!(
                    "corroded employee profile '{}' defines both basic_attack.defense_tile_range and basic_attack_range_preset '{}'",
                    self.id, preset_id
                ));
            }
            let preset = preset_index
                .get(preset_id)
                .and_then(|&index| presets.get(index))
                .ok_or_else(|| {
                    format!(
                        "corroded employee profile '{}' references unknown basic_attack_range_preset '{}'",
                        self.id, preset_id
                    )
                })?;
            basic_attack.defense_tile_range = Some(preset.range.clone());
        }

        Ok(CorrodedEmployeeProfileMetadata {
            id: self.id,
            uuid: self.uuid,
            name: self.name,
            profile_role: self.profile_role,
            basic_attack_range_preset: self.basic_attack_range_preset,
            max_health: self.max_health,
            attack: self.attack,
            defense: self.defense,
            magic_resist: self.magic_resist,
            movement: self.movement,
            basic_attack,
            resonance: self.resonance,
            skill_id: self.skill_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corroded_employee_profile_deserializes_and_builds_combat_profile() {
        let profile: CorrodedEmployeeProfileMetadata = ron::de::from_str(
            r#"(
                id: "broken_guard",
                uuid: "90000000-0000-4000-8000-000000000001",
                name: "Broken Guard",
                max_health: 90,
                attack: 11,
                defense: 2,
                basic_attack: (range_units: 1.0, interval_ms: 1400),
            )"#,
        )
        .expect("profile should deserialize");

        let combat_profile = profile.to_combat_profile();

        assert_eq!(combat_profile.stats.max_health, 90);
        assert_eq!(combat_profile.stats.attack, 11);
        assert_eq!(combat_profile.basic_attack.interval_ms, 1400);
        assert_eq!(
            combat_profile.deployment_affinity,
            DeploymentAffinity::GroundOnly
        );
        assert_eq!(combat_profile.block_capacity, 0);
        assert_eq!(combat_profile.block_radius_units, 0.0);
        assert!(combat_profile.blockable);
    }

    #[test]
    fn corroded_employee_database_resolves_basic_attack_range_presets() {
        let database: CorrodedEmployeeProfileDatabase = ron::de::from_str(
            r#"CorrodedEmployeeProfileDatabase(
                range_presets: [
                    (
                        id: "melee_front_1",
                        range: (
                            include_anchor_tile: true,
                            rows: [
                                ".X.",
                                ".@.",
                                "...",
                            ],
                        ),
                    ),
                ],
                profiles: [
                    (
                        id: "broken_guard",
                        uuid: "90000000-0000-4000-8000-000000000001",
                        name: "Broken Guard",
                        profile_role: normal,
                        basic_attack_range_preset: Some("melee_front_1"),
                        max_health: 90,
                        attack: 11,
                        defense: 2,
                        magic_resist: 0,
                        movement: (speed_units_per_ms: 2500),
                        basic_attack: (
                            range_units: 1.0,
                            range_policy: pattern,
                            defense_tile_range: None,
                            damage_type: Physical,
                            targeting_profile: DefaultForward,
                            air_capable: false,
                            range_role: Melee,
                            interval_ms: 1400,
                            windup_ms: 200,
                            ranged_reposition_ms: 1000,
                            delivery: Instant,
                        ),
                        resonance: (),
                    ),
                ],
            )"#,
        )
        .expect("database should deserialize");

        database.validate_indexes();
        let profile = database
            .get_by_id("broken_guard")
            .expect("profile should exist");
        assert_eq!(
            profile.basic_attack_range_preset.as_deref(),
            Some("melee_front_1")
        );
        assert!(profile.basic_attack.defense_tile_range.is_some());
    }

    #[test]
    fn corroded_employee_database_rejects_unknown_authoring_fields() {
        let result = ron::de::from_str::<CorrodedEmployeeProfileDatabase>(
            r#"CorrodedEmployeeProfileDatabase(
                range_presets: [
                    (
                        id: "melee_front_1",
                        range: (
                            include_anchor_tile: true,
                            rows: [
                                ".X.",
                                ".@.",
                                "...",
                            ],
                        ),
                    ),
                ],
                profiles: [
                    (
                        id: "broken_guard",
                        uuid: "90000000-0000-4000-8000-000000000001",
                        name: "Broken Guard",
                        profile_role: normal,
                        basic_attack_range_preset: Some("melee_front_1"),
                        max_health: 90,
                        attack: 11,
                        defense: 2,
                        old_difficulty: 1,
                    ),
                ],
            )"#,
        );

        assert!(
            result.is_err(),
            "unknown corroded employee profile fields must not deserialize"
        );
        let error = result.expect_err("unknown field should be rejected");
        assert!(
            error.to_string().contains("old_difficulty"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn corroded_employee_database_rejects_implicit_basic_attack_authoring() {
        let result = ron::de::from_str::<CorrodedEmployeeProfileDatabase>(
            r#"CorrodedEmployeeProfileDatabase(
                range_presets: [
                    (
                        id: "melee_front_1",
                        range: (
                            include_anchor_tile: true,
                            rows: [
                                ".X.",
                                ".@.",
                                "...",
                            ],
                        ),
                    ),
                ],
                profiles: [
                    (
                        id: "broken_guard",
                        uuid: "90000000-0000-4000-8000-000000000001",
                        name: "Broken Guard",
                        profile_role: normal,
                        basic_attack_range_preset: Some("melee_front_1"),
                        max_health: 90,
                        attack: 11,
                        defense: 2,
                        basic_attack: (
                            range_units: 1.0,
                            range_policy: pattern,
                            defense_tile_range: None,
                            damage_type: Physical,
                            targeting_profile: DefaultForward,
                            air_capable: false,
                            range_role: Melee,
                            interval_ms: 1400,
                            windup_ms: 200,
                            delivery: Instant,
                        ),
                    ),
                ],
            )"#,
        );

        assert!(
            result.is_err(),
            "live corroded employee basic_attack authoring must reject missing combat fields"
        );
        let error = result.expect_err("missing field should be rejected");
        assert!(
            error.to_string().contains("ranged_reposition_ms"),
            "unexpected error: {error}"
        );
    }

    #[test]
    #[should_panic(expected = "must declare basic_attack_range_preset")]
    fn normal_corroded_employee_profile_requires_basic_attack_range_preset() {
        let database = CorrodedEmployeeProfileDatabase::with_range_presets(
            Vec::new(),
            vec![CorrodedEmployeeProfileMetadata {
                id: "broken_guard".to_string(),
                uuid: Uuid::from_u128(0x9000_0000_0000_4000_8000_0000_0000_0001),
                name: "Broken Guard".to_string(),
                profile_role: CorrodedEmployeeProfileRole::Normal,
                basic_attack_range_preset: None,
                max_health: 90,
                attack: 11,
                defense: 2,
                magic_resist: 0,
                movement: Default::default(),
                basic_attack: BasicAttackDef::default(),
                resonance: Default::default(),
                skill_id: None,
            }],
        );

        database.validate_indexes();
    }

    #[test]
    #[should_panic(expected = "must not use WholeFieldValidTiles")]
    fn normal_corroded_employee_profile_rejects_whole_field_basic_attack() {
        let database = CorrodedEmployeeProfileDatabase::with_range_presets(
            vec![CorrodedEmployeeBasicAttackRangePresetDef {
                id: "melee_front_1".to_string(),
                range: TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
                },
            }],
            vec![CorrodedEmployeeProfileMetadata {
                id: "broken_guard".to_string(),
                uuid: Uuid::from_u128(0x9000_0000_0000_4000_8000_0000_0000_0002),
                name: "Broken Guard".to_string(),
                profile_role: CorrodedEmployeeProfileRole::Normal,
                basic_attack_range_preset: Some("melee_front_1".to_string()),
                max_health: 90,
                attack: 11,
                defense: 2,
                magic_resist: 0,
                movement: Default::default(),
                basic_attack: BasicAttackDef {
                    range_policy: TileRangePolicy::WholeFieldValidTiles,
                    ..Default::default()
                },
                resonance: Default::default(),
                skill_id: None,
            }],
        );

        database.validate_indexes();
    }
}
