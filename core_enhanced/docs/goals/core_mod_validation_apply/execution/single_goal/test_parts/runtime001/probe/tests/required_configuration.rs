//! Read-only metadata comparisons. Host preflight/real save tests are separate.
use game_core::{game::data::content_pack::ContentContract,
    mod_validation::{ModValidationErrorKind, RequiredConfiguration}};
use serde_json::{json, Value};

fn document() -> Value {
    json!({"schema_version":1,"content_identity":{"contract":ContentContract::current(),"semantic_sha256":"ab".repeat(32)},
        "mods":[{"mod_id":"sample","mod_version":"1","packs":["sample"],"dependencies":[]}],
        "packs":[{"path":"official","pack_id":"official","pack_version":"1","dependencies":[]},
                 {"path":"sample","pack_id":"sample","pack_version":"1","dependencies":["official"]}],
        "selection":{"packs":["official","sample"],"winners":[]}})
}
fn parse(value: &Value) -> RequiredConfiguration { serde_json::from_value(value.clone()).unwrap() }

#[test]
fn required_metadata_matches_the_exact_configuration_and_is_not_a_pass_receipt() {
    let original = document();
    let approved_context_requirements = parse(&original);
    approved_context_requirements.validate().unwrap();
    approved_context_requirements.ensure_compatible(&parse(&original)).unwrap();
    let before = serde_json::to_vec(&approved_context_requirements).unwrap();
    let mut variants = vec![];
    let mut value = original.clone(); value["content_identity"]["semantic_sha256"] = json!("cd".repeat(32)); variants.push(value);
    let mut value = original.clone(); value["mods"][0]["mod_version"] = json!("2"); variants.push(value);
    let mut value = original.clone(); value["mods"][0]["mod_id"] = json!("foreign"); variants.push(value);
    let mut value = original.clone(); value["packs"][1]["pack_version"] = json!("2"); variants.push(value);
    let mut value = original.clone(); value["mods"] = json!([]); variants.push(value);
    let mut value = original.clone(); value["selection"]["packs"] = json!(["official"]); variants.push(value);
    let mut value = original.clone(); value["selection"]["winners"] = json!([{"target":{"kind":"Skill","id":"u11_damage_a"},"pack_id":"sample"}]); variants.push(value);
    for changed in variants {
        let error = approved_context_requirements.ensure_compatible(&parse(&changed)).unwrap_err();
        assert_eq!(error.kind, ModValidationErrorKind::ProtectedOriginal);
        assert_eq!(serde_json::to_vec(&approved_context_requirements).unwrap(), before);
        approved_context_requirements.ensure_compatible(&parse(&original)).unwrap();
    }
    for field in ["passed","approval","issuer_key","cache_receipt"] {
        let mut forged = original.clone(); forged[field] = json!(true);
        assert!(serde_json::from_value::<RequiredConfiguration>(forged).is_err());
    }
}

#[test]
fn malformed_required_metadata_cannot_be_treated_as_compatible() {
    let original = parse(&document());
    for hash in ["", "00", "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"] {
        let mut value = document(); value["content_identity"]["semantic_sha256"] = json!(hash);
        assert_eq!(original.ensure_compatible(&parse(&value)).unwrap_err().kind, ModValidationErrorKind::ProtectedOriginal);
    }
    let mut value = document(); value["schema_version"] = json!(2);
    assert_eq!(parse(&value).validate().unwrap_err().kind, ModValidationErrorKind::ProtectedOriginal);
    let mut value = document(); value["mods"][0]["packs"] = json!(["../escape"]);
    assert_eq!(parse(&value).validate().unwrap_err().kind, ModValidationErrorKind::ProtectedOriginal);
    // Equal list lengths do not establish exact pack membership.
    let mut value=document();value["selection"]["packs"]=json!(["official","official"]);
    assert_eq!(parse(&value).validate().unwrap_err().kind,ModValidationErrorKind::ProtectedOriginal);
}
