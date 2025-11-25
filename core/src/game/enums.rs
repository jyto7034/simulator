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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EquipmentType {
    Weapon,
    Suit,
    Accessory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Abnormality,
    Equipment,
    Artifact,
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

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use crate::game::{data::shop_data::ShopType, events::event_selection::bonus::BonusType};

    use super::*;

    // ============================================================
    // OrdealType Tests
    // ============================================================

    #[test]
    fn test_ordeal_max_phases() {
        assert_eq!(OrdealType::Dawn.max_phases(), 5);
        assert_eq!(OrdealType::Noon.max_phases(), 6);
        assert_eq!(OrdealType::Dusk.max_phases(), 5);
        assert_eq!(OrdealType::Midnight.max_phases(), 6);
        assert_eq!(OrdealType::White.max_phases(), 5);
    }

    #[test]
    fn test_ordeal_is_valid_phase() {
        // Dawn은 Phase V까지만 유효
        assert!(OrdealType::Dawn.is_valid_phase(PhaseType::I));
        assert!(OrdealType::Dawn.is_valid_phase(PhaseType::V));
        assert!(!OrdealType::Dawn.is_valid_phase(PhaseType::VI));

        // Noon은 Phase VI까지 유효
        assert!(OrdealType::Noon.is_valid_phase(PhaseType::VI));
    }

    #[test]
    fn test_ordeal_next() {
        assert_eq!(OrdealType::Dawn.next(), Some(OrdealType::Noon));
        assert_eq!(OrdealType::Noon.next(), Some(OrdealType::Dusk));
        assert_eq!(OrdealType::Dusk.next(), Some(OrdealType::Midnight));
        assert_eq!(OrdealType::Midnight.next(), Some(OrdealType::White));
        assert_eq!(OrdealType::White.next(), None);
    }

    #[test]
    fn test_ordeal_is_last() {
        assert!(!OrdealType::Dawn.is_last());
        assert!(!OrdealType::Noon.is_last());
        assert!(!OrdealType::Dusk.is_last());
        assert!(!OrdealType::Midnight.is_last());
        assert!(OrdealType::White.is_last());
    }

    // ============================================================
    // PhaseType Tests
    // ============================================================

    #[test]
    fn test_phase_value() {
        assert_eq!(PhaseType::I.value(), 1);
        assert_eq!(PhaseType::II.value(), 2);
        assert_eq!(PhaseType::III.value(), 3);
        assert_eq!(PhaseType::IV.value(), 4);
        assert_eq!(PhaseType::V.value(), 5);
        assert_eq!(PhaseType::VI.value(), 6);
    }

    #[test]
    fn test_phase_from_value() {
        assert_eq!(PhaseType::from_value(1), Some(PhaseType::I));
        assert_eq!(PhaseType::from_value(2), Some(PhaseType::II));
        assert_eq!(PhaseType::from_value(3), Some(PhaseType::III));
        assert_eq!(PhaseType::from_value(4), Some(PhaseType::IV));
        assert_eq!(PhaseType::from_value(5), Some(PhaseType::V));
        assert_eq!(PhaseType::from_value(6), Some(PhaseType::VI));
        assert_eq!(PhaseType::from_value(0), None);
        assert_eq!(PhaseType::from_value(7), None);
    }

    #[test]
    fn test_phase_next() {
        assert_eq!(PhaseType::I.next(), Some(PhaseType::II));
        assert_eq!(PhaseType::II.next(), Some(PhaseType::III));
        assert_eq!(PhaseType::III.next(), Some(PhaseType::IV));
        assert_eq!(PhaseType::IV.next(), Some(PhaseType::V));
        assert_eq!(PhaseType::V.next(), Some(PhaseType::VI));
        assert_eq!(PhaseType::VI.next(), None);
    }

    #[test]
    fn test_phase_is_last() {
        assert!(!PhaseType::I.is_last());
        assert!(!PhaseType::II.is_last());
        assert!(!PhaseType::III.is_last());
        assert!(!PhaseType::IV.is_last());
        assert!(!PhaseType::V.is_last());
        assert!(PhaseType::VI.is_last());
    }

    #[test]
    fn test_phase_is_last_in_ordeal() {
        // Dawn은 Phase V가 마지막
        assert!(!PhaseType::IV.is_last_in(&OrdealType::Dawn));
        assert!(PhaseType::V.is_last_in(&OrdealType::Dawn));

        // Noon은 Phase VI가 마지막
        assert!(!PhaseType::V.is_last_in(&OrdealType::Noon));
        assert!(PhaseType::VI.is_last_in(&OrdealType::Noon));
    }

    #[test]
    fn test_phase_first() {
        assert_eq!(PhaseType::first(), PhaseType::I);
    }

    // ============================================================
    // GameOption Tests
    // ============================================================

    #[test]
    fn test_game_option_uuid_extraction() {
        use crate::game::data::bonus_data::BonusMetadata;
        use crate::game::data::random_event_data::{EventRiskLevel, RandomEventMetadata};
        use crate::game::data::shop_data::ShopMetadata;
        use crate::game::events::event_selection::random::RandomEventType;

        // Shop UUID 추출
        let shop_uuid = Uuid::new_v4();
        let shop_option = GameOption::Shop {
            shop: ShopMetadata {
                uuid: shop_uuid,
                name: "Test Shop".to_string(),
                items_raw: vec![],
                shop_type: ShopType::Shop,
                can_reroll: false,
                visible_items: vec![],
                hidden_items: vec![],
            },
        };
        assert_eq!(shop_option.uuid(), shop_uuid);

        // Bonus UUID 추출
        let bonus_uuid = Uuid::new_v4();
        let bonus_option = GameOption::Bonus {
            bonus: BonusMetadata {
                uuid: bonus_uuid,
                bonus_type: BonusType::Enkephalin,
                name: "Test Bonus".to_string(),
                description: "Test".to_string(),
                min_amount: 10,
                max_amount: 20,
                icon: String::from("asd"),
            },
        };
        assert_eq!(bonus_option.uuid(), bonus_uuid);

        // Random UUID 추출
        let random_uuid = Uuid::new_v4();
        let random_option = GameOption::Random {
            event: RandomEventMetadata {
                id: "test_event".to_string(),
                uuid: random_uuid,
                event_type: RandomEventType::SuspiciousBox,
                name: "Test Event".to_string(),
                description: "Test".to_string(),
                image: "test.png".to_string(),
                risk_level: EventRiskLevel::Low,
            },
        };
        assert_eq!(random_option.uuid(), random_uuid);

        // SuppressAbnormality UUID 추출
        let suppress_uuid = Uuid::new_v4();
        let suppress_option = GameOption::SuppressAbnormality {
            abnormality_id: "F-01-02".to_string(),
            risk_level: RiskLevel::HE,
            uuid: suppress_uuid,
        };
        assert_eq!(suppress_option.uuid(), suppress_uuid);

        // OrdealBattle UUID 추출
        let battle_uuid = Uuid::new_v4();
        let battle_option = GameOption::OrdealBattle {
            ordeal_type: OrdealType::Dawn,
            difficulty: 1,
            uuid: battle_uuid,
        };
        assert_eq!(battle_option.uuid(), battle_uuid);
    }

    // ============================================================
    // PhaseEvent Tests
    // ============================================================

    #[test]
    fn test_phase_event_type_checks() {
        use crate::game::data::bonus_data::BonusMetadata;
        use crate::game::data::random_event_data::{EventRiskLevel, RandomEventMetadata};
        use crate::game::data::shop_data::ShopMetadata;
        use crate::game::events::event_selection::random::RandomEventType;

        let shop = ShopMetadata {
            uuid: Uuid::new_v4(),
            name: "Test Shop".to_string(),
            items_raw: vec![],
            shop_type: ShopType::Shop,
            can_reroll: false,
            visible_items: vec![],
            hidden_items: vec![],
        };
        let bonus = BonusMetadata {
            bonus_type: BonusType::Enkephalin,
            uuid: Uuid::new_v4(),
            name: "Test Bonus".to_string(),
            description: "desc".to_string(),
            icon: "icon".to_string(),
            min_amount: 1,
            max_amount: 2,
        };
        let random = RandomEventMetadata {
            id: "event".to_string(),
            uuid: Uuid::new_v4(),
            event_type: RandomEventType::SuspiciousBox,
            name: "Random".to_string(),
            description: "desc".to_string(),
            image: "img".to_string(),
            risk_level: EventRiskLevel::Low,
        };

        let event_selection = PhaseEvent::EventSelection {
            shop: shop.clone(),
            bonus: bonus.clone(),
            random: random.clone(),
        };

        assert!(event_selection.is_event_selection());
        assert!(!event_selection.is_suppression());
        assert!(!event_selection.is_ordeal());

        let options = event_selection.options();
        assert_eq!(options.len(), 3);

        let suppression = PhaseEvent::Suppression {
            candidates: [
                SuppressionOption {
                    abnormality_id: "test".into(),
                    risk_level: RiskLevel::ZAYIN,
                    uuid: Uuid::new_v4(),
                },
                SuppressionOption {
                    abnormality_id: "test2".into(),
                    risk_level: RiskLevel::TETH,
                    uuid: Uuid::new_v4(),
                },
                SuppressionOption {
                    abnormality_id: "test3".into(),
                    risk_level: RiskLevel::HE,
                    uuid: Uuid::new_v4(),
                },
            ],
        };

        assert!(!suppression.is_event_selection());
        assert!(suppression.is_suppression());
        assert!(!suppression.is_ordeal());
        assert_eq!(suppression.options().len(), 3);
    }
}
