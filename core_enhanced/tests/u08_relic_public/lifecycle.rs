//! Run-lifecycle assertions through public command and checkpoint ports.
//! Admin grants establish inventory inputs; transfer, replacement and Continue
//! use normal application actions. No detached admin node enters a durable save.
use super::support::*;
use game_core::{
    application::{ApplicationSnapshotKind, CoreApplication},
    game::{
        behavior::{GameStateContextDto, RunSnapshotDto},
        relic::RelicStateDto,
    },
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, io::Write, path::Path, sync::OnceLock};
use uuid::Uuid;

fn slots(state: &RelicStateDto, employee: Uuid) -> [Option<Uuid>; 3] {
    state
        .loadouts
        .iter()
        .find(|row| row.employee_uuid == employee)
        .map(|row| row.slots)
        .unwrap_or([None; 3])
}

fn literal_hit(plan: &Value) -> &Value {
    plan["targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| {
            row["kind"] == "skill_damage"
                && row["site"]["action"]["definition_id"] == "u08_public_literal"
                && row["site"]["effect_id"] == "both"
        })
        .unwrap()
}

fn amount(plan: &Value, effective: u64) {
    assert_eq!(
        literal_hit(plan)["evaluation"],
        json!({
            "status":"available", "base_amount":100, "effective_amount":effective,
        })
    );
}

fn finish_start(app: &mut CoreApplication) -> Vec<Uuid> {
    for _ in 0..128 {
        let run: RunSnapshotDto =
            serde_json::from_value(snapshot(app, ApplicationSnapshotKind::Run)).unwrap();
        if let Some(vn) = run.visual_novel {
            let next = if let Some(choice) = vn.choices.iter().find(|row| row.selectable) {
                json!({"type":"select_visual_novel_choice", "session_id":vn.session_id,
                    "current_scene_id":vn.current_scene_id, "choice_id":choice.choice_id})
            } else {
                json!({"type":"advance_visual_novel", "session_id":vn.session_id,
                    "current_scene_id":vn.current_scene_id})
            };
            command(app, next);
            continue;
        }
        if let GameStateContextDto::SelectingStarterEmployees {
            required_count,
            candidates,
        } = run.game_state_context
        {
            let ids: Vec<_> = candidates
                .into_iter()
                .take(required_count)
                .map(|row| row.employee_definition_id)
                .collect();
            command(
                app,
                json!({"type":"select_starter_employees", "employee_definition_ids":ids, "loan_copy_ids":[]}),
            );
            continue;
        }
        let employees = roster(app).combat_eligible_employee_ids;
        assert!(employees.len() >= 2);
        return employees;
    }
    panic!("normal replacement start flow exceeded fixture bound");
}

#[test]
fn u08_public_successful_transfer_moves_exact_copy_and_only_its_employee_contributions() {
    for corrupt in [false, true] {
        let (mut app, employees) = app();
        let (first, second) = (employees[0], employees[1]);
        for employee in [first, second] {
            equip_fragment(&mut app, employee, "u08_public_fragment");
            equip_weapon(&mut app, employee, "u08_public_weapon");
        }
        let copy = grant_relic(&mut app, "u08_public_blend");
        // Keep the former registration smoke's exact first-grant/slot oracle.
        equip_relic(&mut app, copy, first, 0);
        let initial = relic_state(&mut app);
        assert_eq!(initial.copies.len(), 1);
        assert_eq!(initial.copies[0].copy_uuid, copy);
        assert_eq!(initial.loadouts[0].employee_uuid, first);
        assert_eq!(initial.loadouts[0].slots, [Some(copy), None, None]);
        equip_relic(&mut app, copy, first, 2);
        if corrupt {
            admin(
                &mut app,
                json!({"type":"admin_grant_relic_corruption_catalyst", "amount":1}),
            );
            command(&mut app, json!({"type":"corrupt_relic", "copy_uuid":copy}));
        }
        let expected = if corrupt { 210 } else { 180 };
        amount(&numeric_plan(&mut app, first), expected);
        amount(&numeric_plan(&mut app, second), 100);
        let before = relic_state(&mut app);
        let original_copy = json!(before
            .copies
            .iter()
            .find(|row| row.copy_uuid == copy)
            .unwrap());
        let resources = json!(before.resources);
        assert_eq!(slots(&before, first), [None, None, Some(copy)]);
        assert_eq!(slots(&before, second), [None; 3]);

        // EquipRelic is the public transfer operation: no preliminary unequip.
        equip_relic(&mut app, copy, second, 1);
        let after = relic_state(&mut app);
        assert_eq!(after.copies.len(), 1);
        assert_eq!(json!(after.copies[0]), original_copy);
        assert_eq!(json!(after.resources), resources);
        assert_eq!(slots(&after, first), [None; 3]);
        assert_eq!(slots(&after, second), [None, Some(copy), None]);
        assert_eq!(
            after
                .loadouts
                .iter()
                .flat_map(|row| row.slots)
                .flatten()
                .filter(|id| *id == copy)
                .count(),
            1
        );
        let before_query = public_roots(&app);
        let first_plan = numeric_plan(&mut app, first);
        let second_plan = numeric_plan(&mut app, second);
        amount(&first_plan, 100);
        amount(&second_plan, expected);
        assert!(literal_hit(&first_plan)["effects"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(literal_hit(&second_plan)["effects"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| { row["copy_uuid"] == json!(copy) && row["state"]["status"] == "active" }));
        assert_eq!(first_plan["penalties"], json!([]));
        assert_eq!(
            second_plan["penalties"].as_array().unwrap().len(),
            if corrupt { 2 } else { 0 }
        );
        assert_eq!(public_roots(&app), before_query);

        // This request names only the copy. Clearing the new owner's slot
        // proves the public reverse lookup follows the transfer destination.
        command(&mut app, json!({"type":"unequip_relic", "copy_uuid":copy}));
        let cleared = relic_state(&mut app);
        assert_eq!(slots(&cleared, second), [None; 3]);
        assert_eq!(json!(cleared.copies[0]), original_copy);
        amount(&numeric_plan(&mut app, second), 100);
    }
}

#[test]
fn u08_public_abandon_then_new_run_resets_copies_slots_catalysts_and_equipment_dust() {
    let (mut app, employees) = app();
    let normal = grant_relic(&mut app, "u08_public_blend");
    let corrupt = grant_relic(&mut app, "u08_public_blend");
    equip_relic(&mut app, normal, employees[0], 0);
    equip_relic(&mut app, corrupt, employees[1], 2);
    admin(
        &mut app,
        json!({"type":"admin_grant_relic_corruption_catalyst", "amount":5}),
    );
    admin(
        &mut app,
        json!({"type":"admin_grant_equipment_dust", "amount":23}),
    );
    command(
        &mut app,
        json!({"type":"corrupt_relic", "copy_uuid":corrupt}),
    );
    let initial = relic_state(&mut app);
    assert_eq!(initial.copies.len(), 2);
    assert_eq!(initial.loadouts.len(), 2);
    assert_eq!(initial.resources.relic_corruption_catalysts, 4);
    assert_eq!(initial.resources.equipment_dust, 23);

    // Use an otherwise valid start request so this rejection proves the
    // active-run gate. Abandon is the public transition to NotStarted.
    let before_rejected_start = public_roots(&app);
    assert!(execute(
        &mut app,
        json!({"type":"start_new_game", "game_mode":"Expedition", "replace_active_run":false})
    )
    .is_err());
    assert_eq!(public_roots(&app), before_rejected_start);
    command(&mut app, json!({"type":"abandon_run"}));
    assert!(app.is_not_started());
    assert!(app.export_active_run_checkpoint().unwrap().is_none());
    command(
        &mut app,
        json!({"type":"start_new_game", "game_mode":"Expedition", "replace_active_run":false}),
    );
    let new_employees = finish_start(&mut app);
    let reset = relic_state(&mut app);
    assert!(reset.copies.is_empty());
    assert!(reset.loadouts.is_empty());
    assert!(reset.not_sellable_copy_uuids.is_empty());
    assert_eq!(reset.resources.relic_corruption_catalysts, 0);
    assert_eq!(reset.resources.equipment_dust, 0);
    let before_stale = public_roots(&app);
    for copy in [normal, corrupt] {
        assert!(execute(
            &mut app,
            json!({"type":"equip_relic", "copy_uuid":copy,
            "employee_uuid":new_employees[0], "slot":0})
        )
        .is_err());
        assert_eq!(public_roots(&app), before_stale);
    }
    let saved = app.export_active_run_checkpoint().unwrap().unwrap();
    let bytes = saved.core_payload_json().to_vec();
    let mut restored = CoreApplication::new(content().data.clone(), 0xa508d);
    restored
        .install_authoritative_player_profile(
            snapshot(&app, ApplicationSnapshotKind::PlayerProfile).into(),
        )
        .unwrap();
    restored
        .continue_from_active_run_checkpoint(PLAYER, &saved)
        .unwrap();
    assert_eq!(saved.core_payload_json(), bytes);
    assert_eq!(json!(relic_state(&mut restored)), json!(reset));
}

fn authored_lifecycle_templates(original: &[u8]) -> Vec<u8> {
    use game_core::game::data::event_data::EventId;
    use game_core::game::map::{
        FacilityActivityDefinition, FacilityContentSupport, FacilityReservation,
        FacilityRoomConnection, FacilitySizeAllocation, FacilityTemplateDatabase, MapEdgeDirection,
        MapNodeKindId, MapNodePayload, MapSlotId, MapTemplateId,
    };

    let mut catalog: FacilityTemplateDatabase = ron::de::from_bytes(original).unwrap();
    let mut layout = catalog
        .layouts
        .iter()
        .find(|layout| layout.expedition_default)
        .unwrap()
        .clone();
    let entrance = layout.room(&layout.entrance_room_id).unwrap().clone();
    let mut room = layout.room(&layout.initial_room_id).unwrap().clone();
    room.id = "u08_event_room".into();
    room.template_id = Some(MapTemplateId::new("facility_6A"));
    room.name = "Relic continuation fixture".into();
    room.reservations = vec![FacilityReservation {
        slot_id: MapSlotId::new("1"),
        activity_id: "u08_double_corruption".into(),
    }];
    room.omen_slot = None;
    room.elite_slots.clear();
    room.safe_access_slots = vec![MapSlotId::new("1")];
    room.slot_display.clear();

    layout.id = "u08_relic_continuation_v1".into();
    layout.initial_room_id = room.id.clone();
    layout.connections = vec![FacilityRoomConnection {
        from: entrance.id.clone(),
        to: room.id.clone(),
        direction: MapEdgeDirection::ForwardOnly,
        condition: None,
    }];
    layout.rooms = vec![entrance, room];
    layout.golden_bough_plans.clear();
    for activity in &mut layout.activities {
        activity.golden_bough_candidate = false;
    }
    layout.activities.push(FacilityActivityDefinition {
        id: "u08_double_corruption".into(),
        label: "Two free corruption choices".into(),
        kind_id: MapNodeKindId::new("event_sealed_archive"),
        payload: MapNodePayload::Event {
            event_id: Some(EventId::new("u08_public_double_corruption")),
        },
        support: FacilityContentSupport::Supported,
        condition: None,
        reward_activity: None,
        first_entry_cost: Some(0),
        golden_bough_candidate: false,
    });
    // Slot 1 is adjacent to S in the official 6A template. Other slots retain
    // normal allocation and the current facility difficulty contract.
    layout.allocation.support_counts = BTreeMap::from([(0, 100)]);
    layout.allocation.one_reserved_support_counts = BTreeMap::from([(1, 100)]);
    layout.allocation.sizes = vec![FacilitySizeAllocation {
        size: 6,
        event_counts: BTreeMap::from([(0, 100)]),
        reward_counts: BTreeMap::from([(0, 100)]),
        elite_percent_by_depth: BTreeMap::from([(1, 0)]),
    }];
    catalog.layouts = vec![layout];
    ron::ser::to_string(&catalog).unwrap().into_bytes()
}

fn copy_lifecycle_content(source: &Path, destination: &Path, relative: &Path) {
    let current = source.join(relative);
    fs::create_dir(destination.join(relative)).unwrap();
    for entry in fs::read_dir(&current).unwrap() {
        let entry = entry.unwrap();
        let child = relative.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_lifecycle_content(source, destination, &child);
            continue;
        }
        let original = fs::read(entry.path()).unwrap();
        let replacement = match child.to_str().unwrap() {
            "official/data/map/facility_templates.ron" => {
                Some(authored_lifecycle_templates(&original))
            }
            _ => None,
        };
        let bytes = replacement.as_deref().unwrap_or(&original);
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination.join(&child))
            .unwrap()
            .write_all(bytes)
            .unwrap();
        if replacement.is_none() {
            assert_eq!(fs::read(destination.join(child)).unwrap(), original);
        }
    }
}

fn lifecycle_content() -> &'static PublicContent {
    static CONTENT: OnceLock<PublicContent> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let original = content();
        let root = std::env::temp_dir().join(format!("u08-lifecycle-{}", Uuid::new_v4()));
        // A separate authored test scenario changes only the Event entry path
        // and payload. The seven public001 catalogs and original root stay intact.
        copy_lifecycle_content(&original.root, &root, Path::new(""));
        let selection = original.selection.clone();
        let data =
            game_core::game::data::content_pack::load(&root, &selection, &Default::default())
                .unwrap();
        assert_ne!(data.identity(), original.data.identity());
        assert_eq!(
            fs::read(original.root.join("u08_public/relics.ron")).unwrap(),
            fs::read(root.join("u08_public/relics.ron")).unwrap()
        );
        println!(
            "U08_LIFECYCLE_CONTENT root={} generation={} parent_generation={}",
            root.display(),
            data.identity().semantic_sha256,
            original.data.identity().semantic_sha256
        );
        PublicContent {
            root,
            selection,
            data,
        }
    })
}

fn event_node(app: &mut CoreApplication) -> Value {
    let map = command(app, json!({"type":"request_map_data"}));
    let node = map["map"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["slot_id"] == "1")
        .unwrap()
        .clone();
    assert_eq!(node["category"], "Event");
    assert_eq!(node["state"], "Available");
    node["id"].clone()
}

#[test]
fn u08_public_unfinished_event_continue_rewinds_variants_and_history_but_keeps_pre_entry_paid_cost()
{
    let fixture = lifecycle_content();
    let mut app = CoreApplication::new(fixture.data.clone(), 0xa508d1);
    command(
        &mut app,
        json!({"type":"start_new_game", "game_mode":"Expedition", "replace_active_run":false}),
    );
    let employees = finish_start(&mut app);
    for employee in &employees[..2] {
        equip_fragment(&mut app, *employee, "u08_public_fragment");
        equip_weapon(&mut app, *employee, "u08_public_weapon");
    }
    let selected_copy = grant_relic(&mut app, "u08_public_blend");
    let already_paid = grant_relic(&mut app, "u08_public_blend");
    equip_relic(&mut app, selected_copy, employees[0], 2);
    equip_relic(&mut app, already_paid, employees[1], 0);
    admin(
        &mut app,
        json!({"type":"admin_grant_relic_corruption_catalyst", "amount":3}),
    );
    admin(
        &mut app,
        json!({"type":"admin_grant_equipment_dust", "amount":11}),
    );
    command(
        &mut app,
        json!({"type":"corrupt_relic", "copy_uuid":already_paid}),
    );
    let before_state = json!(relic_state(&mut app));
    assert_eq!(before_state["resources"]["relic_corruption_catalysts"], 2);
    assert_eq!(before_state["resources"]["equipment_dust"], 11);
    amount(&numeric_plan(&mut app, employees[0]), 180);
    amount(&numeric_plan(&mut app, employees[1]), 210);

    let node = event_node(&mut app);
    command(&mut app, json!({"type":"select_map_node", "node_id":node}));
    let pre_entry = app.export_active_run_checkpoint().unwrap().unwrap();
    let pre_entry_bytes = pre_entry.core_payload_json().to_vec();
    command(&mut app, json!({"type":"confirm_enter_node"}));
    let initial = selected_event(&app);
    assert_eq!(initial["node_id"], node);
    assert_eq!(initial["event"]["event_id"], "u08_public_double_corruption");
    assert_eq!(initial["event"]["current_scene_id"], "first");
    assert!(initial["event"]
        .get("relic_corruption_selections")
        .is_none());
    command(
        &mut app,
        super::event::choose(&initial, Some(selected_copy)),
    );
    let current = selected_event(&app);
    assert_eq!(current["event"]["current_scene_id"], "bridge");
    assert_eq!(
        current["event"]["relic_corruption_selections"],
        json!([{"source_scene_id":"first", "copy_uuid":selected_copy}])
    );
    amount(&numeric_plan(&mut app, employees[0]), 210);
    assert_eq!(
        json!(relic_state(&mut app).resources),
        before_state["resources"]
    );

    let unfinished = app.export_active_run_checkpoint().unwrap().unwrap();
    assert_eq!(unfinished.core_payload_json(), pre_entry_bytes);
    assert_eq!(unfinished.loads_used(), pre_entry.loads_used());
    assert_eq!(
        unfinished.run_peak_certified_power(),
        pre_entry.run_peak_certified_power()
    );
    let mut fresh = CoreApplication::new(fixture.data.clone(), 0xa508d2);
    fresh
        .install_authoritative_player_profile(
            snapshot(&app, ApplicationSnapshotKind::PlayerProfile).into(),
        )
        .unwrap();
    fresh
        .continue_from_active_run_checkpoint(PLAYER, &unfinished)
        .unwrap();
    assert_eq!(unfinished.core_payload_json(), pre_entry_bytes);
    assert_eq!(json!(relic_state(&mut fresh)), before_state);
    assert!(matches!(
        serde_json::from_value::<RunSnapshotDto>(snapshot(&fresh, ApplicationSnapshotKind::Run))
            .unwrap()
            .game_state_context,
        GameStateContextDto::ViewingMap
    ));
    assert!(selected_event(&fresh).is_null());
    amount(&numeric_plan(&mut fresh, employees[0]), 180);
    amount(&numeric_plan(&mut fresh, employees[1]), 210);
    assert_eq!(event_node(&mut fresh), node);

    // The original in-process Event still owns its uncommitted current state.
    // Fresh Continue resets only the newly restored application.
    assert_eq!(selected_event(&app), current);
    amount(&numeric_plan(&mut app, employees[0]), 210);
    command(
        &mut fresh,
        json!({"type":"select_map_node", "node_id":node}),
    );
    command(&mut fresh, json!({"type":"confirm_enter_node"}));
    let restarted = selected_event(&fresh);
    assert_eq!(restarted["event"]["current_scene_id"], "first");
    assert!(restarted["event"]
        .get("relic_corruption_selections")
        .is_none());
    command(
        &mut fresh,
        super::event::choose(&restarted, Some(selected_copy)),
    );
    super::event::advance(&mut fresh);
    let second = selected_event(&fresh);
    assert_eq!(second["event"]["current_scene_id"], "second");
    command(&mut fresh, super::event::choose(&second, None));
    super::event::advance(&mut fresh); // End completes and commits the Event.
    let complete = fresh.export_active_run_checkpoint().unwrap().unwrap();
    let document: Value = serde_json::from_slice(complete.core_payload_json()).unwrap();
    let history = json!([
        {"source_scene_id":"first", "copy_uuid":selected_copy},
        {"source_scene_id":"second", "copy_uuid":null},
    ]);
    assert_eq!(
        document["continuation"]["current"]["event_sessions"][node.as_str().unwrap()]
            ["relic_corruption_selections"],
        history
    );
    let committed = json!(relic_state(&mut fresh));
    assert_eq!(committed["resources"], before_state["resources"]);
    let mut after_completion = CoreApplication::new(fixture.data.clone(), 0xa508d3);
    after_completion
        .install_authoritative_player_profile(
            snapshot(&fresh, ApplicationSnapshotKind::PlayerProfile).into(),
        )
        .unwrap();
    after_completion
        .continue_from_active_run_checkpoint(PLAYER, &complete)
        .unwrap();
    assert_eq!(json!(relic_state(&mut after_completion)), committed);
    amount(&numeric_plan(&mut after_completion, employees[0]), 210);
    let roundtrip = after_completion
        .export_active_run_checkpoint()
        .unwrap()
        .unwrap();
    let roundtrip: Value = serde_json::from_slice(roundtrip.core_payload_json()).unwrap();
    assert_eq!(
        roundtrip["continuation"]["current"]["event_sessions"][node.as_str().unwrap()]
            ["relic_corruption_selections"],
        history
    );
}
