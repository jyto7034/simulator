use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::info;
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    data::{build_string_index, build_uuid_index, once_lock_with},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShopType {
    Shop,
    DiscountShop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "ShopMetadataRaw")]
pub struct ShopMetadata {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub shop_type: ShopType,
    pub can_reroll: bool,
    pub visible_items: Vec<Uuid>,

    #[serde(skip_serializing)]
    pub hidden_items: Vec<Uuid>,
}

impl ShopMetadata {
    /// visible_items 목록에서 UUID를 제거합니다.
    pub fn remove_visible_item(&mut self, uuid: Uuid) -> Result<(), GameError> {
        // position() 한 번만 호출 - contains() 중복 제거
        let pos = self
            .visible_items
            .iter()
            .position(|item| *item == uuid)
            .ok_or(GameError::ShopItemNotFound)?;

        // Vec에서 제거
        self.visible_items.remove(pos);
        Ok(())
    }

    pub fn reroll_items(&mut self) {
        std::mem::swap(&mut self.hidden_items, &mut self.visible_items);
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShopMetadataRaw {
    id: String,
    name: String,
    uuid: Uuid,
    shop_type: ShopType,
    can_reroll: bool,
    #[serde(default)]
    visible_items: Vec<Uuid>,
    #[serde(default)]
    stock_items: Vec<Uuid>,
}

impl TryFrom<ShopMetadataRaw> for ShopMetadata {
    type Error = String;

    fn try_from(raw: ShopMetadataRaw) -> Result<Self, Self::Error> {
        let ShopMetadataRaw {
            id,
            name,
            uuid,
            shop_type,
            can_reroll,
            visible_items,
            stock_items,
        } = raw;

        let hidden_items: Vec<Uuid> = {
            let mut stock_seen = HashSet::new();
            for item_uuid in &stock_items {
                if !stock_seen.insert(*item_uuid) {
                    return Err(format!(
                        "shop '{}' has duplicate stock item uuid {}",
                        id, item_uuid
                    ));
                }
            }

            let mut visible_seen = HashSet::new();
            for item_uuid in &visible_items {
                if !visible_seen.insert(*item_uuid) {
                    return Err(format!(
                        "shop '{}' has duplicate visible item uuid {}",
                        id, item_uuid
                    ));
                }
                if !stock_seen.contains(item_uuid) {
                    return Err(format!(
                        "shop '{}' has visible item {} missing from stock_items",
                        id, item_uuid
                    ));
                }
            }

            let visible_lookup = visible_items.iter().copied().collect::<HashSet<_>>();
            stock_items
                .into_iter()
                .filter(|item_uuid| !visible_lookup.contains(item_uuid))
                .collect()
        };

        if can_reroll && hidden_items.is_empty() {
            return Err(format!(
                "rerollable shop '{}' must define stock_items with reserve stock",
                id
            ));
        }

        Ok(ShopMetadata {
            id,
            name,
            uuid,
            shop_type,
            can_reroll,
            visible_items,
            hidden_items,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopDatabase {
    pub shops: Vec<ShopMetadata>,
    #[serde(default)]
    pub pools: Vec<ShopPoolMetadata>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
    #[serde(skip)]
    by_uuid: OnceLock<HashMap<Uuid, usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShopPoolMetadata {
    pub id: String,
    pub shop_ids: Vec<String>,
}

impl ShopDatabase {
    pub fn new(shops: Vec<ShopMetadata>) -> Self {
        Self::new_with_pools(shops, vec![])
    }

    pub fn new_with_pools(shops: Vec<ShopMetadata>, pools: Vec<ShopPoolMetadata>) -> Self {
        info!("Shop: {:?}", shops);
        let by_id = once_lock_with(build_string_index(&shops, "shop id", |item| &item.id));
        let by_uuid = once_lock_with(build_uuid_index(&shops, "shop uuid", |item| item.uuid));

        Self {
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

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        let _ = self.by_uuid();
        for pool in &self.pools {
            assert!(!pool.id.is_empty(), "shop pool id must not be empty");
            for shop_id in &pool.shop_ids {
                assert!(
                    self.get_by_id(shop_id).is_some(),
                    "shop pool '{}' references missing shop '{}'",
                    pool.id,
                    shop_id
                );
            }
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shop_with_items(visible: Vec<Uuid>, hidden: Vec<Uuid>) -> ShopMetadata {
        ShopMetadata {
            id: "shop".to_string(),
            name: "Shop".to_string(),
            uuid: Uuid::from_u128(1),
            shop_type: ShopType::Shop,
            can_reroll: true,
            visible_items: visible,
            hidden_items: hidden,
        }
    }

    #[test]
    fn remove_visible_item_removes_exact_match_or_errors() {
        let a = Uuid::from_u128(10);
        let b = Uuid::from_u128(11);
        let mut shop = shop_with_items(vec![a, b], vec![]);

        shop.remove_visible_item(a).unwrap();
        assert_eq!(shop.visible_items, vec![b]);

        let err = shop.remove_visible_item(a).unwrap_err();
        assert!(matches!(err, GameError::ShopItemNotFound));
    }

    #[test]
    fn reroll_swaps_visible_and_hidden_items() {
        let a = Uuid::from_u128(10);
        let b = Uuid::from_u128(11);
        let mut shop = shop_with_items(vec![a], vec![b]);

        shop.reroll_items();
        assert_eq!(shop.visible_items, vec![b]);
        assert_eq!(shop.hidden_items, vec![a]);
    }

    #[test]
    fn deserializing_rerollable_shop_with_stock_items_builds_hidden_items() {
        let db: ShopDatabase = ron::de::from_str(
            r#"
            ShopDatabase(
                shops: [
                    (
                        id: "shop",
                        name: "Shop",
                        uuid: "00000000-0000-0000-0000-000000000001",
                        shop_type: Shop,
                        can_reroll: true,
                        visible_items: [
                            "00000000-0000-0000-0000-000000000010",
                        ],
                        stock_items: [
                            "00000000-0000-0000-0000-000000000010",
                            "00000000-0000-0000-0000-000000000011",
                            "00000000-0000-0000-0000-000000000012",
                        ],
                    ),
                ],
            )
            "#,
        )
        .expect("shop should deserialize");

        let shop = db.shops.first().expect("shop should exist");
        assert_eq!(shop.visible_items, vec![Uuid::from_u128(0x10)]);
        assert_eq!(
            shop.hidden_items,
            vec![Uuid::from_u128(0x11), Uuid::from_u128(0x12)]
        );
    }

    #[test]
    fn deserializing_rerollable_shop_without_stock_items_is_rejected() {
        let err = ron::de::from_str::<ShopDatabase>(
            r#"
            ShopDatabase(
                shops: [
                    (
                        id: "shop",
                        name: "Shop",
                        uuid: "00000000-0000-0000-0000-000000000001",
                        shop_type: Shop,
                        can_reroll: true,
                        visible_items: [],
                    ),
                ],
            )
            "#,
        )
        .expect_err("rerollable shop without stock should fail");

        assert!(
            err.to_string()
                .contains("must define stock_items with reserve stock"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn deserializing_shop_with_raw_hidden_items_is_rejected() {
        let err = ron::de::from_str::<ShopDatabase>(
            r#"
            ShopDatabase(
                shops: [
                    (
                        id: "shop",
                        name: "Shop",
                        uuid: "00000000-0000-0000-0000-000000000001",
                        shop_type: Shop,
                        can_reroll: true,
                        visible_items: [
                            "00000000-0000-0000-0000-000000000010",
                        ],
                        hidden_items: [
                            "00000000-0000-0000-0000-000000000011",
                        ],
                    ),
                ],
            )
            "#,
        )
        .expect_err("raw hidden_items authoring should fail");

        assert!(
            err.to_string().contains("hidden_items"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn shop_database_rejects_unknown_pool_authoring_fields() {
        let err = ron::de::from_str::<ShopDatabase>(
            r#"
            ShopDatabase(
                shops: [],
                pools: [
                    (
                        id: "default",
                        shop_ids: [],
                        hidden_shop_ids: [],
                    ),
                ],
            )
            "#,
        )
        .expect_err("unknown pool fields should fail");

        assert!(
            err.to_string().contains("hidden_shop_ids"),
            "unexpected error: {err}"
        );
    }
}
