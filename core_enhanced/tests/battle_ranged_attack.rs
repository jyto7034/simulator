mod common;

use std::sync::Arc;

use game_core::game::ability::DeliveryDef;
use game_core::game::battle::core::movement::types::DEFAULT_UNIT_RADIUS;
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::ids::UnitInstanceId;
use game_core::game::battle::replay::{types::TimelineReplayerConfig, TimelineReplayer};
use game_core::game::battle::scenario::{
    BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioEvent, ScenarioEventId,
    ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger, ScenarioUnitRef, ScenarioUnitSpawn,
    WinCondition,
};
use game_core::game::battle::timeline::{HpChangeReason, TimelineEvent};
use game_core::game::battle::types::{BattleUnitDraft, BattleUnitSource};
use game_core::game::battle::validation::{
    TimelineExpectedCounts, TimelineValidator, TimelineValidatorConfig,
};
use game_core::game::data::{
    abnormality_data::{
        AbnormalityMetadata, BasicAttackDef, MovementDef, DEFAULT_INSTANT_BASIC_ATTACK_WINDUP_MS,
    },
    GameDataBase, GameDataBuilder,
};
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::growth::GrowthStack;
use game_core::game::resources::Position;
use uuid::Uuid;

fn unit_draft(owned_uuid: Uuid, base_uuid: Uuid) -> BattleUnitDraft {
    BattleUnitDraft {
        owned_uuid,
        source: BattleUnitSource::Abnormality { base_uuid },
        level: Tier::I,
        growth_stacks: GrowthStack::new(),
        equipped_items: vec![],
        equipped_item_enhancements: vec![],
    }
}

fn spawn_group(
    id: &str,
    side: Side,
    required_for_victory: bool,
    units: Vec<(Uuid, Uuid, Position)>,
) -> ScenarioSpawnGroup {
    let group_id = ScenarioGroupId::new(id);
    let spawns = units
        .into_iter()
        .enumerate()
        .map(
            |(index, (owned_uuid, base_uuid, position))| ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("{}_{}", group_id.0, index)),
                side,
                draft: unit_draft(owned_uuid, base_uuid),
                position,
                instance_salt: index as u32,
            },
        )
        .collect();

    ScenarioSpawnGroup {
        id: group_id,
        side,
        required_for_victory,
        enemy_movement_plan: None,
        spawns,
    }
}

fn battle_scenario(
    player_units: Vec<(Uuid, Uuid, Position)>,
    opponent_units: Vec<(Uuid, Uuid, Position)>,
) -> BattleScenario {
    let player_group_id = "player_initial";
    let enemy_group_id = "enemy_initial";
    BattleScenario {
        battlefield: BattleFieldSpec {
            width: common::BOARD_SIZE.0,
            height: common::BOARD_SIZE.1,
            valid_tiles: Vec::new(),
            obstacles: Vec::new(),
        },
        artifacts: Vec::new(),
        groups: vec![
            spawn_group(player_group_id, Side::Player, false, player_units),
            spawn_group(enemy_group_id, Side::Opponent, true, opponent_units),
        ],
        events: vec![
            ScenarioEvent {
                id: ScenarioEventId::new("spawn_player_initial"),
                trigger: ScenarioTrigger::AtBattleStart,
                action: ScenarioAction::SpawnGroup {
                    group_id: ScenarioGroupId::new(player_group_id),
                },
                once: true,
            },
            ScenarioEvent {
                id: ScenarioEventId::new("spawn_enemy_initial"),
                trigger: ScenarioTrigger::AtBattleStart,
                action: ScenarioAction::SpawnGroup {
                    group_id: ScenarioGroupId::new(enemy_group_id),
                },
                once: true,
            },
        ],
        win_condition: WinCondition::AllRequiredEnemyGroupsDefeated,
        tactical_plan: game_core::game::battle::scenario::TacticalPlan::default(),
    }
}

fn one_vs_one_scenario(
    player_owned_uuid: Uuid,
    player_base_uuid: Uuid,
    player_position: Position,
    opponent_owned_uuid: Uuid,
    opponent_base_uuid: Uuid,
    opponent_position: Position,
) -> BattleScenario {
    battle_scenario(
        vec![(player_owned_uuid, player_base_uuid, player_position)],
        vec![(opponent_owned_uuid, opponent_base_uuid, opponent_position)],
    )
}

fn game_data_from_abnormalities(items: Vec<AbnormalityMetadata>) -> Arc<GameDataBase> {
    GameDataBuilder::empty()
        .with_abnormalities(items)
        .build_arc()
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
            radius_units: 350_000,
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

fn run_battle(
    scenario: BattleScenario,
    game_data: Arc<GameDataBase>,
    seed: u64,
) -> game_core::game::battle::types::BattleResult {
    let mut battle = BattleCore::new_from_scenario(scenario, game_data, seed);
    battle.run_battle().unwrap()
}

fn has_basic_attack_damage(
    timeline: &game_core::game::battle::timeline::Timeline,
    source_id: UnitInstanceId,
    target_id: UnitInstanceId,
) -> bool {
    timeline.entries.iter().any(|entry| match &entry.event {
        TimelineEvent::HpChanged {
            source_instance_id,
            target_instance_id,
            reason,
            ..
        } if *source_instance_id == Some(source_id)
            && *target_instance_id == target_id
            && *reason == HpChangeReason::BasicAttack =>
        {
            true
        }
        _ => false,
    })
}

#[test]
fn ranged_basic_attack_projectile_hits_after_flight_time_and_damages_target() {
    let attacker_base_uuid = Uuid::from_u128(0xCAFE_0001);
    let target_base_uuid = Uuid::from_u128(0xCAFE_0002);
    let attacker_pos = Position::new(0, 0);
    let target_pos = Position::new(3, 0);

    let attacker = abnormality_with_basic_attack(
        "ranged_attacker",
        attacker_base_uuid,
        999,
        1000,
        0,
        0,
        BasicAttackDef {
            range_units: 3.0,
            interval_ms: 100,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 3_000,
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
        0,
        BasicAttackDef {
            range_units: 3.0,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, target]);
    let scenario = one_vs_one_scenario(
        Uuid::from_u128(0xDADA_0001),
        attacker_base_uuid,
        attacker_pos,
        Uuid::from_u128(0xDADA_0002),
        target_base_uuid,
        target_pos,
    );
    let expected = TimelineExpectedCounts::from_scenario(&scenario);
    let result = run_battle(scenario, game_data.clone(), 12345);

    common::write_timeline_export(
        "ranged_basic_attack_projectile_hits_after_flight_time",
        &result.timeline,
    );

    let player_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let opponent_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);
    let attack_time_ms = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                ..
            } if *attacker_instance_id == player_id && *target_instance_id == opponent_id => {
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
            } if *target_instance_id == opponent_id && *reason == HpChangeReason::BasicAttack => {
                Some((entry.time_ms, *hp_before, *hp_after))
            }
            _ => None,
        })
        .expect("missing HpChanged(BasicAttack) for target");

    assert_eq!(hp_before, 10);
    assert_eq!(hp_after, 0);

    let (projectile_id, expected_impact_ms) = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::BasicAttackProjectileLaunched {
                projectile_id,
                attacker_instance_id,
                target_instance_id,
                start,
                aim,
                fired_at_ms,
                expected_impact_time_ms,
            } if *attacker_instance_id == player_id && *target_instance_id == opponent_id => {
                let start_world = start.to_world();
                let aim_world = aim.to_world();
                assert!(start_world.distance(aim_world) > DEFAULT_UNIT_RADIUS);
                assert_eq!(*fired_at_ms, attack_time_ms);
                assert!(*expected_impact_time_ms > *fired_at_ms);
                Some((*projectile_id, *expected_impact_time_ms))
            }
            _ => None,
        })
        .expect("missing basic attack projectile launch timeline event");

    let impact_time_ms = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::BasicAttackProjectileImpacted {
                projectile_id: actual_projectile_id,
                attacker_instance_id,
                target_instance_id,
                hit: true,
                ..
            } if *actual_projectile_id == projectile_id
                && *attacker_instance_id == player_id
                && *target_instance_id == opponent_id =>
            {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("missing basic attack projectile impact timeline event");
    assert!(impact_time_ms <= expected_impact_ms);
    assert_eq!(hp_time_ms, impact_time_ms);

    let mut replay_config = TimelineReplayerConfig::default();
    replay_config.validate_unit_base_uuid = true;
    TimelineReplayer::new(game_data.clone(), replay_config)
        .replay(&result.timeline)
        .unwrap();

    TimelineValidator::new(TimelineValidatorConfig::default())
        .validate(&result.timeline, Some(expected), Some(game_data.as_ref()))
        .unwrap();
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
        0,
        BasicAttackDef {
            range_units: 4.0,
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
        0,
        BasicAttackDef {
            range_units: 4.0,
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
        0,
        BasicAttackDef {
            range_units: 1.0,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, finisher, target]);
    let scenario = battle_scenario(
        vec![
            (
                Uuid::from_u128(0xA100_0001),
                attacker_base_uuid,
                Position::new(0, 0),
            ),
            (
                Uuid::from_u128(0xA100_0002),
                finisher_base_uuid,
                Position::new(0, 1),
            ),
        ],
        vec![(
            Uuid::from_u128(0xB100_0001),
            target_base_uuid,
            Position::new(3, 0),
        )],
    );
    let result = run_battle(scenario, game_data, 2024);

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

    let attacker_hit_after_death = result.timeline.entries.iter().any(|entry| {
        matches!(
            &entry.event,
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                ..
            } if *source_instance_id == Some(attacker_id)
                && *target_instance_id == target_id
                && entry.time_ms >= target_death_time
        )
    });
    assert!(!attacker_hit_after_death);
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
        0,
        BasicAttackDef {
            range_units: 4.0,
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
        0,
        BasicAttackDef {
            range_units: 1.0,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, target]);
    let scenario = one_vs_one_scenario(
        Uuid::from_u128(0xA300_0001),
        attacker_base_uuid,
        Position::new(0, 0),
        Uuid::from_u128(0xB300_0001),
        target_base_uuid,
        Position::new(2, 0),
    );
    let result = run_battle(scenario, game_data, 9001);

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
fn ranged_vs_ranged_both_sides_land_projectile_hits() {
    let player_base_uuid = Uuid::from_u128(0x1500_0001);
    let opponent_base_uuid = Uuid::from_u128(0x2600_0001);

    let player_unit = abnormality_with_basic_attack(
        "ranged_player",
        player_base_uuid,
        40,
        8,
        0,
        0,
        BasicAttackDef {
            range_units: 4.0,
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
        0,
        BasicAttackDef {
            range_units: 4.0,
            interval_ms: 1_000,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 3_000,
                collision: Default::default(),
            },
        },
    );

    let game_data = game_data_from_abnormalities(vec![player_unit, opponent_unit]);
    let scenario = one_vs_one_scenario(
        Uuid::from_u128(0xA400_0001),
        player_base_uuid,
        Position::new(0, 0),
        Uuid::from_u128(0xB400_0001),
        opponent_base_uuid,
        Position::new(3, 0),
    );
    let result = run_battle(scenario, game_data, 2026);

    let player_id = find_unit_instance_id(&result.timeline, Side::Player, player_base_uuid);
    let opponent_id = find_unit_instance_id(&result.timeline, Side::Opponent, opponent_base_uuid);
    assert!(has_basic_attack_damage(
        &result.timeline,
        player_id,
        opponent_id
    ));
    assert!(has_basic_attack_damage(
        &result.timeline,
        opponent_id,
        player_id
    ));
}

#[test]
fn diagonal_basic_attack_uses_continuous_body_range_not_chebyshev_tiles() {
    let attacker_base_uuid = Uuid::from_u128(0x1500_1001);
    let target_base_uuid = Uuid::from_u128(0x2600_1001);

    let attacker = abnormality_with_basic_attack(
        "continuous_diagonal_attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        0,
        BasicAttackDef {
            range_units: 2.2,
            interval_ms: 1,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );
    let target = abnormality_with_basic_attack(
        "continuous_diagonal_target",
        target_base_uuid,
        10,
        1,
        0,
        0,
        BasicAttackDef {
            range_units: 1.0,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let game_data = game_data_from_abnormalities(vec![attacker, target]);
    let scenario = one_vs_one_scenario(
        Uuid::from_u128(0xA400_1001),
        attacker_base_uuid,
        Position::new(0, 0),
        Uuid::from_u128(0xB400_1001),
        target_base_uuid,
        Position::new(2, 2),
    );
    let result = run_battle(scenario, game_data, 556);

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);
    assert!(
        has_basic_attack_damage(&result.timeline, attacker_id, target_id),
        "diagonal target should be hittable when Euclidean body distance is within range + radii"
    );
}

#[test]
fn windup_locks_basic_attack_until_resolve() {
    let attacker_base_uuid = Uuid::from_u128(0x1500_0002);
    let target_base_uuid = Uuid::from_u128(0x2600_0002);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        0,
        BasicAttackDef {
            range_units: 3.0,
            interval_ms: 1,
            windup_ms: 5,
            delivery: DeliveryDef::Instant,
        },
    );
    let target = abnormality_with_basic_attack(
        "target",
        target_base_uuid,
        10,
        1,
        0,
        0,
        BasicAttackDef {
            range_units: 1.0,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let scenario = one_vs_one_scenario(
        Uuid::from_u128(0xA500_0001),
        attacker_base_uuid,
        Position::new(0, 0),
        Uuid::from_u128(0xB500_0001),
        target_base_uuid,
        Position::new(2, 0),
    );
    let game_data = game_data_from_abnormalities(vec![attacker, target]);
    let result = run_battle(scenario, game_data, 777);

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);
    let start_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::AttackStart {
                attacker_instance_id,
                target_instance_id,
                ..
            } if *attacker_instance_id == attacker_id && *target_instance_id == target_id => {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("missing AttackStart");
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

    assert_eq!(resolve_time, start_time + 5);
}

#[test]
fn instant_basic_attack_without_explicit_windup_uses_default_melee_windup() {
    let attacker_base_uuid = Uuid::from_u128(0x1500_0003);
    let target_base_uuid = Uuid::from_u128(0x2600_0003);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        0,
        BasicAttackDef {
            range_units: 3.0,
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
        0,
        BasicAttackDef {
            range_units: 1.0,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let scenario = one_vs_one_scenario(
        Uuid::from_u128(0xA600_0001),
        attacker_base_uuid,
        Position::new(0, 0),
        Uuid::from_u128(0xB600_0001),
        target_base_uuid,
        Position::new(2, 0),
    );
    let game_data = game_data_from_abnormalities(vec![attacker, target]);
    let result = run_battle(scenario, game_data, 778);

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);
    let start_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::AttackStart {
                attacker_instance_id,
                target_instance_id,
                ..
            } if *attacker_instance_id == attacker_id && *target_instance_id == target_id => {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("missing AttackStart");
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

    assert_eq!(
        resolve_time,
        start_time + u64::from(DEFAULT_INSTANT_BASIC_ATTACK_WINDUP_MS)
    );
}

#[test]
fn projectile_basic_attack_with_zero_windup_stays_instant_at_start() {
    let attacker_base_uuid = Uuid::from_u128(0x1500_0004);
    let target_base_uuid = Uuid::from_u128(0x2600_0004);

    let attacker = abnormality_with_basic_attack(
        "attacker",
        attacker_base_uuid,
        100,
        10,
        0,
        0,
        BasicAttackDef {
            range_units: 3.0,
            interval_ms: 1,
            windup_ms: 0,
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 1_000,
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
        0,
        BasicAttackDef {
            range_units: 1.0,
            interval_ms: 60_000,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
    );

    let scenario = one_vs_one_scenario(
        Uuid::from_u128(0xA700_0001),
        attacker_base_uuid,
        Position::new(0, 0),
        Uuid::from_u128(0xB700_0001),
        target_base_uuid,
        Position::new(2, 0),
    );
    let game_data = game_data_from_abnormalities(vec![attacker, target]);
    let result = run_battle(scenario, game_data, 779);

    let attacker_id = find_unit_instance_id(&result.timeline, Side::Player, attacker_base_uuid);
    let target_id = find_unit_instance_id(&result.timeline, Side::Opponent, target_base_uuid);
    let start_time = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::AttackStart {
                attacker_instance_id,
                target_instance_id,
                ..
            } if *attacker_instance_id == attacker_id && *target_instance_id == target_id => {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("missing AttackStart");
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

    assert_eq!(resolve_time, start_time);
}
