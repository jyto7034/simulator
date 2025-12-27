mod common;

use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::{Field, Inventory, Position};
use game_core::game::battle::timeline::TimelineEvent;
use game_core::game::battle::GrowthId;
use game_core::game::data::Item;
use game_core::game::enums::Side;
use game_core::game::events::suppression::SuppressionExecutor;

use common::create_test_game_data;

#[test]
fn uses_inventory_growth_stacks_in_suppression_deck() {
    let game_data = create_test_game_data();
    let abnormality_id = "test_abnorm_1";
    let abnormality_meta = game_data
        .abnormality_data
        .get_by_id(abnormality_id)
        .expect("missing test abnormality metadata");
    let unit_uuid = abnormality_meta.uuid;
    let origin_attack = abnormality_meta.attack;

    let mut world = World::new();
    world.insert_resource(Field::new(3, 3));
    world.insert_resource(Inventory::new());

    {
        let mut field = world.get_resource_mut::<Field>().unwrap();
        field
            // NOTE: PvE encounter in `create_test_game_data()` places the opponent at (1,1),
            // so use a different cell to avoid runtime-field collisions.
            .place(unit_uuid, Side::Player, Position::new(0, 0))
            .unwrap();
    }

    {
        let mut inventory = world.get_resource_mut::<Inventory>().unwrap();
        inventory
            .add_item_owned(unit_uuid, Item::Abnormality(Arc::new(abnormality_meta.clone())))
            .unwrap();
        inventory
            .abnormalities
            .get_growth_stacks_mut(&unit_uuid)
            .unwrap()
            .stacks
            .insert(GrowthId::KillStack, 5);
    }

    let battle_result =
        SuppressionExecutor::start_battle(&mut world, game_data.clone(), abnormality_id).unwrap();

    let player_spawned_attack = battle_result.timeline.entries.iter().find_map(|entry| {
        match &entry.event {
            TimelineEvent::UnitSpawned {
                owner,
                base_uuid,
                stats,
                ..
            } if *owner == Side::Player && *base_uuid == unit_uuid => Some(stats.attack),
            _ => None,
        }
    });

    assert_eq!(player_spawned_attack, Some(origin_attack + 5));
}
