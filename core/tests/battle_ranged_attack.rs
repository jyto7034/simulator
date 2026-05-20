mod common;

use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::ability::DeliveryDef;
use game_core::game::battle::core::movement::TILE_UNITS_PER_TILE;
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::ids::UnitInstanceId;
use game_core::game::battle::replay::{types::TimelineReplayerConfig, TimelineReplayer};
use game_core::game::battle::timeline::{HpChangeReason, TimelineCause, TimelineEvent};
use game_core::game::battle::types::{BattleWinner, OwnedUnit, PlayerDeckInfo};
use game_core::game::battle::validation::{
    TimelineExpectedCounts, TimelineValidator, TimelineValidatorConfig,
};
use game_core::game::data::{
    abnormality_data::{
        AbnormalityDatabase, AbnormalityMetadata, BasicAttackDef, MovementDef,
        DEFAULT_INSTANT_BASIC_ATTACK_WINDUP_MS,
    },
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

fn ceil_div_u64(a: u64, b: u64) -> u64 {
    if a == 0 {
        return 0;
    }
    if b == 0 {
        return 0;
    }
    a.saturating_add(b.saturating_sub(1)) / b
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

fn abnormality_with_basic_attack(
    id: &str,
    uuid: Uuid,
    max_health: u32,
    attack: u32,
    defense: i32,
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
        defense,
        magic_resist: 0,
        movement: MovementDef {
            speed_units_per_ms: move_speed_units_per_ms,
        },
        basic_attack,
        resonance: Default::default(),
        skill_id: None,
    }
}

fn find_unit_instance_id(
    timeline: &game_core::game::battle::timeline::Timeline,
    owner: Side,
    base_uuid: Uuid,
) -> UnitInstanceId {
    timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                owner: entry_owner,
                base_uuid: entry_base_uuid,
                ..
            } if *entry_owner == owner && *entry_base_uuid == base_uuid => Some(*unit_instance_id),
            _ => None,
        })
        .expect("missing UnitSpawned for requested unit")
}

#[test]
fn ranged_basic_attack_projectile_hits_after_flight_time_and_damages_target() {
    // Given: projectile basic attack with known speed and distance.
    let attacker_base_uuid = Uuid::from_u128(0xCAFE_0001);
    let target_base_uuid = Uuid::from_u128(0xCAFE_0002);

    let attacker = AbnormalityMetadata {
        id: "ranged_attacker".to_string(),
        uuid: attacker_base_uuid,
        name: "Ranged Attacker".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 999,
        attack: 1000,
        defense: 0,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: BasicAttackDef {
            range_tiles: 3,
            interval_ms: 100,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 3_000,
                collision: Default::default(),
            },
        },
        resonance: Default::default(),
        skill_id: None,
    };

    let target = AbnormalityMetadata {
        id: "target".to_string(),
        uuid: target_base_uuid,
        name: "Target".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 10,
        attack: 1,
        defense: 0,
        magic_resist: 0,
        movement: MovementDef {
            speed_units_per_ms: 0,
        },
        // Keep target in-range to avoid movement; set a huge interval so it never attacks before death.
        basic_attack: BasicAttackDef {
            range_tiles: 3,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
        resonance: Default::default(),
        skill_id: None,
    };

    let game_data = Arc::new(GameDataBase::new(
        game_core::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![attacker, target])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: common::empty_event_pools(),
        },
    ));

    let attacker_owned = Uuid::from_u128(0xDADA_0001);
    let target_owned = Uuid::from_u128(0xDADA_0002);
    let attacker_pos = Position::new(0, 0);
    let target_pos = Position::new(3, 0);

    let player = deck_single_unit(attacker_owned, attacker_base_uuid, attacker_pos);
    let opponent = deck_single_unit(target_owned, target_base_uuid, target_pos);

    // When: run the battle.
    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        12345,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "ranged_basic_attack_projectile_hits_after_flight_time",
        &result.timeline,
    );

    // Then: the player's projectile attack should land after the correct flight time.
    let mut player_instance_id: Option<_> = None;
    let mut opponent_instance_id: Option<_> = None;
    for entry in &result.timeline.entries {
        if let TimelineEvent::UnitSpawned {
            unit_instance_id,
            owner,
            ..
        } = entry.event
        {
            match owner {
                Side::Player => player_instance_id = Some(unit_instance_id),
                Side::Opponent => opponent_instance_id = Some(unit_instance_id),
            }
        }
    }
    let player_instance_id = player_instance_id.expect("missing player UnitSpawned");
    let opponent_instance_id = opponent_instance_id.expect("missing opponent UnitSpawned");

    let attack_time_ms = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                ..
            } if *attacker_instance_id == player_instance_id
                && *target_instance_id == opponent_instance_id =>
            {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("missing AttackResolve event for ranged attacker");

    let (hp_time_ms, hp_before, hp_after) = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                target_instance_id,
                hp_before,
                hp_after,
                reason,
                ..
            } if *target_instance_id == opponent_instance_id
                && *reason == HpChangeReason::BasicAttack =>
            {
                Some((entry.time_ms, *hp_before, *hp_after))
            }
            _ => None,
        })
        .expect("missing HpChanged(BasicAttack) for target");

    // Distance is Chebyshev tiles.
    let dist_tiles = attacker_pos.chebyshev(&target_pos).max(0) as u64;
    let distance_units = dist_tiles.saturating_mul(TILE_UNITS_PER_TILE);
    let speed_units_per_ms = 3_000_u64;
    let flight_ms = ceil_div_u64(distance_units, speed_units_per_ms);
    let expected_impact_ms = attack_time_ms.saturating_add(flight_ms);

    assert_eq!(hp_before, 10);
    assert_eq!(hp_after, 0);
    assert_eq!(hp_time_ms, expected_impact_ms);

    // And: Replay/Validation should pass (guards deterministic ordering and parent/seq invariants).
    let mut replay_config = TimelineReplayerConfig::default();
    replay_config.validate_unit_base_uuid = true;
    TimelineReplayer::new(game_data.clone(), replay_config)
        .replay(&result.timeline)
        .unwrap();

    let expected = TimelineExpectedCounts::from_decks(&player, &opponent);
    TimelineValidator::new(TimelineValidatorConfig::default())
        .validate(&result.timeline, Some(expected), Some(game_data.as_ref()))
        .unwrap();
}

#[test]
fn ranged_attack_resolve_records_miss_when_target_moves_out_of_range() {
    let lure_base_uuid = Uuid::from_u128(0x1000_0001);
    let attacker_base_uuid = Uuid::from_u128(0x1000_0002);
    let target_base_uuid = Uuid::from_u128(0x2000_0001);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 2,
            interval_ms: 1,
            windup_ms: 5,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 1_000,
                collision: Default::default(),
            },
        },
    );

    let lure = abnormality_with_basic_attack(
        "lure",
        lure_base_uuid,
        100,
        1,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let target = abnormality_with_basic_attack(
        "target",
        target_base_uuid,
        999,
        1,
        0,
        1_000_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, lure, target]);

    let attacker_owned = Uuid::from_u128(0xA000_0001);
    let lure_owned = Uuid::from_u128(0xA000_0002);
    let target_owned = Uuid::from_u128(0xB000_0001);

    let player = deck_with_units(vec![
        (lure_owned, lure_base_uuid, Position::new(3, 1)),
        (attacker_owned, attacker_base_uuid, Position::new(0, 3)),
    ]);
    let opponent = deck_with_units(vec![(target_owned, target_base_uuid, Position::new(1, 1))]);

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        4242,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "ranged_attack_resolve_records_miss_when_target_moves_out_of_range",
        &result.timeline,
    );

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);

    let miss_cause = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::AttackMiss {
                attacker_instance_id,
                target_instance_id,
                ..
            } if *attacker_instance_id == attacker_id && *target_instance_id == target_id => {
                Some(entry.cause)
            }
            _ => None,
        })
        .expect("missing AttackMiss when target moved out of range");

    let miss_parent_seq = match miss_cause {
        TimelineCause::Parent { seq } => seq,
        _ => panic!("AttackMiss should have parent cause"),
    };

    let hit_for_same_attack = result
        .timeline
        .entries
        .iter()
        .any(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                ..
            } if *source_instance_id == Some(attacker_id)
                && *target_instance_id == target_id
                && matches!(entry.cause, TimelineCause::Parent { seq } if seq == miss_parent_seq) =>
            {
                true
            }
            _ => false,
        });
    assert!(
        !hit_for_same_attack,
        "AttackMiss should not apply damage for the same attack"
    );
}

#[test]
fn projectile_misses_when_target_dies_before_impact() {
    let attacker_base_uuid = Uuid::from_u128(0x1100_0001);
    let finisher_base_uuid = Uuid::from_u128(0x1100_0002);
    let target_base_uuid = Uuid::from_u128(0x2200_0001);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        5,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 4,
            interval_ms: 1,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 100_000,
                collision: Default::default(),
            },
        },
    );

    let finisher = abnormality_with_basic_attack(
        "finisher",
        finisher_base_uuid,
        100,
        999,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 4,
            interval_ms: 2,
            windup_ms: 1,
            delivery: DeliveryDef::Instant,
        },
    );

    let target = abnormality_with_basic_attack(
        "target",
        target_base_uuid,
        5,
        1,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, finisher, target]);

    let attacker_owned = Uuid::from_u128(0xA100_0001);
    let finisher_owned = Uuid::from_u128(0xA100_0002);
    let target_owned = Uuid::from_u128(0xB100_0001);

    let player = deck_with_units(vec![
        (attacker_owned, attacker_base_uuid, Position::new(0, 0)),
        (finisher_owned, finisher_base_uuid, Position::new(0, 1)),
    ]);
    let opponent = deck_with_units(vec![(target_owned, target_base_uuid, Position::new(3, 0))]);

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        2024,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "projectile_misses_when_target_dies_before_impact",
        &result.timeline,
    );

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);

    let target_death_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitDied {
                unit_instance_id, ..
            } if *unit_instance_id == target_id => Some(entry.time_ms),
            _ => None,
        })
        .expect("target should die before projectile impact");

    let attacker_hit_after_death = result
        .timeline
        .entries
        .iter()
        .any(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                ..
            } if *source_instance_id == Some(attacker_id)
                && *target_instance_id == target_id
                && entry.time_ms >= target_death_time =>
            {
                true
            }
            _ => false,
        });
    assert!(
        !attacker_hit_after_death,
        "attacker projectile should not apply damage after target death"
    );
}

#[test]
fn projectile_does_not_hit_after_attacker_death_when_battle_ends() {
    let attacker_base_uuid = Uuid::from_u128(0x1200_0001);
    let target_base_uuid = Uuid::from_u128(0x2300_0001);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        10,
        5,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 4,
            interval_ms: 1,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 100_000,
                collision: Default::default(),
            },
        },
    );

    let target = abnormality_with_basic_attack(
        "target",
        target_base_uuid,
        100,
        999,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 4,
            interval_ms: 2,
            windup_ms: 1,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, target]);

    let attacker_owned = Uuid::from_u128(0xA200_0001);
    let target_owned = Uuid::from_u128(0xB200_0001);

    let player = deck_single_unit(attacker_owned, attacker_base_uuid, Position::new(0, 0));
    let opponent = deck_single_unit(target_owned, target_base_uuid, Position::new(3, 0));

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        777,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "projectile_does_not_hit_after_attacker_death_when_battle_ends",
        &result.timeline,
    );

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);

    let attacker_death_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitDied {
                unit_instance_id, ..
            } if *unit_instance_id == attacker_id => Some(entry.time_ms),
            _ => None,
        })
        .expect("attacker should die before projectile impact");

    let attacker_hit_after_death = result
        .timeline
        .entries
        .iter()
        .any(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                ..
            } if *source_instance_id == Some(attacker_id)
                && *target_instance_id == target_id
                && entry.time_ms >= attacker_death_time =>
            {
                true
            }
            _ => false,
        });

    assert!(
        !attacker_hit_after_death,
        "projectile hit should not be recorded after attacker death"
    );
}

#[test]
fn projectile_speed_zero_hits_same_tick_as_attack_resolve() {
    let attacker_base_uuid = Uuid::from_u128(0x1300_0001);
    let target_base_uuid = Uuid::from_u128(0x2400_0001);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 4,
            interval_ms: 1,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 0,
                collision: Default::default(),
            },
        },
    );

    let target = abnormality_with_basic_attack(
        "target",
        target_base_uuid,
        10,
        1,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, target]);

    let player = deck_single_unit(
        Uuid::from_u128(0xA300_0001),
        attacker_base_uuid,
        Position::new(0, 0),
    );
    let opponent = deck_single_unit(
        Uuid::from_u128(0xB300_0001),
        target_base_uuid,
        Position::new(2, 0),
    );

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        9001,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "projectile_speed_zero_hits_same_tick_as_attack_resolve",
        &result.timeline,
    );

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);

    let resolve_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                ..
            } if *attacker_instance_id == attacker_id && *target_instance_id == target_id => {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("missing AttackResolve");

    let impact_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == Some(attacker_id)
                && *target_instance_id == target_id
                && *reason == HpChangeReason::BasicAttack =>
            {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("missing HpChanged for projectile hit");

    assert_eq!(impact_time, resolve_time);
}

#[test]
fn ranged_vs_melee_simple_ranged_wins() {
    let ranged_base_uuid = Uuid::from_u128(0x1400_0001);
    let melee_base_uuid = Uuid::from_u128(0x2500_0001);

    let ranged = abnormality_with_basic_attack(
        "ranged",
        ranged_base_uuid,
        50,
        20,
        0,
        1_000,
        BasicAttackDef {
            range_tiles: 4,
            interval_ms: 1000,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 1_000_000,
                collision: Default::default(),
            },
        },
    );

    let melee = abnormality_with_basic_attack(
        "melee",
        melee_base_uuid,
        40,
        3,
        0,
        1_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 1000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![ranged, melee]);

    let ranged_owned = Uuid::from_u128(0xA300_0001);
    let melee_owned = Uuid::from_u128(0xB300_0001);

    let player = deck_single_unit(ranged_owned, ranged_base_uuid, Position::new(0, 0));
    let opponent = deck_single_unit(melee_owned, melee_base_uuid, Position::new(5, 0));

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        2025,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export("ranged_vs_melee_simple_ranged_wins", &result.timeline);

    assert_eq!(result.winner, BattleWinner::Player);

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, ranged_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, melee_base_uuid);

    let has_ranged_hit = result
        .timeline
        .entries
        .iter()
        .any(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == Some(attacker_id)
                && *target_instance_id == target_id
                && *reason == HpChangeReason::BasicAttack =>
            {
                true
            }
            _ => false,
        });
    assert!(has_ranged_hit, "expected ranged attacker to hit target");
}

#[test]
fn ranged_vs_ranged_both_sides_land_hits_player_wins() {
    let player_base_uuid = Uuid::from_u128(0x1500_0001);
    let opponent_base_uuid = Uuid::from_u128(0x2600_0001);

    let player_unit = abnormality_with_basic_attack(
        "ranged_player",
        player_base_uuid,
        40,
        8,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 4,
            interval_ms: 1_000,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 3_000,
                collision: Default::default(),
            },
        },
    );

    let opponent_unit = abnormality_with_basic_attack(
        "ranged_opponent",
        opponent_base_uuid,
        40,
        5,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 4,
            interval_ms: 1_000,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 3_000,
                collision: Default::default(),
            },
        },
    );

    let game_data = game_data_from_abnormalities(vec![player_unit, opponent_unit]);

    let player_owned = Uuid::from_u128(0xA400_0001);
    let opponent_owned = Uuid::from_u128(0xB400_0001);

    let player = deck_single_unit(player_owned, player_base_uuid, Position::new(0, 0));
    let opponent = deck_single_unit(opponent_owned, opponent_base_uuid, Position::new(3, 0));

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        2026,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "ranged_vs_ranged_both_sides_land_hits_player_wins",
        &result.timeline,
    );

    assert_eq!(result.winner, BattleWinner::Player);

    let player_id = find_unit_instance_id(&result.timeline, Side::Player, player_base_uuid);
    let opponent_id = find_unit_instance_id(&result.timeline, Side::Opponent, opponent_base_uuid);

    let player_hit = result
        .timeline
        .entries
        .iter()
        .any(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == Some(player_id)
                && *target_instance_id == opponent_id
                && *reason == HpChangeReason::BasicAttack =>
            {
                true
            }
            _ => false,
        });
    assert!(player_hit, "expected player ranged unit to hit opponent");

    let opponent_hit = result
        .timeline
        .entries
        .iter()
        .any(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == Some(opponent_id)
                && *target_instance_id == player_id
                && *reason == HpChangeReason::BasicAttack =>
            {
                true
            }
            _ => false,
        });
    assert!(opponent_hit, "expected opponent ranged unit to hit player");
}

#[test]
fn tft_like_field_6v6_mixed_melee_ranged_battle() {
    let move_speed_units_per_ms = 1_200;

    let make_melee = |id: &str, uuid: Uuid, hp: u32, attack: u32| {
        abnormality_with_basic_attack(
            id,
            uuid,
            hp,
            attack,
            2,
            move_speed_units_per_ms,
            BasicAttackDef {
                range_tiles: 1,
                interval_ms: 1_500,
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
            1,
            move_speed_units_per_ms,
            BasicAttackDef {
                range_tiles: 3,
                interval_ms: 1_700,
                windup_ms: 200,
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 3_000_000,
                    collision: Default::default(),
                },
            },
        )
    };

    let player_melee_uuids = [
        Uuid::from_u128(0x1600_0001),
        Uuid::from_u128(0x1600_0002),
        Uuid::from_u128(0x1600_0003),
    ];
    let player_ranged_uuids = [
        Uuid::from_u128(0x1600_0011),
        Uuid::from_u128(0x1600_0012),
        Uuid::from_u128(0x1600_0013),
    ];
    let opponent_melee_uuids = [
        Uuid::from_u128(0x2600_0001),
        Uuid::from_u128(0x2600_0002),
        Uuid::from_u128(0x2600_0003),
    ];
    let opponent_ranged_uuids = [
        Uuid::from_u128(0x2600_0011),
        Uuid::from_u128(0x2600_0012),
        Uuid::from_u128(0x2600_0013),
    ];

    let mut units = Vec::new();
    for (i, uuid) in player_melee_uuids.iter().enumerate() {
        units.push(make_melee(&format!("p_melee_{i}"), *uuid, 55, 12));
    }
    for (i, uuid) in player_ranged_uuids.iter().enumerate() {
        units.push(make_ranged(&format!("p_ranged_{i}"), *uuid, 45, 10));
    }
    for (i, uuid) in opponent_melee_uuids.iter().enumerate() {
        units.push(make_melee(&format!("o_melee_{i}"), *uuid, 55, 12));
    }
    for (i, uuid) in opponent_ranged_uuids.iter().enumerate() {
        units.push(make_ranged(&format!("o_ranged_{i}"), *uuid, 45, 10));
    }

    let game_data = game_data_from_abnormalities(units);

    let mut player_units = Vec::new();
    let mut opponent_units = Vec::new();

    let player_owned = [
        Uuid::from_u128(0xA500_0001),
        Uuid::from_u128(0xA500_0002),
        Uuid::from_u128(0xA500_0003),
        Uuid::from_u128(0xA500_0011),
        Uuid::from_u128(0xA500_0012),
        Uuid::from_u128(0xA500_0013),
    ];
    let opponent_owned = [
        Uuid::from_u128(0xB500_0001),
        Uuid::from_u128(0xB500_0002),
        Uuid::from_u128(0xB500_0003),
        Uuid::from_u128(0xB500_0011),
        Uuid::from_u128(0xB500_0012),
        Uuid::from_u128(0xB500_0013),
    ];

    let player_positions = [
        Position::new(1, 6),
        Position::new(3, 6),
        Position::new(5, 6),
        Position::new(0, 7),
        Position::new(2, 7),
        Position::new(4, 7),
    ];
    let opponent_positions = [
        Position::new(1, 1),
        Position::new(3, 1),
        Position::new(5, 1),
        Position::new(0, 0),
        Position::new(2, 0),
        Position::new(4, 0),
    ];

    for i in 0..3 {
        player_units.push((player_owned[i], player_melee_uuids[i], player_positions[i]));
        opponent_units.push((
            opponent_owned[i],
            opponent_melee_uuids[i],
            opponent_positions[i],
        ));
    }
    for i in 0..3 {
        player_units.push((
            player_owned[i + 3],
            player_ranged_uuids[i],
            player_positions[i + 3],
        ));
        opponent_units.push((
            opponent_owned[i + 3],
            opponent_ranged_uuids[i],
            opponent_positions[i + 3],
        ));
    }

    let player = deck_with_units(player_units);
    let opponent = deck_with_units(opponent_units);

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        3030,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "tft_like_field_6v6_mixed_melee_ranged_battle",
        &result.timeline,
    );

    let deaths = result
        .timeline
        .entries
        .iter()
        .filter(|e| matches!(e.event, TimelineEvent::UnitDied { .. }))
        .count();
    assert!(deaths > 0, "expected at least one unit to die");

    let opponent_frontliner_instance_id = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                owner,
                base_uuid,
                ..
            } if *owner == Side::Opponent && *base_uuid == opponent_melee_uuids[0] => {
                Some(*unit_instance_id)
            }
            _ => None,
        })
        .expect("expected opponent frontline instance id");

    let first_move = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitMoved {
                unit_instance_id,
                from,
                to,
            } if *unit_instance_id == opponent_frontliner_instance_id => Some((*from, *to)),
            _ => None,
        })
        .expect("expected opponent frontline to move");

    assert!(
        first_move.1.y >= first_move.0.y,
        "opponent frontline should not take an immediate backward detour: {:?} -> {:?}",
        first_move.0,
        first_move.1
    );

    let opponent_center_melee_instance_id = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                owner,
                base_uuid,
                ..
            } if *owner == Side::Opponent && *base_uuid == opponent_melee_uuids[1] => {
                Some(*unit_instance_id)
            }
            _ => None,
        })
        .expect("expected opponent center melee instance id");

    let center_first_move = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitMoved {
                unit_instance_id,
                from,
                to,
            } if *unit_instance_id == opponent_center_melee_instance_id => Some((*from, *to)),
            _ => None,
        })
        .expect("expected opponent center melee to move");

    assert!(
        center_first_move.1.y >= center_first_move.0.y,
        "opponent center melee should not take an immediate backward detour: {:?} -> {:?}",
        center_first_move.0,
        center_first_move.1
    );

    let opening_melee_instance_ids = [
        find_unit_instance_id(&result.timeline, Side::Player, player_melee_uuids[0]),
        find_unit_instance_id(&result.timeline, Side::Player, player_melee_uuids[1]),
        find_unit_instance_id(&result.timeline, Side::Player, player_melee_uuids[2]),
        find_unit_instance_id(&result.timeline, Side::Opponent, opponent_melee_uuids[0]),
        find_unit_instance_id(&result.timeline, Side::Opponent, opponent_melee_uuids[1]),
        find_unit_instance_id(&result.timeline, Side::Opponent, opponent_melee_uuids[2]),
    ];

    for unit_instance_id in opening_melee_instance_ids {
        let first_attack_start_time = result
            .timeline
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                TimelineEvent::AttackStart {
                    attacker_instance_id,
                    delivery: Some(game_core::game::battle::timeline::AttackDelivery::Instant),
                    ..
                } if *attacker_instance_id == unit_instance_id => Some(entry.time_ms),
                _ => None,
            })
            .expect("expected opening melee attacker to eventually start an instant attack");

        let move_times_before_first_attack: Vec<u64> = result
            .timeline
            .entries
            .iter()
            .filter_map(|entry| match &entry.event {
                TimelineEvent::UnitMoved {
                    unit_instance_id: moved_unit_id,
                    ..
                } if *moved_unit_id == unit_instance_id
                    && entry.time_ms <= first_attack_start_time =>
                {
                    Some(entry.time_ms)
                }
                _ => None,
            })
            .collect();

        let last_move_time = *move_times_before_first_attack
            .last()
            .expect("opening melee should move before first engage");

        assert_eq!(
            last_move_time,
            1_251,
            "opening melee should finish tile movement at the frontline before actual continuous reach closes: unit={unit_instance_id:?}, move_times={move_times_before_first_attack:?}, first_attack_start_time={first_attack_start_time}"
        );
        assert!(
            last_move_time < first_attack_start_time,
            "opening melee should close actual continuous reach after its last tile step, not keep walking into another tile before attacking: unit={unit_instance_id:?}, last_move_time={last_move_time}, first_attack_start_time={first_attack_start_time}"
        );
    }
}

#[test]
fn tft_like_field_7v7_dense_frontline_prefers_straight_opening_engage() {
    let move_speed_units_per_ms = 1_200;

    let make_melee = |id: &str, uuid: Uuid, hp: u32, attack: u32| {
        abnormality_with_basic_attack(
            id,
            uuid,
            hp,
            attack,
            2,
            move_speed_units_per_ms,
            BasicAttackDef {
                range_tiles: 1,
                interval_ms: 1_500,
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
            1,
            move_speed_units_per_ms,
            BasicAttackDef {
                range_tiles: 3,
                interval_ms: 1_700,
                windup_ms: 200,
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 3_000_000,
                    collision: Default::default(),
                },
            },
        )
    };

    let player_melee_uuids = [
        Uuid::from_u128(0x1700_0001),
        Uuid::from_u128(0x1700_0002),
        Uuid::from_u128(0x1700_0003),
        Uuid::from_u128(0x1700_0004),
    ];
    let player_ranged_uuids = [
        Uuid::from_u128(0x1700_0011),
        Uuid::from_u128(0x1700_0012),
        Uuid::from_u128(0x1700_0013),
    ];
    let opponent_melee_uuids = [
        Uuid::from_u128(0x2700_0001),
        Uuid::from_u128(0x2700_0002),
        Uuid::from_u128(0x2700_0003),
        Uuid::from_u128(0x2700_0004),
    ];
    let opponent_ranged_uuids = [
        Uuid::from_u128(0x2700_0011),
        Uuid::from_u128(0x2700_0012),
        Uuid::from_u128(0x2700_0013),
    ];

    let mut units = Vec::new();
    for (i, uuid) in player_melee_uuids.iter().enumerate() {
        units.push(make_melee(&format!("dense_p_melee_{i}"), *uuid, 58, 12));
    }
    for (i, uuid) in player_ranged_uuids.iter().enumerate() {
        units.push(make_ranged(&format!("dense_p_ranged_{i}"), *uuid, 46, 10));
    }
    for (i, uuid) in opponent_melee_uuids.iter().enumerate() {
        units.push(make_melee(&format!("dense_o_melee_{i}"), *uuid, 58, 12));
    }
    for (i, uuid) in opponent_ranged_uuids.iter().enumerate() {
        units.push(make_ranged(&format!("dense_o_ranged_{i}"), *uuid, 46, 10));
    }

    let game_data = game_data_from_abnormalities(units);

    let mut player_units = Vec::new();
    let mut opponent_units = Vec::new();

    let player_owned = [
        Uuid::from_u128(0xC700_0001),
        Uuid::from_u128(0xC700_0002),
        Uuid::from_u128(0xC700_0003),
        Uuid::from_u128(0xC700_0004),
        Uuid::from_u128(0xC700_0011),
        Uuid::from_u128(0xC700_0012),
        Uuid::from_u128(0xC700_0013),
    ];
    let opponent_owned = [
        Uuid::from_u128(0xD700_0001),
        Uuid::from_u128(0xD700_0002),
        Uuid::from_u128(0xD700_0003),
        Uuid::from_u128(0xD700_0004),
        Uuid::from_u128(0xD700_0011),
        Uuid::from_u128(0xD700_0012),
        Uuid::from_u128(0xD700_0013),
    ];

    let player_positions = [
        Position::new(0, 6),
        Position::new(2, 6),
        Position::new(4, 6),
        Position::new(6, 6),
        Position::new(1, 7),
        Position::new(3, 7),
        Position::new(5, 7),
    ];
    let opponent_positions = [
        Position::new(0, 1),
        Position::new(2, 1),
        Position::new(4, 1),
        Position::new(6, 1),
        Position::new(1, 0),
        Position::new(3, 0),
        Position::new(5, 0),
    ];

    for i in 0..4 {
        player_units.push((player_owned[i], player_melee_uuids[i], player_positions[i]));
        opponent_units.push((
            opponent_owned[i],
            opponent_melee_uuids[i],
            opponent_positions[i],
        ));
    }
    for i in 0..3 {
        player_units.push((
            player_owned[i + 4],
            player_ranged_uuids[i],
            player_positions[i + 4],
        ));
        opponent_units.push((
            opponent_owned[i + 4],
            opponent_ranged_uuids[i],
            opponent_positions[i + 4],
        ));
    }

    let player = deck_with_units(player_units);
    let opponent = deck_with_units(opponent_units);

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        4040,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "tft_like_field_7v7_dense_frontline_prefers_straight_opening_engage",
        &result.timeline,
    );

    let deaths = result
        .timeline
        .entries
        .iter()
        .filter(|e| matches!(e.event, TimelineEvent::UnitDied { .. }))
        .count();
    assert!(deaths > 0, "expected dense battle to resolve with deaths");

    let opening_melee_instance_ids: Vec<(Side, UnitInstanceId)> = player_melee_uuids
        .iter()
        .map(|uuid| {
            (
                Side::Player,
                find_unit_instance_id(&result.timeline, Side::Player, *uuid),
            )
        })
        .chain(opponent_melee_uuids.iter().map(|uuid| {
            (
                Side::Opponent,
                find_unit_instance_id(&result.timeline, Side::Opponent, *uuid),
            )
        }))
        .collect();

    for (side, unit_instance_id) in opening_melee_instance_ids {
        let first_move = result
            .timeline
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                TimelineEvent::UnitMoved {
                    unit_instance_id: moved_unit_id,
                    from,
                    to,
                } if *moved_unit_id == unit_instance_id => Some((*from, *to)),
                _ => None,
            })
            .expect("expected dense frontline melee to move");

        assert_eq!(
            first_move.1.x,
            first_move.0.x,
            "dense opening frontline should prefer straight opening engage over immediate diagonal: side={side:?} unit={unit_instance_id:?} move={first_move:?}"
        );

        match side {
            Side::Player => assert!(
                first_move.1.y < first_move.0.y,
                "player dense frontline should advance forward: unit={unit_instance_id:?} move={first_move:?}"
            ),
            Side::Opponent => assert!(
                first_move.1.y > first_move.0.y,
                "opponent dense frontline should advance forward: unit={unit_instance_id:?} move={first_move:?}"
            ),
        }
    }

    let player_frontline_instance_ids: Vec<UnitInstanceId> = player_melee_uuids
        .iter()
        .map(|uuid| find_unit_instance_id(&result.timeline, Side::Player, *uuid))
        .collect();
    let player_instance_ids: Vec<UnitInstanceId> = player_melee_uuids
        .iter()
        .chain(player_ranged_uuids.iter())
        .map(|uuid| find_unit_instance_id(&result.timeline, Side::Player, *uuid))
        .collect();
    let opponent_frontline_instance_ids: Vec<UnitInstanceId> = opponent_melee_uuids
        .iter()
        .map(|uuid| find_unit_instance_id(&result.timeline, Side::Opponent, *uuid))
        .collect();

    let first_opponent_frontline_death_time = result
        .timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::UnitDied {
                unit_instance_id,
                owner: Side::Opponent,
                ..
            } if opponent_frontline_instance_ids.contains(unit_instance_id) => Some(entry.time_ms),
            _ => None,
        })
        .min()
        .expect("expected dense frontline to eventually kill an opponent melee");

    let player_killers: Vec<UnitInstanceId> = result
        .timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::UnitDied {
                unit_instance_id,
                owner: Side::Opponent,
                killer_instance_id,
            } if entry.time_ms == first_opponent_frontline_death_time
                && opponent_frontline_instance_ids.contains(unit_instance_id) =>
            {
                killer_instance_id.filter(|id| player_instance_ids.contains(id))
            }
            _ => None,
        })
        .collect();

    assert!(
        !player_killers.is_empty(),
        "expected dense frontline collapse to be caused by player units at the first opponent frontline death time"
    );

    for killer_id in player_killers
        .into_iter()
        .filter(|id| player_frontline_instance_ids.contains(id))
    {
        let next_move = result
            .timeline
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                TimelineEvent::UnitMoved {
                    unit_instance_id,
                    from,
                    to,
                } if *unit_instance_id == killer_id
                    && entry.time_ms > first_opponent_frontline_death_time =>
                {
                    Some((entry.time_ms, *from, *to))
                }
                _ => None,
            })
            .expect(
                "expected frontline melee killer to reacquire movement after frontline collapse",
            );

        assert!(
            next_move.0 < first_opponent_frontline_death_time + 1_500,
            "dense frontline killer should reacquire movement before the next attack cadence after frontline collapse: unit={killer_id:?}, death={first_opponent_frontline_death_time}, next_move={next_move:?}"
        );
        assert!(
            next_move.2.y < next_move.1.y,
            "dense frontline killer should continue advancing after frontline collapse instead of hesitating or stepping backward: unit={killer_id:?}, move={next_move:?}"
        );
    }
}

#[test]
fn melee_reacquires_movement_immediately_after_frontliner_dies() {
    let move_speed_units_per_ms = 1_200;

    let player_melee_uuid = Uuid::from_u128(0x1800_0001);
    let opponent_front_uuid = Uuid::from_u128(0x2800_0001);
    let opponent_back_uuid = Uuid::from_u128(0x2800_0002);

    let player_melee = abnormality_with_basic_attack(
        "reacquire_player_melee",
        player_melee_uuid,
        100,
        20,
        2,
        move_speed_units_per_ms,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 1_500,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );
    let opponent_front = abnormality_with_basic_attack(
        "reacquire_opponent_front",
        opponent_front_uuid,
        20,
        1,
        0,
        move_speed_units_per_ms,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );
    let opponent_back = abnormality_with_basic_attack(
        "reacquire_opponent_back",
        opponent_back_uuid,
        100,
        1,
        0,
        move_speed_units_per_ms,
        BasicAttackDef {
            range_tiles: 3,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 3_000_000,
                collision: Default::default(),
            },
        },
    );

    let game_data = game_data_from_abnormalities(vec![player_melee, opponent_front, opponent_back]);

    let player = deck_single_unit(
        Uuid::from_u128(0xE800_0001),
        player_melee_uuid,
        Position::new(1, 2),
    );
    let opponent = deck_with_units(vec![
        (
            Uuid::from_u128(0xF800_0001),
            opponent_front_uuid,
            Position::new(1, 1),
        ),
        (
            Uuid::from_u128(0xF800_0002),
            opponent_back_uuid,
            Position::new(1, 0),
        ),
    ]);

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        5050,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "melee_reacquires_movement_immediately_after_frontliner_dies",
        &result.timeline,
    );

    let player_id = find_unit_instance_id(&result.timeline, Side::Player, player_melee_uuid);
    let front_id = find_unit_instance_id(&result.timeline, Side::Opponent, opponent_front_uuid);

    let front_death_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitDied {
                unit_instance_id, ..
            } if *unit_instance_id == front_id => Some(entry.time_ms),
            _ => None,
        })
        .expect("expected frontliner to die");

    let next_move_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitMoved {
                unit_instance_id, ..
            } if *unit_instance_id == player_id && entry.time_ms > front_death_time => {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("expected melee to move again after frontliner death");

    assert!(
        next_move_time < front_death_time + 1_500,
        "melee should reacquire movement before its next attack cadence after frontliner death: death={front_death_time}, next_move={next_move_time}"
    );
}

#[test]
fn melee_prefers_straight_backline_target_after_frontliner_dies() {
    let move_speed_units_per_ms = 1_200;

    let player_melee_uuid = Uuid::from_u128(0x1900_0001);
    let opponent_front_uuid = Uuid::from_u128(0x2900_0001);
    let straight_back_uuid = Uuid::from_u128(0x2900_0002);
    let diagonal_back_uuid = Uuid::from_u128(0x2900_0003);

    let player_melee = abnormality_with_basic_attack(
        "straight_backline_player_melee",
        player_melee_uuid,
        100,
        20,
        2,
        move_speed_units_per_ms,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 1_500,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );
    let opponent_front = abnormality_with_basic_attack(
        "straight_backline_opponent_front",
        opponent_front_uuid,
        20,
        1,
        0,
        move_speed_units_per_ms,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );
    let make_backliner = |id: &str, uuid: Uuid| {
        abnormality_with_basic_attack(
            id,
            uuid,
            100,
            1,
            0,
            move_speed_units_per_ms,
            BasicAttackDef {
                range_tiles: 3,
                interval_ms: 60_000,
                windup_ms: 0,
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 3_000_000,
                    collision: Default::default(),
                },
            },
        )
    };

    let game_data = game_data_from_abnormalities(vec![
        player_melee,
        opponent_front,
        make_backliner("straight_backline_opponent_straight", straight_back_uuid),
        make_backliner("straight_backline_opponent_diagonal", diagonal_back_uuid),
    ]);

    let player = deck_single_unit(
        Uuid::from_u128(0xE900_0001),
        player_melee_uuid,
        Position::new(2, 4),
    );
    let opponent = deck_with_units(vec![
        (
            Uuid::from_u128(0xF900_0001),
            opponent_front_uuid,
            Position::new(2, 3),
        ),
        (
            Uuid::from_u128(0xF900_0002),
            straight_back_uuid,
            Position::new(2, 1),
        ),
        (
            Uuid::from_u128(0xF900_0003),
            diagonal_back_uuid,
            Position::new(1, 1),
        ),
    ]);

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        6060,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "melee_prefers_straight_backline_target_after_frontliner_dies",
        &result.timeline,
    );

    let player_id = find_unit_instance_id(&result.timeline, Side::Player, player_melee_uuid);
    let front_id = find_unit_instance_id(&result.timeline, Side::Opponent, opponent_front_uuid);

    let front_death_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitDied {
                unit_instance_id, ..
            } if *unit_instance_id == front_id => Some(entry.time_ms),
            _ => None,
        })
        .expect("expected frontliner to die");

    let next_move = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitMoved {
                unit_instance_id,
                from,
                to,
            } if *unit_instance_id == player_id && entry.time_ms > front_death_time => {
                Some((entry.time_ms, *from, *to))
            }
            _ => None,
        })
        .expect("expected melee to move toward exposed backline after frontliner death");

    assert!(
        next_move.0 < front_death_time + 1_500,
        "melee should retarget before the next attack cadence after frontliner death: death={front_death_time}, next_move={next_move:?}"
    );
    assert_eq!(
        next_move.1,
        Position::new(2, 4),
        "expected melee to still be on its original file when the frontliner dies: next_move={next_move:?}"
    );
    assert_eq!(
        next_move.2,
        Position::new(2, 3),
        "melee should prefer the straight exposed backline approach over the equal-distance diagonal option after frontliner death: next_move={next_move:?}"
    );
}

#[test]
fn chebyshev_range_allows_diagonal_in_range() {
    let attacker_base_uuid = Uuid::from_u128(0x1400_0001);
    let target_base_uuid = Uuid::from_u128(0x2500_0001);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 2,
            interval_ms: 1,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let target = abnormality_with_basic_attack(
        "target",
        target_base_uuid,
        10,
        1,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, target]);

    let player = deck_single_unit(
        Uuid::from_u128(0xA400_0001),
        attacker_base_uuid,
        Position::new(0, 0),
    );
    let opponent = deck_single_unit(
        Uuid::from_u128(0xB400_0001),
        target_base_uuid,
        Position::new(2, 2),
    );

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        555,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export("chebyshev_range_allows_diagonal_in_range", &result.timeline);

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);

    let has_hit = result
        .timeline
        .entries
        .iter()
        .any(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == Some(attacker_id)
                && *target_instance_id == target_id
                && *reason == HpChangeReason::BasicAttack =>
            {
                true
            }
            _ => false,
        });
    assert!(
        has_hit,
        "diagonal target should be in range under chebyshev"
    );

    let has_miss = result
        .timeline
        .entries
        .iter()
        .any(|entry| match &entry.event {
            TimelineEvent::AttackMiss {
                attacker_instance_id,
                target_instance_id,
                ..
            } if *attacker_instance_id == attacker_id && *target_instance_id == target_id => true,
            _ => false,
        });
    assert!(!has_miss, "diagonal in-range attack should not miss");
}

#[test]
fn windup_locks_basic_attack_until_resolve() {
    let attacker_base_uuid = Uuid::from_u128(0x1500_0001);
    let target_base_uuid = Uuid::from_u128(0x2600_0001);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 3,
            interval_ms: 1,
            windup_ms: 5,
            delivery: DeliveryDef::Instant,
        },
    );

    let target = abnormality_with_basic_attack(
        "target",
        target_base_uuid,
        25,
        1,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, target]);

    let player = deck_single_unit(
        Uuid::from_u128(0xA500_0001),
        attacker_base_uuid,
        Position::new(0, 0),
    );
    let opponent = deck_single_unit(
        Uuid::from_u128(0xB500_0001),
        target_base_uuid,
        Position::new(1, 0),
    );

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        31337,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export("windup_locks_basic_attack_until_resolve", &result.timeline);

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);

    let attack_starts: Vec<u64> = result
        .timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::AttackStart {
                attacker_instance_id,
                ..
            } if *attacker_instance_id == attacker_id => Some(entry.time_ms),
            _ => None,
        })
        .collect();

    assert!(
        attack_starts.len() >= 2,
        "expected multiple AttackStart entries for lock verification"
    );

    for pair in attack_starts.windows(2) {
        let delta = pair[1].saturating_sub(pair[0]);
        assert!(
            delta >= 5,
            "AttackStart should be delayed until windup completes"
        );
    }
}

#[test]
fn instant_basic_attack_without_explicit_windup_uses_default_melee_windup() {
    let attacker_base_uuid = Uuid::from_u128(0x1600_0001);
    let target_base_uuid = Uuid::from_u128(0x2700_0001);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 1_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let target = abnormality_with_basic_attack(
        "target",
        target_base_uuid,
        100,
        1,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, target]);
    let player = deck_single_unit(
        Uuid::from_u128(0xA600_0001),
        attacker_base_uuid,
        Position::new(0, 0),
    );
    let opponent = deck_single_unit(
        Uuid::from_u128(0xB600_0001),
        target_base_uuid,
        Position::new(1, 0),
    );

    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 20260319);
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    let attack_start_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match entry.event {
            TimelineEvent::AttackStart {
                kind: Some(game_core::game::battle::timeline::AttackKind::Auto),
                ..
            } => Some(entry.time_ms),
            _ => None,
        })
        .expect("missing AttackStart");

    let attack_resolve_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match entry.event {
            TimelineEvent::AttackResolve {
                kind: Some(game_core::game::battle::timeline::AttackKind::Auto),
                ..
            } => Some(entry.time_ms),
            _ => None,
        })
        .expect("missing AttackResolve");

    assert_eq!(
        attack_resolve_time.saturating_sub(attack_start_time),
        DEFAULT_INSTANT_BASIC_ATTACK_WINDUP_MS as u64
    );
}

#[test]
fn projectile_basic_attack_with_zero_windup_stays_instant_at_start() {
    let attacker_base_uuid = Uuid::from_u128(0x1600_0002);
    let target_base_uuid = Uuid::from_u128(0x2700_0002);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 3,
            interval_ms: 1_000,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: TILE_UNITS_PER_TILE as u32,
                collision: Default::default(),
            },
        },
    );

    let target = abnormality_with_basic_attack(
        "target",
        target_base_uuid,
        100,
        1,
        0,
        3_000,
        BasicAttackDef {
            range_tiles: 1,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, target]);
    let player = deck_single_unit(
        Uuid::from_u128(0xA600_0002),
        attacker_base_uuid,
        Position::new(0, 0),
    );
    let opponent = deck_single_unit(
        Uuid::from_u128(0xB600_0002),
        target_base_uuid,
        Position::new(3, 0),
    );

    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 20260320);
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    let attack_start_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match entry.event {
            TimelineEvent::AttackStart {
                kind: Some(game_core::game::battle::timeline::AttackKind::Auto),
                ..
            } => Some(entry.time_ms),
            _ => None,
        })
        .expect("missing AttackStart");

    let attack_resolve_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match entry.event {
            TimelineEvent::AttackResolve {
                kind: Some(game_core::game::battle::timeline::AttackKind::Auto),
                ..
            } => Some(entry.time_ms),
            _ => None,
        })
        .expect("missing AttackResolve");

    assert_eq!(attack_resolve_time, attack_start_time);
}
