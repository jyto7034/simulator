fn projection_after_test_first_entry(core: &GameCore, node_id: MapNodeId) -> Value {
    let progression = &core.state.run.as_ref().unwrap().run_progression;
    assert_eq!(progression.game_mode, GameMode::Expedition);
    assert!(!progression.entry_receipts.contains(&node_id));
    assert_eq!(core.run_policy().expedition_first_entry_cost, 1);
    assert!(core.state.roster.iter().all(|employee|!employee.condition.is_breakdown()));
    // Independently establish the paid-entry state for the whole-root retreat
    // oracle, including forecasts of all other nodes at the new balance.
    // Literal preview arithmetic is separately checked by the U14 cost oracle.
    let mut expected=crate::authoring::clone_shape::controlled_clone(core).unwrap();
    let run=expected.state.run.as_mut().unwrap();
    run.run_progression.deterrence=progression.deterrence-1;
    run.run_progression.entry_receipts.insert(node_id);
    expected.get_run_snapshot_json().unwrap()
}

fn projection_after_test_retreat(mut before: Value, retreats: u8) -> Value {
    let attempt = &mut before["game_state_context"]["abnormality_attempt"];
    assert!(
        attempt.is_object(),
        "fixture must expose its retreat ledger"
    );
    attempt["attempts_started"] = json!(retreats);
    attempt["remaining_attempts"] = json!(3 - retreats);
    attempt["retreats_used"] = json!(retreats);
    attempt["remaining_retreats"] = json!(2 - retreats);
    before
}

fn settle_test_combat_result(
    core: &mut GameCore,
    battle_uuid: Uuid,
    battle: CombatBattleState,
) -> Result<(), GameError> {
    core.execute_application(
        super::super::application::ApplicationRoute::BattleElapsedAdvance,
        move |draft| draft.transition_finished_live_battle_to_result(battle_uuid, battle),
    )
}

use super::*;
// External content-pack fixtures and their loader are only available on Unix.
#[cfg(unix)]
#[path = "resource_lifetimes.rs"]
mod resource_lifetimes;
use crate::game::ability::SkillEffectDef;
use crate::game::battle::buffs::BuffId;
use crate::game::battle::core::movement::types::EventLogVec2;
use crate::game::battle::core::sim::BattleDeployCurrentHpPolicy;
use crate::game::battle::core::{CompendiumCandidate, CompendiumCandidateKind};
use crate::game::battle::enums::BattleEvent;
use crate::game::battle::event_log::{
    BattleEventCause, BattleEventLogEntry, BattleLogEvent, BATTLE_HISTORY_MAX_ENCODED_BYTES,
};
use crate::game::battle::result_stats::collect_battle_result_stats;
use crate::game::battle::tile_range::TileRangePattern;
use crate::game::battle::types::{
    BattleOutcomeId, BattleUnitRole, BattleUnitSourceIdentity, BattleUnitThreatClass,
    BattleVerdict, BattleWinner, EngagementStance, ParticipantBattleResult,
};
use crate::game::behavior::ActionDisabledReasonCode;
use crate::game::behavior::{
    LiveBattleHudBarMode, LiveBattlePresentationEventKindDto, LiveBattleTileEffectDto,
};
use crate::game::combat_preview::DeploymentZoneKind;
use crate::game::data::Item;
use crate::game::enums::Side;
use crate::game::resources::{CombatBattleState, RunFailureReason};
use crate::game::stats::UnitStats;

fn fixture_verdict(winner: BattleWinner) -> BattleVerdict {
    match winner {
        BattleWinner::Player => BattleVerdict::Victory,
        BattleWinner::Opponent => BattleVerdict::Defeat,
        BattleWinner::Draw => panic!("named-outcome fixtures cannot use Draw"),
    }
}

fn fixture_outcome_id(winner: BattleWinner) -> BattleOutcomeId {
    match fixture_verdict(winner) {
        BattleVerdict::Victory => BattleOutcomeId::new("victory"),
        BattleVerdict::Defeat => BattleOutcomeId::new("defeat"),
    }
}

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
    BattleEventLog::from_entries(vec![
        BattleEventLogEntry {
            skill_effect_source: None,
            time_ms: 0,
            seq: 1,
            cause: crate::game::battle::event_log::BattleEventCause::default(),
            source_command_id: None,
            event: BattleLogEvent::UnitSpawned {
                unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
                    0xC011_EC71,
                )),
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
            skill_effect_source: None,
            time_ms: 1_000,
            seq: 2,
            cause: crate::game::battle::event_log::BattleEventCause::default(),
            source_command_id: None,
            event: BattleLogEvent::UnitSpawned {
                unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
                    0xC011_EC72,
                )),
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
            skill_effect_source: None,
            time_ms: 2_000,
            seq: 3,
            cause: crate::game::battle::event_log::BattleEventCause::default(),
            source_command_id: None,
            event: BattleLogEvent::UnitDied {
                unit_instance_id: crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(
                    0xC011_EC72,
                )),
                owner: Side::Opponent,
                killer_instance_id: None,
                world_position: EventLogVec2::default(),
                position: crate::game::resources::Position { x: 0, y: 0 },
            },
        },
    ])
}

fn encounter_compendium_candidates() -> Vec<CompendiumCandidate> {
    vec![
        CompendiumCandidate {
            id: "defense_risk_abno".to_string(),
            kind: CompendiumCandidateKind::Abnormality,
            base_uuid: Uuid::from_u128(20_005),
        },
        CompendiumCandidate {
            id: "corroded_guard".to_string(),
            kind: CompendiumCandidateKind::CorrodedEmployee,
            base_uuid: Uuid::from_u128(21_001),
        },
    ]
}

fn install_encounter_compendium_combat_result(
    core: &mut GameCore,
    node_id: MapNodeId,
    winner: BattleWinner,
) {
    let event_log = encounter_compendium_spawn_log();
    let battle = CombatBattleState {
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner,
        outcome_id: fixture_outcome_id(winner),
        verdict: fixture_verdict(winner),
        grant_node_bonus: winner == BattleWinner::Player,
        advance_encounter_progression: winner == BattleWinner::Player,
        result_stats: test_result_stats(winner, &event_log, &[]),
        event_log,
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: encounter_compendium_candidates(),
    };
    settle_test_combat_result(core, node_id.0, battle).unwrap();
}

fn install_victorious_source_combat_result(
    core: &mut GameCore,
    node_id: MapNodeId,
    encounter_id: &str,
    abnormality_id: &str,
) -> Result<(), GameError> {
    let event_log = BattleEventLog::default();
    let battle = CombatBattleState {
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some(abnormality_id.to_string()),
        encounter_id: encounter_id.to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Player,
        outcome_id: fixture_outcome_id(BattleWinner::Player),
        verdict: fixture_verdict(BattleWinner::Player),
        grant_node_bonus: BattleWinner::Player == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Player == BattleWinner::Player,
        result_stats: test_result_stats(BattleWinner::Player, &event_log, &[]),
        event_log,
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![CompendiumCandidate {
            id: abnormality_id.to_string(),
            kind: CompendiumCandidateKind::Abnormality,
            base_uuid: node_id.0,
        }],
    };
    settle_test_combat_result(core, node_id.0, battle)
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

#[test]
fn presentation_and_battle_start_resolvers_preserve_complete_manifested_previews_and_cache_roles() {
    for (category, kind_id, encounter_id) in [
        (
            MapNodeCategory::Combat,
            "combat_defense",
            "defense_encounter",
        ),
        (MapNodeCategory::Boss, "boss", "boss_risk_encounter"),
    ] {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC0_5103);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_default_starters(&mut core, player_id);
        let node_id = force_map_combat_node(&mut core, category, kind_id, encounter_id);

        let selection = core
            .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
            .expect("presentation resolution should succeed");
        let BehaviorResult::NodePreview {
            combat_preview: Some(presentation),
            ..
        } = selection
        else {
            panic!("manifested combat should publish a presentation preview");
        };

        core.state
            .run
            .as_mut()
            .expect("run state")
            .combat_previews
            .get_mut(&node_id)
            .expect("presentation resolution should publish its cache")
            .encounter_id = Some("cached-presentation-only".to_string());

        let cached_presentation = core
            .combat_preview_for_node(node_id)
            .expect("cache resolution should succeed")
            .expect("manifested combat should retain its cached preview");
        assert_eq!(
            cached_presentation.encounter_id.as_deref(),
            Some("cached-presentation-only")
        );

        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .expect("battle start must regenerate from current authority");
        let regenerated_battle_start = core
            .state
            .active_battle
            .as_ref()
            .expect("battle should be active")
            .combat_preview
            .clone();
        assert_eq!(regenerated_battle_start, presentation);
        assert_ne!(regenerated_battle_start, cached_presentation);
    }
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("abno".to_string()),
        encounter_id: "encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ChooseOne,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
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
    battle.outcome_id = fixture_outcome_id(BattleWinner::Player);
    battle.verdict = fixture_verdict(BattleWinner::Player);
    battle.grant_node_bonus = true;
    battle.advance_encounter_progression = true;
    core.state.active_node_content = Some(ActiveNodeContent::CombatBattle(battle));

    let err = core.handle_complete_combat_result().unwrap_err();

    assert!(matches!(err, GameError::InvalidAction));
}

#[test]
fn combat_trace_export_failure_does_not_block_result_transition() {
    let run_seed = 0xC0BA_77FA_11;
    let battle_uuid = Uuid::from_u128(0xC0BA_77FA_11);
    let mut core = GameCore::new(game_data_with_pve_encounters(), run_seed);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let node = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("trace_failure_abno".to_string()),
        encounter_id: "trace_failure_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: Uuid::from_u128(0xC0BA_77FA_12),
        winner: BattleWinner::Player,
        outcome_id: fixture_outcome_id(BattleWinner::Player),
        verdict: fixture_verdict(BattleWinner::Player),
        grant_node_bonus: BattleWinner::Player == BattleWinner::Player,
        advance_encounter_progression: false,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Player, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };

    settle_test_combat_result(&mut core, battle_uuid, battle)
        .expect("diagnostic failure must not reject the gameplay transition");

    assert_eq!(core.get_state(), GameState::CombatResult { battle_uuid });
    assert!(core.state.active_battle.is_none());
    assert!(
        matches!(&core.state.confirmed_combat_result.as_ref().unwrap().summary,
        crate::game::behavior::SelectedEventSnapshotDto::CombatBattle { encounter_id, has_event_log: false, .. } if encounter_id == "trace_failure_encounter")
    );
    assert!(!trace_path.exists());

    std::fs::remove_file(trace_run_dir).expect("cleanup trace directory blocker");
}

#[test]
fn encounter_compendium_profile_merge_is_single_set_monotonic_and_catalog_resolved() {
    use std::collections::BTreeSet;

    use crate::game::encounter_compendium::EncounterCompendiumProfileState;

    let game_data = game_data_with_pve_encounters();
    let mut core = GameCore::new(game_data.clone(), 0xC011_EC70);
    core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
        registered_ids: BTreeSet::from([
            "defense_risk_abno".to_string(),
            "corroded_guard".to_string(),
        ]),
    })
    .unwrap();
    core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
        registered_ids: BTreeSet::from(["low_risk_abno".to_string()]),
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
    assert!(!undiscovered.registered);
    assert_eq!(undiscovered.name, None);
    assert_eq!(undiscovered.risk_level, None);
    assert_eq!(
        snapshot.encounter_compendium.corrosion_profiles[0].profile_id,
        "corroded_guard"
    );

    core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
        registered_ids: BTreeSet::from(["dormant_abnormality".to_string()]),
    })
    .unwrap();
    assert!(core
        .encounter_compendium_profile()
        .registered_ids
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
fn successful_opponent_spawn_populates_process_local_candidate_ledger() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC74);
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

    let active = core.state.active_battle.as_ref().expect("active battle");
    let candidates = active.battle.compendium_candidates();
    assert!(candidates.iter().any(|candidate| {
        candidate.id == "defense_risk_abno"
            && candidate.kind == CompendiumCandidateKind::Abnormality
    }));
    assert_eq!(
        active.clone().battle.compendium_candidates(),
        candidates,
        "same-process active-session cloning must retain the attempt ledger"
    );
    assert!(core
        .encounter_compendium_profile()
        .registered_ids
        .is_empty());
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
        .unwrap()
        .expect("not-started state should support a transaction rollback snapshot");
    core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
        registered_ids: std::collections::BTreeSet::from(["dormant_after_capture".to_string()]),
    })
    .unwrap();
    core.state.run_power_authority.power_model_version = 2;
    core.state.run_power_authority.run_peak_certified_power = 900;
    core.state.run_power_authority.current_floor_power_anchor = 800;

    core.restore_active_run_transaction_rollback(rollback);

    assert!(core
        .encounter_compendium_profile()
        .registered_ids
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
        registered_ids: std::collections::BTreeSet::from([
            "dormant_before_common_transaction".to_string()
        ]),
    })
    .unwrap();

    let error = core
        .with_state_transaction(|core| -> Result<(), crate::game::behavior::GameError> {
            core.merge_encounter_compendium_profile(EncounterCompendiumProfileState {
                registered_ids: std::collections::BTreeSet::from([
                    "dormant_during_common_transaction".to_string(),
                ]),
            })?;
            Err(crate::game::behavior::GameError::InvalidAction)
        })
        .unwrap_err();

    assert!(matches!(
        error,
        crate::game::behavior::GameError::InvalidAction
    ));
    assert_eq!(
        core.encounter_compendium_profile().registered_ids,
        std::collections::BTreeSet::from([
            "dormant_before_common_transaction".to_string(),
            "dormant_during_common_transaction".to_string(),
        ])
    );
}

#[test]
fn common_non_battle_transaction_rejects_active_battle_before_running_operation() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC73);
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
    let before = battle_advance_atomic_snapshot(&core);
    let mut operation_ran = false;

    let error = core
        .with_state_transaction(|_| -> Result<(), GameError> {
            operation_ran = true;
            Ok(())
        })
        .unwrap_err();

    assert!(matches!(error, GameError::InvalidAction));
    assert!(!operation_ran);
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
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
    assert_eq!(initial_anchor, core.run_policy().floor_power_anchor_policy(GameMode::Expedition,0).unwrap().authored_minimum_power);

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
fn durable_continue_recomputes_battle_setup_instead_of_trusting_serialized_preview_cache() {
    let mut game_data = game_data_with_pve_encounters();
    // Author a non-default setup scale before constructing the run. If Continue
    // reuses the deserialized presentation cache, the exact 110 oracle fails.
    let policy = Arc::make_mut(&mut Arc::get_mut(&mut game_data).unwrap().run_policy);
    for profile in &mut policy.combat_difficulty.profiles {
        for band in &mut profile.ordinary_bands {
            band.tuning.max_health_multiplier_percent = 110;
            band.tuning.attack_multiplier_percent = 110;
            band.tuning.defense_multiplier_percent = 110;
        }
    }
    let player_id = Uuid::from_u128(1);
    let mut source = GameCore::new(game_data, 0xC011_CAC4);
    start_new_game_with_default_starters(&mut source, player_id);
    let combat_id = force_map_combat_node(
        &mut source,
        MapNodeCategory::Combat,
        "combat_defense",
        "defense_encounter",
    );
    let preview = source
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode { node_id: combat_id },
        )
        .unwrap();
    let BehaviorResult::NodePreview {
        combat_preview: Some(authoritative),
        ..
    } = preview
    else {
        panic!("authored combat preview");
    };
    assert_eq!(authoritative.enemy_stat_scale.max_health_percent, 110);
    assert_eq!(authoritative.enemy_stat_scale.attack_percent, 110);
    assert_eq!(authoritative.enemy_stat_scale.defense_percent, 110);
    source
        .execute(player_id, PlayerBehavior::CancelSelectedNode)
        .unwrap();
    let saved = source.export_active_run_checkpoint().unwrap().unwrap();
    let document: Value = serde_json::from_slice(&saved.core_payload_json).unwrap();
    let persisted =
        &document["continuation"]["current"]["combat_previews"][combat_id.0.to_string()];
    assert!(persisted.is_object());
    assert!(persisted.get("enemy_stat_scale").is_none());
    let mut restored = core_with_profile_for_continue(&source, 0xBAD_C011);
    restored
        .continue_from_active_run_checkpoint(
            player_id,
            &saved.core_payload_json,
            saved.loads_used,
            saved.run_peak_certified_power,
        )
        .unwrap();
    assert_eq!(
        restored.state.run.as_ref().unwrap().combat_previews[&combat_id].enemy_stat_scale,
        Default::default()
    );
    restored
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode { node_id: combat_id },
        )
        .unwrap();
    restored
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let active = restored.state.active_battle.as_ref().unwrap();
    assert_eq!(active.combat_preview, authoritative);
    let enemy = active
        .battle
        .units
        .values()
        .find(|u| u.owner == Side::Opponent)
        .expect("time-zero authored enemy");
    assert_eq!(
        enemy.source_identity,
        crate::game::battle::types::BattleUnitSourceIdentity::Abnormality {
            base_uuid: Uuid::from_u128(20_005),
            abnormality_id: "defense_risk_abno".to_string()
        }
    );
}

#[test]
fn native_runtime_facts_preserve_actual_same_definition_inventory_copies_through_player_drafts() {
    use crate::game::battle::{core::BattleCore, scenario::*};
    use crate::game::combat_setup::player_spawns::player_scenario_start_from_positions;
    let mut core = GameCore::new(live_game_data_from_ron(), 0xCA04_007);
    start_new_game_with_default_starters(&mut core, Uuid::from_u128(1));
    let employees: Vec<_> = core.state.roster.iter().take(2).map(|e| e.uuid).collect();
    assert_eq!(employees.len(), 2);
    let mut expected = std::collections::BTreeMap::new();
    let mut expected_slots = std::collections::BTreeMap::new();
    let mut expected_layouts = std::collections::BTreeMap::new();
    for id in &employees {
        let item_slot = &core.state.roster.get(id).unwrap().loadout.item_slot;
        expected_layouts.insert(*id, item_slot.layout_kind());
        for (address, equipped) in item_slot.iter_with_address() {
            let owned = core
                .state
                .inventory
                .equipments
                .get_item(&equipped.instance_uuid)
                .unwrap();
            assert_eq!(owned.equipped_to, Some(*id));
            expected.insert(equipped.instance_uuid, equipped.base_uuid);
            expected_slots.insert(equipped.instance_uuid, address);
        }
    }
    assert!(expected
        .values()
        .any(|base| expected.values().filter(|other| *other == base).count() >= 2));
    let saved = serde_json::to_vec(&core.state.inventory).unwrap();
    let restored: crate::game::resources::Inventory = serde_json::from_slice(&saved).unwrap();
    for copy in expected.keys() {
        assert!(restored.equipments.get_item(copy).is_some());
    }
    let start = player_scenario_start_from_positions(
        &core.state.roster,
        &restored,
        &core.state.skill_fragments,
        core.game_data.as_ref(),
        employees
            .iter()
            .enumerate()
            .map(|(i, id)| (*id, Position::new(i as i32 + 1, 1)))
            .collect(),
    )
    .unwrap();
    for spawn in &start.spawn_group.spawns {
        assert_eq!(
            spawn.draft.equipment_layout,
            Some(expected_layouts[&spawn.draft.owned_uuid])
        );
        for binding in &spawn.draft.equipped_items {
            let copy = binding.persistent_copy_uuid.unwrap();
            assert_eq!(expected[&copy], binding.base_uuid);
            assert_eq!(binding.slot, Some(expected_slots[&copy]));
        }
    }
    let group_id = start.spawn_group.id.clone();
    let scenario = BattleScenario {
        battlefield: BattleFieldSpec {
            walkable_paths: Vec::new(),
            object_placement: None,
            width: 5,
            height: 5,
            valid_tiles: vec![],
            obstacles: vec![],
        },
        groups: vec![start.spawn_group],
        events: vec![ScenarioEvent {
            id: ScenarioEventId::new("native_copy_spawn"),
            trigger: ScenarioTrigger::AtBattleStart,
            action: ScenarioAction::SpawnGroup { group_id },
        }],
        terminal_outcomes: BattleScenario::fixture_survival_outcomes(10_000),
        tactical_plan: Default::default(),
        boss_runtime: None,
    };
    for trigger in [
        ScenarioTrigger::AtBattleStart,
        ScenarioTrigger::Wave(ScenarioWaveActivation {
            trigger_id: "native_copy_wave".into(),
            priority: 0,
            primary: ScenarioWavePrimaryTrigger::AtTimeMs { time_ms: 0 },
            deadline_ms: None,
        }),
    ] {
        let mut scenario = scenario.clone();
        scenario.events[0].trigger = trigger;
        let mut battle = BattleCore::new_from_scenario(scenario, core.game_data.clone(), 9);
        let mut execution = battle.start_battle_execution().unwrap();
        battle
            .step_battle_execution_until(&mut execution, 0)
            .unwrap();
        let observed: std::collections::BTreeMap<_, _> = battle
            .runtime_fact_view()
            .providers()
            .map(|(provider, _)| {
                (
                    provider.persistent_copy_uuid.unwrap(),
                    (provider.definition_uuid, provider.runtime_item_instance_id),
                )
            })
            .collect();
        assert_eq!(
            observed
                .iter()
                .map(|(copy, (definition, _))| (*copy, *definition))
                .collect::<std::collections::BTreeMap<_, _>>(),
            expected
        );
        let runtime_ids: std::collections::BTreeSet<_> = battle
            .runtime_fact_view()
            .providers()
            .map(|(provider, _)| provider.runtime_item_instance_id)
            .collect();
        assert_eq!(runtime_ids.len(), expected.len());
        let mut runtime_slots = std::collections::BTreeMap::new();
        for unit in battle.units.values() {
            let slots = unit
                .equipment_slots
                .as_deref()
                .expect("player projection must retain a known equipment layout");
            assert_eq!(slots.layout, expected_layouts[&unit.source_owned_uuid]);
            for entry in slots.slots.iter().flatten() {
                let copy = entry.binding.persistent_copy_uuid.unwrap();
                runtime_slots.insert(
                    copy,
                    (entry.binding.slot.unwrap(), entry.runtime_item_instance_id),
                );
            }
        }
        assert_eq!(
            runtime_slots
                .iter()
                .map(|(copy, (slot, _))| (*copy, *slot))
                .collect::<std::collections::BTreeMap<_, _>>(),
            expected_slots
        );
        for (copy, (_, runtime_id)) in &runtime_slots {
            assert_eq!(*runtime_id, observed[copy].1);
        }
        let snapshot = battle.runtime_fact_snapshot().unwrap();
        drop(battle);
        assert_eq!(snapshot.providers.len(), expected.len());
    }
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
    let equipped_instance_id = original_item_slot
        .iter()
        .find(|equipped| equipped.equipment_type == EquipmentType::Weapon)
        .unwrap()
        .instance_uuid;
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
fn consuming_victory_commits_all_attempt_candidates_across_checkpoint_and_new_run() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC72);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    core.save_run_checkpoint_for_test().unwrap();
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
            activation: crate::game::data::pve_data::PveWaveActivationData::at_time(
                "preview_only_unspawned_wave",
                99_999,
            ),
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
        committed.registered_ids,
        std::collections::BTreeSet::from([
            "defense_risk_abno".to_string(),
            "corroded_guard".to_string(),
        ])
    );
    assert!(!committed.registered_ids.contains("low_risk_abno"));

    core.handle_load_run_checkpoint().unwrap();
    assert_eq!(core.encounter_compendium_profile(), committed);

    core.execute(player_id, PlayerBehavior::AbandonRun).unwrap();
    core.execute(
        player_id,
        PlayerBehavior::StartNewGame {
            game_mode: GameMode::Expedition,
            replace_active_run: false,
        },
    )
    .unwrap();
    assert_eq!(core.encounter_compendium_profile(), committed);
}

#[test]
fn every_defeat_and_retreat_leave_registered_set_unchanged() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC73);
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
    install_encounter_compendium_combat_result(&mut core, node_id, BattleWinner::Opponent);

    let retry = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert!(matches!(retry, BehaviorResult::NodeCompleted { .. }));
    assert_eq!(
        core.encounter_compendium_profile(),
        crate::game::encounter_compendium::EncounterCompendiumProfileState::default()
    );

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
    core.merge_encounter_compendium_profile(
        crate::game::encounter_compendium::EncounterCompendiumProfileState {
            registered_ids: std::collections::BTreeSet::from(["low_risk_abno".to_string()]),
        },
    )
    .unwrap();
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(
        core.encounter_compendium_profile().registered_ids,
        std::collections::BTreeSet::from(["low_risk_abno".to_string()]),
        "battle-entry rollback must not replace newer game-global profile knowledge"
    );

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    install_encounter_compendium_combat_result(&mut core, node_id, BattleWinner::Opponent);
    let completed = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let BehaviorResult::NodeCompleted { .. } = completed else {
        panic!("exhausted defeat should consume the node and return to planning");
    };

    assert_eq!(
        core.encounter_compendium_profile().registered_ids,
        std::collections::BTreeSet::from(["low_risk_abno".to_string()])
    );
}

#[test]
fn combat_result_snapshot_exposes_typed_result_stats() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let battle_uuid = Uuid::from_u128(0xCAFE);
    let employee_uuid = Uuid::from_u128(0xE0);
    let unit_instance_id = crate::game::battle::ids::UnitInstanceId::from(Uuid::from_u128(0xE1));
    let mut event_log = BattleEventLog::default();
    event_log.entries.push(BattleEventLogEntry {
        skill_effect_source: None,
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("abno".to_string()),
        encounter_id: "encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: battle_uuid,
        winner: BattleWinner::Player,
        outcome_id: fixture_outcome_id(BattleWinner::Player),
        verdict: fixture_verdict(BattleWinner::Player),
        grant_node_bonus: BattleWinner::Player == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Player == BattleWinner::Player,
        event_log,
        result_stats,
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results,
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![CompendiumCandidate {
            id: "low_risk_abno".to_string(),
            kind: CompendiumCandidateKind::Abnormality,
            base_uuid: Uuid::from_u128(10_001),
        }],
    }));

    let selected = core
        .get_selected_event_snapshot_json()
        .unwrap()
        .expect("combat selected event");

    assert_eq!(selected["type"], "combat_battle");
    assert_eq!(selected["outcome_id"], "victory");
    assert_eq!(selected["verdict"], "victory");
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
    assert_eq!(selected["has_event_log"], true);
    assert!(selected.get("bonus_objectives").is_none());
}

#[test]
fn combat_terminal_settlement_failure_does_not_partially_apply_rewards_on_retry() {
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

    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
    let employee_uuid = employee_ids[0];
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("low_risk_abno".to_string()),
        encounter_id: "low_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Player,
        outcome_id: fixture_outcome_id(BattleWinner::Player),
        verdict: fixture_verdict(BattleWinner::Player),
        grant_node_bonus: BattleWinner::Player == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Player == BattleWinner::Player,
        result_stats: test_result_stats(BattleWinner::Player, &event_log, &participant_results),
        event_log,
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
        compendium_candidates: vec![],
    };

    core.state
        .run
        .as_mut()
        .unwrap()
        .map_progression
        .current_node_id = None;
    let grant_roots_before = serialized_grant_roots(&core);

    for _ in 0..2 {
        let err = settle_test_combat_result(&mut core, node_id.0, battle.clone()).unwrap_err();
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
        assert!(matches!(core.get_state(), GameState::InBattle { .. }));
        assert_eq!(serialized_grant_roots(&core), grant_roots_before);
        assert!(core
            .encounter_compendium_profile()
            .registered_ids
            .is_empty());
    }
}

#[test]
fn normal_boss_direct_victory_uses_exclusive_first_and_repeat_source_branches() {
    let player_id = Uuid::from_u128(1);
    let fragment_id =
        crate::game::data::skill_fragment_data::SkillFragmentId::from("fragment_boss_risk_abno");

    let mut first = GameCore::new(game_data_with_pve_encounters(), 0xB055_0001);
    start_new_game_with_mode_and_default_starters(&mut first, player_id, GameMode::Expedition);
    let first_node = force_map_combat_node(
        &mut first,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "boss_risk_encounter",
    );
    first
        .state
        .run
        .as_mut()
        .unwrap()
        .normal_boss_cycle
        .assigned_source_ids
        .push("source_boss_risk".to_string());
    first
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: first_node,
            },
        )
        .unwrap();
    first
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    install_victorious_source_combat_result(
        &mut first,
        first_node,
        "boss_risk_encounter",
        "boss_risk_abno",
    )
    .unwrap();
    first
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let first_run = first.state.run.as_ref().unwrap();
    assert_eq!(first.state.skill_fragments.count(&fragment_id), 1);
    assert_eq!(first.state.skill_fragments.fragment_dust(), 0);
    assert!(first_run
        .completed_reward_source_ids
        .contains("source_boss_risk"));
    assert!(first_run.normal_boss_cycle.assigned_source_ids.is_empty());

    let mut repeat = GameCore::new(game_data_with_pve_encounters(), 0xB055_0002);
    start_new_game_with_mode_and_default_starters(&mut repeat, player_id, GameMode::Expedition);
    let repeat_node = force_map_combat_node(
        &mut repeat,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "boss_risk_encounter",
    );
    {
        let run = repeat.state.run.as_mut().unwrap();
        run.completed_reward_source_ids
            .insert("source_boss_risk".to_string());
        run.normal_boss_cycle
            .assigned_source_ids
            .push("source_boss_risk".to_string());
    }
    repeat
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: repeat_node,
            },
        )
        .unwrap();
    repeat
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    install_victorious_source_combat_result(
        &mut repeat,
        repeat_node,
        "boss_risk_encounter",
        "boss_risk_abno",
    )
    .unwrap();
    repeat
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    let repeat_run = repeat.state.run.as_ref().unwrap();
    assert_eq!(repeat.state.skill_fragments.count(&fragment_id), 0);
    assert_eq!(repeat.state.skill_fragments.fragment_dust(), 0);
    assert_eq!(repeat.state.crafting_resources.equipment_dust, 10);
    assert!(repeat_run
        .completed_reward_source_ids
        .contains("source_boss_risk"));
    assert!(repeat_run.normal_boss_cycle.assigned_source_ids.is_empty());
}

#[test]
fn normal_boss_repeat_reward_failure_rolls_back_cycle_ledger_and_result_state() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xB055_0003);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "boss_risk_encounter",
    );
    {
        let run = core.state.run.as_mut().unwrap();
        run.completed_reward_source_ids
            .insert("source_boss_risk".to_string());
        run.normal_boss_cycle
            .assigned_source_ids
            .push("source_boss_risk".to_string());
    }
    core.apply_grant_effects(&[RewardEffect::GrantEquipmentDust { amount: u32::MAX }]).unwrap();
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let state_before = core.get_run_snapshot_json().unwrap();
    let error = install_victorious_source_combat_result(
        &mut core,
        node_id,
        "boss_risk_encounter",
        "boss_risk_abno",
    )
    .unwrap_err();
    assert!(matches!(error, GameError::InvalidAction));
    assert_eq!(core.get_run_snapshot_json().unwrap(), state_before);
    assert_eq!(core.state.crafting_resources.equipment_dust, u32::MAX);
    assert!(matches!(core.get_state(), GameState::InBattle { .. }));
    let run = core.state.run.as_ref().unwrap();
    assert!(run.completed_reward_source_ids.contains("source_boss_risk"));
    assert_eq!(
        run.normal_boss_cycle.assigned_source_ids,
        ["source_boss_risk"]
    );
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
        skill_effect_source: None,
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
    let event_log = BattleEventLog::from_entries(vec![spawn(1, first_unit), spawn(2, second_unit)]);
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("low_risk_abno".to_string()),
        encounter_id: "low_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Player,
        outcome_id: fixture_outcome_id(BattleWinner::Player),
        verdict: fixture_verdict(BattleWinner::Player),
        grant_node_bonus: BattleWinner::Player == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Player == BattleWinner::Player,
        event_log,
        result_stats,
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results,
        employee_conditions,
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();
    let inventory_before = serde_json::to_value(&core.state.inventory).unwrap();
    let skill_fragments_before = serde_json::to_value(&core.state.skill_fragments).unwrap();
    let uuid_manager_before = serde_json::to_value(&core.state.uuid_manager).unwrap();
    let enkephalin_before = serde_json::to_value(&core.state.enkephalin).unwrap();

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
    assert_eq!(
        serde_json::to_value(&core.state.inventory).unwrap(),
        inventory_before
    );
    assert_eq!(
        serde_json::to_value(&core.state.skill_fragments).unwrap(),
        skill_fragments_before
    );
    assert_eq!(
        serde_json::to_value(&core.state.uuid_manager).unwrap(),
        uuid_manager_before
    );
    assert_eq!(
        serde_json::to_value(&core.state.enkephalin).unwrap(),
        enkephalin_before
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
    let event_log = BattleEventLog::from_entries(vec![BattleEventLogEntry {
        skill_effect_source: None,
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
    }]);
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("low_risk_abno".to_string()),
        encounter_id: "low_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        event_log,
        result_stats,
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results,
        employee_conditions,
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    let employee = core.state.roster.get(&employee_uuid).unwrap();
    assert_eq!(
        employee.condition.trauma,
        run_policy().post_battle.incapacitation_trauma
    );
    assert_eq!(employee.condition.physical_damage, 1);
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
    assert_eq!(
        core.get_action_capabilities(),
        vec![
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::StartNewGame,
                crate::game::behavior::ActionScope::Gameplay
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicState,
                crate::game::behavior::ActionScope::Query
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicNumericPlan,
                crate::game::behavior::ActionScope::Query
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicEffectivePlan,
                crate::game::behavior::ActionScope::Query
            ),
        ]
    );
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
    start_new_game_with_legacy_sword_fixture(&mut core, player_id);
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
    assert_eq!(player_unit.hud.resonance_current, Some(20));
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

    // The diagnostic log has a finite history window. Prove initialization
    // events while they are current, then independently collect each actual
    // bucket's new entries so the terminal export can be checked as a suffix.
    let history_limits = core.game_data.runtime_safety().battle_history();
    let mut observed_entries = core.state.active_battle.as_ref().unwrap()
        .battle.event_log.entries.iter().cloned().collect::<Vec<_>>();
    assert!(observed_entries.iter().any(|entry|
        matches!(entry.event, BattleLogEvent::BattleStart { .. })));
    assert!(observed_entries.iter().any(|entry|
        matches!(entry.event, BattleLogEvent::UnitSpawned { .. })));

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
        let after_seq = observed_entries.last().map_or(0, |entry| entry.seq);
        observed_entries.extend(core.state.active_battle.as_ref().unwrap()
            .battle.event_log.entries.iter().filter(|entry| entry.seq > after_seq).cloned());
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

    let trace_path =
        crate::game::world::combat_trace::debug_combat_trace_path(core.run_seed, battle_uuid);
    let trace: Value = serde_json::from_slice(&std::fs::read(&trace_path).unwrap()).unwrap();
    assert_eq!(
        trace["node_type"],
        json!(crate::game::combat_preview::CombatNodeType::Defense)
    );
    let event_log: BattleEventLog = serde_json::from_value(trace["event_log"].clone()).unwrap();
    assert_eq!(
        event_log.version,
        crate::game::battle::event_log::BATTLE_EVENT_LOG_VERSION
    );
    assert!(matches!(event_log.entries.last().unwrap().event, BattleLogEvent::BattleEnd { .. }));
    let after_seq = observed_entries.last().map_or(0, |entry| entry.seq);
    // The final bucket has already transferred out of active_battle. Its newly
    // committed tail is read from the export; every preceding bucket is the
    // independently collected runtime witness above.
    observed_entries.extend(event_log.entries.iter().filter(|entry| entry.seq > after_seq).cloned());
    assert!(observed_entries.windows(2).all(|pair| pair[1].seq == pair[0].seq + 1),
        "sampling each actual bucket must not skip an event");
    assert!(observed_entries.iter().any(|entry| matches!(entry.event, BattleLogEvent::BattleStart { .. })));
    assert!(observed_entries.iter().any(|entry| matches!(entry.event, BattleLogEvent::UnitSpawned { .. })));
    assert!(matches!(observed_entries.last().unwrap().event, BattleLogEvent::BattleEnd { .. }));
    let mut first_kept = 0;
    loop {
        let suffix = &observed_entries[first_kept..];
        let first_ms = suffix.iter().map(|entry| entry.time_ms).min().unwrap();
        let last_ms = suffix.iter().map(|entry| entry.time_ms).max().unwrap();
        if suffix.len() <= history_limits.max_events
            && serde_json::to_vec(suffix).unwrap().len() <= history_limits.max_encoded_entry_array_bytes
            && last_ms - first_ms <= history_limits.max_span_ms {
            break;
        }
        first_kept += 1;
        assert!(first_kept < observed_entries.len(), "one terminal event fits the official history policy");
    }
    assert_eq!(serde_json::to_value(&event_log.entries).unwrap(),
        serde_json::to_value(&observed_entries[first_kept..]).unwrap(),
        "debug export must be exactly the retained suffix under all three current limits");
    let path = write_world_debug_event_log_export("defense_combat_node_smoke", &event_log);
    assert!(path.exists());
    assert!(core.state.active_node_content.is_none());
    assert!(core.state.confirmed_combat_result.is_some());
    let trace_path =
        crate::game::world::combat_trace::debug_combat_trace_path(core.run_seed, battle_uuid);
    assert!(trace_path.exists());
    std::fs::remove_file(&trace_path).expect("cleanup smoke combat trace file");
}

#[test]
fn live_ron_defense_route_playable_path_runs_to_combat_result() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0001);
    let player_id = Uuid::from_u128(0xD3F3);
    start_new_game_with_legacy_sword_fixture(&mut core, player_id);
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
    let mut saw_spawn = match &result {
        BehaviorResult::BattleAdvanced { battle_update, .. } => battle_update.events_delta.events.iter()
            .any(|entry| matches!(entry.event, LiveBattlePresentationEventKindDto::UnitSpawned { .. })),
        _ => false,
    };
    deploy_positions.retain(|position| {
        core.state.active_battle.as_ref().is_some_and(|active| {
            active
                .battle
                .player_tactical_slot_occupant(*position, None)
                .is_none()
        })
    });
    assert!(
        deploy_positions.len() >= 2,
        "live defense should expose multiple unoccupied ground deployment cells"
    );
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
        saw_spawn |= battle_update.events_delta.events.iter()
            .any(|entry| matches!(entry.event, LiveBattlePresentationEventKindDto::UnitSpawned { .. }));
        if !battle_finished {
            if let (Some(employee_uuid), Some(position), Some(deployment)) = (
                third_employee,
                third_position,
                battle_update.checkpoint.deployment.as_ref(),
            ) {
                let deploy_cost = deployment
                    .unit_deploy_costs
                    .iter()
                    .find(|cost| cost.employee_uuid == employee_uuid)
                    .map(|cost| cost.effective_deploy_cost)
                    .unwrap_or(u32::MAX);
                if battle_update.checkpoint.tactical_resource.current >= deploy_cost {
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
    let trace_path =
        crate::game::world::combat_trace::debug_combat_trace_path(core.run_seed, battle_uuid);
    let trace: Value = serde_json::from_slice(&std::fs::read(&trace_path).unwrap()).unwrap();
    let event_log: BattleEventLog = serde_json::from_value(trace["event_log"].clone()).unwrap();
    assert!(event_log
        .entries
        .iter()
        .any(|entry| matches!(entry.event, BattleLogEvent::BattleEnd { .. })));
    assert!(saw_spawn, "the actual public stream must contain a spawned unit");
    let combat_node_uuid: Uuid = serde_json::from_value(trace["combat_node_uuid"].clone()).unwrap();
    let event_log_entry_count = event_log.entries.len();
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
    assert_eq!(setup.setup_version, 2);
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
    assert_eq!(
        core.get_action_capabilities(),
        vec![
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicState,
                crate::game::behavior::ActionScope::Query
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicNumericPlan,
                crate::game::behavior::ActionScope::Query
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicEffectivePlan,
                crate::game::behavior::ActionScope::Query
            ),
        ]
    );
    let snapshot = core.get_run_snapshot_json().unwrap();
    assert_eq!(snapshot["skill_catalog"]["version"], 3);
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
        battle_update.checkpoint.tactical_resource.current,
        run_policy().tactical_resource.initial
    );
    assert_eq!(
        battle_update.checkpoint.tactical_resource.maximum,
        run_policy().tactical_resource.maximum
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

#[derive(Debug, PartialEq, Eq)]
pub(super) struct BattleAdvanceAtomicSnapshot {
    run_seed: u64,
    run_payload_json: String,
    run_checkpoint_json: String,
    player_profile_json: String,
    battle_time_ms: u64,
    terminal_outcome: Option<crate::game::battle::core::sim::BattleFinishSignal>,
    finalized: bool,
    event_log_json: String,
    event_queue: Vec<String>,
    battle_core_state: String,
    runtime_facts: String,
    battle_state_revision: u64,
    employee_attempt: String,
    live_deployment: String,
    tactical_resource: String,
    playback: crate::game::behavior::BattlePlaybackState,
    playback_delta_remainder: u64,
    tactical_focus: Option<crate::game::behavior::LiveBattleTacticalFocusDto>,
    interaction: crate::game::behavior::LiveBattleInteractionDto,
    visual_novel_json: String,
    visual_novel_triggers_json: String,
    uuid_manager_json: String,
    game_state: String,
    active_node_content: String,
    active_battle_present: bool,
}

pub(super) fn battle_advance_atomic_snapshot(core: &GameCore) -> BattleAdvanceAtomicSnapshot {
    let active = core
        .state
        .active_battle
        .as_ref()
        .expect("atomic snapshot requires an active battle");
    BattleAdvanceAtomicSnapshot {
        run_seed: core.run_seed,
        run_payload_json: serde_json::to_string(&core.state.run.as_ref().map(|run| {
            crate::game::world::state::RunCheckpointPayload::capture(
                &core.state,
                run,
                core.game_data.identity(),
            )
            .unwrap()
        }))
        .unwrap(),
        run_checkpoint_json: serde_json::to_string(&core.state.run_checkpoint).unwrap(),
        player_profile_json: serde_json::to_string(&core.state.player_profile).unwrap(),
        battle_time_ms: active.execution.last_event_time_ms(),
        terminal_outcome: active.execution.terminal_outcome(),
        finalized: active.execution.is_finalized(),
        event_log_json: active.battle.event_log.to_json_string().unwrap(),
        event_queue: active.battle.event_queue_debug_snapshot(),
        battle_core_state: active.battle.transaction_debug_snapshot(),
        runtime_facts: serde_json::to_string(&active.battle.runtime_fact_view()).unwrap(),
        battle_state_revision: active.battle_state_revision,
        employee_attempt: format!("{:?}", active.employee_attempt),
        live_deployment: format!("{:?}", active.live_deployment),
        tactical_resource: format!("{:?}", active.tactical_resource),
        playback: active.playback,
        playback_delta_remainder: active.playback_delta_remainder,
        tactical_focus: active.tactical_focus_dto(),
        interaction: active.interaction_dto(),
        visual_novel_json: serde_json::to_string(&core.state.visual_novel).unwrap(),
        visual_novel_triggers_json: serde_json::to_string(&core.state.visual_novel_triggers)
            .unwrap(),
        uuid_manager_json: serde_json::to_string(&core.state.uuid_manager).unwrap(),
        game_state: format!("{:?}", core.state.lifecycle.phase()),
        active_node_content: format!("{:?}", core.state.active_node_content),
        active_battle_present: core.state.active_battle.is_some(),
    }
}

struct TacticalWorldMutationFixture {
    core: GameCore,
    player_id: Uuid,
    first_employee_uuid: Uuid,
    second_employee_uuid: Uuid,
    first_position: Position,
    second_position: Position,
}

fn tactical_world_mutation_fixture(seed: u64) -> TacticalWorldMutationFixture {
    let mut core = GameCore::new(game_data_with_pve_encounters(), seed);
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
    let first_position = ground_deployment_cell_with_right_range(&preview);
    let employees = core.roster().unwrap().combat_eligible_employee_ids();
    let first_employee_uuid = employees[0];
    let second_employee_uuid = employees[1];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let second_position = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .combat_preview
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .find(|position| *position != first_position)
        .expect("atomic tactical fixture must expose two ground deployment cells");
    TacticalWorldMutationFixture {
        core,
        player_id,
        first_employee_uuid,
        second_employee_uuid,
        first_position,
        second_position,
    }
}

fn execute_current_battle_mutation_after(
    core: &mut GameCore,
    player_id: Uuid,
    behavior: PlayerBehavior,
    after_seq: u64,
) -> Result<BehaviorResult, GameError> {
    let active = core
        .state
        .active_battle
        .as_ref()
        .expect("battle mutation test requires active battle");
    let authority = crate::game::behavior::BattleStateAuthorityDto {
        battle_uuid: active.battle_uuid,
        battle_state_revision: active.battle_state_revision,
    };
    core.execute_with_transport_context(player_id, behavior, None, after_seq, Some(authority))
}

fn assert_future_cursor_rejection_is_atomic(
    core: &mut GameCore,
    player_id: Uuid,
    behavior: PlayerBehavior,
) {
    let future_seq = u64::MAX;
    let before = battle_advance_atomic_snapshot(core);
    let retry_behavior = behavior.clone();
    let error = execute_current_battle_mutation_after(core, player_id, behavior, future_seq)
        .expect_err("a future event cursor must reject the tactical mutation");
    assert!(matches!(
        error,
        GameError::InvalidBattleResyncSeq {
            requested,
            latest,
        } if requested == future_seq && latest < future_seq
    ));
    assert_eq!(battle_advance_atomic_snapshot(core), before);

    let retry_error =
        execute_current_battle_mutation_after(core, player_id, retry_behavior, future_seq)
            .expect_err("retry from the same basis must reject identically");
    assert_eq!(format!("{retry_error:?}"), format!("{error:?}"));
    assert_eq!(battle_advance_atomic_snapshot(core), before);
}

#[test]
fn ca003_playback_commands_reject_future_cursor_atomically_and_retry_identically() {
    let TacticalWorldMutationFixture {
        mut core,
        player_id,
        ..
    } = tactical_world_mutation_fixture(0xCA03_0001);

    assert_future_cursor_rejection_is_atomic(&mut core, player_id, PlayerBehavior::PauseBattle);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();

    assert_future_cursor_rejection_is_atomic(&mut core, player_id, PlayerBehavior::ResumeBattle);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle).unwrap();

    assert_future_cursor_rejection_is_atomic(
        &mut core,
        player_id,
        PlayerBehavior::SetBattleSpeed {
            speed: crate::game::behavior::BattlePlaybackSpeed::X2,
        },
    );
}

#[test]
fn tactical_world_future_cursor_is_atomic_for_deploy() {
    let TacticalWorldMutationFixture {
        mut core,
        player_id,
        first_employee_uuid,
        first_position,
        ..
    } = tactical_world_mutation_fixture(0xA701_C020);
    assert_future_cursor_rejection_is_atomic(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: first_employee_uuid,
            position: first_position,
            facing: FacingDirection::Right,
        },
    );
}

#[test]
fn tactical_world_future_cursor_is_atomic_for_withdraw() {
    let TacticalWorldMutationFixture {
        mut core,
        player_id,
        first_employee_uuid,
        first_position,
        ..
    } = tactical_world_mutation_fixture(0xA701_C021);
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: first_employee_uuid,
            position: first_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    assert_future_cursor_rejection_is_atomic(
        &mut core,
        player_id,
        PlayerBehavior::WithdrawUnit {
            employee_uuid: first_employee_uuid,
        },
    );
}

#[test]
fn tactical_world_future_cursor_is_atomic_for_stance() {
    let TacticalWorldMutationFixture {
        mut core,
        player_id,
        first_employee_uuid,
        first_position,
        ..
    } = tactical_world_mutation_fixture(0xA701_C022);
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: first_employee_uuid,
            position: first_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    core.advance_active_battle_by(1_500).unwrap();
    assert_future_cursor_rejection_is_atomic(
        &mut core,
        player_id,
        PlayerBehavior::SetEngagementStance {
            employee_uuid: first_employee_uuid,
            stance: EngagementStance::Defense,
        },
    );
}

#[test]
fn tactical_world_future_cursor_is_atomic_for_relocation() {
    let TacticalWorldMutationFixture {
        mut core,
        player_id,
        first_employee_uuid,
        first_position,
        second_position,
        ..
    } = tactical_world_mutation_fixture(0xA701_C023);
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: first_employee_uuid,
            position: first_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    core.advance_active_battle_by(1_500).unwrap();
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::SetEngagementStance {
            employee_uuid: first_employee_uuid,
            stance: EngagementStance::Defense,
        },
    )
    .unwrap();
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .tactical_resource
        .set_current_for_test(10);
    assert_future_cursor_rejection_is_atomic(
        &mut core,
        player_id,
        PlayerBehavior::TacticalRelocateUnit {
            employee_uuid: first_employee_uuid,
            destination: second_position,
            facing: FacingDirection::Left,
        },
    );
}

#[test]
fn tactical_world_future_cursor_is_atomic_for_swap() {
    let TacticalWorldMutationFixture {
        mut core,
        player_id,
        first_employee_uuid,
        second_employee_uuid,
        first_position,
        second_position,
    } = tactical_world_mutation_fixture(0xA701_C024);
    for (employee_uuid, position, facing) in [
        (first_employee_uuid, first_position, FacingDirection::Right),
        (second_employee_uuid, second_position, FacingDirection::Left),
    ] {
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position,
                facing,
            },
        )
        .unwrap();
    }
    core.advance_active_battle_by(1_500).unwrap();
    for employee_uuid in [first_employee_uuid, second_employee_uuid] {
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::SetEngagementStance {
                employee_uuid,
                stance: EngagementStance::Defense,
            },
        )
        .unwrap();
    }
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .tactical_resource
        .set_current_for_test(10);
    assert_future_cursor_rejection_is_atomic(
        &mut core,
        player_id,
        PlayerBehavior::TacticalSwapUnits {
            first_employee_uuid,
            second_employee_uuid,
            first_facing: FacingDirection::Up,
            second_facing: FacingDirection::Down,
        },
    );
}

#[test]
fn tactical_world_revision_overflow_rejects_before_mutation() {
    let TacticalWorldMutationFixture {
        mut core,
        player_id,
        first_employee_uuid,
        first_position,
        ..
    } = tactical_world_mutation_fixture(0xA701_C026);
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .battle_state_revision = u64::MAX;
    let before = battle_advance_atomic_snapshot(&core);
    let error = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: first_employee_uuid,
            position: first_position,
            facing: FacingDirection::Right,
        },
    )
    .expect_err("revision overflow must reject before tactical mutation");
    assert!(matches!(error, GameError::BattleStateRevisionOverflow));
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
}

#[test]
fn tactical_world_late_employee_reconciliation_failure_is_atomic() {
    let TacticalWorldMutationFixture {
        mut core,
        player_id,
        first_employee_uuid,
        second_employee_uuid,
        first_position,
        second_position,
    } = tactical_world_mutation_fixture(0xA701_C025);
    let first_deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: first_employee_uuid,
            position: first_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: first_unit_instance_id,
        ..
    } = first_deployed
    else {
        panic!("expected first deployment");
    };
    let second_deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: second_employee_uuid,
            position: second_position,
            facing: FacingDirection::Left,
        },
    )
    .unwrap();
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: second_unit_instance_id,
        ..
    } = second_deployed
    else {
        panic!("expected second deployment");
    };
    {
        let active = core.state.active_battle.as_mut().unwrap();
        assert!(active
            .battle
            .units
            .get(&first_unit_instance_id)
            .unwrap()
            .is_active());
        active
            .battle
            .units
            .get_mut(&second_unit_instance_id)
            .unwrap()
            .lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
        active.employee_attempt.roster.clear();
    }

    let before = battle_advance_atomic_snapshot(&core);
    let error = core
        .handle_withdraw_unit(first_employee_uuid, None, 0)
        .expect_err("missing employee attempt state must fail at real reconciliation");
    assert!(matches!(error, GameError::BattleRuntimeFault(fault)
        if fault.code == crate::game::battle::runtime_control::RuntimeFaultCode::Invariant
            && fault.site == "reconcile_incapacitated_roster"));
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
}

#[derive(Debug, PartialEq, Eq)]
struct BattleStartAtomicSnapshot {
    game_state: String,
    active_node_content: String,
    node_session: String,
    run_snapshot: String,
    inventory_snapshot: String,
    roster_snapshot: String,
    uuid_manager: String,
    enkephalin: String,
    visual_novel: String,
    visual_novel_triggers: String,
    active_battle_present: bool,
}

fn battle_start_atomic_snapshot(core: &GameCore) -> BattleStartAtomicSnapshot {
    BattleStartAtomicSnapshot {
        game_state: format!("{:?}", core.state.lifecycle.phase()),
        active_node_content: format!("{:?}", core.state.active_node_content),
        node_session: format!("{:?}", core.state.node_session),
        run_snapshot: serde_json::to_string(&core.get_run_snapshot_json().unwrap()).unwrap(),
        inventory_snapshot: serde_json::to_string(&core.get_inventory_snapshot_json().unwrap())
            .unwrap(),
        roster_snapshot: serde_json::to_string(&core.get_employee_roster_snapshot_json().unwrap())
            .unwrap(),
        uuid_manager: serde_json::to_string(&core.state.uuid_manager).unwrap(),
        enkephalin: format!("{:?}", core.state.enkephalin),
        visual_novel: serde_json::to_string(&core.state.visual_novel).unwrap(),
        visual_novel_triggers: serde_json::to_string(&core.state.visual_novel_triggers).unwrap(),
        active_battle_present: core.state.active_battle.is_some(),
    }
}

fn game_data_with_track_and_rejoin_fixture(with_boss_binding: bool) -> Arc<GameDataBase> {
    let mut game_data = game_data_with_pve_encounters();
    let database = Arc::get_mut(&mut game_data)
        .expect("fixture game data should be uniquely owned before GameCore construction");
    if with_boss_binding {
        database.boss_runtime_data = Arc::new(
            crate::game::data::boss_runtime_data::tests::valid_catalog()
                .compile()
                .expect("TrackAndRejoin fixture catalog should compile"),
        );
    }
    let abnormalities = Arc::make_mut(&mut database.abnormality_data);
    let actor = abnormalities
        .items
        .iter_mut()
        .find(|item| item.id == "defense_route_abno")
        .expect("TrackAndRejoin fixture actor should exist");
    actor.max_health = 100_000;
    actor.attack = 0;
    let pve = Arc::make_mut(&mut database.pve_data);
    let encounter = pve
        .encounters
        .iter_mut()
        .find(|encounter| encounter.id == "defense_route_encounter")
        .expect("TrackAndRejoin fixture encounter should exist");
    let trigger_id = encounter.waves[0].activation.trigger_id.clone();
    encounter.waves[0].activation =
        crate::game::data::pve_data::PveWaveActivationData::at_time(trigger_id, 10);
    encounter.boss_runtime =
        with_boss_binding.then(|| crate::game::data::pve_data::PveBossRuntimeBindingData {
            encounter_id: crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new(
                "tracking_encounter",
            ),
            actors: vec![crate::game::data::pve_data::PveBossActorBindingData {
                actor_slot_id: crate::game::data::boss_runtime_data::BossActorSlotId::new(
                    "primary",
                ),
                wave_id: "wave_0".to_string(),
                spawn_index: 0,
            }],
        });
    game_data
}

fn game_data_with_raw_track_and_rejoin_source_fixture() -> Arc<GameDataBase> {
    let boss_source =
        ron::ser::to_string(&crate::game::data::boss_runtime_data::tests::valid_catalog())
            .expect("TrackAndRejoin raw Boss catalog should serialize");
    let mut pve = crate::game::data::live_ron::parse_live_ron::<PveEncounterDatabase>(
        "data/pve/encounters.ron",
        "pve_encounters",
        crate::game::data::live_ron::GameplayRonSourceSet::embedded().pve_encounters,
    )
    .expect("embedded PvE source should parse before fixture mutation");
    pve.encounters
        .retain(|encounter| encounter.id != "reference_cwbs_final_boss");
    for live_encounter in &mut pve.encounters {
        live_encounter.boss_runtime = None;
    }
    let encounter = pve
        .encounters
        .iter_mut()
        .find(|encounter| encounter.id == "suppress_nameless_fetus")
        .expect("raw source fixture encounter should exist");
    let actor_wave = encounter
        .waves
        .iter_mut()
        .find(|wave| wave.id == "primary_abnormality")
        .expect("raw source fixture actor wave should exist");
    let trigger_id = actor_wave.activation.trigger_id.clone();
    actor_wave.activation =
        crate::game::data::pve_data::PveWaveActivationData::at_time(trigger_id, 10);
    encounter.boss_runtime = Some(crate::game::data::pve_data::PveBossRuntimeBindingData {
        encounter_id: crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new(
            "tracking_encounter",
        ),
        actors: vec![crate::game::data::pve_data::PveBossActorBindingData {
            actor_slot_id: crate::game::data::boss_runtime_data::BossActorSlotId::new("primary"),
            wave_id: actor_wave.id.clone(),
            spawn_index: 0,
        }],
    });
    let pve_source =
        ron::ser::to_string(&pve).expect("TrackAndRejoin raw PvE binding should serialize");
    let mut sources = crate::game::data::live_ron::GameplayRonSourceSet::embedded();
    let mut summoned: crate::game::data::summoned_unit_data::SummonedUnitCatalogData =
        ron::de::from_str(sources.summoned_units).unwrap();
    summoned.definitions.clear(); // retain official non-boss object references
    let summoned_source = ron::ser::to_string(&summoned).unwrap();
    sources.boss_runtime = &boss_source;
    sources.summoned_units = &summoned_source;
    sources.pve_encounters = &pve_source;
    GameDataBase::load_live_sources(sources)
        .expect("whole raw TrackAndRejoin source candidate should publish atomically")
        .into_fixture_database_arc()
}

fn start_track_and_rejoin_fixture_at_approach(
    game_data: Arc<GameDataBase>,
    seed: u64,
) -> (
    GameCore,
    Uuid,
    Uuid,
    crate::game::battle::ids::UnitInstanceId,
) {
    let mut core = GameCore::new(game_data, seed);
    let player_id = Uuid::from_u128(seed as u128);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "defense_route_encounter",
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .expect("TrackAndRejoin fixture preview should resolve");
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("TrackAndRejoin fixture should enter live battle");
    let deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect("player should deploy before the Boss actor wave");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: player_unit_id,
        ..
    } = deployed
    else {
        panic!("expected player deployment result");
    };
    core.advance_active_battle_by(10)
        .expect("late Boss actor wave should advance");
    let actor = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()
        .into_iter()
        .next()
        .expect("Boss actor runtime snapshot at approach");
    assert_eq!(actor.state_id.as_deref(), Some("approach"));
    assert_eq!(actor.target_id, Some(player_unit_id));
    (core, player_id, employee_uuid, player_unit_id)
}

#[test]
fn track_and_rejoin_raw_live_bundle_reaches_typed_world_setup_identity() {
    let mut core = GameCore::new(
        game_data_with_raw_track_and_rejoin_source_fixture(),
        0xB055_0400,
    );
    let player_id = Uuid::from_u128(0xB055_0400);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_nameless_fetus",
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .expect("raw TrackAndRejoin source fixture preview should resolve");
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids().into_iter()
        .find(|id| core.roster().unwrap().get(id).unwrap().employee_definition_id == "records_clerk")
        .expect("the selected sword dealer deploys on Ground");
    let started = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("raw TrackAndRejoin source fixture should enter live battle");
    assert!(
        !serde_json::to_string(&started)
            .unwrap()
            .contains("boss_runtime"),
        "raw source publication must not widen the public battle shape"
    );
    let deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect("player should deploy before the raw authored actor wave");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: player_unit_id,
        ..
    } = deployed
    else {
        panic!("expected raw source fixture player deployment");
    };
    let spawned = core
        .advance_active_battle_by(10)
        .expect("raw authored Boss actor wave should advance");
    let BehaviorResult::BattleAdvanced {
        battle_update,
        finished: false,
        ..
    } = spawned
    else {
        panic!("expected active raw source fixture after actor spawn");
    };
    let actor = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()
        .into_iter()
        .next()
        .expect("raw binding should create one typed Boss actor runtime");
    let actor_id = actor
        .unit_id
        .expect("raw actor binding should resolve exactly");
    assert_eq!(actor.pattern_id.as_deref(), Some("track_and_rejoin"));
    assert_eq!(actor.state_id.as_deref(), Some("approach"));
    assert_eq!(actor.target_id, Some(player_unit_id));
    assert!(battle_update
        .events_delta
        .events
        .iter()
        .any(|entry| matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::UnitSpawned {
                unit_instance_id,
                owner: Side::Opponent,
                ..
            } if unit_instance_id == actor_id
        )));
    assert!(!serde_json::to_string(&battle_update)
        .unwrap()
        .contains("boss_runtime"));
}

#[test]
fn track_and_rejoin_fixture_runs_from_world_setup_through_reconnect_and_route_combat() {
    let mut core = GameCore::new(game_data_with_track_and_rejoin_fixture(true), 0xB055_0401);
    let player_id = Uuid::from_u128(0xB055_0401);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "defense_route_encounter",
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .expect("TrackAndRejoin fixture preview should resolve");
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let started = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("TrackAndRejoin fixture should enter live battle");
    let BehaviorResult::BattleAdvanced {
        battle_setup_snapshot: Some(setup),
        battle_update: initial_update,
        finished: false,
        ..
    } = started
    else {
        panic!("expected TrackAndRejoin battle setup and initial update");
    };
    assert!(!serde_json::to_string(&setup)
        .unwrap()
        .contains("boss_runtime"));
    assert!(!serde_json::to_string(&initial_update)
        .unwrap()
        .contains("boss_runtime"));
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0]
            .unit_id,
        None,
        "encounter runtime should exist before the authored late actor spawn"
    );

    let deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect("player should deploy before the Boss actor wave");
    let BehaviorResult::BattleUnitDeployed { battle_update, .. } = deployed else {
        panic!("expected deployment update");
    };
    let player_unit_id = battle_update
        .events_delta
        .events
        .iter()
        .find_map(|entry| match entry.event {
            LiveBattlePresentationEventKindDto::UnitDeployed {
                employee_uuid: event_employee_uuid,
                unit_instance_id,
                ..
            } if event_employee_uuid == employee_uuid => Some(unit_instance_id),
            _ => None,
        })
        .expect("deployment update should expose the player runtime identity");

    let before_reconnect = battle_advance_atomic_snapshot(&core);
    let reconnect = core
        .active_battle_reconnect_projection(0)
        .expect("pre-spawn reconnect projection should succeed");
    assert!(matches!(reconnect, BehaviorResult::BattleAdvanced { .. }));
    assert_eq!(battle_advance_atomic_snapshot(&core), before_reconnect);

    let spawned = core
        .advance_active_battle_by(10)
        .expect("late Boss actor spawn should advance");
    let BehaviorResult::BattleAdvanced {
        battle_update: spawned_update,
        finished: false,
        ..
    } = spawned
    else {
        panic!("expected active battle after Boss actor spawn");
    };
    let initial_actor = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()
        .into_iter()
        .next()
        .expect("Boss actor runtime snapshot");
    let actor_id = initial_actor
        .unit_id
        .expect("late actor should bind exactly");
    assert_eq!(
        initial_actor.pattern_id.as_deref(),
        Some("track_and_rejoin")
    );
    assert_eq!(initial_actor.state_id.as_deref(), Some("approach"));
    assert_eq!(initial_actor.target_id, Some(player_unit_id));
    assert!(spawned_update
        .events_delta
        .events
        .iter()
        .any(|entry| matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::UnitSpawned {
                unit_instance_id,
                owner: Side::Opponent,
                ..
            } if unit_instance_id == actor_id
        )));

    let before_active_reconnect = battle_advance_atomic_snapshot(&core);
    let active_reconnect = core
        .active_battle_reconnect_projection(spawned_update.events_delta.to_seq)
        .expect("active Pattern reconnect projection should succeed");
    let BehaviorResult::BattleAdvanced {
        battle_update: active_reconnect_update,
        ..
    } = active_reconnect
    else {
        panic!("expected active Pattern reconnect update");
    };
    assert!(active_reconnect_update.events_delta.events.is_empty());
    assert_eq!(
        battle_advance_atomic_snapshot(&core),
        before_active_reconnect
    );
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0],
        initial_actor
    );

    let locked_target = initial_actor.target_id;
    let mut last_cursor = initial_actor.route_cursor_index;
    let mut observed_rejoin = false;
    let mut movement_start_count = spawned_update
        .events_delta
        .events
        .iter()
        .filter(|entry| {
            matches!(
                entry.event,
                LiveBattlePresentationEventKindDto::MovementSegmentStarted {
                    unit_instance_id,
                    ..
                } if unit_instance_id == actor_id
            )
        })
        .count();
    let mut reached_route_combat = false;
    for _ in 0..120 {
        let advanced = core
            .advance_active_battle_by(100)
            .expect("TrackAndRejoin world slice should advance");
        let BehaviorResult::BattleAdvanced {
            battle_update,
            finished: false,
            ..
        } = advanced
        else {
            panic!("TrackAndRejoin fixture should not terminate before RouteCombat");
        };
        movement_start_count += battle_update
            .events_delta
            .events
            .iter()
            .filter(|entry| {
                matches!(
                    entry.event,
                    LiveBattlePresentationEventKindDto::MovementSegmentStarted {
                        unit_instance_id,
                        ..
                    } if unit_instance_id == actor_id
                )
            })
            .count();
        let actor = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()
            .into_iter()
            .next()
            .expect("Boss actor runtime snapshot while active");
        assert!(actor.route_cursor_index >= last_cursor);
        last_cursor = actor.route_cursor_index;
        if actor.pattern_id.is_some() {
            assert_eq!(actor.target_id, locked_target);
        }
        observed_rejoin |= actor.state_id.as_deref() == Some("rejoin");
        if actor.control_node_id.as_deref() == Some("route_combat") && actor.pattern_id.is_none() {
            reached_route_combat = true;
            break;
        }
    }
    assert!(
        observed_rejoin,
        "world slice should enter authored rejoin State"
    );
    assert!(
        reached_route_combat,
        "world slice should resume RouteCombat"
    );
    assert!(
        movement_start_count >= 2,
        "approach and rejoin must use movement leaf"
    );

    let before_final_reconnect = battle_advance_atomic_snapshot(&core);
    let reconnect = core
        .active_battle_reconnect_projection(0)
        .expect("post-pattern reconnect projection should succeed");
    let BehaviorResult::BattleAdvanced { battle_update, .. } = reconnect else {
        panic!("expected reconnect battle update");
    };
    assert!(!serde_json::to_string(&battle_update)
        .unwrap()
        .contains("boss_runtime"));
    assert_eq!(
        battle_advance_atomic_snapshot(&core),
        before_final_reconnect
    );
}

#[test]
fn track_and_rejoin_paired_no_binding_keeps_ordinary_route_authority() {
    let mut core = GameCore::new(game_data_with_track_and_rejoin_fixture(false), 0xB055_0402);
    let player_id = Uuid::from_u128(0xB055_0402);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "defense_route_encounter",
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .expect("paired ordinary fixture preview should resolve");
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let started = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("paired ordinary fixture should enter live battle");
    let BehaviorResult::BattleAdvanced {
        battle_setup_snapshot: Some(setup),
        battle_update: initial_update,
        finished: false,
        ..
    } = started
    else {
        panic!("expected paired ordinary setup and initial update");
    };
    assert!(!serde_json::to_string(&setup)
        .unwrap()
        .contains("boss_runtime"));
    assert!(!serde_json::to_string(&initial_update)
        .unwrap()
        .contains("boss_runtime"));
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.battle.scenario().boss_runtime.is_none());
    assert!(active
        .battle
        .boss_actor_runtime_debug_snapshots()
        .is_empty());

    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect("paired ordinary player deployment should succeed");
    let spawned = core
        .advance_active_battle_by(10)
        .expect("paired ordinary late wave should advance");
    let BehaviorResult::BattleAdvanced {
        battle_update,
        finished: false,
        ..
    } = spawned
    else {
        panic!("expected paired ordinary battle to remain active");
    };
    let actor_id = battle_update
        .events_delta
        .events
        .iter()
        .find_map(|entry| match entry.event {
            LiveBattlePresentationEventKindDto::UnitSpawned {
                unit_instance_id,
                owner: Side::Opponent,
                ..
            } => Some(unit_instance_id),
            _ => None,
        })
        .expect("paired ordinary wave should expose its actor identity");
    let moved = core
        .advance_active_battle_by(100)
        .expect("ordinary planner tick should advance the authored route");
    let BehaviorResult::BattleAdvanced {
        battle_update: moved_update,
        finished: false,
        ..
    } = moved
    else {
        panic!("expected paired ordinary battle after planner tick");
    };
    assert!(moved_update
        .events_delta
        .events
        .iter()
        .any(|entry| matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::MovementSegmentStarted {
                unit_instance_id,
                ..
            } if unit_instance_id == actor_id
        )));
    let actor = &core.state.active_battle.as_ref().unwrap().battle.units[&actor_id];
    assert!(actor.enemy_movement_plan.is_some());
    assert!(core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()
        .is_empty());

    let before_reconnect = battle_advance_atomic_snapshot(&core);
    let reconnect = core
        .active_battle_reconnect_projection(0)
        .expect("paired ordinary reconnect projection should succeed");
    let BehaviorResult::BattleAdvanced { battle_update, .. } = reconnect else {
        panic!("expected paired ordinary reconnect update");
    };
    assert!(!serde_json::to_string(&battle_update)
        .unwrap()
        .contains("boss_runtime"));
    assert_eq!(battle_advance_atomic_snapshot(&core), before_reconnect);
}

#[test]
fn track_and_rejoin_no_target_uses_authored_route_without_public_boss_state() {
    let mut game_data = game_data_with_track_and_rejoin_fixture(true);
    let database =
        Arc::get_mut(&mut game_data).expect("fixture game data should remain uniquely owned");
    let mut catalog = crate::game::data::boss_runtime_data::tests::valid_catalog();
    let crate::game::data::boss_runtime_data::BossControlNodeData::PatternCall { selector, .. } =
        &mut catalog.actor_behaviors[0].nodes[0]
    else {
        unreachable!()
    };
    let crate::game::data::boss_runtime_data::BossPatternSelectorData::FixedSequence {
        candidates,
        ..
    } = selector
    else {
        unreachable!()
    };
    let crate::game::data::boss_runtime_data::BossParameterValueData::TargetQuery(query) =
        &mut candidates[0].arguments[0].value
    else {
        unreachable!()
    };
    query
        .filters
        .push(crate::game::data::boss_runtime_data::BossTargetFilterData::Grounded);
    database.boss_runtime_data = Arc::new(catalog.compile().unwrap());

    let mut core = GameCore::new(game_data, 0xB055_0403);
    let player_id = Uuid::from_u128(0xB055_0403);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "defense_route_encounter",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let started = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("no-target fixture should enter live battle");
    let BehaviorResult::BattleAdvanced {
        finished: false, ..
    } = started
    else {
        panic!("expected active no-target battle");
    };
    let active = core.state.active_battle.as_mut().unwrap();
    for unit in active
        .battle
        .units
        .values_mut()
        .filter(|unit| unit.owner == Side::Player)
    {
        unit.mobility_kind = crate::game::battle::types::MobilityKind::Airborne;
    }
    let spawned = core.advance_active_battle_by(10).unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_update,
        finished: false,
        ..
    } = spawned
    else {
        panic!("expected active no-target battle after actor spawn");
    };
    let actor = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()
        .into_iter()
        .next()
        .expect("no-target actor runtime");
    assert_eq!(actor.control_node_id.as_deref(), Some("route_combat"));
    assert!(actor.pattern_id.is_none());
    assert!(actor.target_id.is_none());
    assert!(!serde_json::to_string(&battle_update)
        .unwrap()
        .contains("boss_runtime"));

    let before_reconnect = battle_advance_atomic_snapshot(&core);
    core.active_battle_reconnect_projection(0).unwrap();
    assert_eq!(battle_advance_atomic_snapshot(&core), before_reconnect);
}

#[test]
fn track_and_rejoin_timeout_uses_battle_time_and_pause_freezes_both_deadlines() {
    let mut game_data = game_data_with_track_and_rejoin_fixture(true);
    let database =
        Arc::get_mut(&mut game_data).expect("fixture game data should remain uniquely owned");
    let abnormalities = Arc::make_mut(&mut database.abnormality_data);
    abnormalities
        .items
        .iter_mut()
        .find(|item| item.id == "defense_route_abno")
        .unwrap()
        .movement
        .speed_units_per_ms = 0;
    let (mut core, player_id, _, _) =
        start_track_and_rejoin_fixture_at_approach(game_data, 0xB055_0404);
    let approach = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    let approach_state_activation = approach.state_activation_id;

    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle)
        .expect("fixture battle should pause");
    let paused = battle_advance_atomic_snapshot(&core);
    assert!(core
        .advance_active_battle_for_server_tick(50_000)
        .expect("paused server tick should be a valid no-op")
        .is_none());
    assert_eq!(battle_advance_atomic_snapshot(&core), paused);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle)
        .expect("fixture battle should resume");

    core.advance_active_battle_by(4_999).unwrap();
    let before_approach_deadline = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    assert_eq!(
        before_approach_deadline.state_id.as_deref(),
        Some("approach")
    );
    assert_eq!(
        before_approach_deadline.state_activation_id,
        approach_state_activation
    );
    core.advance_active_battle_by(1).unwrap();
    let rejoin = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    assert_eq!(rejoin.state_id.as_deref(), Some("rejoin"));
    assert_ne!(rejoin.state_activation_id, approach_state_activation);

    core.advance_active_battle_by(4_999).unwrap();
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0]
            .state_activation_id,
        rejoin.state_activation_id
    );
    core.advance_active_battle_by(1).unwrap();
    let route = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    assert_eq!(route.control_node_id.as_deref(), Some("route_combat"));
    assert!(route.pattern_id.is_none());
    assert_ne!(route.state_activation_id, rejoin.state_activation_id);
}

#[test]
fn track_and_rejoin_withdrawn_target_invalidates_once_without_reacquisition() {
    let (mut core, player_id, employee_uuid, player_unit_id) =
        start_track_and_rejoin_fixture_at_approach(
            game_data_with_track_and_rejoin_fixture(true),
            0xB055_0405,
        );
    let initial_actor = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    let pattern_activation_id = initial_actor.pattern_activation_id;

    let withdrawn = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::WithdrawUnit { employee_uuid },
    )
    .expect("locked target withdrawal should commit");
    let BehaviorResult::BattleUnitWithdrawn { battle_update, .. } = withdrawn else {
        panic!("expected target withdrawal update");
    };
    assert_eq!(
        battle_update
            .events_delta
            .events
            .iter()
            .filter(|entry| matches!(
                entry.event,
                LiveBattlePresentationEventKindDto::UnitWithdrawn {
                    unit_instance_id,
                    ..
                } if unit_instance_id == player_unit_id
            ))
            .count(),
        1
    );

    let mut reached_route_combat = false;
    for _ in 0..120 {
        core.advance_active_battle_by(100).unwrap();
        let actor = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0]
            .clone();
        assert!(
            actor.target_id.is_none(),
            "withdrawn target must not be reacquired"
        );
        if actor.control_node_id.as_deref() == Some("route_combat") && actor.pattern_id.is_none() {
            reached_route_combat = true;
            let resident_activation = actor.state_activation_id;
            core.advance_active_battle_by(100).unwrap();
            let stable = core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .boss_actor_runtime_debug_snapshots()[0]
                .clone();
            assert_eq!(stable.state_activation_id, resident_activation);
            assert_ne!(stable.pattern_activation_id, pattern_activation_id);
            break;
        }
    }
    assert!(reached_route_combat);
}

#[test]
fn track_and_rejoin_active_locked_target_with_unusable_path_takes_authored_recovery() {
    let (mut core, _, _, player_unit_id) = start_track_and_rejoin_fixture_at_approach(
        game_data_with_track_and_rejoin_fixture(true),
        0xB055_0406,
    );
    let initial_actor = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .units
        .get_mut(&player_unit_id)
        .unwrap()
        .set_world_position(crate::game::battle::core::movement::types::WorldVec2::new(
            -100.0, -100.0,
        ));

    core.advance_active_battle_by(100).unwrap();

    let recovered = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    assert_eq!(recovered.state_id.as_deref(), Some("rejoin"));
    assert_eq!(recovered.target_id, Some(player_unit_id));
    assert_ne!(
        recovered.state_activation_id,
        initial_actor.state_activation_id
    );
    assert!(core.state.active_battle.as_ref().unwrap().battle.units[&player_unit_id].is_active());

    let mut reached_route_combat = false;
    for _ in 0..120 {
        core.advance_active_battle_by(100).unwrap();
        let actor = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0]
            .clone();
        if actor.control_node_id.as_deref() == Some("route_combat") && actor.pattern_id.is_none() {
            assert!(actor.target_id.is_none());
            reached_route_combat = true;
            break;
        }
    }
    assert!(reached_route_combat);

    let before_reconnect = battle_advance_atomic_snapshot(&core);
    core.active_battle_reconnect_projection(0).unwrap();
    assert_eq!(battle_advance_atomic_snapshot(&core), before_reconnect);
}

#[test]
fn track_and_rejoin_actor_death_terminates_once_before_stale_runtime_work() {
    let (mut core, _, _, _) = start_track_and_rejoin_fixture_at_approach(
        game_data_with_track_and_rejoin_fixture(true),
        0xB055_0407,
    );
    let actor_id = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .unit_id
        .unwrap();
    {
        let actor = core
            .state
            .active_battle
            .as_mut()
            .unwrap()
            .battle
            .units
            .get_mut(&actor_id)
            .unwrap();
        actor.stats.current_health = 0;
        actor.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
    }

    let finished = core
        .advance_active_battle_by(0)
        .expect("terminal reconciliation should accept injected actor death");
    let BehaviorResult::BattleAdvanced {
        battle_update,
        finished: true,
        ..
    } = finished
    else {
        panic!("required Boss actor death should finish the encounter exactly once");
    };
    assert!(!serde_json::to_string(&battle_update)
        .unwrap()
        .contains("boss_runtime"));
    assert!(core.state.active_battle.is_none());
    assert!(matches!(core.get_state(), GameState::CombatResult { .. }));
    assert!(matches!(
        core.advance_active_battle_by(0),
        Err(GameError::InvalidAction)
    ));
}

fn enter_live_little_red_reference(
    game_data: Arc<GameDataBase>,
    seed: u64,
    deploy_employee: bool,
    legacy_sword_fixture: bool,
) -> (
    GameCore,
    Uuid,
    Option<Uuid>,
    Option<crate::game::battle::ids::UnitInstanceId>,
) {
    let mut core = GameCore::new(game_data, seed);
    let player_id = Uuid::from_u128(seed as u128);
    start_new_game_with_default_starters(&mut core, player_id);
    if legacy_sword_fixture { apply_legacy_sword_numerical_fixture(&mut core, player_id); }
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "suppress_little_red",
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .expect("live Little Red preview should resolve");
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    let started = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("live Little Red should enter battle through the production setup path");
    let BehaviorResult::BattleAdvanced {
        battle_setup_snapshot: Some(setup),
        battle_update,
        finished: false,
        ..
    } = started
    else {
        panic!("expected active live Little Red setup");
    };
    assert_eq!(setup.encounter_id, "suppress_little_red");
    assert!(!serde_json::to_string(&setup)
        .unwrap()
        .contains("boss_runtime"));
    assert!(!serde_json::to_string(&battle_update)
        .unwrap()
        .contains("boss_runtime"));
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.battle.scenario().boss_runtime.is_some());
    let actors = active.battle.boss_actor_runtime_debug_snapshots();
    assert_eq!(actors.len(), 1);
    assert!(actors[0].unit_id.is_none());

    if !deploy_employee {
        return (core, player_id, None, None);
    }
    let deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect("player should deploy before the live Little Red wave");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id, ..
    } = deployed
    else {
        panic!("expected live Little Red deployment result");
    };
    (core, player_id, Some(employee_uuid), Some(unit_instance_id))
}

fn isolate_live_little_red_target(
    core: &mut GameCore,
    player_unit_id: crate::game::battle::ids::UnitInstanceId,
) {
    let active = core.state.active_battle.as_mut().unwrap();
    for (unit_id, unit) in &mut active.battle.units {
        if unit.owner != Side::Player {
            continue;
        }
        if *unit_id == player_unit_id {
            unit.stats.max_health = 1_000_000;
            unit.stats.current_health = 1_000_000;
        } else {
            unit.set_world_position(crate::game::battle::core::movement::types::WorldVec2::new(
                -100.0, -100.0,
            ));
        }
    }
}

#[test]
fn live_little_red_runs_finite_pursuit_pause_reconnect_route_combat_and_terminal_flow() {
    let (mut core, player_id, _, player_unit_id) =
        enter_live_little_red_reference(live_game_data_from_ron(), 0xB055_0501, true, false);
    let player_unit_id = player_unit_id.unwrap();
    isolate_live_little_red_target(&mut core, player_unit_id);

    let spawned = core
        .advance_active_battle_by(5_000)
        .expect("the production Little Red wave should spawn");
    let BehaviorResult::BattleAdvanced {
        battle_update,
        finished: false,
        ..
    } = spawned
    else {
        panic!("live Little Red should remain active after spawning");
    };
    assert!(!serde_json::to_string(&battle_update)
        .unwrap()
        .contains("boss_runtime"));
    let initial_actor = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    let actor_id = initial_actor
        .unit_id
        .expect("Little Red should bind exactly");
    assert_eq!(
        initial_actor.pattern_id.as_deref(),
        Some("little_red_pursuit")
    );
    assert_eq!(initial_actor.state_id.as_deref(), Some("move_to_target"));
    assert_eq!(initial_actor.target_id, Some(player_unit_id));

    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle)
        .expect("live Little Red battle should pause");
    let paused = battle_advance_atomic_snapshot(&core);
    assert!(core
        .advance_active_battle_for_server_tick(50_000)
        .expect("paused server tick should be a valid no-op")
        .is_none());
    assert_eq!(battle_advance_atomic_snapshot(&core), paused);
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle)
        .expect("live Little Red battle should resume");

    let before_reconnect = battle_advance_atomic_snapshot(&core);
    let reconnect = core
        .active_battle_reconnect_projection(battle_update.events_delta.to_seq)
        .expect("active live Little Red reconnect should succeed");
    let BehaviorResult::BattleAdvanced {
        battle_update: reconnect_update,
        ..
    } = reconnect
    else {
        panic!("expected live Little Red reconnect update");
    };
    assert!(reconnect_update.events_delta.events.is_empty());
    assert!(!serde_json::to_string(&reconnect_update)
        .unwrap()
        .contains("boss_runtime"));
    assert_eq!(battle_advance_atomic_snapshot(&core), before_reconnect);

    let mut saw_rejoin = false;
    let mut reached_route_combat = false;
    let mut last_route_cursor = initial_actor.route_cursor_index;
    for _ in 0..120 {
        let advanced = core
            .advance_active_battle_by(100)
            .expect("live Little Red pursuit slice should advance");
        let BehaviorResult::BattleAdvanced {
            finished: false, ..
        } = advanced
        else {
            panic!("live Little Red should reach RouteCombat before terminal resolution");
        };
        let actor = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0]
            .clone();
        assert!(actor.route_cursor_index >= last_route_cursor);
        last_route_cursor = actor.route_cursor_index;
        if actor.pattern_id.is_some() {
            assert_eq!(actor.target_id, Some(player_unit_id));
        }
        saw_rejoin |= actor.state_id.as_deref() == Some("rejoin_forward_route");
        if actor.control_node_id.as_deref() == Some("route_combat") && actor.pattern_id.is_none() {
            reached_route_combat = true;
            break;
        }
    }
    assert!(saw_rejoin);
    assert!(reached_route_combat);

    {
        let active = core.state.active_battle.as_mut().unwrap();
        let actor = active.battle.units.get_mut(&actor_id).unwrap();
        actor.stats.current_health = 0;
        actor.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
    }
    let interrupted = core
        .advance_active_battle_by(0)
        .expect("live Little Red death should reconcile without stale runtime work");
    assert!(matches!(
        interrupted,
        BehaviorResult::BattleAdvanced {
            finished: false,
            ..
        }
    ));
    let interrupted_actor = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    assert_eq!(interrupted_actor.unit_id, Some(actor_id));
    assert_eq!(
        interrupted_actor.control_node_id.as_deref(),
        Some("route_combat")
    );
    assert!(interrupted_actor.pattern_id.is_none());
    assert!(interrupted_actor.state_id.is_none());
    assert!(interrupted_actor.target_id.is_none());
    let repeated = core
        .advance_active_battle_by(0)
        .expect("dead Boss actor should not schedule duplicate runtime work");
    assert!(matches!(
        repeated,
        BehaviorResult::BattleAdvanced {
            finished: false,
            ..
        }
    ));
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0],
        interrupted_actor
    );

    for unit in core
        .state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .units
        .values_mut()
        .filter(|unit| unit.owner == Side::Opponent)
    {
        unit.stats.current_health = 0;
        unit.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
    }
    let terminal = core
        .advance_active_battle_by(0)
        .expect("all required live opponents dead should resolve terminal state");
    assert!(matches!(
        terminal,
        BehaviorResult::BattleAdvanced { finished: true, .. }
    ));
    assert!(core.state.active_battle.is_none());
    assert!(matches!(core.get_state(), GameState::CombatResult { .. }));
    assert!(matches!(
        core.advance_active_battle_by(0),
        Err(GameError::InvalidAction)
    ));
}

#[test]
fn live_cwbs_reference_connects_conditional_wave_summon_signal_outcome_and_victory_registration() {
    const ENCOUNTER_ID: &str = "reference_cwbs_final_boss";
    const BOSS_ID: &str = "x-00-00_cwbs_reference_boss";
    const MINION_ID: &str = "cwbs_reference_minion";

    let game_data = live_game_data_from_ron();
    let encounter = game_data
        .pve_data
        .get_by_id(ENCOUNTER_ID)
        .expect("live CWBS reference encounter");
    assert!(encounter.is_reference_only());

    let mut core = GameCore::new(game_data, 0xC0B5_0601);
    let player_id = Uuid::from_u128(0xC0B5_0601);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_reference_only",
        ENCOUNTER_ID,
    );
    let preview = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .expect("reference preview should resolve only when explicitly addressed");
    let deploy_position = first_ground_deployment_cell(&preview);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("reference encounter should enter through the normal combat setup path");
    let deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect("reference battle should allow a player deployment");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: player_unit_id,
        ..
    } = deployed
    else {
        panic!("expected deployed player unit");
    };
    {
        let player = core
            .state
            .active_battle
            .as_mut()
            .unwrap()
            .battle
            .units
            .get_mut(&player_unit_id)
            .unwrap();
        player.stats.max_health = 1_000_000;
        player.stats.current_health = 1_000_000;
        player.stats.attack = 0;
    }

    let initial = core
        .advance_active_battle_by(0)
        .expect("time-zero reference wave and summon should settle");
    assert!(matches!(
        initial,
        BehaviorResult::BattleAdvanced {
            finished: false,
            ..
        }
    ));
    let boss_unit_id = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .unit_id
        .expect("reference boss actor should bind to the time-zero Wave spawn");
    {
        let battle = &core.state.active_battle.as_ref().unwrap().battle;
        let summon = battle.summon_runtime_debug_snapshot();
        assert_eq!(summon.pending_count, 0);
        assert_eq!(summon.batch_count, 1);
        assert_eq!(summon.counters.len(), 1);
        assert_eq!(summon.counters[0].1, MINION_ID);
        assert_eq!((summon.counters[0].2, summon.counters[0].3), (1, 1));
        assert!(battle.units.values().any(|unit| {
            unit.is_active()
                && matches!(
                    &unit.source_identity,
                    BattleUnitSourceIdentity::SummonedUnit { definition_id, .. }
                        if definition_id == MINION_ID
                )
        }));
        assert!(battle.units.contains_key(&boss_unit_id));
        let candidates = battle.compendium_candidates();
        assert!(candidates.iter().any(|candidate| {
            candidate
                == &CompendiumCandidate {
                    id: BOSS_ID.to_string(),
                    kind: CompendiumCandidateKind::Abnormality,
                    base_uuid: Uuid::parse_str("c0de0600-0000-0000-0000-000000000001").unwrap(),
                }
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.id == "corroded_rusher"
                && candidate.kind == CompendiumCandidateKind::CorrodedEmployee
        }));
        assert_eq!(
            candidates.len(),
            2,
            "the summoned unit must never become a compendium candidate"
        );
    }

    core.advance_active_battle_by(1_000)
        .expect("authored cleanup signal should remove the exact summon batch");
    {
        let battle = &core.state.active_battle.as_ref().unwrap().battle;
        let summon = battle.summon_runtime_debug_snapshot();
        assert_eq!((summon.counters[0].2, summon.counters[0].3), (0, 1));
        assert!(summon.batches[0].2, "batch clear should latch exactly once");
        assert!(battle.boss_signal_latched_for_test(
            &crate::game::data::boss_runtime_data::BossSignalId::new("cwbs_reference_cleanup",),
        ));
        assert!(!battle.units.values().any(|unit| {
            unit.is_active()
                && matches!(
                    &unit.source_identity,
                    BattleUnitSourceIdentity::SummonedUnit { definition_id, .. }
                        if definition_id == MINION_ID
                )
        }));
    }

    {
        let battle = &mut core.state.active_battle.as_mut().unwrap().battle;
        for unit in battle
            .units
            .values_mut()
            .filter(|unit| unit.owner == Side::Opponent && unit.is_active())
        {
            unit.stats.current_health = 0;
            unit.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
        }
    }
    let settled = core
        .advance_active_battle_by(0)
        .expect("required Wave groups should settle to the named Victory outcome");
    assert!(matches!(
        settled,
        BehaviorResult::BattleAdvanced { finished: true, .. }
    ));
    let receipt = core
        .state
        .confirmed_combat_result
        .as_ref()
        .expect("confirmed result");
    let crate::game::behavior::SelectedEventSnapshotDto::CombatBattle {
        outcome_id,
        verdict,
        rewards,
        ..
    } = &receipt.summary
    else {
        panic!("combat summary")
    };
    assert_eq!(outcome_id.as_str(), "reference_victory");
    assert_eq!(*verdict, BattleVerdict::Victory);
    assert!(rewards.is_empty());
    let committed_profile = core.encounter_compendium_profile();

    core.execute(player_id, PlayerBehavior::CompleteCombatResult)
        .expect("reference Victory should consume atomically");
    let profile = core.encounter_compendium_profile();
    assert_eq!(profile, committed_profile);
    assert!(profile.registered_ids.contains(BOSS_ID));
    assert!(profile.registered_ids.contains("corroded_rusher"));
    assert!(!profile.registered_ids.contains(MINION_ID));
    assert!(!core.state.player_profile.all_live_final_bosses_defeated);
}

#[test]
fn live_little_red_no_target_and_locked_target_failure_paths_rejoin_authored_route() {
    let (mut no_target, _, _, _) =
        enter_live_little_red_reference(live_game_data_from_ron(), 0xB055_0502, false, true);
    for unit in no_target
        .state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .units
        .values_mut()
        .filter(|unit| unit.owner == Side::Player)
    {
        unit.set_world_position(crate::game::battle::core::movement::types::WorldVec2::new(
            -100.0, -100.0,
        ));
    }
    no_target.advance_active_battle_by(5_000).unwrap();
    let actor = no_target
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    assert_eq!(actor.control_node_id.as_deref(), Some("route_combat"));
    assert!(actor.pattern_id.is_none());
    assert!(actor.target_id.is_none());

    for (seed, failure) in [(0xB055_0503, "invalidation"), (0xB055_0504, "path")] {
        let (mut core, player_id, employee_uuid, player_unit_id) =
            enter_live_little_red_reference(live_game_data_from_ron(), seed, true, true);
        let player_unit_id = player_unit_id.unwrap();
        isolate_live_little_red_target(&mut core, player_unit_id);
        core.advance_active_battle_by(5_000).unwrap();
        let initial = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0]
            .clone();
        assert_eq!(initial.target_id, Some(player_unit_id));
        let pattern_activation_id = initial.pattern_activation_id;

        if failure == "invalidation" {
            execute_current_battle_mutation(
                &mut core,
                player_id,
                PlayerBehavior::WithdrawUnit {
                    employee_uuid: employee_uuid.unwrap(),
                },
            )
            .expect("locked live Little Red target should withdraw");
        } else {
            core.state
                .active_battle
                .as_mut()
                .unwrap()
                .battle
                .units
                .get_mut(&player_unit_id)
                .unwrap()
                .set_world_position(crate::game::battle::core::movement::types::WorldVec2::new(
                    -100.0, -100.0,
                ));
        }

        let mut reached_route_combat = false;
        for _ in 0..120 {
            core.advance_active_battle_by(100).unwrap();
            let actor = core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .battle
                .boss_actor_runtime_debug_snapshots()[0]
                .clone();
            if failure == "invalidation" {
                assert!(actor.target_id.is_none());
            }
            if actor.control_node_id.as_deref() == Some("route_combat")
                && actor.pattern_id.is_none()
            {
                assert_ne!(actor.pattern_activation_id, pattern_activation_id);
                reached_route_combat = true;
                break;
            }
        }
        assert!(reached_route_combat, "{failure} should reach RouteCombat");
    }
}

#[test]
fn live_little_red_timeout_uses_battle_time_and_has_an_exhaustive_rejoin_exit() {
    let mut game_data = live_game_data_from_ron();
    let database = Arc::get_mut(&mut game_data)
        .expect("live data should remain uniquely owned before GameCore construction");
    Arc::make_mut(&mut database.abnormality_data)
        .items
        .iter_mut()
        .find(|item| item.id == "f-01-57_little_red")
        .unwrap()
        .movement
        .speed_units_per_ms = 0;
    let (mut core, _, _, player_unit_id) =
        enter_live_little_red_reference(game_data, 0xB055_0505, true, false);
    let player_unit_id = player_unit_id.unwrap();
    isolate_live_little_red_target(&mut core, player_unit_id);
    core.advance_active_battle_by(5_000).unwrap();
    let approach = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    assert_eq!(approach.state_id.as_deref(), Some("move_to_target"));

    core.advance_active_battle_by(9_999).unwrap();
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0]
            .state_activation_id,
        approach.state_activation_id
    );
    core.advance_active_battle_by(1).unwrap();
    let rejoin = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    assert_eq!(rejoin.state_id.as_deref(), Some("rejoin_forward_route"));
    assert_ne!(rejoin.state_activation_id, approach.state_activation_id);

    core.advance_active_battle_by(9_999).unwrap();
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .boss_actor_runtime_debug_snapshots()[0]
            .state_activation_id,
        rejoin.state_activation_id
    );
    core.advance_active_battle_by(1).unwrap();
    let route = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()[0]
        .clone();
    assert_eq!(route.control_node_id.as_deref(), Some("route_combat"));
    assert!(route.pattern_id.is_none());
}

#[test]
fn live_non_opt_in_boss_encounter_keeps_ordinary_runtime_and_public_shape() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xB055_0506);
    let player_id = Uuid::from_u128(0xB055_0506);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "suppress_big_bad_wolf",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let started = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(!serde_json::to_string(&started)
        .unwrap()
        .contains("boss_runtime"));
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.battle.scenario().boss_runtime.is_none());
    assert!(active
        .battle
        .boss_actor_runtime_debug_snapshots()
        .is_empty());

    let before_reconnect = battle_advance_atomic_snapshot(&core);
    core.active_battle_reconnect_projection(0).unwrap();
    assert_eq!(battle_advance_atomic_snapshot(&core), before_reconnect);
    let spawned = core.advance_active_battle_by(5_000).unwrap();
    assert!(!serde_json::to_string(&spawned)
        .unwrap()
        .contains("boss_runtime"));
    assert!(core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .boss_actor_runtime_debug_snapshots()
        .is_empty());
}

#[test]
fn late_boss_setup_failure_does_not_mutate_world_or_session_state() {
    let mut game_data = game_data_with_pve_encounters();
    let database = Arc::get_mut(&mut game_data)
        .expect("fixture game data should be uniquely owned before GameCore construction");
    let boss_runtime = crate::game::data::boss_runtime_data::tests::valid_catalog()
        .compile()
        .expect("Boss fixture catalog should compile");
    database.boss_runtime_data = Arc::new(boss_runtime);
    let pve = Arc::make_mut(&mut database.pve_data);
    let encounter = pve
        .encounters
        .iter_mut()
        .find(|encounter| encounter.id == "boss_risk_encounter")
        .expect("Boss fixture encounter should exist");
    encounter.boss_runtime = Some(crate::game::data::pve_data::PveBossRuntimeBindingData {
        encounter_id: crate::game::data::boss_runtime_data::BossEncounterRuntimeId::new(
            "tracking_encounter",
        ),
        actors: vec![crate::game::data::pve_data::PveBossActorBindingData {
            actor_slot_id: crate::game::data::boss_runtime_data::BossActorSlotId::new("primary"),
            wave_id: "wave_0".to_string(),
            spawn_index: 0,
        }],
    });

    let mut core = GameCore::new(game_data, 0xB055_0210);
    let player_id = Uuid::from_u128(0xB055_0211);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "boss_risk_encounter",
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .expect("Boss node selection should reach confirmation");
    let before = battle_start_atomic_snapshot(&core);

    let error = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect_err("route-dependent Boss setup should reject a route-less Boss arena");

    assert!(matches!(
        error,
        GameError::InvalidStaticData(message) if message.contains("requires a non-empty authored route")
    ));
    assert_eq!(battle_start_atomic_snapshot(&core), before);
    assert!(core.state.active_battle.is_none());
}

#[test]
fn mutating_advance_rejects_future_cursor_without_any_state_change() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xB055_C003);
    let player_id = Uuid::from_u128(0xB055_C003);
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

    let before = battle_advance_atomic_snapshot(&core);
    let error = core
        .advance_active_battle_by_after_seq(10, u64::MAX)
        .expect_err("future cursor must reject a mutating advance");
    assert!(matches!(error, GameError::InvalidBattleResyncSeq { .. }));
    let after = battle_advance_atomic_snapshot(&core);

    assert_eq!(after, before);
}

#[test]
fn elapsed_and_bucket_advance_reject_revision_overflow_without_any_state_change() {
    for entrypoint in ["elapsed", "bucket"] {
        let seed = if entrypoint == "elapsed" {
            0xB055_C004
        } else {
            0xB055_C005
        };
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
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core.state
            .active_battle
            .as_mut()
            .unwrap()
            .battle_state_revision = u64::MAX;

        let before = battle_advance_atomic_snapshot(&core);
        let error = match entrypoint {
            "elapsed" => core
                .advance_active_battle_by_after_seq(10, 0)
                .expect_err("elapsed advance must reject revision overflow"),
            "bucket" => core
                .advance_active_battle_to_next_event_bucket()
                .expect_err("bucket advance must reject revision overflow"),
            _ => unreachable!(),
        };
        assert!(matches!(error, GameError::BattleStateRevisionOverflow));
        assert_eq!(battle_advance_atomic_snapshot(&core), before);
    }
}

#[test]
fn advance_rolls_back_core_and_world_when_visual_novel_trigger_fails() {
    let mut game_data = live_game_data_from_ron();
    let database = Arc::get_mut(&mut game_data)
        .expect("live data should be uniquely owned before GameCore construction");
    let trigger_database = Arc::get_mut(&mut database.visual_novel_trigger_data)
        .expect("trigger data should be uniquely owned before GameCore construction");
    trigger_database.triggers.push(VisualNovelTrigger {
        id: VisualNovelTriggerId::new("valid_vn_before_missing_vn"),
        source: VisualNovelTriggerSource::BattleTimeReached { battle_time_ms: 1 },
        vn_id: crate::game::data::visual_novel_data::VisualNovelId::new(
            "white_night_confession_intro",
        ),
        repeat: VisualNovelTriggerRepeat::OncePerBattle,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        priority: 2_000,
        blocking: false,
    });
    trigger_database.triggers.push(VisualNovelTrigger {
        id: VisualNovelTriggerId::new("missing_vn_during_battle_advance"),
        source: VisualNovelTriggerSource::BattleTimeReached { battle_time_ms: 1 },
        vn_id: crate::game::data::visual_novel_data::VisualNovelId::new("missing_vn"),
        repeat: VisualNovelTriggerRepeat::OncePerBattle,
        checkpoint_policy: VisualNovelTriggerCheckpointPolicy::RewindWithRunCheckpoint,
        priority: 1_000,
        blocking: false,
    });
    let mut core = GameCore::new(game_data, 0xB055_C006);
    let player_id = Uuid::from_u128(0xB055_C006);
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

    let before = battle_advance_atomic_snapshot(&core);
    let error = core
        .advance_active_battle_for_server_tick_after_seq(10, 0)
        .expect_err("missing trigger VN must fail after staged Core work");
    assert!(matches!(error, GameError::InvalidStaticData(_)));
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
}

#[test]
fn advance_rolls_back_core_and_world_when_employee_reconcile_fails() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xB055_C007);
    let player_id = Uuid::from_u128(0xB055_C007);
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
        unit_instance_id, ..
    } = deployed
    else {
        panic!("expected deployed employee");
    };
    {
        let active = core.state.active_battle.as_mut().unwrap();
        let unit = active.battle.units.get_mut(&unit_instance_id).unwrap();
        unit.stats.current_health = 0;
        unit.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
        active.employee_attempt.roster = crate::game::employee::EmployeeRoster::new();
    }

    let before = battle_advance_atomic_snapshot(&core);
    let error = core
        .advance_active_battle_by_after_seq(50, 0)
        .expect_err("missing attempt employee must fail reconciliation");
    assert!(matches!(error, GameError::BattleRuntimeFault(fault)
        if fault.code == crate::game::battle::runtime_control::RuntimeFaultCode::Invariant
            && fault.site == "reconcile_incapacitated_roster"));
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
    let retry = core.advance_active_battle_by_after_seq(50, 0).unwrap_err();
    assert!(matches!(retry, GameError::BattleRuntimeFault(fault)
        if fault.site == "reconcile_incapacitated_roster"));
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
}

#[test]
fn finishing_advance_rejects_future_cursor_without_consuming_terminal_state() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xB055_C008);
    let player_id = Uuid::from_u128(0xB055_C008);
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

    let before = battle_advance_atomic_snapshot(&core);
    let error = core
        .advance_active_battle_by_after_seq(10, u64::MAX)
        .expect_err("future cursor must reject before terminal state is consumed");
    assert!(matches!(error, GameError::InvalidBattleResyncSeq { .. }));
    assert_eq!(battle_advance_atomic_snapshot(&core), before);
    assert!(core
        .state
        .active_battle
        .as_ref()
        .is_some_and(|active| !active.execution.is_finished()));
}

#[test]
fn elapsed_and_bucket_advance_publish_equivalent_success_state() {
    fn start(seed: u64) -> GameCore {
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
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();
        core
    }

    let mut elapsed_core = start(0xB055_C009);
    let mut bucket_core = start(0xB055_C009);
    let elapsed = elapsed_core
        .advance_active_battle_by_after_seq(50, 0)
        .expect("elapsed advance should succeed");
    let bucket = bucket_core
        .advance_active_battle_to_next_event_bucket()
        .expect("bucket advance should succeed");

    assert_eq!(
        serde_json::to_value(&elapsed).unwrap(),
        serde_json::to_value(&bucket).unwrap()
    );
    let elapsed_active = elapsed_core.state.active_battle.as_ref().unwrap();
    let bucket_active = bucket_core.state.active_battle.as_ref().unwrap();
    assert_eq!(
        elapsed_active.execution.last_event_time_ms(),
        bucket_active.execution.last_event_time_ms()
    );
    assert_eq!(
        elapsed_active.battle.event_log.to_json_string().unwrap(),
        bucket_active.battle.event_log.to_json_string().unwrap()
    );
    assert_eq!(
        elapsed_active.battle.event_queue_debug_snapshot(),
        bucket_active.battle.event_queue_debug_snapshot()
    );
    assert_eq!(
        elapsed_active.battle_state_revision,
        bucket_active.battle_state_revision
    );
}

#[test]
fn live_world_tick_reaches_authored_time_outcome_without_revision_churn() {
    let mut game_data = live_game_data_from_ron();
    let database = Arc::get_mut(&mut game_data)
        .expect("live data should be uniquely owned before GameCore construction");
    let pve = Arc::get_mut(&mut database.pve_data)
        .expect("PVE data should be uniquely owned before GameCore construction");
    let encounter = pve
        .encounters
        .iter_mut()
        .find(|encounter| encounter.id == "suppress_burrowing_heaven")
        .expect("live encounter fixture");
    encounter
        .terminal_outcomes
        .retain(|outcome| outcome.verdict == BattleVerdict::Victory);
    let victory_outcome_id = encounter.terminal_outcomes[0].id.clone();
    encounter
        .outcome_policies
        .retain(|policy| policy.outcome_id == victory_outcome_id);
    encounter.terminal_outcomes[0].condition =
        crate::game::data::pve_data::PveTerminalConditionData::TimeReached { time_ms: 10 };
    for wave in &mut encounter.waves {
        wave.required_for_victory = false;
    }

    let mut core = GameCore::new(game_data, 0xB055_C001);
    let player_id = Uuid::from_u128(0xB055_C001);
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
    let revision_before = core
        .state
        .active_battle
        .as_ref()
        .expect("battle should be active before the cap tick")
        .battle_state_revision;

    let result = core
        .advance_active_battle_for_server_tick(10)
        .expect("server tick should succeed")
        .expect("server tick should emit the terminal update");
    let BehaviorResult::BattleAdvanced {
        finished,
        battle_update,
        ..
    } = result
    else {
        panic!("expected terminal battle update");
    };

    assert!(finished);
    assert_eq!(battle_update.checkpoint.battle_time_ms, 10);
    assert_eq!(battle_update.battle_state_revision, revision_before + 1);
    assert!(matches!(core.get_state(), GameState::CombatResult { .. }));
    assert!(core.state.active_battle.is_none());
    assert!(matches!(
        core.advance_active_battle_for_server_tick(1),
        Err(GameError::InvalidAction)
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
fn expired_live_history_rejects_strict_cursor_and_recovery_returns_stable_resync_snapshot() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_5102);
    let player_id = Uuid::from_u128(0xD3F3_5102);
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

    let (expired_through, latest_before_expiry) = {
        let active = core.state.active_battle.as_mut().unwrap();
        let latest = active.battle.event_log.latest_seq();
        let expired_through = latest + 1;
        active.battle.event_log.append_with_encoded_len_for_test(
            BattleEventLogEntry {
                skill_effect_source: None,
                time_ms: active.execution.last_event_time_ms(),
                seq: expired_through,
                cause: BattleEventCause::default(),
                source_command_id: None,
                event: BattleLogEvent::BattleEnd {
                    winner: BattleWinner::Draw,
                },
            },
            Some(BATTLE_HISTORY_MAX_ENCODED_BYTES - 1),
        );
        (expired_through, latest)
    };
    assert!(latest_before_expiry > 0);

    let error = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState { since_seq: Some(0) },
        )
        .expect_err("a strict cursor below the retained floor must request snapshot resync");
    assert!(matches!(
        error,
        GameError::BattleSnapshotResyncRequired {
            requested: 0,
            oldest_available_after_seq,
            latest,
        } if oldest_available_after_seq == expired_through && latest == expired_through
    ));

    let recover = |core: &mut GameCore| {
        let result = core
            .execute(player_id, PlayerBehavior::RecoverBattleSetupLoss)
            .expect("setup-loss recovery must tolerate an expired diagnostic cursor");
        let BehaviorResult::BattleAdvanced {
            battle_setup_snapshot: Some(_),
            battle_update,
            ..
        } = result
        else {
            panic!("expected setup plus typed resync snapshot");
        };
        battle_update
    };
    let first = recover(&mut core);
    let retry = recover(&mut core);

    assert_eq!(
        first.message_type,
        crate::game::behavior::LiveBattleUpdateMessageType::SnapshotResyncRequired
    );
    assert_eq!(first.events_delta.after_seq, expired_through);
    assert_eq!(first.events_delta.to_seq, expired_through);
    assert!(first.events_delta.events.is_empty());
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&retry).unwrap()
    );
}

#[test]
fn live_freischutz_field_work_setup_progress_and_recovery_are_authoritative() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xF43_5101);
    let player_id = Uuid::from_u128(0xF43_5101);
    start_new_game_with_legacy_sword_fixture(&mut core, player_id);
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "suppress_freischutz",
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let started = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid,
        battle_setup_snapshot: Some(setup),
        battle_update: first_update,
        finished: false,
        ..
    } = started
    else {
        panic!("expected Freischutz live battle start");
    };
    assert_eq!(setup.setup_version, 2);
    assert_eq!(setup.encounter_id, "suppress_freischutz");
    assert_eq!(setup.field_work_objects.len(), 1);
    let setup_object = &setup.field_work_objects[0];
    assert_eq!(setup_object.id, "black_box_emergency_repair");
    assert_eq!(setup_object.position, Position::new(2, 6));
    assert_eq!(setup_object.required_work_ms, 10_000);
    assert_eq!(setup_object.work_cells.len(), 1);
    assert_eq!(setup_object.work_cells[0].id, "repair_cell");
    assert_eq!(setup_object.work_cells[0].position, Position::new(2, 6));
    assert_eq!(
        setup_object.work_cells[0].tile_kind,
        crate::game::battle::types::TacticalTileKind::Ground
    );
    assert!(matches!(
        setup_object.presentation_kind,
        crate::game::behavior::LiveBattleFieldWorkPresentationKindDto::EmergencyRepairTerminal
    ));
    assert!(matches!(
        setup_object.completion_effect,
        crate::game::behavior::LiveBattleFieldWorkCompletionEffectDto::RepairProtectedUnit {
            ref unit_ref,
            flat_hp: 100,
        } if unit_ref == "black_box_device"
    ));
    let initial_object = &first_update.checkpoint.field_work_objects[0];
    assert_eq!(initial_object.id, "black_box_emergency_repair");
    assert_eq!(initial_object.accumulated_work_ms, 0);
    assert_eq!(initial_object.required_work_ms, 10_000);
    assert!(matches!(
        initial_object.status,
        crate::game::behavior::LiveBattleFieldWorkObjectStatusDto::Active
    ));
    assert_eq!(initial_object.work_cells[0].worker_unit_instance_id, None);

    let employee_uuid = first_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("deployment state")
        .unit_deploy_costs
        .iter()
        .find(|entry| entry.eligibility.enabled)
        .expect("deployable starter")
        .employee_uuid;
    let deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: Position::new(2, 6),
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let BehaviorResult::BattleUnitDeployed {
        battle_update: deployed_update,
        ..
    } = deployed
    else {
        panic!("expected direct deployment onto the work cell");
    };
    let deployed_unit = deployed_update
        .checkpoint
        .deployment
        .as_ref()
        .unwrap()
        .deployed_units
        .iter()
        .find(|entry| entry.employee_uuid == employee_uuid)
        .expect("deployed employee projection");
    assert_eq!(deployed_unit.position, Position::new(2, 6));
    assert_eq!(
        deployed_unit.tactical_state.stance,
        EngagementStance::Defense
    );
    assert_eq!(
        deployed_unit.tactical_state.activity,
        crate::game::battle::types::UnitExclusiveActivity::FieldWorking
    );
    assert_eq!(
        deployed_unit.tactical_state.tile_effect,
        LiveBattleTileEffectDto::None
    );
    assert_eq!(deployed_unit.tactical_state.effective_block_capacity, 0);
    let worker_unit_instance_id = deployed_unit.unit_instance_id;
    assert!(!deployed_unit.skill_readiness.eligibility.enabled);
    let worker_checkpoint = deployed_update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.unit_instance_id == worker_unit_instance_id)
        .expect("worker checkpoint actor");
    assert_eq!(
        worker_checkpoint.control.skill_start_disabled_reason,
        Some(ActionDisabledReasonCode::FieldWorking)
    );
    assert_eq!(
        deployed_update.checkpoint.field_work_objects[0].work_cells[0].worker_unit_instance_id,
        Some(worker_unit_instance_id)
    );
    assert!(deployed_update
        .events_delta
        .events
        .iter()
        .any(|entry| matches!(
            entry.event,
            LiveBattlePresentationEventKindDto::FieldWorkAssignmentStarted {
                ref object_id,
                ref work_cell_id,
                worker_unit_instance_id: event_worker,
            } if object_id == "black_box_emergency_repair"
                && work_cell_id == "repair_cell"
                && event_worker == worker_unit_instance_id
        )));

    let paused =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    let BehaviorResult::BattlePlaybackChanged {
        battle_update: paused_update,
        ..
    } = paused
    else {
        panic!("expected paused field work update");
    };
    assert!(paused_update.checkpoint.playback.paused);
    assert_eq!(
        paused_update.checkpoint.field_work_objects[0].accumulated_work_ms,
        0
    );
    assert!(core
        .advance_active_battle_for_server_tick(1_000)
        .unwrap()
        .is_none());
    let resumed =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle)
            .unwrap();
    let BehaviorResult::BattlePlaybackChanged {
        battle_update: resumed_update,
        ..
    } = resumed
    else {
        panic!("expected resumed field work update");
    };
    assert!(!resumed_update.checkpoint.playback.paused);
    assert_eq!(
        resumed_update.checkpoint.field_work_objects[0].accumulated_work_ms, 0,
        "paused wall time must not contribute or catch up on resume"
    );

    let advanced = core
        .advance_active_battle_for_server_tick(1_000)
        .unwrap()
        .expect("running field work should produce a revisioned update");
    let BehaviorResult::BattleAdvanced {
        battle_update: advanced_update,
        finished: false,
        ..
    } = advanced
    else {
        panic!("expected running Freischutz battle update");
    };
    assert_eq!(
        advanced_update.checkpoint.field_work_objects[0].accumulated_work_ms,
        1_000
    );
    let current_revision = advanced_update.battle_state_revision;

    let recovered = core
        .execute(player_id, PlayerBehavior::RecoverBattleSetupLoss)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: recovered_battle_uuid,
        battle_setup_snapshot: Some(recovered_setup),
        battle_update: recovered_update,
        ..
    } = recovered
    else {
        panic!("expected field work setup-loss recovery projection");
    };
    assert_eq!(recovered_battle_uuid, battle_uuid);
    assert_eq!(recovered_setup.field_work_objects, setup.field_work_objects);
    assert_eq!(recovered_update.battle_state_revision, current_revision);
    assert_eq!(
        recovered_update.checkpoint.field_work_objects,
        advanced_update.checkpoint.field_work_objects
    );
    let recovered_unit = recovered_update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.unit_instance_id == worker_unit_instance_id)
        .expect("recovered worker checkpoint");
    assert_eq!(
        recovered_unit.tactical_state.as_ref().unwrap().activity,
        crate::game::battle::types::UnitExclusiveActivity::FieldWorking
    );
}

#[test]
fn disconnect_auto_pause_is_idempotent_and_requires_explicit_resume() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0100);
    let player_id = Uuid::from_u128(0xD3F3_0100);
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

    let revision_before = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_state_revision;
    assert!(core.auto_pause_active_battle_for_disconnect().unwrap());
    let active = core.state.active_battle.as_ref().unwrap();
    assert!(active.playback.paused);
    assert_eq!(active.battle_state_revision, revision_before + 1);

    assert!(!core.auto_pause_active_battle_for_disconnect().unwrap());
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .battle_state_revision,
        revision_before + 1
    );
    assert!(core
        .advance_active_battle_for_server_tick(50_000)
        .unwrap()
        .is_none());

    let resumed =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle)
            .unwrap();
    let BehaviorResult::BattlePlaybackChanged { battle_update, .. } = resumed else {
        panic!("explicit ResumeBattle should publish playback state");
    };
    assert!(!battle_update.checkpoint.playback.paused);
    assert_eq!(battle_update.battle_state_revision, revision_before + 2);
}

#[test]
fn live_defense_playback_pause_freezes_server_tick_and_resume_advances() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0101);
    let player_id = Uuid::from_u128(0xD3F3_0101);
    start_new_game_with_legacy_sword_fixture(&mut core, player_id);
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
    let paused_resource = battle_update.checkpoint.tactical_resource;
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
    assert_eq!(
        paused_snapshot["game_state_context"]["tactical_resource"]["current"],
        serde_json::json!(paused_resource.current)
    );
    assert!(paused_snapshot["game_state_context"]["deployment"]
        .get("current_cost")
        .is_none());
    let mut legacy_root_context = paused_snapshot["game_state_context"].clone();
    legacy_root_context
        .as_object_mut()
        .unwrap()
        .remove("tactical_resource");
    assert!(
        serde_json::from_value::<crate::game::behavior::GameStateContextDto>(legacy_root_context)
            .is_err(),
        "tactical_resource is required in the root in-battle context"
    );

    let paused_tick = core.advance_active_battle_for_server_tick(50_000).unwrap();
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
    assert_eq!(battle_update.checkpoint.tactical_resource, paused_resource);
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
        .advance_active_battle_for_server_tick(1_000)
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
    assert_eq!(battle_time_ms, 1_000);
    assert_eq!(
        battle_update.checkpoint.tactical_resource.current,
        paused_resource.current.saturating_add(1),
        "paused wall time must not recover or catch up after resume"
    );
    assert_eq!(
        battle_update.checkpoint.tactical_resource.maximum,
        paused_resource.maximum
    );
    assert_eq!(battle_update.battle_state_revision, 5);
}

#[test]
fn paused_live_battle_commits_deploy_and_withdraw_without_advancing_time() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD3F3_0108);
    let player_id = Uuid::from_u128(0xD3F3_0108);
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
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids().into_iter()
        .find(|id| core.roster().unwrap().get(id).unwrap().employee_definition_id == "records_clerk")
        .expect("the selected sword dealer deploys on Ground");
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let paused =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    let BehaviorResult::BattlePlaybackChanged {
        battle_update: paused_update,
        ..
    } = paused
    else {
        panic!("expected paused battle update");
    };
    let frozen_time_ms = paused_update.checkpoint.battle_time_ms;
    let paused_revision = paused_update.battle_state_revision;
    let initial_resource = paused_update.checkpoint.tactical_resource.current;
    assert!(paused_update.checkpoint.playback.paused);
    assert_eq!(frozen_time_ms, 0);

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
        panic!("expected paused deployment update");
    };
    let deployed_resource = deployed_update.checkpoint.tactical_resource.current;
    assert_eq!(deployed_update.battle_state_revision, paused_revision + 1);
    assert_eq!(deployed_update.checkpoint.battle_time_ms, frozen_time_ms);
    assert!(deployed_update.checkpoint.playback.paused);
    assert!(deployed_resource < initial_resource);

    let before_rejected_deploy =
        serde_json::to_value(core.active_battle_reconnect_projection(0).unwrap()).unwrap();
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect_err("an already deployed employee must be rejected while paused");
    let after_rejected_deploy =
        serde_json::to_value(core.active_battle_reconnect_projection(0).unwrap()).unwrap();
    assert_eq!(after_rejected_deploy, before_rejected_deploy);

    let withdrawn = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::WithdrawUnit { employee_uuid },
    )
    .unwrap();
    let BehaviorResult::BattleUnitWithdrawn {
        battle_update: withdrawn_update,
        ..
    } = withdrawn
    else {
        panic!("expected paused withdrawal update");
    };
    assert_eq!(
        withdrawn_update.battle_state_revision,
        deployed_update.battle_state_revision + 1
    );
    assert_eq!(withdrawn_update.checkpoint.battle_time_ms, frozen_time_ms);
    assert!(withdrawn_update.checkpoint.playback.paused);
    let withdrawn_redeploy = &withdrawn_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live deployment state after withdrawal")
        .redeploying_units[0];
    assert_eq!(
        withdrawn_redeploy.ready_at_ms,
        frozen_time_ms + run_policy().live_deployment.withdraw_redeploy_cooldown_ms
    );
    assert_eq!(
        withdrawn_update.checkpoint.tactical_resource.current, deployed_resource,
        "withdrawal must not refund the committed deployment cost"
    );

    let before_rejected_withdraw =
        serde_json::to_value(core.active_battle_reconnect_projection(0).unwrap()).unwrap();
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::WithdrawUnit { employee_uuid },
    )
    .expect_err("an already withdrawn employee must be rejected while paused");
    let after_rejected_withdraw =
        serde_json::to_value(core.active_battle_reconnect_projection(0).unwrap()).unwrap();
    assert_eq!(after_rejected_withdraw, before_rejected_withdraw);

    assert!(core
        .advance_active_battle_for_server_tick(10)
        .unwrap()
        .is_none());
    let state = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState { since_seq: None },
        )
        .unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = state else {
        panic!("expected latest paused battle state");
    };
    assert_eq!(
        battle_update.battle_state_revision,
        withdrawn_update.battle_state_revision
    );
    assert_eq!(battle_update.checkpoint.battle_time_ms, frozen_time_ms);
    assert!(battle_update.checkpoint.playback.paused);
    let reconnect_redeploy = &battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("reconnect deployment state")
        .redeploying_units[0];
    assert_eq!(
        reconnect_redeploy.ready_at_ms,
        withdrawn_redeploy.ready_at_ms
    );
    assert_eq!(
        reconnect_redeploy.deploy_cost,
        withdrawn_redeploy.deploy_cost
    );
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
    start_new_game_with_legacy_sword_fixture(&mut core, player_id);
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

    assert_eq!(
        core.get_action_capabilities(),
        vec![
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicState,
                crate::game::behavior::ActionScope::Query
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicNumericPlan,
                crate::game::behavior::ActionScope::Query
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicEffectivePlan,
                crate::game::behavior::ActionScope::Query
            ),
        ]
    );
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
    start_new_game_with_legacy_sword_fixture(&mut core, player_id);
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
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
    let employee_uuid = employee_ids[0];

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
    let before_resource = before_update.checkpoint.tactical_resource.current;
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
        vec![Position::new(deploy_position.x + 1, deploy_position.y)]
    );
    assert!(!result.facings.right.range_previews.active_skill.available);
    // The typed result always carries every facing. A one-cell forward weapon can
    // legitimately project an empty range when a rotated cell falls off-grid.
    let _all_facings = (
        &result.facings.up,
        &result.facings.down,
        &result.facings.left,
        &result.facings.right,
    );

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
    assert_eq!(
        after_update.checkpoint.tactical_resource.current,
        before_resource
    );
    assert!(after_deployment.deployed_units.is_empty());
    assert!(!after_update
        .checkpoint
        .units
        .iter()
        .any(|unit| matches!(unit.unit_source, BattleUnitSourceIdentity::Employee { .. })));
}

#[test]
fn control_status_checkpoint_reconnect_projects_exact_derived_state_and_requires_shape() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xCC02_0001);
    let player_id = Uuid::from_u128(0xCC02_0002);
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
        unit_instance_id, ..
    } = deployed
    else {
        panic!("expected live deploy result");
    };

    core.advance_active_battle_by(5)
        .expect("battle clock should advance before reconnect projection");
    let battle_time_ms = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .execution
        .last_event_time_ms();
    {
        let active = core.state.active_battle.as_mut().unwrap();
        for (buff_name, duration_ms) in [("stun", 100), ("freeze", 200), ("silence", 300)] {
            active
                .battle
                .process_event_for_test(
                    BattleEvent::ApplyBuff {
                        time_ms: battle_time_ms,
                        caster_instance_id: unit_instance_id,
                        target_instance_id: unit_instance_id,
                        buff_id: BuffId::from_name(buff_name),
                        duration_ms,
                        cause: BattleEventCause::default(),
                    },
                    battle_time_ms,
                )
                .unwrap();
        }
    }

    let update = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle_update_dto_after(0, &core.state.roster)
        .unwrap();
    assert_eq!(update.checkpoint.battle_time_ms, battle_time_ms);
    assert_eq!(
        update.checkpoint.tactical_resource,
        crate::game::behavior::LiveBattleTacticalResourceDto {
            current: run_policy().tactical_resource.initial
                - run_policy().tactical_resource.base_deploy_cost,
            maximum: run_policy().tactical_resource.maximum,
            recovery_per_second: run_policy().tactical_resource.recovery_per_second,
        }
    );
    let unit = update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.unit_instance_id == unit_instance_id)
        .expect("controlled unit should be present in reconnect checkpoint");
    assert!(unit.control.hard_cc_active);
    assert_eq!(
        unit.control.hard_cc_release_time_ms,
        Some(battle_time_ms.saturating_add(200))
    );
    assert!(unit.control.silenced);
    assert_eq!(
        unit.control.skill_start_disabled_reason,
        Some(ActionDisabledReasonCode::HardCrowdControl)
    );

    let checkpoint_json = serde_json::to_value(&update.checkpoint).unwrap();
    let mut missing_resource_json = checkpoint_json.clone();
    missing_resource_json
        .as_object_mut()
        .unwrap()
        .remove("tactical_resource");
    assert!(
        serde_json::from_value::<crate::game::behavior::LiveBattleStateCheckpointDto>(
            missing_resource_json
        )
        .is_err(),
        "tactical_resource is a required checkpoint field"
    );

    let mut legacy_deployment_json = checkpoint_json.clone();
    legacy_deployment_json["deployment"]
        .as_object_mut()
        .unwrap()
        .insert("current_cost".to_string(), serde_json::json!(10));
    assert!(
        serde_json::from_value::<crate::game::behavior::LiveBattleStateCheckpointDto>(
            legacy_deployment_json
        )
        .is_err(),
        "legacy resource fields must not parse through deployment"
    );

    let mut checkpoint_json = checkpoint_json;
    let controlled_json = checkpoint_json["units"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|unit| unit["control"]["hard_cc_active"] == serde_json::Value::Bool(true))
        .expect("serialized checkpoint should include the control projection");
    controlled_json.as_object_mut().unwrap().remove("control");
    assert!(
        serde_json::from_value::<crate::game::behavior::LiveBattleStateCheckpointDto>(
            checkpoint_json
        )
        .is_err()
    );
}

#[test]
fn all_weaponless_roster_uses_existing_no_deployable_run_rule() {
    let mut core = GameCore::new(empty_game_data(), 0xA11C_E003);
    let player_id = Uuid::from_u128(0xA11C_E003);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
    for employee_uuid in employee_ids {
        force_unequip_employee_weapon(&mut core, employee_uuid);
    }
    assert_eq!(
        core.current_run_failure_reason().unwrap(),
        Some(RunFailureReason::NoDeployableEmployees)
    );
}

#[test]
fn weaponless_employee_is_listed_but_battle_actions_reject_with_condition_priority() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0xA11C_E001);
    let player_id = Uuid::from_u128(0xA11C_E001);
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
    force_unequip_employee_weapon(&mut core, employee_uuid);

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let candidate = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .live_deployment_dto(core.roster().unwrap())
        .unwrap()
        .unit_deploy_costs
        .into_iter()
        .find(|candidate| candidate.employee_uuid == employee_uuid)
        .expect("weaponless employee remains visible in the formation candidate list");
    assert_eq!(
        candidate.eligibility.disabled_reason_code,
        Some(ActionDisabledReasonCode::WeaponRequired)
    );
    let root = core.get_run_snapshot_json().unwrap();
    let root_candidate = root["game_state_context"]["deployment"]["unit_deploy_costs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| candidate["employee_uuid"] == serde_json::json!(employee_uuid))
        .expect("root checkpoint must preserve weaponless formation candidate");
    assert_eq!(
        root_candidate["eligibility"]["disabled_reason_code"],
        "weapon_required"
    );

    let (resource_before, revision_before, seq_before) = {
        let active = core.state.active_battle.as_ref().unwrap();
        (
            active.tactical_resource.current(),
            active.battle_state_revision,
            active.latest_event_log_seq(),
        )
    };
    let deploy_error = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect_err("weaponless deployment must fail closed");
    assert!(matches!(
        deploy_error,
        GameError::ActionUnavailable {
            action: ActionKind::DeployUnit,
            reason: ActionDisabledReasonCode::WeaponRequired,
        }
    ));
    let range_error = core
        .execute(
            player_id,
            PlayerBehavior::RequestDeploymentRangePreview {
                employee_uuid,
                position: deploy_position,
            },
        )
        .expect_err("weaponless range preview must fail closed");
    assert!(matches!(
        range_error,
        GameError::ActionUnavailable {
            action: ActionKind::RequestDeploymentRangePreview,
            reason: ActionDisabledReasonCode::WeaponRequired,
        }
    ));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.tactical_resource.current(), resource_before);
    assert_eq!(active.battle_state_revision, revision_before);
    assert_eq!(active.latest_event_log_seq(), seq_before);
    assert!(active
        .live_deployment
        .as_ref()
        .unwrap()
        .redeploy_locks
        .is_empty());

    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .employee_attempt
        .roster
        .get_mut(&employee_uuid)
        .unwrap()
        .condition
        .physical_damage = crate::game::employee::MAX_PHYSICAL_DAMAGE - 1;
    let condition_error = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect_err("condition reason must outrank weapon_required");
    assert!(
        matches!(
            &condition_error,
            GameError::ActionUnavailable {
                action: ActionKind::DeployUnit,
                reason: ActionDisabledReasonCode::CriticalPhysicalDamage,
            }
        ),
        "unexpected condition-priority error: {condition_error:?}"
    );
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.tactical_resource.current(), resource_before);
    assert_eq!(active.battle_state_revision, revision_before);
    assert_eq!(active.latest_event_log_seq(), seq_before);
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
        vec![Position::new(deploy_position.x + 1, deploy_position.y)]
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
    assert!(!battle_update.action_capabilities.is_empty());
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live deploy update should expose deployment state");
    assert_eq!(
        battle_update.checkpoint.tactical_resource.current,
        run_policy()
            .tactical_resource
            .initial
            .saturating_sub(run_policy().tactical_resource.base_deploy_cost)
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
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .tactical_resource
        .set_current_for_test(run_policy().tactical_resource.base_deploy_cost);
    let second_deploy_position = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .combat_preview
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .find(|position| *position != deploy_position)
        .expect("fixture should expose a second ground deployment cell");
    let (
        revision_before_occupied_deploy,
        resource_before_occupied_deploy,
        seq_before_occupied_deploy,
    ) = {
        let active = core.state.active_battle.as_ref().unwrap();
        (
            active.battle_state_revision,
            active.tactical_resource.current(),
            active.latest_event_log_seq(),
        )
    };
    let occupied_error = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: second_employee_uuid,
            position: deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect_err("active allied tactical slots must reject a second deployment");
    assert!(matches!(occupied_error, GameError::PositionOccupied));
    {
        let active = core.state.active_battle.as_ref().unwrap();
        assert_eq!(
            active.battle_state_revision,
            revision_before_occupied_deploy
        );
        assert_eq!(
            active.tactical_resource.current(),
            resource_before_occupied_deploy
        );
        assert_eq!(active.latest_event_log_seq(), seq_before_occupied_deploy);
    }
    let second_deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: second_employee_uuid,
            position: second_deploy_position,
            facing: FacingDirection::Right,
        },
    )
    .expect("a distinct allied deployment tile should remain valid");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: second_unit_instance_id,
        ..
    } = second_deployed
    else {
        panic!("expected second live deploy result");
    };
    let (expected_withdraw_redeploy_hp, initial_runtime_state) = {
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
        let initial_runtime_state = (
            unit.resonance_current,
            unit.next_action_time,
            unit.next_basic_attack_ms,
        );
        let damaged_hp = unit.stats.max_health / 2;
        unit.stats.current_health = damaged_hp;
        unit.resonance_current = if unit.resonance_current == 0 {
            unit.resonance_max
        } else {
            0
        };
        unit.next_action_time = 999_999;
        unit.next_basic_attack_ms = 999_999;
        unit.current_target = Some(second_unit_instance_id);
        unit.pending_basic_attack = true;
        unit.pending_cast = true;
        let recovered_hp = damaged_hp
            .saturating_add(
                unit.stats.max_health.saturating_mul(
                    BattleDeployCurrentHpPolicy::WITHDRAW_REDEPLOY_RECOVERY_PERCENT,
                ) / 100,
            )
            .min(unit.stats.max_health);
        (recovered_hp, initial_runtime_state)
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
    assert!(!battle_update.action_capabilities.is_empty());
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live withdraw update should expose deployment state");
    assert_eq!(withdrawn_unit, unit_instance_id);
    assert!(deployment.deployed_units.iter().any(|unit| {
        unit.employee_uuid == second_employee_uuid
            && unit.unit_instance_id == second_unit_instance_id
            && unit.position == second_deploy_position
    }));
    assert_eq!(deployment.redeploying_units.len(), 1);
    assert_eq!(deployment.redeploying_units[0].employee_uuid, employee_uuid);
    assert_eq!(deployment.redeploying_units[0].ready_at_ms, 30_000);
    assert_eq!(
        deployment.redeploying_units[0].deploy_cost,
        (u64::from(run_policy().tactical_resource.base_deploy_cost)
            * u64::from(run_policy().tactical_resource.redeploy_cost_multiplier_pct))
        .div_ceil(100) as u32
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
        let deploy_cost = deployment
            .redeploy_locks
            .get(&employee_uuid)
            .expect("withdraw should create redeploy lock")
            .deploy_cost;
        deployment
            .redeploy_locks
            .get_mut(&employee_uuid)
            .expect("withdraw should create redeploy lock")
            .ready_at_ms = ready_at_ms;
        active.tactical_resource.set_current_for_test(deploy_cost);
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
    assert_eq!(
        battle_update.checkpoint.tactical_resource.current, 0,
        "redeploy must commit its full locked price exactly once"
    );
    assert!(deployment.redeploying_units.is_empty());
    let redeployed_projection = deployment
        .deployed_units
        .iter()
        .find(|unit| {
            unit.employee_uuid == employee_uuid
                && unit.unit_instance_id == redeployed_unit_instance_id
        })
        .expect("redeployed employee must be projected");
    assert_eq!(
        redeployed_projection.tactical_state.stance,
        EngagementStance::Attack
    );
    assert_eq!(
        redeployed_projection
            .tactical_state
            .relocation_cooldown_until_ms,
        0,
        "redeploy creates a new runtime instance with no inherited relocation cooldown"
    );
    assert_eq!(
        redeployed_projection
            .tactical_state
            .relocation_eligibility
            .disabled_reason_code,
        Some(ActionDisabledReasonCode::InsufficientResources)
    );
    assert_eq!(
        redeployed_projection
            .tactical_state
            .stance_cooldown_until_ms,
        battle_update.checkpoint.battle_time_ms + 1_500
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
    assert_eq!(
        (
            redeployed_unit.resonance_current,
            redeployed_unit.next_action_time,
            redeployed_unit.next_basic_attack_ms,
        ),
        initial_runtime_state,
        "redeploy must rebuild initial charge/cooldowns instead of inheriting transient state"
    );
}

#[test]
fn live_defense_stance_command_projects_cooldown_focus_pause_and_atomic_rejections() {
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
    let undeployed_employee_uuid = employee_ids[1];
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
        battle_update,
        ..
    } = deployed
    else {
        panic!("expected deploy result");
    };
    let deployed_tactical = &battle_update
        .checkpoint
        .deployment
        .as_ref()
        .unwrap()
        .deployed_units[0]
        .tactical_state;
    assert_eq!(deployed_tactical.stance, EngagementStance::Attack);
    assert_eq!(deployed_tactical.tile_effect, LiveBattleTileEffectDto::None);
    assert_eq!(deployed_tactical.stance_cooldown_until_ms, 1_500);
    assert!(!deployed_tactical.stance_eligibility.defense.enabled);
    assert_eq!(
        deployed_tactical
            .stance_eligibility
            .defense
            .disabled_reason_code,
        Some(ActionDisabledReasonCode::StanceCooldown)
    );

    let (revision_before, seq_before, resource_before) = {
        let active = core.state.active_battle.as_ref().unwrap();
        (
            active.battle_state_revision,
            active.latest_event_log_seq(),
            active.tactical_resource.current(),
        )
    };
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::SetEngagementStance {
                employee_uuid,
                stance: EngagementStance::Defense,
            },
        ),
        Err(GameError::ActionUnavailable {
            action: ActionKind::SetEngagementStance,
            reason: ActionDisabledReasonCode::StanceCooldown,
        })
    ));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.battle_state_revision, revision_before);
    assert_eq!(active.latest_event_log_seq(), seq_before);
    assert_eq!(active.tactical_resource.current(), resource_before);
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::SetEngagementStance {
                employee_uuid: undeployed_employee_uuid,
                stance: EngagementStance::Defense,
            },
        ),
        Err(GameError::ActionUnavailable {
            action: ActionKind::SetEngagementStance,
            reason: ActionDisabledReasonCode::UnitUnavailable,
        })
    ));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.battle_state_revision, revision_before);
    assert_eq!(active.latest_event_log_seq(), seq_before);
    assert_eq!(active.tactical_resource.current(), resource_before);

    core.advance_active_battle_by(1_500).unwrap();
    let resource_before_toggle = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .tactical_resource
        .current();
    let changed = execute_current_battle_mutation_with_source(
        &mut core,
        player_id,
        PlayerBehavior::SetEngagementStance {
            employee_uuid,
            stance: EngagementStance::Defense,
        },
        "stance-defense",
    )
    .unwrap();
    let BehaviorResult::BattleEngagementStanceChanged {
        stance,
        battle_update,
        ..
    } = changed
    else {
        panic!("expected stance result");
    };
    assert!(!battle_update.action_capabilities.is_empty());
    assert_eq!(stance, EngagementStance::Defense);
    assert_eq!(
        battle_update.checkpoint.tactical_resource.current, resource_before_toggle,
        "stance changes must not consume tactical resource"
    );
    let tactical = battle_update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.unit_instance_id == unit_instance_id)
        .and_then(|unit| unit.tactical_state)
        .expect("deployed unit checkpoint must expose tactical state");
    assert_eq!(tactical.stance, EngagementStance::Defense);
    assert_eq!(tactical.tile_effect, LiveBattleTileEffectDto::DefenseFocus);
    assert_eq!(tactical.stance_cooldown_until_ms, 3_000);
    assert_eq!(
        battle_update
            .checkpoint
            .units
            .iter()
            .find(|unit| unit.unit_instance_id == unit_instance_id)
            .unwrap()
            .control
            .skill_start_disabled_reason,
        Some(ActionDisabledReasonCode::DefenseStance)
    );
    assert!(battle_update.events_delta.events.iter().any(|entry| {
        entry.source_command_id.as_deref() == Some("stance-defense")
            && matches!(
                entry.event,
                LiveBattlePresentationEventKindDto::StanceChanged {
                    unit_instance_id: event_unit_id,
                    before: EngagementStance::Attack,
                    after: EngagementStance::Defense,
                    cooldown_until_ms: 3_000,
                } if event_unit_id == unit_instance_id
            )
    }));

    let (revision_before_same, seq_before_same, resource_before_same) = {
        let active = core.state.active_battle.as_ref().unwrap();
        (
            active.battle_state_revision,
            active.latest_event_log_seq(),
            active.tactical_resource.current(),
        )
    };
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::SetEngagementStance {
                employee_uuid,
                stance: EngagementStance::Defense,
            },
        ),
        Err(GameError::ActionUnavailable {
            reason: ActionDisabledReasonCode::AlreadyActive,
            ..
        })
    ));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.battle_state_revision, revision_before_same);
    assert_eq!(active.latest_event_log_seq(), seq_before_same);
    assert_eq!(active.tactical_resource.current(), resource_before_same);

    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    assert!(core
        .advance_active_battle_for_server_tick(50_000)
        .unwrap()
        .is_none());
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .execution
            .last_event_time_ms(),
        1_500
    );
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::SetEngagementStance {
                employee_uuid,
                stance: EngagementStance::Attack,
            },
        ),
        Err(GameError::ActionUnavailable {
            reason: ActionDisabledReasonCode::StanceCooldown,
            ..
        })
    ));
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle).unwrap();
    core.advance_active_battle_by(1_500).unwrap();
    let changed_back = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::SetEngagementStance {
            employee_uuid,
            stance: EngagementStance::Attack,
        },
    )
    .unwrap();
    let BehaviorResult::BattleEngagementStanceChanged { battle_update, .. } = changed_back else {
        panic!("expected attack stance result");
    };
    let tactical = battle_update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.unit_instance_id == unit_instance_id)
        .and_then(|unit| unit.tactical_state)
        .unwrap();
    assert_eq!(tactical.stance, EngagementStance::Attack);
    assert_eq!(tactical.tile_effect, LiveBattleTileEffectDto::None);
}

#[test]
fn live_tactical_relocation_is_revisioned_costed_pause_safe_and_atomic() {
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
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let relocation_destination = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .combat_preview
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .find(|position| *position != deploy_position)
        .expect("fixture should expose a relocation destination");

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
        unit_instance_id, ..
    } = deployed
    else {
        panic!("expected deploy result");
    };
    let tactical = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .live_deployment_dto(&core.state.roster)
        .unwrap()
        .deployed_units[0]
        .tactical_state;
    assert_eq!(tactical.relocation_eligibility.disabled_reason_code, None);

    core.advance_active_battle_by(1_500).unwrap();
    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::SetEngagementStance {
            employee_uuid,
            stance: EngagementStance::Defense,
        },
    )
    .unwrap();
    let (outside_deployment_position, platform_position) = {
        let active = core.state.active_battle.as_ref().unwrap();
        let deployment_cells = active
            .combat_preview
            .deployment_zones
            .iter()
            .flat_map(|zone| zone.cells.iter().copied())
            .collect::<std::collections::HashSet<_>>();
        let outside = active
            .battle
            .battlefield
            .valid_positions()
            .into_iter()
            .find(|position| !deployment_cells.contains(position))
            .expect("fixture should expose a non-deployment battlefield cell");
        let platform = active
            .combat_preview
            .deployment_zones
            .iter()
            .filter(|zone| zone.kind == DeploymentZoneKind::Platform)
            .flat_map(|zone| zone.cells.iter().copied())
            .next()
            .expect("fixture should expose a platform deployment cell");
        (outside, platform)
    };
    let (
        revision_before_destination_rejections,
        seq_before_destination_rejections,
        resource_before_destination_rejections,
    ) = {
        let active = core.state.active_battle.as_ref().unwrap();
        (
            active.battle_state_revision,
            active.latest_event_log_seq(),
            active.tactical_resource.current(),
        )
    };
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::TacticalRelocateUnit {
                employee_uuid,
                destination: outside_deployment_position,
                facing: FacingDirection::Right,
            },
        ),
        Err(GameError::OutOfBounds)
    ));
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::TacticalRelocateUnit {
                employee_uuid,
                destination: platform_position,
                facing: FacingDirection::Right,
            },
        ),
        Err(GameError::InvalidAction)
    ));
    {
        let active = core.state.active_battle.as_ref().unwrap();
        assert_eq!(
            active.battle_state_revision,
            revision_before_destination_rejections
        );
        assert_eq!(
            active.latest_event_log_seq(),
            seq_before_destination_rejections
        );
        assert_eq!(
            active.tactical_resource.current(),
            resource_before_destination_rejections
        );
    }
    {
        let active = core.state.active_battle.as_mut().unwrap();
        active
            .tactical_resource
            .set_current_for_test(run_policy().tactical_relocation.cost - 1);
    }
    let (revision_before_resource_rejection, seq_before_resource_rejection) = {
        let active = core.state.active_battle.as_ref().unwrap();
        let tactical = active
            .live_deployment_dto(&core.state.roster)
            .unwrap()
            .deployed_units[0]
            .tactical_state;
        assert_eq!(
            tactical.relocation_eligibility.disabled_reason_code,
            Some(ActionDisabledReasonCode::InsufficientResources)
        );
        (active.battle_state_revision, active.latest_event_log_seq())
    };
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::TacticalRelocateUnit {
                employee_uuid,
                destination: relocation_destination,
                facing: FacingDirection::Right,
            },
        ),
        Err(GameError::ActionUnavailable {
            action: ActionKind::TacticalRelocateUnit,
            reason: ActionDisabledReasonCode::InsufficientResources,
        })
    ));
    {
        let active = core.state.active_battle.as_ref().unwrap();
        assert_eq!(
            active.battle_state_revision,
            revision_before_resource_rejection
        );
        assert_eq!(active.latest_event_log_seq(), seq_before_resource_rejection);
        assert_eq!(
            active.tactical_resource.current(),
            run_policy().tactical_relocation.cost - 1
        );
    }
    {
        let active = core.state.active_battle.as_mut().unwrap();
        active.tactical_resource.set_current_for_test(10);
        let unit = active.battle.units.get_mut(&unit_instance_id).unwrap();
        unit.stats.current_health = 43;
        unit.resonance_current = 17;
        unit.next_basic_attack_ms = 8_000;
        unit.ranged_reposition_until_ms = 9_000;
    }
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    assert!(core
        .advance_active_battle_for_server_tick(50_000)
        .unwrap()
        .is_none());
    let (revision_before, seq_before, resource_before) = {
        let active = core.state.active_battle.as_ref().unwrap();
        (
            active.battle_state_revision,
            active.latest_event_log_seq(),
            active.tactical_resource.current(),
        )
    };

    let relocated = execute_current_battle_mutation_with_source(
        &mut core,
        player_id,
        PlayerBehavior::TacticalRelocateUnit {
            employee_uuid,
            destination: relocation_destination,
            facing: FacingDirection::Right,
        },
        "relocate-paused",
    )
    .unwrap();
    let BehaviorResult::BattleUnitTacticallyRelocated {
        destination,
        facing,
        battle_update,
        ..
    } = relocated
    else {
        panic!("expected relocation result");
    };
    assert!(!battle_update.action_capabilities.is_empty());
    assert_eq!(destination, relocation_destination);
    assert_eq!(facing, FacingDirection::Right);
    assert_eq!(battle_update.checkpoint.battle_time_ms, 1_500);
    assert!(battle_update.checkpoint.playback.paused);
    assert_eq!(battle_update.battle_state_revision, revision_before + 1);
    assert_eq!(
        battle_update.checkpoint.tactical_resource.current,
        resource_before - run_policy().tactical_relocation.cost
    );
    let deployed = &battle_update
        .checkpoint
        .deployment
        .as_ref()
        .unwrap()
        .deployed_units[0];
    assert_eq!(deployed.position, relocation_destination);
    assert_eq!(deployed.facing, FacingDirection::Right);
    assert_eq!(deployed.tactical_state.stance, EngagementStance::Defense);
    assert_eq!(
        deployed.tactical_state.tile_effect,
        LiveBattleTileEffectDto::DefenseFocus
    );
    assert_eq!(deployed.tactical_state.stance_cooldown_until_ms, 3_000);
    assert_eq!(deployed.tactical_state.relocation_cooldown_until_ms, 11_500);
    assert_eq!(
        deployed
            .tactical_state
            .relocation_eligibility
            .disabled_reason_code,
        Some(ActionDisabledReasonCode::RelocationCooldown)
    );
    assert!(battle_update.events_delta.events.iter().any(|entry| {
        entry.source_command_id.as_deref() == Some("relocate-paused")
            && matches!(
                entry.event,
                LiveBattlePresentationEventKindDto::UnitTacticallyRelocated {
                    unit_instance_id: event_unit_id,
                    from_position,
                    to_position,
                    cooldown_until_ms: 11_500,
                    ..
                } if event_unit_id == unit_instance_id
                    && from_position == deploy_position
                    && to_position == relocation_destination
            )
    }));
    {
        let active = core.state.active_battle.as_ref().unwrap();
        let unit = &active.battle.units[&unit_instance_id];
        assert_eq!(unit.stats.current_health, 43);
        assert_eq!(unit.resonance_current, 17);
        assert_eq!(unit.next_basic_attack_ms, 8_000);
        assert_eq!(unit.ranged_reposition_until_ms, 9_000);
    }

    let revision_after = battle_update.battle_state_revision;
    let seq_after = battle_update.events_delta.to_seq;
    let resource_after = battle_update.checkpoint.tactical_resource.current;
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::TacticalRelocateUnit {
                employee_uuid,
                destination: deploy_position,
                facing: FacingDirection::Left,
            },
        ),
        Err(GameError::ActionUnavailable {
            action: ActionKind::TacticalRelocateUnit,
            reason: ActionDisabledReasonCode::RelocationCooldown,
        })
    ));
    {
        let active = core.state.active_battle.as_ref().unwrap();
        assert_eq!(active.battle_state_revision, revision_after);
        assert_eq!(active.latest_event_log_seq(), seq_after);
        assert_eq!(active.tactical_resource.current(), resource_after);
        assert_eq!(active.latest_event_log_seq(), seq_before + 1);
    }

    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle).unwrap();
    core.advance_active_battle_by(10_000).unwrap();
    {
        let active = core.state.active_battle.as_mut().unwrap();
        active
            .tactical_resource
            .set_current_for_test(run_policy().tactical_relocation.cost);
    }
    let (revision_before_noop, seq_before_noop, resource_before_noop) = {
        let active = core.state.active_battle.as_ref().unwrap();
        (
            active.battle_state_revision,
            active.latest_event_log_seq(),
            active.tactical_resource.current(),
        )
    };
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::TacticalRelocateUnit {
                employee_uuid,
                destination: relocation_destination,
                facing: FacingDirection::Right,
            },
        ),
        Err(GameError::ActionUnavailable {
            reason: ActionDisabledReasonCode::AlreadyActive,
            ..
        })
    ));
    {
        let active = core.state.active_battle.as_ref().unwrap();
        assert_eq!(active.battle_state_revision, revision_before_noop);
        assert_eq!(active.latest_event_log_seq(), seq_before_noop);
        assert_eq!(active.tactical_resource.current(), resource_before_noop);
    }

    let facing_only = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::TacticalRelocateUnit {
            employee_uuid,
            destination: relocation_destination,
            facing: FacingDirection::Left,
        },
    )
    .expect("same-tile facing-only relocation should pay and start cooldown");
    let BehaviorResult::BattleUnitTacticallyRelocated { battle_update, .. } = facing_only else {
        panic!("expected facing-only relocation result");
    };
    assert_eq!(battle_update.checkpoint.tactical_resource.current, 0);
    assert_eq!(
        battle_update
            .checkpoint
            .deployment
            .as_ref()
            .unwrap()
            .deployed_units[0]
            .facing,
        FacingDirection::Left
    );
}

#[test]
fn live_tactical_swap_costs_once_updates_once_and_works_while_paused() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 0x5c03);
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
    let first_position = ground_deployment_cell_with_right_range(&preview);
    let employees = core.roster().unwrap().combat_eligible_employee_ids();
    let first_employee_uuid = employees[0];
    let second_employee_uuid = employees[1];
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let second_position = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .combat_preview
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .find(|position| *position != first_position)
        .expect("fixture must expose two ground deployment cells");
    let first_deploy = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: first_employee_uuid,
            position: first_position,
            facing: FacingDirection::Right,
        },
    )
    .unwrap();
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: first_unit_id,
        ..
    } = first_deploy
    else {
        panic!("expected first deployment");
    };
    let second_deploy = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: second_employee_uuid,
            position: second_position,
            facing: FacingDirection::Left,
        },
    )
    .unwrap();
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: second_unit_id,
        ..
    } = second_deploy
    else {
        panic!("expected second deployment");
    };
    core.advance_active_battle_by(1_500).unwrap();
    for employee_uuid in [first_employee_uuid, second_employee_uuid] {
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::SetEngagementStance {
                employee_uuid,
                stance: EngagementStance::Defense,
            },
        )
        .unwrap();
    }
    {
        let active = core.state.active_battle.as_mut().unwrap();
        active.tactical_resource.set_current_for_test(10);
        active
            .battle
            .units
            .get_mut(&first_unit_id)
            .unwrap()
            .stats
            .current_health = 43;
        active
            .battle
            .units
            .get_mut(&second_unit_id)
            .unwrap()
            .stats
            .current_health = 57;
    }
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    let (revision_before, seq_before) = {
        let active = core.state.active_battle.as_ref().unwrap();
        (active.battle_state_revision, active.latest_event_log_seq())
    };
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::TacticalSwapUnits {
                first_employee_uuid,
                second_employee_uuid: first_employee_uuid,
                first_facing: FacingDirection::Up,
                second_facing: FacingDirection::Down,
            },
        ),
        Err(GameError::ActionUnavailable {
            action: ActionKind::TacticalSwapUnits,
            reason: ActionDisabledReasonCode::AlreadyActive,
        })
    ));
    {
        let active = core.state.active_battle.as_ref().unwrap();
        assert_eq!(active.battle_state_revision, revision_before);
        assert_eq!(active.latest_event_log_seq(), seq_before);
        assert_eq!(active.tactical_resource.current(), 10);
    }
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .tactical_resource
        .set_current_for_test(4);
    assert!(matches!(
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::TacticalSwapUnits {
                first_employee_uuid,
                second_employee_uuid,
                first_facing: FacingDirection::Up,
                second_facing: FacingDirection::Down,
            },
        ),
        Err(GameError::ActionUnavailable {
            action: ActionKind::TacticalSwapUnits,
            reason: ActionDisabledReasonCode::InsufficientResources,
        })
    ));
    {
        let active = core.state.active_battle.as_ref().unwrap();
        assert_eq!(active.battle_state_revision, revision_before);
        assert_eq!(active.latest_event_log_seq(), seq_before);
        assert_eq!(active.tactical_resource.current(), 4);
        assert_eq!(
            active.battle.unit_projected_tile(first_unit_id),
            Some(first_position)
        );
        assert_eq!(
            active.battle.unit_projected_tile(second_unit_id),
            Some(second_position)
        );
    }
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .tactical_resource
        .set_current_for_test(10);

    let swapped = execute_current_battle_mutation_with_source(
        &mut core,
        player_id,
        PlayerBehavior::TacticalSwapUnits {
            first_employee_uuid,
            second_employee_uuid,
            first_facing: FacingDirection::Up,
            second_facing: FacingDirection::Down,
        },
        "swap-paused",
    )
    .unwrap();
    let BehaviorResult::BattleUnitsTacticallySwapped {
        first_destination,
        second_destination,
        battle_update,
        ..
    } = swapped
    else {
        panic!("expected tactical swap result");
    };
    assert!(!battle_update.action_capabilities.is_empty());
    assert_eq!(first_destination, second_position);
    assert_eq!(second_destination, first_position);
    assert_eq!(battle_update.battle_state_revision, revision_before + 1);
    assert_eq!(battle_update.checkpoint.battle_time_ms, 1_500);
    assert!(battle_update.checkpoint.playback.paused);
    assert_eq!(battle_update.checkpoint.tactical_resource.current, 5);
    assert_eq!(battle_update.events_delta.to_seq, seq_before + 1);
    let deployment = battle_update.checkpoint.deployment.as_ref().unwrap();
    let first = deployment
        .deployed_units
        .iter()
        .find(|unit| unit.employee_uuid == first_employee_uuid)
        .unwrap();
    let second = deployment
        .deployed_units
        .iter()
        .find(|unit| unit.employee_uuid == second_employee_uuid)
        .unwrap();
    assert_eq!(
        (first.position, first.facing),
        (second_position, FacingDirection::Up)
    );
    assert_eq!(
        (second.position, second.facing),
        (first_position, FacingDirection::Down)
    );
    assert_eq!(first.tactical_state.relocation_cooldown_until_ms, 11_500);
    assert_eq!(second.tactical_state.relocation_cooldown_until_ms, 11_500);
    assert_eq!(first.tactical_state.stance, EngagementStance::Defense);
    assert_eq!(second.tactical_state.stance, EngagementStance::Defense);
    assert!(battle_update.events_delta.events.iter().any(|entry| {
        entry.source_command_id.as_deref() == Some("swap-paused")
            && matches!(
                entry.event,
                LiveBattlePresentationEventKindDto::UnitsTacticallySwapped {
                    first_unit_instance_id,
                    second_unit_instance_id,
                    cooldown_until_ms: 11_500,
                    ..
                } if first_unit_instance_id == first_unit_id
                    && second_unit_instance_id == second_unit_id
            )
    }));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.battle.units[&first_unit_id].stats.current_health, 43);
    assert_eq!(
        active.battle.units[&second_unit_id].stats.current_health,
        57
    );
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
    let employee_uuids = core.roster().unwrap().combat_eligible_employee_ids();
    let employee_uuid = employee_uuids[0];
    let guard_employee_uuid = employee_uuids[1];
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
        unit_instance_id, ..
    } = deployed
    else {
        panic!("expected live deploy result");
    };
    let guard_position = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .combat_preview
        .deployment_zones
        .iter()
        .filter(|zone| zone.kind == DeploymentZoneKind::Ground)
        .flat_map(|zone| zone.cells.iter().copied())
        .find(|position| *position != deploy_position)
        .expect("fixture should expose a guard deployment cell");
    let guard_deployed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::DeployUnit {
            employee_uuid: guard_employee_uuid,
            position: guard_position,
            facing: FacingDirection::Right,
        },
    )
    .expect("a second living player combatant should keep the reconciliation battle active");
    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: guard_unit_instance_id,
        battle_update: guard_deployed_update,
        ..
    } = guard_deployed
    else {
        panic!("expected guard deploy result");
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
        guard_deployed_update.battle_state_revision + 1
    );
    assert!(advanced_update
        .checkpoint
        .deployment
        .as_ref()
        .is_some_and(|deployment| {
            deployment.deployed_units.len() == 1
                && deployment.deployed_units[0].unit_instance_id == guard_unit_instance_id
        }));

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
    assert_eq!(deployment.deployed_units.len(), 1);
    assert_eq!(
        deployment.deployed_units[0].unit_instance_id, guard_unit_instance_id,
        "only the living guard may remain in the deployed DTO state"
    );
    assert_eq!(deployment.redeploying_units.len(), 1);
    assert_eq!(deployment.redeploying_units[0].employee_uuid, employee_uuid);
    assert_eq!(deployment.redeploying_units[0].ready_at_ms, 90_000);
    assert_eq!(
        deployment.redeploying_units[0].deploy_cost,
        (u64::from(run_policy().tactical_resource.base_deploy_cost)
            * u64::from(run_policy().tactical_resource.redeploy_cost_multiplier_pct))
        .div_ceil(100) as u32
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
        let deploy_cost = deployment
            .redeploy_locks
            .get(&employee_uuid)
            .expect("death should create redeploy lock")
            .deploy_cost;
        deployment
            .redeploy_locks
            .get_mut(&employee_uuid)
            .expect("death should create redeploy lock")
            .ready_at_ms = ready_at_ms;
        active.tactical_resource.set_current_for_test(deploy_cost);
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
        let deploy_cost = lock.deploy_cost;
        active.tactical_resource.set_current_for_test(deploy_cost);
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
    assert!(deployment
        .redeploying_units
        .iter()
        .any(|unit| unit.employee_uuid == employee_uuid));
    let unavailable = deployment
        .unit_deploy_costs
        .iter()
        .find(|unit| unit.employee_uuid == employee_uuid)
        .expect("ineligible employee remains visible");
    assert!(!unavailable.eligibility.enabled);
    assert_eq!(
        unavailable.eligibility.disabled_reason_code,
        attempt_employee.battle_deployment_disabled_reason()
    );

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
        .tactical_resource
        .base_deploy_cost
        .saturating_sub(run_policy().tactical_resource.base_deploy_cost.div_ceil(2));
    let unit_cost = deployment
        .unit_deploy_costs
        .iter()
        .find(|cost| cost.employee_uuid == employee_uuid)
        .expect("employee deploy cost dto");
    assert_eq!(
        unit_cost.base_deploy_cost,
        run_policy().tactical_resource.base_deploy_cost
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

    let BehaviorResult::BattleUnitDeployed {
        unit_instance_id: first_unit_instance_id,
        battle_update,
        ..
    } = deployed
    else {
        panic!("expected live deploy result");
    };
    assert_eq!(
        battle_update.checkpoint.tactical_resource.current,
        run_policy()
            .tactical_resource
            .initial
            .saturating_sub(reduced_cost)
    );

    let withdrawn = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::WithdrawUnit { employee_uuid },
    )
    .unwrap();
    let BehaviorResult::BattleUnitWithdrawn { battle_update, .. } = withdrawn else {
        panic!("expected withdrawal after reduced initial deployment");
    };
    let redeploy_cost = (u64::from(run_policy().tactical_resource.base_deploy_cost)
        * u64::from(run_policy().tactical_resource.redeploy_cost_multiplier_pct))
    .div_ceil(100) as u32;
    let reduced_redeploy_cost = redeploy_cost.saturating_sub(redeploy_cost.div_ceil(2));
    let candidate = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .unwrap()
        .unit_deploy_costs
        .iter()
        .find(|cost| cost.employee_uuid == employee_uuid)
        .unwrap();
    assert_eq!(candidate.base_deploy_cost, redeploy_cost);
    assert_eq!(candidate.effective_deploy_cost, reduced_redeploy_cost);

    let cost_before_redeploy = {
        let active = core.state.active_battle.as_mut().unwrap();
        let now_ms = active.execution.last_event_time_ms();
        let deployment = active.live_deployment.as_mut().unwrap();
        deployment
            .redeploy_locks
            .get_mut(&employee_uuid)
            .unwrap()
            .ready_at_ms = now_ms;
        active.tactical_resource.current()
    };
    let redeployed = execute_current_battle_mutation(
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
        unit_instance_id: second_unit_instance_id,
        battle_update,
        ..
    } = redeployed
    else {
        panic!("expected reduced-cost redeployment");
    };
    assert_ne!(second_unit_instance_id, first_unit_instance_id);
    assert_eq!(
        battle_update.checkpoint.tactical_resource.current,
        cost_before_redeploy.saturating_sub(reduced_redeploy_cost)
    );
}

#[test]
fn activate_skill_uses_equipped_manual_fragment_in_live_defense() {
    let skill_id = SkillId::from("manual_fragment_guard");
    let fragment = active_skill_fragment("manual_fragment", 0xD3F3_2001, skill_id.as_str());
    let skill = SkillDef {
        action_policy: None,
        resonance_admission: None,
        use_lifetime: crate::game::ability::SkillUseLifetime::Action,
        flow: None,
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
        action_policy: None,
        resonance_admission: None,
        use_lifetime: crate::game::ability::SkillUseLifetime::Action,
        flow: None,
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
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            copy_uuid,
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

    let paused =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    let BehaviorResult::BattlePlaybackChanged {
        battle_update: paused_update,
        ..
    } = paused
    else {
        panic!("expected playback update before paused manual activation");
    };
    let frozen_time_ms = paused_update.checkpoint.battle_time_ms;
    let paused_revision = paused_update.battle_state_revision;
    assert!(paused_update.checkpoint.playback.paused);

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
    assert_eq!(battle_update.battle_state_revision, paused_revision + 1);
    assert_eq!(battle_update.checkpoint.battle_time_ms, frozen_time_ms);
    assert!(battle_update.checkpoint.playback.paused);
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

    assert!(core
        .advance_active_battle_for_server_tick(10)
        .unwrap()
        .is_none());

    let speed_changed = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::SetBattleSpeed {
            speed: crate::game::behavior::BattlePlaybackSpeed::X2,
        },
    )
    .unwrap();
    let BehaviorResult::BattlePlaybackChanged {
        battle_update: speed_update,
        ..
    } = speed_changed
    else {
        panic!("expected stored speed update while paused");
    };
    assert_eq!(
        speed_update.battle_state_revision,
        battle_update.battle_state_revision + 1
    );
    assert_eq!(speed_update.checkpoint.battle_time_ms, frozen_time_ms);
    assert!(speed_update.checkpoint.playback.paused);
    assert_eq!(
        speed_update.checkpoint.playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X2
    );

    let resumed =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::ResumeBattle)
            .unwrap();
    let BehaviorResult::BattlePlaybackChanged {
        battle_update: resumed_update,
        ..
    } = resumed
    else {
        panic!("expected resume after paused manual activation");
    };
    assert_eq!(
        resumed_update.battle_state_revision,
        speed_update.battle_state_revision + 1
    );
    assert_eq!(resumed_update.checkpoint.battle_time_ms, frozen_time_ms);
    assert!(!resumed_update.checkpoint.playback.paused);

    let advanced = core
        .advance_active_battle_for_server_tick(1)
        .unwrap()
        .expect("stored x2 speed should advance the resumed battle");
    let BehaviorResult::BattleAdvanced { battle_update, .. } = advanced else {
        panic!("expected battle advance after manual cast");
    };
    assert_eq!(battle_update.checkpoint.battle_time_ms, frozen_time_ms + 2);
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
fn automatic_unit_manual_fragment_readiness_disables_when_no_target_exists() {
    let skill_id = SkillId::from("manual_fragment_mark_target");
    let fragment = active_skill_fragment("target_fragment", 0xD3F3_2005, skill_id.as_str());
    let tile_range = crate::game::battle::tile_range::TileRangePattern {
        include_anchor_tile: false,
        rows: vec![".@X".to_string()],
    };
    let skill = SkillDef {
        action_policy: None,
        resonance_admission: None,
        use_lifetime: crate::game::ability::SkillUseLifetime::Action,
        flow: None,
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
    let initial_charge = consumable_meta(
        0xE005,
        "u05_initial_charge",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xE005_0001), &initial_charge);
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            copy_uuid,
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
    assert_eq!(deployed["skill_readiness"]["target_required"], false);
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

fn player_unit_skill_focus_fixture() -> (
    GameCore,
    Uuid,
    Uuid,
    SkillId,
    crate::game::battle::ids::UnitInstanceId,
    crate::game::battle::ids::UnitInstanceId,
) {
    let skill_id = SkillId::from("manual_fragment_player_unit");
    let fragment = active_skill_fragment("player_unit_fragment", 0xD3F3_2006, skill_id.as_str());
    let skill = SkillDef {
        action_policy: None,
        resonance_admission: None,
        use_lifetime: crate::game::ability::SkillUseLifetime::Action,
        flow: None,
        id: skill_id.clone(),
        name: skill_id.to_string(),
        kind: Default::default(),
        cast_targeting: SkillCastTargetingDef::explicit_with_selection(
            SkillCastSelection::PlayerUnit,
            SkillTarget::EnemySingle {
                rule: Default::default(),
            },
            crate::game::ability::SkillTargetReach::Global,
            None,
            false,
        ),
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "player_selected_hit".to_string(),
            delay_ms: 0,
            target: SkillTarget::EnemySingle {
                rule: Default::default(),
            },
            targeting: crate::game::ability::StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![SkillEffectDef::Damage {
                amount: 10,
                damage_type: crate::game::battle::damage::DamageType::Magic,
            }
            .authored("effect_01")],
            presentation: Default::default(),
        }],
    };
    let weapon = weapon_equipment(0xE008, "player_unit_fragment_sword", WeaponArchetype::Sword);
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
        Uuid::from_u128(0xE008_0001),
    );
    let initial_charge = consumable_meta(
        0xE005,
        "u05_initial_charge",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xE005_0001), &initial_charge);
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            copy_uuid,
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
    let spawned = core
        .advance_active_battle_for_server_tick(5_000)
        .unwrap()
        .expect("the first authored opponent wave should spawn");
    let BehaviorResult::BattleAdvanced {
        battle_update: spawned_update,
        ..
    } = spawned
    else {
        panic!("expected opponent wave advance");
    };
    let target_id = spawned_update
        .checkpoint
        .units
        .iter()
        .find(|unit| unit.owner == Side::Opponent)
        .expect("player-selected unit skill requires an opponent")
        .unit_instance_id;
    let caster_id = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .live_deployment
        .as_ref()
        .unwrap()
        .deployed_units[&employee_uuid]
        .unit_instance_id;
    let caster = core
        .state
        .active_battle
        .as_mut()
        .unwrap()
        .battle
        .units
        .get_mut(&caster_id)
        .unwrap();
    caster.next_action_time = 0;
    caster.resonance_current = caster.resonance_max;
    (
        core,
        player_id,
        employee_uuid,
        skill_id,
        caster_id,
        target_id,
    )
}

#[test]
fn player_unit_skill_focus_preview_cancel_and_paused_activation_are_atomic() {
    let (mut core, player_id, employee_uuid, skill_id, _caster_id, target_id) =
        player_unit_skill_focus_fixture();
    let deploy_position = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .battle
        .unit_projected_tile(_caster_id)
        .unwrap();
    let exact_target = SkillCastTarget::Unit {
        unit_instance_id: target_id,
    };
    let current = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState { since_seq: None },
        )
        .unwrap();
    let BehaviorResult::BattleState {
        battle_update: current_update,
        ..
    } = current
    else {
        panic!("expected current battle state");
    };
    assert!(current_update.action_capabilities.iter().any(|capability| {
        capability.action == ActionKind::BeginSkillTargeting && capability.enabled
    }));
    assert!(
        !current_update.action_capabilities.iter().any(|capability| {
            capability.action == ActionKind::ActivateSkill && capability.enabled
        })
    );
    assert!(
        !current_update.action_capabilities.iter().any(|capability| {
            matches!(
                capability.action,
                ActionKind::CancelSkillTargeting | ActionKind::PreviewSkillTarget
            )
        })
    );

    let (revision_before_focus, seq_before_focus) = {
        let active = core.state.active_battle.as_ref().unwrap();
        (active.battle_state_revision, active.latest_event_log_seq())
    };
    let no_focus_error = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::ActivateSkill {
            employee_uuid,
            skill_id: skill_id.clone(),
            target: Some(exact_target),
        },
    )
    .expect_err("player-selected activation must require matching focus");
    assert!(matches!(
        no_focus_error,
        GameError::ActionUnavailable {
            action: ActionKind::ActivateSkill,
            reason: ActionDisabledReasonCode::NoEligiblePayload,
        }
    ));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(active.battle_state_revision, revision_before_focus);
    assert_eq!(active.latest_event_log_seq(), seq_before_focus);

    assert_future_cursor_rejection_is_atomic(
        &mut core,
        player_id,
        PlayerBehavior::BeginSkillTargeting {
            employee_uuid,
            skill_id: skill_id.clone(),
        },
    );

    let focused = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::BeginSkillTargeting {
            employee_uuid,
            skill_id: skill_id.clone(),
        },
    )
    .unwrap();
    let BehaviorResult::BattleSkillTargetingFocusChanged {
        battle_update: focus_update,
        ..
    } = focused
    else {
        panic!("expected authoritative focus update");
    };
    assert_eq!(
        focus_update.battle_state_revision,
        revision_before_focus + 1
    );
    assert_eq!(
        focus_update.checkpoint.playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X0_5
    );
    assert!(!focus_update.checkpoint.playback.paused);
    let focus = focus_update
        .checkpoint
        .tactical_focus
        .as_ref()
        .expect("checkpoint must expose authoritative targeting focus");
    assert_eq!(focus.skill_id, skill_id);
    assert_eq!(
        focus.purpose,
        crate::game::behavior::LiveBattleTacticalFocusPurposeDto::PlayerUnitSkill
    );
    assert_eq!(
        focus.restore_playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X1
    );
    let focused_unit = focus_update
        .checkpoint
        .deployment
        .as_ref()
        .unwrap()
        .deployed_units
        .iter()
        .find(|unit| unit.employee_uuid == employee_uuid)
        .unwrap();
    let focused_readiness = &focused_unit.skill_readiness;
    assert_eq!(
        focused_readiness.selection,
        Some(SkillCastSelection::PlayerUnit)
    );
    assert!(focused_readiness.target_required);
    assert!(focus_update.action_capabilities.iter().any(|capability| {
        capability.action == ActionKind::ActivateSkill && capability.enabled
    }));
    assert!(!focus_update.action_capabilities.iter().any(|capability| {
        capability.action == ActionKind::BeginSkillTargeting && capability.enabled
    }));
    assert!(focus_update.action_capabilities.iter().any(|capability| {
        capability.action == ActionKind::CancelSkillTargeting && capability.enabled
    }));
    assert!(focus_update.action_capabilities.iter().any(|capability| {
        capability.action == ActionKind::PreviewSkillTarget && capability.enabled
    }));

    let mismatch_preview = core
        .execute(
            player_id,
            PlayerBehavior::PreviewSkillTarget {
                employee_uuid,
                skill_id: skill_id.clone(),
                target: SkillCastTarget::Tile {
                    position: deploy_position,
                },
            },
        )
        .unwrap();
    let BehaviorResult::SkillTargetPreview { result } = mismatch_preview else {
        panic!("expected read-only target preview");
    };
    assert_eq!(
        result.battle_state_revision,
        focus_update.battle_state_revision
    );
    assert_eq!(
        result.eligibility.disabled_reason_code,
        Some(ActionDisabledReasonCode::SkillTargetTypeMismatch)
    );

    let valid_preview = core
        .execute(
            player_id,
            PlayerBehavior::PreviewSkillTarget {
                employee_uuid,
                skill_id: skill_id.clone(),
                target: exact_target,
            },
        )
        .unwrap();
    let BehaviorResult::SkillTargetPreview { result } = valid_preview else {
        panic!("expected valid read-only target preview");
    };
    assert!(result.eligibility.enabled);
    assert_eq!(
        result.battle_state_revision,
        focus_update.battle_state_revision
    );

    let reconnected = core
        .execute(
            player_id,
            PlayerBehavior::RequestBattleState { since_seq: None },
        )
        .unwrap();
    let BehaviorResult::BattleState { battle_update, .. } = reconnected else {
        panic!("expected reconnect checkpoint");
    };
    assert_eq!(
        battle_update.battle_state_revision,
        focus_update.battle_state_revision
    );
    assert_eq!(
        battle_update.checkpoint.tactical_focus,
        focus_update.checkpoint.tactical_focus
    );

    assert_future_cursor_rejection_is_atomic(
        &mut core,
        player_id,
        PlayerBehavior::ActivateSkill {
            employee_uuid,
            skill_id: skill_id.clone(),
            target: Some(exact_target),
        },
    );
    assert_future_cursor_rejection_is_atomic(
        &mut core,
        player_id,
        PlayerBehavior::CancelSkillTargeting,
    );

    let missing_target_error = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::ActivateSkill {
            employee_uuid,
            skill_id: skill_id.clone(),
            target: None,
        },
    )
    .expect_err("player-unit activation must require an explicit unit payload");
    assert!(matches!(
        missing_target_error,
        GameError::ActionUnavailable {
            action: ActionKind::ActivateSkill,
            reason: ActionDisabledReasonCode::SkillTargetRequired,
        }
    ));
    let active = core.state.active_battle.as_ref().unwrap();
    assert_eq!(
        active.battle_state_revision,
        focus_update.battle_state_revision
    );
    assert!(active.tactical_focus.is_some());
    assert_eq!(active.latest_event_log_seq(), seq_before_focus);

    let cancelled =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::CancelSkillTargeting)
            .unwrap();
    let BehaviorResult::BattleSkillTargetingFocusChanged {
        battle_update: cancel_update,
        ..
    } = cancelled
    else {
        panic!("expected focus cancellation update");
    };
    assert_eq!(
        cancel_update.battle_state_revision,
        focus_update.battle_state_revision + 1
    );
    assert!(cancel_update.checkpoint.tactical_focus.is_none());
    assert_eq!(
        cancel_update.checkpoint.playback,
        crate::game::behavior::BattlePlaybackState::default()
    );
    assert_eq!(
        core.state
            .active_battle
            .as_ref()
            .unwrap()
            .latest_event_log_seq(),
        seq_before_focus
    );
    assert!(cancel_update.action_capabilities.iter().any(|capability| {
        capability.action == ActionKind::BeginSkillTargeting && capability.enabled
    }));
    assert!(!cancel_update
        .action_capabilities
        .iter()
        .any(|capability| { capability.action == ActionKind::CancelSkillTargeting }));

    let paused =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::PauseBattle).unwrap();
    let BehaviorResult::BattlePlaybackChanged {
        battle_update: pause_update,
        ..
    } = paused
    else {
        panic!("expected pause update");
    };
    let focused_while_paused = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::BeginSkillTargeting {
            employee_uuid,
            skill_id: skill_id.clone(),
        },
    )
    .unwrap();
    let BehaviorResult::BattleSkillTargetingFocusChanged {
        battle_update: paused_focus_update,
        ..
    } = focused_while_paused
    else {
        panic!("expected paused focus update");
    };
    assert_eq!(
        paused_focus_update.battle_state_revision,
        pause_update.battle_state_revision + 1
    );
    assert!(paused_focus_update.checkpoint.playback.paused);
    assert_eq!(
        paused_focus_update.checkpoint.playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X0_5
    );
    assert!(
        paused_focus_update
            .checkpoint
            .tactical_focus
            .as_ref()
            .unwrap()
            .restore_playback
            .paused
    );

    let revision_before_activation = paused_focus_update.battle_state_revision;
    let activated = execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::ActivateSkill {
            employee_uuid,
            skill_id: skill_id.clone(),
            target: Some(exact_target),
        },
    )
    .unwrap();
    let BehaviorResult::BattleSkillActivated { battle_update, .. } = activated else {
        panic!("expected player-selected skill activation");
    };
    assert_eq!(
        battle_update.battle_state_revision,
        revision_before_activation + 1
    );
    assert!(battle_update.checkpoint.tactical_focus.is_none());
    assert!(battle_update.checkpoint.playback.paused);
    assert_eq!(
        battle_update.checkpoint.playback.speed,
        crate::game::behavior::BattlePlaybackSpeed::X1
    );
    assert!(battle_update.events_delta.events.iter().any(|entry| {
        matches!(
            &entry.event,
            LiveBattlePresentationEventKindDto::ManualCastStart {
                skill_id: actual_skill_id,
                target: Some(SkillCastTarget::Unit { unit_instance_id }),
                ..
            } if actual_skill_id == &skill_id && *unit_instance_id == target_id
        )
    }));
}

#[test]
fn manual_fragment_can_restore_tactical_resource_and_enable_extra_deployment() {
    let skill_id = SkillId::from("manual_fragment_tactical_resource");
    let fragment =
        active_skill_fragment("tactical_resource_fragment", 0xD3F3_2002, skill_id.as_str());
    let skill = SkillDef {
        action_policy: None,
        resonance_admission: None,
        use_lifetime: crate::game::ability::SkillUseLifetime::Action,
        flow: None,
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
            id: "restore_tactical_resource".to_string(),
            delay_ms: 0,
            target: SkillTarget::SelfUnit,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![SkillEffectDef::ModifyTacticalResource {
                amount: run_policy().tactical_resource.maximum as i32 + 50,
            }
            .authored("effect_01")],
            presentation: Default::default(),
        }],
    };
    let weapon = weapon_equipment(
        0xE007,
        "tactical_resource_fragment_sword",
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
    let initial_charge = consumable_meta(
        0xE005,
        "u05_initial_charge",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&caster_uuid)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xE005_0001), &initial_charge);
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid: caster_uuid,
            copy_uuid,
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
            active.tactical_resource.current(),
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
    .expect_err("third deployment should require restored tactical resource");
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
    assert_eq!(
        active.tactical_resource.current(),
        cost_before_failed_deploy
    );
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
        panic!("expected live deployment after tactical-resource skill");
    };
    assert_eq!(
        battle_update.checkpoint.tactical_resource.current,
        run_policy().tactical_resource.maximum
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
        panic!("expected third deployment after tactical-resource restore");
    };
    let deployment = battle_update
        .checkpoint
        .deployment
        .as_ref()
        .expect("live deploy update should expose deployment state");
    assert_eq!(deployment.deployed_units.len(), 3);
    assert_eq!(
        battle_update.checkpoint.tactical_resource.current,
        run_policy()
            .tactical_resource
            .maximum
            .saturating_sub(run_policy().tactical_resource.base_deploy_cost)
    );
}

#[test]
fn authored_skill_negative_tactical_resource_delta_clamps_at_zero() {
    let skill_id = SkillId::from("manual_fragment_tactical_resource_drain");
    let fragment = active_skill_fragment(
        "tactical_resource_drain_fragment",
        0xD3F3_2003,
        skill_id.as_str(),
    );
    let skill = SkillDef {
        action_policy: None,
        resonance_admission: None,
        use_lifetime: crate::game::ability::SkillUseLifetime::Action,
        flow: None,
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
            id: "drain_tactical_resource".to_string(),
            delay_ms: 0,
            target: SkillTarget::SelfUnit,
            targeting: Default::default(),
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![
                SkillEffectDef::ModifyTacticalResource { amount: i32::MIN }.authored("effect_01")
            ],
            presentation: Default::default(),
        }],
    };
    let weapon = weapon_equipment(
        0xE008,
        "tactical_resource_drain_sword",
        WeaponArchetype::Sword,
    );
    let mut core = GameCore::new(
        game_data_with_pve_equipment_and_active_skill_fragments(
            vec![weapon.clone()],
            vec![skill],
            vec![fragment.clone()],
        ),
        124,
    );
    let player_id = Uuid::from_u128(2);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_uuid = core.roster().unwrap().combat_eligible_employee_ids()[0];
    grant_and_equip_weapon(
        &mut core,
        employee_uuid,
        weapon,
        Uuid::from_u128(0xE008_0001),
    );
    let initial_charge = consumable_meta(
        0xE005,
        "u05_initial_charge",
        ConsumableTier::Critical,
        ConsumableEffect::InitialSkillCharge { percent: 100 },
    );
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .apply_consumable_modifier(Uuid::from_u128(0xE005_0001), &initial_charge);
    let copy_uuid = core
        .state
        .skill_fragments
        .grant(
            &fragment,
            &mut core.state.uuid_manager,
            &core.state.skill_fragment_policy,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::EquipSkillFragment {
            employee_uuid,
            copy_uuid,
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

    execute_current_battle_mutation(
        &mut core,
        player_id,
        PlayerBehavior::ActivateSkill {
            employee_uuid,
            skill_id,
            target: None,
        },
    )
    .unwrap();
    let advanced = core.advance_active_battle_by(1).unwrap();
    let BehaviorResult::BattleAdvanced { battle_update, .. } = advanced else {
        panic!("expected battle update after tactical-resource drain");
    };
    assert_eq!(battle_update.checkpoint.tactical_resource.current, 0);
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
    let exact_pre_entry = projection_after_test_first_entry(&core, node_id);

    let entered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: first_battle_uuid,
        battle_update: first_update,
        ..
    } = entered
    else {
        panic!("expected first active battle");
    };
    assert_eq!(
        first_update.checkpoint.tactical_resource.current,
        run_policy().tactical_resource.initial
    );
    core.state
        .active_battle
        .as_mut()
        .unwrap()
        .tactical_resource
        .apply_delta(-7);
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
        projection_after_test_retreat(exact_pre_entry.clone(), 1),
        "retreat changes only its ledger in the authoritative pre-entry projection"
    );

    let reentered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let BehaviorResult::BattleAdvanced {
        battle_uuid: second_battle_uuid,
        battle_update: second_update,
        ..
    } = reentered
    else {
        panic!("expected reentered active battle");
    };
    assert_eq!(
        second_battle_uuid, first_battle_uuid,
        "restored UUID/RNG continuation must reproduce the abandoned entry identity"
    );
    assert_eq!(
        second_update.checkpoint.tactical_resource.current,
        run_policy().tactical_resource.initial,
        "retreat must discard the previous battle-local resource pool"
    );
    execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(
        core.get_run_snapshot_json().unwrap(),
        projection_after_test_retreat(exact_pre_entry, 2)
    );
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        result_stats: test_result_stats(BattleWinner::Opponent, &event_log, &[]),
        event_log,
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions,
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();

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
                revival: None,
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner,
        outcome_id: fixture_outcome_id(winner),
        verdict: fixture_verdict(winner),
        grant_node_bonus: winner == BattleWinner::Player,
        advance_encounter_progression: winner == BattleWinner::Player,
        result_stats: test_result_stats(winner, &event_log, &[]),
        event_log,
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions,
        compendium_candidates: vec![],
    };
    settle_test_combat_result(core, node_id.0, battle).unwrap();
}

fn game_data_with_weaponless_nonstarter_recruits() -> Arc<GameDataBase> {
    let base = GameCore::new(game_data_with_pve_encounters(), 129);
    let mut definitions = base.game_data.employee_definition_data.definitions.clone();
    for definition in &mut definitions {
        if !definition.starter_eligible {
            // Valid nonstarter definitions may have no normal weapon. Keep all
            // six role-compatible starters unchanged for the real departure.
            definition.initial_loadout = EmployeeInitialLoadout::default();
        }
    }
    GameDataBuilder::from_validated(&base.game_data)
        .with_employee_definitions(EmployeeDefinitionDatabase::new(definitions)).build_arc()
}

fn prepare_weaponless_nonstarter_queue(core: &mut GameCore) {
    let run = core.state.run.as_ref().unwrap();
    let mut queue = run.recruitment.queue_definition_ids().map(str::to_owned).collect::<Vec<_>>();
    // I: place real nonstarter candidates in this node's first two offers.
    // Unchosen armed starter definitions remain in the exact queue afterwards.
    queue.sort_by_key(|id| (core.game_data.employee_definition_data.get_by_id(id).unwrap().starter_eligible, id.clone()));
    for id in queue.iter().take(2) {
        let definition = core.game_data.employee_definition_data.get_by_id(id).unwrap();
        assert!(!definition.starter_eligible);
        assert!(definition.initial_loadout.equipment_ids.is_empty());
    }
    let recruitment: crate::game::recruitment::RecruitmentState =
        serde_json::from_value(json!({"queue":queue, "active_session":null})).unwrap();
    recruitment.validate_exact_ownership(&core.game_data.employee_definition_data.definitions, &core.state.roster).unwrap();
    core.state.run.as_mut().unwrap().recruitment = recruitment;
}

#[test]
fn weaponless_recruitment_is_not_a_no_deployable_recovery_path() {
    let mut core = GameCore::new(game_data_with_weaponless_nonstarter_recruits(), 129);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    prepare_weaponless_nonstarter_queue(&mut core);
    core.save_run_checkpoint_for_test().unwrap();
    assert!(core.state.run_checkpoint.can_load());
    let (combat_id, _recruitment_id, _save_point_id) =
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
    assert!(!core.has_combat_recovery_restriction());
}

#[test]
fn weaponless_victory_commits_rewards_then_fails_the_run() {
    let mut core = GameCore::new(game_data_with_weaponless_nonstarter_recruits(), 131);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    prepare_weaponless_nonstarter_queue(&mut core);
    let (combat_id, _recruitment_id, _save_point_id) =
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
}

#[test]
fn weaponless_headquarters_does_not_defer_failure() {
    let mut core = GameCore::new(game_data_with_weaponless_nonstarter_recruits(), 132);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    prepare_weaponless_nonstarter_queue(&mut core);
    let (combat_id, _recruitment_id, _) = install_combat_recovery_test_map(&mut core, true);
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
fn two_retreats_consume_only_retreat_budget_and_third_entry_cannot_retreat() {
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
    let pre_entry = projection_after_test_first_entry(&core, node_id);
    let mut first_battle_uuid = None;

    for retreat in 1..=2 {
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
        assert_eq!(
            core.get_run_snapshot_json().unwrap(),
            projection_after_test_retreat(pre_entry.clone(), retreat)
        );
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
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let before = core.get_run_snapshot_json().unwrap();
    assert!(
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle)
            .is_err()
    );
    assert_eq!(core.get_run_snapshot_json().unwrap(), before);
}

#[test]
fn failed_live_defense_battle_consumes_node_on_first_defeat() {
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    assert_eq!(core.get_state(), GameState::ViewingMap);
    assert_eq!(
        core.run_state().unwrap().map.node(node_id).unwrap().state,
        MapNodeState::Completed
    );
    assert!(core.state.active_battle.is_none());
    assert!(core.state.active_node_content.is_none());
    assert!(core.state.node_session.is_none());
    assert!(core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .is_err());
}

#[test]
fn final_boss_retreat_changes_only_its_budget() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let run = core.state.run.as_mut().expect("run state");
        run.run_progression.mode_state = crate::game::map::RunProgressionModeState::Tutorial {
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
    let pre_entry = projection_after_test_first_entry(&core, node_id);
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let result =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle)
            .unwrap();
    assert!(matches!(result, BehaviorResult::NodePreview { .. }));
    assert_eq!(
        core.get_run_snapshot_json().unwrap(),
        projection_after_test_retreat(pre_entry, 1)
    );
    assert!(serde_json::to_value(&core.state.player_profile).unwrap().get("standard_tutorial_completed").is_none());
}

#[test]
fn final_boss_defeat_with_valid_checkpoint_requires_explicit_decision() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    core.save_run_checkpoint_for_test().unwrap();
    {
        let run = core.state.run.as_mut().expect("run state");
        run.run_progression.mode_state = crate::game::map::RunProgressionModeState::Tutorial {
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
        encounter_id: "final_boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        result_stats: test_result_stats(BattleWinner::Opponent, &event_log, &[]),
        event_log,
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();

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
        .registered_ids
        .is_empty());
}

#[test]
fn forced_boss_omen_defeat_immediately_fails_endless_run() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    core.save_run_checkpoint_for_test().unwrap();
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
        encounter_id: "final_boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();

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
    assert_eq!(
        core.get_action_capabilities(),
        vec![
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::StartNewGame,
                crate::game::behavior::ActionScope::Gameplay
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicState,
                crate::game::behavior::ActionScope::Query
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicNumericPlan,
                crate::game::behavior::ActionScope::Query
            ),
            crate::game::behavior::ActionCapabilityDto::enabled(
                ActionKind::RequestRelicEffectivePlan,
                crate::game::behavior::ActionScope::Query
            ),
        ]
    );
}

#[test]
fn forced_boss_omen_failure_requires_decision_and_executes_valid_checkpoint_load() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 134);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    core.save_run_checkpoint_for_test().unwrap();
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
        encounter_id: "final_boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();
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
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("final_boss_risk_abno".to_string()),
        encounter_id: "final_boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Player,
        outcome_id: fixture_outcome_id(BattleWinner::Player),
        verdict: fixture_verdict(BattleWinner::Player),
        grant_node_bonus: BattleWinner::Player == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Player == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Player, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    let BehaviorResult::CombatRewardsGranted { completion, .. } = result else {
        panic!("forced boss victory should grant combat rewards before opening the exit Gate");
    };
    assert!(matches!(*completion, BehaviorResult::NodeCompleted { .. }));
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    let final_fragment_id = crate::game::data::skill_fragment_data::SkillFragmentId::from(
        "fragment_final_boss_risk_abno",
    );
    assert_eq!(core.state.skill_fragments.count(&final_fragment_id), 1);
    assert_eq!(core.state.skill_fragments.fragment_dust(), 0);
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .completed_reward_source_ids
        .contains("source_final_boss_risk"));
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
    let run = core.state.run.as_ref().unwrap();
    assert_eq!(run.map_progression.current_node_id, Some(node_id));
    assert_eq!(run.map_progression.completed_node_ids, vec![node_id]);
    assert_eq!(run.map_progression.available_node_ids, vec![gate_id]);
    assert_eq!(
        run.map.node(node_id).unwrap().state,
        MapNodeState::Completed
    );
    assert!(run.map.node(gate_id).is_some_and(|node| {
        node.category == MapNodeCategory::Gate
            && node.kind_id.as_str() == "gate_stairs"
            && node.state == MapNodeState::Available
            && node.visibility == MapNodeVisibility::Revealed
    }));
    let template = core.facility_template_for_map(&run.map).unwrap();
    assert!(run
        .map_progression
        .validate_checkpoint_state(&run.map, &template));

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
            game_mode: GameMode::Expedition,
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
    assert_eq!(core.state.skill_fragments.count(&final_fragment_id), 1);
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .completed_reward_source_ids
        .contains("source_final_boss_risk"));
}

#[test]
fn boss_kind_combat_defeat_consumes_node_on_first_defeat() {
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("boss_abno".to_string()),
        encounter_id: "boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    assert_eq!(core.get_state(), GameState::ViewingMap);
    assert_eq!(
        core.run_state().unwrap().map.node(node_id).unwrap().state,
        MapNodeState::Completed
    );
    assert!(serde_json::to_value(&core.state.player_profile).unwrap().get("standard_tutorial_completed").is_none());
}

#[test]
fn standard_completion_normal_boss_defeat_and_retreat_never_complete_tutorial() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let run = core.state.run.as_mut().expect("run state");
        run.run_progression.mode_state = crate::game::map::RunProgressionModeState::Tutorial {
            floor_index: run_policy().setup.standard_floor_count - 1,
            max_floors: run_policy().setup.standard_floor_count,
        };
    }
    let node_id = force_map_combat_node(
        &mut core,
        MapNodeCategory::Boss,
        "boss_abnormality",
        "boss_risk_encounter",
    );
    core.state.run.as_mut().unwrap().map.terminals = vec![MapTerminalNode {
        node_id,
        resolution: FacilityTerminalResolution::StandardCompletion,
    }];

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let event_log = BattleEventLog::default();
    let battle = CombatBattleState {
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("boss_risk_abno".to_string()),
        encounter_id: "boss_risk_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Boss,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Boss,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        result_stats: test_result_stats(BattleWinner::Opponent, &event_log, &[]),
        event_log,
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    let mut retreat_core = core.clone();
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();

    let defeated = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert!(matches!(
        defeated,
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
    assert!(serde_json::to_value(&core.state.player_profile).unwrap().get("standard_tutorial_completed").is_none());
    assert_eq!(
        core.run_state().unwrap().map.node(node_id).unwrap().state,
        MapNodeState::Completed
    );
    assert!(core.export_active_run_checkpoint().unwrap().is_none());
    let retreated = execute_current_battle_mutation(
        &mut retreat_core,
        player_id,
        PlayerBehavior::RetreatBattle,
    )
    .unwrap();
    assert!(matches!(retreated, BehaviorResult::NodePreview { .. }));
    assert!(serde_json::to_value(&retreat_core.state.player_profile).unwrap().get("standard_tutorial_completed").is_none());
    assert_eq!(
        retreat_core
            .run_state()
            .unwrap()
            .abnormality_attempt_state(node_id)
            .retreats_used,
        1
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
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
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
    let battle = CombatBattleState {
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: event_node_id.0,
        winner: BattleWinner::Player,
        outcome_id: fixture_outcome_id(BattleWinner::Player),
        verdict: fixture_verdict(BattleWinner::Player),
        grant_node_bonus: BattleWinner::Player == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Player == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Player, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, event_node_id.0, battle).unwrap();

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
    let BehaviorResult::NodeCompleted { .. } = *completion else {
        panic!("authored Event room victory should consume the node after rewards");
    };
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
fn event_started_combat_defeat_consumes_node_and_clears_session() {
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
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: event_node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, event_node_id.0, battle).unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    assert_eq!(core.get_state(), GameState::ViewingMap);
    assert_eq!(
        core.run_state()
            .unwrap()
            .map
            .node(event_node_id)
            .unwrap()
            .state,
        MapNodeState::Completed
    );
    assert!(core
        .run_state()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .is_none());
    assert!(core
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: event_node_id
            }
        )
        .is_err());
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
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
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
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: event_node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, event_node_id.0, battle).unwrap();

    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();

    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
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

    for retreat in 1..=2 {
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
            json!(3 - retreat)
        );
    }

    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let battle = CombatBattleState {
        bonus_reward_effects: Vec::new(),
        deterrence: Default::default(),
        credit_settlements: Default::default(), typed_resource_result: None,
        runtime_facts: None,
        primary_abnormality_id: Some("defense_risk_abno".to_string()),
        encounter_id: "defense_encounter".to_string(),
        node_type: crate::game::combat_preview::CombatNodeType::Defense,
        mission_variant: crate::game::combat_preview::CombatMissionVariant::Defense,
        abnormality_uuid: node_id.0,
        winner: BattleWinner::Opponent,
        outcome_id: fixture_outcome_id(BattleWinner::Opponent),
        verdict: fixture_verdict(BattleWinner::Opponent),
        grant_node_bonus: BattleWinner::Opponent == BattleWinner::Player,
        advance_encounter_progression: BattleWinner::Opponent == BattleWinner::Player,
        event_log: BattleEventLog::default(),
        result_stats: test_result_stats(BattleWinner::Opponent, &BattleEventLog::default(), &[]),
        reward_mode: RewardMode::ClaimAll,
        rewards: vec![],
        participant_results: vec![],
        employee_conditions: core.state.roster.clone(),
        compendium_candidates: vec![],
    };
    settle_test_combat_result(&mut core, node_id.0, battle).unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::CompleteCombatResult)
        .unwrap();
    assert!(matches!(result, BehaviorResult::NodeCompleted { .. }));
    assert!(core
        .roster()
        .unwrap()
        .get(&employee_uuid)
        .unwrap()
        .active_consumable_modifier
        .is_none());
}

#[test]
fn owned_clone_shape_complete_world_preserves_entry_checkpoint_and_retreat_continuation() {
    use crate::authoring::{
        clone_shape::controlled_clone,
        control::{
            self, CancellationToken, ExecutionLimits, ExecutionPhase, NativeStorageProvenance,
        },
    };
    let game_data = game_data_with_pve_encounters();
    let limits = ExecutionLimits {
        max_work: 1 << 30,
        max_clone_slots: 1 << 20,
        ..ExecutionLimits::default()
    };
    let token = CancellationToken::default();
    let phase = ExecutionPhase::TrialPreparation;
    let player_id = Uuid::from_u128(1);
    let ((mut source, entry_projection, entry_uuid), _, provenance) = control::run_with_provenance(
        &limits,
        &token,
        phase,
        NativeStorageProvenance::fresh(),
        || {
            let mut core = GameCore::new(game_data, 127);
            start_new_game_with_default_starters(&mut core, player_id);
            let node_id = force_first_available_node(
                &mut core,
                MapNodeCategory::SavePoint,
                "save_point",
                MapNodePayload::SavePoint,
            );
            core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
                .unwrap();
            core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
                .unwrap();
            core.execute(player_id, PlayerBehavior::CompleteNode)
                .unwrap();
            core.save_run_checkpoint_for_test().unwrap();
            let node_id = force_map_combat_node(
                &mut core,
                MapNodeCategory::Combat,
                "combat_defense",
                "defense_encounter",
            );
            core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
                .unwrap();
            let entry_projection = projection_after_test_first_entry(&core, node_id);
            let entry_uuid = serde_json::to_value(&core.state.uuid_manager).unwrap();
            core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
                .unwrap();
            assert!(core
                .state
                .active_battle
                .as_ref()
                .unwrap()
                .entry_snapshot
                .is_some());
            assert!(core.state.run_checkpoint.payload.is_some());
            (core, entry_projection, entry_uuid)
        },
    )
    .unwrap();
    let original = source.get_run_snapshot_json().unwrap();
    let (candidate, usage, _) =
        control::run_with_provenance(&limits, &token, phase, provenance, || {
            controlled_clone(&source)
        })
        .unwrap();
    let mut candidate = candidate.unwrap();
    assert_eq!(candidate.get_run_snapshot_json().unwrap(), original);
    assert!(Arc::ptr_eq(
        source.state.active_battle.as_ref().unwrap().entry_snapshot.as_ref().unwrap(),
        candidate.state.active_battle.as_ref().unwrap().entry_snapshot.as_ref().unwrap(),
    ));
    assert!(std::ptr::eq(&*source.game_data, &*candidate.game_data));
    assert!(Arc::ptr_eq(
        source.state.continue_state.encoded.as_ref().unwrap(),
        candidate.state.continue_state.encoded.as_ref().unwrap()
    ));
    let mut exact = limits.clone();
    exact.max_work = usage.work;
    exact.max_clone_slots = usage.clone_slots_peak;
    let (copy, exact_usage, _) =
        control::run_with_provenance(&exact, &token, phase, provenance, || {
            controlled_clone(&source)
        })
        .unwrap();
    assert!(copy.is_ok());
    assert_eq!(usage, exact_usage);
    drop(copy);
    for short_work in [true, false] {
        let mut short = exact.clone();
        if short_work {
            short.max_work -= 1;
        } else {
            short.max_clone_slots -= 1;
        }
        let failure = control::run_with_provenance(&short, &token, phase, provenance, || {
            controlled_clone(&source)
        })
        .err()
        .unwrap();
        assert_eq!(
            failure.code,
            if short_work {
                "authoring_work_limit"
            } else {
                "authoring_clone_limit"
            }
        );
        assert_eq!(source.get_run_snapshot_json().unwrap(), original);
    }
    // Live owners remain independent even though their immutable entry is shared.
    for employee in candidate.state.roster.iter_mut() {
        employee.condition.physical_damage = 3;
    }
    candidate.state.inventory.add_item_owned(
        Uuid::from_u128(0xEC01),
        Item::Equipment(Arc::new(equipment_meta(
            0xEC02, "shared_entry_candidate", EquipmentType::Accessory,
        ))),
    ).unwrap();
    assert_ne!(candidate.get_run_snapshot_json().unwrap(), original);
    assert_eq!(source.get_run_snapshot_json().unwrap(), original);
    for employee in source.state.roster.iter_mut() {
        employee.condition.physical_damage = 5;
    }
    source.state.inventory.add_item_owned(
        Uuid::from_u128(0xEC03),
        Item::Equipment(Arc::new(equipment_meta(
            0xEC04, "shared_entry_source", EquipmentType::Accessory,
        ))),
    ).unwrap();
    let source_live = source.get_run_snapshot_json().unwrap();
    assert_ne!(source_live, candidate.get_run_snapshot_json().unwrap());
    let entry = source.state.active_battle.as_ref().unwrap().entry_snapshot.as_ref().unwrap();
    let weak_entry = Arc::downgrade(entry);

    // Copy accounting follows the storage actually copied: an Arc handle is
    // smaller than materializing the captured roster, inventory and checkpoint.
    let (handle, handle_usage, _) = control::run_with_provenance(
        &limits, &token, phase, provenance, || controlled_clone(entry),
    ).unwrap();
    drop(handle.unwrap());
    let (owned, owned_usage, _) = control::run_with_provenance(
        &limits, &token, phase, provenance, || controlled_clone(entry.as_ref()),
    ).unwrap();
    let owned = owned.unwrap();
    assert!(owned_usage.work > handle_usage.work);
    assert!(owned_usage.clone_slots_peak > handle_usage.clone_slots_peak);

    // A sole owner moves under limits too small to copy its children. The weak
    // observer also proves the consumed allocation has no retained strong owner.
    let unique = Arc::new(owned);
    let weak_unique = Arc::downgrade(&unique);
    let mut unique_target = controlled_clone(&source).unwrap();
    let move_limits = ExecutionLimits {
        max_work: 1,
        max_clone_slots: 1,
        ..limits.clone()
    };
    let (restored, move_usage, _) = control::run_with_provenance(
        &move_limits, &token, phase, provenance, || {
            super::super::state::BattleEntrySnapshot::restore_shared(unique, &mut unique_target.state)
        },
    ).unwrap();
    restored.unwrap();
    assert_eq!(move_usage.work, 0);
    assert_eq!(move_usage.clone_slots_peak, 0);
    assert!(weak_unique.upgrade().is_none());
    assert_eq!(unique_target.get_run_snapshot_json().unwrap(), entry_projection);
    assert_eq!(serde_json::to_value(&unique_target.state.uuid_manager).unwrap(), entry_uuid);
    drop(unique_target);

    // Shared restoration admits the full owned copy before changing any target
    // field. Exact and one-short limits are measured on the real restore API.
    let mut restored_target = controlled_clone(&source).unwrap();
    let (restored, restore_usage, _) = control::run_with_provenance(
        &limits, &token, phase, provenance, || {
            super::super::state::BattleEntrySnapshot::restore_shared(Arc::clone(entry), &mut restored_target.state)
        },
    ).unwrap();
    restored.unwrap();
    assert_eq!(restored_target.get_run_snapshot_json().unwrap(), entry_projection);
    assert_eq!(serde_json::to_value(&restored_target.state.uuid_manager).unwrap(), entry_uuid);
    assert!(restore_usage.work >= owned_usage.work);
    assert_eq!(restore_usage.clone_slots_peak, owned_usage.clone_slots_peak);
    drop(restored_target);
    let exact_restore = ExecutionLimits {
        max_work: restore_usage.work,
        max_clone_slots: restore_usage.clone_slots_peak,
        ..limits.clone()
    };
    for short in [None, Some(true), Some(false)] {
        let mut target = controlled_clone(&source).unwrap();
        let before_uuid = serde_json::to_value(&target.state.uuid_manager).unwrap();
        let mut bound = exact_restore.clone();
        if let Some(short_work) = short {
            if short_work { bound.max_work -= 1; } else { bound.max_clone_slots -= 1; }
        }
        let result = control::run_with_provenance(&bound, &token, phase, provenance, || {
            super::super::state::BattleEntrySnapshot::restore_shared(Arc::clone(entry), &mut target.state)
        });
        if let Some(short_work) = short {
            let failure = result.err().unwrap();
            assert_eq!(failure.code, if short_work { "authoring_work_limit" } else { "authoring_clone_limit" });
            assert_eq!(target.get_run_snapshot_json().unwrap(), source_live);
            assert_eq!(serde_json::to_value(&target.state.uuid_manager).unwrap(), before_uuid);
            assert!(Arc::ptr_eq(entry, target.state.active_battle.as_ref().unwrap().entry_snapshot.as_ref().unwrap()));
        } else {
            let (restored, usage, _) = result.unwrap();
            restored.unwrap();
            assert_eq!(usage, restore_usage);
            assert_eq!(target.get_run_snapshot_json().unwrap(), entry_projection);
        }
    }

    // Native fuel is independent of authoring limits. Refusal in the shared
    // preflight must leave the complete target available for an ordinary retry.
    for clone_fault in [true, false] {
        use crate::game::battle::runtime_control::{self, RuntimeFactLimits, RuntimeFaultCode};
        let mut target = controlled_clone(&source).unwrap();
        let before_uuid = serde_json::to_value(&target.state.uuid_manager).unwrap();
        let native_limits = if clone_fault {
            RuntimeFactLimits { max_clone_slots: 1, ..Default::default() }
        } else {
            RuntimeFactLimits { max_work: 1, ..Default::default() }
        };
        let error = runtime_control::run(native_limits, || {
            super::super::state::BattleEntrySnapshot::restore_shared(Arc::clone(entry), &mut target.state)
        }).unwrap_err();
        assert!(matches!(error, GameError::BattleRuntimeFault(ref fault)
            if fault.code == if clone_fault { RuntimeFaultCode::CloneLimit } else { RuntimeFaultCode::WorkLimit }), "{error:?}");
        assert_eq!(target.get_run_snapshot_json().unwrap(), source_live);
        assert_eq!(serde_json::to_value(&target.state.uuid_manager).unwrap(), before_uuid);
        assert!(Arc::ptr_eq(entry, target.state.active_battle.as_ref().unwrap().entry_snapshot.as_ref().unwrap()));
        super::super::state::BattleEntrySnapshot::restore_shared(Arc::clone(entry), &mut target.state).unwrap();
        assert_eq!(target.get_run_snapshot_json().unwrap(), entry_projection);
    }

    // The public command creates an outer draft, so it must also preserve the
    // original active battle, ledger and UUID when its bounded attempt fails.
    let mut retreat_probe = controlled_clone(&source).unwrap();
    let (retreated, retreat_usage, _) = control::run_with_provenance(
        &limits, &token, phase, provenance, || {
            execute_current_battle_mutation(&mut retreat_probe, player_id, PlayerBehavior::RetreatBattle)
        },
    ).unwrap();
    retreated.unwrap();
    assert_eq!(retreat_probe.get_run_snapshot_json().unwrap(), projection_after_test_retreat(entry_projection.clone(), 1));
    drop(retreat_probe);
    for short_work in [true, false] {
        let mut target = controlled_clone(&source).unwrap();
        let before_uuid = serde_json::to_value(&target.state.uuid_manager).unwrap();
        let mut bound = ExecutionLimits {
            max_work: retreat_usage.work,
            max_clone_slots: retreat_usage.clone_slots_peak,
            ..limits.clone()
        };
        if short_work { bound.max_work -= 1; } else { bound.max_clone_slots -= 1; }
        let failure = control::run_with_provenance(&bound, &token, phase, provenance, || {
            execute_current_battle_mutation(&mut target, player_id, PlayerBehavior::RetreatBattle)
        }).err().unwrap();
        assert_eq!(failure.code, if short_work { "authoring_work_limit" } else { "authoring_clone_limit" });
        assert_eq!(target.get_run_snapshot_json().unwrap(), source_live);
        assert_eq!(serde_json::to_value(&target.state.uuid_manager).unwrap(), before_uuid);
        assert!(Arc::ptr_eq(entry, target.state.active_battle.as_ref().unwrap().entry_snapshot.as_ref().unwrap()));
    }

    execute_current_battle_mutation(&mut candidate, player_id, PlayerBehavior::RetreatBattle).unwrap();
    assert_eq!(source.get_run_snapshot_json().unwrap(), source_live);
    assert!(weak_entry.upgrade().is_some());
    execute_current_battle_mutation(&mut source, player_id, PlayerBehavior::RetreatBattle).unwrap();
    assert!(weak_entry.upgrade().is_none());
    assert_eq!(candidate.get_run_snapshot_json().unwrap(), projection_after_test_retreat(entry_projection.clone(), 1));
    assert_eq!(source.get_run_snapshot_json().unwrap(), projection_after_test_retreat(entry_projection, 1));
    assert_eq!(serde_json::to_value(&candidate.state.uuid_manager).unwrap(), entry_uuid);
    assert_eq!(serde_json::to_value(&source.state.uuid_manager).unwrap(), entry_uuid);
}

#[test]
fn owned_capture_shape_prices_only_captured_roots_and_rejects_before_delivery() {
    use crate::authoring::control::{
        self, CancellationToken, ExecutionLimits, ExecutionPhase, ExecutionUsage,
        NativeStorageProvenance,
    };
    use crate::game::world::{
        grant_state::EconomyGrantState,
        state::{BattleEmployeeAttemptState, BattleEntrySnapshot, RunCheckpointPayload},
    };
    fn check<T: crate::authoring::clone_shape::CloneShape>(
        provenance: NativeStorageProvenance,
        capture: impl Fn() -> Result<T, GameError>,
    ) -> ExecutionUsage {
        let limits = ExecutionLimits::default();
        let token = CancellationToken::default();
        let phase = ExecutionPhase::TrialPreparation;
        let (value, usage, _) =
            control::run_with_provenance(&limits, &token, phase, provenance, &capture).unwrap();
        let value = value.ok().unwrap();
        // Independent completed-owner visitor includes fixed fields freshly
        // created by capture, while never traversing the excluded source roots.
        let (_, completed_usage, _) =
            control::run_with_provenance(&limits, &token, phase, provenance, || {
                let mut meter = control::CloneMeter::default();
                value.visit_clone_shape(&mut meter).unwrap();
                meter.finish("completed_storage", "completed_copy").unwrap();
            })
            .unwrap();
        assert_eq!(usage, completed_usage);
        drop(value);
        let exact = ExecutionLimits {
            max_work: usage.work,
            max_clone_slots: usage.clone_slots_peak,
            ..limits.clone()
        };
        let (value, exact_usage, _) =
            control::run_with_provenance(&exact, &token, phase, provenance, &capture).unwrap();
        assert!(value.is_ok());
        assert_eq!(usage, exact_usage);
        drop(value);
        for short_work in [true, false] {
            let mut short = exact.clone();
            if short_work {
                short.max_work -= 1;
            } else {
                short.max_clone_slots -= 1;
            }
            let failure = control::run_with_provenance(&short, &token, phase, provenance, &capture)
                .err()
                .unwrap();
            assert_eq!(
                failure.code,
                if short_work {
                    "authoring_work_limit"
                } else {
                    "authoring_clone_limit"
                }
            );
        }
        usage
    }
    fn check_all(core: &GameCore, provenance: NativeStorageProvenance) -> Vec<ExecutionUsage> {
        vec![
            check(provenance, || {
                core.state
                    .clone_non_battle_transaction_state()?
                    .ok_or(GameError::InvalidAction)
            }),
            check(provenance, || BattleEntrySnapshot::capture(&core.state)),
            check(provenance, || {
                RunCheckpointPayload::capture(
                    &core.state,
                    core.state.run.as_ref().unwrap(),
                    core.content_identity(),
                )
            }),
            check(provenance, || EconomyGrantState::capture(&core.state)),
            check(provenance, || {
                BattleEmployeeAttemptState::new(&core.state.roster)
            }),
        ]
    }
    let data = game_data_with_pve_encounters();
    let limits = ExecutionLimits::default();
    let (mut source, _, provenance) = control::run_with_provenance(
        &limits,
        &CancellationToken::default(),
        ExecutionPhase::TrialPreparation,
        NativeStorageProvenance::fresh(),
        || {
            let mut core = GameCore::new(data, 127);
            start_new_game_with_default_starters(&mut core, Uuid::from_u128(1));
            core
        },
    )
    .unwrap();
    let before = source.get_run_snapshot_json().unwrap();
    let usage = check_all(&source, provenance);
    assert_eq!(source.get_run_snapshot_json().unwrap(), before);
    // These captures deliberately exclude the permanent profile. A borrowed
    // whole-state surrogate would incorrectly charge its large dormant ID.
    source
        .state
        .player_profile
        .encounter_compendium
        .registered_ids
        .insert("uncaptured_id".repeat(16384));
    assert_eq!(check_all(&source, provenance), usage);
    assert_eq!(source.get_run_snapshot_json().unwrap(), before);
}

#[test]
fn owned_capture_shape_sticky_failure_restores_run_state_and_moves_live_profile() {
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    let mut source = GameCore::new(game_data_with_pve_encounters(), 0xC011_EC73);
    source
        .state
        .player_profile
        .encounter_compendium
        .registered_ids
        .insert("before".into());
    let before_uuid = serde_json::to_value(&source.state.uuid_manager).unwrap();
    let before_amount = source.state.enkephalin.amount;
    let reached = std::cell::Cell::new(false);
    let failure = control::run(
        &ExecutionLimits::default(),
        &CancellationToken::default(),
        ExecutionPhase::TrialExecution,
        || {
            source.with_state_transaction(|core| {
                reached.set(true);
                core.state.enkephalin.amount = 999;
                core.state.uuid_manager.next_employee();
                core.state
                    .player_profile
                    .encounter_compendium
                    .registered_ids
                    .insert("during".into());
                control::charge(u64::MAX, "late_transaction_failure")
            })
        },
    )
    .err()
    .unwrap();
    assert!(reached.get());
    assert_eq!(failure.code, "authoring_work_limit");
    assert_eq!(failure.site, "late_transaction_failure");
    assert_eq!(source.state.enkephalin.amount, before_amount);
    assert_eq!(
        serde_json::to_value(&source.state.uuid_manager).unwrap(),
        before_uuid
    );
    assert_eq!(
        source
            .state
            .player_profile
            .encounter_compendium
            .registered_ids,
        std::collections::BTreeSet::from(["before".into(), "during".into()])
    );
    assert!(!control::is_active());
}

#[test]
fn authoring_lifecycle_tracks_actual_time_and_retreat_uuid_reuse_and_inner_rollback() {
    use crate::game::world::authoring_lifecycle::TrialBattleStatus;
    let data = live_game_data_from_ron();
    assert!(GameCore::new(data.clone(), 127)
        .authoring_lifecycle()
        .is_none());
    let mut core = GameCore::new_for_authoring(data.into(), 127);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let node = force_map_combat_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_monster",
        "suppress_burrowing_heaven",
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    let before = core.authoring_lifecycle().unwrap();
    assert_eq!(before.entries, 0);
    assert!(!before.is_ready());
    let before_run = core.get_run_snapshot_json().unwrap();
    let before_observation = serde_json::to_value(core.authoring_observation().unwrap()).unwrap();
    let error = core.with_state_transaction(|draft| {
        draft.execute(player, PlayerBehavior::ConfirmEnterNode)?;
        assert!(draft.authoring_lifecycle().unwrap().is_ready());
        Err::<(), _>(GameError::InvalidAction)
    });
    assert!(error.is_err());
    assert_eq!(core.authoring_lifecycle(), Some(before));
    assert_eq!(core.get_run_snapshot_json().unwrap(), before_run);
    assert_eq!(
        serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
        before_observation
    );
    assert!(core.state.active_battle.is_none());

    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let started = core.authoring_lifecycle().unwrap();
    assert!(started.is_ready());
    assert_eq!(started.entries, 1);
    assert_eq!(started.total_elapsed_ms, 0);
    assert_eq!(started.current.unwrap().status, TrialBattleStatus::Running);
    let uuid = started.current.unwrap().battle_uuid;
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::PauseBattle).unwrap();
    assert!(core
        .advance_active_battle_for_server_tick_after_seq(10, 0)
        .unwrap()
        .is_none());
    assert_eq!(core.authoring_lifecycle(), Some(started));
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::ResumeBattle).unwrap();
    execute_current_battle_mutation(
        &mut core,
        player,
        PlayerBehavior::SetBattleSpeed {
            speed: crate::game::behavior::BattlePlaybackSpeed::X2,
        },
    )
    .unwrap();
    assert!(core
        .advance_active_battle_for_server_tick_after_seq(10, 0)
        .unwrap()
        .is_some());
    let scaled = core.authoring_lifecycle().unwrap();
    assert_eq!(scaled.total_elapsed_ms, 20);
    core.advance_active_battle_to_next_event_bucket().unwrap();
    let advanced = core.authoring_lifecycle().unwrap();
    let actual = core
        .state
        .active_battle
        .as_ref()
        .unwrap()
        .execution
        .last_event_time_ms();
    assert_eq!(advanced.total_elapsed_ms, actual);
    assert_eq!(advanced.current.unwrap().last_battle_time_ms, actual);
    assert!(actual > 20);
    assert_eq!(core.clone().authoring_lifecycle(), Some(advanced));
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
    let retreated = core.authoring_lifecycle().unwrap();
    assert!(!retreated.is_ready());
    assert_eq!(retreated.current.unwrap().status, TrialBattleStatus::Exited);
    assert_eq!(retreated.total_elapsed_ms, actual);
    assert!(core.authoring_observation().unwrap().active.is_none());
    assert!(core
        .authoring_observation()
        .unwrap()
        .last_terminal
        .is_none());
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    let restarted = core.authoring_lifecycle().unwrap();
    assert!(restarted.is_ready());
    assert_eq!(restarted.entries, 2);
    assert_eq!(restarted.current.unwrap().entry_ordinal, 2);
    assert_eq!(restarted.current.unwrap().battle_uuid, uuid);
    assert_eq!(restarted.total_elapsed_ms, actual);
    let key = core
        .authoring_observation()
        .unwrap()
        .active
        .unwrap()
        .key
        .unwrap();
    assert_eq!(key.entry_ordinal, 2);
    assert_eq!(key.battle_uuid, uuid);
}

#[test]
fn authoring_lifecycle_keeps_actual_terminal_after_removal_including_time_zero() {
    use crate::authoring::trace::{TraceAttempt, TraceLimits, TraceOptions};
    use crate::game::world::authoring_lifecycle::TrialBattleStatus;
    for terminal_time in [0, 10] {
        let mut data = live_game_data_from_ron();
        let database = Arc::get_mut(&mut data).unwrap();
        let pve = Arc::get_mut(&mut database.pve_data).unwrap();
        let encounter = pve
            .encounters
            .iter_mut()
            .find(|entry| entry.id == "suppress_burrowing_heaven")
            .unwrap();
        encounter
            .terminal_outcomes
            .retain(|outcome| outcome.verdict == BattleVerdict::Victory);
        let outcome_id = encounter.terminal_outcomes[0].id.clone();
        encounter
            .outcome_policies
            .retain(|policy| policy.outcome_id == outcome_id);
        encounter.terminal_outcomes[0].condition = if terminal_time == 0 {
            crate::game::data::pve_data::PveTerminalConditionData::AllRequiredEnemyGroupsDefeated
        } else {
            crate::game::data::pve_data::PveTerminalConditionData::TimeReached {
                time_ms: terminal_time,
            }
        };
        for wave in &mut encounter.waves {
            wave.required_for_victory = false;
        }
        let mut prior_update = None;
        let mut prior_mandatory = None;
        for trace_options in [
            None,
            Some(TraceOptions::default()),
            Some(TraceOptions {
                limits: Some(TraceLimits::default()),
            }),
        ] {
            let enabled = trace_options.is_some();
            let _trace_attempt = TraceAttempt::enter(trace_options.unwrap_or_default()).unwrap();
            let mut core = if enabled {
                GameCore::new_for_authoring(data.clone().into(), 0xC011_EC74)
            } else {
                GameCore::new(data.clone(), 0xC011_EC74)
            };
            if let Some(options) = trace_options {
                core.set_authoring_trace_options(options);
            }
            let player = Uuid::from_u128(1);
            start_new_game_with_default_starters(&mut core, player);
            let node = force_map_combat_node(
                &mut core,
                MapNodeCategory::Combat,
                "combat_monster",
                "suppress_burrowing_heaven",
            );
            core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
                .unwrap();
            let started = core
                .execute(player, PlayerBehavior::ConfirmEnterNode)
                .unwrap();
            let result = if terminal_time > 0 {
                core.advance_active_battle_by_after_seq(50, 0).unwrap()
            } else {
                started
            };
            let BehaviorResult::BattleAdvanced {
                finished: true,
                battle_update,
                ..
            } = result
            else {
                panic!("actual public terminal result required")
            };
            let actual_time = battle_update.checkpoint.battle_time_ms;
            let update = serde_json::to_value(&battle_update).unwrap();
            if let Some(prior) = &prior_update {
                assert_eq!(prior, &update);
            }
            prior_update = Some(update);
            if terminal_time == 0 {
                assert_eq!(actual_time, 0);
            } else {
                assert!(actual_time >= terminal_time && actual_time <= 50);
            }
            assert!(core.state.active_battle.is_none());
            if !enabled {
                assert!(core.authoring_lifecycle().is_none());
                core.execute(player, PlayerBehavior::CompleteCombatResult)
                    .unwrap();
                continue;
            }
            let finished = core.authoring_lifecycle().unwrap();
            assert!(finished.is_ready());
            assert_eq!(finished.total_elapsed_ms, actual_time);
            let battle = finished.current.unwrap();
            assert_eq!(battle.last_battle_time_ms, actual_time);
            assert!(battle.time_zero_completed);
            assert_eq!(battle.status, TrialBattleStatus::Finished);
            assert_eq!(battle.terminal_verdict, Some(BattleVerdict::Victory));
            let observation = core.authoring_observation().unwrap();
            assert!(observation.active.is_none());
            crate::game::world::authoring_observation::assert_terminal_storage_contract(&core);
            let capsule = observation.last_terminal.unwrap();
            assert_eq!(capsule.key.entry_ordinal, battle.entry_ordinal);
            assert_eq!(capsule.key.battle_uuid, battle.battle_uuid);
            assert_eq!(capsule.actual_time_ms, actual_time);
            assert_eq!(capsule.outcome_id.as_str(), outcome_id);
            assert_eq!(capsule.verdict, BattleVerdict::Victory);
            assert_eq!(capsule.mandatory.key, Some(capsule.key));
            let retained = serde_json::to_value(&observation).unwrap();
            assert!(retained["last_terminal"].get("trace").is_none());
            if let Some(prior) = &prior_mandatory {
                assert_eq!(prior, &retained);
            }
            prior_mandatory = Some(retained.clone());
            let trace_view = core.authoring_trace().unwrap();
            assert!(trace_view.active.is_none());
            let (trace_key, trace) = trace_view.last_terminal.unwrap();
            assert_eq!(trace_key, capsule.key);
            assert_eq!(trace.enabled, trace_options.unwrap().limits.is_some());
            assert!(trace.gap.is_none());
            let trace_bytes = trace.ndjson.to_vec();
            let cloned = core.clone();
            assert!(std::ptr::eq(
                cloned
                    .authoring_observation()
                    .unwrap()
                    .last_terminal
                    .unwrap(),
                capsule
            ));
            let result = core.execute(player, PlayerBehavior::CompleteCombatResult);
            assert!(result.is_ok(), "{result:?}");
            let consumed = core.authoring_lifecycle().unwrap();
            assert!(!consumed.is_ready());
            assert_eq!(consumed.total_elapsed_ms, actual_time);
            assert_eq!(
                consumed.current.unwrap().terminal_verdict,
                battle.terminal_verdict
            );
            assert_eq!(consumed.current.unwrap().status, TrialBattleStatus::Exited);
            assert_eq!(
                serde_json::to_value(core.authoring_observation().unwrap()).unwrap(),
                retained
            );
            let trace_view = core.authoring_trace().unwrap();
            let (key, trace) = trace_view.last_terminal.unwrap();
            assert_eq!(key, trace_key);
            assert_eq!(trace.ndjson, trace_bytes);
        }
    }
}

include!("r09_combat.inc.rs");

// Private fixture and complete root oracle for the public application fault tests.
// The typed dynamic expression is compiled by the real skill catalog before entry.
impl GameCore {
    pub(crate) fn u03_expression_fault_fixture() -> (Self, Uuid, Uuid, u64) {
        let authored = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/r11/hello_world_result.ron"))
            .replace("Ratio(numerator: Read(Time(RemainingTime)), denominator: Const((value: 100, unit: Milliseconds)), rounding: Floor)",
                     "Ratio(numerator: Const((value: 1, unit: Magnitude)), denominator: Read(Stat(subject: Source, stat: SpellPower)), rounding: Floor)");
        let (mut core, player, employee) =
            r11_live_sustained_fixture(false, false, 0, Some(&authored));
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::ActivateSkill {
                employee_uuid: employee,
                skill_id: "r08_strike".into(),
                target: None,
            },
        )
        .unwrap();
        core.advance_active_battle_for_server_tick(400).unwrap();
        let activation = core
            .state
            .active_battle
            .as_ref()
            .unwrap()
            .battle
            .sustained_effect_snapshot()
            .unwrap()
            .active[0]
            .activation;
        (core, player, employee, activation)
    }
    pub(crate) fn u03_complete_root_fingerprint(&self) -> String {
        format!("{:?}", battle_advance_atomic_snapshot(self))
    }
    pub(crate) fn u03_exhaust_battle_revision_fixture(&mut self) {
        self.state
            .active_battle
            .as_mut()
            .unwrap()
            .battle_state_revision = u64::MAX;
    }
}

impl GameCore {
    pub(crate) fn u03_reconcile_origin_fixture() -> (Self, Uuid, Uuid, Position) {
        let mut core = GameCore::new(game_data_with_pve_encounters(), 0xB055_C007);
        let player_id = Uuid::from_u128(0xB055_C007);
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
        assert!(matches!(
            deployed,
            BehaviorResult::BattleUnitDeployed { .. }
        ));
        (core, player_id, employee_uuid, deploy_position)
    }

    pub(crate) fn u03_corrupt_reconcile_fixture(&mut self, kind: &str) {
        if kind == "context" {
            self.state.lifecycle.transition_to(GameState::InBattle {
                battle_uuid: Uuid::nil(),
            });
            return;
        }
        let active = self.state.active_battle.as_mut().unwrap();
        let unit_id = active
            .live_deployment
            .as_ref()
            .unwrap()
            .deployed_units
            .values()
            .next()
            .unwrap()
            .unit_instance_id;
        let unit = active.battle.units.get_mut(&unit_id).unwrap();
        match kind {
            "incapacitated" => {
                unit.stats.current_health = 0;
                unit.lifecycle = crate::game::battle::core::types::RuntimeUnitLifecycle::Dead;
                active.employee_attempt.roster = crate::game::employee::EmployeeRoster::new();
            }
            "corroded_roster" => {
                unit.corroded = true;
                active.employee_attempt.roster = crate::game::employee::EmployeeRoster::new();
            }
            "corroded_source" => {
                unit.corroded = true;
                unit.source_identity =
                    crate::game::battle::types::BattleUnitSourceIdentity::TestFixture {
                        base_uuid: Uuid::nil(),
                    };
            }
            _ => panic!("unknown corruption fixture"),
        }
    }
}

impl GameCore {
    pub(crate) fn u03_manual_reconcile_fixture() -> (Self, Uuid, Uuid, Position) {
        let TacticalWorldMutationFixture {
            mut core,
            player_id,
            first_employee_uuid,
            second_employee_uuid,
            first_position,
            second_position,
        } = tactical_world_mutation_fixture(0xB055_C008);
        execute_current_battle_mutation(
            &mut core,
            player_id,
            PlayerBehavior::DeployUnit {
                employee_uuid: first_employee_uuid,
                position: first_position,
                facing: FacingDirection::Right,
            },
        )
        .unwrap();
        let retained: Vec<_> = core
            .roster()
            .unwrap()
            .iter()
            .filter(|employee| employee.uuid != first_employee_uuid)
            .cloned()
            .collect();
        core.u03_corrupt_reconcile_fixture("corroded_roster");
        for employee in retained {
            core.state
                .active_battle
                .as_mut()
                .unwrap()
                .employee_attempt
                .roster
                .add(employee);
        }
        (core, player_id, second_employee_uuid, second_position)
    }
}

impl GameCore {
    pub(crate) fn u03_boss_activation_fault_fixture() -> Self {
        let authored = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/r12/world_corrosion.ron"
        ));
        let (mut core, _, _) =
            r12_live_cross_fixture(false, false, 0, Some(authored), false, true, true);
        core.state
            .active_battle
            .as_mut()
            .unwrap()
            .battle
            .u03_exhaust_boss_activation_fixture();
        core
    }
}

impl GameCore {
    pub(crate) fn u03_sustained_origin_fixture(unsupported: bool) -> (Self, Uuid, Uuid, u64) {
        let authored = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/r11/hello_world_result.ron"
        ));
        let authored = if unsupported {
            authored
                .replace(
                    "goodbye_skill: Some(\"r11_goodbye\")",
                    "goodbye_skill: None",
                )
                .replace(
                    "Read(Time(RemainingTime))",
                    "Const((value: 100, unit: Milliseconds))",
                )
        } else {
            authored.to_owned()
        };
        let (mut core, player, employee) =
            r11_live_sustained_fixture(false, false, 0, Some(&authored));
        execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::ActivateSkill {
                employee_uuid: employee,
                skill_id: "r08_strike".into(),
                target: None,
            },
        )
        .unwrap();
        core.advance_active_battle_for_server_tick(400).unwrap();
        let active = core.state.active_battle.as_mut().unwrap();
        let activation = active.battle.sustained_effect_snapshot().unwrap().active[0].activation;
        for unit in active.battle.units.values_mut() {
            if unit.owner == crate::game::enums::Side::Player {
                unit.resonance_current = unit.resonance_max;
            }
        }
        (core, player, employee, activation)
    }
    pub(crate) fn u03_corrupt_goodbye_fixture(&mut self) {
        self.state
            .active_battle
            .as_mut()
            .unwrap()
            .battle
            .u03_missing_goodbye_definition_fixture();
    }
    pub(crate) fn u03_admitted_deadline_fixture(focus: u32) -> (Self, Uuid, Uuid) {
        let authored = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/r11/hello_world_result.ron"
        ))
        .replace("focus_time_ms: 100", &format!("focus_time_ms: {focus}"));
        let (mut core, player, employee) =
            r11_live_sustained_fixture(false, false, 0, Some(&authored));
        core.state
            .active_battle
            .as_mut()
            .unwrap()
            .execution
            .u03_exhaust_simulation_time_fixture();
        (core, player, employee)
    }
    pub(crate) fn u03_swap_origin_fixture(missing_first: bool) -> (Self, Uuid, Uuid, Uuid) {
        let TacticalWorldMutationFixture {
            mut core,
            player_id,
            first_employee_uuid,
            second_employee_uuid,
            first_position,
            second_position,
        } = tactical_world_mutation_fixture(0xB055_C009);
        for (employee_uuid, position) in [
            (first_employee_uuid, first_position),
            (second_employee_uuid, second_position),
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
        let active = core.state.active_battle.as_mut().unwrap();
        let employee = if missing_first {
            first_employee_uuid
        } else {
            second_employee_uuid
        };
        let unit =
            active.live_deployment.as_ref().unwrap().deployed_units[&employee].unit_instance_id;
        assert!(active.battle.units.remove(&unit).is_some());
        (core, player_id, first_employee_uuid, second_employee_uuid)
    }
}

#[test]
fn trusted_goodbye_failure_keeps_private_gamecore_retry_and_complete_root() {
    let (mut core, player, employee, activation) = GameCore::u03_sustained_origin_fixture(false);
    core.u03_corrupt_goodbye_fixture();
    let before = battle_advance_atomic_snapshot(&core);
    for _ in 0..2 {
        let error = execute_current_battle_mutation(
            &mut core,
            player,
            PlayerBehavior::ReinputSustainedSkill {
                employee_uuid: employee,
                skill_id: "r08_strike".into(),
                activation,
                target: None,
            },
        )
        .unwrap_err();
        assert!(matches!(error, GameError::BattleRuntimeFault(fault)
            if fault.site == "reinput_goodbye_definition"));
        assert_eq!(battle_advance_atomic_snapshot(&core), before);
    }
}

include!("u04_continuation.inc.rs");

include!("u06_interaction.inc.rs");

include!("input_plan.inc.rs");

include!("prepared_volley.inc.rs");

include!("paired_areas.inc.rs");

include!("fusion_placement.inc.rs");

include!("status_strike.inc.rs");

include!("relation_action.inc.rs");

include!("counter_channel.inc.rs");
