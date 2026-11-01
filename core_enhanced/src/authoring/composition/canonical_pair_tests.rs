use super::*;
use serde_json::{json, Value as Json};

#[test]
fn paired_area_append_preserves_installation_lifetime_and_native_validation() {
    let native = crate::authoring::node_graph::compile(
        &crate::authoring::node_graph::tests::paired_areas_fixture().to_string(),
        "fragment_yin_yang_cycle".into(),
    )
    .unwrap()
    .load_native()
    .unwrap();
    let document = super::owned_action_tests::module(&native);
    let artifact = compile_json(&document.to_string()).unwrap();
    let loaded = Artifact::from_ron(&artifact.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let (old, new) = (&native.skills[0], &loaded.skills[0]);
    assert_eq!(
        serde_json::to_value(&new.steps[0].effects).unwrap(),
        serde_json::to_value(&old.steps[0].effects).unwrap()
    );
    assert_eq!(new.cast_targeting, old.cast_targeting);
    assert_eq!(new.resonance_admission, old.resonance_admission);
    assert_eq!(
        new.use_lifetime,
        crate::game::ability::SkillUseLifetime::Action
    );
    for fault in ["admission", "preparation", "completion", "selection"] {
        let mut invalid = document.clone();
        match fault {
            "admission" => invalid["skills"][0]["admission"]["capacity"] = json!(99),
            "preparation" => invalid["skills"][0]["prepare_ms"] = json!(1),
            "selection" => {
                invalid["skills"][0]["cast_targeting"]["Explicit"]["selection"] =
                    json!("player_tile")
            }
            _ => invalid["skills"][0]["body"]["Action"]["wait"] = json!("EffectsSettled"),
        }
        let error = compile_json(&invalid.to_string()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Native, "{fault}: {error:?}");
    }
    let mut invalid = document.clone();
    let mut effect = document["skills"][0]["body"]["Action"]["effects"][0]["Append"]["effects"]
        ["Literal"]
        .clone();
    effect["Effects"][0]["op"]["PairedAreas"]["program"]["duration_ms"] = json!(0);
    invalid["values"] = json!([{ "name": "unused-invalid-pair", "value": effect }]);
    let error = compile_json(&invalid.to_string()).unwrap_err();
    assert_eq!(error.code, DiagnosticCode::Native);
    assert_eq!(
        error.source.definition,
        "values[0].value.Effects[0].op.PairedAreas.program"
    );
    if let Some(root) = std::env::var_os("PAIRED_SOURCE_EXPORT_DIR") {
        let root = std::path::PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("paired.technical.json"))
            .unwrap()
            .write_all(serde_json::to_string_pretty(&document).unwrap().as_bytes())
            .unwrap();
    }
}

pub(crate) const ORIGINAL: &str =
    include_str!("../../../../game_resources/authoring/yin-yang/fragment.skill.yaml");
fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_pair.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved() -> Json {
    let mut d = parse_yaml(ORIGINAL).unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &inputs());
    d
}
pub(crate) fn catalog(yang: bool) -> crate::game::data::skill_fragment_data::SkillFragmentMetadata {
    let mut d: Json = serde_json::from_str(if yang {
        include_str!("../../../../game_resources/authoring/yin-yang/yang.catalog.json")
    } else {
        include_str!("../../../../game_resources/authoring/yin-yang/yin.catalog.json")
    })
    .unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &inputs());
    serde_json::from_value(d).unwrap()
}
pub(crate) fn relics() -> crate::game::data::relic_data::RelicDatabase {
    let mut d: Json = serde_json::from_str(include_str!(
        "../../../../game_resources/authoring/relics/paired-modifiers.catalog.json"
    ))
    .unwrap();
    super::canonical_area_tests::replace_holes(&mut d, &inputs());
    let additional: crate::game::data::relic_data::RelicDatabase =
        serde_json::from_value(d).unwrap();
    let mut shared = super::canonical_hatred_tests::relics();
    shared.relics.extend(additional.relics);
    shared
}
pub(crate) fn program(d: &mut Json) -> &mut Json {
    &mut d["values"][0]["value"]["Effects"][0]["op"]["PairedAreas"]["program"]
}

#[test]
fn canonical_pair_source_preserves_both_fragment_identities_and_every_unknown() {
    let original = parse_yaml(ORIGINAL).unwrap();
    let document = resolved();
    let mut holes = vec![];
    super::canonical_area_tests::holes(&original, "", &mut holes);
    assert_eq!(holes.len(), 18);
    for (path, marker) in holes {
        let mut one = document.clone();
        *one.pointer_mut(&path).unwrap() = marker;
        let error = compile_json(&one.to_string()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Unresolved, "{path}: {error:?}");
        assert!(!error.source.definition.is_empty());
    }
    assert_eq!(
        compile_yaml(ORIGINAL).unwrap_err().code,
        DiagnosticCode::Unresolved
    );
    let artifact = compile_json(&document.to_string()).unwrap();
    let native = Artifact::from_ron(&artifact.to_ron().unwrap())
        .unwrap()
        .load_native()
        .unwrap();
    let old = crate::authoring::node_graph::compile(
        &crate::authoring::node_graph::tests::paired_areas_fixture().to_string(),
        "pair".into(),
    )
    .unwrap()
    .load_native()
    .unwrap();
    assert_eq!(
        serde_json::to_value(&native.skills[0].steps[0].effects).unwrap(),
        serde_json::to_value(&old.skills[0].steps[0].effects).unwrap()
    );
    assert_eq!(
        native.skills[0].cast_targeting,
        old.skills[0].cast_targeting
    );
    assert_eq!(
        native.skills[0].resonance_admission,
        old.skills[0].resonance_admission
    );
    assert_eq!(
        native.skills[0].use_lifetime,
        crate::game::ability::SkillUseLifetime::Action
    );
    let relics = relics();
    let mut d = document;
    let p = program(&mut d);
    assert_eq!(p["modifiers"].as_array().unwrap().len(), 6);
    for m in p["modifiers"].as_array().unwrap() {
        assert!(relics
            .get_by_id(m["relic_definition"].as_str().unwrap())
            .is_some());
        if m["relic_definition"] == "yin-yang-dragon" {
            assert_eq!(m["change"]["CoincidentBurst"]["max_hp_percent"], 25);
        }
        if m["relic_definition"] == "long-breath" {
            assert_eq!(
                m["change"]["Duration"]["percent"],
                json!({"numerator":25,"denominator":1})
            );
        }
    }
    let (yin, yang) = (catalog(false), catalog(true));
    assert_eq!(yin.id.as_str(), "fragment_yin_trace");
    assert_eq!(yang.id.as_str(), "fragment_yang_trace");
    assert_eq!(yin.uuid.to_string(), "730e8400-e29b-41d4-a716-446655440039");
    assert_eq!(
        yang.uuid.to_string(),
        "730e8400-e29b-41d4-a716-446655440040"
    );
    assert_ne!(yin.origin, yang.origin);
    for metadata in [yin, yang] {
        let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
            forms,
            ..
        } = metadata.effect
        else {
            panic!()
        };
        assert_eq!(forms.len(), 1);
        assert_eq!(forms["base"].skill_id, native.skills[0].id);
        assert_eq!(forms["base"].resonance_max, 100);
    }
}
