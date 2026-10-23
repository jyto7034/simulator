//! Restore authority and native owners use an internal map fixture, normal
//! Event commands, and the public CoreApplication Continue decoder. This is
//! separate from the external-pack and non-admin map traversal acceptance.
use super::*;
use crate::application::{ApplicationCheckpoint, ApplicationSnapshotKind, CoreApplication};
use crate::game::{
    battle::runtime_control as native,
    data::{
        event_data::{EventDatabase, EventId, EventSceneId},
        GameDataBuilder, ValidatedGameData,
    },
    relic::RelicVariant,
    resources::CommittedRelicCorruptionSelection,
};
use std::sync::OnceLock;

fn data() -> ValidatedGameData {
    static DATA: OnceLock<ValidatedGameData> = OnceLock::new();
    DATA.get_or_init(|| {
        let (base, _, _) = super::u08_relics::setup();
        let authored: EventDatabase = ron::de::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/u08/public001/events.ron"
        )))
        .unwrap();
        let mut events = base.game_data.event_data.events.clone();
        let mut alias = authored.events[0].clone();
        alias.id = EventId::new("u08_owner_same_graph_different_event");
        events.extend(authored.events);
        events.push(alias);
        let generation = GameDataBuilder::from_validated(&base.game_data)
            .with_events(EventDatabase::new(events))
            .build_arc();
        GameCore::new(generation, 0).game_data
    })
    .clone()
}

fn setup() -> (GameCore, Uuid, Vec<Uuid>) {
    let mut core = GameCore::new(data(), 0xa508b);
    let player = Uuid::from_u128(0xa508b);
    start_new_game_with_default_starters(&mut core, player);
    let employees = core.state.roster.combat_eligible_employee_ids();
    (core, player, employees)
}

fn enter(core: &mut GameCore, player: Uuid, event: &str) -> MapNodeId {
    let node = force_first_available_node(
        core,
        MapNodeCategory::Event,
        "event_story",
        MapNodePayload::Event {
            event_id: Some(EventId::new(event)),
        },
    );
    core.execute(player, PlayerBehavior::SelectMapNode { node_id: node })
        .unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    node
}

fn choose(
    core: &mut GameCore,
    player: Uuid,
    node: MapNodeId,
    source: &str,
    copy: Option<Uuid>,
) -> Result<BehaviorResult, GameError> {
    let event = core.state.run.as_ref().unwrap().event_sessions[&node]
        .event_id
        .clone();
    core.execute(
        player,
        PlayerBehavior::ChooseRelicCorruption {
            node_id: node,
            event_id: event,
            current_scene_id: EventSceneId::new(source),
            copy_uuid: copy,
        },
    )
}

fn advance(core: &mut GameCore, player: Uuid, node: MapNodeId) {
    let current = &core.state.run.as_ref().unwrap().event_sessions[&node];
    core.execute(
        player,
        PlayerBehavior::AdvanceEventScene {
            node_id: node,
            event_id: current.event_id.clone(),
            current_scene_id: current.current_scene_id.clone(),
        },
    )
    .unwrap();
}

fn completed_double() -> (GameCore, Uuid, MapNodeId, Vec<Uuid>) {
    let (mut core, player, _) = setup();
    let copies = super::u08_relics::grant(&mut core, 2);
    let node = enter(&mut core, player, "u08_public_double_corruption");
    choose(&mut core, player, node, "first", Some(copies[0])).unwrap();
    advance(&mut core, player, node);
    choose(&mut core, player, node, "second", Some(copies[1])).unwrap();
    advance(&mut core, player, node);
    (core, player, node, copies)
}

fn app_roots(app: &CoreApplication) -> Value {
    let snapshots: Vec<_> = ApplicationSnapshotKind::ALL
        .iter()
        .map(|kind| app.snapshot(*kind).unwrap().read_model().as_value().clone())
        .collect();
    let checkpoint = app.export_active_run_checkpoint().unwrap().map(|saved| json!({"bytes":saved.core_payload_json(), "loads":saved.loads_used(), "peak":saved.run_peak_certified_power()}));
    json!({"snapshots":snapshots,"checkpoint":checkpoint})
}

fn restore(
    profile: &crate::game::encounter_compendium::PlayerProfileState,
    document: &Value,
    player: Uuid,
    peak: u64,
) -> Result<CoreApplication, crate::application::ApplicationError> {
    let mut app = CoreApplication::new(data(), 0xdead);
    app.install_authoritative_player_profile(serde_json::to_value(profile).unwrap().into()).unwrap();
    let before = app_roots(&app);
    let checkpoint = ApplicationCheckpoint::new(serde_json::to_vec(document).unwrap(), 0, peak);
    match app.continue_from_active_run_checkpoint(player, &checkpoint) {
        Ok(_) => Ok(app),
        Err(error) => {
            assert_eq!(
                app_roots(&app),
                before,
                "rejected decode cannot publish any partial restored root"
            );
            Err(error)
        }
    }
}

#[test]
fn relic_event_completed_history_restore_rejects_malformed_order_copy_scene_and_map_authority() {
    let (core, player, node, copies) = completed_double();
    let exported = core.export_active_run_checkpoint().unwrap().unwrap();
    let original: Value = serde_json::from_slice(&exported.core_payload_json).unwrap();
    let node_key = node.0.to_string();
    let other_node = core
        .state
        .run
        .as_ref()
        .unwrap()
        .map
        .nodes
        .iter()
        .find(|row| row.category != MapNodeCategory::Event)
        .unwrap()
        .id;
    let session = &original["continuation"]["current"]["event_sessions"][&node_key];
    assert_eq!(
        session["relic_corruption_selections"],
        json!([
            {"source_scene_id":"first","copy_uuid":copies[0]}, {"source_scene_id":"second","copy_uuid":copies[1]},
        ])
    );
    restore(&core.state.player_profile, &original, player, exported.run_peak_certified_power).unwrap();
    for case in [
        "some_empty",
        "duplicate_scene",
        "repeat_copy",
        "missing_scene",
        "not_selection",
        "reverse_order",
        "current_path",
        "normal_copy",
        "non_event_node",
        "wrong_event_same_graph",
    ] {
        let mut document = original.clone();
        let current = &mut document["continuation"]["current"];
        match case {
            "some_empty" => {
                current["event_sessions"][&node_key]["relic_corruption_selections"] = json!([])
            }
            "duplicate_scene" => {
                current["event_sessions"][&node_key]["relic_corruption_selections"][1]
                    ["source_scene_id"] = json!("first")
            }
            "repeat_copy" => {
                current["event_sessions"][&node_key]["relic_corruption_selections"][1]
                    ["copy_uuid"] = json!(copies[0])
            }
            "missing_scene" => {
                current["event_sessions"][&node_key]["relic_corruption_selections"][0]
                    ["source_scene_id"] = json!("missing")
            }
            "not_selection" => {
                current["event_sessions"][&node_key]["relic_corruption_selections"][0]
                    ["source_scene_id"] = json!("bridge")
            }
            "reverse_order" => current["event_sessions"][&node_key]["relic_corruption_selections"]
                .as_array_mut()
                .unwrap()
                .reverse(),
            "current_path" => {
                current["event_sessions"][&node_key]["current_scene_id"] = json!("bridge")
            }
            "normal_copy" => {
                current["relics"]["copies"][copies[0].to_string()]["variant"] = json!("Normal")
            }
            "non_event_node" => {
                let mut session = current["event_sessions"]
                    .as_object_mut()
                    .unwrap()
                    .remove(&node_key)
                    .unwrap();
                session["node_id"] = json!(other_node);
                current["event_sessions"][other_node.0.to_string()] = session;
            }
            "wrong_event_same_graph" => {
                current["event_sessions"][&node_key]["event_id"] =
                    json!("u08_owner_same_graph_different_event")
            }
            _ => unreachable!(),
        }
        refresh_active_run_checksum(&mut document);
        let error = restore(&core.state.player_profile, &document, player, exported.run_peak_certified_power)
            .err()
            .unwrap_or_else(|| panic!("accepted forged {case}"));
        assert_eq!(error.code(), "invalid_static_data", "{case}: {error:?}");
    }
    assert_eq!(
        serde_json::from_slice::<Value>(&exported.core_payload_json).unwrap(),
        original,
        "tampered test documents cannot modify their positive control"
    );
}

#[test]
fn relic_event_restore_uses_the_employee_check_outcome_to_reject_the_untraversed_branch() {
    let (mut core, player, employees) = setup();
    let node = enter(&mut core, player, "u08_public_branch_corruption");
    core.execute(
        player,
        PlayerBehavior::ResolveEventChoiceCheck {
            node_id: node,
            event_id: EventId::new("u08_public_branch_corruption"),
            current_scene_id: EventSceneId::new("check"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("attempt"),
            employee_uuid: employees[0],
        },
    )
    .unwrap();
    let branch = core.state.run.as_ref().unwrap().event_sessions[&node]
        .current_scene_id
        .0
        .clone();
    assert!(matches!(branch.as_str(), "success" | "failure"));
    choose(&mut core, player, node, &branch, None).unwrap();
    advance(&mut core, player, node);
    let exported = core.export_active_run_checkpoint().unwrap().unwrap();
    let original: Value = serde_json::from_slice(&exported.core_payload_json).unwrap();
    restore(&core.state.player_profile, &original, player, exported.run_peak_certified_power).unwrap();
    for case in [
        "branch_only",
        "contradictory_roll_and_path",
        "missing_check",
    ] {
        let mut forged = original.clone();
        let session = &mut forged["continuation"]["current"]["event_sessions"][node.0.to_string()];
        match case {
            "branch_only" => {
                session["relic_corruption_selections"][0]["source_scene_id"] =
                    json!(if branch == "success" {
                        "failure"
                    } else {
                        "success"
                    })
            }
            "contradictory_roll_and_path" => {
                // The path agrees with the forged outcome, but natural 1 must
                // still be CriticalFailure regardless of historical modifiers.
                session["committed_check"]["raw_roll"] = json!(1);
                let modifier = session["committed_check"]["modifier_total"]
                    .as_u64()
                    .unwrap();
                session["committed_check"]["total_roll"] = json!(modifier + 1);
                session["committed_check"]["outcome"] = json!("success");
                session["relic_corruption_selections"][0]["source_scene_id"] = json!("success");
            }
            "missing_check" => {
                session.as_object_mut().unwrap().remove("committed_check");
            }
            _ => unreachable!(),
        }
        refresh_active_run_checksum(&mut forged);
        assert_eq!(
            restore(&core.state.player_profile, &forged, player, exported.run_peak_certified_power)
                .err()
                .unwrap()
                .code(),
            "invalid_static_data",
            "{case}"
        );
    }
}

#[test]
fn relic_event_history_survives_a_later_legal_dismantle_of_its_selected_copy() {
    let (mut core, player, _) = setup();
    // Intro has no authored Maintenance slot. Select the supported fixture map
    // before any receipt exists; changing maps after the Event would invalidate
    // its historical node identity. Both nodes then use normal entry commands.
    force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    let template = core.state.run.as_ref().unwrap().map.map_template_id.clone();
    let copies = super::u08_relics::grant(&mut core, 2);
    let node = enter(&mut core, player, "u08_public_double_corruption");
    choose(&mut core, player, node, "first", Some(copies[0])).unwrap();
    advance(&mut core, player, node);
    choose(&mut core, player, node, "second", Some(copies[1])).unwrap();
    advance(&mut core, player, node);
    assert_eq!(
        core.state.run.as_ref().unwrap().map.map_template_id,
        template
    );
    assert!(core.state.run.as_ref().unwrap().map.node(node).is_some());

    let maintenance = force_first_available_node(
        &mut core,
        MapNodeCategory::Maintenance,
        "maintenance",
        MapNodePayload::Maintenance,
    );
    assert_eq!(
        core.state.run.as_ref().unwrap().map.map_template_id,
        template
    );
    assert!(core.state.run.as_ref().unwrap().map.node(node).is_some());
    assert_ne!(maintenance, node);
    core.execute(
        player,
        PlayerBehavior::SelectMapNode {
            node_id: maintenance,
        },
    )
    .unwrap();
    core.execute(player, PlayerBehavior::ConfirmEnterNode)
        .unwrap();
    core.execute(
        player,
        PlayerBehavior::DismantleRelic {
            copy_uuid: copies[0],
            confirmed: true,
        },
    )
    .unwrap();
    core.execute(player, PlayerBehavior::CompleteNode).unwrap();
    assert_eq!(
        core.state.run.as_ref().unwrap().map.map_template_id,
        template
    );
    assert!(core.state.run.as_ref().unwrap().map.node(node).is_some());
    assert!(core.state.relics.get(copies[0]).is_none());
    assert_eq!(
        core.state.relics.get(copies[1]).unwrap().variant,
        RelicVariant::Corrupt
    );
    assert_eq!(core.state.crafting_resources.equipment_dust, 7);
    let saved = core.export_active_run_checkpoint().unwrap().unwrap();
    let document: Value = serde_json::from_slice(&saved.core_payload_json).unwrap();
    assert_eq!(
        document["continuation"]["current"]["event_sessions"][node.0.to_string()]
            ["relic_corruption_selections"][0]["copy_uuid"],
        json!(copies[0])
    );
    restore(&core.state.player_profile, &document, player, saved.run_peak_certified_power).unwrap();
}

fn core_roots(core: &GameCore) -> Value {
    json!({"world":non_battle_state_without_profile_fingerprint(core),
        "relics":core.state.relics, "materials":core.state.crafting_resources,
        "continue":core.export_active_run_checkpoint().unwrap().map(|saved|saved.core_payload_json)})
}

#[test]
fn relic_event_decline_only_activates_independent_history_owners_and_admits_before_allocation() {
    let (mut probe, player, _) = setup();
    let node = enter(&mut probe, player, "u08_public_double_corruption");
    assert!(!probe.state.has_relic_owner());
    assert_eq!(probe.state.relic_owned_extent().unwrap(), 0);
    let (result, admissions) = native::observe_retained_admissions(|| {
        native::run(native::RuntimeFactLimits::default(), || {
            choose(&mut probe, player, node, "first", None)
        })
    });
    result.unwrap();
    let append = admissions
        .iter()
        .position(|row| row.0 == "relic_event_selection_append" && row.2 > row.1)
        .expect("first decline must admit receipt storage");
    assert!(admissions
        .iter()
        .any(|row| row.0 == "relic_event_session_clone"));
    assert!(admissions
        .iter()
        .any(|row| row.0 == "relic_event_history_projection"));
    let cap = admissions[append].2 - 1;
    println!("U08_DECLINE_ADMISSIONS {admissions:?}");
    let (mut rejected, rejected_player, _) = setup();
    let rejected_node = enter(
        &mut rejected,
        rejected_player,
        "u08_public_double_corruption",
    );
    let before = core_roots(&rejected);
    let (result, prefix) = native::observe_retained_admissions(|| {
        native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: cap,
                ..Default::default()
            },
            || choose(&mut rejected, rejected_player, rejected_node, "first", None),
        )
    });
    let GameError::BattleRuntimeFault(fault) = result.unwrap_err() else {
        panic!("expected typed retained refusal")
    };
    assert_eq!(fault.code, native::RuntimeFaultCode::RetainedByteLimit);
    assert_eq!(fault.site, "relic_event_selection_append");
    assert_eq!(fault.limit, Some(cap as u64));
    assert!(fault.requested.unwrap() > cap as u64);
    assert_eq!(prefix, admissions[..append]);
    assert_eq!(core_roots(&rejected), before);
    assert!(!rejected.state.has_relic_owner());

    assert!(probe.state.has_relic_owner());
    assert!(probe.state.relics.is_empty());
    assert!(probe.state.crafting_resources.is_empty());
    assert!(!probe.state.uuid_manager.has_relic_stream());
    assert_eq!(probe.state.run.as_ref().unwrap().event_sessions.len(), 1);
    assert_persisted_active_event_session_equal(&probe, node);
    let stored = &probe.state.run.as_ref().unwrap().event_sessions[&node];
    let active = probe
        .state
        .active_node_content
        .as_ref()
        .unwrap()
        .as_event()
        .unwrap();
    let stored_history = stored.relic_corruption_selections.as_ref().unwrap();
    let active_history = active.relic_corruption_selections.as_ref().unwrap();
    assert_eq!(stored_history.len(), 1);
    assert_eq!(
        stored_history[0],
        CommittedRelicCorruptionSelection {
            source_scene_id: EventSceneId::new("first"),
            copy_uuid: None
        }
    );
    assert_ne!(
        stored_history.as_ptr(),
        active_history.as_ptr(),
        "run and active sessions own independent Vec allocations"
    );
    assert_ne!(
        stored_history[0].source_scene_id.0.as_ptr(),
        active_history[0].source_scene_id.0.as_ptr()
    );
    assert!(
        probe.state.relic_owned_extent().unwrap()
            >= 2 * (std::mem::size_of::<CommittedRelicCorruptionSelection>() + "first".len())
    );
    let (snapshot, query_admissions) = native::observe_retained_admissions(|| {
        native::run(native::RuntimeFactLimits::default(), || {
            probe.event_scene_snapshot(active)
        })
    });
    let snapshot = snapshot.unwrap();
    let projected = snapshot.relic_corruption_selections.as_ref().unwrap();
    assert_eq!(projected, stored_history);
    assert_ne!(
        projected.as_ptr(),
        active_history.as_ptr(),
        "DTO history is another owned value"
    );
    assert!(query_admissions
        .iter()
        .any(|row| row.0 == "relic_event_history_projection"));

    let (_, advance_admissions) =
        native::observe_retained_admissions(|| advance(&mut probe, player, node));
    assert!(
        advance_admissions
            .iter()
            .any(|row| row.0 == "relic_event_session_clone"),
        "legacy Advance with history must price its session clones"
    );
    choose(&mut probe, player, node, "second", None).unwrap();
    advance(&mut probe, player, node);
    assert!(probe.state.active_node_content.is_none());
    assert!(probe.state.continue_state.has_relic_owner());
    probe.save_run_checkpoint_for_test().unwrap();
    let run_history = probe.state.run.as_ref().unwrap().event_sessions[&node]
        .relic_corruption_selections
        .as_ref()
        .unwrap();
    let checkpoint = probe.state.run_checkpoint.payload.as_ref().unwrap();
    let checkpoint_history = checkpoint.event_sessions[&node]
        .relic_corruption_selections
        .as_ref()
        .unwrap();
    assert_ne!(run_history.as_ptr(), checkpoint_history.as_ptr());
    let (entry, entry_admissions) = native::observe_retained_admissions(|| {
        native::run(native::RuntimeFactLimits::default(), || {
            super::super::state::BattleEntrySnapshot::capture(&probe.state)
        })
    });
    let mut entry_restored = GameCore::new(data(), 0xdead);
    entry.unwrap().restore(&mut entry_restored.state);
    assert!(entry_restored.state.has_relic_owner());
    let entry_history = entry_restored.state.run.as_ref().unwrap().event_sessions[&node]
        .relic_corruption_selections
        .as_ref()
        .unwrap();
    assert_ne!(run_history.as_ptr(), entry_history.as_ptr());
    assert_ne!(checkpoint_history.as_ptr(), entry_history.as_ptr());
    assert!(
        entry_admissions
            .iter()
            .filter(|row| row.0 == "relic_event_session_clone")
            .count()
            >= 2,
        "entry capture admits current and checkpoint history clones separately"
    );

    let exported = probe.export_active_run_checkpoint().unwrap().unwrap();
    let mut restored = CoreApplication::new(data(), 0xdead);
    restored.install_authoritative_player_profile(serde_json::to_value(&probe.state.player_profile).unwrap().into()).unwrap();
    let checkpoint = ApplicationCheckpoint::new(
        exported.core_payload_json.to_vec(),
        exported.loads_used,
        exported.run_peak_certified_power,
    );
    let (result, decode_admissions) = native::observe_retained_admissions(|| {
        restored.continue_from_active_run_checkpoint(player, &checkpoint)
    });
    result.unwrap();
    assert!(decode_admissions.iter().any(|row| row.0 == "relic_continue_value"), "decline-only history must activate predecode admission without any copy or RELICCOP stream");
    println!("U08_DECLINE_LAYOUT EventSessionState={} ActiveNodeContent={} EventSceneSnapshotDto={} CommittedRelicCorruptionSelection={}",
        std::mem::size_of::<crate::game::resources::EventSessionState>(),
        std::mem::size_of::<crate::game::resources::ActiveNodeContent>(),
        std::mem::size_of::<crate::game::behavior::EventSceneSnapshotDto>(),
        std::mem::size_of::<CommittedRelicCorruptionSelection>());
}
