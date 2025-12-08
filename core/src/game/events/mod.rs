use std::time::SystemTime;

use bevy_ecs::world::World;

pub mod event_selection;
pub mod ordeal_battle;
pub mod suppression;

pub trait EventGenerator {
    type Output;

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output;
}

use crate::{ecs::components::Player, game::data::GameDataBase};

/// 선택적 컨텍스트 필드 그룹
#[derive(Default)]
pub struct GeneratorExtras {
    pub opponent_data: Option<Player>,
}

pub struct GeneratorContext<'w> {
    pub world: &'w World,
    pub game_data: &'w GameDataBase,
    pub random_seed: u64,
    pub timestamp: SystemTime,
    pub extras: GeneratorExtras,
}

impl<'w> GeneratorContext<'w> {
    /// 기본 GeneratorContext 생성 (extras는 모두 None)
    pub fn new(world: &'w World, game_data: &'w GameDataBase, random_seed: u64) -> Self {
        Self {
            world,
            game_data,
            random_seed,
            timestamp: SystemTime::now(),
            extras: Default::default(),
        }
    }

    /// Ordeal 전투용 Context 생성 (opponent_data 포함)
    pub fn with_opponent(
        world: &'w World,
        game_data: &'w GameDataBase,
        random_seed: u64,
        opponent_data: Player,
    ) -> Self {
        Self {
            world,
            game_data,
            random_seed,
            timestamp: SystemTime::now(),
            extras: GeneratorExtras {
                opponent_data: Some(opponent_data),
                ..Default::default()
            },
        }
    }
}
