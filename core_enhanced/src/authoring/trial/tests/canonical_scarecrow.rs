use super::*;
use crate::authoring::composition::{compile_yaml, parse_yaml, DiagnosticCode};
use crate::game::data::skill_fragment_data::SkillFragmentMetadata;

pub(super) const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../game_resources/authoring/scarecrow/fragment.skill.yaml"
));
pub(super) const CATALOG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../game_resources/authoring/scarecrow/fragment.catalog.json"
));
const PREPARED: &str = "/values/0/value/Effects/0/op/Sustained/definition/PreparedState";

fn replace_unresolved(document: &mut Value, path: &str, value: Value) {
    let original = document
        .pointer_mut(path)
        .unwrap_or_else(|| panic!("missing {path}"));
    assert_eq!(original.as_object().unwrap().len(), 1, "{path}");
    assert!(
        original["Unresolved"]
            .as_str()
            .is_some_and(|s| !s.trim().is_empty()),
        "{path}"
    );
    *original = value;
}

/// Explicit acceptance-test values, never shipping defaults. All known source
/// structure and stable skill/fragment identities come from the originals.
pub(super) fn resolved_source(reward: u64) -> String {
    let mut document = parse_yaml(ORIGINAL).unwrap();
    for (path, value) in [
        (format!("{PREPARED}/numerator"), json!(100)),
        (format!("{PREPARED}/denominator"), json!(2)),
        (
            format!("{PREPARED}/program/handlers/0/body/0/GrantOwnerResonance/amount"),
            json!({"moment":"Live","root":{"Const":{"value":reward,"unit":"Magnitude"}}}),
        ),
        ("/skills/0/admission/capacity".into(), json!(5)),
        (
            "/skills/0/cast_targeting/Explicit/selection".into(),
            json!("self"),
        ),
        ("/skills/0/prepare_ms".into(), json!(0)),
    ] {
        replace_unresolved(&mut document, &path, value);
    }
    // JSON is a supported YAML subset; this is an explicit complete copy.
    serde_json::to_string_pretty(&document).unwrap()
}

pub(super) fn resolved_catalog() -> SkillFragmentMetadata {
    let mut document: Value = serde_json::from_str(CATALOG).unwrap();
    for (path, value) in [
        ("/rarity", json!("Common")),
        ("/equip_limit", json!("OwnedCopies")),
        ("/compatibility", json!({})),
        ("/effect/ActiveSkill/forms/base/resonance_max", json!(5)),
    ] {
        replace_unresolved(&mut document, path, value);
    }
    serde_json::from_value(document).unwrap()
}

#[test]
fn canonical_scarecrow_unresolved_original_and_every_hole_keep_known_meaning() {
    let original = parse_yaml(ORIGINAL).unwrap();
    assert!(original.get("equipment_attacks").is_none());
    assert_eq!(original["values"].as_array().unwrap().len(), 1);
    let state = original.pointer(PREPARED).unwrap();
    assert_eq!(
        state["program"]["handlers"][0]["event"],
        json!({"BasicAttackKill":{"primary_only":false,"once_per_action":true}})
    );
    assert_eq!(state["program"]["end_on_source_exit"], true);
    assert_eq!(state["program"]["owns_use"], false);
    assert_eq!(state["program"]["replacement"], "ReplaceSameSource");
    assert_eq!(original["skills"][0]["admission"]["requirement"], "Full");
    assert_eq!(original["skills"][0]["admission"]["cost"], "All");
    assert_eq!(
        compile_yaml(ORIGINAL).unwrap_err().code,
        DiagnosticCode::Unresolved
    );
    let resolved = parse_yaml(&resolved_source(2)).unwrap();
    for path in [
        format!("{PREPARED}/numerator"),
        format!("{PREPARED}/denominator"),
        format!("{PREPARED}/program/handlers/0/body/0/GrantOwnerResonance/amount"),
        "/skills/0/admission/capacity".into(),
        "/skills/0/cast_targeting/Explicit/selection".into(),
        "/skills/0/prepare_ms".into(),
    ] {
        let mut unresolved = resolved.clone();
        *unresolved.pointer_mut(&path).unwrap() = original.pointer(&path).unwrap().clone();
        let error = compile_yaml(&serde_json::to_string(&unresolved).unwrap()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Unresolved, "{path}: {error:?}");
        assert!(!error.source.definition.is_empty());
    }
    let artifact = compile_yaml(&resolved_source(2)).unwrap();
    assert!(artifact.equipment_attacks().is_empty());
    assert_eq!(
        artifact.load_native().unwrap().skills[0].id.as_str(),
        "fragment_scarecrow_wisdom_harvest"
    );
    assert!(artifact
        .validated_native_ron()
        .unwrap()
        .contains("GrantOwnerResonance"));
    assert!(!artifact
        .validated_native_ron()
        .unwrap()
        .contains("ModifyStats"));
    assert!(!artifact
        .validated_native_ron()
        .unwrap()
        .contains("ApplyOwnerState"));
    assert!(!artifact
        .validated_native_ron()
        .unwrap()
        .contains("magazine"));
    assert!(serde_json::from_str::<SkillFragmentMetadata>(CATALOG).is_err());
    let metadata = resolved_catalog();
    assert_eq!(metadata.id.as_str(), "fragment_scarecrow_trace");
    assert_eq!(
        metadata.uuid.to_string(),
        "730e8400-e29b-41d4-a716-446655440047"
    );
    // Resolving a copy never changes either canonical draft.
    assert_eq!(parse_yaml(ORIGINAL).unwrap(), original);
}
