//! Actual external A/B replacement failure and recovery, using public APIs only.
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

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(args.len(), 3, "FIXTURE_ROOT VERIFIED_REPLAY NEW_OUTPUT");
    let fixture = Path::new(&args[0]);
    let replay = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    fs::create_dir(output).unwrap();
    let raw = fs::read(replay.join("scenario.json")).unwrap();
    let document: Value = serde_json::from_slice(&raw).unwrap();
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
        let baseline_b =
            TrialSession::prepare(load("b"), parse(), settings.clone(), &cancel).unwrap();
        let b_work = baseline_b.accepted_work();
        let b_bytes = baseline_b.receipt_bytes();
        let mut exact = settings.clone();
        exact.execution.max_work = b_work;
        exact.max_receipt_bytes = b_bytes;
        let exact_b = TrialSession::prepare(load("b"), parse(), exact, &cancel).unwrap();
        assert_eq!(receipts(&exact_b), receipts(&baseline_b));
        let mut session =
            TrialSession::prepare(load("a"), parse(), settings.clone(), &cancel).unwrap();
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
        let before = capture(&session, &cancel);
        save(&directory.join("a_before.json"), &before);
        let cancelled = CancellationToken::default();
        cancelled.cancel();
        for repeat in 0..2 {
            // Acquisition failure occurs before any replacement token exists.
            let missing = ContentSelection {
                packs: vec!["missing-pack".into()],
                winners: vec![],
            };
            let error = match authoring::validate(
                &fixture.join("b"),
                &missing,
                &Default::default(),
                &cancel,
            ) {
                Ok(_) => panic!("missing pack unexpectedly validated"),
                Err(error) => error,
            };
            assert_eq!(capture(&session, &cancel), before);
            let failure_json: Value = serde_json::from_str(
                authoring::render_error(&error, &Default::default(), &cancel)
                    .unwrap()
                    .document(),
            )
            .unwrap();
            reports.push(
                json!({"mode":mode,"repeat":repeat,"case":"acquisition","diagnostic":failure_json}),
            );
            for case in [
                "cancel",
                "initial_work",
                "clone",
                "authority",
                "late_command",
                "not_ready",
                "ready_work",
                "ready_receipt",
            ] {
                let mut candidate_limits = settings.clone();
                let mut candidate_document = document.clone();
                let mut expected_position = TrialPosition::Initial;
                match case {
                    "cancel" => {}
                    "initial_work" => candidate_limits.execution.max_work = 1,
                    "clone" => candidate_limits.execution.max_clone_slots = 1,
                    "authority" => {
                        let command = &mut candidate_document["preparation_steps"][6];
                        command["bindings"] = json!([]);
                        command["context"] = json!({"expected_battle_authority":{
                            "battle_uuid":"00000000-0000-0000-0000-000000000000","battle_state_revision":0}});
                        expected_position = TrialPosition::Step { index: 6 };
                    }
                    "late_command" => {
                        candidate_document["preparation_steps"]
                            .as_array_mut()
                            .unwrap()
                            .push(json!({"kind":"command","command":{"type":"unknown_command"}}));
                        expected_position = TrialPosition::Step { index: 9 };
                    }
                    "not_ready" => {
                        candidate_document["preparation_steps"] = json!([]);
                        // Avoid invalid forward bindings in the irrelevant execution suffix.
                        candidate_document["steps"] = json!([]);
                        expected_position = TrialPosition::PreparationReady;
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
                let candidate_raw = if matches!(case, "authority" | "late_command" | "not_ready") {
                    serde_json::to_vec(&candidate_document).unwrap()
                } else {
                    raw.clone()
                };
                let candidate =
                    TrialScenario::from_json_slice(&candidate_raw, &Default::default()).unwrap();
                let failure = session
                    .replace(
                        load("b"),
                        candidate,
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
            }
        }
        session.run_to_end(&cancel).unwrap();
        let a_result = receipts(&session);
        assert_eq!(
            a_result,
            fs::read_to_string(replay.join(format!("a_{mode}/receipts.ndjson"))).unwrap()
        );
        fs::write(directory.join("a_continued.ndjson"), a_result).unwrap();
        let a_identity = serde_json::to_value(session.content_identity()).unwrap();
        session
            .replace(load("b"), parse(), settings.clone(), &cancel)
            .unwrap();
        assert_ne!(
            serde_json::to_value(session.content_identity()).unwrap(),
            a_identity
        );
        assert_eq!(capture(&session, &cancel), capture(&baseline_b, &cancel));
        session.run_to_end(&cancel).unwrap();
        let b_result = receipts(&session);
        assert_eq!(
            b_result,
            fs::read_to_string(replay.join(format!("b_{mode}/receipts.ndjson"))).unwrap()
        );
        fs::write(directory.join("b_replaced.ndjson"), b_result).unwrap();
        save(&directory.join("b_final.json"), &capture(&session, &cancel));
        println!(
            "{}",
            json!({"mode":mode,"a_continuation_equal":true,"b_replacement_equal":true,"b_prepare_work":b_work,"b_prepare_receipt_bytes":b_bytes})
        );
    }
    save(
        &output.join("RESULT.json"),
        &json!({"passed":true,"modes":3,"failed_attempts":reports.len(),
        "acquisition_failures":6,"replacement_failures":48,"successful_replacements":3,
        "a_continuation_replays":3,"b_completed_replays":3,"attempts":reports}),
    );
}
