use super::*;
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
        .node(core.state.run.as_ref().unwrap().map.terminal_node_id)
        .is_some());
    assert!(available_state_node_ids.iter().all(|node_id| {
        map.nodes
            .iter()
            .any(|node| node.id == *node_id && node.state == MapNodeState::Available)
    }));
    assert!(core.state.node_session.is_none());
    assert!(matches!(core.get_state(), GameState::ViewingMap));
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
        "this regression fixture must match MapGenerator's shared UUID prefix family"
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
    let map = MapGenerator::generate(123, MapGenerationConfig::default());
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
    assert_eq!(preview.spawn_waves[0].time_ms, 0);
}

#[test]
fn node_confirm_allows_reselecting_another_available_node_before_entering() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let selectable_node_ids = core
        .state
        .run
        .as_ref()
        .expect("run state")
        .map_progression
        .selectable_node_ids(&core.state.run.as_ref().expect("run state").map);
    assert!(
        selectable_node_ids.len() >= 2,
        "map fixture should expose at least two available nodes"
    );
    let first_node_id = selectable_node_ids[0];
    let second_node_id = selectable_node_ids[1];
    {
        let map = &mut core.state.run.as_mut().expect("run state").map;
        for (node_id, kind_id) in [
            (first_node_id, "support_first_preview"),
            (second_node_id, "support_second_preview"),
        ] {
            let node = map.node_mut(node_id).expect("node exists");
            node.category = MapNodeCategory::Support;
            node.kind_id = MapNodeKindId::new(kind_id);
            node.payload = MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            };
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
    assert!(core
        .get_allowed_actions()
        .contains(&ActionKind::SelectMapNode));

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
        | BehaviorResult::SupportState { node_id, .. }
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
    let selectable_node_ids = run.map_progression.selectable_node_ids(&run.map);
    let blocked_node_id = run
        .map
        .nodes
        .iter()
        .find(|node| !selectable_node_ids.contains(&node.id))
        .expect("generated map should contain a non-selectable future node")
        .id;
    assert!(!run
        .map_progression
        .is_node_selectable(&run.map, blocked_node_id));

    let result = core.execute(
        player_id,
        PlayerBehavior::SelectMapNode {
            node_id: blocked_node_id,
        },
    );

    assert!(matches!(result, Err(GameError::InvalidAction)));
    assert!(core.state.node_session.is_none());
    assert!(matches!(core.get_state(), GameState::ViewingMap));
}

#[test]
fn confirmed_gate_transition_advances_floor_and_recovers_living_trauma_without_checkpoint() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let employee_ids = core.roster().unwrap().available_employee_ids();
    {
        let roster = core.roster_mut().unwrap();
        roster.get_mut(&employee_ids[0]).unwrap().trauma = 90;
        roster
            .get_mut(&employee_ids[0])
            .unwrap()
            .health
            .set_current_hp(7);
        roster.get_mut(&employee_ids[1]).unwrap().trauma = 5;
        roster.get_mut(&employee_ids[2]).unwrap().trauma = 50;
        roster.get_mut(&employee_ids[2]).unwrap().life_state = EmployeeLifeState::Dead;
    }
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
        ..
    } = result
    else {
        panic!("expected gate to advance floor");
    };
    assert_eq!(game_mode, GameMode::Standard);
    assert_eq!(floor_index, 1);
    assert!(matches!(core.get_state(), GameState::ViewingMap));
    let roster = core.roster().unwrap();
    assert_eq!(roster.get(&employee_ids[0]).unwrap().trauma, 81);
    assert_eq!(roster.get(&employee_ids[0]).unwrap().health.current_hp, 7);
    assert_eq!(roster.get(&employee_ids[1]).unwrap().trauma, 5);
    assert_eq!(roster.get(&employee_ids[2]).unwrap().trauma, 50);
    assert!(!core.state.run_checkpoint.can_load());
    assert_eq!(
        core.state.run.as_ref().unwrap().run_progression.game_mode,
        GameMode::Standard
    );
}

#[test]
fn endless_gate_transition_advances_floor_without_max_cap_or_run_completion() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);

    let initial_snapshot = core.get_run_snapshot_json().unwrap();
    let initial_progression = initial_snapshot["run_progression"]
        .as_object()
        .expect("run progression object");
    assert_eq!(initial_progression["game_mode"], "Endless");
    assert_eq!(initial_progression["floor_index"], 0);
    assert!(
        !initial_progression.contains_key("max_floors"),
        "Endless snapshots must not expose a max floor cap"
    );
    assert!(initial_progression["current_floor_seed"].is_u64());

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
    } = result
    else {
        panic!("Endless Gate should advance to the next floor instead of completing the run");
    };
    assert_eq!(game_mode, GameMode::Endless);
    assert_eq!(floor_index, 1);
    assert!(!map.nodes.is_empty());
    assert!(matches!(core.get_state(), GameState::ViewingMap));

    let run = core.state.run.as_ref().expect("run should continue");
    assert_eq!(run.run_progression.game_mode, GameMode::Endless);
    assert_eq!(run.run_progression.floor_index(), 1);
    assert_eq!(run.run_progression.max_floors(), None);
    assert_eq!(
        run.map
            .node(run.map.terminal_node_id)
            .expect("terminal node exists")
            .category,
        MapNodeCategory::Gate
    );

    let advanced_snapshot = core.get_run_snapshot_json().unwrap();
    let advanced_progression = advanced_snapshot["run_progression"]
        .as_object()
        .expect("run progression object");
    assert_eq!(advanced_progression["game_mode"], "Endless");
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
    let employee_id = core.roster().unwrap().available_employee_ids()[0];
    {
        let employee = core.roster_mut().unwrap().get_mut(&employee_id).unwrap();
        employee.trauma = 80;
        employee.health.set_current_hp(11);
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
    assert_eq!(employee.trauma, 80);
    assert_eq!(employee.health.current_hp, 11);
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
    let allowed = core.get_allowed_actions();
    assert!(allowed.contains(&ActionKind::DeployUnit));
    assert!(allowed.contains(&ActionKind::WithdrawUnit));
}

#[test]
fn safe_node_entry_delivers_pending_research_fragments() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);
    let fragment_id = starter_basic_attack_fragment_id();
    let starting_count = core.state.skill_fragments.count(&fragment_id);
    core.state
        .skill_fragments
        .add_research_progress(&fragment_id, 100)
        .expect("research completion should be queued");
    assert_eq!(
        core.state
            .skill_fragments
            .pending_research_deliveries()
            .collect::<Vec<_>>(),
        vec![(&fragment_id, 1)]
    );

    let node_id = force_first_available_node(
        &mut core,
        MapNodeCategory::Support,
        "support_rest",
        MapNodePayload::Support {
            support_type: SupportNodeType::Rest,
            support_mode: crate::game::map::SupportNodeMode::Known,
            choices: vec![],
        },
    );
    core.execute(player_id, PlayerBehavior::SelectMapNode { node_id })
        .unwrap();
    let result = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .unwrap();

    let BehaviorResult::SupportState {
        research_deliveries,
        ..
    } = result
    else {
        panic!("expected support state");
    };
    assert_eq!(research_deliveries.len(), 1);
    assert_eq!(research_deliveries[0].fragment_id, fragment_id);
    assert_eq!(research_deliveries[0].count, 1);
    assert_eq!(research_deliveries[0].total_count, starting_count + 1);
    assert_eq!(
        core.state.skill_fragments.count(&fragment_id),
        starting_count + 1
    );
    assert!(core
        .state
        .skill_fragments
        .pending_research_deliveries()
        .next()
        .is_none());
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
        node.category = MapNodeCategory::Support;
        node.kind_id = crate::game::map::MapNodeKindId::new("support_rest");
        node.payload = MapNodePayload::Support {
            support_type: SupportNodeType::Rest,
            support_mode: SupportNodeMode::Known,
            choices: vec![],
        };
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
        | BehaviorResult::SupportState { node_id, .. }
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
    assert!(matches!(err, GameError::InvalidAction));
}

#[test]
fn generated_standard_maps_use_gate_before_final_floor_and_boss_on_final_floor() {
    let mut core = GameCore::new(game_data_with_pve_encounters(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let map = match core
        .execute(player_id, PlayerBehavior::RequestMapData)
        .unwrap()
    {
        BehaviorResult::MapState { map } => map,
        other => panic!("expected map state, got {other:?}"),
    };

    let run = core.state.run.as_ref().expect("run state");
    let terminal = run
        .map
        .node(run.map.terminal_node_id)
        .expect("terminal node exists");
    assert_eq!(terminal.category, MapNodeCategory::Gate);
    assert_eq!(terminal.kind_id.as_str(), "gate_stairs");
    let terminal_parent_ids = run
        .map
        .edges
        .iter()
        .filter_map(|edge| {
            (edge.to_node_id == run.map.terminal_node_id).then_some(edge.from_node_id)
        })
        .collect::<Vec<_>>();
    assert!(
        !terminal_parent_ids.is_empty(),
        "Standard non-final Gate must have incoming Elite parents"
    );
    for parent_id in terminal_parent_ids {
        let parent = run.map.node(parent_id).expect("gate parent exists");
        assert_eq!(parent.kind_id.as_str(), "combat_elite");
        assert!(matches!(
            &parent.payload,
            MapNodePayload::Encounter {
                encounter_id: Some(id)
            } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
                encounter.encounter_class == crate::game::data::pve_data::PveEncounterClass::Elite
            })
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
    final_floor_progression.mode_state = crate::game::map::RunProgressionModeState::Standard {
        floor_index: run_policy().setup.standard_floor_count - 1,
        max_floors: run_policy().setup.standard_floor_count,
    };
    let abnormality_research =
        crate::game::abnormality_research::RunAbnormalityResearchState::initialize(
            core.game_data.as_ref(),
        );
    let mut boss_omen = crate::game::boss_omen::BossOmenRunState::default();
    let (final_map, _) = core.generate_current_floor_map(
        &final_floor_progression,
        &abnormality_research,
        &crate::game::abnormality_research::RunAbnormalityEncounterHistory::default(),
        &mut boss_omen,
    );
    let final_terminal = final_map
        .node(final_map.terminal_node_id)
        .expect("final terminal exists");
    assert_eq!(final_terminal.category, MapNodeCategory::Boss);
    assert!(matches!(
        &final_terminal.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if id == "final_boss_risk_encounter"
    ));
}

#[test]
fn map_encounter_assignment_respects_node_kind_and_boss_purpose() {
    let core = GameCore::new(game_data_with_pve_encounters(), 123);
    let normal_node_id = MapNodeId::new(Uuid::from_u128(0xA001));
    let elite_node_id = MapNodeId::new(Uuid::from_u128(0xA002));
    let boss_node_id = MapNodeId::new(Uuid::from_u128(0xA003));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID),
        edges: vec![
            crate::game::map::MapEdgeDto {
                from_node_id: normal_node_id,
                to_node_id: elite_node_id,
                direction: crate::game::map::MapEdgeDirection::Bidirectional,
            },
            crate::game::map::MapEdgeDto {
                from_node_id: elite_node_id,
                to_node_id: boss_node_id,
                direction: crate::game::map::MapEdgeDirection::Bidirectional,
            },
        ],
        nodes: vec![
            MapNode {
                id: normal_node_id,
                depth: 0,
                lane: 0,
                slot_id: MapSlotId::for_grid_position(0, 0),
                kind_id: MapNodeKindId::new("combat_monster"),
                category: MapNodeCategory::Combat,
                state: MapNodeState::Available,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Encounter { encounter_id: None },
                omen: None,
            },
            MapNode {
                id: elite_node_id,
                depth: 3,
                lane: 0,
                slot_id: MapSlotId::for_grid_position(3, 0),
                kind_id: MapNodeKindId::new("combat_elite"),
                category: MapNodeCategory::Combat,
                state: MapNodeState::Unavailable,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Encounter { encounter_id: None },
                omen: None,
            },
            MapNode {
                id: boss_node_id,
                depth: 7,
                lane: 0,
                slot_id: MapSlotId::for_grid_position(7, 0),
                kind_id: MapNodeKindId::new("boss_abnormality"),
                category: MapNodeCategory::Boss,
                state: MapNodeState::Unavailable,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Encounter { encounter_id: None },
                omen: None,
            },
        ],
        start_node_ids: vec![normal_node_id],
        terminal_node_id: boss_node_id,
    };
    let run_progression = RunProgression::new(123, GameMode::Standard, 3);

    super::map_encounters::assign_map_encounters(
        &core.game_data.pve_data,
        &mut map,
        &run_progression,
        &crate::game::abnormality_research::RunAbnormalityResearchState::initialize(
            core.game_data.as_ref(),
        ),
        &crate::game::abnormality_research::RunAbnormalityEncounterHistory::default(),
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
        map_template_id: MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID),
        edges: vec![crate::game::map::MapEdgeDto {
            from_node_id: current_node_id,
            to_node_id: side_node_id,
            direction: crate::game::map::MapEdgeDirection::Bidirectional,
        }],
        nodes: vec![
            MapNode {
                id: current_node_id,
                depth: 1,
                lane: 0,
                slot_id: MapSlotId::for_grid_position(1, 0),
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
            },
            MapNode {
                id: side_node_id,
                depth: 2,
                lane: 0,
                slot_id: MapSlotId::for_grid_position(2, 0),
                kind_id: MapNodeKindId::new("combat_monster"),
                category: MapNodeCategory::Combat,
                state: MapNodeState::Available,
                visibility: MapNodeVisibility::Revealed,
                payload: MapNodePayload::Encounter { encounter_id: None },
                omen: None,
            },
        ],
        start_node_ids: vec![side_node_id],
        terminal_node_id: side_node_id,
    };
    let mut progression = crate::game::map::MapProgression {
        current_node_id: Some(current_node_id),
        available_node_ids: vec![side_node_id],
        completed_node_ids: vec![current_node_id],
    };
    let mut run_progression = RunProgression::new(123, GameMode::Endless, 3);
    run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Endless { floor_index: 3 };
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
    assert_eq!(map.terminal_node_id, forced_boss_id);
    assert!(matches!(
        &forced_boss.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if id == "suppress_white_night"
    ));
    assert_eq!(progression.available_node_ids, vec![forced_boss_id]);
    assert_eq!(progression.selectable_node_ids(&map), vec![forced_boss_id]);
    assert_eq!(
        map.node(side_node_id).expect("side node exists").state,
        MapNodeState::Unavailable
    );
}

#[test]
fn active_boss_omen_chain_overlays_matching_event_node() {
    let game_data = live_game_data_from_ron();
    let event_node_id = MapNodeId::new(Uuid::from_u128(0xB0_0101));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID),
        edges: vec![],
        nodes: vec![MapNode {
            id: event_node_id,
            depth: 1,
            lane: 0,
            slot_id: MapSlotId::for_grid_position(1, 0),
            kind_id: MapNodeKindId::new("event_story"),
            category: MapNodeCategory::Event,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Event { event_id: None },
            omen: None,
        }],
        start_node_ids: vec![event_node_id],
        terminal_node_id: event_node_id,
    };
    let mut run_progression = RunProgression::new(123, GameMode::Endless, 3);
    run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Endless { floor_index: 3 };
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
        map_template_id: MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID),
        edges: vec![],
        nodes: vec![MapNode {
            id: combat_node_id,
            depth: 1,
            lane: 0,
            slot_id: MapSlotId::for_grid_position(1, 0),
            kind_id: MapNodeKindId::new("combat_monster"),
            category: MapNodeCategory::Combat,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Encounter { encounter_id: None },
            omen: None,
        }],
        start_node_ids: vec![combat_node_id],
        terminal_node_id: combat_node_id,
    };
    let mut run_progression = RunProgression::new(123, GameMode::Endless, 3);
    run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Endless { floor_index: 3 };
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
        .with_starter_employee_data(base.starter_employee_data.clone())
        .with_recruitment_employee_data(base.recruitment_employee_data.clone())
        .with_artifact_data(base.artifact_data.clone())
        .with_consumable_data(base.consumable_data.clone())
        .with_equipment_data(base.equipment_data.clone())
        .with_shop_data(base.shop_data.clone())
        .with_reward_data(base.reward_data.clone())
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
    effects: Vec<crate::game::data::event_data::EventChoiceEffect>,
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
                        starts_combat: effects.iter().any(|effect| {
                            matches!(
                                effect,
                                crate::game::data::event_data::EventChoiceEffect::StartCombat { .. }
                            )
                        }),
                        ..Default::default()
                    },
                    effects,
                    next: None,
                }],
            },
        }],
    }
}

fn force_single_event_node_run(core: &mut GameCore, event_id: &str) -> MapNodeId {
    let start_node_id = MapNodeId::new(Uuid::from_u128(0xE1_0001));
    let event_node_id = MapNodeId::new(Uuid::from_u128(0xE1_0002));
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
    event.event_id
}

#[test]
fn live_awakened_fragment_can_trigger_boss_omen_overlay() {
    let game_data = live_game_data_from_ron();
    let fragment_id = game_data
        .skill_fragment_data
        .fragments
        .iter()
        .find_map(|fragment| match &fragment.effect {
            SkillFragmentEffectDef::ActiveSkill {
                awakened_skill_id: Some(_),
                ..
            } => Some(fragment.id.clone()),
            _ => None,
        })
        .expect("live data must contain at least one awakenable active fragment");
    let mut skill_fragments = crate::game::skill_fragment::SkillFragmentInventory::new();
    skill_fragments
        .add_id(&fragment_id)
        .expect("live awakenable fragment should be grantable");
    skill_fragments
        .add_fragment_dust(100)
        .expect("test should be able to add awakening dust");
    skill_fragments
        .awaken_with_dust(&fragment_id, &game_data.skill_fragment_data)
        .expect("live awakenable fragment should be awakenable");

    let event_node_id = MapNodeId::new(Uuid::from_u128(0xB0_0301));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID),
        edges: vec![],
        nodes: vec![MapNode {
            id: event_node_id,
            depth: 1,
            lane: 0,
            slot_id: MapSlotId::for_grid_position(1, 0),
            kind_id: MapNodeKindId::new("event_story"),
            category: MapNodeCategory::Event,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Event { event_id: None },
            omen: None,
        }],
        start_node_ids: vec![event_node_id],
        terminal_node_id: event_node_id,
    };
    let mut run_progression = RunProgression::new(123, GameMode::Endless, 3);
    run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Endless { floor_index: 3 };
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
        "awakened live fragment should trigger omen placement"
    );
    assert!(boss_omen.provisional.is_some());
    assert!(boss_omen.placed_source.is_some());
}

#[test]
fn event_choice_effects_are_atomic_when_later_effect_panics() {
    let event = event_choice_definition(
        "atomic_choice_event",
        "bad_choice",
        vec![
            crate::game::data::event_data::EventChoiceEffect::Grant {
                effects: vec![RewardEffect::GrantEnkephalin { amount: 10 }],
            },
            crate::game::data::event_data::EventChoiceEffect::StartCombat {
                encounter_id: "defense_encounter".to_string(),
                primary_abnormality_id: Some("defense_risk_abno".to_string()),
            },
            crate::game::data::event_data::EventChoiceEffect::StartCombat {
                encounter_id: "defense_encounter".to_string(),
                primary_abnormality_id: Some("defense_risk_abno".to_string()),
            },
        ],
    );
    let game_data = game_data_with_event_definitions(game_data_with_pve_encounters(), vec![event]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Standard);
    core.set_enkephalin(5);
    let event_node_id = force_single_event_node_run(&mut core, "atomic_choice_event");
    let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        core.execute(
            player_id,
            PlayerBehavior::SelectEventChoice {
                node_id: event_node_id,
                event_id,
                choice_id: crate::game::data::event_data::EventChoiceId::new("bad_choice"),
            },
        )
    }));

    assert!(
        result.is_err(),
        "duplicate StartCombat is a data invariant panic"
    );
    assert_eq!(core.get_enkephalin(), 5, "Grant must not partially apply");
    let session = core
        .state
        .run
        .as_ref()
        .expect("run")
        .event_sessions
        .get(&event_node_id)
        .expect("event session should remain");
    assert!(session.committed_choice_id.is_none());
    assert!(session.started_combat.is_none());
}

#[test]
fn apply_boss_omen_step_result_effect_fails_fast_until_implemented() {
    let event = event_choice_definition(
        "boss_omen_result_event",
        "apply_result",
        vec![
            crate::game::data::event_data::EventChoiceEffect::Grant {
                effects: vec![RewardEffect::GrantEnkephalin { amount: 10 }],
            },
            crate::game::data::event_data::EventChoiceEffect::ApplyBossOmenStepResult,
        ],
    );
    let game_data = game_data_with_event_definitions(game_data_with_pve_encounters(), vec![event]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Standard);
    core.set_enkephalin(5);
    let event_node_id = force_single_event_node_run(&mut core, "boss_omen_result_event");
    let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        core.execute(
            player_id,
            PlayerBehavior::SelectEventChoice {
                node_id: event_node_id,
                event_id,
                choice_id: crate::game::data::event_data::EventChoiceId::new("apply_result"),
            },
        )
    }));

    assert!(
        result.is_err(),
        "ApplyBossOmenStepResult must fail fast until mechanics are implemented"
    );
    assert_eq!(
        core.get_enkephalin(),
        5,
        "earlier Grant effects must not partially apply before fail-fast"
    );
    let session = core
        .state
        .run
        .as_ref()
        .expect("run")
        .event_sessions
        .get(&event_node_id)
        .expect("event session should remain");
    assert!(session.committed_choice_id.is_none());
    assert!(session.started_combat.is_none());
}

#[test]
fn event_choice_starts_combat_and_retreat_preserves_committed_choice() {
    let event = event_choice_definition(
        "combat_choice_event",
        "fight",
        vec![
            crate::game::data::event_data::EventChoiceEffect::StartCombat {
                encounter_id: "defense_encounter".to_string(),
                primary_abnormality_id: Some("defense_risk_abno".to_string()),
            },
        ],
    );
    let game_data = game_data_with_event_definitions(game_data_with_pve_encounters(), vec![event]);
    let mut core = GameCore::new(game_data, 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Standard);
    let event_node_id = force_single_event_node_run(&mut core, "combat_choice_event");
    let event_id = enter_single_scene_event(&mut core, player_id, event_node_id);

    let result = core
        .execute(
            player_id,
            PlayerBehavior::SelectEventChoice {
                node_id: event_node_id,
                event_id: event_id.clone(),
                choice_id: crate::game::data::event_data::EventChoiceId::new("fight"),
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
        session.committed_choice_id.as_ref().map(|id| id.as_str()),
        Some("fight")
    );
    assert!(session.started_combat.is_some());

    let retreat = core
        .execute(player_id, PlayerBehavior::RetreatBattle)
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
        session.committed_choice_id.as_ref().map(|id| id.as_str()),
        Some("fight")
    );
    assert!(session.started_combat.is_some());

    let resumed = core
        .execute(player_id, PlayerBehavior::ConfirmEnterNode)
        .expect("re-entering committed event should resume combat");
    assert!(matches!(resumed, BehaviorResult::BattleAdvanced { .. }));
}

#[test]
fn event_node_commands_advance_scene_select_choice_and_complete_node() {
    let mut core = GameCore::new(live_game_data_from_ron(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Standard);

    let start_node_id = MapNodeId::new(Uuid::from_u128(0xE0_0001));
    let event_node_id = MapNodeId::new(Uuid::from_u128(0xE0_0002));
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
                    event_id: Some(crate::game::data::event_data::EventId::new(
                        "white_night_confession_01",
                    )),
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
    assert_eq!(event.choices.len(), 2);

    let chosen = core
        .execute(
            player_id,
            PlayerBehavior::SelectEventChoice {
                node_id: event_node_id,
                event_id: crate::game::data::event_data::EventId::new("white_night_confession_01"),
                choice_id: crate::game::data::event_data::EventChoiceId::new("listen"),
            },
        )
        .expect("event choice should advance to the result scene");
    let event = match chosen {
        BehaviorResult::EventState { event, .. } => event,
        other => panic!("expected event result scene, got {other:?}"),
    };
    assert_eq!(event.current_scene_id.as_str(), "end");

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
        matches!(completed, BehaviorResult::FloorAdvanced { .. }),
        "expected FloorAdvanced after terminal event node, got {completed:?}"
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
fn terminal_event_boss_omen_step_consumption_forces_boss_on_next_floor() {
    let mut core = GameCore::new(live_game_data_from_ron(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_mode_and_default_starters(&mut core, player_id, GameMode::Endless);
    let event_node_id = force_single_event_node_run(&mut core, "white_night_confession_01");
    {
        let run = core.state.run.as_mut().expect("run");
        run.run_progression.mode_state =
            crate::game::map::RunProgressionModeState::Endless { floor_index: 3 };
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
        PlayerBehavior::SelectEventChoice {
            node_id: event_node_id,
            event_id: crate::game::data::event_data::EventId::new("white_night_confession_01"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("listen"),
        },
    )
    .expect("event choice should advance to end scene");

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
        matches!(result, BehaviorResult::FloorAdvanced { .. }),
        "terminal event completion should advance to the next floor safe state"
    );
    let run = core.state.run.as_ref().expect("run");
    let forced_boss_id = run
        .boss_omen
        .forced_boss_node_id()
        .expect("completed omen step should force a boss node");
    assert_eq!(
        run.map_progression.selectable_node_ids(&run.map),
        vec![forced_boss_id]
    );
    let forced_boss = run
        .map
        .node(forced_boss_id)
        .expect("forced boss node exists");
    assert_eq!(forced_boss.category, MapNodeCategory::Boss);
}

#[test]
fn final_standard_floor_boss_node_selects_final_boss_encounter() {
    let core = GameCore::new(game_data_with_pve_encounters(), 123);
    let boss_node_id = MapNodeId::new(Uuid::from_u128(0xA004));
    let mut map = RunMap {
        map_template_id: MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID),
        edges: vec![],
        nodes: vec![MapNode {
            id: boss_node_id,
            depth: 7,
            lane: 0,
            slot_id: MapSlotId::for_grid_position(7, 0),
            kind_id: MapNodeKindId::new("boss_abnormality"),
            category: MapNodeCategory::Boss,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Encounter { encounter_id: None },
            omen: None,
        }],
        start_node_ids: vec![boss_node_id],
        terminal_node_id: boss_node_id,
    };
    let mut run_progression = RunProgression::new(123, GameMode::Standard, 3);
    run_progression.mode_state = crate::game::map::RunProgressionModeState::Standard {
        floor_index: 2,
        max_floors: 3,
    };

    super::map_encounters::assign_map_encounters(
        &core.game_data.pve_data,
        &mut map,
        &run_progression,
        &crate::game::abnormality_research::RunAbnormalityResearchState::initialize(
            core.game_data.as_ref(),
        ),
        &crate::game::abnormality_research::RunAbnormalityEncounterHistory::default(),
        core.run_policy(),
    );

    let boss = map.node(boss_node_id).expect("boss node exists");
    assert!(matches!(
        &boss.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
            encounter.encounter_class == crate::game::data::pve_data::PveEncounterClass::FinalBoss
                && encounter.primary_abnormality_id.as_deref() == Some("final_boss_risk_abno")
                && encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Boss)
        })
    ));
}

#[test]
fn boss_completion_advances_standard_floors_until_run_complete() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

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
            node.category = MapNodeCategory::Support;
            node.kind_id = crate::game::map::MapNodeKindId::new("support_rest");
            node.payload = MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            };
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
            } => {
                floor_advanced_count = floor_advanced_count.saturating_add(1);
                assert_eq!(game_mode, GameMode::Standard);
                assert_eq!(floor_index, floor_advanced_count);
                assert!(!map.nodes.is_empty());
                assert!(matches!(core.get_state(), GameState::ViewingMap));
            }
            BehaviorResult::RunComplete { map } => {
                assert_eq!(
                    floor_advanced_count,
                    u32::from(run_policy().setup.standard_floor_count - 1)
                );
                assert!(!map.nodes.is_empty());
                assert!(matches!(core.get_state(), GameState::RunComplete));
                break;
            }
            other => panic!("unexpected map progression result: {other:?}"),
        }
    }
}
