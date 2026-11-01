//! Finite external combat fixture discovery/replay through public authoring only.
use game_core::{
    authoring::{
        self, CancellationToken, TraceLimits, TraceOptions, TrialLimits, TrialScenario,
        TrialSession,
    },
    game::data::content_pack::ContentSelection,
};
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path};

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(
        args.len(),
        6,
        "ROOT SELECTION SCENARIO NEW_OUTPUT discovery|replay off|on|gap"
    );
    let output = Path::new(&args[3]);
    fs::create_dir(output).unwrap();
    let selection: ContentSelection = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let raw = fs::read(&args[2]).unwrap();
    let mut input: Value = serde_json::from_slice(&raw).unwrap();
    let cancel = CancellationToken::default();
    let content = authoring::validate(
        Path::new(&args[0]),
        &selection,
        &Default::default(),
        &cancel,
    )
    .unwrap();
    let identity = serde_json::to_value(content.token().identity()).unwrap();
    let mut limits = TrialLimits::default();
    limits.trace = match args[5].as_str() {
        "off" => TraceOptions::default(),
        "on" => TraceOptions {
            limits: Some(TraceLimits::default()),
        },
        "gap" => TraceOptions {
            limits: Some(TraceLimits {
                max_bytes: 1,
                max_row_bytes: 1,
                ..Default::default()
            }),
        },
        _ => panic!("unknown trace mode"),
    };
    let scenario = TrialScenario::from_json_slice(&raw, &Default::default()).unwrap();
    let mut session = TrialSession::prepare(content, scenario, limits, &cancel).unwrap();
    let mut first_sourced = Value::Null;
    let mut first_terminal = Value::Null;
    let mut failure = None;
    while !session.is_complete() {
        if let Err(error) = session.advance(&cancel) {
            failure = Some(error);
            break;
        }
        let observation = session.observation().unwrap();
        let mandatory = observation.active().or_else(|| {
            observation
                .last_terminal()
                .map(|terminal| terminal.mandatory())
        });
        if first_sourced.is_null() {
            if let Some(witness) = mandatory.and_then(|battle| battle.latest_nonzero_sourced_hp()) {
                first_sourced = serde_json::to_value(witness).unwrap();
            }
        }
        if let Some(terminal) = observation.last_terminal() {
            if first_terminal.is_null() {
                first_terminal = serde_json::to_value(terminal).unwrap();
            }
            if args[4] == "discovery" {
                break;
            }
        }
    }
    let mut receipt = fs::File::create(output.join("receipts.ndjson")).unwrap();
    for document in session.receipt_documents() {
        writeln!(receipt, "{document}").unwrap();
    }
    let final_observation = serde_json::to_value(session.observation().unwrap()).unwrap();
    let trace = session.trace().unwrap();
    for (window, view) in [
        ("active", trace.active()),
        ("terminal", trace.last_terminal()),
    ] {
        if let Some(view) = view {
            fs::write(output.join(format!("{window}_trace.ndjson")), view.ndjson()).unwrap();
            fs::write(
                output.join(format!("{window}_trace.json")),
                serde_json::to_vec_pretty(&json!({
                    "key":view.key(),"enabled":view.enabled(),"rows":view.retained_rows(),
                    "last":view.last_retained_ordinal(),"gap":view.gap(),
                }))
                .unwrap(),
            )
            .unwrap();
        }
    }
    let diagnostic = failure.as_ref().map(|failure| {
        serde_json::from_str::<Value>(
            authoring::render_trial_failure(failure, &Default::default(), &cancel)
                .unwrap()
                .document(),
        )
        .unwrap()
    });
    let summary = json!({"mode":args[4],"identity":identity,"input_sha256":session.input_sha256(),
        "accepted_steps":session.next_step_index(),"is_complete":session.is_complete(),
        "first_sourced":first_sourced,"first_terminal":first_terminal,"final_observation":final_observation,
        "accepted_work":session.accepted_work(),"receipt_bytes":session.receipt_bytes(),"failure":diagnostic});
    fs::write(
        output.join("RESULT.json"),
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
    if args[4] == "discovery" && !first_terminal.is_null() {
        let prep = input["preparation_steps"].as_array().unwrap().len();
        input["steps"]
            .as_array_mut()
            .unwrap()
            .truncate(session.next_step_index() - prep);
        fs::write(
            output.join("terminal_scenario.json"),
            serde_json::to_vec(&input).unwrap(),
        )
        .unwrap();
        input["steps"]
            .as_array_mut()
            .unwrap()
            .push(json!({"kind":"command","command":{"type":"complete_combat_result"}}));
        fs::write(
            output.join("consumed_scenario.json"),
            serde_json::to_vec(&input).unwrap(),
        )
        .unwrap();
    }
    println!(
        "{}",
        json!({"accepted_steps":session.next_step_index(),"is_complete":session.is_complete(),
        "sourced":!first_sourced.is_null(),"terminal":!first_terminal.is_null(),"failure":diagnostic})
    );
    assert!(
        failure.is_none(),
        "actual trial failed; original diagnostic and accepted receipts saved"
    );
    assert!(
        !first_terminal.is_null(),
        "finite fixture did not reach an actual terminal"
    );
    assert!(
        !first_sourced.is_null(),
        "terminal fixture has no actual sourced HP change"
    );
}
