fn u04_continue(core: &GameCore, player: Uuid) -> GameCore {
    let export = core.export_active_run_checkpoint().unwrap().expect("confirmed continuation");
    let mut restarted = core_with_profile_for_continue(&core, 0xDEAD);
    restarted.continue_from_active_run_checkpoint(
        player, &export.core_payload_json, export.loads_used, export.run_peak_certified_power,
    ).unwrap();
    restarted
}

#[test]
fn u04_continue_before_first_savepoint_restores_current_roster_and_map() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xC004_0001);
    let player = Uuid::from_u128(1);
    start_new_game_with_legacy_sword_fixture(&mut core, player);
    assert!(core.state.run_checkpoint.payload.is_none());
    let restored = u04_continue(&core, player);
    assert_eq!(restored.get_state(), GameState::ViewingMap);
    assert_eq!(serde_json::to_value(&core.state.roster).unwrap(), serde_json::to_value(&restored.state.roster).unwrap());
    assert_eq!(core.get_run_snapshot_json().unwrap(), restored.get_run_snapshot_json().unwrap());
    assert!(restored.state.run_checkpoint.payload.is_none());
    assert_eq!(restored.state.run_checkpoint.loads_used, 0);
}

#[test]
fn u04_continue_unfinished_battle_preserves_two_retreats_and_third_entry_without_quit_charge() {
    let mut core = GameCore::new(legacy_pool_game_data_from_ron(), 0xC004_0002);
    let player = Uuid::from_u128(1);
    start_new_game_with_legacy_sword_fixture(&mut core, player);
    let node = force_map_combat_node(&mut core, MapNodeCategory::Combat, "combat_monster", "suppress_burrowing_heaven");
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
    let initial_roster = serde_json::to_value(&core.state.roster).unwrap();
    for retreat in 1..=2 {
        execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
        assert_eq!(core.run_state().unwrap().abnormality_attempt_state(node).retreats_used, retreat);
        core = u04_continue(&core, player);
        assert_eq!(core.get_state(), GameState::ViewingMap);
        assert_eq!(core.state.run_checkpoint.loads_used, 0);
        core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
        core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
    }
    assert!(!core.active_battle_retreat_available());
    let attempt = core.run_state().unwrap().abnormality_attempt_state(node);
    assert_eq!(attempt.attempts_started, 3);
    assert_eq!(attempt.retreats_used, 2);
    let before = core.get_run_snapshot_json().unwrap();
    assert!(execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).is_err());
    assert_eq!(core.get_run_snapshot_json().unwrap(), before);
    for _ in 0..2 {
        core = u04_continue(&core, player);
        assert_eq!(serde_json::to_value(&core.state.roster).unwrap(), initial_roster);
        assert_eq!(core.run_state().unwrap().abnormality_attempt_state(node).retreats_used, 2);
        core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
        core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
        assert_eq!(core.run_state().unwrap().abnormality_attempt_state(node).attempts_started, 3);
        assert!(!core.active_battle_retreat_available());
    }
}

#[test]
fn u04_live_terminal_commits_before_ack_and_continue_cannot_repeat_grants() {
    let mut data = live_game_data_from_ron();
    let database = Arc::get_mut(&mut data).unwrap();
    let pve = Arc::get_mut(&mut database.pve_data).unwrap();
    let encounter_id = "suppress_burrowing_heaven";
    let encounter = pve.encounters.iter_mut().find(|entry| entry.id == encounter_id).unwrap();
    encounter.terminal_outcomes.retain(|outcome| outcome.verdict == BattleVerdict::Victory);
    let outcome_id = encounter.terminal_outcomes[0].id.clone();
    encounter.outcome_policies.retain(|policy| policy.outcome_id == outcome_id);
    encounter.terminal_outcomes[0].condition = crate::game::data::pve_data::PveTerminalConditionData::TimeReached { time_ms: 10 };
    let mut core = GameCore::new(data, 0xC004_0003);
    let player = Uuid::from_u128(1);
    start_new_game_with_legacy_sword_fixture(&mut core, player);
    let node = force_map_combat_node(&mut core, MapNodeCategory::Combat, "combat_monster", "suppress_burrowing_heaven");
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
    let result = core.advance_active_battle_by_after_seq(50, 0).unwrap();
    assert!(matches!(result, BehaviorResult::BattleAdvanced { finished: true, .. }));
    assert!(matches!(core.get_state(), GameState::CombatResult { .. }));
    assert_eq!(core.run_state().unwrap().map.node(node).unwrap().state, MapNodeState::Completed);
    assert!(core.state.active_battle.is_none());
    let amount = core.state.enkephalin.amount;
    let roster = serde_json::to_value(&core.state.roster).unwrap();
    let inventory = serde_json::to_value(&core.state.inventory).unwrap();
    let summary = core.get_selected_event_snapshot_json().unwrap();
    core = u04_continue(&core, player);
    assert!(matches!(core.get_state(), GameState::CombatResult { .. }));
    assert_eq!(core.get_selected_event_snapshot_json().unwrap(), summary);
    core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    assert_eq!(core.state.enkephalin.amount, amount);
    assert_eq!(serde_json::to_value(&core.state.roster).unwrap(), roster);
    assert_eq!(serde_json::to_value(&core.state.inventory).unwrap(), inventory);
    assert!(core.execute(player, PlayerBehavior::CompleteCombatResult).is_err());
    core = u04_continue(&core, player);
    assert_eq!(core.get_state(), GameState::ViewingMap);
    assert_eq!(core.state.enkephalin.amount, amount);
    assert_eq!(core.run_state().unwrap().map.node(node).unwrap().state, MapNodeState::Completed);
}

fn u04_choose_one_fixture(count: usize) -> (GameCore, Uuid, Uuid, Vec<Uuid>) {
    u04_choose_one_fixture_with_terminal(count, false)
}

fn u04_terminal_profile_fixture() -> crate::game::encounter_compendium::PlayerProfileState {
    use crate::game::expedition_lifecycle::{FirstReturnStory, VaultEquipment};
    let mut profile = crate::game::encounter_compendium::PlayerProfileState::default();
    profile.highest_floor_reached = 7;
    profile.encounter_compendium.registered_ids.insert("u04_prior_terminal_knowledge".into());
    profile.lifecycle.expedition_sequence = 1;
    profile.lifecycle.first_return_story = FirstReturnStory::Consumed;
    let copy_uuid = Uuid::from_u128(0xC004_F001);
    profile.lifecycle.vault.equipment.insert(copy_uuid, VaultEquipment {
            weapon_stage: Default::default(),
        copy_uuid, definition_id: "u04_dormant_returned_weapon".into(),
        equipment_type: EquipmentType::Weapon, enhancement_level: 0,
    });
    profile.validate_persisted().expect("valid dormant assets and prior permanent knowledge");
    assert!(profile.lifecycle.active_expedition.is_none());
    profile
}

fn u04_choose_one_fixture_with_terminal(count: usize, terminal: bool) -> (GameCore, Uuid, Uuid, Vec<Uuid>) {
    let mut data = live_game_data_from_ron();
    let database = Arc::get_mut(&mut data).unwrap();
    let reward_ids = (0..count).map(|index| Uuid::from_u128(0xC004_1000 + index as u128)).collect::<Vec<_>>();
    let mut rewards = (*database.reward_data).clone();
    for (index, uuid) in reward_ids.iter().enumerate() {
        rewards.rewards.push(RewardMetadata {
            id: format!("u04_choice_{index}"), uuid: *uuid, name: format!("choice {index}"),
            description: String::new(), icon: String::new(),
            effects: vec![RewardEffect::GrantEnkephalin { amount: 17 + index as u32 },
                RewardEffect::GrantExperience { amount: 31, target: ExperienceTargetPolicy::CombatParticipants }],
        });
    }
    if count > 0 {
        rewards.rewards.iter_mut().find(|reward| reward.uuid == reward_ids[0]).unwrap().effects.push(
            RewardEffect::GrantSkillFragment { fragment_id: database.skill_fragment_data.fragments[0].id.clone() }
        );
    }
    if terminal {
        // This fixture exercises the encounter's authored choice pool, without
        // the live final slot's separate ClaimAll reward override.
        for profile in &mut Arc::make_mut(&mut database.map_catalogs.node_assignment_profiles).profiles {
            for assignment in &mut profile.fixed_slot_assignments {
                assignment.authored_combat = None;
            }
        }
    }
    // Rebuild indexes after authoring the fixture, through its regular codec.
    database.reward_data = Arc::new(serde_json::from_value(serde_json::to_value(rewards).unwrap()).unwrap());
    let pve = Arc::get_mut(&mut database.pve_data).unwrap();
    let encounter_id = if terminal { "suppress_nothing_there" } else { "suppress_burrowing_heaven" };
    let encounter = pve.encounters.iter_mut().find(|entry| entry.id == encounter_id).unwrap();
    encounter.terminal_outcomes.retain(|outcome| outcome.verdict == BattleVerdict::Victory);
    let outcome_id = encounter.terminal_outcomes[0].id.clone();
    encounter.outcome_policies.retain(|policy| policy.outcome_id == outcome_id);
    encounter.terminal_outcomes[0].condition = crate::game::data::pve_data::PveTerminalConditionData::TimeReached { time_ms: 10 };
    encounter.outcome_policies[0].reward_mode = RewardMode::ChooseOne;
    encounter.outcome_policies[0].reward_uuids = reward_ids.clone();
    encounter.outcome_policies[0].grant_node_bonus = false;
    let mut core = GameCore::new(data, 0xC004_0004);
    let player = Uuid::from_u128(1);
    if terminal {
        // I is installed before the public Tutorial start. It is a valid
        // existing profile, with no fabricated active expedition or loan.
        core.state.player_profile = u04_terminal_profile_fixture();
        // I targets the Tutorial terminal owner. Start that mode normally so
        // mode, instance and deterrence agree before moving to its final floor.
        start_new_game_with_mode_and_default_starters(&mut core, player, GameMode::Tutorial);
        apply_legacy_sword_numerical_fixture(&mut core, player);
    } else {
        start_new_game_with_legacy_sword_fixture(&mut core, player);
    }
    let node = if terminal {
        force_map_combat_node(&mut core, MapNodeCategory::Boss, "boss_abnormality", encounter_id)
    } else {
        force_map_combat_node(&mut core, MapNodeCategory::Combat, "combat_monster", encounter_id)
    };
    if terminal {
        let run = core.state.run.as_mut().unwrap();
        run.run_progression.mode_state = crate::game::map::RunProgressionModeState::Tutorial {
            floor_index: run_policy().setup.standard_floor_count - 1,
            max_floors: run_policy().setup.standard_floor_count,
        };
        assert_eq!(run.map.terminal_resolution(node), Some(FacilityTerminalResolution::StandardCompletion));
    }
    let preview = core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
    let position = first_ground_deployment_cell(&preview);
    let employee = core.roster().unwrap().combat_eligible_employee_ids()[0];
    core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
    execute_current_battle_mutation(&mut core, player, PlayerBehavior::DeployUnit {
        employee_uuid: employee, position, facing: FacingDirection::Right,
    }).unwrap();
    core.advance_active_battle_by_after_seq(50, 0).unwrap();
    (core, player, employee, reward_ids)
}

#[test]
fn u04_choose_one_two_candidates_and_participant_context_survive_select_claim_restarts() {
    let (mut core, player, employee, candidates) = u04_choose_one_fixture(2);
    let amount = core.state.enkephalin.amount;
    let experience = core.state.roster.get(&employee).unwrap().experience;
    let other_experience = core.state.roster.iter().filter(|other| other.uuid != employee)
        .map(|other| (other.uuid, other.experience)).collect::<Vec<_>>();
    core = u04_continue(&core, player);
    core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    let reward = core.state.active_node_content.as_ref().unwrap().as_reward().unwrap();
    assert_eq!(reward.rewards.iter().map(|reward| reward.uuid).collect::<Vec<_>>(), candidates);
    assert!(!reward.can_skip);
    assert!(core.execute(player, PlayerBehavior::ExitReward).is_err());
    assert!(core.execute(player, PlayerBehavior::ClaimReward).is_err());
    core.execute(player, PlayerBehavior::SelectReward { reward_id: candidates[1] }).unwrap();
    core = u04_continue(&core, player);
    assert_eq!(core.state.active_node_content.as_ref().unwrap().as_reward().unwrap().selected_reward_uuid, Some(candidates[1]));
    core.execute(player, PlayerBehavior::ClaimReward).unwrap();
    assert_eq!(core.state.enkephalin.amount, amount + 18);
    assert_eq!(core.state.roster.get(&employee).unwrap().experience, experience + 31);
    for (id, experience) in other_experience {
        assert_eq!(core.state.roster.get(&id).unwrap().experience, experience);
    }
    core = u04_continue(&core, player);
    assert!(matches!(core.get_state(), GameState::InRewardClaimed { .. }));
    assert!(core.execute(player, PlayerBehavior::ClaimReward).is_err());
    assert!(core.execute(player, PlayerBehavior::SelectReward { reward_id: candidates[0] }).is_err());
    core.execute(player, PlayerBehavior::ExitReward).unwrap();
    core = u04_continue(&core, player);
    assert_eq!(core.get_state(), GameState::ViewingMap);
    assert_eq!(core.state.enkephalin.amount, amount + 18);
    assert!(core.state.combat_reward_continuation.is_none());
}

#[test]
fn u04_choose_one_zero_candidates_has_explicit_empty_result_and_no_claim() {
    let (mut core, player, _, _) = u04_choose_one_fixture(0);
    let summary = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(summary["no_reward_candidates"], true);
    assert_eq!(summary["rewards"], json!([]));
    assert!(core.state.combat_reward_continuation.is_none());
    core = u04_continue(&core, player);
    assert_eq!(core.get_selected_event_snapshot_json().unwrap().unwrap(), summary);
    core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    assert_eq!(core.get_state(), GameState::ViewingMap);
    assert!(core.execute(player, PlayerBehavior::ClaimReward).is_err());
}

#[test]
fn u04_pending_continue_rejects_tampered_entitlement_and_missing_option_fields_atomically() {
    let (core, player, _, _) = u04_choose_one_fixture(2);
    let export = core.export_active_run_checkpoint().unwrap().unwrap();
    let document: serde_json::Value = serde_json::from_slice(&export.core_payload_json).unwrap();
    for field in ["savepoint", "recovery_restriction", "result", "reward", "reward_continuation"] {
        let mut bad = document.clone();
        bad["continuation"].as_object_mut().unwrap().remove(field);
        refresh_active_run_checksum(&mut bad);
        let mut restarted = core_with_profile_for_continue(&core, 1);
        assert!(restarted.continue_from_active_run_checkpoint(player, &serde_json::to_vec(&bad).unwrap(), 0, 0).is_err(), "missing {field}");
        assert!(restarted.state.run.is_none());
    }
    for case in 0..5 {
        let mut bad = document.clone();
        let continuation = &mut bad["continuation"];
        match case {
            0 => continuation["reward"]["rewards"][0]["effects"][0]["GrantEnkephalin"]["amount"] = serde_json::json!(99999),
            1 => continuation["reward"]["rewards"][0]["uuid"] = serde_json::json!(Uuid::nil()),
            2 => continuation["reward"]["selected_reward_uuid"] = serde_json::json!(Uuid::nil()),
            3 => continuation["reward_continuation"]["context"]["combat_participant_employee_ids"] = serde_json::json!([Uuid::nil()]),
            _ => continuation["reward_continuation"]["battle_uuid"] = serde_json::json!(Uuid::nil()),
        }
        refresh_active_run_checksum(&mut bad);
        let mut restarted = core_with_profile_for_continue(&core, 1);
        for _ in 0..2 {
            let error = restarted.continue_from_active_run_checkpoint(player, &serde_json::to_vec(&bad).unwrap(), 0, 0).unwrap_err();
            assert!(matches!(error, GameError::InvalidStaticData(_)), "case {case}: {error:?}");
            assert!(restarted.state.run.is_none());
            assert!(restarted.state.player.is_none());
            assert!(restarted.export_active_run_checkpoint().unwrap().is_none());
        }
    }
}

#[test]
fn u04_each_unfinished_interactive_node_restores_its_preentry_roots() {
    let cases = [
        (MapNodeCategory::SavePoint, "save_point", MapNodePayload::SavePoint),
        (MapNodeCategory::Maintenance, "maintenance", MapNodePayload::Maintenance),
        (MapNodeCategory::HeadquartersContact, "headquarters_contact", MapNodePayload::HeadquartersContact { shop_pool_id: Some("headquarters_basic_supplies".into()), candidate_count: 3 }),
        (MapNodeCategory::Shop, "shop_general", MapNodePayload::Shop { shop_id: Some("map_shop".into()), shop_pool_id: None }),
        (MapNodeCategory::Reward, "reward_treasure", MapNodePayload::Reward { reward_pool_id: Some("default_treasures".into()) }),
    ];
    let player = Uuid::from_u128(1);
    for (category, kind, payload) in cases {
        let mut core = GameCore::new(game_data_with_map_content(), 0xC004_0200);
        start_new_game_with_default_starters(&mut core, player);
        core.state.enkephalin.amount = 1000;
        let node = force_first_available_node(&mut core, category, kind, payload);
        core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
        let expected = u04_continue(&core, player);
        core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
        if category == MapNodeCategory::Shop {
            let offer_id = core.state.active_node_content.as_ref().unwrap().as_shop().unwrap().offers[0].offer_id;
            core.execute(player, PlayerBehavior::PurchaseItem { offer_id }).unwrap();
            assert!(core.state.enkephalin.amount < expected.state.enkephalin.amount);
        } else if category == MapNodeCategory::Reward {
            core.execute(player, PlayerBehavior::ClaimReward).unwrap();
            assert!(core.state.enkephalin.amount > expected.state.enkephalin.amount);
        }
        let restored = u04_continue(&core, player);
        let expected_snapshot=expected.get_run_snapshot_json().unwrap();
        assert_eq!(restored.get_run_snapshot_json().unwrap(), expected_snapshot, "{category:?}: unfinished activity restores effects and its pre-entry cost");
        assert_eq!(json!(restored.state.inventory), json!(expected.state.inventory), "{category:?}");
        assert_eq!(json!(restored.state.uuid_manager), json!(expected.state.uuid_manager), "{category:?}");
        assert!(restored.state.run_checkpoint.payload.is_none(), "entering SavePoint alone cannot create one");
        assert_eq!(restored.state.run_checkpoint.loads_used, 0);
    }
}

#[test]
fn u04_unfinished_event_combat_restores_before_event_instead_of_after_choice() {
    let event = combat_choice_event_definition("event_started_victory", "fight", "defense_encounter", "defense_risk_abno");
    let data = game_data_with_event_definitions_for_combat_tests(game_data_with_pve_encounters(), vec![event]);
    let mut core = GameCore::new(data, 123);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let node = force_single_event_node_run_for_combat_tests(&mut core, "event_started_victory");
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
    let expected = u04_continue(&core, player);
    // The helper selects and enters through the ordinary Event command path.
    core.execute(player, PlayerBehavior::CancelSelectedNode).unwrap();
    enter_event_combat_choice(&mut core, player, node, "event_started_victory", "fight");
    assert!(core.state.active_battle.is_some());
    let restored = u04_continue(&core, player);
    let expected_snapshot=expected.get_run_snapshot_json().unwrap();
    assert_eq!(restored.get_run_snapshot_json().unwrap(), expected_snapshot);
    assert_eq!(json!(restored.state.uuid_manager), json!(expected.state.uuid_manager));
    assert!(restored.run_state().unwrap().event_sessions.is_empty());
}

#[test]
fn u04_continue_retains_active_visual_novel_and_queue_at_a_stable_boundary() {
    use crate::game::visual_novel::{VisualNovelSession, VisualNovelSessionPresentationMode, VisualNovelSource};
    let mut core = GameCore::new(game_data_with_map_content(), 0xC004_0202);
    let player = Uuid::from_u128(1);
    start_new_game_with_default_starters(&mut core, player);
    let session = |id: &str| VisualNovelSession {
        session_id: id.into(), source: VisualNovelSource::Map,
        vn_id: VisualNovelId::new("hq_briefing_standard_intro"),
        current_scene_id: VisualNovelSceneId::new("start"), committed_choice_id: None,
        blocking: false, presentation_mode: VisualNovelSessionPresentationMode::NonBlocking,
    };
    core.state.visual_novel.active = Some(session("u04-active"));
    core.state.visual_novel.queue = vec![session("u04-queued")];
    core.refresh_continue_after_operation(super::super::application::ApplicationRoute::Behavior(ActionKind::CompleteNode)).unwrap();
    let mut restored = u04_continue(&core, player);
    assert_eq!(restored.state.visual_novel, core.state.visual_novel);
    restored.execute(player, PlayerBehavior::AdvanceVisualNovel { session_id: "u04-active".into(), current_scene_id: VisualNovelSceneId::new("start") }).unwrap();
    let expected = restored.state.visual_novel.clone();
    assert_eq!(expected.active.as_ref().unwrap().session_id, "u04-queued");
    restored = u04_continue(&restored, player);
    assert_eq!(restored.state.visual_novel, expected);
}

#[test]
fn u04_terminal_choice_survives_ack_and_claim_restart_then_exit_invalidates_continue() {
    let (mut core, player, _, candidates) = u04_choose_one_fixture_with_terminal(2, true);
    assert_eq!(core.state.confirmed_combat_result.as_ref().unwrap().continuation_phase, GameState::RunComplete);
    let profile = u04_terminal_profile_fixture();
    assert_eq!(core.state.player_profile, profile);
    let ownership = u04_terminal_active_ownership(&core);
    let amount = core.state.enkephalin.amount;
    core = u04_continue(&core, player);
    assert_eq!(u04_terminal_active_ownership(&core), ownership);
    assert_eq!(core.state.player_profile, profile);
    core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    assert!(matches!(core.get_state(), GameState::InReward { .. }));
    assert_eq!(u04_terminal_active_ownership(&core), ownership, "ACK preserves actors while a reward is pending");
    core = u04_continue(&core, player);
    assert_eq!(u04_terminal_active_ownership(&core), ownership);
    assert_eq!(core.state.player_profile, profile);
    core.execute(player, PlayerBehavior::SelectReward { reward_id: candidates[1] }).unwrap();
    core.execute(player, PlayerBehavior::ClaimReward).unwrap();
    assert!(matches!(core.get_state(), GameState::InRewardClaimed { .. }));
    assert_eq!(u04_terminal_active_ownership(&core), ownership, "Claim keeps employee and copy ownership until Exit");
    assert_eq!(core.state.player_profile, profile);
    core = u04_continue(&core, player);
    assert_eq!(u04_terminal_active_ownership(&core), ownership);
    assert_eq!(core.state.player_profile, profile);
    assert_eq!(core.state.enkephalin.amount, amount + 18);
    let before_retry = u04_terminal_roots(&core);
    assert!(core.execute(player, PlayerBehavior::ClaimReward).is_err());
    assert_eq!(u04_terminal_roots(&core), before_retry);
    core.execute(player, PlayerBehavior::ExitReward).unwrap();
    u04_assert_terminal_assets_cleared(&core, &profile);
    let terminal = u04_terminal_roots(&core);
    for behavior in [PlayerBehavior::ExitReward, PlayerBehavior::CompleteCombatResult,
        PlayerBehavior::ClaimReward] {
        assert!(core.execute(player, behavior).is_err());
        assert_eq!(u04_terminal_roots(&core), terminal, "terminal retries cannot re-grant or re-settle");
    }
}

fn u04_terminal_active_ownership(core: &GameCore) -> Value {
    assert_eq!(core.state.roster.len(), 3, "terminal battle actors remain owned until final acknowledgement");
    let run = core.state.run.as_ref().unwrap();
    run.recruitment.validate_exact_ownership(&core.game_data.employee_definition_data.definitions,
        &core.state.roster).expect("pending reward cannot sever recruitment/roster ownership");
    let employees = core.state.roster.iter().map(|employee| employee.uuid)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(core.state.inventory.equipments.iter().next().is_some());
    for copy in core.state.inventory.equipments.iter() {
        if let Some(owner) = copy.equipped_to { assert!(employees.contains(&owner)); }
    }
    json!({"employees": employees, "inventory": core.state.inventory,
        "fragments": core.state.skill_fragments, "recruitment": run.recruitment})
}

fn u04_terminal_roots(core: &GameCore) -> Value {
    json!({"runtime": non_battle_state_without_profile_fingerprint(core),
        "profile": core.state.player_profile,
        "continue": core.export_active_run_checkpoint().unwrap().map(|checkpoint| checkpoint.core_payload_json)})
}

fn u04_assert_terminal_assets_cleared(
    core: &GameCore,
    profile: &crate::game::encounter_compendium::PlayerProfileState,
) {
    assert_eq!(core.get_state(), GameState::RunComplete);
    assert_eq!(&core.state.player_profile, profile, "Tutorial termination preserves the exact permanent profile");
    assert_eq!(core.state.roster.len(), 0);
    assert_eq!(serde_json::to_value(&core.state.inventory).unwrap(),
        serde_json::to_value(crate::game::resources::inventory::Inventory::default()).unwrap());
    assert!(core.state.skill_fragments.copies().next().is_none());
    assert!(core.state.relics.is_empty());
    assert_eq!(core.state.crafting_resources, Default::default());
    assert!(core.state.run_checkpoint.payload.is_none());
    assert_eq!(core.state.run_checkpoint.loads_used, 0);
    assert!(core.state.confirmed_combat_result.is_none());
    assert!(core.state.combat_reward_continuation.is_none());
    assert!(core.state.active_battle.is_none());
    assert!(core.state.active_node_content.is_none());
    assert!(core.state.node_session.is_none());
    assert!(core.export_active_run_checkpoint().unwrap().is_none());
}

#[test]
fn u04_terminal_zero_candidates_keeps_actors_until_ack_then_clears_without_repeat() {
    let (mut core, player, _, candidates) = u04_choose_one_fixture_with_terminal(0, true);
    assert!(candidates.is_empty());
    let profile = u04_terminal_profile_fixture();
    assert_eq!(core.state.player_profile, profile);
    assert_eq!(core.state.confirmed_combat_result.as_ref().unwrap().continuation_phase, GameState::RunComplete);
    assert!(core.state.combat_reward_continuation.is_none());
    let summary = core.get_selected_event_snapshot_json().unwrap().unwrap();
    assert_eq!(summary["no_reward_candidates"], true);
    assert_eq!(summary["rewards"], json!([]));
    let ownership = u04_terminal_active_ownership(&core);
    // Terminal results without a pending reward are already non-resumable;
    // runtime actors remain owned until the result is acknowledged.
    assert!(core.export_active_run_checkpoint().unwrap().is_none());
    let before_claim = u04_terminal_roots(&core);
    assert!(core.execute(player, PlayerBehavior::ClaimReward).is_err());
    assert_eq!(u04_terminal_roots(&core), before_claim);
    assert_eq!(u04_terminal_active_ownership(&core), ownership);
    let acknowledgement = core.execute(player, PlayerBehavior::CompleteCombatResult).unwrap();
    let BehaviorResult::CombatRewardsGranted { completion, .. } = acknowledgement else {
        panic!("terminal victory acknowledges its combat rewards");
    };
    assert!(matches!(*completion, BehaviorResult::RunComplete { .. }));
    u04_assert_terminal_assets_cleared(&core, &profile);
    let terminal = u04_terminal_roots(&core);
    for behavior in [PlayerBehavior::CompleteCombatResult, PlayerBehavior::ClaimReward,
        PlayerBehavior::ExitReward] {
        assert!(core.execute(player, behavior).is_err());
        assert_eq!(u04_terminal_roots(&core), terminal);
    }
}

#[test]
fn u04_confirmed_receipt_rejects_phase_outcome_ack_and_entitlement_inconsistency() {
    let (core, player, _, candidates) = u04_choose_one_fixture(2);
    let export = core.export_active_run_checkpoint().unwrap().unwrap();
    let document: serde_json::Value = serde_json::from_slice(&export.core_payload_json).unwrap();
    for case in 0..7 {
        let mut bad = document.clone();
        let payload = &mut bad["continuation"];
        match case {
            0 => payload["result"]["acknowledgement"] = json!(BehaviorResult::RunAbandoned),
            1 => payload["result"]["summary"]["winner"] = json!(crate::game::battle::types::BattleWinner::Opponent),
            2 => payload["result"]["summary"]["no_reward_candidates"] = json!(true),
            3 => payload["result"]["summary"]["rewards"][0]["uuid"] = json!(Uuid::nil()),
            4 => payload["reward"]["selected_reward_uuid"] = json!(candidates[0]),
            5 => { payload["reward"] = json!(null); payload["reward_continuation"] = json!(null); },
            _ => { payload["result"]["continuation_phase"] = json!(GameState::RunComplete); payload["reward_continuation"]["return_phase"] = json!(GameState::RunComplete); },
        }
        refresh_active_run_checksum(&mut bad);
        let mut restarted = core_with_profile_for_continue(&core, 1);
        assert!(restarted.continue_from_active_run_checkpoint(player, &serde_json::to_vec(&bad).unwrap(), 0, 0).is_err(), "case {case}");
        assert!(restarted.state.run.is_none(), "case {case}");
    }
}

#[test]
fn u04_savepoint_load_rewinds_retreats_while_continue_preserves_load_budget_and_profile() {
    let mut core = GameCore::new(live_game_data_from_ron(), 0xC004_0400);
    let player = Uuid::from_u128(1);
    start_new_game_with_legacy_sword_fixture(&mut core, player);
    let node = force_map_combat_node(&mut core, MapNodeCategory::Combat, "combat_monster", "suppress_burrowing_heaven");
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
    core.save_run_checkpoint_for_test().unwrap();
    core.state.player_profile.highest_floor_reached = 25;
    for expected_loads in 1..=3 {
        if matches!(core.get_state(), GameState::ViewingMap) {
            core.execute(player, PlayerBehavior::SelectMapNode { node_id: node }).unwrap();
        }
        for _ in 0..2 {
            core.execute(player, PlayerBehavior::ConfirmEnterNode).unwrap();
            execute_current_battle_mutation(&mut core, player, PlayerBehavior::RetreatBattle).unwrap();
        }
        assert_eq!(core.run_state().unwrap().abnormality_attempt_state(node).retreats_used, 2);
        let profile = core.state.player_profile.clone();
        core = u04_continue(&core, player);
        core.state.player_profile.merge(profile);
        assert_eq!(core.run_state().unwrap().abnormality_attempt_state(node).retreats_used, 2);
        assert_eq!(core.state.run_checkpoint.loads_used, expected_loads - 1);
        core.execute(player, PlayerBehavior::LoadRunCheckpoint).unwrap();
        assert_eq!(core.run_state().unwrap().abnormality_attempt_state(node).retreats_used, 0);
        assert_eq!(core.state.run_checkpoint.loads_used, expected_loads);
        assert_eq!(core.state.player_profile.highest_floor_reached, 25);
        let profile = core.state.player_profile.clone();
        core = u04_continue(&core, player);
        core.state.player_profile.merge(profile);
        assert_eq!(core.state.run_checkpoint.loads_used, expected_loads);
    }
    let before = core.get_run_snapshot_json().unwrap();
    assert!(core.execute(player, PlayerBehavior::LoadRunCheckpoint).is_err());
    assert_eq!(core.get_run_snapshot_json().unwrap(), before);
}


#[test]
fn u04_shared_savepoint_encoding_restores_independent_rewind_and_rejects_bad_references() {
    let player = Uuid::from_u128(1);
    let mut core = GameCore::new(live_game_data_from_ron(), 0xC004_0500);
    start_new_game_with_legacy_sword_fixture(&mut core, player);
    core.save_run_checkpoint_for_test().unwrap();
    let export = core.export_active_run_checkpoint().unwrap().unwrap();
    let document: Value = serde_json::from_slice(&export.core_payload_json).unwrap();
    assert_eq!(document["continuation"]["savepoint"], json!({"current":{}}));
    let original_amount = core.state.enkephalin.amount;
    for invalid_savepoint in [
        json!({"current":{}, "snapshot": {"checkpoint":{}}}),
        json!({"current":{"extra": true}}),
        json!({"missing":{}}),
        json!({"snapshot":{}}),
        json!({"snapshot":{"checkpoint":null}}),
    ] {
        let mut bad = document.clone();
        bad["continuation"]["savepoint"] = invalid_savepoint;
        refresh_active_run_checksum(&mut bad);
        let mut target = GameCore::new(core.game_data.clone(), 99);
        assert!(target.continue_from_active_run_checkpoint(player, &serde_json::to_vec(&bad).unwrap(), 0, 0).is_err());
        assert!(target.state.run.is_none());
    }
    core = u04_continue(&core, player);
    core.execute_admin_command(crate::game::world::admin::AdminCommand::AdminSetEnkephalin { amount: original_amount + 100 }).unwrap();
    let changed = core.export_active_run_checkpoint().unwrap().unwrap();
    let changed: Value = serde_json::from_slice(&changed.core_payload_json).unwrap();
    assert!(changed["continuation"]["savepoint"]["snapshot"]["checkpoint"].is_object());
    assert_eq!(core.state.run_checkpoint.payload.as_ref().unwrap().enkephalin.amount, original_amount);
    core = u04_continue(&core, player);
    assert_eq!(core.state.enkephalin.amount, original_amount + 100);
    core.execute(player, PlayerBehavior::LoadRunCheckpoint).unwrap();
    assert_eq!(core.state.enkephalin.amount, original_amount);
    assert_eq!(core.state.run_checkpoint.loads_used, 1);
}
