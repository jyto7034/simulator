use super::*;

#[test]
fn u12_lifecycle_canonical_vault_preserves_distinct_copies_and_dormant_definitions() {
    let vault = ExpeditionVault { materials: Default::default(),
        equipment: BTreeMap::from([(Uuid::from_u128(12101), equipment(12101, "missing_dormant_weapon")),
            (Uuid::from_u128(12102), equipment(12102, "missing_dormant_weapon"))]),
        fragments: BTreeMap::from([(Uuid::from_u128(12103), fragment(12103, "missing_dormant_fragment"))]),
    };
    vault.validate().unwrap();
    let encoded = serde_json::to_vec(&vault).unwrap();
    let decoded: ExpeditionVault = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, vault);
    assert_eq!(decoded.equipment.len(), 2, "same definition is not same copy");
    assert_eq!(decoded.fragments[&Uuid::from_u128(12103)].selected_form, "awakened");
}

#[test]
fn u12_lifecycle_vault_key_embedded_cross_kind_and_form_corruption_reject() {
    let good = ExpeditionVault { materials: Default::default(),
        equipment: BTreeMap::from([(Uuid::from_u128(12111), equipment(12111, "weapon"))]),
        fragments: BTreeMap::from([(Uuid::from_u128(12112), fragment(12112, "fragment"))]),
    };
    good.validate().unwrap();
    for mutation in 0..7 {
        let mut bad = good.clone();
        match mutation {
            0 => bad.equipment.get_mut(&Uuid::from_u128(12111)).unwrap().copy_uuid = Uuid::nil(),
            1 => bad.equipment.get_mut(&Uuid::from_u128(12111)).unwrap().copy_uuid = Uuid::from_u128(12113),
            2 => { bad.fragments.insert(Uuid::from_u128(12111), fragment(12111, "fragment")); },
            3 => bad.equipment.get_mut(&Uuid::from_u128(12111)).unwrap().enhancement_level = 1,
            4 => bad.fragments.get_mut(&Uuid::from_u128(12112)).unwrap().selected_form = "not_unlocked".into(),
            5 => bad.fragments.get_mut(&Uuid::from_u128(12112)).unwrap().unlocked_forms.clear(),
            6 => bad.equipment.get_mut(&Uuid::from_u128(12111)).unwrap().definition_id = "alias / not canonical".into(),
            _ => unreachable!(),
        }
        assert!(bad.validate().is_err(), "independent corrupt ownership edge {mutation}");
    }
}

#[test]
fn u12_lifecycle_first_return_receipt_requires_expedition_and_unknown_fields_reject() {
    for receipt in [FirstReturnStory::Pending, FirstReturnStory::Consumed] {
        let lifecycle = ExpeditionLifecycle { first_return_story: receipt, ..Default::default() };
        assert!(lifecycle.validate().is_err());
        let lifecycle = ExpeditionLifecycle { expedition_sequence: 1, ..lifecycle };
        lifecycle.validate().unwrap();
    }
    let mut document = serde_json::to_value(ExpeditionLifecycle::default()).unwrap();
    document["standard_tutorial_completed"] = json!(true);
    assert!(serde_json::from_value::<ExpeditionLifecycle>(document).is_err());
}

#[test]
fn u12_lifecycle_raw_vault_decode_rejects_duplicate_uuid_keys_and_fragment_forms() {
    let id = Uuid::from_u128(12121);
    let equipment = serde_json::to_string(&equipment(12121, "dormant_weapon")).unwrap();
    let fragment = serde_json::to_string(&fragment(12121, "dormant_fragment")).unwrap();
    for (kind, copy, other) in [("equipment", equipment, "fragments"), ("fragments", fragment, "equipment")] {
        let positive = format!("{{\"{kind}\":{{\"{id}\":{copy}}},\"{other}\":{{}},\"materials\":{{}}}}");
        let valid: ExpeditionVault = serde_json::from_str(&positive).unwrap();
        valid.validate().unwrap();
        // Deliberately keep raw text: parsing through Value would erase the duplicate.
        let repeated = format!("{{\"{kind}\":{{\"{id}\":{copy},\"{id}\":{copy}}},\"{other}\":{{}},\"materials\":{{}}}}");
        assert!(serde_json::from_str::<ExpeditionVault>(&repeated).is_err(), "duplicate raw key in {kind}");
    }
    let fragment_json = serde_json::to_value(super::fragment(12122, "dormant_fragment")).unwrap();
    let mut duplicate_forms = fragment_json.clone();
    duplicate_forms["unlocked_forms"] = json!(["base", "base", "awakened"]);
    assert!(serde_json::from_value::<VaultFragment>(duplicate_forms).is_err());
    let mut excessive_forms = fragment_json;
    excessive_forms["unlocked_forms"] = json!((0..17).map(|n| format!("form_{n}")).collect::<Vec<_>>());
    excessive_forms["selected_form"] = json!("form_0");
    assert!(serde_json::from_value::<VaultFragment>(excessive_forms).is_err());
}

#[test]
fn u12_lifecycle_knowledge_merge_preserves_exact_ownership_and_refuses_asset_import() {
    use crate::game::encounter_compendium::PlayerProfileState;
    let mut profile = PlayerProfileState::default();
    profile.lifecycle.expedition_sequence = 3;
    profile.lifecycle.vault.equipment.insert(Uuid::from_u128(12131), equipment(12131, "dormant_weapon"));
    let ownership = profile.lifecycle.clone();
    let mut knowledge = profile.clone();
    knowledge.highest_floor_reached = 7;
    knowledge.encounter_compendium.registered_ids.insert("dormant_abnormality".into());
    profile.merge(knowledge).unwrap();
    assert_eq!(profile.lifecycle, ownership);
    assert_eq!(profile.highest_floor_reached, 7);
    assert!(profile.encounter_compendium.registered_ids.contains("dormant_abnormality"));
    for changed in 0..3 {
        let before = profile.clone();
        let mut incoming = profile.clone();
        match changed {
            0 => incoming.lifecycle.vault.equipment.clear(),
            1 => { incoming.lifecycle.vault.equipment.insert(Uuid::from_u128(12132), equipment(12132, "weapon")); },
            2 => incoming.lifecycle.expedition_sequence += 1,
            _ => unreachable!(),
        }
        assert!(profile.merge(incoming).is_err());
        assert_eq!(profile, before, "knowledge import must not union or overwrite exact assets");
    }
}

fn saved_expedition() -> (GameCore, Uuid) {
    let player = Uuid::from_u128(12141);
    let mut core = GameCore::new(empty_game_data(), 12141);
    // The profile has a previous returned expedition whose story awaits ack.
    core.state.player_profile.lifecycle.expedition_sequence = 1;
    core.state.player_profile.lifecycle.first_return_story = FirstReturnStory::Pending;
    start_new_game_with_default_starters(&mut core, player);
    let savepoint = force_first_available_node(&mut core, MapNodeCategory::SavePoint, "save_point", MapNodePayload::SavePoint);
    select_and_confirm_map_node(&mut core, player, savepoint);
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    (core, player)
}

#[test]
fn u12_lifecycle_fresh_departure_continue_preserves_canonical_bytes_and_next_recruit_identity() {
    // Reproduce the public departure that exposed the absent-VN-stream bug.
    // No test fixture writes UUID counters or captures a synthetic SavePoint.
    let player = Uuid::from_u128(12143);
    let mut source = GameCore::new(legacy_pool_game_data_from_ron(), 12143);
    start_new_game_with_default_starters(&mut source, player);
    assert!(matches!(source.get_state(), GameState::ViewingMap));
    assert_eq!(source.state.roster.len(), 3);
    assert!(source.state.run_checkpoint.payload.is_none());
    let saved = source.export_active_run_checkpoint().unwrap().unwrap();
    let document: Value = serde_json::from_slice(&saved.core_payload_json).unwrap();
    assert!(document["continuation"]["current"]["uuid_manager"]["counters"]
        .as_object().unwrap().get("6220401612985551955").is_none(),
        "departure naturally has no post-instance visual-novel stream");
    let mut restored = core_with_profile_for_continue(&source, 987654);
    restored.continue_from_active_run_checkpoint(player, &saved.core_payload_json,
        saved.loads_used, saved.run_peak_certified_power).unwrap();
    let recaptured = restored.export_active_run_checkpoint().unwrap().unwrap();
    assert_eq!(recaptured.core_payload_json, saved.core_payload_json,
        "fresh Continue must preserve the exact canonical writer bytes");
    assert_eq!(recaptured.loads_used, saved.loads_used);
    assert_eq!(recaptured.run_peak_certified_power, saved.run_peak_certified_power);
    assert_eq!(core_roots(&restored), core_roots(&source));

    let headquarters = source.selectable_map_node_ids().into_iter().find(|id|
        source.state.run.as_ref().unwrap().map.node(*id).unwrap().category
            == MapNodeCategory::HeadquartersContact)
        .expect("legacy pool development departure exposes immediate HQ");
    let employees_before: BTreeSet<_> = source.state.roster.iter().map(|employee| employee.uuid).collect();
    for behavior in [PlayerBehavior::SelectMapNode { node_id: headquarters }, PlayerBehavior::ConfirmEnterNode] {
        let expected = serde_json::to_value(source.execute(player, behavior.clone()).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(restored.execute(player, behavior).unwrap()).unwrap(), expected);
        assert_eq!(core_roots(&restored), core_roots(&source));
    }
    let BehaviorResult::HeadquartersContactState { recruitment_candidates, .. } =
        source.headquarters_contact_state_result().unwrap() else { panic!("HQ candidate projection"); };
    let employee_definition_id = recruitment_candidates.first().expect("actual hire candidate")
        .employee_definition_id.clone();
    let expected = serde_json::to_value(source.execute(player, PlayerBehavior::RecruitEmployee {
        employee_definition_id: employee_definition_id.clone() }).unwrap()).unwrap();
    assert_eq!(serde_json::to_value(restored.execute(player, PlayerBehavior::RecruitEmployee {
        employee_definition_id }).unwrap()).unwrap(), expected);
    let hired = source.state.roster.iter().filter(|employee| !employees_before.contains(&employee.uuid))
        .collect::<Vec<_>>();
    assert_eq!(hired.len(), 1);
    assert!(hired[0].is_battle_deployable());
    assert_eq!(core_roots(&restored), core_roots(&source),
        "the next legal allocation preserves employee, equipment and fragment identities");
    let expected = source.export_active_run_checkpoint().unwrap().unwrap();
    let actual = restored.export_active_run_checkpoint().unwrap().unwrap();
    assert_eq!(actual.core_payload_json, expected.core_payload_json);
    assert_eq!(actual.loads_used, expected.loads_used);
    assert_eq!(actual.run_peak_certified_power, expected.run_peak_certified_power);
}

#[test]
fn u12_lifecycle_current_continue_requires_matching_profile_and_preserves_copy_bindings() {
    let (source, player) = saved_expedition();
    let export = source.export_active_run_checkpoint().unwrap().unwrap();
    let mut positive = GameCore::new(source.game_data.clone(), 999);
    positive.state.player_profile = source.state.player_profile.clone();
    positive.continue_from_active_run_checkpoint(player, &export.core_payload_json,
        export.loads_used, export.run_peak_certified_power).unwrap();
    assert_eq!(serde_json::to_value(&positive.state.roster).unwrap(), serde_json::to_value(&source.state.roster).unwrap());
    assert_eq!(serde_json::to_value(&positive.state.inventory).unwrap(), serde_json::to_value(&source.state.inventory).unwrap());
    assert_eq!(serde_json::to_value(&positive.state.skill_fragments).unwrap(), serde_json::to_value(&source.state.skill_fragments).unwrap());
    assert_eq!(positive.state.player_profile, source.state.player_profile);
    for wrong in 0..4 {
        let mut destination = GameCore::new(source.game_data.clone(), 999);
        destination.state.player_profile = source.state.player_profile.clone();
        match wrong {
            0 => destination.state.player_profile.lifecycle.active_expedition = None,
            1 => {
                destination.state.player_profile.lifecycle.expedition_sequence += 1;
                destination.state.player_profile.lifecycle.active_expedition = Some(ExpeditionInstance::new(player, 3).unwrap());
            }
            2 => {
                destination.state.player_profile.lifecycle.active_expedition = Some(ExpeditionInstance::new(Uuid::from_u128(12142), 2).unwrap());
            }
            3 => {
                let held = source.state.inventory.equipments.iter().next().unwrap();
                destination.state.player_profile.lifecycle.vault.equipment.insert(held.instance_uuid,
                    equipment(held.instance_uuid.as_u128(), &held.meta.id));
            }
            _ => unreachable!(),
        }
        let before = core_roots(&destination);
        assert!(destination.continue_from_active_run_checkpoint(player, &export.core_payload_json,
            export.loads_used, export.run_peak_certified_power).is_err(), "wrong pair {wrong}");
        assert_eq!(core_roots(&destination), before);
    }
}

#[test]
fn u12_lifecycle_same_instance_savepoint_accepts_newer_knowledge_and_story_ack_without_rollback() {
    let (mut core, player) = saved_expedition();
    core.validate_run_checkpoint_payload(core.state.run_checkpoint.payload.as_ref().unwrap())
        .expect("the writer's SavePoint must be loadable before permanent knowledge changes");
    let identity = core.state.player_profile.lifecycle.active_expedition;
    let saved_people = serde_json::to_value(&core.state.roster).unwrap();
    core.state.player_profile.highest_floor_reached = 99;
    core.state.player_profile.encounter_compendium.registered_ids.insert("newer_permanent_knowledge".into());
    assert_eq!(core.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Pending);
    core.execute(player, PlayerBehavior::AcknowledgeFirstReturnStory).unwrap();
    assert_eq!(core.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Consumed);
    core.state.enkephalin.amount = 17;
    let current_profile = core.state.player_profile.clone();
    core.execute(player, PlayerBehavior::LoadRunCheckpoint).unwrap();
    assert_eq!(core.state.player_profile, current_profile);
    assert_eq!(core.state.player_profile.lifecycle.active_expedition, identity);
    assert_eq!(serde_json::to_value(&core.state.roster).unwrap(), saved_people);
    assert_eq!(core.state.enkephalin.amount, 500);
    assert_eq!(core.state.run_checkpoint.loads_used, 1);
    let after_load = core_roots(&core);
    core.execute(player, PlayerBehavior::AcknowledgeFirstReturnStory).unwrap();
    assert_eq!(core_roots(&core), after_load, "ack after rewind remains idempotent");
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = core_with_profile_for_continue(&core, 12144);
    restored.continue_from_active_run_checkpoint(player, &saved.core_payload_json,
        saved.loads_used, saved.run_peak_certified_power).unwrap();
    assert_eq!(restored.state.player_profile, current_profile);
    assert_eq!(restored.state.run_checkpoint.loads_used, 1);
    let before_retry = core_roots(&restored);
    restored.execute(player, PlayerBehavior::AcknowledgeFirstReturnStory).unwrap();
    assert_eq!(core_roots(&restored), before_retry);
}

#[test]
fn u12_lifecycle_never_story_ack_rejects_and_terminated_other_instance_checkpoints_stay_stale() {
    let player = Uuid::from_u128(12145);
    let mut never = GameCore::new(empty_game_data(), 12145);
    let blank = core_roots(&never);
    assert!(never.execute(player, PlayerBehavior::AcknowledgeFirstReturnStory).is_err());
    assert_eq!(core_roots(&never), blank);
    start_new_game_with_default_starters(&mut never, player);
    assert_eq!(never.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Never);
    let active = core_roots(&never);
    assert!(never.execute(player, PlayerBehavior::AcknowledgeFirstReturnStory).is_err());
    assert_eq!(core_roots(&never), active);

    let (mut source, player) = saved_expedition();
    let old = source.export_active_run_checkpoint().unwrap().unwrap();
    let old_instance = source.state.player_profile.lifecycle.active_expedition;
    let mut positive = core_with_profile_for_continue(&source, 12146);
    positive.continue_from_active_run_checkpoint(player, &old.core_payload_json,
        old.loads_used, old.run_peak_certified_power).unwrap();
    source.execute(player, PlayerBehavior::AcknowledgeFirstReturnStory).unwrap();
    source.execute(player, PlayerBehavior::AbandonRun).unwrap();
    assert!(source.state.player_profile.lifecycle.active_expedition.is_none());
    assert!(source.export_active_run_checkpoint().unwrap().is_none());
    assert_eq!(source.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Consumed);
    let terminal = core_roots(&source);
    source.execute(player, PlayerBehavior::AcknowledgeFirstReturnStory).unwrap();
    assert_eq!(core_roots(&source), terminal);
    let mut stale = core_with_profile_for_continue(&source, 12147);
    let before = core_roots(&stale);
    assert!(stale.continue_from_active_run_checkpoint(player, &old.core_payload_json,
        old.loads_used, old.run_peak_certified_power).is_err());
    assert_eq!(core_roots(&stale), before, "terminal profile cannot resurrect the old run");

    start_new_game_with_default_starters(&mut source, player);
    assert_ne!(source.state.player_profile.lifecycle.active_expedition, old_instance);
    assert_eq!(source.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Consumed);
    let current = source.export_active_run_checkpoint().unwrap().unwrap();
    let mut current_positive = core_with_profile_for_continue(&source, 12148);
    current_positive.continue_from_active_run_checkpoint(player, &current.core_payload_json,
        current.loads_used, current.run_peak_certified_power).unwrap();
    let mut other_instance = core_with_profile_for_continue(&source, 12149);
    let before = core_roots(&other_instance);
    assert!(other_instance.continue_from_active_run_checkpoint(player, &old.core_payload_json,
        old.loads_used, old.run_peak_certified_power).is_err());
    assert_eq!(core_roots(&other_instance), before, "new run accepts only its own checkpoint");
    let before_retry = core_roots(&current_positive);
    current_positive.execute(player, PlayerBehavior::AcknowledgeFirstReturnStory).unwrap();
    assert_eq!(core_roots(&current_positive), before_retry);
}

#[test]
fn u12_lifecycle_old_continue_eleven_and_savepoint_twelve_reject_after_current_positive() {
    let (source, player) = saved_expedition();
    let export = source.export_active_run_checkpoint().unwrap().unwrap();
    let current: Value = serde_json::from_slice(&export.core_payload_json).unwrap();
    assert_eq!(current["version"], 18);
    let mut destination = GameCore::new(source.game_data.clone(), 123);
    destination.state.player_profile = source.state.player_profile.clone();
    destination.continue_from_active_run_checkpoint(player, &export.core_payload_json,
        export.loads_used, export.run_peak_certified_power).unwrap();
    let mut old_continue = current.clone(); old_continue["version"] = json!(11);
    refresh_active_run_checksum(&mut old_continue);
    let mut old_savepoint = json!({"version":12,"checkpoint":current["continuation"]["current"],"checkpoint_sha256":""});
    refresh_active_run_checksum(&mut old_savepoint);
    for legacy in [old_continue, old_savepoint] {
        let original = serde_json::to_vec(&legacy).unwrap();
        let mut blank = GameCore::new(source.game_data.clone(), 123);
        blank.state.player_profile = source.state.player_profile.clone();
        let before = core_roots(&blank);
        for _ in 0..2 {
            assert!(blank.continue_from_active_run_checkpoint(player, &original,
                export.loads_used, export.run_peak_certified_power).is_err());
            assert_eq!(core_roots(&blank), before);
            assert_eq!(serde_json::to_vec(&legacy).unwrap(), original);
        }
    }
}
