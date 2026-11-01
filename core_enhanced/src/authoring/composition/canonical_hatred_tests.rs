use super::canonical_area_tests::{holes, replace_holes};
use super::*;
use serde_json::{json, Value as Json};

pub(crate) fn original(bloom: bool) -> &'static str {
    if bloom {
        include_str!("../../../../game_resources/authoring/queen-of-hatred/bloom.skill.yaml")
    } else {
        include_str!("../../../../game_resources/authoring/queen-of-hatred/base.skill.yaml")
    }
}
pub(crate) const CATALOG: &str =
    include_str!("../../../../game_resources/authoring/queen-of-hatred/fragment.catalog.json");
fn inputs() -> Json {
    serde_json::from_str::<Json>(include_str!(
        "../../../tests/fixtures/skill_composition/canonical_hatred.values.json"
    ))
    .unwrap()["values"]
        .clone()
}
pub(crate) fn resolved(bloom: bool) -> Json {
    let mut d = parse_yaml(original(bloom)).unwrap();
    replace_holes(&mut d, &inputs());
    d
}
pub(crate) fn catalog() -> crate::game::data::skill_fragment_data::SkillFragmentMetadata {
    let mut d: Json = serde_json::from_str(CATALOG).unwrap();
    replace_holes(&mut d, &inputs());
    serde_json::from_value(d).unwrap()
}
pub(crate) fn relics() -> crate::game::data::relic_data::RelicDatabase {
    let mut d: Json = serde_json::from_str(include_str!(
        "../../../../game_resources/authoring/relics/counter-modifiers.catalog.json"
    ))
    .unwrap();
    replace_holes(&mut d, &inputs());
    let additional: crate::game::data::relic_data::RelicDatabase =
        serde_json::from_value(d).unwrap();
    let mut shared = super::canonical_knight_tests::relics();
    shared.relics.extend(additional.relics);
    shared
}
pub(crate) fn rename_root(d: &mut Json, root: &str) {
    let default_entry = d["values"][0]["value"]["Effects"][0]["op"]["CounterChannel"]["program"]
        ["default_entry"]
        .as_u64()
        .unwrap() as usize;
    for (i, s) in d["skills"].as_array_mut().unwrap().iter_mut().enumerate() {
        s["id"] = json!(if i == default_entry {
            root.to_string()
        } else {
            format!("{root}__entry_{i}")
        });
    }
    for v in d["values"].as_array_mut().unwrap() {
        v["value"]["Effects"][0]["op"]["CounterChannel"]["root"] = json!(root);
    }
}

#[test]
fn canonical_hatred_sources_preserve_complete_menu_and_reject_unresolved_values() {
    let metadata = catalog();
    assert!(
        serde_json::from_str::<crate::game::data::skill_fragment_data::SkillFragmentMetadata>(
            CATALOG
        )
        .is_err()
    );
    assert_eq!(metadata.id.as_str(), "fragment_queen_of_hatred_trace");
    assert_eq!(
        metadata.uuid.to_string(),
        "730e8400-e29b-41d4-a716-446655440008"
    );
    let crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill {
        forms, ..
    } = metadata.effect
    else {
        panic!()
    };
    let relics = relics();
    for bloom in [false, true] {
        let original = parse_yaml(original(bloom)).unwrap();
        let d = resolved(bloom);
        let mut pending = vec![];
        holes(&original, "", &mut pending);
        assert_eq!(pending.len(), if bloom { 146 } else { 104 });
        for (path, marker) in pending {
            let mut one = d.clone();
            *one.pointer_mut(&path).unwrap() = marker;
            let error = compile_json(&one.to_string()).unwrap_err();
            assert_eq!(error.code, DiagnosticCode::Unresolved, "{path}: {error:?}");
            assert!(!error.source.definition.is_empty());
        }
        let artifact = compile_json(&d.to_string()).unwrap();
        let native = Artifact::from_ron(&artifact.to_ron().unwrap())
            .unwrap()
            .load_native()
            .unwrap();
        let form = if bloom { "bloom" } else { "base" };
        assert!(native.skills.iter().any(|s| s.id == forms[form].skill_id));
        assert_eq!(forms[form].resonance_max, 100);
        assert_eq!(native.skills.len(), 3);
        let mut comparable = d.clone();
        rename_root(&mut comparable, "r11_aura");
        let new = compile_json(&comparable.to_string())
            .unwrap()
            .load_native()
            .unwrap();
        let old = crate::authoring::node_graph::compile(
            &crate::authoring::node_graph::tests::counter_channel_fixture(bloom).to_string(),
            "r11_aura".into(),
        )
        .unwrap()
        .load_native()
        .unwrap();
        for (new, old) in new.skills.iter().zip(&old.skills) {
            assert_eq!(
                serde_json::to_value(&new.steps[0].effects).unwrap(),
                serde_json::to_value(&old.steps[0].effects).unwrap()
            );
            assert_eq!(new.cast_targeting, old.cast_targeting);
            assert_eq!(new.resonance_admission, old.resonance_admission);
            assert!(new
                .use_lifetime
                .owns_effect(&new.steps[0].id, &new.steps[0].effects[0].id));
        }
        let program = &d["values"][0]["value"]["Effects"][0]["op"]["CounterChannel"]["program"];
        assert_eq!(program["capacity"], 6);
        assert_eq!(program["interval_ms"], 15000);
        assert_eq!(
            program["entries"][if bloom { 1 } else { 0 }]["completion_award"],
            json!({"base":1,"damage_and_healing_bonus":1})
        );
        assert_eq!(
            program["entries"][if bloom { 0 } else { 1 }]["minimum"],
            if bloom { 0 } else { 3 }
        );
        assert_eq!(program["entries"][2]["spend"], "EndAll");
        for m in program["modifiers"].as_array().unwrap() {
            assert!(relics
                .get_by_id(m["relic_definition"].as_str().unwrap())
                .is_some());
        }
        // A menu declaration is one program: changing one copy must be rejected.
        let mut mismatch = d.clone();
        mismatch["values"][1]["value"]["Effects"][0]["op"]["CounterChannel"]["program"]
            ["interval_ms"] = json!(14999);
        assert!(compile_json(&mismatch.to_string()).is_err());
    }
}

#[test]
fn canonical_counter_menu_ids_preserve_namespace_and_reject_obsolete_children() {
    for root in ["independent-love", "modx::love"] {
        let mut source = resolved(false);
        rename_root(&mut source, root);
        let native = compile_json(&source.to_string())
            .unwrap()
            .load_native()
            .unwrap();
        for (i, skill) in native.skills.iter().enumerate() {
            let expected = if i == 0 {
                root.to_owned()
            } else {
                format!("{root}__entry_{i}")
            };
            assert_eq!(skill.id.as_str(), expected);
            let (namespace, _) =
                crate::game::data::content_pack::canonical_key_parts(skill.id.as_str()).unwrap();
            assert_eq!(
                namespace,
                if root.starts_with("modx::") {
                    Some("modx")
                } else {
                    None
                }
            );
        }
        // Prior generated IDs never passed normal pack namespace resolution.
        // They are rejected explicitly; no resolver alias changes content identity.
        source["skills"][1]["id"] = json!(format!("{root}::entry:1"));
        let error = compile_json(&source.to_string()).unwrap_err();
        assert_eq!(error.code, DiagnosticCode::Native);
    }
}
