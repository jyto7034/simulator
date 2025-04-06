use uuid::Uuid;

use crate::{
    card::Card,
    enums::ZoneType,
    exception::GameError,
    game::Game,
    server::input_handler::{InputAnswer, InputRequest},
};

pub enum HandlerType {
    General(
        Box<
            dyn FnOnce(&mut Game, &Card, InputAnswer) -> Result<EffectResult, GameError>
                + Send
                + Sync,
        >,
    ),
}

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
    NeedsInput {
        inner: InputRequest,
        handler: HandlerType,
    },
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum EffectSpeed {
    Fast,   // 스피드 3
    Medium, // 스피드 2
    Slow,   // 스피드 1
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
