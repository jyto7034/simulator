use std::time::SystemTime;

use bevy_ecs::world::World;

pub mod event_selection;
pub mod ordeal_battle;
pub mod suppression;

pub trait EventGenerator {
    type Output;

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output;
}

pub trait EventExecutor {
    type Input;

    fn execute(&self, ctx: &ExecutorContext, input: Self::Input) -> Result<(), EventError>;
}

#[derive(Debug)]
pub enum EventError {
    InvalidSelection,
    InsufficientResources,
}

use crate::ecs::components::Player;
use crate::game::data::GameData;

/// 선택적 컨텍스트 필드 그룹
#[derive(Default)]
pub struct GeneratorExtras {
    pub opponent_data: Option<Player>,
}

pub struct GeneratorContext<'w> {
    pub world: &'w World,
    pub game_data: &'w GameData,
    pub random_seed: u64,
    pub timestamp: SystemTime,
    pub extras: GeneratorExtras,
}

impl<'w> GeneratorContext<'w> {
    /// 기본 GeneratorContext 생성 (extras는 모두 None)
    pub fn new(world: &'w World, game_data: &'w GameData, random_seed: u64) -> Self {
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
        game_data: &'w GameData,
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

pub struct ExecutorContext<'w> {
    pub world: &'w mut World,
    pub player_id: &'w str,
    pub session_id: &'w str,
    pub timestamp: SystemTime,
}

impl<'w> ExecutorContext<'w> {
    pub fn new(world: &'w mut World, player_id: &'w str, session_id: &'w str) -> Self {
        Self {
            world,
            player_id,
            session_id,
            timestamp: SystemTime::now(),
        }
    }

    /// 편의 메서드: World 접근
    pub fn world(&self) -> &World {
        self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }
}
