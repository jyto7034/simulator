use std::time::SystemTime;

use bevy_ecs::world::World;

pub mod game;

pub struct GeneratorContext<'w> {
    pub world: &'w World,
    pub random_seed: u64,
    pub timestamp: SystemTime,
}

impl<'w> GeneratorContext<'w> {
    pub fn new(world: &'w World, random_seed: u64) -> Self {
        Self {
            world,
            random_seed,
            timestamp: SystemTime::now(),
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
