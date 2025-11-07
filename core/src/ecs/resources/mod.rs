use bevy_ecs::resource::Resource;

use crate::game::enums::{MoveTo, OrdealType, PhaseType};

#[derive(Resource)]
pub struct Enkephalin {
    pub amount: u32,
}

impl Enkephalin {
    pub fn new(initial_amount: u32) -> Self {
        Self {
            amount: initial_amount,
        }
    }
}

#[derive(Resource)]
pub struct Level {
    pub level: u32,
}

impl Level {
    pub fn new(initial_level: u32) -> Self {
        Self {
            level: initial_level,
        }
    }
}

/// 게임 진행 상황 (Ordeal, Phase) - 순수 데이터만
#[derive(Resource, Debug, Clone)]
pub struct GameProgression {
    pub current_ordeal: OrdealType,
    pub current_phase: PhaseType,
}

impl GameProgression {
    pub fn new() -> Self {
        Self {
            current_ordeal: OrdealType::Dawn,
            current_phase: PhaseType::I,
        }
    }
}

impl Default for GameProgression {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource)]
pub struct WinCount {
    pub count: u32,
}

impl WinCount {
    pub fn new(initial_count: u32) -> Self {
        Self {
            count: initial_count,
        }
    }
}
