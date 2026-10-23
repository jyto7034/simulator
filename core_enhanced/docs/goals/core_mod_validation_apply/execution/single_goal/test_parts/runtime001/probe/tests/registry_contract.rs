//! Static registry admission is a necessary input gate, not measured approval.
use game_core::mod_validation::{ModRegistry, ModValidationErrorKind};
use serde_json::{json, Value};

fn base() -> Value {
    json!({"schema_version":1,"mods":[
        {"mod_id":"base","mod_version":"1","packs":["official"],"dependencies":[]},
        {"mod_id":"leaf","mod_version":"1","packs":["leaf/skills","leaf/items"],"dependencies":["base"]},
        {"mod_id":"left","mod_version":"1","packs":["left"],"dependencies":["leaf"]},
        {"mod_id":"right","mod_version":"1","packs":["right"],"dependencies":["leaf"]},
        {"mod_id":"top","mod_version":"1","packs":["top"],"dependencies":["left","right"]}
    ],"selection":{"mods":["top"],"winners":[]}})
}
fn load(value: &Value) -> Result<ModRegistry, game_core::mod_validation::ModValidationError> {
    ModRegistry::from_json_slice(&serde_json::to_vec(value).unwrap())
}
fn invalid(value: &Value, label: &str) {
    let error = load(value).expect_err(label);
    assert_eq!(error.kind, ModValidationErrorKind::InvalidInput, "{label}: {error}");
    assert!(!error.field_path.is_empty() && error.field_path.len()<=4096, "{label}: bounded location");
    assert!(!error.reason.is_empty() && error.reason.len() <= 4096, "{label}: bounded cause");
}

#[test]
fn multipack_diamond_is_accepted_without_synthesizing_approval() {
    let original = base();
    let parsed = load(&original).unwrap();
    assert_eq!(parsed.mods[1].packs.len(), 2);
    assert_eq!(parsed.selection.mods, ["top"]);
    assert_eq!(serde_json::to_value(&parsed).unwrap(), original);
    assert_eq!(load(&serde_json::to_value(&parsed).unwrap()).unwrap(), parsed);
    // A trusted base is separately supplied to Candidate. Empty user selection
    // must not be confused with the authority to choose that base.
    let empty = json!({"schema_version":1,"mods":[],"selection":{"mods":[],"winners":[]}});
    assert!(load(&empty).is_ok());
}

#[test]
fn duplicate_ownership_missing_dependency_and_cycles_refuse_locally() {
    let original = base();
    let accepted = load(&original).unwrap();
    let mut cases = vec![];
    let mut value = original.clone();
    let duplicate_mod = value["mods"][1].clone();
    value["mods"].as_array_mut().unwrap().push(duplicate_mod);
    cases.push(("duplicate mod", value));
    let mut value = original.clone(); value["mods"][2]["packs"] = json!(["leaf/skills"]);
    cases.push(("duplicate pack owner", value));
    let mut value = original.clone(); value["mods"][1]["packs"] = json!(["leaf/skills","leaf/skills"]);
    cases.push(("same member twice", value));
    let mut value = original.clone(); value["mods"][1]["dependencies"] = json!(["base","base"]);
    cases.push(("duplicate dependency", value));
    let mut value = original.clone(); value["mods"][1]["dependencies"] = json!(["absent"]);
    cases.push(("missing dependency", value));
    let mut value = original.clone(); value["mods"][1]["dependencies"] = json!(["leaf"]);
    cases.push(("self cycle", value));
    let mut value = original.clone(); value["mods"][1]["dependencies"] = json!(["top"]);
    cases.push(("transitive cycle", value));
    let mut value = original.clone(); value["selection"]["mods"] = json!(["top","top"]);
    cases.push(("duplicate selected id", value));
    let mut value = original.clone(); value["selection"]["mods"] = json!(["missing"]);
    cases.push(("unknown selected id", value));
    for (label, value) in cases {
        invalid(&value, label);
        assert_eq!(load(&original).unwrap(), accepted, "{label}: next independent admission changed");
    }
}

#[test]
fn registry_has_no_mod_provided_trusted_profile_or_pass_field() {
    for field in ["passed", "approved", "issuer", "profile", "engine_identity", "required_coverage"] {
        let mut value = base(); value[field] = json!("untrusted");
        invalid(&value, field);
        let mut value = base(); value["mods"][1][field] = json!(true);
        invalid(&value, field);
        let mut value = base(); value["selection"][field] = json!({});
        invalid(&value, field);
    }
    let mut value = base(); value["schema_version"] = json!(2);
    invalid(&value, "unsupported schema");
    for field in ["schema_version", "mods", "selection"] {
        let mut value = base(); value.as_object_mut().unwrap().remove(field);
        invalid(&value, "missing required field");
    }
}

#[test]
fn registry_paths_are_relative_unique_authored_members() {
    for path in ["", ".", "..", "../escape", "/absolute", "a/../b", "a/./b", "a//b", "a\\b", "C:drive"] {
        let mut value = base(); value["mods"][1]["packs"] = json!([path]);
        invalid(&value, path);
    }
    let mut value = base(); value["mods"][1]["packs"] = json!([]);
    invalid(&value, "mod with no pack");
    assert!(load(&base()).is_ok(), "normal nested relative members remain valid");
}

#[test]
fn explicit_writer_target_must_be_unique_before_selection_is_normalized() {
    let writer = |pack: &str| json!({"target":{"kind":"Skill","id":"u11_damage_a"},"pack_id":pack});
    for winners in [vec![writer("left"),writer("right")], vec![writer("left"),writer("left")]] {
        let mut value = base(); value["selection"]["winners"] = json!(winners);
        invalid(&value,"duplicate explicit winner target cannot silently become last-writer-wins");
    }
    let mut value = base();
    value["selection"]["winners"] = json!([writer("left"),
        {"target":{"kind":"Equipment","id":"u11_damage_a"},"pack_id":"right"}]);
    assert!(load(&value).is_ok(),"definition kind is part of a writer target key");
}

#[test]
fn malformed_authored_identifiers_cannot_expand_failure_locations_without_bound() {
    for id in ["x".repeat(32_768),"한".repeat(10_000)] {
        let mut value=base();value["mods"][1]["mod_id"]=json!(id);
        invalid(&value,"oversized authored identifier has bounded typed failure");
    }
    for version in [" "," 1","1 ","v\n1"] {
        let mut value=base();value["mods"][1]["mod_version"]=json!(version);
        invalid(&value,"noncanonical version");
    }
}

#[test]
fn raw_duplicate_json_fields_are_rejected_before_any_normalization() {
    for raw in [
        r#"{"schema_version":1,"schema_version":1,"mods":[],"selection":{"mods":[],"winners":[]}}"#,
        r#"{"schema_version":1,"mods":[],"selection":{"mods":[],"mods":[],"winners":[]}}"#,
        r#"{"schema_version":1,"mods":[{"mod_id":"a","mod_id":"b","mod_version":"1","packs":["p"],"dependencies":[]}],"selection":{"mods":["b"],"winners":[]}}"#,
    ] {
        let error=ModRegistry::from_json_slice(raw.as_bytes()).unwrap_err();
        assert_eq!(error.kind,ModValidationErrorKind::InvalidInput);
        assert!(error.field_path.len()<=4096 && error.reason.len()<=4096);
    }
}

#[test]
fn registry_current_finite_byte_and_mod_count_boundaries_are_real_inputs() {
    // Values are the current reviewed public ingress contract, not Lab criteria.
    const BYTES: usize = 1024 * 1024;
    let value = base();
    let mut exact = serde_json::to_vec(&value).unwrap();
    exact.resize(BYTES, b' ');
    assert!(ModRegistry::from_json_slice(&exact).is_ok());
    exact.push(b' ');
    assert_eq!(ModRegistry::from_json_slice(&exact).unwrap_err().kind, ModValidationErrorKind::InvalidInput);
    let mods: Vec<_> = (0..128).map(|i| json!({"mod_id":format!("m{i}"),"mod_version":"1",
        "packs":[format!("p{i}")],"dependencies":if i==0 {vec![]} else {vec![format!("m{}",i-1)]}})).collect();
    let mut exact = json!({"schema_version":1,"mods":mods,"selection":{"mods":["m127"],"winners":[]}});
    assert!(load(&exact).is_ok(), "maximum finite chain should admit");
    exact["mods"].as_array_mut().unwrap().push(json!({"mod_id":"m128","mod_version":"1","packs":["p128"],"dependencies":["m127"]}));
    invalid(&exact, "one over mod count");
}
