use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::AbilityActivationBinding,
    battle::{damage::DamageType, tile_range::TileRangePattern},
    behavior::GameError,
    data::event_data::EventChoiceKeyId,
    data::{build_string_index, build_uuid_index},
    employee::EventAptitude,
    enums::RiskLevel,
    stats::{StatModifier, TriggeredEffects},
};

fn default_allow_duplicate_equip() -> bool {
    true
}

fn default_cannot_unequip_reason() -> String {
    "equipment_bound".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon,
    Armor,
    Accessory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WeaponRangeRole {
    #[default]
    Melee,
    Ranged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WeaponArchetype {
    #[default]
    Sword,
    Spear,
    Shield,
    Bow,
    Gun,
    Staff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TargetingProfile {
    #[default]
    DefaultForward,
    AirFirst,
    LowDefenseFirst,
    LowMagicResistFirst,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponCombatProfile {
    pub range_role: WeaponRangeRole,
    pub weapon_archetype: WeaponArchetype,
    pub block_capacity: u32,
    pub damage_type: DamageType,
    pub targeting_profile: TargetingProfile,
    #[serde(default)]
    pub air_capable: bool,
    pub range_units: f32,
    pub defense_tile_range: TileRangePattern,
    #[serde(default = "default_weapon_attack_interval_ms")]
    pub interval_ms: u64,
    #[serde(default = "default_weapon_attack_windup_ms")]
    pub windup_ms: u32,
    #[serde(default = "default_weapon_attack_reposition_ms")]
    pub ranged_reposition_ms: u64,
    #[serde(default = "default_weapon_attack_delivery")]
    pub delivery: crate::game::ability::DeliveryDef,
}

impl Default for WeaponCombatProfile {
    fn default() -> Self {
        Self {
            range_role: WeaponRangeRole::Melee,
            weapon_archetype: WeaponArchetype::Sword,
            block_capacity: 0,
            damage_type: DamageType::Physical,
            targeting_profile: TargetingProfile::DefaultForward,
            air_capable: false,
            range_units: 1.0,
            defense_tile_range: TileRangePattern {
                include_anchor_tile: false,
                rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
            },
            interval_ms: default_weapon_attack_interval_ms(),
            windup_ms: default_weapon_attack_windup_ms(),
            ranged_reposition_ms: default_weapon_attack_reposition_ms(),
            delivery: default_weapon_attack_delivery(),
        }
    }
}

fn default_weapon_attack_interval_ms() -> u64 {
    1500
}

fn default_weapon_attack_windup_ms() -> u32 {
    crate::game::data::abnormality_data::DEFAULT_INSTANT_BASIC_ATTACK_WINDUP_MS
}

fn default_weapon_attack_reposition_ms() -> u64 {
    1000
}

fn default_weapon_attack_delivery() -> crate::game::ability::DeliveryDef {
    crate::game::ability::DeliveryDef::Instant
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentMaterialType {
    Fragment,
    Core,
    Blueprint,
    Residue,
    Generic,
}

pub type EquipmentItem = EquipmentMetadata;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventCheckModifierDefinition {
    pub aptitude: EventAptitude,
    pub amount: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub equipment_type: EquipmentType,
    pub rarity: RiskLevel,
    pub price: u32,
    #[serde(default = "default_allow_duplicate_equip")]
    pub allow_duplicate_equip: bool,
    #[serde(default)]
    pub bound: bool,
    #[serde(default = "default_cannot_unequip_reason")]
    pub cannot_unequip_reason: String,
    /// 트리거 기반 효과 (Permanent = 상시 적용)
    #[serde(default)]
    pub triggered_effects: TriggeredEffects,
    /// 장기적으로 사용하는 proc/activation 기반 ability 연결
    #[serde(default)]
    pub ability_activations: Vec<AbilityActivationBinding>,
    #[serde(default)]
    pub weapon_profile: Option<WeaponCombatProfile>,
    #[serde(default)]
    pub event_check_modifiers: Vec<EventCheckModifierDefinition>,
    #[serde(default)]
    pub event_choice_keys: Vec<EventChoiceKeyId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentMaterialMetadata {
    pub id: String,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub material_type: EquipmentMaterialType,
    pub rarity: RiskLevel,
    #[serde(default)]
    pub equipment_type: Option<EquipmentType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentMaterialCost {
    pub material_id: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentDismantleRecipeMetadata {
    pub equipment_id: String,
    pub yields: Vec<EquipmentMaterialCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentEnhancementRecipeMetadata {
    pub equipment_id: String,
    pub max_level: u8,
    pub costs_per_level: Vec<EquipmentMaterialCost>,
    #[serde(default)]
    pub modifiers_per_level: Vec<StatModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentDatabase {
    pub items: Vec<EquipmentMetadata>,
    #[serde(default)]
    pub materials: Vec<EquipmentMaterialMetadata>,
    #[serde(default)]
    pub dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
    #[serde(default)]
    pub enhancement_recipes: Vec<EquipmentEnhancementRecipeMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_material_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_dismantle_equipment_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_enhancement_equipment_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl EquipmentDatabase {
    pub fn new(items: Vec<EquipmentMetadata>) -> Self {
        Self::with_materials(items, Vec::new())
    }

    pub fn with_materials(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
    ) -> Self {
        Self::with_materials_and_dismantles(items, materials, Vec::new())
    }

    pub fn with_materials_and_dismantles(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
    ) -> Self {
        Self::with_all(items, materials, dismantle_recipes, Vec::new())
    }

    pub fn with_all(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
        enhancement_recipes: Vec<EquipmentEnhancementRecipeMetadata>,
    ) -> Self {
        Self {
            items,
            materials,
            dismantle_recipes,
            enhancement_recipes,
            by_id: OnceLock::new(),
            by_material_id: OnceLock::new(),
            by_dismantle_equipment_id: OnceLock::new(),
            by_enhancement_equipment_id: OnceLock::new(),
            by_uuid: OnceLock::new(),
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.items, "equipment id", |item| &item.id))
    }

    fn by_material_id(&self) -> &HashMap<String, usize> {
        self.by_material_id.get_or_init(|| {
            build_string_index(&self.materials, "equipment material id", |item| &item.id)
        })
    }

    fn by_dismantle_equipment_id(&self) -> &HashMap<String, usize> {
        self.by_dismantle_equipment_id.get_or_init(|| {
            build_string_index(
                &self.dismantle_recipes,
                "equipment dismantle recipe equipment id",
                |recipe| &recipe.equipment_id,
            )
        })
    }

    fn by_enhancement_equipment_id(&self) -> &HashMap<String, usize> {
        self.by_enhancement_equipment_id.get_or_init(|| {
            build_string_index(
                &self.enhancement_recipes,
                "equipment enhancement recipe equipment id",
                |recipe| &recipe.equipment_id,
            )
        })
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid
            .get_or_init(|| build_uuid_index(&self.items, "equipment uuid", |item| item.uuid))
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .expect("invalid equipment authoring contract");
        let _ = self.by_id();
        let _ = self.by_material_id();
        let _ = self.by_dismantle_equipment_id();
        let _ = self.by_enhancement_equipment_id();
        let _ = self.by_uuid();
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        self.validate_weapon_profiles()?;
        self.validate_event_check_metadata()?;
        self.validate_dismantle_recipe_references()?;
        self.validate_enhancement_recipe_references()
    }

    fn validate_event_check_metadata(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (item_index, item) in self.items.iter().enumerate() {
            let mut seen_aptitudes = HashSet::new();
            for (modifier_index, modifier) in item.event_check_modifiers.iter().enumerate() {
                let path = format!("items[{item_index}].event_check_modifiers[{modifier_index}]");
                if !matches!(modifier.amount, 1 | 2) {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        format!("{path}.amount"),
                        "event check modifier amount must be 1 or 2",
                    ));
                }
                if !seen_aptitudes.insert(modifier.aptitude) {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        format!("{path}.aptitude"),
                        "duplicate event check modifier aptitude",
                    ));
                }
            }

            let mut seen_keys = HashSet::new();
            for (key_index, key) in item.event_choice_keys.iter().enumerate() {
                let path = format!("items[{item_index}].event_choice_keys[{key_index}]");
                if !key.is_valid() {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        path,
                        "event choice key must be a non-empty lower_snake_case identifier",
                    ));
                }
                if !seen_keys.insert(key.as_str()) {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        path,
                        "duplicate event choice key",
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_weapon_profiles(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (item_index, item) in self.items.iter().enumerate() {
            let profile_path = format!("items[{item_index}].weapon_profile");
            match (item.equipment_type, item.weapon_profile.as_ref()) {
                (EquipmentType::Weapon, Some(profile)) => {
                    profile
                        .validate_runtime_contract(&item.id)
                        .map_err(|error| {
                            let cause = match error {
                                GameError::InvalidStaticData(cause) => cause,
                                other => format!("unexpected weapon validation error: {other:?}"),
                            };
                            crate::game::data::DataValidationError::at(
                                &item.id,
                                profile_path,
                                cause,
                            )
                        })?;
                }
                (EquipmentType::Weapon, None) => {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        profile_path,
                        "weapon equipment is missing weapon_profile",
                    ));
                }
                (_, Some(_)) => {
                    return Err(crate::game::data::DataValidationError::at(
                        &item.id,
                        profile_path,
                        "non-weapon equipment must not define weapon_profile",
                    ));
                }
                (_, None) => {}
            }
        }
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Option<&EquipmentMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.items.get(index))
    }

    pub fn get_material_by_id(&self, id: &str) -> Option<&EquipmentMaterialMetadata> {
        self.by_material_id()
            .get(id)
            .and_then(|&index| self.materials.get(index))
    }

    pub fn get_dismantle_recipe_by_equipment_id(
        &self,
        equipment_id: &str,
    ) -> Option<&EquipmentDismantleRecipeMetadata> {
        self.by_dismantle_equipment_id()
            .get(equipment_id)
            .and_then(|&index| self.dismantle_recipes.get(index))
    }

    pub fn get_enhancement_recipe_by_equipment_id(
        &self,
        equipment_id: &str,
    ) -> Option<&EquipmentEnhancementRecipeMetadata> {
        self.by_enhancement_equipment_id()
            .get(equipment_id)
            .and_then(|&index| self.enhancement_recipes.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&EquipmentMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.items.get(index))
    }

    fn validate_dismantle_recipe_references(
        &self,
    ) -> Result<(), crate::game::data::DataValidationError> {
        let mut equipment_ids = HashSet::new();
        for (recipe_index, recipe) in self.dismantle_recipes.iter().enumerate() {
            let recipe_path = format!("dismantle_recipes[{recipe_index}]");
            if !equipment_ids.insert(recipe.equipment_id.as_str()) {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.equipment_id"),
                    "duplicate dismantle recipe equipment_id",
                ));
            }
            if !self.items.iter().any(|item| item.id == recipe.equipment_id) {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.equipment_id"),
                    "references missing equipment",
                ));
            }

            if recipe.yields.is_empty() {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.yields"),
                    "must have at least one yield",
                ));
            }

            for (yield_index, material) in recipe.yields.iter().enumerate() {
                let yield_path = format!("{recipe_path}.yields[{yield_index}]");
                if material.amount == 0 {
                    return Err(crate::game::data::DataValidationError::at(
                        &recipe.equipment_id,
                        format!("{yield_path}.amount"),
                        "yield amount must be greater than zero",
                    ));
                }
                if !self
                    .materials
                    .iter()
                    .any(|candidate| candidate.id == material.material_id)
                {
                    return Err(crate::game::data::DataValidationError::at(
                        &recipe.equipment_id,
                        format!("{yield_path}.material_id"),
                        format!("references missing material '{}'", material.material_id),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_enhancement_recipe_references(
        &self,
    ) -> Result<(), crate::game::data::DataValidationError> {
        let mut equipment_ids = HashSet::new();
        for (recipe_index, recipe) in self.enhancement_recipes.iter().enumerate() {
            let recipe_path = format!("enhancement_recipes[{recipe_index}]");
            if !equipment_ids.insert(recipe.equipment_id.as_str()) {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.equipment_id"),
                    "duplicate enhancement recipe equipment_id",
                ));
            }
            if !self.items.iter().any(|item| item.id == recipe.equipment_id) {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.equipment_id"),
                    "references missing equipment",
                ));
            }
            if recipe.max_level == 0 {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.max_level"),
                    "max_level must be greater than zero",
                ));
            }
            if recipe.costs_per_level.is_empty() {
                return Err(crate::game::data::DataValidationError::at(
                    &recipe.equipment_id,
                    format!("{recipe_path}.costs_per_level"),
                    "must have at least one cost",
                ));
            }
            for (cost_index, cost) in recipe.costs_per_level.iter().enumerate() {
                let cost_path = format!("{recipe_path}.costs_per_level[{cost_index}]");
                if cost.amount == 0 {
                    return Err(crate::game::data::DataValidationError::at(
                        &recipe.equipment_id,
                        format!("{cost_path}.amount"),
                        "cost amount must be greater than zero",
                    ));
                }
                if !self
                    .materials
                    .iter()
                    .any(|candidate| candidate.id == cost.material_id)
                {
                    return Err(crate::game::data::DataValidationError::at(
                        &recipe.equipment_id,
                        format!("{cost_path}.material_id"),
                        format!("references missing material '{}'", cost.material_id),
                    ));
                }
            }
        }

        Ok(())
    }
}

impl WeaponCombatProfile {
    pub(crate) fn validate_runtime_contract(&self, owner_label: &str) -> Result<(), GameError> {
        if self.range_units <= 0.0 {
            return Err(GameError::InvalidStaticData(format!(
                "weapon '{}' weapon_profile range_units must be > 0",
                owner_label
            )));
        }
        if self.interval_ms == 0 {
            return Err(GameError::InvalidStaticData(format!(
                "weapon '{}' weapon_profile interval_ms must be > 0",
                owner_label
            )));
        }
        self.defense_tile_range.validate().map_err(|error| {
            GameError::InvalidStaticData(format!(
                "weapon '{}' weapon_profile has invalid defense_tile_range: {}",
                owner_label, error
            ))
        })?;
        if matches!(
            self.delivery,
            crate::game::ability::DeliveryDef::TileArea { .. }
        ) {
            return Err(GameError::InvalidStaticData(format!(
                "weapon '{}' weapon_profile delivery must be Instant or Projectile",
                owner_label
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{EquipmentDatabase, WeaponArchetype};
    use ron::de::from_str;

    #[test]
    fn removed_combination_recipes_field_is_rejected() {
        let error = from_str::<EquipmentDatabase>("(items: [], recipes: [])")
            .expect_err("removed equipment combination recipe schema must fail closed");

        assert!(error.to_string().contains("recipes"));
    }

    #[test]
    fn removed_grenade_launcher_weapon_archetype_is_rejected() {
        let error = from_str::<WeaponArchetype>("GrenadeLauncher")
            .expect_err("removed grenade launcher schema must fail closed");

        assert!(error.to_string().contains("GrenadeLauncher"));
    }

    fn event_metadata_database(metadata: &str) -> EquipmentDatabase {
        from_str::<EquipmentDatabase>(&format!(
            r#"EquipmentDatabase(items: [(
                id: "event_tool",
                uuid: "00000000-0000-0000-0000-000000000001",
                name: "Event Tool",
                equipment_type: Accessory,
                rarity: ZAYIN,
                price: 1,
                {metadata}
            )])"#
        ))
        .expect("event equipment fixture should deserialize")
    }

    #[test]
    fn event_check_modifier_accepts_only_one_or_two() {
        let database =
            event_metadata_database("event_check_modifiers: [(aptitude: Analysis, amount: 3)],");

        let error = database
            .validate_contract()
            .expect_err("out-of-policy modifier must fail closed");
        assert!(error.cause.contains("amount must be 1 or 2"));
    }

    #[test]
    fn duplicate_event_modifier_aptitude_is_rejected() {
        let database = event_metadata_database(
            "event_check_modifiers: [(aptitude: Analysis, amount: 1), (aptitude: Analysis, amount: 2)],",
        );

        let error = database
            .validate_contract()
            .expect_err("duplicate aptitude must fail closed");
        assert!(error
            .cause
            .contains("duplicate event check modifier aptitude"));
    }

    #[test]
    fn duplicate_or_invalid_event_choice_keys_are_rejected() {
        let duplicate =
            event_metadata_database("event_choice_keys: [\"sealed_archive\", \"sealed_archive\"],");
        let duplicate_error = duplicate
            .validate_contract()
            .expect_err("duplicate key must fail closed");
        assert!(duplicate_error.cause.contains("duplicate event choice key"));

        let invalid = event_metadata_database("event_choice_keys: [\"Sealed Archive\"],");
        let invalid_error = invalid
            .validate_contract()
            .expect_err("invalid key must fail closed");
        assert!(invalid_error.cause.contains("lower_snake_case"));
    }
}
