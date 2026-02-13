use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::game::behavior::GameError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShopType {
    Shop,
    DiscountShop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopMetadata {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub shop_type: ShopType,
    pub can_reroll: bool,
    pub visible_items: Vec<Uuid>,

    #[serde(skip)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopDatabase {
    pub shops: Vec<ShopMetadata>,
}

impl ShopDatabase {
    pub fn new(shops: Vec<ShopMetadata>) -> Self {
        info!("Shop: {:?}", shops);
        Self { shops }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&ShopMetadata> {
        self.shops.iter().find(|item| item.id == id)
    }

    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&ShopMetadata> {
        self.shops.iter().find(|item| item.uuid == *uuid)
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
}
