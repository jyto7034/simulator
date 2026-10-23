use super::*;

#[test]
fn u12_lifecycle_tutorial_preset_is_three_and_abandon_repeat_never_mutates_permanent_profile() {
    let mut app = CoreApplication::new(empty_game_data(), 12201);
    let player = Uuid::from_u128(12201);
    let profile_before = snapshot(&app, ApplicationSnapshotKind::PlayerProfile);
    for _ in 0..2 {
        issue(&mut app, player, json!({"type":"start_new_game","game_mode":"Tutorial","replace_active_run":false}));
        let running = finish_briefing(&mut app, player);
        assert_eq!(running["game_state"], "viewing_map");
        assert_eq!(running["roster"]["employees"].as_array().unwrap().len(), 3);
        assert_eq!(snapshot(&app, ApplicationSnapshotKind::PlayerProfile), profile_before);
        reject_unchanged(&mut app, player, json!({"type":"select_starter_employees",
            "employee_definition_ids":["candidate_0","candidate_2","candidate_4"],"loan_copy_ids":[]}));
        issue(&mut app, player, json!({"type":"abandon_run"}));
        assert!(app.export_active_run_checkpoint().unwrap().is_none());
        assert_eq!(snapshot(&app, ApplicationSnapshotKind::PlayerProfile), profile_before);
    }
    start_selected_public(&mut app, player);
    assert_eq!(snapshot(&app, ApplicationSnapshotKind::PlayerProfile)["lifecycle"]["expedition_sequence"], 1);
}

#[test]
fn u12_lifecycle_skip_tutorial_closes_blocking_briefing_and_is_invalid_in_expedition() {
    let mut app = CoreApplication::new(empty_game_data(), 12211);
    let player = Uuid::from_u128(12211);
    let profile_before = snapshot(&app, ApplicationSnapshotKind::PlayerProfile);
    issue(&mut app, player, json!({"type":"start_new_game","game_mode":"Tutorial","replace_active_run":false}));
    assert!(!snapshot(&app, ApplicationSnapshotKind::Run)["visual_novel"].is_null());
    issue(&mut app, player, json!({"type":"skip_tutorial"}));
    assert!(snapshot(&app, ApplicationSnapshotKind::Run)["visual_novel"].is_null());
    assert_eq!(snapshot(&app, ApplicationSnapshotKind::PlayerProfile), profile_before);
    start_selected_public(&mut app, player);
    reject_unchanged(&mut app, player, json!({"type":"skip_tutorial"}));
}

#[test]
fn u12_lifecycle_tutorial_continue_rejects_expedition_deterrence_receipt_and_pending_state() {
    let player = Uuid::from_u128(12221);
    let mut source = GameCore::new(empty_game_data(), 12221);
    start_new_game_with_mode_and_default_starters(&mut source, player, GameMode::Tutorial);
    let saved = source.export_active_run_checkpoint().unwrap().unwrap();
    let document: Value = serde_json::from_slice(&saved.core_payload_json).unwrap();
    let progression = &document["continuation"]["current"]["run_progression"];
    assert_eq!(progression["deterrence"], 0);
    assert_eq!(progression["entry_receipts"], json!([]));
    assert!(progression["pending_continuation"].is_null());
    let mut positive = core_with_profile_for_continue(&source, 999);
    positive.continue_from_active_run_checkpoint(player, &saved.core_payload_json,
        saved.loads_used, saved.run_peak_certified_power).unwrap();
    assert_eq!(positive.state.player_profile, source.state.player_profile);
    assert_eq!(positive.state.roster.len(), 3);
    let node = source.state.run.as_ref().unwrap().map_progression.available_node_ids[0];
    for (field, value) in [
        ("deterrence", json!(1)),
        ("entry_receipts", json!([node])),
        ("pending_continuation", json!({"node_id":node,"depth":1,"cause":"Exhaustion"})),
    ] {
        let mut bad = document.clone();
        bad["continuation"]["current"]["run_progression"][field] = value;
        refresh_active_run_checksum(&mut bad);
        let mut destination = core_with_profile_for_continue(&source, 998);
        let before = core_roots(&destination);
        let bytes = serde_json::to_vec(&bad).unwrap();
        assert_eq!(GameCore::inspect_checkpoint_requirements(&bytes, usize::MAX).unwrap(),
            *source.content_identity(), "tutorial {field} reaches semantic validation");
        for _ in 0..2 {
            assert!(destination.continue_from_active_run_checkpoint(player, &bytes,
                saved.loads_used, saved.run_peak_certified_power).is_err(), "tutorial {field}");
            assert_eq!(core_roots(&destination), before);
        }
    }
}
