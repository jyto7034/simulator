mod common;

use std::sync::Arc;

use game_core::game::ability::AbilityActivationDef;
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::event_log::{BattleEventLog, BattleEventLogEntry, BattleLogEvent};
use game_core::game::battle::ids::UnitInstanceId;
use game_core::game::battle::scenario::{
    BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioArtifact, ScenarioEvent,
    ScenarioEventId, ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger, ScenarioUnitRef,
    ScenarioUnitSpawn, WinCondition,
};
use game_core::game::battle::tile_range::TileRangePattern;
use game_core::game::battle::types::{BattleUnitDraft, BattleUnitSource};
use game_core::game::data::{
    abnormality_data::AbnormalityMetadata, artifact_data::ArtifactDatabase,
    equipment_data::EquipmentDatabase, GameDataBase, GameDataBuilder,
};
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::growth::GrowthStack;
use game_core::game::resources::Position;
use uuid::Uuid;

fn test_abnormality(id: &str, uuid: Uuid, max_health: u32, attack: u32) -> AbnormalityMetadata {
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
        threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
        omen_chain_id: None,
        response_complete_skill_fragment_id: None,
        movement: Default::default(),
        basic_attack: game_core::game::data::abnormality_data::BasicAttackDef {
            defense_tile_range: Some(TileRangePattern {
                include_anchor_tile: false,
                rows: vec![".X.".to_string(), ".@.".to_string(), "...".to_string()],
            }),
            ..Default::default()
        },
        resonance: Default::default(),
        skill_id: None,
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    }
}

fn unit_draft(owned_uuid: Uuid, base_uuid: Uuid, equipped_items: Vec<Uuid>) -> BattleUnitDraft {
    BattleUnitDraft {
        owned_uuid,
        source: BattleUnitSource::Abnormality { base_uuid },
        threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
        level: Tier::I,
        stat_scale: Default::default(),
        growth_stacks: GrowthStack::new(),
        equipped_items,
        equipped_item_enhancements: vec![],
    }
}

fn spawn_group(
    id: &str,
    side: Side,
    required_for_victory: bool,
    units: Vec<(Uuid, Uuid, Position, Vec<Uuid>)>,
) -> ScenarioSpawnGroup {
    let group_id = ScenarioGroupId::new(id);
    let spawns = units
        .into_iter()
        .enumerate()
        .map(
            |(index, (owned_uuid, base_uuid, position, equipped_items))| ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("{}_{}", group_id.0, index)),
                side,
                draft: unit_draft(owned_uuid, base_uuid, equipped_items),
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

fn scenario_with_loadout(
    owned_uuid: Uuid,
    base_uuid: Uuid,
    pos: Position,
    equipped_items: Vec<Uuid>,
    artifacts: Vec<Uuid>,
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
        artifacts: artifacts
            .into_iter()
            .enumerate()
            .map(|(index, base_uuid)| ScenarioArtifact {
                side: Side::Player,
                base_uuid,
                instance_salt: index as u32,
            })
            .collect(),
        groups: vec![
            spawn_group(
                player_group_id,
                Side::Player,
                false,
                vec![(owned_uuid, base_uuid, pos, equipped_items)],
            ),
            spawn_group(
                enemy_group_id,
                Side::Opponent,
                true,
                vec![(
                    Uuid::from_u128(0xAC02),
                    Uuid::from_u128(0xAB02),
                    Position::new(1, 0),
                    vec![],
                )],
            ),
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

fn game_data_for_live_item_skill_tests(force_proc_item_ids: &[&str]) -> Arc<GameDataBase> {
    let live = common::load_game_data_from_ron();

    let attacker = test_abnormality("item_skill_attacker", Uuid::from_u128(0xAB01), 200, 10);
    let target = test_abnormality("item_skill_target", Uuid::from_u128(0xAB02), 250, 1);

    let mut equipments = live
        .equipment_data
        .items
        .iter()
        .filter(|meta| matches!(meta.id.as_str(), "paradise_lost" | "resonance_pendant"))
        .cloned()
        .collect::<Vec<_>>();
    for equipment in &mut equipments {
        if !force_proc_item_ids.contains(&equipment.id.as_str()) {
            continue;
        }
        for activation in &mut equipment.ability_activations {
            let AbilityActivationDef::TriggerProc {
                proc_chance_percent,
                ..
            } = &mut activation.activation;
            *proc_chance_percent = 100;
        }
    }

    GameDataBuilder::live_defaults()
        .with_abnormalities(vec![attacker, target])
        .with_artifact_data(Arc::new(ArtifactDatabase::new(vec![])))
        .with_equipment_data(Arc::new(EquipmentDatabase::new(equipments)))
        .with_skill_data(Arc::clone(&live.skill_data))
        .build_arc()
}

fn run_battle_with_loadout(
    game_data: Arc<GameDataBase>,
    equipped_items: Vec<Uuid>,
    artifacts: Vec<Uuid>,
) -> BattleEventLog {
    let attacker_base_uuid = Uuid::from_u128(0xAB01);
    let scenario = scenario_with_loadout(
        Uuid::from_u128(0xAC01),
        attacker_base_uuid,
        Position::new(0, 0),
        equipped_items,
        artifacts,
    );

    let mut battle = BattleCore::new_from_scenario(scenario, game_data, 20260416);
    battle
        .run_battle()
        .expect("battle should complete")
        .event_log
}

fn find_unit_instance_id(
    event_log: &BattleEventLog,
    base_uuid: Uuid,
    owner: Side,
) -> UnitInstanceId {
    event_log
        .entries
        .iter()
        .find_map(|entry| match entry.event {
            BattleLogEvent::UnitSpawned {
                unit_instance_id,
                base_uuid: spawned_base_uuid,
                owner: spawned_owner,
                ..
            } if spawned_base_uuid == base_uuid && spawned_owner == owner => Some(unit_instance_id),
            _ => None,
        })
        .expect("missing UnitSpawned entry for test unit")
}

fn find_first_cast_seq(event_log: &BattleEventLog, caster: UnitInstanceId, skill_id: &str) -> u64 {
    event_log
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            BattleLogEvent::AbilityCast {
                caster_instance_id,
                skill_id: actual_skill_id,
                ..
            } if *caster_instance_id == caster && actual_skill_id == skill_id => Some(entry.seq),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing AbilityCast for {skill_id}"))
}

fn descendants_of(event_log: &BattleEventLog, root_seq: u64) -> Vec<&BattleEventLogEntry> {
    let mut relevant_seqs = vec![root_seq];
    let mut changed = true;

    while changed {
        changed = false;
        for entry in &event_log.entries {
            if entry
                .cause
                .parent_seq()
                .is_some_and(|parent_seq| relevant_seqs.contains(&parent_seq))
                && !relevant_seqs.contains(&entry.seq)
            {
                relevant_seqs.push(entry.seq);
                changed = true;
            }
        }
    }

    event_log
        .entries
        .iter()
        .filter(|entry| entry.seq != root_seq && relevant_seqs.contains(&entry.seq))
        .collect()
}

#[test]
fn live_item_and_artifact_ability_ids_resolve_to_skill_defs() {
    let game_data = common::load_game_data_from_ron();

    for ability_id in ["paradise_lost_judgement", "resonance_pendant_opening_focus"] {
        assert!(
            game_data.skill_data.get_by_id(ability_id).is_some(),
            "missing live skill definition for {ability_id}"
        );
    }
}

#[test]
fn live_on_battle_start_item_and_artifact_skills_apply_expected_self_effects() {
    let game_data = game_data_for_live_item_skill_tests(&[]);
    let pendant_uuid = game_data
        .equipment_data
        .get_by_id("resonance_pendant")
        .expect("missing resonance pendant")
        .uuid;

    let event_log = run_battle_with_loadout(game_data, vec![pendant_uuid], vec![]);
    let attacker_id = find_unit_instance_id(&event_log, Uuid::from_u128(0xAB01), Side::Player);

    let focus_seq = find_first_cast_seq(&event_log, attacker_id, "resonance_pendant_opening_focus");
    let focus_entries = descendants_of(&event_log, focus_seq);
    assert!(focus_entries.iter().any(|entry| matches!(
        entry.event,
        BattleLogEvent::ResonanceChanged {
            unit_instance_id,
            before,
            after,
            ..
        } if unit_instance_id == attacker_id && after.saturating_sub(before) == 30
    )));
}

#[test]
fn live_on_attack_proc_item_skills_apply_expected_debuffs_when_forced_to_proc() {
    let game_data = game_data_for_live_item_skill_tests(&["paradise_lost"]);
    let paradise_uuid = game_data
        .equipment_data
        .get_by_id("paradise_lost")
        .expect("missing paradise lost")
        .uuid;

    let event_log = run_battle_with_loadout(game_data, vec![paradise_uuid], vec![]);
    let attacker_id = find_unit_instance_id(&event_log, Uuid::from_u128(0xAB01), Side::Player);
    let target_id = find_unit_instance_id(&event_log, Uuid::from_u128(0xAB02), Side::Opponent);

    let judgement_seq = find_first_cast_seq(&event_log, attacker_id, "paradise_lost_judgement");
    let judgement_entries = descendants_of(&event_log, judgement_seq);
    assert!(judgement_entries.iter().any(|entry| matches!(
        entry.event,
        BattleLogEvent::HpChanged {
            target_instance_id,
            delta,
            ..
        } if target_instance_id == target_id && delta < 0
    )));
    assert!(judgement_entries.iter().any(|entry| matches!(
        entry.event,
        BattleLogEvent::BuffApplied {
            target_instance_id,
            buff_id,
            ..
        } if target_instance_id == target_id
            && buff_id == game_core::game::battle::buffs::BuffId::from_name("silence")
    )));
}
