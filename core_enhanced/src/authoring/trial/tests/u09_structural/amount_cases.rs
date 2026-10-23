//! Actual public retained queries for the generated 40 -> 75 damage occurrence.
use super::*;
use crate::authoring::{AmountBattle, AmountQuery};

#[test]
fn rv004_external_structural_amount_active_exact_preview_explain_are_read_only() {
    let (_owned, token, doc) = if let Some(input) = std::env::var_os("U09_PUBLIC_INPUT") {
        let input = PathBuf::from(input);
        let selection =
            serde_json::from_slice(&fs::read(input.join("selection.json")).unwrap()).unwrap();
        let expected: Value =
            serde_json::from_slice(&fs::read(input.join("content_manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest(&input.join("content")), expected);
        let token = crate::game::data::content_pack::load(
            &input.join("content"),
            &selection,
            &Default::default(),
        )
        .unwrap();
        let doc = serde_json::from_slice(&fs::read(input.join("scenario.json")).unwrap()).unwrap();
        (None, token, doc)
    } else {
        let (fixture, token, doc) = fixture();
        let doc = scenario(&token, doc);
        (Some(fixture), token, doc)
    };
    let cancel = CancellationToken::default();
    let session = simulate(
        fixture_content(&token),
        checked(&doc),
        TrialLimits {
            amount_observation: true,
            ..Default::default()
        },
        &cancel,
    )
    .unwrap();
    assert_plan(&session.step(17).unwrap().public_result().unwrap()["payload"]);
    let before = session.receipt_documents().collect::<Vec<_>>();
    let work = session.accepted_work();
    let state = session
        .state
        .application
        .certified_input_fingerprint()
        .unwrap();
    let limits = ExecutionLimits::default();
    let latest = AmountQuery::Latest {
        battle: AmountBattle::Active,
    };
    let preview: Value = serde_json::from_str(
        session
            .preview_amount(latest, &limits, &cancel)
            .unwrap()
            .document(),
    )
    .unwrap();
    let explain: Value = serde_json::from_str(
        session
            .explain_amount(latest, &limits, &cancel)
            .unwrap()
            .document(),
    )
    .unwrap();
    let exact = AmountQuery::Exact {
        battle_key: session
            .observation()
            .unwrap()
            .active()
            .unwrap()
            .key()
            .unwrap(),
        evaluation_ordinal: preview["identity"]["evaluation_ordinal"].as_u64().unwrap(),
    };
    let exact_preview: Value = serde_json::from_str(
        session
            .preview_amount(exact, &limits, &cancel)
            .unwrap()
            .document(),
    )
    .unwrap();
    let exact_explain: Value = serde_json::from_str(
        session
            .explain_amount(exact, &limits, &cancel)
            .unwrap()
            .document(),
    )
    .unwrap();
    // Preserve query evidence before assertions. Values below are authored
    // arithmetic: original23+7=30; generated(40+10)*150/100=75.
    if let Some(export) = std::env::var_os("U09_PUBLIC_CLI_EXPORT") {
        let export = PathBuf::from(export);
        let dir = export.with_file_name(format!(
            "{}_amount",
            export.file_name().unwrap().to_str().unwrap()
        ));
        fs::create_dir(&dir).unwrap();
        let mut receipts = before.join("\n");
        receipts.push('\n');
        write_new(&dir.join("trial.jsonl"), receipts.as_bytes());
        write_json(
            &dir.join("queries.json"),
            &json!({"preview":preview,"explain":explain,"exact_preview":exact_preview,"exact_explain":exact_explain}),
        );
    }
    for result in [&preview, &explain, &exact_preview, &exact_explain] {
        assert_eq!(result["status"], "evaluated", "{result}");
        assert_eq!(result["basis"], "recorded_evaluation");
        assert_eq!(
            result["typed_base_amount"],
            json!({"value":40,"unit":"Magnitude"})
        );
        assert_eq!(
            result["narrowed_base_amount"],
            json!({"kind":"damage","value":40})
        );
        assert_eq!(
            result["effective_relic_amount"],
            json!({"kind":"damage","value":75})
        );
        assert_eq!(result["identity"], preview["identity"]);
        assert_eq!(result["trace"], preview["trace"]);
        let fingerprint = result["semantic_fingerprint"].as_str().unwrap();
        assert_eq!(fingerprint.len(), 64);
        assert!(fingerprint.bytes().all(|c| c.is_ascii_hexdigit()));
        assert!(fingerprint.bytes().any(|c| c != b'0'));
    }
    assert_eq!(explain["static_plan"]["evaluation"]["performed"], false);
    assert_eq!(exact_explain["static_plan"], explain["static_plan"]);
    assert_eq!(session.receipt_documents().collect::<Vec<_>>(), before);
    assert_eq!(session.accepted_work(), work);
    assert_eq!(
        session
            .state
            .application
            .certified_input_fingerprint()
            .unwrap(),
        state
    );
    println!("RV004 actual_public_amount_queries active_and_exact_preview_explain base40_effective75=true receipts_work_state_unchanged=true");
}
