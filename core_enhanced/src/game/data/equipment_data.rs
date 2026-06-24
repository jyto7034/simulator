use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::AbilityActivationBinding,
    battle::{damage::DamageType, tile_range::TileRangePattern},
    behavior::GameError,
    data::{build_string_index, build_uuid_index, once_lock_with},
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
    GrenadeLauncher,
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
pub struct WeaponCombatProfile {
    pub range_role: WeaponRangeRole,
    pub weapon_archetype: WeaponArchetype,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct EquipmentRecipeMetadata {
    pub ingredients: Vec<Uuid>,
    pub result: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentMaterialCost {
    pub material_id: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentDismantleRecipeMetadata {
    pub equipment_id: String,
    pub yields: Vec<EquipmentMaterialCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentEnhancementRecipeMetadata {
    pub equipment_id: String,
    pub max_level: u8,
    pub costs_per_level: Vec<EquipmentMaterialCost>,
    #[serde(default)]
    pub modifiers_per_level: Vec<StatModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentDatabase {
    pub items: Vec<EquipmentMetadata>,
    #[serde(default)]
    pub materials: Vec<EquipmentMaterialMetadata>,
    #[serde(default)]
    pub recipes: Vec<EquipmentRecipeMetadata>,
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
    #[serde(skip)]
    by_recipe: OnceLock<HashMap<RecipeKey, Uuid>>,
}

impl EquipmentDatabase {
    pub fn new(items: Vec<EquipmentMetadata>) -> Self {
        Self::with_recipes(items, Vec::new())
    }

    pub fn with_recipes(
        items: Vec<EquipmentMetadata>,
        recipes: Vec<EquipmentRecipeMetadata>,
    ) -> Self {
        Self::with_materials_and_recipes(items, Vec::new(), recipes)
    }

    pub fn with_materials_and_recipes(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        recipes: Vec<EquipmentRecipeMetadata>,
    ) -> Self {
        Self::with_materials_recipes_and_dismantles(items, materials, recipes, Vec::new())
    }

    pub fn with_materials_recipes_and_dismantles(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        recipes: Vec<EquipmentRecipeMetadata>,
        dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
    ) -> Self {
        Self::with_all(items, materials, recipes, dismantle_recipes, Vec::new())
    }

    pub fn with_all(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        recipes: Vec<EquipmentRecipeMetadata>,
        dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
        enhancement_recipes: Vec<EquipmentEnhancementRecipeMetadata>,
    ) -> Self {
        let by_id = once_lock_with(build_string_index(&items, "equipment id", |item| &item.id));
        let by_material_id = once_lock_with(build_string_index(
            &materials,
            "equipment material id",
            |item| &item.id,
        ));
        let by_dismantle_equipment_id = once_lock_with(build_string_index(
            &dismantle_recipes,
            "equipment dismantle recipe equipment id",
            |recipe| &recipe.equipment_id,
        ));
        let by_enhancement_equipment_id = once_lock_with(build_string_index(
            &enhancement_recipes,
            "equipment enhancement recipe equipment id",
            |recipe| &recipe.equipment_id,
        ));
        let by_uuid = once_lock_with(build_uuid_index(&items, "equipment uuid", |item| item.uuid));
        let by_recipe = once_lock_with(build_recipe_index(&recipes));

        Self {
            items,
            materials,
            recipes,
            dismantle_recipes,
            enhancement_recipes,
            by_id,
            by_material_id,
            by_dismantle_equipment_id,
            by_enhancement_equipment_id,
            by_uuid,
            by_recipe,
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

    fn by_recipe(&self) -> &HashMap<RecipeKey, Uuid> {
        self.by_recipe
            .get_or_init(|| build_recipe_index(&self.recipes))
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_material_id();
        let _ = self.by_dismantle_equipment_id();
        let _ = self.by_enhancement_equipment_id();
        let _ = self.by_uuid();
        let _ = self.by_recipe();
        self.validate_weapon_profiles()
            .expect("equipment weapon profiles must be valid");
        self.validate_recipe_references()
            .expect("equipment recipes must reference existing equipment");
        self.validate_dismantle_recipe_references()
            .expect("equipment dismantle recipes must reference existing data");
        self.validate_enhancement_recipe_references()
            .expect("equipment enhancement recipes must reference existing data");
    }

    fn validate_weapon_profiles(&self) -> Result<(), GameError> {
        for item in &self.items {
            match (item.equipment_type, item.weapon_profile.as_ref()) {
                (EquipmentType::Weapon, Some(profile)) => {
                    profile.validate_runtime_contract(&item.id)?;
                }
                (EquipmentType::Weapon, None) => {
                    return Err(GameError::InvalidStaticData(format!(
                        "weapon equipment '{}' is missing weapon_profile",
                        item.id
                    )));
                }
                (_, Some(_)) => {
                    return Err(GameError::InvalidStaticData(format!(
                        "non-weapon equipment '{}' must not define weapon_profile",
                        item.id
                    )));
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

    pub fn combination_result(&self, left: Uuid, right: Uuid) -> Option<&EquipmentMetadata> {
        self.by_recipe()
            .get(&RecipeKey::new(left, right))
            .and_then(|result_uuid| self.get_by_uuid(result_uuid))
    }

    fn validate_recipe_references(&self) -> Result<(), GameError> {
        for recipe in &self.recipes {
            if recipe.ingredients.len() != 2 {
                return Err(GameError::InvalidStaticData(format!(
                    "equipment recipe for result {} must have exactly two ingredients",
                    recipe.result
                )));
            }

            for ingredient in &recipe.ingredients {
                if self.get_by_uuid(ingredient).is_none() {
                    return Err(GameError::InvalidStaticData(format!(
                        "equipment recipe references missing ingredient uuid {}",
                        ingredient
                    )));
                }
            }

            if self.get_by_uuid(&recipe.result).is_none() {
                return Err(GameError::InvalidStaticData(format!(
                    "equipment recipe references missing result uuid {}",
                    recipe.result
                )));
            }
        }

        Ok(())
    }

    fn validate_dismantle_recipe_references(&self) -> Result<(), GameError> {
        for recipe in &self.dismantle_recipes {
            if self.get_by_id(&recipe.equipment_id).is_none() {
                return Err(GameError::InvalidStaticData(format!(
                    "equipment dismantle recipe references missing equipment '{}'",
                    recipe.equipment_id
                )));
            }

            if recipe.yields.is_empty() {
                return Err(GameError::InvalidStaticData(format!(
                    "equipment dismantle recipe for '{}' must have at least one yield",
                    recipe.equipment_id
                )));
            }

            for material in &recipe.yields {
                if material.amount == 0 {
                    return Err(GameError::InvalidStaticData(format!(
                        "equipment dismantle recipe for '{}' has zero yield for material '{}'",
                        recipe.equipment_id, material.material_id
                    )));
                }
                if self.get_material_by_id(&material.material_id).is_none() {
                    return Err(GameError::InvalidStaticData(format!(
                        "equipment dismantle recipe for '{}' references missing material '{}'",
                        recipe.equipment_id, material.material_id
                    )));
                }
            }
        }

        Ok(())
    }

    fn validate_enhancement_recipe_references(&self) -> Result<(), GameError> {
        for recipe in &self.enhancement_recipes {
            if self.get_by_id(&recipe.equipment_id).is_none() {
                return Err(GameError::InvalidStaticData(format!(
                    "equipment enhancement recipe references missing equipment '{}'",
                    recipe.equipment_id
                )));
            }
            if recipe.max_level == 0 {
                return Err(GameError::InvalidStaticData(format!(
                    "equipment enhancement recipe for '{}' must have max_level > 0",
                    recipe.equipment_id
                )));
            }
            if recipe.costs_per_level.is_empty() {
                return Err(GameError::InvalidStaticData(format!(
                    "equipment enhancement recipe for '{}' must have at least one cost",
                    recipe.equipment_id
                )));
            }
            for cost in &recipe.costs_per_level {
                if cost.amount == 0 {
                    return Err(GameError::InvalidStaticData(format!(
                        "equipment enhancement recipe for '{}' has zero cost for material '{}'",
                        recipe.equipment_id, cost.material_id
                    )));
                }
                if self.get_material_by_id(&cost.material_id).is_none() {
                    return Err(GameError::InvalidStaticData(format!(
                        "equipment enhancement recipe for '{}' references missing material '{}'",
                        recipe.equipment_id, cost.material_id
                    )));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RecipeKey {
    first: Uuid,
    second: Uuid,
}

impl RecipeKey {
    fn new(left: Uuid, right: Uuid) -> Self {
        if left.as_u128() <= right.as_u128() {
            Self {
                first: left,
                second: right,
            }
        } else {
            Self {
                first: right,
                second: left,
            }
        }
    }
}

fn build_recipe_index(recipes: &[EquipmentRecipeMetadata]) -> HashMap<RecipeKey, Uuid> {
    let mut index = HashMap::with_capacity(recipes.len());

    for recipe in recipes {
        if recipe.ingredients.len() != 2 {
            panic!(
                "equipment recipe for result {} must have exactly two ingredients",
                recipe.result
            );
        }
        let key = RecipeKey::new(recipe.ingredients[0], recipe.ingredients[1]);
        if let Some(previous_result) = index.insert(key, recipe.result) {
            panic!(
                "duplicate equipment recipe for ingredients {} + {}: {} and {}",
                key.first, key.second, previous_result, recipe.result
            );
        }
    }

    index
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::stats::TriggeredEffects;

    fn equipment(uuid: u128, id: &str, equipment_type: EquipmentType) -> EquipmentMetadata {
        EquipmentMetadata {
            id: id.to_string(),
            uuid: Uuid::from_u128(uuid),
            name: id.to_string(),
            equipment_type,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: TriggeredEffects::default(),
            ability_activations: vec![],
            weapon_profile: (equipment_type == EquipmentType::Weapon)
                .then(WeaponCombatProfile::default),
        }
    }

    #[test]
    fn recipes_are_order_independent() {
        let a = equipment(1, "a", EquipmentType::Weapon);
        let b = equipment(2, "b", EquipmentType::Armor);
        let result = equipment(3, "result", EquipmentType::Accessory);
        let db = EquipmentDatabase::with_recipes(
            vec![a, b, result],
            vec![EquipmentRecipeMetadata {
                ingredients: vec![Uuid::from_u128(1), Uuid::from_u128(2)],
                result: Uuid::from_u128(3),
            }],
        );

        assert_eq!(
            db.combination_result(Uuid::from_u128(1), Uuid::from_u128(2))
                .map(|item| item.id.as_str()),
            Some("result")
        );
        assert_eq!(
            db.combination_result(Uuid::from_u128(2), Uuid::from_u128(1))
                .map(|item| item.id.as_str()),
            Some("result")
        );
    }
}
