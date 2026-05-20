use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::data::{
    random_event_data::{RandomEventInnerMetadata, RandomEventMetadata},
    shop_data::{ShopMetadata, ShopType},
};
use crate::game::events::event_selection::random::RandomEventType;

// ============================================================
// 기타 Enums
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum RiskLevel {
    ZAYIN,
    TETH,
    HE,
    WAW,
    ALEPH,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Tier {
    I,
    II,
    III,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Lane {
    Front,
    Mid,
    Back,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Side {
    Opponent,
    Player,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RewardMode {
    ClaimAll,
    ChooseOne,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShopEventOption {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub shop_type: ShopType,
    pub can_reroll: bool,
    pub visible_items: Vec<Uuid>,
}

impl From<&ShopMetadata> for ShopEventOption {
    fn from(value: &ShopMetadata) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            uuid: value.uuid,
            shop_type: value.shop_type,
            can_reroll: value.can_reroll,
            visible_items: value.visible_items.clone(),
        }
    }
}

impl From<ShopMetadata> for ShopEventOption {
    fn from(value: ShopMetadata) -> Self {
        Self::from(&value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RandomEventOption {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub event_type: RandomEventType,
    pub risk_level: RiskLevel,
    pub description: String,
    pub image: String,
    pub inner_metadata: RandomEventInnerMetadata,
}

impl From<&RandomEventMetadata> for RandomEventOption {
    fn from(value: &RandomEventMetadata) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            uuid: value.uuid,
            event_type: value.event_type.clone(),
            risk_level: value.risk_level,
            description: value.description.clone(),
            image: value.image.clone(),
            inner_metadata: value.inner_metadata.clone(),
        }
    }
}

impl From<RandomEventMetadata> for RandomEventOption {
    fn from(value: RandomEventMetadata) -> Self {
        Self::from(&value)
    }
}

// ============================================================
// 내부 행동 타입 (통합 핸들러용)
// ============================================================

/// 상점 내부 행동
pub enum ShopAction {
    Purchase { item_uuid: Uuid },
    Sell { item_uuid: Uuid },
    Reroll,
    Exit,
}

/// 랜덤 이벤트 내부 행동
pub enum RandomEventAction {
    SelectChoice { choice_id: String },
    Exit,
}

/// 보상 세션 내부 행동
pub enum RewardAction {
    Claim,
    Exit,
}
