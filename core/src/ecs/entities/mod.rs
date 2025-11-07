use bevy_ecs::{entity::Entity, world::World};
use uuid::Uuid;

use crate::ecs::components::{Player, PlayerBundle};

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
