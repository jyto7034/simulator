use super::*;
use crate::game::ability::SkillEffectDef;
use crate::game::battle::core::movement::types::EventLogVec2;
use crate::game::battle::core::sim::BattleDeployCurrentHpPolicy;
use crate::game::battle::event_log::{BattleEventLogEntry, BattleLogEvent};
use crate::game::battle::result_stats::collect_battle_result_stats;
use crate::game::battle::tile_range::TileRangePattern;
use crate::game::battle::types::{
    BattleUnitRole, BattleUnitSourceIdentity, BattleUnitThreatClass, BattleWinner,
    ParticipantBattleResult,
};
use crate::game::behavior::ActionDisabledReasonCode;
use crate::game::behavior::{LiveBattleHudBarMode, LiveBattlePresentationEventKindDto};
use crate::game::data::Item;
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

fn encounter_compendium_spawn_log() -> BattleEventLog {
    let abnormality_uuid = Uuid::from_u128(20_005);
    let corrosion_uuid = Uuid::from_u128(21_001);
    BattleEventLog {
        version: crate::game::battle::event_log::BATTLE_EVENT_LOG_VERSION,
        entries: vec![
            BattleEventLogEntry {
                time_ms: 0,
                seq: 1,
                cause: crate::game::battle::event_log::BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::UnitSpawned {
                    unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(
                        Uuid::from_u128(0xC011_EC71),
                    ),
                    owner: Side::Opponent,
                    role: BattleUnitRole::Combatant,
                    threat_class: BattleUnitThreatClass::Elite,
                    mobility_kind: crate::game::battle::types::MobilityKind::Ground,
                    base_uuid: abnormality_uuid,
                    unit_source: BattleUnitSourceIdentity::Abnormality {
                        abnormality_id: "defense_risk_abno".to_string(),
                        base_uuid: abnormality_uuid,
                    },
                    world_position: EventLogVec2::default(),
                    stats: UnitStats::default(),
                },
            },
            BattleEventLogEntry {
                time_ms: 1_000,
                seq: 2,
                cause: crate::game::battle::event_log::BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::UnitSpawned {
                    unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(
                        Uuid::from_u128(0xC011_EC72),
                    ),
                    owner: Side::Opponent,
                    role: BattleUnitRole::Combatant,
                    threat_class: BattleUnitThreatClass::Normal,
                    mobility_kind: crate::game::battle::types::MobilityKind::Ground,
                    base_uuid: corrosion_uuid,
                    unit_source: BattleUnitSourceIdentity::CorrodedEmployee {
                        profile_id: "corroded_guard".to_string(),
                        base_uuid: corrosion_uuid,
                    },
                    world_position: EventLogVec2::default(),
                    stats: UnitStats::default(),
                },
            },
            BattleEventLogEntry {
                time_ms: 2_000,
                seq: 3,
                cause: crate::game::battle::event_log::BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::UnitDied {
                    unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(
                        Uuid::from_u128(0xC011_EC72),
                    ),
                    owner: Side::Opponent,
                    killer_instance_id: None,
                    world_position: EventLogVec2::default(),
                    position: crate::game::resources::Position { x: 0, y: 0 },
                },
            },
        ],
    }
}

fn install_encounter_compendium_combat_result(
    core: &mut GameCore,
    node_id: MapNodeId,
    winner: BattleWinner,
) {
    let event_log = encounter_compendium_spawn_log();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner,
        result_stats: test_result_stats(winner, &event_log, &[]),
        event_log,
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();
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
        .with_employee_definition_data(base.employee_definition_data.clone())
        .with_consumable_data(base.consumable_data.clone())
        .with_equipment_data(base.equipment_data.clone())
        .with_shop_data(base.shop_data.clone())
        .with_reward_data(base.reward_data.clone())
        .with_visual_novel_data(base.visual_novel_data.clone())
        .with_visual_novel_trigger_data(base.visual_novel_trigger_data.clone())
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
    let presentation = |id: &str| crate::game::data::event_data::EventScenePresentation {
        panels: vec![crate::game::data::event_data::EventPresentationPanel {
            image_id: format!("{id}_image"),
            text_ids: vec![format!("{id}_text")],
        }],
    };
    crate::game::data::event_data::EventDefinition {
        id: crate::game::data::event_data::EventId::new(event_id),
        entry_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
        scenes: vec![
            crate::game::data::event_data::EventSceneDefinition {
                id: crate::game::data::event_data::EventSceneId::new("choice"),
                presentation: presentation("choice"),
                next: crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices {
                    choices: vec![crate::game::data::event_data::EventChoiceDefinition {
                        id: crate::game::data::event_data::EventChoiceId::new(choice_id),
                        label_id: "test_choice_label".to_string(),
                        check: crate::game::data::event_data::EventChoiceCheckDefinition {
                            aptitude: crate::game::employee::EventAptitude::Security,
                            dc: 8,
                            required_choice_key: None,
                        },
                        success_action: Some(
                            crate::game::data::event_data::EventSuccessAction::StartCombat {
                                encounter_id: encounter_id.to_string(),
                                primary_abnormality_id: Some(primary_abnormality_id.to_string()),
                            },
                        ),
                        success_scene_id: crate::game::data::event_data::EventSceneId::new(
                            "success",
                        ),
                        failure_scene_id: crate::game::data::event_data::EventSceneId::new(
                            "failure",
                        ),
                    }],
                },
            },
            crate::game::data::event_data::EventSceneDefinition {
                id: crate::game::data::event_data::EventSceneId::new("success"),
                presentation: presentation("success"),
                next: crate::game::data::event_data::EventSceneNext::End,
            },
            crate::game::data::event_data::EventSceneDefinition {
                id: crate::game::data::event_data::EventSceneId::new("failure"),
                presentation: presentation("failure"),
                next: crate::game::data::event_data::EventSceneNext::End,
            },
        ],
    }
}

fn force_single_event_node_run_for_combat_tests(core: &mut GameCore, event_id: &str) -> MapNodeId {
    let generated_node_id = force_first_available_node(
        core,
        MapNodeCategory::Event,
        "event_story",
        MapNodePayload::Event {
            event_id: Some(crate::game::data::event_data::EventId::new(event_id)),
        },
    );
    let stable_node_id = MapNodeId::new(Uuid::from_u128(0xC1_0002));
    let run = core.state.run.as_mut().expect("run");
    run.map.node_mut(generated_node_id).expect("event node").id = stable_node_id;
    run.map_progression.available_node_ids = vec![stable_node_id];
    stable_node_id
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
    let employee_uuid = core
        .state
        .roster
        .iter()
        .find(|employee| employee.is_alive())
        .expect("fixture should have a living employee")
        .uuid;
    let started = core
        .execute(
            player_id,
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id: event_node_id,
                event_id: crate::game::data::event_data::EventId::new(event_id),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
                choice_id: crate::game::data::event_data::EventChoiceId::new(choice_id),
                employee_uuid,
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
        employee_conditions: core.state.roster.clone(),
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
fn combat_trace_export_failure_does_not_block_result_transition() {
    let run_seed = 0xC0BA_77FA_11;
    let battle_uuid = Uuid::from_u128(0xC0BA_77FA_11);
    let mut core = GameCore::new(empty_game_data(), run_seed);
    let trace_path =
        crate::game::world::combat_trace::debug_combat_trace_path(run_seed, battle_uuid);
    let trace_run_dir = trace_path.parent().expect("trace run directory");
    let trace_root = trace_run_dir.parent().expect("trace root directory");
    std::fs::create_dir_all(trace_root).expect("create trace root");
    if trace_run_dir.is_dir() {
        std::fs::remove_dir_all(trace_run_dir).expect("remove stale trace run directory");
    } else if trace_run_dir.exists() {
        std::fs::remove_file(trace_run_dir).expect("remove stale trace blocker");
    }
    std::fs::File::create(trace_run_dir).expect("create trace directory blocker");

    let battle = CombatBattleState {
        primary_abnormality_id: Some("trace_failure_abno".to_string()),
        encounter_id: "trace_failure_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: Uuid::from_u128(0xC0BA_77FA_12),
        winner: BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Player, &BattleEventLog::default(), &[]),
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
    };

    core.transition_finished_live_battle_to_result(battle_uuid, battle)
        .expect("diagnostic failure must not reject the gameplay transition");

    assert_eq!(core.get_state(), GameState::CombatResult { battle_uuid });
    assert!(core.state.active_battle.is_none());
    assert_eq!(
        core.state
            .active_node_content
            .as_ref()
            .expect("active combat result content")
            .as_combat_battle()
            .expect("combat result")
            .encounter_id,
        "trace_failure_encounter"
    );
    assert!(!trace_path.exists());

    std::fs::remove_file(trace_run_dir).expect("cleanup trace directory blocker");
}

#[test]
fn encounter_compendium_profile_merge_is_strict_monotonic_and_catalog_resolved() {
    use std::collections::BTreeSet;

    use crate::game::encounter_compendium::EncounterCompendiumProfileState;

    let game_data = game_data_with_pve_encounters();
    let mut core = GameCore::new(game_data.clone(), 0xC011_EC70);
    core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
        discovered_abnormality_ids: BTreeSet::from(["defense_risk_abno".to_string()]),
        defeated_abnormality_ids: BTreeSet::new(),
        discovered_corrosion_profile_ids: BTreeSet::from(["corroded_guard".to_string()]),
        defeated_corrosion_profile_ids: BTreeSet::new(),
    })
    .unwrap();
    core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
        discovered_abnormality_ids: BTreeSet::from(["low_risk_abno".to_string()]),
        defeated_abnormality_ids: BTreeSet::new(),
        discovered_corrosion_profile_ids: BTreeSet::new(),
        defeated_corrosion_profile_ids: BTreeSet::new(),
    })
    .unwrap();
    let after_union = core.encounter_compendium_profile();
    core.merge_encounter_compendium_profile(after_union.clone())
        .unwrap();
    assert_eq!(core.encounter_compendium_profile(), after_union);

    let snapshot = core.get_run_snapshot_dto().unwrap();
    let authored_abnormality_ids = game_data
        .abnormality_data
        .items
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        snapshot
            .encounter_compendium
            .abnormalities
            .iter()
            .map(|entry| entry.abnormality_id.as_str())
            .collect::<Vec<_>>(),
        authored_abnormality_ids
    );
    assert_eq!(
        snapshot
            .encounter_compendium
            .abnormalities
            .iter()
            .find(|entry| entry.abnormality_id == "defense_risk_abno")
            .and_then(|entry| entry.name.as_deref()),
        Some("defense_risk_abno")
    );
    let undiscovered = snapshot
        .encounter_compendium
        .abnormalities
        .iter()
        .find(|entry| entry.abnormality_id == "boss_risk_abno")
        .unwrap();
    assert!(!undiscovered.discovered);
    assert!(!undiscovered.defeated);
    assert_eq!(undiscovered.name, None);
    assert_eq!(undiscovered.risk_level, None);
    assert_eq!(
        snapshot.encounter_compendium.corrosion_profiles[0].profile_id,
        "corroded_guard"
    );

    core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
        discovered_abnormality_ids: BTreeSet::from(["dormant_abnormality".to_string()]),
        defeated_abnormality_ids: BTreeSet::new(),
        discovered_corrosion_profile_ids: BTreeSet::new(),
        defeated_corrosion_profile_ids: BTreeSet::new(),
    })
    .unwrap();
    assert!(core
        .encounter_compendium_profile()
        .discovered_abnormality_ids
        .contains("dormant_abnormality"));

    let serialized = serde_json::to_string(&core.encounter_compendium_profile()).unwrap();
    let restored_profile = serde_json::from_str(&serialized).unwrap();
    let restored =
        GameCore::new_with_encounter_compendium_profile(game_data, 0xC011_EC71, restored_profile)
            .unwrap();
    assert_eq!(
        restored.encounter_compendium_profile(),
        core.encounter_compendium_profile()
    );
}

#[test]
fn active_run_transaction_rollback_preserves_profile_and_peak_but_restores_floor_anchor() {
    use crate::game::encounter_compendium::EncounterCompendiumProfileState;

    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC72);
    core.state.run_power_authority.power_model_version = 2;
    core.state.run_power_authority.run_peak_certified_power = 100;
    core.state.run_power_authority.current_floor_power_anchor = 100;
    let rollback = core
        .capture_active_run_transaction_rollback()
        .expect("not-started state should support a transaction rollback snapshot");
    core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
        discovered_abnormality_ids: std::collections::BTreeSet::from([
            "dormant_after_capture".to_string()
        ]),
        defeated_abnormality_ids: std::collections::BTreeSet::new(),
        discovered_corrosion_profile_ids: std::collections::BTreeSet::new(),
        defeated_corrosion_profile_ids: std::collections::BTreeSet::new(),
    })
    .unwrap();
    core.state.run_power_authority.power_model_version = 2;
    core.state.run_power_authority.run_peak_certified_power = 900;
    core.state.run_power_authority.current_floor_power_anchor = 800;

    core.restore_active_run_transaction_rollback(rollback);

    assert!(core
        .encounter_compendium_profile()
        .discovered_abnormality_ids
        .contains("dormant_after_capture"));
    assert_eq!(core.state.run_power_authority.power_model_version, 2);
    assert_eq!(core.state.run_power_authority.run_peak_certified_power, 900);
    assert_eq!(
        core.state.run_power_authority.current_floor_power_anchor,
        100
    );
}

#[test]
fn failed_common_transaction_preserves_profile_knowledge_from_before_and_during_operation() {
    use crate::game::encounter_compendium::EncounterCompendiumProfileState;

    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC73);
    core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
        discovered_abnormality_ids: std::collections::BTreeSet::from([
            "dormant_before_common_transaction".to_string(),
        ]),
        ..Default::default()
    })
    .unwrap();

    let error = core
        .with_state_transaction(|core| -> Result<(), crate::game::behavior::GameError> {
            core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
                discovered_abnormality_ids: std::collections::BTreeSet::from([
                    "dormant_during_common_transaction".to_string(),
                ]),
                ..Default::default()
            })?;
            Err(crate::game::behavior::GameError::InvalidAction)
        })
        .unwrap_err();

    assert!(matches!(
        error,
        crate::game::behavior::GameError::InvalidAction
    ));
    assert_eq!(
        core.encounter_compendium_profile()
            .discovered_abnormality_ids,
        std::collections::BTreeSet::from([
            "dormant_before_common_transaction".to_string(),
            "dormant_during_common_transaction".to_string(),
        ])
    );
}

#[test]
fn run_power_authority_initializes_at_floor_manifestation_and_peaks_on_actual_combat_entry() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC73);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let initial_peak = core.state.run_power_authority.run_peak_certified_power;
    let initial_anchor = core.state.run_power_authority.current_floor_power_anchor;
    assert_eq!(
        core.state.run_power_authority.power_model_version,
        core.game_data
            .run_policy
            .combat_difficulty
            .power_model_version
    );
    assert!(initial_peak > 0);
    assert!(initial_anchor >= initial_peak);

    let employee_id = core.roster().unwrap().iter().next().unwrap().uuid;
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_id)
        .unwrap()
        .combat_profile
        .battle_profile
        .stats
        .attack += 1_000;
    let stronger = crate::game::combat_power::certified_combat_power_for_roster(
        &core.state.roster,
        &core.state.inventory,
        &core.state.skill_fragments,
        core.game_data.as_ref(),
    )
    .unwrap()
    .total;
    assert!(stronger > initial_peak);

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

    assert_eq!(
        core.state.run_power_authority.run_peak_certified_power,
        stronger
    );
    assert_eq!(
        core.state.run_power_authority.current_floor_power_anchor, initial_anchor,
        "same-Floor combat entry may raise the run peak but must not rescale the manifested Floor"
    );

    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle).unwrap();
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_id)
        .unwrap()
        .combat_profile
        .battle_profile
        .stats
        .attack = 1;
    let weaker = crate::game::combat_power::certified_combat_power_for_roster(
        &core.state.roster,
        &core.state.inventory,
        &core.state.skill_fragments,
        core.game_data.as_ref(),
    )
    .unwrap()
    .total;
    assert!(weaker < stronger);
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert_eq!(
        core.state.run_power_authority.run_peak_certified_power, stronger,
        "a weaker later entry must not lower the run peak"
    );
}

#[test]
fn certified_power_tracks_level_equipment_enhancement_and_fragment_loadout() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xC011_EC74);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_id = core.roster().unwrap().iter().next().unwrap().uuid;
    let certify = |core: &GameCore| {
        crate::game::combat_power::certified_combat_power_for_roster(
            &core.state.roster,
            &core.state.inventory,
            &core.state.skill_fragments,
            core.game_data.as_ref(),
        )
        .unwrap()
        .total
    };
    let baseline = certify(&core);

    core.roster_mut()
        .unwrap()
        .get_mut(&employee_id)
        .unwrap()
        .level += 1;
    assert!(
        certify(&core) > baseline,
        "employee level must affect power"
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_id)
        .unwrap()
        .level -= 1;

    let original_item_slot = core
        .roster()
        .unwrap()
        .get(&employee_id)
        .unwrap()
        .loadout
        .item_slot
        .clone();
    let equipped_instance_id = original_item_slot.iter().next().unwrap().instance_uuid;
    core.state
        .inventory
        .equipments
        .get_item_mut(&equipped_instance_id)
        .unwrap()
        .enhancement_level = 1;
    assert!(
        certify(&core) > baseline,
        "equipped item enhancement must affect power"
    );
    core.state
        .inventory
        .equipments
        .get_item_mut(&equipped_instance_id)
        .unwrap()
        .enhancement_level = 0;

    core.roster_mut()
        .unwrap()
        .get_mut(&employee_id)
        .unwrap()
        .loadout
        .item_slot = crate::game::resources::item_slot::ItemSlot::default();
    assert!(
        certify(&core) < baseline,
        "removing equipped permanent stats must lower power"
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_id)
        .unwrap()
        .loadout
        .item_slot = original_item_slot;

    let original_fragments = core
        .roster()
        .unwrap()
        .get(&employee_id)
        .unwrap()
        .skill_fragments
        .clone();
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_id)
        .unwrap()
        .skill_fragments = crate::game::skill_fragment::SkillFragmentLoadout::default();
    assert!(
        certify(&core) < baseline,
        "removing an equipped persistent fragment must lower power"
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_id)
        .unwrap()
        .skill_fragments = original_fragments;
    assert_eq!(certify(&core), baseline);
}

#[test]
fn consuming_victory_commits_spawned_compendium_union_across_checkpoint_and_new_run() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC72);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    core.save_run_checkpoint().unwrap();
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
    core.state
        .run
        .as_mut()
        .unwrap()
        .combat_previews
        .get_mut(&node_id)
        .unwrap()
        .spawn_waves
        .push(crate::game::combat_preview::SpawnWave {
            id: "preview_only_unspawned_wave".to_string(),
            time_ms: 99_999,
            spawn_zone_ids: vec![],
            route_id: None,
            enemy_entries: vec![crate::game::combat_preview::SpawnWaveEnemyEntry {
                kind: crate::game::combat_preview::EnemyKind::Abnormality,
                profile_id: None,
                abnormality_id: "low_risk_abno".to_string(),
                tier: crate::game::enums::Tier::I,
                count: 1,
                appearance_seeds: vec![],
            }],
            required_for_victory: false,
        });
    install_encounter_compendium_combat_result(&mut core, node_id, BattleWinner::Player);

    core.execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    let committed = core.encounter_compendium_profile();
    assert_eq!(
        committed.discovered_abnormality_ids,
        std::collections::BTreeSet::from(["defense_risk_abno".to_string()])
    );
    assert_eq!(
        committed.discovered_corrosion_profile_ids,
        std::collections::BTreeSet::from(["corroded_guard".to_string()])
    );
    assert!(!committed
        .discovered_abnormality_ids
        .contains("low_risk_abno"));
    assert_eq!(
        committed.defeated_abnormality_ids,
        std::collections::BTreeSet::from(["defense_risk_abno".to_string()])
    );
    assert_eq!(
        committed.defeated_corrosion_profile_ids,
        std::collections::BTreeSet::from(["corroded_guard".to_string()])
    );

    core.handle_load_run_checkpoint().unwrap();
    assert_eq!(core.encounter_compendium_profile(), committed);

    core.transition_to(GameState::NotStarted).unwrap();
    core.execute(
        player_id,
        PlayerBehavior::StartNewGame {
            game_mode: GameMode::Standard,
            replace_active_run: false,
        },
    )
    .unwrap();
    assert_eq!(core.encounter_compendium_profile(), committed);
}

#[test]
fn retryable_defeat_and_retreat_do_not_commit_but_exhausted_defeat_does() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC73);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let fragment_id = starter_basic_attack_fragment_id();
    let starting_fragment_count = core.state.skill_fragments.count(&fragment_id);
    core.state
        .skill_fragments
        .add_research_progress(&fragment_id, 100)
        .expect("research completion should queue delivery");
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
    install_encounter_compendium_combat_result(&mut core, node_id, BattleWinner::Opponent);

    let retry = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert!(matches!(retry, BehaviorResult::NodePreview { .. }));
    assert_eq!(
        core.encounter_compendium_profile(),
        crate::game::encounter_compendium::EncounterCompendiumProfileState::default()
    );
    assert_eq!(
        core.state.skill_fragments.count(&fragment_id),
        starting_fragment_count,
        "retryable defeat must not deliver research"
    );

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.merge_encounter_compendium_profile(
        crate::game::encounter_compendium::EncounterCompendiumProfileState {
            discovered_abnormality_ids: std::collections::BTreeSet::from([
                "low_risk_abno".to_string()
            ]),
            defeated_abnormality_ids: std::collections::BTreeSet::new(),
            discovered_corrosion_profile_ids: std::collections::BTreeSet::new(),
            defeated_corrosion_profile_ids: std::collections::BTreeSet::new(),
        },
    )
    .unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(
        core.encounter_compendium_profile()
            .discovered_abnormality_ids,
        std::collections::BTreeSet::from(["low_risk_abno".to_string()]),
        "battle-entry rollback must not replace newer game-global profile knowledge"
    );
    assert_eq!(
        core.state.skill_fragments.count(&fragment_id),
        starting_fragment_count,
        "retreat must not deliver research"
    );

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.state
        .run
        .as_mut()
        .unwrap()
        .abnormality_attempts
        .get_mut(&node_id)
        .unwrap()
        .attempts_started = crate::game::world::state::ABNORMALITY_MAX_ATTEMPTS;
    install_encounter_compendium_combat_result(&mut core, node_id, BattleWinner::Opponent);
    let completed = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let BehaviorResult::NodeCompleted {
        research_deliveries,
        ..
    } = completed
    else {
        panic!("exhausted defeat should consume the node and return to planning");
    };
    assert_eq!(research_deliveries.len(), 1);
    assert_eq!(research_deliveries[0].fragment_id, fragment_id);
    assert_eq!(
        core.state.skill_fragments.count(&fragment_id),
        starting_fragment_count + 1
    );

    assert!(core
        .encounter_compendium_profile()
        .discovered_abnormality_ids
        .contains("defense_risk_abno"));
    assert!(core
        .encounter_compendium_profile()
        .discovered_corrosion_profile_ids
        .contains("corroded_guard"));
    assert!(core
        .encounter_compendium_profile()
        .defeated_abnormality_ids
        .is_empty());
    assert!(core
        .encounter_compendium_profile()
        .defeated_corrosion_profile_ids
        .is_empty());
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
        employee_conditions: core.state.roster.clone(),
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

    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
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
        employee_conditions: core.state.roster.clone(),
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
    assert_eq!(
        action_capability(&core, ActionKind::CompleteCombatResult)
            .expect("combat completion remains relevant")
            .disabled_reason_code,
        Some(ActionDisabledReasonCode::CompletionUnavailable)
    );

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
fn combat_result_commits_attempt_conditions_and_survival_xp_once_per_employee() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 125);
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

    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let experience_before = core.state.roster.get(&employee_uuid).unwrap().experience;
    let mut employee_conditions = core.state.roster.clone();
    for _ in 0..2 {
        employee_conditions
            .get_mut(&employee_uuid)
            .unwrap()
            .apply_incapacitation(run_policy().post_battle.incapacitation_trauma);
    }
    let first_unit = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC0_1001));
    let second_unit = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC0_1002));
    let spawn = |seq, unit_instance_id| BattleEventLogEntry {
        seq,
        time_ms: seq * 100,
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
    };
    let event_log = BattleEventLog {
        entries: vec![spawn(1, first_unit), spawn(2, second_unit)],
        ..BattleEventLog::default()
    };
    let participant_results = vec![
        ParticipantBattleResult {
            unit_instance_id: second_unit,
            owned_uuid: employee_uuid,
            side: Side::Player,
            survived: true,
            final_hp: 15,
            max_hp: 30,
            became_incapacitated: false,
        },
        ParticipantBattleResult {
            unit_instance_id: first_unit,
            owned_uuid: employee_uuid,
            side: Side::Player,
            survived: false,
            final_hp: 0,
            max_hp: 30,
            became_incapacitated: true,
        },
    ];
    let result_stats = test_result_stats(BattleWinner::Player, &event_log, &participant_results);
    assert_eq!(result_stats.employees.len(), 1);
    assert!(result_stats.employees[0].survived);
    assert!(result_stats.employees[0].became_incapacitated);
    let battle = CombatBattleState {
        primary_abnormality_id: Some("low_risk_abno".to_string()),
        encounter_id: "low_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Player,
        event_log,
        result_stats,
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results,
        employee_conditions,
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();

    core.execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    let employee = core.state.roster.get(&employee_uuid).unwrap();
    assert_eq!(
        employee.condition.trauma,
        run_policy()
            .post_battle
            .incapacitation_trauma
            .saturating_mul(2)
    );
    assert_eq!(employee.condition.physical_damage, 2);
    assert_eq!(
        employee.experience,
        experience_before + run_policy().growth.post_battle_survival_xp,
        "employee-level survival XP must be granted once across redeploy instances"
    );
}

#[test]
fn combat_defeat_commits_attempt_incapacitation_without_reapplying_it() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 126);
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

    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let mut employee_conditions = core.state.roster.clone();
    employee_conditions
        .get_mut(&employee_uuid)
        .unwrap()
        .apply_incapacitation(run_policy().post_battle.incapacitation_trauma);
    let unit_instance_id =
        crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xC0_2001));
    let event_log = BattleEventLog {
        entries: vec![BattleEventLogEntry {
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
        }],
        ..BattleEventLog::default()
    };
    let participant_results = vec![ParticipantBattleResult {
        unit_instance_id,
        owned_uuid: employee_uuid,
        side: Side::Player,
        survived: false,
        final_hp: 0,
        max_hp: 30,
        became_incapacitated: true,
    }];
    let result_stats = test_result_stats(BattleWinner::Opponent, &event_log, &participant_results);
    let battle = CombatBattleState {
        primary_abnormality_id: Some("low_risk_abno".to_string()),
        encounter_id: "low_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        event_log,
        result_stats,
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results,
        employee_conditions,
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
    let employee = core.state.roster.get(&employee_uuid).unwrap();
    assert_eq!(
        employee.condition.trauma,
        run_policy().post_battle.incapacitation_trauma
    );
    assert_eq!(employee.condition.physical_damage, 1);
}

#[test]
fn endless_final_response_completion_returns_run_complete_without_planning_delivery() {
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
    let victory_research_gain = run_policy().abnormality_research.victory_research_gain;
    let research = &mut core.state.run.as_mut().unwrap().abnormality_research;
    for (abnormality_id, entry) in &mut research.entries {
        if abnormality_id == "low_risk_abno" {
            entry.research_points = entry
                .research_required
                .saturating_sub(victory_research_gain);
            entry.response_complete = false;
            entry.completed_at_floor = None;
            entry.unique_fragment_granted = false;
        } else {
            entry.research_points = entry.research_required;
            entry.response_complete = true;
            entry.completed_at_floor = Some(0);
            entry.unique_fragment_granted = true;
        }
    }
    assert!(!research.all_response_complete());

    let pending_fragment_id = starter_basic_attack_fragment_id();
    let starting_fragment_count = core.state.skill_fragments.count(&pending_fragment_id);
    core.state
        .skill_fragments
        .add_research_progress(&pending_fragment_id, 100)
        .expect("final response setup should queue a planning-return delivery");

    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
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
        employee_conditions: core.state.roster.clone(),
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
        completion,
        ..
    } = result
    else {
        panic!("expected combat rewards granted");
    };
    let BehaviorResult::RunComplete { map } = *completion else {
        panic!("final Endless response must immediately complete the run");
    };
    assert!(!map.nodes.is_empty());
    assert!(matches!(core.get_state(), GameState::RunComplete));
    assert!(skill_fragment_diffs.iter().any(|diff| {
        diff.fragment_id == pending_fragment_id
            && diff.count_before == starting_fragment_count
            && diff.count_after == starting_fragment_count + 1
    }));
    let research = &core.state.run.as_ref().unwrap().abnormality_research;
    let entry = research.entries.get("low_risk_abno").unwrap();
    assert_eq!(entry.research_points, 100);
    assert!(entry.response_complete);
    assert!(entry.unique_fragment_granted);
    assert!(research.all_response_complete());
    assert_eq!(
        core.state.skill_fragments.count(&pending_fragment_id),
        starting_fragment_count + 1,
        "RunComplete must not deliver the queued Safe Zone research copy"
    );
    assert_eq!(
        core.state
            .skill_fragments
            .pending_research_deliveries()
            .collect::<Vec<_>>(),
        vec![(&pending_fragment_id, 1)]
    );
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

    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
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
        employee_conditions: core.state.roster.clone(),
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
fn combat_map_node_fails_run_when_no_employee_can_be_deployed() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let roster = core.roster_mut().unwrap();
        for employee in roster.iter_mut() {
            employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE - 1;
        }
    }
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        MapNodePayload::Encounter {
            encounter_id: Some("low_risk_encounter".to_string()),
        },
    );
    normalize_other_available_save_points(&mut core, node_id);

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
    assert!(core.get_action_capabilities().is_empty());
}

#[test]
fn combat_map_node_fails_run_when_all_living_employees_are_critical() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let roster = core.roster_mut().unwrap();
        for employee in roster.iter_mut() {
            employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE - 1;
        }
    }
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        MapNodePayload::Encounter {
            encounter_id: Some("low_risk_encounter".to_string()),
        },
    );
    normalize_other_available_save_points(&mut core, node_id);

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
fn checkpoint_availability_defers_no_deployable_failure_for_explicit_decision() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let save_point_node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    select_and_confirm_map_node(&mut core, player_id, save_point_node_id);
    core.execute(player_id, PlayerBehavior::CompleteNode)
        .expect("SavePoint should complete and save checkpoint");
    assert!(core.state.run_checkpoint.can_load());

    {
        let roster = core.roster_mut().unwrap();
        for employee in roster.iter_mut() {
            employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE - 1;
        }
    }
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

    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(matches!(
        result,
        BehaviorResult::RunFailureDecisionRequired {
            reason: RunFailureReason::NoDeployableEmployees,
            ..
        }
    ));
    assert!(matches!(
        core.get_state(),
        GameState::AwaitingRunFailureDecision {
            reason: RunFailureReason::NoDeployableEmployees,
            ..
        }
    ));
    let capabilities = core.get_action_capabilities();
    for action in [ActionKind::LoadRunCheckpoint, ActionKind::ConfirmRunFailure] {
        assert!(capabilities
            .iter()
            .any(|capability| capability.action == action && capability.enabled));
    }
    assert!(!capabilities
        .iter()
        .any(|capability| capability.action == ActionKind::AbandonRun));

    core.state
        .run_checkpoint
        .payload
        .as_mut()
        .unwrap()
        .map_progression
        .current_node_id = Some(MapNodeId::new(Uuid::from_u128(u128::MAX)));
    let loads_before = core.state.run_checkpoint.loads_used;
    let load = action_capability(&core, ActionKind::LoadRunCheckpoint).unwrap();
    assert!(!load.enabled);
    assert_eq!(
        load.disabled_reason_code,
        Some(ActionDisabledReasonCode::CheckpointUnavailable)
    );
    assert!(
        action_capability(&core, ActionKind::ConfirmRunFailure)
            .unwrap()
            .enabled
    );
    assert!(core
        .execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .is_err());
    assert_eq!(core.state.run_checkpoint.loads_used, loads_before);
    assert!(matches!(
        core.get_state(),
        GameState::AwaitingRunFailureDecision { .. }
    ));
    assert!(matches!(
        core.execute(player_id, PlayerBehavior::ConfirmRunFailure)
            .unwrap(),
        BehaviorResult::RunFailed {
            reason: RunFailureReason::NoDeployableEmployees,
            ..
        }
    ));
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];

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
    let deploy_result = execute_current_battle_mutation_with_source(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
        deploy_source_command_id,
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
    assert!(
        player_unit.range_previews.is_some(),
        "player checkpoint actors keep the authoritative player preview"
    );

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
    assert!(
        abnormality_enemy.range_previews.is_none(),
        "opponent checkpoint actors must not project enemy range preview"
    );
    let checkpoint_json = serde_json::to_value(&spawn_update.checkpoint).unwrap();
    let checkpoint_units = checkpoint_json["units"].as_array().unwrap();
    assert!(checkpoint_units
        .iter()
        .any(|unit| { unit["owner"] == "Player" && unit.get("range_previews").is_some() }));
    assert!(checkpoint_units
        .iter()
        .filter(|unit| unit["owner"] == "Opponent")
        .all(|unit| unit.get("range_previews").is_none()));

    let mut finished = false;
    for _ in 0..10_000 {
        let result = core.advance_active_battle_to_next_event_bucket().unwrap();
        if let BehaviorResult::BattleAdvanced { finished: true, .. } = result {
            finished = true;
            break;
        }
    }
    assert!(finished, "defense live battle should eventually finish");

    let battle_uuid = match core.get_state() {
        GameState::CombatResult { battle_uuid } => battle_uuid,
        state => panic!("expected combat result, got {state:?}"),
    };

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
    let trace_path =
        crate::game::world::combat_trace::debug_combat_trace_path(core.run_seed, battle_uuid);
    assert!(trace_path.exists());
    std::fs::remove_file(&trace_path).expect("cleanup smoke combat trace file");
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
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
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
        let result = execute_current_battle_mutation(
            &mut core,
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
                    let result = execute_current_battle_mutation(
                        &mut core,
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

    let battle_uuid = match core.get_state() {
        GameState::CombatResult { battle_uuid } => battle_uuid,
        state => panic!("expected combat result, got {state:?}"),
    };
    let (combat_node_uuid, event_log_entry_count) = {
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
    let trace_path =
        crate::game::world::combat_trace::debug_combat_trace_path(core.run_seed, battle_uuid);
    assert!(trace_path.exists(), "debug combat trace file should exist");
    let trace_json: serde_json::Value = serde_json::from_reader(
        std::fs::File::open(&trace_path).expect("open debug combat trace file"),
    )
    .expect("debug combat trace should be valid json");
    assert_eq!(trace_json["battle_uuid"], json!(battle_uuid));
    assert_eq!(trace_json["combat_node_uuid"], json!(combat_node_uuid));
    assert_eq!(
        trace_json["event_log"]["entries"]
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
    assert!(trace_path.exists());
    std::fs::remove_file(&trace_path).expect("cleanup debug combat trace file");
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
    assert_eq!(first_update.battle_state_revision, 1);
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
    assert_eq!(
        setup.catalog_refs.abnormality_ids,
        vec!["f-05-32_warm_hearted_woodsman".to_string()],
        "actual abnormality setup refs must be exact, non-empty and catalog-resolved"
    );
    assert!(first_update
        .events_delta
        .events
        .iter()
        .any(|entry| matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::BattleStart { .. }
        )));
    assert!(core.get_action_capabilities().is_empty());
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["skill_catalog"]["version"], 2);
    assert_eq!(snapshot["game_state_context"]["type"], "in_battle");
    assert_eq!(snapshot["game_state_context"]["battle_state_revision"], 1);
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
    assert_eq!(battle_update.battle_state_revision, 1);
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
        .battle_update_dto_after(0, &core.state.roster)
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
    assert_eq!(battle_update.battle_state_revision, 1);
    assert!(battle_update.events_delta.events.is_empty());
    let update = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_update_dto_after(last_seq, &core.state.roster)
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
    assert_eq!(
        setup.catalog_refs.abnormality_ids,
        vec!["d-01-110_clouded_monk".to_string()],
        "mixed encounters must preload only the actual abnormality catalog entry"
    );
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
fn battle_setup_snapshot_corroded_only_catalog_refs_exclude_empty_abnormality_id() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xCA7A_10A0);
    let player_id = Uuid::from_u128(0xCA7A_10A0);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "normal_corroded_patrol",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_setup_snapshot: Some(setup),
        finished: false,
        ..
    } = result
    else {
        panic!("expected corroded-only live battle start with setup snapshot");
    };

    assert!(
        setup.catalog_refs.abnormality_ids.is_empty(),
        "corroded-employee-only encounters must not preload an abnormality catalog entry"
    );
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
fn live_defense_setup_loss_recovery_preserves_battle_revision_and_state() {
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
    assert_eq!(first_update.battle_state_revision, 1);
    assert!(matches!(core.get_state(), GameState::InBattle { .. }));
    assert!(core.state.active_battle.is_some());

    let paused =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    let BehaviorResult::BattlePlaybackChanged {
        battle_update: paused_update,
        ..
    } = paused
    else {
        panic!("expected playback update before setup recovery");
    };
    assert_eq!(paused_update.battle_state_revision, 2);

    let recovered = core
        .execute(player_id, PlayerBehavior::RecoverBattleSetupLoss)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: recovered_battle_uuid,
        battle_setup_snapshot: Some(setup),
        battle_update: recovered_update,
        ..
    } = recovered
    else {
        panic!("expected immutable setup plus current battle update");
    };
    assert_eq!(recovered_battle_uuid, first_battle_uuid);
    assert_eq!(setup.battle_uuid, first_battle_uuid);
    assert_eq!(recovered_update.battle_uuid, first_battle_uuid);
    assert_eq!(recovered_update.battle_state_revision, 2);
    assert_eq!(recovered_update.events_delta.after_seq, 0);
    assert_eq!(
        recovered_update
            .events_delta
            .events
            .first()
            .map(|entry| entry.seq),
        Some(1)
    );
    assert_eq!(
        recovered_update.checkpoint.at_seq,
        recovered_update.events_delta.to_seq
    );
    assert!(recovered_update.checkpoint.playback.paused);
    assert!(
        matches!(core.get_state(), GameState::InBattle { battle_uuid } if battle_uuid == first_battle_uuid)
    );
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.node_id, node_id);
    assert_eq!(active.battle_uuid, first_battle_uuid);
    assert_eq!(active.battle_state_revision, 2);
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
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
    assert!(battle_update
        .action_capabilities
        .iter()
        .any(|capability| capability.action == ActionKind::PauseBattle && capability.enabled));
    assert!(!battle_update
        .action_capabilities
        .iter()
        .any(|capability| capability.action == ActionKind::ResumeBattle));
    assert!(battle_update.action_capabilities.iter().any(|capability| {
        capability.action == ActionKind::RequestBattleState
            && capability.scope == crate::game::behavior::ActionScope::Query
            && capability.enabled
    }));
    assert!(battle_update.action_capabilities.iter().any(|capability| {
        capability.action == ActionKind::RecoverBattleSetupLoss
            && capability.scope == crate::game::behavior::ActionScope::Recovery
            && capability.enabled
    }));
    let deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let BehaviorResult::BattleUnitDeployed {
        battle_update: deployed_update,
        ..
    } = deployed
    else {
        panic!("expected deployed update");
    };
    assert_eq!(deployed_update.battle_state_revision, 2);

    let paused =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    let BehaviorResult::BattlePlaybackChanged { battle_update, .. } = paused else {
        panic!("expected playback changed");
    };
    let playback = battle_update.checkpoint.playback;
    let paused_time_ms = battle_update.checkpoint.battle_time_ms;
    let paused_seq = battle_update.events_delta.to_seq;
    assert!(playback.paused);
    assert_eq!(paused_time_ms, 0);
    assert_eq!(battle_update.battle_state_revision, 3);
    assert!(battle_update
        .action_capabilities
        .iter()
        .any(|capability| capability.action == ActionKind::ResumeBattle && capability.enabled));
    assert!(!battle_update
        .action_capabilities
        .iter()
        .any(|capability| capability.action == ActionKind::PauseBattle));

    let repeated_pause =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle)
            .unwrap_err();
    assert!(matches!(
        repeated_pause,
        GameError::ActionUnavailable {
            action: ActionKind::PauseBattle,
            reason: crate::game::behavior::ActionDisabledReasonCode::AlreadyActive,
        }
    ));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.battle_state_revision, 3);
    assert_eq!(active.latest_event_log_seq(), paused_seq);
    assert_eq!(active.execution.last_event_time_ms(), paused_time_ms);
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
    assert_eq!(battle_update.battle_state_revision, 3);

    let resumed =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle)
            .unwrap();
    let BehaviorResult::BattlePlaybackChanged { battle_update, .. } = resumed else {
        panic!("expected playback changed");
    };
    let playback = battle_update.checkpoint.playback;
    assert!(!playback.paused);
    assert_eq!(battle_update.battle_state_revision, 4);

    let repeated_resume =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle)
            .unwrap_err();
    assert!(matches!(
        repeated_resume,
        GameError::ActionUnavailable {
            action: ActionKind::ResumeBattle,
            reason: crate::game::behavior::ActionDisabledReasonCode::AlreadyActive,
        }
    ));
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle_state_revision,
        4
    );

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
    assert_eq!(battle_update.battle_state_revision, 5);
}

#[test]
fn supplied_stale_battle_authority_rejects_before_playback_mutation() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0102);
    let player_id = Uuid::from_u128(0xD3F3_0102);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_burrowing_heaven",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let active = core.state.active_battle.as_ref().unwrap();
    let current = crate::game::behavior::BattleStateAuthorityDto {
        battle_uuid: active.battle_uuid,
        battle_state_revision: active.battle_state_revision,
    };
    let stale = crate::game::behavior::BattleStateAuthorityDto {
        battle_uuid: current.battle_uuid,
        battle_state_revision: current.battle_state_revision - 1,
    };

    let missing = core
        .execute(player_id, PlayerBehavior::PauseBattle)
        .unwrap_err();
    assert!(matches!(
        missing,
        GameError::MissingBattleStateAuthority { current: actual } if actual == current
    ));
    assert!(!core.state.active_battle.as_ref().unwrap().playback.paused);

    let error = core
        .execute_with_transport_context(
            player_id,
            PlayerBehavior::PauseBattle,
            Some("stale-pause"),
            0,
            Some(stale),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        GameError::StaleBattleStateAuthority {
            expected,
            current: actual,
        } if expected == stale && actual == current
    ));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.battle_state_revision, current.battle_state_revision);
    assert!(!active.playback.paused);

    let result = core
        .execute_with_transport_context(
            player_id,
            PlayerBehavior::PauseBattle,
            Some("current-pause"),
            0,
            Some(current),
        )
        .unwrap();
    let BehaviorResult::BattlePlaybackChanged { battle_update, .. } = result else {
        panic!("expected playback update");
    };
    assert_eq!(
        battle_update.battle_state_revision,
        current.battle_state_revision + 1
    );
    assert!(battle_update.checkpoint.playback.paused);
}

#[test]
fn battle_revision_overflow_rejects_mutation_without_changing_playback() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_01FF);
    let player_id = Uuid::from_u128(0xD3F3_01FF);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_burrowing_heaven",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let active = core.state.active_battle.as_mut().unwrap();
    active.battle_state_revision = u64::MAX;
    assert!(!active.playback.paused);

    let error = execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle)
        .expect_err("overflow must reject before playback mutation");

    assert!(matches!(error, GameError::BattleStateRevisionOverflow));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.battle_state_revision, u64::MAX);
    assert!(!active.playback.paused);
}

fn visual_novel_session_for_battle(
    session_id: &str,
    battle_uuid: Uuid,
    current_scene_id: &str,
    blocking: bool,
) -> crate::game::visual_novel::VisualNovelSession {
    crate::game::visual_novel::VisualNovelSession {
        session_id: session_id.to_string(),
        source: crate::game::visual_novel::VisualNovelSource::Battle { battle_uuid },
        vn_id: crate::game::data::visual_novel_data::VisualNovelId::new(
            "white_night_confession_intro",
        ),
        current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
            current_scene_id,
        ),
        committed_choice_id: None,
        blocking,
        presentation_mode: crate::game::visual_novel::VisualNovelSessionPresentationMode::Cinematic,
    }
}

fn start_live_defense_battle_for_vn_test(
    seed: u64,
) -> (GameCore, Uuid, Uuid, crate::game::map::MapNodeId) {
    let mut core = GameCore::new(live_game_data_from_ron(), seed);
    let player_id = Uuid::from_u128(seed as u128);
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
    let BehaviorResult::BattleAdvanced { battle_uuid, .. } = result else {
        panic!("expected live battle start");
    };
    (core, player_id, battle_uuid, node_id)
}

#[test]
fn live_battle_update_projects_active_visual_novel_overlay() {
    let (mut core, player_id, battle_uuid, _) = start_live_defense_battle_for_vn_test(0xD3F3_0103);
    core.state.visual_novel.active = Some(visual_novel_session_for_battle(
        "battle_vn_projection",
        battle_uuid,
        "white_night_speaks",
        false,
    ));

    let state = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState { since_seq: None },
        )
        .unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = state else {
        panic!("expected battle state");
    };
    let visual_novel = battle_update
        .visual_novel
        .expect("battle_update should include active VN projection");

    assert_eq!(
        visual_novel.source,
        crate::game::behavior::VisualNovelSourceDto::Battle { battle_uuid }
    );
    assert_eq!(visual_novel.session_id, "battle_vn_projection");
    assert_eq!(visual_novel.current_scene_id, "white_night_speaks");
    assert_eq!(visual_novel.script_id, "white_night_confession_intro");
    assert_eq!(visual_novel.speakers[0].speaker_id, "white_night");
    assert_eq!(
        visual_novel.speakers[0].position,
        crate::game::behavior::VisualNovelSpeakerPositionDto::Center
    );
}

#[test]
fn battle_started_trigger_is_attached_to_first_battle_update() {
    let base = game_data_with_pve_encounters();
    let mut triggers = base.visual_novel_trigger_data.triggers.clone();
    triggers.push(VisualNovelTrigger {
        id: VisualNovelTriggerId::new("low_risk_battle_started_vn"),
        source: VisualNovelTriggerSource::BattleStarted {
            encounter_id: "low_risk_encounter".to_string(),
            node_type: crate::game::combat_preview::CombatNodeType::Defense,
            mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        },
        vn_id: VisualNovelId::new("hq_briefing_standard_intro"),
        repeat: VisualNovelTriggerRepeat::OncePerBattle,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        priority: 50,
        blocking: true,
    });
    let game_data =
        game_data_with_visual_novel_triggers(base, VisualNovelTriggerDatabase::new(triggers));
    let mut core = GameCore::new(game_data, 0xD3F3_0104);
    let player_id = Uuid::from_u128(0xD3F3_0104);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "low_risk_encounter",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid,
        battle_update,
        ..
    } = result
    else {
        panic!("expected live battle start");
    };
    let visual_novel = battle_update
        .visual_novel
        .expect("battle-start trigger should attach VN to first battle update");

    assert_eq!(
        visual_novel.source,
        crate::game::behavior::VisualNovelSourceDto::Battle { battle_uuid }
    );
    assert_eq!(visual_novel.vn_id, "hq_briefing_standard_intro");
    assert_eq!(visual_novel.current_scene_id, "start");
    assert!(visual_novel.blocking);
}

#[test]
fn battle_time_trigger_fires_once_when_threshold_is_crossed() {
    let base = game_data_with_pve_encounters();
    let mut triggers = base.visual_novel_trigger_data.triggers.clone();
    triggers.push(VisualNovelTrigger {
        id: VisualNovelTriggerId::new("low_risk_battle_time_vn"),
        source: VisualNovelTriggerSource::BattleTimeReached {
            battle_time_ms: 5_000,
        },
        vn_id: VisualNovelId::new("hq_briefing_standard_intro"),
        repeat: VisualNovelTriggerRepeat::OncePerBattle,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        priority: 50,
        blocking: true,
    });
    let game_data =
        game_data_with_visual_novel_triggers(base, VisualNovelTriggerDatabase::new(triggers));
    let mut core = GameCore::new(game_data, 0xD3F3_0105);
    let player_id = Uuid::from_u128(0xD3F3_0105);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "low_risk_encounter",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let started = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_update: first_update,
        ..
    } = started
    else {
        panic!("expected live battle start");
    };
    assert!(first_update.visual_novel.is_none());

    let triggered = core
        .advance_active_battle_for_server_tick(5_000)
        .unwrap()
        .expect("threshold crossing should produce a battle update");
    let BehaviorResult::BattleAdvanced { battle_update, .. } = triggered else {
        panic!("expected battle update");
    };
    let visual_novel = battle_update
        .visual_novel
        .expect("threshold trigger should attach VN to battle update");
    assert_eq!(visual_novel.vn_id, "hq_briefing_standard_intro");
    assert_eq!(visual_novel.current_scene_id, "start");

    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: visual_novel.session_id,
            current_scene_id: VisualNovelSceneId::new("start"),
        },
    )
    .unwrap();
    let next_update = core.advance_active_battle_for_server_tick(1_000).unwrap();

    assert!(core.state.visual_novel.active.is_none());
    assert!(core.state.visual_novel.queue.is_empty());
    if let Some(BehaviorResult::BattleAdvanced { battle_update, .. }) = next_update {
        assert!(
            battle_update.visual_novel.is_none(),
            "timed trigger should not refire after threshold crossing"
        );
    }
}

#[test]
fn blocking_visual_novel_pauses_live_battle_tick_and_uses_battle_update_transport() {
    let (mut core, player_id, battle_uuid, _) = start_live_defense_battle_for_vn_test(0xD3F3_0104);
    core.state.visual_novel.active = Some(visual_novel_session_for_battle(
        "battle_vn_blocking",
        battle_uuid,
        "start",
        true,
    ));
    let initial_time_ms = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .execution
        .last_event_time_ms();
    let initial_revision = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_state_revision;
    let initial_seq = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .latest_event_log_seq();

    assert!(core.get_action_capabilities().is_empty());
    let reconnect = core.active_battle_reconnect_projection(0).unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_update: blocking_update,
        ..
    } = reconnect
    else {
        panic!("expected blocking battle projection");
    };
    let projected_actions = blocking_update
        .action_capabilities
        .iter()
        .map(|capability| capability.action)
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        projected_actions,
        std::collections::HashSet::from([
            ActionKind::RequestBattleState,
            ActionKind::RecoverBattleSetupLoss,
            ActionKind::AdvanceVisualNovel,
        ])
    );
    assert!(core
        .advance_active_battle_for_server_tick(1_000)
        .unwrap()
        .is_none());
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .execution
            .last_event_time_ms(),
        initial_time_ms
    );

    let advanced = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: "battle_vn_blocking".to_string(),
            current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                "start",
            ),
        },
    )
    .unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = advanced else {
        panic!("battle VN command should use battle_update transport");
    };
    assert_eq!(battle_update.battle_state_revision, initial_revision + 1);
    assert_eq!(battle_update.events_delta.to_seq, initial_seq);
    assert_eq!(battle_update.server_battle_time_ms, initial_time_ms);
    assert_eq!(
        battle_update
            .visual_novel
            .as_ref()
            .expect("advanced VN should remain active")
            .current_scene_id,
        "white_night_speaks"
    );

    let ended = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: "battle_vn_blocking".to_string(),
            current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                "white_night_speaks",
            ),
        },
    )
    .unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = ended else {
        panic!("battle VN close should use battle_update transport");
    };
    assert_eq!(battle_update.battle_state_revision, initial_revision + 2);
    assert_eq!(battle_update.events_delta.to_seq, initial_seq);
    assert_eq!(battle_update.server_battle_time_ms, initial_time_ms);
    assert!(battle_update.visual_novel.is_none());
    assert!(core
        .advance_active_battle_for_server_tick(10)
        .unwrap()
        .is_some());
}

#[test]
fn blocking_visual_novel_execution_preflight_rejects_every_omitted_battle_action_without_mutation()
{
    let (mut core, player_id, battle_uuid, _) = start_live_defense_battle_for_vn_test(0xD3F3_0130);
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
    let deployed_employee_uuid = employee_ids[0];
    let candidate_employee_uuid = employee_ids[1];
    let deployment_cells = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .combat_preview
        .deployment_zones
        .iter()
        .find(|zone| zone.kind == crate::game::combat_preview::DeploymentZoneKind::Ground)
        .expect("live defense battle should expose a ground deployment zone")
        .cells
        .clone();
    let deployed_position = deployment_cells[0];
    let candidate_position = deployment_cells[1];
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: deployed_employee_uuid,
            position: deployed_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let deployed_skill_id = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .live_deployment_dto(&core.state.roster)
        .unwrap()
        .deployed_units
        .iter()
        .find(|unit| unit.employee_uuid == deployed_employee_uuid)
        .and_then(|unit| unit.skill_readiness.skill_id.clone())
        .unwrap_or_else(|| SkillId::new("context_gate_probe"));

    core.state.visual_novel.active = Some(visual_novel_session_for_battle(
        "battle_vn_execution_gate",
        battle_uuid,
        "start",
        true,
    ));

    let blocked_behaviors = vec![
        PlayerBehavior::DeployUnit {
            employee_uuid: candidate_employee_uuid,
            position: candidate_position,
            facing: FacingDirection::Right,
        },
        PlayerBehavior::RequestDeploymentRangePreview {
            employee_uuid: candidate_employee_uuid,
            position: candidate_position,
        },
        PlayerBehavior::WithdrawUnit {
            employee_uuid: deployed_employee_uuid,
        },
        PlayerBehavior::ActivateSkill {
            employee_uuid: deployed_employee_uuid,
            skill_id: deployed_skill_id,
            target: None,
        },
        PlayerBehavior::PauseBattle,
        PlayerBehavior::ResumeBattle,
        PlayerBehavior::SetBattleSpeed {
            speed: crate::game::behavior::BattlePlaybackSpeed::X2,
        },
        PlayerBehavior::RetreatBattle,
    ];

    for behavior in blocked_behaviors {
        let action = behavior.kind();
        let before = serde_json::to_value(core.active_battle_reconnect_projection(0).unwrap())
            .expect("battle projection should serialize before rejected command");
        let error = if action.requires_battle_authority() {
            execute_current_battle_mutation(&mut core, player_id, behavior)
        } else {
            core.execute(player_id, behavior)
        }
        .expect_err("blocking VN must reject every omitted battle action before payload handling");

        assert!(matches!(
            error,
            GameError::ActionUnavailable {
                action: actual_action,
                reason: ActionDisabledReasonCode::ContextIrrelevant,
            } if actual_action == action
        ));
        let after = serde_json::to_value(core.active_battle_reconnect_projection(0).unwrap())
            .expect("battle projection should serialize after rejected command");
        assert_eq!(
            after, before,
            "rejected {action:?} must not change state, event log, deployment cost, or revision"
        );
    }
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
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
    let employee_uuid = employee_ids[0];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();

    let changed = execute_current_battle_mutation(
        &mut core,
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

    execute_current_battle_mutation(
        &mut core,
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
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
    let employee_uuid = employee_ids[0];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let changed = execute_current_battle_mutation(
        &mut core,
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
    let half_speed_revision = battle_update.battle_state_revision;
    assert_eq!(
        playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X0_5
    );
    let current_speed = battle_update
        .checkpoint
        .playback_eligibility
        .speeds
        .iter()
        .find(|option| option.speed == crate::game::behavior::BattlePlaybackSpeed::X0_5)
        .unwrap();
    assert_eq!(
        current_speed.eligibility.disabled_reason_code,
        Some(crate::game::behavior::ActionDisabledReasonCode::AlreadyActive)
    );

    let first_tick = core.advance_active_battle_for_server_tick(1).unwrap();
    assert!(
        first_tick.is_none(),
        "first 1ms tick at 0.5x should only accumulate fractional time"
    );

    let unchanged = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::SetBattleSpeed {
            speed: crate::game::behavior::BattlePlaybackSpeed::X0_5,
        },
    )
    .unwrap_err();
    assert!(matches!(
        unchanged,
        GameError::ActionUnavailable {
            action: ActionKind::SetBattleSpeed,
            reason: crate::game::behavior::ActionDisabledReasonCode::AlreadyActive,
        }
    ));
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle_state_revision,
        half_speed_revision
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];

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

    let before_update = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_update_dto_after(0, &core.state.roster)
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
        .battle_update_dto_after(0, &core.state.roster)
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
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
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

    let deployed = execute_current_battle_mutation(
        &mut core,
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
        .battle_update_dto_after(0, &core.state.roster)
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
    let checkpoint_range_previews = checkpoint_unit
        .range_previews
        .as_ref()
        .expect("deployed player unit should retain range previews");
    assert_eq!(
        checkpoint_range_previews.basic_attack.cells,
        pending_range_preview.basic_attack.cells
    );
    assert_eq!(
        checkpoint_range_previews.active_skill.reason,
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
    let second_deployed = execute_current_battle_mutation(
        &mut core,
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

    let withdrawn = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::WithdrawUnit { employee_uuid },
    )
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

    let revision_after_withdraw = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_state_revision;
    let repeated_withdraw = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::WithdrawUnit { employee_uuid },
    )
    .expect_err("an already withdrawn employee must not mutate deployment");
    assert!(matches!(repeated_withdraw, GameError::UnitNotFound));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.battle_state_revision, revision_after_withdraw);
    let deployment = active.live_deployment.as_ref().unwrap();
    assert!(!deployment.deployed_units.contains_key(&employee_uuid));
    assert!(deployment.redeploy_locks.contains_key(&employee_uuid));

    let cooldown_candidate = active
        .live_deployment_dto(&core.state.roster)
        .unwrap()
        .unit_deploy_costs
        .into_iter()
        .find(|candidate| candidate.employee_uuid == employee_uuid)
        .unwrap();
    assert_eq!(
        cooldown_candidate.eligibility.disabled_reason_code,
        Some(crate::game::behavior::ActionDisabledReasonCode::RedeployCooldown)
    );
    let cooldown_error = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap_err();
    assert!(matches!(
        cooldown_error,
        GameError::ActionUnavailable {
            action: ActionKind::DeployUnit,
            reason: crate::game::behavior::ActionDisabledReasonCode::RedeployCooldown,
        }
    ));

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
    let redeployed = execute_current_battle_mutation(
        &mut core,
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
fn live_deployment_reconciles_defeated_player_unit_during_authoritative_advance() {
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let deployed = execute_current_battle_mutation(
        &mut core,
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
        battle_update: deployed_update,
        ..
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

    let advanced = core
        .advance_active_battle_by(0)
        .expect("authoritative advance should finalize deployment projection");
    let BehaviorResult::BattleAdvanced {
        battle_update: advanced_update,
        ..
    } = advanced
    else {
        panic!("expected battle advance after injected defeat");
    };
    assert_eq!(
        advanced_update.battle_state_revision,
        deployed_update.battle_state_revision + 1
    );
    assert!(advanced_update
        .checkpoint
        .deployment
        .as_ref()
        .is_some_and(|deployment| deployment.deployed_units.is_empty()));

    let state = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState { since_seq: None },
        )
        .unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = state else {
        panic!("expected read-only battle state after committed reconciliation");
    };
    assert_eq!(
        battle_update.battle_state_revision,
        advanced_update.battle_state_revision
    );
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

    let persistent_employee = core
        .state
        .roster
        .get(&employee_uuid)
        .expect("battle-entry roster should retain the pre-attempt employee state");
    assert_eq!(persistent_employee.condition.trauma, 0);
    assert_eq!(persistent_employee.condition.physical_damage, 0);
    let attempt_employee = core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .expect("active attempt should project its working employee state");
    assert_eq!(
        attempt_employee.condition.trauma,
        run_policy().post_battle.incapacitation_trauma
    );
    assert_eq!(attempt_employee.condition.physical_damage, 1);
    let roster_snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let projected_employee = roster_snapshot["employees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|employee| employee["uuid"] == serde_json::json!(employee_uuid))
        .unwrap();
    assert_eq!(
        projected_employee["condition"]["trauma"],
        serde_json::json!(run_policy().post_battle.incapacitation_trauma),
        "the existing roster DTO must project current attempt-local condition"
    );

    core.advance_active_battle_by(0)
        .expect("repeated reconciliation should remain idempotent");
    let attempt_employee = core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .expect("attempt employee should remain available after one tactical death");
    assert_eq!(
        attempt_employee.condition.trauma,
        run_policy().post_battle.incapacitation_trauma,
        "one dead runtime instance must apply its penalty exactly once"
    );
    assert_eq!(attempt_employee.condition.physical_damage, 1);

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
    let redeployed = execute_current_battle_mutation(
        &mut core,
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
    let expected_death_redeploy_hp = core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .battle_start_hp_for_max(redeployed_unit.stats.max_health);
    assert_eq!(
        redeployed_unit.stats.current_health, expected_death_redeploy_hp,
        "death redeploy HP must derive from current attempt-local trauma"
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

    {
        let active = core.state.active_battle.as_mut().unwrap();
        let unit = active
            .battle
            .units
            .get_mut(&redeployed_unit_instance_id)
            .unwrap();
        unit.stats.current_health = 0;
        unit.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
    }
    core.advance_active_battle_by(0)
        .expect("second tactical death should reconcile into the same employee attempt");
    let attempt_employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert_eq!(
        attempt_employee.condition.trauma,
        run_policy()
            .post_battle
            .incapacitation_trauma
            .saturating_mul(2)
    );
    assert_eq!(attempt_employee.condition.physical_damage, 2);

    {
        let active = core.state.active_battle.as_mut().unwrap();
        let deployment = active.live_deployment.as_mut().unwrap();
        let lock = deployment.redeploy_locks.get_mut(&employee_uuid).unwrap();
        lock.ready_at_ms = active.execution.last_event_time_ms();
        deployment.current_cost = lock.deploy_cost;
    }
    let third_deployment = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect("employee should remain deployable after two nonfatal tactical deaths");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: third_unit_instance_id,
        ..
    } = third_deployment
    else {
        panic!("expected third deployment");
    };
    {
        let active = core.state.active_battle.as_ref().unwrap();
        let unit = active.battle.units.get(&third_unit_instance_id).unwrap();
        let expected_hp = active
            .employee_roster()
            .get(&employee_uuid)
            .unwrap()
            .battle_start_hp_for_max(unit.stats.max_health);
        assert_eq!(unit.stats.current_health, expected_hp);
    }

    {
        let active = core.state.active_battle.as_mut().unwrap();
        let unit = active
            .battle
            .units
            .get_mut(&third_unit_instance_id)
            .unwrap();
        unit.stats.current_health = 0;
        unit.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
    }
    core.advance_active_battle_by(0)
        .expect("third tactical death should commit the third penalty exactly once");
    let attempt_employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert_eq!(attempt_employee.condition.physical_damage, 3);
    assert!(attempt_employee.is_alive());
    assert_eq!(
        core.state
            .roster
            .get(&employee_uuid)
            .unwrap()
            .condition
            .physical_damage,
        0,
        "attempt-local condition must not mutate the battle-entry roster before commit"
    );
    let active = core.state.active_battle.as_ref().unwrap();
    let deployment = active
        .live_deployment_dto(active.employee_roster())
        .expect("live deployment projection");
    assert!(!deployment
        .redeploying_units
        .iter()
        .any(|unit| unit.employee_uuid == employee_uuid));
    assert!(!deployment
        .unit_deploy_costs
        .iter()
        .any(|unit| unit.employee_uuid == employee_uuid));

    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle)
        .expect("battle-stage abandonment should discard the active employee attempt");
    assert!(core.state.active_battle.is_none());
    let restored_employee = core.roster().unwrap().get(&employee_uuid).unwrap();
    assert_eq!(restored_employee.condition.trauma, 0);
    assert_eq!(restored_employee.condition.physical_damage, 0);
    assert!(restored_employee.is_alive());
    let restored_snapshot = core.get_employee_roster_snapshot_json().unwrap();
    let restored_projection = restored_snapshot["employees"]
        .as_array()
        .unwrap()
        .iter()
        .find(|employee| employee["uuid"] == serde_json::json!(employee_uuid))
        .unwrap();
    assert_eq!(
        restored_projection["condition"]["trauma"],
        serde_json::json!(0)
    );
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
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

    let deployed = execute_current_battle_mutation(
        &mut core,
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
            target: SkillTarget::SelfUnit,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![],
            presentation: Default::default(),
        }],
    };
    let area_skill = SkillDef {
        id: SkillId::from("catalog_target_area"),
        name: "Catalog Target Area".to_string(),
        kind: Default::default(),
        cast_targeting: SkillCastTargetingDef::explicit(
            SkillTarget::EnemySingle {
                rule: crate::game::ability::UnitTargetRule::Nearest,
            },
            crate::game::ability::SkillTargetReach::Pattern,
            Some(TileRangePattern {
                include_anchor_tile: false,
                rows: vec![".@X".to_string()],
            }),
            false,
        ),
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "area".to_string(),
            delay_ms: 0,
            target: SkillTarget::EnemySingle {
                rule: crate::game::ability::UnitTargetRule::Nearest,
            },
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::TileArea {
                footprint: TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec!["XXX".to_string(), "X@X".to_string(), "XXX".to_string()],
                },
                area: crate::game::ability::SkillTileAreaDeliveryDef {
                    anchor: crate::game::ability::SkillAreaAnchorSource::CastTarget,
                    tile_origin: crate::game::ability::SkillTileAreaOrigin::Anchor,
                    ..Default::default()
                },
            },
            effects: vec![],
            presentation: Default::default(),
        }],
    };
    let weapon = weapon_equipment(0xE005, "manual_fragment_sword", WeaponArchetype::Sword);
    let mut core = GameCore::new(
        game_data_with_pve_equipment_and_active_skill_fragments(
            vec![weapon.clone()],
            vec![skill, area_skill],
            vec![fragment.clone()],
        ),
        123,
    );
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
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
    execute_current_battle_mutation(
        &mut core,
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
    assert!(catalog_skill["cast_target"].get("reach").is_some());
    assert!(catalog_skill["cast_target"].get("range_policy").is_none());
    assert!(catalog_skill["steps"][0].get("range_policy").is_none());
    assert!(catalog_skill["steps"][0]
        .get("defense_tile_range")
        .is_none());

    let area_skill = catalog_skills
        .iter()
        .find(|skill| skill["skill_id"] == "catalog_target_area")
        .expect("live target-centered area skill should be projected");
    assert_eq!(area_skill["cast_target"]["reach"], "pattern");
    assert!(area_skill["cast_target"]["target_range"].is_object());
    assert!(area_skill["steps"][0]["tile_area"]["footprint"].is_object());
    assert_eq!(area_skill["steps"][0]["tile_area"]["tile_origin"], "Anchor");

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

    let activated = execute_current_battle_mutation(
        &mut core,
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
fn manual_fragment_readiness_disables_payload_when_target_is_missing() {
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
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
    execute_current_battle_mutation(
        &mut core,
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
        deployed["skill_readiness"]["manual_activation_allowed"], false,
        "Core must not advertise a skill payload with no valid target"
    );
    assert_eq!(deployed["skill_readiness"]["target_required"], true);
    assert_eq!(deployed["skill_readiness"]["target_available"], false);
    assert_eq!(
        deployed["skill_readiness"]["eligibility"]["disabled_reason_code"],
        "no_valid_target"
    );
    assert_eq!(deployed["skill_readiness"]["eligibility"]["enabled"], false);

    let activation_error = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::ActivateSkill {
            employee_uuid,
            skill_id,
            target: None,
        },
    )
    .unwrap_err();
    assert!(matches!(
        activation_error,
        GameError::ActionUnavailable {
            action: ActionKind::ActivateSkill,
            reason: crate::game::behavior::ActionDisabledReasonCode::NoValidTarget,
        }
    ));
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
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
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
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
    }

    let (revision_before_failed_deploy, salt_before_failed_deploy, cost_before_failed_deploy) = {
        let active = core.state.active_battle.as_ref().unwrap();
        let deployment = active.live_deployment.as_ref().unwrap();
        (
            active.battle_state_revision,
            deployment.next_instance_salt,
            deployment.current_cost,
        )
    };

    let projected_deployment = core
        .state
        .active_battle
        .as_ref()
        .and_then(|active| active.live_deployment_dto(&core.state.roster))
        .unwrap();
    let third_candidate = projected_deployment
        .unit_deploy_costs
        .iter()
        .find(|candidate| candidate.employee_uuid == third_uuid)
        .unwrap();
    assert!(!third_candidate.eligibility.enabled);
    assert_eq!(
        third_candidate.eligibility.disabled_reason_code,
        Some(crate::game::behavior::ActionDisabledReasonCode::InsufficientResources)
    );

    let insufficient = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: third_uuid,
            position: deploy_positions[2],
            facing: FacingDirection::Right,
        },
    )
    .expect_err("third deployment should require restored stabilization");
    assert!(matches!(
        insufficient,
        GameError::ActionUnavailable {
            action: ActionKind::DeployUnit,
            reason: crate::game::behavior::ActionDisabledReasonCode::InsufficientResources,
        }
    ));
    let active = core.state.active_battle.as_ref().unwrap();
    let deployment = active.live_deployment.as_ref().unwrap();
    assert_eq!(active.battle_state_revision, revision_before_failed_deploy);
    assert_eq!(deployment.next_instance_salt, salt_before_failed_deploy);
    assert_eq!(deployment.current_cost, cost_before_failed_deploy);
    assert!(!deployment.deployed_units.contains_key(&third_uuid));

    execute_current_battle_mutation(
        &mut core,
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

    let deployed = execute_current_battle_mutation(
        &mut core,
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
fn retreat_battle_restores_exact_pre_entry_snapshot_and_rng_continuation() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 127);
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
    let exact_pre_entry = core.get_run_snapshot_json().unwrap();

    let entered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: first_battle_uuid,
        ..
    } = entered
    else {
        panic!("expected first active battle");
    };
    for index in 1..=58_u128 {
        core.state
            .inventory
            .add_item_owned(
                Uuid::from_u128(0xB700_0000 + index),
                Item::Equipment(Arc::new(equipment_meta(
                    0xB710_0000 + index,
                    &format!("retreat_inventory_probe_{index}"),
                    EquipmentType::Accessory,
                ))),
            )
            .unwrap();
    }
    assert_eq!(core.state.inventory.reserved_slot_count(), 70);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle).unwrap();

    assert_eq!(
        core.get_run_snapshot_json().unwrap(),
        exact_pre_entry,
        "battle-stage abandonment must restore the exact authoritative pre-entry projection"
    );

    let reentered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: second_battle_uuid,
        ..
    } = reentered
    else {
        panic!("expected reentered active battle");
    };
    assert_eq!(
        second_battle_uuid, first_battle_uuid,
        "restored UUID/RNG continuation must reproduce the abandoned entry identity"
    );
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(core.get_run_snapshot_json().unwrap(), exact_pre_entry);
}

#[test]
fn committed_fatal_defeat_preempts_retry_while_attempts_remain() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 128);
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

    let mut employee_conditions = core.state.roster.clone();
    for employee in employee_conditions.iter_mut() {
        employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE;
    }
    let event_log = BattleEventLog::default();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        result_stats: test_result_stats(BattleWinner::Opponent, &event_log, &[]),
        event_log,
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions,
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
            reason: RunFailureReason::NoLivingEmployees,
            ..
        }
    ));
    assert!(matches!(
        core.get_state(),
        GameState::RunFailed {
            reason: RunFailureReason::NoLivingEmployees
        }
    ));
}

fn install_combat_recovery_test_map(
    core: &mut GameCore,
    immediate_recruitment: bool,
) -> (MapNodeId, MapNodeId, MapNodeId) {
    let start_id = MapNodeId::new(Uuid::from_u128(0xA150_01));
    let combat_id = MapNodeId::new(Uuid::from_u128(0xA150_02));
    let recruitment_id = MapNodeId::new(Uuid::from_u128(0xA150_03));
    let save_point_id = MapNodeId::new(Uuid::from_u128(0xA150_04));
    let maintenance_id = MapNodeId::new(Uuid::from_u128(0xA150_05));
    let supply_id = MapNodeId::new(Uuid::from_u128(0xA150_06));
    let crossroads_id = MapNodeId::new(Uuid::from_u128(0xA150_07));
    let guard_id = MapNodeId::new(Uuid::from_u128(0xA150_08));
    let gate_id = MapNodeId::new(Uuid::from_u128(0xA150_09));
    let power_model_version = core
        .game_data
        .run_policy
        .combat_difficulty
        .power_model_version;
    let floor_power_anchor = core.state.run_power_authority.current_floor_power_anchor;
    let node = |id, slot_id: &str, kind_id: &str, category, state, payload| MapNode {
        id,
        slot_id: MapSlotId::new(slot_id),
        kind_id: MapNodeKindId::new(kind_id),
        category,
        state,
        visibility: MapNodeVisibility::Revealed,
        payload,
        omen: None,
        combat_difficulty: matches!(category, MapNodeCategory::Combat | MapNodeCategory::Boss)
            .then(|| crate::game::map::MapNodeCombatDifficulty {
                power_model_version,
                difficulty_profile_id: "default_facility".to_string(),
                class: if kind_id == "combat_elite" {
                    crate::game::map::MapNodeCombatDifficultyClass::Role(
                        crate::game::data::run_policy_data::CombatNodeDifficultyRole::Elite,
                    )
                } else {
                    crate::game::map::MapNodeCombatDifficultyClass::Ordinary(
                        crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Standard,
                    )
                },
                monolith_stage: None,
                floor_power_anchor,
                target_power: floor_power_anchor,
            }),
    };
    let transit_state = if immediate_recruitment {
        MapNodeState::Completed
    } else {
        MapNodeState::Unavailable
    };
    let map = RunMap {
        map_template_id: MapTemplateId::new("standard_branching_a"),
        facility_template_version: FACILITY_TEMPLATE_VERSION,
        nodes: vec![
            node(
                start_id,
                "entry",
                "start",
                MapNodeCategory::Start,
                MapNodeState::Completed,
                MapNodePayload::None,
            ),
            node(
                combat_id,
                "dispatch_hall",
                "combat_defense",
                MapNodeCategory::Combat,
                MapNodeState::Available,
                MapNodePayload::Encounter {
                    encounter_id: Some("defense_encounter".to_string()),
                },
            ),
            node(
                save_point_id,
                "save_archive",
                "save_point_a",
                MapNodeCategory::SavePoint,
                MapNodeState::Unavailable,
                MapNodePayload::SavePoint,
            ),
            node(
                maintenance_id,
                "maintenance_bay",
                "maintenance",
                MapNodeCategory::Maintenance,
                transit_state,
                MapNodePayload::Maintenance,
            ),
            node(
                recruitment_id,
                "hq_terminal",
                "headquarters_contact",
                MapNodeCategory::HeadquartersContact,
                MapNodeState::Unavailable,
                MapNodePayload::HeadquartersContact {
                    shop_pool_id: None,
                    candidate_count: 2,
                },
            ),
            node(
                supply_id,
                "supply_depot",
                "shop",
                MapNodeCategory::Shop,
                transit_state,
                MapNodePayload::Shop {
                    shop_id: None,
                    shop_pool_id: None,
                },
            ),
            node(
                crossroads_id,
                "crossroads_archive",
                "event_story",
                MapNodeCategory::Event,
                transit_state,
                MapNodePayload::Event { event_id: None },
            ),
            node(
                guard_id,
                "containment_guard",
                "combat_elite",
                MapNodeCategory::Combat,
                MapNodeState::Unavailable,
                MapNodePayload::Encounter { encounter_id: None },
            ),
            node(
                gate_id,
                "floor_gate",
                "gate_stairs",
                MapNodeCategory::Gate,
                MapNodeState::Unavailable,
                MapNodePayload::None,
            ),
        ],
        start_node_ids: vec![start_id],
        terminals: vec![MapTerminalNode {
            node_id: gate_id,
            resolution: FacilityTerminalResolution::FloorGate,
        }],
    };
    let progression = crate::game::map::MapProgression {
        current_node_id: Some(start_id),
        available_node_ids: vec![combat_id],
        completed_node_ids: if immediate_recruitment {
            vec![start_id, maintenance_id, supply_id, crossroads_id]
        } else {
            vec![start_id]
        },
    };
    let previous_run = core.state.run.as_ref().unwrap();
    let run_progression = previous_run.run_progression.clone();
    let recruitment = previous_run.recruitment.clone();
    core.state.run =
        Some(RunState::new(map, progression, run_progression).with_recruitment(recruitment));
    (combat_id, recruitment_id, save_point_id)
}

fn install_no_deployable_combat_result(
    core: &mut GameCore,
    node_id: MapNodeId,
    winner: BattleWinner,
) {
    let mut employee_conditions = core.state.roster.clone();
    for employee in employee_conditions.iter_mut() {
        employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE - 1;
    }
    let event_log = BattleEventLog::default();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner,
        result_stats: test_result_stats(winner, &event_log, &[]),
        event_log,
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions,
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();
}

#[test]
fn no_deployable_defeat_allows_immediate_recovery_nodes_then_rechecks() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 129);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    core.save_run_checkpoint().unwrap();
    assert!(core.state.run_checkpoint.can_load());
    let (combat_id, recruitment_id, save_point_id) =
        install_combat_recovery_test_map(&mut core, true);
    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode { node_id: combat_id },
    )
    .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    install_no_deployable_combat_result(&mut core, combat_id, BattleWinner::Opponent);

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    assert_eq!(
        core.selectable_map_node_ids(),
        vec![recruitment_id, save_point_id]
    );
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(
        snapshot["map_navigation"]["selectable_node_ids"],
        json!([recruitment_id, save_point_id])
    );
    assert_eq!(
        snapshot["combat_recovery_restriction"],
        json!({
            "reason": "NoDeployableEmployees",
            "allowed_node_ids": [recruitment_id, save_point_id],
        })
    );
    assert!(snapshot["action_capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |capability| capability["action"] == json!("LoadRunCheckpoint")
                && capability["enabled"] == json!(true)
        ));
    let save_point_projection = snapshot["map"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"] == json!(save_point_id))
        .unwrap();
    assert_eq!(save_point_projection["state"], json!("Available"));

    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: recruitment_id,
        },
    )
    .unwrap();
    let entered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::HeadquartersContactState {
        recruitment_candidates,
        ..
    } = entered
    else {
        panic!("typed recovery node should enter headquarters recruitment");
    };
    let employee_definition_id = recruitment_candidates[0].employee_definition_id.clone();
    let recruited = core
        .execute(
            player_id,
            PlayerBehavior::RecruitEmployee {
                employee_definition_id,
            },
        )
        .unwrap();
    assert!(matches!(
        recruited,
        BehaviorResult::EmployeeRecruited { .. }
    ));
    assert!(!core.has_combat_recovery_restriction());
    assert!(!core
        .roster()
        .unwrap()
        .combat_eligible_employee_ids()
        .is_empty());
    assert!(matches!(core.get_state(), GameState::ViewingMap));
}

#[test]
fn no_deployable_victory_uses_the_same_typed_recovery_restriction() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 131);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let (combat_id, recruitment_id, save_point_id) =
        install_combat_recovery_test_map(&mut core, true);
    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode { node_id: combat_id },
    )
    .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    install_no_deployable_combat_result(&mut core, combat_id, BattleWinner::Player);

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let BehaviorResult::CombatRewardsGranted { completion, .. } = result else {
        panic!("victory should retain its normal committed reward result");
    };
    assert!(matches!(*completion, BehaviorResult::NodeCompleted { .. }));
    assert_eq!(
        core.selectable_map_node_ids(),
        vec![recruitment_id, save_point_id]
    );
}

#[test]
fn recovery_node_that_resolves_without_restoring_deployability_fails_run() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 132);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let (combat_id, recruitment_id, _) = install_combat_recovery_test_map(&mut core, true);
    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode { node_id: combat_id },
    )
    .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    install_no_deployable_combat_result(&mut core, combat_id, BattleWinner::Opponent);
    core.execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let fragment_id = starter_basic_attack_fragment_id();
    let starting_fragment_count = core.state.skill_fragments.count(&fragment_id);
    core.state
        .skill_fragments
        .add_research_progress(&fragment_id, 100)
        .expect("recovery-node activity should queue delivery");
    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: recruitment_id,
        },
    )
    .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::RequestEmergencySupplies)
        .unwrap();
    let BehaviorResult::EmergencySuppliesGranted { completion, .. } = result else {
        panic!("headquarters supplies should resolve its node");
    };
    assert!(matches!(
        *completion,
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
    assert_eq!(
        core.state.skill_fragments.count(&fragment_id),
        starting_fragment_count,
        "recovery resolution ending in RunFailed must not deliver research"
    );
    assert_eq!(
        core.state
            .skill_fragments
            .pending_research_deliveries()
            .collect::<Vec<_>>(),
        vec![(&fragment_id, 1)]
    );
}

#[test]
fn future_recruitment_and_immediate_save_point_do_not_defer_no_deployable_failure() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 130);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let (combat_id, _, _) = install_combat_recovery_test_map(&mut core, false);
    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode { node_id: combat_id },
    )
    .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    install_no_deployable_combat_result(&mut core, combat_id, BattleWinner::Opponent);

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
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
}

#[test]
fn headquarters_without_authored_candidates_is_not_a_recovery_capability() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 133);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let (combat_id, headquarters_id, _) = install_combat_recovery_test_map(&mut core, true);
    core.state
        .run
        .as_mut()
        .unwrap()
        .map
        .node_mut(headquarters_id)
        .unwrap()
        .payload = MapNodePayload::HeadquartersContact {
        shop_pool_id: None,
        candidate_count: 0,
    };
    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode { node_id: combat_id },
    )
    .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    install_no_deployable_combat_result(&mut core, combat_id, BattleWinner::Opponent);

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
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
fn repeated_retreats_never_consume_attempts_or_resolve_the_node() {
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
    let pre_entry = core.get_run_snapshot_json().unwrap();
    let mut first_battle_uuid = None;

    for _ in 0..4 {
        let entered = core
            .execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        let BehaviorResult::BattleAdvanced {
            battle_uuid,
            finished: false,
            ..
        } = entered
        else {
            panic!("expected active battle");
        };
        assert_eq!(*first_battle_uuid.get_or_insert(battle_uuid), battle_uuid);
        let result =
            execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle)
                .unwrap();
        assert!(matches!(result, BehaviorResult::NodePreview { .. }));
        assert_eq!(core.get_run_snapshot_json().unwrap(), pre_entry);
    }

    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .node(node_id)
        .is_some_and(|node| {
            node.state != crate::game::map::MapNodeState::Completed
                && node.visibility == crate::game::map::MapNodeVisibility::Revealed
        }));
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
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
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
        employee_conditions: core.state.roster.clone(),
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
fn final_boss_retreat_is_also_exact_and_non_consuming() {
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
    core.state.run.as_mut().unwrap().map.terminals = vec![MapTerminalNode {
        node_id,
        resolution: FacilityTerminalResolution::RunFinalBoss,
    }];
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let pre_entry = core.get_run_snapshot_json().unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let result =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle)
            .unwrap();
    assert!(matches!(result, BehaviorResult::NodePreview { .. }));
    assert_eq!(core.get_run_snapshot_json().unwrap(), pre_entry);
}

#[test]
fn final_boss_defeat_with_valid_checkpoint_requires_explicit_decision() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let fragment_id = starter_basic_attack_fragment_id();
    let starting_fragment_count = core.state.skill_fragments.count(&fragment_id);
    core.state
        .skill_fragments
        .add_research_progress(&fragment_id, 100)
        .expect("research completion should queue delivery");
    core.save_run_checkpoint().unwrap();
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
    core.state.run.as_mut().unwrap().map.terminals = vec![MapTerminalNode {
        node_id,
        resolution: FacilityTerminalResolution::RunFinalBoss,
    }];
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let event_log = encounter_compendium_spawn_log();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
        encounter_id: "final_boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        result_stats: test_result_stats(BattleWinner::Opponent, &event_log, &[]),
        event_log,
        bonus_objectives: vec![],
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
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
        BehaviorResult::RunFailureDecisionRequired {
            reason: RunFailureReason::BossDefeated,
            ..
        }
    ));
    assert!(matches!(
        core.get_state(),
        GameState::AwaitingRunFailureDecision {
            reason: RunFailureReason::BossDefeated,
            ..
        }
    ));
    let action_kinds = core
        .get_action_capabilities()
        .into_iter()
        .map(|capability| capability.action)
        .collect::<Vec<_>>();
    assert!(action_kinds.contains(&ActionKind::LoadRunCheckpoint));
    assert!(action_kinds.contains(&ActionKind::ConfirmRunFailure));
    assert!(!action_kinds.contains(&ActionKind::AbandonRun));
    assert!(core
        .encounter_compendium_profile()
        .discovered_abnormality_ids
        .contains("defense_risk_abno"));
    assert!(core
        .encounter_compendium_profile()
        .discovered_corrosion_profile_ids
        .contains("corroded_guard"));
    assert_eq!(
        core.state.skill_fragments.count(&fragment_id),
        starting_fragment_count,
        "RunFailed must not synthesize a planning-return delivery"
    );
    assert_eq!(
        core.state
            .skill_fragments
            .pending_research_deliveries()
            .collect::<Vec<_>>(),
        vec![(&fragment_id, 1)]
    );
}

#[test]
fn forced_boss_omen_defeat_immediately_fails_endless_run() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);
    core.save_run_checkpoint().unwrap();
    core.state.run_checkpoint.loads_used = crate::game::world::state::RUN_CHECKPOINT_MAX_LOADS;
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
        employee_conditions: core.state.roster.clone(),
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
    assert!(core.get_action_capabilities().is_empty());
}

#[test]
fn forced_boss_omen_failure_requires_decision_and_executes_valid_checkpoint_load() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 134);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);
    core.save_run_checkpoint().unwrap();
    let checkpoint_map = core.current_map_view().unwrap();
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
        employee_conditions: core.state.roster.clone(),
    };
    core.state.active_battle = None;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));
    core.transition_to(GameState::CombatResult {
        battle_uuid: node_id.0,
    })
    .unwrap();
    let failed = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    assert!(matches!(
        failed,
        BehaviorResult::RunFailureDecisionRequired {
            reason: RunFailureReason::BossDefeated,
            ..
        }
    ));
    assert!(matches!(
        core.get_state(),
        GameState::AwaitingRunFailureDecision {
            reason: RunFailureReason::BossDefeated,
            ..
        }
    ));
    let capability = action_capability(&core, ActionKind::LoadRunCheckpoint)
        .expect("BossOmen failure decision should project checkpoint load");
    assert!(capability.enabled);
    assert!(
        action_capability(&core, ActionKind::ConfirmRunFailure)
            .expect("failure decision should permit explicit confirmation")
            .enabled
    );
    assert!(action_capability(&core, ActionKind::AbandonRun).is_none());

    let loaded = core
        .execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap();
    assert!(matches!(loaded, BehaviorResult::MapState { .. }));
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    assert_eq!(core.state.run_checkpoint.loads_used, 1);
    assert_eq!(core.current_map_view().unwrap(), checkpoint_map);
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .boss_omen
        .forced_boss
        .is_none());
}

#[test]
fn forced_boss_omen_victory_opens_dedicated_gate_before_endless_floor_advance() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .condition
        .trauma = 90;
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_omen_final_boss",
        "final_boss_risk_encounter",
    );
    {
        let run = core.state.run.as_mut().unwrap();
        run.map.nodes.retain(|existing| existing.id == node_id);
        run.map.start_node_ids = vec![node_id];
        run.map.map_template_id = MapTemplateId::new("boss_omen_capstone");
        run.map.node_mut(node_id).expect("forced boss node").slot_id =
            MapSlotId::new("boss_chamber");
        run.map_progression = crate::game::map::MapProgression::from_map(&run.map);
        let gate_id = MapNodeId::new(Uuid::from_u128(0xB055_E017));
        run.map.nodes.push(MapNode {
            id: gate_id,
            slot_id: MapSlotId::new("exit_gate"),
            kind_id: MapNodeKindId::new("gate_stairs"),
            category: MapNodeCategory::Gate,
            state: MapNodeState::Unavailable,
            visibility: MapNodeVisibility::Concealed,
            payload: MapNodePayload::None,
            omen: None,
            combat_difficulty: None,
        });
        run.map.terminals = vec![MapTerminalNode {
            node_id: gate_id,
            resolution: FacilityTerminalResolution::BossOmenExitGate,
        }];
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
        employee_conditions: core.state.roster.clone(),
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
        panic!("forced boss victory should grant combat rewards before opening the exit Gate");
    };
    assert!(matches!(*completion, BehaviorResult::NodeCompleted { .. }));
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    assert_eq!(
        core.roster()
            .unwrap()
            .get(&employee_uuid)
            .unwrap()
            .condition
            .trauma,
        90,
        "boss victory must not apply Gate trauma recovery before Gate confirmation"
    );
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .run_progression
            .floor_index(),
        0,
        "forced boss victory must stay on the current Floor until Gate confirmation"
    );
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
    let gate_ids = core.selectable_map_node_ids();
    assert_eq!(gate_ids.len(), 1);
    let gate_id = gate_ids[0];
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .node(gate_id)
        .is_some_and(
            |node| node.category == MapNodeCategory::Gate && node.kind_id.as_str() == "gate_stairs"
        ));

    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode { node_id: gate_id },
    )
    .unwrap();
    let advanced = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(matches!(
        advanced,
        BehaviorResult::FloorAdvanced {
            game_mode: GameMode::Endless,
            floor_index: 1,
            ..
        }
    ));
    assert_eq!(
        core.roster()
            .unwrap()
            .get(&employee_uuid)
            .unwrap()
            .condition
            .trauma,
        81
    );
}

#[test]
fn boss_kind_combat_defeat_reenters_while_attempts_remain() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "boss_abnormality",
        "boss_risk_encounter",
    );
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .map
            .terminal_resolution(node_id),
        None
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
        employee_conditions: core.state.roster.clone(),
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
    let fragment_id = starter_basic_attack_fragment_id();
    let starting_fragment_count = core.state.skill_fragments.count(&fragment_id);
    core.state
        .skill_fragments
        .add_research_progress(&fragment_id, 100)
        .expect("research completion should queue delivery");
    core.set_enkephalin(5);
    let event_node_id =
        force_single_event_node_run_for_combat_tests(&mut core, "event_started_victory");
    enter_event_combat_choice(
        &mut core,
        player_id,
        event_node_id,
        "event_started_victory",
        "fight",
    );
    assert_eq!(
        core.state.skill_fragments.count(&fragment_id),
        starting_fragment_count,
        "Event entry and nested combat start must not deliver research"
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
        employee_conditions: core.state.roster.clone(),
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

    let BehaviorResult::CombatRewardsGranted {
        enkephalin,
        completion,
        ..
    } = result
    else {
        panic!("terminal Event combat victory should grant combat rewards");
    };
    assert_eq!(enkephalin, 5);
    assert_eq!(core.get_enkephalin(), 5);
    let BehaviorResult::NodeCompleted {
        research_deliveries,
        ..
    } = *completion
    else {
        panic!("authored Event room victory should consume the node after rewards");
    };
    assert_eq!(research_deliveries.len(), 1);
    assert_eq!(research_deliveries[0].fragment_id, fragment_id);
    assert_eq!(
        core.state.skill_fragments.count(&fragment_id),
        starting_fragment_count + 1
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
        employee_conditions: core.state.roster.clone(),
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
        session
            .committed_check
            .as_ref()
            .map(|check| check.choice_id.as_str()),
        Some("fight")
    );
    assert!(session.started_combat.is_some());
}

#[test]
fn event_started_combat_defeat_has_no_duplicate_event_reward_policy() {
    let event = combat_choice_event_definition(
        "event_started_defeat_reward",
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
    core.set_enkephalin(5);
    let event_node_id =
        force_single_event_node_run_for_combat_tests(&mut core, "event_started_defeat_reward");
    enter_event_combat_choice(
        &mut core,
        player_id,
        event_node_id,
        "event_started_defeat_reward",
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
        employee_conditions: core.state.roster.clone(),
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
    assert_eq!(
        core.get_enkephalin(),
        5,
        "Event-started combat uses the normal battle reward boundary only"
    );
}

#[test]
fn retreat_discards_battle_local_threat_warning_mutations() {
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
    let original_warnings = core
        .combat_preview_for_node(node_id)
        .unwrap()
        .unwrap()
        .threat_warnings;
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

    let result =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle)
            .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(preview),
        ..
    } = result
    else {
        panic!("retreat should return node preview while attempts remain");
    };

    assert_eq!(preview.threat_warnings, original_warnings);

    let cached = core
        .state
        .run
        .as_ref()
        .unwrap()
        .combat_previews
        .get(&node_id)
        .expect("retreat should restore the cached node preview");
    assert_eq!(cached.threat_warnings, original_warnings);
}

#[test]
fn consumable_modifier_survives_abandonment_and_expires_on_committed_result() {
    let consumable = consumable_meta(
        0xC030,
        "reentry_ampoule",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 20 },
    );
    let owned_uuid = consumable.uuid;
    let mut core = GameCore::new(
        game_data_with_pve_and_consumables(vec![consumable.clone()]),
        123,
    );
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.inventory_mut()
        .unwrap()
        .add_consumable(Arc::new(consumable), 1)
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

    for _ in 0..2 {
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle)
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
            json!(3)
        );
    }

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let battle = CombatBattleState {
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
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
        employee_conditions: core.state.roster.clone(),
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
    assert!(core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .active_consumable_modifier
        .is_none());
}
