use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::SkillId,
    battle::{damage::DamageType, types::UnitCombatProfile},
    data::{
        build_string_index, build_uuid_index,
        equipment_data::{TargetingProfile, WeaponArchetype, WeaponRangeRole},
        once_lock_with,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillFragmentEquipLimit {
    OwnedCopies,
    GlobalExclusive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillFragmentAcquisitionSource {
    AbnormalityContainment { abnormality_id: String },
    RareReward,
    DependentConcept { concept_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillFragmentDependency {
    SourceAbnormality { abnormality_id: String },
    Concept { concept_id: String },
    RequiresFragment { fragment_id: SkillFragmentId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillFragmentOrigin {
    Abnormality { abnormality_id: String },
    Concept { concept_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FragmentCorrosionClass {
    #[default]
    Ordinary,
    ExcludedFinalBoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillFragmentEffectDef {
    BasicAttackModifier {
        attack_bonus: u32,
        attack_interval_ms_reduction: u32,
    },
    ActiveSkill {
        #[serde(
            deserialize_with = "crate::game::skill_fragment::bounded::map::<_, _, _, MAX_FRAGMENT_FORMS>"
        )]
        forms: BTreeMap<String, SkillFragmentForm>,
        #[serde(default)]
        dismantle_recipe: Option<SkillFragmentRecipe>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        passive_guard: Option<ParryGuardDef>,
    },
}

/// Narrow exception for an active parry fragment; no additional active slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParryGuardDef {
    pub reduction_percent: u32,
    pub reflection_chance_percent: u32,
    pub reflection_percent: u32,
}
crate::authoring::clone_shape::copy_shape!(ParryGuardDef);
impl ParryGuardDef {
    pub fn validate(&self) -> Result<(), String> {
        if self.reduction_percent > 100
            || self.reflection_chance_percent > 100
            || self.reflection_percent > 100
        {
            return Err("passive parry guard percentages must be in 0..100".into());
        }
        Ok(())
    }
}

pub const BASE_FRAGMENT_FORM: &str = "base";
pub const MAX_FRAGMENT_FORMS: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillFragmentForm {
    pub skill_id: SkillId,
    /// Capacity belongs to the selected fragment form, not the employee profile.
    pub resonance_max: u32,
    /// Additional requirements; definition-level compatibility still applies.
    #[serde(default)]
    pub compatibility: Option<SkillFragmentCompatibilityRequirements>,
    #[serde(default)]
    pub bloom_recipe: Option<SkillFragmentRecipe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillFragmentRecipe {
    pub enkephalin: u32,
    pub materials: Vec<crate::game::data::equipment_data::EquipmentMaterialCost>,
}

impl SkillFragmentRecipe {
    pub fn aggregated_materials(&self) -> Option<BTreeMap<String, u32>> {
        if self.materials.len() > 64 || (self.enkephalin == 0 && self.materials.is_empty()) {
            return None;
        }
        let mut amounts = BTreeMap::<String, u32>::new();
        for cost in &self.materials {
            if cost.amount == 0 || cost.material_id.is_empty() || cost.material_id.len() > 128 {
                return None;
            }
            let amount = amounts.entry(cost.material_id.clone()).or_default();
            *amount = amount.checked_add(cost.amount)?;
        }
        Some(amounts)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillFragmentCompatibilityRequirements {
    #[serde(default)]
    pub allowed_range_roles: Vec<WeaponRangeRole>,
    #[serde(default)]
    pub allowed_weapon_archetypes: Vec<WeaponArchetype>,
    #[serde(default)]
    pub allowed_damage_types: Vec<DamageType>,
    #[serde(default)]
    pub allowed_targeting_profiles: Vec<TargetingProfile>,
    #[serde(default)]
    pub requires_air_capable: Option<bool>,
    #[serde(default)]
    pub block_capacity_min: Option<u32>,
    #[serde(default)]
    pub required_capability_tags: Vec<String>,
    #[serde(default)]
    pub incompatible_capability_tags: Vec<String>,
}

impl SkillFragmentCompatibilityRequirements {
    pub fn is_empty(&self) -> bool {
        self.allowed_range_roles.is_empty()
            && self.allowed_weapon_archetypes.is_empty()
            && self.allowed_damage_types.is_empty()
            && self.allowed_targeting_profiles.is_empty()
            && self.requires_air_capable.is_none()
            && self.block_capacity_min.is_none()
            && self.required_capability_tags.is_empty()
            && self.incompatible_capability_tags.is_empty()
    }

    fn validate_static_contract(&self) -> Result<(), CompatibilityValidationError> {
        if has_duplicate(&self.allowed_range_roles) {
            return Err(CompatibilityValidationError::new(
                "allowed_range_roles",
                "contains duplicate values",
            ));
        }
        if has_duplicate(&self.allowed_weapon_archetypes) {
            return Err(CompatibilityValidationError::new(
                "allowed_weapon_archetypes",
                "contains duplicate values",
            ));
        }
        if has_duplicate(&self.allowed_damage_types) {
            return Err(CompatibilityValidationError::new(
                "allowed_damage_types",
                "contains duplicate values",
            ));
        }
        if has_duplicate(&self.allowed_targeting_profiles) {
            return Err(CompatibilityValidationError::new(
                "allowed_targeting_profiles",
                "contains duplicate values",
            ));
        }
        if matches!(self.block_capacity_min, Some(0)) {
            return Err(CompatibilityValidationError::new(
                "block_capacity_min",
                "must be greater than zero",
            ));
        }
        validate_capability_tags("required_capability_tags", &self.required_capability_tags)?;
        validate_capability_tags(
            "incompatible_capability_tags",
            &self.incompatible_capability_tags,
        )?;
        for tag in &self.required_capability_tags {
            if self
                .incompatible_capability_tags
                .iter()
                .any(|incompatible| incompatible == tag)
            {
                return Err(CompatibilityValidationError::new(
                    "incompatible_capability_tags",
                    format!("tag '{tag}' cannot be both required and incompatible"),
                ));
            }
        }
        Ok(())
    }

    pub fn evaluate(
        &self,
        context: SkillFragmentCompatibilityContext<'_>,
    ) -> SkillFragmentCompatibilityReport {
        let mut failure_codes = Vec::new();
        let weapon_profile = context.combat_profile.weapon_profile.as_ref();

        let needs_weapon = !self.allowed_range_roles.is_empty()
            || !self.allowed_weapon_archetypes.is_empty()
            || !self.allowed_damage_types.is_empty()
            || !self.allowed_targeting_profiles.is_empty()
            || self.requires_air_capable.is_some();

        if needs_weapon && weapon_profile.is_none() {
            failure_codes.push(SkillFragmentCompatibilityFailureCode::WeaponRequired);
        }

        if let Some(weapon_profile) = weapon_profile {
            if !self.allowed_range_roles.is_empty()
                && !self
                    .allowed_range_roles
                    .contains(&weapon_profile.range_role)
            {
                failure_codes.push(SkillFragmentCompatibilityFailureCode::RangeRoleMismatch);
            }

            if !self.allowed_weapon_archetypes.is_empty()
                && !self
                    .allowed_weapon_archetypes
                    .contains(&weapon_profile.weapon_archetype)
            {
                failure_codes.push(SkillFragmentCompatibilityFailureCode::WeaponArchetypeMismatch);
            }

            if !self.allowed_damage_types.is_empty()
                && !self
                    .allowed_damage_types
                    .contains(&weapon_profile.damage_type)
            {
                failure_codes.push(SkillFragmentCompatibilityFailureCode::DamageTypeMismatch);
            }

            if !self.allowed_targeting_profiles.is_empty()
                && !self
                    .allowed_targeting_profiles
                    .contains(&weapon_profile.targeting_profile)
            {
                failure_codes.push(SkillFragmentCompatibilityFailureCode::TargetingProfileMismatch);
            }

            if let Some(required_air_capable) = self.requires_air_capable {
                if weapon_profile.air_capable != required_air_capable {
                    failure_codes
                        .push(SkillFragmentCompatibilityFailureCode::AirCapabilityMismatch);
                }
            }
        }

        if let Some(block_capacity_min) = self.block_capacity_min {
            let base_block_capacity = weapon_profile
                .map(|profile| profile.block_capacity)
                .unwrap_or(0);
            if base_block_capacity < block_capacity_min {
                failure_codes.push(SkillFragmentCompatibilityFailureCode::BlockCapacityTooLow);
            }
        }

        for required_tag in &self.required_capability_tags {
            if !context
                .capability_tags
                .iter()
                .any(|tag| tag == required_tag)
            {
                failure_codes.push(SkillFragmentCompatibilityFailureCode::MissingCapabilityTag);
                break;
            }
        }

        for incompatible_tag in &self.incompatible_capability_tags {
            if context
                .capability_tags
                .iter()
                .any(|tag| tag == incompatible_tag)
            {
                failure_codes
                    .push(SkillFragmentCompatibilityFailureCode::IncompatibleCapabilityTag);
                break;
            }
        }

        SkillFragmentCompatibilityReport {
            is_compatible: failure_codes.is_empty(),
            failure_codes,
        }
    }
}

fn has_duplicate<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values.iter().skip(index + 1).any(|other| other == value))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompatibilityValidationError {
    field_path: &'static str,
    cause: String,
}

impl CompatibilityValidationError {
    fn new(field_path: &'static str, cause: impl Into<String>) -> Self {
        Self {
            field_path,
            cause: cause.into(),
        }
    }
}

fn validate_capability_tags(
    field_name: &'static str,
    tags: &[String],
) -> Result<(), CompatibilityValidationError> {
    if tags.iter().any(|tag| tag.trim().is_empty()) {
        return Err(CompatibilityValidationError::new(
            field_name,
            "contains an empty tag",
        ));
    }
    if has_duplicate(tags) {
        return Err(CompatibilityValidationError::new(
            field_name,
            "contains duplicate tags",
        ));
    }
    Ok(())
}

impl SkillFragmentCompatibilityReport {
    pub fn compatible() -> Self {
        Self {
            is_compatible: true,
            failure_codes: Vec::new(),
        }
    }

    pub fn add_failure(&mut self, failure_code: SkillFragmentCompatibilityFailureCode) {
        if !self.failure_codes.contains(&failure_code) {
            self.failure_codes.push(failure_code);
            self.is_compatible = false;
        }
    }

    pub fn add_failure_first(&mut self, failure_code: SkillFragmentCompatibilityFailureCode) {
        if !self.failure_codes.contains(&failure_code) {
            self.failure_codes.insert(0, failure_code);
            self.is_compatible = false;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SkillFragmentCompatibilityContext<'a> {
    pub combat_profile: &'a UnitCombatProfile,
    pub capability_tags: &'a [String],
}

impl<'a> SkillFragmentCompatibilityContext<'a> {
    pub fn from_combat_profile(combat_profile: &'a UnitCombatProfile) -> Self {
        Self {
            combat_profile,
            capability_tags: &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillFragmentCompatibilityReport {
    pub is_compatible: bool,
    #[serde(default)]
    pub failure_codes: Vec<SkillFragmentCompatibilityFailureCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillFragmentCompatibilityFailureCode {
    WeaponRequired,
    RangeRoleMismatch,
    WeaponArchetypeMismatch,
    DamageTypeMismatch,
    TargetingProfileMismatch,
    AirCapabilityMismatch,
    BlockCapacityTooLow,
    MissingCapabilityTag,
    IncompatibleCapabilityTag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillFragmentMetadata {
    pub id: SkillFragmentId,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub rarity: SkillFragmentRarity,
    pub equip_limit: SkillFragmentEquipLimit,
    #[serde(default)]
    pub origin: Option<SkillFragmentOrigin>,
    #[serde(default)]
    pub corrosion_class: FragmentCorrosionClass,
    pub sources: Vec<SkillFragmentAcquisitionSource>,
    #[serde(default)]
    pub dependencies: Vec<SkillFragmentDependency>,
    #[serde(default)]
    pub compatibility: SkillFragmentCompatibilityRequirements,
    pub effect: SkillFragmentEffectDef,
}

impl SkillFragmentMetadata {
    pub fn compatibility_report(
        &self,
        combat_profile: &UnitCombatProfile,
    ) -> SkillFragmentCompatibilityReport {
        match self.effect {
            SkillFragmentEffectDef::BasicAttackModifier { .. } => {
                SkillFragmentCompatibilityReport::compatible()
            }
            SkillFragmentEffectDef::ActiveSkill { .. } => {
                let mut report = self.compatibility.evaluate(
                    SkillFragmentCompatibilityContext::from_combat_profile(combat_profile),
                );
                if combat_profile.weapon_profile.is_none() {
                    report.add_failure_first(SkillFragmentCompatibilityFailureCode::WeaponRequired);
                }
                report
            }
        }
    }

    pub fn form_compatibility_report(
        &self,
        form_id: &str,
        profile: &UnitCombatProfile,
    ) -> Option<SkillFragmentCompatibilityReport> {
        let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &self.effect else {
            return None;
        };
        let form = forms.get(form_id)?;
        let mut report = self.compatibility_report(profile);
        if let Some(extra) = &form.compatibility {
            let extra = extra.evaluate(SkillFragmentCompatibilityContext::from_combat_profile(
                profile,
            ));
            report.is_compatible &= extra.is_compatible;
            for failure in extra.failure_codes {
                if !report.failure_codes.contains(&failure) {
                    report.failure_codes.push(failure);
                }
            }
        }
        Some(report)
    }

    #[cfg(test)]
    pub fn starter_basic_attack() -> Self {
        Self {
            corrosion_class: FragmentCorrosionClass::Ordinary,
            id: SkillFragmentId::from("starter_basic_attack_enhancement"),
            uuid: Uuid::from_u128(0x5354_4152_5445_525f_4652_4147_0000_0001),
            name: "Starter Basic Attack Enhancement".to_string(),
            description:
                "A baseline fragment that lets employees perform reinforced basic attacks."
                    .to_string(),
            rarity: SkillFragmentRarity::Common,
            equip_limit: SkillFragmentEquipLimit::OwnedCopies,
            origin: Some(SkillFragmentOrigin::Concept {
                concept_id: "employee_baseline_training".to_string(),
            }),
            sources: vec![SkillFragmentAcquisitionSource::DependentConcept {
                concept_id: "employee_baseline_training".to_string(),
            }],
            dependencies: vec![],
            compatibility: SkillFragmentCompatibilityRequirements::default(),
            effect: SkillFragmentEffectDef::BasicAttackModifier {
                attack_bonus: 2,
                attack_interval_ms_reduction: 0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillFragmentDatabase {
    pub fragments: Vec<SkillFragmentMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl SkillFragmentDatabase {
    // Only assemble here: final cross-catalog validation must precede indexes.
    pub(crate) fn from_raw_parts(fragments: Vec<SkillFragmentMetadata>) -> Self {
        Self {
            fragments,
            by_id: OnceLock::new(),
            by_uuid: OnceLock::new(),
        }
    }

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

    #[cfg(test)]
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
        self.validate_contract()
            .expect("invalid skill fragment compatibility contract");
        let _ = self.by_id();
        let _ = self.by_uuid();
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (fragment_index, fragment) in self.fragments.iter().enumerate() {
            if let SkillFragmentEffectDef::ActiveSkill {
                forms,
                dismantle_recipe,
                passive_guard,
            } = &fragment.effect
            {
                let invalid = |field: String, cause: String| {
                    crate::game::data::DataValidationError::at(
                        fragment.id.as_str(),
                        format!("fragments[{fragment_index}].effect.ActiveSkill.{field}"),
                        cause,
                    )
                };
                if let Some(guard) = passive_guard {
                    guard
                        .validate()
                        .map_err(|cause| invalid("passive_guard".into(), cause))?;
                }
                if !forms.contains_key(BASE_FRAGMENT_FORM) || forms.len() > MAX_FRAGMENT_FORMS {
                    return Err(invalid(
                        "forms".into(),
                        "base form required and at most16 forms".into(),
                    ));
                }
                for (id, form) in forms {
                    if form.resonance_max == 0 {
                        return Err(invalid(
                            format!("forms[{id}].resonance_max"),
                            "resonance_max must be positive".into(),
                        ));
                    }
                    if id.is_empty()
                        || id.len() > 64
                        || !id
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                    {
                        return Err(invalid("forms".into(), "invalid stable form ID".into()));
                    }
                    if id == BASE_FRAGMENT_FORM && form.bloom_recipe.is_some() {
                        return Err(invalid(
                            format!("forms[{id}].bloom_recipe"),
                            "base is always unlocked".into(),
                        ));
                    }
                    if let Some(requirements) = &form.compatibility {
                        requirements.validate_static_contract().map_err(|e| {
                            invalid(
                                format!("forms[{id}].compatibility.{}", e.field_path),
                                e.cause,
                            )
                        })?;
                    }
                    if form
                        .bloom_recipe
                        .as_ref()
                        .is_some_and(|r| r.aggregated_materials().is_none())
                    {
                        return Err(invalid(
                            format!("forms[{id}].bloom_recipe"),
                            "invalid or overflowing recipe".into(),
                        ));
                    }
                }
                if dismantle_recipe
                    .as_ref()
                    .is_some_and(|r| r.aggregated_materials().is_none())
                {
                    return Err(invalid(
                        "dismantle_recipe".into(),
                        "invalid or overflowing recipe".into(),
                    ));
                }
            }

            fragment
                .compatibility
                .validate_static_contract()
                .map_err(|error| {
                    crate::game::data::DataValidationError::at(
                        fragment.id.as_str(),
                        format!(
                            "fragments[{fragment_index}].compatibility.{}",
                            error.field_path
                        ),
                        error.cause,
                    )
                })?;
        }
        Ok(())
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
    use crate::game::{
        battle::types::UnitCombatProfile,
        data::equipment_data::{
            TargetingProfile, WeaponArchetype, WeaponCombatProfile, WeaponRangeRole,
        },
    };

    #[test]
    fn metadata_preserves_sources_and_dependencies_for_future_concepts() {
        let metadata = SkillFragmentMetadata {
            corrosion_class:
                crate::game::data::skill_fragment_data::FragmentCorrosionClass::Ordinary,
            id: SkillFragmentId::from("freischutz_fragment"),
            uuid: Uuid::from_u128(10),
            name: "Freischutz Fragment".to_string(),
            description: "desc".to_string(),
            rarity: SkillFragmentRarity::Rare,
            equip_limit: SkillFragmentEquipLimit::OwnedCopies,
            origin: Some(SkillFragmentOrigin::Abnormality {
                abnormality_id: "t-02-43_freischutz".to_string(),
            }),
            sources: vec![SkillFragmentAcquisitionSource::AbnormalityContainment {
                abnormality_id: "t-02-43_freischutz".to_string(),
            }],
            dependencies: vec![SkillFragmentDependency::SourceAbnormality {
                abnormality_id: "t-02-43_freischutz".to_string(),
            }],
            compatibility: SkillFragmentCompatibilityRequirements::default(),
            effect: SkillFragmentEffectDef::ActiveSkill {
                forms: std::collections::BTreeMap::from([(
                    "base".into(),
                    crate::game::data::skill_fragment_data::SkillFragmentForm {
                        resonance_max: 100,
                        skill_id: SkillId::from("fragment_freischutz_black_round"),
                        compatibility: None,
                        bloom_recipe: None,
                    },
                )]),
                passive_guard: None,
                dismantle_recipe: None,
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

    #[test]
    fn compatibility_requirements_accept_matching_weapon_profile() {
        let mut profile = UnitCombatProfile::employee_default();
        profile
            .apply_weapon_profile(&WeaponCombatProfile {
                range_role: WeaponRangeRole::Ranged,
                weapon_archetype: WeaponArchetype::Gun,
                block_capacity: 1,
                damage_type: DamageType::Magic,
                targeting_profile: TargetingProfile::AirFirst,
                air_capable: true,
                ..WeaponCombatProfile::default()
            })
            .unwrap();

        let requirements = SkillFragmentCompatibilityRequirements {
            allowed_range_roles: vec![WeaponRangeRole::Ranged],
            allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
            allowed_damage_types: vec![DamageType::Magic],
            allowed_targeting_profiles: vec![TargetingProfile::AirFirst],
            requires_air_capable: Some(true),
            block_capacity_min: Some(1),
            required_capability_tags: vec![],
            incompatible_capability_tags: vec![],
        };

        let report = requirements.evaluate(SkillFragmentCompatibilityContext::from_combat_profile(
            &profile,
        ));

        assert!(report.is_compatible);
        assert!(report.failure_codes.is_empty());
    }

    #[test]
    fn compatibility_requirements_report_stable_failure_codes() {
        let profile = UnitCombatProfile::employee_default();
        let requirements = SkillFragmentCompatibilityRequirements {
            allowed_range_roles: vec![WeaponRangeRole::Ranged],
            allowed_weapon_archetypes: vec![WeaponArchetype::Gun],
            allowed_damage_types: vec![DamageType::Magic],
            allowed_targeting_profiles: vec![TargetingProfile::AirFirst],
            requires_air_capable: Some(true),
            block_capacity_min: Some(2),
            required_capability_tags: vec!["charge".to_string()],
            incompatible_capability_tags: vec![],
        };

        let report = requirements.evaluate(SkillFragmentCompatibilityContext::from_combat_profile(
            &profile,
        ));

        assert!(!report.is_compatible);
        assert_eq!(
            report.failure_codes,
            vec![
                SkillFragmentCompatibilityFailureCode::WeaponRequired,
                SkillFragmentCompatibilityFailureCode::BlockCapacityTooLow,
                SkillFragmentCompatibilityFailureCode::MissingCapabilityTag,
            ]
        );
    }

    #[test]
    fn compatibility_requirements_static_validation_rejects_duplicate_axes() {
        let requirements = SkillFragmentCompatibilityRequirements {
            allowed_range_roles: vec![WeaponRangeRole::Ranged, WeaponRangeRole::Ranged],
            ..SkillFragmentCompatibilityRequirements::default()
        };

        let err = requirements.validate_static_contract().unwrap_err();

        assert_eq!(err.field_path, "allowed_range_roles");
        assert!(err.cause.contains("duplicate"));
    }

    #[test]
    fn compatibility_requirements_static_validation_rejects_empty_or_conflicting_tags() {
        let empty_tag = SkillFragmentCompatibilityRequirements {
            required_capability_tags: vec![" ".to_string()],
            ..SkillFragmentCompatibilityRequirements::default()
        };

        let err = empty_tag.validate_static_contract().unwrap_err();

        assert_eq!(err.field_path, "required_capability_tags");
        assert!(err.cause.contains("empty tag"));

        let conflicting_tags = SkillFragmentCompatibilityRequirements {
            required_capability_tags: vec!["charge".to_string()],
            incompatible_capability_tags: vec!["charge".to_string()],
            ..SkillFragmentCompatibilityRequirements::default()
        };

        let err = conflicting_tags.validate_static_contract().unwrap_err();

        assert_eq!(err.field_path, "incompatible_capability_tags");
        assert!(err.cause.contains("both required and incompatible"));
    }

    #[test]
    fn compatibility_requirements_static_validation_rejects_zero_block_minimum() {
        let requirements = SkillFragmentCompatibilityRequirements {
            block_capacity_min: Some(0),
            ..SkillFragmentCompatibilityRequirements::default()
        };

        let err = requirements.validate_static_contract().unwrap_err();

        assert_eq!(err.field_path, "block_capacity_min");
        assert!(err.cause.contains("greater than zero"));
    }
}
