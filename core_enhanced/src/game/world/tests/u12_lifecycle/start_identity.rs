use super::*;

#[test]
fn u12_lifecycle_instance_sequence_is_monotonic_player_scoped_and_checked() {
    let a = Uuid::from_u128(12001);
    let b = Uuid::from_u128(12002);
    let first = ExpeditionLifecycle::default().next_instance(a).unwrap();
    assert_eq!(first.sequence, 1);
    assert_eq!(first.player_id, a);
    assert!(!first.uuid.is_nil());
    assert_eq!(first, ExpeditionLifecycle::default().next_instance(a).unwrap());
    assert_ne!(first.uuid, ExpeditionLifecycle::default().next_instance(b).unwrap().uuid);
    let mut lifecycle = ExpeditionLifecycle::default();
    lifecycle.expedition_sequence = first.sequence;
    lifecycle.active_expedition = Some(first);
    assert!(lifecycle.next_instance(a).is_err(), "live departure cannot be overwritten");
    lifecycle.active_expedition = None;
    let second = lifecycle.next_instance(a).unwrap();
    assert_eq!(second.sequence, 2);
    assert_ne!(first.uuid, second.uuid);
    lifecycle.expedition_sequence = u64::MAX;
    let before = lifecycle.clone();
    assert!(lifecycle.next_instance(a).is_err());
    assert_eq!(lifecycle, before, "overflow cannot wrap or mutate the profile");
    assert!(ExpeditionInstance::new(a, 0).is_err());
    assert!(ExpeditionInstance::new(Uuid::nil(), 1).is_err());
}

#[test]
fn u12_lifecycle_identity_tampering_and_inconsistent_active_sequence_reject() {
    let original = ExpeditionInstance::new(Uuid::from_u128(12011), 7).unwrap();
    for field in ["player_id", "sequence", "uuid"] {
        let mut value = serde_json::to_value(original).unwrap();
        value[field] = if field == "sequence" { json!(8) } else { json!(Uuid::from_u128(12012)) };
        let tampered: ExpeditionInstance = serde_json::from_value(value).unwrap();
        assert!(tampered.validate().is_err(), "single changed field {field}");
    }
    let mut lifecycle = ExpeditionLifecycle::default();
    lifecycle.active_expedition = Some(original);
    lifecycle.expedition_sequence = 6;
    assert!(lifecycle.validate().is_err());
    lifecycle.expedition_sequence = 7;
    lifecycle.validate().unwrap();
}

#[test]
fn u12_lifecycle_legacy_start_modes_and_hidden_replacement_reject_without_mutation() {
    let mut app = CoreApplication::new(empty_game_data(), 12021);
    let player = Uuid::from_u128(12021);
    for mode in ["Standard", "Endless", "standard", "endless", "Unknown"] {
        reject_unchanged(&mut app, player,
            json!({"type":"start_new_game","game_mode":mode,"replace_active_run":false}));
    }
    reject_unchanged(&mut app, player,
        json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":true}));
}

#[test]
fn u12_lifecycle_expedition_available_on_fresh_profile_and_active_start_rejected() {
    let mut app = CoreApplication::new(empty_game_data(), 12031);
    let player = Uuid::from_u128(12031);
    issue(&mut app, player,
        json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}));
    let state = snapshot(&app, ApplicationSnapshotKind::Run);
    assert_eq!(state["game_state"], "pre_run_briefing");
    assert!(snapshot(&app, ApplicationSnapshotKind::PlayerProfile)
        .get("standard_tutorial_completed").is_none());
    for replacement in [false, true] {
        reject_unchanged(&mut app, player,
            json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":replacement}));
    }
}

#[test]
fn u12_lifecycle_starter_command_requires_explicit_loan_list() {
    let ids = json!(["candidate_0", "candidate_2", "candidate_4"]);
    assert!(command(json!({"type":"select_starter_employees","employee_definition_ids":ids})).is_err());
    assert!(command(json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":[]})).is_ok());
    assert!(command(json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":null})).is_err());
}

#[test]
fn u12_lifecycle_starter_invalid_selections_and_missing_loans_preserve_every_root() {
    let mut app = CoreApplication::new(empty_game_data(), 12041);
    let player = Uuid::from_u128(12041);
    issue(&mut app, player, json!({"type":"start_new_game","game_mode":"Expedition","replace_active_run":false}));
    let selection = finish_briefing(&mut app, player);
    let ids: Vec<_> = selection["game_state_context"]["candidates"].as_array().unwrap()
        .iter().map(|candidate| candidate["employee_definition_id"].clone()).collect();
    for invalid in [json!([]), json!([ids[0]]), json!([ids[0],ids[1]]),
        json!([ids[0],ids[1],ids[2],ids[3]]), json!([ids[0],ids[0],ids[1]]),
        json!([ids[0],ids[1],"not_a_candidate"])] {
        reject_unchanged(&mut app, player, json!({"type":"select_starter_employees",
            "employee_definition_ids":invalid,"loan_copy_ids":[]}));
    }
    reject_unchanged(&mut app, player, json!({"type":"select_starter_employees",
        "employee_definition_ids":[ids[0],ids[1],ids[2]]}));
    let absent = Uuid::from_u128(12999);
    for loans in [json!([absent]), json!([absent,absent]),
        json!([absent, Uuid::from_u128(12998), Uuid::from_u128(12997)])] {
        reject_unchanged(&mut app, player, json!({"type":"select_starter_employees",
            "employee_definition_ids":[ids[0],ids[1],ids[2]],"loan_copy_ids":loans}));
    }
    issue(&mut app, player, json!({"type":"select_starter_employees",
        "employee_definition_ids":[ids[0],ids[1],ids[2]],"loan_copy_ids":[]}));
    assert_eq!(snapshot(&app, ApplicationSnapshotKind::Run)["roster"]["employees"].as_array().unwrap().len(), 3);
}

#[test]
fn u12_lifecycle_same_scenario_seed_preserves_map_but_new_departure_changes_people_and_copy_ids() {
    let mut app = CoreApplication::new(empty_game_data(), 12051);
    let player = Uuid::from_u128(12051);
    let first = start_selected_public(&mut app, player);
    let first_profile = snapshot(&app, ApplicationSnapshotKind::PlayerProfile);
    let first_inventory = snapshot(&app, ApplicationSnapshotKind::Inventory);
    issue(&mut app, player, json!({"type":"abandon_run"}));
    assert!(app.export_active_run_checkpoint().unwrap().is_none());
    let second = start_selected_public(&mut app, player);
    let second_profile = snapshot(&app, ApplicationSnapshotKind::PlayerProfile);
    assert_eq!(first["map"], second["map"], "scenario seed controls the map, independently of instance identity");
    let people = |run: &Value| run["roster"]["employees"].as_array().unwrap().iter()
        .map(|employee| employee["uuid"].as_str().unwrap().to_owned()).collect::<BTreeSet<_>>();
    assert!(people(&first).is_disjoint(&people(&second)));
    assert_eq!(first_profile["lifecycle"]["expedition_sequence"], 1);
    assert_eq!(second_profile["lifecycle"]["expedition_sequence"], 2);
    assert_ne!(first_profile["lifecycle"]["active_expedition"], second_profile["lifecycle"]["active_expedition"]);
    assert_ne!(first_inventory, snapshot(&app, ApplicationSnapshotKind::Inventory), "fresh loadout copies must have fresh identity");
    for employee in second["roster"]["employees"].as_array().unwrap() {
        assert!(employee.get("psychology").is_none());
        assert!(employee.get("profile").is_some());
        assert!(employee.get("condition").is_some());
        assert!(employee.get("eligibility").is_some());
        assert!(employee.get("role").is_some());
    }
}

#[test]
fn u12_lifecycle_title_shell_with_authoritative_active_marker_rejects_both_start_modes() {
    let mut active = CoreApplication::new(empty_game_data(), 12061);
    let player = Uuid::from_u128(12061);
    start_selected_public(&mut active, player);
    let original = all_snapshots(&active);
    let original_checkpoint = active.export_active_run_checkpoint().unwrap().unwrap();
    let profile = active.snapshot(ApplicationSnapshotKind::PlayerProfile).unwrap().read_model().clone();
    let mut title = CoreApplication::new(empty_game_data(), 12061);
    title.install_authoritative_player_profile(profile).unwrap();
    assert!(title.is_not_started());
    assert!(title.export_active_run_checkpoint().unwrap().is_none());
    for mode in ["Tutorial", "Expedition"] {
        for replace in [false, true] {
            reject_unchanged(&mut title, player,
                json!({"type":"start_new_game","game_mode":mode,"replace_active_run":replace}));
        }
    }
    assert_eq!(all_snapshots(&active), original);
    assert_eq!(active.export_active_run_checkpoint().unwrap().unwrap().core_payload_json(), original_checkpoint.core_payload_json());
    title.continue_from_active_run_checkpoint(player, &original_checkpoint).unwrap();
    assert_eq!(snapshot(&title, ApplicationSnapshotKind::PlayerProfile), snapshot(&active, ApplicationSnapshotKind::PlayerProfile));
}
