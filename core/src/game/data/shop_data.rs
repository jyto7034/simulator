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
