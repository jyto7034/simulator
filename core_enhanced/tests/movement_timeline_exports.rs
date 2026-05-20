mod common;

use std::path::PathBuf;
use std::sync::Arc;

use game_core::game::battle::core::BattleCore;
use game_core::game::battle::scenario::{
    BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioEvent, ScenarioEventId,
    ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger, ScenarioUnitRef, ScenarioUnitSpawn,
    WinCondition,
};
use game_core::game::battle::timeline::{MovementStopReason, TimelineEvent};
use game_core::game::battle::types::{BattleUnitDraft, BattleUnitSource};
use game_core::game::data::abnormality_data::AbnormalityMetadata;
use game_core::game::data::{GameDataBase, GameDataBuilder};
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::growth::GrowthStack;
use game_core::game::resources::Position;
use uuid::Uuid;

fn abnormality(
    id: &str,
    uuid: Uuid,
    range_units: u8,
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
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: Default::default(),
        resonance: Default::default(),
        skill_id: None,
    };
    meta.basic_attack.range_units = f32::from(range_units);
    meta.basic_attack.interval_ms = attack_interval_ms;
    meta.movement.speed_units_per_ms = speed_units_per_ms;
    meta
}

fn minimal_game_data(abnormalities: Vec<AbnormalityMetadata>) -> Arc<GameDataBase> {
    GameDataBuilder::empty()
        .with_abnormalities(abnormalities)
        .build_arc()
}

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

fn export_timeline(name: &str, timeline: &game_core::game::battle::timeline::Timeline) -> PathBuf {
    common::write_timeline_export(name, timeline)
}

fn any_unit_moved(timeline: &game_core::game::battle::timeline::Timeline) -> bool {
    timeline
        .entries
        .iter()
        .any(|e| matches!(e.event, TimelineEvent::MovementSegmentStarted { .. }))
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

    let scenario = battle_scenario(
        vec![
            (mover_owned, mover_base, Position::new(3, 7)),
            (blocker1_owned, blocker_base, Position::new(3, 6)),
            (blocker2_owned, blocker_base, Position::new(3, 5)),
        ],
        vec![(enemy_owned, enemy_base, Position::new(3, 0))],
    );
    let mut battle = BattleCore::new_from_scenario(scenario, game_data, 123);
    let result = battle.run_battle().expect("battle runs");

    assert!(any_unit_moved(&result.timeline));

    let path = export_timeline("movement_detour_blockers", &result.timeline);
    println!("wrote timeline: {}", path.display());
}

#[test]
fn movement_ignores_legacy_blocked_ring_and_uses_continuous_space_exports_timeline() {
    // Scenario:
    // - Player mover at (3,7) wants to get in range 1 of enemy at (3,0).
    // - Every legacy destination tile within range is occupied by allied blockers.
    // - Continuous movement no longer treats this tile ring as authoritative,
    //   so movement should still occur.
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

    let scenario = battle_scenario(units, vec![(enemy_owned, enemy_base, Position::new(3, 0))]);
    let mut battle = BattleCore::new_from_scenario(scenario, game_data, 999);
    let result = battle.run_battle().expect("battle runs");

    assert!(any_unit_moved(&result.timeline));

    let path = export_timeline("movement_continuous_ignores_blocked_ring", &result.timeline);
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

    let scenario = battle_scenario(units, vec![(enemy_owned, enemy_base, Position::new(3, 0))]);
    let mut battle = BattleCore::new_from_scenario(scenario, game_data, 4242);
    let result = battle.run_battle().expect("battle runs");

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
