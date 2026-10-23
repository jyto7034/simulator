use super::*;

// Explicit depth fixture: this proves the owner boundary, not ordinary traversal.
fn arrive_at_gate(depth: u32) -> (GameCore, Uuid) {
    let player = Uuid::from_u128(12500 + u128::from(depth));
    let mut core = GameCore::new(empty_game_data(), 12500);
    core.state.player_profile.lifecycle.vault = ExpeditionVault { materials: Default::default(),
        equipment: BTreeMap::from([(Uuid::from_u128(12599), equipment(12599, "dormant_weapon"))]),
        fragments: BTreeMap::new(),
    };
    start_new_game_with_default_starters(&mut core, player);
    core.state.run.as_mut().unwrap().run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Expedition { floor_index: depth - 1 };
    let gate = force_first_available_node(&mut core, MapNodeCategory::Gate, "gate_stairs", MapNodePayload::None);
    select_and_confirm_map_node(&mut core, player, gate);
    (core, player)
}

#[test]
fn u12_lifecycle_depth_three_gate_return_moves_held_copies_once_and_terminates_people() {
    let (mut core, player) = arrive_at_gate(3);
    let weapon = core.state.inventory.equipments.iter()
        .find(|copy| copy.meta.equipment_type == EquipmentType::Weapon).unwrap().instance_uuid;
    core.state.inventory.equipments.get_item_mut(&weapon).unwrap().enhancement_level = 4;
    let active_equipment: BTreeSet<_> = core.state.inventory.equipments.iter().map(|copy| copy.instance_uuid).collect();
    let active_fragments: BTreeMap<_, _> = core.state.skill_fragments.copies()
        .map(|copy| (copy.copy_uuid, VaultFragment { investment: Default::default(), copy_uuid: copy.copy_uuid, definition_id: copy.definition_id.clone(),
            unlocked_forms: copy.unlocked_forms.clone(), selected_form: copy.selected_form.clone() })).collect();
    let mut expected_equipment: BTreeMap<_, _> = core.state.inventory.equipments.iter()
        .map(|copy| (copy.instance_uuid, VaultEquipment {
            weapon_stage: copy.weapon_stage, copy_uuid: copy.instance_uuid, definition_id: copy.meta.id.clone(),
            equipment_type: copy.meta.equipment_type, enhancement_level: if copy.meta.equipment_type == EquipmentType::Weapon { 0 } else { copy.enhancement_level } })).collect();
    expected_equipment.insert(Uuid::from_u128(12599), equipment(12599, "dormant_weapon"));
    core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":"Return"})).unwrap();
    assert_eq!(core.state.player_profile.lifecycle.vault.equipment, expected_equipment);
    assert_eq!(core.state.player_profile.lifecycle.vault.fragments, active_fragments);
    assert_eq!(core.state.player_profile.lifecycle.vault.equipment[&weapon].enhancement_level, 0);
    assert!(active_equipment.iter().all(|copy| core.state.player_profile.lifecycle.vault.equipment.contains_key(copy)));
    assert_eq!(core.state.roster.len(), 0);
    assert!(core.state.player_profile.lifecycle.active_expedition.is_none());
    assert_eq!(core.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Pending);
    assert!(core.export_active_run_checkpoint().unwrap().is_none());
    let before = core_roots(&core);
    assert!(core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":"Return"})).is_err());
    assert_eq!(core_roots(&core), before);
}

#[test]
fn u12_lifecycle_current_depth_two_gate_cannot_return_and_depth_three_continue_keeps_assets() {
    let (mut shallow, player) = arrive_at_gate(2);
    let before = core_roots(&shallow);
    assert!(core_command(&mut shallow, player, json!({"type":"choose_expedition_continuation","choice":"Return"})).is_err());
    assert_eq!(core_roots(&shallow), before, "destination depth three cannot authorize a depth-two return");
    let (mut core, player) = arrive_at_gate(3);
    let profile = core.state.player_profile.clone();
    let employees: BTreeSet<_> = core.state.roster.iter().map(|employee| employee.uuid).collect();
    let copies: BTreeSet<_> = core.state.inventory.equipments.iter().map(|copy| copy.instance_uuid).collect();
    core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":"Continue"})).unwrap();
    assert_eq!(core.state.player_profile.lifecycle, profile.lifecycle);
    assert_eq!(core.state.roster.iter().map(|employee| employee.uuid).collect::<BTreeSet<_>>(), employees);
    assert_eq!(core.state.inventory.equipments.iter().map(|copy| copy.instance_uuid).collect::<BTreeSet<_>>(), copies);
    assert_eq!(core.state.run.as_ref().unwrap().run_progression.floor_index(), 3);
    assert!(core.export_active_run_checkpoint().unwrap().is_some());
}

#[test]
fn u12_lifecycle_first_return_story_pending_survives_profile_roundtrip_and_ack_is_once() {
    let (mut core, player) = arrive_at_gate(3);
    core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":"Return"})).unwrap();
    let profile_wire = serde_json::to_vec(&core.state.player_profile).unwrap();
    let mut restarted = GameCore::new(core.game_data.clone(), 12500);
    restarted.state.player_profile = serde_json::from_slice(&profile_wire).unwrap();
    assert_eq!(restarted.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Pending);
    core_command(&mut restarted, player, json!({"type":"acknowledge_first_return_story"})).unwrap();
    assert_eq!(restarted.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Consumed);
    let consumed = core_roots(&restarted);
    core_command(&mut restarted, player, json!({"type":"acknowledge_first_return_story"})).unwrap();
    assert_eq!(core_roots(&restarted), consumed, "retry cannot repeat grants or change receipt");
    start_new_game_with_default_starters(&mut restarted, player);
    restarted.state.run.as_mut().unwrap().run_progression.mode_state =
        crate::game::map::RunProgressionModeState::Expedition { floor_index: 2 };
    let gate = force_first_available_node(&mut restarted, MapNodeCategory::Gate, "gate_stairs", MapNodePayload::None);
    select_and_confirm_map_node(&mut restarted, player, gate);
    core_command(&mut restarted, player, json!({"type":"choose_expedition_continuation","choice":"Return"})).unwrap();
    assert_eq!(restarted.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Consumed);
}

fn exhaust_before_savepoint(depth: u32) -> (GameCore, Uuid, MapNodeId, Uuid) {
    let player = Uuid::from_u128(12550 + u128::from(depth));
    let mut core = GameCore::new(empty_game_data(), 12550);
    start_new_game_with_default_starters(&mut core, player);
    let employee = core.state.roster.iter().next().unwrap().uuid;
    core.state.roster.get_mut(&employee).unwrap().condition.trauma = 80;
    let progression = &mut core.state.run.as_mut().unwrap().run_progression;
    progression.mode_state = crate::game::map::RunProgressionModeState::Expedition { floor_index: depth - 1 };
    progression.deterrence = 1;
    let node = force_first_available_node(&mut core, MapNodeCategory::SavePoint, "save_point", MapNodePayload::SavePoint);
    select_and_confirm_map_node(&mut core, player, node);
    let progression = &core.state.run.as_ref().unwrap().run_progression;
    assert_eq!(progression.deterrence, 0);
    assert!(progression.entry_receipts.contains(&node));
    let pending = progression.pending_continuation.unwrap();
    assert_eq!(pending.depth, depth);
    assert_eq!(pending.node_id, node);
    assert_eq!(pending.cause, crate::game::expedition_lifecycle::ExpeditionContinuationCause::Exhaustion);
    assert_eq!(core.state.roster.get(&employee).unwrap().condition.trauma, 80);
    assert!(core.state.run_checkpoint.payload.is_none(), "exhaustion choice precedes SavePoint activity");
    (core, player, node, employee)
}

#[test]
fn u12_lifecycle_exhaustion_stores_receipt_before_activity_and_continue_restore_does_not_double_charge() {
    for depth in [1, 2, 3] {
        let (source, player, node, employee) = exhaust_before_savepoint(depth);
        let saved = source.export_active_run_checkpoint().unwrap().unwrap();
        let mut core = core_with_profile_for_continue(&source, 999);
        core.continue_from_active_run_checkpoint(player, &saved.core_payload_json,
            saved.loads_used, saved.run_peak_certified_power).unwrap();
        // U14 resumes an unfinished activity before first entry, including cost.
        assert!(core.state.run.as_ref().unwrap().run_progression.pending_continuation.is_none());
        assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence,1);
        assert!(!core.state.run.as_ref().unwrap().run_progression.entry_receipts.contains(&node));
        select_and_confirm_map_node(&mut core,player,node);
        assert_eq!(core.state.run.as_ref().unwrap().run_progression.pending_continuation,
            source.state.run.as_ref().unwrap().run_progression.pending_continuation);
        for command in [json!({"type":"complete_node"}), json!({"type":"confirm_enter_node"}), json!({"type":"cancel_selected_node"})] {
            let before = core_roots(&core);
            assert!(core_command(&mut core, player, command).is_err());
            assert_eq!(core_roots(&core), before);
        }
        if depth < 3 {
            let before = core_roots(&core);
            assert!(core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":"Return"})).is_err());
            assert_eq!(core_roots(&core), before);
        }
        core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":"Continue"})).unwrap();
        assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence, 0);
        assert!(core.state.run.as_ref().unwrap().run_progression.pending_continuation.is_none());
        assert!(core.state.run.as_ref().unwrap().run_progression.entry_receipts.contains(&node));
        core.execute(player, PlayerBehavior::CompleteNode).unwrap();
        assert_eq!(core.state.roster.get(&employee).unwrap().condition.trauma, 68);
        assert!(core.state.run_checkpoint.payload.is_some());
        core.execute(player, PlayerBehavior::LoadRunCheckpoint).unwrap();
        assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence, 0);
        assert!(core.state.run.as_ref().unwrap().run_progression.pending_continuation.is_none());
        assert!(core.state.run.as_ref().unwrap().run_progression.entry_receipts.contains(&node));
    }
}

#[test]
fn u12_lifecycle_exhaustion_depth_split_returns_or_abandons_without_running_pending_activity() {
    for depth in [1, 2, 3, 4] {
        let (mut core, player, _, _) = exhaust_before_savepoint(depth);
        let vault_before = core.state.player_profile.lifecycle.vault.clone();
        let choice = if depth < 3 { "Abandon" } else { "Return" };
        if depth >= 3 {
            let before = core_roots(&core);
            assert!(core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":"Abandon"})).is_err());
            assert_eq!(core_roots(&core), before);
        }
        core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":choice})).unwrap();
        assert!(core.state.player_profile.lifecycle.active_expedition.is_none());
        assert_eq!(core.state.roster.len(), 0);
        assert!(core.export_active_run_checkpoint().unwrap().is_none());
        assert!(core.state.run_checkpoint.payload.is_none(), "pending activity never creates a SavePoint");
        if depth < 3 {
            assert_eq!(core.state.player_profile.lifecycle.vault, vault_before);
            assert_eq!(core.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Never);
        } else {
            assert!(core.state.player_profile.lifecycle.vault.equipment.len() > vault_before.equipment.len());
            assert_eq!(core.state.player_profile.lifecycle.first_return_story, FirstReturnStory::Pending);
        }
    }
}

#[test]
fn u12_lifecycle_gate_never_charges_deterrence_and_stale_choice_cause_rejects_atomically() {
    let (mut core, player) = arrive_at_gate(3);
    assert_eq!(core.state.run.as_ref().unwrap().run_progression.deterrence, 12);
    let pending = core.state.run.as_ref().unwrap().run_progression.pending_continuation.unwrap();
    assert_eq!(pending.cause, crate::game::expedition_lifecycle::ExpeditionContinuationCause::Gate);
    assert!(!core.state.run.as_ref().unwrap().run_progression.entry_receipts.contains(&pending.node_id));
    core.state.run.as_mut().unwrap().run_progression.pending_continuation.as_mut().unwrap().depth = 2;
    let before = core_roots(&core);
    assert!(core_command(&mut core, player, json!({"type":"choose_expedition_continuation","choice":"Return"})).is_err());
    assert_eq!(core_roots(&core), before);
}

// Mutations below change one persisted relation at a time, after proving that
// the same writer output and separately accepted profile really can Continue.
#[test]
fn u12_lifecycle_pending_continue_rejects_each_phase_session_node_and_cause_mismatch() {
    for gate in [true] {
        let (source, player) = if gate { arrive_at_gate(3) } else {
            let (core, player, _, _) = exhaust_before_savepoint(3);
            (core, player)
        };
        let saved = source.export_active_run_checkpoint().unwrap().unwrap();
        let document: Value = serde_json::from_slice(&saved.core_payload_json).unwrap();
        let mut positive = core_with_profile_for_continue(&source, 12591);
        positive.continue_from_active_run_checkpoint(player, &saved.core_payload_json,
            saved.loads_used, saved.run_peak_certified_power).unwrap();
        assert_eq!(positive.state.node_session, source.state.node_session);
        assert_eq!(positive.state.run.as_ref().unwrap().run_progression.pending_continuation,
            source.state.run.as_ref().unwrap().run_progression.pending_continuation);
        for pointer in ["/continuation/pending_node_session", "/continuation/phase/NodeConfirm"] {
            let mut unknown = document.clone();
            unknown.pointer_mut(pointer).unwrap().as_object_mut().unwrap()
                .insert("unrecognized_u12_field".into(), json!(true));
            // Keep the writer's original typed checksum: dropping the unknown
            // member would otherwise restore precisely the accepted document.
            let bytes = serde_json::to_vec(&unknown).unwrap();
            assert!(GameCore::inspect_checkpoint_requirements(&bytes, usize::MAX).is_err(), "strict nested {pointer}");
            let mut destination = core_with_profile_for_continue(&source, 12590);
            let before = core_roots(&destination);
            assert!(destination.continue_from_active_run_checkpoint(player, &bytes,
                saved.loads_used, saved.run_peak_certified_power).is_err());
            assert_eq!(core_roots(&destination), before);
        }
        let mut cases = Vec::new();
        for (name, path, value) in [
            ("wrong phase", "/continuation/phase", json!("ViewingMap")),
            ("missing session", "/continuation/pending_node_session", Value::Null),
            ("session node", "/continuation/pending_node_session/node_id", json!(Uuid::nil())),
            ("session kind", "/continuation/pending_node_session/kind_id", json!("different_kind")),
            ("session category", "/continuation/pending_node_session/category", json!("Start")),
            ("session payload", "/continuation/pending_node_session/payload", if gate { json!("SavePoint") } else { json!("None") }),
            ("phase node", "/continuation/phase/NodeConfirm/node_id", json!(Uuid::nil())),
            ("phase kind", "/continuation/phase/NodeConfirm/kind_id", json!("different_kind")),
            ("phase category", "/continuation/phase/NodeConfirm/category", json!("Start")),
            ("pending node", "/continuation/current/run_progression/pending_continuation/node_id", json!(Uuid::nil())),
            ("pending depth", "/continuation/current/run_progression/pending_continuation/depth", json!(2)),
            ("pending cause", "/continuation/current/run_progression/pending_continuation/cause", if gate { json!("Exhaustion") } else { json!("Gate") }),
        ] {
            let mut bad = document.clone();
            let target = bad.pointer_mut(path).expect("mutation must hit a real current field");
            assert_ne!(*target, value, "{name} must change input");
            *target = value;
            cases.push((name, bad));
        }
        let mut missing_field = document.clone();
        assert!(missing_field["continuation"].as_object_mut().unwrap().remove("pending_node_session").is_some());
        cases.push(("required session field omitted", missing_field));
        let mut unavailable = document.clone();
        let pending_id = document["continuation"]["current"]["run_progression"]["pending_continuation"]["node_id"].clone();
        let nodes = unavailable["continuation"]["current"]["map"]["nodes"].as_array_mut().unwrap();
        let selected = nodes.iter_mut().find(|node| node["id"] == pending_id).unwrap();
        assert_eq!(selected["state"], "Available");
        selected["state"] = json!("Locked");
        cases.push(("pending node no longer available", unavailable));
        let mut unreachable = document.clone();
        let available = unreachable["continuation"]["current"]["map_progression"]["available_node_ids"].as_array_mut().unwrap();
        assert!(available.contains(&pending_id));
        available.retain(|id| *id != pending_id);
        cases.push(("pending node no longer selectable", unreachable));
        if gate {
            let mut coherent_bad_payload = document.clone();
            coherent_bad_payload["continuation"]["pending_node_session"]["payload"] = json!("SavePoint");
            let nodes = coherent_bad_payload["continuation"]["current"]["map"]["nodes"].as_array_mut().unwrap();
            nodes.iter_mut().find(|node| node["id"] == pending_id).unwrap()["payload"] = json!("SavePoint");
            cases.push(("coherent gate node and session have non-gate payload", coherent_bad_payload));
        }
        if !gate {
            let mut no_receipt = document.clone();
            no_receipt["continuation"]["current"]["run_progression"]["entry_receipts"] = json!([]);
            cases.push(("exhaustion receipt absent", no_receipt));
            let mut nonzero = document.clone();
            nonzero["continuation"]["current"]["run_progression"]["deterrence"] = json!(1);
            cases.push(("exhaustion with remaining deterrence", nonzero));
        }
        for (name, mut bad) in cases {
            refresh_active_run_checksum(&mut bad);
            let bytes = serde_json::to_vec(&bad).unwrap();
            if name != "required session field omitted" {
                assert_eq!(GameCore::inspect_checkpoint_requirements(&bytes, usize::MAX).unwrap(),
                    *source.content_identity(), "checksum/type preflight must pass: {name}");
            }
            let mut destination = core_with_profile_for_continue(&source, 12592);
            let before = core_roots(&destination);
            for _ in 0..2 {
                assert!(destination.continue_from_active_run_checkpoint(player, &bytes,
                    saved.loads_used, saved.run_peak_certified_power).is_err(), "gate={gate}, {name}");
                assert_eq!(core_roots(&destination), before, "gate={gate}, {name}");
            }
        }
    }
}

#[test]
fn u12_lifecycle_pending_choice_cannot_be_hidden_in_current_or_snapshot_savepoint() {
    let (source, player) = arrive_at_gate(3);
    let saved = source.export_active_run_checkpoint().unwrap().unwrap();
    let document: Value = serde_json::from_slice(&saved.core_payload_json).unwrap();
    let mut positive = core_with_profile_for_continue(&source, 12593);
    positive.continue_from_active_run_checkpoint(player, &saved.core_payload_json,
        saved.loads_used, saved.run_peak_certified_power).unwrap();
    assert!(document["continuation"]["savepoint"].is_null());
    for alias in [false, true] {
        let mut bad = document.clone();
        bad["continuation"]["savepoint"] = if alias { json!({"current":{}}) } else {
            json!({"snapshot":{"checkpoint":document["continuation"]["current"]}})
        };
        refresh_active_run_checksum(&mut bad);
        let bytes = serde_json::to_vec(&bad).unwrap();
        assert_eq!(GameCore::inspect_checkpoint_requirements(&bytes, usize::MAX).unwrap(),
            *source.content_identity());
        let mut destination = core_with_profile_for_continue(&source, 12594);
        let before = core_roots(&destination);
        assert!(destination.continue_from_active_run_checkpoint(player, &bytes,
            saved.loads_used, saved.run_peak_certified_power).is_err(), "pending SavePoint alias={alias}");
        assert_eq!(core_roots(&destination), before);
    }
    // Standalone SavePoint validation shares no continuation phase/session:
    // normal saved state succeeds, while a pending activity cannot be saved.
    let (mut completed, _, _, _) = exhaust_before_savepoint(3);
    core_command(&mut completed, player, json!({"type":"choose_expedition_continuation","choice":"Continue"})).unwrap();
    completed.execute(player, PlayerBehavior::CompleteNode).unwrap();
    let checkpoint = completed.state.run_checkpoint.payload.as_ref().unwrap();
    completed.validate_run_checkpoint_payload(checkpoint).unwrap();
    let pending: crate::game::world::state::RunCheckpointPayload =
        serde_json::from_value(document["continuation"]["current"].clone()).unwrap();
    assert!(source.validate_run_checkpoint_payload(&pending).is_err());
}
