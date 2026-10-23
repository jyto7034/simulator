use super::*;
use crate::game::data::content_pack::{self, tests::ExternalFixture, *};
use crate::game::data::reward_data::{RewardDatabase, RewardMetadata, RewardPoolMetadata};

fn seed_compendium(
    core: &mut GameCore,
) -> crate::game::encounter_compendium::EncounterCompendiumProfileState {
    let live_id = core.game_data.abnormality_data.items[0].id.clone();
    core.merge_encounter_compendium_profile(
        crate::game::encounter_compendium::EncounterCompendiumProfileState {
            registered_ids: std::collections::BTreeSet::from([
                live_id,
                "dormant_external_compendium_id".into(),
            ]),
        },
    )
    .unwrap();
    core.encounter_compendium_profile()
}

fn finish_started_briefing(core: &mut GameCore, player: Uuid, start: BehaviorResult) {
    let BehaviorResult::RunStartFlowStarted {
        visual_novel: Some(vn),
        ..
    } = start
    else {
        panic!("generation start must open the real briefing");
    };
    core.execute(
        player,
        PlayerBehavior::AdvanceVisualNovel {
            session_id: vn.session_id,
            current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId::new(
                vn.current_scene_id,
            ),
        },
    )
    .unwrap();
    let employee_definition_ids = core
        .state
        .starter_candidates
        .iter()
        .take(core.game_data.run_policy.setup.starter_employee_count)
        .map(|d| d.employee_definition_id.clone())
        .collect();
    core.execute(
        player,
        PlayerBehavior::SelectStarterEmployees {
            employee_definition_ids,
            loan_copy_ids: vec![],
        },
    )
    .unwrap();
}

fn savepoint(core: &mut GameCore, player: Uuid) {
    let node = force_first_available_node(
        core,
        MapNodeCategory::SavePoint,
        "save_point",
        MapNodePayload::SavePoint,
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
}

fn enter_battle(core: &mut GameCore, player: Uuid) {
    let node = force_first_available_node(
        core,
        MapNodeCategory::Combat,
        "combat_monster",
        MapNodePayload::Encounter {
            encounter_id: Some("suppress_warm_hearted_woodsman".into()),
        },
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(matches!(core.get_state(), GameState::InBattle { .. }));
}

#[test]
fn external_b_start_pins_actual_battle_resource_and_export_while_a_remains_pinned() {
    let files = ExternalFixture::official();
    let a = files.load();
    let player = Uuid::from_u128(0xCB02);
    let mut active_a = GameCore::new(a.clone(), 0xCB02);
    start_new_game_with_default_starters(&mut active_a, player);
    savepoint(&mut active_a, player);
    enter_battle(&mut active_a, player);
    let a_snapshot = active_a.get_run_snapshot_json().unwrap();
    let a_export = active_a.export_active_run_checkpoint().unwrap().unwrap();
    let mut policy = (*a.run_policy).clone();
    assert_eq!(policy.tactical_resource.initial, 20);
    policy.tactical_resource.initial = 31;
    files.write(
        "official/data/run/policy.ron",
        ron::ser::to_string(&policy).unwrap(),
    );
    let b = files.load();
    assert_ne!(a.identity(), b.identity());
    assert_eq!(active_a.content_identity(), a.identity());
    assert_eq!(active_a.get_run_snapshot_json().unwrap(), a_snapshot);
    assert_eq!(
        active_a.export_active_run_checkpoint().unwrap().unwrap(),
        a_export
    );
    let battle_a = active_a.state.active_battle.as_ref().unwrap();
    assert_eq!(battle_a.tactical_resource.current(), 20);
    assert!(Arc::ptr_eq(&battle_a.battle.game_data, &a.database_arc()));

    let mut active_b = GameCore::new(a.clone(), 0xCB03);
    active_b.state.player_profile.highest_floor_reached = 17;
    let compendium = seed_compendium(&mut active_b);
    let start = active_b
        .execute_with_transport_context_and_generation(
            player,
            PlayerBehavior::StartNewGame {
                game_mode: GameMode::Expedition,
                replace_active_run: false,
            },
            None,
            0,
            None,
            Some(b.clone()),
        )
        .unwrap();
    assert_eq!(active_b.content_identity(), b.identity());
    assert_eq!(active_b.encounter_compendium_profile(), compendium);
    finish_started_briefing(&mut active_b, player, start);
    savepoint(&mut active_b, player);
    assert_eq!(active_b.encounter_compendium_profile(), compendium);
    let b_export = active_b.export_active_run_checkpoint().unwrap().unwrap();
    let a_document: Value = serde_json::from_slice(&a_export.core_payload_json).unwrap();
    let b_document: Value = serde_json::from_slice(&b_export.core_payload_json).unwrap();
    assert_eq!(
        a_document["continuation"]["current"]["content_identity"],
        json!(a.identity())
    );
    assert_eq!(
        b_document["continuation"]["current"]["content_identity"],
        json!(b.identity())
    );
    assert_ne!(
        a_document["continuation"]["current"]["content_identity"],
        b_document["continuation"]["current"]["content_identity"]
    );
    enter_battle(&mut active_b, player);
    let battle_b = active_b.state.active_battle.as_ref().unwrap();
    assert_eq!(battle_b.tactical_resource.current(), 31);
    assert!(Arc::ptr_eq(&battle_b.battle.game_data, &b.database_arc()));
    assert!(!Arc::ptr_eq(&battle_b.battle.game_data, &a.database_arc()));
    assert_eq!(active_b.state.player_profile.highest_floor_reached, 17);
    assert_eq!(active_a.get_run_snapshot_json().unwrap(), a_snapshot);
}

#[test]
fn fixture_b_late_briefing_preparation_failure_preserves_complete_a_authority_on_retry() {
    // This is deliberate fixture-only invalid static data. The public external
    // loader rejects the missing RunStart trigger before any token is published.
    let b: crate::game::data::ValidatedGameData = GameDataBuilder::empty()
        .with_employee_definitions(test_employee_definition_database())
        .with_visual_novels(test_hq_briefing_visual_novel_database())
        .with_equipment(vec![starter_weapon_meta(), starter_equipment_meta()])
        .with_skill_fragment_data(Arc::new(SkillFragmentDatabase::new(vec![
            SkillFragmentMetadata::starter_basic_attack(),
        ])))
        .build_arc()
        .into();
    let files = ExternalFixture::official();
    let mut a = GameCore::new(files.load(), 0xCB04);
    a.state.player_profile.highest_floor_reached = 19;
    let compendium = seed_compendium(&mut a);
    let identity = a.content_identity().clone();
    assert_ne!(&identity, b.identity());
    let before = non_battle_state_without_profile_fingerprint(&a);
    let profile = a.player_profile();
    let mut previous = None;
    for _ in 0..2 {
        let error = a
            .execute_with_transport_context_and_generation(
                Uuid::from_u128(0xCB04),
                PlayerBehavior::StartNewGame {
                    game_mode: GameMode::Expedition,
                    replace_active_run: false,
                },
                None,
                0,
                None,
                Some(b.clone()),
            )
            .unwrap_err();
        assert!(
            matches!(&error, GameError::InvalidStaticData(message) if message.contains("missing RunStart visual novel trigger"))
        );
        if let Some(previous) = previous {
            assert_eq!(format!("{error:?}"), previous);
        }
        previous = Some(format!("{error:?}"));
        assert_eq!(a.content_identity(), &identity);
        assert_eq!(a.run_seed, 0xCB04);
        assert_eq!(a.player_profile(), profile);
        assert_eq!(a.encounter_compendium_profile(), compendium);
        assert_eq!(non_battle_state_without_profile_fingerprint(&a), before);
        assert!(a.state.player.is_none());
        assert!(a.state.visual_novel.active.is_none());
    }
}

fn manifest(
    id: &str,
    namespace: PackNamespace,
    dependencies: Vec<&str>,
    files: Vec<(CatalogKind, &str)>,
) -> PackManifest {
    PackManifest {
        pack_id: id.into(),
        pack_version: "1".into(),
        namespace,
        contract: ContentContract::current(),
        required_capabilities: vec![],
        dependencies: dependencies.into_iter().map(str::to_owned).collect(),
        files: files
            .into_iter()
            .map(|(kind, path)| CatalogFile {
                kind,
                path: path.into(),
            })
            .collect(),
        operations: vec![],
    }
}

#[test]
fn valid_external_remove_updates_namespaced_lookup_registry_recipe_and_reward_acquisition() {
    let files = ExternalFixture::official();
    let official = files.load();
    let mut left = official.equipment_data.items[0].clone();
    left.id = "left::same".into();
    left.uuid = Uuid::from_u128(0xCB10);
    let mut right = left.clone();
    right.id = "right::same".into();
    right.uuid = Uuid::from_u128(0xCB11);
    let mut recipe = official.equipment_data.dismantle_recipes[0].clone();
    recipe.equipment_id = right.id.clone();
    let left_data = EquipmentDatabase::with_all(vec![left.clone()], vec![], vec![recipe], vec![]);
    let right_data = EquipmentDatabase::with_all(vec![right.clone()], vec![], vec![], vec![]);
    for (id, equipment, item, uuid) in [
        ("left", left_data, &left, 0xCB12),
        ("right", right_data, &right, 0xCB13),
    ] {
        let reward = RewardMetadata {
            id: format!("{id}::reward"),
            uuid: Uuid::from_u128(uuid),
            name: id.into(),
            description: "external Remove fixture".into(),
            icon: "fixture".into(),
            effects: vec![RewardEffect::GrantEquipment {
                equipment_id: item.id.clone(),
            }],
        };
        let pools = if id == "left" {
            vec![RewardPoolMetadata {
                id: "left::pool".into(),
                reward_ids: vec!["left::reward".into(), "right::reward".into()],
            }]
        } else {
            vec![]
        };
        files.write(
            &format!("{id}/equipment.ron"),
            ron::ser::to_string(&equipment).unwrap(),
        );
        files.write(
            &format!("{id}/rewards.ron"),
            ron::ser::to_string(&RewardDatabase::new_with_pools(vec![reward], pools)).unwrap(),
        );
        let dependencies = if id == "left" {
            vec!["official", "right"]
        } else {
            vec!["official"]
        };
        files.manifest(
            id,
            manifest(
                id,
                PackNamespace::Named(id.into()),
                dependencies,
                vec![
                    (CatalogKind::Equipment, "equipment.ron"),
                    (CatalogKind::Rewards, "rewards.ron"),
                ],
            ),
        );
    }
    let mut selection = files.selection();
    selection.packs.extend(["left".into(), "right".into()]);
    let a = content_pack::load(files.root(), &selection, &InputLimits::default()).unwrap();
    assert!(a.equipment_data.get_by_id(&left.id).is_some());
    assert!(a.equipment_data.get_by_uuid(&left.uuid).is_some());
    assert!(a.item(&left.uuid).is_some());
    assert!(a.item(&right.uuid).is_some());
    assert_eq!(
        a.equipment_data
            .get_dismantle_recipe_by_equipment_id(&right.id)
            .unwrap()
            .equipment_id,
        right.id
    );
    let mut patch = manifest(
        "remove",
        PackNamespace::Global,
        vec!["left"],
        vec![(CatalogKind::Rewards, "rewards.ron")],
    );
    patch.operations = vec![
        DefinitionOperation {
            target: DefinitionKey {
                kind: DefinitionKind::Equipment,
                id: left.id.clone(),
            },
            operation: OperationKind::Remove,
            after: vec!["left".into()],
        },
        DefinitionOperation {
            target: DefinitionKey {
                kind: DefinitionKind::Reward,
                id: "left::reward".into(),
            },
            operation: OperationKind::Remove,
            after: vec!["left".into()],
        },
        DefinitionOperation {
            target: DefinitionKey {
                kind: DefinitionKind::RewardPool,
                id: "left::pool".into(),
            },
            operation: OperationKind::Replace,
            after: vec!["left".into()],
        },
    ];
    files.manifest("remove", patch);
    let dangling = RewardDatabase::new_with_pools(
        vec![],
        vec![RewardPoolMetadata {
            id: "left::pool".into(),
            reward_ids: vec!["left::reward".into(), "right::reward".into()],
        }],
    );
    files.write(
        "remove/rewards.ron",
        ron::ser::to_string(&dangling).unwrap(),
    );
    selection.packs.push("remove".into());
    let error = match content_pack::load(files.root(), &selection, &InputLimits::default()) {
        Ok(_) => panic!("Remove with dangling pool reference published"),
        Err(error) => error,
    };
    assert_eq!(error.source_file, "remove/rewards.ron");
    assert!(error.cause.contains("left::reward"), "{error}");
    let fixed = RewardDatabase::new_with_pools(
        vec![],
        vec![RewardPoolMetadata {
            id: "left::pool".into(),
            reward_ids: vec!["right::reward".into()],
        }],
    );
    files.write("remove/rewards.ron", ron::ser::to_string(&fixed).unwrap());
    let b = content_pack::load(files.root(), &selection, &InputLimits::default()).unwrap();
    assert!(b.equipment_data.items.iter().all(|item| item.id != left.id));
    assert!(b.equipment_data.get_by_id(&left.id).is_none());
    assert!(b.equipment_data.get_by_uuid(&left.uuid).is_none());
    assert!(b.item(&left.uuid).is_none());
    assert!(b.item(&right.uuid).is_some());
    assert!(b.reward_data.get_by_id("left::reward").is_none());
    assert!(b
        .equipment_data
        .get_dismantle_recipe_by_equipment_id(&right.id)
        .is_some());
    assert!(a.item(&left.uuid).is_some());
    selection.packs.reverse();
    assert_eq!(
        content_pack::load(files.root(), &selection, &InputLimits::default())
            .unwrap()
            .identity(),
        b.identity()
    );
    let mut core = GameCore::new(b, 0xCB14);
    let player = Uuid::from_u128(0xCB14);
    start_new_game_with_default_starters(&mut core, player);
    let node = force_first_available_node(
        &mut core,
        MapNodeCategory::Reward,
        "reward_treasure",
        MapNodePayload::Reward {
            reward_pool_id: Some("left::pool".into()),
        },
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    let result = core
        .execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    assert!(
        matches!(result, BehaviorResult::RewardState { rewards, .. } if rewards.len() == 1 && rewards[0].id == "right::reward")
    );
    assert!(matches!(
        core.execute(player, PlayerBehavior::ClaimReward).unwrap(),
        BehaviorResult::RewardGranted { .. }
    ));
    assert_eq!(
        core.state
            .inventory
            .equipments
            .iter()
            .filter(|owned| owned.meta.id == right.id)
            .count(),
        1
    );
    assert!(core
        .state
        .inventory
        .equipments
        .iter()
        .all(|owned| owned.meta.id != left.id));
}
