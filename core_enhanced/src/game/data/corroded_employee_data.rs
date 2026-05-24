use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::SkillId,
    battle::types::DeploymentAffinity,
    data::{
        abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
        build_string_index, build_uuid_index, once_lock_with,
    },
    stats::UnitStats,
};

fn default_magic_resist() -> i32 {
    0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrodedEmployeeProfileMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
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
            movement: self.movement.clone(),
            resonance: self.resonance.clone(),
            skill_id: self.skill_id.clone(),
            deployment_affinity: DeploymentAffinity::GroundOnly,
            block_capacity: 0,
            block_radius_units: 0.0,
            blockable: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrodedEmployeeProfileDatabase {
    pub profiles: Vec<CorrodedEmployeeProfileMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl CorrodedEmployeeProfileDatabase {
    pub fn new(profiles: Vec<CorrodedEmployeeProfileMetadata>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &profiles,
            "corroded employee profile id",
            |profile| &profile.id,
        ));
        let by_uuid = once_lock_with(build_uuid_index(
            &profiles,
            "corroded employee profile uuid",
            |profile| profile.uuid,
        ));

        Self {
            profiles,
            by_id,
            by_uuid,
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
        let _ = self.by_id();
        let _ = self.by_uuid();
        for profile in &self.profiles {
            assert!(
                profile.max_health > 0,
                "corroded employee profile '{}' max_health must be greater than zero",
                profile.id
            );
            assert!(
                profile.basic_attack.interval_ms > 0,
                "corroded employee profile '{}' attack interval must be greater than zero",
                profile.id
            );
        }
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
}
