//! Admin enters the authored Event context; every selection and transition below
//! uses the normal public command. Full map entry is a separate acceptance path.
use super::support::*;
use game_core::application::CoreApplication;
use serde_json::{json, Value};
use uuid::Uuid;

pub(super) fn enter(app: &mut CoreApplication, id: &str) -> Value {
    let entered = admin(app, json!({"type":"admin_enter_event", "event_id":id}));
    let selected = selected_event(app);
    assert_eq!(selected["type"], "event");
    assert_eq!(selected["node_id"], entered["node_id"]);
    assert_eq!(selected["event"]["event_id"], id);
    selected
}

pub(super) fn choose(source: &Value, copy: Option<Uuid>) -> Value {
    json!({"type":"choose_relic_corruption", "node_id":source["node_id"],
        "event_id":source["event"]["event_id"], "current_scene_id":source["event"]["current_scene_id"], "copy_uuid":copy})
}

pub(super) fn advance(app: &mut CoreApplication) -> Value {
    let current = selected_event(app);
    command(
        app,
        json!({"type":"advance_event_scene", "node_id":current["node_id"],
        "event_id":current["event"]["event_id"], "current_scene_id":current["event"]["current_scene_id"]}),
    )
}

fn candidates(scene: &Value) -> &[Value] {
    assert_eq!(scene["event"]["interaction"]["type"], "relic_corruption");
    assert_eq!(scene["event"]["interaction"]["can_decline"], true);
    assert_eq!(scene["event"]["interaction"]["catalyst_cost"], 0);
    scene["event"]["interaction"]["candidates"]
        .as_array()
        .unwrap()
}

#[test]
fn u08_public_event_normal_reward_grants_feed_multiple_free_exact_copy_selections() {
    let (mut app, employees) = app();
    let first = claim_relic_reward(&mut app);
    let second = claim_relic_reward(&mut app);
    let already_corrupt = claim_relic_reward(&mut app);
    let unsupported = grant_relic(&mut app, "u08_public_exact");
    admin(
        &mut app,
        json!({"type":"admin_grant_relic_corruption_catalyst", "amount":1}),
    );
    command(
        &mut app,
        json!({"type":"corrupt_relic", "copy_uuid":already_corrupt}),
    );
    equip_relic(&mut app, first, employees[0], 2);
    assert_eq!(
        relic_state(&mut app).resources.relic_corruption_catalysts,
        0
    );

    let initial = enter(&mut app, "u08_public_double_corruption");
    assert!(initial["event"]
        .get("relic_corruption_selections")
        .is_none());
    let offered: Vec<Uuid> = candidates(&initial)
        .iter()
        .map(|row| serde_json::from_value(row["copy"]["copy_uuid"].clone()).unwrap())
        .collect();
    let mut expected = vec![first, second];
    expected.sort();
    assert_eq!(
        offered, expected,
        "only normal copies with an authored corruption are eligible"
    );
    assert!(!offered.contains(&unsupported));
    assert!(!offered.contains(&already_corrupt));
    for row in candidates(&initial) {
        assert_eq!(row["eligibility"], json!({"enabled":true}));
        assert_eq!(row["copy"]["variant"], "Normal");
        assert_eq!(row["effects"].as_array().unwrap().len(), 1);
        assert_eq!(row["effects"][0]["id"], "corrupt_damage");
        assert_eq!(row["effects"][0]["flat"], 40);
        assert_eq!(row["effects"][0]["percent"], 50);
        assert_eq!(
            row["penalties"],
            json!([{"DefenseFlat":{"amount":-15}}, {"MagicResistFlat":{"amount":-10}}])
        );
    }

    let first_command = choose(&initial, Some(first));
    let accepted = command(&mut app, first_command.clone());
    assert_eq!(accepted["event"]["current_scene_id"], "bridge");
    assert_eq!(
        accepted["event"]["relic_corruption_selections"],
        json!([{"source_scene_id":"first", "copy_uuid":first}])
    );
    let state = relic_state(&mut app);
    assert_eq!(state.resources.relic_corruption_catalysts, 0);
    assert_eq!(
        json!(
            state
                .copies
                .iter()
                .find(|copy| copy.copy_uuid == first)
                .unwrap()
                .variant
        ),
        "Corrupt"
    );
    assert_eq!(
        state
            .loadouts
            .iter()
            .find(|row| row.employee_uuid == employees[0])
            .unwrap()
            .slots,
        [None, None, Some(first)]
    );
    advance(&mut app);
    let next = selected_event(&app);
    assert_eq!(next["event"]["current_scene_id"], "second");
    assert_eq!(candidates(&next).len(), 1);
    assert_eq!(candidates(&next)[0]["copy"]["copy_uuid"], json!(second));
    command(&mut app, choose(&next, Some(second)));
    let terminal = selected_event(&app);
    assert_eq!(terminal["event"]["current_scene_id"], "end");
    assert_eq!(
        terminal["event"]["relic_corruption_selections"],
        json!([
            {"source_scene_id":"first", "copy_uuid":first}, {"source_scene_id":"second", "copy_uuid":second}
        ])
    );
    let before_retry = public_roots(&app);
    let retried = command(&mut app, first_command.clone());
    assert_eq!(
        retried["event"], terminal["event"],
        "historical exact retry returns the current scene without rewinding"
    );
    assert_eq!(public_roots(&app), before_retry);
    assert!(execute(&mut app, choose(&initial, Some(second))).is_err());
    assert_eq!(public_roots(&app), before_retry);
    advance(&mut app); // End completes the active node.
    let completed = public_roots(&app);
    assert!(
        execute(&mut app, first_command).is_err(),
        "a completed inactive Event grants no selection authority"
    );
    assert_eq!(public_roots(&app), completed);
}

#[test]
fn u08_public_event_empty_candidates_still_allow_decline_and_exact_historical_retry() {
    let (mut app, _) = app();
    let initial = enter(&mut app, "u08_public_double_corruption");
    assert!(candidates(&initial).is_empty());
    let decline = choose(&initial, None);
    command(&mut app, decline.clone());
    let after = selected_event(&app);
    assert_eq!(after["event"]["current_scene_id"], "bridge");
    assert_eq!(
        after["event"]["relic_corruption_selections"],
        json!([{"source_scene_id":"first", "copy_uuid":null}])
    );
    let before = public_roots(&app);
    assert_eq!(command(&mut app, decline)["event"], after["event"]);
    assert!(execute(&mut app, choose(&initial, Some(Uuid::nil()))).is_err());
    assert_eq!(public_roots(&app), before);
    advance(&mut app);
    let second = selected_event(&app);
    assert!(candidates(&second).is_empty());
    command(&mut app, choose(&second, None));
    assert_eq!(
        selected_event(&app)["event"]["relic_corruption_selections"],
        json!([
            {"source_scene_id":"first", "copy_uuid":null}, {"source_scene_id":"second", "copy_uuid":null}
        ])
    );
    let state = relic_state(&mut app);
    assert!(state.copies.is_empty());
    assert_eq!(state.resources.equipment_dust, 0);
    assert_eq!(state.resources.relic_corruption_catalysts, 0);
}

#[test]
fn u08_public_event_rejects_wrong_authority_and_follows_the_committed_employee_check_branch() {
    let (mut app, employees) = app();
    let copy = claim_relic_reward(&mut app);
    let initial = enter(&mut app, "u08_public_branch_corruption");
    let before = public_roots(&app);
    assert!(
        execute(&mut app, choose(&initial, Some(copy))).is_err(),
        "employee check is not a corruption authority"
    );
    assert_eq!(public_roots(&app), before);
    let resolved = command(
        &mut app,
        json!({"type":"resolve_event_choice_check", "node_id":initial["node_id"],
        "event_id":"u08_public_branch_corruption", "current_scene_id":"check", "choice_id":"attempt", "employee_uuid":employees[0]}),
    );
    let branch = resolved["event"]["current_scene_id"].as_str().unwrap();
    let outcome = resolved["event"]["committed_check"]["outcome"]
        .as_str()
        .unwrap();
    assert_eq!(
        branch,
        if matches!(outcome, "success" | "critical_success") {
            "success"
        } else {
            "failure"
        }
    );
    let scene = selected_event(&app);
    assert_eq!(candidates(&scene).len(), 1);
    let valid = choose(&scene, Some(copy));
    for (field, wrong) in [
        ("node_id", json!(Uuid::nil())),
        ("event_id", json!("u08_public_double_corruption")),
        (
            "current_scene_id",
            json!(if branch == "success" {
                "failure"
            } else {
                "success"
            }),
        ),
        ("copy_uuid", json!(Uuid::nil())),
    ] {
        let mut request = valid.clone();
        request[field] = wrong;
        let before = public_roots(&app);
        assert!(
            execute(&mut app, request).is_err(),
            "forged {field} must fail before mutation"
        );
        assert_eq!(public_roots(&app), before);
    }
    command(&mut app, valid);
    assert_eq!(
        selected_event(&app)["event"]["relic_corruption_selections"],
        json!([{"source_scene_id":branch, "copy_uuid":copy}])
    );
}
