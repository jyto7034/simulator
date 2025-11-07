use serde::{Deserialize, Serialize};

use crate::game::events::{
    event_selection::EventSelectionOptions, ordeal_battle::OrdealSelectionOptions,
    suppression::SuppressSelectionOptions,
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

#[derive(Clone, Copy, Debug)]
pub enum PhaseEventType {
    EventSelection,
    Suppression,
    Ordeal,
}

#[derive(Debug, Clone)]
pub enum PhaseEvent {
    EventSelection(EventSelectionOptions),
    Suppression(SuppressSelectionOptions),
    Ordeal(OrdealSelectionOptions),
}

pub struct PhaseSchedule {
    pub phase: PhaseType,
    pub event_type: PhaseEventType,
}

impl PhaseEvent {
    /// 이벤트 타입 반환
    pub const fn kind(&self) -> PhaseEventType {
        match self {
            Self::EventSelection(_) => PhaseEventType::EventSelection,
            Self::Suppression(_) => PhaseEventType::Suppression,
            Self::Ordeal(_) => PhaseEventType::Ordeal,
        }
    }

    /// 특정 타입인지 체크
    pub const fn is_event_selection(&self) -> bool {
        matches!(self, Self::EventSelection(_))
    }

    pub const fn is_suppression(&self) -> bool {
        matches!(self, Self::Suppression(_))
    }

    pub const fn is_ordeal(&self) -> bool {
        matches!(self, Self::Ordeal(_))
    }
}
