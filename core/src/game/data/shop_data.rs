use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::game::{
    behavior::GameError,
    data::GameData,
    data::ItemReference,
};

/// 상점 타입
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShopType {
    Shop,         // 일반 상점
    DiscountShop, // 할인 상점
}

/// 상점 상품 (ID로 참조)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShopProduct {
    Abnormality(String), // 환상체 ID (예: "scorched_girl")
    Equipment(String),   // 장비 ID (예: "justitia")
    Artifact(String),    // 아티팩트 ID (예: "one_sin")
}

impl ShopProduct {
    /// 상품의 ID 반환
    pub fn id(&self) -> &str {
        match self {
            Self::Abnormality(id) => id,
            Self::Equipment(id) => id,
            Self::Artifact(id) => id,
        }
    }

    pub fn uuid(&self, game_data: &GameData) -> Uuid {
        match self {
            Self::Abnormality(id) => game_data.get_abnormality_from_product(&id).unwrap().uuid,
            Self::Equipment(id) => game_data.get_equipment_from_product(&id).unwrap().uuid,
            Self::Artifact(id) => game_data.get_artifact_from_product(&id).unwrap().uuid,
        }
    }
}

/// 런타임에서 사용하는 상점 아이템은 ItemReference 그대로 활용
pub type ShopItem = ItemReference;

/// 상점
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopMetadata {
    pub name: String,
    pub uuid: Uuid,
    pub shop_type: ShopType,
    pub can_reroll: bool,

    /// RON에서 로드한 전체 아이템
    #[serde(rename = "items", default)]
    pub items_raw: Vec<ShopProduct>,

    /// 현재 플레이어에게 보이는 아이템
    #[serde(default, skip_deserializing)]
    pub visible_items: Vec<ShopItem>,

    /// 리롤 시 보여줄 숨겨진 아이템
    #[serde(default, skip_deserializing)]
    pub hidden_items: Vec<ShopItem>,
}

impl ShopMetadata {
    /// RON에서 로드한 raw 아이템을 실제 메타데이터로 해석하여 visible/hidden 구성
    pub fn hydrate<R: rand::Rng>(
        &mut self,
        game_data: &GameData,
        rng: &mut R,
    ) -> Result<(), GameError> {
        use rand::seq::SliceRandom;

        let mut resolved: Vec<ShopItem> = self
            .items_raw
            .iter()
            .map(|p| resolve_item_reference(p, game_data))
            .collect::<Result<_, _>>()?;

        if self.can_reroll {
            resolved.shuffle(rng);
            let half = (resolved.len() + 1) / 2; // 올림
            self.visible_items = resolved.iter().take(half).cloned().collect();
            self.hidden_items = resolved.into_iter().skip(half).collect();
        } else {
            self.visible_items = resolved;
            self.hidden_items.clear();
        }

        Ok(())
    }

    /// 리롤: visible과 hidden을 swap
    pub fn reroll_items(&mut self) {
        std::mem::swap(&mut self.visible_items, &mut self.hidden_items);
    }

    pub fn find_item(&self, uuid: Uuid) -> Result<&ShopItem, GameError> {
        self.find_visible_item(uuid).ok_or(GameError::InvalidEvent)
    }

    pub fn find_visible_item(&self, uuid: Uuid) -> Option<&ShopItem> {
        self.visible_items.iter().find(|item| item.uuid() == uuid)
    }

    /// 현재 보이는 아이템 목록에서 UUID로 아이템을 제거하고 반환
    pub fn remove_visible_item(&mut self, uuid: Uuid) -> Result<ShopItem, GameError> {
        let idx = self
            .visible_items
            .iter()
            .position(|item| item.uuid() == uuid)
            .ok_or(GameError::InvalidEvent)?;

        Ok(self.visible_items.remove(idx))
    }
}

fn resolve_item_reference(product: &ShopProduct, game_data: &GameData) -> Result<ItemReference, GameError> {
    match product {
        ShopProduct::Abnormality(id) => game_data
            .get_abnormality_from_product(id)
            .cloned()
            .map(Arc::new)
            .map(ItemReference::Abnormality)
            .ok_or(GameError::InvalidEvent),
        ShopProduct::Equipment(id) => game_data
            .get_equipment_from_product(id)
            .cloned()
            .map(Arc::new)
            .map(ItemReference::Equipment)
            .ok_or(GameError::InvalidEvent),
        ShopProduct::Artifact(id) => game_data
            .get_artifact_from_product(id)
            .cloned()
            .map(Arc::new)
            .map(ItemReference::Artifact)
            .ok_or(GameError::InvalidEvent),
    }
}

/// RON 파일 최상위 구조체
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopDatabase {
    pub shops: Vec<ShopMetadata>,

    #[serde(skip)]
    shop_map: HashMap<Uuid, ShopMetadata>,
}

impl ShopDatabase {
    /// Database 생성 (HashMap 초기화)
    pub fn new(shops: Vec<ShopMetadata>) -> Self {
        let shop_map = shops.iter().map(|s| (s.uuid, s.clone())).collect();

        Self { shops, shop_map }
    }

    /// RON 역직렬화 후 HashMap 초기화
    pub fn init_map(&mut self) {
        self.shop_map = self.shops.iter().map(|s| (s.uuid, s.clone())).collect();
    }

    /// UUID로 상점 조회 (O(1))
    pub fn get_by_uuid(&self, uuid: &Uuid) -> Option<&ShopMetadata> {
        self.shop_map.get(uuid)
    }
}
