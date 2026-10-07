//! A later authored resource cost fails through the public production session.
use game_core::authoring::{self, CancellationToken, TrialScenario, TrialSession};
use game_core::game::data::content_pack::ContentSelection;
use serde_json::{json, Value};
use std::{fs, path::Path};

fn state(session: &TrialSession) -> Value {
    json!({"index":session.next_step_index(),"work":session.accepted_work(),
        "observation":session.observation().unwrap(),
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
    let content = authoring::validate(
        Path::new(&args[0]),
        &selection,
        &Default::default(),
        &cancel,
    )
    .unwrap();
    let mut session = TrialSession::prepare(
        content,
        TrialScenario::from_json_slice(&raw, &Default::default()).unwrap(),
        Default::default(),
        &cancel,
    )
    .unwrap();
    while session.next_step_index() < 10 {
        session.advance(&cancel).unwrap();
    }
    assert_eq!(
        session
            .observation()
            .unwrap()
            .active_typed_resources()
            .unwrap()
            .entries()
            .count(),
        0
    );
    let before = state(&session);
    let mut failures = Vec::new();
    for _ in 0..2 {
        let error = session
            .advance(&cancel)
            .err()
            .expect("second cost must be insufficient");
        let diagnostic: Value = serde_json::from_str(
            authoring::render_trial_failure(&error, &Default::default(), &cancel)
                .unwrap()
                .document(),
        )
        .unwrap();
        assert_eq!(
            diagnostic["error"]["diagnostic"]["code"],
            "resource_insufficient"
        );
        assert_eq!(state(&session), before);
        failures.push(diagnostic);
    }
    assert_eq!(failures[0], failures[1]);
    fs::write(
        output,
        serde_json::to_vec_pretty(
            &json!({"passed":true,"accepted_before":before,"failures":failures,"preserved":true}),
        )
        .unwrap(),
    )
    .unwrap();
    println!(
        "{}",
        json!({"passed":true,"refusals":2,"accepted_steps":10})
    );
}
