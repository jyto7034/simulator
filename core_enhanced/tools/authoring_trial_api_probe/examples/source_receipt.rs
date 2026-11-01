//! Exact byte-source binding, limits and replacement through public production APIs.
use game_core::{
    application::ApplicationSnapshotKind,
    authoring::{
        self, AuthoringContent, CancellationToken, ExecutionPhase, TraceLimits, TraceOptions,
        TrialLimits, TrialPosition, TrialScenario, TrialSession,
    },
    game::data::content_pack::ContentSelection,
};
use serde_json::{json, Value};
use std::{fs, path::Path};

fn receipts(session: &TrialSession) -> String {
    let mut text = session.receipt_documents().collect::<Vec<_>>().join("\n");
    text.push('\n');
    text
}
fn capture(session: &TrialSession, cancel: &CancellationToken) -> Value {
    let reconnect = session.reconnect(0, &Default::default(), cancel);
    if session.observation().unwrap().active().is_some() {
        assert!(reconnect.is_ok(), "active battle reconnect must succeed");
    }
    let reconnect = match reconnect {
        Ok(document) => json!({"ok":document.document()}),
        Err(error) => {
            assert!(
                matches!(&error, authoring::AuthoringError::Application { phase:ExecutionPhase::Inspection, diagnostic }
                if diagnostic.code() == "invalid_action")
            );
            json!({"error":error})
        }
    };
    let trace = session.trace().unwrap();
    let windows: Vec<_> = [trace.active(), trace.last_terminal()].into_iter().map(|window| window.map(|view| json!({
        "key":view.key(),"enabled":view.enabled(),"rows":view.retained_rows(),"last":view.last_retained_ordinal(),
        "gap":view.gap(),"bytes":std::str::from_utf8(view.ndjson()).unwrap(),
    }))).collect();
    let limits = session.limits();
    let optional_limits = limits.trace.limits.map(|l| {
        json!({"max_rows":l.max_rows,"max_bytes":l.max_bytes,
        "max_row_bytes":l.max_row_bytes,"max_attempt_work":l.max_attempt_work})
    });
    json!({"identity":session.content_identity(),"initial":session.initial(),"input_sha256":session.input_sha256(),
        "next_step_index":session.next_step_index(),"is_complete":session.is_complete(),"accepted_work":session.accepted_work(),
        "copied_bytes":session.copied_bytes(),"receipt_bytes":session.receipt_bytes(),"receipts":receipts(session),
        "lifecycle":session.lifecycle(),"observation":session.observation().unwrap(),"trace":windows,
        "limits":{"execution":limits.execution,"total_work":limits.max_total_work,"receipt_bytes":limits.max_receipt_bytes,"trace":optional_limits},
        "snapshot":session.snapshot(ApplicationSnapshotKind::Run,&Default::default(),cancel).unwrap().document(),
        "reconnect":reconnect})
}
fn save(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn check_sources(session: &TrialSession, expected: &Value) {
    for (index, document) in session.receipt_documents().enumerate() {
        let document: Value = serde_json::from_str(document).unwrap();
        assert_eq!(
            &document["payload"]["source_receipt"], expected,
            "receipt {index}"
        );
    }
}
fn without_source(document: &str) -> Vec<Value> {
    document
        .lines()
        .map(|line| {
            let mut value: Value = serde_json::from_str(line).unwrap();
            assert!(value["payload"]
                .as_object_mut()
                .unwrap()
                .remove("source_receipt")
                .is_some());
            value
        })
        .collect()
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(
        args.len(),
        3,
        "COPIED_FIXTURE_ROOT SCENARIO_JSON NEW_OUTPUT"
    );
    let fixture = Path::new(&args[0]);
    let raw = fs::read(&args[1]).unwrap();
    let output = Path::new(&args[2]);
    fs::create_dir(output).unwrap();
    let selection: ContentSelection =
        serde_json::from_slice(&fs::read(fixture.join("selection.json")).unwrap()).unwrap();
    let cancel = CancellationToken::default();
    let load = |generation: &str| -> AuthoringContent {
        authoring::validate(
            &fixture.join(generation),
            &selection,
            &Default::default(),
            &cancel,
        )
        .unwrap()
    };
    let parse = || TrialScenario::from_json_slice(&raw, &Default::default()).unwrap();
    let mut reports = Vec::new();
    let mut observations = Vec::new();
    let mut first_a = None;
    let mut first_b = None;
    for mode in ["off", "on", "gap"] {
        let mut settings = TrialLimits::default();
        settings.trace = TraceOptions {
            limits: match mode {
                "off" => None,
                "on" => Some(TraceLimits::default()),
                _ => Some(TraceLimits {
                    max_bytes: 1,
                    max_row_bytes: 1,
                    ..Default::default()
                }),
            },
        };
        let directory = output.join(mode);
        fs::create_dir(&directory).unwrap();
        let a = load("A");
        let b = load("B");
        let a_source = serde_json::to_value(a.token().source_receipt()).unwrap();
        let b_source = serde_json::to_value(b.token().source_receipt()).unwrap();
        let identity = serde_json::to_value(a.token().identity()).unwrap();
        assert_eq!(
            identity,
            serde_json::to_value(b.token().identity()).unwrap()
        );
        assert_ne!(a_source, b_source);
        // Catalog schema 3 adds Resources: 25 catalog files plus the manifest.
        assert_eq!(a_source["files"].as_array().unwrap().len(), 26);
        save(&directory.join("a_source.json"), &a_source);
        save(&directory.join("b_source.json"), &b_source);
        let baseline_b = TrialSession::prepare(b, parse(), settings.clone(), &cancel).unwrap();
        check_sources(&baseline_b, &b_source);
        let b_work = baseline_b.accepted_work();
        let b_bytes = baseline_b.receipt_bytes();
        let initial_bytes = baseline_b.initial_receipt().document().len();
        let mut exact = settings.clone();
        exact.execution.max_work = b_work;
        exact.max_receipt_bytes = b_bytes;
        let exact_b = TrialSession::prepare(load("B"), parse(), exact, &cancel).unwrap();
        assert_eq!(receipts(&exact_b), receipts(&baseline_b));
        let mut initial_exact = settings.clone();
        initial_exact.max_receipt_bytes = initial_bytes;
        let next_failure =
            TrialSession::prepare(load("B"), parse(), initial_exact, &cancel).unwrap_err();
        assert_eq!(next_failure.position(), TrialPosition::Step { index: 0 });
        // This is a caller-owned isolated fixture mutation after token acquisition.
        // Production must serialize the captured token, never reopen this path.
        let source_file = fixture.join("A/official/data/skills/base.ron");
        let original = fs::read_to_string(&source_file).unwrap();
        assert_eq!(original.matches("// audit byte identity A").count(), 1);
        let edited = original.replacen("// audit byte identity A", "// audit byte identity C", 1);
        assert_eq!(original.len(), edited.len());
        fs::write(&source_file, &edited).unwrap();
        let later = load("A");
        assert_eq!(
            identity,
            serde_json::to_value(later.token().identity()).unwrap()
        );
        assert_ne!(
            a_source,
            serde_json::to_value(later.token().source_receipt()).unwrap()
        );
        let mut session = TrialSession::prepare(a, parse(), settings.clone(), &cancel).unwrap();
        check_sources(&session, &a_source);
        fs::write(&source_file, &original).unwrap();
        assert_eq!(fs::read_to_string(&source_file).unwrap(), original);
        session.advance(&cancel).unwrap();
        session.advance(&cancel).unwrap();
        assert_eq!(session.next_step_index(), 11);
        assert!(session
            .observation()
            .unwrap()
            .active()
            .unwrap()
            .latest_nonzero_sourced_hp()
            .is_some());
        check_sources(&session, &a_source);
        let before = capture(&session, &cancel);
        save(&directory.join("a_before.json"), &before);
        let cancelled = CancellationToken::default();
        cancelled.cancel();
        for repeat in 0..2 {
            for case in [
                "cancel",
                "initial_work",
                "initial_receipt",
                "initial_result",
                "ready_work",
                "ready_receipt",
            ] {
                let mut candidate_limits = settings.clone();
                let mut expected_position = TrialPosition::Initial;
                match case {
                    "cancel" => {}
                    "initial_work" => candidate_limits.execution.max_work = 1,
                    "initial_receipt" => candidate_limits.max_receipt_bytes = initial_bytes - 1,
                    "initial_result" => {
                        candidate_limits.execution.max_result_bytes = initial_bytes - 1
                    }
                    "ready_work" => {
                        candidate_limits.execution.max_work = b_work - 1;
                        expected_position = TrialPosition::PreparationReady;
                    }
                    "ready_receipt" => {
                        candidate_limits.max_receipt_bytes = b_bytes - 1;
                        expected_position = TrialPosition::PreparationReady;
                    }
                    _ => unreachable!(),
                }
                let failure = session
                    .replace(
                        load("B"),
                        parse(),
                        candidate_limits,
                        if case == "cancel" {
                            &cancelled
                        } else {
                            &cancel
                        },
                    )
                    .unwrap_err();
                let diagnostic: Value = serde_json::from_str(
                    authoring::render_trial_failure(&failure, &Default::default(), &cancel)
                        .unwrap()
                        .document(),
                )
                .unwrap();
                reports
                    .push(json!({"mode":mode,"repeat":repeat,"case":case,"diagnostic":diagnostic}));
                save(&output.join("attempts.json"), &json!(reports));
                assert_eq!(failure.position(), expected_position, "{mode}/{case}");
                assert_eq!(failure.operation_phase(), ExecutionPhase::TrialPreparation);
                if matches!(case, "ready_work" | "ready_receipt") {
                    assert_eq!(diagnostic["error"]["diagnostic"]["phase"], "receipt");
                }
                assert_eq!(capture(&session, &cancel), before, "{mode}/{case}");
                check_sources(&session, &a_source);
            }
        }
        session.run_to_end(&cancel).unwrap();
        check_sources(&session, &a_source);
        let a_result = receipts(&session);
        let a_reference =
            authoring::simulate(load("A"), parse(), settings.clone(), &cancel).unwrap();
        assert_eq!(receipts(&a_reference), a_result);
        fs::write(directory.join("a.ndjson"), &a_result).unwrap();
        session
            .replace(load("B"), parse(), settings.clone(), &cancel)
            .unwrap();
        assert_eq!(
            serde_json::to_value(session.content_identity()).unwrap(),
            identity
        );
        check_sources(&session, &b_source);
        assert_eq!(receipts(&session), receipts(&baseline_b));
        session.run_to_end(&cancel).unwrap();
        check_sources(&session, &b_source);
        let b_result = receipts(&session);
        let b_reference =
            authoring::simulate(load("B"), parse(), settings.clone(), &cancel).unwrap();
        assert_eq!(receipts(&b_reference), b_result);
        assert_ne!(a_result, b_result);
        assert_eq!(without_source(&a_result), without_source(&b_result));
        fs::write(directory.join("b.ndjson"), &b_result).unwrap();
        if let Some(first) = &first_a {
            assert_eq!(first, &a_result);
        }
        if let Some(first) = &first_b {
            assert_eq!(first, &b_result);
        }
        first_a = Some(a_result);
        first_b = Some(b_result);
        let trace = session.trace().unwrap();
        let terminal = trace.last_terminal().unwrap();
        match mode {
            "off" => assert!(!terminal.enabled()),
            "on" => assert!(terminal.retained_rows() > 0),
            _ => {
                assert_eq!(terminal.retained_rows(), 0);
                assert!(terminal.gap().unwrap().dropped_rows > 0);
            }
        }
        observations.push(json!({"mode":mode,"initial_bytes":initial_bytes,"ready_bytes":b_bytes,"ready_work":b_work,"final_work":session.accepted_work(),"final_bytes":session.receipt_bytes(),"steps":session.next_step_index(),"receipts":session.receipt_documents().count(),"source_files":25,"source_capture_after_caller_edit":true,"trace_rows":terminal.retained_rows(),"trace_gap":terminal.gap(),"lifecycle":session.lifecycle(),"observation":session.observation().unwrap()}));
        save(&output.join("observations.json"), &json!(observations));
    }
    assert_eq!(reports.len(), 36);
    save(
        &output.join("receipt.json"),
        &json!({"passed":true,"failed_replacements":reports.len(),"successful_replacements":3,"exact_ready_boundaries":3,"exact_initial_next_step_rejection":3,"post_load_caller_edit_proofs":3,"a_independent_continuations":3,"b_independent_replays":3,"same_semantic_different_bytes":true,"only_source_receipt_differs_across_equal_byte_length_inputs":true,"off_on_gap_receipts_equal_per_source":true,"observations":observations}),
    );
    println!("source byte receipt API: 36 rejected replacements, 3 successful swaps; exact limits/caller edits/replay/trace parity pass");
}
