//! Actual large resource state: finite refusal preserves accepted state; explicit
//! shorter clock commands complete without changing the engineering limits.
use game_core::authoring::{self, CancellationToken, TrialScenario, TrialSession};
use game_core::game::data::content_pack::ContentSelection;
use serde_json::{json, Value};
use std::{fs, path::Path};

fn state(session: &TrialSession) -> Value {
    json!({"index":session.next_step_index(),"work":session.accepted_work(),
        "receipt_bytes":session.receipt_bytes(),"observation":session.observation().unwrap(),
        "receipts":session.receipt_documents().collect::<Vec<_>>()})
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 4, "ROOT SELECTION SCENARIO NEW_OUTPUT");
    let output = Path::new(&args[3]);
    assert!(!output.exists());
    let selection: ContentSelection = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let raw = fs::read(&args[2]).unwrap();
    let cancel = CancellationToken::default();
    let prepare = |input: &[u8]| {
        let content = authoring::validate(
            Path::new(&args[0]),
            &selection,
            &Default::default(),
            &cancel,
        )
        .unwrap();
        TrialSession::prepare(
            content,
            TrialScenario::from_json_slice(input, &Default::default()).unwrap(),
            Default::default(),
            &cancel,
        )
        .unwrap()
    };
    let mut original = prepare(&raw);
    while original.next_step_index() < 11 {
        original.advance(&cancel).unwrap();
    }
    let view = original.observation().unwrap();
    let resources = view.active_typed_resources().unwrap();
    assert_eq!(resources.entries().count(), 256);
    assert!(resources.entries().all(|(_, value)| value.balance == 5));
    let before = state(&original);
    let mut failures = Vec::new();
    for _ in 0..2 {
        let failure = original
            .advance(&cancel)
            .err()
            .expect("long clock must exceed native clone budget");
        let diagnostic: Value = serde_json::from_str(
            authoring::render_trial_failure(&failure, &Default::default(), &cancel)
                .unwrap()
                .document(),
        )
        .unwrap();
        assert_eq!(
            diagnostic["error"]["diagnostic"]["code"],
            "battle_runtime_clone_limit"
        );
        let details = &diagnostic["error"]["diagnostic"]["details"];
        assert!(details["requested"].as_u64().unwrap() > details["limit"].as_u64().unwrap());
        assert_eq!(state(&original), before);
        failures.push(diagnostic);
    }
    assert_eq!(failures[0], failures[1]);
    let mut input: Value = serde_json::from_slice(&raw).unwrap();
    let steps = input["steps"].as_array_mut().unwrap();
    assert_eq!(steps[2]["clock"]["delta_ms"], 60000);
    let mut short = steps[2].clone();
    short["clock"]["delta_ms"] = json!(5000);
    // The same combat fixture's resource_32 reference terminates at 27450ms.
    // Four explicit 5s clocks after the accepted 10s prefix cover that terminal.
    steps.splice(2..3, std::iter::repeat(short).take(4));
    let split = serde_json::to_vec(&input).unwrap();
    let split_file = output.with_extension("scenario.json");
    assert!(!split_file.exists());
    fs::write(&split_file, &split).unwrap();
    let mut replay = prepare(&split);
    replay.run_to_end(&cancel).unwrap();
    let after = state(&replay);
    let entries = after["observation"]["run_resources"]["entries"]
        .as_array()
        .unwrap();
    assert_eq!(entries.len(), 256);
    assert!(entries.iter().all(|entry| entry["balance"] == 5));
    assert!(after["observation"]["active_typed_resources"].is_null());
    assert_eq!(after["observation"]["last_terminal"]["actual_time_ms"],27450);
    let mut repeat = prepare(&split);
    repeat.run_to_end(&cancel).unwrap();
    assert_eq!(state(&repeat), after);
    assert_eq!(state(&original), before);
    let result = json!({"passed":true,"original_before":before,"failures":failures,
        "split_scenario":split_file,"split_final":after,"split_replay_matches":true,
        "bounds_unchanged":true,"resource_entries":256});
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    println!(
        "{}",
        json!({"passed":true,"refusals":2,"split_steps":replay.next_step_index(),"resource_entries":256})
    );
}
