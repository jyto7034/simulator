use bevy_ecs::{entity::Entity, world::World};
use uuid::Uuid;

use crate::ecs::components::{Player, PlayerBundle};

// TODO: bevy_ecs System 구현 위치
// 현재는 비어있음
pub mod progression;

pub fn spawn_player(world: &mut World, player_id: Uuid) -> Entity {
    world
        .spawn(PlayerBundle {
            player: Player {
                id: player_id,
                name: "Hero".to_string(),
            },
        })
        .id()
}
