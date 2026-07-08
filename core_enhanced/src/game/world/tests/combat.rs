use super::*;
use crate::game::ability::SkillEffectDef;
use crate::game::battle::core::movement::types::EventLogVec2;
use crate::game::battle::core::sim::BattleDeployCurrentHpPolicy;
use crate::game::battle::event_log::{BattleEventLogEntry, BattleLogEvent};
use crate::game::battle::result_stats::collect_battle_result_stats;
use crate::game::battle::types::{
    BattleUnitRole, BattleUnitSourceIdentity, BattleUnitThreatClass, BattleWinner,
    ParticipantBattleResult,
};
use crate::game::behavior::{LiveBattleHudBarMode, LiveBattlePresentationEventKindDto};
use crate::game::enums::Side;
use crate::game::resources::{CombatBattleState, RunFailureReason};
use crate::game::stats::UnitStats;

fn test_result_stats(
    winner: BattleWinner,
    event_log: &BattleEventLog,
    participant_results: &[ParticipantBattleResult],
) -> crate::game::battle::result_stats::BattleResultStatsDto {
    collect_battle_result_stats(
        winner,
        event_log,
        participant_results,
        &crate::game::data::run_policy_data::RunPolicyData::builtin().battle_result_stats,
    )
}

fn force_map_combat_node(
    core: &mut GameCore,
    category: MapNodeCategory,
    kind_id: &str,
    encounter_id: &str,
) -> MapNodeId {
    force_first_available_node(
        core,
        category,
        kind_id,
        MapNodePayload::Encounter {
            encounter_id: Some(encounter_id.to_string()),
        },
    )
}

fn game_data_with_event_definitions_for_combat_tests(
    base: std::sync::Arc<GameDataBase>,
    events: Vec<crate::game::data::event_data::EventDefinition>,
) -> std::sync::Arc<GameDataBase> {
    GameDataBuilder::empty()
        .with_abnormality_data(base.abnormality_data.clone())
        .with_corroded_employee_data(base.corroded_employee_data.clone())
        .with_corroded_wave_data(base.corroded_wave_data.clone())
        .with_starter_employee_data(base.starter_employee_data.clone())
        .with_recruitment_employee_data(base.recruitment_employee_data.clone())
        .with_artifact_data(base.artifact_data.clone())
        .with_consumable_data(base.consumable_data.clone())
        .with_equipment_data(base.equipment_data.clone())
        .with_shop_data(base.shop_data.clone())
        .with_reward_data(base.reward_data.clone())
        .with_event_data(std::sync::Arc::new(
            crate::game::data::event_data::EventDatabase::new(events),
        ))
        .with_pve_data(base.pve_data.clone())
        .with_boss_omen_data(base.boss_omen_data.clone())
        .with_run_policy_data(base.run_policy.clone())
        .with_skill_data(base.skill_data.clone())
        .with_buff_data(base.buff_data.clone())
        .with_skill_fragment_data(base.skill_fragment_data.clone())
        .build_arc()
}

fn combat_choice_event_definition(
    event_id: &str,
    choice_id: &str,
    encounter_id: &str,
    primary_abnormality_id: &str,
) -> crate::game::data::event_data::EventDefinition {
    crate::game::data::event_data::EventDefinition {
        id: crate::game::data::event_data::EventId::new(event_id),
        entry_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
        scenes: vec![crate::game::data::event_data::EventSceneDefinition {
            id: crate::game::data::event_data::EventSceneId::new("choice"),
            presentation: crate::game::data::event_data::EventScenePresentation {
                background_id: "test_background".to_string(),
                script_id: "test_script".to_string(),
                speaker_id: None,
                portrait_id: None,
            },
            next: crate::game::data::event_data::EventSceneNext::Choices {
                choices: vec![crate::game::data::event_data::EventChoiceDefinition {
                    id: crate::game::data::event_data::EventChoiceId::new(choice_id),
                    label_id: "test_choice_label".to_string(),
                    preview: crate::game::data::event_data::EventChoicePreview {
                        starts_combat: true,
                        ..Default::default()
                    },
                    effects: vec![
                        crate::game::data::event_data::EventChoiceEffect::StartCombat {
                            encounter_id: encounter_id.to_string(),
                            primary_abnormality_id: Some(primary_abnormality_id.to_string()),
                        },
                    ],
                    next: None,
                }],
            },
        }],
    }
}

fn force_single_event_node_run_for_combat_tests(core: &mut GameCore, event_id: &str) -> MapNodeId {
    let start_node_id = MapNodeId::new(Uuid::from_u128(0xC1_0001));
    let event_node_id = MapNodeId::new(Uuid::from_u128(0xC1_0002));
    let map = RunMap {
        map_template_id: MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID),
        edges: vec![crate::game::map::MapEdgeDto {
            from_node_id: start_node_id,
            to_node_id: event_node_id,
            direction: crate::game::map::MapEdgeDirection::Bidirectional,
        }],
        nodes: vec![
            MapNode {
                id: start_node_id,
                depth: 0,
                lane: 0,
                slot_id: MapSlotId::for_grid_position(0, 0),
                kind_id: MapNodeKindId::new("start"),
                category: MapNodeCategory::Start,
                state: MapNodeState::Completed,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::None,
                omen: None,
            },
            MapNode {
                id: event_node_id,
                depth: 1,
                lane: 0,
                slot_id: MapSlotId::for_grid_position(1, 0),
                kind_id: MapNodeKindId::new("event_story"),
                category: MapNodeCategory::Event,
                state: MapNodeState::Available,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Event {
                    event_id: Some(crate::game::data::event_data::EventId::new(event_id)),
                },
                omen: None,
            },
        ],
        start_node_ids: vec![event_node_id],
        terminal_node_id: event_node_id,
    };
    let progression = crate::game::map::MapProgression {
        current_node_id: Some(start_node_id),
        available_node_ids: vec![event_node_id],
        completed_node_ids: vec![start_node_id],
    };
    let run_progression = core
        .state
        .run
        .as_ref()
        .expect("run")
        .run_progression
        .clone();
    core.state.run = Some(RunState::new(map, progression, run_progression));
    event_node_id
}

fn enter_event_combat_choice(
    core: &mut GameCore,
    player_id: Uuid,
    event_node_id: MapNodeId,
    event_id: &str,
    choice_id: &str,
) {
    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: event_node_id,
        },
    )
    .expect("event node should be selectable");
    let entered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("event node should enter");
    assert!(matches!(entered, BehaviorResult::EventState { .. }));
    let started = core
        .execute(
            player_id,
            PlayerBehavior::SelectEventChoice {
                node_id: event_node_id,
                event_id: crate::game::data::event_data::EventId::new(event_id),
                choice_id: crate::game::data::event_data::EventChoiceId::new(choice_id),
            },
        )
        .expect("event choice should start combat");
    assert!(matches!(started, BehaviorResult::BattleAdvanced { .. }));
}

fn first_ground_deployment_cell(preview: &BehaviorResult) -> Position {
    let BehaviorResult::NodePreview {
        combat_preview: Some(combat_preview),
        ..
    } = preview
    else {
        panic!("expected combat preview");
    };
    combat_preview
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .and_then(|zone| zone.cells.first().copied())
        .expect("expected ground deployment cell")
}

fn ground_deployment_cell_with_right_range(preview: &BehaviorResult) -> Position {
    let BehaviorResult::NodePreview {
        combat_preview: Some(combat_preview),
        ..
    } = preview
    else {
        panic!("expected combat preview");
    };
    let walkable = combat_preview
        .tiles
        .iter()
        .filter(|tile| {
            !matches!(
                tile.kind,
                crate::game::combat_preview::BattlefieldTileKind::Obstacle
            )
        })
        .map(|tile| tile.position)
        .collect::<std::collections::HashSet<_>>();
    combat_preview
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .and_then(|zone| {
            zone.cells.iter().copied().find(|cell| {
                walkable.contains(cell) && walkable.contains(&Position::new(cell.x + 1, cell.y))
            })
        })
        .expect("expected ground deployment cell with right-facing range")
}

#[test]
fn combat_result_without_node_session_is_rejected() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let abnormality_uuid = Uuid::from_u128(0xBEEF);

    core.transition_to(GameState::CombatResult {
        battle_uuid: abnormality_uuid,
    })
    .unwrap();
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(CombatBattleState {
        primary_abnormality_id: Some("abno".to_string()),
        encounter_id: "encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid,
        winner: BattleWinner::Opponent,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        bonus_objectives: vec![],
        reward_mode: RewardMode::ChooseOne,
        rewards: vec![],
        participant_results: vec![],
    }));

    let err = core.handle_complete_combat_result().unwrap_err();

    assert!(matches!(err, GameError::InvalidAction));

    let mut battle = core
        .state
        .active_node_content
        .as_ref()
        .unwrap()
        .as_combat_battle()
        .unwrap()
        .clone();
    battle.winner = BattleWinner::Player;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));

    let err = core.handle_complete_combat_result().unwrap_err();

    assert!(matches!(err, GameError::InvalidAction));
}

#[test]
fn combat_result_snapshot_exposes_typed_result_stats() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let battle_uuid = Uuid::from_u128(0xCAFE);
    let employee_uuid = Uuid::from_u128(0xE0);
    let unit_instance_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xE1));
    let mut event_log = BattleEventLog::default();
    event_log.entries.push(BattleEventLogEntry {
        seq: 1,
        time_ms: 0,
        cause: Default::default(),
        source_command_id: None,
        event: BattleLogEvent::UnitSpawned {
            unit_instance_id,
            owner: Side::Player,
            role: BattleUnitRole::Combatant,
            threat_class: BattleUnitThreatClass::Normal,
            mobility_kind: Default::default(),
            base_uuid: employee_uuid,
            unit_source: BattleUnitSourceIdentity::Employee {
                employee_uuid,
                base_uuid: employee_uuid,
            },
            world_position: EventLogVec2::default(),
            stats: UnitStats::default(),
        },
    });
    let participant_results = vec![ParticipantBattleResult {
        unit_instance_id,
        owned_uuid: employee_uuid,
        side: Side::Player,
        survived: true,
        final_hp: 42,
        max_hp: 100,
        became_incapacitated: false,
    }];
    let result_stats = test_result_stats(BattleWinner::Player, &event_log, &participant_results);

    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(CombatBattleState {
        primary_abnormality_id: Some("abno".to_string()),
        encounter_id: "encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: battle_uuid,
        winner: BattleWinner::Player,
        event_log,
        result_stats,
        bonus_objectives: vec![
            crate::game::pve_bonus_objectives::PveBonusObjectiveOutcomeDto {
                id: "fast_clear".to_string(),
                condition:
                    crate::game::data::pve_data::PveBonusObjectiveConditionData::ClearWithin {
                        time_ms: 90_000,
                    },
                research_bonus: 20,
                satisfied: true,
                presentation: Some("abnormality_part_obtained".to_string()),
            },
        ],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results,
    }));

    let selected = core
        .get_selected_event_snapshot_json()
        .unwrap()
        .expect("combat selected event");

    assert_eq!(selected["type"], "combat_battle");
    assert_eq!(selected["result_stats"]["battle"]["duration_ms"], 0);
    assert_eq!(selected["result_stats"]["battle"]["winner"], "Player");
    assert_eq!(
        selected["result_stats"]["employees"][0]["employee_uuid"],
        serde_json::json!(employee_uuid)
    );
    assert_eq!(selected["result_stats"]["employees"][0]["final_hp"], 42);
    assert_eq!(
        selected["result_stats"]["mvp"]["employee_uuid"],
        serde_json::json!(employee_uuid)
    );
    assert_eq!(selected["bonus_objectives"][0]["id"], "fast_clear");
    assert_eq!(selected["bonus_objectives"][0]["satisfied"], true);
    assert_eq!(selected["bonus_objectives"][0]["research_bonus"], 20);
}

#[test]
fn combat_result_completion_failure_does_not_partially_apply_rewards_on_retry() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        "low_risk_encounter",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    let experience_before = core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .experience;
    let enkephalin_before = core.state.enkephalin.amount;
    let equipment_count_before = core.state.inventory.equipments.len();
    let completed_node_count_before = core
        .state
        .run
        .as_ref()
        .unwrap()
        .map_progression
        .completed_node_ids
        .len();
    let participant_results = vec![ParticipantBattleResult {
        unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
            0xC0_0002,
        )),
        owned_uuid: employee_uuid,
        side: Side::Player,
        survived: true,
        final_hp: 100,
        max_hp: 100,
        became_incapacitated: false,
    }];
    let event_log = BattleEventLog::default();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("low_risk_abno".to_string()),
        encounter_id: "low_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Player,
        result_stats: test_result_stats(BattleWinner::Player, &event_log, &participant_results),
        event_log,
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![RewardOption {
            id: "combat_xp".to_string(),
            uuid: Uuid::from_u128(0xC0_0001),
            name: "Combat XP".to_string(),
            description: "combat xp".to_string(),
            icon: "test".to_string(),
            effects: vec![
                RewardEffect::GrantExperience {
                    amount: 10,
                    target: ExperienceTargetPolicy::CombatParticipants,
                },
                RewardEffect::GrantEnkephalin { amount: 7 },
                RewardEffect::GrantEquipment {
                    equipment_id: "standard_armor".to_string(),
                },
            ],
        }],
        participant_results,
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();

    core.state
        .run
        .as_mut()
        .unwrap()
        .map_progression
        .current_node_id = None;
    assert!(!core
        .get_allowed_actions()
        .contains(&ActionKind::CompleteCombatResult));

    for _ in 0..2 {
        let err = core.handle_complete_combat_result().unwrap_err();
        assert!(matches!(err, GameError::InvalidAction));
        let employee_after = core.roster().unwrap().get(&employee_uuid).unwrap();
        assert_eq!(employee_after.experience, experience_before);
        assert_eq!(core.state.enkephalin.amount, enkephalin_before);
        assert_eq!(
            core.state.inventory.equipments.len(),
            equipment_count_before
        );
        let run = core.state.run.as_ref().unwrap();
        assert_eq!(run.map_progression.current_node_id, None);
        assert_eq!(
            run.map_progression.completed_node_ids.len(),
            completed_node_count_before
        );
        assert!(core.state.node_session.is_some());
        assert!(matches!(core.get_state(), GameState::CombatResult { .. }));
    }
}

#[test]
fn endless_combat_victory_updates_abnormality_research_and_grants_completion_fragment() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        "low_risk_encounter",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.state
        .run
        .as_mut()
        .unwrap()
        .abnormality_research
        .entries
        .get_mut("low_risk_abno")
        .unwrap()
        .research_points = 80;

    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    let participant_results = vec![ParticipantBattleResult {
        unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
            0xC0_1002,
        )),
        owned_uuid: employee_uuid,
        side: Side::Player,
        survived: true,
        final_hp: 100,
        max_hp: 100,
        became_incapacitated: false,
    }];
    let event_log = BattleEventLog::default();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("low_risk_abno".to_string()),
        encounter_id: "low_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Player,
        result_stats: test_result_stats(BattleWinner::Player, &event_log, &participant_results),
        event_log,
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results,
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    let BehaviorResult::CombatRewardsGranted {
        skill_fragment_diffs,
        ..
    } = result
    else {
        panic!("expected combat rewards granted");
    };
    assert!(skill_fragment_diffs.iter().any(|diff| {
        diff.fragment_id == starter_basic_attack_fragment_id()
            && diff.count_before == 0
            && diff.count_after == 1
    }));
    let entry = core
        .state
        .run
        .as_ref()
        .unwrap()
        .abnormality_research
        .entries
        .get("low_risk_abno")
        .unwrap();
    assert_eq!(entry.research_points, 100);
    assert!(entry.response_complete);
    assert!(entry.unique_fragment_granted);
}

#[test]
fn endless_combat_victory_adds_satisfied_bonus_objective_research() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        "low_risk_encounter",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.state
        .run
        .as_mut()
        .unwrap()
        .abnormality_research
        .entries
        .get_mut("low_risk_abno")
        .unwrap()
        .research_points = 40;

    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    let participant_results = vec![ParticipantBattleResult {
        unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
            0xC0_1003,
        )),
        owned_uuid: employee_uuid,
        side: Side::Player,
        survived: true,
        final_hp: 100,
        max_hp: 100,
        became_incapacitated: false,
    }];
    let event_log = BattleEventLog::default();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("low_risk_abno".to_string()),
        encounter_id: "low_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Player,
        result_stats: test_result_stats(BattleWinner::Player, &event_log, &participant_results),
        event_log,
        bonus_objectives: vec![
            crate::game::pve_bonus_objectives::PveBonusObjectiveOutcomeDto {
                id: "fast_clear".to_string(),
                condition:
                    crate::game::data::pve_data::PveBonusObjectiveConditionData::ClearWithin {
                        time_ms: 90_000,
                    },
                research_bonus: 40,
                satisfied: true,
                presentation: Some("abnormality_part_obtained".to_string()),
            },
        ],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results,
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();

    core.execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    let entry = core
        .state
        .run
        .as_ref()
        .unwrap()
        .abnormality_research
        .entries
        .get("low_risk_abno")
        .unwrap();
    assert_eq!(entry.research_points, 100);
    assert!(entry.response_complete);
}

#[test]
fn post_battle_resolution_applies_employee_incapacitation_trauma() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");

    core.apply_post_battle_resolution(&[ParticipantBattleResult {
        unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xCAFE)),
        owned_uuid: employee_uuid,
        side: Side::Player,
        survived: false,
        final_hp: 0,
        max_hp: 30,
        became_incapacitated: true,
    }])
    .unwrap();

    let roster = core.roster().unwrap();
    let employee = roster.get(&employee_uuid).unwrap();
    assert_eq!(
        employee.trauma,
        run_policy().post_battle.incapacitation_trauma
    );
    assert_eq!(employee.injuries.len(), 1);
    assert_eq!(employee.injuries[0].id, "battle_incapacitation");
    assert_eq!(
        employee.life_state,
        crate::game::employee::EmployeeLifeState::Alive
    );
}

#[test]
fn repeated_post_battle_incapacitation_can_kill_employee_and_remove_from_roster_order() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core
        .roster()
        .unwrap()
        .available_employee_ids()
        .first()
        .copied()
        .expect("starter employee");

    let participant = ParticipantBattleResult {
        unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xCAFE)),
        owned_uuid: employee_uuid,
        side: Side::Player,
        survived: false,
        final_hp: 0,
        max_hp: 30,
        became_incapacitated: true,
    };
    core.apply_post_battle_resolution(&[participant.clone(), participant.clone(), participant])
        .unwrap();

    let roster = core.roster().unwrap();
    let employee = roster.get(&employee_uuid).unwrap();
    assert_eq!(
        employee.life_state,
        crate::game::employee::EmployeeLifeState::Dead
    );
    assert!(!roster.available_employee_ids().contains(&employee_uuid));
    let roster_order = core.roster_order().unwrap();
    assert!(roster_order.slot_of(employee_uuid).is_none());
}

#[test]
fn combat_map_node_fails_run_when_no_employee_can_be_deployed() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let roster = core.roster_mut().unwrap();
        for employee in roster.iter_mut() {
            employee.availability = EmployeeAvailability::Unavailable;
        }
    }
    core.sync_roster_order_with_owned_units().unwrap();
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        MapNodePayload::Encounter {
            encounter_id: Some("low_risk_encounter".to_string()),
        },
    );
    force_other_available_support_nodes_to_rest(&mut core, node_id);

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::RunFailed {
            reason: RunFailureReason::NoDeployableEmployees,
            ..
        }
    ));
    assert!(matches!(
        core.get_state(),
        GameState::RunFailed {
            reason: RunFailureReason::NoDeployableEmployees
        }
    ));
    assert!(core.state.node_session.is_none());
    assert!(core.get_allowed_actions().is_empty());
}

#[test]
fn combat_map_node_fails_run_when_all_living_employees_have_zero_hp() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let roster = core.roster_mut().unwrap();
        for employee in roster.iter_mut() {
            employee.health.set_current_hp(0);
        }
    }
    core.sync_roster_order_with_owned_units().unwrap();
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        MapNodePayload::Encounter {
            encounter_id: Some("low_risk_encounter".to_string()),
        },
    );
    force_other_available_support_nodes_to_rest(&mut core, node_id);

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::RunFailed {
            reason: RunFailureReason::NoDeployableEmployees,
            ..
        }
    ));
}

#[test]
fn combat_map_node_is_blocked_when_checkpoint_load_can_recover_no_deployable_roster() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let save_point_node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_save_point",
        MapNodePayload::Support {
            support_type: SupportNodeType::SavePoint,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        },
    );
    select_and_confirm_map_node(&mut core, player_id, save_point_node_id);
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .expect("SavePoint should complete and save checkpoint");
    assert!(core.state.run_checkpoint.can_load());

    {
        let roster = core.roster_mut().unwrap();
        for employee in roster.iter_mut() {
            employee.health.set_current_hp(0);
        }
    }
    core.sync_roster_order_with_owned_units().unwrap();

    let combat_node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        MapNodePayload::Encounter {
            encounter_id: Some("low_risk_encounter".to_string()),
        },
    );

    let preview = core
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: combat_node_id,
            },
        )
        .unwrap();
    assert!(matches!(preview, BehaviorResult::NodePreview { .. }));

    let err = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap_err();

    assert!(matches!(err, GameError::InvalidAction));
    assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .map_progression
        .available_node_ids
        .contains(&combat_node_id));
    core.execute(player_id, PlayerBehavior::CancelSelectedNode)
        .unwrap();
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::LoadRunCheckpoint));

    core.execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();
    assert!(core.roster().unwrap().available_employee_ids().len() > 0);
}

#[test]
fn defense_combat_node_smoke_writes_debug_event_log_export() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0101);
    let player_id = Uuid::from_u128(0xD3F3_0101);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_warm_hearted_woodsman",
    );

    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(combat_preview),
        ..
    } = &preview
    else {
        panic!("expected defense combat preview");
    };
    assert_eq!(
        combat_preview.node_type,
        crate::game::combat_preview::CombatNodeType::Defense
    );
    assert!(combat_preview
        .routes
        .iter()
        .any(|route| route.id == "defense_main"));
    assert!(combat_preview
        .spawn_waves
        .iter()
        .filter(|wave| wave.required_for_victory)
        .all(|wave| wave.route_id.as_deref() == Some("defense_main")));
    assert!(combat_preview.spawn_waves.iter().any(|wave| {
        wave.route_id.as_deref() == Some("defense_main") && wave.required_for_victory
    }));
    assert!(combat_preview
        .deployment_zones
        .iter()
        .any(|zone| { zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground }));
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];

    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid,
        finished: false,
        ..
    } = result
    else {
        panic!("expected defense combat to enter live battle");
    };
    assert!(matches!(
        core.get_state(),
        GameState::InBattle {
            battle_uuid: state_uuid,
        } if state_uuid == battle_uuid
    ));
    let deploy_source_command_id = "deploy-live-defense";
    let deploy_result = core
        .execute_with_source_command_id(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
            Some(deploy_source_command_id),
        )
        .unwrap();
    let BehaviorResult::BattleUnitDeployed { battle_update, .. } = deploy_result else {
        panic!("expected live deploy result");
    };
    let events = &battle_update.events_delta.events;
    assert!(events.iter().any(|entry| matches!(
        entry.event,
        LiveBattlePresentationEventKindDto::UnitSpawned {
            owner: Side::Player,
            threat_class: BattleUnitThreatClass::Normal,
            ..
        }
    )));
    assert!(events.iter().any(|entry| matches!(
        entry.event,
        LiveBattlePresentationEventKindDto::UnitDeployed {
            employee_uuid: event_employee_uuid,
            position,
            facing,
            ..
        } if event_employee_uuid == employee_uuid
            && position == deploy_position
            && facing == FacingDirection::Right
    )));
    assert!(events.iter().any(|entry| {
        matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::UnitDeployed { .. }
        ) && entry.source_command_id.as_deref() == Some(deploy_source_command_id)
    }));
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live deploy update should expose deployment state");
    assert_eq!(deployment.deployed_units.len(), 1);
    let player_unit = battle_update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.owner == Side::Player && unit.role == BattleUnitRole::Combatant)
        .expect("live deploy checkpoint should include deployed player unit");
    assert_eq!(player_unit.hud.threat_class, BattleUnitThreatClass::Normal);
    assert_eq!(
        player_unit.hud.bar_mode,
        LiveBattleHudBarMode::HpAndResonance
    );
    assert_eq!(player_unit.hud.resonance_current, Some(0));
    assert!(player_unit.hud.resonance_max.is_some());

    let defense_object = battle_update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.role == BattleUnitRole::DefenseObject)
        .expect("live deploy checkpoint should include defense object");
    assert_eq!(
        defense_object.hud.threat_class,
        BattleUnitThreatClass::Normal
    );
    assert_eq!(defense_object.hud.bar_mode, LiveBattleHudBarMode::HpOnly);
    assert_eq!(defense_object.hud.resonance_current, None);
    assert_eq!(defense_object.hud.resonance_max, None);

    assert!(
        !battle_update
            .checkpoint
            .units
            .iter()
            .any(|unit| unit.owner == Side::Opponent),
        "live deploy checkpoint should not include delayed opponent wave units before 5s"
    );

    let spawn_result = core
        .advance_active_battle_for_server_tick(5_000)
        .unwrap()
        .expect("5s tick should spawn the first delayed opponent wave");
    let BehaviorResult::BattleAdvanced {
        battle_update: spawn_update,
        finished: false,
        ..
    } = spawn_result
    else {
        panic!("expected delayed opponent wave spawn update");
    };
    let abnormality_enemy = spawn_update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.owner == Side::Opponent)
        .expect("5s checkpoint should include opponent unit");
    assert_eq!(
        abnormality_enemy.hud.threat_class,
        BattleUnitThreatClass::Normal
    );
    assert_eq!(abnormality_enemy.hud.bar_mode, LiveBattleHudBarMode::HpOnly);
    assert_eq!(abnormality_enemy.hud.resonance_current, None);
    assert_eq!(abnormality_enemy.hud.resonance_max, None);

    let mut finished = false;
    for _ in 0..10_000 {
        let result = core.advance_active_battle_to_next_event_bucket().unwrap();
        if let BehaviorResult::BattleAdvanced { finished: true, .. } = result {
            finished = true;
            break;
        }
    }
    assert!(finished, "defense live battle should eventually finish");

    let battle = core
        .state
        .active_node_content
        .as_ref()
        .unwrap()
        .as_combat_battle()
        .unwrap();
    assert_eq!(
        battle.node_type,
        crate::game::combat_preview::CombatNodeType::Defense
    );
    assert_eq!(
        battle.event_log.version,
        crate::game::battle::event_log::BATTLE_EVENT_LOG_VERSION
    );
    assert!(battle
        .event_log
        .entries
        .iter()
        .any(|entry| matches!(entry.event, BattleLogEvent::BattleStart { .. })));
    assert!(battle
        .event_log
        .entries
        .iter()
        .any(|entry| matches!(entry.event, BattleLogEvent::UnitSpawned { .. })));
    assert!(battle
        .event_log
        .entries
        .iter()
        .any(|entry| matches!(entry.event, BattleLogEvent::BattleEnd { .. })));

    let path = write_world_debug_event_log_export("defense_combat_node_smoke", &battle.event_log);
    println!("wrote debug event log: {}", path.display());
    assert!(path.exists());
    let record_path = core
        .state
        .run
        .as_ref()
        .unwrap()
        .battle_record_debug_export_path(battle.abnormality_uuid);
    assert!(record_path.exists());
    std::fs::remove_file(&record_path).expect("cleanup smoke battle record file");
}

#[test]
fn live_ron_defense_route_playable_path_runs_to_combat_result() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0001);
    let player_id = Uuid::from_u128(0xD3F3);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_warm_hearted_woodsman",
    );

    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(combat_preview),
        ..
    } = &preview
    else {
        panic!("expected live defense combat preview");
    };
    assert_eq!(
        combat_preview.node_type,
        crate::game::combat_preview::CombatNodeType::Defense
    );
    let route = combat_preview
        .routes
        .iter()
        .find(|route| route.id == "defense_main")
        .expect("live defense route should be authored");
    assert!(combat_preview
        .spawn_waves
        .iter()
        .filter(|wave| wave.required_for_victory)
        .all(|wave| wave.route_id.as_deref() == Some("defense_main")));
    assert!(combat_preview.spawn_waves.iter().any(|wave| {
        wave.route_id.as_deref() == Some("defense_main") && wave.required_for_victory
    }));

    let mut deploy_positions = combat_preview
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .collect::<Vec<_>>();
    deploy_positions.sort_by_key(|position| {
        (
            position.manhattan(&route.end),
            (position.x - route.end.x).abs(),
            position.y,
            position.x,
        )
    });
    deploy_positions.dedup();
    assert!(
        deploy_positions.len() >= 2,
        "live defense should expose multiple ground deployment cells near the route endpoint"
    );
    let employee_ids = core.roster().unwrap().available_employee_ids();
    assert!(
        employee_ids.len() >= 2,
        "starter selection should provide enough employees for live defense smoke"
    );

    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(matches!(
        result,
        BehaviorResult::BattleAdvanced {
            finished: false,
            ..
        }
    ));
    assert!(matches!(core.get_state(), GameState::InBattle { .. }));
    for (employee_uuid, position) in employee_ids.iter().take(2).zip(deploy_positions.iter()) {
        let result = core
            .execute(
                player_id,
                PlayerBehavior::DeployUnit {
                    employee_uuid: *employee_uuid,
                    position: *position,
                    facing: FacingDirection::Right,
                },
            )
            .unwrap();
        assert!(matches!(result, BehaviorResult::BattleUnitDeployed { .. }));
    }

    let mut third_employee = employee_ids.get(2).copied();
    let third_position = deploy_positions.get(2).copied();
    let mut finished = false;
    for tick_index in 0..20_000 {
        let result = match core.advance_active_battle_to_next_event_bucket() {
            Ok(result) => result,
            Err(error) => {
                let bodies = core
                    .state
                    .active_battle
                    .as_ref()
                    .map(|active| active.battle.live_unit_bodies())
                    .unwrap_or_default();
                panic!(
                    "live RON defense tick {tick_index} failed with {error:?}; bodies={bodies:?}"
                );
            }
        };
        let BehaviorResult::BattleAdvanced {
            finished: battle_finished,
            battle_update,
            ..
        } = result
        else {
            panic!("expected live battle tick result");
        };
        if !battle_finished {
            if let (Some(employee_uuid), Some(position), Some(deployment)) = (
                third_employee,
                third_position,
                battle_update.checkpoint.deployment.as_ref(),
            ) {
                if deployment.current_cost >= deployment.base_deploy_cost {
                    let result = core
                        .execute(
                            player_id,
                            PlayerBehavior::DeployUnit {
                                employee_uuid,
                                position,
                                facing: FacingDirection::Right,
                            },
                        )
                        .unwrap();
                    assert!(matches!(result, BehaviorResult::BattleUnitDeployed { .. }));
                    third_employee = None;
                }
            }
        }
        if battle_finished {
            finished = true;
            break;
        }
    }
    assert!(
        finished,
        "live RON defense battle should finish deterministically"
    );
    assert!(matches!(core.get_state(), GameState::CombatResult { .. }));

    let (abnormality_uuid, event_log_entry_count) = {
        let battle = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_combat_battle()
            .unwrap();
        assert_eq!(
            battle.node_type,
            crate::game::combat_preview::CombatNodeType::Defense
        );
        assert!(battle
            .event_log
            .entries
            .iter()
            .any(|entry| matches!(entry.event, BattleLogEvent::BattleEnd { .. })));
        assert!(battle
            .event_log
            .entries
            .iter()
            .any(|entry| matches!(entry.event, BattleLogEvent::UnitSpawned { .. })));
        (battle.abnormality_uuid, battle.event_log.entries.len())
    };
    assert_eq!(core.battle_records().len(), 1);
    let record = &core.battle_records()[0];
    assert_eq!(record.abnormality_uuid, abnormality_uuid);
    assert_eq!(record.event_log.entries.len(), event_log_entry_count);
    let record_path = core
        .state
        .run
        .as_ref()
        .unwrap()
        .battle_record_debug_export_path(abnormality_uuid);
    assert!(record_path.exists(), "battle record file should exist");
    let record_json: serde_json::Value = serde_json::from_reader(
        std::fs::File::open(&record_path).expect("open battle record file"),
    )
    .expect("battle record should be valid json");
    assert_eq!(record_json["abnormality_uuid"], json!(abnormality_uuid));
    assert_eq!(
        record_json["event_log"]["entries"]
            .as_array()
            .expect("event_log entries array")
            .len(),
        event_log_entry_count
    );

    let completion = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let returned_to_retry = matches!(completion, BehaviorResult::NodePreview { .. });
    match completion {
        BehaviorResult::CombatRewardsGranted {
            outcome,
            completion,
            ..
        } => {
            assert!(outcome.mission_success);
            assert!(matches!(
                *completion,
                BehaviorResult::NodeCompleted { .. }
                    | BehaviorResult::FloorAdvanced { .. }
                    | BehaviorResult::RunComplete { .. }
            ));
        }
        BehaviorResult::NodeCompleted {
            outcome: Some(_), ..
        }
        | BehaviorResult::FloorAdvanced { .. }
        | BehaviorResult::RunComplete { .. } => {}
        BehaviorResult::NodePreview { category, .. } => {
            assert_eq!(category, MapNodeCategory::Combat);
        }
        other => panic!("unexpected combat completion result: {other:?}"),
    }
    if returned_to_retry {
        assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    } else {
        assert!(matches!(
            core.get_state(),
            GameState::ViewingMap | GameState::RunComplete
        ));
    }
    assert_eq!(core.battle_records().len(), 1);
    let record = &core.battle_records()[0];
    assert_eq!(record.abnormality_uuid, abnormality_uuid);
    assert_eq!(record.event_log.entries.len(), event_log_entry_count);
    assert!(record_path.exists());
    std::fs::remove_file(&record_path).expect("cleanup battle record file");
}

#[test]
fn battle_setup_snapshot_live_defense_state_request_returns_battle_update_without_advancing_cursor()
{
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0102);
    let player_id = Uuid::from_u128(0xD3F3_0102);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_warm_hearted_woodsman",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid,
        battle_setup_snapshot: Some(setup),
        battle_update: first_update,
        finished: false,
        ..
    } = result
    else {
        panic!("expected live battle start with setup snapshot");
    };
    assert_eq!(setup.battle_uuid, battle_uuid);
    assert_eq!(first_update.battle_uuid, battle_uuid);
    assert_eq!(setup.setup_version, 1);
    assert_eq!(setup.encounter_id, "suppress_warm_hearted_woodsman");
    assert!(setup.battlefield.width > 0);
    assert!(setup.battlefield.height > 0);
    assert!(!setup.battlefield.valid_tiles.is_empty());
    assert!(!setup.routes.is_empty());
    assert!(!setup.deployment_zones.is_empty());
    assert!(!setup.spawn_zones.is_empty());
    assert!(!setup.tactical_points.is_empty());
    assert!(setup.static_objects.is_empty());
    assert!(setup.tactical_points.iter().any(|point| {
        point.point_type == crate::game::behavior::LiveBattleSetupTacticalPointType::RouteEndpoint
    }));
    assert!(setup.initial_units.is_empty());
    assert_eq!(
        setup.catalog_refs.battlefield_template_id,
        "corridor_hook_control_01"
    );
    assert!(setup
        .catalog_refs
        .abnormality_ids
        .iter()
        .any(|id| id == "f-05-32_warm_hearted_woodsman"));
    assert!(first_update
        .events_delta
        .events
        .iter()
        .any(|entry| matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::BattleStart { .. }
        )));
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::RequestBattleState));
    assert!(core.get_allowed_actions().contains(&ActionKind::DeployUnit));
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::WithdrawUnit));
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["game_state_context"]["type"], "in_battle");
    assert_eq!(
        snapshot["game_state_context"]["combat_preview"]["node_type"],
        "Defense"
    );
    assert!(snapshot["game_state_context"]["combat_preview"]["routes"]
        .as_array()
        .is_some_and(|routes| routes.iter().any(|route| {
            route["id"] == "defense_main"
                && route["cells"]
                    .as_array()
                    .is_some_and(|cells| !cells.is_empty())
        })));
    assert!(
        snapshot["game_state_context"]["combat_preview"]["deployment_zones"]
            .as_array()
            .is_some_and(|zones| zones.iter().any(|zone| {
                zone["kind"] == "Ground"
                    && zone["cells"]
                        .as_array()
                        .is_some_and(|cells| !cells.is_empty())
            }))
    );

    let state = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState { since_seq: None },
        )
        .unwrap();
    let BehaviorResult::BattleState {
        node_type,
        combat_preview,
        battle_update,
        finished,
        ..
    } = state
    else {
        panic!("expected battle state");
    };
    assert_eq!(
        node_type,
        crate::game::combat_preview::CombatNodeType::Defense
    );
    assert_eq!(
        combat_preview.node_type,
        crate::game::combat_preview::CombatNodeType::Defense
    );
    assert!(combat_preview
        .deployment_zones
        .iter()
        .any(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground));
    assert!(!combat_preview.routes.is_empty());
    assert!(!finished);
    let events = &battle_update.events_delta.events;
    let last_seq = battle_update.events_delta.to_seq;
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live defense should expose deployment state");
    assert!(deployment.deployed_units.is_empty());
    assert_eq!(deployment.battle_time_ms, 0);
    assert_eq!(
        deployment.current_cost,
        run_policy().live_deployment.initial_cost
    );
    assert!(events.iter().any(|entry| matches!(
        entry.event,
        LiveBattlePresentationEventKindDto::BattleStart { .. }
    )));
    assert_eq!(last_seq, events.last().map(|entry| entry.seq).unwrap_or(0));
    assert_eq!(events.first().map(|entry| entry.seq), Some(1));

    let active = core.state.active_battle.as_ref().unwrap();
    let update = active
        .battle_update_dto_after(None, &core.state.roster)
        .unwrap();
    assert_eq!(
        update.message_type,
        crate::game::behavior::LiveBattleUpdateMessageType::BattleUpdate
    );
    assert_eq!(update.events_delta.after_seq, 0);
    assert_eq!(
        update.events_delta.to_seq,
        update
            .events_delta
            .events
            .last()
            .map(|entry| entry.seq)
            .unwrap_or(0)
    );
    assert_eq!(update.checkpoint.at_seq, update.events_delta.to_seq);
    assert_eq!(
        update.checkpoint.battle_time_ms,
        update.server_battle_time_ms
    );

    let repeated = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState {
                since_seq: Some(last_seq),
            },
        )
        .unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = repeated else {
        panic!("expected repeated battle state");
    };
    assert!(battle_update.events_delta.events.is_empty());
    let update = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_update_dto_after(Some(last_seq), &core.state.roster)
        .unwrap();
    assert!(update.events_delta.events.is_empty());
    assert_eq!(update.events_delta.after_seq, last_seq);
    assert_eq!(update.events_delta.to_seq, last_seq);
    assert_eq!(update.checkpoint.at_seq, last_seq);
}

#[test]
fn authored_defense_route_setup_snapshot_matches_runtime_route_cells() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xD3F3_0200);
    let player_id = Uuid::from_u128(0xD3F3_0200);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "defense_route_encounter",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_setup_snapshot: Some(setup),
        ..
    } = result
    else {
        panic!("expected live battle start with setup snapshot");
    };
    let setup_route = setup
        .routes
        .first()
        .expect("setup snapshot should expose generated route");
    assert_eq!(setup_route.id, "defense_main");
    assert!(
        setup_route.cells.len() > 1,
        "generated setup route should expose route cells"
    );

    let active = core
        .state
        .active_battle
        .as_ref()
        .expect("battle should remain active");
    let crate::game::battle::scenario::EnemyMovementPlan::PathAlongCells { cells } =
        &active.battle.scenario().tactical_plan.enemy_plan;
    assert_eq!(&setup_route.cells, cells);
}

#[test]
fn live_battle_update_exposes_adjacent_immutable_movement_segments() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xD3F3_0201);
    let player_id = Uuid::from_u128(0xD3F3_0201);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "defense_route_encounter",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced { battle_update, .. } = result else {
        panic!("expected live battle start");
    };

    let mut movement_segments = battle_update
        .events_delta
        .events
        .iter()
        .filter_map(|entry| match &entry.event {
            LiveBattlePresentationEventKindDto::MovementSegmentStarted {
                unit_instance_id,
                start,
                target,
                started_at_ms,
                ends_at_ms,
                ..
            } => Some((
                entry.seq,
                *unit_instance_id,
                *start,
                *target,
                *started_at_ms,
                *ends_at_ms,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();

    for _ in 0..10 {
        let Some(result) = core.advance_active_battle_for_server_tick(50).unwrap() else {
            continue;
        };
        let BehaviorResult::BattleAdvanced { battle_update, .. } = result else {
            panic!("expected live battle tick result");
        };
        movement_segments.extend(
            battle_update
                .events_delta
                .events
                .iter()
                .filter_map(|entry| match &entry.event {
                    LiveBattlePresentationEventKindDto::MovementSegmentStarted {
                        unit_instance_id,
                        start,
                        target,
                        started_at_ms,
                        ends_at_ms,
                        ..
                    } => Some((
                        entry.seq,
                        *unit_instance_id,
                        *start,
                        *target,
                        *started_at_ms,
                        *ends_at_ms,
                    )),
                    _ => None,
                }),
        );
    }

    movement_segments.sort_by_key(|(seq, unit_instance_id, ..)| (*unit_instance_id, *seq));
    let has_adjacent_pair = movement_segments.windows(2).any(|window| {
        let [left, right] = window else {
            return false;
        };
        let (_, left_unit, _, left_target, _, left_ends_at_ms) = *left;
        let (_, right_unit, right_start, _, right_started_at_ms, right_ends_at_ms) = *right;
        left_unit == right_unit
            && left_ends_at_ms == right_started_at_ms
            && right_ends_at_ms == right_started_at_ms + 50
            && left_target == right_start
    });

    assert!(
        has_adjacent_pair,
        "live battle_update should expose adjacent immutable 50ms movement segments: {movement_segments:?}"
    );
}

#[test]
fn battle_setup_snapshot_exposes_survival_timer_without_encirclement_variant() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xB1E5_7A2);
    let player_id = Uuid::from_u128(0xB1E5_7A2);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_clouded_monk",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_setup_snapshot: Some(setup),
        battle_update,
        finished: false,
        ..
    } = result
    else {
        panic!("expected survival timer live battle start with setup snapshot");
    };

    assert_eq!(
        setup.node_type,
        crate::game::combat_preview::CombatNodeType::Defense
    );
    assert_eq!(
        setup.mission_variant,
        crate::game::combat_preview::CombatMissionVariant::Defense
    );
    assert_eq!(setup.survive_timer_ms, Some(45_000));
    assert_eq!(battle_update.checkpoint.battle_time_ms, 0);
    assert!(
        !battle_update.events_delta.events.iter().any(|entry| {
            matches!(
                entry.event,
                LiveBattlePresentationEventKindDto::UnitSpawned {
                    owner: Side::Opponent,
                    ..
                }
            )
        }),
        "live RON field enemies should use the authored 5s spawn delay, not spawn at battle start"
    );
    assert!(
        !battle_update
            .checkpoint
            .units
            .iter()
            .any(|unit| unit.owner == Side::Opponent),
        "initial checkpoint should not expose delayed opponent wave units before 5s"
    );
    assert!(!battle_update.events_delta.events.iter().any(|entry| {
        matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::BattleEnd { .. }
        )
    }));

    let spawned = core
        .advance_active_battle_for_server_tick(5_000)
        .unwrap()
        .expect("5s tick should advance to the first authored wave spawn");
    let BehaviorResult::BattleAdvanced {
        battle_update: spawn_update,
        finished: false,
        ..
    } = spawned
    else {
        panic!("expected first delayed wave spawn update");
    };
    assert_eq!(spawn_update.checkpoint.battle_time_ms, 5_000);
    assert!(spawn_update.events_delta.events.iter().any(|entry| {
        matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::UnitSpawned {
                owner: Side::Opponent,
                threat_class: BattleUnitThreatClass::Normal,
                ..
            }
        )
    }));
    let corroded_enemy = spawn_update
        .checkpoint
        .units
        .iter()
        .find(|unit| {
            unit.owner == Side::Opponent && unit.hud.threat_class == BattleUnitThreatClass::Normal
        })
        .expect("live field battle should include opponent corroded employee unit");
    assert_eq!(
        corroded_enemy.hud.threat_class,
        BattleUnitThreatClass::Normal
    );
    assert_eq!(corroded_enemy.hud.bar_mode, LiveBattleHudBarMode::HpOnly);
    assert!(corroded_enemy.hud.resonance_current.is_none());
    assert!(corroded_enemy.hud.resonance_max.is_none());
    let BattleUnitSourceIdentity::CorrodedEmployee {
        profile_id,
        base_uuid,
    } = &corroded_enemy.unit_source
    else {
        panic!("live corroded enemy checkpoint should expose corroded employee source identity");
    };
    assert_ne!(*base_uuid, Uuid::nil());
    assert!(
        !setup
            .catalog_refs
            .abnormality_ids
            .iter()
            .any(|abnormality_id| abnormality_id == profile_id),
        "corroded profile_id must not leak into abnormality preload catalog refs"
    );
    let spawn_source = spawn_update
        .events_delta
        .events
        .iter()
        .find_map(|entry| match &entry.event {
            LiveBattlePresentationEventKindDto::UnitSpawned {
                unit_instance_id,
                unit_source,
                ..
            } if *unit_instance_id == corroded_enemy.unit_instance_id => Some(unit_source),
            _ => None,
        })
        .expect("UnitSpawned should carry source identity for checkpoint unit");
    assert_eq!(spawn_source, &corroded_enemy.unit_source);
    assert!(!spawn_update.events_delta.events.iter().any(|entry| {
        matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::BattleEnd { .. }
        )
    }));
}

#[test]
fn live_defense_battle_state_rejects_future_since_seq() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_5100);
    let player_id = Uuid::from_u128(0xD3F3_5100);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_burrowing_heaven",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced { battle_update, .. } = result else {
        panic!("expected live battle start");
    };
    let latest = battle_update.events_delta.to_seq;

    let err = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState {
                since_seq: Some(latest + 1),
            },
        )
        .expect_err("future since_seq must be rejected");

    assert!(matches!(
        err,
        GameError::InvalidBattleResyncSeq {
            requested,
            latest: actual_latest,
        } if requested == latest + 1 && actual_latest == latest
    ));
}

#[test]
fn live_defense_setup_loss_recovery_returns_to_node_confirm_and_restarts_event_log() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_5101);
    let player_id = Uuid::from_u128(0xD3F3_5101);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_burrowing_heaven",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let first = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: first_battle_uuid,
        battle_update: first_update,
        ..
    } = first
    else {
        panic!("expected first live battle start");
    };
    assert_eq!(
        first_update
            .events_delta
            .events
            .first()
            .map(|entry| entry.seq),
        Some(1)
    );
    assert!(matches!(core.get_state(), GameState::InBattle { .. }));
    assert!(core.state.active_battle.is_some());

    let recovered = core
        .execute(player_id, PlayerBehavior::RecoverBattleSetupLoss)
        .unwrap();
    let BehaviorResult::NodePreview {
        node_id: recovered_node_id,
        combat_preview: Some(_),
        ..
    } = recovered
    else {
        panic!("expected node preview after setup-loss recovery");
    };
    assert_eq!(recovered_node_id, node_id);
    assert!(
        matches!(core.get_state(), GameState::NodeConfirm { node_id: state_node_id, .. } if state_node_id == node_id)
    );
    assert!(core.state.active_battle.is_none());
    assert!(core.state.node_session.is_some());
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::ConfirmEnterNode));

    let second = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: second_battle_uuid,
        battle_setup_snapshot: Some(setup),
        battle_update: second_update,
        ..
    } = second
    else {
        panic!("expected second live battle start");
    };
    assert_eq!(second_battle_uuid, first_battle_uuid);
    assert_eq!(setup.battle_uuid, second_battle_uuid);
    assert_eq!(second_update.events_delta.after_seq, 0);
    assert_eq!(
        second_update
            .events_delta
            .events
            .first()
            .map(|entry| entry.seq),
        Some(1)
    );
    assert_eq!(
        second_update.checkpoint.at_seq,
        second_update.events_delta.to_seq
    );
}

#[test]
fn live_defense_playback_pause_freezes_server_tick_and_resume_advances() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0101);
    let player_id = Uuid::from_u128(0xD3F3_0101);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_burrowing_heaven",
    );

    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced { battle_update, .. } = result else {
        panic!("expected live battle start");
    };
    let battle_time_ms = battle_update.checkpoint.battle_time_ms;
    let playback = battle_update.checkpoint.playback;
    assert_eq!(battle_time_ms, 0);
    assert!(!playback.paused);
    assert_eq!(
        playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X1
    );
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::PauseBattle));
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::ResumeBattle));
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::SetBattleSpeed));
    core.execute(
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();

    let paused = core
        .execute(player_id, PlayerBehavior::PauseBattle)
        .unwrap();
    let BehaviorResult::BattlePlaybackChanged { battle_update, .. } = paused else {
        panic!("expected playback changed");
    };
    let playback = battle_update.checkpoint.playback;
    let paused_time_ms = battle_update.checkpoint.battle_time_ms;
    assert!(playback.paused);
    assert_eq!(paused_time_ms, 0);
    let paused_snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(
        paused_snapshot["game_state_context"]["playback"]["paused"],
        true
    );

    let paused_tick = core.advance_active_battle_for_server_tick(10).unwrap();
    assert!(paused_tick.is_none());
    let state = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState { since_seq: None },
        )
        .unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = state else {
        panic!("expected paused battle state");
    };
    let playback = battle_update.checkpoint.playback;
    let battle_time_ms = battle_update.checkpoint.battle_time_ms;
    assert!(playback.paused);
    assert_eq!(battle_time_ms, paused_time_ms);

    let resumed = core
        .execute(player_id, PlayerBehavior::ResumeBattle)
        .unwrap();
    let BehaviorResult::BattlePlaybackChanged { battle_update, .. } = resumed else {
        panic!("expected playback changed");
    };
    let playback = battle_update.checkpoint.playback;
    assert!(!playback.paused);

    let advanced = core
        .advance_active_battle_for_server_tick(10)
        .unwrap()
        .expect("resume should advance the live battle");
    let BehaviorResult::BattleAdvanced {
        battle_update,
        finished,
        ..
    } = advanced
    else {
        panic!("expected battle advanced");
    };
    let battle_time_ms = battle_update.checkpoint.battle_time_ms;
    let playback = battle_update.checkpoint.playback;
    assert!(!finished);
    assert!(!playback.paused);
    assert_eq!(battle_time_ms, 10);
}

#[test]
fn live_defense_playback_speed_scales_server_tick_delta() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0102);
    let player_id = Uuid::from_u128(0xD3F3_0102);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_burrowing_heaven",
    );

    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_ids = core.roster().unwrap().available_employee_ids();
    let employee_uuid = employee_ids[0];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();

    let changed = core
        .execute(
            player_id,
            PlayerBehavior::SetBattleSpeed {
                speed: crate::game::behavior::BattlePlaybackSpeed::X2,
            },
        )
        .unwrap();
    let BehaviorResult::BattlePlaybackChanged { battle_update, .. } = changed else {
        panic!("expected playback changed");
    };
    let playback = battle_update.checkpoint.playback;
    assert_eq!(
        playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X2
    );

    let x2_tick = core
        .advance_active_battle_for_server_tick(10)
        .unwrap()
        .expect("x2 tick should advance");
    let BehaviorResult::BattleAdvanced {
        battle_update,
        finished,
        ..
    } = x2_tick
    else {
        panic!("expected x2 battle advanced");
    };
    let battle_time_ms = battle_update.checkpoint.battle_time_ms;
    let playback = battle_update.checkpoint.playback;
    assert!(!finished);
    assert_eq!(battle_time_ms, 20);
    assert_eq!(
        playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X2
    );

    core.execute(
        player_id,
        PlayerBehavior::SetBattleSpeed {
            speed: crate::game::behavior::BattlePlaybackSpeed::X3,
        },
    )
    .unwrap();
    let x3_tick = core
        .advance_active_battle_for_server_tick(10)
        .unwrap()
        .expect("x3 tick should advance");
    let BehaviorResult::BattleAdvanced {
        battle_update,
        finished,
        ..
    } = x3_tick
    else {
        panic!("expected x3 battle advanced");
    };
    let battle_time_ms = battle_update.checkpoint.battle_time_ms;
    let playback = battle_update.checkpoint.playback;
    assert!(!finished);
    assert_eq!(battle_time_ms, 50);
    assert_eq!(
        playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X3
    );
}

#[test]
fn live_defense_half_speed_accumulates_fractional_server_ticks() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0103);
    let player_id = Uuid::from_u128(0xD3F3_0103);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_burrowing_heaven",
    );

    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_ids = core.roster().unwrap().available_employee_ids();
    let employee_uuid = employee_ids[0];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let changed = core
        .execute(
            player_id,
            PlayerBehavior::SetBattleSpeed {
                speed: crate::game::behavior::BattlePlaybackSpeed::X0_5,
            },
        )
        .unwrap();
    let BehaviorResult::BattlePlaybackChanged { battle_update, .. } = changed else {
        panic!("expected playback changed");
    };
    let playback = battle_update.checkpoint.playback;
    assert_eq!(
        playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X0_5
    );

    let first_tick = core.advance_active_battle_for_server_tick(1).unwrap();
    assert!(
        first_tick.is_none(),
        "first 1ms tick at 0.5x should only accumulate fractional time"
    );

    let second_tick = core
        .advance_active_battle_for_server_tick(1)
        .unwrap()
        .expect("second 1ms tick at 0.5x should advance one simulation ms");
    let BehaviorResult::BattleAdvanced {
        battle_update,
        finished,
        ..
    } = second_tick
    else {
        panic!("expected half-speed battle advanced");
    };
    let battle_time_ms = battle_update.checkpoint.battle_time_ms;
    let playback = battle_update.checkpoint.playback;
    assert!(!finished);
    assert_eq!(battle_time_ms, 1);
    assert_eq!(
        playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X0_5
    );
}

#[test]
fn pending_deployment_range_preview_returns_all_facings_without_mutating_battle() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );

    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = ground_deployment_cell_with_right_range(&preview);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];

    let not_live = core
        .execute(
            player_id,
            PlayerBehavior::RequestDeploymentRangePreview {
                employee_uuid,
                position: deploy_position,
            },
        )
        .unwrap_err();
    assert!(matches!(not_live, GameError::InvalidAction));

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::RequestDeploymentRangePreview));

    let before_update = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_update_dto_after(None, &core.state.roster)
        .unwrap();
    let before_cost = before_update
        .checkpoint
        .deployment
        .as_ref()
        .unwrap()
        .current_cost;
    assert!(before_update
        .checkpoint
        .deployment
        .as_ref()
        .unwrap()
        .deployed_units
        .is_empty());

    let result = core
        .execute(
            player_id,
            PlayerBehavior::RequestDeploymentRangePreview {
                employee_uuid,
                position: deploy_position,
            },
        )
        .unwrap();
    let BehaviorResult::DeploymentRangePreview { result } = result else {
        panic!("expected deployment range preview result");
    };

    assert_eq!(result.employee_uuid, employee_uuid);
    assert_eq!(result.position, deploy_position);
    assert_eq!(
        result.facings.right.range_previews.basic_attack.cells,
        vec![
            deploy_position,
            Position::new(deploy_position.x + 1, deploy_position.y)
        ]
    );
    assert!(!result.facings.right.range_previews.active_skill.available);
    assert!(!result
        .facings
        .up
        .range_previews
        .basic_attack
        .cells
        .is_empty());
    assert!(!result
        .facings
        .down
        .range_previews
        .basic_attack
        .cells
        .is_empty());
    assert!(!result
        .facings
        .left
        .range_previews
        .basic_attack
        .cells
        .is_empty());

    let after_update = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_update_dto_after(None, &core.state.roster)
        .unwrap();
    assert_eq!(
        after_update.events_delta.to_seq, before_update.events_delta.to_seq,
        "preview must not emit event_log events"
    );
    let after_deployment = after_update.checkpoint.deployment.as_ref().unwrap();
    assert_eq!(after_deployment.current_cost, before_cost);
    assert!(after_deployment.deployed_units.is_empty());
    assert!(!after_update
        .checkpoint
        .units
        .iter()
        .any(|unit| matches!(unit.unit_source, BattleUnitSourceIdentity::Employee { .. })));
}

#[test]
fn live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );

    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = ground_deployment_cell_with_right_range(&preview);
    let employee_ids = core.roster().unwrap().available_employee_ids();
    let employee_uuid = employee_ids[0];
    let second_employee_uuid = employee_ids[1];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let pending_range_preview = core
        .deployment_range_preview_dto(employee_uuid, deploy_position, FacingDirection::Right)
        .unwrap();
    assert!(pending_range_preview.basic_attack.available);
    assert_eq!(
        pending_range_preview.basic_attack.cells,
        vec![
            deploy_position,
            Position::new(deploy_position.x + 1, deploy_position.y)
        ]
    );
    assert!(!pending_range_preview.active_skill.available);

    let deployed = core
        .execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id,
        battle_update,
        ..
    } = deployed
    else {
        panic!("expected live deploy result");
    };
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live deploy update should expose deployment state");
    assert_eq!(
        deployment.current_cost,
        run_policy()
            .live_deployment
            .initial_cost
            .saturating_sub(run_policy().live_deployment.base_deploy_cost)
    );
    assert_eq!(deployment.deployed_units.len(), 1);
    assert_eq!(deployment.deployed_units[0].employee_uuid, employee_uuid);
    assert_eq!(
        deployment.deployed_units[0].unit_instance_id,
        unit_instance_id
    );
    assert_eq!(deployment.deployed_units[0].position, deploy_position);
    assert_eq!(deployment.deployed_units[0].facing, FacingDirection::Right);
    let update = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_update_dto_after(None, &core.state.roster)
        .unwrap();
    assert_eq!(update.checkpoint.at_seq, update.events_delta.to_seq);
    assert!(update.events_delta.events.iter().any(|entry| matches!(
        entry.event,
        LiveBattlePresentationEventKindDto::UnitDeployed {
            employee_uuid: event_employee_uuid,
            unit_instance_id: event_unit_instance_id,
            position,
            facing,
        } if event_employee_uuid == employee_uuid
            && event_unit_instance_id == unit_instance_id
            && position == deploy_position
            && facing == FacingDirection::Right
    )));
    assert!(update
        .checkpoint
        .deployment
        .as_ref()
        .is_some_and(|deployment| {
            deployment.deployed_units.iter().any(|unit| {
                unit.employee_uuid == employee_uuid
                    && unit.unit_instance_id == unit_instance_id
                    && unit.position == deploy_position
                    && unit.facing == FacingDirection::Right
            })
        }));
    let checkpoint_unit = update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.unit_instance_id == unit_instance_id)
        .expect("deployed unit should be present in checkpoint");
    assert_eq!(
        checkpoint_unit.range_previews.basic_attack.cells,
        pending_range_preview.basic_attack.cells
    );
    assert_eq!(
        checkpoint_unit.range_previews.active_skill.reason,
        pending_range_preview.active_skill.reason
    );
    if let Some(deployment) = core
        .state
        .active_battle
        .as_mut()
        .and_then(|battle| battle.live_deployment.as_mut())
    {
        deployment.current_cost = run_policy().live_deployment.base_deploy_cost;
    }
    let second_deployed = core
        .execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid: second_employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .expect("allied deployment should allow shared deployment tiles");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: second_unit_instance_id,
        ..
    } = second_deployed
    else {
        panic!("expected second live deploy result");
    };
    let expected_withdraw_redeploy_hp = {
        let active = core
            .state
            .active_battle
            .as_mut()
            .expect("active battle should remain available");
        let unit = active
            .battle
            .units
            .get_mut(&unit_instance_id)
            .expect("deployed unit should exist in battle core");
        let damaged_hp = unit.stats.max_health / 2;
        unit.stats.current_health = damaged_hp;
        damaged_hp
            .saturating_add(
                unit.stats.max_health.saturating_mul(
                    BattleDeployCurrentHpPolicy::WITHDRAW_REDEPLOY_RECOVERY_PERCENT,
                ) / 100,
            )
            .min(unit.stats.max_health)
    };

    let withdrawn = core
        .execute(player_id, PlayerBehavior::WithdrawUnit { employee_uuid })
        .unwrap();
    let BehaviorResult::BattleUnitWithdrawn {
        unit_instance_id: withdrawn_unit,
        battle_update,
        ..
    } = withdrawn
    else {
        panic!("expected live withdraw result");
    };
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live withdraw update should expose deployment state");
    assert_eq!(withdrawn_unit, unit_instance_id);
    assert!(deployment.deployed_units.iter().any(|unit| {
        unit.employee_uuid == second_employee_uuid
            && unit.unit_instance_id == second_unit_instance_id
            && unit.position == deploy_position
    }));
    assert_eq!(deployment.redeploying_units.len(), 1);
    assert_eq!(deployment.redeploying_units[0].employee_uuid, employee_uuid);
    assert_eq!(deployment.redeploying_units[0].ready_at_ms, 30_000);
    assert_eq!(
        deployment.redeploying_units[0].deploy_cost,
        run_policy()
            .live_deployment
            .base_deploy_cost
            .saturating_mul(run_policy().live_deployment.redeploy_cost_multiplier_pct)
            / 100
    );

    {
        let active = core
            .state
            .active_battle
            .as_mut()
            .expect("active battle should remain available");
        let ready_at_ms = active.execution.last_event_time_ms();
        let deployment = active
            .live_deployment
            .as_mut()
            .expect("live deployment should remain available");
        deployment.current_cost = deployment
            .redeploy_locks
            .get(&employee_uuid)
            .expect("withdraw should create redeploy lock")
            .deploy_cost;
        deployment
            .redeploy_locks
            .get_mut(&employee_uuid)
            .expect("withdraw should create redeploy lock")
            .ready_at_ms = ready_at_ms;
    }
    let redeployed = core
        .execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .expect("withdrawn employee should redeploy after lock is ready");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: redeployed_unit_instance_id,
        battle_update,
        ..
    } = redeployed
    else {
        panic!("expected redeploy result");
    };
    assert_ne!(
        redeployed_unit_instance_id, unit_instance_id,
        "redeploy must create a new runtime unit instance"
    );
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("redeploy update should expose deployment state");
    assert!(deployment.redeploying_units.is_empty());
    assert!(deployment.deployed_units.iter().any(|unit| {
        unit.employee_uuid == employee_uuid && unit.unit_instance_id == redeployed_unit_instance_id
    }));
    let active = core
        .state
        .active_battle
        .as_ref()
        .expect("active battle should remain available");
    assert!(matches!(
        active
            .battle
            .units
            .get(&unit_instance_id)
            .expect("old withdrawn unit should remain in registry")
            .lifecycle,
        crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn
    ));
    let redeployed_unit = active
        .battle
        .units
        .get(&redeployed_unit_instance_id)
        .expect("redeployed unit should exist in battle core");
    assert!(redeployed_unit.is_active());
    assert_eq!(
        redeployed_unit.stats.current_health,
        expected_withdraw_redeploy_hp
    );
    assert_eq!(redeployed_unit.current_target, None);
    assert!(!redeployed_unit.pending_basic_attack);
    assert!(!redeployed_unit.pending_cast);
}

#[test]
fn live_deployment_reconciles_defeated_player_unit_before_state_dto() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 124);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );

    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let deployed = core
        .execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id, ..
    } = deployed
    else {
        panic!("expected live deploy result");
    };

    {
        let active = core
            .state
            .active_battle
            .as_mut()
            .expect("active battle should remain available");
        let unit = active
            .battle
            .units
            .get_mut(&unit_instance_id)
            .expect("deployed unit should exist in battle core");
        assert_eq!(
            unit.body.projected_tile(),
            deploy_position,
            "deployed unit position should be sourced from RuntimeUnit body"
        );
        unit.stats.current_health = 0;
        unit.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
    }

    let state = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState { since_seq: None },
        )
        .unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = state else {
        panic!("expected battle state after deployment reconciliation");
    };
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("battle state should expose live deployment");
    assert!(
        deployment.deployed_units.is_empty(),
        "defeated units removed from battlefield must not remain deployed in DTO state"
    );
    assert_eq!(deployment.redeploying_units.len(), 1);
    assert_eq!(deployment.redeploying_units[0].employee_uuid, employee_uuid);
    assert_eq!(deployment.redeploying_units[0].ready_at_ms, 90_000);
    assert_eq!(
        deployment.redeploying_units[0].deploy_cost,
        run_policy()
            .live_deployment
            .base_deploy_cost
            .saturating_mul(run_policy().live_deployment.redeploy_cost_multiplier_pct)
            / 100
    );

    {
        let active = core
            .state
            .active_battle
            .as_mut()
            .expect("active battle should remain available");
        let ready_at_ms = active.execution.last_event_time_ms();
        let deployment = active
            .live_deployment
            .as_mut()
            .expect("live deployment should remain available");
        deployment.current_cost = deployment
            .redeploy_locks
            .get(&employee_uuid)
            .expect("death should create redeploy lock")
            .deploy_cost;
        deployment
            .redeploy_locks
            .get_mut(&employee_uuid)
            .expect("death should create redeploy lock")
            .ready_at_ms = ready_at_ms;
    }
    let redeployed = core
        .execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .expect("defeated employee should redeploy after lock is ready");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: redeployed_unit_instance_id,
        battle_update,
        ..
    } = redeployed
    else {
        panic!("expected death redeploy result");
    };
    assert_ne!(
        redeployed_unit_instance_id, unit_instance_id,
        "death redeploy must create a new runtime unit instance"
    );
    let active = core
        .state
        .active_battle
        .as_ref()
        .expect("active battle should remain available");
    assert!(matches!(
        active
            .battle
            .units
            .get(&unit_instance_id)
            .expect("old dead unit should remain in registry")
            .lifecycle,
        crate::game::battle::core::types::RuntimeUnitLifecycle::Dead
    ));
    let redeployed_unit = active
        .battle
        .units
        .get(&redeployed_unit_instance_id)
        .expect("redeployed unit should exist in battle core");
    let expected_death_redeploy_hp = redeployed_unit
        .stats
        .max_health
        .saturating_mul(BattleDeployCurrentHpPolicy::DEATH_REDEPLOY_PERCENT)
        / 100;
    assert_eq!(
        redeployed_unit.stats.current_health,
        expected_death_redeploy_hp
            .max(1)
            .min(redeployed_unit.stats.max_health)
    );
    assert!(redeployed_unit.is_active());
    assert_eq!(redeployed_unit.current_target, None);
    assert!(!redeployed_unit.pending_basic_attack);
    assert!(!redeployed_unit.pending_cast);
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("redeploy update should expose deployment state");
    assert!(deployment.redeploying_units.is_empty());
    assert!(deployment.deployed_units.iter().any(|unit| {
        unit.employee_uuid == employee_uuid && unit.unit_instance_id == redeployed_unit_instance_id
    }));
}

#[test]
fn deploy_cost_reduction_consumable_reduces_live_deployment_cost_for_employee() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );

    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    let consumable = consumable_meta(
        0xD3F3_3001,
        "deploy_cost_capsule",
        ConsumableTier::Uncommon,
        ConsumableEffect::DeployCostReduction { percent: 50 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xD3F3_3002), &consumable);

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let deployment = core
        .state
        .active_battle
        .as_ref()
        .and_then(|active| active.live_deployment_dto(&core.state.roster))
        .expect("live deployment dto");
    let reduced_cost = run_policy()
        .live_deployment
        .base_deploy_cost
        .saturating_sub(run_policy().live_deployment.base_deploy_cost.div_ceil(2));
    let unit_cost = deployment
        .unit_deploy_costs
        .iter()
        .find(|cost| cost.employee_uuid == employee_uuid)
        .expect("employee deploy cost dto");
    assert_eq!(
        unit_cost.base_deploy_cost,
        run_policy().live_deployment.base_deploy_cost
    );
    assert_eq!(unit_cost.effective_deploy_cost, reduced_cost);

    let deployed = core
        .execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position: deploy_position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();

    let BehaviorResult::BattleUnitDeployed { battle_update, .. } = deployed else {
        panic!("expected live deploy result");
    };
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live deploy update should expose deployment state");
    assert_eq!(
        deployment.current_cost,
        run_policy()
            .live_deployment
            .initial_cost
            .saturating_sub(reduced_cost)
    );
}

#[test]
fn activate_skill_uses_equipped_manual_fragment_in_live_defense() {
    let skill_id = SkillId::from("manual_fragment_guard");
    let fragment = active_skill_fragment("manual_fragment", 0xD3F3_2001, skill_id.as_str());
    let skill = SkillDef {
        id: skill_id.clone(),
        name: skill_id.to_string(),
        kind: Default::default(),
        cast_targeting: SkillCastTargetingDef::explicit(
            SkillTarget::SelfUnit,
            Default::default(),
            None,
            false,
        ),
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "self_guard".to_string(),
            delay_ms: 0,
            range_policy: Default::default(),
            defense_tile_range: None,
            air_capable: false,
            target: SkillTarget::SelfUnit,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![],
            presentation: Default::default(),
        }],
    };
    let weapon = weapon_equipment(0xE005, "manual_fragment_sword", WeaponArchetype::Sword);
    let mut core = GameCore::new(
        game_data_with_pve_equipment_and_active_skill_fragments(
            vec![weapon.clone()],
            vec![skill],
            vec![fragment.clone()],
        ),
        123,
    );
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        weapon,
        Uuid::from_u128(0xE005_0001),
    );
    let consumable = consumable_meta(
        0xD3F3_3003,
        "skill_charge_serum",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xD3F3_3004), &consumable);
    core.state.skill_fragments.add(&fragment).unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            fragment_id: fragment.id.clone(),
        },
    )
    .unwrap();
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = first_ground_deployment_cell(&preview);
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::ActivateSkill));
    core.execute(
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let snapshot = core.get_run_snapshot_json().unwrap();
    let catalog_skills = snapshot["skill_catalog"]["skills"].as_array().unwrap();
    let catalog_skill = catalog_skills
        .iter()
        .find(|skill| skill["skill_id"] == json!(skill_id))
        .expect("equipped manual skill should be exposed in skill catalog");
    assert_eq!(catalog_skill["display_name"], skill_id.as_str());
    assert_eq!(catalog_skill["steps"][0]["delivery"], "instant");

    let deployed_units = snapshot["game_state_context"]["deployment"]["deployed_units"]
        .as_array()
        .unwrap();
    let deployed = deployed_units
        .iter()
        .find(|unit| unit["employee_uuid"] == json!(employee_uuid))
        .expect("deployed employee should be exposed in live deployment snapshot");
    assert_eq!(deployed["position"], json!(deploy_position));
    assert_eq!(deployed["skill_readiness"]["skill_id"], json!(skill_id));
    assert_eq!(deployed["skill_readiness"]["activation_mode"], "manual");
    assert_eq!(
        deployed["skill_readiness"]["manual_activation_allowed"],
        true
    );
    assert_eq!(deployed["skill_readiness"]["target_required"], false);
    assert_eq!(deployed["skill_readiness"]["target_available"], true);
    assert_eq!(
        deployed["skill_readiness"]["resonance_current"],
        deployed["skill_readiness"]["resonance_max"]
    );

    let activated = core
        .execute(
            player_id,
            PlayerBehavior::ActivateSkill {
                employee_uuid,
                skill_id: skill_id.clone(),
                target: None,
            },
        )
        .unwrap();
    let BehaviorResult::BattleSkillActivated {
        battle_update,
        unit_instance_id,
        ..
    } = activated
    else {
        panic!("expected live skill activation result");
    };
    let events = &battle_update.events_delta.events;
    assert!(events.iter().any(|entry| {
        matches!(
            &entry.event,
            LiveBattlePresentationEventKindDto::ManualCastStart {
                caster_instance_id,
                skill_id: actual_skill_id,
                ..
            } if *caster_instance_id == unit_instance_id && *actual_skill_id == skill_id
        )
    }));

    let advanced = core.advance_active_battle_by(1).unwrap();
    let BehaviorResult::BattleAdvanced { battle_update, .. } = advanced else {
        panic!("expected battle advance after manual cast");
    };
    let events = &battle_update.events_delta.events;
    assert!(events.iter().any(|entry| {
        matches!(
            &entry.event,
            LiveBattlePresentationEventKindDto::AbilityCast {
                caster_instance_id,
                skill_id: actual_skill_id,
                ..
            } if *caster_instance_id == unit_instance_id && *actual_skill_id == skill_id
        )
    }));
}

#[test]
fn manual_fragment_readiness_keeps_button_enabled_when_target_is_missing() {
    let skill_id = SkillId::from("manual_fragment_mark_target");
    let fragment = active_skill_fragment("target_fragment", 0xD3F3_2005, skill_id.as_str());
    let tile_range = crate::game::battle::tile_range::TileRangePattern {
        include_anchor_tile: false,
        rows: vec![".@X".to_string()],
    };
    let skill = SkillDef {
        id: skill_id.clone(),
        name: skill_id.to_string(),
        kind: Default::default(),
        cast_targeting: SkillCastTargetingDef::explicit(
            SkillTarget::EnemySingle {
                rule: Default::default(),
            },
            Default::default(),
            Some(tile_range.clone()),
            false,
        ),
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "mark".to_string(),
            delay_ms: 0,
            range_policy: Default::default(),
            defense_tile_range: Some(tile_range),
            air_capable: false,
            target: SkillTarget::EnemySingle {
                rule: Default::default(),
            },
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![],
            presentation: Default::default(),
        }],
    };
    let weapon = weapon_equipment(0xE006, "target_fragment_sword", WeaponArchetype::Sword);
    let mut core = GameCore::new(
        game_data_with_pve_equipment_and_active_skill_fragments(
            vec![weapon.clone()],
            vec![skill],
            vec![fragment.clone()],
        ),
        123,
    );
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        weapon,
        Uuid::from_u128(0xE006_0001),
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .combat_profile
        .battle_profile
        .resonance
        .start = 100;
    core.state.skill_fragments.add(&fragment).unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            fragment_id: fragment.id.clone(),
        },
    )
    .unwrap();
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let deploy_position = first_ground_deployment_cell(&preview);
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();

    let snapshot = core.get_run_snapshot_json().unwrap();
    let deployed_units = snapshot["game_state_context"]["deployment"]["deployed_units"]
        .as_array()
        .unwrap();
    let deployed = deployed_units
        .iter()
        .find(|unit| unit["employee_uuid"] == json!(employee_uuid))
        .expect("deployed employee should be exposed in live deployment snapshot");
    assert_eq!(deployed["skill_readiness"]["skill_id"], json!(skill_id));
    assert_eq!(
        deployed["skill_readiness"]["manual_activation_allowed"], true,
        "manual button should remain available so Unity can open target selection"
    );
    assert_eq!(deployed["skill_readiness"]["target_required"], true);
    assert_eq!(deployed["skill_readiness"]["target_available"], false);
    assert_eq!(
        deployed["skill_readiness"]["target_block_reason"],
        "no_valid_target"
    );
    assert!(deployed["skill_readiness"]["can_activate_reason"].is_null());
}

#[test]
fn manual_fragment_can_restore_stabilization_and_enable_extra_deployment() {
    let skill_id = SkillId::from("manual_fragment_stabilize");
    let fragment = active_skill_fragment("stabilization_fragment", 0xD3F3_2002, skill_id.as_str());
    let skill = SkillDef {
        id: skill_id.clone(),
        name: skill_id.to_string(),
        kind: Default::default(),
        cast_targeting: SkillCastTargetingDef::explicit(
            SkillTarget::SelfUnit,
            Default::default(),
            None,
            false,
        ),
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "restore_stabilization".to_string(),
            delay_ms: 0,
            range_policy: Default::default(),
            defense_tile_range: None,
            air_capable: false,
            target: SkillTarget::SelfUnit,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![SkillEffectDef::ModifyStabilization {
                amount: run_policy().live_deployment.max_cost as i32 + 50,
            }],
            presentation: Default::default(),
        }],
    };
    let weapon = weapon_equipment(
        0xE007,
        "stabilization_fragment_sword",
        WeaponArchetype::Sword,
    );
    let mut core = GameCore::new(
        game_data_with_pve_equipment_and_active_skill_fragments(
            vec![weapon.clone()],
            vec![skill],
            vec![fragment.clone()],
        ),
        123,
    );
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_ids = core.roster().unwrap().available_employee_ids();
    let caster_uuid = employee_ids[0];
    let second_uuid = employee_ids[1];
    let third_uuid = employee_ids[2];
    grant_and_equip_weapon(&mut core, caster_uuid, weapon, Uuid::from_u128(0xE007_0001));
    core.roster_mut()
        .unwrap()
        .get_mut(&caster_uuid)
        .unwrap()
        .combat_profile
        .battle_profile
        .resonance
        .start = 100;
    core.state.skill_fragments.add(&fragment).unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: caster_uuid,
            fragment_id: fragment.id.clone(),
        },
    )
    .unwrap();

    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(combat_preview),
        ..
    } = &preview
    else {
        panic!("expected node preview");
    };
    let mut deploy_positions = combat_preview
        .deployment_zones
        .iter()
        .flat_map(|zone| zone.cells.iter().copied())
        .collect::<Vec<_>>();
    deploy_positions.sort_by_key(|position| (position.y, position.x));
    deploy_positions.dedup();
    let route_end = combat_preview.routes.first().map(|route| route.end);
    deploy_positions.retain(|position| Some(*position) != route_end);
    assert!(deploy_positions.len() >= 3);

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    for (employee_uuid, position) in [
        (caster_uuid, deploy_positions[0]),
        (second_uuid, deploy_positions[1]),
    ] {
        core.execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
    }

    let insufficient = core
        .execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid: third_uuid,
                position: deploy_positions[2],
                facing: FacingDirection::Right,
            },
        )
        .expect_err("third deployment should require restored stabilization");
    assert!(matches!(insufficient, GameError::InsufficientResources));

    core.execute(
        player_id,
        PlayerBehavior::ActivateSkill {
            employee_uuid: caster_uuid,
            skill_id: skill_id.clone(),
            target: None,
        },
    )
    .unwrap();
    let advanced = core.advance_active_battle_by(1).unwrap();
    let BehaviorResult::BattleAdvanced { battle_update, .. } = advanced else {
        panic!("expected live deployment after stabilization skill");
    };
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live deployment should be exposed after stabilization skill");
    assert_eq!(
        deployment.current_cost,
        run_policy().live_deployment.max_cost
    );

    let deployed = core
        .execute(
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid: third_uuid,
                position: deploy_positions[2],
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
    let BehaviorResult::BattleUnitDeployed { battle_update, .. } = deployed else {
        panic!("expected third deployment after stabilization restore");
    };
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live deploy update should expose deployment state");
    assert_eq!(deployment.deployed_units.len(), 3);
    assert_eq!(
        deployment.current_cost,
        run_policy()
            .live_deployment
            .max_cost
            .saturating_sub(run_policy().live_deployment.base_deploy_cost)
    );
}

#[test]
fn retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["game_state_context"]["type"], "node_confirm");
    assert_eq!(
        snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
        json!(3)
    );
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(matches!(
        result,
        BehaviorResult::BattleAdvanced {
            finished: false,
            ..
        }
    ));
    assert!(matches!(core.get_state(), GameState::InBattle { .. }));
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::RetreatBattle));
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["game_state_context"]["type"], "in_battle");
    assert_eq!(snapshot["game_state_context"]["can_retreat"], json!(true));
    assert_eq!(
        snapshot["game_state_context"]["abnormality_attempt"]["attempts_started"],
        json!(1)
    );
    assert_eq!(
        snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
        json!(2)
    );
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    let employee_before = core.roster().unwrap().get(&employee_uuid).unwrap().clone();

    let result = core
        .execute(player_id, PlayerBehavior::RetreatBattle)
        .unwrap();

    let BehaviorResult::NodePreview {
        node_id: retreated_node_id,
        ..
    } = result
    else {
        panic!("expected retreat to return to node confirm while attempts remain");
    };
    assert_eq!(retreated_node_id, node_id);
    assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(
        snapshot["game_state_context"]["abnormality_attempt"]["attempts_started"],
        json!(1)
    );
    assert_eq!(
        snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
        json!(2)
    );
    assert!(core.state.active_battle.is_none());
    assert!(core.state.node_session.is_some());
    assert!(core.state.active_node_content.is_none());
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .node(node_id)
        .is_some_and(|node| {
            node.state == crate::game::map::MapNodeState::Unavailable
                && node.visibility == crate::game::map::MapNodeVisibility::Revealed
        }));
    let employee_after = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert_eq!(
        employee_after.health.current_hp,
        employee_before.health.current_hp
    );
    assert_eq!(employee_after.trauma, employee_before.trauma);
    assert_eq!(employee_after.experience, employee_before.experience);

    for expected_remaining in [1, 0] {
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        let result = core
            .execute(player_id, PlayerBehavior::RetreatBattle)
            .unwrap();

        if expected_remaining > 0 {
            let BehaviorResult::NodePreview { .. } = result else {
                panic!("expected retreat to keep node alive before attempts are exhausted");
            };
            assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
            let snapshot = core.get_run_snapshot_json().unwrap();
            assert_eq!(
                snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
                json!(expected_remaining)
            );
            continue;
        }

        let BehaviorResult::NodeCompleted {
            outcome: Some(outcome),
            ..
        } = result
        else {
            panic!("expected third retreat to consume the exhausted abnormality");
        };
        assert!(!outcome.mission_success);
        assert_eq!(outcome.node_id, node_id);
        let combat = outcome.combat.expect("combat summary");
        assert_eq!(
            combat.node_type,
            crate::game::combat_preview::CombatNodeType::Defense
        );
        assert_eq!(combat.winner, BattleWinner::Draw);
        assert!(combat.retreated);
        assert!(outcome.employee_changes.is_empty());
        assert!(outcome.inventory_diff.added.is_empty());
        assert!(outcome.research_deliveries.is_empty());
        assert!(matches!(core.get_state(), GameState::ViewingMap));
        assert!(core.state.active_battle.is_none());
        assert!(core.state.node_session.is_none());
        assert!(core.state.active_node_content.is_none());
        let record = core
            .battle_records()
            .iter()
            .find(|record| record.abnormality_uuid == node_id.0)
            .expect("exhausted retreat should record the completed draw battle");
        assert_eq!(record.winner, BattleWinner::Draw);
        assert!(record
            .event_log
            .entries
            .iter()
            .any(|entry| matches!(entry.event, BattleLogEvent::BattleStart { .. })));
        let record_path = core
            .state
            .run
            .as_ref()
            .unwrap()
            .battle_record_debug_export_path(record.abnormality_uuid);
        assert!(record_path.exists(), "draw battle record file should exist");
        let record_json: serde_json::Value = serde_json::from_reader(
            std::fs::File::open(&record_path).expect("open draw battle record file"),
        )
        .expect("draw battle record should be valid json");
        assert_eq!(record_json["winner"], json!("Draw"));
        std::fs::remove_file(&record_path).expect("cleanup draw battle record file");
        assert!(core
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(node_id)
            .is_some_and(|node| node.state == crate::game::map::MapNodeState::Completed));
    }
}

#[test]
fn failed_live_defense_battle_reenters_until_attempts_are_exhausted() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("defense_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    let BehaviorResult::NodePreview {
        node_id: failed_node_id,
        ..
    } = result
    else {
        panic!("expected failed combat to return to node confirm while attempts remain");
    };
    assert_eq!(failed_node_id, node_id);
    assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(
        snapshot["game_state_context"]["abnormality_attempt"]["attempts_started"],
        json!(1)
    );
    assert_eq!(
        snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
        json!(2)
    );
    assert!(core.state.active_battle.is_none());
    assert!(core.state.active_node_content.is_none());
    assert!(core.state.node_session.is_some());
}

#[test]
fn final_boss_retreat_is_allowed_until_third_attempt_fails_run() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let run = core.state.run.as_mut().expect("run state");
        run.run_progression.mode_state = crate::game::map::RunProgressionModeState::Standard {
            floor_index: run_policy().setup.standard_floor_count - 1,
            max_floors: run_policy().setup.standard_floor_count,
        };
    }
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "final_boss_risk_encounter",
    );
    core.state.run.as_mut().unwrap().map.terminal_node_id = node_id;
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();

    for expected_remaining in [2, 1] {
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        let snapshot = core.get_run_snapshot_json().unwrap();
        assert_eq!(snapshot["game_state_context"]["can_retreat"], json!(true));
        assert_eq!(
            snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
            json!(expected_remaining)
        );
        let result = core
            .execute(player_id, PlayerBehavior::RetreatBattle)
            .unwrap();
        assert!(matches!(result, BehaviorResult::NodePreview { .. }));
        assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    }

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::RetreatBattle)
        .unwrap();

    let BehaviorResult::RunFailed {
        reason: RunFailureReason::BossDefeated,
        outcome: Some(outcome),
    } = result
    else {
        panic!("expected final boss third retreat to fail the run");
    };
    assert_eq!(outcome.node_id, node_id);
    let combat = outcome.combat.expect("combat summary");
    assert_eq!(
        combat.node_type,
        crate::game::combat_preview::CombatNodeType::Boss
    );
    assert_eq!(combat.winner, BattleWinner::Draw);
    assert!(combat.retreated);
    assert!(matches!(
        core.get_state(),
        GameState::RunFailed {
            reason: RunFailureReason::BossDefeated
        }
    ));
}

#[test]
fn final_boss_defeat_immediately_fails_run() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let run = core.state.run.as_mut().expect("run state");
        run.run_progression.mode_state = crate::game::map::RunProgressionModeState::Standard {
            floor_index: run_policy().setup.standard_floor_count - 1,
            max_floors: run_policy().setup.standard_floor_count,
        };
    }
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "final_boss_risk_encounter",
    );
    core.state.run.as_mut().unwrap().map.terminal_node_id = node_id;
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
        encounter_id: "final_boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::RunFailed {
            reason: RunFailureReason::BossDefeated,
            ..
        }
    ));
    assert!(matches!(
        core.get_state(),
        GameState::RunFailed {
            reason: RunFailureReason::BossDefeated
        }
    ));
}

#[test]
fn forced_boss_omen_defeat_immediately_fails_endless_run() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_omen_final_boss",
        "final_boss_risk_encounter",
    );
    core.state.run.as_mut().unwrap().boss_omen.forced_boss =
        Some(crate::game::boss_omen::ForcedBossOmenNodeState {
            node_id,
            chain_id: crate::game::data::boss_omen_data::BossOmenChainId::new("test_chain"),
            boss_abnormality_id: "final_boss_risk_abno".to_string(),
            encounter_id: "final_boss_risk_encounter".to_string(),
        });
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
        encounter_id: "final_boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    assert!(matches!(
        result,
        BehaviorResult::RunFailed {
            reason: RunFailureReason::BossDefeated,
            ..
        }
    ));
}

#[test]
fn forced_boss_omen_victory_clears_chain_and_advances_endless_floor() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_omen_final_boss",
        "final_boss_risk_encounter",
    );
    {
        let run = core.state.run.as_mut().unwrap();
        run.map.terminal_node_id = node_id;
        run.boss_omen.forced_boss = Some(crate::game::boss_omen::ForcedBossOmenNodeState {
            node_id,
            chain_id: crate::game::data::boss_omen_data::BossOmenChainId::new("test_chain"),
            boss_abnormality_id: "final_boss_risk_abno".to_string(),
            encounter_id: "final_boss_risk_encounter".to_string(),
        });
    }
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
        encounter_id: "final_boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Player, &BattleEventLog::default(), &[]),
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    let BehaviorResult::CombatRewardsGranted { completion, .. } = result else {
        panic!("forced boss victory should grant combat rewards before advancing");
    };
    let BehaviorResult::FloorAdvanced {
        game_mode,
        floor_index,
        ..
    } = *completion
    else {
        panic!("forced boss victory should advance Endless floor after rewards");
    };
    assert_eq!(game_mode, GameMode::Endless);
    assert_eq!(floor_index, 1);
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .boss_omen
            .forced_boss_node_id(),
        None
    );
    assert!(core.state.run.as_ref().unwrap().boss_omen.active.is_none());
}

#[test]
fn non_final_boss_defeat_reenters_while_attempts_remain() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "boss_risk_encounter",
    );
    assert_ne!(
        core.state.run.as_ref().unwrap().map.terminal_node_id,
        node_id
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("boss_abno".to_string()),
        encounter_id: "boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    assert!(matches!(result, BehaviorResult::NodePreview { .. }));
    assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(
        snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
        json!(2)
    );
}

#[test]
fn event_started_combat_victory_consumes_event_node() {
    let event = combat_choice_event_definition(
        "event_started_victory",
        "fight",
        "defense_encounter",
        "defense_risk_abno",
    );
    let game_data = game_data_with_event_definitions_for_combat_tests(
        game_data_with_pve_encounters(),
        vec![event],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Standard);
    let event_node_id =
        force_single_event_node_run_for_combat_tests(&mut core, "event_started_victory");
    enter_event_combat_choice(
        &mut core,
        player_id,
        event_node_id,
        "event_started_victory",
        "fight",
    );
    let battle = CombatBattleState {
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: event_node_id.0,
        winner: BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Player, &BattleEventLog::default(), &[]),
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: event_node_id.0,
    })
    .unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    let BehaviorResult::CombatRewardsGranted { completion, .. } = result else {
        panic!("terminal Event combat victory should grant combat rewards");
    };
    assert!(
        matches!(*completion, BehaviorResult::FloorAdvanced { .. }),
        "terminal Event node victory should consume the node and advance after rewards"
    );
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .is_none());
}

#[test]
fn event_started_combat_defeat_reenters_while_attempts_remain() {
    let event = combat_choice_event_definition(
        "event_started_defeat",
        "fight",
        "defense_encounter",
        "defense_risk_abno",
    );
    let game_data = game_data_with_event_definitions_for_combat_tests(
        game_data_with_pve_encounters(),
        vec![event],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Standard);
    let event_node_id =
        force_single_event_node_run_for_combat_tests(&mut core, "event_started_defeat");
    enter_event_combat_choice(
        &mut core,
        player_id,
        event_node_id,
        "event_started_defeat",
        "fight",
    );
    let battle = CombatBattleState {
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: event_node_id.0,
        winner: BattleWinner::Opponent,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: event_node_id.0,
    })
    .unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    assert!(matches!(result, BehaviorResult::NodePreview { .. }));
    assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    let session = core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .expect("event combat session should remain for retry");
    assert_eq!(
        session.committed_choice_id.as_ref().map(|id| id.as_str()),
        Some("fight")
    );
    assert!(session.started_combat.is_some());
}

#[test]
fn retreat_marks_rumor_threat_warning_disproved_on_reentry_preview() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let active = core
        .state
        .active_battle
        .as_mut()
        .expect("battle should be active");
    active.combat_preview.threat_warnings = vec![
        ThreatWarning {
            tag: ThreatWarningTag::ArmoredEnemyPossible,
            status: ThreatWarningStatus::Unverified,
            source: ThreatWarningSource::Briefing,
        },
        ThreatWarning {
            tag: ThreatWarningTag::AirEnemyPossible,
            status: ThreatWarningStatus::Unverified,
            source: ThreatWarningSource::Rumor,
        },
    ];

    let result = core
        .execute(player_id, PlayerBehavior::RetreatBattle)
        .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(preview),
        ..
    } = result
    else {
        panic!("retreat should return node preview while attempts remain");
    };

    assert!(preview.threat_warnings.iter().any(|warning| {
        warning.source == ThreatWarningSource::Briefing
            && warning.status == ThreatWarningStatus::Unverified
    }));
    assert!(preview.threat_warnings.iter().any(|warning| {
        warning.source == ThreatWarningSource::Rumor
            && warning.status == ThreatWarningStatus::Disproved
    }));

    let cached = core
        .state
        .run
        .as_ref()
        .unwrap()
        .combat_previews
        .get(&node_id)
        .expect("retreat should update cached node preview");
    assert!(cached.threat_warnings.iter().any(|warning| {
        warning.source == ThreatWarningSource::Rumor
            && warning.status == ThreatWarningStatus::Disproved
    }));
}

#[test]
fn consumable_modifier_survives_retreat_reentry_and_expires_when_abnormality_is_resolved() {
    let consumable = consumable_meta(
        0xC030,
        "reentry_ampoule",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 20 },
    );
    let owned_uuid = Uuid::from_u128(0xC030_0001);
    let mut core = GameCore::new(
        game_data_with_pve_and_consumables(vec![consumable.clone()]),
        123,
    );
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().available_employee_ids()[0];
    core.inventory_mut()
        .unwrap()
        .consumables
        .add_item(OwnedConsumable::new(owned_uuid, Arc::new(consumable)))
        .unwrap();

    core.execute(
        player_id,
        PlayerBehavior::UseConsumableItem {
            item_uuid: owned_uuid,
            target_employee_uuid: employee_uuid,
        },
    )
    .unwrap();
    assert_eq!(
        core.roster()
            .unwrap()
            .get(&employee_uuid)
            .unwrap()
            .active_consumable_modifier
            .as_ref()
            .unwrap()
            .remaining_combat_nodes,
        1
    );

    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();

    for expected_remaining_attempts in [2, 1] {
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core.execute(player_id, PlayerBehavior::RetreatBattle)
            .unwrap();
        assert_eq!(
            core.roster()
                .unwrap()
                .get(&employee_uuid)
                .unwrap()
                .active_consumable_modifier
                .as_ref()
                .unwrap()
                .remaining_combat_nodes,
            1
        );
        let snapshot = core.get_run_snapshot_json().unwrap();
        assert_eq!(
            snapshot["game_state_context"]["abnormality_attempt"]["remaining_attempts"],
            json!(expected_remaining_attempts)
        );
    }

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::RetreatBattle)
        .unwrap();
    assert!(matches!(
        result,
        BehaviorResult::NodeCompleted {
            outcome: Some(_),
            ..
        }
    ));
    assert!(core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .active_consumable_modifier
        .is_none());
}
