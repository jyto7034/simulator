use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::enums::Tier;

/// 상점 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShopType {
    Shop,         // 일반 상점
    DiscountShop, // 할인 상점
}

/// 아이템
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub uuid: Uuid,
    pub name: String,
    pub price: u32,
    pub tier: Tier,
}

impl Item {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: "test".to_string(),
            price: 12,
            tier: Tier::I,
        }
    }
}

/// 상점
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    pub name: String,
    pub uuid: Uuid,
    pub items: Vec<Item>,
}

/// RON 파일 최상위 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopDatabase {
    pub shops: Vec<Shop>,
}
