use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::data::{
    bonus_data::{BonusMetadata, BonusType},
    random_event_data::{RandomEventInnerMetadata, RandomEventMetadata},
    shop_data::{ShopMetadata, ShopType},
};
use crate::game::events::event_selection::random::RandomEventType;

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
            // Keep in sync with `OrdealScheduler::get_phase_schedule`.
            Self::Dawn => 6,
            Self::Noon => 6,
            Self::Dusk => 5,
            Self::Midnight => 5,
            Self::White => 6,
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
pub struct BonusEventOption {
    pub id: String,
    pub bonus_type: BonusType,
    pub uuid: Uuid,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub amount: u32,
}

impl From<&BonusMetadata> for BonusEventOption {
    fn from(value: &BonusMetadata) -> Self {
        Self {
            id: value.id.clone(),
            bonus_type: value.bonus_type,
            uuid: value.uuid,
            name: value.name.clone(),
            description: value.description.clone(),
            icon: value.icon.clone(),
            amount: value.amount,
        }
    }
}

impl From<BonusMetadata> for BonusEventOption {
    fn from(value: BonusMetadata) -> Self {
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
// GameOption
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameOption {
    // EventSelection 옵션들
    Shop {
        shop: ShopEventOption,
    },
    Bonus {
        bonus: BonusEventOption,
    },
    Random {
        event: RandomEventOption,
    },

    // Suppression 옵션들
    SuppressAbnormality {
        abnormality_id: String,
        encounter_id: String,
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
        shop: ShopEventOption,
        bonus: BonusEventOption,
        random: RandomEventOption,
    },
    Suppression {
        candidates: Vec<SuppressionOption>,
    },
    Ordeal {
        candidates: Vec<OrdealOption>,
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
    ) -> Option<(&ShopEventOption, &BonusEventOption, &RandomEventOption)> {
        match self {
            PhaseEvent::EventSelection {
                shop,
                bonus,
                random,
            } => Some((shop, bonus, random)),
            _ => None,
        }
    }

    pub fn as_suppression(&self) -> Option<&[SuppressionOption]> {
        match self {
            PhaseEvent::Suppression { candidates } => Some(candidates.as_slice()),
            _ => None,
        }
    }

    pub fn as_ordeal(&self) -> Option<&[OrdealOption]> {
        match self {
            PhaseEvent::Ordeal { candidates } => Some(candidates.as_slice()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_type_value_roundtrip_and_next() {
        assert_eq!(PhaseType::I.value(), 1);
        assert_eq!(PhaseType::VI.value(), 6);

        for value in 1..=6 {
            let phase = PhaseType::from_value(value).unwrap();
            assert_eq!(phase.value(), value);
        }
        assert!(PhaseType::from_value(0).is_none());
        assert!(PhaseType::from_value(7).is_none());

        assert_eq!(PhaseType::I.next(), Some(PhaseType::II));
        assert_eq!(PhaseType::VI.next(), None);
        assert!(PhaseType::VI.is_last());
        assert!(!PhaseType::V.is_last());
    }

    #[test]
    fn ordeal_type_phase_validation_and_progression() {
        assert_eq!(OrdealType::Dawn.max_phases(), 6);
        assert_eq!(OrdealType::Noon.max_phases(), 6);

        assert!(OrdealType::Dawn.is_valid_phase(PhaseType::V));
        assert!(OrdealType::Dawn.is_valid_phase(PhaseType::VI));

        assert_eq!(OrdealType::Dawn.next(), Some(OrdealType::Noon));
        assert_eq!(OrdealType::White.next(), None);
        assert!(OrdealType::White.is_last());
        assert!(!OrdealType::Midnight.is_last());
    }
}

pub struct PhaseSchedule {
    pub phase: PhaseType,
    pub event_type: PhaseEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionOption {
    pub abnormality_id: String,
    pub encounter_id: String,
    pub risk_level: RiskLevel,
    pub uuid: Uuid,
}

impl From<SuppressionOption> for GameOption {
    fn from(option: SuppressionOption) -> Self {
        GameOption::SuppressAbnormality {
            abnormality_id: option.abnormality_id,
            encounter_id: option.encounter_id,
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
