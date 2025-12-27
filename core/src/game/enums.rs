use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::data::{
    bonus_data::BonusMetadata, random_event_data::RandomEventMetadata, shop_data::ShopMetadata,
};

pub trait MoveTo {
    type Output;
    fn next(&self) -> Option<Self::Output>;
    fn is_last(&self) -> bool;
}

// ============================================================
// OrdealType
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum OrdealType {
    Dawn,     // 여명
    Noon,     // 정오
    Dusk,     // 어스름
    Midnight, // 자정
    White,    // 백색
}

impl OrdealType {
    pub const fn max_phases(&self) -> u8 {
        match self {
            Self::Dawn => 5,
            Self::Noon => 6,
            Self::Dusk => 5,
            Self::Midnight => 6,
            Self::White => 5,
        }
    }

    pub fn is_valid_phase(&self, phase: PhaseType) -> bool {
        phase.value() <= self.max_phases()
    }
}

impl MoveTo for OrdealType {
    type Output = Self;

    fn next(&self) -> Option<Self::Output> {
        match self {
            Self::Dawn => Some(Self::Noon),
            Self::Noon => Some(Self::Dusk),
            Self::Dusk => Some(Self::Midnight),
            Self::Midnight => Some(Self::White),
            Self::White => None,
        }
    }

    fn is_last(&self) -> bool {
        matches!(self, Self::White)
    }
}

// ============================================================
// PhaseType
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum PhaseType {
    I,
    II,
    III,
    IV,
    V,
    VI,
}

impl PhaseType {
    /// Phase를 숫자로 변환 (1-based)
    pub const fn value(&self) -> u8 {
        match self {
            Self::I => 1,
            Self::II => 2,
            Self::III => 3,
            Self::IV => 4,
            Self::V => 5,
            Self::VI => 6,
        }
    }

    /// 숫자에서 Phase 생성
    pub const fn from_value(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::I),
            2 => Some(Self::II),
            3 => Some(Self::III),
            4 => Some(Self::IV),
            5 => Some(Self::V),
            6 => Some(Self::VI),
            _ => None,
        }
    }

    pub const fn first() -> Self {
        Self::I
    }

    /// 특정 Ordeal에서 마지막 Phase인지
    pub fn is_last_in(&self, ordeal: &OrdealType) -> bool {
        self.value() == ordeal.max_phases()
    }
}

impl MoveTo for PhaseType {
    type Output = Self;

    fn next(&self) -> Option<Self::Output> {
        Self::from_value(self.value() + 1)
    }

    fn is_last(&self) -> bool {
        matches!(self, Self::VI)
    }
}

// ============================================================
// 기타 Enums
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ZoneType {
    Inventory,
    Field,
}

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
pub enum PhaseEventType {
    EventSelection,
    Suppression,
    Ordeal,
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

// ============================================================
// GameOption
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameOption {
    // EventSelection 옵션들
    Shop {
        shop: ShopMetadata, // 상점 전체 데이터 (이름, 아이템 목록, uuid 등 모두 포함)
    },
    Bonus {
        bonus: BonusMetadata, // 보너스 전체 데이터 (타입, 이름, 설명, 수량 범위 등 모두 포함)
    },
    Random {
        event: RandomEventMetadata, // 랜덤 이벤트 전체 데이터 (이름, 설명, 이미지, 위험도 등 모두 포함)
    },

    // Suppression 옵션들
    SuppressAbnormality {
        abnormality_id: String,
        risk_level: RiskLevel,
        uuid: Uuid, // TODO: Abnormality 전체 메타데이터로 변경 예정
    },

    // Ordeal 옵션들
    OrdealBattle {
        ordeal_type: OrdealType,
        difficulty: u8,
        uuid: Uuid, // TODO: OrdealBattle 전체 메타데이터로 변경 예정
    },
}

impl GameOption {
    /// GameOption에서 uuid 추출
    pub fn uuid(&self) -> Uuid {
        match self {
            GameOption::Shop { shop } => shop.uuid,
            GameOption::Bonus { bonus } => bonus.uuid,
            GameOption::Random { event } => event.uuid,
            GameOption::SuppressAbnormality { uuid, .. } => *uuid,
            GameOption::OrdealBattle { uuid, .. } => *uuid,
        }
    }
}

// ============================================================
// PhaseEvent
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseEvent {
    EventSelection {
        shop: ShopMetadata,
        bonus: BonusMetadata,
        random: RandomEventMetadata,
    },
    Suppression {
        candidates: [SuppressionOption; 3],
    },
    Ordeal {
        candidates: [OrdealOption; 3],
    },
}

impl PhaseEvent {
    pub fn options(&self) -> Vec<GameOption> {
        match self {
            PhaseEvent::EventSelection {
                shop,
                bonus,
                random,
            } => vec![
                GameOption::Shop { shop: shop.clone() },
                GameOption::Bonus {
                    bonus: bonus.clone(),
                },
                GameOption::Random {
                    event: random.clone(),
                },
            ],
            PhaseEvent::Suppression { candidates } => {
                candidates.iter().cloned().map(GameOption::from).collect()
            }
            PhaseEvent::Ordeal { candidates } => {
                candidates.iter().cloned().map(GameOption::from).collect()
            }
        }
    }

    pub fn is_event_selection(&self) -> bool {
        matches!(self, PhaseEvent::EventSelection { .. })
    }

    pub fn is_suppression(&self) -> bool {
        matches!(self, PhaseEvent::Suppression { .. })
    }

    pub fn is_ordeal(&self) -> bool {
        matches!(self, PhaseEvent::Ordeal { .. })
    }

    pub fn as_event_selection(
        &self,
    ) -> Option<(&ShopMetadata, &BonusMetadata, &RandomEventMetadata)> {
        match self {
            PhaseEvent::EventSelection {
                shop,
                bonus,
                random,
            } => Some((shop, bonus, random)),
            _ => None,
        }
    }

    pub fn as_suppression(&self) -> Option<&[SuppressionOption; 3]> {
        match self {
            PhaseEvent::Suppression { candidates } => Some(candidates),
            _ => None,
        }
    }

    pub fn as_ordeal(&self) -> Option<&[OrdealOption; 3]> {
        match self {
            PhaseEvent::Ordeal { candidates } => Some(candidates),
            _ => None,
        }
    }

    pub fn event_type(&self) -> PhaseEventType {
        match self {
            PhaseEvent::EventSelection { .. } => PhaseEventType::EventSelection,
            PhaseEvent::Suppression { .. } => PhaseEventType::Suppression,
            PhaseEvent::Ordeal { .. } => PhaseEventType::Ordeal,
        }
    }
}

pub struct PhaseSchedule {
    pub phase: PhaseType,
    pub event_type: PhaseEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionOption {
    pub abnormality_id: String,
    pub risk_level: RiskLevel,
    pub uuid: Uuid,
}

impl From<SuppressionOption> for GameOption {
    fn from(option: SuppressionOption) -> Self {
        GameOption::SuppressAbnormality {
            abnormality_id: option.abnormality_id,
            risk_level: option.risk_level,
            uuid: option.uuid,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdealOption {
    pub ordeal_type: OrdealType,
    pub difficulty: u8,
    pub uuid: Uuid,
}

impl From<OrdealOption> for GameOption {
    fn from(option: OrdealOption) -> Self {
        GameOption::OrdealBattle {
            ordeal_type: option.ordeal_type,
            difficulty: option.difficulty,
            uuid: option.uuid,
        }
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

/// 보너스 내부 행동
pub enum BonusAction {
    Claim,
    Exit,
}
