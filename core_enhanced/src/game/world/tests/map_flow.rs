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

    assert_eq!(map.act_index, 0);
    assert_eq!(map.max_acts, run_policy().setup.default_max_acts);
    assert_eq!(map.current_node_id, None);
    assert!(!map.nodes.is_empty());
    assert!(!map.edges.is_empty());
    let available_state_node_ids = map_node_ids_by_state(&map, MapNodeState::Available);
    assert!(!available_state_node_ids.is_empty());
    assert!(map.nodes.iter().any(|node| node.id == map.boss_node_id));
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
    let available_progression_node_ids = core
        .state
        .run
        .as_ref()
        .expect("run state")
        .map_progression
        .available_node_ids
        .clone();
    assert!(
        available_progression_node_ids.len() >= 2,
        "map fixture should expose at least two available nodes"
    );
    let first_node_id = available_progression_node_ids[0];
    let second_node_id = available_progression_node_ids[1];
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
fn generated_map_assigns_pve_encounters_to_combat_and_boss_nodes() {
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

    let boss = map
        .nodes
        .iter()
        .find(|node| node.id == map.boss_node_id)
        .expect("boss node exists");
    assert!(matches!(
        &boss.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if id == "boss_risk_encounter"
    ));

    assert!(map.nodes.iter().any(|node| {
        node.category == MapNodeCategory::Combat
            && matches!(
                &node.payload,
                MapNodePayload::Encounter {
                    encounter_id: Some(_)
                }
            )
    }));
}

#[test]
fn map_encounter_assignment_respects_node_kind_and_boss_purpose() {
    let core = GameCore::new(game_data_with_pve_encounters(), 123);
    let normal_node_id = MapNodeId::new(Uuid::from_u128(0xA001));
    let elite_node_id = MapNodeId::new(Uuid::from_u128(0xA002));
    let boss_node_id = MapNodeId::new(Uuid::from_u128(0xA003));
    let mut map = RunMap {
        nodes: vec![
            MapNode {
                id: normal_node_id,
                depth: 0,
                lane: 0,
                kind_id: MapNodeKindId::new("combat_monster"),
                category: MapNodeCategory::Combat,
                state: MapNodeState::Available,
                outgoing: vec![elite_node_id],
                payload: MapNodePayload::Encounter { encounter_id: None },
            },
            MapNode {
                id: elite_node_id,
                depth: 3,
                lane: 0,
                kind_id: MapNodeKindId::new("combat_elite"),
                category: MapNodeCategory::Combat,
                state: MapNodeState::Hidden,
                outgoing: vec![boss_node_id],
                payload: MapNodePayload::Encounter { encounter_id: None },
            },
            MapNode {
                id: boss_node_id,
                depth: 7,
                lane: 0,
                kind_id: MapNodeKindId::new("boss_abnormality"),
                category: MapNodeCategory::Boss,
                state: MapNodeState::Hidden,
                outgoing: vec![],
                payload: MapNodePayload::Encounter { encounter_id: None },
            },
        ],
        start_node_ids: vec![normal_node_id],
        boss_node_id,
    };
    let run_progression = RunProgression::new(123, 3);

    super::map_encounters::assign_map_encounters(
        &core.game_data.pve_data,
        &mut map,
        &run_progression,
    );

    let normal = map.node(normal_node_id).expect("normal node exists");
    assert!(matches!(
        &normal.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
            encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Defense)
                && encounter.node_type != Some(crate::game::combat_preview::CombatNodeType::Boss)
        })
    ));
    let elite = map.node(elite_node_id).expect("elite node exists");
    assert!(matches!(
        &elite.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if core.game_data.pve_data.get_by_id(id).is_some_and(|encounter| {
            encounter.node_type == Some(crate::game::combat_preview::CombatNodeType::Defense)
                && encounter.node_type != Some(crate::game::combat_preview::CombatNodeType::Boss)
        })
    ));
    let boss = map.node(boss_node_id).expect("boss node exists");
    assert!(matches!(
        &boss.payload,
        MapNodePayload::Encounter {
            encounter_id: Some(id)
        } if id == "boss_risk_encounter"
    ));
}

#[test]
fn boss_completion_advances_acts_until_run_complete() {
    let mut core = GameCore::new(empty_game_data(), 123);
    let player_id = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player_id);

    let mut act_complete_count = 0_u8;
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
            BehaviorResult::ActComplete { act_index, map } => {
                act_complete_count = act_complete_count.saturating_add(1);
                assert_eq!(act_index, act_complete_count);
                assert_eq!(map.act_index, act_index);
                assert_eq!(map.max_acts, run_policy().setup.default_max_acts);
                assert!(matches!(core.get_state(), GameState::ViewingMap));
            }
            BehaviorResult::RunComplete { map } => {
                assert_eq!(act_complete_count, run_policy().setup.default_max_acts - 1);
                assert_eq!(map.act_index, run_policy().setup.default_max_acts - 1);
                assert_eq!(map.max_acts, run_policy().setup.default_max_acts);
                assert!(matches!(core.get_state(), GameState::RunComplete));
                break;
            }
            other => panic!("unexpected map progression result: {other:?}"),
        }
    }
}
