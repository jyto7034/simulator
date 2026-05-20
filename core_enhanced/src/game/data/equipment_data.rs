use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::AbilityActivationBinding,
    behavior::GameError,
    data::{build_string_index, build_uuid_index, once_lock_with},
    enums::RiskLevel,
    stats::{StatModifier, TriggeredEffects},
};

fn default_allow_duplicate_equip() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon,
    Armor,
    Accessory,
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
    /// 트리거 기반 효과 (Permanent = 상시 적용)
    #[serde(default)]
    pub triggered_effects: TriggeredEffects,
    /// 장기적으로 사용하는 proc/activation 기반 ability 연결
    #[serde(default)]
    pub ability_activations: Vec<AbilityActivationBinding>,
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
pub struct EquipmentRestorationRecipeMetadata {
    pub id: String,
    pub result_equipment_id: String,
    pub costs: Vec<EquipmentMaterialCost>,
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
    pub restoration_recipes: Vec<EquipmentRestorationRecipeMetadata>,
    #[serde(default)]
    pub dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
    #[serde(default)]
    pub enhancement_recipes: Vec<EquipmentEnhancementRecipeMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_material_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_restoration_recipe_id: OnceLock<HashMap<String, usize>>,
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
        Self::with_materials_recipes_and_restorations(items, materials, recipes, Vec::new())
    }

    pub fn with_materials_recipes_and_restorations(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        recipes: Vec<EquipmentRecipeMetadata>,
        restoration_recipes: Vec<EquipmentRestorationRecipeMetadata>,
    ) -> Self {
        Self::with_materials_recipes_restorations_and_dismantles(
            items,
            materials,
            recipes,
            restoration_recipes,
            Vec::new(),
        )
    }

    pub fn with_materials_recipes_restorations_and_dismantles(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        recipes: Vec<EquipmentRecipeMetadata>,
        restoration_recipes: Vec<EquipmentRestorationRecipeMetadata>,
        dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
    ) -> Self {
        Self::with_all(
            items,
            materials,
            recipes,
            restoration_recipes,
            dismantle_recipes,
            Vec::new(),
        )
    }

    pub fn with_all(
        items: Vec<EquipmentMetadata>,
        materials: Vec<EquipmentMaterialMetadata>,
        recipes: Vec<EquipmentRecipeMetadata>,
        restoration_recipes: Vec<EquipmentRestorationRecipeMetadata>,
        dismantle_recipes: Vec<EquipmentDismantleRecipeMetadata>,
        enhancement_recipes: Vec<EquipmentEnhancementRecipeMetadata>,
    ) -> Self {
        let by_id = once_lock_with(build_string_index(&items, "equipment id", |item| &item.id));
        let by_material_id = once_lock_with(build_string_index(
            &materials,
            "equipment material id",
            |item| &item.id,
        ));
        let by_restoration_recipe_id = once_lock_with(build_string_index(
            &restoration_recipes,
            "equipment restoration recipe id",
            |recipe| &recipe.id,
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
            restoration_recipes,
            dismantle_recipes,
            enhancement_recipes,
            by_id,
            by_material_id,
            by_restoration_recipe_id,
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

    fn by_restoration_recipe_id(&self) -> &HashMap<String, usize> {
        self.by_restoration_recipe_id.get_or_init(|| {
            build_string_index(
                &self.restoration_recipes,
                "equipment restoration recipe id",
                |recipe| &recipe.id,
            )
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
        let _ = self.by_restoration_recipe_id();
        let _ = self.by_dismantle_equipment_id();
        let _ = self.by_enhancement_equipment_id();
        let _ = self.by_uuid();
        let _ = self.by_recipe();
        self.validate_recipe_references()
            .expect("equipment recipes must reference existing equipment");
        self.validate_restoration_recipe_references()
            .expect("equipment restoration recipes must reference existing data");
        self.validate_dismantle_recipe_references()
            .expect("equipment dismantle recipes must reference existing data");
        self.validate_enhancement_recipe_references()
            .expect("equipment enhancement recipes must reference existing data");
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

    pub fn get_restoration_recipe_by_id(
        &self,
        id: &str,
    ) -> Option<&EquipmentRestorationRecipeMetadata> {
        self.by_restoration_recipe_id()
            .get(id)
            .and_then(|&index| self.restoration_recipes.get(index))
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

    fn validate_restoration_recipe_references(&self) -> Result<(), GameError> {
        for recipe in &self.restoration_recipes {
            if self.get_by_id(&recipe.result_equipment_id).is_none() {
                return Err(GameError::InvalidStaticData(format!(
                    "equipment restoration recipe '{}' references missing result equipment '{}'",
                    recipe.id, recipe.result_equipment_id
                )));
            }

            if recipe.costs.is_empty() {
                return Err(GameError::InvalidStaticData(format!(
                    "equipment restoration recipe '{}' must have at least one material cost",
                    recipe.id
                )));
            }

            for cost in &recipe.costs {
                if cost.amount == 0 {
                    return Err(GameError::InvalidStaticData(format!(
                        "equipment restoration recipe '{}' has zero cost for material '{}'",
                        recipe.id, cost.material_id
                    )));
                }
                if self.get_material_by_id(&cost.material_id).is_none() {
                    return Err(GameError::InvalidStaticData(format!(
                        "equipment restoration recipe '{}' references missing material '{}'",
                        recipe.id, cost.material_id
                    )));
                }
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
            triggered_effects: TriggeredEffects::default(),
            ability_activations: vec![],
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
