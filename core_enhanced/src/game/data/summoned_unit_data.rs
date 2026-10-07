use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::{SkillActivationMode, SkillId},
    battle::{
        damage::DamageModifiers,
        types::{DeploymentAffinity, MobilityKind, UnitCombatProfile},
    },
    stats::UnitStats,
};

use super::{
    abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
    boss_runtime_data::BossActorBehaviorId,
    skill_data::SkillDatabase,
    DataValidationError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SummonedUnitDefinitionId(pub String);

impl SummonedUnitDefinitionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummonerRemovalPolicyData {
    Persist,
    Despawn,
}

crate::authoring::clone_shape::newtype_shape!(SummonedUnitDefinitionId);
crate::authoring::clone_shape::copy_shape!(SummonerRemovalPolicyData);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummonedUnitCatalogData {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objects: Vec<super::independent_object_data::IndependentObjectDefinition>,
    #[serde(default)]
    pub definitions: Vec<SummonedUnitDefinitionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummonedUnitDefinitionData {
    pub id: SummonedUnitDefinitionId,
    pub base_uuid: Uuid,
    pub display_name: String,
    pub max_health: u32,
    pub attack: u32,
    pub defense: i32,
    pub magic_resistance: i32,
    pub movement: MovementDef,
    pub basic_attack: BasicAttackDef,
    pub resonance: ResonanceDef,
    pub skill_id: Option<SkillId>,
    pub max_alive: u32,
    pub max_total: u32,
    pub on_summoner_removed: SummonerRemovalPolicyData,
    pub actor_behavior_id: BossActorBehaviorId,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompiledSummonedUnitDefinition {
    pub id: SummonedUnitDefinitionId,
    pub base_uuid: Uuid,
    pub display_name: String,
    pub combat_profile: UnitCombatProfile,
    pub max_alive: u32,
    pub max_total: u32,
    pub on_summoner_removed: SummonerRemovalPolicyData,
    pub actor_behavior_id: BossActorBehaviorId,
}

#[derive(Debug, Clone, Default)]
pub struct SummonedUnitDatabase {
    pub(crate) objects: super::independent_object_data::IndependentObjectDatabase,
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    authored: SummonedUnitCatalogData,
    definitions: Vec<Arc<CompiledSummonedUnitDefinition>>,
    by_id: HashMap<SummonedUnitDefinitionId, usize>,
}

impl SummonedUnitDatabase {
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub(crate) fn authored_for_fixture_identity(&self) -> &SummonedUnitCatalogData {
        &self.authored
    }

    pub fn get_by_id(
        &self,
        id: &SummonedUnitDefinitionId,
    ) -> Option<&Arc<CompiledSummonedUnitDefinition>> {
        self.by_id
            .get(id)
            .and_then(|index| self.definitions.get(*index))
    }

    pub fn definitions(&self) -> &[Arc<CompiledSummonedUnitDefinition>] {
        &self.definitions
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub(crate) fn validate_skill_references(
        &self,
        skills: &SkillDatabase,
    ) -> Result<(), DataValidationError> {
        self.objects.validate_skill_references(skills)?;
        for definition in &self.definitions {
            if let Some(skill_id) = &definition.combat_profile.skill_id {
                if skills.get_by_id(skill_id.as_str()).is_none() {
                    return Err(DataValidationError::at(
                        definition.id.as_str(),
                        "skill_id",
                        format!("references missing skill '{}'", skill_id.as_str()),
                    ));
                }
            }
        }
        Ok(())
    }
}

impl SummonedUnitCatalogData {
    pub(crate) fn compile(&self) -> Result<SummonedUnitDatabase, DataValidationError> {
        let mut by_id = HashMap::new();
        let mut by_uuid = HashMap::new();
        let mut definitions = Vec::with_capacity(self.definitions.len());

        for (index, definition) in self.definitions.iter().enumerate() {
            validate_stable_id(definition.id.as_str(), definition.id.as_str(), "id")?;
            if by_id.insert(definition.id.clone(), index).is_some() {
                return Err(DataValidationError::at(
                    definition.id.as_str(),
                    "id",
                    format!(
                        "duplicate summoned-unit definition id '{}'",
                        definition.id.as_str()
                    ),
                ));
            }
            if let Some(previous) = by_uuid.insert(definition.base_uuid, definition.id.clone()) {
                return Err(DataValidationError::at(
                    definition.id.as_str(),
                    "base_uuid",
                    format!(
                        "base UUID {} is already owned by summoned-unit definition '{}'",
                        definition.base_uuid,
                        previous.as_str()
                    ),
                ));
            }
            if definition.display_name.trim().is_empty() {
                return Err(DataValidationError::at(
                    definition.id.as_str(),
                    "display_name",
                    "display name must not be empty",
                ));
            }
            if definition.max_health == 0 {
                return Err(DataValidationError::at(
                    definition.id.as_str(),
                    "max_health",
                    "max_health must be greater than zero",
                ));
            }
            if definition.max_alive == 0 || definition.max_total == 0 {
                return Err(DataValidationError::at(
                    definition.id.as_str(),
                    "limits",
                    "max_alive and max_total must both be greater than zero",
                ));
            }
            if definition.movement.speed_units_per_ms == 0 || definition.movement.radius_units == 0
            {
                return Err(DataValidationError::at(
                    definition.id.as_str(),
                    "movement",
                    "movement speed and body radius must both be greater than zero",
                ));
            }
            definition
                .basic_attack
                .try_validate_runtime_contract()
                .map_err(|cause| {
                    DataValidationError::at(
                        definition.id.as_str(),
                        format!("basic_attack{}", cause.field_suffix),
                        cause.cause,
                    )
                })?;
            validate_stable_id(
                definition.actor_behavior_id.as_str(),
                definition.id.as_str(),
                "actor_behavior_id",
            )?;

            let mut stats = UnitStats::with_values(
                definition.max_health,
                definition.max_health,
                definition.attack,
                definition.defense,
                definition.basic_attack.interval_ms,
            );
            stats.magic_resist = definition.magic_resistance;
            stats.move_speed_units_per_ms = definition.movement.speed_units_per_ms;
            let profile = UnitCombatProfile {
                stats,
                basic_attack: definition.basic_attack.clone(),
                basic_attack_speed_percent: 100,
                weapon_profile: None,
                movement: definition.movement.clone(),
                resonance: definition.resonance.clone(),
                skill_id: definition.skill_id.clone(),
                fragment_source: None,
                skill_activation_mode: SkillActivationMode::Auto,
                deployment_affinity: DeploymentAffinity::GroundOnly,
                block_capacity: 0,
                block_radius_units: 0.0,
                blockable: true,
                mobility_kind: MobilityKind::Ground,
                target_traits: Vec::new(),
                incoming_damage_modifiers: DamageModifiers::default(),
            };
            definitions.push(Arc::new(CompiledSummonedUnitDefinition {
                id: definition.id.clone(),
                base_uuid: definition.base_uuid,
                display_name: definition.display_name.clone(),
                combat_profile: profile,
                max_alive: definition.max_alive,
                max_total: definition.max_total,
                on_summoner_removed: definition.on_summoner_removed,
                actor_behavior_id: definition.actor_behavior_id.clone(),
            }));
        }

        let objects =
            super::independent_object_data::IndependentObjectDatabase::compile(&self.objects)?;
        for object in objects.definitions() {
            if by_uuid.contains_key(&object.definition.base_uuid) {
                return Err(DataValidationError::at(
                    object.definition.id.as_str(),
                    "base_uuid",
                    "UUID is already owned by a boss summoned unit",
                ));
            }
        }
        Ok(SummonedUnitDatabase {
            objects,
            definitions,
            by_id,
            #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
            authored: self.clone(),
        })
    }
}

pub(super) fn validate_stable_id(
    id: &str,
    object_id: &str,
    path: &str,
) -> Result<(), DataValidationError> {
    let (_, id) = super::content_pack::canonical_key_parts(id).ok_or_else(|| {
        DataValidationError::at(
            object_id,
            path,
            "invalid global or namespace::local catalog ID",
        )
    })?;
    let valid = !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && id
            .as_bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(DataValidationError::at(
            object_id,
            path,
            "ID must use lower_snake_case and start with a lowercase letter",
        ))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn definition() -> SummonedUnitDefinitionData {
        SummonedUnitDefinitionData {
            id: SummonedUnitDefinitionId::new("fixture_minion"),
            base_uuid: Uuid::from_u128(0x5155_4d4d_4f4e),
            display_name: "Fixture Minion".to_string(),
            max_health: 100,
            attack: 12,
            defense: 3,
            magic_resistance: 4,
            movement: MovementDef::default(),
            basic_attack: BasicAttackDef::default(),
            resonance: ResonanceDef::default(),
            skill_id: None,
            max_alive: 2,
            max_total: 4,
            on_summoner_removed: SummonerRemovalPolicyData::Persist,
            actor_behavior_id: BossActorBehaviorId::new("fixture_behavior"),
        }
    }

    #[test]
    fn catalog_builds_an_independent_combat_profile_and_requires_positive_limits() {
        let catalog = SummonedUnitCatalogData {
            objects: Vec::new(),
            definitions: vec![definition()],
        };
        let compiled = catalog.compile().unwrap();
        let item = compiled
            .get_by_id(&SummonedUnitDefinitionId::new("fixture_minion"))
            .unwrap();
        assert_eq!(item.combat_profile.stats.max_health, 100);
        assert_eq!(item.combat_profile.stats.attack, 12);
        assert_eq!(item.max_alive, 2);
        assert_eq!(item.max_total, 4);
        assert_eq!(item.combat_profile.skill_id, None);

        let mut invalid = definition();
        invalid.max_alive = 0;
        assert!(SummonedUnitCatalogData {
            objects: Vec::new(),
            definitions: vec![invalid]
        }
        .compile()
        .unwrap_err()
        .cause
        .contains("greater than zero"));
    }

    #[test]
    fn duplicate_definition_identity_is_rejected() {
        let first = definition();
        let mut second = definition();
        second.base_uuid = Uuid::from_u128(2);
        assert!(SummonedUnitCatalogData {
            objects: Vec::new(),
            definitions: vec![first, second]
        }
        .compile()
        .unwrap_err()
        .cause
        .contains("duplicate"));
    }

    #[test]
    fn ron_reader_rejects_missing_required_and_unknown_catalog_fields() {
        let encoded = ::ron::ser::to_string(&SummonedUnitCatalogData {
            objects: Vec::new(),
            definitions: vec![definition()],
        })
        .unwrap();
        let missing = encoded.replace("max_total:4,", "");
        assert_ne!(missing, encoded, "fixture must remove max_total");
        assert!(::ron::de::from_str::<SummonedUnitCatalogData>(&missing).is_err());

        assert!(::ron::de::from_str::<SummonedUnitCatalogData>(
            "(definitions:[],legacy_summons:[])"
        )
        .is_err());
    }
}
