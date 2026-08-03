use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::game::data::{
    build_string_index, build_uuid_index, once_lock_with, DataValidationError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShopContext {
    HeadquartersSupport,
    UnknownExploration,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShopMerchandise {
    Equipment { equipment_id: String },
    Consumable { consumable_id: String },
    Monolith { abnormality_id: String },
}

impl ShopMerchandise {
    pub fn definition_id(&self) -> &str {
        match self {
            Self::Equipment { equipment_id } => equipment_id,
            Self::Consumable { consumable_id } => consumable_id,
            Self::Monolith { abnormality_id } => abnormality_id,
        }
    }

    pub fn identity_key(&self) -> String {
        match self {
            Self::Equipment { equipment_id } => format!("equipment:{equipment_id}"),
            Self::Consumable { consumable_id } => format!("consumable:{consumable_id}"),
            Self::Monolith { abnormality_id } => format!("monolith:{abnormality_id}"),
        }
    }

    pub fn is_equipment(&self) -> bool {
        matches!(self, Self::Equipment { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopMerchandiseEntry {
    pub merchandise: ShopMerchandise,
    pub quantity: u32,
    pub weight: u32,
    #[serde(default)]
    pub repeatable: bool,
    pub buy_percent: Option<u32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopMerchandisePreset {
    pub id: String,
    pub entries: Vec<ShopMerchandiseEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopPresetReference {
    pub preset_id: String,
    pub weight_multiplier: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopRefreshPolicy {
    pub base_cost: u32,
    pub max_uses: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopSellPolicy {
    pub price_percent: u32,
    pub accepted_tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopMetadata {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub context: ShopContext,
    pub offer_count: usize,
    pub default_buy_percent: u32,
    pub preset_refs: Vec<ShopPresetReference>,
    pub local_entries: Vec<ShopMerchandiseEntry>,
    pub refresh_policy: Option<ShopRefreshPolicy>,
    pub sell_policy: Option<ShopSellPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopPoolMetadata {
    pub id: String,
    pub shop_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopDatabase {
    pub merchandise_presets: Vec<ShopMerchandisePreset>,
    pub shops: Vec<ShopMetadata>,
    pub pools: Vec<ShopPoolMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

impl ShopDatabase {
    pub fn new(shops: Vec<ShopMetadata>) -> Self {
        Self::new_with_catalog(shops, vec![], vec![])
    }

    pub fn new_with_pools(shops: Vec<ShopMetadata>, pools: Vec<ShopPoolMetadata>) -> Self {
        Self::new_with_catalog(shops, pools, vec![])
    }

    pub fn new_with_catalog(
        shops: Vec<ShopMetadata>,
        pools: Vec<ShopPoolMetadata>,
        merchandise_presets: Vec<ShopMerchandisePreset>,
    ) -> Self {
        info!("Shop: {:?}", shops);
        let by_id = once_lock_with(build_string_index(&shops, "shop id", |item| &item.id));
        let by_uuid = once_lock_with(build_uuid_index(&shops, "shop uuid", |item| item.uuid));
        Self {
            merchandise_presets,
            shops,
            pools,
            by_id,
            by_uuid,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.shops, "shop id", |item| &item.id))
    }

    fn by_uuid(&self) -> &HashMap<Uuid, usize> {
        self.by_uuid
            .get_or_init(|| build_uuid_index(&self.shops, "shop uuid", |item| item.uuid))
    }

    pub fn get_by_id(&self, id: &str) -> Option<&ShopMetadata> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.shops.get(index))
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&ShopMetadata> {
        self.by_uuid()
            .get(uuid)
            .and_then(|&index| self.shops.get(index))
    }

    pub fn pool_by_id(&self, id: &str) -> Option<&ShopPoolMetadata> {
        self.pools.iter().find(|pool| pool.id == id)
    }

    pub fn preset_by_id(&self, id: &str) -> Option<&ShopMerchandisePreset> {
        self.merchandise_presets
            .iter()
            .find(|preset| preset.id == id)
    }

    pub fn flattened_entries(
        &self,
        shop: &ShopMetadata,
    ) -> Result<Vec<ShopMerchandiseEntry>, String> {
        let mut entries = Vec::new();
        let mut identities = HashSet::new();
        for reference in &shop.preset_refs {
            let preset = self.preset_by_id(&reference.preset_id).ok_or_else(|| {
                format!(
                    "shop '{}' references missing merchandise preset '{}'",
                    shop.id, reference.preset_id
                )
            })?;
            for preset_entry in &preset.entries {
                let mut entry = preset_entry.clone();
                entry.weight = entry
                    .weight
                    .checked_mul(reference.weight_multiplier)
                    .ok_or_else(|| {
                        format!(
                            "shop '{}' weight overflows for merchandise '{}'",
                            shop.id,
                            entry.merchandise.identity_key()
                        )
                    })?;
                let identity = entry.merchandise.identity_key();
                if !identities.insert(identity.clone()) {
                    return Err(format!(
                        "shop '{}' composes duplicate merchandise '{}'",
                        shop.id, identity
                    ));
                }
                entries.push(entry);
            }
        }
        for entry in &shop.local_entries {
            let identity = entry.merchandise.identity_key();
            if !identities.insert(identity.clone()) {
                return Err(format!(
                    "shop '{}' composes duplicate merchandise '{}'",
                    shop.id, identity
                ));
            }
            entries.push(entry.clone());
        }
        entries.sort_by_key(|entry| entry.merchandise.identity_key());
        Ok(entries)
    }

    pub fn merchandise_tags(&self, merchandise: &ShopMerchandise) -> HashSet<&str> {
        self.merchandise_presets
            .iter()
            .flat_map(|preset| &preset.entries)
            .chain(self.shops.iter().flat_map(|shop| &shop.local_entries))
            .filter(|entry| entry.merchandise == *merchandise)
            .flat_map(|entry| entry.tags.iter().map(String::as_str))
            .collect()
    }

    pub(crate) fn validate_contract(&self) -> Result<(), DataValidationError> {
        let mut preset_ids = HashSet::new();
        for (preset_index, preset) in self.merchandise_presets.iter().enumerate() {
            if preset.id.trim().is_empty() || !preset_ids.insert(preset.id.as_str()) {
                return Err(DataValidationError::at(
                    &preset.id,
                    format!("merchandise_presets[{preset_index}].id"),
                    "Shop merchandise preset id must be nonempty and unique",
                ));
            }
            validate_entries(&preset.id, "entries", &preset.entries)?;
        }

        let mut shop_ids = HashSet::new();
        let mut shop_uuids = HashSet::new();
        for (shop_index, shop) in self.shops.iter().enumerate() {
            if shop.id.trim().is_empty() || shop.name.trim().is_empty() {
                return Err(DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}]"),
                    "Shop id and name must not be empty",
                ));
            }
            if !shop_ids.insert(shop.id.as_str()) || !shop_uuids.insert(shop.uuid) {
                return Err(DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}]"),
                    "Shop id and UUID must be unique",
                ));
            }
            if shop.offer_count == 0 || shop.default_buy_percent == 0 {
                return Err(DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}].offer_count"),
                    "Shop offer_count and default_buy_percent must be positive",
                ));
            }
            let mut referenced = HashSet::new();
            for (reference_index, reference) in shop.preset_refs.iter().enumerate() {
                if reference.weight_multiplier == 0 {
                    return Err(DataValidationError::at(
                        &shop.id,
                        format!("shops[{shop_index}].preset_refs[{reference_index}]"),
                        "Shop preset weight_multiplier must be positive",
                    ));
                }
                if !referenced.insert(reference.preset_id.as_str()) {
                    return Err(DataValidationError::at(
                        &shop.id,
                        format!("shops[{shop_index}].preset_refs[{reference_index}]"),
                        format!("duplicates preset reference '{}'", reference.preset_id),
                    ));
                }
                if self.preset_by_id(&reference.preset_id).is_none() {
                    return Err(DataValidationError::at(
                        &shop.id,
                        format!("shops[{shop_index}].preset_refs[{reference_index}].preset_id"),
                        format!("references missing preset '{}'", reference.preset_id),
                    ));
                }
            }
            validate_entries(&shop.id, "local_entries", &shop.local_entries)?;
            let flattened = self.flattened_entries(shop).map_err(|cause| {
                DataValidationError::at(&shop.id, format!("shops[{shop_index}]"), cause)
            })?;
            if flattened.len() < shop.offer_count {
                return Err(DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}].offer_count"),
                    format!(
                        "requires {} unique merchandise but only {} are composed",
                        shop.offer_count,
                        flattened.len()
                    ),
                ));
            }
            if shop
                .refresh_policy
                .is_some_and(|policy| policy.max_uses == 0)
            {
                return Err(DataValidationError::at(
                    &shop.id,
                    format!("shops[{shop_index}].refresh_policy.max_uses"),
                    "Shop refresh max_uses must be positive when refresh is enabled",
                ));
            }
            if let Some(sell_policy) = &shop.sell_policy {
                if sell_policy.price_percent == 0 || sell_policy.price_percent > 100 {
                    return Err(DataValidationError::at(
                        &shop.id,
                        format!("shops[{shop_index}].sell_policy.price_percent"),
                        "Shop sell price_percent must be between 1 and 100",
                    ));
                }
                if let Some(tags) = &sell_policy.accepted_tags {
                    validate_tags(&shop.id, "sell_policy.accepted_tags", tags)?;
                }
            }
        }

        let mut pool_ids = HashSet::new();
        for (pool_index, pool) in self.pools.iter().enumerate() {
            if pool.id.trim().is_empty() || !pool_ids.insert(pool.id.as_str()) {
                return Err(DataValidationError::at(
                    &pool.id,
                    format!("pools[{pool_index}].id"),
                    "Shop pool id must be nonempty and unique",
                ));
            }
            if pool.shop_ids.is_empty() {
                return Err(DataValidationError::at(
                    &pool.id,
                    format!("pools[{pool_index}].shop_ids"),
                    "Shop pool must reference at least one Shop",
                ));
            }
            let mut shop_ids = HashSet::new();
            for (shop_index, shop_id) in pool.shop_ids.iter().enumerate() {
                if !shop_ids.insert(shop_id.as_str()) {
                    return Err(DataValidationError::at(
                        &pool.id,
                        format!("pools[{pool_index}].shop_ids[{shop_index}]"),
                        format!("duplicates Shop '{shop_id}'"),
                    ));
                }
                if self.get_by_id(shop_id).is_none() {
                    return Err(DataValidationError::at(
                        &pool.id,
                        format!("pools[{pool_index}].shop_ids[{shop_index}]"),
                        format!("references missing Shop '{shop_id}'"),
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .unwrap_or_else(|error| panic!("invalid Shop data: {error:?}"));
        let _ = self.by_id();
        let _ = self.by_uuid();
    }
}

fn validate_entries(
    owner_id: &str,
    field: &str,
    entries: &[ShopMerchandiseEntry],
) -> Result<(), DataValidationError> {
    let mut identities = HashSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let identity = entry.merchandise.identity_key();
        if entry.quantity == 0 || entry.weight == 0 || entry.buy_percent == Some(0) {
            return Err(DataValidationError::at(
                owner_id,
                format!("{field}[{index}]"),
                "Shop merchandise quantity, weight, and optional buy_percent must be positive",
            ));
        }
        if entry.merchandise.is_equipment() && entry.quantity != 1 {
            return Err(DataValidationError::at(
                owner_id,
                format!("{field}[{index}].quantity"),
                "Equipment Shop merchandise quantity must be one",
            ));
        }
        if !identities.insert(identity.clone()) {
            return Err(DataValidationError::at(
                owner_id,
                format!("{field}[{index}].merchandise"),
                format!("duplicates merchandise '{identity}'"),
            ));
        }
        validate_tags(owner_id, &format!("{field}[{index}].tags"), &entry.tags)?;
    }
    Ok(())
}

fn validate_tags(owner_id: &str, field: &str, tags: &[String]) -> Result<(), DataValidationError> {
    let mut seen = HashSet::new();
    for tag in tags {
        if tag.trim().is_empty() || !seen.insert(tag.as_str()) {
            return Err(DataValidationError::at(
                owner_id,
                field,
                "Shop merchandise tags must be nonempty and unique",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str) -> ShopMerchandiseEntry {
        ShopMerchandiseEntry {
            merchandise: ShopMerchandise::Consumable {
                consumable_id: id.to_string(),
            },
            quantity: 1,
            weight: 2,
            repeatable: false,
            buy_percent: None,
            tags: vec!["supply".to_string()],
        }
    }

    fn shop() -> ShopMetadata {
        ShopMetadata {
            id: "shop".to_string(),
            name: "Shop".to_string(),
            uuid: Uuid::from_u128(1),
            context: ShopContext::UnknownExploration,
            offer_count: 2,
            default_buy_percent: 100,
            preset_refs: vec![ShopPresetReference {
                preset_id: "common".to_string(),
                weight_multiplier: 3,
            }],
            local_entries: vec![entry("local")],
            refresh_policy: Some(ShopRefreshPolicy {
                base_cost: 0,
                max_uses: 2,
            }),
            sell_policy: Some(ShopSellPolicy {
                price_percent: 40,
                accepted_tags: None,
            }),
        }
    }

    #[test]
    fn composed_entries_are_unique_sorted_and_weighted() {
        let db = ShopDatabase::new_with_catalog(
            vec![shop()],
            vec![ShopPoolMetadata {
                id: "default".to_string(),
                shop_ids: vec!["shop".to_string()],
            }],
            vec![ShopMerchandisePreset {
                id: "common".to_string(),
                entries: vec![entry("preset")],
            }],
        );
        db.validate_contract().unwrap();
        let flattened = db.flattened_entries(&db.shops[0]).unwrap();
        assert_eq!(flattened.len(), 2);
        assert_eq!(flattened[0].merchandise.definition_id(), "local");
        assert_eq!(flattened[1].merchandise.definition_id(), "preset");
        assert_eq!(flattened[1].weight, 6);
    }

    #[test]
    fn duplicate_composed_merchandise_is_rejected() {
        let mut value = shop();
        value.local_entries = vec![entry("same")];
        let db = ShopDatabase::new_with_catalog(
            vec![value],
            vec![],
            vec![ShopMerchandisePreset {
                id: "common".to_string(),
                entries: vec![entry("same")],
            }],
        );
        let error = db.validate_contract().unwrap_err();
        assert!(error.cause.contains("duplicate merchandise"));
    }

    #[test]
    fn legacy_fixed_page_shape_is_rejected() {
        let error = ron::de::from_str::<ShopDatabase>(
            r#"ShopDatabase(
                merchandise_presets: [],
                shops: [(
                    id: "legacy", name: "Legacy",
                    uuid: "00000000-0000-0000-0000-000000000001",
                    shop_type: Shop, can_reroll: true,
                    visible_items: [], stock_items: [],
                )],
                pools: [],
            )"#,
        )
        .expect_err("prototype Shop fields must not decode");
        assert!(error.to_string().contains("shop_type"));
    }

    #[test]
    fn repeatable_defaults_false_and_unknown_fields_are_rejected() {
        let value: ShopMerchandiseEntry = ron::de::from_str(
            r#"(
                merchandise: (type: consumable, consumable_id: "tonic"),
                quantity: 2,
                weight: 1,
                buy_percent: None,
                tags: ["supply"],
            )"#,
        )
        .unwrap();
        assert!(!value.repeatable);

        assert!(ron::de::from_str::<ShopRefreshPolicy>(
            "(base_cost: 0, max_uses: 1, legacy: true)"
        )
        .is_err());
    }
}
