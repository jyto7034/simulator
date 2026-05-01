mod common;

use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::ability::DeliveryDef;
use game_core::game::battle::core::{
    movement::types::{TimelineVec2, DEFAULT_MOVEMENT_TICK_MS},
    BattleCore,
};
use game_core::game::battle::timeline::{HpChangeReason, Timeline, TimelineEvent};
use game_core::game::battle::types::{BattleWinner, OwnedUnit, PlayerDeckInfo};
use game_core::game::data::{
    abnormality_data::{AbnormalityDatabase, AbnormalityMetadata, BasicAttackDef, MovementDef},
    artifact_data::ArtifactDatabase,
    bonus_data::BonusDatabase,
    equipment_data::EquipmentDatabase,
    pve_data::PveEncounterDatabase,
    random_event_data::RandomEventDatabase,
    shop_data::ShopDatabase,
    skill_data::SkillDatabase,
    GameDataBase,
};
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::growth::GrowthStack;
use uuid::Uuid;

fn abnormality_with_basic_attack(
    id: &str,
    uuid: Uuid,
    max_health: u32,
    attack: u32,
    move_speed_units_per_ms: u32,
    basic_attack: BasicAttackDef,
) -> AbnormalityMetadata {
    AbnormalityMetadata {
        id: id.to_string(),
        uuid,
        name: id.to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health,
        attack,
        defense: 0,
        magic_resist: 0,
        movement: MovementDef {
            speed_units_per_ms: move_speed_units_per_ms,
        },
        basic_attack,
        resonance: Default::default(),
        skill_id: None,
    }
}

fn game_data_from_abnormalities(items: Vec<AbnormalityMetadata>) -> Arc<GameDataBase> {
    Arc::new(GameDataBase::new(
        game_core::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(items)),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: common::empty_event_pools(),
        },
    ))
}

fn deck_single_unit(owned_uuid: Uuid, base_uuid: Uuid, pos: Position) -> PlayerDeckInfo {
    let mut positions = HashMap::new();
    positions.insert(owned_uuid, pos);
    PlayerDeckInfo {
        units: vec![OwnedUnit {
            owned_uuid,
            base_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        }],
        artifacts: vec![],
        positions,
    }
}

fn deck_with_units(units: Vec<(Uuid, Uuid, Position)>) -> PlayerDeckInfo {
    let mut positions = HashMap::new();
    let mut owned = Vec::new();

    for (owned_uuid, base_uuid, pos) in units {
        positions.insert(owned_uuid, pos);
        owned.push(OwnedUnit {
            owned_uuid,
            base_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        });
    }

    PlayerDeckInfo {
        units: owned,
        artifacts: vec![],
        positions,
    }
}

fn unit_spawn_time(timeline: &Timeline, owner: Side, base_uuid: Uuid) -> Option<u64> {
    timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitSpawned {
                owner: entry_owner,
                base_uuid: entry_base_uuid,
                ..
            } if *entry_owner == owner && *entry_base_uuid == base_uuid => Some(entry.time_ms),
            _ => None,
        })
}

fn assert_timeline_vec2_is_finite_and_in_bounds(position: TimelineVec2, board_size: (u8, u8)) {
    let x = position.x_milli as f32 / 1_000.0;
    let y = position.y_milli as f32 / 1_000.0;
    assert!(x.is_finite(), "timeline x must be finite: {position:?}");
    assert!(y.is_finite(), "timeline y must be finite: {position:?}");
    assert!(
        (0.0..=f32::from(board_size.0)).contains(&x),
        "timeline x must stay inside board bounds: {position:?}"
    );
    assert!(
        (0.0..=f32::from(board_size.1)).contains(&y),
        "timeline y must stay inside board bounds: {position:?}"
    );
}

fn assert_live_bodies_are_stable(battle: &BattleCore, board_size: (u8, u8)) {
    let bodies = battle.live_unit_bodies();
    for (unit_id, body) in &bodies {
        assert!(
            body.position.x.is_finite() && body.position.y.is_finite(),
            "live unit body must stay finite: {unit_id} {:?}",
            body.position
        );
        assert!(
            body.position.x >= -0.001
                && body.position.y >= -0.001
                && body.position.x <= f32::from(board_size.0) + 0.001
                && body.position.y <= f32::from(board_size.1) + 0.001,
            "live unit body must stay inside board bounds: {unit_id} {:?}",
            body.position
        );
    }

    // Unit-unit depenetration is intentionally disabled: units block movement
    // queries, but they do not forcibly push each other out of overlap.
}

#[test]
fn rapier_backend_full_battle_melee_closes_distance_and_deals_damage() {
    let player_base_uuid = Uuid::from_u128(0x9100_0001);
    let opponent_base_uuid = Uuid::from_u128(0x9100_0002);
    let basic_attack = |range_units| BasicAttackDef {
        range_units,
        interval_ms: 600,
        windup_ms: 0,
        delivery: DeliveryDef::Instant,
    };
    let game_data = game_data_from_abnormalities(vec![
        abnormality_with_basic_attack(
            "rapier_player_melee",
            player_base_uuid,
            120,
            45,
            1_400,
            basic_attack(1.0),
        ),
        abnormality_with_basic_attack(
            "rapier_opponent_melee",
            opponent_base_uuid,
            70,
            8,
            1_400,
            basic_attack(1.0),
        ),
    ]);
    let player = deck_single_unit(
        Uuid::from_u128(0xA100_0001),
        player_base_uuid,
        Position::new(3, 6),
    );
    let opponent = deck_single_unit(
        Uuid::from_u128(0xA100_0002),
        opponent_base_uuid,
        Position::new(3, 1),
    );

    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 9100);
    let mut world = World::new();
    let result = battle
        .run_battle_with_setup(&mut world, |core| {
            core.use_rapier_continuous_movement_backend();
        })
        .expect("battle runs with Rapier movement backend");

    assert_eq!(result.winner, BattleWinner::Player);
    assert!(
        result
            .timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::MovementSegmentStarted { .. })),
        "expected Rapier battle to emit continuous movement segments"
    );
    assert!(
        result.timeline.entries.iter().any(|entry| matches!(
            entry.event,
            TimelineEvent::HpChanged {
                reason: HpChangeReason::BasicAttack,
                ..
            }
        )),
        "expected units to reach attack range and deal basic attack damage"
    );
}

#[test]
fn rapier_backend_dense_mixed_team_battle_progresses_and_is_reproducible() {
    let make_melee = |id: &str, uuid: Uuid, hp: u32, attack: u32| {
        abnormality_with_basic_attack(
            id,
            uuid,
            hp,
            attack,
            1_250,
            BasicAttackDef {
                range_units: 1.0,
                interval_ms: 700,
                windup_ms: 0,
                delivery: DeliveryDef::Instant,
            },
        )
    };
    let make_ranged = |id: &str, uuid: Uuid, hp: u32, attack: u32| {
        abnormality_with_basic_attack(
            id,
            uuid,
            hp,
            attack,
            1_050,
            BasicAttackDef {
                range_units: 3.0,
                interval_ms: 850,
                windup_ms: 100,
                delivery: DeliveryDef::Instant,
            },
        )
    };

    let player_melee_uuids = [
        Uuid::from_u128(0x9300_0001),
        Uuid::from_u128(0x9300_0002),
        Uuid::from_u128(0x9300_0003),
    ];
    let player_ranged_uuids = [
        Uuid::from_u128(0x9300_0011),
        Uuid::from_u128(0x9300_0012),
        Uuid::from_u128(0x9300_0013),
    ];
    let opponent_melee_uuids = [
        Uuid::from_u128(0x9400_0001),
        Uuid::from_u128(0x9400_0002),
        Uuid::from_u128(0x9400_0003),
    ];
    let opponent_ranged_uuids = [
        Uuid::from_u128(0x9400_0011),
        Uuid::from_u128(0x9400_0012),
        Uuid::from_u128(0x9400_0013),
    ];

    let mut abnormalities = Vec::new();
    for (i, uuid) in player_melee_uuids.iter().enumerate() {
        abnormalities.push(make_melee(
            &format!("rapier_dense_p_melee_{i}"),
            *uuid,
            85,
            22,
        ));
    }
    for (i, uuid) in player_ranged_uuids.iter().enumerate() {
        abnormalities.push(make_ranged(
            &format!("rapier_dense_p_ranged_{i}"),
            *uuid,
            62,
            18,
        ));
    }
    for (i, uuid) in opponent_melee_uuids.iter().enumerate() {
        abnormalities.push(make_melee(
            &format!("rapier_dense_o_melee_{i}"),
            *uuid,
            72,
            14,
        ));
    }
    for (i, uuid) in opponent_ranged_uuids.iter().enumerate() {
        abnormalities.push(make_ranged(
            &format!("rapier_dense_o_ranged_{i}"),
            *uuid,
            54,
            12,
        ));
    }

    let game_data = game_data_from_abnormalities(abnormalities);
    let player = deck_with_units(vec![
        (
            Uuid::from_u128(0xB300_0001),
            player_melee_uuids[0],
            Position::new(0, 6),
        ),
        (
            Uuid::from_u128(0xB300_0002),
            player_melee_uuids[1],
            Position::new(3, 6),
        ),
        (
            Uuid::from_u128(0xB300_0003),
            player_melee_uuids[2],
            Position::new(6, 6),
        ),
        (
            Uuid::from_u128(0xB300_0011),
            player_ranged_uuids[0],
            Position::new(1, 7),
        ),
        (
            Uuid::from_u128(0xB300_0012),
            player_ranged_uuids[1],
            Position::new(3, 7),
        ),
        (
            Uuid::from_u128(0xB300_0013),
            player_ranged_uuids[2],
            Position::new(5, 7),
        ),
    ]);
    let opponent = deck_with_units(vec![
        (
            Uuid::from_u128(0xC300_0001),
            opponent_melee_uuids[0],
            Position::new(0, 1),
        ),
        (
            Uuid::from_u128(0xC300_0002),
            opponent_melee_uuids[1],
            Position::new(3, 1),
        ),
        (
            Uuid::from_u128(0xC300_0003),
            opponent_melee_uuids[2],
            Position::new(6, 1),
        ),
        (
            Uuid::from_u128(0xC300_0011),
            opponent_ranged_uuids[0],
            Position::new(1, 0),
        ),
        (
            Uuid::from_u128(0xC300_0012),
            opponent_ranged_uuids[1],
            Position::new(3, 0),
        ),
        (
            Uuid::from_u128(0xC300_0013),
            opponent_ranged_uuids[2],
            Position::new(5, 0),
        ),
    ]);

    let run_once = || {
        let mut battle = BattleCore::new(
            &player,
            &opponent,
            game_data.clone(),
            common::BOARD_SIZE,
            9300,
        );
        let mut world = World::new();
        battle
            .run_battle_with_setup(&mut world, |core| {
                core.use_rapier_continuous_movement_backend();
            })
            .expect("dense mixed battle runs with Rapier movement backend")
    };

    let first = run_once();
    let second = run_once();

    assert_eq!(first.winner, BattleWinner::Player);
    let movement_segments = first
        .timeline
        .entries
        .iter()
        .filter(|entry| matches!(entry.event, TimelineEvent::MovementSegmentStarted { .. }))
        .count();
    assert!(
        movement_segments >= 6,
        "expected several units to move in dense Rapier battle, got {movement_segments}"
    );

    let basic_attack_hits = first
        .timeline
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.event,
                TimelineEvent::HpChanged {
                    reason: HpChangeReason::BasicAttack,
                    ..
                }
            )
        })
        .count();
    assert!(
        basic_attack_hits >= 6,
        "expected dense Rapier battle to resolve multiple attacks, got {basic_attack_hits}"
    );

    let deaths = first
        .timeline
        .entries
        .iter()
        .filter(|entry| matches!(entry.event, TimelineEvent::UnitDied { .. }))
        .count();
    assert!(deaths > 0, "expected dense Rapier battle to produce deaths");

    assert_eq!(first.winner, second.winner);
    assert_eq!(
        serde_json::to_string(&first.timeline).expect("serialize first dense timeline"),
        serde_json::to_string(&second.timeline).expect("serialize second dense timeline")
    );
}

#[test]
fn rapier_backend_static_obstacles_and_dense_mixed_team_battle_progress() {
    let make_melee = |id: &str, uuid: Uuid, hp: u32, attack: u32| {
        abnormality_with_basic_attack(
            id,
            uuid,
            hp,
            attack,
            1_250,
            BasicAttackDef {
                range_units: 1.0,
                interval_ms: 700,
                windup_ms: 0,
                delivery: DeliveryDef::Instant,
            },
        )
    };
    let make_ranged = |id: &str, uuid: Uuid, hp: u32, attack: u32| {
        abnormality_with_basic_attack(
            id,
            uuid,
            hp,
            attack,
            1_050,
            BasicAttackDef {
                range_units: 3.0,
                interval_ms: 850,
                windup_ms: 100,
                delivery: DeliveryDef::Instant,
            },
        )
    };

    let player_melee_uuids = [
        Uuid::from_u128(0x9500_0001),
        Uuid::from_u128(0x9500_0002),
        Uuid::from_u128(0x9500_0003),
    ];
    let player_ranged_uuids = [
        Uuid::from_u128(0x9500_0011),
        Uuid::from_u128(0x9500_0012),
        Uuid::from_u128(0x9500_0013),
    ];
    let opponent_melee_uuids = [
        Uuid::from_u128(0x9600_0001),
        Uuid::from_u128(0x9600_0002),
        Uuid::from_u128(0x9600_0003),
    ];
    let opponent_ranged_uuids = [
        Uuid::from_u128(0x9600_0011),
        Uuid::from_u128(0x9600_0012),
        Uuid::from_u128(0x9600_0013),
    ];

    let mut abnormalities = Vec::new();
    for (i, uuid) in player_melee_uuids.iter().enumerate() {
        abnormalities.push(make_melee(
            &format!("rapier_obstacle_p_melee_{i}"),
            *uuid,
            85,
            22,
        ));
    }
    for (i, uuid) in player_ranged_uuids.iter().enumerate() {
        abnormalities.push(make_ranged(
            &format!("rapier_obstacle_p_ranged_{i}"),
            *uuid,
            62,
            18,
        ));
    }
    for (i, uuid) in opponent_melee_uuids.iter().enumerate() {
        abnormalities.push(make_melee(
            &format!("rapier_obstacle_o_melee_{i}"),
            *uuid,
            72,
            14,
        ));
    }
    for (i, uuid) in opponent_ranged_uuids.iter().enumerate() {
        abnormalities.push(make_ranged(
            &format!("rapier_obstacle_o_ranged_{i}"),
            *uuid,
            54,
            12,
        ));
    }

    let game_data = game_data_from_abnormalities(abnormalities);
    let player = deck_with_units(vec![
        (
            Uuid::from_u128(0xB500_0001),
            player_melee_uuids[0],
            Position::new(0, 6),
        ),
        (
            Uuid::from_u128(0xB500_0002),
            player_melee_uuids[1],
            Position::new(3, 6),
        ),
        (
            Uuid::from_u128(0xB500_0003),
            player_melee_uuids[2],
            Position::new(6, 6),
        ),
        (
            Uuid::from_u128(0xB500_0011),
            player_ranged_uuids[0],
            Position::new(1, 7),
        ),
        (
            Uuid::from_u128(0xB500_0012),
            player_ranged_uuids[1],
            Position::new(3, 7),
        ),
        (
            Uuid::from_u128(0xB500_0013),
            player_ranged_uuids[2],
            Position::new(5, 7),
        ),
    ]);
    let opponent = deck_with_units(vec![
        (
            Uuid::from_u128(0xC500_0001),
            opponent_melee_uuids[0],
            Position::new(0, 1),
        ),
        (
            Uuid::from_u128(0xC500_0002),
            opponent_melee_uuids[1],
            Position::new(3, 1),
        ),
        (
            Uuid::from_u128(0xC500_0003),
            opponent_melee_uuids[2],
            Position::new(6, 1),
        ),
        (
            Uuid::from_u128(0xC500_0011),
            opponent_ranged_uuids[0],
            Position::new(1, 0),
        ),
        (
            Uuid::from_u128(0xC500_0012),
            opponent_ranged_uuids[1],
            Position::new(3, 0),
        ),
        (
            Uuid::from_u128(0xC500_0013),
            opponent_ranged_uuids[2],
            Position::new(5, 0),
        ),
    ]);
    let static_obstacles = [
        Position::new(2, 4),
        Position::new(3, 4),
        Position::new(4, 4),
    ];

    let run_once = || {
        let mut battle = BattleCore::new(
            &player,
            &opponent,
            game_data.clone(),
            common::BOARD_SIZE,
            9500,
        );
        let mut world = World::new();
        battle
            .run_battle_with_setup(&mut world, |core| {
                core.use_rapier_continuous_movement_backend();
                for obstacle in static_obstacles {
                    core.battlefield
                        .add_static_obstacle(obstacle)
                        .expect("test obstacle should not overlap spawn positions");
                }
            })
            .expect("dense obstacle battle runs with Rapier movement backend")
    };

    let first = run_once();
    let second = run_once();

    assert_eq!(first.winner, BattleWinner::Player);
    let movement_segments = first
        .timeline
        .entries
        .iter()
        .filter(|entry| matches!(entry.event, TimelineEvent::MovementSegmentStarted { .. }))
        .count();
    assert!(
        movement_segments >= 6,
        "expected obstacle battle to require several movement segments, got {movement_segments}"
    );
    let basic_attack_hits = first
        .timeline
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.event,
                TimelineEvent::HpChanged {
                    reason: HpChangeReason::BasicAttack,
                    ..
                }
            )
        })
        .count();
    assert!(
        basic_attack_hits >= 6,
        "expected obstacle battle to still resolve attacks, got {basic_attack_hits}"
    );
    assert!(
        first
            .timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::UnitDied { .. })),
        "expected obstacle battle to produce deaths"
    );
    assert_eq!(
        serde_json::to_string(&first.timeline).expect("serialize first obstacle timeline"),
        serde_json::to_string(&second.timeline).expect("serialize second obstacle timeline")
    );
}

#[test]
fn rapier_backend_long_battle_smoke_keeps_bodies_stable_with_static_obstacles() {
    let make_unit = |id: &str, uuid: Uuid, range_units: f32| {
        abnormality_with_basic_attack(
            id,
            uuid,
            10_000,
            1,
            900_000,
            BasicAttackDef {
                range_units,
                interval_ms: 750,
                windup_ms: 0,
                delivery: DeliveryDef::Instant,
            },
        )
    };

    let player_uuids = [
        Uuid::from_u128(0x9700_0001),
        Uuid::from_u128(0x9700_0002),
        Uuid::from_u128(0x9700_0003),
        Uuid::from_u128(0x9700_0004),
        Uuid::from_u128(0x9700_0005),
        Uuid::from_u128(0x9700_0006),
    ];
    let opponent_uuids = [
        Uuid::from_u128(0x9800_0001),
        Uuid::from_u128(0x9800_0002),
        Uuid::from_u128(0x9800_0003),
        Uuid::from_u128(0x9800_0004),
        Uuid::from_u128(0x9800_0005),
        Uuid::from_u128(0x9800_0006),
    ];

    let mut abnormalities = Vec::new();
    for (i, uuid) in player_uuids.iter().enumerate() {
        abnormalities.push(make_unit(
            &format!("rapier_long_p_{i}"),
            *uuid,
            if i % 2 == 0 { 1.0 } else { 3.0 },
        ));
    }
    for (i, uuid) in opponent_uuids.iter().enumerate() {
        abnormalities.push(make_unit(
            &format!("rapier_long_o_{i}"),
            *uuid,
            if i % 2 == 0 { 1.0 } else { 3.0 },
        ));
    }

    let game_data = game_data_from_abnormalities(abnormalities);
    let player = deck_with_units(vec![
        (
            Uuid::from_u128(0xD700_0001),
            player_uuids[0],
            Position::new(0, 6),
        ),
        (
            Uuid::from_u128(0xD700_0002),
            player_uuids[1],
            Position::new(1, 7),
        ),
        (
            Uuid::from_u128(0xD700_0003),
            player_uuids[2],
            Position::new(3, 6),
        ),
        (
            Uuid::from_u128(0xD700_0004),
            player_uuids[3],
            Position::new(3, 7),
        ),
        (
            Uuid::from_u128(0xD700_0005),
            player_uuids[4],
            Position::new(6, 6),
        ),
        (
            Uuid::from_u128(0xD700_0006),
            player_uuids[5],
            Position::new(5, 7),
        ),
    ]);
    let opponent = deck_with_units(vec![
        (
            Uuid::from_u128(0xE700_0001),
            opponent_uuids[0],
            Position::new(0, 1),
        ),
        (
            Uuid::from_u128(0xE700_0002),
            opponent_uuids[1],
            Position::new(1, 0),
        ),
        (
            Uuid::from_u128(0xE700_0003),
            opponent_uuids[2],
            Position::new(3, 1),
        ),
        (
            Uuid::from_u128(0xE700_0004),
            opponent_uuids[3],
            Position::new(3, 0),
        ),
        (
            Uuid::from_u128(0xE700_0005),
            opponent_uuids[4],
            Position::new(6, 1),
        ),
        (
            Uuid::from_u128(0xE700_0006),
            opponent_uuids[5],
            Position::new(5, 0),
        ),
    ]);

    let static_obstacles = [
        Position::new(2, 3),
        Position::new(3, 3),
        Position::new(4, 3),
        Position::new(2, 4),
        Position::new(4, 4),
    ];

    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 9700);
    let mut world = World::new();
    let result = battle
        .run_battle_with_setup(&mut world, |core| {
            core.use_rapier_continuous_movement_backend();
            for obstacle in static_obstacles {
                core.battlefield
                    .add_static_obstacle(obstacle)
                    .expect("long smoke obstacle should not overlap spawn positions");
            }
        })
        .expect("long Rapier smoke battle runs");

    assert_eq!(result.winner, BattleWinner::Draw);
    let battle_end_ms = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match entry.event {
            TimelineEvent::BattleEnd { .. } => Some(entry.time_ms),
            _ => None,
        })
        .expect("long smoke battle should end");
    assert_eq!(battle_end_ms, 60_000);
    assert!(battle_end_ms / DEFAULT_MOVEMENT_TICK_MS >= 1_000);

    let movement_segments = result
        .timeline
        .entries
        .iter()
        .filter(|entry| matches!(entry.event, TimelineEvent::MovementSegmentStarted { .. }))
        .count();
    assert!(
        movement_segments >= 6,
        "long smoke should exercise movement across multiple units, got {movement_segments}"
    );
    let basic_attack_hits = result
        .timeline
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.event,
                TimelineEvent::HpChanged {
                    reason: HpChangeReason::BasicAttack,
                    ..
                }
            )
        })
        .count();
    assert!(
        basic_attack_hits >= 20,
        "long smoke should keep combat active, got {basic_attack_hits} hits"
    );

    for entry in &result.timeline.entries {
        match entry.event {
            TimelineEvent::MovementSegmentStarted { start, target, .. } => {
                assert_timeline_vec2_is_finite_and_in_bounds(start, common::BOARD_SIZE);
                assert_timeline_vec2_is_finite_and_in_bounds(target, common::BOARD_SIZE);
            }
            TimelineEvent::MovementStopped { world_position, .. } => {
                assert_timeline_vec2_is_finite_and_in_bounds(world_position, common::BOARD_SIZE);
            }
            _ => {}
        }
    }

    assert_live_bodies_are_stable(&battle, common::BOARD_SIZE);
    assert_eq!(battle.active_basic_projectile_count(), 0);
    assert_eq!(battle.active_skill_projectile_count(), 0);
    assert_eq!(battle.active_area_count(), 0);
}

#[test]
fn rapier_backend_full_battle_is_reproducible_for_same_seed() {
    let player_base_uuid = Uuid::from_u128(0x9200_0001);
    let opponent_base_uuid = Uuid::from_u128(0x9200_0002);
    let game_data = game_data_from_abnormalities(vec![
        abnormality_with_basic_attack(
            "rapier_replay_player",
            player_base_uuid,
            100,
            40,
            1_200,
            BasicAttackDef {
                range_units: 1.0,
                interval_ms: 700,
                windup_ms: 0,
                delivery: DeliveryDef::Instant,
            },
        ),
        abnormality_with_basic_attack(
            "rapier_replay_opponent",
            opponent_base_uuid,
            80,
            10,
            1_200,
            BasicAttackDef {
                range_units: 1.0,
                interval_ms: 700,
                windup_ms: 0,
                delivery: DeliveryDef::Instant,
            },
        ),
    ]);
    let player = deck_single_unit(
        Uuid::from_u128(0xA200_0001),
        player_base_uuid,
        Position::new(2, 6),
    );
    let opponent = deck_single_unit(
        Uuid::from_u128(0xA200_0002),
        opponent_base_uuid,
        Position::new(4, 1),
    );

    let run_once = || {
        let mut battle = BattleCore::new(
            &player,
            &opponent,
            game_data.clone(),
            common::BOARD_SIZE,
            9200,
        );
        let mut world = World::new();
        battle
            .run_battle_with_setup(&mut world, |core| {
                core.use_rapier_continuous_movement_backend();
            })
            .expect("battle runs with Rapier movement backend")
    };

    let first = run_once();
    let second = run_once();

    assert_eq!(first.winner, second.winner);
    assert_eq!(
        unit_spawn_time(&first.timeline, Side::Player, player_base_uuid),
        unit_spawn_time(&second.timeline, Side::Player, player_base_uuid)
    );
    assert_eq!(
        serde_json::to_string(&first.timeline).expect("serialize first timeline"),
        serde_json::to_string(&second.timeline).expect("serialize second timeline")
    );
}
