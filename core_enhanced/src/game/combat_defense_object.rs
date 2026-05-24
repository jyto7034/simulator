use uuid::Uuid;

use crate::game::{
    battle::{
        scenario::{
            ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitSpawn, TacticalPlan, WinCondition,
        },
        types::{BattleUnitDraft, BattleUnitSource, DeploymentAffinity, UnitCombatProfile},
    },
    combat_mission_policy::DEFAULT_DEFENSE_POINT_ID,
    data::abnormality_data::{BasicAttackDef, MovementDef, ResonanceDef},
    enums::{Side, Tier},
    growth::GrowthStack,
    resources::Position,
    stats::UnitStats,
};

pub const DEFAULT_DEFENSE_OBJECT_GROUP: &str = "defense_object";
const DEFAULT_DEFENSE_OBJECT_OWNED_UUID: u128 = 0xDEF0_0000_0000_0000_0000_0000_0000_0001;
const DEFAULT_DEFENSE_OBJECT_BASE_UUID: u128 = 0xDEF0_0000_0000_0000_0000_0000_0000_0002;
const DEFAULT_DEFENSE_OBJECT_HP: u32 = 350;

pub fn defense_object_group_for_win_condition(
    win_condition: &WinCondition,
    tactical_plan: &TacticalPlan,
) -> Option<ScenarioSpawnGroup> {
    let unit_ref = match win_condition {
        WinCondition::ProtectUnit { unit_ref } => unit_ref,
        _ => return None,
    };
    let position = tactical_plan
        .points
        .iter()
        .find(|point| point.id.0 == DEFAULT_DEFENSE_POINT_ID)
        .or_else(|| tactical_plan.points.first())
        .map(|point| point.position)
        .unwrap_or(Position::new(0, 0));
    Some(ScenarioSpawnGroup {
        id: ScenarioGroupId::new(DEFAULT_DEFENSE_OBJECT_GROUP),
        side: Side::Player,
        required_for_victory: false,
        enemy_movement_plan: None,
        spawns: vec![ScenarioUnitSpawn {
            unit_ref: unit_ref.clone(),
            side: Side::Player,
            draft: BattleUnitDraft {
                owned_uuid: Uuid::from_u128(DEFAULT_DEFENSE_OBJECT_OWNED_UUID),
                source: BattleUnitSource::DefenseObject {
                    base_uuid: Uuid::from_u128(DEFAULT_DEFENSE_OBJECT_BASE_UUID),
                    profile: default_defense_object_profile(),
                },
                level: Tier::I,
                growth_stacks: GrowthStack::new(),
                equipped_items: vec![],
                equipped_item_enhancements: vec![],
            },
            position,
            instance_salt: 0,
        }],
    })
}

fn default_defense_object_profile() -> UnitCombatProfile {
    let basic_attack = BasicAttackDef {
        range_units: 0.0,
        interval_ms: 1500,
        windup_ms: 0,
        delivery: Default::default(),
    };
    let movement = MovementDef {
        speed_units_per_ms: 0,
        radius_units: 350_000,
    };
    let mut stats = UnitStats::with_values(
        DEFAULT_DEFENSE_OBJECT_HP,
        DEFAULT_DEFENSE_OBJECT_HP,
        0,
        8,
        basic_attack.interval_ms,
    );
    stats.magic_resist = 8;
    stats.move_speed_units_per_ms = 0;

    UnitCombatProfile {
        stats,
        basic_attack,
        movement,
        resonance: ResonanceDef::default(),
        skill_id: None,
        deployment_affinity: DeploymentAffinity::Any,
        block_capacity: 0,
        block_radius_units: 0.0,
        blockable: false,
    }
}
