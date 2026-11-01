use super::*;
use sha2::{Digest, Sha256};

const YAML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/woodsman_monk.skill.yaml"
));
const JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/woodsman_monk.json"
));

#[test]
fn yaml_original_compiles_to_identical_native_meaning_with_original_source_identity() {
    assert_eq!(
        parse_yaml(YAML).unwrap(),
        serde_json::from_str::<serde_json::Value>(JSON).unwrap()
    );
    let artifact = compile_yaml(YAML).unwrap();
    assert_eq!(artifact.native_ron, compile_json(JSON).unwrap().native_ron);
    assert_eq!(
        artifact.source_digest(),
        format!("{:x}", Sha256::digest(YAML.as_bytes()))
    );
    assert_ne!(
        artifact.source_digest(),
        compile_json(JSON).unwrap().source_digest()
    );
    Artifact::from_ron(&artifact.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
}

#[test]
fn yaml_scalar_and_rejection_contract_matches_shared_corpus() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/skill_composition/yaml_cases.json"
    )))
    .unwrap();
    for case in cases.as_array().unwrap() {
        let result = parse_yaml(case["source"].as_str().unwrap());
        if case["accepted"] == true {
            assert_eq!(result.unwrap(), case["value"], "{case}");
        } else {
            assert!(result.is_err(), "{case}");
        }
    }
    let deep = format!("{}1{}", "[".repeat(2048), "]".repeat(2048));
    assert!(parse_yaml(&deep).is_err());
}

#[test]
fn yaml_compiler_keeps_unresolved_type_and_reference_errors_separate() {
    for (needle, replacement, expected) in [
        (
            "Shared: burst-power",
            "Shared: missing-power",
            DiagnosticCode::Reference,
        ),
        (
            "Shared: burst-power",
            "Unresolved: 위력 미정",
            DiagnosticCode::Unresolved,
        ),
        (
            "Shared: burst-power",
            "Literal: { Count: 3 }",
            DiagnosticCode::Type,
        ),
    ] {
        assert!(YAML.contains(needle));
        let source = YAML.replace(needle, replacement);
        let error = compile_yaml(&source).unwrap_err();
        assert_eq!(error.code, expected);
        assert!(error.source.definition.starts_with("skills[1]"));
        assert_eq!(
            error.source_digest,
            format!("{:x}", Sha256::digest(source.as_bytes()))
        );
    }
}

#[test]
fn source_bridge_echoes_identity_and_never_emits_artifact_for_rejection() {
    let response = compile_request(CompileRequest {
        source: "version: [".into(),
        epoch: 2,
        revision: 3,
        request_id: 4,
    })
    .unwrap();
    assert_eq!(
        (response.epoch, response.revision, response.request_id),
        (2, 3, 4)
    );
    let value = serde_json::to_value(response).unwrap();
    assert_eq!(value["status"], "rejected");
    assert!(value.get("artifact").is_none());
    assert!(compile_request(CompileRequest {
        source: YAML.into(),
        epoch: u64::MAX,
        revision: 0,
        request_id: 0
    })
    .is_err());
}

#[test]
fn run_bridge_rejects_mismatched_definition_before_reading_content() {
    let response = run_composition(
        CompositionRunRequest {
            source: YAML.into(),
            epoch: 2,
            revision: 3,
            request_id: 9,
            artifact_digest: "wrong-definition".into(),
            root: "not-a-content-root".into(),
            selection: crate::game::data::content_pack::ContentSelection {
                packs: vec!["official".into()],
                winners: vec![],
            },
            scenario: "{}".into(),
            lab: None,
        },
        &crate::authoring::CancellationToken::default(),
    )
    .unwrap();
    let result = serde_json::to_value(response).unwrap();
    assert_eq!(result["status"], "rejected");
    assert_eq!(result["failure"]["stage"], "definition");
    assert_eq!(result["accepted_steps"], 0);
    assert!(result["identity"].is_null());
    assert_eq!(result["revision"], 3);
}

const NATIVE_DRAFT_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/skill_composition/scarecrow_headshot.skill.yaml"
));
const PREPARED_NUMERATOR: &str =
    "/values/0/value/Effects/0/op/Sustained/definition/PreparedState/numerator";

#[test]
fn draft_leaf_native_holes_keep_structure_and_exact_diagnostics_until_resolved() {
    let complete = parse_yaml(NATIVE_DRAFT_SOURCE).unwrap();
    let expected = compile_yaml(NATIVE_DRAFT_SOURCE).unwrap();
    for (pointer, path) in [
        (
            PREPARED_NUMERATOR,
            "values[0].value.Effects[0].op.Sustained.definition.PreparedState.numerator",
        ),
        (
            "/values/0/value/Effects/0/op/Sustained/definition/PreparedState/program/owns_use",
            "values[0].value.Effects[0].op.Sustained.definition.PreparedState.program.owns_use",
        ),
        (
            "/values/0/value/Effects/0/op/Sustained/definition/PreparedState/program/id",
            "values[0].value.Effects[0].op.Sustained.definition.PreparedState.program.id",
        ),
        (
            "/values/1/value/AttackProgram/magazine/reload_ms",
            "values[1].value.AttackProgram.magazine.reload_ms",
        ),
    ] {
        let mut draft = complete.clone();
        *draft.pointer_mut(pointer).unwrap() =
            serde_json::json!({"Unresolved": "원문 수치·정책 미정"});
        let stored = serde_json::to_string(&draft).unwrap();
        let reopened = parse_yaml(&stored).unwrap();
        assert_eq!(reopened, draft);
        assert_eq!(reopened["definitions"], complete["definitions"]);
        assert_eq!(reopened["skills"], complete["skills"]);
        for error in [
            compile_json(&stored).unwrap_err(),
            compile_yaml(&stored).unwrap_err(),
        ] {
            assert_eq!(error.code, DiagnosticCode::Unresolved, "{error:?}");
            assert_eq!(error.source.definition, path);
            assert_eq!(error.reason, "원문 수치·정책 미정");
            assert_eq!(error.document, "scarecrow-headshot-fixture");
            assert_eq!(
                error.source_digest,
                format!("{:x}", Sha256::digest(stored.as_bytes()))
            );
            assert!(error.source.calls.is_empty());
        }
        let mut resolved = reopened;
        *resolved.pointer_mut(pointer).unwrap() = complete.pointer(pointer).unwrap().clone();
        assert_eq!(resolved, complete);
        let artifact = compile_json(&serde_json::to_string(&resolved).unwrap()).unwrap();
        assert_eq!(artifact.native_ron, expected.native_ron);
        Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap();
    }
}

#[test]
fn draft_leaf_literal_native_hole_and_yaml_bridge_preserve_request_identity() {
    let mut draft = parse_yaml(NATIVE_DRAFT_SOURCE).unwrap();
    let mut effects = draft["values"][0]["value"].clone();
    effects["Effects"][0]["op"]["Sustained"]["definition"]["PreparedState"]["numerator"] =
        serde_json::json!({"Unresolved": "배율 미정"});
    draft["skills"][0]["body"]["Call"]["arguments"][0]["value"] =
        serde_json::json!({"Literal": effects});
    let source = serde_json::to_string(&draft).unwrap();
    let error = compile_json(&source).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Unresolved, "{error:?}");
    assert_eq!(error.source.definition, "skills[0].body.Call.arguments[0].value.Literal.Effects[0].op.Sustained.definition.PreparedState.numerator");

    // The readable YAML source keeps all known sibling fields and the exact bytes
    // supplied by the author; no numeric fallback or previous artifact is emitted.
    let yaml = NATIVE_DRAFT_SOURCE.replace("numerator: 5", "numerator: { Unresolved: 배율 미정 }");
    let response = serde_json::to_value(
        compile_request(CompileRequest {
            source: yaml.clone(),
            epoch: 11,
            revision: 27,
            request_id: 38,
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(response["status"], "rejected");
    assert_eq!(response["epoch"], 11);
    assert_eq!(response["revision"], 27);
    assert_eq!(response["request_id"], 38);
    assert!(response.get("artifact").is_none());
    let error = compile_yaml(&yaml).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Unresolved);
    assert_eq!(
        error.source.definition,
        "values[0].value.Effects[0].op.Sustained.definition.PreparedState.numerator"
    );
    assert_eq!(
        error.source_digest,
        format!("{:x}", Sha256::digest(yaml.as_bytes()))
    );
    assert_eq!(
        parse_yaml(&yaml).unwrap()["values"][0]["value"]["Effects"][0]["op"]["Sustained"]
            ["definition"]["PreparedState"]["denominator"],
        2
    );
}

#[test]
fn draft_leaf_malformed_markers_and_prior_type_errors_remain_parse_errors() {
    let complete = parse_yaml(NATIVE_DRAFT_SOURCE).unwrap();
    for marker in [
        serde_json::json!({"Unresolved": ""}),
        serde_json::json!({"Unresolved": " \n\t"}),
        serde_json::json!({"Unresolved": 5}),
        serde_json::json!({"Unresolved": "pending", "fallback": 5}),
        serde_json::json!({"unresolved": "pending"}),
        serde_json::json!("Unresolved"),
    ] {
        let mut draft = complete.clone();
        *draft.pointer_mut(PREPARED_NUMERATOR).unwrap() = marker;
        let error = compile_json(&serde_json::to_string(&draft).unwrap()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Parse, "{error:?}");
    }
    let mut draft = complete;
    *draft.pointer_mut(PREPARED_NUMERATOR).unwrap() = serde_json::json!({"Unresolved": "pending"});
    draft["id"] = serde_json::json!(12);
    let error = compile_json(&serde_json::to_string(&draft).unwrap()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Parse);
    assert_eq!(error.source.definition, "id");
}

#[test]
fn draft_leaf_lookup_does_not_reclassify_duplicate_or_malformed_json() {
    let mut draft = parse_yaml(NATIVE_DRAFT_SOURCE).unwrap();
    *draft.pointer_mut(PREPARED_NUMERATOR).unwrap() = serde_json::json!({"Unresolved": "pending"});
    let source = serde_json::to_string(&draft).unwrap();
    let marker = r#""numerator":{"Unresolved":"pending"}"#;
    assert!(source.contains(marker));
    for invalid in [
        source.replace(marker, &format!("{marker},\"numerator\":5")),
        source.replace(
            marker,
            &format!("{marker},\"numerator\":{{\"Unresolved\":\"second\"}}"),
        ),
        source.replace(
            r#"{"Unresolved":"pending"}"#,
            r#"{"Unresolved":"pending","\u0055nresolved":"other"}"#,
        ),
        format!("{source} {{}}"),
        format!("{source} trailing"),
        source[..source.len() - 1].to_owned(),
    ] {
        let error = compile_json(&invalid).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Parse, "{error:?}");
    }
    let duplicate_yaml = NATIVE_DRAFT_SOURCE.replace(
        "numerator: 5",
        "numerator: { Unresolved: pending, Unresolved: other }",
    );
    assert_eq!(
        compile_yaml(&duplicate_yaml).unwrap_err().code,
        DiagnosticCode::Parse
    );
}

#[test]
fn draft_leaf_whole_native_expression_and_enum_markers_report_the_containing_field() {
    let complete = parse_yaml(NATIVE_DRAFT_SOURCE).unwrap();
    for (pointer, expected) in [
        ("/values/0/value/Effects/0/op/Sustained/definition/PreparedState/program/handlers/0/body/0/GrantOwnerResonance/amount",
         "values[0].value.Effects[0].op.Sustained.definition.PreparedState.program.handlers[0].body[0].GrantOwnerResonance.amount"),
        ("/skills/0/cast_targeting/Explicit/selection", "skills[0].cast_targeting.Explicit.selection"),
    ] {
        for marker in [
            serde_json::json!({"Unresolved":"expression pending"}),
            serde_json::json!({"Unresolved":""}),
            serde_json::json!({"Unresolved":3}),
            serde_json::json!({"Unresolved":"pending","fallback":3}),
        ] {
            let mut draft = complete.clone();
            *draft.pointer_mut(pointer).unwrap() = marker.clone();
            let source = serde_json::to_string(&draft).unwrap();
            let diagnostic = compile_json(&source).unwrap_err();
            if marker == serde_json::json!({"Unresolved":"expression pending"}) {
                assert_eq!(diagnostic.code, DiagnosticCode::Unresolved, "{diagnostic:?}");
                assert_eq!(diagnostic.source.definition, expected);
                assert_eq!(diagnostic.document, complete["id"].as_str().unwrap());
                assert_eq!(diagnostic.source_digest, format!("{:x}", Sha256::digest(source.as_bytes())));
                // A duplicate even at the reserved marker key remains Parse.
                let duplicate = source.replace(r#""Unresolved":"expression pending""#,
                    r#""Unresolved":"expression pending","\u0055nresolved":"other""#);
                assert_eq!(compile_json(&duplicate).unwrap_err().code, DiagnosticCode::Parse);
            } else {
                assert_eq!(diagnostic.code, DiagnosticCode::Parse, "{diagnostic:?}");
            }
        }
    }
}
