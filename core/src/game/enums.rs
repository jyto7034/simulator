#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OrdealType {
    Dawn,     // 여명
    Noon,     // 정오
    Dusk,     // 어스름
    Midnight, // 자정
    White,    // 백색
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    ZAYIN,
    TETH,
    HE,
    WAW,
    ALEPH,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

pub struct PhaseSchedule {
    pub phase_number: u8,
    pub event_type: PhaseEventType,
}
