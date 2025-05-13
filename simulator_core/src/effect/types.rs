use uuid::Uuid;

use crate::{enums::ZoneType, server::input_handler::InputRequest};

pub struct EffectInfo {
    pub effect_id: Uuid,
    pub effect_type: EffectType,
    pub from_location: ZoneType,
    pub to_location: ZoneType,
}

impl EffectInfo {
    pub fn new(
        effect_id: Uuid,
        effect_type: EffectType,
        from_location: ZoneType,
        to_location: ZoneType,
    ) -> Self {
        Self {
            effect_id,
            effect_type,
            from_location,
            to_location,
        }
    }
}

pub enum EffectResult {
    // 효과가 완전히 실행됨
    Completed,

    // 사용자 입력이 필요함
    NeedsInput { inner: InputRequest },
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy)]
pub enum EffectSpeed {
    Quick = 3,  // 스피드 3
    Medium = 2, // 스피드 2
    Slow = 1,   // 스피드 1
}

impl EffectSpeed {
    pub fn is_faster_than(&self, other: EffectSpeed) -> bool {
        self > &other
    }
    pub fn is_slower_than(&self, other: EffectSpeed) -> bool {
        self < &other
    }
    pub fn is_equal_to(&self, other: EffectSpeed) -> bool {
        self == &other
    }
    pub fn can_it_chain(&self, other: EffectSpeed) -> bool {
        self.is_faster_than(other) || self.is_equal_to(other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Dig,
    Draw,
    ModifyStat,
}

#[derive(Debug, Clone, Copy)]
pub enum EffectProcessPhase {
    ImmediatePhase, // 즉발 효과 처리 중
    ChainPhase,     // 체인 효과 처리 중
    InputWaiting,   // 사용자 입력 대기 중
}
