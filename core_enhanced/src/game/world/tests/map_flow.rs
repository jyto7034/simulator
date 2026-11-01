use super::*;
use crate::game::behavior::{ActionCapabilityDto, ActionDisabledReasonCode, ActionScope};
#[test]
fn start_new_game_exposes_initial_map_without_entering_a_node_session() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);

    start_new_game_with_default_starters(&mut core, player_id);
    let map = match core
        .execute(player_id, PlayerBehavior::RequestMapData)
        .unwrap()
    {
        BehaviorResult::MapState { map } => map,
        other => panic!("expected map state, got {other:?}"),
    };

    assert_eq!(
        core.state
            .run
            .as_ref()
            .expect("run state")
            .map_progression
            .current_node_id,
        core.state
            .run
            .as_ref()
            .expect("run state")
            .map
            .nodes
            .iter()
            .find(|node| node.category == MapNodeCategory::Start)
            .map(|node| node.id)
    );
    assert!(!map.nodes.is_empty());
    let available_state_node_ids = map_node_ids_by_state(&map, MapNodeState::Available);
    assert!(!available_state_node_ids.is_empty());
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .terminals
        .iter()
        .all(|terminal| core
            .state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(terminal.node_id)
            .is_some()));
    assert!(available_state_node_ids.iter().all(|node_id| {
        map.nodes
            .iter()
            .any(|node| node.id == *node_id && node.state == MapNodeState::Available)
    }));
    assert!(core.state.node_session.is_none());
    assert!(matches!(core.get_state(), GameState::ViewingMap));
}

#[test]
fn map_projection_exposes_only_qualitative_difficulty_and_dynamic_readiness() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let (combat_node_id, fixed_difficulty) = core
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .nodes
        .iter()
        .find_map(|node| {
            node.combat_difficulty
                .clone()
                .map(|difficulty| (node.id, difficulty))
        })
        .expect("manifested Floor must contain a combat difficulty authority");
    let before = core.current_map_view().unwrap();
    let before_node = before
        .nodes
        .iter()
        .find(|node| node.id == combat_node_id)
        .unwrap();
    assert!(before_node.difficulty_band.is_some());
    assert!(before_node.relative_threat.is_some());
    assert!(before_node.readiness_warnings.is_empty());

    let wire = serde_json::to_value(before_node).unwrap();
    assert!(wire.get("difficulty_band").is_some());
    assert!(wire.get("relative_threat").is_some());
    assert!(wire.get("target_power").is_none());
    assert!(wire.get("floor_power_anchor").is_none());
    assert!(wire.get("power_model_version").is_none());
    assert!(wire.get("difficulty_profile_id").is_none());

    for employee in core.roster_mut().unwrap().iter_mut() {
        employee.condition.trauma = 100;
    }
    let after = core.current_map_view().unwrap();
    let after_node = after
        .nodes
        .iter()
        .find(|node| node.id == combat_node_id)
        .unwrap();
    assert_eq!(after_node.difficulty_band, before_node.difficulty_band);
    assert_eq!(
        after_node.relative_threat,
        Some(crate::game::map::RelativeThreatBandDto::VeryHigh),
        "zero combat-eligible certified power must be presented as maximum relative threat"
    );
    assert!(after_node
        .readiness_warnings
        .contains(&crate::game::map::CombatReadinessWarningDto::NoDeployableEmployees));
    assert!(after_node
        .readiness_warnings
        .contains(&crate::game::map::CombatReadinessWarningDto::TraumaBreakdown));
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .map
            .node(combat_node_id)
            .unwrap()
            .combat_difficulty
            .as_ref(),
        Some(&fixed_difficulty),
        "readiness changes must not rewrite the manifested target or anchor"
    );
}

#[test]
fn manifested_combat_preview_rejects_missing_node_difficulty_authority() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_defense",
        MapNodePayload::Encounter {
            encounter_id: Some("defense_encounter".to_string()),
        },
    );
    core.state
        .run
        .as_mut()
        .unwrap()
        .map
        .node_mut(node_id)
        .unwrap()
        .combat_difficulty = None;

    let error = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .expect_err("manifested Combat nodes must not receive Standard by fallback");
    assert!(matches!(
        error,
        GameError::InvalidStaticData(ref message)
            if message.contains("missing combat difficulty authority")
    ));
}

#[test]
fn common_map_and_checkpoint_capabilities_are_typed_and_revalidated() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);

    assert_eq!(
        core.get_action_capabilities(),
        vec![ActionCapabilityDto::enabled(
            ActionKind::StartNewGame,
            ActionScope::Gameplay,
        )]
    );
    let irrelevant = core
        .execute(player_id, PlayerBehavior::RequestMapData)
        .unwrap_err();
    assert!(matches!(
        irrelevant,
        GameError::ActionUnavailable {
            action: ActionKind::RequestMapData,
            reason: ActionDisabledReasonCode::ContextIrrelevant
        }
    ));

    start_new_game_with_default_starters(&mut core, player_id);
    let capabilities = core.get_action_capabilities();
    assert!(capabilities.contains(&ActionCapabilityDto::enabled(
        ActionKind::RequestMapData,
        ActionScope::Query,
    )));
    assert!(capabilities.contains(&ActionCapabilityDto::enabled(
        ActionKind::SelectMapNode,
        ActionScope::Gameplay,
    )));
    assert!(capabilities.contains(&ActionCapabilityDto::disabled(
        ActionKind::LoadRunCheckpoint,
        ActionScope::Recovery,
        ActionDisabledReasonCode::CheckpointUnavailable,
    )));

    let checkpoint_error = core
        .execute(player_id, PlayerBehavior::LoadRunCheckpoint)
        .unwrap_err();
    assert!(matches!(
        checkpoint_error,
        GameError::ActionUnavailable {
            action: ActionKind::LoadRunCheckpoint,
            reason: ActionDisabledReasonCode::CheckpointUnavailable
        }
    ));

    let invalid_node = MapNodeId::new(Uuid::from_u128(u128::MAX));
    let payload_error = core
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: invalid_node,
            },
        )
        .unwrap_err();
    assert!(matches!(
        payload_error,
        GameError::ActionUnavailable {
            action: ActionKind::SelectMapNode,
            reason: ActionDisabledReasonCode::NoEligiblePayload
        }
    ));

    let node = core
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .nodes
        .iter()
        .find(|node| node.category != MapNodeCategory::Start)
        .unwrap();
    let node_id = node.id;
    let kind_id = node.kind_id.clone();
    let category = node.category;
    core.state.node_session = Some(crate::game::map::NodeSession {
        node_id,
        kind_id: kind_id.clone(),
        category,
        payload: node.payload.clone(),
    });
    core.transition_to(GameState::InNode {
        node_id,
        kind_id,
        category,
    })
    .unwrap();

    let stale_projection_error = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap_err();
    assert!(matches!(
        stale_projection_error,
        GameError::ActionUnavailable {
            action: ActionKind::SelectMapNode,
            reason: ActionDisabledReasonCode::ContextIrrelevant
        }
    ));
}

#[test]
fn combat_preview_seed_uses_full_generated_node_uuid() {
    const MAP_NS: u64 = 0x524D_4150; // "RMAP"
    const PREVIEW_NS: u64 = 0x5052_4556; // "PREV"

    let core = GameCore::new(empty_game_data(), 123);
    let map_seed = crate::game::determinism::seed_with_namespace(123, MAP_NS);
    let node_ids = (0..32)
        .map(|index| {
            MapNodeId::new(crate::game::determinism::uuid_v4_from_seed(
                map_seed, MAP_NS, index,
            ))
        })
        .collect::<Vec<_>>();

    assert!(
        node_ids
            .windows(2)
            .all(|pair| pair[0].0.as_bytes()[..8] == pair[1].0.as_bytes()[..8]),
        "this regression fixture must match deterministic UUID prefix behavior"
    );

    let pair = node_ids
        .iter()
        .copied()
        .flat_map(|left| node_ids.iter().copied().map(move |right| (left, right)))
        .find(|(left, right)| {
            left != right
                && core.node_seed(*left, PREVIEW_NS) % 7 != core.node_seed(*right, PREVIEW_NS) % 7
        })
        .expect("full UUID seed mixing should vary combat archetype buckets");

    let left_seed = core.node_seed(pair.0, PREVIEW_NS);
    let right_seed = core.node_seed(pair.1, PREVIEW_NS);
    let game_data = game_data_with_pve_encounters();
    let left = crate::game::combat_preview::CombatPreview::try_generate_for_node(
        pair.0,
        MapNodeCategory::Combat,
        Some("elite_risk_encounter"),
        game_data.as_ref(),
        left_seed,
    )
    .expect("left combat preview should generate");
    let right = crate::game::combat_preview::CombatPreview::try_generate_for_node(
        pair.1,
        MapNodeCategory::Combat,
        Some("elite_risk_encounter"),
        game_data.as_ref(),
        right_seed,
    )
    .expect("right combat preview should generate");

    assert_ne!(left.archetype, right.archetype);
    assert_ne!(left.battlefield_template_id, right.battlefield_template_id);
}

#[test]
fn generated_map_node_ids_feed_distinct_node_seeds() {
    const PREVIEW_NS: u64 = 0x5052_4556; // "PREV"

    let core = GameCore::new(empty_game_data(), 123);
    let map = test_facility_map(123);
    let mut seeds = map
        .nodes
        .iter()
        .filter(|node| {
            matches!(
                node.category,
                MapNodeCategory::Combat | MapNodeCategory::Boss
            )
        })
        .map(|node| core.node_seed(node.id, PREVIEW_NS))
        .collect::<Vec<_>>();

    seeds.sort_unstable();
    seeds.dedup();

    assert!(
        seeds.len() > 1,
        "generated map combat nodes should not collapse to one preview seed"
    );
}

#[test]
fn combat_node_preview_exposes_basic_briefing() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        MapNodePayload::Encounter {
            encounter_id: Some("low_risk_encounter".to_string()),
        },
    );

    let result = core
        .execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();

    let BehaviorResult::NodePreview {
        combat_preview: Some(preview),
        ..
    } = result
    else {
        panic!("expected combat node preview");
    };
    assert_eq!(preview.node_id, node_id);
    assert_eq!(preview.encounter_id.as_deref(), Some("low_risk_encounter"));
    assert!(!preview.deployment_zones.is_empty());
    assert!(!preview.spawn_zones.is_empty());
    assert!(!preview.spawn_waves.is_empty());
    assert_eq!(preview.spawn_waves[0].activation.at_time_ms(), Some(0));
}

#[test]
fn node_confirm_allows_reselecting_another_available_node_before_entering() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    {
        let run = core.state.run.as_mut().expect("run state");
        let candidates = run
            .map
            .nodes
            .iter()
            .filter(|node| node.category != MapNodeCategory::Start)
            .take(2)
            .map(|node| node.id)
            .collect::<Vec<_>>();
        assert_eq!(candidates.len(), 2);
        for node_id in &candidates {
            run.map.node_mut(*node_id).expect("node exists").state = MapNodeState::Available;
        }
        run.map.start_node_ids = candidates;
        run.map_progression.current_node_id = None;
    }
    let run = core.state.run.as_ref().expect("run state");
    let template = core
        .game_data
        .map_catalogs
        .facility_templates
        .template(&run.map.map_template_id)
        .expect("generated map template");
    let selectable_node_ids = run.map_progression.selectable_node_ids(&run.map, template);
    assert!(
        selectable_node_ids.len() >= 2,
        "map fixture should expose at least two available nodes"
    );
    let first_node_id = selectable_node_ids[0];
    let second_node_id = selectable_node_ids[1];
    {
        let map = &mut core.state.run.as_mut().expect("run state").map;
        for (node_id, kind_id) in [
            (first_node_id, "save_point_first_preview"),
            (second_node_id, "save_point_second_preview"),
        ] {
            let node = map.node_mut(node_id).expect("node exists");
            node.category = MapNodeCategory::SavePoint;
            node.kind_id = MapNodeKindId::new(kind_id);
            node.payload = MapNodePayload::SavePoint;
        }
    }

    let first_preview = core
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: first_node_id,
            },
        )
        .unwrap();
    assert!(matches!(
        first_preview,
        BehaviorResult::NodePreview {
            node_id,
            ..
        } if node_id == first_node_id
    ));
    assert!(matches!(
        core.get_state(),
        GameState::NodeConfirm {
            node_id,
            ..
        } if node_id == first_node_id
    ));
    assert!(action_capability_enabled(&core, ActionKind::SelectMapNode));

    let second_preview = core
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: second_node_id,
            },
        )
        .unwrap();
    assert!(matches!(
        second_preview,
        BehaviorResult::NodePreview {
            node_id,
            ..
        } if node_id == second_node_id
    ));
    assert!(matches!(
        core.get_state(),
        GameState::NodeConfirm {
            node_id,
            ..
        } if node_id == second_node_id
    ));
    assert_eq!(
        core.state
            .node_session
            .as_ref()
            .expect("preview session should be staged")
            .node_id,
        second_node_id
    );

    let entered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    match entered {
        BehaviorResult::NodeEntered { node_id, .. }
        | BehaviorResult::SavePointState { node_id, .. }
        | BehaviorResult::HeadquartersContactState { node_id, .. } => {
            assert_eq!(node_id, second_node_id);
        }
        other => panic!("expected second node to be entered, got {other:?}"),
    }
}

#[test]
fn select_map_node_rejects_nodes_outside_core_navigation() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let run = core.state.run.as_ref().expect("run state");
    let template = core
        .game_data
        .map_catalogs
        .facility_templates
        .template(&run.map.map_template_id)
        .expect("generated map template");
    let selectable_node_ids = run.map_progression.selectable_node_ids(&run.map, template);
    let blocked_node_id = run
        .map
        .nodes
        .iter()
        .find(|node| !selectable_node_ids.contains(&node.id))
        .expect("generated map should contain a non-selectable future node")
        .id;
    assert!(!run
        .map_progression
        .is_node_selectable(&run.map, template, blocked_node_id));

    let result = core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: blocked_node_id,
        },
    );

    assert!(matches!(
        result,
        Err(GameError::ActionUnavailable {
            action: ActionKind::SelectMapNode,
            reason: ActionDisabledReasonCode::NoEligiblePayload
        })
    ));
    assert!(core.state.node_session.is_none());
    assert!(matches!(core.get_state(), GameState::ViewingMap));
}

#[test]
fn confirmed_gate_transition_advances_floor_and_recovers_living_trauma_without_checkpoint() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
    {
        let roster = core.roster_mut().unwrap();
        roster
            .get_mut(&employee_ids[0])
            .unwrap()
            .condition
            .physical_damage = 5;
        roster.get_mut(&employee_ids[0]).unwrap().condition.trauma = 90;
        roster.get_mut(&employee_ids[1]).unwrap().condition.trauma = 100;
        roster.get_mut(&employee_ids[2]).unwrap().condition =
            EmployeeCondition::new(MAX_PHYSICAL_DAMAGE, 50);
    }
    let gate_node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Gate,
        "gate_stairs",
        MapNodePayload::None,
    );

    let profile_before = core
        .roster()
        .unwrap()
        .iter()
        .map(|employee| (employee.uuid, employee.profile.clone()))
        .collect::<std::collections::HashMap<_, _>>();
    let preview = core
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: gate_node_id,
            },
        )
        .unwrap();
    let BehaviorResult::NodePreview {
        trauma_recovery_preview: Some(preview),
        ..
    } = preview
    else {
        panic!("Gate NodePreview must include authoritative recovery projection");
    };
    assert_eq!(
        preview.recovery,
        crate::game::data::run_policy_data::TraumaRecovery::CurrentPercent { percent: 10 }
    );
    assert!(preview.physical_damage_unchanged);
    assert!(!preview.replaces_run_checkpoint);
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let BehaviorResult::FloorAdvanced {
        game_mode,
        floor_index,
        ..
    } = result
    else {
        panic!("expected gate to advance floor");
    };
    assert_eq!(game_mode, GameMode::Expedition);
    assert_eq!(floor_index, 1);
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    let roster = core.roster().unwrap();
    assert_eq!(
        roster
            .get(&employee_ids[0])
            .unwrap()
            .condition
            .physical_damage,
        5
    );
    assert_eq!(roster.get(&employee_ids[0]).unwrap().condition.trauma, 81);
    assert_eq!(roster.get(&employee_ids[1]).unwrap().condition.trauma, 90);
    assert_eq!(roster.get(&employee_ids[2]).unwrap().condition.trauma, 50);
    for employee in roster.iter() {
        assert_eq!(
            &employee.profile,
            profile_before.get(&employee.uuid).unwrap(),
            "Gate recovery must preserve authored personality and aptitude"
        );
    }
    assert!(!core.state.run_checkpoint.can_load());
    assert_eq!(
        core.state.run.as_ref().unwrap().run_progression.game_mode,
        GameMode::Expedition
    );

    let trauma_after_transition = core
        .roster()
        .unwrap()
        .iter()
        .map(|employee| (employee.uuid, employee.condition.trauma))
        .collect::<std::collections::HashMap<_, _>>();
    assert!(core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .is_err());
    for employee in core.roster().unwrap().iter() {
        assert_eq!(
            employee.condition.trauma, trauma_after_transition[&employee.uuid],
            "Gate transition must apply recovery exactly once"
        );
    }
}

#[test]
fn endless_gate_transition_advances_floor_without_max_cap_or_run_completion() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);

    let initial_snapshot = core.get_run_snapshot_json().unwrap();
    let initial_progression = initial_snapshot["run_progression"]
        .as_object()
        .expect("run progression object");
    assert_eq!(initial_progression["game_mode"], "Expedition");
    assert_eq!(initial_progression["floor_index"], 0);
    assert!(
        !initial_progression.contains_key("max_floors"),
        "Endless snapshots must not expose a max floor cap"
    );
    assert!(initial_progression["current_floor_seed"].is_u64());
    assert_eq!(core.state.player_profile.highest_floor_reached, 1);
    assert_eq!(core.state.player_profile.longest_deathless_floor_streak, 0);

    let gate_node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Gate,
        "gate_stairs",
        MapNodePayload::None,
    );

    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: gate_node_id,
        },
    )
    .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let BehaviorResult::FloorAdvanced {
        game_mode,
        floor_index,
        map,
        ..
    } = result
    else {
        panic!("Endless Gate should advance to the next floor instead of completing the run");
    };
    assert_eq!(game_mode, GameMode::Expedition);
    assert_eq!(floor_index, 1);
    assert!(!map.nodes.is_empty());
    assert!(matches!(core.get_state(), GameState::ViewingMap));

    let run = core.state.run.as_ref().expect("run should continue");
    assert_eq!(run.run_progression.game_mode, GameMode::Expedition);
    assert_eq!(run.run_progression.floor_index(), 1);
    assert_eq!(run.run_progression.max_floors(), None);
    assert_eq!(core.state.player_profile.highest_floor_reached, 2);
    assert_eq!(core.state.player_profile.longest_deathless_floor_streak, 1);
    assert_eq!(
        run.map
            .node(run.map.terminals[0].node_id)
            .expect("terminal node exists")
            .category,
        MapNodeCategory::Gate
    );

    let advanced_snapshot = core.get_run_snapshot_json().unwrap();
    let advanced_progression = advanced_snapshot["run_progression"]
        .as_object()
        .expect("run progression object");
    assert_eq!(advanced_progression["game_mode"], "Expedition");
    assert_eq!(advanced_progression["floor_index"], 1);
    assert!(
        !advanced_progression.contains_key("max_floors"),
        "Endless snapshots must stay max-free after Gate transitions"
    );
}

#[test]
fn gate_preview_does_not_advance_floor_or_apply_reward_without_confirm() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_id = core.roster().unwrap().combat_eligible_employee_ids()[0];
    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_id).unwrap();
        employee.condition.trauma = 80;
    }
    let gate_node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Gate,
        "gate_stairs",
        MapNodePayload::None,
    );
    let before_floor_index = core
        .state
        .run
        .as_ref()
        .unwrap()
        .run_progression
        .floor_index();

    let result = core
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: gate_node_id,
            },
        )
        .unwrap();

    let BehaviorResult::NodePreview { category, .. } = result else {
        panic!("expected gate selection to return node preview");
    };
    assert_eq!(category, MapNodeCategory::Gate);
    assert!(matches!(core.get_state(), GameState::NodeConfirm { .. }));
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .run_progression
            .floor_index(),
        before_floor_index
    );
    let employee = core.roster().unwrap().get(&employee_id).unwrap();
    assert_eq!(employee.condition.trauma, 80);
    assert!(!core.state.run_checkpoint.can_load());
}

#[test]
fn combat_node_confirm_starts_live_battle() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Combat,
        "combat_low_risk",
        MapNodePayload::Encounter {
            encounter_id: Some("low_risk_encounter".to_string()),
        },
    );

    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    assert!(matches!(result, BehaviorResult::BattleAdvanced { .. }));
    assert!(matches!(core.get_state(), GameState::InBattle { .. }));
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
}

#[test]
fn map_progression_selects_completes_and_unlocks_next_nodes() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let map = match core
        .execute(player_id, PlayerBehavior::RequestMapData)
        .unwrap()
    {
        BehaviorResult::MapState { map } => map,
        other => panic!("expected map state, got {other:?}"),
    };
    let available_state_node_ids = map_node_ids_by_state(&map, MapNodeState::Available);
    assert!(!available_state_node_ids.is_empty());
    let first_node_id = available_state_node_ids[0];
    {
        let node = core
            .state
            .run
            .as_mut()
            .expect("run state")
            .map
            .node_mut(first_node_id)
            .expect("node exists");
        node.category = MapNodeCategory::SavePoint;
        node.kind_id = crate::game::map::MapNodeKindId::new("save_point");
        node.payload = MapNodePayload::SavePoint;
    }

    let preview = core
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: first_node_id,
            },
        )
        .unwrap();
    assert!(matches!(
        preview,
        BehaviorResult::NodePreview {
            node_id,
            ..
        } if node_id == first_node_id
    ));
    let entered = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    match entered {
        BehaviorResult::NodeEntered { node_id, .. }
        | BehaviorResult::SavePointState { node_id, .. }
        | BehaviorResult::HeadquartersContactState { node_id, .. } => {
            assert_eq!(node_id, first_node_id);
        }
        BehaviorResult::ShopState { .. } | BehaviorResult::RewardState { .. } => {}
        other => panic!("expected entered map node content, got {other:?}"),
    }
    let session = core
        .state
        .node_session
        .as_ref()
        .expect("node session is staged after node entry");
    assert_eq!(session.node_id, first_node_id);
    assert!(matches!(core.get_state(), GameState::InNode { .. }));

    let completed = core
        .execute(player_id, PlayerBehavior::CompleteNode)
        .unwrap();
    let map = match completed {
        BehaviorResult::NodeCompleted { map, .. } => map,
        other => panic!("expected node completed, got {other:?}"),
    };
    let completed_state_node_ids = map_node_ids_by_state(&map, MapNodeState::Completed);
    let available_state_node_ids = map_node_ids_by_state(&map, MapNodeState::Available);
    assert!(completed_state_node_ids.contains(&first_node_id));
    assert!(!available_state_node_ids.contains(&first_node_id));
    assert!(!available_state_node_ids.is_empty());
    assert!(core.state.node_session.is_none());
    assert!(matches!(core.get_state(), GameState::ViewingMap));

    let err = core
        .execute(
            player_id,
            PlayerBehavior::SelectMapNode {
                node_id: first_node_id,
            },
        )
        .unwrap_err();
    assert!(matches!(
        err,
        GameError::ActionUnavailable {
            action: ActionKind::SelectMapNode,
            reason: ActionDisabledReasonCode::NoEligiblePayload
        }
    ));
}

#[test]
fn generated_standard_maps_use_fixed_gate_and_nothing_there_normal_boss_terminal() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Tutorial);

    let map = match core
        .execute(player_id, PlayerBehavior::RequestMapData)
        .unwrap()
    {
        BehaviorResult::MapState { map } => map,
        other => panic!("expected map state, got {other:?}"),
    };

    let run = core.state.run.as_ref().expect("run state");
    let terminal_node_id = run.map.terminals[0].node_id;
    let terminal = run
        .map
        .node(terminal_node_id)
        .expect("terminal node exists");
    assert_eq!(terminal.category, MapNodeCategory::Gate);
    assert_eq!(terminal.kind_id.as_str(), "gate_stairs");
    let template = core
        .game_data
        .map_catalogs
        .facility_templates
        .template(&run.map.map_template_id)
        .expect("generated map template");
    let terminal_parent_ids = template
        .connections
        .iter()
        .filter_map(|connection| {
            (connection.to_slot_id == terminal.slot_id)
                .then(|| {
                    run.map
                        .nodes
                        .iter()
                        .find(|node| node.slot_id == connection.from_slot_id)
                        .map(|node| node.id)
                })
                .flatten()
        })
        .collect::<Vec<_>>();
    assert!(
        !terminal_parent_ids.is_empty(),
        "Standard Floor 1 Gate must have an incoming fixed guard"
    );
    for parent_id in terminal_parent_ids {
        let parent = run.map.node(parent_id).expect("gate parent exists");
        assert_eq!(parent.kind_id.as_str(), "combat_monster");
        assert!(matches!(
            &parent.payload,
            MapNodePayload::Encounter {
                encounter_id: Some(id)
            } if id == "normal_breach_response"
        ));
    }

    assert!(map.nodes.iter().any(|node| {
        node.category == MapNodeCategory::Combat
            && matches!(
                &node.payload,
                MapNodePayload::Encounter {
                    encounter_id: Some(_)
                }
            )
    }));

    let mut final_floor_progression = run.run_progression.clone();
    final_floor_progression.mode_state = crate::game::map::RunProgressionModeState::Tutorial {
        floor_index: run_policy().setup.standard_floor_count - 1,
        max_floors: run_policy().setup.standard_floor_count,
    };
    let mut encounter_progression =
        crate::game::encounter_progression::EncounterProgressionState::default();
    let (final_map, _, _) = core.generate_current_floor_map(
        &final_floor_progression,
        &crate::game::map::FacilityRunState::default(),
        &mut encounter_progression,
    );
    let final_terminal = final_map
        .node(final_map.terminals[0].node_id)
        .expect("final terminal exists");
    assert_eq!(final_terminal.category, MapNodeCategory::Boss);
    assert_eq!(
        final_map.terminals[0].resolution,
        FacilityTerminalResolution::StandardCompletion
    );
    assert!(matches!(
        &final_terminal.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if id == "suppress_nothing_there"
    ));
    assert_eq!(
        final_terminal.combat_difficulty.as_ref().unwrap().class,
        crate::game::map::MapNodeCombatDifficultyClass::Role(
            crate::game::data::run_policy_data::CombatNodeDifficultyRole::NormalBoss
        )
    );
    assert!(final_map.nodes.iter().all(|node| {
        node.combat_difficulty.as_ref().is_none_or(|difficulty| {
            difficulty.class
                != crate::game::map::MapNodeCombatDifficultyClass::Role(
                    crate::game::data::run_policy_data::CombatNodeDifficultyRole::FinalBoss,
                )
        })
    }));
}

#[test]
fn map_encounter_assignment_respects_node_kind_and_boss_purpose() {
    let core = GameCore::new(game_data_with_pve_encounters(), 123);
    let normal_node_id = MapNodeId::new(Uuid::from_u128(0xA001));
    let elite_node_id = MapNodeId::new(Uuid::from_u128(0xA002));
    let boss_node_id = MapNodeId::new(Uuid::from_u128(0xA003));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new("standard_final_a"),
        facility_template_version: FACILITY_TEMPLATE_VERSION,
        nodes: vec![
            MapNode {
                id: normal_node_id,
                slot_id: MapSlotId::new("entry"),
                kind_id: MapNodeKindId::new("combat_monster"),
                category: MapNodeCategory::Combat,
                state: MapNodeState::Available,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Encounter { encounter_id: None },
                omen: None,
                combat_difficulty: None,
            },
            MapNode {
                id: elite_node_id,
                slot_id: MapSlotId::new("final_approach"),
                kind_id: MapNodeKindId::new("combat_elite"),
                category: MapNodeCategory::Combat,
                state: MapNodeState::Unavailable,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Encounter { encounter_id: None },
                omen: None,
                combat_difficulty: None,
            },
            MapNode {
                id: boss_node_id,
                slot_id: MapSlotId::new("final_chamber"),
                kind_id: MapNodeKindId::new("boss_abnormality"),
                category: MapNodeCategory::Boss,
                state: MapNodeState::Unavailable,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Encounter { encounter_id: None },
                omen: None,
                combat_difficulty: None,
            },
        ],
        start_node_ids: vec![normal_node_id],
        terminals: vec![MapTerminalNode {
            node_id: boss_node_id,
            resolution: FacilityTerminalResolution::RunFinalBoss,
        }],
    };
    let run_progression = RunProgression::new(123, GameMode::Expedition, 3);
    let mut encounter_progression =
        crate::game::encounter_progression::EncounterProgressionState::default();

    super::map_encounters::assign_map_encounters(
        &core.game_data.pve_data,
        &mut map,
        &run_progression,
        &mut encounter_progression,
        core.run_policy(),
    );

    let normal = map.node(normal_node_id).expect("normal node exists");
    assert!(matches!(
        &normal.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
            encounter.encounter_class == crate::game::data::pve_data::PveEncounterClass::Normal
                && encounter.primary_abnormality_id.is_none()
                && encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Defense)
        })
    ));
    let elite = map.node(elite_node_id).expect("elite node exists");
    assert!(matches!(
        &elite.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
            encounter.encounter_class == crate::game::data::pve_data::PveEncounterClass::Elite
                && encounter.primary_abnormality_id.is_some()
                && encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Defense)
        })
    ));
    let boss = map.node(boss_node_id).expect("boss node exists");
    assert!(matches!(
        &boss.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
            encounter.encounter_class == crate::game::data::pve_data::PveEncounterClass::NormalBoss
                && encounter.primary_abnormality_id.is_some()
                && encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Boss)
        })
    ));
}

#[test]
fn completed_boss_omen_chain_forces_single_boss_node_ahead_of_current_room() {
    let game_data = live_game_data_from_ron();
    let current_node_id = MapNodeId::new(Uuid::from_u128(0xB0_0001));
    let side_node_id = MapNodeId::new(Uuid::from_u128(0xB0_0002));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new("standard_intro_a"),
        facility_template_version: FACILITY_TEMPLATE_VERSION,
        nodes: vec![
            MapNode {
                id: current_node_id,
                slot_id: MapSlotId::new("entry"),
                kind_id: MapNodeKindId::new("event_story"),
                category: MapNodeCategory::Event,
                state: MapNodeState::Completed,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Event {
                    event_id: Some(crate::game::data::event_data::EventId::new(
                        "white_night_confession_01",
                    )),
                },
                omen: None,
                combat_difficulty: None,
            },
            MapNode {
                id: side_node_id,
                slot_id: MapSlotId::new("intake_corridor"),
                kind_id: MapNodeKindId::new("combat_monster"),
                category: MapNodeCategory::Combat,
                state: MapNodeState::Available,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Encounter { encounter_id: None },
                omen: None,
                combat_difficulty: Some(crate::game::map::MapNodeCombatDifficulty {
                    power_model_version: 1,
                    difficulty_profile_id: "default_facility".to_string(),
                    class: crate::game::map::MapNodeCombatDifficultyClass::Ordinary(
                        crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Standard,
                    ),
                    revival: None,
                    floor_power_anchor: 175,
                    target_power: 175,
                }),
            },
        ],
        start_node_ids: vec![side_node_id],
        terminals: vec![],
    };
    let mut progression = crate::game::map::MapProgression {
        current_node_id: Some(current_node_id),
        available_node_ids: vec![side_node_id],
        completed_node_ids: vec![current_node_id],
    };
    let mut run_progression = RunProgression::new(123, GameMode::Expedition, 3);
    run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Expedition { floor_index: 3 };
    let mut boss_omen = crate::game::boss_omen::BossOmenRunState {
        active: Some(crate::game::boss_omen::ActiveBossOmenState {
            chain_id: crate::game::data::boss_omen_data::BossOmenChainId::new("white_night"),
            boss_abnormality_id: "o-01-45_white_night".to_string(),
            next_step_index: 1,
            completed_steps: vec![crate::game::data::boss_omen_data::BossOmenStepId::new(
                "confession_01",
            )],
        }),
        ..Default::default()
    };

    crate::game::boss_omen::force_boss_node_if_ready(
        &mut map,
        &mut progression,
        &run_progression,
        game_data.as_ref(),
        &mut boss_omen,
    );

    let forced_boss_id = boss_omen
        .forced_boss_node_id()
        .expect("completed chain should force a boss node");
    let forced_boss = map.node(forced_boss_id).expect("forced boss node exists");
    assert_eq!(forced_boss.category, MapNodeCategory::Boss);
    assert_eq!(map.map_template_id.as_str(), "boss_omen_capstone");
    assert_eq!(map.nodes.len(), 2);
    assert_eq!(map.terminal_resolution(forced_boss_id), None);
    let exit_gate = map
        .terminals
        .iter()
        .find(|terminal| terminal.resolution == FacilityTerminalResolution::BossOmenExitGate)
        .expect("authored capstone must precommit its exit Gate");
    assert!(map.node(exit_gate.node_id).is_some_and(|node| {
        node.category == MapNodeCategory::Gate
            && node.visibility == MapNodeVisibility::Concealed
            && node.state == MapNodeState::Unavailable
    }));
    assert!(matches!(
        &forced_boss.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if id == "suppress_white_night"
    ));
    assert_eq!(progression.available_node_ids, vec![forced_boss_id]);
    let capstone_template = game_data
        .map_catalogs
        .facility_templates
        .template(&map.map_template_id)
        .expect("boss omen capstone template");
    assert_eq!(
        progression.selectable_node_ids(&map, capstone_template),
        vec![forced_boss_id]
    );
    assert!(map.node(side_node_id).is_none());
}

#[test]
fn active_boss_omen_chain_overlays_matching_event_node() {
    let game_data = live_game_data_from_ron();
    let event_node_id = MapNodeId::new(Uuid::from_u128(0xB0_0101));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new("standard_intro_a"),
        facility_template_version: FACILITY_TEMPLATE_VERSION,
        nodes: vec![MapNode {
            id: event_node_id,
            slot_id: MapSlotId::new("intake_corridor"),
            kind_id: MapNodeKindId::new("event_story"),
            category: MapNodeCategory::Event,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Event {
                event_id: Some(crate::game::data::event_data::EventId::new(
                    "white_night_confession_01",
                )),
            },
            omen: None,
            combat_difficulty: None,
        }],
        start_node_ids: vec![event_node_id],
        terminals: vec![],
    };
    let mut run_progression = RunProgression::new(123, GameMode::Expedition, 3);
    run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Expedition { floor_index: 3 };
    let mut boss_omen = crate::game::boss_omen::BossOmenRunState {
        active: Some(crate::game::boss_omen::ActiveBossOmenState {
            chain_id: crate::game::data::boss_omen_data::BossOmenChainId::new("white_night"),
            boss_abnormality_id: "o-01-45_white_night".to_string(),
            next_step_index: 0,
            completed_steps: vec![],
        }),
        ..Default::default()
    };

    crate::game::boss_omen::apply_boss_omen_to_map(
        &mut map,
        &run_progression,
        &core_empty_skill_fragments(),
        game_data.as_ref(),
        &mut boss_omen,
    );

    let node = map.node(event_node_id).expect("event node exists");
    let omen = node.omen.as_ref().expect("event node should receive omen");
    assert!(omen.present);
    assert_eq!(
        omen.source_kind,
        crate::game::data::boss_omen_data::BossOmenSourceKind::Event
    );
    assert_eq!(omen.hint_id, "white_night_confession");
    assert!(matches!(
        &node.payload,
        MapNodePayload::Event {
            event_id: Some(id)
        } if id.as_str() == "white_night_confession_01"
    ));
    assert!(boss_omen.placed_source.as_ref().is_some_and(|placed| {
        placed.node_id == event_node_id
            && placed.chain_id.as_str() == "white_night"
            && placed
                .event_id
                .as_ref()
                .is_some_and(|id| id.as_str() == "white_night_confession_01")
    }));
}

#[test]
fn active_boss_omen_chain_defers_when_required_source_node_is_missing() {
    let game_data = live_game_data_from_ron();
    let combat_node_id = MapNodeId::new(Uuid::from_u128(0xB0_0201));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new("standard_intro_a"),
        facility_template_version: FACILITY_TEMPLATE_VERSION,
        nodes: vec![MapNode {
            id: combat_node_id,
            slot_id: MapSlotId::new("intake_corridor"),
            kind_id: MapNodeKindId::new("combat_monster"),
            category: MapNodeCategory::Combat,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Encounter { encounter_id: None },
            omen: None,
            combat_difficulty: None,
        }],
        start_node_ids: vec![combat_node_id],
        terminals: vec![],
    };
    let mut run_progression = RunProgression::new(123, GameMode::Expedition, 3);
    run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Expedition { floor_index: 3 };
    let active = crate::game::boss_omen::ActiveBossOmenState {
        chain_id: crate::game::data::boss_omen_data::BossOmenChainId::new("white_night"),
        boss_abnormality_id: "o-01-45_white_night".to_string(),
        next_step_index: 0,
        completed_steps: vec![],
    };
    let mut boss_omen = crate::game::boss_omen::BossOmenRunState {
        active: Some(active.clone()),
        ..Default::default()
    };

    crate::game::boss_omen::apply_boss_omen_to_map(
        &mut map,
        &run_progression,
        &core_empty_skill_fragments(),
        game_data.as_ref(),
        &mut boss_omen,
    );

    assert!(map.nodes.iter().all(|node| node.omen.is_none()));
    assert!(boss_omen.placed_source.is_none());
    assert_eq!(boss_omen.active, Some(active));
}

fn core_empty_skill_fragments() -> crate::game::skill_fragment::SkillFragmentInventory {
    crate::game::skill_fragment::SkillFragmentInventory::default()
}

fn game_data_with_event_definitions(
    base: Arc<GameDataBase>,
    events: Vec<crate::game::data::event_data::EventDefinition>,
) -> Arc<GameDataBase> {
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
        .with_event_data(Arc::new(crate::game::data::event_data::EventDatabase::new(
            events,
        )))
        .with_pve_data(base.pve_data.clone())
        .with_boss_omen_data(base.boss_omen_data.clone())
        .with_run_policy_data(base.run_policy.clone())
        .with_skill_data(base.skill_data.clone())
        .with_buff_data(base.buff_data.clone())
        .with_skill_fragment_data(base.skill_fragment_data.clone())
        .build_arc()
}

fn event_choice_definition(
    event_id: &str,
    choice_id: &str,
    success_action: Option<crate::game::data::event_data::EventSuccessAction>,
) -> crate::game::data::event_data::EventDefinition {
    fn scene(
        id: &str,
        next: crate::game::data::event_data::EventSceneNext,
    ) -> crate::game::data::event_data::EventSceneDefinition {
        crate::game::data::event_data::EventSceneDefinition {
            id: crate::game::data::event_data::EventSceneId::new(id),
            presentation: crate::game::data::event_data::EventScenePresentation {
                panels: vec![crate::game::data::event_data::EventPresentationPanel {
                    image_id: format!("{id}_image"),
                    text_ids: vec![format!("{id}_text")],
                }],
            },
            next,
        }
    }
    crate::game::data::event_data::EventDefinition {
        id: crate::game::data::event_data::EventId::new(event_id),
        entry_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
        scenes: vec![
            scene(
                "choice",
                crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices {
                    choices: vec![crate::game::data::event_data::EventChoiceDefinition {
                        id: crate::game::data::event_data::EventChoiceId::new(choice_id),
                        label_id: "test_choice_label".to_string(),
                        check: crate::game::data::event_data::EventChoiceCheckDefinition {
                            aptitude: crate::game::employee::EventAptitude::Maintenance,
                            dc: 12,
                            required_choice_key: None,
                        },
                        success_action,
                        success_scene_id: crate::game::data::event_data::EventSceneId::new(
                            "success",
                        ),
                        failure_scene_id: crate::game::data::event_data::EventSceneId::new(
                            "failure",
                        ),
                    }],
                },
            ),
            scene(
                "success",
                crate::game::data::event_data::EventSceneNext::End,
            ),
            scene(
                "failure",
                crate::game::data::event_data::EventSceneNext::End,
            ),
        ],
    }
}

fn first_living_employee_uuid(core: &GameCore) -> Uuid {
    core.state
        .roster
        .iter()
        .find(|employee| employee.is_alive())
        .expect("fixture should have a living employee")
        .uuid
}

fn start_live_run_with_archive_starters(core: &mut GameCore, player_id: Uuid) {
    let start = core
        .execute(
            player_id,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
        )
        .expect("live run should open its pre-run briefing");
    let BehaviorResult::RunStartFlowStarted {
        visual_novel: Some(visual_novel),
        ..
    } = start
    else {
        panic!("live run should expose its pre-run briefing")
    };
    core.execute(
        player_id,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: visual_novel.session_id,
            current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                visual_novel.current_scene_id,
            ),
        },
    )
    .expect("live pre-run briefing should close");
    core.execute(
        player_id,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids: vec![
                "maintenance_tech".to_string(),
                "records_clerk".to_string(),
                "field_medic".to_string(),
            ],
            loan_copy_ids: vec![],
        },
    )
    .expect("approved live archive starter set should start the run");
}

fn live_employee_uuid(core: &GameCore, definition_id: &str) -> Uuid {
    core.state
        .roster
        .iter()
        .find(|employee| employee.employee_definition_id == definition_id)
        .unwrap_or_else(|| panic!("live roster should contain {definition_id}"))
        .uuid
}

fn enter_live_archive_admin(core: &mut GameCore) -> MapNodeId {
    core.execute_admin_command(crate::game::world::AdminCommand::AdminEnterEvent {
        event_id: "sealed_archive_recovery_01".to_string(),
    })
    .expect("live sealed archive Event should enter through the production admin path");
    let selected = core
        .get_run_snapshot_json()
        .expect("live archive snapshot should serialize");
    MapNodeId::new(
        Uuid::parse_str(
            selected["selected_event"]["node_id"]
                .as_str()
                .expect("live archive node id should serialize as a UUID"),
        )
        .expect("live archive node id should be a UUID"),
    )
}

fn live_archive_projection(
    core: &GameCore,
    node_id: MapNodeId,
) -> crate::game::behavior::EventSceneSnapshotDto {
    let session = core
        .state
        .run
        .as_ref()
        .expect("live run")
        .event_sessions
        .get(&node_id)
        .expect("live archive session");
    core.event_scene_snapshot(session)
        .expect("live archive projection should be valid")
}

fn force_single_event_node_run(core: &mut GameCore, event_id: &str) -> MapNodeId {
    let generated_node_id = force_first_available_node(
        core,
        MapNodeCategory::Event,
        "event_story",
        MapNodePayload::Event {
            event_id: Some(crate::game::data::event_data::EventId::new(event_id)),
        },
    );
    let stable_node_id = MapNodeId::new(Uuid::from_u128(0xE1_0002));
    let run = core.state.run.as_mut().expect("run");
    run.map.node_mut(generated_node_id).expect("event node").id = stable_node_id;
    run.map_progression.available_node_ids = vec![stable_node_id];
    stable_node_id
}

fn enter_single_scene_event(
    core: &mut GameCore,
    player_id: Uuid,
    event_node_id: MapNodeId,
) -> crate::game::data::event_data::EventId {
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
    let event = match entered {
        BehaviorResult::EventState { event, .. } => event,
        other => panic!("expected event state, got {other:?}"),
    };
    assert_eq!(event.current_scene_id.as_str(), "choice");
    assert_persisted_active_event_session_equal(core, event_node_id);
    event.event_id
}

#[test]
fn event_entry_without_run_rejects_before_creating_active_session() {
    let event_id = crate::game::data::event_data::EventId::new("no_run_event");
    let game_data = game_data_with_event_definitions(
        game_data_with_pve_encounters(),
        vec![event_choice_definition("no_run_event", "choice", None)],
    );
    let mut core = GameCore::new(game_data, 123);
    let node_id = MapNodeId::new(Uuid::from_u128(0xE1_00FF));

    let error = core.enter_event_node(node_id, event_id).unwrap_err();

    assert!(matches!(error, GameError::MissingResource("RunState")));
    assert!(core.state.run.is_none());
    assert!(core.state.active_node_content.is_none());
    assert!(matches!(core.get_state(), GameState::NotStarted));
}

#[test]
fn event_command_result_and_root_snapshot_share_one_inner_event_dto() {
    let event_id = "shared_event_envelope";
    let game_data = game_data_with_event_definitions(
        game_data_with_pve_encounters(),
        vec![event_choice_definition(event_id, "choice", None)],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let event_node_id = force_single_event_node_run(&mut core, event_id);

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
    let BehaviorResult::EventState { node_id, event, .. } = entered else {
        panic!("expected EventState command result");
    };

    let snapshot = core.get_run_snapshot_json().expect("root snapshot");
    assert_eq!(snapshot["selected_event"]["type"], "event");
    assert_eq!(snapshot["selected_event"]["node_id"], node_id.0.to_string());
    assert_eq!(
        snapshot["selected_event"]["event"],
        serde_json::to_value(event).expect("event DTO serializes")
    );
}

#[test]
fn event_snapshot_serializes_typed_panels_choices_candidates_and_capability() {
    let event_id = "typed_event_fixture";
    let game_data = game_data_with_event_definitions(
        game_data_with_pve_encounters(),
        vec![event_choice_definition(event_id, "begin", None)],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let event_node_id = force_single_event_node_run(&mut core, event_id);

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
    let BehaviorResult::EventState { event, .. } = entered else {
        panic!("expected typed EventState");
    };
    let value = serde_json::to_value(event).expect("event serializes");
    assert_eq!(
        value["presentation"]["panels"][0]["image_id"],
        "choice_image"
    );
    assert_eq!(
        value["presentation"]["panels"][0]["text_ids"][0],
        "choice_text"
    );
    assert_eq!(value["interaction"]["type"], "employee_check_choices");
    assert!(value["interaction"].get("required_roll").is_none());
    assert!(value["interaction"].get("employee_candidates").is_none());
    assert_eq!(value["interaction"]["choices"][0]["choice_id"], "begin");
    assert_eq!(
        value["interaction"]["choices"][0]["aptitude"],
        "Maintenance"
    );
    assert_eq!(value["interaction"]["choices"][0]["dc"], 12);
    assert!(value["interaction"]["choices"][0]
        .get("required_choice_key")
        .is_none());
    assert_eq!(
        value["interaction"]["choices"][0]["eligibility"]["enabled"],
        true
    );
    assert!(value["interaction"]["choices"][0]["employee_candidates"]
        .as_array()
        .is_some_and(|candidates| !candidates.is_empty()));
    assert_eq!(
        value["interaction"]["choices"][0]["employee_candidates"][0]["employee_modifier"],
        3
    );
    assert_eq!(
        value["interaction"]["choices"][0]["employee_candidates"][0]["success_chance_percent"],
        60
    );
    let mut legacy = value.clone();
    legacy["interaction"]["required_roll"] = serde_json::json!(12);
    assert!(
        serde_json::from_value::<crate::game::behavior::EventSceneSnapshotDto>(legacy).is_err()
    );
    assert!(core.get_action_capabilities().iter().any(|capability| {
        capability.action == ActionKind::ResolveEventChoiceCheck && capability.enabled
    }));
}

#[test]
fn event_check_persists_one_result_and_commits_success_grant_at_most_once() {
    let event_consumable = consumable_meta(
        0xE770,
        "event_grant_tonic",
        ConsumableTier::Common,
        ConsumableEffect::TraumaMitigation { percent: 10 },
    );
    let mut event = event_choice_definition(
        "event_grant_once",
        "take",
        Some(crate::game::data::event_data::EventSuccessAction::Grant {
            effects: vec![
                RewardEffect::GrantEnkephalin { amount: 7 },
                RewardEffect::GrantConsumable {
                    consumable_id: event_consumable.id.clone(),
                    quantity: 3,
                },
            ],
        }),
    );
    let crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices { choices } =
        &mut event.scenes[0].next
    else {
        unreachable!()
    };
    choices[0].check.dc = 8;
    let game_data = game_data_with_event_definitions(
        game_data_with_pve_and_consumables(vec![event_consumable]),
        vec![event],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    core.set_enkephalin(5);
    let event_node_id = force_single_event_node_run(&mut core, "event_grant_once");
    let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);
    let employee_uuid = first_living_employee_uuid(&core);
    let skill_fragments_before = serde_json::to_value(&core.state.skill_fragments).unwrap();
    let roster_before = serde_json::to_value(&core.state.roster).unwrap();
    let uuid_manager_before = serde_json::to_value(&core.state.uuid_manager).unwrap();
    let resolved = core
        .execute(
            player_id,
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id: event_node_id,
                event_id: event_id.clone(),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
                choice_id: crate::game::data::event_data::EventChoiceId::new("take"),
                employee_uuid,
            },
        )
        .expect("employee check should resolve");
    let BehaviorResult::EventState { event, .. } = resolved else {
        panic!("check should return an EventState result");
    };
    let resolved_scene_id = event.current_scene_id.clone();
    let committed = event.committed_check.expect("check result should persist");
    assert!(matches!(
        committed.outcome,
        crate::game::data::event_data::EventCheckOutcome::Success
            | crate::game::data::event_data::EventCheckOutcome::CriticalSuccess
    ));
    assert_eq!(committed.employee_uuid, employee_uuid);
    assert_eq!(committed.source_scene_id.as_str(), "choice");
    assert_eq!(committed.aptitude, EventAptitude::Maintenance);
    assert_eq!(committed.dc, 8);
    assert!((1..=20).contains(&committed.raw_roll));
    assert_eq!(committed.modifier_total, 3);
    assert_eq!(committed.total_roll, committed.raw_roll + 3);
    assert!(matches!(
        committed.modifier_components.as_slice(),
        [
            crate::game::resources::EventCheckModifierComponent::EmployeeAptitude {
                aptitude: EventAptitude::Maintenance,
                amount: 3,
            }
        ]
    ));
    let committed_json = serde_json::to_value(&committed).unwrap();
    assert!(committed_json.get("required_roll").is_none());
    assert!(committed_json.get("equipment_modifier").is_none());
    assert_eq!(core.get_enkephalin(), 12);
    let committed_state = core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .unwrap()
        .committed_check
        .as_ref()
        .unwrap()
        .clone();
    assert_persisted_active_event_session_equal(&core, event_node_id);
    assert_eq!(committed_state.aptitude, EventAptitude::Maintenance);
    assert_eq!(committed_state.dc, 8);
    assert!(matches!(
        committed_state.modifier_components.as_slice(),
        [
            crate::game::resources::EventCheckModifierComponent::EmployeeAptitude {
                aptitude: EventAptitude::Maintenance,
                amount: 3,
            }
        ]
    ));
    assert_eq!(committed_state.modifier_total, 3);
    assert_eq!(committed_state.total_roll, committed_state.raw_roll + 3);
    assert_eq!(
        core.state
            .inventory
            .consumables
            .get_by_definition_id("event_grant_tonic")
            .unwrap()
            .quantity,
        3
    );
    assert_eq!(
        serde_json::to_value(&core.state.skill_fragments).unwrap(),
        skill_fragments_before
    );
    assert_eq!(
        serde_json::to_value(&core.state.roster).unwrap(),
        roster_before
    );
    assert_eq!(
        serde_json::to_value(&core.state.uuid_manager).unwrap(),
        uuid_manager_before
    );

    core.save_run_checkpoint_for_test().unwrap();
    core.handle_advance_event_scene(event_node_id, event_id.clone(), resolved_scene_id)
        .unwrap();
    core.set_enkephalin(999);
    core.handle_load_run_checkpoint().unwrap();
    assert_eq!(core.get_enkephalin(), 12);
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .event_sessions
            .get(&event_node_id)
            .unwrap()
            .committed_check
            .as_ref()
            .unwrap(),
        &committed_state
    );
    core.enter_event_node(event_node_id, event_id.clone())
        .unwrap();
    assert_persisted_active_event_session_equal(&core, event_node_id);

    let repeated = core
        .handle_resolve_event_choice_check(
            event_node_id,
            event_id,
            crate::game::data::event_data::EventSceneId::new("choice"),
            crate::game::data::event_data::EventChoiceId::new("take"),
            employee_uuid,
        )
        .expect("an exact duplicate should return the committed result");
    let BehaviorResult::EventState { event, .. } = repeated else {
        panic!("duplicate should return current EventState");
    };
    assert_eq!(
        event.committed_check.expect("committed result").raw_roll,
        committed.raw_roll
    );
    assert_eq!(core.get_enkephalin(), 12);
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .event_sessions
            .get(&event_node_id)
            .unwrap()
            .committed_check
            .as_ref()
            .unwrap(),
        &committed_state
    );
    assert_eq!(
        core.state
            .inventory
            .consumables
            .get_by_definition_id("event_grant_tonic")
            .unwrap()
            .quantity,
        3
    );
    assert_persisted_active_event_session_equal(&core, event_node_id);
}

#[test]
fn event_grant_late_failure_preserves_all_five_roots_and_both_session_copies() {
    let mut event = event_choice_definition(
        "event_grant_late_failure",
        "take",
        Some(crate::game::data::event_data::EventSuccessAction::Grant {
            effects: vec![
                RewardEffect::GrantEnkephalin { amount: 7 },
                RewardEffect::GrantEquipment {
                    equipment_id: "missing_event_grant_equipment".to_string(),
                },
            ],
        }),
    );
    let crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices { choices } =
        &mut event.scenes[0].next
    else {
        unreachable!()
    };
    choices[0].check.dc = 8;
    let game_data = game_data_with_event_definitions(game_data_with_pve_encounters(), vec![event]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let event_node_id = force_single_event_node_run(&mut core, "event_grant_late_failure");
    let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);
    let employee_uuid = first_living_employee_uuid(&core);
    let roots_before = serialized_grant_roots(&core);
    let persisted_sessions_before =
        serde_json::to_value(&core.state.run.as_ref().unwrap().event_sessions).unwrap();
    let active_session_before = serde_json::to_value(
        core.state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_event()
            .unwrap(),
    )
    .unwrap();

    let error = core
        .handle_resolve_event_choice_check(
            event_node_id,
            event_id,
            crate::game::data::event_data::EventSceneId::new("choice"),
            crate::game::data::event_data::EventChoiceId::new("take"),
            employee_uuid,
        )
        .unwrap_err();

    assert!(matches!(error, GameError::InvalidStaticData(_)));
    assert_eq!(serialized_grant_roots(&core), roots_before);
    assert_eq!(
        serde_json::to_value(&core.state.run.as_ref().unwrap().event_sessions).unwrap(),
        persisted_sessions_before
    );
    assert_eq!(
        serde_json::to_value(
            core.state
                .active_node_content
                .as_ref()
                .unwrap()
                .as_event()
                .unwrap(),
        )
        .unwrap(),
        active_session_before
    );
}

#[test]
fn event_check_failure_is_persisted_gameplay_state_and_final_advance_completes() {
    let mut event = event_choice_definition("failure_event", "attempt", None);
    let crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices { choices } =
        &mut event.scenes[0].next
    else {
        unreachable!()
    };
    choices[0].check.dc = 20;
    let game_data = game_data_with_event_definitions(game_data_with_pve_encounters(), vec![event]);

    for seed in 1..=100 {
        let mut core = GameCore::new(game_data.clone(), seed);
        let player_id = Uuid::from_u128(1);
        start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
        let event_node_id = force_single_event_node_run(&mut core, "failure_event");
        let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);
        let employee_uuid = first_living_employee_uuid(&core);
        let before = core.state.roster.get(&employee_uuid).unwrap().clone();
        let resolved = core
            .execute(
                player_id,
                PlayerBehavior::ResolveEventChoiceCheck {
                    node_id: event_node_id,
                    event_id: event_id.clone(),
                    current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
                    choice_id: crate::game::data::event_data::EventChoiceId::new("attempt"),
                    employee_uuid,
                },
            )
            .expect("employee check should resolve");
        let BehaviorResult::EventState { event, .. } = resolved else {
            panic!("check should return EventState");
        };
        let committed = event.committed_check.expect("committed check");
        if matches!(
            committed.outcome,
            crate::game::data::event_data::EventCheckOutcome::Success
                | crate::game::data::event_data::EventCheckOutcome::CriticalSuccess
        ) {
            continue;
        }

        assert_eq!(event.current_scene_id.as_str(), "failure");
        assert_eq!(committed.employee_uuid, employee_uuid);
        let after = core.state.roster.get(&employee_uuid).unwrap();
        assert_eq!(after.condition, before.condition);
        assert_eq!(after.experience, before.experience);

        let completed = core
            .execute(
                player_id,
                PlayerBehavior::AdvanceEventScene {
                    node_id: event_node_id,
                    event_id,
                    current_scene_id: crate::game::data::event_data::EventSceneId::new("failure"),
                },
            )
            .expect("failure result panels should complete on final advance");
        assert!(matches!(completed, BehaviorResult::NodeCompleted { .. }));
        return;
    }

    panic!("fixture seeds should include at least one non-natural-20 roll");
}

#[test]
fn live_white_night_bootstrap_requires_no_global_awakened_fragment() {
    let game_data = live_game_data_from_ron();
    let skill_fragments = crate::game::skill_fragment::SkillFragmentInventory::new();

    let event_node_id = MapNodeId::new(Uuid::from_u128(0xB0_0301));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new("standard_intro_a"),
        facility_template_version: FACILITY_TEMPLATE_VERSION,
        nodes: vec![MapNode {
            id: event_node_id,
            slot_id: MapSlotId::new("intake_corridor"),
            kind_id: MapNodeKindId::new("event_story"),
            category: MapNodeCategory::Event,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Event {
                event_id: Some(crate::game::data::event_data::EventId::new(
                    "white_night_confession_01",
                )),
            },
            omen: None,
            combat_difficulty: None,
        }],
        start_node_ids: vec![event_node_id],
        terminals: vec![],
    };
    let mut run_progression = RunProgression::new(123, GameMode::Expedition, 3);
    run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Expedition { floor_index: 3 };
    let mut boss_omen = crate::game::boss_omen::BossOmenRunState::default();

    crate::game::boss_omen::apply_boss_omen_to_map(
        &mut map,
        &run_progression,
        &skill_fragments,
        game_data.as_ref(),
        &mut boss_omen,
    );

    let node = map.node(event_node_id).expect("event node exists");
    assert!(
        node.omen.is_some(),
        "WhiteNight required_fragment: None must not inherit a global awakened-fragment gate"
    );
    assert!(boss_omen.provisional.is_some());
    assert!(boss_omen.placed_source.is_some());
}

#[test]
fn event_check_rejects_stale_scene_choice_and_employee_identities_without_mutation() {
    let event = event_choice_definition("identity_event", "check", None);
    let game_data = game_data_with_event_definitions(game_data_with_pve_encounters(), vec![event]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let event_node_id = force_single_event_node_run(&mut core, "identity_event");
    let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);
    let employee_uuid = first_living_employee_uuid(&core);

    for behavior in [
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id: event_node_id,
            event_id: event_id.clone(),
            current_scene_id: crate::game::data::event_data::EventSceneId::new("stale"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("check"),
            employee_uuid,
        },
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id: event_node_id,
            event_id: event_id.clone(),
            current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("stale"),
            employee_uuid,
        },
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id: event_node_id,
            event_id: event_id.clone(),
            current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("check"),
            employee_uuid: Uuid::from_u128(0xDEAD),
        },
    ] {
        assert!(matches!(
            core.execute(player_id, behavior),
            Err(GameError::ActionUnavailable {
                action: ActionKind::ResolveEventChoiceCheck,
                reason: ActionDisabledReasonCode::NoEligiblePayload,
            })
        ));
    }
    let session = core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .unwrap();
    assert!(session.committed_check.is_none());
}

#[test]
fn event_resolve_revalidates_equipped_key_and_commits_shared_evaluator_provenance() {
    let mut tool = equipment_meta(0xA11CE, "event_archive_tool", EquipmentType::Accessory);
    tool.event_check_modifiers = vec![
        crate::game::data::equipment_data::EventCheckModifierDefinition {
            aptitude: EventAptitude::Analysis,
            amount: 2,
        },
    ];
    tool.event_choice_keys = vec![crate::game::data::event_data::EventChoiceKeyId::new(
        "sealed_archive_access",
    )];
    let mut event = event_choice_definition("keyed_event", "inspect", None);
    let crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices { choices } =
        &mut event.scenes[0].next
    else {
        unreachable!()
    };
    choices[0].check.aptitude = EventAptitude::Analysis;
    choices[0].check.dc = 16;
    choices[0].check.required_choice_key = Some(
        crate::game::data::event_data::EventChoiceKeyId::new("sealed_archive_access"),
    );
    let choice = choices[0].clone();
    let game_data = game_data_with_event_definitions(
        game_data_with_display_items(vec![tool.clone()]),
        vec![event],
    );
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let event_node_id = force_single_event_node_run(&mut core, "keyed_event");
    let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);
    let employee_uuid = first_living_employee_uuid(&core);

    let missing_key = core
        .evaluate_event_choice_for_employee(&choice, employee_uuid)
        .unwrap();
    assert_eq!(
        missing_key.disabled_reason,
        Some(ActionDisabledReasonCode::MissingEventChoiceKey)
    );

    let owned_uuid = Uuid::from_u128(0xA11CE_01);
    let tool_base_uuid = tool.uuid;
    let tool_type = tool.equipment_type;
    core.inventory_mut()
        .unwrap()
        .equipments
        .add_item(OwnedEquipment::new(owned_uuid, Arc::new(tool)))
        .unwrap();
    core.inventory_mut()
        .unwrap()
        .equipments
        .get_item_mut(&owned_uuid)
        .unwrap()
        .equipped_to = Some(employee_uuid);
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .loadout
        .item_slot
        .equip(
            EquippedRef {
                instance_uuid: owned_uuid,
                base_uuid: tool_base_uuid,
                equipment_type: tool_type,
            },
            true,
        )
        .unwrap();
    let projected = core
        .evaluate_event_choice_for_employee(&choice, employee_uuid)
        .unwrap();
    assert!(projected.disabled_reason.is_none());
    assert_eq!(projected.employee_modifier, 0);
    assert_eq!(projected.equipment_modifier, 2);
    assert_eq!(projected.success_chance_percent, 35);

    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .loadout
        .item_slot
        .remove_by_instance(owned_uuid)
        .unwrap();
    core.inventory_mut()
        .unwrap()
        .equipments
        .get_item_mut(&owned_uuid)
        .unwrap()
        .equipped_to = None;
    assert!(matches!(
        core.execute(
            player_id,
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id: event_node_id,
                event_id: event_id.clone(),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
                choice_id: crate::game::data::event_data::EventChoiceId::new("inspect"),
                employee_uuid,
            },
        ),
        Err(GameError::ActionUnavailable {
            action: ActionKind::ResolveEventChoiceCheck,
            reason: ActionDisabledReasonCode::NoEligiblePayload,
        })
    ));
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .unwrap()
        .committed_check
        .is_none());

    core.inventory_mut()
        .unwrap()
        .equipments
        .get_item_mut(&owned_uuid)
        .unwrap()
        .equipped_to = Some(employee_uuid);
    core.roster_mut()
        .unwrap()
        .get_mut(&employee_uuid)
        .unwrap()
        .loadout
        .item_slot
        .equip(
            EquippedRef {
                instance_uuid: owned_uuid,
                base_uuid: tool_base_uuid,
                equipment_type: tool_type,
            },
            true,
        )
        .unwrap();
    core.execute(
        player_id,
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id: event_node_id,
            event_id,
            current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("inspect"),
            employee_uuid,
        },
    )
    .expect("re-equipped provider should allow resolve");

    let committed = core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .unwrap()
        .committed_check
        .as_ref()
        .unwrap();
    assert_eq!(committed.modifier_total, projected.modifier_total);
    assert!(matches!(
        committed.modifier_components.as_slice(),
        [crate::game::resources::EventCheckModifierComponent::Equipment {
            equipment_instance_uuid,
            equipment_definition_id,
            aptitude: EventAptitude::Analysis,
            amount: 2,
        }] if *equipment_instance_uuid == owned_uuid
            && equipment_definition_id == "event_archive_tool"
    ));
    assert_eq!(
        committed
            .choice_key_evidence
            .as_ref()
            .unwrap()
            .equipment_instance_uuid,
        owned_uuid
    );
}

#[test]
fn precheck_checkpoint_reuses_raw_roll_for_different_valid_choice_and_employee() {
    let mut event = event_choice_definition("checkpoint_roll_event", "maintain", None);
    let crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices { choices } =
        &mut event.scenes[0].next
    else {
        unreachable!()
    };
    choices[0].check.aptitude = EventAptitude::Maintenance;
    choices[0].check.dc = 8;
    let mut analysis_choice = choices[0].clone();
    analysis_choice.id = crate::game::data::event_data::EventChoiceId::new("analyze");
    analysis_choice.label_id = "analyze_label".to_string();
    analysis_choice.check.aptitude = EventAptitude::Analysis;
    analysis_choice.check.dc = 20;
    choices.push(analysis_choice);

    let game_data = game_data_with_event_definitions(game_data_with_pve_encounters(), vec![event]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let employee_ids = core.roster().unwrap().combat_eligible_employee_ids();
    let maintenance_employee = employee_ids[0];
    let analysis_employee = employee_ids[1];
    core.roster_mut()
        .unwrap()
        .get_mut(&analysis_employee)
        .unwrap()
        .profile
        .event_aptitude = EventAptitude::Analysis;
    let event_node_id = force_single_event_node_run(&mut core, "checkpoint_roll_event");
    core.save_run_checkpoint_for_test().unwrap();
    let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);

    assert!(core
        .execute(
            player_id,
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id: event_node_id,
                event_id: event_id.clone(),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
                choice_id: crate::game::data::event_data::EventChoiceId::new("missing_choice"),
                employee_uuid: maintenance_employee,
            },
        )
        .is_err());
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .unwrap()
        .committed_check
        .is_none());

    core.execute(
        player_id,
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id: event_node_id,
            event_id: event_id.clone(),
            current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("maintain"),
            employee_uuid: maintenance_employee,
        },
    )
    .unwrap();
    let first = core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .unwrap()
        .committed_check
        .clone()
        .unwrap();
    let first_result_scene = core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .unwrap()
        .current_scene_id
        .clone();

    core.handle_advance_event_scene(event_node_id, event_id.clone(), first_result_scene)
        .unwrap();
    core.handle_load_run_checkpoint().unwrap();
    let restored_event_id = enter_single_scene_event(&mut core, player_id, event_node_id);
    assert_eq!(restored_event_id, event_id);
    core.execute(
        player_id,
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id: event_node_id,
            event_id,
            current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("analyze"),
            employee_uuid: analysis_employee,
        },
    )
    .unwrap();
    let second = core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&event_node_id)
        .unwrap()
        .committed_check
        .as_ref()
        .unwrap();

    assert_eq!(first.raw_roll, second.raw_roll);
    assert_ne!(first.choice_id, second.choice_id);
    assert_ne!(first.employee_uuid, second.employee_uuid);
    assert_eq!(first.dc, 8);
    assert_eq!(second.dc, 20);
}

#[test]
fn event_check_candidates_use_living_state_only_and_disable_when_none_are_living() {
    let event = event_choice_definition("eligibility_event", "check", None);
    let game_data = game_data_with_event_definitions(game_data_with_pve_encounters(), vec![event]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let event_node_id = force_single_event_node_run(&mut core, "eligibility_event");
    enter_single_scene_event(&mut core, player_id, event_node_id);
    for employee in core.state.roster.iter_mut() {
        employee.condition.physical_damage = MAX_PHYSICAL_DAMAGE;
    }
    let snapshot = core
        .event_scene_snapshot(
            core.state
                .active_node_content
                .as_ref()
                .unwrap()
                .as_event()
                .unwrap(),
        )
        .unwrap();
    let crate::game::behavior::EventInteractionDto::EmployeeCheckChoices { choices } =
        snapshot.interaction
    else {
        panic!("expected employee check interaction");
    };
    let employee_candidates = &choices[0].employee_candidates;
    assert!(employee_candidates
        .iter()
        .all(|candidate| !candidate.eligibility.enabled));
    assert!(choices.iter().all(|choice| !choice.eligibility.enabled));
    assert!(core.get_action_capabilities().iter().any(|capability| {
        capability.action == ActionKind::ResolveEventChoiceCheck
            && !capability.enabled
            && capability.disabled_reason_code == Some(ActionDisabledReasonCode::NoEligiblePayload)
    }));
}

#[test]
fn event_success_action_starts_combat_and_retreat_preserves_committed_check() {
    let mut event = event_choice_definition(
        "combat_choice_event",
        "fight",
        Some(
            crate::game::data::event_data::EventSuccessAction::StartCombat {
                encounter_id: "defense_encounter".to_string(),
                primary_abnormality_id: Some("defense_risk_abno".to_string()),
            },
        ),
    );
    let crate::game::data::event_data::EventSceneNext::EmployeeCheckChoices { choices } =
        &mut event.scenes[0].next
    else {
        unreachable!()
    };
    choices[0].check.dc = 8;
    let game_data = game_data_with_event_definitions(game_data_with_pve_encounters(), vec![event]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let event_node_id = force_single_event_node_run(&mut core, "combat_choice_event");
    let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);
    let employee_uuid = first_living_employee_uuid(&core);

    let result = core
        .execute(
            player_id,
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id: event_node_id,
                event_id: event_id.clone(),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
                choice_id: crate::game::data::event_data::EventChoiceId::new("fight"),
                employee_uuid,
            },
        )
        .expect("combat choice should start battle");
    assert!(matches!(result, BehaviorResult::BattleAdvanced { .. }));
    let session = core
        .state
        .run
        .as_ref()
        .expect("run")
        .event_sessions
        .get(&event_node_id)
        .expect("event session should persist during battle");
    assert_eq!(
        session
            .committed_check
            .as_ref()
            .map(|check| check.choice_id.as_str()),
        Some("fight")
    );
    assert!(session.started_combat.is_some());
    let committed_before_retreat = session.committed_check.clone().unwrap();

    let retreat =
        execute_current_battle_mutation(&mut core, player_id, PlayerBehavior::RetreatBattle)
            .expect("retreat should return to node confirm");
    assert!(matches!(retreat, BehaviorResult::NodePreview { .. }));
    let session = core
        .state
        .run
        .as_ref()
        .expect("run")
        .event_sessions
        .get(&event_node_id)
        .expect("event session should survive retreat");
    assert_eq!(
        session
            .committed_check
            .as_ref()
            .map(|check| check.choice_id.as_str()),
        Some("fight")
    );
    assert!(session.started_combat.is_some());
    assert_eq!(
        session.committed_check.as_ref().unwrap(),
        &committed_before_retreat
    );

    let resumed = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("re-entering committed event should resume combat");
    assert!(matches!(resumed, BehaviorResult::BattleAdvanced { .. }));
}

#[test]
fn live_sealed_archive_projection_exposes_meaningful_candidates_and_rejects_missing_key() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD1CE);
    let player_id = Uuid::from_u128(0xA11CE);
    start_live_run_with_archive_starters(&mut core, player_id);
    let node_id = enter_live_archive_admin(&mut core);
    let technician = live_employee_uuid(&core, "maintenance_tech");
    let analyst = live_employee_uuid(&core, "records_clerk");
    let medic = live_employee_uuid(&core, "field_medic");

    let projection = live_archive_projection(&core, node_id);
    assert_eq!(projection.event_id.as_str(), "sealed_archive_recovery_01");
    assert_eq!(projection.current_scene_id.as_str(), "archive_check");
    let crate::game::behavior::EventInteractionDto::EmployeeCheckChoices { choices } =
        projection.interaction
    else {
        panic!("live archive entry must be the checked scene")
    };
    assert_eq!(choices.len(), 2);
    let decode = choices
        .iter()
        .find(|choice| choice.choice_id.as_str() == "decode_archive_seal")
        .expect("decode choice");
    assert_eq!(decode.aptitude, EventAptitude::Analysis);
    assert_eq!(decode.dc, 16);
    assert_eq!(
        decode.required_choice_key.as_ref().map(|key| key.as_str()),
        Some("sealed_archive_access")
    );
    let technician_decode = decode
        .employee_candidates
        .iter()
        .find(|candidate| candidate.employee_uuid == technician)
        .expect("technician decode evaluation");
    assert!(technician_decode.eligibility.enabled);
    assert_eq!(technician_decode.employee_modifier, 0);
    assert_eq!(technician_decode.equipment_modifier, 2);
    assert_eq!(technician_decode.modifier_total, 2);
    assert_eq!(technician_decode.success_chance_percent, 35);
    assert_eq!(
        technician_decode
            .choice_key_evidence
            .as_ref()
            .map(|evidence| evidence.equipment_definition_id.as_str()),
        Some("archive_scanner")
    );
    let analyst_decode = decode
        .employee_candidates
        .iter()
        .find(|candidate| candidate.employee_uuid == analyst)
        .expect("analyst decode evaluation");
    assert!(!analyst_decode.eligibility.enabled);
    assert_eq!(analyst_decode.employee_modifier, 3);
    assert_eq!(analyst_decode.equipment_modifier, 0);
    assert_eq!(analyst_decode.success_chance_percent, 40);
    assert_eq!(
        analyst_decode.eligibility.disabled_reason_code,
        Some(ActionDisabledReasonCode::MissingEventChoiceKey)
    );

    let stabilize = choices
        .iter()
        .find(|choice| choice.choice_id.as_str() == "stabilize_archive_power")
        .expect("stabilize choice");
    assert_eq!(stabilize.aptitude, EventAptitude::Maintenance);
    assert_eq!(stabilize.dc, 12);
    assert!(stabilize.required_choice_key.is_none());
    let technician_stabilize = stabilize
        .employee_candidates
        .iter()
        .find(|candidate| candidate.employee_uuid == technician)
        .expect("technician stabilize evaluation");
    assert_eq!(technician_stabilize.modifier_total, 3);
    assert_eq!(technician_stabilize.success_chance_percent, 60);
    let medic_stabilize = stabilize
        .employee_candidates
        .iter()
        .find(|candidate| candidate.employee_uuid == medic)
        .expect("medic stabilize evaluation");
    assert_eq!(medic_stabilize.modifier_total, 0);
    assert_eq!(medic_stabilize.success_chance_percent, 45);

    let enkephalin_before = core.get_enkephalin();
    assert!(matches!(
        core.execute(
            player_id,
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id,
                event_id: crate::game::data::event_data::EventId::new(
                    "sealed_archive_recovery_01",
                ),
                current_scene_id: crate::game::data::event_data::EventSceneId::new(
                    "archive_check",
                ),
                choice_id: crate::game::data::event_data::EventChoiceId::new(
                    "decode_archive_seal",
                ),
                employee_uuid: analyst,
            },
        ),
        Err(GameError::ActionUnavailable {
            action: ActionKind::ResolveEventChoiceCheck,
            reason: ActionDisabledReasonCode::MissingEventChoiceKey,
        })
    ));
    assert_eq!(core.get_enkephalin(), enkephalin_before);
    assert!(core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&node_id)
        .unwrap()
        .committed_check
        .is_none());

    let resolved = core
        .execute(
            player_id,
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id,
                event_id: crate::game::data::event_data::EventId::new("sealed_archive_recovery_01"),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("archive_check"),
                choice_id: crate::game::data::event_data::EventChoiceId::new(
                    "stabilize_archive_power",
                ),
                employee_uuid: technician,
            },
        )
        .expect("technician should resolve the live maintenance choice");
    let BehaviorResult::EventState { event, .. } = resolved else {
        panic!("live archive check should return EventState")
    };
    let committed = event.committed_check.expect("live check should commit");
    assert_eq!(committed.raw_roll, 11);
    assert_eq!(committed.modifier_total, 3);
    assert_eq!(committed.total_roll, 14);
    assert_eq!(
        committed.outcome,
        crate::game::data::event_data::EventCheckOutcome::Success
    );
    assert_eq!(event.current_scene_id.as_str(), "archive_power_stabilized");
    assert_eq!(core.get_enkephalin(), enkephalin_before + 30);
}

#[test]
fn live_sealed_archive_executes_success_failure_and_both_critical_branches() {
    use crate::game::data::event_data::EventCheckOutcome;

    let game_data = live_game_data_from_ron();
    let mut seen = [false; 4];
    for seed in 0..512 {
        let mut core = GameCore::new(game_data.clone(), seed);
        let player_id = Uuid::from_u128(0xA11CE);
        start_live_run_with_archive_starters(&mut core, player_id);
        let node_id = enter_live_archive_admin(&mut core);
        let technician = live_employee_uuid(&core, "maintenance_tech");
        let enkephalin_before = core.get_enkephalin();
        let resolved = core
            .execute(
                player_id,
                PlayerBehavior::ResolveEventChoiceCheck {
                    node_id,
                    event_id: crate::game::data::event_data::EventId::new(
                        "sealed_archive_recovery_01",
                    ),
                    current_scene_id: crate::game::data::event_data::EventSceneId::new(
                        "archive_check",
                    ),
                    choice_id: crate::game::data::event_data::EventChoiceId::new(
                        "decode_archive_seal",
                    ),
                    employee_uuid: technician,
                },
            )
            .expect("equipped scanner should resolve every live decode branch");
        let BehaviorResult::EventState { event, .. } = resolved else {
            panic!("live decode should return EventState")
        };
        let committed = event.committed_check.expect("live decode should commit");
        let expected = match committed.raw_roll {
            1 => EventCheckOutcome::CriticalFailure,
            20 => EventCheckOutcome::CriticalSuccess,
            14..=19 => EventCheckOutcome::Success,
            _ => EventCheckOutcome::Failure,
        };
        assert_eq!(committed.modifier_total, 2);
        assert_eq!(committed.total_roll, committed.raw_roll + 2);
        assert_eq!(committed.outcome, expected);
        assert_eq!(
            committed
                .choice_key_evidence
                .as_ref()
                .map(|evidence| evidence.equipment_definition_id.as_str()),
            Some("archive_scanner")
        );
        if matches!(
            expected,
            EventCheckOutcome::Success | EventCheckOutcome::CriticalSuccess
        ) {
            assert_eq!(event.current_scene_id.as_str(), "archive_decoded");
            assert_eq!(core.get_enkephalin(), enkephalin_before + 80);
        } else {
            assert_eq!(event.current_scene_id.as_str(), "archive_seal_rejected");
            assert_eq!(core.get_enkephalin(), enkephalin_before);
        }
        let outcome_index = match expected {
            EventCheckOutcome::CriticalFailure => 0,
            EventCheckOutcome::Failure => 1,
            EventCheckOutcome::Success => 2,
            EventCheckOutcome::CriticalSuccess => 3,
        };
        seen[outcome_index] = true;
        if seen.into_iter().all(|covered| covered) {
            return;
        }
    }

    panic!("live deterministic seed sample did not cover all four approved outcomes: {seen:?}");
}

#[test]
fn live_sealed_archive_duplicate_and_precheck_checkpoint_reuse_one_raw_roll() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xD1CE);
    let player_id = Uuid::from_u128(0xA11CE);
    start_live_run_with_archive_starters(&mut core, player_id);
    core.save_run_checkpoint_for_test()
        .expect("pre-check live checkpoint should save from the map");
    let enkephalin_at_checkpoint = core.get_enkephalin();
    let node_id = enter_live_archive_admin(&mut core);
    let technician = live_employee_uuid(&core, "maintenance_tech");
    let event_id = crate::game::data::event_data::EventId::new("sealed_archive_recovery_01");
    let check_scene = crate::game::data::event_data::EventSceneId::new("archive_check");

    core.execute(
        player_id,
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id,
            event_id: event_id.clone(),
            current_scene_id: check_scene.clone(),
            choice_id: crate::game::data::event_data::EventChoiceId::new("stabilize_archive_power"),
            employee_uuid: technician,
        },
    )
    .expect("first live choice should resolve");
    let first = core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&node_id)
        .unwrap()
        .committed_check
        .clone()
        .unwrap();
    assert_eq!(core.get_enkephalin(), enkephalin_at_checkpoint + 30);

    core.execute(
        player_id,
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id,
            event_id: event_id.clone(),
            current_scene_id: check_scene,
            choice_id: crate::game::data::event_data::EventChoiceId::new("stabilize_archive_power"),
            employee_uuid: technician,
        },
    )
    .expect("exact live duplicate should cross preflight and reuse the committed result");
    assert_eq!(core.get_enkephalin(), enkephalin_at_checkpoint + 30);
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .event_sessions
            .get(&node_id)
            .unwrap()
            .committed_check
            .as_ref(),
        Some(&first)
    );
    assert!(core
        .execute(
            player_id,
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id,
                event_id: event_id.clone(),
                current_scene_id: crate::game::data::event_data::EventSceneId::new(
                    "archive_check",
                ),
                choice_id: crate::game::data::event_data::EventChoiceId::new(
                    "decode_archive_seal",
                ),
                employee_uuid: technician,
            },
        )
        .is_err());
    assert_eq!(core.get_enkephalin(), enkephalin_at_checkpoint + 30);
    assert_eq!(
        core.state
            .run
            .as_ref()
            .unwrap()
            .event_sessions
            .get(&node_id)
            .unwrap()
            .committed_check
            .as_ref(),
        Some(&first)
    );

    core.handle_advance_event_scene(
        node_id,
        event_id.clone(),
        crate::game::data::event_data::EventSceneId::new("archive_power_stabilized"),
    )
    .expect("live result scene should complete");
    core.handle_load_run_checkpoint()
        .expect("live pre-check checkpoint should restore");
    assert_eq!(core.get_enkephalin(), enkephalin_at_checkpoint);

    let restored_node_id = enter_live_archive_admin(&mut core);
    assert_eq!(restored_node_id, node_id);
    core.execute(
        player_id,
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id: restored_node_id,
            event_id,
            current_scene_id: crate::game::data::event_data::EventSceneId::new("archive_check"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("decode_archive_seal"),
            employee_uuid: technician,
        },
    )
    .expect("different valid live choice should resolve after restore");
    let second = core
        .state
        .run
        .as_ref()
        .unwrap()
        .event_sessions
        .get(&restored_node_id)
        .unwrap()
        .committed_check
        .as_ref()
        .unwrap();
    assert_eq!(first.raw_roll, second.raw_roll);
    assert_ne!(first.choice_id, second.choice_id);
    assert_eq!(second.modifier_total, 2);
    assert_eq!(core.get_enkephalin(), enkephalin_at_checkpoint);
}

#[test]
fn live_event_panels_advance_through_typed_route_and_complete_node() {
    let mut core = GameCore::new(live_game_data_from_ron(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let event_node_id = force_single_event_node_run(&mut core, "white_night_confession_01");

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
    let event = match entered {
        BehaviorResult::EventState { event, .. } => event,
        other => panic!("expected event state, got {other:?}"),
    };
    assert_eq!(event.current_scene_id.as_str(), "start");
    assert_persisted_active_event_session_equal(&core, event_node_id);

    let advanced = core
        .execute(
            player_id,
            PlayerBehavior::AdvanceEventScene {
                node_id: event_node_id,
                event_id: crate::game::data::event_data::EventId::new("white_night_confession_01"),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("start"),
            },
        )
        .expect("event scene should advance");
    let event = match advanced {
        BehaviorResult::EventState { event, .. } => event,
        other => panic!("expected event state, got {other:?}"),
    };
    assert_eq!(event.current_scene_id.as_str(), "choice");
    assert!(matches!(
        event.interaction,
        crate::game::behavior::EventInteractionDto::Advance
    ));
    assert_persisted_active_event_session_equal(&core, event_node_id);

    let advanced = core
        .execute(
            player_id,
            PlayerBehavior::AdvanceEventScene {
                node_id: event_node_id,
                event_id: crate::game::data::event_data::EventId::new("white_night_confession_01"),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
            },
        )
        .expect("event panels should advance to the result scene");
    let event = match advanced {
        BehaviorResult::EventState { event, .. } => event,
        other => panic!("expected event result scene, got {other:?}"),
    };
    assert_eq!(event.current_scene_id.as_str(), "end");
    assert_persisted_active_event_session_equal(&core, event_node_id);

    let completed = core
        .execute(
            player_id,
            PlayerBehavior::AdvanceEventScene {
                node_id: event_node_id,
                event_id: crate::game::data::event_data::EventId::new("white_night_confession_01"),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("end"),
            },
        )
        .expect("terminal event scene should complete the node");
    assert!(
        matches!(completed, BehaviorResult::NodeCompleted { .. }),
        "expected NodeCompleted after authored event room, got {completed:?}"
    );
    assert!(core
        .state
        .run
        .as_ref()
        .expect("run")
        .event_sessions
        .get(&event_node_id)
        .is_none());
}

#[test]
fn boss_omen_event_step_consumption_forces_authored_capstone() {
    let mut core = GameCore::new(live_game_data_from_ron(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Expedition);
    let event_node_id = force_single_event_node_run(&mut core, "white_night_confession_01");
    {
        let run = core.state.run.as_mut().expect("run");
        run.run_progression.mode_state =
            crate::game::map::RunProgressionModeState::Expedition { floor_index: 3 };
        run.boss_omen.placed_source = Some(crate::game::boss_omen::PlacedBossOmenSourceState {
            node_id: event_node_id,
            chain_id: crate::game::data::boss_omen_data::BossOmenChainId::new("white_night"),
            boss_abnormality_id: "o-01-45_white_night".to_string(),
            step_id: crate::game::data::boss_omen_data::BossOmenStepId::new("confession_01"),
            step_index: 0,
            source_kind: crate::game::data::boss_omen_data::BossOmenSourceKind::Event,
            hint_id: "white_night_confession".to_string(),
            title_id: "white_night_confession_title".to_string(),
            description_id: "white_night_confession_01".to_string(),
            event_id: Some(crate::game::data::event_data::EventId::new(
                "white_night_confession_01",
            )),
            encounter_id: None,
            confirmed: false,
        });
    }

    core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: event_node_id,
        },
    )
    .expect("event node should be selectable");
    core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("event node should enter");
    core.execute(
        player_id,
        PlayerBehavior::AdvanceEventScene {
            node_id: event_node_id,
            event_id: crate::game::data::event_data::EventId::new("white_night_confession_01"),
            current_scene_id: crate::game::data::event_data::EventSceneId::new("start"),
        },
    )
    .expect("event should advance to choice");
    core.execute(
        player_id,
        PlayerBehavior::AdvanceEventScene {
            node_id: event_node_id,
            event_id: crate::game::data::event_data::EventId::new("white_night_confession_01"),
            current_scene_id: crate::game::data::event_data::EventSceneId::new("choice"),
        },
    )
    .expect("event panels should advance to end scene");

    let result = core
        .execute(
            player_id,
            PlayerBehavior::AdvanceEventScene {
                node_id: event_node_id,
                event_id: crate::game::data::event_data::EventId::new("white_night_confession_01"),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("end"),
            },
        )
        .expect("terminal omen event should complete");

    assert!(
        matches!(result, BehaviorResult::NodeCompleted { .. }),
        "event completion should commit before the authored boss capstone"
    );
    let run = core.state.run.as_ref().expect("run");
    let forced_boss_id = run
        .boss_omen
        .forced_boss_node_id()
        .expect("completed omen step should force a boss node");
    assert_eq!(
        run.map_progression.selectable_node_ids(
            &run.map,
            core.game_data
                .map_catalogs
                .facility_templates
                .template(&run.map.map_template_id)
                .expect("boss omen capstone template"),
        ),
        vec![forced_boss_id]
    );
    let forced_boss = run
        .map
        .node(forced_boss_id)
        .expect("forced boss node exists");
    assert_eq!(forced_boss.category, MapNodeCategory::Boss);
}

#[test]
fn generic_standard_boss_assignment_never_selects_final_boss_encounter() {
    let core = GameCore::new(game_data_with_pve_encounters(), 123);
    let boss_node_id = MapNodeId::new(Uuid::from_u128(0xA004));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new("standard_final_a"),
        facility_template_version: FACILITY_TEMPLATE_VERSION,
        nodes: vec![MapNode {
            id: boss_node_id,
            slot_id: MapSlotId::new("final_chamber"),
            kind_id: MapNodeKindId::new("boss_abnormality"),
            category: MapNodeCategory::Boss,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Encounter { encounter_id: None },
            omen: None,
            combat_difficulty: None,
        }],
        start_node_ids: vec![boss_node_id],
        terminals: vec![MapTerminalNode {
            node_id: boss_node_id,
            resolution: FacilityTerminalResolution::RunFinalBoss,
        }],
    };
    let mut run_progression = RunProgression::new(123, GameMode::Tutorial, 3);
    run_progression.mode_state = crate::game::map::RunProgressionModeState::Tutorial {
        floor_index: 2,
        max_floors: 3,
    };
    let mut encounter_progression =
        crate::game::encounter_progression::EncounterProgressionState::default();

    super::map_encounters::assign_map_encounters(
        &core.game_data.pve_data,
        &mut map,
        &run_progression,
        &mut encounter_progression,
        core.run_policy(),
    );

    let boss = map.node(boss_node_id).expect("boss node exists");
    assert!(matches!(
        &boss.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
            encounter.encounter_class == crate::game::data::pve_data::PveEncounterClass::NormalBoss
                && encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Boss)
        })
    ));
}

#[test]
fn tutorial_completion_advances_finite_floors_without_permanent_profile_changes() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    let profile_before = core.state.player_profile.clone();
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Tutorial);

    let mut floor_advanced_count = 0_u32;
    loop {
        let map = match core
            .execute(player_id, PlayerBehavior::RequestMapData)
            .unwrap()
        {
            BehaviorResult::MapState { map } => map,
            other => panic!("expected map state, got {other:?}"),
        };
        let next_node = map_node_ids_by_state(&map, MapNodeState::Available)
            .first()
            .copied()
            .expect("available node");
        {
            let run = core.state.run.as_mut().expect("run state");
            let node = run.map.node_mut(next_node).expect("node exists");
            node.category = MapNodeCategory::SavePoint;
            node.kind_id = crate::game::map::MapNodeKindId::new("save_point");
            node.payload = MapNodePayload::SavePoint;
        }
        core.execute(
            player_id,
            PlayerBehavior::SelectMapNode { node_id: next_node },
        )
        .unwrap();
        core.execute(player_id, PlayerBehavior::ConfirmEnterNode)
            .unwrap();

        match core
            .execute(player_id, PlayerBehavior::CompleteNode)
            .unwrap()
        {
            BehaviorResult::NodeCompleted { .. } => {}
            BehaviorResult::FloorAdvanced {
                game_mode,
                floor_index,
                map,
                ..
            } => {
                floor_advanced_count = floor_advanced_count.saturating_add(1);
                assert_eq!(game_mode, GameMode::Tutorial);
                assert_eq!(floor_index, floor_advanced_count);
                assert!(!map.nodes.is_empty());
                assert!(matches!(core.get_state(), GameState::ViewingMap));
                assert_eq!(core.state.player_profile, profile_before);
            }
            BehaviorResult::RunComplete { map } => {
                assert_eq!(
                    floor_advanced_count,
                    u32::from(run_policy().setup.standard_floor_count - 1)
                );
                assert!(!map.nodes.is_empty());
                assert!(matches!(core.get_state(), GameState::RunComplete));
                assert_eq!(core.state.player_profile, profile_before);
                assert_eq!(core.state.player_profile.highest_floor_reached, 0);
                assert_eq!(core.state.player_profile.longest_deathless_floor_streak, 0);
                break;
            }
            other => panic!("unexpected map progression result: {other:?}"),
        }
    }
}
