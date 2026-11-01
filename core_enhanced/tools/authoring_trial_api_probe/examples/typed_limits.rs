//! Nonempty resource terminal and Complete receipts through public TrialSession.
use game_core::authoring::{self, CancellationToken, TrialLimits, TrialScenario, TrialSession};
use game_core::game::data::content_pack::ContentSelection;
use serde_json::{json, Value};
use std::{fs, path::Path};

fn state(session: &TrialSession) -> Value {
    json!({"index": session.next_step_index(), "work": session.accepted_work(),
        "receipt_bytes": session.receipt_bytes(),
        "receipts": session.receipt_documents().collect::<Vec<_>>(),
        "observation": session.observation().unwrap()})
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 4, "ROOT SELECTION SCENARIO NEW_OUTPUT");
    let output = Path::new(&args[3]);
    assert!(!output.exists());
    let selection: ContentSelection = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let raw = fs::read(&args[2]).unwrap();
    let cancel = CancellationToken::default();
    let prepare = |limits| {
        let content = authoring::validate(
            Path::new(&args[0]),
            &selection,
            &Default::default(),
            &cancel,
        )
        .unwrap();
        TrialSession::prepare(
            content,
            TrialScenario::from_json_slice(&raw, &Default::default()).unwrap(),
            limits,
            &cancel,
        )
        .unwrap()
    };
    let mut baseline = prepare(TrialLimits::default());
    let mut states = vec![state(&baseline)];
    while !baseline.is_complete() {
        baseline.advance(&cancel).unwrap();
        states.push(state(&baseline));
    }
    assert_eq!(baseline.next_step_index(), 13);
    assert!(!states[states.len() - 2]["observation"]["run_resources"].is_null());
    let entries = states.last().unwrap()["observation"]["run_resources"]["entries"]
        .as_array()
        .unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|entry| entry["balance"] == 5));
    let mut cases = Vec::new();
    for (index, kind) in [
        (10usize, "receipt"),
        (10, "work"),
        (11, "receipt"),
        (11, "work"),
        (12, "receipt"),
        (12, "work"),
        (10, "queue"),
        (11, "clone"),
    ] {
        let before = states.iter().find(|state| state["index"] == index).unwrap();
        let after = states
            .iter()
            .find(|state| state["index"] == index + 1)
            .unwrap();
        let mut limits = TrialLimits::default();
        let step_receipt: Value = after["receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| serde_json::from_str::<Value>(row.as_str().unwrap()).unwrap())
            .find(|row| row["payload"]["kind"] == "step" && row["payload"]["index"] == index)
            .unwrap();
        let expected = match kind {
            "work" => {
                limits.max_total_work = after["work"].as_u64().unwrap() - 1;
                "authoring_work_limit"
            }
            "receipt" => {
                limits.max_receipt_bytes = after["receipt_bytes"].as_u64().unwrap() as usize - 1;
                "authoring_receipt_limit"
            }
            "queue" => {
                limits.execution.max_pending_events = step_receipt["usage"]["pending_events_peak"]
                    .as_u64()
                    .unwrap() as usize
                    - 1;
                "authoring_queue_limit"
            }
            "clone" => {
                limits.execution.max_clone_slots =
                    step_receipt["usage"]["clone_slots_peak"].as_u64().unwrap() as usize - 1;
                "authoring_clone_limit"
            }
            _ => unreachable!(),
        };
        let mut session = prepare(limits);
        while session.next_step_index() < index {
            session.advance(&cancel).unwrap();
        }
        assert_eq!(&state(&session), before);
        let mut diagnostics = Vec::new();
        for _ in 0..2 {
            let failure = session
                .advance(&cancel)
                .err()
                .expect("boundary must reject");
            let document: Value = serde_json::from_str(
                authoring::render_trial_failure(&failure, &Default::default(), &cancel)
                    .unwrap()
                    .document(),
            )
            .unwrap();
            assert_eq!(
                document["error"]["diagnostic"]["code"], expected,
                "{document}"
            );
            assert_eq!(&state(&session), before);
            diagnostics.push(document);
        }
        assert_eq!(diagnostics[0], diagnostics[1]);
        cases.push(
            json!({"index": index, "expected": expected, "before": before,
                "refused_twice": diagnostics, "preserved": true}),
        );
    }
    let mut replay = prepare(TrialLimits::default());
    replay.run_to_end(&cancel).unwrap();
    assert_eq!(state(&replay), state(&baseline));
    let result = json!({"passed": true, "cases": cases, "baseline": states,
        "normal_replay_matches": true, "execution": "default-release public TrialSession"});
    fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    println!(
        "{}",
        json!({"passed":true,"cases":result["cases"].as_array().unwrap().len()})
    );
}
