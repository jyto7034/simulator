mod common;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::timeline::{MovementStopReason, TimelineEvent};
use game_core::game::battle::types::{OwnedUnit, PlayerDeckInfo};
use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::artifact_data::ArtifactDatabase;
use game_core::game::data::bonus_data::BonusDatabase;
use game_core::game::data::equipment_data::EquipmentDatabase;
use game_core::game::data::pve_data::PveEncounterDatabase;
use game_core::game::data::random_event_data::RandomEventDatabase;
use game_core::game::data::shop_data::ShopDatabase;
use game_core::game::data::skill_data::SkillDatabase;
use game_core::game::data::GameDataBase;
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::growth::GrowthStack;
use uuid::Uuid;

fn abnormality(
    id: &str,
    uuid: Uuid,
    range_tiles: u8,
    attack_interval_ms: u64,
    speed_units_per_ms: u32,
) -> AbnormalityMetadata {
    let mut meta = AbnormalityMetadata {
        id: id.to_string(),
        uuid,
        name: id.to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 999,
        attack: 1,
        defense: 0,
        movement: Default::default(),
        basic_attack: Default::default(),
        resonance: Default::default(),
        skill_id: None,
    };
    meta.basic_attack.range_tiles = range_tiles;
    meta.basic_attack.interval_ms = attack_interval_ms;
    meta.movement.speed_units_per_ms = speed_units_per_ms;
    meta
}

fn minimal_game_data(abnormalities: Vec<AbnormalityMetadata>) -> Arc<GameDataBase> {
    Arc::new(GameDataBase::new(
        Arc::new(AbnormalityDatabase::new(abnormalities)),
        Arc::new(ArtifactDatabase::new(vec![])),
        Arc::new(EquipmentDatabase::new(vec![])),
        Arc::new(ShopDatabase::new(vec![])),
        Arc::new(BonusDatabase::new(vec![])),
        Arc::new(RandomEventDatabase::new(vec![])),
        Arc::new(PveEncounterDatabase::new(vec![])),
        Arc::new(SkillDatabase::new(vec![])),
        common::empty_event_pools(),
    ))
}

fn deck(
    units: Vec<(Uuid, Uuid, Position)>,
    side: Side,
) -> (PlayerDeckInfo, Vec<(Uuid, Uuid, Position, Side)>) {
    let mut positions = HashMap::new();
    let mut owned_units = Vec::new();
    let mut debug_units = Vec::new();

    for (owned_uuid, base_uuid, pos) in units {
        positions.insert(owned_uuid, pos);
        owned_units.push(OwnedUnit {
            owned_uuid,
            base_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        });
        debug_units.push((owned_uuid, base_uuid, pos, side));
    }

    (
        PlayerDeckInfo {
            units: owned_units,
            artifacts: vec![],
            positions,
        },
        debug_units,
    )
}

fn export_timeline(name: &str, timeline: &game_core::game::battle::timeline::Timeline) -> PathBuf {
    common::write_timeline_export(name, timeline)
}

fn any_unit_moved(timeline: &game_core::game::battle::timeline::Timeline) -> bool {
    timeline
        .entries
        .iter()
        .any(|e| matches!(e.event, TimelineEvent::UnitMoved { .. }))
}

fn any_movement_stopped_reason(
    timeline: &game_core::game::battle::timeline::Timeline,
    reason: MovementStopReason,
) -> bool {
    timeline.entries.iter().any(|e| {
        matches!(
            e.event,
            TimelineEvent::MovementStopped { reason: r, .. } if r == reason
        )
    })
}

#[test]
fn movement_detours_around_static_blockers_exports_timeline() {
    // Scenario:
    // - Player mover at (3,7) wants to reach enemy at (3,0).
    // - Two allied blockers occupy the direct lane, forcing a detour.
    let mover_base = Uuid::from_u128(0xB000_0001);
    let enemy_base = Uuid::from_u128(0xB000_0002);
    let blocker_base = Uuid::from_u128(0xB000_00FF);

    // Keep attacks out of the sim by scheduling them far beyond MAX_BATTLE_TIME_MS.
    let attack_interval_ms = 1_000_000;

    let game_data = minimal_game_data(vec![
        abnormality("mover", mover_base, 1, attack_interval_ms, 10_000),
        abnormality("enemy", enemy_base, 1, attack_interval_ms, 10_000),
        // Blockers should never move: give them an absurd range so they "acquire target" immediately.
        abnormality("blocker", blocker_base, 99, attack_interval_ms, 10_000),
    ]);

    let mover_owned = Uuid::from_u128(0xD000_0001);
    let blocker1_owned = Uuid::from_u128(0xD000_00F1);
    let blocker2_owned = Uuid::from_u128(0xD000_00F2);
    let enemy_owned = Uuid::from_u128(0xD000_0002);

    let (player_deck, _debug_player) = deck(
        vec![
            (mover_owned, mover_base, Position::new(3, 7)),
            (blocker1_owned, blocker_base, Position::new(3, 6)),
            (blocker2_owned, blocker_base, Position::new(3, 5)),
        ],
        Side::Player,
    );
    let (opponent_deck, _debug_opponent) = deck(
        vec![(enemy_owned, enemy_base, Position::new(3, 0))],
        Side::Opponent,
    );

    let mut world = World::new();
    let mut battle = BattleCore::new(
        &player_deck,
        &opponent_deck,
        game_data,
        common::BOARD_SIZE,
        123,
    );
    let result = battle.run_battle(&mut world).expect("battle runs");

    assert!(any_unit_moved(&result.timeline));
    assert!(
        any_movement_stopped_reason(&result.timeline, MovementStopReason::TargetAcquired)
            || any_movement_stopped_reason(&result.timeline, MovementStopReason::Arrived)
    );

    let path = export_timeline("movement_detour_blockers", &result.timeline);
    println!("wrote timeline: {}", path.display());
}

#[test]
fn movement_unreachable_when_all_in_range_tiles_blocked_exports_timeline() {
    // Scenario:
    // - Player mover at (3,7) wants to get in range 1 of enemy at (3,0).
    // - Every empty destination tile within range is occupied by allied blockers.
    // - Result: no UnitMoved events should occur.
    let mover_base = Uuid::from_u128(0xC000_0001);
    let enemy_base = Uuid::from_u128(0xC000_0002);
    let blocker_base = Uuid::from_u128(0xC000_00FF);

    let attack_interval_ms = 1_000_000;

    let game_data = minimal_game_data(vec![
        abnormality("mover", mover_base, 1, attack_interval_ms, 10_000),
        abnormality("enemy", enemy_base, 1, attack_interval_ms, 10_000),
        abnormality("blocker", blocker_base, 99, attack_interval_ms, 10_000),
    ]);

    let mover_owned = Uuid::from_u128(0xD100_0001);
    let enemy_owned = Uuid::from_u128(0xD100_0002);

    // Block all (Chebyshev) distance<=1 tiles around enemy except the enemy tile itself.
    let blocker_positions = [
        Position::new(2, 0),
        Position::new(4, 0),
        Position::new(2, 1),
        Position::new(3, 1),
        Position::new(4, 1),
    ];
    let mut units = vec![(mover_owned, mover_base, Position::new(3, 7))];
    for (i, pos) in blocker_positions.iter().copied().enumerate() {
        units.push((Uuid::from_u128(0xD100_00F0 + i as u128), blocker_base, pos));
    }

    let (player_deck, _debug_player) = deck(units, Side::Player);
    let (opponent_deck, _debug_opponent) = deck(
        vec![(enemy_owned, enemy_base, Position::new(3, 0))],
        Side::Opponent,
    );

    let mut world = World::new();
    let mut battle = BattleCore::new(
        &player_deck,
        &opponent_deck,
        game_data,
        common::BOARD_SIZE,
        999,
    );
    let result = battle.run_battle(&mut world).expect("battle runs");

    assert!(!any_unit_moved(&result.timeline));

    let path = export_timeline("movement_unreachable_blocked_ring", &result.timeline);
    println!("wrote timeline: {}", path.display());
}

#[test]
fn movement_collision_uses_local_step_reservations_without_forced_wait_repath() {
    // Scenario:
    // - Two movers converge toward the same corridor.
    // - With step-local reservations, both should be able to progress without
    //   globally reserving the whole destination path and forcing an early WaitRepath.
    let mover_base = Uuid::from_u128(0xD200_0001);
    let enemy_base = Uuid::from_u128(0xD200_0002);
    let blocker_base = Uuid::from_u128(0xD200_00FF);

    let attack_interval_ms = 1_000_000;

    let game_data = minimal_game_data(vec![
        abnormality("mover", mover_base, 1, attack_interval_ms, 10_000),
        abnormality("enemy", enemy_base, 1, attack_interval_ms, 10_000),
        abnormality("blocker", blocker_base, 99, attack_interval_ms, 10_000),
    ]);

    let mover1_owned = Uuid::from_u128(0xD200_1001);
    let mover2_owned = Uuid::from_u128(0xD200_1002);
    let enemy_owned = Uuid::from_u128(0xD200_2001);

    // Maze-like blockers that force both movers to have the same first step_to (3,6),
    // causing a collision and triggering WaitRepath for one mover.
    let blocker_positions = [
        // y=6: only merge tile (3,6) is open.
        Position::new(0, 6),
        Position::new(1, 6),
        Position::new(2, 6),
        Position::new(4, 6),
        Position::new(5, 6),
        Position::new(6, 6),
        // block alternative first-steps adjacent to starts at y=7.
        Position::new(1, 7),
        Position::new(3, 7),
        Position::new(5, 7),
        // y=5..2: corridor down x=3 only.
        Position::new(0, 5),
        Position::new(1, 5),
        Position::new(2, 5),
        Position::new(4, 5),
        Position::new(5, 5),
        Position::new(6, 5),
        Position::new(0, 4),
        Position::new(1, 4),
        Position::new(2, 4),
        Position::new(4, 4),
        Position::new(5, 4),
        Position::new(6, 4),
        Position::new(0, 3),
        Position::new(1, 3),
        Position::new(2, 3),
        Position::new(4, 3),
        Position::new(5, 3),
        Position::new(6, 3),
        Position::new(0, 2),
        Position::new(1, 2),
        Position::new(2, 2),
        Position::new(4, 2),
        Position::new(5, 2),
        Position::new(6, 2),
        // y=1: allow (2,1) and (4,1) as in-range destinations; block (3,1).
        Position::new(0, 1),
        Position::new(1, 1),
        Position::new(3, 1),
        Position::new(5, 1),
        Position::new(6, 1),
        // y=0: keep enemy at (3,0); block other tiles to reduce alternates.
        Position::new(0, 0),
        Position::new(1, 0),
        Position::new(2, 0),
        Position::new(4, 0),
        Position::new(5, 0),
        Position::new(6, 0),
    ];

    let mut units = vec![
        (mover1_owned, mover_base, Position::new(2, 7)),
        (mover2_owned, mover_base, Position::new(4, 7)),
    ];
    for (i, pos) in blocker_positions.iter().copied().enumerate() {
        units.push((Uuid::from_u128(0xD200_00F0 + i as u128), blocker_base, pos));
    }

    let (player_deck, _debug_player) = deck(units, Side::Player);
    let (opponent_deck, _debug_opponent) = deck(
        vec![(enemy_owned, enemy_base, Position::new(3, 0))],
        Side::Opponent,
    );

    let mut world = World::new();
    let mut battle = BattleCore::new(
        &player_deck,
        &opponent_deck,
        game_data,
        common::BOARD_SIZE,
        4242,
    );
    let result = battle.run_battle(&mut world).expect("battle runs");

    assert!(any_unit_moved(&result.timeline));
    assert!(
        !any_movement_stopped_reason(&result.timeline, MovementStopReason::WaitRepath),
        "step-local reservations should avoid early wait_repath in this corridor case"
    );

    let path = export_timeline(
        "movement_collision_local_step_reservations",
        &result.timeline,
    );
    println!("wrote timeline: {}", path.display());
}
