use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{
    ability::AbilityActivationBinding,
    behavior::GameError,
    data::{build_string_index, build_uuid_index, once_lock_with},
    enums::RiskLevel,
    stats::TriggeredEffects,
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
pub struct EquipmentRecipeMetadata {
    pub ingredients: Vec<Uuid>,
    pub result: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentDatabase {
    pub items: Vec<EquipmentMetadata>,
    #[serde(default)]
    pub recipes: Vec<EquipmentRecipeMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
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
        let by_id = once_lock_with(build_string_index(&items, "equipment id", |item| &item.id));
        let by_uuid = once_lock_with(build_uuid_index(&items, "equipment uuid", |item| item.uuid));
        let by_recipe = once_lock_with(build_recipe_index(&recipes));

        Self {
            items,
            recipes,
            by_id,
            by_uuid,
            by_recipe,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.items, "equipment id", |item| &item.id))
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
        let _ = self.by_uuid();
        let _ = self.by_recipe();
        self.validate_recipe_references()
            .expect("equipment recipes must reference existing equipment");
    }

    pub fn get_by_id(&self, id: &str) -> Option<&EquipmentMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.items.get(index))
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
