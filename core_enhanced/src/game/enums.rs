use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::data::shop_data::{ShopContext, ShopMerchandise};

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
pub struct ShopOfferOption {
    pub offer_id: Uuid,
    pub merchandise: ShopMerchandise,
    pub item_uuid: Uuid,
    pub quantity: u32,
    pub total_price: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShopRefreshOption {
    pub current_cost: u32,
    pub max_uses: u32,
    pub uses_consumed: u32,
    pub remaining_uses: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShopEventOption {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub context: ShopContext,
    pub offers: Vec<ShopOfferOption>,
    pub refresh: Option<ShopRefreshOption>,
    pub selling_enabled: bool,
}

// ============================================================
// 내부 행동 타입 (통합 핸들러용)
// ============================================================

/// 상점 내부 행동
pub enum ShopAction {
    Purchase { offer_id: Uuid },
    Sell { item_uuid: Uuid, quantity: u32 },
    Refresh,
    Exit,
}

/// 보상 세션 내부 행동
pub enum RewardAction {
    Claim,
    Exit,
}
