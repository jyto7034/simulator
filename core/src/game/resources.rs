use bevy_ecs::resource::Resource;

use crate::game::enums::OrdealType;

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

#[derive(Resource)]
pub struct CurrentOrdeal {
    pub ordeal_type: OrdealType,
}

impl CurrentOrdeal {
    pub fn new(ordeal_type: OrdealType) -> Self {
        Self { ordeal_type }
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
