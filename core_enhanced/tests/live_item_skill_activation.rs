mod common;

use std::sync::Arc;

use game_core::game::ability::AbilityActivationDef;
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::ids::UnitInstanceId;
use game_core::game::battle::scenario::{
    BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioArtifact, ScenarioEvent,
    ScenarioEventId, ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger, ScenarioUnitRef,
    ScenarioUnitSpawn, WinCondition,
};
use game_core::game::battle::tile_range::TileRangePattern;
use game_core::game::battle::timeline::{Timeline, TimelineEntry, TimelineEvent};
use game_core::game::battle::types::{BattleUnitDraft, BattleUnitSource};
use game_core::game::data::{
    abnormality_data::AbnormalityMetadata, artifact_data::ArtifactDatabase,
    equipment_data::EquipmentDatabase, GameDataBase, GameDataBuilder,
};
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::growth::GrowthStack;
use game_core::game::resources::Position;
use game_core::game::stats::{StatId, StatModifierKind};
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
        level: Tier::I,
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
        .filter(|meta| {
            matches!(
                meta.id.as_str(),
                "paradise_lost" | "fourth_match" | "penitence_armor" | "resonance_pendant"
            )
        })
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

    let artifacts = live
        .artifact_data
        .items
        .iter()
        .filter(|meta| meta.id == "red_ribbon")
        .cloned()
        .collect::<Vec<_>>();

    GameDataBuilder::live_defaults()
        .with_abnormalities(vec![attacker, target])
        .with_artifact_data(Arc::new(ArtifactDatabase::new(artifacts)))
        .with_equipment_data(Arc::new(EquipmentDatabase::new(equipments)))
        .with_skill_data(Arc::clone(&live.skill_data))
        .build_arc()
}

fn run_battle_with_loadout(
    game_data: Arc<GameDataBase>,
    equipped_items: Vec<Uuid>,
    artifacts: Vec<Uuid>,
) -> Timeline {
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
        .timeline
}

fn find_unit_instance_id(timeline: &Timeline, base_uuid: Uuid, owner: Side) -> UnitInstanceId {
    timeline
        .entries
        .iter()
        .find_map(|entry| match entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                base_uuid: spawned_base_uuid,
                owner: spawned_owner,
                ..
            } if spawned_base_uuid == base_uuid && spawned_owner == owner => Some(unit_instance_id),
            _ => None,
        })
        .expect("missing UnitSpawned entry for test unit")
}

fn find_first_cast_seq(timeline: &Timeline, caster: UnitInstanceId, skill_id: &str) -> u64 {
    timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::AbilityCast {
                caster_instance_id,
                skill_id: actual_skill_id,
                ..
            } if *caster_instance_id == caster && actual_skill_id == skill_id => Some(entry.seq),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing AbilityCast for {skill_id}"))
}

fn descendants_of(timeline: &Timeline, root_seq: u64) -> Vec<&TimelineEntry> {
    let mut relevant_seqs = vec![root_seq];
    let mut changed = true;

    while changed {
        changed = false;
        for entry in &timeline.entries {
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

    timeline
        .entries
        .iter()
        .filter(|entry| entry.seq != root_seq && relevant_seqs.contains(&entry.seq))
        .collect()
}

#[test]
fn live_item_and_artifact_ability_ids_resolve_to_skill_defs() {
    let game_data = common::load_game_data_from_ron();

    for ability_id in [
        "paradise_lost_judgement",
        "fourth_match_ember",
        "penitence_guard",
        "resonance_pendant_opening_focus",
        "red_ribbon_haste",
    ] {
        assert!(
            game_data.skill_data.get_by_id(ability_id).is_some(),
            "missing live skill definition for {ability_id}"
        );
    }
}

#[test]
fn live_on_battle_start_item_and_artifact_skills_apply_expected_self_effects() {
    let game_data = game_data_for_live_item_skill_tests(&[]);
    let penitence_uuid = game_data
        .equipment_data
        .get_by_id("penitence_armor")
        .expect("missing penitence armor")
        .uuid;
    let pendant_uuid = game_data
        .equipment_data
        .get_by_id("resonance_pendant")
        .expect("missing resonance pendant")
        .uuid;
    let ribbon_uuid = game_data
        .artifact_data
        .get_by_id("red_ribbon")
        .expect("missing red ribbon")
        .uuid;

    let timeline = run_battle_with_loadout(
        game_data,
        vec![penitence_uuid, pendant_uuid],
        vec![ribbon_uuid],
    );
    let attacker_id = find_unit_instance_id(&timeline, Uuid::from_u128(0xAB01), Side::Player);

    let guard_seq = find_first_cast_seq(&timeline, attacker_id, "penitence_guard");
    let guard_entries = descendants_of(&timeline, guard_seq);
    assert!(guard_entries.iter().any(|entry| matches!(
        entry.event,
        TimelineEvent::StatChanged {
            target_instance_id,
            modifier,
            ..
        } if target_instance_id == attacker_id
            && modifier.stat == StatId::Defense
            && modifier.kind == StatModifierKind::Flat
            && modifier.value == 8
    )));

    let focus_seq = find_first_cast_seq(&timeline, attacker_id, "resonance_pendant_opening_focus");
    let focus_entries = descendants_of(&timeline, focus_seq);
    assert!(focus_entries.iter().any(|entry| matches!(
        entry.event,
        TimelineEvent::ResonanceChanged {
            unit_instance_id,
            before,
            after,
            ..
        } if unit_instance_id == attacker_id && after.saturating_sub(before) == 30
    )));

    let haste_seq = find_first_cast_seq(&timeline, attacker_id, "red_ribbon_haste");
    let haste_entries = descendants_of(&timeline, haste_seq);
    assert!(haste_entries.iter().any(|entry| matches!(
        entry.event,
        TimelineEvent::StatChanged {
            target_instance_id,
            modifier,
            ..
        } if target_instance_id == attacker_id
            && modifier.stat == StatId::AttackIntervalMs
            && modifier.kind == StatModifierKind::Flat
            && modifier.value == -100
    )));
    assert!(haste_entries.iter().any(|entry| matches!(
        entry.event,
        TimelineEvent::StatChanged {
            target_instance_id,
            modifier,
            ..
        } if target_instance_id == attacker_id
            && modifier.stat == StatId::MoveSpeedUnitsPerMs
            && modifier.kind == StatModifierKind::Flat
            && modifier.value == 250
    )));
}

#[test]
fn live_on_attack_proc_item_skills_apply_expected_debuffs_when_forced_to_proc() {
    let game_data = game_data_for_live_item_skill_tests(&["paradise_lost", "fourth_match"]);
    let paradise_uuid = game_data
        .equipment_data
        .get_by_id("paradise_lost")
        .expect("missing paradise lost")
        .uuid;
    let ember_uuid = game_data
        .equipment_data
        .get_by_id("fourth_match")
        .expect("missing fourth match")
        .uuid;

    let timeline = run_battle_with_loadout(game_data, vec![paradise_uuid, ember_uuid], vec![]);
    let attacker_id = find_unit_instance_id(&timeline, Uuid::from_u128(0xAB01), Side::Player);
    let target_id = find_unit_instance_id(&timeline, Uuid::from_u128(0xAB02), Side::Opponent);

    let judgement_seq = find_first_cast_seq(&timeline, attacker_id, "paradise_lost_judgement");
    let judgement_entries = descendants_of(&timeline, judgement_seq);
    assert!(judgement_entries.iter().any(|entry| matches!(
        entry.event,
        TimelineEvent::HpChanged {
            target_instance_id,
            delta,
            ..
        } if target_instance_id == target_id && delta < 0
    )));
    assert!(judgement_entries.iter().any(|entry| matches!(
        entry.event,
        TimelineEvent::BuffApplied {
            target_instance_id,
            buff_id,
            ..
        } if target_instance_id == target_id
            && buff_id == game_core::game::battle::buffs::BuffId::from_name("silence")
    )));

    let ember_seq = find_first_cast_seq(&timeline, attacker_id, "fourth_match_ember");
    let ember_entries = descendants_of(&timeline, ember_seq);
    assert!(ember_entries.iter().any(|entry| matches!(
        entry.event,
        TimelineEvent::HpChanged {
            target_instance_id,
            delta,
            ..
        } if target_instance_id == target_id && delta < 0
    )));
    assert!(ember_entries.iter().any(|entry| matches!(
        entry.event,
        TimelineEvent::BuffApplied {
            target_instance_id,
            buff_id,
            ..
        } if target_instance_id == target_id
            && buff_id == game_core::game::battle::buffs::BuffId::from_name("poison")
    )));
}
